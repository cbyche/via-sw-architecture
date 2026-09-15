//! The logger itself: correlation context, the six emit methods, child
//! loggers, and the process-global instance.

use crate::env::{EnvSettings, EnvSource, ProcessEnv};
use crate::level::LogLevel;
use crate::record::{DEFAULT_COMPONENT, GATEWAY_COMPONENT, GATEWAY_LOG_FILE_NAME, LogRecord};
use crate::redact::redact_map;
use crate::sink::{
    ConsoleSink, DEFAULT_MAX_BYTES, DEFAULT_MAX_FILES, FileSinkOptions, JsonLineFileSink, LogSink,
    SinkErrorHandler, log_sink_failure_message,
};
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

thread_local! {
    /// Upstream uses `AsyncLocalStorage`; a thread-local is the closest
    /// synchronous equivalent. Async call sites should carry correlation on a
    /// `tracing` span instead — [`crate::ViaLayer`] folds span fields into the
    /// same position in the record.
    static LOG_CONTEXT: RefCell<Vec<Map<String, Value>>> = const { RefCell::new(Vec::new()) };
}

/// Run `body` with `context` merged into every record it emits.
///
/// **External contract** — upstream `runWithLogContext`
/// (`shared/logger.mjs:239-241`). The context *replaces* any enclosing one
/// rather than merging with it, matching `AsyncLocalStorage#run`, and is
/// redacted on entry.
pub fn run_with_log_context<T>(context: Map<String, Value>, body: impl FnOnce() -> T) -> T {
    LOG_CONTEXT.with(|stack| stack.borrow_mut().push(redact_map(&context)));
    let guard = ContextGuard;
    let result = body();
    drop(guard);
    result
}

struct ContextGuard;

impl Drop for ContextGuard {
    fn drop(&mut self) {
        LOG_CONTEXT.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

/// The innermost active log context, already redacted.
#[must_use]
pub fn current_log_context() -> Map<String, Value> {
    LOG_CONTEXT.with(|stack| stack.borrow().last().cloned().unwrap_or_default())
}

/// Construction parameters for a [`Logger`].
#[derive(Clone, Debug)]
pub struct LoggerOptions {
    /// Written into every record's `component` field.
    pub component: String,
    /// Log file name; defaults to `<component>.log`.
    pub file_name: String,
    /// Directory for the file sink.
    pub directory: PathBuf,
    /// Threshold level.
    pub level: LogLevel,
    /// Mount the console sink.
    pub console_enabled: bool,
    /// Mount the file sink.
    pub file_enabled: bool,
    /// Rotation threshold in bytes.
    pub max_bytes: u64,
    /// Retained file count.
    pub max_files: u32,
    /// Fields merged into every record, ahead of context and caller fields.
    pub base: Map<String, Value>,
}

impl LoggerOptions {
    /// Options for `component`, with the contract defaults and a log directory
    /// resolved from `env`.
    #[must_use]
    pub fn new(component: impl Into<String>, env: &dyn EnvSource, home: &Path) -> Self {
        let component = component.into();
        let settings = EnvSettings::from_env(env, home);
        Self {
            file_name: format!("{component}.log"),
            component,
            directory: settings.directory,
            level: settings.level,
            console_enabled: settings.console_enabled,
            file_enabled: settings.file_enabled,
            max_bytes: settings.max_bytes,
            max_files: settings.max_files,
            base: Map::new(),
        }
    }

    /// Options for the Gateway logger.
    ///
    /// **External contract** — component `gateway`, file `gateway.log`
    /// (upstream `server/src/core/logger.mjs:11-16`).
    #[must_use]
    pub fn gateway(env: &dyn EnvSource, home: &Path) -> Self {
        let mut options = Self::new(GATEWAY_COMPONENT, env, home);
        options.file_name = GATEWAY_LOG_FILE_NAME.to_owned();
        options
    }

    /// Options with no sinks at all and no environment lookups — the shape a
    /// test or an embedder starts from before attaching its own sinks.
    #[must_use]
    pub fn detached(component: impl Into<String>) -> Self {
        let component = component.into();
        Self {
            file_name: format!("{component}.log"),
            component,
            directory: PathBuf::new(),
            level: LogLevel::default(),
            console_enabled: false,
            file_enabled: false,
            max_bytes: DEFAULT_MAX_BYTES,
            max_files: DEFAULT_MAX_FILES,
            base: Map::new(),
        }
    }
}

struct Inner {
    component: String,
    level: LogLevel,
    pid: u32,
    base: Map<String, Value>,
    sinks: Vec<Arc<dyn LogSink>>,
    file_path: Option<PathBuf>,
}

/// A structured logger writing `via.log/v1` records to its sinks.
///
/// Cheap to clone; clones share the same sinks.
#[derive(Clone)]
pub struct Logger {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for Logger {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Logger")
            .field("component", &self.inner.component)
            .field("level", &self.inner.level)
            .field("file_path", &self.inner.file_path)
            .field("sinks", &self.inner.sinks.len())
            .finish()
    }
}

impl Logger {
    /// Build a logger from `options`, mounting the console and file sinks the
    /// options enable.
    ///
    /// The file sink's failure handler writes
    /// [`crate::LOG_SINK_FAILURE_PREFIX_ZH`] to stderr **once**, and only when
    /// the console is enabled — upstream `shared/logger.mjs:277-281`.
    #[must_use]
    pub fn new(options: LoggerOptions) -> Self {
        let mut sinks: Vec<Arc<dyn LogSink>> = Vec::new();
        let mut file_path = None;
        if options.file_enabled {
            let sink = JsonLineFileSink::new(FileSinkOptions {
                directory: options.directory.clone(),
                file_name: options.file_name.clone(),
                max_bytes: options.max_bytes,
                max_files: options.max_files,
                on_error: options
                    .console_enabled
                    .then(|| stderr_failure_handler() as SinkErrorHandler),
            });
            file_path = Some(sink.path().to_path_buf());
            sinks.push(Arc::new(sink));
        }
        if options.console_enabled {
            sinks.push(Arc::new(ConsoleSink::new()));
        }
        Self::build(options, sinks, file_path)
    }

    /// Build a logger with explicit sinks, ignoring
    /// [`LoggerOptions::console_enabled`] and
    /// [`LoggerOptions::file_enabled`].
    #[must_use]
    pub fn with_sinks(options: LoggerOptions, sinks: Vec<Arc<dyn LogSink>>) -> Self {
        Self::build(options, sinks, None)
    }

    fn build(
        options: LoggerOptions,
        sinks: Vec<Arc<dyn LogSink>>,
        file_path: Option<PathBuf>,
    ) -> Self {
        let component = if options.component.is_empty() {
            DEFAULT_COMPONENT.to_owned()
        } else {
            options.component
        };
        Self {
            inner: Arc::new(Inner {
                component,
                level: options.level,
                pid: std::process::id(),
                base: redact_map(&options.base),
                sinks,
                file_path,
            }),
        }
    }

    /// The component name stamped on every record.
    #[must_use]
    pub fn component(&self) -> &str {
        &self.inner.component
    }

    /// The threshold level.
    #[must_use]
    pub fn level(&self) -> LogLevel {
        self.inner.level
    }

    /// The file this logger appends to, when it has a file sink.
    #[must_use]
    pub fn file_path(&self) -> Option<&Path> {
        self.inner.file_path.as_deref()
    }

    /// Fields merged into every record.
    #[must_use]
    pub fn base(&self) -> &Map<String, Value> {
        &self.inner.base
    }

    /// A logger sharing this one's sinks, with `context` added to its base
    /// fields.
    ///
    /// **External contract** — upstream `logger.child`
    /// (`shared/logger.mjs:327-339`): the child's base is
    /// `{...parentBase, ...redact(context)}` and the file sink is shared, not
    /// reopened.
    #[must_use]
    pub fn child(&self, context: Map<String, Value>) -> Self {
        let mut base = self.inner.base.clone();
        base.extend(redact_map(&context));
        Self {
            inner: Arc::new(Inner {
                component: self.inner.component.clone(),
                level: self.inner.level,
                pid: self.inner.pid,
                base,
                sinks: self.inner.sinks.clone(),
                file_path: self.inner.file_path.clone(),
            }),
        }
    }

    /// Emit at `trace`.
    pub fn trace(&self, event: &str, fields: Map<String, Value>, message: &str) {
        self.emit(LogLevel::Trace, event, fields, message);
    }

    /// Emit at `debug`.
    pub fn debug(&self, event: &str, fields: Map<String, Value>, message: &str) {
        self.emit(LogLevel::Debug, event, fields, message);
    }

    /// Emit at `info`.
    pub fn info(&self, event: &str, fields: Map<String, Value>, message: &str) {
        self.emit(LogLevel::Info, event, fields, message);
    }

    /// Emit at `warn`.
    pub fn warn(&self, event: &str, fields: Map<String, Value>, message: &str) {
        self.emit(LogLevel::Warn, event, fields, message);
    }

    /// Emit at `error`.
    pub fn error(&self, event: &str, fields: Map<String, Value>, message: &str) {
        self.emit(LogLevel::Error, event, fields, message);
    }

    /// Emit at `fatal`.
    pub fn fatal(&self, event: &str, fields: Map<String, Value>, message: &str) {
        self.emit(LogLevel::Fatal, event, fields, message);
    }

    /// Emit at `level`, merging base fields, the active log context and
    /// `fields` under the spine.
    ///
    /// **External contract** — upstream `emit`
    /// (`shared/logger.mjs:286-314`).
    pub fn emit(&self, level: LogLevel, event: &str, fields: Map<String, Value>, message: &str) {
        self.emit_with_context(level, event, &[current_log_context()], fields, message);
    }

    /// Emit at `level` with explicit context layers, applied in order between
    /// the base fields and `fields`.
    ///
    /// Each layer is redacted separately, so each is independently capped at
    /// [`crate::MAX_COLLECTION_ITEMS`] — the same as upstream, which redacts
    /// `base`, the async context and `fields` as three separate calls.
    pub fn emit_with_context(
        &self,
        level: LogLevel,
        event: &str,
        context: &[Map<String, Value>],
        fields: Map<String, Value>,
        message: &str,
    ) {
        if !level.is_enabled_for(self.inner.level) {
            return;
        }
        let mut extra = self.inner.base.clone();
        for layer in context {
            extra.extend(redact_map(layer));
        }
        extra.extend(redact_map(&fields));
        // Upstream tests the *original* message for truthiness and only then
        // scrubs it (`shared/logger.mjs:303`).
        let message = if message.is_empty() {
            String::new()
        } else {
            crate::redact::scrub_string(message)
        };
        let mut record = LogRecord::new(level, &self.inner.component, event, extra, &message);
        record.pid = self.inner.pid;
        self.write(&record);
    }

    /// Emit a fully-formed record.
    ///
    /// For call sites that need to control the exact contract shape — a
    /// replayed record, a record whose `time` is not "now", or one assembled
    /// by another crate. The record's `extra` and `message` are redacted
    /// first; redaction is idempotent, so a record already built through
    /// [`Logger::emit`] is unchanged.
    pub fn log_record(&self, record: &LogRecord) {
        if !record.level.is_enabled_for(self.inner.level) {
            return;
        }
        self.write(&record.redacted());
    }

    fn write(&self, record: &LogRecord) {
        for sink in &self.inner.sinks {
            sink.write_record(record);
        }
    }

    /// Flush every sink.
    pub fn flush(&self) {
        for sink in &self.inner.sinks {
            sink.flush();
        }
    }
}

fn stderr_failure_handler() -> Arc<dyn Fn(&str) + Send + Sync> {
    // Upstream reports the first failure only, for the lifetime of the logger
    // (`fileFailureReported`), so a wedged disk cannot flood stderr.
    let reported = AtomicBool::new(false);
    Arc::new(move |detail: &str| {
        if reported.swap(true, Ordering::SeqCst) {
            return;
        }
        // Logging must never interrupt the application.
        let _ = std::io::stderr().write_all(log_sink_failure_message(detail).as_bytes());
    })
}

static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

/// Install the process-global logger.
///
/// # Errors
///
/// Returns [`SetGlobalLoggerError`] if a global logger is already installed.
pub fn set_global_logger(logger: Logger) -> Result<(), SetGlobalLoggerError> {
    GLOBAL_LOGGER.set(logger).map_err(|_| SetGlobalLoggerError)
}

/// The process-global logger, if one has been installed.
#[must_use]
pub fn global_logger() -> Option<&'static Logger> {
    GLOBAL_LOGGER.get()
}

/// Emit a fully-formed record through the process-global logger.
///
/// A no-op when no global logger is installed — logging must never interrupt
/// the application, and a missing logger is not an error the caller can act
/// on.
pub fn log_record(record: &LogRecord) {
    if let Some(logger) = global_logger() {
        logger.log_record(record);
    }
}

/// A global logger was already installed.
#[derive(Clone, Copy, Debug, thiserror::Error)]
#[error("a global via-log logger is already installed")]
pub struct SetGlobalLoggerError;

/// Build the Gateway logger against the real process environment.
///
/// **External contract** — upstream `server/src/core/logger.mjs:11-16`,
/// including the test-process detection that disables both sinks.
#[must_use]
pub fn gateway_logger() -> Logger {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut options = LoggerOptions::gateway(&ProcessEnv, &home);
    if crate::env::is_test_process(&ProcessEnv, std::env::args()) {
        options.console_enabled = false;
        options.file_enabled = false;
    }
    Logger::new(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MemorySink;
    use serde_json::json;

    fn memory_logger(level: LogLevel) -> (Logger, Arc<MemorySink>) {
        let sink = Arc::new(MemorySink::new());
        let mut options = LoggerOptions::detached("gateway");
        options.level = level;
        let logger = Logger::with_sinks(options, vec![sink.clone()]);
        (logger, sink)
    }

    #[test]
    fn context_is_merged_between_base_and_fields() {
        let (logger, sink) = memory_logger(LogLevel::Trace);
        let child = logger.child(
            [("provider".to_owned(), json!("dashscope"))]
                .into_iter()
                .collect(),
        );
        run_with_log_context(
            [
                ("sessionId".to_owned(), json!("session-1")),
                ("turnId".to_owned(), json!("turn-2")),
            ]
            .into_iter()
            .collect(),
            || {
                child.info(
                    "realtime.connected",
                    [("durationMs".to_owned(), json!(42))].into_iter().collect(),
                    "Realtime ready",
                );
            },
        );
        let records = sink.records();
        assert_eq!(records.len(), 1);
        let keys: Vec<&str> = records[0]
            .as_object()
            .map(|map| map.keys().map(String::as_str).collect())
            .unwrap_or_default();
        assert_eq!(
            keys,
            [
                "provider",
                "sessionId",
                "turnId",
                "durationMs",
                "schema",
                "time",
                "level",
                "component",
                "event",
                "pid",
                "message"
            ]
        );
    }

    #[test]
    fn context_does_not_leak_past_the_closure() {
        run_with_log_context(
            [("sessionId".to_owned(), json!("s"))].into_iter().collect(),
            || {},
        );
        assert!(current_log_context().is_empty());
    }
}
