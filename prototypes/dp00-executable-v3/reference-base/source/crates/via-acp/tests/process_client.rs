//! The ACP client, exercised against a real agent process.
//!
//! Every test here spawns `tests/bin/fake_agent.rs` — a separate executable
//! speaking NDJSON on stdio — so what is under test is the whole path: spawn,
//! framing, correlation, the inbound notification and request streams, the
//! credential boundary, and the teardown ladder. A mock transport would leave
//! all six untested.
//!
//! Phase 2's delivery criterion (`docs/architecture.md` §15) is *"a scripted
//! fake ACP agent completes a prompt turn through the trait"*; that is
//! [`a_prompt_turn_completes_through_the_downstream_trait`].

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use serde_json::{Value, json};
use via_acp::{
    AcpBackendProfile, AcpDownstreamAgent, AcpError, AcpProcessClient, AcpSessionRegistry,
    BackendEnv, CancelReason, CancelSignal, PermissionDecision, Prompt, PromptOptions,
    SessionDetails, SpawnSpec,
};
use via_catalog::EnvironmentPolicy;
use via_core::EnvMap;
use via_downstream::{
    BackendCapabilities, CancelScope, DownstreamAgent, HarnessError, PromptRequest, SessionKey,
    StopReason,
};

/// The fake agent's path, from Cargo.
const FAKE_AGENT: &str = env!("CARGO_BIN_EXE_via-acp-fake-agent");

/// A policy that forwards one invented namespace and nothing else.
///
/// `via-acp` must never name a backend, so the fixtures here are invented
/// namespaces rather than catalogue entries.
const EXAMPLE_POLICY: EnvironmentPolicy = EnvironmentPolicy {
    names: &["EXAMPLE_API_KEY"],
    prefixes: &["FAKE_"],
    explicit_list_environment: None,
};

fn env_with(pairs: &[(&str, &str)]) -> BackendEnv {
    let mut env = EnvMap::new();
    // The child needs a `PATH` to be executable at all on some systems, and the
    // fixture is driven entirely by `FAKE_*`.
    if let Ok(path) = std::env::var("PATH") {
        env.set("PATH", path);
    }
    for (name, value) in pairs {
        env.set(*name, *value);
    }
    BackendEnv::project(&EXAMPLE_POLICY, &env, &[])
}

fn spec(script: &[(&str, &str)]) -> SpawnSpec {
    SpawnSpec::new(FAKE_AGENT).env(env_with(script))
}

fn client(script: &[(&str, &str)]) -> AcpProcessClient {
    AcpProcessClient::builder()
        .label("Example Agent")
        .spawn_spec(spec(script))
        .build()
}

fn capabilities() -> BackendCapabilities {
    BackendCapabilities {
        delegation: false,
        permissions: true,
        backend_ui: false,
        native_session_history: false,
        external_mcp: false,
        native_delegation: false,
        session_mcp: false,
    }
}

#[tokio::test]
async fn initializes_against_a_real_agent_process() {
    let client = client(&[]);
    let initialize = client
        .start()
        .await
        .expect("the fixture answers initialize");
    assert_eq!(initialize.protocol_version, via_acp::PROTOCOL_VERSION);
    assert_eq!(
        initialize
            .agent_info
            .as_ref()
            .map(|info| info.name.as_str()),
        Some("fake-acp-agent")
    );
    assert!(client.is_ready().await);
    client.close().await;
    assert!(!client.is_ready().await, "close tears the connection down");
}

#[tokio::test]
async fn concurrent_callers_share_one_initialization_and_one_process() {
    // Upstream: 'shares an in-flight ACP initialization across concurrent
    // callers' (acp-process-client.test.mjs:30-57).
    let client = client(&[("FAKE_SESSION_PREFIX", "shared")]);
    let first = {
        let client = client.clone();
        tokio::spawn(async move { client.start().await })
    };
    let second = {
        let client = client.clone();
        tokio::spawn(async move { client.start().await })
    };
    let first = first.await.expect("joined").expect("initialized");
    let second = second.await.expect("joined").expect("initialized");
    assert_eq!(first.protocol_version, second.protocol_version);

    // One process: the fixture numbers sessions per process, so two agents
    // would both mint `shared-1`.
    let a = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("first session");
    let b = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("second session");
    let id = |session: &via_acp::SharedSession| {
        session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .session_id
            .clone()
    };
    assert_eq!(id(&a), "shared-1");
    assert_eq!(id(&b), "shared-2");
    client.close().await;
}

#[tokio::test]
async fn an_incompatible_protocol_version_closes_the_connection() {
    let client = client(&[("FAKE_PROTOCOL_VERSION", "0")]);
    let error = client.start().await.expect_err("v0 is not v1");
    assert_eq!(
        error,
        AcpError::ProtocolVersionMismatch {
            label: "Example Agent".to_owned(),
            agent: "0".to_owned(),
            client: "1".to_owned(),
        }
    );
    assert_eq!(
        error.message(via_i18n::Locale::Zh),
        "Example Agent ACP 协议版本不兼容（Agent=0，Client=1）"
    );
    assert!(!client.is_ready().await);
}

#[tokio::test]
async fn native_stderr_survives_a_child_that_dies_during_initialization() {
    // Upstream: 'preserves native stderr when ACP closes during
    // initialization' (acp-process-client.test.mjs:111-122). The point is that
    // stdout can close a tick before the child does, in which case the SDK
    // reports only "connection closed" while the backend already explained
    // itself on stderr.
    let client = client(&[
        ("FAKE_STDERR", "native configuration required"),
        ("FAKE_EXIT_BEFORE_INITIALIZE", "2"),
    ]);
    let error = client.start().await.expect_err("the child exits first");
    let message = error.message(via_i18n::Locale::Zh);
    assert!(
        message.contains("native configuration required"),
        "the backend's own diagnostic must survive: {message}"
    );
    assert!(message.starts_with("Example Agent ACP "), "{message}");
    assert_eq!(
        error.body(),
        "native configuration required",
        "and it is also the error body an API client receives"
    );
}

#[tokio::test]
async fn a_prompt_turn_completes_and_collects_only_agent_message_text() {
    let updates = json!([
        { "sessionUpdate": "agent_thought_chunk",
          "content": { "type": "text", "text": "thinking out loud" } },
        { "sessionUpdate": "agent_message_chunk",
          "content": { "type": "text", "text": "  the " } },
        { "sessionUpdate": "agent_message_chunk",
          "content": { "type": "text", "text": "answer  " } },
        { "sessionUpdate": "agent_message_chunk",
          "content": { "type": "image", "data": "AA==" } },
        { "sessionUpdate": "user_message_chunk",
          "content": { "type": "text", "text": "echo" } },
    ])
    .to_string();
    let client = client(&[("FAKE_UPDATES", &updates)]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();

    let seen: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let observer = {
        let seen = Arc::clone(&seen);
        Arc::new(move |_id: &str, update: &Value| {
            seen.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(update.clone());
        })
    };

    let turn = client
        .prompt(
            &session_id,
            Prompt::text("do the thing"),
            PromptOptions {
                on_update: Some(observer),
                ..PromptOptions::default()
            },
        )
        .await
        .expect("the turn completes");

    assert_eq!(
        turn.content, "the answer",
        "reasoning, images and echoes contribute nothing; the join is trimmed once"
    );
    assert_eq!(
        seen.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len(),
        5,
        "the observer still sees every update, including the ones that are not text"
    );
    client.close().await;
}

#[tokio::test]
async fn a_second_prompt_on_one_session_is_refused_with_409() {
    let client = client(&[("FAKE_SLEEP_MS", "400")]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();

    let first = {
        let client = client.clone();
        let session_id = session_id.clone();
        tokio::spawn(async move {
            client
                .prompt(&session_id, Prompt::text("one"), PromptOptions::default())
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(80)).await;
    let error = client
        .prompt(&session_id, Prompt::text("two"), PromptOptions::default())
        .await
        .expect_err("one in-flight prompt per session");
    assert_eq!(error.status(), 409);
    assert_eq!(
        error,
        AcpError::SessionBusy {
            label: "Example Agent".to_owned(),
            id: session_id.clone(),
        }
    );
    assert!(first.await.expect("joined").is_ok());

    // And the slot is released once the first turn finishes.
    assert!(
        client
            .prompt(&session_id, Prompt::text("three"), PromptOptions::default())
            .await
            .is_ok()
    );
    client.close().await;
}

#[tokio::test]
async fn a_cancelled_stop_reason_is_an_error_and_never_a_result() {
    // The catalogued rule: "a Rust port that returns Ok on 'cancelled' would
    // silently complete cancelled Work."
    let client = client(&[("FAKE_STOP_REASON", "cancelled")]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();
    let error = client
        .prompt(&session_id, Prompt::text("stop"), PromptOptions::default())
        .await
        .expect_err("cancelled is a failure");
    assert!(matches!(error, AcpError::Cancelled { .. }));
    client.close().await;
}

#[tokio::test]
async fn a_caller_signal_cancels_the_turn_and_keeps_its_reason() {
    let client = client(&[("FAKE_SLEEP_MS", "3000")]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();

    let signal = CancelSignal::new();
    let turn = {
        let client = client.clone();
        let session_id = session_id.clone();
        let signal = signal.clone();
        tokio::spawn(async move {
            client
                .prompt(
                    &session_id,
                    Prompt::text("slow"),
                    PromptOptions {
                        signal: Some(signal),
                        ..PromptOptions::default()
                    },
                )
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(120)).await;
    signal.abort(CancelReason::Caller("the user changed their mind".into()));

    let error = tokio::time::timeout(Duration::from_secs(5), turn)
        .await
        .expect("the turn gives up promptly")
        .expect("joined")
        .expect_err("a cancelled turn is a failure");
    assert_eq!(
        error,
        AcpError::Cancelled {
            reason: Some("the user changed their mind".to_owned()),
        },
        "the caller's own reason is what the turn reports, not a generic timeout"
    );
    client.close().await;
}

#[tokio::test]
async fn a_deadline_cancels_the_turn_by_itself() {
    let client = client(&[("FAKE_SLEEP_MS", "3000")]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();
    let error = tokio::time::timeout(
        Duration::from_secs(5),
        client.prompt(
            &session_id,
            Prompt::text("slow"),
            PromptOptions {
                timeout: Some(Some(Duration::from_millis(80))),
                ..PromptOptions::default()
            },
        ),
    )
    .await
    .expect("the deadline fires well before the fixture answers")
    .expect_err("a timed-out turn is a failure");
    assert_eq!(
        error,
        AcpError::Cancelled { reason: None },
        "a deadline has no caller reason to carry"
    );
    client.close().await;
}

#[tokio::test]
async fn a_permission_request_is_answered_with_the_preferred_option() {
    let options = json!([
        { "optionId": "always", "name": "Always allow", "kind": "allow_always" },
        { "optionId": "once", "name": "Allow once", "kind": "allow_once" },
    ])
    .to_string();
    let asked: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let handler = {
        let asked = Arc::clone(&asked);
        Arc::new(move |request: via_acp::PermissionRequest| {
            asked
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(request.tool_name().to_owned());
            Box::pin(async { PermissionDecision::Approve })
                as futures::future::BoxFuture<'static, PermissionDecision>
        })
    };
    let client = AcpProcessClient::builder()
        .label("Example Agent")
        .spawn_spec(spec(&[("FAKE_REQUEST_PERMISSION", &options)]))
        .on_permission(handler)
        .build();

    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();
    let turn = client
        .prompt(
            &session_id,
            Prompt::text("ask me"),
            PromptOptions::default(),
        )
        .await
        .expect("the turn completes once permission is answered");

    assert_eq!(
        asked
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_slice(),
        ["shell"],
        "the handler reads the undeclared `name` field the agent actually sent"
    );
    let outcome = turn
        .response
        .meta
        .as_ref()
        .and_then(|meta| meta.get("permissionOutcome"))
        .cloned()
        .expect("the fixture echoes what it was told");
    assert_eq!(
        outcome,
        json!({ "outcome": "selected", "optionId": "once" }),
        "approval prefers allow_once over allow_always, even when always is offered first"
    );
    client.close().await;
}

#[tokio::test]
async fn a_permission_prompt_stops_the_turn_clock() {
    // The turn's deadline is shorter than the time the handler takes. Upstream:
    // 'pauses the prompt timeout while waiting for user permission'.
    let options =
        json!([{ "optionId": "once", "name": "Allow once", "kind": "allow_once" }]).to_string();
    let handler = Arc::new(move |_request: via_acp::PermissionRequest| {
        Box::pin(async {
            tokio::time::sleep(Duration::from_millis(400)).await;
            PermissionDecision::Approve
        }) as futures::future::BoxFuture<'static, PermissionDecision>
    });
    let client = AcpProcessClient::builder()
        .label("Example Agent")
        .spawn_spec(spec(&[("FAKE_REQUEST_PERMISSION", &options)]))
        .on_permission(handler)
        .build();
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();

    let turn = client
        .prompt(
            &session_id,
            Prompt::text("ask me"),
            PromptOptions {
                timeout: Some(Some(Duration::from_millis(200))),
                ..PromptOptions::default()
            },
        )
        .await;
    assert!(
        turn.is_ok(),
        "a turn parked on a permission prompt must not time out: {turn:?}"
    );
    client.close().await;
}

#[tokio::test]
async fn with_no_handler_a_permission_request_is_cancelled_not_rejected() {
    let options = json!([
        { "optionId": "once", "name": "Allow once", "kind": "allow_once" },
        { "optionId": "no", "name": "Reject", "kind": "reject_once" },
    ])
    .to_string();
    let client = client(&[("FAKE_REQUEST_PERMISSION", &options)]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();
    let turn = client
        .prompt(
            &session_id,
            Prompt::text("ask me"),
            PromptOptions::default(),
        )
        .await
        .expect("the turn still completes");
    let outcome = turn
        .response
        .meta
        .as_ref()
        .and_then(|meta| meta.get("permissionOutcome"))
        .cloned()
        .expect("the fixture echoes what it was told");
    assert_eq!(
        outcome,
        json!({ "outcome": "cancelled" }),
        "with nobody to decide, the answer is `cancelled` — never a rejection"
    );
    client.close().await;
}

#[tokio::test]
async fn a_json_rpc_error_is_wrapped_without_the_stderr_tail() {
    let client = client(&[
        ("FAKE_PROMPT_ERROR", "Internal error"),
        ("FAKE_ERROR_DETAILS", "missing scope: operator.write"),
        ("FAKE_STDERR", "\u{1b}[1;36m banner \u{1b}[0m"),
    ]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();
    let error = client
        .prompt(&session_id, Prompt::text("fail"), PromptOptions::default())
        .await
        .expect_err("the agent answered with an error");

    let message = error.message(via_i18n::Locale::Zh);
    assert_eq!(
        message,
        "Example Agent ACP session/prompt 失败：Internal error（missing scope: operator.write）",
        "the fullwidth colon and parentheses are catalogued"
    );
    assert!(
        !message.contains("banner"),
        "an agent that answered with an error has already said what it wants to say"
    );
    assert!(
        !message.contains('\u{1b}'),
        "and never carries terminal paint"
    );
    assert_eq!(error.body(), "missing scope: operator.write");
    client.close().await;
}

#[tokio::test]
async fn session_list_pagination_asks_for_the_arithmetic_upstream_asks_for() {
    let client = client(&[(
        "FAKE_CAPABILITIES",
        &json!({ "sessionCapabilities": { "list": {} } }).to_string(),
    )]);
    let sessions = client
        .list_sessions(Some("/work".into()), 3, None)
        .await
        .expect("the agent declares list");
    assert_eq!(sessions.len(), 3, "the result is truncated to the limit");
    let requested: Vec<u64> = sessions
        .iter()
        .filter_map(|session| {
            session
                .get("_meta")
                .and_then(|meta| meta.get("requestedLimit"))
                .and_then(Value::as_u64)
        })
        .collect();
    assert_eq!(
        requested,
        [3, 3, 3],
        "the first page asks for min(100, max(1, 3 - 0)) = 3"
    );
    client.close().await;
}

#[tokio::test]
async fn an_agent_that_does_not_declare_list_is_never_asked() {
    let client = client(&[]);
    let sessions = client
        .list_sessions(None, 20, None)
        .await
        .expect("an undeclared capability is not an error");
    assert!(sessions.is_empty());
    client.close().await;
}

#[tokio::test]
async fn resume_is_refused_when_the_agent_supports_neither_path() {
    let client = client(&[]);
    let error = client
        .resume_session(
            "old-session",
            "/work",
            Vec::new(),
            None,
            SessionDetails::default(),
        )
        .await
        .expect_err("neither resume nor loadSession is declared");
    assert_eq!(
        error,
        AcpError::ResumeUnsupported {
            label: "Example Agent".to_owned(),
        },
        "starting a fresh conversation instead would silently lose the user's context"
    );
    client.close().await;
}

#[tokio::test]
async fn resume_wins_over_load_when_both_are_declared() {
    let client = client(&[(
        "FAKE_CAPABILITIES",
        &json!({ "loadSession": true, "sessionCapabilities": { "resume": {} } }).to_string(),
    )]);
    let session = client
        .resume_session(
            "old-session",
            "/work",
            Vec::new(),
            None,
            SessionDetails::default(),
        )
        .await
        .expect("resume is taken");
    assert_eq!(
        session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .session_id,
        "old-session",
        "the agent echoed the id back, which only the resume/load path does"
    );
    client.close().await;
}

#[tokio::test]
async fn load_is_the_fallback_when_only_load_is_declared() {
    let client = client(&[(
        "FAKE_CAPABILITIES",
        &json!({ "loadSession": true }).to_string(),
    )]);
    let session = client
        .resume_session(
            "legacy-session",
            "/work",
            Vec::new(),
            None,
            SessionDetails::default(),
        )
        .await
        .expect("load is the legacy path");
    assert_eq!(
        session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .session_id,
        "legacy-session"
    );
    client.close().await;
}

#[tokio::test]
async fn closing_a_session_the_agent_cannot_close_keeps_the_local_record() {
    // The catalogued divergence: the close path drops the local record, the
    // cancel fallback does not.
    let client = client(&[]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();
    client.close_session(&session_id).await.expect("falls back");
    assert!(
        client.session(&session_id).is_some(),
        "the session still exists on the agent's side, so the record stays"
    );

    let closing = self::client(&[(
        "FAKE_CAPABILITIES",
        &json!({ "sessionCapabilities": { "close": {} } }).to_string(),
    )]);
    let session = closing
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();
    closing.close_session(&session_id).await.expect("closes");
    assert!(
        closing.session(&session_id).is_none(),
        "a closed session is dropped"
    );
    client.close().await;
    closing.close().await;
}

#[tokio::test]
async fn the_child_receives_only_the_projected_environment() {
    // The credential boundary, observed from inside the child: Rust's Command
    // inherits by default, so an agent that can see a variable nobody projected
    // is the failure this test exists to catch.
    let echo = "EXAMPLE_API_KEY,OTHER_API_KEY,VIA_ENV_LOADED,PATH,HOME";
    let client = client(&[("FAKE_ECHO_ENV", echo), ("EXAMPLE_API_KEY", "kept")]);
    let session = client
        .new_session("/work", Vec::new(), None, SessionDetails::default())
        .await
        .expect("session");
    let session_id = session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_id
        .clone();
    let turn = client
        .prompt(
            &session_id,
            Prompt::text("what do you see"),
            PromptOptions::default(),
        )
        .await
        .expect("turn");
    let seen: Value = serde_json::from_str(&turn.content).expect("the fixture reports JSON");

    assert_eq!(seen["EXAMPLE_API_KEY"], json!("kept"));
    assert_eq!(seen["VIA_ENV_LOADED"], json!("1"), "always stamped");
    assert_eq!(
        seen["OTHER_API_KEY"],
        Value::Null,
        "a namespace this policy does not declare is invisible to the child"
    );
    assert_eq!(
        seen["HOME"],
        Value::Null,
        "and the parent's own environment is not inherited: HOME was never projected"
    );
    client.close().await;
}

#[tokio::test]
#[cfg(unix)]
async fn the_teardown_ladder_escalates_past_a_child_that_ignores_sigterm() {
    // The fixture ignores SIGTERM *and* parks on stdin EOF: a wrapper launcher
    // that exits while the real agent keeps running is exactly the case that
    // makes a bare SIGKILL-after-grace, or a bare EOF, insufficient.
    let directory = tempfile::TempDir::new().expect("tempdir");
    let pid_file = directory.path().join("agent.pid");
    let client = client(&[
        ("FAKE_IGNORE_SIGTERM", "1"),
        ("FAKE_PID_FILE", &pid_file.to_string_lossy()),
    ]);
    client.start().await.expect("initialized");

    let pid: i32 = std::fs::read_to_string(&pid_file)
        .expect("the fixture recorded its pid")
        .trim()
        .parse()
        .expect("a pid");
    assert!(process_alive(pid), "the child is up before teardown");

    let started = std::time::Instant::now();
    tokio::time::timeout(Duration::from_secs(10), client.close())
        .await
        .expect("teardown completes rather than hanging on a stubborn child");
    let elapsed = started.elapsed();

    assert!(
        elapsed >= via_acp::limits::PROCESS_TREE_GRACE,
        "SIGTERM is given its full {:?} grace before SIGKILL, took {elapsed:?}",
        via_acp::limits::PROCESS_TREE_GRACE
    );
    assert!(
        elapsed < via_acp::limits::PROCESS_TREE_GRACE * 6,
        "and the grace is bounded, so shutdown fits inside the Gateway's own \
         deadline: took {elapsed:?}"
    );

    // The escalation actually landed: this is the assertion the whole ladder
    // exists for. A test that only measured elapsed time would pass against a
    // client that waited 750 ms and then gave up.
    for _ in 0..100 {
        if !process_alive(pid) {
            assert!(!client.is_ready().await);
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("the child survived SIGKILL");
}

/// Whether `pid` still names a live process.
#[cfg(unix)]
fn process_alive(pid: i32) -> bool {
    use nix::sys::signal::kill;
    use nix::unistd::Pid;
    // Signal 0 performs the permission and existence check without delivering.
    // `EPERM` means it exists but is not ours, which is still "alive".
    match kill(Pid::from_raw(pid), None) {
        Ok(()) => true,
        Err(errno) => errno == nix::errno::Errno::EPERM,
    }
}

#[tokio::test]
async fn a_prompt_turn_completes_through_the_downstream_trait() {
    // Phase 2's delivery criterion, end to end: a scripted fake ACP agent
    // completes a prompt turn through `DownstreamAgent`/`HarnessSession`.
    let directory = tempfile::TempDir::new().expect("tempdir");
    let registry = Arc::new(
        AcpSessionRegistry::builder()
            .file_path(directory.path().join("state/acp-sessions.json"))
            .build(),
    );
    let mut profile = AcpBackendProfile::new(
        // The generic ACP entry point: the one backend id this crate is allowed
        // to name, because it names the protocol rather than a product.
        "acp",
        "ACP Agent",
        spec(&[(
            "FAKE_UPDATES",
            &json!([{ "sessionUpdate": "agent_message_chunk",
                      "content": { "type": "text", "text": "done" } }])
            .to_string(),
        )]),
        capabilities(),
    );
    profile.spawn.cwd = Some(directory.path().to_path_buf());

    let agent = AcpDownstreamAgent::declare(profile, Arc::clone(&registry))
        .expect("the descriptor validates against the catalog");
    assert_eq!(agent.descriptor().id(), "acp");

    let key = SessionKey::coordinator("acp", "owner one");
    let session = agent.open(&key).await.expect("session opens");
    assert!(!session.session_id().is_empty());

    let outcome = session
        .prompt(PromptRequest::text("owner one", "do the thing"))
        .await
        .expect("the turn completes");
    assert_eq!(outcome.content(), "done");
    assert_eq!(outcome.stop_reason(), &StopReason::EndTurn);

    // The session id was persisted under the coordinator key, so a later turn
    // resumes rather than starting over.
    let record = registry.get(&key).expect("the registry recorded it");
    assert_eq!(record.session_id, session.session_id());
    assert_eq!(record.cwd, directory.path().to_string_lossy());

    assert!(agent.health().await.is_ok(), "the process is up");
    agent.client().close().await;
}

#[tokio::test]
async fn cancelling_a_delegation_needs_the_delegation_capability() {
    let directory = tempfile::TempDir::new().expect("tempdir");
    let registry = Arc::new(AcpSessionRegistry::builder().build());
    let mut profile = AcpBackendProfile::new("acp", "ACP Agent", spec(&[]), capabilities());
    profile.spawn.cwd = Some(directory.path().to_path_buf());
    let agent = AcpDownstreamAgent::declare(profile, registry).expect("declared");
    let session = agent
        .open(&SessionKey::coordinator("acp", "owner"))
        .await
        .expect("session");

    let refused = session
        .cancel(CancelScope::Delegation {
            delegation_id: "d-1".to_owned(),
        })
        .await
        .expect_err("this profile does not declare delegation");
    assert!(matches!(refused, HarnessError::CancelUnsupported));

    let outcome = session
        .cancel(CancelScope::Session)
        .await
        .expect("a transport cancel is always available");
    assert_eq!(
        outcome.status_str(),
        "cancelling",
        "a notification is a request, not a confirmation"
    );
    agent.client().close().await;
}
