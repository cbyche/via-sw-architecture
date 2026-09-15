//! Referents — a stable handle bound to a concrete object that survives a turn.
//!
//! `docs/architecture.md` §5 marks this **new**: *"nothing to port from either
//! upstream."* So the four questions it raises are answered here, in the order
//! a reviewer will ask them.
//!
//! # 1. What may be a referent
//!
//! Exactly five things, and the rule is *an identity VIA can act on*:
//!
//! | [`ReferentTarget`] | Bound by | "that one" means |
//! | --- | --- | --- |
//! | [`Screen`](ReferentTarget::Screen) | [`crate::deixis::DeixisView`] | a row, card or control the host enumerated |
//! | [`Input`](ReferentTarget::Input) | the turn's attachments | an `input_N` reference `via-conversation` already mints |
//! | [`Work`](ReferentTarget::Work) | the Work queue | a `work_id` — "cancel that one" |
//! | [`NoteList`](ReferentTarget::NoteList) | `via-conversation`'s named lists | a list the user just heard back |
//! | [`Directory`](ReferentTarget::Directory) | `<runtime_context>` | `client_working_directory` — "this directory", which `PROMPT.md` names by field |
//!
//! Free text is **not** a referent, and neither is a line of `MEMORY.md`. Both
//! are content; neither has an identity, so "delete that one" could not be
//! executed against either without first inventing what it denotes. `PROMPT.md`
//! already tells the model not to invent a referent; this registry is the half
//! of that rule that VIA can enforce.
//!
//! # 2. How long one lives
//!
//! [`DEFAULT_RETENTION_TURNS`] turns after the turn that bound it — the binding
//! turn and the one after it. That is short on purpose. "The second one" is
//! about the enumeration the user just heard; a referent that survives five
//! turns is a stale hit waiting to happen, and a stale hit is worse than a
//! miss because the miss asks and the hit acts.
//!
//! The registry is also capped at [`DEFAULT_CAPACITY`] bindings, evicting the
//! oldest first. Handles are **never reused** — [`ReferentRegistry::bind`]
//! draws from a monotonic counter — because a reused handle would let a stale
//! `ref_3` silently resolve to a different object, which is the exact failure
//! the retention window exists to prevent.
//!
//! # 3. How a stale one is detected
//!
//! Three detectors and a fail-closed fourth, all in
//! [`ReferentRegistry::staleness`]:
//!
//! | [`Staleness`] | Cause |
//! | --- | --- |
//! | [`Expired`](Staleness::Expired) | the turn ended, and then another one did |
//! | [`Moved`](Staleness::Moved) | the object is still on screen but its [`SurfaceFingerprint`] changed — it was renumbered, relabelled, or its actions changed |
//! | [`Gone`](Staleness::Gone) | the screen changed and the object is not in the new snapshot |
//! | [`SurfaceUnknown`](Staleness::SurfaceUnknown) | a screen referent with no current snapshot to vouch for it |
//!
//! The fourth is the fail-closed one. A session that has stopped receiving
//! snapshots does not get to keep acting on its last idea of the screen.
//!
//! # 4. What happens when resolution is ambiguous
//!
//! It reports candidates and stops. That is `via-conversation`'s rule for notes
//! (`frontend-notes.mjs:50-83`), for its reason — *"guessing is the one thing
//! the resolver never does, because a wrong guess on `remove` silently destroys
//! data"* — and the ladder here is the same shape:
//!
//! 1. an exact match, case-insensitively, over label and aliases;
//! 2. otherwise a **bidirectional** substring match, accepted only if unique;
//! 3. otherwise the candidates go back to the model, which asks the user.
//!
//! With `via-conversation`'s asymmetry kept too: when several match *exactly*
//! the substring pass is skipped, so two identically-labelled objects report
//! each other rather than one being picked arbitrarily.
//!
//! Two things are layered on top of that ladder, and both preserve "never
//! guess":
//!
//! - the ladder runs over **live** referents first. Only if nothing live
//!   matches does it run again over stale ones, and a unique stale match is
//!   reported as [`Resolution::Stale`] with its reason — so the model can say
//!   *"that has gone from the screen"* instead of *"I don't know what you
//!   mean"*.
//! - the candidate list is always **live** referents. Offering a dead one as a
//!   choice would invite the model to pick it.

use indexmap::IndexMap;
use serde::Serialize;
use via_conversation::notes::{MAX_ITEM_CHARS, MAX_RESULT_CANDIDATES};
use via_conversation::text::clean_bounded;

use crate::deixis::{MAX_SURFACE_OBJECTS, SurfaceObject, SurfaceSnapshot};
use crate::error::ContextError;

/// How many turns after its binding turn a referent stays resolvable.
///
/// Two: the turn that bound it and the next one. See the module documentation
/// for why it is not more.
pub const DEFAULT_RETENTION_TURNS: u64 = 2;

/// How many non-screen referents one turn is expected to contribute.
///
/// `via-voice` caps a turn's attachments at sixteen; this allows that many
/// again for Work items, note lists and the working directory.
pub const CONVERSATION_HEADROOM: usize = 32;

/// How many bindings the registry holds before evicting the oldest.
///
/// Two full screens plus [`CONVERSATION_HEADROOM`]. Sized rather than picked:
/// eviction is oldest-first and the conversation's referents are always the
/// older ones, so a registry that only held one screen would let a single
/// re-enumeration evict the `input_N` the user is in the middle of talking
/// about.
pub const DEFAULT_CAPACITY: usize = 2 * MAX_SURFACE_OBJECTS + CONVERSATION_HEADROOM;

/// The code-point cap on a label or an alias.
///
/// [`MAX_ITEM_CHARS`](via_conversation::notes::MAX_ITEM_CHARS) — the same bound
/// `via-conversation` puts on a spoken note item, because a referent label is
/// the same kind of thing: something a person says out loud once.
pub const MAX_LABEL_CHARS: usize = MAX_ITEM_CHARS;

/// How many aliases one binding may carry.
pub const MAX_ALIASES: usize = 8;

/// The prefix every handle has.
pub const HANDLE_PREFIX: &str = "ref_";

/// What a referent points at.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ReferentTarget {
    /// An object the host enumerated on screen.
    Screen {
        /// The host's own stable id for it.
        object_id: String,
    },
    /// An attachment, by the `input_N` reference `via-conversation` mints.
    Input {
        /// `input_1`, `input_2`, …
        reference: String,
    },
    /// A Work item, by `work_id`.
    Work {
        /// The `work_id`.
        work_id: String,
    },
    /// One of the user's named lists.
    NoteList {
        /// The list key.
        key: String,
    },
    /// A directory — in practice `<runtime_context>`'s
    /// `client_working_directory`.
    Directory {
        /// The path, as the client sent it.
        path: String,
    },
}

impl ReferentTarget {
    /// A stable machine token for the kind of thing this is.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Screen { .. } => "screen",
            Self::Input { .. } => "input",
            Self::Work { .. } => "work",
            Self::NoteList { .. } => "note_list",
            Self::Directory { .. } => "directory",
        }
    }

    /// The identity inside the target. Never empty in a bound referent.
    #[must_use]
    pub fn identity(&self) -> &str {
        match self {
            Self::Screen { object_id } => object_id,
            Self::Input { reference } => reference,
            Self::Work { work_id } => work_id,
            Self::NoteList { key } => key,
            Self::Directory { path } => path,
        }
    }
}

/// Everything about an on-screen object that, if it changes, invalidates what
/// the user pointed at.
///
/// Position is in it deliberately: if the third row becomes the fifth, "the
/// third one" no longer denotes what the user meant, even though the object
/// itself is unchanged. Label and actions are in it for the same reason from
/// the other direction — the object is where it was, but it is not what it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SurfaceFingerprint {
    /// The 1-based position the object held.
    pub ordinal: usize,
    /// The label it carried.
    pub label: String,
    /// The kind the host declared.
    pub kind: String,
    /// The affordances the host declared.
    pub actions: Vec<String>,
}

impl SurfaceFingerprint {
    /// Fingerprint `object` at 1-based position `ordinal`.
    #[must_use]
    pub fn of(object: &SurfaceObject, ordinal: usize) -> Self {
        Self {
            ordinal,
            label: object.label.clone(),
            kind: object.kind.clone(),
            actions: object.actions.clone(),
        }
    }
}

/// The screen a referent was bound from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SurfaceMark {
    /// The snapshot generation it came from.
    pub generation: u64,
    /// What it looked like then.
    pub fingerprint: SurfaceFingerprint,
}

/// Why a referent may no longer be acted on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Staleness {
    /// The turn it was bound in is more than
    /// [`DEFAULT_RETENTION_TURNS`] turns behind.
    Expired,
    /// Still on screen, but renumbered, relabelled, or with different actions.
    Moved,
    /// The screen changed and it is not in the new snapshot.
    Gone,
    /// A screen referent, and there is no current snapshot to vouch for it.
    SurfaceUnknown,
}

impl Staleness {
    /// A stable machine token, for a tool result or a log record.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expired => "expired",
            Self::Moved => "moved",
            Self::Gone => "gone",
            Self::SurfaceUnknown => "surface_unknown",
        }
    }

    /// Every reason, in declaration order.
    pub const ALL: [Self; 4] = [Self::Expired, Self::Moved, Self::Gone, Self::SurfaceUnknown];
}

/// A bound referent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Referent {
    /// `ref_1`, `ref_2`, … Never reused.
    pub handle: String,
    /// What the user would call it, cleaned and bounded.
    pub label: String,
    /// Other things the user might call it.
    pub aliases: Vec<String>,
    /// What it points at.
    pub target: ReferentTarget,
    /// Its 1-based position in the enumeration that bound it, if any.
    pub ordinal: Option<usize>,
    /// The turn it was bound in.
    pub bound_turn: u64,
    /// The screen it came from, if any.
    pub mark: Option<SurfaceMark>,
}

impl Referent {
    /// A candidate report of this referent.
    #[must_use]
    pub fn candidate(&self) -> Candidate {
        Candidate {
            handle: self.handle.clone(),
            label: self.label.clone(),
            kind: self.target.kind(),
            ordinal: self.ordinal,
        }
    }

    /// Whether this is the screen object `object_id`, still looking exactly as
    /// `fingerprint` describes.
    fn is_screen_object(&self, object_id: &str, fingerprint: &SurfaceFingerprint) -> bool {
        matches!(
            (&self.target, &self.mark),
            (ReferentTarget::Screen { object_id: id }, Some(mark))
                if id == object_id && mark.fingerprint == *fingerprint
        )
    }

    /// Whether `needle` — already cleaned and lowercased — equals the label or
    /// any alias.
    fn matches_exact(&self, needle: &str) -> bool {
        std::iter::once(&self.label)
            .chain(self.aliases.iter())
            .any(|value| !value.is_empty() && value.to_lowercase() == needle)
    }

    /// Whether `needle` and the label or any alias contain one another.
    fn matches_partial(&self, needle: &str) -> bool {
        std::iter::once(&self.label)
            .chain(self.aliases.iter())
            .any(|value| {
                if value.is_empty() {
                    return false;
                }
                let lowered = value.to_lowercase();
                lowered.contains(needle) || needle.contains(lowered.as_str())
            })
    }
}

/// What the model is told about one thing it might have meant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Candidate {
    /// The handle to send back.
    pub handle: String,
    /// What to call it when asking.
    pub label: String,
    /// Which kind of thing it is.
    pub kind: &'static str,
    /// Its position, when it came from an enumeration.
    pub ordinal: Option<usize>,
}

/// How a phrase, a handle or an ordinal resolved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "resolution")]
pub enum Resolution {
    /// Exactly one live referent.
    Found(Box<Referent>),
    /// Several plausible ones. **The model asks; it never picks.**
    Ambiguous {
        /// Every live referent that matched, capped at
        /// [`MAX_RESULT_CANDIDATES`](via_conversation::notes::MAX_RESULT_CANDIDATES).
        candidates: Vec<Candidate>,
    },
    /// Nothing live matched.
    NotFound {
        /// The live referents there are, capped, so the model can offer them.
        candidates: Vec<Candidate>,
    },
    /// One referent matched, and it may not be acted on.
    Stale {
        /// Which one.
        handle: String,
        /// Why.
        reason: Staleness,
        /// What is live instead, capped.
        candidates: Vec<Candidate>,
    },
}

impl Resolution {
    /// The referent, when there is exactly one.
    #[must_use]
    pub fn found(&self) -> Option<&Referent> {
        match self {
            Self::Found(referent) => Some(referent),
            _ => None,
        }
    }

    /// Whether this resolution is actionable.
    #[must_use]
    pub const fn is_found(&self) -> bool {
        matches!(self, Self::Found(_))
    }

    /// A stable machine token for the outcome.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Found(_) => "found",
            Self::Ambiguous { .. } => "ambiguous",
            Self::NotFound { .. } => "not_found",
            Self::Stale { .. } => "stale",
        }
    }
}

/// What to bind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferentBinding {
    /// What the user would call it.
    pub label: String,
    /// Other things the user might call it.
    pub aliases: Vec<String>,
    /// Its 1-based position in the enumeration that produced it, if any.
    pub ordinal: Option<usize>,
    /// The screen it came from, if any.
    pub mark: Option<SurfaceMark>,
    /// What it points at.
    pub target: Option<ReferentTarget>,
}

impl ReferentBinding {
    /// A binding for `target` labelled `label`.
    #[must_use]
    pub fn new(target: ReferentTarget, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            aliases: Vec::new(),
            ordinal: None,
            mark: None,
            target: Some(target),
        }
    }

    /// Add alternative spoken names.
    #[must_use]
    pub fn with_aliases<I, S>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.aliases = aliases.into_iter().map(Into::into).collect();
        self
    }

    /// Record the enumeration position and the screen it came from.
    #[must_use]
    pub fn from_surface(mut self, ordinal: usize, mark: SurfaceMark) -> Self {
        self.ordinal = Some(ordinal);
        self.mark = Some(mark);
        self
    }
}

/// The screen the registry last observed.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedSurface {
    generation: u64,
    objects: IndexMap<String, SurfaceFingerprint>,
}

/// The bindings for one session.
///
/// Plain state with `&mut self` methods and no interior mutability. The four
/// concurrency invariants of `docs/architecture.md` §11 are elsewhere; a
/// referent registry has no ordering requirement across tasks, so it belongs to
/// whichever task owns the session rather than acquiring a lock of its own.
#[derive(Clone, Debug)]
pub struct ReferentRegistry {
    turn: u64,
    next_handle: u64,
    retention_turns: u64,
    capacity: usize,
    entries: IndexMap<String, Referent>,
    surface: Option<ObservedSurface>,
}

impl Default for ReferentRegistry {
    fn default() -> Self {
        Self {
            turn: 0,
            next_handle: 1,
            retention_turns: DEFAULT_RETENTION_TURNS,
            capacity: DEFAULT_CAPACITY,
            entries: IndexMap::new(),
            surface: None,
        }
    }
}

impl ReferentRegistry {
    /// A registry with the default retention and capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A registry with explicit limits.
    ///
    /// # Errors
    ///
    /// [`ContextError::InvalidLimit`] when either limit is zero. A capacity of
    /// zero holds nothing; a retention of zero expires a referent inside the
    /// turn that bound it, which would make "that one" never work at all.
    pub fn with_limits(retention_turns: u64, capacity: usize) -> Result<Self, ContextError> {
        if retention_turns == 0 {
            return Err(ContextError::InvalidLimit {
                limit: "retention_turns",
            });
        }
        if capacity == 0 {
            return Err(ContextError::InvalidLimit { limit: "capacity" });
        }
        Ok(Self {
            retention_turns,
            capacity,
            ..Self::default()
        })
    }

    /// The current turn.
    #[must_use]
    pub const fn turn(&self) -> u64 {
        self.turn
    }

    /// How many bindings are held, live or not.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The generation of the last observed screen.
    #[must_use]
    pub fn surface_generation(&self) -> Option<u64> {
        self.surface.as_ref().map(|surface| surface.generation)
    }

    /// Advance to the next turn, dropping everything that has expired.
    ///
    /// Returns the new turn number. The caller maps its own turn id — `via-voice`'s
    /// `voice-<epoch>-<generation>` or `text_<uuid>` — onto this counter; the
    /// registry only needs the ordering.
    pub fn begin_turn(&mut self) -> u64 {
        self.turn = self.turn.saturating_add(1);
        self.drop_expired();
        self.turn
    }

    /// Bind a referent and return its handle.
    ///
    /// # Errors
    ///
    /// [`ContextError::UnbindableReferent`] when there is no target, or the
    /// target's identity is blank. Everything else is bounded rather than
    /// refused: the label and each alias are cleaned and clipped to
    /// [`MAX_LABEL_CHARS`], and the alias list to [`MAX_ALIASES`].
    pub fn bind(&mut self, binding: ReferentBinding) -> Result<String, ContextError> {
        let Some(target) = binding.target else {
            return Err(ContextError::UnbindableReferent {
                reason: "no target",
            });
        };
        if target.identity().trim().is_empty() {
            return Err(ContextError::UnbindableReferent {
                reason: "the target's identity is blank",
            });
        }

        // Eviction is oldest-binding-first, which is insertion order.
        while self.entries.len() >= self.capacity {
            if self.entries.shift_remove_index(0).is_none() {
                break;
            }
        }

        let handle = format!("{HANDLE_PREFIX}{}", self.next_handle);
        self.next_handle = self.next_handle.saturating_add(1);
        let referent = Referent {
            handle: handle.clone(),
            label: clean_bounded(&binding.label, MAX_LABEL_CHARS),
            aliases: binding
                .aliases
                .iter()
                .map(|alias| clean_bounded(alias, MAX_LABEL_CHARS))
                .filter(|alias| !alias.is_empty())
                .take(MAX_ALIASES)
                .collect(),
            target,
            ordinal: binding.ordinal,
            bound_turn: self.turn,
            mark: binding.mark,
        };
        self.entries.insert(handle.clone(), referent);
        Ok(handle)
    }

    /// Bind a screen object, reusing its handle when the object is unchanged.
    ///
    /// Re-reading a screen that has not moved must not churn handles. If it
    /// did, two problems would follow at once: the registry would fill with
    /// duplicates of the same row until a phrase that resolved last turn came
    /// back [`Ambiguous`](Resolution::Ambiguous), and the model would be handed
    /// a different `ref_N` for something it was already told about.
    ///
    /// Reuse is conditional on the [`SurfaceFingerprint`] being **identical** —
    /// same position, same label, same kind, same actions. That condition is
    /// exactly [`staleness`](Self::staleness)'s definition of *not*
    /// [`Moved`](Staleness::Moved), and it is what keeps the reuse safe: an
    /// object that moved gets a **new** handle, and the old one stays behind to
    /// report `Moved`. Refreshing a moved object in place would do the opposite
    /// — it would make the model's stale idea of `ref_1` silently point at a
    /// different row.
    ///
    /// A reused binding has its turn and generation renewed, because it has
    /// just been shown to the model again and deserves a full lease.
    ///
    /// # Errors
    ///
    /// As [`bind`](Self::bind), when there is nothing to bind.
    pub fn bind_surface_object(
        &mut self,
        binding: ReferentBinding,
    ) -> Result<String, ContextError> {
        let reusable = match (&binding.target, &binding.mark) {
            (Some(ReferentTarget::Screen { object_id }), Some(mark)) => self
                .entries
                .iter()
                .find(|(_, referent)| referent.is_screen_object(object_id, &mark.fingerprint))
                .map(|(handle, _)| (handle.clone(), mark.clone())),
            _ => None,
        };
        let Some((handle, mark)) = reusable else {
            return self.bind(binding);
        };
        let turn = self.turn;
        if let Some(referent) = self.entries.get_mut(&handle) {
            referent.bound_turn = turn;
            referent.mark = Some(mark);
        }
        Ok(handle)
    }

    /// Record the screen the session is looking at.
    ///
    /// # Errors
    ///
    /// [`ContextError::SurfaceGenerationReused`] when `snapshot` repeats a
    /// generation the registry has already seen with a different object list.
    /// The generation *is* the ordering contract: a source that renumbers
    /// without advancing it would silently change what "the third one" means,
    /// so the contradiction is reported rather than absorbed.
    pub fn observe_surface(&mut self, snapshot: &SurfaceSnapshot) -> Result<(), ContextError> {
        let objects: IndexMap<String, SurfaceFingerprint> = snapshot.fingerprints().collect();
        if let Some(previous) = &self.surface
            && previous.generation == snapshot.generation
            && previous.objects != objects
        {
            return Err(ContextError::SurfaceGenerationReused {
                generation: snapshot.generation,
            });
        }
        self.surface = Some(ObservedSurface {
            generation: snapshot.generation,
            objects,
        });
        Ok(())
    }

    /// Forget the screen.
    ///
    /// Every screen referent becomes [`Staleness::SurfaceUnknown`] rather than
    /// staying resolvable against a snapshot nobody can confirm.
    pub fn forget_surface(&mut self) {
        self.surface = None;
    }

    /// One referent by handle, whatever its freshness.
    #[must_use]
    pub fn get(&self, handle: &str) -> Option<&Referent> {
        self.entries.get(handle)
    }

    /// Why `referent` may not be acted on, or `None` when it is fine.
    #[must_use]
    pub fn staleness(&self, referent: &Referent) -> Option<Staleness> {
        if self.turn.saturating_sub(referent.bound_turn) >= self.retention_turns {
            return Some(Staleness::Expired);
        }
        let ReferentTarget::Screen { object_id } = &referent.target else {
            return None;
        };
        let mark = referent.mark.as_ref()?;
        let Some(surface) = &self.surface else {
            return Some(Staleness::SurfaceUnknown);
        };
        match surface.objects.get(object_id) {
            None => Some(Staleness::Gone),
            Some(current) if *current != mark.fingerprint => Some(Staleness::Moved),
            Some(_) => None,
        }
    }

    /// Every referent that may still be acted on, in binding order.
    pub fn live(&self) -> impl Iterator<Item = &Referent> {
        self.entries
            .values()
            .filter(|referent| self.staleness(referent).is_none())
    }

    /// The live referents, as candidates, capped at
    /// [`MAX_RESULT_CANDIDATES`](via_conversation::notes::MAX_RESULT_CANDIDATES).
    #[must_use]
    pub fn candidates(&self) -> Vec<Candidate> {
        self.live()
            .take(MAX_RESULT_CANDIDATES)
            .map(Referent::candidate)
            .collect()
    }

    /// Resolve one handle.
    #[must_use]
    pub fn resolve_handle(&self, handle: &str) -> Resolution {
        match self.entries.get(handle.trim()) {
            Some(referent) => self.settle(referent),
            None => Resolution::NotFound {
                candidates: self.candidates(),
            },
        }
    }

    /// Resolve a spoken phrase.
    ///
    /// The ladder is the module documentation's, in order: a handle, then an
    /// exact match over live referents, then a unique bidirectional substring,
    /// then the same two passes over stale ones so a dead match is reported as
    /// dead, then candidates.
    #[must_use]
    pub fn resolve(&self, phrase: &str) -> Resolution {
        let needle = clean_bounded(phrase, MAX_LABEL_CHARS).to_lowercase();
        if needle.is_empty() {
            return Resolution::NotFound {
                candidates: self.candidates(),
            };
        }
        if is_handle(&needle) {
            return self.resolve_handle(&needle);
        }

        let live: Vec<&Referent> = self.live().collect();
        match Self::ladder(&live, &needle) {
            Rung::One(referent) => return self.settle(referent),
            Rung::Several(matches) => {
                return Resolution::Ambiguous {
                    candidates: cap(&matches),
                };
            }
            Rung::None => {}
        }

        let stale: Vec<&Referent> = self
            .entries
            .values()
            .filter(|referent| self.staleness(referent).is_some())
            .collect();
        if let Rung::One(referent) = Self::ladder(&stale, &needle) {
            return self.settle(referent);
        }
        Resolution::NotFound {
            candidates: self.candidates(),
        }
    }

    /// [`Resolution::Found`] or [`Resolution::Stale`], never a guess.
    fn settle(&self, referent: &Referent) -> Resolution {
        match self.staleness(referent) {
            Some(reason) => Resolution::Stale {
                handle: referent.handle.clone(),
                reason,
                candidates: self.candidates(),
            },
            None => Resolution::Found(Box::new(referent.clone())),
        }
    }

    /// `via-conversation`'s two-pass match, over an arbitrary set.
    ///
    /// Exact first. When several match exactly the substring pass is skipped
    /// entirely — `frontend-notes.mjs:72-83`'s asymmetry, kept for its reason:
    /// two identically-named things must report each other rather than have one
    /// picked arbitrarily.
    fn ladder<'a>(pool: &[&'a Referent], needle: &str) -> Rung<'a> {
        let exact: Vec<&Referent> = pool
            .iter()
            .copied()
            .filter(|referent| referent.matches_exact(needle))
            .collect();
        if let [only] = exact.as_slice() {
            return Rung::One(only);
        }
        let partial: Vec<&Referent> = if exact.is_empty() {
            pool.iter()
                .copied()
                .filter(|referent| referent.matches_partial(needle))
                .collect()
        } else {
            exact
        };
        match partial.as_slice() {
            [] => Rung::None,
            [only] => Rung::One(only),
            _ => Rung::Several(partial),
        }
    }

    fn drop_expired(&mut self) {
        let turn = self.turn;
        let retention = self.retention_turns;
        self.entries
            .retain(|_, referent| turn.saturating_sub(referent.bound_turn) < retention);
    }
}

/// Whether `value` has the exact shape of a handle: [`HANDLE_PREFIX`] and at
/// least one digit, and nothing else.
///
/// The whole shape rather than the prefix alone, so a label a host chose —
/// `ref_docs`, `ref_manual` — still resolves as a label instead of being routed
/// to a handle lookup that can only miss.
fn is_handle(value: &str) -> bool {
    match value.strip_prefix(HANDLE_PREFIX) {
        Some(digits) => !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()),
        None => false,
    }
}

/// One rung's answer.
enum Rung<'a> {
    None,
    One(&'a Referent),
    Several(Vec<&'a Referent>),
}

fn cap(matches: &[&Referent]) -> Vec<Candidate> {
    matches
        .iter()
        .take(MAX_RESULT_CANDIDATES)
        .map(|referent| referent.candidate())
        .collect()
}
