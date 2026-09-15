//! `BackendRuntimeState`, asserted transition by transition.
//!
//! Ported from `server/src/agent/backend-runtime-state.mjs` and the parts of
//! `server/test/backend-lifecycle.test.mjs` that concern runtime readiness
//! rather than installation. The lifecycle test's other three cases — the
//! installation/configuration/authentication spec composition — belong to
//! `via-backends`.
//!
//! Two things get asserted here that a looser test would skip: the **key
//! order** of every emitted value, because `/api/health` publishes it and
//! upstream's object spread makes it differ per transition, and the **order**
//! of the failure classifier's tests, because a message matching two patterns
//! must land on the first.

mod common;

use std::sync::{Arc, Mutex};

use pretty_assertions::assert_eq;
use serde_json::json;
use via_catalog::Ownership;
use via_i18n::{Locale, keys, t};
use via_process::{
    AcpConnection, BackendFailure, BackendRuntimeState, BackendRuntimeStateOptions,
    BackendStatusCode, BackendStatusKind, DEFAULT_BACKOFF_MS, InitializedAgent,
    backend_failure_code,
};

/// A state with a controllable clock and stderr tail.
struct Fixture {
    state: BackendRuntimeState,
    clock: Arc<Mutex<i64>>,
    stderr: Arc<Mutex<String>>,
}

fn fixture(connection: Option<AcpConnection>, ownership: Ownership) -> Fixture {
    let clock = Arc::new(Mutex::new(1_000_000_i64));
    let stderr = Arc::new(Mutex::new(String::new()));
    let clock_reader = Arc::clone(&clock);
    let stderr_reader = Arc::clone(&stderr);
    let mut options =
        BackendRuntimeStateOptions::new("fixture", ownership, connection, "Fixture Backend");
    options.now = Arc::new(move || *clock_reader.lock().expect("clock"));
    options.stderr = Arc::new(move || stderr_reader.lock().expect("stderr").clone());
    Fixture {
        state: BackendRuntimeState::new(options),
        clock,
        stderr,
    }
}

fn keys_of(value: &serde_json::Map<String, serde_json::Value>) -> Vec<&str> {
    value.keys().map(String::as_str).collect()
}

// ── the envelope and its key order ──────────────────────────────────────────

#[test]
fn the_initial_value_is_stopped_and_not_started() {
    let fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    assert_eq!(fixture.state.code(), BackendStatusCode::NotStarted);
    assert_eq!(fixture.state.status_kind(), BackendStatusKind::Stopped);
    assert!(!fixture.state.ok());
    assert!(!fixture.state.should_backoff());
    assert_eq!(
        keys_of(fixture.state.value()),
        vec![
            "ok",
            "status",
            "code",
            "protocol",
            "ownership",
            "transport",
            "acpConnection"
        ],
    );
}

#[test]
fn an_absent_acp_connection_is_json_null() {
    let fixture = fixture(None, Ownership::External);
    assert_eq!(fixture.state.value()["acpConnection"], json!(null));
    assert_eq!(fixture.state.value()["ownership"], "external");
}

#[test]
fn starting_spreads_the_previous_value_and_appends_error() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture.state.starting("");
    assert_eq!(fixture.state.code(), BackendStatusCode::Starting);
    assert_eq!(fixture.state.status_kind(), BackendStatusKind::Starting);
    assert_eq!(
        keys_of(fixture.state.value()),
        vec![
            "ok",
            "status",
            "code",
            "protocol",
            "ownership",
            "transport",
            "acpConnection",
            "error"
        ],
        "`starting` spreads the previous value, so `error` lands last",
    );
}

#[test]
fn waiting_rebuilds_the_value_and_puts_error_before_the_envelope() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture.state.ready(None);
    fixture.state.waiting("the backend is still coming up");
    assert_eq!(fixture.state.code(), BackendStatusCode::BackendStarting);
    assert_eq!(fixture.state.status_kind(), BackendStatusKind::Starting);
    assert_eq!(
        keys_of(fixture.state.value()),
        vec![
            "ok",
            "status",
            "code",
            "error",
            "protocol",
            "ownership",
            "transport",
            "acpConnection"
        ],
        "`waiting` does not spread, so `error` lands before the envelope",
    );
    assert!(
        !fixture.state.value().contains_key("capabilities"),
        "`waiting` must drop a previous ready()'s capabilities",
    );
}

#[test]
fn starting_after_ready_keeps_the_stale_capabilities_upstream_keeps() {
    // A faithfulness case, not a design choice: `starting` spreads the previous
    // value, so an earlier `ready`'s `agentInfo` and `capabilities` survive it.
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture.state.ready(Some(&InitializedAgent {
        agent_info: Some(json!({"name": "fixture"})),
        agent_capabilities: [("promptCapabilities".to_owned(), json!({"image": true}))]
            .into_iter()
            .collect(),
    }));
    fixture.state.starting("");
    assert_eq!(
        fixture.state.value()["agentInfo"],
        json!({"name": "fixture"})
    );
    assert_eq!(
        fixture.state.value()["capabilities"],
        json!({"promptCapabilities": {"image": true}}),
    );
}

#[test]
fn ready_defaults_agent_info_to_null_and_capabilities_to_an_object() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture.state.ready(None);
    assert!(fixture.state.ok());
    assert_eq!(fixture.state.code(), BackendStatusCode::Ready);
    assert_eq!(fixture.state.value()["agentInfo"], json!(null));
    assert_eq!(fixture.state.value()["capabilities"], json!({}));
    assert_eq!(
        keys_of(fixture.state.value()),
        vec![
            "ok",
            "status",
            "code",
            "agentInfo",
            "capabilities",
            "protocol",
            "ownership",
            "transport",
            "acpConnection"
        ],
    );
}

#[test]
fn stopped_blanks_the_error_rather_than_removing_it() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture
        .state
        .failed(&BackendFailure::message("something broke"));
    fixture.state.stopped();
    assert_eq!(fixture.state.code(), BackendStatusCode::Stopped);
    assert_eq!(fixture.state.value()["error"], "");
    assert!(
        !fixture.state.should_backoff(),
        "stopping clears the backoff"
    );
}

// ── failure, stderr and the backoff ─────────────────────────────────────────

#[test]
fn a_failure_arms_the_backoff_and_publishes_a_retry_window() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture
        .state
        .failed(&BackendFailure::message("start failed"));
    assert_eq!(fixture.state.status_kind(), BackendStatusKind::Failed);
    assert!(fixture.state.should_backoff());
    assert_eq!(
        fixture.state.status(true)["retryAfterMs"],
        json!(DEFAULT_BACKOFF_MS),
    );

    *fixture.clock.lock().unwrap() += DEFAULT_BACKOFF_MS / 3;
    assert!(fixture.state.should_backoff());
    assert_eq!(
        fixture.state.status(true)["retryAfterMs"],
        json!(DEFAULT_BACKOFF_MS - DEFAULT_BACKOFF_MS / 3),
    );

    *fixture.clock.lock().unwrap() += DEFAULT_BACKOFF_MS;
    assert!(!fixture.state.should_backoff());
    assert_eq!(
        fixture.state.status(true)["retryAfterMs"],
        json!(0),
        "the retry window is floored at zero, never negative",
    );
}

#[test]
fn ready_clears_a_recorded_failure() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture
        .state
        .failed(&BackendFailure::message("start failed"));
    fixture.state.ready(None);
    assert!(!fixture.state.should_backoff());
    assert!(!fixture.state.status(true).contains_key("retryAfterMs"));
}

#[test]
fn a_captured_stderr_tail_is_appended_to_the_failure_message() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    *fixture.stderr.lock().unwrap() = "  cannot open config  ".to_owned();
    fixture
        .state
        .failed(&BackendFailure::message("start failed"));
    let error = fixture.state.value()["error"]
        .as_str()
        .expect("a failure carries an error")
        .to_owned();
    assert!(error.starts_with("start failed"), "{error}");
    assert!(error.ends_with("cannot open config"), "{error}");

    // An empty tail adds nothing at all — not even a separator.
    *fixture.stderr.lock().unwrap() = "   ".to_owned();
    fixture
        .state
        .failed(&BackendFailure::message("start failed"));
    assert_eq!(fixture.state.value()["error"], "start failed");
}

// ── the dead-client overlay ─────────────────────────────────────────────────

#[test]
fn a_ready_backend_whose_client_is_gone_reports_process_exited() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture.state.ready(None);
    let status = fixture.state.status(false);
    assert_eq!(status["ok"], false);
    assert_eq!(status["status"], "stopped");
    assert_eq!(status["code"], "PROCESS_EXITED");
    assert_eq!(
        status["error"],
        json!(t(Locale::En, keys::ACP_PROCESS_EXITED_LABEL).replace("{label}", "Fixture Backend")),
    );
    // The underlying value is untouched: `status()` is a view, not a mutation.
    assert!(fixture.state.ok());
}

#[test]
fn a_backend_that_was_never_ready_is_not_rewritten_by_a_dead_client() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture.state.waiting("still coming up");
    let status = fixture.state.status(false);
    assert_eq!(status["code"], "BACKEND_STARTING");
}

#[test]
fn the_dead_client_overlay_wins_over_the_retry_window() {
    let mut fixture = fixture(Some(AcpConnection::Process), Ownership::Owned);
    fixture.state.failed(&BackendFailure::message("boom"));
    fixture.state.ready(None);
    fixture.state.failed(&BackendFailure::message("boom again"));
    // Not ok, so the overlay does not apply and the retry window does.
    assert!(fixture.state.status(false).contains_key("retryAfterMs"));

    fixture.state.ready(None);
    let status = fixture.state.status(false);
    assert_eq!(status["code"], "PROCESS_EXITED");
    assert!(
        !status.contains_key("retryAfterMs"),
        "the two overlays are exclusive, as upstream's early return makes them",
    );
}

#[test]
fn the_process_exited_message_is_localized_in_all_three_locales() {
    let mut rendered = Vec::new();
    for locale in [Locale::En, Locale::Zh, Locale::Ko] {
        let mut options = BackendRuntimeStateOptions::new(
            "fixture",
            Ownership::Owned,
            Some(AcpConnection::Process),
            "Fixture Backend",
        );
        options.locale = locale;
        let mut state = BackendRuntimeState::new(options);
        state.ready(None);
        let message = state.status(false)["error"]
            .as_str()
            .expect("an error")
            .to_owned();
        assert!(message.contains("Fixture Backend"), "{message}");
        assert!(!message.contains('{'), "{message}");
        rendered.push(message);
    }
    rendered.sort();
    rendered.dedup();
    assert_eq!(rendered.len(), 3, "the three locales must differ");
}

// ── the failure classifier ──────────────────────────────────────────────────

#[test]
fn an_enoent_code_is_not_installed_whatever_the_message_says() {
    assert_eq!(
        backend_failure_code(&BackendFailure {
            code: Some("enoent".to_owned()),
            message: "connection timed out".to_owned(),
        }),
        BackendStatusCode::NotInstalled,
        "the code is upper-cased before the test, and it wins over the message",
    );
    assert_eq!(
        backend_failure_code(&BackendFailure::from_io(&std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no such file",
        ))),
        BackendStatusCode::NotInstalled,
    );
    assert_eq!(
        backend_failure_code(&BackendFailure::from_io(&std::io::Error::other("nope"))),
        BackendStatusCode::StartFailed,
    );
}

#[test]
fn the_classifier_matches_both_languages_upstream_ships() {
    let cases: &[(&str, BackendStatusCode)] = &[
        ("Missing API key", BackendStatusCode::ConfigRequired),
        ("no credential found", BackendStatusCode::ConfigRequired),
        (
            "backend is not configured",
            BackendStatusCode::ConfigRequired,
        ),
        ("后台未配置", BackendStatusCode::ConfigRequired),
        ("缺少凭据", BackendStatusCode::ConfigRequired),
        ("UNAUTHORIZED", BackendStatusCode::AuthRequired),
        ("you are not logged in", BackendStatusCode::AuthRequired),
        ("run login first", BackendStatusCode::AuthRequired),
        ("请先认证", BackendStatusCode::AuthRequired),
        ("请先登录", BackendStatusCode::AuthRequired),
        (
            "protocol version 99 unsupported",
            BackendStatusCode::ProtocolMismatch,
        ),
        ("协议版本不兼容", BackendStatusCode::ProtocolMismatch),
        ("start timeout", BackendStatusCode::StartTimeout),
        ("the request timed out", BackendStatusCode::StartTimeout),
        ("启动超时", BackendStatusCode::StartTimeout),
        (
            "process exited with code 1",
            BackendStatusCode::ProcessExited,
        ),
        ("进程已退出", BackendStatusCode::ProcessExited),
        ("something else entirely", BackendStatusCode::StartFailed),
        ("", BackendStatusCode::StartFailed),
    ];
    for (message, expected) in cases {
        assert_eq!(
            backend_failure_code(&BackendFailure::message(*message)),
            *expected,
            "`{message}`",
        );
    }
}

#[test]
fn the_classifier_tests_in_order_so_the_first_match_wins() {
    // Every one of these matches two patterns. The earlier test must win, or a
    // credential problem starts reporting as a timeout.
    let cases: &[(&str, BackendStatusCode)] = &[
        (
            "api key request timed out",
            BackendStatusCode::ConfigRequired,
        ),
        ("login timed out", BackendStatusCode::AuthRequired),
        (
            "protocol version handshake timed out",
            BackendStatusCode::ProtocolMismatch,
        ),
        (
            "timed out; the process exited",
            BackendStatusCode::StartTimeout,
        ),
    ];
    for (message, expected) in cases {
        assert_eq!(
            backend_failure_code(&BackendFailure::message(*message)),
            *expected,
            "`{message}`",
        );
    }
}

#[test]
fn the_classifier_folds_case_and_trims() {
    assert_eq!(
        backend_failure_code(&BackendFailure::message("   NOT LOGGED IN   ")),
        BackendStatusCode::AuthRequired,
    );
    assert_eq!(
        backend_failure_code(&BackendFailure {
            code: Some("  enoent  ".to_owned()),
            message: String::new(),
        }),
        BackendStatusCode::NotInstalled,
    );
}

#[test]
fn a_korean_failure_message_lands_on_start_failed() {
    // Recorded, not fixed: upstream's patterns cover `en` and `zh` only, and
    // widening them would be a behaviour change rather than a port. See
    // `docs/deviations/phase-2.md`.
    assert_eq!(
        backend_failure_code(&BackendFailure::message("로그인이 필요합니다")),
        BackendStatusCode::StartFailed,
    );
}

// ── the transient set ───────────────────────────────────────────────────────

#[test]
fn only_the_three_cold_start_codes_are_transient() {
    for code in BackendStatusCode::all() {
        let expected = matches!(
            code,
            BackendStatusCode::NotStarted
                | BackendStatusCode::Starting
                | BackendStatusCode::BackendStarting
        );
        assert_eq!(code.is_transient(), expected, "{}", code.as_str());
    }
}
