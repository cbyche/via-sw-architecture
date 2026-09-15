//! Provenance fencing — one delimiter, extended from ARGO's, driven by where
//! the *bytes* came from rather than by who fetched them.
//!
//! # What was already right, and what was missing
//!
//! `docs/architecture.md` §5: *"Fencing needs a provenance dimension, not a
//! fourth delimiter. ARGO's mechanism is correct and fail-closed; what is
//! missing is classification — a first-party tool that returns third-party
//! bytes is currently trusted."*
//!
//! The ARGO upstream has three unrelated conventions, each covering one narrow
//! slice:
//!
//! | Where | Delimiter | Neutralisation |
//! | --- | --- | --- |
//! | `tinicore/src/triage/triage.rs`, `classifier/binary/llm.rs` | `<\|user_input_start\|>` / `<\|user_input_end\|>` | every occurrence of either sentinel replaced by `[REDACTED]` |
//! | `tinicore/src/agent/cognitive_recall.rs` | `<recalled_memories>` | a space inserted after the leading `<` so the tag no longer parses |
//! | `tinicore/src/workflow/schema.rs` | `<answer>` / `</answer>` | `<`/`>` escaped to `&lt;`/`&gt;`, case- and whitespace-insensitively |
//!
//! All three are correct and all three fail closed. VIA adds no fourth marker
//! syntax. It takes the **sentinel-pair shape** — the one ARGO chose precisely
//! because `<|…|>` cannot be confused with the XML-ish tags the rest of the
//! prompt uses — and it takes **both** neutralisation mechanisms, each where
//! the upstream uses it:
//!
//! 1. VIA's own sentinels are **redacted** to [`REDACTION_PLACEHOLDER`], the
//!    triage rule. A placeholder rather than deletion is load-bearing: simple
//!    removal lets `<|via_untru<|via_untrusted_end|>sted_end|>` realign into a
//!    valid closing sentinel, and a placeholder that is not itself a substring
//!    of the sentinel makes that unrepresentable. Asserted directly by
//!    `the_placeholder_cannot_realign_into_a_sentinel`.
//! 2. Tag openers are **escaped** to `&lt;`, the answer-fence rule, case- and
//!    whitespace-insensitively so `< / user_preferences >` is covered too.
//!
//! # One deliberate widening: no tag allow-list
//!
//! ARGO escapes *the one tag it fences with*. That is right for a fence around
//! a single known payload and wrong here, because a VIA prompt carries six
//! model-visible tags (`<user_preferences>`, `<user_memory>`,
//! `<runtime_context>`, `<recent_conversation>`, `<assistant_profile …>`,
//! `<input_parts>`) and a seventh will be added one day by someone who does not
//! know this file exists. A blocklist of tag names would then have a hole in it
//! from the moment the tag landed.
//!
//! So [`neutralize`] escapes **anything that could open a tag at all** — `<`
//! followed by optional whitespace, an optional `/`, more optional whitespace,
//! and a character that could begin a name (`[A-Za-z_]`, `!`, `?`, `|`).
//! Markup inside an untrusted block does get mangled, and that is the correct
//! outcome: a page's raw HTML is being quoted as evidence, not rendered.
//!
//! The whitespace tolerance costs one class of false positive and buys the
//! attack it exists to stop. `a < b` becomes `a &lt; b`, because ARGO's own
//! escaper is whitespace-tolerant for the reason its comment gives — *"a model
//! treats those as the same closing tag"* — and dropping the tolerance would
//! let `< / user_preferences >` through. Erring toward a mangled comparison
//! inside a block of quoted evidence is the right direction; erring toward a
//! forged directive block is not. Numeric comparisons, which are the common
//! ones, are untouched: `<3 s` and `< 5 items` both survive, because a digit
//! cannot begin a tag name.
//!
//! # Growth is bounded, and truncation happens first
//!
//! Redaction shrinks (`<|via_untrusted_end|>`, 21 code points, becomes 10).
//! Escaping grows by 3 code points per escaped `<`, and the shortest escapable
//! run is 2 code points (`<a`), so the ceiling is 2.5×. The cap is applied to
//! the **raw** body before neutralisation — ARGO's `#2069` round-3 review #5
//! lesson — so the bounded escape growth cannot be used to inflate a body past
//! [`MAX_FENCED_CHARS`], and the truncation marker VIA appends is its own
//! trusted text carrying no fence token.
//!
//! # Why the framing does not name the sentinels
//!
//! ARGO's classifier framing names both sentinels by literal name, which makes
//! every prompt carry each one at least twice and forces its own test to note
//! that *"the real fence is the last one"*. VIA's block is delimited
//! structurally — a line that is exactly [`FENCE_OPEN`] and a line that is
//! exactly [`FENCE_CLOSE`] — so the framing describes the content instead. One
//! literal occurrence of each sentinel per block means there is no first-versus-
//! last ambiguity to reason about.

use via_conversation::context::MAX_PROMPT_CHARS;
use via_conversation::text::{bounded_code_points, code_point_len, trim};
use via_i18n::{Locale, format as i18n_format, keys};

/// The opening sentinel of a fenced block.
///
/// The `<|…|>` shape is ARGO's (`tinicore/src/triage/triage.rs:92`), chosen
/// there because such sentinels *"are extraordinarily unlikely to appear in
/// real Korean/English requests"* and because they cannot be mistaken for one
/// of the prompt's own tags.
pub const FENCE_OPEN: &str = "<|via_untrusted_start|>";

/// The closing sentinel of a fenced block.
pub const FENCE_CLOSE: &str = "<|via_untrusted_end|>";

/// What every occurrence of either sentinel inside an untrusted body becomes.
///
/// ARGO's `SENTINEL_PLACEHOLDER` (`classifier/binary/llm.rs:183`), for ARGO's
/// reason: it is *"a visible diagnostic trail that can never realign into a
/// valid sentinel via simple substring removal"*.
pub const REDACTION_PLACEHOLDER: &str = "[REDACTED]";

/// The name inside [`FENCE_OPEN`].
const OPEN_NAME: &str = "via_untrusted_start";

/// The name inside [`FENCE_CLOSE`].
const CLOSE_NAME: &str = "via_untrusted_end";

/// The code-point cap on one fenced body.
///
/// Derived rather than chosen: half of
/// [`MAX_PROMPT_CHARS`](via_conversation::context::MAX_PROMPT_CHARS), so no
/// single block of untrusted content can outweigh the core policy that governs
/// it. Deriving it means a change to the policy cap moves this one too, instead
/// of leaving a second magic number behind.
pub const MAX_FENCED_CHARS: usize = MAX_PROMPT_CHARS / 2;

/// Where a fenced block's bytes came from.
///
/// This is the **content** dimension, not the caller's. A first-party VIA tool
/// that reads a web page produces [`FenceSource::Web`]; the tool's own trust is
/// irrelevant to what its bytes are. See
/// [`Provenance::classify`](crate::pack::Provenance::classify).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FenceSource {
    /// Whatever is on the host's screen — labels, titles, row text.
    Screen,
    /// A tool result composed from bytes VIA did not author.
    Tool,
    /// A page, a feed, an API response.
    Web,
    /// A file's contents.
    File,
    /// A backend agent's own output.
    Backend,
    /// Provenance could not be established.
    ///
    /// The default, and the fail-closed one: content nobody classified is
    /// fenced as unknown rather than trusted.
    #[default]
    Unknown,
}

impl FenceSource {
    /// A stable machine token, interpolated into the framing sentence.
    ///
    /// A fixed enum rather than a caller string: the token reaches the model
    /// inside our own trusted framing line, so it must not be something a third
    /// party can choose.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::Tool => "tool",
            Self::Web => "web",
            Self::File => "file",
            Self::Backend => "backend",
            Self::Unknown => "unknown",
        }
    }

    /// Every source, in declaration order.
    pub const ALL: [Self; 6] = [
        Self::Screen,
        Self::Tool,
        Self::Web,
        Self::File,
        Self::Backend,
        Self::Unknown,
    ];
}

/// Make `body` unable to escape the fence it is about to be wrapped in.
///
/// Two passes, in order, each one of ARGO's:
///
/// 1. every VIA sentinel — case-insensitively, tolerating whitespace inside the
///    marker — becomes [`REDACTION_PLACEHOLDER`];
/// 2. every `<` that could open a tag becomes `&lt;`.
///
/// # Termination and idempotence
///
/// Both passes scan left to right and advance by at least one code point per
/// step, so each is linear and terminates by construction. Neither can create
/// work for the other: redaction emits a placeholder containing no `<` and no
/// `|`, and escaping only ever removes a `<`. Applying [`neutralize`] to its own
/// output is therefore a no-op, which
/// `neutralizing_twice_changes_nothing_the_first_pass_did_not` asserts over an
/// adversarial corpus — the bounded form of "this converges".
#[must_use]
pub fn neutralize(body: &str) -> String {
    escape_tag_openers(&redact_sentinels(body))
}

/// Wrap `body` in the fence, with its framing line, in `locale`.
///
/// Returns `""` for an empty body: an empty fence is noise, and a block with
/// nothing in it is not evidence.
///
/// The body is trimmed, capped at [`MAX_FENCED_CHARS`] code points **before**
/// neutralisation, then neutralised. A truncated body gains VIA's own marker,
/// which is trusted text and carries no fence token.
#[must_use]
pub fn wrap(locale: Locale, source: FenceSource, body: &str) -> String {
    let trimmed = trim(body);
    if trimmed.is_empty() {
        return String::new();
    }
    let capped = if code_point_len(trimmed) > MAX_FENCED_CHARS {
        let mut clipped = bounded_code_points(trimmed, MAX_FENCED_CHARS);
        clipped.push('\n');
        clipped.push_str(via_i18n::t(locale, keys::CONTEXT_FENCED_TRUNCATED));
        clipped
    } else {
        trimmed.to_owned()
    };
    [
        FENCE_OPEN,
        &i18n_format(
            locale,
            keys::CONTEXT_UNTRUSTED_FRAMING,
            &[("source", source.as_str())],
        ),
        &neutralize(&capped),
        FENCE_CLOSE,
    ]
    .join("\n")
}

/// Replace every VIA sentinel with [`REDACTION_PLACEHOLDER`].
///
/// Tolerant in exactly the ways a model is: case is ignored and ASCII
/// whitespace is allowed between the marker's parts, so `<| VIA_UNTRUSTED_END
/// |>` is redacted as readily as the canonical spelling. That tolerance is
/// the upstream's `neutralize_answer_fence`, cited in this module's own
/// documentation table — *"a model treats those as the same closing tag, so a
/// case-sensitive escape would be trivially bypassable"*.
fn redact_sentinels(body: &str) -> String {
    let chars: Vec<char> = body.chars().collect();
    let mut out = String::with_capacity(body.len());
    let mut index = 0usize;
    while index < chars.len() {
        // `sentinel_end` answers `Some(end)` only with `end > index`, so this
        // branch advances the scan just as the fall-through does.
        if chars[index] == '<'
            && let Some(end) = sentinel_end(&chars, index)
        {
            out.push_str(REDACTION_PLACEHOLDER);
            index = end;
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

/// The index one past a VIA sentinel starting at `start`, if one is there.
///
/// The grammar accepted is `<` ws\* `|` ws\* NAME ws\* `|` ws\* `>` with NAME
/// matched case-insensitively against [`OPEN_NAME`] or [`CLOSE_NAME`]. The
/// shortest accepted form is 5 code points, so a match always advances.
fn sentinel_end(chars: &[char], start: usize) -> Option<usize> {
    let mut index = start;
    index = expect(chars, index, '<')?;
    index = skip_whitespace(chars, index);
    index = expect(chars, index, '|')?;
    index = skip_whitespace(chars, index);

    let name_start = index;
    while chars
        .get(index)
        .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
    {
        index += 1;
    }
    let name: String = chars[name_start..index].iter().collect();

    index = skip_whitespace(chars, index);
    index = expect(chars, index, '|')?;
    index = skip_whitespace(chars, index);
    index = expect(chars, index, '>')?;

    (name.eq_ignore_ascii_case(OPEN_NAME) || name.eq_ignore_ascii_case(CLOSE_NAME)).then_some(index)
}

fn expect(chars: &[char], index: usize, wanted: char) -> Option<usize> {
    (chars.get(index) == Some(&wanted)).then_some(index + 1)
}

fn skip_whitespace(chars: &[char], mut index: usize) -> usize {
    while chars.get(index).is_some_and(char::is_ascii_whitespace) {
        index += 1;
    }
    index
}

/// Escape every `<` that could open a tag.
///
/// Only the `<` is escaped. That is sufficient — a tag that cannot open cannot
/// close a block or forge one — and it halves the growth an ARGO-style
/// `<`-and-`>` escape would cost. The trailing `>` is left alone because on its
/// own it is inert punctuation.
fn escape_tag_openers(body: &str) -> String {
    let chars: Vec<char> = body.chars().collect();
    let mut out = String::with_capacity(body.len());
    for index in 0..chars.len() {
        if chars[index] == '<' && opens_a_tag(&chars, index) {
            out.push_str("&lt;");
        } else {
            out.push(chars[index]);
        }
    }
    out
}

/// Whether the `<` at `start` is followed by something that could name a tag.
///
/// `<` ws\* `/`? ws\* then `[A-Za-z_!?|]`. The `!` and `?` cover `<!--` and
/// `<?xml`; the `|` covers a sentinel-shaped marker that
/// [`redact_sentinels`] did not recognise, which is exactly the case that
/// should not be left looking like a control token.
fn opens_a_tag(chars: &[char], start: usize) -> bool {
    let mut index = skip_whitespace(chars, start + 1);
    if chars.get(index) == Some(&'/') {
        index = skip_whitespace(chars, index + 1);
    }
    chars
        .get(index)
        .is_some_and(|c| c.is_ascii_alphabetic() || matches!(c, '_' | '!' | '?' | '|'))
}
