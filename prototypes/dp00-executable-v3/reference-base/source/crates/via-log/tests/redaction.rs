//! Adversarial tests for the redaction boundary.
//!
//! A log line that leaks `DASHSCOPE_API_KEY` or a gateway token is a real
//! incident, so these tests are written to *break* the redactor rather than to
//! demonstrate it. The upstream cases from `test/logger.test.mjs:62-94` and
//! `:124-131` are ported first; the rest are new.

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::sync::Arc;
use via_log::{
    CIRCULAR, ErrorRecord, LogLevel, Logger, LoggerOptions, MAX_COLLECTION_ITEMS, MAX_DEPTH,
    MAX_DEPTH_MARKER, MAX_STRING_CHARS, MemorySink, REDACTED, TRUNCATED_MARKER, TRUNCATED_SUFFIX,
    fields, is_sensitive_key, no_fields, redact_map, redact_value, redact_value_for_key,
    scrub_string,
};

fn memory_logger() -> (Logger, Arc<MemorySink>) {
    let sink = Arc::new(MemorySink::new());
    let mut options = LoggerOptions::detached("desktop");
    options.level = LogLevel::Trace;
    (Logger::with_sinks(options, vec![sink.clone()]), sink)
}

// ── Ported from upstream test/logger.test.mjs ───────────────────────────────

/// Upstream `test/logger.test.mjs:62-94` — "redacts secrets recursively from
/// fields, errors and free text".
#[test]
fn upstream_recursive_redaction_case() {
    let (logger, sink) = memory_logger();

    let mut error =
        ErrorRecord::from_message("request used Bearer abc.def.ghi and sk-secretvalue123");
    error
        .extra
        .insert("authorization".to_owned(), json!("Bearer should-not-leak"));

    logger.error(
        "request.failed",
        fields([
            ("apiKey", json!("sk-privatevalue123")),
            (
                "nested",
                json!({
                    "accessToken": "token-value",
                    "inputTokens": 123,
                    "safe": "https://example.com/callback?token=hidden-value&ok=1",
                }),
            ),
            ("error", error.to_value()),
        ]),
        "",
    );

    let raw = sink.lines().concat();
    for secret in [
        "privatevalue",
        "token-value",
        "abc.def",
        "secretvalue",
        "should-not-leak",
        "hidden-value",
    ] {
        assert!(!raw.contains(secret), "log line leaked {secret:?}:\n{raw}");
    }

    let record = &sink.records()[0];
    assert_eq!(record["apiKey"], json!(REDACTED));
    assert_eq!(record["nested"]["accessToken"], json!(REDACTED));
    // The one that must survive: `inputTokens` does not end in `token`.
    assert_eq!(record["nested"]["inputTokens"], json!(123));
    assert_eq!(
        record["nested"]["safe"],
        json!("https://example.com/callback?token=[REDACTED]&ok=1")
    );
    assert_eq!(record["error"]["authorization"], json!(REDACTED));
    assert_eq!(
        record["error"]["message"],
        json!("request used Bearer [REDACTED] and [REDACTED]")
    );
}

/// Upstream `test/logger.test.mjs:124-131`. A `serde_json::Value` cannot hold
/// a cycle, so the marker is asserted as a constant rather than produced; see
/// the recorded deviation in the crate docs.
#[test]
fn circular_marker_is_the_upstream_literal() {
    assert_eq!(CIRCULAR, "[Circular]");
}

// ── Adversarial: nesting ────────────────────────────────────────────────────

/// A sensitive key three levels down is still redacted — the walk is
/// recursive, not top-level.
#[test]
fn sensitive_key_nested_three_levels_deep() {
    let (logger, sink) = memory_logger();
    logger.info(
        "backend.spawn",
        fields([(
            "backend",
            json!({
                "openclaw": {
                    "environment": {
                        "DASHSCOPE_API_KEY": "sk-live-do-not-leak-me",
                        "VIA_BACKEND_MODEL": "qwen3.7-max",
                    }
                }
            }),
        )]),
        "",
    );

    let raw = sink.lines().concat();
    assert!(
        !raw.contains("do-not-leak-me"),
        "three-level nesting leaked:\n{raw}"
    );
    let record = &sink.records()[0];
    assert_eq!(
        record["backend"]["openclaw"]["environment"]["DASHSCOPE_API_KEY"],
        json!(REDACTED)
    );
    // The non-sensitive sibling at the same depth survives intact.
    assert_eq!(
        record["backend"]["openclaw"]["environment"]["VIA_BACKEND_MODEL"],
        json!("qwen3.7-max")
    );
}

/// A sensitive key hidden inside an array inside an object.
#[test]
fn sensitive_key_inside_an_array() {
    let redacted = redact_map(&fields([(
        "headers",
        json!([{ "name": "authorization", "value": "Bearer leak-me" }, { "cookie": "sid=leak" }]),
    )]));
    let rendered = serde_json::to_string(&redacted).expect("serializes");
    assert!(!rendered.contains("leak-me"), "{rendered}");
    assert!(!rendered.contains("sid=leak"), "{rendered}");
    // `value` is not a sensitive *key*, so the value pattern catches it and
    // keeps the scheme word — upstream's `${scheme} [REDACTED]` shape.
    assert_eq!(redacted["headers"][0]["value"], json!("Bearer [REDACTED]"));
    // `cookie` is a sensitive key, so the whole value goes.
    assert_eq!(redacted["headers"][1]["cookie"], json!(REDACTED));
    // `name` is safe, and its literal value is not a token shape.
    assert_eq!(redacted["headers"][0]["name"], json!("authorization"));
}

/// A sensitive key redacts its whole subtree, not only its strings.
#[test]
fn sensitive_key_redacts_the_whole_subtree() {
    let redacted = redact_map(&fields([(
        "credentials",
        json!({ "user": "alice", "rounds": 12, "inner": { "pin": 1234 } }),
    )]));
    assert_eq!(redacted["credentials"], json!(REDACTED));
}

// ── Adversarial: free text ──────────────────────────────────────────────────

/// A `Bearer` token mid-sentence in a message string, with prose either side.
#[test]
fn bearer_token_mid_sentence_in_a_message() {
    let (logger, sink) = memory_logger();
    logger.warn(
        "gateway.auth_failed",
        no_fields(),
        "the upstream rejected header Authorization: Bearer eyJhbGciOi.J9.sig-9_x~+/= while retrying",
    );

    let raw = sink.lines().concat();
    assert!(
        !raw.contains("eyJhbGciOi"),
        "message leaked a token:\n{raw}"
    );
    assert_eq!(
        sink.records()[0]["message"],
        json!("the upstream rejected header Authorization: Bearer [REDACTED] while retrying")
    );
}

/// The scheme word keeps its original casing, and `Basic` is covered too.
#[test]
fn auth_scheme_word_is_preserved_verbatim() {
    assert_eq!(scrub_string("bearer abc12345"), "bearer [REDACTED]");
    assert_eq!(scrub_string("BEARER abc12345"), "BEARER [REDACTED]");
    assert_eq!(scrub_string("Basic dXNlcjpwYXNz"), "Basic [REDACTED]");
    // Multiple credentials in one string are all replaced.
    assert_eq!(
        scrub_string("first Bearer aaaa and then Basic bbbb ok"),
        "first Bearer [REDACTED] and then Basic [REDACTED] ok"
    );
}

/// `sk-` keys and URL query secrets, in free text and in the middle of a
/// larger string.
#[test]
fn sk_keys_and_url_query_secrets_in_free_text() {
    assert_eq!(
        scrub_string("using sk-abcdefgh1234 for the call"),
        "using [REDACTED] for the call"
    );
    // Fewer than 8 characters after `sk-` is not the key shape upstream matches.
    assert_eq!(scrub_string("sk-short"), "sk-short");
    // `[^&\s]+` is greedy and stops only at `&` or whitespace, so a trailing
    // `#fragment` is swallowed with the secret. Faithful to upstream, and the
    // safe direction to err in.
    assert_eq!(
        scrub_string("GET https://host/v1?api_key=abc&ok=1&access_token=def#frag"),
        "GET https://host/v1?api_key=[REDACTED]&ok=1&access_token=[REDACTED]"
    );
    // The `?`/`&` anchor matters: a bare `token=` mid-path is not a query pair.
    assert_eq!(scrub_string("path/token=visible"), "path/token=visible");
}

// ── Adversarial: key naming ─────────────────────────────────────────────────

/// Keys that differ only in case, or only in separator, must all redact.
#[test]
fn key_matching_is_case_and_separator_insensitive() {
    for key in [
        "apiKey",
        "APIKEY",
        "ApiKey",
        "api_key",
        "API_KEY",
        "api-key",
        "Api-Key",
        "DASHSCOPE_API_KEY",
        "dashscope_api_key",
        "authorization",
        "Authorization",
        "AUTHORIZATION",
        "Cookie",
        "credential",
        "CREDENTIALS",
        "password",
        "Password",
        "userPassword",
        "secret",
        "clientSecret",
        "SECRET",
        "token",
        "Token",
        "accessToken",
        "refresh_token",
        "gatewayToken",
    ] {
        assert!(is_sensitive_key(key), "{key} must be treated as sensitive");
        assert_eq!(
            redact_value_for_key(&json!("super-secret-value"), key),
            json!(REDACTED),
            "{key} must redact",
        );
    }
}

/// Keys that merely *resemble* sensitive names must survive — `token$` is the
/// one anchored alternative, and that anchoring is what keeps usage counters
/// readable.
#[test]
fn near_miss_keys_are_not_redacted() {
    for key in [
        "inputTokens",
        "outputTokens",
        "totalTokens",
        "tokenCount",
        "tokensPerSecond",
        "apiKeys",
        "keyApi",
        "passwordless",
        "secrets",
        "secretary",
        "cookies",
    ] {
        let expectation = !matches!(
            key,
            "apiKeys" | "passwordless" | "secrets" | "secretary" | "cookies"
        );
        assert_eq!(
            !is_sensitive_key(key),
            expectation,
            "{key} classification changed",
        );
    }
    // The load-bearing one, asserted directly.
    assert!(!is_sensitive_key("inputTokens"));
    assert!(!is_sensitive_key("tokenCount"));
    assert_eq!(
        redact_value_for_key(&json!(4096), "inputTokens"),
        json!(4096)
    );
}

/// A value that merely *looks* like a credential, sitting under a safe key,
/// is left alone — the redactor must not degrade diagnostics by guessing.
#[test]
fn token_shaped_value_under_a_safe_key_survives() {
    let (logger, sink) = memory_logger();
    logger.info(
        "work.completed",
        fields([
            // 32 hex-ish characters: looks like a secret, matches no pattern.
            ("fingerprint", json!("AbCdEf0123456789AbCdEf0123456789")),
            ("workId", json!("work_01J8Z9QF7K3M4N5P6R7S8T9V0W")),
            ("model", json!("qwen-audio-3.0-realtime-plus")),
            ("bearerCount", json!(3)),
            // No whitespace after the scheme word, so not the auth shape.
            ("note", json!("BearerTokenPolicy applies here")),
            // `sk_` with an underscore is not upstream's `sk-` shape.
            ("prefix", json!("sk_live_1234567890")),
        ]),
        "done",
    );

    let record = &sink.records()[0];
    assert_eq!(
        record["fingerprint"],
        json!("AbCdEf0123456789AbCdEf0123456789")
    );
    assert_eq!(record["workId"], json!("work_01J8Z9QF7K3M4N5P6R7S8T9V0W"));
    assert_eq!(record["model"], json!("qwen-audio-3.0-realtime-plus"));
    assert_eq!(record["bearerCount"], json!(3));
    assert_eq!(record["note"], json!("BearerTokenPolicy applies here"));
    assert_eq!(record["prefix"], json!("sk_live_1234567890"));
}

// ── Adversarial: the caps ───────────────────────────────────────────────────

#[test]
fn depth_cap_writes_the_max_depth_marker() {
    // Build `{"a":{"a":{...}}}` deep enough to cross MAX_DEPTH.
    let mut value = json!("leaf");
    for _ in 0..(MAX_DEPTH + 2) {
        value = json!({ "a": value });
    }
    let redacted = redact_value(&value);
    let mut cursor = &redacted;
    for _ in 0..MAX_DEPTH {
        cursor = &cursor["a"];
    }
    assert_eq!(cursor, &json!(MAX_DEPTH_MARKER));
}

#[test]
fn array_cap_appends_the_truncated_marker() {
    let items: Vec<Value> = (0..MAX_COLLECTION_ITEMS + 5).map(|n| json!(n)).collect();
    let redacted = redact_value(&json!(items));
    let array = redacted.as_array().expect("array");
    assert_eq!(array.len(), MAX_COLLECTION_ITEMS + 1);
    assert_eq!(array[MAX_COLLECTION_ITEMS], json!(TRUNCATED_MARKER));
}

#[test]
fn object_cap_clips_without_a_marker() {
    let map = (0..MAX_COLLECTION_ITEMS + 5)
        .map(|n| (format!("k{n}"), json!(n)))
        .collect();
    let redacted = redact_map(&map);
    assert_eq!(redacted.len(), MAX_COLLECTION_ITEMS);
    assert!(!redacted.contains_key(&format!("k{MAX_COLLECTION_ITEMS}")));
}

#[test]
fn string_cap_appends_the_truncated_suffix() {
    let long = "a".repeat(MAX_STRING_CHARS + 10);
    let scrubbed = scrub_string(&long);
    assert_eq!(
        scrubbed.chars().count(),
        MAX_STRING_CHARS + TRUNCATED_SUFFIX.chars().count()
    );
    assert!(scrubbed.ends_with(TRUNCATED_SUFFIX));

    let exact = "a".repeat(MAX_STRING_CHARS);
    assert_eq!(scrub_string(&exact), exact);
}

// ── Idempotence ─────────────────────────────────────────────────────────────

/// `Logger::log_record` redacts, and redaction is idempotent, so a record that
/// has already been through the pipeline is unchanged by a second pass.
#[test]
fn redaction_is_idempotent() {
    let source = fields([
        ("apiKey", json!("sk-abcdefgh1234")),
        ("message", json!("used Bearer aaaa")),
        ("url", json!("https://h/v1?token=x&ok=1")),
    ]);
    let once = redact_map(&source);
    let twice = redact_map(&once);
    assert_eq!(once, twice);
}
