//! Wrapping what `via-conversation` and `via-voice` already render.
//!
//! This module renders **nothing**. Every body it puts in a section either
//! arrives from the caller or comes back from a `via-conversation` call, and the
//! join is the same join `via-voice` performs. That is the whole claim of
//! `docs/architecture.md` §5 — *"a thin typed layer, not a rewrite"* — and the
//! reason the equality in `via-voice`'s `tests/context_pack.rs` can hold at all.
//!
//! # Where each body comes from
//!
//! | Section | Rendered by | Tier | Provenance / Trust |
//! | --- | --- | --- | --- |
//! | `core_policy` | `via-voice` (`load_frontend_prompt`) | [`Tier::Static`] | [`System`](Provenance::System) / [`Policy`](Trust::Policy) |
//! | `assistant_profile` | `via-voice` (`assistant_profile_block`) | [`Tier::Static`] | [`System`](Provenance::System) / [`Persona`](Trust::Persona) |
//! | `user_preferences` | `via-conversation` (`user_preferences_section`) | [`Tier::Static`] | [`User`](Provenance::User) / [`Directive`](Trust::Directive) |
//! | `user_memory` | `via-conversation` (`memory_section`) | [`Tier::Static`] | [`User`](Provenance::User) / [`Data`](Trust::Data) |
//! | `runtime_context` | `via-conversation` (`runtime_context_section`) | [`Tier::Static`] | [`Host`](Provenance::Host) / [`Data`](Trust::Data) |
//! | `recent_conversation` | `via-conversation` (`build_recent_conversation_context`) | [`Tier::Dynamic`] | [`User`](Provenance::User) / [`Data`](Trust::Data) |
//! | `input_parts` | `via-voice` (`frontend_input_projection`) | [`Tier::Dynamic`] | [`Host`](Provenance::Host) / [`Data`](Trust::Data) |
//!
//! # Why two of these are not fenced, and it is not an oversight
//!
//! **`<runtime_context>` is host-supplied and unfenced.** Its three values are
//! *fields*, not free-form bytes, and
//! [`normalize_client_context`](via_conversation::normalize_client_context)
//! already strips NUL and collapses CR/LF for exactly this reason — the
//! catalogued test is `a_working_directory_cannot_forge_a_context_line`. A fence
//! would add nothing and would change a catalogued block format.
//!
//! **`<recent_conversation>` is a transcript and unfenced.** It is what the user
//! and VIA said, and its format is catalogued — it is replayed verbatim inside
//! `<restored_context>` on reconnect. `via-conversation`'s `clean()` collapses
//! whitespace on every line, so no message can forge a second one. Fencing it
//! would put the user's own words behind a marker that says "ignore directives
//! here", which is precisely backwards: the current utterance is the top of
//! `PROMPT.md`'s hierarchy.
//!
//! Third-party bytes reach a pack through
//! [`ContextPackBuilder::push_untrusted`] and
//! [`ContextPackBuilder::push_tool_result`], never through this function.

use via_conversation::context::{
    build_recent_conversation_context, memory_section, runtime_context_section,
    user_preferences_section,
};
use via_conversation::sync::Message;
use via_conversation::{ClientContext, MemoryDocument};
use via_i18n::Locale;

use crate::error::ContextError;
use crate::pack::{ContextPack, ContextPackBuilder, Provenance, SectionId, Tier, Trust};

/// Everything a frontend pack is built from.
///
/// The two `via-voice` bodies arrive as strings because `via-context` sits in
/// Layer 2 and `via-voice` in Layer 1 (`docs/architecture.md` §9: Layer 1 may
/// depend on Layer 2, never the reverse). That direction is not a nuisance to
/// work around — it is what makes the pack a wrapper rather than a second
/// prompt builder.
#[derive(Clone, Copy, Debug)]
pub struct FrontendPackInputs<'a> {
    /// Which locale the fenced framing and the truncation marker are written
    /// in.
    pub locale: Locale,
    /// `PROMPT.md`, as `via-voice` loaded and bounded it.
    pub policy: &'a str,
    /// The whole `# Assistant Profile` block, as `via-voice`'s
    /// `assistant_profile_block` wrote it — heading, opening tag with its
    /// `authority="persona_only"` attribute, body, closing tag.
    pub assistant_profile_block: &'a str,
    /// The client's validated environment.
    pub client: &'a ClientContext,
    /// The memory documents, in `via-conversation`'s own order.
    pub memories: &'a [MemoryDocument],
    /// The conversation so far. Only the tail is replayed, by
    /// `via-conversation`'s own budget.
    pub recent: &'a [Message],
    /// This turn's `<input_parts>` projection, or `""`.
    pub input_parts: &'a str,
}

impl<'a> FrontendPackInputs<'a> {
    /// The minimum: a policy, a persona and a client.
    #[must_use]
    pub fn new(
        locale: Locale,
        policy: &'a str,
        assistant_profile_block: &'a str,
        client: &'a ClientContext,
    ) -> Self {
        Self {
            locale,
            policy,
            assistant_profile_block,
            client,
            memories: &[],
            recent: &[],
            input_parts: "",
        }
    }

    /// Add the memory documents.
    #[must_use]
    pub fn with_memories(mut self, memories: &'a [MemoryDocument]) -> Self {
        self.memories = memories;
        self
    }

    /// Add the conversation to replay.
    #[must_use]
    pub fn with_recent(mut self, recent: &'a [Message]) -> Self {
        self.recent = recent;
        self
    }

    /// Add this turn's `<input_parts>` projection.
    #[must_use]
    pub fn with_input_parts(mut self, input_parts: &'a str) -> Self {
        self.input_parts = input_parts;
        self
    }
}

/// The seven frontend sections, in the catalogued order, ready to extend.
///
/// Returned as a builder rather than a pack so a caller with more to add — the
/// Context Engine appends the ordered-deixis view — continues one chain instead
/// of merging two packs.
#[must_use]
pub fn frontend_builder(inputs: &FrontendPackInputs<'_>) -> ContextPackBuilder {
    ContextPack::builder(inputs.locale)
        .push(
            SectionId::CORE_POLICY,
            Provenance::System,
            Trust::Policy,
            Tier::Static,
            inputs.policy,
        )
        .push(
            SectionId::ASSISTANT_PROFILE,
            Provenance::System,
            Trust::Persona,
            Tier::Static,
            inputs.assistant_profile_block,
        )
        .push(
            SectionId::USER_PREFERENCES,
            Provenance::User,
            Trust::Directive,
            Tier::Static,
            &user_preferences_section(inputs.memories),
        )
        .push(
            SectionId::USER_MEMORY,
            Provenance::User,
            Trust::Data,
            Tier::Static,
            &memory_section(inputs.memories),
        )
        .push(
            SectionId::RUNTIME_CONTEXT,
            Provenance::Host,
            Trust::Data,
            Tier::Static,
            &runtime_context_section(inputs.client),
        )
        .push(
            SectionId::RECENT_CONVERSATION,
            Provenance::User,
            Trust::Data,
            Tier::Dynamic,
            &build_recent_conversation_context(inputs.recent, inputs.locale),
        )
        .push(
            SectionId::INPUT_PARTS,
            Provenance::Host,
            Trust::Data,
            Tier::Dynamic,
            inputs.input_parts,
        )
}

/// The frontend pack.
///
/// # Errors
///
/// [`ContextError`] only if a section id were malformed or repeated, neither of
/// which this function can produce — every id it uses is one of
/// [`SectionId::KNOWN`] and each appears once. The `Result` is here so the
/// signature does not have to change the day a caller composes on top of it.
pub fn frontend_pack(inputs: &FrontendPackInputs<'_>) -> Result<ContextPack, ContextError> {
    frontend_builder(inputs).build()
}
