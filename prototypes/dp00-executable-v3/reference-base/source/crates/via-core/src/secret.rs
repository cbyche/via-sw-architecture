//! A string that must not appear in a log line or a health payload.
//!
//! `via-log` redacts values whose *key* looks sensitive
//! (`api[_-]?key|authorization|cookie|credential|password|secret|token$`), which
//! catches a field logged by name. It cannot catch a whole `Config` formatted
//! with `{:?}`, and that is exactly the shape an operator reaches for while
//! debugging a startup failure.
//!
//! [`Secret`] closes that: `Debug` and `Serialize` both emit
//! [`REDACTED`], and reading the value takes an explicit
//! [`expose`](Secret::expose) that is greppable.

/// The marker substituted for a secret.
///
/// **External contract** — `shared/logger.mjs:24` (`'[REDACTED]'`), the same
/// marker `via-log` writes, so a redacted config field and a redacted log field
/// read alike.
pub const REDACTED: &str = "[REDACTED]";

/// A credential.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// Wrap a value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Read the value. Named to make every use site greppable.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Whether no credential is set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl core::fmt::Debug for Secret {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(if self.0.is_empty() { "\"\"" } else { REDACTED })
    }
}

impl serde::Serialize for Secret {
    /// Serialises the marker, never the value.
    ///
    /// An empty secret serialises as the empty string so "nothing is
    /// configured" stays distinguishable from "something is, and you may not
    /// see it" — which is the distinction `/api/health` and the config snapshot
    /// both need.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(if self.0.is_empty() { "" } else { REDACTED })
    }
}

impl From<&str> for Secret {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_secret_never_renders_itself() {
        let secret = Secret::new("sk-live-0123456789");
        assert_eq!(format!("{secret:?}"), REDACTED);
        assert_eq!(
            serde_json::to_string(&secret).expect("a string always serialises"),
            "\"[REDACTED]\""
        );
        assert_eq!(secret.expose(), "sk-live-0123456789");
    }

    #[test]
    fn an_unset_secret_stays_distinguishable() {
        let secret = Secret::default();
        assert!(secret.is_empty());
        assert_eq!(format!("{secret:?}"), "\"\"");
        assert_eq!(
            serde_json::to_string(&secret).expect("a string always serialises"),
            "\"\""
        );
    }
}
