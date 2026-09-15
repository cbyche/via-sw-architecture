//! Log levels.
//!
//! **External contract.** The six emittable level names, the `silent`
//! threshold, and their numeric ordering are reproduced from upstream
//! `shared/logger.mjs:14-22` (`LOG_LEVELS`). `VIA_LOG_LEVEL` accepts exactly
//! these seven names; anything else falls back to `info`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The default level when nothing selects one, and the fallback for an
/// unrecognised `VIA_LOG_LEVEL`.
///
/// **External contract** — upstream `shared/logger.mjs:42,248,258`.
pub const DEFAULT_LOG_LEVEL: LogLevel = LogLevel::Info;

/// A VIA log level.
///
/// The variants are declared in ascending severity, so the derived [`Ord`]
/// agrees with [`LogLevel::severity`] and with upstream's numeric map.
///
/// `Silent` is a *threshold-only* level: it has no emit method, because
/// upstream exposes no `logger.silent()`. Selecting it suppresses every
/// record, which is what upstream's `Number.POSITIVE_INFINITY` achieves.
///
/// **External contract** — the wire form is the lowercase name, written into
/// the `level` field of every `via.log/v1` record
/// (upstream `shared/logger.mjs:14-22,301`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// `trace` — severity 10.
    Trace,
    /// `debug` — severity 20.
    Debug,
    /// `info` — severity 30. The default.
    Info,
    /// `warn` — severity 40. First level routed to stderr.
    Warn,
    /// `error` — severity 50.
    Error,
    /// `fatal` — severity 60. The highest emittable level.
    Fatal,
    /// `silent` — severity `+Infinity`. Threshold only; never emitted.
    Silent,
}

impl LogLevel {
    /// Every level, in ascending severity order.
    ///
    /// **External contract** — the accepted `VIA_LOG_LEVEL` values
    /// (upstream `shared/logger.mjs:14-22,42-45`).
    pub const ALL: [LogLevel; 7] = [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
        LogLevel::Fatal,
        LogLevel::Silent,
    ];

    /// The six levels that can appear in a record's `level` field.
    pub const EMITTABLE: [LogLevel; 6] = [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
        LogLevel::Fatal,
    ];

    /// The lowercase wire name.
    ///
    /// **External contract** — upstream `shared/logger.mjs:14-22`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Fatal => "fatal",
            LogLevel::Silent => "silent",
        }
    }

    /// The numeric severity.
    ///
    /// **External contract** — `trace:10, debug:20, info:30, warn:40,
    /// error:50, fatal:60, silent:+Infinity`
    /// (upstream `shared/logger.mjs:14-22`). `f64` is used rather than an
    /// integer so `silent` can carry the literal infinity upstream uses.
    #[must_use]
    pub fn severity(self) -> f64 {
        match self {
            LogLevel::Trace => 10.0,
            LogLevel::Debug => 20.0,
            LogLevel::Info => 30.0,
            LogLevel::Warn => 40.0,
            LogLevel::Error => 50.0,
            LogLevel::Fatal => 60.0,
            LogLevel::Silent => f64::INFINITY,
        }
    }

    /// Whether a record at this level clears a `threshold`.
    ///
    /// Mirrors upstream's `if (LOG_LEVELS[entryLevel] < threshold) return`
    /// (`shared/logger.mjs:287`).
    #[must_use]
    pub fn is_enabled_for(self, threshold: LogLevel) -> bool {
        self.severity() >= threshold.severity()
    }

    /// Whether the console sink routes this level to stderr.
    ///
    /// **External contract** — `warn` and above go to stderr, below to stdout
    /// (upstream `shared/logger.mjs:307`).
    #[must_use]
    pub fn goes_to_stderr(self) -> bool {
        self.severity() >= LogLevel::Warn.severity()
    }

    /// Resolve a user-supplied level name.
    ///
    /// Trims and lowercases, then requires an exact match against one of the
    /// seven names; anything else yields `fallback`.
    ///
    /// **External contract** — upstream `normalizeLogLevel`
    /// (`shared/logger.mjs:42-45`).
    #[must_use]
    pub fn normalize(value: &str, fallback: LogLevel) -> LogLevel {
        match value.trim().to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            "fatal" => LogLevel::Fatal,
            "silent" => LogLevel::Silent,
            _ => fallback,
        }
    }

    /// Resolve an optional level name, treating `None` and the empty string
    /// the same way upstream's `String(value || '')` does.
    #[must_use]
    pub fn normalize_opt(value: Option<&str>, fallback: LogLevel) -> LogLevel {
        LogLevel::normalize(value.unwrap_or_default(), fallback)
    }

    /// Map a [`tracing::Level`] onto a VIA level.
    ///
    /// `tracing` has no `fatal`, so [`LogLevel::Fatal`] is reachable only
    /// through this crate's direct API.
    #[must_use]
    pub fn from_tracing(level: &tracing::Level) -> LogLevel {
        match *level {
            tracing::Level::TRACE => LogLevel::Trace,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::ERROR => LogLevel::Error,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        DEFAULT_LOG_LEVEL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_mapping_is_the_upstream_contract() {
        assert_eq!(LogLevel::Trace.severity(), 10.0);
        assert_eq!(LogLevel::Debug.severity(), 20.0);
        assert_eq!(LogLevel::Info.severity(), 30.0);
        assert_eq!(LogLevel::Warn.severity(), 40.0);
        assert_eq!(LogLevel::Error.severity(), 50.0);
        assert_eq!(LogLevel::Fatal.severity(), 60.0);
        assert!(LogLevel::Silent.severity().is_infinite());
        assert!(LogLevel::Silent.severity().is_sign_positive());
    }

    #[test]
    fn declaration_order_matches_severity_order() {
        for pair in LogLevel::ALL.windows(2) {
            assert!(pair[0] < pair[1]);
            assert!(pair[0].severity() < pair[1].severity());
        }
    }

    #[test]
    fn silent_suppresses_every_emittable_level() {
        for level in LogLevel::EMITTABLE {
            assert!(!level.is_enabled_for(LogLevel::Silent));
        }
    }
}
