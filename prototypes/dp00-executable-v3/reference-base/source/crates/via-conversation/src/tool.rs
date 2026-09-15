//! The shape every tool failure takes, and the gate both tools share.
//!
//! Ported from `server/src/voice/tools/tool-call-handler.mjs:16,31-44`.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};

/// The sensitive-content gate both tools apply before anything is stored.
///
/// **External contract** — `tool-call-handler.mjs:16`:
/// `` /(?:pass(?:word)?|secret|api[_ -]?key|access[_ -]?token|credential|验证码|密码|密钥|令牌|\bsk-[a-z0-9_-]+)/i ``
///
/// It is deliberately blunt. A false positive costs one refused write that the
/// user can rephrase; a false negative writes a credential into a Markdown file
/// that is replayed into every future session's prompt.
pub const SENSITIVE_MEMORY_PATTERN: &str = r"(?i)(?:pass(?:word)?|secret|api[_ -]?key|access[_ -]?token|credential|验证码|密码|密钥|令牌|\bsk-[a-z0-9_-]+)";

static SENSITIVE_MEMORY: Lazy<Option<Regex>> =
    Lazy::new(|| Regex::new(SENSITIVE_MEMORY_PATTERN).ok());

/// Whether `value` trips the sensitive-content gate.
///
/// A pattern that failed to compile would silently disable the gate, so the
/// fallback treats **everything** as sensitive rather than nothing. Refusing to
/// write is the safe direction; the pattern is a literal and cannot in fact
/// fail.
#[must_use]
pub fn is_sensitive(value: &str) -> bool {
    match SENSITIVE_MEMORY.as_ref() {
        Some(pattern) => pattern.is_match(value),
        None => true,
    }
}

/// The `status` a refusal carries by default.
///
/// **External contract** — `tool-call-handler.mjs:33`.
pub const FAILED_STATUS: &str = "failed";

/// The `status` a policy refusal carries.
///
/// **External contract** — `tool-call-handler.mjs:1029,1096`. `rejected` says
/// "this will not be done", as distinct from `failed`'s "this did not work",
/// and the model is expected to explain rather than retry.
pub const REJECTED_STATUS: &str = "rejected";

/// A tool refusal, in the exact shape the model receives.
///
/// **External contract** — `tool-call-handler.mjs:31-44`:
/// `{status, error: true, error_code, user_message, retryable, ...details}`,
/// in that order. `error_code` is stable and unlocalized; `user_message` is
/// the localized sentence beside it (`docs/architecture.md` §16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolFailure {
    /// `failed` or `rejected`.
    pub status: &'static str,
    /// The stable code.
    pub error_code: String,
    /// The localized sentence.
    pub user_message: String,
    /// Whether the model may try again.
    pub retryable: bool,
    /// Extra fields, appended in insertion order after `retryable`.
    pub details: Map<String, Value>,
}

impl ToolFailure {
    /// A plain refusal: `status: 'failed'`, not retryable, no details.
    #[must_use]
    pub fn new(error_code: impl Into<String>, user_message: impl Into<String>) -> Self {
        Self {
            status: FAILED_STATUS,
            error_code: error_code.into(),
            user_message: user_message.into(),
            retryable: false,
            details: Map::new(),
        }
    }

    /// Mark the refusal retryable.
    #[must_use]
    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }

    /// Report `status: 'rejected'` instead of `failed`.
    #[must_use]
    pub fn rejected(mut self) -> Self {
        self.status = REJECTED_STATUS;
        self
    }

    /// Append one detail field after `retryable`.
    #[must_use]
    pub fn detail(mut self, key: &str, value: Value) -> Self {
        self.details.insert(key.to_owned(), value);
        self
    }

    /// The wire object.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut out = Map::new();
        out.insert("status".into(), Value::from(self.status));
        out.insert("error".into(), Value::Bool(true));
        out.insert("error_code".into(), Value::from(self.error_code.clone()));
        out.insert(
            "user_message".into(),
            Value::from(self.user_message.clone()),
        );
        out.insert("retryable".into(), Value::Bool(self.retryable));
        for (key, value) in &self.details {
            out.insert(key.clone(), value.clone());
        }
        Value::Object(out)
    }
}

impl Serialize for ToolFailure {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_value().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failure_serializes_in_the_catalogued_field_order() {
        let failure = ToolFailure::new("edit_not_found", "read it again")
            .retryable()
            .detail("documents", Value::Array(vec![]));
        let Value::Object(object) = failure.to_value() else {
            panic!("a failure is an object");
        };
        assert_eq!(
            object.keys().collect::<Vec<_>>(),
            vec![
                "status",
                "error",
                "error_code",
                "user_message",
                "retryable",
                "documents"
            ]
        );
        assert_eq!(object["status"], Value::from("failed"));
        assert_eq!(object["error"], Value::Bool(true));
    }

    #[test]
    fn the_gate_catches_both_languages_and_a_key_shape() {
        for value in [
            "my password is hunter2",
            "PASSWORD",
            "the api key",
            "api-key",
            "api_key",
            "access token",
            "credential",
            "验证码是 1234",
            "我的密码是 123456",
            "密钥",
            "令牌",
            "sk-abcdef123456",
        ] {
            assert!(is_sensitive(value), "{value} should have been refused");
        }
    }

    #[test]
    fn the_gate_lets_ordinary_memory_through() {
        for value in [
            "用户每天早上跑步",
            "the user lives in Shanghai",
            "- 称呼：老大",
            "",
        ] {
            assert!(!is_sensitive(value), "{value} should have been allowed");
        }
    }
}
