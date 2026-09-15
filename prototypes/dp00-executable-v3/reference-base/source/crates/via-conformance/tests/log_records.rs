//! `via.log/v1` — the record spine, the levels, the redaction patterns and the
//! log directory.
//!
//! Eleven catalogue rows, and every one of them is read by something outside
//! this program: support tooling parses the JSON lines, a human reads the
//! console lines, and the redaction patterns are the only thing standing
//! between a `DASHSCOPE_API_KEY` and a file on disk.
//!
//! Six rows are `Divergent`. Two renames account for all six —
//! `docs/rebrand.md` renames the upstream log schema string and the upstream
//! `QWEN_AUDIO_LOG_*` / config-dir environment prefixes — plus one spelling
//! difference: JavaScript's `\b` is ASCII-only and Rust's is Unicode-aware, so
//! the four value patterns carry `(?-u:\b)` where upstream carries `\b`. Every
//! one of those is *derived* from the upstream value here rather than retyped,
//! so a half-applied rename or a hand-edited regex fails.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;
use serde_json::{Value, json};

use via_conformance::expect_contract;
use via_conformance::value::{
    field_name, js_object_fields, list, quoted_literals, rebranded, rebranded_schema, unquote,
};
use via_log::{
    API_KEY_VALUE_PATTERN, AUTH_VALUE_PATTERN, CIRCULAR, CONFIG_DIRECTORY_NAME, DEFAULT_LOG_LEVEL,
    DEFAULT_MAX_BYTES, DEFAULT_MAX_FILES, ENV_CONFIG_DIR, ENV_LOG_CONSOLE, ENV_LOG_DIR,
    ENV_LOG_FILE, ENV_LOG_LEVEL, ENV_LOG_MAX_BYTES, ENV_LOG_MAX_FILES, ENV_NODE_ENV,
    ENV_TEST_PROCESS, ENV_XDG_CONFIG_HOME, EnvSettings, FileSinkOptions, GATEWAY_COMPONENT,
    GATEWAY_LOG_FILE_NAME, JsonLineFileSink, LOG_DIRECTORY_MODE, LOG_DIRECTORY_NAME, LOG_FILE_MODE,
    LOG_SCHEMA, LogLevel, LogRecord, LogSink, LoggerOptions, MAX_COLLECTION_ITEMS, MAX_DEPTH,
    MAX_DEPTH_MARKER, MAX_MAX_BYTES, MAX_MAX_FILES, MAX_STRING_CHARS, MIN_MAX_BYTES, MIN_MAX_FILES,
    NODE_ENV_TEST, REDACTED, RESERVED_FIELDS, SENSITIVE_KEY_PATTERN, TEST_PROCESS_VALUE,
    TRUNCATED_MARKER, TRUNCATED_SUFFIX, URL_SECRET_VALUE_PATTERN, default_log_directory, fields,
    is_sensitive_key, is_test_process, no_fields, scrub_string,
};

/// Every SCREAMING_SNAKE_CASE identifier in a catalogued value, in order,
/// without repeats.
///
/// An identifier must contain an underscore, which is what tells
/// `QWEN_AUDIO_LOG_DIR` and `NODE_ENV` apart from a shouted English word like
/// `BOTH` in the same sentence.
fn env_names(value: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut current = String::new();
    for character in value.chars().chain(std::iter::once(' ')) {
        if character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_' {
            current.push(character);
            continue;
        }
        if current.contains('_') && current.len() > 1 && !found.contains(&current) {
            found.push(current.clone());
        }
        current.clear();
    }
    found
}

/// Translate a JavaScript regex literal `/body/flags` into the Rust pattern it
/// must become.
///
/// Two mechanical rules, and no others:
///
/// * the `i` flag becomes an inline `(?i)`; the `g` flag has no Rust
///   counterpart because "replace every match" is `Regex::replace_all`, which
///   is how this crate uses all three value patterns;
/// * `\b` becomes `(?-u:\b)`, because JavaScript's word boundary is
///   ASCII-only and Rust's default `\b` is Unicode-aware. The behaviour is
///   identical; only the spelling differs, which is why the rows that carry a
///   pattern are `Divergent` rather than `Asserted`.
fn js_regex_to_rust(literal: &str) -> String {
    let literal = literal.trim();
    let body_and_flags = literal
        .strip_prefix('/')
        .unwrap_or_else(|| panic!("`{literal}` is not a JavaScript regex literal"));
    let last = body_and_flags
        .rfind('/')
        .unwrap_or_else(|| panic!("`{literal}` has no closing slash"));
    let (body, flags) = body_and_flags.split_at(last);
    let flags = &flags[1..];
    assert!(
        flags.chars().all(|flag| flag == 'i' || flag == 'g'),
        "unhandled regex flags in `{literal}`"
    );
    let prefix = if flags.contains('i') { "(?i)" } else { "" };
    format!("{prefix}{}", body.replace(r"\b", r"(?-u:\b)"))
}

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

#[test]
fn log_levels_numeric_mapping_and_stream_split() {
    let contract = expect_contract(
        "default-value",
        "LOG_LEVELS numeric mapping and stream split",
    );
    let (mapping, rest) = contract
        .exact_value
        .split_once(';')
        .expect("the mapping is the first clause");

    // `trace:10, debug:20, …, silent:+Infinity`, parsed rather than retyped.
    let catalogued: Vec<(&str, &str)> = list(mapping)
        .into_iter()
        .map(|pair| {
            pair.split_once(':')
                .unwrap_or_else(|| panic!("`{pair}` is not `name:severity`"))
        })
        .collect();
    assert_eq!(catalogued.len(), 7, "{}", contract.file);
    assert_eq!(
        catalogued.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        LogLevel::ALL.iter().map(|l| l.as_str()).collect::<Vec<_>>(),
        "the level names, in upstream's declaration order"
    );
    for ((name, severity), level) in catalogued.iter().zip(LogLevel::ALL) {
        if *severity == "+Infinity" {
            assert!(level.severity().is_infinite() && level.severity().is_sign_positive());
        } else {
            let expected: f64 = severity
                .parse()
                .unwrap_or_else(|_| panic!("`{severity}` for `{name}` is not a number"));
            assert_eq!(level.severity(), expected, "severity of `{name}`");
        }
    }

    // `default level 'info'`
    assert!(rest.contains("default level 'info'"));
    assert_eq!(DEFAULT_LOG_LEVEL, LogLevel::Info);
    assert_eq!(DEFAULT_LOG_LEVEL.as_str(), "info");
    // An unrecognised name falls back to it rather than failing.
    for unknown in ["", "verbose", "warning", "10", "silent!"] {
        assert_eq!(
            LogLevel::normalize(unknown, DEFAULT_LOG_LEVEL),
            LogLevel::Info
        );
    }
    // …but the seven real names, in any case, resolve to themselves.
    for level in LogLevel::ALL {
        assert_eq!(
            LogLevel::normalize(&level.as_str().to_uppercase(), LogLevel::Trace),
            level
        );
    }

    // `warn and above go to stderr, below to stdout`
    assert!(rest.contains("warn and above go to stderr, below to stdout"));
    for level in LogLevel::ALL {
        assert_eq!(
            level.goes_to_stderr(),
            level >= LogLevel::Warn,
            "stream for `{level}`"
        );
    }

    // The threshold comparison is `>=`, and `silent` therefore suppresses
    // everything — the whole point of `+Infinity`.
    assert!(LogLevel::Info.is_enabled_for(LogLevel::Info));
    assert!(!LogLevel::Debug.is_enabled_for(LogLevel::Info));
    for level in LogLevel::EMITTABLE {
        assert!(!level.is_enabled_for(LogLevel::Silent));
    }
}

#[test]
fn log_rotation_defaults_and_bounds() {
    let contract = expect_contract("default-value", "log rotation defaults and bounds");
    let value = &contract.exact_value;

    assert!(
        value.contains("DEFAULT_MAX_BYTES = 10485760 (10 MiB)"),
        "{value}"
    );
    assert!(value.contains("DEFAULT_MAX_FILES = 5"), "{value}");
    assert!(
        value.contains("MAX_BYTES [1024, 1073741824] and MAX_FILES [1, 100]"),
        "{value}"
    );
    assert!(
        value.contains("backups named <path>.1 through <path>.<maxFiles-1>"),
        "{value}"
    );
    assert!(
        value.contains("directory mode 0o700, file mode 0o600"),
        "{value}"
    );

    assert_eq!(DEFAULT_MAX_BYTES, 10_485_760);
    assert_eq!(DEFAULT_MAX_BYTES, 10 * 1024 * 1024, "10 MiB, not 10 MB");
    assert_eq!(DEFAULT_MAX_FILES, 5);
    assert_eq!(MIN_MAX_BYTES, 1024);
    assert_eq!(MAX_MAX_BYTES, 1_073_741_824);
    assert_eq!(MIN_MAX_FILES, 1);
    assert_eq!(MAX_MAX_FILES, 100);
    assert_eq!(LOG_DIRECTORY_MODE, 0o700);
    assert_eq!(LOG_FILE_MODE, 0o600);

    // Clamping, in both directions and at both edges — out-of-range values are
    // silently corrected, never rejected.
    let home = Path::new("/home/tester");
    for (raw, expected_bytes) in [
        ("0", MIN_MAX_BYTES),
        ("1023", MIN_MAX_BYTES),
        ("1024", 1024),
        ("1073741825", MAX_MAX_BYTES),
        ("nonsense", DEFAULT_MAX_BYTES),
    ] {
        let settings = EnvSettings::from_env(&env(&[(ENV_LOG_MAX_BYTES, raw)]), home);
        assert_eq!(settings.max_bytes, expected_bytes, "MAX_BYTES={raw}");
    }
    for (raw, expected_files) in [
        ("0", MIN_MAX_FILES),
        ("1", 1),
        ("101", MAX_MAX_FILES),
        ("-5", MIN_MAX_FILES),
        ("", DEFAULT_MAX_FILES),
    ] {
        let settings = EnvSettings::from_env(&env(&[(ENV_LOG_MAX_FILES, raw)]), home);
        assert_eq!(settings.max_files, expected_files, "MAX_FILES={raw}");
    }

    // The backup names, produced by a real rotation rather than asserted from
    // a helper: `<path>.1` … `<path>.<maxFiles-1>`, and nothing past it.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let sink = JsonLineFileSink::new(FileSinkOptions {
        directory: dir.path().to_path_buf(),
        file_name: GATEWAY_LOG_FILE_NAME.to_owned(),
        max_bytes: MIN_MAX_BYTES,
        max_files: 3,
        on_error: None,
    });
    let record = |index: usize| LogRecord {
        extra: fields([("padding", json!("x".repeat(600))), ("index", json!(index))]),
        schema: LOG_SCHEMA.to_owned(),
        time: "2026-08-22T10:36:00.000Z".to_owned(),
        level: LogLevel::Info,
        component: GATEWAY_COMPONENT.to_owned(),
        event: "rotate.probe".to_owned(),
        pid: 4242,
        message: None,
    };
    for index in 0..8 {
        sink.write_record(&record(index));
    }
    let base = dir.path().join(GATEWAY_LOG_FILE_NAME);
    assert!(base.exists(), "the live file");
    assert!(base.with_extension("log.1").exists(), "<path>.1");
    assert!(base.with_extension("log.2").exists(), "<path>.2");
    assert!(
        !base.with_extension("log.3").exists(),
        "maxFiles = 3 keeps two backups, not three"
    );
}

#[test]
fn redaction_patterns_and_caps() {
    let contract = expect_contract("default-value", "logger redaction patterns and caps");
    let mut catalogued: BTreeMap<&str, &str> = BTreeMap::new();
    for clause in contract.exact_value.split(';') {
        if let Some((name, value)) = clause.split_once('=') {
            catalogued.insert(name.trim(), value.trim());
        }
    }

    assert_eq!(unquote(catalogued["REDACTED"]), REDACTED);
    assert_eq!(REDACTED, "[REDACTED]");

    // The four patterns, each translated from upstream's literal by the two
    // documented rules rather than retyped.
    for (name, shipped) in [
        ("SENSITIVE_KEY", SENSITIVE_KEY_PATTERN),
        ("AUTH_VALUE", AUTH_VALUE_PATTERN),
        ("API_KEY_VALUE", API_KEY_VALUE_PATTERN),
        ("URL_SECRET_VALUE", URL_SECRET_VALUE_PATTERN),
    ] {
        let literal = catalogued
            .get(name)
            .unwrap_or_else(|| panic!("the catalogue no longer carries `{name}`"));
        assert_eq!(
            js_regex_to_rust(literal),
            shipped,
            "`{name}` must be upstream's pattern with only the two documented \
             spelling changes"
        );
    }

    // The caps and their markers.
    assert_eq!(
        catalogued["MAX_STRING_CHARS"]
            .split_whitespace()
            .next()
            .expect("a number"),
        MAX_STRING_CHARS.to_string()
    );
    assert_eq!(MAX_STRING_CHARS, 32_000);
    assert_eq!(
        quoted_literals(catalogued["MAX_STRING_CHARS"]),
        [TRUNCATED_SUFFIX]
    );
    let leading_number = |clause: &str| {
        clause
            .split_whitespace()
            .next()
            .expect("a number")
            .to_owned()
    };
    assert_eq!(
        leading_number(catalogued["MAX_COLLECTION_ITEMS"]),
        MAX_COLLECTION_ITEMS.to_string()
    );
    assert_eq!(
        quoted_literals(catalogued["MAX_COLLECTION_ITEMS"]),
        [TRUNCATED_MARKER]
    );
    assert_eq!(MAX_COLLECTION_ITEMS, 100);
    assert_eq!(
        leading_number(catalogued["MAX_DEPTH"]),
        MAX_DEPTH.to_string()
    );
    assert_eq!(quoted_literals(catalogued["MAX_DEPTH"]), [MAX_DEPTH_MARKER]);
    assert_eq!(MAX_DEPTH, 8);
    assert!(
        contract
            .exact_value
            .contains("circular marker '[Circular]'")
    );
    assert_eq!(CIRCULAR, "[Circular]");

    // Behaviour, because a pattern that compiles is not a pattern that works.
    // The key test is a *substring* match, except `token$`, which is anchored —
    // that anchor is the only thing keeping `inputTokens` readable.
    for sensitive in [
        "apiKey",
        "api_key",
        "API-KEY",
        "authorization",
        "Cookie",
        "credential",
        "password",
        "clientSecret",
        "token",
        "accessToken",
    ] {
        assert!(is_sensitive_key(sensitive), "`{sensitive}` must redact");
    }
    for readable in [
        "inputTokens",
        "outputTokens",
        "tokenCount",
        "event",
        "durationMs",
    ] {
        assert!(
            !is_sensitive_key(readable),
            "`{readable}` must stay readable"
        );
    }

    // The `g` flag: every match in a string, not just the first.
    assert_eq!(
        scrub_string("Bearer aaaaaaaa and Basic bbbbbbbb"),
        format!("Bearer {REDACTED} and Basic {REDACTED}")
    );
    assert_eq!(
        scrub_string("sk-abcdefgh then sk-ijklmnop"),
        format!("{REDACTED} then {REDACTED}")
    );
    assert_eq!(
        scrub_string("https://x/y?token=secret&api_key=other&keep=1"),
        format!("https://x/y?token={REDACTED}&api_key={REDACTED}&keep=1")
    );
    // The ASCII-only word boundary: a `sk-` run that is too short does not
    // match, and one glued to a word character does not either.
    assert_eq!(scrub_string("sk-short"), "sk-short");
    assert_eq!(scrub_string("no secrets here"), "no secrets here");
}

#[test]
fn redaction_sentinels_and_rotation_defaults() {
    let contract = expect_contract(
        "default-value",
        "redaction sentinels and log rotation defaults",
    );
    let value = &contract.exact_value;

    let sentinels = quoted_literals(value);
    assert_eq!(sentinels, [REDACTED, CIRCULAR], "{value}");
    assert!(
        value.contains("DEFAULT_MAX_BYTES = 10*1024*1024"),
        "{value}"
    );
    assert!(value.contains("DEFAULT_MAX_FILES = 5"), "{value}");
    assert_eq!(DEFAULT_MAX_BYTES, 10 * 1024 * 1024);
    assert_eq!(DEFAULT_MAX_FILES, 5);

    // The two override variables are renamed per docs/rebrand.md, which is why
    // this row is Divergent. Derived, so a partial rename fails. The clause is
    // sliced first, because `DEFAULT_MAX_BYTES` and `DEFAULT_MAX_FILES` are
    // constant names in the same sentence, not environment variables.
    let overrides = env_names(
        value
            .split_once("overridden by")
            .expect("the catalogue still names the overrides")
            .1,
    );
    assert_eq!(overrides.len(), 2, "{value}");
    assert_eq!(
        overrides
            .iter()
            .map(|name| rebranded(name))
            .collect::<Vec<_>>(),
        [ENV_LOG_MAX_BYTES, ENV_LOG_MAX_FILES]
    );
    for name in &overrides {
        assert_ne!(name.as_str(), rebranded(name), "`{name}` must be renamed");
    }
}

#[test]
fn log_schema_and_record_spine() {
    let schema = expect_contract("log-schema", "LOG_SCHEMA");
    let spine = expect_contract("json-field", "LOG_SCHEMA and log record spine");
    let envelope = expect_contract("file-format", "log record envelope");

    // The schema string: upstream's, renamed by the documented rule.
    let upstream_schema = unquote(
        schema
            .exact_value
            .split(';')
            .next()
            .expect("the schema is the first clause"),
    );
    assert_eq!(upstream_schema, "qwaudio.log/v1", "{}", schema.file);
    assert_eq!(LOG_SCHEMA, rebranded_schema(upstream_schema));
    assert_ne!(LOG_SCHEMA, upstream_schema);
    assert!(spine.exact_value.contains(upstream_schema));
    assert!(envelope.exact_value.contains(upstream_schema));

    // The spine, in order, from all three records. The three spread sources
    // come first and are not fields; the seven spine keys follow.
    let spine_of = |raw: &str| -> (Vec<String>, Vec<String>) {
        let all: Vec<String> = js_object_fields(raw)
            .unwrap_or_else(|| panic!("no object shape in: {raw}"))
            .into_iter()
            .map(|field| field_name(field).to_owned())
            .collect();
        let (spreads, keys): (Vec<String>, Vec<String>) =
            all.into_iter().partition(|name| name.starts_with("..."));
        (spreads, keys)
    };

    let (envelope_spreads, envelope_keys) = spine_of(&envelope.exact_value);
    assert_eq!(
        envelope_spreads,
        ["...base", "...asyncContext", "...fields"],
        "caller-supplied maps are spread first"
    );
    assert_eq!(
        envelope_keys, RESERVED_FIELDS,
        "the seven spine keys, in order"
    );

    let (record_spreads, record_keys) = spine_of(&spine.exact_value);
    assert_eq!(record_spreads, ["...base", "...context", "...fields"]);
    assert_eq!(record_keys, RESERVED_FIELDS);

    let (_, schema_keys) = spine_of(&schema.exact_value);
    assert_eq!(schema_keys, RESERVED_FIELDS);

    // "the spine is spread LAST so caller fields cannot override it".
    assert!(spine.exact_value.contains("the spine is spread LAST"));
    let colliding = LogRecord::new(
        LogLevel::Warn,
        GATEWAY_COMPONENT,
        "realtime.retry",
        fields([
            ("schema", json!("forged")),
            ("level", json!("trace")),
            ("component", json!("attacker")),
            ("event", json!("forged.event")),
            ("pid", json!(0)),
            ("provider", json!("dashscope")),
        ]),
        "",
    );
    let map = colliding.to_map();
    assert_eq!(map["schema"], json!(LOG_SCHEMA));
    assert_eq!(map["level"], json!("warn"));
    assert_eq!(map["component"], json!(GATEWAY_COMPONENT));
    assert_eq!(map["event"], json!("realtime.retry"));
    assert_ne!(map["pid"], json!(0));
    assert_eq!(
        map["provider"],
        json!("dashscope"),
        "an ordinary field survives"
    );
    // Overwriting keeps the caller's *position*, exactly as JS object spread
    // does: the five forged keys stay where they were written.
    assert_eq!(
        map.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "schema",
            "level",
            "component",
            "event",
            "pid",
            "provider",
            "time"
        ]
    );

    // `message` is deliberately NOT in that list: upstream spreads it only
    // when the call site passed one, so a `message` field survives.
    let carried = LogRecord::new(
        LogLevel::Info,
        GATEWAY_COMPONENT,
        "task.progress",
        fields([("message", json!("from the fields map"))]),
        "",
    );
    assert_eq!(carried.to_map()["message"], json!("from the fields map"));

    // The console line, byte for byte against the catalogued template.
    assert!(envelope.exact_value.contains(
        "`${time} ${LEVEL} ${component} ${event}${message?': '+message:''}\
         ${details?' '+JSON.stringify(details):''}\\n`"
    ));
    let record = LogRecord {
        extra: fields([("provider", json!("dashscope"))]),
        schema: LOG_SCHEMA.to_owned(),
        time: "2026-08-22T10:36:00.000Z".to_owned(),
        level: LogLevel::Info,
        component: GATEWAY_COMPONENT.to_owned(),
        event: "realtime.connected".to_owned(),
        pid: 4242,
        message: Some("ready".to_owned()),
    };
    assert_eq!(
        record.to_console_line(),
        "2026-08-22T10:36:00.000Z INFO gateway realtime.connected: ready \
         {\"provider\":\"dashscope\"}\n"
    );
    // No message and no details: neither optional clause appears.
    let bare = LogRecord {
        extra: no_fields(),
        message: None,
        ..record.clone()
    };
    assert_eq!(
        bare.to_console_line(),
        "2026-08-22T10:36:00.000Z INFO gateway realtime.connected\n"
    );

    // `details omits schema,time,level,component,event,pid,message` — the same
    // seven names, parsed out of the catalogue.
    let omitted = list(
        envelope
            .exact_value
            .split_once("details omits ")
            .expect("the catalogue still names the omitted keys")
            .1
            .trim_end_matches('.'),
    );
    assert_eq!(
        omitted, RESERVED_FIELDS,
        "the console detail exclusion list"
    );

    // The JSON-lines shape: one object per line, newline terminated.
    let line = record.to_line();
    assert!(line.ends_with('\n'));
    assert_eq!(line.lines().count(), 1, "one JSON object per line");
    let parsed: Value = serde_json::from_str(line.trim_end()).expect("valid JSON");
    assert_eq!(parsed["schema"], json!(LOG_SCHEMA));

    // `file name <component>.log at mode 0o600; rotation to .1, .2, …`
    assert!(
        schema
            .exact_value
            .contains("file name `<component>.log` at mode 0o600")
    );
    assert!(schema.exact_value.contains("rotation to `.1`, `.2`"));
    let options = LoggerOptions::new("realtime", &env(&[]), Path::new("/home/tester"));
    assert_eq!(options.file_name, "realtime.log");
    assert_eq!(
        LoggerOptions::gateway(&env(&[]), Path::new("/home/tester")).file_name,
        GATEWAY_LOG_FILE_NAME
    );
    assert_eq!(GATEWAY_LOG_FILE_NAME, "gateway.log");
    assert_eq!(GATEWAY_COMPONENT, "gateway");
    assert_eq!(LOG_FILE_MODE, 0o600);
}

#[test]
fn logging_environment_variables() {
    let contract = expect_contract("env-var", "logging");
    let value = &contract.exact_value;

    // Every environment name the contract mentions, in the order it mentions
    // them, mapped through the documented rename rule. Nothing is retyped, so
    // a rename applied to some names and not others fails here.
    let upstream_names = env_names(value);
    assert_eq!(
        upstream_names
            .iter()
            .map(|name| rebranded(name))
            .collect::<Vec<_>>(),
        [
            ENV_LOG_LEVEL,
            ENV_LOG_DIR,
            ENV_CONFIG_DIR,
            ENV_XDG_CONFIG_HOME,
            ENV_LOG_CONSOLE,
            ENV_LOG_FILE,
            ENV_LOG_MAX_BYTES,
            ENV_LOG_MAX_FILES,
            ENV_NODE_ENV,
        ],
        "{value}"
    );
    // XDG_CONFIG_HOME and NODE_ENV belong to other specifications and are KEPT
    // verbatim; every product-owned name is renamed.
    for name in &upstream_names {
        let renamed = rebranded(name);
        if name == ENV_XDG_CONFIG_HOME || name == ENV_NODE_ENV {
            assert_eq!(&renamed, name, "`{name}` is not ours to rename");
        } else {
            assert_ne!(&renamed, name, "`{name}` must be renamed");
            assert!(renamed.starts_with("VIA_"));
        }
    }

    let home = Path::new("/home/tester");

    // Level: default info, unknown value falls back to info.
    assert!(value.contains("default 'info' (unknown value falls back to 'info')"));
    assert_eq!(EnvSettings::from_env(&env(&[]), home).level, LogLevel::Info);
    assert_eq!(
        EnvSettings::from_env(&env(&[(ENV_LOG_LEVEL, "nonsense")]), home).level,
        LogLevel::Info
    );
    assert_eq!(
        EnvSettings::from_env(&env(&[(ENV_LOG_LEVEL, "debug")]), home).level,
        LogLevel::Debug
    );

    // Console / file: disabled ONLY on exactly "0". This is the subtlety a
    // naive port turns into a truthiness check.
    assert!(value.contains("disables console only when exactly '0'"));
    assert!(value.contains("disables the file sink only when exactly '0'"));
    for (raw, enabled) in [
        ("0", false),
        ("false", true),
        ("", true),
        ("00", true),
        ("1", true),
    ] {
        let settings = EnvSettings::from_env(&env(&[(ENV_LOG_CONSOLE, raw)]), home);
        assert_eq!(settings.console_enabled, enabled, "CONSOLE={raw:?}");
        let settings = EnvSettings::from_env(&env(&[(ENV_LOG_FILE, raw)]), home);
        assert_eq!(settings.file_enabled, enabled, "FILE={raw:?}");
    }
    assert!(EnvSettings::from_env(&env(&[]), home).console_enabled);
    assert!(EnvSettings::from_env(&env(&[]), home).file_enabled);

    // Bounds, restated in this contract as well as in the rotation one.
    assert!(value.contains("default 10485760 clamped [1024, 1073741824]"));
    assert!(value.contains("default 5 clamped [1,100]"));

    // Gateway logger identity.
    assert!(value.contains("component 'gateway', fileName 'gateway.log'"));
    assert_eq!(GATEWAY_COMPONENT, "gateway");
    assert_eq!(GATEWAY_LOG_FILE_NAME, "gateway.log");

    // Test detection: upstream's two arms, reproduced verbatim.
    assert!(value.contains("NODE_ENV==='test'"));
    assert!(value.contains("disables BOTH console and file"));
    assert_eq!(ENV_NODE_ENV, "NODE_ENV");
    assert_eq!(NODE_ENV_TEST, "test");
    assert!(is_test_process(
        &env(&[(ENV_NODE_ENV, NODE_ENV_TEST)]),
        Vec::<String>::new()
    ));
    assert!(is_test_process(&env(&[]), ["/repo/tests/logger.rs"]));
    assert!(is_test_process(&env(&[]), [r"C:\repo\test\logger.rs"]));
    assert!(!is_test_process(&env(&[]), ["/repo/src/main.rs"]));
    // Compared with `===`, so a near miss is not a match.
    assert!(!is_test_process(
        &env(&[(ENV_NODE_ENV, "testing")]),
        Vec::<String>::new()
    ));

    // VIA's third arm, which is the one that actually fires for a Rust build:
    // a `cargo test` binary is `target/<profile>/deps/<name>-<hash>`, which
    // matches neither upstream arm. Recorded in docs/deviations/phase-0.md.
    let cargo_argv = ["/repo/target/debug/deps/via_log-3f1c2a9b4d5e6f70"];
    assert!(
        !is_test_process(&env(&[]), cargo_argv),
        "neither upstream arm can see a cargo test binary"
    );
    assert!(is_test_process(
        &env(&[(ENV_TEST_PROCESS, TEST_PROCESS_VALUE)]),
        cargo_argv
    ));
    assert_eq!(ENV_TEST_PROCESS, "VIA_TEST");
    assert_eq!(TEST_PROCESS_VALUE, "1");
    assert!(!is_test_process(
        &env(&[(ENV_TEST_PROCESS, "true")]),
        cargo_argv
    ));
    assert!(
        !value.contains(ENV_TEST_PROCESS),
        "VIA_TEST is VIA's own signal, not an upstream contract"
    );
}

#[test]
fn default_log_directory_resolution_order() {
    let contract = expect_contract("file-path", "default log directory resolution order");
    let steps: Vec<&str> = contract.exact_value.split('>').map(str::trim).collect();
    assert_eq!(steps.len(), 4, "{}", contract.exact_value);

    // The three environment names, renamed by the documented rule.
    let upstream_names = env_names(&contract.exact_value);
    assert_eq!(
        upstream_names
            .iter()
            .map(|name| rebranded(name))
            .collect::<Vec<_>>(),
        [ENV_LOG_DIR, ENV_CONFIG_DIR, ENV_XDG_CONFIG_HOME]
    );

    // The path segment is renamed too: docs/rebrand.md maps the upstream
    // config directory name onto `via`.
    let upstream_segment = "qwaudio";
    assert!(
        steps[2].ends_with(&format!("/{upstream_segment}/logs")),
        "{:?}",
        steps[2]
    );
    assert!(
        steps[3].ends_with(&format!("/{upstream_segment}/logs")),
        "{:?}",
        steps[3]
    );
    assert_eq!(CONFIG_DIRECTORY_NAME, "via");
    assert_ne!(CONFIG_DIRECTORY_NAME, upstream_segment);
    assert_eq!(LOG_DIRECTORY_NAME, "logs");

    // The order, one step at a time, each shadowing the ones after it. The
    // second step is the subtlety: the config directory wins over XDG.
    let home = Path::new("/home/tester");
    let all = env(&[
        (ENV_LOG_DIR, "/opt/logs"),
        (ENV_CONFIG_DIR, "/opt/config"),
        (ENV_XDG_CONFIG_HOME, "/opt/xdg"),
    ]);
    assert_eq!(
        default_log_directory(&all, home),
        PathBuf::from("/opt/logs")
    );

    let without_log_dir = env(&[
        (ENV_CONFIG_DIR, "/opt/config"),
        (ENV_XDG_CONFIG_HOME, "/opt/xdg"),
    ]);
    assert_eq!(
        default_log_directory(&without_log_dir, home),
        PathBuf::from("/opt/config/logs"),
        "the config directory is consulted BEFORE XDG"
    );

    let xdg_only = env(&[(ENV_XDG_CONFIG_HOME, "/opt/xdg")]);
    assert_eq!(
        default_log_directory(&xdg_only, home),
        PathBuf::from("/opt/xdg/via/logs")
    );

    assert_eq!(
        default_log_directory(&env(&[]), home),
        PathBuf::from("/home/tester/.config/via/logs")
    );

    // An empty value is not a value — JavaScript's truthiness check, kept.
    let empty = env(&[(ENV_LOG_DIR, ""), (ENV_CONFIG_DIR, "")]);
    assert_eq!(
        default_log_directory(&empty, home),
        PathBuf::from("/home/tester/.config/via/logs")
    );
}

#[test]
fn config_directory_environment_variable() {
    // The contract is filed under its upstream name.
    let upstream_config_dir = "QWAUDIO_CONFIG_DIR";
    let contract = expect_contract("env-var", upstream_config_dir);
    let value = &contract.exact_value;

    assert_eq!(ENV_CONFIG_DIR, rebranded(upstream_config_dir));
    assert_eq!(ENV_CONFIG_DIR, "VIA_CONFIG_DIR");

    // The half this crate owns: `the log directory is <configDir>/logs`.
    assert!(
        value.contains("the log directory is <configDir>/logs"),
        "{value}"
    );
    assert_eq!(LOG_DIRECTORY_NAME, "logs");
    assert_eq!(
        default_log_directory(
            &env(&[(ENV_CONFIG_DIR, "/opt/via-config")]),
            Path::new("/home/tester")
        ),
        PathBuf::from("/opt/via-config/logs")
    );
    // `absolute path` — a relative value is absolutized rather than passed on.
    assert!(value.starts_with("absolute path"), "{value}");
    let resolved = default_log_directory(
        &env(&[(ENV_CONFIG_DIR, "relative/config")]),
        Path::new("/home/tester"),
    );
    assert!(resolved.is_absolute(), "{resolved:?}");
    assert!(resolved.ends_with("relative/config/logs"), "{resolved:?}");

    // The half it does not: `when unset the config directory is <platform
    // base>/qwaudio`. Resolving the platform base — and the data directory
    // beside it — is `via-core`'s job; this crate never computes a config
    // directory, only a log directory under one it is given. That is why the
    // row is Partial.
    assert!(
        value.contains("when unset the config directory is <platform base>/"),
        "{value}"
    );
}

#[test]
fn files_and_directories_the_product_creates() {
    let contract = expect_contract("file-path", "files and directories the product creates");
    let value = &contract.exact_value;

    // The one entry this crate creates.
    assert!(value.contains("logs/gateway.log (+ .1 … .N)"), "{value}");
    assert_eq!(LOG_DIRECTORY_NAME, "logs");
    assert_eq!(GATEWAY_LOG_FILE_NAME, "gateway.log");
    assert_eq!(LOG_DIRECTORY_MODE, 0o700);
    assert_eq!(LOG_FILE_MODE, 0o600);

    let dir = tempfile::TempDir::new().expect("tempdir");
    let logs = dir.path().join(LOG_DIRECTORY_NAME);
    let sink = JsonLineFileSink::new(FileSinkOptions::new(&logs, GATEWAY_LOG_FILE_NAME));
    assert!(!logs.exists(), "nothing is created until the first record");
    sink.write_record(&LogRecord::new(
        LogLevel::Info,
        GATEWAY_COMPONENT,
        "gateway.ready",
        no_fields(),
        "listening",
    ));
    let written = logs.join(GATEWAY_LOG_FILE_NAME);
    assert!(written.exists(), "logs/gateway.log");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&logs).expect("metadata").permissions().mode() & 0o777,
            LOG_DIRECTORY_MODE
        );
        assert_eq!(
            fs::metadata(&written)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777,
            LOG_FILE_MODE
        );
    }

    // Everything else on the list belongs to the crate that creates it, and
    // most of them to `via-core`, which resolves both directories. Named here
    // so the Partial classification states what is still owed rather than
    // leaving it to a comment.
    for owed in [
        "config.env",
        "state.env",
        "USER.md",
        "ASSISTANT.md",
        "MEMORY.md",
        "frontend-notes.json",
        "workspace/",
        "gateway.lock",
        "tasks.json",
        "memory-audit.jsonl",
        "state/acp-sessions.json",
        "models/wake-word/",
        "skins/",
    ] {
        assert!(
            value.contains(owed),
            "`{owed}` is no longer catalogued; the Partial row's remainder is stale"
        );
    }
    // The two directories the list is split across are the ones via-core
    // resolves; this crate is handed one of them.
    assert!(value.contains("In dataDirectory:"), "{value}");
    assert!(value.contains("In configDirectory:"), "{value}");
}
