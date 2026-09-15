//! The duplicate-`response.create` filter, and the accounting that makes it
//! safe.
//!
//! Ported from ARGO `tinicore/src/llm/providers/openai_live.rs:132-190,1128-1141`
//! (`is_benign_cancel_race`, `is_active_response_conflict`,
//! `creates_in_flight`) against **ARGO bring-up §8b**, which is the evidence.
//!
//! # What §8b established
//!
//! The litellm gateway issues **its own** duplicate `response.create` against
//! the upstream on each VAD commit; the upstream refuses it (a response is
//! already streaming) and the gateway relays the refusal to us. A 2026-08-02 run
//! with full outbound logging pinned the shape — four occurrences, all
//! identical:
//!
//! ```text
//! inbound  speech_stopped → committed
//! inbound  response.created          ← server VAD opens resp_X
//!    (~700 ms, ZERO outbound frames from us)
//! inbound  error: already has an active response: resp_X
//! ```
//!
//! Three things were established: the client was **passive** in every error
//! window; the complained-about id is the response the server itself just
//! created; and our own `response.create`s never bounce.
//!
//! # The accounting
//!
//! [`CreateAccounting`] counts client `response.create`s not yet answered by a
//! `response.created`. The decrement is **saturating**, because server VAD opens
//! responses no client asked for and an unsaturated counter would go negative
//! and then never recover.
//!
//! An `already has an active response` error with **zero creates unaccounted for
//! cannot be ours** and is dropped with a trace line. One that *could* concern a
//! create of ours still surfaces — so a client regression that double-sends
//! stays visible — and the surfaced one consumes its slot.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

/// The GA error code for the conflict.
///
/// External contract — ARGO `openai_live.rs:170`. Matched first, because it is
/// the precise signal; the message match below is the relay fallback.
pub const ACTIVE_RESPONSE_CONFLICT_CODE: &str = "conversation_already_has_active_response";

/// The invariant phrase of the conflict message.
///
/// External contract — ARGO `openai_live.rs:180`. Deliberately the phrase rather
/// than the whole sentence: the wording differs between OpenAI, Azure and the
/// gateways in between, and the sentence carries a response id that changes
/// every time.
pub const ACTIVE_RESPONSE_CONFLICT_PHRASE: &str = "already has an active response";

/// The GA error code for a cancel that found nothing to cancel.
///
/// External contract — ARGO `openai_live.rs:139`.
pub const CANCEL_NOT_ACTIVE_CODE: &str = "response_cancel_not_active";

/// The two halves of the benign-cancel message, both of which must be present.
///
/// External contract — ARGO `openai_live.rs:143-144`. Requiring both is what
/// keeps an unrelated cancel failure visible: `"cancel"` alone matches
/// `"cancelled by the user"`, and `"no active response"` alone matches
/// `"no active response stream"`.
pub const CANCEL_RACE_PHRASES: [&str; 2] = ["cancel", "no active response"];

/// Read `error.<field>`, falling back to a relay's flattened `<field>`.
///
/// Both nestings occur: OpenAI wraps the detail in `error`, some relays flatten
/// it onto the frame.
fn error_field<'a>(event: &'a Value, field: &str) -> &'a str {
    event
        .get("error")
        .and_then(|error| error.get(field))
        .or_else(|| event.get(field))
        .and_then(Value::as_str)
        .unwrap_or_default()
}

/// Whether an `error` frame is just "your cancel arrived too late".
///
/// External contract — ARGO `openai_live.rs:136-150`. Matched on the code first
/// — `response_cancel_not_active` is the precise signal — and on the message
/// only as a fallback, because a relay may forward the text while dropping the
/// structured fields.
///
/// The bug this exists for: with server VAD the user speaks **before** the model
/// does, so a host wiring barge-in to speech-start cancels an idle session and
/// puts `Cancellation failed: no active response found` on screen for every
/// first utterance.
///
/// In VIA this is not a swallow but a classification: it becomes
/// [`ErrorClass::NoActiveResponse`](via_realtime::ErrorClass::NoActiveResponse),
/// which `via-realtime` already suppresses.
#[must_use]
pub fn is_benign_cancel_race(code: &str, message: &str) -> bool {
    if code.eq_ignore_ascii_case(CANCEL_NOT_ACTIVE_CODE) {
        return true;
    }
    let lowered = message.to_ascii_lowercase();
    CANCEL_RACE_PHRASES
        .iter()
        .all(|phrase| lowered.contains(phrase))
}

/// Whether an `error` frame is the "already has an active response" conflict.
///
/// External contract — ARGO `openai_live.rs:159-190`. Takes the whole frame
/// because both nestings occur. Matched on the GA code first, and on the
/// invariant phrase of the message as the relay fallback.
///
/// The caller decides what to do with a match: this conflict is only *noise*
/// when no client `response.create` is unaccounted for — see
/// [`CreateAccounting::conflict_is_ours`].
#[must_use]
pub fn is_active_response_conflict(event: &Value) -> bool {
    if event.get("type").and_then(Value::as_str) != Some("error") {
        return false;
    }
    if error_field(event, "code").eq_ignore_ascii_case(ACTIVE_RESPONSE_CONFLICT_CODE) {
        return true;
    }
    error_field(event, "message")
        .to_ascii_lowercase()
        .contains(ACTIVE_RESPONSE_CONFLICT_PHRASE)
}

/// `response.create`s the client sent and the provider has not yet answered.
///
/// Shared between the transport's write half (which counts them out) and its
/// read half (which counts them back and decides what a conflict means), so it
/// is an `Arc` over an atomic rather than a field on either.
#[derive(Debug, Clone, Default)]
pub struct CreateAccounting {
    in_flight: Arc<AtomicUsize>,
}

impl CreateAccounting {
    /// A fresh, empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many client creates are unaccounted for right now.
    #[must_use]
    pub fn in_flight(&self) -> usize {
        self.in_flight.load(Ordering::SeqCst)
    }

    /// Record a client `response.create` going out.
    pub fn sent(&self) {
        self.in_flight.fetch_add(1, Ordering::SeqCst);
    }

    /// Record a `response.created` coming back.
    ///
    /// **Saturating.** Server VAD opens responses no client asked for, so this
    /// is reached far more often than [`sent`](Self::sent) — an unsaturated
    /// counter would go negative on the first server turn and never recover,
    /// which would make every later conflict look like ours.
    pub fn answered(&self) {
        let _ = self
            .in_flight
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1));
    }

    /// Whether an active-response conflict *could* concern a create of ours.
    ///
    /// `true` consumes one unaccounted create, so a burst of two conflicts
    /// against two of our own creates surfaces twice and then stops. `false`
    /// means the ledger is empty: the client sent nothing that could have caused
    /// this, so the relay did it to itself and the frame is noise.
    #[must_use]
    pub fn conflict_is_ours(&self) -> bool {
        self.in_flight
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn the_cancel_race_matcher_needs_both_halves() {
        // "cancel" alone or "no active response" alone is not the signal;
        // requiring both keeps an unrelated cancel failure visible.
        assert!(!is_benign_cancel_race("", "cancelled by the user"));
        assert!(!is_benign_cancel_race(
            "",
            "no active response stream is open"
        ));
        assert!(is_benign_cancel_race(
            "",
            "Cancellation failed: no active response found"
        ));
    }

    #[test]
    fn the_cancel_race_matcher_takes_the_code_first_and_case_insensitively() {
        for (code, message) in [
            (
                "response_cancel_not_active",
                "Cancellation failed: no active response found",
            ),
            ("RESPONSE_CANCEL_NOT_ACTIVE", "some relay rewording"),
            ("", "cancel failed — NO ACTIVE RESPONSE to cancel"),
        ] {
            assert!(
                is_benign_cancel_race(code, message),
                "code={code:?} message={message:?}"
            );
        }
    }

    #[test]
    fn the_conflict_matches_the_code_and_both_nestings_of_the_message() {
        // The GA code, nested under `error`.
        assert!(is_active_response_conflict(&json!({
            "type": "error",
            "error": { "code": "conversation_already_has_active_response", "message": "x" }
        })));
        // The message as observed through the litellm gateway, nested.
        assert!(is_active_response_conflict(&json!({
            "type": "error",
            "error": { "message": "Conversation already has an active response in progress: \
                       resp_x. Wait until the response is finished before creating a new one." }
        })));
        // A relay that flattens the detail onto the frame.
        assert!(is_active_response_conflict(&json!({
            "type": "error",
            "message": "Conversation already has an active response in progress: resp_x."
        })));
        // A relay that flattens only the code.
        assert!(is_active_response_conflict(&json!({
            "type": "error",
            "code": "CONVERSATION_ALREADY_HAS_ACTIVE_RESPONSE"
        })));
    }

    #[test]
    fn the_conflict_leaves_other_frames_alone() {
        // A different provider complaint must keep surfacing.
        assert!(!is_active_response_conflict(&json!({
            "type": "error",
            "error": { "code": "rate_limit_exceeded", "message": "slow down" }
        })));
        // Only `error` frames qualify, whatever the body says.
        assert!(!is_active_response_conflict(&json!({
            "type": "response.created",
            "message": "already has an active response"
        })));
        assert!(!is_active_response_conflict(&json!({ "type": "error" })));
        assert!(!is_active_response_conflict(&Value::Null));
    }

    #[test]
    fn an_empty_ledger_says_a_conflict_is_not_ours() {
        let ledger = CreateAccounting::new();
        assert_eq!(ledger.in_flight(), 0);
        assert!(!ledger.conflict_is_ours());
        // And it stays at zero — the saturating decrement is what keeps a
        // server-VAD session from driving the count negative.
        assert_eq!(ledger.in_flight(), 0);
    }

    #[test]
    fn a_conflict_against_our_own_create_surfaces_once_and_consumes_its_slot() {
        let ledger = CreateAccounting::new();
        ledger.sent();
        assert!(ledger.conflict_is_ours(), "the client did send one");
        assert!(
            !ledger.conflict_is_ours(),
            "the second conflict has nothing left to blame"
        );
    }

    #[test]
    fn server_vad_responses_never_drive_the_ledger_negative() {
        let ledger = CreateAccounting::new();
        // The bring-up shape: the server opens responses nobody asked for.
        for _ in 0..5 {
            ledger.answered();
        }
        assert_eq!(ledger.in_flight(), 0);
        // A create of ours is still accounted for afterwards.
        ledger.sent();
        assert_eq!(ledger.in_flight(), 1);
        assert!(ledger.conflict_is_ours());
    }

    #[test]
    fn a_created_answers_exactly_one_outstanding_create() {
        let ledger = CreateAccounting::new();
        ledger.sent();
        ledger.sent();
        assert_eq!(ledger.in_flight(), 2);
        ledger.answered();
        assert_eq!(ledger.in_flight(), 1);
        ledger.answered();
        assert_eq!(ledger.in_flight(), 0);
        assert!(!ledger.conflict_is_ours());
    }

    #[test]
    fn the_ledger_is_shared_by_its_clones() {
        // The write half and the read half hold clones; they must see one count.
        let write = CreateAccounting::new();
        let read = write.clone();
        write.sent();
        assert_eq!(read.in_flight(), 1);
        read.answered();
        assert_eq!(write.in_flight(), 0);
    }
}
