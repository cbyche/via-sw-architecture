//! This provider's error corpus, and nothing else.
//!
//! The vocabulary the corpus maps onto is [`via_realtime::ErrorClass`] —
//! catalogued as `provider error classification vocabulary`, seven strings the
//! Gateway branches on. Nothing here invents a class name.
//!
//! # What the input is
//!
//! `event.error.message || event.message`, **not** the composed sentence —
//! [`via_realtime::realtime_event_classification_text`]. A provider's corpus
//! therefore sees the provider's own words, without the error code prefixed.
//! The one exception is a connect failure: this crate's candidate walk composes
//! its own aggregate (`connect.rs`), and that text is what reaches
//! `classify_error` when a whole walk fails — which is why the HTTP-status
//! spellings below cover both `tungstenite`'s wording and this crate's.
//!
//! # Why literals and not a regex
//!
//! Every pattern here is a literal substring of a lowercased haystack. There is
//! no alternation group, no word boundary and no `.*` span — the two matchers
//! that *are* structured ([`is_benign_cancel_race`] and its sibling matcher)
//! live in [`crate::conflict`] and are hand-written there for the same reason
//! `via-realtime` writes its one pattern by hand: a `LazyLock<Regex>` whose
//! constructor can fail inside a classification path can only degrade to a class
//! that is *shown to the user*, and a corpus that needs no regex should not
//! carry one.
//!
//! # Case folding
//!
//! The haystack is lowercased with [`str::to_ascii_lowercase`] and every pattern
//! is written lowercase — the exact equivalent of JavaScript's non-`u` `i` flag,
//! which is the folding upstream's own corpora use.
//!
//! # What is deliberately absent
//!
//! - **`input_busy`.** DashScope refuses a response with `user is speaking`;
//!   neither OpenAI Realtime nor Azure produces that sentence, and inventing a
//!   pattern for it would classify some unrelated message as a retryable busy.
//! - **`capacity_busy`.** A session-slot limit is the huggingface/speech-to-speech
//!   service's concept, not this one's.

use via_realtime::{ErrorClass, is_recoverable_realtime_inactivity_error};

use crate::conflict::{
    ACTIVE_RESPONSE_CONFLICT_CODE, ACTIVE_RESPONSE_CONFLICT_PHRASE, is_benign_cancel_race,
};

/// Credential failures.
///
/// `incorrect api key provided` is OpenAI's own sentence and is quoted in ARGO
/// bring-up §5 as the answer a missing endpoint produced from a host nobody
/// configured. The two `unexpected server response:` spellings are
/// `tungstenite`'s rendering of a rejected upgrade; the two `http ` spellings
/// are [`crate::connect::describe_ws_error`]'s, which is what a failed candidate
/// walk carries.
pub const CREDENTIAL_PATTERNS: [&str; 9] = [
    "incorrect api key",
    "invalid api key",
    "invalid_api_key",
    "invalid api-key",
    "authentication failed",
    "unauthorized",
    "unexpected server response: 401",
    "unexpected server response: 403",
    "http 401",
];

/// Authorization and network-policy refusals.
///
/// The two `x-ms-error-code` values are Azure's, and they arrive **in a header**
/// rather than in a body — which is exactly why `describe_ws_error` names that
/// header explicitly. Without them a `403` with an empty body is
/// indistinguishable from a disabled local-auth policy, a network ACL, a gateway
/// that refuses upgrades, and a key without access to the model.
pub const AUTHORIZATION_PATTERNS: [&str; 4] = [
    "http 403",
    "authorizationfailed",
    "publicnetworkaccessdisabled",
    "access denied",
];

/// Quota and billing.
///
/// The first three are OpenAI's and Azure's own wording. The last two are
/// catalogued (`realtime provider fatal error classification`) as DashScope's,
/// and they are here for one specific reason: **a gateway relays its upstream's
/// prose verbatim**. litellm is a multi-upstream proxy, and the wire protocol at
/// that host is OpenAI's whatever it forwards to — so a session reaching
/// DashScope through it is configured as this provider and receives DashScope's
/// sentences. Retrying cannot fix either of them.
pub const QUOTA_PATTERNS: [&str; 5] = [
    "insufficient_quota",
    "exceeded your current quota",
    "billing hard limit",
    "free allocated quota exceeded",
    "free tier of the model has been exhausted",
];

/// Model and deployment access.
///
/// `deploymentnotfound` is Azure's: a resource that has the model but not a
/// deployment by that name. On Azure the deployment name is arbitrary, so this
/// is the ordinary spelling of "you named the wrong thing", and it is fatal
/// because retrying cannot fix it.
pub const MODEL_ACCESS_PATTERNS: [&str; 5] = [
    "model_not_found",
    "model not found",
    "deploymentnotfound",
    "does not exist or you do not have access",
    "do not have access to model",
];

/// Every `fatal` pattern, in the order they are checked.
///
/// Order is not semantically load-bearing — the classes are the same — but it is
/// the order the four tables above document, and a reader tracing a message
/// through the corpus follows it.
#[must_use]
pub fn fatal_patterns() -> Vec<&'static str> {
    CREDENTIAL_PATTERNS
        .iter()
        .chain(AUTHORIZATION_PATTERNS.iter())
        .chain(QUOTA_PATTERNS.iter())
        .chain(MODEL_ACCESS_PATTERNS.iter())
        .copied()
        .collect()
}

/// Classify one of this provider's error strings.
///
/// The order is the contract:
///
/// 1. the shared inactivity closure — `via-realtime` owns the sentence and the
///    reconnect that follows it;
/// 2. the benign cancel race, so a barge-in that worked does not put an error on
///    screen (ARGO bring-up §2: *"`response.cancel` on every speech-start → an
///    error on every first utterance"*);
/// 3. the single-response-slot conflict, which this provider declares
///    [`single_response_slot`](via_realtime::ProviderCapabilities::single_response_slot)
///    for and `via-realtime` therefore retries on its bounded busy ladder;
/// 4. the `fatal` corpus;
/// 5. everything else, which is shown.
#[must_use]
pub fn classify_openai_error(message: &str) -> ErrorClass {
    if is_recoverable_realtime_inactivity_error(message) {
        return ErrorClass::Inactivity;
    }
    if is_benign_cancel_race("", message) {
        return ErrorClass::NoActiveResponse;
    }
    let lowered = message.to_ascii_lowercase();
    if lowered.contains(ACTIVE_RESPONSE_CONFLICT_PHRASE)
        || lowered.contains(ACTIVE_RESPONSE_CONFLICT_CODE)
    {
        return ErrorClass::ResponseSlotBusy;
    }
    if fatal_patterns()
        .iter()
        .any(|pattern| lowered.contains(pattern))
    {
        return ErrorClass::Fatal;
    }
    ErrorClass::Other
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn the_inactivity_closure_is_the_shared_one() {
        assert_eq!(
            classify_openai_error(
                "session was closed because no response was generated for 120 seconds"
            ),
            ErrorClass::Inactivity
        );
    }

    #[test]
    fn a_cancel_with_nothing_to_cancel_is_never_shown() {
        // The bug this pins: with server VAD the user speaks BEFORE the model
        // does, so a host wiring barge-in to speech-start cancelled an idle
        // session and put this on screen for every first utterance.
        assert_eq!(
            classify_openai_error("Cancellation failed: no active response found"),
            ErrorClass::NoActiveResponse
        );
        assert!(ErrorClass::NoActiveResponse.is_suppressed());
    }

    #[test]
    fn the_active_response_conflict_is_a_busy_slot_not_a_failure() {
        for message in [
            "Conversation already has an active response in progress: resp_x.",
            "conversation_already_has_active_response",
        ] {
            assert_eq!(
                classify_openai_error(message),
                ErrorClass::ResponseSlotBusy,
                "{message}"
            );
        }
        // Not suppressed: the session retries it, and a retry that keeps failing
        // has to be able to surface.
        assert!(!ErrorClass::ResponseSlotBusy.is_suppressed());
    }

    #[test]
    fn every_fatal_pattern_classifies_as_fatal_in_any_casing() {
        for pattern in fatal_patterns() {
            assert_eq!(
                classify_openai_error(pattern),
                ErrorClass::Fatal,
                "{pattern}"
            );
            assert_eq!(
                classify_openai_error(&pattern.to_ascii_uppercase()),
                ErrorClass::Fatal,
                "{pattern} uppercased"
            );
        }
    }

    #[test]
    fn the_sentences_these_endpoints_actually_send_classify_as_fatal() {
        for message in [
            // OpenAI, quoted in the upstream ARGO bring-up §5.
            "Incorrect API key provided: sk-***. You can find your API key at \
             https://platform.openai.com/account/api-keys.",
            // tungstenite's rendering of a refused upgrade.
            "Unexpected server response: 401",
            // This crate's own aggregate, one candidate.
            "no realtime endpoint completed a session (1 attempt(s)): \
             wss://r.openai.azure.com/openai/v1/realtime?model=d → HTTP 403 \
             [PublicNetworkAccessDisabled] (empty body)",
            // Azure, deployment name wrong.
            "The API deployment for this resource does not exist. DeploymentNotFound",
            // OpenAI, model access.
            "The model `gpt-realtime-2.1` does not exist or you do not have access to it.",
            // Quota.
            "You exceeded your current quota, please check your plan and billing details.",
        ] {
            assert_eq!(
                classify_openai_error(message),
                ErrorClass::Fatal,
                "{message}"
            );
        }
    }

    #[test]
    fn an_unrelated_failure_stays_visible() {
        // The classification must stay narrow — a message hidden behind a
        // suppressed class is a silent session.
        for message in [
            "rate limit exceeded",
            "Invalid value: 'session.voice'.",
            "Unknown parameter: 'session.type'.",
            "cancelled by the user",
            "no active response stream is open",
            "the server is restarting",
            "",
        ] {
            assert_eq!(
                classify_openai_error(message),
                ErrorClass::Other,
                "{message}"
            );
        }
        assert!(!ErrorClass::Other.is_suppressed());
    }

    #[test]
    fn a_404_or_500_is_not_fatal_because_the_next_candidate_may_answer() {
        // Only 401/403 are credential- or policy-shaped. A 404 is a wrong route,
        // which the candidate walk exists to move past, and a 5xx is transient.
        for message in [
            "Unexpected server response: 404",
            "HTTP 404: not found",
            "HTTP 500: upstream unavailable",
        ] {
            assert_eq!(
                classify_openai_error(message),
                ErrorClass::Other,
                "{message}"
            );
        }
    }

    #[test]
    fn no_fatal_pattern_is_a_substring_of_another() {
        // Two patterns where one contains the other would make the shorter one
        // dead weight and hide a drift in the longer.
        let patterns = fatal_patterns();
        for (index, outer) in patterns.iter().enumerate() {
            for (other, inner) in patterns.iter().enumerate() {
                if index != other {
                    assert!(!outer.contains(inner), "`{outer}` contains `{inner}`");
                }
            }
        }
    }

    #[test]
    fn every_pattern_is_lowercase_so_the_folding_is_exact() {
        for pattern in fatal_patterns() {
            assert_eq!(pattern, pattern.to_ascii_lowercase(), "{pattern}");
        }
    }
}
