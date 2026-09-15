//! Contract assertions for `via-log`.
//!
//! Every value asserted here appears in `docs/reference/contracts.json` or in
//! upstream `shared/logger.mjs` / `server/src/core/logger.mjs`. The intent is
//! that changing a literal in the crate turns this file red, not that the code
//! is merely exercised.

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use via_log::{
    DEFAULT_COMPONENT, DEFAULT_EVENT, DEFAULT_LOG_LEVEL, DEFAULT_MAX_BYTES, DEFAULT_MAX_FILES,
    ENV_CONFIG_DIR, ENV_LOG_CONSOLE, ENV_LOG_DIR, ENV_LOG_FILE, ENV_LOG_LEVEL, ENV_LOG_MAX_BYTES,
    ENV_LOG_MAX_FILES, ENV_XDG_CONFIG_HOME, EnvSettings, GATEWAY_COMPONENT, GATEWAY_LOG_FILE_NAME,
    LOG_SCHEMA, LOG_SINK_FAILURE_PREFIX_ZH, LogLevel, LogRecord, Logger, LoggerOptions, MemorySink,
    RESERVED_FIELDS, default_log_directory, fields, log_sink_failure_message, no_fields,
    safe_event_name,
};

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn memory_logger(level: LogLevel) -> (Logger, Arc<MemorySink>) {
    let sink = Arc::new(MemorySink::new());
    let mut options = LoggerOptions::detached(GATEWAY_COMPONENT);
    options.level = level;
    (Logger::with_sinks(options, vec![sink.clone()]), sink)
}

fn keys(record: &Value) -> Vec<String> {
    record
        .as_object()
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

// ── 1. The record spine ─────────────────────────────────────────────────────

/// contracts.json `log-schema` / `LOG_SCHEMA`, rebranded per docs/rebrand.md
/// (`qwaudio.log/v1` → `via.log/v1`).
#[test]
fn log_schema_is_via_log_v1() {
    assert_eq!(LOG_SCHEMA, "via.log/v1");
    assert!(
        !LOG_SCHEMA.contains("qwaudio") && !LOG_SCHEMA.contains("qwen"),
        "the schema string must carry no upstream identity"
    );
}

/// contracts.json `file-format` / `log record envelope`:
/// `{...base, ...context, ...fields, schema, time, level, component, event,
/// pid, message?}`.
#[test]
fn record_field_order_is_the_contract_envelope() {
    let (logger, sink) = memory_logger(LogLevel::Trace);
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

    let records = sink.records();
    assert_eq!(records.len(), 1);
    assert_eq!(
        keys(&records[0]),
        [
            // base (from `child`)
            "provider",
            // async/log context
            "sessionId",
            "turnId",
            // caller fields
            "durationMs",
            // the spine, written last
            "schema",
            "time",
            "level",
            "component",
            "event",
            "pid",
            "message",
        ]
    );
    assert_eq!(records[0]["schema"], json!(LOG_SCHEMA));
    assert_eq!(records[0]["level"], json!("info"));
    assert_eq!(records[0]["component"], json!("gateway"));
    assert_eq!(records[0]["event"], json!("realtime.connected"));
    assert_eq!(records[0]["provider"], json!("dashscope"));
    assert_eq!(records[0]["sessionId"], json!("session-1"));
    assert_eq!(records[0]["turnId"], json!("turn-2"));
    assert_eq!(records[0]["durationMs"], json!(42));
    assert_eq!(records[0]["message"], json!("Realtime ready"));
    assert_eq!(records[0]["pid"], json!(std::process::id()));
}

/// contracts.json `log-schema`: "Caller-supplied fields named
/// schema/level/component/event/pid MUST be ignored, not merged."
/// Ported from upstream `test/logger.test.mjs:133-156`.
#[test]
fn diagnostic_fields_cannot_replace_the_envelope() {
    let (logger, sink) = memory_logger(LogLevel::Trace);
    logger.info(
        "gateway.ready",
        fields([
            ("schema", json!("untrusted")),
            ("level", json!("fatal")),
            ("component", json!("other")),
            ("event", json!("other")),
            ("pid", json!(-1)),
        ]),
        "",
    );

    let record = &sink.records()[0];
    assert_eq!(record["schema"], json!(LOG_SCHEMA));
    assert_eq!(record["level"], json!("info"));
    assert_eq!(record["component"], json!("gateway"));
    assert_eq!(record["event"], json!("gateway.ready"));
    assert_eq!(record["pid"], json!(std::process::id()));
    // JavaScript object spread overwrites in place, so the colliding keys keep
    // their original positions and no duplicate is appended. `preserve_order`
    // reproduces that exactly.
    assert_eq!(
        keys(record),
        ["schema", "level", "component", "event", "pid", "time"]
    );
}

/// Upstream spreads `message` only when the call site supplied one, so a
/// `message` *field* survives an emit that passes none
/// (`shared/logger.mjs:303`). It is the one reserved name that is not forced.
#[test]
fn a_message_field_survives_when_no_message_is_passed() {
    let (logger, sink) = memory_logger(LogLevel::Trace);
    logger.info(
        "gateway.ready",
        fields([("message", json!("from field"))]),
        "",
    );
    assert_eq!(sink.records()[0]["message"], json!("from field"));

    sink.clear();
    logger.info(
        "gateway.ready",
        fields([("message", json!("from field"))]),
        "from argument",
    );
    assert_eq!(sink.records()[0]["message"], json!("from argument"));
}

/// contracts.json `file-format`: the console line format, and the `details`
/// object that omits the seven reserved keys.
#[test]
fn console_line_matches_the_contract_format() {
    let record = LogRecord::new(
        LogLevel::Warn,
        "gateway",
        "realtime.disconnected",
        fields([("code", json!(1006))]),
        "socket closed",
    );
    let line = record.to_console_line();
    assert_eq!(
        line,
        format!(
            "{} WARN gateway realtime.disconnected: socket closed {{\"code\":1006}}\n",
            record.time
        )
    );

    let bare = LogRecord::new(LogLevel::Info, "gateway", "gateway.ready", no_fields(), "");
    assert_eq!(
        bare.to_console_line(),
        format!("{} INFO gateway gateway.ready\n", bare.time)
    );
}

/// contracts.json `file-format`: `details` omits
/// schema/time/level/component/event/pid/message.
#[test]
fn reserved_fields_are_the_seven_spine_keys() {
    assert_eq!(
        RESERVED_FIELDS,
        [
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

/// Upstream `safeEventName` / `String(component || 'app')`.
#[test]
fn event_and_component_fallbacks() {
    assert_eq!(DEFAULT_EVENT, "log");
    assert_eq!(DEFAULT_COMPONENT, "app");
    assert_eq!(safe_event_name(""), "log");
    assert_eq!(safe_event_name("   "), "log");

    let record = LogRecord::new(LogLevel::Info, "", "", no_fields(), "");
    assert_eq!(record.component, "app");
    assert_eq!(record.event, "log");
}

/// The Gateway logger's identity (upstream `server/src/core/logger.mjs:11-16`).
#[test]
fn gateway_logger_identity() {
    assert_eq!(GATEWAY_COMPONENT, "gateway");
    assert_eq!(GATEWAY_LOG_FILE_NAME, "gateway.log");

    let options = LoggerOptions::gateway(&env(&[]), Path::new("/home/tester"));
    assert_eq!(options.component, "gateway");
    assert_eq!(options.file_name, "gateway.log");
}

// ── 2. Levels ───────────────────────────────────────────────────────────────

/// contracts.json `default-value` / `LOG_LEVELS numeric mapping and stream
/// split`: trace:10, debug:20, info:30, warn:40, error:50, fatal:60,
/// silent:+Infinity; default 'info'; warn and above go to stderr.
#[test]
fn log_levels_are_the_contract_mapping() {
    let expected: [(LogLevel, &str, f64); 7] = [
        (LogLevel::Trace, "trace", 10.0),
        (LogLevel::Debug, "debug", 20.0),
        (LogLevel::Info, "info", 30.0),
        (LogLevel::Warn, "warn", 40.0),
        (LogLevel::Error, "error", 50.0),
        (LogLevel::Fatal, "fatal", 60.0),
        (LogLevel::Silent, "silent", f64::INFINITY),
    ];
    assert_eq!(LogLevel::ALL.len(), 7);
    for (index, (level, name, severity)) in expected.into_iter().enumerate() {
        assert_eq!(LogLevel::ALL[index], level);
        assert_eq!(level.as_str(), name);
        assert_eq!(level.severity(), severity);
        assert_eq!(
            serde_json::to_value(level).expect("levels serialize"),
            json!(name)
        );
    }
    assert_eq!(DEFAULT_LOG_LEVEL, LogLevel::Info);

    for level in LogLevel::EMITTABLE {
        assert_eq!(level.goes_to_stderr(), level.severity() >= 40.0, "{level}");
    }
}

/// `VIA_LOG_LEVEL` accepts exactly the seven names; anything else is `info`.
#[test]
fn unknown_level_names_fall_back_to_info() {
    // Recognised names, including the trimming and lowercasing upstream applies.
    for (name, expected) in [
        ("trace", LogLevel::Trace),
        ("DEBUG", LogLevel::Debug),
        (" info ", LogLevel::Info),
        ("Warn", LogLevel::Warn),
        ("error", LogLevel::Error),
        ("FATAL", LogLevel::Fatal),
        ("\tsilent\n", LogLevel::Silent),
    ] {
        assert_eq!(
            LogLevel::normalize(name, LogLevel::Fatal),
            expected,
            "{name:?}",
        );
    }
    for name in ["", "verbose", "warning", "off", "10", "nope"] {
        assert_eq!(
            LogLevel::normalize(name, LogLevel::Info),
            LogLevel::Info,
            "{name}",
        );
    }
}

/// Upstream `if (LOG_LEVELS[entryLevel] < threshold) return`.
#[test]
fn threshold_suppresses_lower_levels() {
    let (logger, sink) = memory_logger(LogLevel::Warn);
    logger.trace("t", no_fields(), "");
    logger.debug("d", no_fields(), "");
    logger.info("ignored", no_fields(), "");
    logger.warn("w", no_fields(), "");
    logger.error("e", no_fields(), "");
    logger.fatal("f", no_fields(), "");

    let events: Vec<String> = sink
        .records()
        .iter()
        .map(|record| record["event"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(events, ["w", "e", "f"]);
}

#[test]
fn silent_suppresses_everything() {
    let (logger, sink) = memory_logger(LogLevel::Silent);
    for level in LogLevel::EMITTABLE {
        logger.emit(level, "anything", no_fields(), "");
    }
    assert!(sink.records().is_empty());
}

// ── 4. Sinks ────────────────────────────────────────────────────────────────

/// contracts.json `default-value` / `log rotation defaults and bounds`.
#[test]
fn rotation_defaults_and_bounds() {
    assert_eq!(DEFAULT_MAX_BYTES, 10_485_760);
    assert_eq!(DEFAULT_MAX_FILES, 5);

    let settings = EnvSettings::from_env(&env(&[]), Path::new("/home/tester"));
    assert_eq!(settings.max_bytes, 10_485_760);
    assert_eq!(settings.max_files, 5);

    let clamped_low = EnvSettings::from_env(
        &env(&[(ENV_LOG_MAX_BYTES, "1"), (ENV_LOG_MAX_FILES, "0")]),
        Path::new("/home/tester"),
    );
    assert_eq!(clamped_low.max_bytes, 1024);
    assert_eq!(clamped_low.max_files, 1);

    let clamped_high = EnvSettings::from_env(
        &env(&[
            (ENV_LOG_MAX_BYTES, "99999999999"),
            (ENV_LOG_MAX_FILES, "9999"),
        ]),
        Path::new("/home/tester"),
    );
    assert_eq!(clamped_high.max_bytes, 1_073_741_824);
    assert_eq!(clamped_high.max_files, 100);

    // Unparsable values fall back rather than clamping.
    let garbage = EnvSettings::from_env(
        &env(&[(ENV_LOG_MAX_BYTES, "lots"), (ENV_LOG_MAX_FILES, "many")]),
        Path::new("/home/tester"),
    );
    assert_eq!(garbage.max_bytes, 10_485_760);
    assert_eq!(garbage.max_files, 5);
}

/// The stderr message written when the log sink itself fails.
/// Upstream `shared/logger.mjs:280`, rebranded per docs/rebrand.md:
/// `qwen-audio-agent 日志写入失败：<message>` → `VIA 日志写入失败：<message>`.
#[test]
fn log_sink_failure_message_is_the_rebranded_zh_contract() {
    assert_eq!(LOG_SINK_FAILURE_PREFIX_ZH, "VIA 日志写入失败：");
    assert!(
        !LOG_SINK_FAILURE_PREFIX_ZH.contains("qwen"),
        "the product name must be rebranded"
    );
    assert_eq!(
        log_sink_failure_message("EACCES: permission denied, open '/x/gateway.log'"),
        "VIA 日志写入失败：EACCES: permission denied, open '/x/gateway.log'\n"
    );
}

// ── Environment ─────────────────────────────────────────────────────────────

/// contracts.json `file-path` / `default log directory resolution order`,
/// rebranded: `VIA_LOG_DIR` > `$VIA_CONFIG_DIR/logs` >
/// `$XDG_CONFIG_HOME/via/logs` > `~/.config/via/logs`.
#[test]
fn log_directory_resolution_order() {
    let home = Path::new("/home/tester");

    assert_eq!(
        default_log_directory(
            &env(&[
                (ENV_LOG_DIR, "/var/log/via"),
                (ENV_CONFIG_DIR, "/opt/config"),
                (ENV_XDG_CONFIG_HOME, "/opt/xdg"),
            ]),
            home
        ),
        PathBuf::from("/var/log/via")
    );

    assert_eq!(
        default_log_directory(
            &env(&[
                (ENV_CONFIG_DIR, "/opt/config"),
                (ENV_XDG_CONFIG_HOME, "/opt/xdg"),
            ]),
            home
        ),
        PathBuf::from("/opt/config/logs")
    );

    assert_eq!(
        default_log_directory(&env(&[(ENV_XDG_CONFIG_HOME, "/opt/xdg")]), home),
        PathBuf::from("/opt/xdg/via/logs")
    );

    assert_eq!(
        default_log_directory(&env(&[]), home),
        PathBuf::from("/home/tester/.config/via/logs")
    );
}

/// The rebranded environment variable names (docs/rebrand.md).
#[test]
fn environment_variable_names_are_rebranded() {
    assert_eq!(ENV_LOG_LEVEL, "VIA_LOG_LEVEL");
    assert_eq!(ENV_LOG_DIR, "VIA_LOG_DIR");
    assert_eq!(ENV_LOG_CONSOLE, "VIA_LOG_CONSOLE");
    assert_eq!(ENV_LOG_FILE, "VIA_LOG_FILE");
    assert_eq!(ENV_LOG_MAX_BYTES, "VIA_LOG_MAX_BYTES");
    assert_eq!(ENV_LOG_MAX_FILES, "VIA_LOG_MAX_FILES");
    assert_eq!(ENV_CONFIG_DIR, "VIA_CONFIG_DIR");
    assert_eq!(ENV_XDG_CONFIG_HOME, "XDG_CONFIG_HOME");
    assert_eq!(via_log::CONFIG_DIRECTORY_NAME, "via");
    assert_eq!(via_log::LOG_DIRECTORY_NAME, "logs");
}

/// contracts.json `env-var` / `logging`: the console and file switches disable
/// their sink only when the value is exactly `"0"`.
#[test]
fn console_and_file_switches_only_honour_exactly_zero() {
    let home = Path::new("/home/tester");
    for (value, enabled) in [
        ("0", false),
        ("1", true),
        ("false", true),
        ("", true),
        ("00", true),
        (" 0", true),
    ] {
        let settings = EnvSettings::from_env(&env(&[(ENV_LOG_CONSOLE, value)]), home);
        assert_eq!(settings.console_enabled, enabled, "console={value:?}");
        let settings = EnvSettings::from_env(&env(&[(ENV_LOG_FILE, value)]), home);
        assert_eq!(settings.file_enabled, enabled, "file={value:?}");
    }
    let unset = EnvSettings::from_env(&env(&[]), home);
    assert!(unset.console_enabled);
    assert!(unset.file_enabled);
}

/// contracts.json `env-var` / `logging`: `VIA_LOG_LEVEL` default `info`, with
/// unknown values falling back to `info`.
#[test]
fn level_comes_from_the_environment() {
    let home = Path::new("/home/tester");
    assert_eq!(
        EnvSettings::from_env(&env(&[(ENV_LOG_LEVEL, "debug")]), home).level,
        LogLevel::Debug
    );
    assert_eq!(
        EnvSettings::from_env(&env(&[(ENV_LOG_LEVEL, "shouty")]), home).level,
        LogLevel::Info
    );
    assert_eq!(EnvSettings::from_env(&env(&[]), home).level, LogLevel::Info);
}

/// contracts.json `env-var` / `logging`: a test process disables BOTH sinks.
#[test]
fn test_process_detection() {
    assert!(via_log::is_test_process(
        &env(&[("NODE_ENV", "test")]),
        Vec::<String>::new()
    ));
    assert!(via_log::is_test_process(
        &env(&[]),
        ["/repo/crates/via-log/tests/contracts.rs"]
    ));
    assert!(!via_log::is_test_process(
        &env(&[("NODE_ENV", "production")]),
        ["/repo/target/debug/via"]
    ));
}
