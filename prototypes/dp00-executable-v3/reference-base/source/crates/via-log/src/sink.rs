//! Where records go: a JSON-lines file, the console, or memory.
//!
//! Every sink obeys one rule taken from upstream
//! (`shared/logger.mjs:310-312`): **logging must never interrupt the
//! application.** A sink swallows its own I/O errors; the file sink reports
//! the first one through an error handler and then keeps quiet.

use crate::level::LogLevel;
use crate::record::LogRecord;
use crate::redact::scrub_string;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Bytes written to one log file before it rotates.
///
/// **External contract** — upstream `DEFAULT_MAX_BYTES = 10 * 1024 * 1024`
/// (`shared/logger.mjs:24`).
pub const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024;

/// Number of log files retained, the live one included.
///
/// **External contract** — upstream `DEFAULT_MAX_FILES = 5`
/// (`shared/logger.mjs:25`).
pub const DEFAULT_MAX_FILES: u32 = 5;

/// Lower clamp for `VIA_LOG_MAX_BYTES`.
///
/// **External contract** — upstream `shared/logger.mjs:156,265-270`.
pub const MIN_MAX_BYTES: u64 = 1024;

/// Upper clamp for `VIA_LOG_MAX_BYTES`.
///
/// **External contract** — upstream `shared/logger.mjs:269` (`1024³`).
pub const MAX_MAX_BYTES: u64 = 1024 * 1024 * 1024;

/// Lower clamp for `VIA_LOG_MAX_FILES`.
///
/// **External contract** — upstream `shared/logger.mjs:157,275`.
pub const MIN_MAX_FILES: u32 = 1;

/// Upper clamp for `VIA_LOG_MAX_FILES`.
///
/// **External contract** — upstream `shared/logger.mjs:275`.
pub const MAX_MAX_FILES: u32 = 100;

/// Directory mode for the log directory (Unix only).
///
/// **External contract** — upstream `mkdir(directory, { mode: 0o700 })`
/// (`shared/logger.mjs:173`).
pub const LOG_DIRECTORY_MODE: u32 = 0o700;

/// File mode for every log file (Unix only).
///
/// **External contract** — upstream `chmod(path, 0o600)` and
/// `appendFile(..., { mode: 0o600 })` (`shared/logger.mjs:176,205`).
pub const LOG_FILE_MODE: u32 = 0o600;

/// Prefix of the message written to stderr when the log sink itself fails.
///
/// **External contract** — upstream
/// `` `qwen-audio-agent 日志写入失败：${scrubString(error.message)}\n` ``
/// (`shared/logger.mjs:280`), renamed per `docs/rebrand.md` to the VIA
/// product name.
///
/// **Why this stays a literal.** Phase 0 left a `TODO(via-i18n)` here saying
/// the string would become a catalog lookup once `via-i18n` existed. It exists,
/// and the lookup is not available: `docs/architecture.md` §9's adjacency table
/// gives `shared → ∅`, so a Leaf-band crate may depend on no VIA crate at all,
/// and `via-arch-test`'s `leaf_violations` fails the build if one tries. VIA is
/// still trilingual — `via-i18n` carries this sentence as
/// `log.sink_write_failed` in `en`, `zh` and `ko`, and a caller with a locale
/// renders that. This constant is the sink's own last-resort line, written to
/// stderr when the logger itself cannot write, which is the one place no
/// catalog lookup can be assumed to work.
///
/// The two copies cannot drift:
/// `via-conformance`'s `tests/localized_leaf_messages.rs` asserts this constant
/// equals `log.sink_write_failed`'s `zh` value rendered with an empty detail.
pub const LOG_SINK_FAILURE_PREFIX_ZH: &str = "VIA 日志写入失败：";

/// Render the log-sink failure line, newline included.
///
/// The detail is scrubbed before it is printed: a failing write can carry a
/// path, and a path can carry a token.
#[must_use]
pub fn log_sink_failure_message(detail: &str) -> String {
    format!("{LOG_SINK_FAILURE_PREFIX_ZH}{}\n", scrub_string(detail))
}

/// A destination for finished records.
///
/// Implementations must not panic and must not propagate errors.
pub trait LogSink: Send + Sync {
    /// Write one record.
    fn write_record(&self, record: &LogRecord);

    /// Flush any buffered state. The default is a no-op.
    fn flush(&self) {}
}

/// Callback invoked with the message of the first failure in a run of
/// failures.
pub type SinkErrorHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// Append-only JSON-lines file sink with size-based rotation.
///
/// Each record is written and flushed as a single line, so a crash loses at
/// most the record being written. Initialisation is lazy: nothing touches the
/// filesystem until the first record, matching upstream's deferred `drain`.
pub struct JsonLineFileSink {
    directory: PathBuf,
    path: PathBuf,
    max_bytes: u64,
    max_files: u32,
    on_error: Option<SinkErrorHandler>,
    state: Mutex<FileState>,
}

#[derive(Debug, Default)]
struct FileState {
    file: Option<File>,
    size: u64,
    failed: bool,
}

/// Construction parameters for [`JsonLineFileSink`].
#[derive(Clone)]
pub struct FileSinkOptions {
    /// Directory holding the log file. Created with mode `0o700`.
    pub directory: PathBuf,
    /// File name inside `directory`, conventionally `<component>.log`.
    pub file_name: String,
    /// Rotation threshold in bytes; clamped to at least [`MIN_MAX_BYTES`].
    pub max_bytes: u64,
    /// Files retained; clamped to at least [`MIN_MAX_FILES`].
    pub max_files: u32,
    /// Invoked with the failure message when a write fails.
    pub on_error: Option<SinkErrorHandler>,
}

impl std::fmt::Debug for FileSinkOptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FileSinkOptions")
            .field("directory", &self.directory)
            .field("file_name", &self.file_name)
            .field("max_bytes", &self.max_bytes)
            .field("max_files", &self.max_files)
            .field("on_error", &self.on_error.is_some())
            .finish()
    }
}

impl FileSinkOptions {
    /// Options for `<directory>/<file_name>` with the contract defaults.
    #[must_use]
    pub fn new(directory: impl Into<PathBuf>, file_name: impl Into<String>) -> Self {
        Self {
            directory: directory.into(),
            file_name: file_name.into(),
            max_bytes: DEFAULT_MAX_BYTES,
            max_files: DEFAULT_MAX_FILES,
            on_error: None,
        }
    }
}

impl JsonLineFileSink {
    /// Build a sink. No I/O happens here.
    ///
    /// `max_bytes` and `max_files` are raised to their minimums, reproducing
    /// upstream's `Math.max(1024, maxBytes)` / `Math.max(1, maxFiles)`
    /// (`shared/logger.mjs:156-157`).
    #[must_use]
    pub fn new(options: FileSinkOptions) -> Self {
        let directory = absolute_path(&options.directory);
        let path = directory.join(&options.file_name);
        Self {
            directory,
            path,
            max_bytes: options.max_bytes.max(MIN_MAX_BYTES),
            max_files: options.max_files.max(MIN_MAX_FILES),
            on_error: options.on_error,
            state: Mutex::new(FileState::default()),
        }
    }

    /// The file this sink appends to.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The directory this sink writes into.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Rotation threshold actually in force, after clamping.
    #[must_use]
    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    /// File count actually in force, after clamping.
    #[must_use]
    pub fn max_files(&self) -> u32 {
        self.max_files
    }

    fn open(&self, state: &mut FileState) -> io::Result<()> {
        create_log_directory(&self.directory)?;
        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(LOG_FILE_MODE);
        }
        let file = options.open(&self.path)?;
        state.size = file.metadata()?.len();
        state.file = Some(file);
        // Upstream chmods an existing file back to 0o600 on every
        // initialisation, so a file created before the mode was enforced is
        // repaired rather than left readable.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(LOG_FILE_MODE))?;
        }
        Ok(())
    }

    fn append(&self, state: &mut FileState, line: &str) -> io::Result<()> {
        if state.file.is_none() {
            self.open(state)?;
        }
        let bytes = line.len() as u64;
        // Upstream rotates only once the file is non-empty, so a single record
        // larger than `max_bytes` is written rather than looping.
        if state.size > 0 && state.size + bytes > self.max_bytes {
            state.file = None;
            rotate(&self.path, self.max_files)?;
            self.open(state)?;
            state.size = 0;
        }
        let file = state
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("log file is not open"))?;
        file.write_all(line.as_bytes())?;
        file.flush()?;
        state.size += bytes;
        Ok(())
    }
}

impl std::fmt::Debug for JsonLineFileSink {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JsonLineFileSink")
            .field("path", &self.path)
            .field("max_bytes", &self.max_bytes)
            .field("max_files", &self.max_files)
            .finish_non_exhaustive()
    }
}

impl LogSink for JsonLineFileSink {
    fn write_record(&self, record: &LogRecord) {
        let line = record.to_line();
        let mut report = None;
        {
            // A poisoned mutex means a previous writer panicked mid-write. The
            // state is still structurally valid, and refusing to log after an
            // unrelated panic would be worse than continuing.
            let mut state = match self.state.lock() {
                Ok(state) => state,
                Err(poisoned) => poisoned.into_inner(),
            };
            match self.append(&mut state, &line) {
                Ok(()) => state.failed = false,
                Err(error) => {
                    state.file = None;
                    // Upstream reports on every transition into the failed
                    // state, not once per process; the logger-level handler
                    // adds the once-only guard.
                    if !state.failed {
                        report = Some(error.to_string());
                    }
                    state.failed = true;
                }
            }
        }
        if let (Some(message), Some(handler)) = (report, self.on_error.as_ref()) {
            handler(&message);
        }
    }

    fn flush(&self) {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(file) = state.file.as_mut() {
            let _ = file.flush();
        }
    }
}

/// Rotate `<path>` to `<path>.1`, shifting existing backups up.
///
/// **External contract** — upstream `rotate` (`shared/logger.mjs:117-144`):
/// backups are `<path>.1` … `<path>.<maxFiles - 1>`; with `maxFiles == 1`
/// there are no backups and the live file is simply removed. A missing file at
/// any step is not an error.
fn rotate(path: &Path, max_files: u32) -> io::Result<()> {
    let backups = max_files.saturating_sub(1);
    if backups == 0 {
        return ignore_missing(std::fs::remove_file(path));
    }
    ignore_missing(std::fs::remove_file(numbered(path, backups)))?;
    for index in (1..backups).rev() {
        ignore_missing(std::fs::rename(
            numbered(path, index),
            numbered(path, index + 1),
        ))?;
    }
    ignore_missing(std::fs::rename(path, numbered(path, 1)))
}

fn numbered(path: &Path, index: u32) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_os_string();
    name.push(format!(".{index}"));
    PathBuf::from(name)
}

fn ignore_missing(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(unix)]
fn create_log_directory(directory: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(LOG_DIRECTORY_MODE)
        .create(directory)
}

#[cfg(not(unix))]
fn create_log_directory(directory: &Path) -> io::Result<()> {
    // Windows has no mode bits; the directory inherits its parent's ACL. This
    // is a recorded platform difference, not an oversight.
    std::fs::create_dir_all(directory)
}

pub(crate) fn absolute_path(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Console sink: human-readable lines split across stdout and stderr.
///
/// **External contract** — `warn` and above go to stderr, everything below to
/// stdout (upstream `shared/logger.mjs:307`). [`ConsoleSink::stderr_only`]
/// sends every level to stderr, for hosts that reserve stdout for a protocol.
#[derive(Clone, Copy, Debug, Default)]
pub struct ConsoleSink {
    stderr_only: bool,
}

impl ConsoleSink {
    /// The contract sink: `warn` and above to stderr, the rest to stdout.
    #[must_use]
    pub fn new() -> Self {
        Self { stderr_only: false }
    }

    /// Route every level to stderr.
    #[must_use]
    pub fn stderr_only() -> Self {
        Self { stderr_only: true }
    }
}

impl LogSink for ConsoleSink {
    fn write_record(&self, record: &LogRecord) {
        let line = record.to_console_line();
        // Errors are swallowed: logging must never interrupt the application.
        if self.stderr_only || record.level.goes_to_stderr() {
            let _ = io::stderr().write_all(line.as_bytes());
        } else {
            let _ = io::stdout().write_all(line.as_bytes());
        }
    }

    fn flush(&self) {
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();
    }
}

/// In-memory sink, for tests and for downstream crates that need to assert on
/// what was logged.
#[derive(Debug, Default)]
pub struct MemorySink {
    lines: Mutex<Vec<String>>,
}

impl MemorySink {
    /// An empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every line written so far, newline included.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        match self.lines.lock() {
            Ok(lines) => lines.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Every line written so far, parsed. Unparsable lines are skipped.
    #[must_use]
    pub fn records(&self) -> Vec<serde_json::Value> {
        self.lines()
            .iter()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    /// Drop everything written so far.
    pub fn clear(&self) {
        match self.lines.lock() {
            Ok(mut lines) => lines.clear(),
            Err(poisoned) => poisoned.into_inner().clear(),
        }
    }
}

impl LogSink for MemorySink {
    fn write_record(&self, record: &LogRecord) {
        let line = record.to_line();
        match self.lines.lock() {
            Ok(mut lines) => lines.push(line),
            Err(poisoned) => poisoned.into_inner().push(line),
        }
    }
}

/// Adapter that forwards only records at or above `threshold`.
pub struct LevelFilterSink {
    threshold: LogLevel,
    inner: Arc<dyn LogSink>,
}

impl LevelFilterSink {
    /// Wrap `inner`, dropping anything below `threshold`.
    #[must_use]
    pub fn new(threshold: LogLevel, inner: Arc<dyn LogSink>) -> Self {
        Self { threshold, inner }
    }
}

impl LogSink for LevelFilterSink {
    fn write_record(&self, record: &LogRecord) {
        if record.level.is_enabled_for(self.threshold) {
            self.inner.write_record(record);
        }
    }

    fn flush(&self) {
        self.inner.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_message_is_the_rebranded_contract_value() {
        assert_eq!(LOG_SINK_FAILURE_PREFIX_ZH, "VIA 日志写入失败：");
        assert_eq!(
            log_sink_failure_message("EACCES: permission denied"),
            "VIA 日志写入失败：EACCES: permission denied\n"
        );
    }

    #[test]
    fn rotation_defaults_are_the_contract_values() {
        assert_eq!(DEFAULT_MAX_BYTES, 10_485_760);
        assert_eq!(DEFAULT_MAX_FILES, 5);
        assert_eq!(MIN_MAX_BYTES, 1024);
        assert_eq!(MAX_MAX_BYTES, 1_073_741_824);
        assert_eq!(MIN_MAX_FILES, 1);
        assert_eq!(MAX_MAX_FILES, 100);
    }

    #[test]
    fn numbered_appends_the_backup_index() {
        assert_eq!(
            numbered(Path::new("/logs/gateway.log"), 2),
            PathBuf::from("/logs/gateway.log.2")
        );
    }
}
