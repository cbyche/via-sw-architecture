//! `via-log` — VIA's structured logging.
//!
//! A leaf crate (`docs/architecture.md` §9) that owns one thing: the
//! `via.log/v1` JSON-lines record, from the field order on the wire down to
//! the regexes that keep credentials off disk.
//!
//! Ported from upstream `qwen-audio-agent` v1.11.0 —
//! `shared/logger.mjs` and `server/src/core/logger.mjs` — with the identity
//! substitutions in `docs/rebrand.md` applied (`qwaudio.log/v1` →
//! `via.log/v1`, `QWEN_AUDIO_LOG_*` → `VIA_LOG_*`, `QWAUDIO_CONFIG_DIR` →
//! `VIA_CONFIG_DIR`, `~/.config/qwaudio` → `~/.config/via`).
//!
//! # The four pieces
//!
//! * [`LogRecord`] — the spine. Caller fields first, then
//!   `schema`/`time`/`level`/`component`/`event`/`pid`/`message`, written last
//!   so a caller cannot forge them.
//! * [`LogLevel`] — six levels plus `silent`, with upstream's numeric
//!   ordering.
//! * [`redact`] — the security boundary. Key-shaped secrets, `Bearer`/`Basic`
//!   credentials, `sk-` keys and URL query secrets, plus depth, width and
//!   length caps.
//! * [`LogSink`] — [`JsonLineFileSink`] (append, flushed per line, rotating)
//!   and [`ConsoleSink`] (`warn` and above to stderr).
//!
//! # Two ways in
//!
//! Ordinary `tracing` macros, through [`ViaLayer`]:
//!
//! ```
//! use std::sync::Arc;
//! use tracing_subscriber::layer::SubscriberExt;
//! use via_log::{Logger, LoggerOptions, MemorySink, ViaLayer};
//!
//! let sink = Arc::new(MemorySink::new());
//! let logger = Logger::with_sinks(LoggerOptions::detached("gateway"), vec![sink.clone()]);
//! let subscriber = tracing_subscriber::registry().with(ViaLayer::new(logger));
//!
//! tracing::subscriber::with_default(subscriber, || {
//!     tracing::info!(event = "gateway.ready", port = 8765, "listening");
//! });
//!
//! assert_eq!(sink.records()[0]["event"], "gateway.ready");
//! ```
//!
//! Or [`Logger::log_record`] / [`log_record`], for call sites that need to
//! control the exact contract shape.
//!
//! # Recorded deviations from upstream
//!
//! * **Correlation context.** Upstream uses `AsyncLocalStorage`, which follows
//!   a value across `await`. [`run_with_log_context`] is a thread-local and
//!   does not; async call sites should carry correlation on a `tracing` span,
//!   which [`ViaLayer`] folds into the same position in the record.
//! * **`[Circular]`.** A [`serde_json::Value`] cannot contain a cycle, so the
//!   marker is reproduced as a constant but never produced.
//! * **String truncation** counts Unicode scalar values where upstream counts
//!   UTF-16 code units. Identical for the ASCII payloads the cap exists to
//!   bound.
//! * **`fatal`** has no `tracing` level; it is reachable only through
//!   [`Logger::fatal`] and [`Logger::log_record`].
//! * **Windows** has no mode bits, so [`LOG_DIRECTORY_MODE`] and
//!   [`LOG_FILE_MODE`] are Unix-only.
//! * **Test-process detection** keeps both of upstream's arms
//!   (`NODE_ENV === 'test'` and a `test/` argv segment) and adds a third,
//!   [`ENV_TEST_PROCESS`] (`VIA_TEST=1`), because neither upstream arm can
//!   fire for a Rust build. See [`is_test_process`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod env;
pub mod layer;
pub mod level;
pub mod logger;
pub mod record;
pub mod redact;
pub mod sink;

pub use env::{
    CONFIG_DIRECTORY_NAME, ENV_CONFIG_DIR, ENV_LOG_CONSOLE, ENV_LOG_DIR, ENV_LOG_FILE,
    ENV_LOG_LEVEL, ENV_LOG_MAX_BYTES, ENV_LOG_MAX_FILES, ENV_NODE_ENV, ENV_TEST_PROCESS,
    ENV_XDG_CONFIG_HOME, EnvSettings, EnvSource, LOG_DIRECTORY_NAME, NODE_ENV_TEST, ProcessEnv,
    TEST_PROCESS_VALUE, bounded_integer, default_log_directory, default_log_directory_from_process,
    is_test_process,
};
pub use layer::{EVENT_FIELD, MESSAGE_FIELD, ViaLayer};
pub use level::{DEFAULT_LOG_LEVEL, LogLevel};
pub use logger::{
    Logger, LoggerOptions, SetGlobalLoggerError, current_log_context, gateway_logger,
    global_logger, log_record, run_with_log_context, set_global_logger,
};
pub use record::{
    DEFAULT_COMPONENT, DEFAULT_EVENT, GATEWAY_COMPONENT, GATEWAY_LOG_FILE_NAME, LOG_SCHEMA,
    LogRecord, RESERVED_FIELDS, now_iso8601, safe_event_name,
};
pub use redact::{
    API_KEY_VALUE_PATTERN, AUTH_VALUE_PATTERN, CIRCULAR, ErrorRecord, MAX_COLLECTION_ITEMS,
    MAX_DEPTH, MAX_DEPTH_MARKER, MAX_STRING_CHARS, REDACTED, SENSITIVE_KEY_PATTERN,
    TRUNCATED_MARKER, TRUNCATED_SUFFIX, URL_SECRET_VALUE_PATTERN, is_sensitive_key, redact_map,
    redact_value, redact_value_for_key, scrub_string,
};
pub use sink::{
    ConsoleSink, DEFAULT_MAX_BYTES, DEFAULT_MAX_FILES, FileSinkOptions, JsonLineFileSink,
    LOG_DIRECTORY_MODE, LOG_FILE_MODE, LOG_SINK_FAILURE_PREFIX_ZH, LevelFilterSink, LogSink,
    MAX_MAX_BYTES, MAX_MAX_FILES, MIN_MAX_BYTES, MIN_MAX_FILES, MemorySink, SinkErrorHandler,
    log_sink_failure_message,
};

/// Build a field map from `(name, value)` pairs, preserving insertion order.
///
/// ```
/// let fields = via_log::fields([("durationMs", 42.into()), ("ok", true.into())]);
/// assert_eq!(fields.keys().collect::<Vec<_>>(), ["durationMs", "ok"]);
/// ```
#[must_use]
pub fn fields<I, K>(pairs: I) -> serde_json::Map<String, serde_json::Value>
where
    I: IntoIterator<Item = (K, serde_json::Value)>,
    K: Into<String>,
{
    pairs
        .into_iter()
        .map(|(key, value)| (key.into(), value))
        .collect()
}

/// An empty field map — the unambiguous spelling of `fields([])`.
#[must_use]
pub fn no_fields() -> serde_json::Map<String, serde_json::Value> {
    serde_json::Map::new()
}
