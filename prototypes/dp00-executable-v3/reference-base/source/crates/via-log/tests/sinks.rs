//! File and console sink behaviour, including the ported rotation test and
//! the log-sink failure path.

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use via_log::{
    FileSinkOptions, JsonLineFileSink, LOG_SINK_FAILURE_PREFIX_ZH, LogLevel, LogRecord, LogSink,
    Logger, LoggerOptions, MemorySink, fields, log_sink_failure_message, no_fields,
};

fn file_logger(directory: &Path, level: LogLevel, max_bytes: u64, max_files: u32) -> Logger {
    let mut options = LoggerOptions::detached("gateway");
    options.directory = directory.to_path_buf();
    options.file_name = "gateway.log".to_owned();
    options.level = level;
    options.console_enabled = false;
    options.file_enabled = true;
    options.max_bytes = max_bytes;
    options.max_files = max_files;
    Logger::new(options)
}

fn records(path: &Path) -> Vec<Value> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

fn sorted_entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Ported from upstream `test/logger.test.mjs:29-60` — the file sink writes
/// standard JSONL, at mode `0o600`.
#[test]
fn writes_json_lines_at_mode_600() {
    let directory = tempfile::tempdir().expect("tempdir");
    let logger = file_logger(directory.path(), LogLevel::Info, 1024 * 1024, 5);
    let child = logger.child(fields([("provider", json!("dashscope"))]));

    via_log::run_with_log_context(
        fields([
            ("sessionId", json!("session-1")),
            ("turnId", json!("turn-2")),
        ]),
        || {
            child.info(
                "realtime.connected",
                fields([("durationMs", json!(42))]),
                "Realtime ready",
            );
        },
    );
    logger.flush();

    let path = directory.path().join("gateway.log");
    assert_eq!(logger.file_path(), Some(path.as_path()));
    let entries = records(&path);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["schema"], json!(via_log::LOG_SCHEMA));
    assert_eq!(entries[0]["level"], json!("info"));
    assert_eq!(entries[0]["component"], json!("gateway"));
    assert_eq!(entries[0]["event"], json!("realtime.connected"));
    assert_eq!(entries[0]["sessionId"], json!("session-1"));
    assert_eq!(entries[0]["turnId"], json!("turn-2"));
    assert_eq!(entries[0]["provider"], json!("dashscope"));
    assert_eq!(entries[0]["durationMs"], json!(42));
    assert_eq!(entries[0]["message"], json!("Realtime ready"));
    let time = entries[0]["time"].as_str().expect("time is a string");
    assert!(time.ends_with('Z'), "{time}");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file_mode = std::fs::metadata(&path).expect("stat").permissions().mode();
        assert_eq!(file_mode & 0o777, via_log::LOG_FILE_MODE);
        // The directory mode is asserted in `directory_is_created_lazily`; the
        // temp directory here already exists, and the sink deliberately leaves
        // an existing directory's mode alone (upstream's `mkdir` mode applies
        // only to directories it creates).
    }
}

/// Ported from upstream `test/logger.test.mjs:96-122` — "honors log levels and
/// rotates bounded files": with `maxFiles = 3` the directory settles at
/// `gateway.log`, `gateway.log.1`, `gateway.log.2`, and the suppressed `info`
/// record appears in none of them.
#[test]
fn honors_log_levels_and_rotates_bounded_files() {
    let directory = tempfile::tempdir().expect("tempdir");
    let logger = file_logger(directory.path(), LogLevel::Warn, 1024, 3);

    logger.info("ignored", fields([("value", json!("not written"))]), "");
    for index in 0..30 {
        logger.warn(
            "rotation.test",
            fields([("index", json!(index)), ("payload", json!("x".repeat(160)))]),
            "",
        );
    }
    logger.flush();

    assert_eq!(
        sorted_entries(directory.path()),
        ["gateway.log", "gateway.log.1", "gateway.log.2"]
    );

    let every_event: Vec<String> = sorted_entries(directory.path())
        .iter()
        .flat_map(|name| records(&directory.path().join(name)))
        .map(|record| record["event"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert!(!every_event.is_empty());
    assert!(
        !every_event.iter().any(|event| event == "ignored"),
        "a below-threshold record reached disk"
    );
}

/// `maxFiles = 1` means no backups at all: the live file is discarded rather
/// than renamed (upstream `shared/logger.mjs:119-125`).
#[test]
fn max_files_of_one_keeps_no_backups() {
    let directory = tempfile::tempdir().expect("tempdir");
    let logger = file_logger(directory.path(), LogLevel::Trace, 1024, 1);
    for index in 0..20 {
        logger.info(
            "rotation.test",
            fields([("index", json!(index)), ("payload", json!("y".repeat(160)))]),
            "",
        );
    }
    logger.flush();
    assert_eq!(sorted_entries(directory.path()), ["gateway.log"]);
}

/// An existing file is appended to, not truncated, and its size seeds the
/// rotation accounting.
#[test]
fn existing_file_is_appended_to() {
    let directory = tempfile::tempdir().expect("tempdir");
    {
        let logger = file_logger(directory.path(), LogLevel::Trace, 1024 * 1024, 5);
        logger.info("first", no_fields(), "");
        logger.flush();
    }
    {
        let logger = file_logger(directory.path(), LogLevel::Trace, 1024 * 1024, 5);
        logger.info("second", no_fields(), "");
        logger.flush();
    }
    let entries = records(&directory.path().join("gateway.log"));
    let events: Vec<&str> = entries
        .iter()
        .map(|record| record["event"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(events, ["first", "second"]);
}

/// The log directory is created on first write, not at construction.
#[test]
fn directory_is_created_lazily() {
    let root = tempfile::tempdir().expect("tempdir");
    let directory = root.path().join("nested").join("logs");
    let sink = JsonLineFileSink::new(FileSinkOptions::new(&directory, "gateway.log"));
    assert!(!directory.exists(), "construction must not touch the disk");

    sink.write_record(&LogRecord::new(
        LogLevel::Info,
        "gateway",
        "gateway.ready",
        no_fields(),
        "",
    ));
    assert!(directory.exists());
    assert_eq!(records(&directory.join("gateway.log")).len(), 1);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&directory)
            .expect("stat")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, via_log::LOG_DIRECTORY_MODE);
    }
}

/// A failing sink reports once per transition into the failed state, carries
/// the rebranded message, and never propagates the error to the caller.
#[test]
fn sink_failure_is_reported_once_and_never_propagates() {
    let root = tempfile::tempdir().expect("tempdir");
    // A regular file where the log *directory* should be: `mkdir` cannot
    // succeed, so every write fails.
    let blocked = root.path().join("logs");
    std::fs::write(&blocked, b"not a directory").expect("seed blocker");

    let reported: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&reported);
    let sink = JsonLineFileSink::new(FileSinkOptions {
        directory: blocked.clone(),
        file_name: "gateway.log".to_owned(),
        max_bytes: 1024 * 1024,
        max_files: 5,
        on_error: Some(Arc::new(move |detail: &str| {
            captured
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(detail.to_owned());
        })),
    });

    for _ in 0..3 {
        sink.write_record(&LogRecord::new(
            LogLevel::Error,
            "gateway",
            "gateway.failed",
            no_fields(),
            "",
        ));
    }

    let messages = reported
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        messages.len(),
        1,
        "a wedged sink must not flood: {messages:?}"
    );
    let rendered = log_sink_failure_message(&messages[0]);
    assert!(
        rendered.starts_with(LOG_SINK_FAILURE_PREFIX_ZH),
        "{rendered}"
    );
    assert!(rendered.ends_with('\n'), "{rendered}");
    assert!(!blocked.is_dir());
}

/// A logger with the file sink disabled writes no file at all.
#[test]
fn file_sink_can_be_disabled() {
    let directory = tempfile::tempdir().expect("tempdir");
    let mut options = LoggerOptions::detached("gateway");
    options.directory = directory.path().to_path_buf();
    options.console_enabled = false;
    options.file_enabled = false;
    let logger = Logger::new(options);
    logger.info("gateway.ready", no_fields(), "");
    logger.flush();

    assert_eq!(logger.file_path(), None);
    assert!(sorted_entries(directory.path()).is_empty());
}

/// `Logger::log_record` emits a caller-assembled record verbatim — except that
/// it is redacted first.
#[test]
fn log_record_emits_a_caller_assembled_record() {
    let sink = Arc::new(MemorySink::new());
    let mut options = LoggerOptions::detached("gateway");
    options.level = LogLevel::Trace;
    let logger = Logger::with_sinks(options, vec![sink.clone()]);

    let mut record = LogRecord::new(
        LogLevel::Fatal,
        "replay",
        "gateway.crashed",
        fields([("apiKey", json!("sk-abcdefgh1234"))]),
        "restoring from snapshot",
    );
    record.time = "2026-08-22T10:36:00.123Z".to_owned();
    logger.log_record(&record);

    let emitted = &sink.records()[0];
    assert_eq!(emitted["time"], json!("2026-08-22T10:36:00.123Z"));
    assert_eq!(emitted["level"], json!("fatal"));
    assert_eq!(emitted["component"], json!("replay"));
    assert_eq!(emitted["event"], json!("gateway.crashed"));
    assert_eq!(emitted["apiKey"], json!(via_log::REDACTED));
    assert_eq!(emitted["message"], json!("restoring from snapshot"));
}

/// The free `log_record` is a no-op until a global logger is installed, then
/// routes to it.
#[test]
fn global_log_record_routes_to_the_installed_logger() {
    // No global installed yet in this test binary: must not panic.
    let orphan = LogRecord::new(LogLevel::Info, "gateway", "before.install", no_fields(), "");
    if via_log::global_logger().is_none() {
        via_log::log_record(&orphan);
    }

    let sink = Arc::new(MemorySink::new());
    let mut options = LoggerOptions::detached("gateway");
    options.level = LogLevel::Trace;
    let logger = Logger::with_sinks(options, vec![sink.clone()]);
    if via_log::set_global_logger(logger).is_ok() {
        via_log::log_record(&LogRecord::new(
            LogLevel::Info,
            "gateway",
            "after.install",
            no_fields(),
            "",
        ));
        assert_eq!(sink.records()[0]["event"], json!("after.install"));
    }
}

/// Sinks compose: the same record reaches every attached sink.
#[test]
fn multiple_sinks_receive_the_same_record() {
    let first = Arc::new(MemorySink::new());
    let second = Arc::new(MemorySink::new());
    let mut options = LoggerOptions::detached("gateway");
    options.level = LogLevel::Trace;
    let sinks: Vec<Arc<dyn LogSink>> = vec![first.clone(), second.clone()];
    let logger = Logger::with_sinks(options, sinks);
    logger.info("gateway.ready", no_fields(), "");
    assert_eq!(first.lines(), second.lines());
    assert_eq!(first.lines().len(), 1);
}

/// Sinks can carry their own threshold, so the file can keep `trace` while the
/// console stays at `warn`.
#[test]
fn level_filter_sink_narrows_one_destination() {
    let verbose = Arc::new(MemorySink::new());
    let quiet = Arc::new(MemorySink::new());
    let mut options = LoggerOptions::detached("gateway");
    options.level = LogLevel::Trace;
    let sinks: Vec<Arc<dyn LogSink>> = vec![
        verbose.clone(),
        Arc::new(via_log::LevelFilterSink::new(LogLevel::Warn, quiet.clone())),
    ];
    let logger = Logger::with_sinks(options, sinks);

    logger.debug("noisy", no_fields(), "");
    logger.warn("loud", no_fields(), "");

    assert_eq!(verbose.records().len(), 2);
    assert_eq!(quiet.records().len(), 1);
    assert_eq!(quiet.records()[0]["event"], json!("loud"));
}

/// A default-constructed [`FileSinkOptions`] resolves a relative directory to
/// an absolute path, matching upstream's `resolve()`.
#[test]
fn relative_directories_are_made_absolute() {
    let sink = JsonLineFileSink::new(FileSinkOptions::new("relative-logs", "gateway.log"));
    assert!(sink.path().is_absolute(), "{:?}", sink.path());
    assert!(
        sink.path()
            .ends_with(PathBuf::from("relative-logs/gateway.log"))
    );
    assert_eq!(sink.max_bytes(), via_log::DEFAULT_MAX_BYTES);
    assert_eq!(sink.max_files(), via_log::DEFAULT_MAX_FILES);
}
