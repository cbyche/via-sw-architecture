//! [`ContextPack`] — a typed layer over the blocks that already exist.
//!
//! `docs/architecture.md` §5: *"ContextPack is a thin typed layer, not a
//! rewrite … Rewriting prompt assembly would be re-deriving a working
//! implementation."*
//!
//! So nothing in this module renders a block. `via-conversation` renders
//! `<user_preferences>`, `<user_memory>`, `<runtime_context>` and
//! `<recent_conversation>`; `via-voice` renders `PROMPT.md`, the
//! `<assistant_profile authority="persona_only">` wrapper and `<input_parts>`.
//! This module receives those strings, attaches the four facts none of them
//! carries — **where it came from**, **what authority it has**, **when it goes
//! stale**, **what it costs** — and joins them back in the catalogued order.
//!
//! The proof that the layer is thin is an equality, not a paragraph:
//! `via-voice`'s `tests/context_pack.rs` asserts
//! `pack.instructions() == via_voice::assemble_frontend_instructions(…)` for the
//! same inputs. It lives there because `via-voice` is the only crate that may
//! see both (`docs/architecture.md` §9: Layer 1 may depend on Layer 2, never the
//! reverse).
//!
//! # The three dimensions, and why they are three
//!
//! | | Answers | Set by |
//! | --- | --- | --- |
//! | [`Provenance`] | who produced the *bytes* | the caller, or [`Provenance::classify`] |
//! | [`Trust`] | what instruction authority they carry | the caller, floored by provenance |
//! | [`Tier`] | how often they change | the caller |
//!
//! Collapsing any two of them is the bug `docs/architecture.md` §5 names: a
//! first-party tool returning third-party bytes is trusted only if provenance
//! and tool identity are the same field. They are not, here.
//!
//! # The two invariants, and their direction
//!
//! 1. **Provenance floors trust.** [`Provenance::ThirdParty`] forces
//!    [`Trust::Untrusted`], whatever the caller asked for. The clamp only ever
//!    moves downward; nothing raises trust.
//! 2. **Untrusted implies fenced.** A section constructed at
//!    [`Trust::Untrusted`] is fenced by [`crate::fence::wrap`] *inside its own
//!    constructor*, so there is no order of calls in which a third-party body
//!    reaches [`ContextPack::render`] unfenced. Forgetting the fence is not
//!    representable.
//!
//! # Order
//!
//! Sections render grouped by tier — [`Tier::Static`], then
//! [`Tier::SemiStatic`], then [`Tier::Dynamic`] — and in insertion order within
//! a tier, joined with a blank line. That reproduces `via-voice`'s own
//! assembly for the static tier exactly, because its four blocks are pushed in
//! the catalogued order and it joins them the same way.

use indexmap::IndexMap;
use serde::Serialize;
use via_conversation::text::trim;
use via_i18n::Locale;

use crate::error::ContextError;
use crate::fence::{self, FenceSource};

/// The code-point cap on a section id.
///
/// Ids are short machine names; the cap exists so a host-supplied one cannot
/// carry an essay into a log record.
pub const MAX_SECTION_ID_CHARS: usize = 64;

/// How often a section's bytes change — ARGO's `PromptTier`, by its own
/// meaning.
///
/// From `tinicore/src/prompt/builder.rs:464-477`. The tier is a *cache*
/// statement: a provider may anchor a prompt-cache breakpoint at the end of the
/// stable prefix, and per-turn content placed before the history makes the
/// history uncacheable for every later turn.
///
/// The variants keep ARGO's names and gain VIA's occupants:
///
/// | Tier | ARGO | VIA |
/// | --- | --- | --- |
/// | [`Static`](Self::Static) | persona, safety, locale policy | `PROMPT.md`, `<assistant_profile>`, `<user_preferences>`, `<user_memory>`, `<runtime_context>` |
/// | [`SemiStatic`](Self::SemiStatic) | the tool-definitions section | the ordered-deixis view — invalidated by a new surface generation, not by a new turn |
/// | [`Dynamic`](Self::Dynamic) | datetime, retrieved memory | `<recent_conversation>`, `<input_parts>`, fenced tool evidence |
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// Stable for the session. VIA sets a session's instructions once, so this
    /// tier is exactly what `via-voice` puts in them.
    Static,
    /// Stable until a specific thing changes, and that thing is not the turn.
    SemiStatic,
    /// Per turn.
    Dynamic,
}

impl Tier {
    /// The three tiers, in render order.
    pub const ALL: [Self; 3] = [Self::Static, Self::SemiStatic, Self::Dynamic];

    /// A stable machine token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::SemiStatic => "semi_static",
            Self::Dynamic => "dynamic",
        }
    }
}

/// Whether a *tool* is one VIA ships.
///
/// Half of the classification `docs/architecture.md` §5 says is missing. On its
/// own it decides nothing about content — see [`Provenance::classify`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolTrust {
    /// A tool VIA ships and whose output VIA composes: `get_current_time`,
    /// `get_agent_task_status`, `notes`, `memory`.
    FirstParty,
    /// A tool a backend, an MCP server or a plugin registered.
    ThirdParty,
}

/// Whether a tool *authored* its bytes or *fetched* them.
///
/// The other half. This is the dimension that did not exist before, and the one
/// that closes the hole: `Own` means the tool composed the answer from state
/// VIA owns, `External` means the bytes came from somewhere VIA does not
/// control and merely travelled through the tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSource {
    /// Composed by the tool from VIA-owned state.
    Own,
    /// Read, fetched or scraped from outside VIA.
    External,
}

/// Who produced a section's bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// VIA's own packaged documents — `PROMPT.md`, `ASSISTANT.md`.
    System,
    /// The user, directly: their utterance, and the documents they edit.
    User,
    /// The host client process: `<runtime_context>`, `<input_parts>` metadata,
    /// the surface it registers.
    ///
    /// Host-supplied *fields* are normalized rather than fenced, because their
    /// shape is fixed and catalogued —
    /// [`normalize_client_context`](via_conversation::normalize_client_context)
    /// strips NUL and collapses CR/LF for exactly this reason. Host-supplied
    /// *free-form bytes*, such as a screen label, are not `Host`; they are
    /// [`ThirdParty`](Self::ThirdParty).
    Host,
    /// A first-party VIA tool's own composed output.
    Tool,
    /// Bytes from outside VIA, however they arrived.
    ThirdParty,
}

impl Provenance {
    /// Classify a tool result by **both** dimensions.
    ///
    /// This is the fix `docs/architecture.md` §5 asks for, as a table:
    ///
    /// | tool | content | provenance |
    /// | --- | --- | --- |
    /// | [`ToolTrust::FirstParty`] | [`ContentSource::Own`] | [`Provenance::Tool`] |
    /// | [`ToolTrust::FirstParty`] | [`ContentSource::External`] | [`Provenance::ThirdParty`] |
    /// | [`ToolTrust::ThirdParty`] | either | [`Provenance::ThirdParty`] |
    ///
    /// The second row is the one that did not exist. A first-party screen
    /// reader, a first-party fetch tool and a first-party file reader all land
    /// there: the fence follows the bytes, not the caller.
    #[must_use]
    pub const fn classify(tool: ToolTrust, content: ContentSource) -> Self {
        match (tool, content) {
            (ToolTrust::FirstParty, ContentSource::Own) => Self::Tool,
            _ => Self::ThirdParty,
        }
    }

    /// The highest [`Trust`] this provenance may carry.
    ///
    /// Only [`ThirdParty`](Self::ThirdParty) constrains anything; every other
    /// provenance leaves the caller's declared trust alone, because provenance
    /// says where bytes came from and not what they mean.
    #[must_use]
    pub const fn trust_ceiling(self) -> Trust {
        match self {
            Self::ThirdParty => Trust::Untrusted,
            _ => Trust::Policy,
        }
    }

    /// A stable machine token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Host => "host",
            Self::Tool => "tool",
            Self::ThirdParty => "third_party",
        }
    }

    /// Every provenance, in declaration order.
    pub const ALL: [Self; 5] = [
        Self::System,
        Self::User,
        Self::Host,
        Self::Tool,
        Self::ThirdParty,
    ];
}

/// What instruction authority a section carries.
///
/// The ladder is `PROMPT.md`'s own, reproduced rather than invented — the
/// catalogued *frontend instruction hierarchy* contract
/// (`docs/reference/contracts.json`): the user's current request outranks
/// `<user_preferences>`, which outranks `<assistant_profile>`, and
/// `<user_memory>` / `<recent_conversation>` / `<runtime_context>` /
/// `<input_parts>` carry no instruction authority at all.
///
/// `Ord` is that ladder and **not** the render order. Render order is the
/// catalogued block order, which `via-conversation` owns; the two deliberately
/// disagree (`<user_memory>` renders after `<assistant_profile>` while ranking
/// below it).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Trust {
    /// Evidence from outside. Fenced, and the fence says so. Never obeyed.
    Untrusted,
    /// Facts and transcript. Used for understanding, never followed.
    Data,
    /// The persona: default name, personality, register. Bounded by
    /// `authority="persona_only"`.
    Persona,
    /// User-authorized directive material — `<user_preferences>`. Followed, and
    /// it overrides the persona.
    Directive,
    /// Core policy. Outranks everything, including the user's preferences,
    /// because it is what makes the preferences safe to follow.
    Policy,
}

impl Trust {
    /// The five levels, lowest authority first.
    pub const ALL: [Self; 5] = [
        Self::Untrusted,
        Self::Data,
        Self::Persona,
        Self::Directive,
        Self::Policy,
    ];

    /// A stable machine token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Untrusted => "untrusted",
            Self::Data => "data",
            Self::Persona => "persona",
            Self::Directive => "directive",
            Self::Policy => "policy",
        }
    }
}

/// A section's stable name.
///
/// Validated at construction because it reaches diagnostics and log records and
/// a host may supply one: `[a-z][a-z0-9_]*`, at most
/// [`MAX_SECTION_ID_CHARS`] code points. A value carrying a newline could forge
/// a second log line, so it is refused rather than sanitized.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SectionId(String);

impl SectionId {
    /// `PROMPT.md` — the core policy prompt.
    pub const CORE_POLICY: &'static str = "core_policy";
    /// The `<assistant_profile authority="persona_only">` block.
    pub const ASSISTANT_PROFILE: &'static str = "assistant_profile";
    /// The `<user_preferences>` block.
    pub const USER_PREFERENCES: &'static str = "user_preferences";
    /// The `<user_memory>` block.
    pub const USER_MEMORY: &'static str = "user_memory";
    /// The `<runtime_context>` block.
    pub const RUNTIME_CONTEXT: &'static str = "runtime_context";
    /// The `<recent_conversation>` block.
    pub const RECENT_CONVERSATION: &'static str = "recent_conversation";
    /// The `<input_parts>` projection of this turn's attachments.
    pub const INPUT_PARTS: &'static str = "input_parts";
    /// The enumerated, stably-ordered view of on-screen objects.
    pub const ON_SCREEN: &'static str = "on_screen";

    /// Every id VIA itself uses, in render order.
    pub const KNOWN: [&'static str; 8] = [
        Self::CORE_POLICY,
        Self::ASSISTANT_PROFILE,
        Self::USER_PREFERENCES,
        Self::USER_MEMORY,
        Self::RUNTIME_CONTEXT,
        Self::RECENT_CONVERSATION,
        Self::INPUT_PARTS,
        Self::ON_SCREEN,
    ];

    /// [`ON_SCREEN`](Self::ON_SCREEN), infallibly.
    ///
    /// [`crate::deixis::DeixisView::section`] cannot return a `Result` for a
    /// name it wrote itself, and this crate carries no `expect()` outside its
    /// tests. `every_known_section_id_is_well_formed` asserts that this and
    /// [`new`](Self::new) agree, so the shortcut cannot drift into producing an
    /// id `new` would refuse.
    #[must_use]
    pub fn on_screen() -> Self {
        Self(Self::ON_SCREEN.to_owned())
    }

    /// Validate and wrap a section name.
    ///
    /// # Errors
    ///
    /// [`ContextError::SectionId`] when the value is empty, too long, or
    /// outside `[a-z][a-z0-9_]*`.
    pub fn new(value: &str) -> Result<Self, ContextError> {
        let refuse = |reason: &'static str| ContextError::SectionId {
            value: value.chars().take(MAX_SECTION_ID_CHARS).collect(),
            reason,
        };
        let mut chars = value.chars();
        let Some(first) = chars.next() else {
            return Err(refuse("it is empty"));
        };
        if !first.is_ascii_lowercase() {
            return Err(refuse("it does not start with a lowercase ASCII letter"));
        }
        if value.chars().count() > MAX_SECTION_ID_CHARS {
            return Err(refuse("it is longer than MAX_SECTION_ID_CHARS"));
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(refuse("it has a character outside [a-z0-9_]"));
        }
        Ok(Self(value.to_owned()))
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Estimate what `text` costs in tokens.
///
/// Deterministic and **an estimate**: it is for budgeting a pack, never for
/// billing or for a hard provider limit. ARGO's own budgeting heuristic is
/// `chars / 4` (`tinicore/src/workflow/schema.rs`), which is close for ASCII
/// and badly wrong for the locale half of VIA's catalog — `zh` is upstream's
/// own text and a Han character is roughly one token, not a quarter of one. So
/// the estimate splits: ASCII code points at four to the token, everything else
/// at one each.
///
/// Two properties hold and are asserted: it never decreases when text is
/// appended, and for the same code-point count a non-ASCII string never
/// estimates below an ASCII one.
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    let mut ascii = 0usize;
    let mut wide = 0usize;
    for c in text.chars() {
        if c.is_ascii() {
            ascii += 1;
        } else {
            wide += 1;
        }
    }
    ascii.div_ceil(4) + wide
}

/// One block of context, with the facts the block itself does not carry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ContextSection {
    /// The section's stable name.
    pub id: SectionId,
    /// Who produced the bytes.
    pub source: Provenance,
    /// What authority they carry, after the provenance floor.
    pub trust: Trust,
    /// How often they change.
    pub tier: Tier,
    /// [`estimate_tokens`] of [`body`](Self::body) — the *fenced* body, because
    /// that is what is actually sent.
    pub tokens: usize,
    /// The rendered block, fenced when [`trust`](Self::trust) is
    /// [`Trust::Untrusted`].
    pub body: String,
}

impl ContextSection {
    /// Build a section, applying both invariants.
    ///
    /// `body` is a block some other crate already rendered. It is trimmed;
    /// if `source` is [`Provenance::ThirdParty`] the declared `trust` is
    /// floored to [`Trust::Untrusted`], and an untrusted body is wrapped by
    /// [`crate::fence::wrap`] here — the only place a fence is applied, so it
    /// is the only place one could be forgotten, and it never is.
    ///
    /// `fence_source` is ignored for a trusted section.
    #[must_use]
    pub fn new(
        locale: Locale,
        id: SectionId,
        source: Provenance,
        trust: Trust,
        tier: Tier,
        fence_source: FenceSource,
        body: &str,
    ) -> Self {
        let trust = trust.min(source.trust_ceiling());
        let body = if trust == Trust::Untrusted {
            fence::wrap(locale, fence_source, body)
        } else {
            trim(body).to_owned()
        };
        Self {
            id,
            source,
            trust,
            tier,
            tokens: estimate_tokens(&body),
            body,
        }
    }

    /// Whether the body is empty after trimming or fencing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    /// Whether this section went through [`crate::fence::wrap`].
    #[must_use]
    pub fn is_fenced(&self) -> bool {
        self.trust == Trust::Untrusted && !self.body.is_empty()
    }
}

/// An ordered set of [`ContextSection`]s with a deterministic render.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ContextPack {
    sections: Vec<ContextSection>,
}

impl ContextPack {
    /// Start a pack for `locale`.
    #[must_use]
    pub fn builder(locale: Locale) -> ContextPackBuilder {
        ContextPackBuilder {
            locale,
            sections: IndexMap::new(),
            error: None,
        }
    }

    /// Every section, in insertion order.
    #[must_use]
    pub fn sections(&self) -> &[ContextSection] {
        &self.sections
    }

    /// One section by name.
    #[must_use]
    pub fn section(&self, id: &str) -> Option<&ContextSection> {
        self.sections
            .iter()
            .find(|section| section.id.as_str() == id)
    }

    /// How many sections.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// Whether the pack has no sections.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// The estimated token cost of everything in the pack.
    #[must_use]
    pub fn tokens(&self) -> usize {
        self.sections.iter().map(|section| section.tokens).sum()
    }

    /// The estimated token cost of one tier.
    #[must_use]
    pub fn tokens_in(&self, tier: Tier) -> usize {
        self.sections
            .iter()
            .filter(|section| section.tier == tier)
            .map(|section| section.tokens)
            .sum()
    }

    /// One tier's sections, joined with a blank line, in insertion order.
    #[must_use]
    pub fn render_tier(&self, tier: Tier) -> String {
        self.sections
            .iter()
            .filter(|section| section.tier == tier)
            .map(|section| section.body.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// The whole pack: tier by tier, insertion order within a tier, joined with
    /// a blank line.
    #[must_use]
    pub fn render(&self) -> String {
        Tier::ALL
            .into_iter()
            .map(|tier| self.render_tier(tier))
            .filter(|block| !block.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// The session instructions: [`Tier::Static`], nothing else.
    ///
    /// This is the string `via-voice`'s `assemble_frontend_instructions`
    /// produces for the same inputs, and `via-voice`'s own test suite asserts
    /// the equality. Everything below `Static` is per-turn or per-surface and a
    /// session's instructions are set once — a replay pinned into the
    /// instructions would still be describing the first turn an hour later.
    #[must_use]
    pub fn instructions(&self) -> String {
        self.render_tier(Tier::Static)
    }
}

/// Accumulates sections, and the first thing that went wrong.
///
/// Errors are held rather than returned per call so the chain stays readable;
/// [`build`](Self::build) is where a pack either exists or does not. The first
/// error wins and later pushes are ignored, so a diagnostic points at the cause
/// rather than at a consequence.
#[derive(Debug)]
pub struct ContextPackBuilder {
    locale: Locale,
    sections: IndexMap<String, ContextSection>,
    error: Option<ContextError>,
}

impl ContextPackBuilder {
    /// Add a section whose bytes VIA itself produced.
    ///
    /// An empty body adds nothing: a section with no content is not context,
    /// and dropping it here is what makes [`ContextPack::render`] a plain join
    /// — the same rule `via-voice`'s assembly applies when it filters empty
    /// blocks.
    #[must_use]
    pub fn push(self, id: &str, source: Provenance, trust: Trust, tier: Tier, body: &str) -> Self {
        self.push_section(id, source, trust, tier, FenceSource::Unknown, body)
    }

    /// Add a section whose bytes came from outside VIA.
    ///
    /// Trust is [`Trust::Untrusted`] by construction and the body is fenced;
    /// there is no argument that could make it otherwise.
    #[must_use]
    pub fn push_untrusted(self, id: &str, tier: Tier, from: FenceSource, body: &str) -> Self {
        self.push_section(
            id,
            Provenance::ThirdParty,
            Trust::Untrusted,
            tier,
            from,
            body,
        )
    }

    /// Add a tool result, classified by both dimensions.
    ///
    /// The one call site that closes `docs/architecture.md` §5's hole: pass what
    /// ran it *and* where its bytes came from, and the fence follows the bytes.
    #[must_use]
    pub fn push_tool_result(
        self,
        id: &str,
        tier: Tier,
        tool: ToolTrust,
        content: ContentSource,
        from: FenceSource,
        body: &str,
    ) -> Self {
        let source = Provenance::classify(tool, content);
        self.push_section(id, source, Trust::Data, tier, from, body)
    }

    fn push_section(
        mut self,
        id: &str,
        source: Provenance,
        trust: Trust,
        tier: Tier,
        from: FenceSource,
        body: &str,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        let id = match SectionId::new(id) {
            Ok(id) => id,
            Err(error) => {
                self.error = Some(error);
                return self;
            }
        };
        // A repeated id is refused, never merged and never replaced. The map
        // would happily overwrite, and that overwrite is an injection surface:
        // a forged `<user_preferences>` block arriving from third-party content
        // after the real one would silently become the only one the model sees.
        // Refusing is the only safe answer — the caller has a bug either way,
        // and a pack that cannot be assembled is better than one that lies.
        if self.sections.contains_key(id.as_str()) {
            self.error = Some(ContextError::DuplicateSection {
                id: id.as_str().to_owned(),
            });
            return self;
        }
        let section = ContextSection::new(self.locale, id.clone(), source, trust, tier, from, body);
        if !section.is_empty() {
            self.sections.insert(id.as_str().to_owned(), section);
        }
        self
    }

    /// Finish.
    ///
    /// # Errors
    ///
    /// The first [`ContextError`] any `push` hit: a malformed id, or a repeated
    /// one.
    pub fn build(self) -> Result<ContextPack, ContextError> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(ContextPack {
                sections: self.sections.into_values().collect(),
            }),
        }
    }
}
