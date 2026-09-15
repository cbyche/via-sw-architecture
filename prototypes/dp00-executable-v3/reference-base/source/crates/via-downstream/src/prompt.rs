//! What goes into a turn, and what comes back.
//!
//! Upstream's call is
//! `client.prompt(sessionId, blocks, { signal, timeoutMs, onUpdate })`
//! (`server/src/agent/acp-process-client.mjs:436-511`) returning
//! `{ content, response }`. This is that call with the transport removed:
//! [`PromptRequest`] carries what any harness shape needs and
//! [`PromptOutcome`] carries what every harness shape returns.
//!
//! # `cancelled` is not a return value
//!
//! `docs/reference/contracts.json` (`json-rpc-method` / *session/prompt*) is
//! unusually direct about the one thing a port must not get wrong:
//!
//! > `stopReason 'cancelled'` is a hard error, not a normal return — a Rust
//! > port that returns `Ok` on `'cancelled'` would silently complete cancelled
//! > Work.
//!
//! So [`PromptOutcome::new`] is fallible and rejects
//! [`StopReason::Cancelled`], and it is the only constructor. A harness cannot
//! hand back a successful outcome for a cancelled turn however it is written;
//! it gets [`HarnessError::Cancelled`] instead, which is what upstream throws.

use crate::error::HarnessError;

/// Why a turn stopped.
///
/// ACP's `StopReason` (`agent-client-protocol` v2, `v2/agent.rs`), which is the
/// vocabulary every shipped harness shape reports in. The `Other` arm is ACP's
/// own escape hatch for a value a future revision adds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StopReason {
    /// The turn ended normally.
    EndTurn,
    /// The agent hit its token limit.
    MaxTokens,
    /// The agent hit its request limit before returning idle.
    MaxTurnRequests,
    /// The agent refused to continue.
    Refusal,
    /// The turn was cancelled.
    ///
    /// Never appears inside a [`PromptOutcome`] — see the module docs.
    Cancelled,
    /// A value this vocabulary does not know.
    ///
    /// ACP reserves values beginning with `_` for implementation-specific
    /// extensions; anything else is a future ACP variant. Either way the turn
    /// ended, and the caller is told what the harness said rather than being
    /// told a plausible-looking lie.
    Other(String),
}

impl StopReason {
    /// The wire spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::EndTurn => "end_turn",
            Self::MaxTokens => "max_tokens",
            Self::MaxTurnRequests => "max_turn_requests",
            Self::Refusal => "refusal",
            Self::Cancelled => "cancelled",
            Self::Other(value) => value,
        }
    }

    /// Parse a wire spelling. Total: an unknown value becomes [`Self::Other`].
    #[must_use]
    pub fn from_wire(value: &str) -> Self {
        match value {
            "end_turn" => Self::EndTurn,
            "max_tokens" => Self::MaxTokens,
            "max_turn_requests" => Self::MaxTurnRequests,
            "refusal" => Self::Refusal,
            "cancelled" => Self::Cancelled,
            other => Self::Other(other.to_owned()),
        }
    }

    /// Whether this is the cancellation reason that must not be reported as
    /// success.
    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

/// A file the user attached to a turn.
///
/// The three fields `inputFileParts` yields (`shared/input-parts.mjs`), which
/// is the transport-neutral shape upstream carries *above* ACP before
/// `inputPartsToAcpBlocks` turns them into content blocks
/// (`server/src/agent/acp-content.mjs:25-65`). The size limits, the
/// `data:`/`http:`/`https:` protocol allow-list and the `data:` URL parser all
/// belong to whoever ports `input-parts.mjs`; restating them here would be a
/// second copy of a security boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptAttachment {
    /// Where the bytes are: a `data:` URL, or an `http(s):` link.
    pub url: String,
    /// The name to show, and to build a resource URI from.
    pub filename: String,
    /// The MIME type, which decides whether this becomes an image, an audio
    /// block, an inline resource or a link.
    pub mime: String,
}

/// One turn's worth of input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromptRequest {
    /// The user's text.
    pub text: String,
    /// Anything attached to it, in order.
    pub attachments: Vec<PromptAttachment>,
    /// Whose turn this is — the same owner id a coordinator
    /// [`SessionKey`](crate::SessionKey) is built from.
    pub owner_id: String,
    /// The Work this turn is being carried out for, when there is one.
    ///
    /// Upstream's `coordinationRunId`
    /// (`server/src/agent/acp-backend-adapter.mjs:756`). It is correlation
    /// only: `docs/architecture.md` §6 — *"a harness answers prompts and emits
    /// events; it never owns a queue, a `work_id`, or a permission decision."*
    pub work_id: Option<String>,
    /// A per-turn deadline in milliseconds.
    ///
    /// `None` means the caller owns the deadline, which is upstream's
    /// `timeoutMs: 0` for delegated work
    /// (`acp-backend-adapter.mjs:774`) — the client's own pausable timer runs
    /// it, so that a turn parked on a permission prompt does not time out.
    pub timeout_ms: Option<u64>,
}

impl PromptRequest {
    /// A text-only turn.
    #[must_use]
    pub fn text(owner_id: &str, text: &str) -> Self {
        Self {
            text: text.to_owned(),
            owner_id: owner_id.to_owned(),
            ..Self::default()
        }
    }

    /// Correlate this turn with a Work id.
    #[must_use]
    pub fn with_work_id(mut self, work_id: &str) -> Self {
        self.work_id = Some(work_id.to_owned());
        self
    }

    /// Attach a file.
    #[must_use]
    pub fn with_attachment(mut self, attachment: PromptAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }
}

/// What a completed turn produced.
///
/// Private fields, one fallible constructor: see the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptOutcome {
    content: String,
    stop_reason: StopReason,
}

impl PromptOutcome {
    /// Build an outcome for a turn that ended.
    ///
    /// `content` is upstream's `active.text.join('').trim()`
    /// (`acp-process-client.mjs:499`) — the agent's message text for the turn,
    /// already assembled by the transport.
    ///
    /// # Errors
    ///
    /// [`HarnessError::Cancelled`] when `stop_reason` is
    /// [`StopReason::Cancelled`]. Upstream throws at exactly this point
    /// (`acp-process-client.mjs:500-502`); returning `Ok` here would let
    /// cancelled Work be recorded as completed.
    pub fn new(content: &str, stop_reason: StopReason) -> Result<Self, HarnessError> {
        if stop_reason.is_cancelled() {
            return Err(HarnessError::Cancelled);
        }
        Ok(Self {
            content: content.to_owned(),
            stop_reason,
        })
    }

    /// The agent's text for the turn.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Why the turn stopped. Never [`StopReason::Cancelled`].
    #[must_use]
    pub const fn stop_reason(&self) -> &StopReason {
        &self.stop_reason
    }

    /// Whether the turn ended normally rather than against a limit or a
    /// refusal.
    #[must_use]
    pub fn is_end_turn(&self) -> bool {
        self.stop_reason == StopReason::EndTurn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cancelled_turn_is_never_an_outcome() {
        let error = PromptOutcome::new("half a result", StopReason::Cancelled)
            .expect_err("cancelled is an error, not a return");
        assert!(error.is_cancelled());
    }

    #[test]
    fn every_other_stop_reason_returns() {
        for reason in [
            StopReason::EndTurn,
            StopReason::MaxTokens,
            StopReason::MaxTurnRequests,
            StopReason::Refusal,
            StopReason::Other("_vendor_stop".to_owned()),
        ] {
            let outcome = PromptOutcome::new("done", reason.clone()).expect("not cancelled");
            assert_eq!(outcome.stop_reason(), &reason);
            assert_eq!(outcome.content(), "done");
        }
    }

    #[test]
    fn stop_reasons_round_trip() {
        for wire in [
            "end_turn",
            "max_tokens",
            "max_turn_requests",
            "refusal",
            "cancelled",
            "_vendor",
        ] {
            assert_eq!(StopReason::from_wire(wire).as_str(), wire);
        }
    }
}
