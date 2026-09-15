//! Why the Context Engine refused.
//!
//! Every variant is a *refusal*, never a guess. That is the rule the whole
//! crate is built on: an ambiguous referent reports its candidates, a stale one
//! reports why it is stale, and a malformed section is rejected rather than
//! rendered. Nothing here falls back to "probably the first one".

/// Why a pack, a binding or a surface refresh was refused.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ContextError {
    /// A section id is not a well-formed identifier.
    ///
    /// Ids reach diagnostics and log records, and a host may supply one. A
    /// value carrying a newline could forge a second log line, so the grammar
    /// is `[a-z][a-z0-9_]*` bounded at [`crate::pack::MAX_SECTION_ID_CHARS`]
    /// and anything else is refused here rather than sanitized silently.
    #[error("`{value}` is not a well-formed section id: {reason}")]
    SectionId {
        /// What was offered.
        value: String,
        /// Which rule it broke.
        reason: &'static str,
    },
    /// Two sections claim the same id.
    ///
    /// Refused rather than merged or last-wins: two `<user_preferences>` blocks
    /// in one prompt is an injection surface, and silently keeping one of them
    /// hides which.
    #[error("the pack already has a section called `{id}`")]
    DuplicateSection {
        /// The repeated id.
        id: String,
    },
    /// A referent was offered with no label and no target identity.
    ///
    /// A referent must denote something VIA can act on. A binding with nothing
    /// to say and nothing to point at is not a referent, it is a blank.
    #[error("a referent needs a target identity: {reason}")]
    UnbindableReferent {
        /// Which rule it broke.
        reason: &'static str,
    },
    /// The registry is configured with a limit it cannot honour.
    #[error("{limit} must be at least 1")]
    InvalidLimit {
        /// Which limit.
        limit: &'static str,
    },
    /// A surface source answered with a snapshot that contradicts itself.
    ///
    /// The generation *is* the ordering contract (`docs/architecture.md` §5:
    /// the order is the contract). A source that changes its objects without
    /// advancing its generation would silently renumber the user's "third
    /// one", so the contradiction is reported instead of being absorbed.
    #[error("surface generation {generation} was reused for a different object list")]
    SurfaceGenerationReused {
        /// The generation that was reused.
        generation: u64,
    },
    /// The surface source could not answer.
    ///
    /// Carries the source's own message. A session whose screen cannot be read
    /// resolves nothing on screen; it does not guess from the last snapshot.
    #[error("the surface source failed: {detail}")]
    Surface {
        /// The source's message.
        detail: String,
    },
}
