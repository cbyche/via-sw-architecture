//! Reading a provider's `error` event.
//!
//! Two upstream files land here because they answer the same question from two
//! sides: `realtime-provider.mjs:46-59` composes the sentence a person sees, and
//! `realtime-errors.mjs` recognises the one closure that must not be shown to
//! anybody at all.
//!
//! The composed sentence is load-bearing twice over — the catalogue is explicit
//! that *"this composed string is both what `classifyError` matches on and what
//! is sent to the client as `{ type: 'error', message }`"* — so the composition
//! is reproduced exactly rather than approximated.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use via_i18n::{Locale, keys, t};

/// How a provider classifies one of its own error strings.
///
/// External contract — `error-code` / *provider error classification
/// vocabulary*: the Gateway branches on these exact names, so they are a closed
/// set shared by every provider rather than free-form strings.
///
/// | Variant | What the Gateway does with it |
/// | --- | --- |
/// | [`Inactivity`](Self::Inactivity) | suppresses the user-facing error and reconnects |
/// | [`InputBusy`](Self::InputBusy) | retries a `model`-origin response transparently |
/// | [`NoActiveResponse`](Self::NoActiveResponse) | swallowed silently |
/// | [`Fatal`](Self::Fatal) | suppresses the error text **and blocks the connection** |
/// | [`CapacityBusy`](Self::CapacityBusy) | swallowed silently; the caller backs off |
/// | [`ResponseSlotBusy`](Self::ResponseSlotBusy) | retried when the provider declares one response slot |
/// | [`Other`](Self::Other) | surfaced to the user |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    /// The provider closed an idle session. Recoverable; never shown.
    Inactivity,
    /// A response was refused because the user is still speaking.
    InputBusy,
    /// A `response.cancel` arrived with nothing to cancel.
    NoActiveResponse,
    /// Credentials, quota, billing or model access. Not recoverable.
    Fatal,
    /// Every session slot on the service is in use.
    CapacityBusy,
    /// The single per-session response slot is occupied.
    ResponseSlotBusy,
    /// Anything else.
    Other,
}

impl ErrorClass {
    /// The wire string the Gateway branches on.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inactivity => "inactivity",
            Self::InputBusy => "input_busy",
            Self::NoActiveResponse => "no_active_response",
            Self::Fatal => "fatal",
            Self::CapacityBusy => "capacity_busy",
            Self::ResponseSlotBusy => "response_slot_busy",
            Self::Other => "other",
        }
    }

    /// Whether this class must never reach the user as an error message.
    ///
    /// `server/src/voice/realtime-gateway.mjs:1353-1437`: `capacity_busy` and
    /// `no_active_response` are swallowed silently, and `inactivity` and `fatal`
    /// suppress the user-facing error (`fatal` additionally blocks the
    /// connection).
    #[must_use]
    pub const fn is_suppressed(self) -> bool {
        matches!(
            self,
            Self::Inactivity | Self::Fatal | Self::CapacityBusy | Self::NoActiveResponse
        )
    }
}

impl core::fmt::Display for ErrorClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The provider inactivity closure, recognised so it is never shown.
///
/// External contract — `realtime-errors.mjs:1-6`, test-locked against the
/// literal upstream text at `server/test/realtime-errors.test.mjs:5-31`. It
/// matches with or without the trailing period, and it must **not** match
/// `Cannot create response while user is speaking.` or `Authentication failed`:
/// over-matching hides a real auth failure, under-matching spams reconnects.
///
/// ```
/// use via_realtime::is_recoverable_realtime_inactivity_error;
///
/// assert!(is_recoverable_realtime_inactivity_error(
///     "Your session was closed because no response was generated for 180 seconds."
/// ));
/// assert!(!is_recoverable_realtime_inactivity_error("Authentication failed"));
/// ```
/// # Why this is not a `Regex`
///
/// The pattern is one ASCII literal, one `\d+` and one more ASCII literal. A
/// compiled regex would need a `LazyLock` whose constructor can fail, and the
/// only thing a failed construction could do inside an error-classification path
/// is panic or silently stop recognising the closure — both worse than the ten
/// lines below. JavaScript's `\d` is ASCII-only and its `i` flag on an ASCII
/// pattern is ASCII case folding, so `to_ascii_lowercase` is the exact
/// equivalent rather than an approximation.
#[must_use]
pub fn is_recoverable_realtime_inactivity_error(message: &str) -> bool {
    const HEAD: &str = "session was closed because no response was generated for ";
    const TAIL: &str = " seconds";

    let text = message.trim().to_ascii_lowercase();
    let mut from = 0usize;
    while let Some(offset) = text[from..].find(HEAD) {
        let after = &text[from + offset + HEAD.len()..];
        let digits = after.chars().take_while(char::is_ascii_digit).count();
        if digits > 0 && after[digits..].starts_with(TAIL) {
            return true;
        }
        from += offset + 1;
    }
    false
}

/// Compose the sentence a provider `error` event means.
///
/// External contract — `json-field` / *realtimeEventErrorMessage composition*
/// (`realtime-provider.mjs:51-59`):
///
/// > Join with `': '` the deduped non-empty trimmed values of
/// > `[event.error.code, event.error.type, event.error.message, event.message]`;
/// > fallback `实时语音服务错误`.
///
/// The fallback is [`via_i18n::keys::REALTIME_SERVICE_ERROR`], whose `zh` column
/// *is* that string.
///
/// De-duplication is what stops a provider that puts the same sentence in
/// `error.message` and `message` from saying it twice.
///
/// ```
/// use serde_json::json;
/// use via_i18n::Locale;
/// use via_realtime::realtime_event_error_message;
///
/// let message = realtime_event_error_message(
///     &json!({
///         "type": "error",
///         "error": {
///             "code": "AllocationQuota.FreeTierOnly",
///             "type": "insufficient_quota",
///             "message": "The free tier of the model has been exhausted."
///         }
///     }),
///     Locale::En,
/// );
/// assert_eq!(
///     message,
///     "AllocationQuota.FreeTierOnly: insufficient_quota: \
///      The free tier of the model has been exhausted."
/// );
/// ```
#[must_use]
pub fn realtime_event_error_message(event: &Value, locale: Locale) -> String {
    realtime_event_error_message_with_fallback(event, t(locale, keys::REALTIME_SERVICE_ERROR))
}

/// [`realtime_event_error_message`] with an explicit fallback.
///
/// Upstream's second parameter, kept because the Gateway supplies its own
/// sentence in one place.
#[must_use]
pub fn realtime_event_error_message_with_fallback(event: &Value, fallback: &str) -> String {
    let error = event.get("error");
    let details = [
        error.and_then(|error| error.get("code")),
        error.and_then(|error| error.get("type")),
        error.and_then(|error| error.get("message")),
        event.get("message"),
    ];

    let mut parts: Vec<String> = Vec::with_capacity(details.len());
    for detail in details {
        let text = detail.map(js_string_or_empty).unwrap_or_default();
        let trimmed = text.trim();
        if trimmed.is_empty() || parts.iter().any(|seen| seen == trimmed) {
            continue;
        }
        parts.push(trimmed.to_owned());
    }

    if parts.is_empty() {
        return fallback.to_owned();
    }
    parts.join(": ")
}

/// The classification input upstream passes to `provider.classifyError`.
///
/// **Not** the composed message: `realtime-provider.mjs:637-639` classifies
/// `event.error?.message || event.message || ''`, so a provider's regex corpus
/// sees the provider's own sentence rather than one with the code prefixed.
#[must_use]
pub fn realtime_event_classification_text(event: &Value) -> String {
    let from_error = event
        .get("error")
        .and_then(|error| error.get("message"))
        .map(js_string_or_empty)
        .unwrap_or_default();
    if !from_error.is_empty() {
        return from_error;
    }
    event
        .get("message")
        .map(js_string_or_empty)
        .unwrap_or_default()
}

/// `String(value || '')`, reproduced.
///
/// The leading `|| ''` is why this is not `Value::to_string()`: a falsy value —
/// `null`, `false`, `0`, `""` — becomes the empty string *before* it is
/// stringified, so it drops out of the composition rather than contributing the
/// word `null`. Everything else follows `String()`: an array joins its elements
/// with `,` and an object is `[object Object]`.
fn js_string_or_empty(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(false) => String::new(),
        Value::Bool(true) => "true".to_owned(),
        Value::Number(number) => {
            if number.as_f64().is_some_and(|value| value == 0.0) {
                String::new()
            } else {
                number.to_string()
            }
        }
        Value::String(text) => text.clone(),
        // An empty array is truthy in JS and stringifies to `""`, so it survives
        // the `|| ''` and is then dropped by the emptiness filter — the same
        // outcome by a different route.
        Value::Array(items) => items
            .iter()
            .map(js_array_element)
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_owned(),
    }
}

/// `Array.prototype.toString` renders `null` and `undefined` elements as the
/// empty string, and everything else as `String(element)` — note the absence of
/// the falsy short-circuit, so a nested `0` really is `"0"` here.
fn js_array_element(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(js_array_element)
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn a_repeated_sentence_is_said_once() {
        let message = realtime_event_error_message_with_fallback(
            &json!({
                "error": { "code": "Busy", "message": "already running" },
                "message": "already running",
            }),
            "fallback",
        );
        assert_eq!(message, "Busy: already running");
    }

    #[test]
    fn every_field_is_trimmed_before_it_is_deduplicated() {
        let message = realtime_event_error_message_with_fallback(
            &json!({
                "error": { "code": "  Busy  ", "type": "\tBusy\n" },
            }),
            "fallback",
        );
        assert_eq!(message, "Busy");
    }

    #[test]
    fn an_event_with_nothing_usable_falls_back() {
        for event in [
            json!({}),
            json!({ "error": {} }),
            json!({ "error": { "code": "", "type": "   ", "message": null }, "message": false }),
            json!({ "error": { "code": 0 } }),
        ] {
            assert_eq!(
                realtime_event_error_message_with_fallback(&event, "fallback"),
                "fallback",
                "{event}"
            );
        }
    }

    #[test]
    fn the_field_order_is_code_then_type_then_message_then_message() {
        let message = realtime_event_error_message_with_fallback(
            &json!({
                "message": "outer",
                "error": { "message": "inner", "type": "kind", "code": "CODE" },
            }),
            "fallback",
        );
        assert_eq!(message, "CODE: kind: inner: outer");
    }

    #[test]
    fn a_non_string_detail_follows_javascript_string_coercion() {
        assert_eq!(
            realtime_event_error_message_with_fallback(
                &json!({ "error": { "code": 401 } }),
                "fallback"
            ),
            "401"
        );
        assert_eq!(
            realtime_event_error_message_with_fallback(
                &json!({ "error": { "code": ["a", "b"] } }),
                "fallback"
            ),
            "a,b"
        );
        assert_eq!(
            realtime_event_error_message_with_fallback(
                &json!({ "error": { "code": { "nested": 1 } } }),
                "fallback"
            ),
            "[object Object]"
        );
    }

    #[test]
    fn classification_reads_the_providers_own_sentence_not_the_composition() {
        let event = json!({
            "error": { "code": "CODE", "message": "Cannot create response while user is speaking." },
        });
        assert_eq!(
            realtime_event_classification_text(&event),
            "Cannot create response while user is speaking."
        );
        assert_eq!(
            realtime_event_error_message_with_fallback(&event, "fallback"),
            "CODE: Cannot create response while user is speaking."
        );
    }

    #[test]
    fn classification_falls_through_to_the_top_level_message() {
        assert_eq!(
            realtime_event_classification_text(&json!({ "message": "top level" })),
            "top level"
        );
        assert_eq!(
            realtime_event_classification_text(
                &json!({ "error": { "message": "" }, "message": "top level" })
            ),
            "top level"
        );
        assert_eq!(realtime_event_classification_text(&json!({})), "");
    }

    #[test]
    fn the_inactivity_closure_is_matched_with_and_without_a_period() {
        assert!(is_recoverable_realtime_inactivity_error(
            "Your session was closed because no response was generated for 180 seconds."
        ));
        assert!(is_recoverable_realtime_inactivity_error(
            "Your session was closed because no response was generated for 300 seconds"
        ));
        assert!(is_recoverable_realtime_inactivity_error(
            "YOUR SESSION WAS CLOSED BECAUSE NO RESPONSE WAS GENERATED FOR 60 SECONDS"
        ));
    }

    #[test]
    fn an_actionable_error_is_never_called_recoverable() {
        for message in [
            "Cannot create response while user is speaking.",
            "Authentication failed",
            "",
            "session was closed because no response was generated for seconds",
            // The duration is required, not optional: with the digits removed
            // the tail still lines up, so a matcher that only looked for
            // ` seconds` would accept it.
            "session was closed because no response was generated for  seconds",
            "session was closed because no response was generated for -1 seconds",
            "session was closed because no response was generated",
        ] {
            assert!(
                !is_recoverable_realtime_inactivity_error(message),
                "{message}"
            );
        }
    }

    #[test]
    fn the_suppressed_classes_are_the_four_the_gateway_never_shows() {
        assert!(ErrorClass::Inactivity.is_suppressed());
        assert!(ErrorClass::Fatal.is_suppressed());
        assert!(ErrorClass::CapacityBusy.is_suppressed());
        assert!(ErrorClass::NoActiveResponse.is_suppressed());
        assert!(!ErrorClass::Other.is_suppressed());
        assert!(!ErrorClass::InputBusy.is_suppressed());
        assert!(!ErrorClass::ResponseSlotBusy.is_suppressed());
    }
}
