//! The two error corpora, and nothing else.
//!
//! Ported from `server/src/voice/providers/dashscope.mjs:16-29` and
//! `server/src/voice/providers/s2s.mjs:14-23`. Both are catalogued verbatim —
//! `DashScope classifyError regexes` and `s2s classifyError regexes` — because
//! *classification is string matching against a service's error prose*, and the
//! patterns are the whole of the contract with that prose.
//!
//! The vocabulary the patterns map onto is [`via_realtime::ErrorClass`]; this
//! module never invents a class name.
//!
//! # What the input is
//!
//! `event.error.message || event.message`, **not** the composed sentence —
//! [`via_realtime::realtime_event_classification_text`]. A provider's corpus
//! therefore sees the provider's own words, without the error code prefixed,
//! which is why `'InvalidApiKey: Invalid API-key provided.'` and
//! `'Invalid API-key provided.'` must both classify as
//! [`Fatal`](ErrorClass::Fatal).
//!
//! # Case folding, exactly
//!
//! Every upstream pattern carries JavaScript's `i` flag with **no** `u` flag,
//! which is ASCII case folding: `/k/i` does not match U+212A KELVIN SIGN there,
//! but Rust's Unicode-aware `(?i)` does. So the haystack is lowercased with
//! [`str::to_ascii_lowercase`] and the patterns are written lowercase, which is
//! the exact equivalent rather than an approximation — the same reasoning
//! [`via_realtime::is_recoverable_realtime_inactivity_error`] records. `\b`
//! is written `(?-u:\b)` for the same reason: JavaScript's non-`u` word
//! boundary is `[A-Za-z0-9_]`.
//!
//! # Why `regex` here and not in `via-realtime`
//!
//! `via-realtime` writes its one pattern out by hand and says why: it is a
//! literal, a `\d+` and a literal, and a `LazyLock` whose constructor can fail
//! would be worse. This corpus is not that. It is a word boundary, three
//! optional-separator classes, four alternation groups and a `.*` span, and
//! transcribing those into string scans is exactly how an expired credential
//! quietly stops being recognised as `fatal`.
//!
//! The fallible construction is handled rather than hidden: the sets are
//! `Option<RegexSet>` and a `None` degrades classification to
//! [`Other`](ErrorClass::Other) — the class that is *shown to the user* — so the
//! failure mode is a visible auth error rather than a silently suppressed one.
//! [`patterns_compile`] is the assertion that it never happens.

use std::sync::LazyLock;

use regex::RegexSet;
use via_realtime::{ErrorClass, is_recoverable_realtime_inactivity_error};

/// The DashScope `input_busy` literal.
///
/// External contract — `dashscope.mjs:18`, `/user is speaking/i`. A literal with
/// no metacharacter, so a lowercased `contains` is the pattern.
pub const DASHSCOPE_INPUT_BUSY: &str = "user is speaking";

/// The `no_active_response` literal, shared by both providers.
///
/// External contract — `dashscope.mjs:19`, `s2s.mjs:21`,
/// `/no active response/i`.
pub const NO_ACTIVE_RESPONSE: &str = "no active response";

/// The speech-to-speech `response_slot_busy` literal.
///
/// External contract — `s2s.mjs:20`, `/another response is in progress/i`. The
/// single per-session response slot refuses a concurrent `response.create`, and
/// the session retries it transparently when the provider declares
/// `single_response_slot`.
pub const SPEECH_TO_SPEECH_RESPONSE_SLOT_BUSY: &str = "another response is in progress";

/// The four DashScope `fatal` patterns, in upstream's declaration order.
///
/// External contract — `dashscope.mjs:20-27`. Lowercased, and `\b` written
/// ASCII-only; see the module docs for why that is an exact transcription
/// rather than a change.
///
/// | # | Covers |
/// | --- | --- |
/// | 0 | credentials — a bad key, a failed handshake, a 401/403 upgrade |
/// | 1 | billing — an account in arrears |
/// | 2 | quota — the free tier exhausted |
/// | 3 | model access — denied, or no such model |
pub const DASHSCOPE_FATAL_PATTERNS: [&str; 4] = [
    r"invalid[_ -]?api[_ -]?key|incorrect api key|authentication failed|unauthorized|unexpected server response: (?:401|403)",
    r"(?-u:\b)arrearage(?-u:\b)|account is not in good standing",
    r"allocationquota\.freetieronly|free allocated quota exceeded|free tier .* exhausted",
    r"model(?:\.|_)?accessdenied|model[_ -]?not[_ -]?found",
];

/// The speech-to-speech `capacity_busy` pattern.
///
/// External contract — `s2s.mjs:15-17`, test-locked upstream against the
/// literal service sentence
/// *"All 1 session slots are in use. Disconnect an existing client first."*
pub const SPEECH_TO_SPEECH_CAPACITY_PATTERN: &str =
    r"session_limit_reached|session slots? (?:are|is) in use";

/// The compiled DashScope `fatal` corpus. See the module docs for the `Option`.
static DASHSCOPE_FATAL: LazyLock<Option<RegexSet>> =
    LazyLock::new(|| RegexSet::new(DASHSCOPE_FATAL_PATTERNS).ok());

/// The compiled speech-to-speech `capacity_busy` pattern.
static SPEECH_TO_SPEECH_CAPACITY: LazyLock<Option<RegexSet>> =
    LazyLock::new(|| RegexSet::new([SPEECH_TO_SPEECH_CAPACITY_PATTERN]).ok());

/// Whether every pattern in this module compiled.
///
/// Exists so the guarantee the module docs make is checkable from outside —
/// `tests/contracts.rs` asserts it, and so does `via-conformance`.
#[must_use]
pub fn patterns_compile() -> bool {
    DASHSCOPE_FATAL.is_some() && SPEECH_TO_SPEECH_CAPACITY.is_some()
}

fn matches(set: &LazyLock<Option<RegexSet>>, lowercased: &str) -> bool {
    set.as_ref().is_some_and(|set| set.is_match(lowercased))
}

/// Classify one DashScope error string.
///
/// External contract — `dashscope.mjs:16-29`. The order is upstream's and it is
/// load-bearing: the inactivity closure is checked **first**, because
/// *"Your session was closed because no response was generated for 180
/// seconds."* also contains the word `response` and must never be shown to
/// anybody.
///
/// ```
/// use via_realtime::ErrorClass;
/// use via_realtime_dashscope::classify_dashscope_error as classify;
///
/// assert_eq!(classify("InvalidApiKey: Invalid API-key provided."), ErrorClass::Fatal);
/// assert_eq!(classify("Unexpected server response: 401"), ErrorClass::Fatal);
/// assert_eq!(
///     classify("Cannot create response while user is speaking."),
///     ErrorClass::InputBusy,
/// );
/// // A plain rate limit is *not* fatal: retrying it is the right thing.
/// assert_eq!(
///     classify("You exceeded your current quota, please check your plan."),
///     ErrorClass::Other,
/// );
/// ```
#[must_use]
pub fn classify_dashscope_error(message: &str) -> ErrorClass {
    if is_recoverable_realtime_inactivity_error(message) {
        return ErrorClass::Inactivity;
    }
    let text = message.to_ascii_lowercase();
    if text.contains(DASHSCOPE_INPUT_BUSY) {
        return ErrorClass::InputBusy;
    }
    if text.contains(NO_ACTIVE_RESPONSE) {
        return ErrorClass::NoActiveResponse;
    }
    if matches(&DASHSCOPE_FATAL, &text) {
        return ErrorClass::Fatal;
    }
    ErrorClass::Other
}

/// Classify one speech-to-speech error string.
///
/// External contract — `s2s.mjs:14-23`. Note what is *absent*: this corpus has
/// no `fatal` and no `inactivity` arm. A user-run local service has no
/// credentials to expire and no idle-session policy, and inventing either here
/// would suppress an error the operator needs to see.
///
/// ```
/// use via_realtime::ErrorClass;
/// use via_realtime_dashscope::classify_speech_to_speech_error as classify;
///
/// assert_eq!(
///     classify("All 1 session slots are in use. Disconnect an existing client first."),
///     ErrorClass::CapacityBusy,
/// );
/// assert_eq!(classify("session_limit_reached"), ErrorClass::CapacityBusy);
/// assert_eq!(
///     classify("Another response is in progress."),
///     ErrorClass::ResponseSlotBusy,
/// );
/// ```
#[must_use]
pub fn classify_speech_to_speech_error(message: &str) -> ErrorClass {
    let text = message.to_ascii_lowercase();
    if matches(&SPEECH_TO_SPEECH_CAPACITY, &text) {
        return ErrorClass::CapacityBusy;
    }
    if text.contains(SPEECH_TO_SPEECH_RESPONSE_SLOT_BUSY) {
        return ErrorClass::ResponseSlotBusy;
    }
    if text.contains(NO_ACTIVE_RESPONSE) {
        return ErrorClass::NoActiveResponse;
    }
    ErrorClass::Other
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[test]
    fn every_pattern_compiles() {
        assert!(patterns_compile());
        assert!(DASHSCOPE_FATAL.is_some());
        assert!(SPEECH_TO_SPEECH_CAPACITY.is_some());
    }

    #[rstest]
    // The five upstream test-locks, `realtime-provider.test.mjs:106-121`.
    #[case("InvalidApiKey: Invalid API-key provided.", ErrorClass::Fatal)]
    #[case(
        "Arrearage: Access denied, please make sure your account is in good standing.",
        ErrorClass::Fatal
    )]
    #[case(
        "AllocationQuota.FreeTierOnly: The free tier of the model has been exhausted.",
        ErrorClass::Fatal
    )]
    #[case("Free allocated quota exceeded.", ErrorClass::Fatal)]
    #[case("Unexpected server response: 401", ErrorClass::Fatal)]
    #[case(
        "You exceeded your current quota, please check your plan.",
        ErrorClass::Other
    )]
    fn the_upstream_account_corpus_classifies_as_it_did(
        #[case] message: &str,
        #[case] expected: ErrorClass,
    ) {
        assert_eq!(classify_dashscope_error(message), expected, "{message}");
    }

    #[rstest]
    // Every separator spelling the `[_ -]?` classes admit.
    #[case("invalid_api_key")]
    #[case("invalid-api-key")]
    #[case("invalid api key")]
    #[case("invalidapikey")]
    #[case("INVALID API-KEY")]
    #[case("incorrect api key")]
    #[case("Authentication failed")]
    #[case("Unauthorized")]
    #[case("Unexpected server response: 403")]
    #[case("Model.AccessDenied")]
    #[case("model_accessdenied")]
    #[case("ModelAccessDenied")]
    #[case("model-not-found")]
    #[case("model_not_found")]
    #[case("modelnotfound")]
    #[case("Free tier for this model is exhausted")]
    fn each_fatal_spelling_is_recognised(#[case] message: &str) {
        assert_eq!(
            classify_dashscope_error(message),
            ErrorClass::Fatal,
            "{message}"
        );
    }

    #[rstest]
    // A 500 is a transient server fault, not a credential problem.
    #[case("Unexpected server response: 500")]
    // `arrearage` is a whole word upstream; a substring must not trip it.
    #[case("no arrearages recorded")]
    // The `.*` span does not cross a line break, which is JavaScript's `.` too.
    #[case("free tier is large\nquota exhausted")]
    // Nothing about the model, so pattern 3 must not reach across the sentence.
    #[case("the found model was not the requested one")]
    fn a_near_miss_is_never_called_fatal(#[case] message: &str) {
        assert_eq!(
            classify_dashscope_error(message),
            ErrorClass::Other,
            "{message}"
        );
    }

    #[test]
    fn the_word_boundary_is_a_word_boundary() {
        assert_eq!(classify_dashscope_error("Arrearage"), ErrorClass::Fatal);
        assert_eq!(
            classify_dashscope_error("account arrearage detected"),
            ErrorClass::Fatal
        );
        assert_eq!(classify_dashscope_error("prearrearages"), ErrorClass::Other);
    }

    #[test]
    fn the_inactivity_closure_outranks_every_other_arm() {
        // The sentence contains `response`, and a corpus that checked
        // `no active response` first would still not match it — but a corpus
        // that checked `fatal` first would, if the service ever prefixed it
        // with a code. Order is the guarantee.
        let closure = "Your session was closed because no response was generated for 180 seconds.";
        assert_eq!(classify_dashscope_error(closure), ErrorClass::Inactivity);
        assert!(ErrorClass::Inactivity.is_suppressed());
    }

    #[test]
    fn input_busy_outranks_no_active_response() {
        // Both literals can appear in one sentence; upstream checks speaking
        // first, and the two lead to different recoveries.
        assert_eq!(
            classify_dashscope_error("no active response: user is speaking"),
            ErrorClass::InputBusy
        );
    }

    #[rstest]
    #[case(
        "All 1 session slots are in use. Disconnect an existing client first.",
        ErrorClass::CapacityBusy
    )]
    #[case("session_limit_reached", ErrorClass::CapacityBusy)]
    #[case("The session slot is in use", ErrorClass::CapacityBusy)]
    #[case("Another response is in progress.", ErrorClass::ResponseSlotBusy)]
    #[case("No active response to cancel", ErrorClass::NoActiveResponse)]
    #[case("something else entirely", ErrorClass::Other)]
    // The DashScope corpus does not leak into this one: a local service has no
    // credential to be invalid.
    #[case("Invalid API-key provided.", ErrorClass::Other)]
    #[case(
        "Your session was closed because no response was generated for 180 seconds.",
        ErrorClass::Other
    )]
    fn the_speech_to_speech_corpus_is_exactly_three_arms(
        #[case] message: &str,
        #[case] expected: ErrorClass,
    ) {
        assert_eq!(
            classify_speech_to_speech_error(message),
            expected,
            "{message}"
        );
    }

    #[test]
    fn an_empty_message_is_other_for_both_providers() {
        assert_eq!(classify_dashscope_error(""), ErrorClass::Other);
        assert_eq!(classify_speech_to_speech_error(""), ErrorClass::Other);
    }

    #[test]
    fn classification_never_folds_non_ascii_case() {
        // U+212A KELVIN SIGN case-folds to ASCII `k` under Unicode rules, so a
        // Rust `(?i)key` would match this and JavaScript's non-`u` `/key/i`
        // would not. The ASCII lowercasing is what keeps the two agreeing.
        assert_eq!(
            classify_dashscope_error("invalid api \u{212A}ey"),
            ErrorClass::Other
        );
        // …while the ASCII spelling of the same sentence is still fatal.
        assert_eq!(
            classify_dashscope_error("invalid api Key"),
            ErrorClass::Fatal
        );
    }
}
