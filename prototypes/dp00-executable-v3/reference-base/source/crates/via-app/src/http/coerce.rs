//! `String(x || '')` and `Number(x)`, for request bodies.
//!
//! Every route in `gateway-application.mjs` reads its body through one of two
//! JavaScript coercions, and reproducing them is what keeps a wrongly-typed
//! field on the **catalogued** error path instead of a new one:
//!
//! | Upstream | Here | Why it matters |
//! | --- | --- | --- |
//! | `String(req.body?.decision \|\| '')` | [`js_string`] | `{"decision": true}` must be the catalogued **400**, not a deserialization 422 |
//! | `String(owner \|\| '').trim()` | [`js_string`] | `{"owner": 5}` suspends for `"5"`, it does not fail |
//! | `Number(ttlMs)` | [`js_number`] | `{"ttlMs": "60000"}` is 60 000 ms |
//!
//! A typed `Option<String>` field would answer 422 for all three, and 422 is
//! not in the catalogue.
//!
//! # Recorded deviation
//!
//! `String()` of an **array or object** is `"1,2"` / `"[object Object]"` in
//! JavaScript. Here it is the value's JSON rendering. Neither form is ever
//! equal to `always`, `reject`, or a non-empty owner a host would use, so the
//! observable outcome — the catalogued 400, or a hold under a nonsense name —
//! is the same. Reproducing `[object Object]` exactly would be reproducing a
//! JavaScript wart, not a contract.

use serde_json::Value;

/// `String(value || '')`.
///
/// The `|| ''` is what makes every falsy value the empty string: `null`,
/// `undefined`, `false`, `0` and `""` all coerce to `""`, which is why a
/// missing field and a `false` one take the same branch.
#[must_use]
pub fn js_string(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) | Some(Value::Bool(false)) => String::new(),
        Some(Value::Bool(true)) => "true".to_owned(),
        Some(Value::Number(number)) => {
            // `0 || ''` is `''`; every other number stringifies.
            if number.as_f64() == Some(0.0) {
                String::new()
            } else {
                number.to_string()
            }
        }
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
    }
}

/// `Number(value)`, answering `None` for `NaN`.
///
/// `Number("60000")` is `60000`, `Number(true)` is `1`, `Number(null)` is `0`,
/// and `Number("abc")` is `NaN` — which the caller then treats as *"fall back
/// to the default"*, exactly as `Number.isFinite` does upstream.
#[must_use]
pub fn js_number(value: Option<&Value>) -> Option<f64> {
    match value {
        None => None,
        Some(Value::Null) => Some(0.0),
        Some(Value::Bool(flag)) => Some(if *flag { 1.0 } else { 0.0 }),
        Some(Value::Number(number)) => number.as_f64(),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                // `Number('')` is 0, and `Number('   ')` is 0 too.
                return Some(0.0);
            }
            trimmed.parse::<f64>().ok()
        }
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn every_falsy_value_becomes_the_empty_string() {
        for falsy in [json!(null), json!(false), json!(0), json!("")] {
            assert_eq!(js_string(Some(&falsy)), "", "{falsy}");
        }
        assert_eq!(js_string(None), "");
    }

    #[test]
    fn a_wrongly_typed_field_stringifies_rather_than_failing() {
        assert_eq!(js_string(Some(&json!(true))), "true");
        assert_eq!(js_string(Some(&json!(5))), "5");
        assert_eq!(js_string(Some(&json!("always"))), "always");
    }

    #[test]
    fn number_reproduces_the_falls_back_cases() {
        assert_eq!(js_number(Some(&json!("60000"))), Some(60_000.0));
        assert_eq!(js_number(Some(&json!(60_000))), Some(60_000.0));
        assert_eq!(js_number(Some(&json!(null))), Some(0.0));
        assert_eq!(js_number(Some(&json!("abc"))), None);
        assert_eq!(js_number(None), None);
        assert_eq!(js_number(Some(&json!(true))), Some(1.0));
    }

    #[test]
    fn a_structured_value_is_rendered_rather_than_refused() {
        // Deliberately not `[object Object]`; see the module documentation.
        assert_eq!(js_string(Some(&json!({ "a": 1 }))), "{\"a\":1}");
        assert_eq!(js_number(Some(&json!([1, 2]))), None);
    }
}
