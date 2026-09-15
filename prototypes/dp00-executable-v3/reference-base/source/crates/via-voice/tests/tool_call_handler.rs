//! Every catalogued output shape of the tool-call handler.
//!
//! `server/test/tool-call-handler.test.mjs`,
//! `server/test/tool-call-handler-schedule.test.mjs` and
//! `server/test/result-delivery.test.mjs`, ported.
//!
//! The shapes are contract: they are the model's whole view of what happened,
//! and a renamed field or a `null` where upstream writes an absent key changes
//! what the model says out loud.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use serde_json::{Value, json};
use tokio::sync::Mutex;
use via_conversation::{
    FrontendMemoryService, FrontendNotesStore, MarkdownContextStore, MemoryTool, NotesTool,
};
use via_i18n::{Locale, keys, t};
use via_protocol::{SessionMode, WorkKind, WorkStatus};
use via_voice::assets::InputAssetRegistry;
use via_voice::frontend::{
    FunctionOutputOptions, ResponseOutcome, ResponseRequestContext, VoiceFrontend,
};
use via_voice::input::InputPart;
use via_voice::mode::ModePlan;
use via_voice::permission::{PermissionDecision, SessionPermissionPolicy};
use via_voice::provider::{ProviderCapabilities, ProviderView};
use via_voice::response::ResponseOrigin;
use via_voice::tools::catalog;
use via_voice::tools::handler::{
    AssumeAvailable, BackendAvailability, ClientContext, CommittedTurn, DelegationRequest,
    DelegationRunners, PermissionResponder, StatusQueryRequest, ToolCall, ToolCallHandler,
    ToolCallHandlerConfig, ToolCallOutcome,
};
use via_voice::tools::transcripts::TurnTranscripts;
use via_work::{NewWork, WorkManager, WorkQuery, testing::GatedRunner, testing::ImmediateRunner};

// ---------------------------------------------------------------------------
// doubles
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct StubProvider;

impl ProviderView for StubProvider {
    fn key(&self) -> &str {
        "mock"
    }
    fn label(&self) -> &str {
        "Mock"
    }
    fn input_sample_rate(&self) -> u32 {
        16_000
    }
    fn output_sample_rate(&self) -> u32 {
        24_000
    }
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            per_response_instructions: true,
            ..ProviderCapabilities::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SentOutput {
    call_id: String,
    output: Value,
    create_response: bool,
    instructions: Option<String>,
    context: ResponseRequestContext,
}

#[derive(Debug, Default)]
struct RecordingFrontend {
    provider: StubProvider,
    sent: Mutex<Vec<SentOutput>>,
    ensured: AtomicUsize,
}

impl RecordingFrontend {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            provider: StubProvider,
            sent: Mutex::new(Vec::new()),
            ensured: AtomicUsize::new(0),
        })
    }

    async fn last(&self) -> SentOutput {
        self.sent
            .lock()
            .await
            .last()
            .cloned()
            .expect("the handler wrote an output")
    }
}

impl Default for StubProvider {
    fn default() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl VoiceFrontend for RecordingFrontend {
    fn ready(&self) -> bool {
        true
    }
    fn provider(&self) -> &dyn ProviderView {
        &self.provider
    }
    async fn append_audio(&self, _audio_base64: &str) {}
    async fn speak(
        &self,
        _text: &str,
        _origin: ResponseOrigin,
        _context: ResponseRequestContext,
    ) -> ResponseOutcome {
        ResponseOutcome::completed("resp_1")
    }
    async fn inject_result(
        &self,
        _text: &str,
        _origin: ResponseOrigin,
        _context: ResponseRequestContext,
        _inject_context: bool,
    ) -> ResponseOutcome {
        ResponseOutcome::completed("resp_1")
    }
    async fn ensure_response(
        &self,
        _context: ResponseRequestContext,
        _instructions: Option<String>,
    ) -> ResponseOutcome {
        self.ensured.fetch_add(1, Ordering::SeqCst);
        ResponseOutcome::completed("resp_1")
    }
    async fn send_function_output(
        &self,
        call_id: &str,
        output: &Value,
        context: ResponseRequestContext,
        options: FunctionOutputOptions,
    ) -> ResponseOutcome {
        self.sent.lock().await.push(SentOutput {
            call_id: call_id.to_owned(),
            output: output.clone(),
            create_response: options.create_response,
            instructions: options.instructions,
            context,
        });
        ResponseOutcome::completed("resp_1")
    }
    async fn append_user_input_context(
        &self,
        _parts: &[InputPart],
        _accompanies_voice: bool,
    ) -> ResponseOutcome {
        ResponseOutcome::default()
    }
    async fn send_user_input(
        &self,
        _parts: &[InputPart],
        _context: ResponseRequestContext,
    ) -> ResponseOutcome {
        ResponseOutcome::default()
    }
    async fn append_user_context(&self, _text: &str) -> ResponseOutcome {
        ResponseOutcome::default()
    }
    async fn update_agent_context(&self) {}
    async fn cancel(&self) {}
    async fn cancel_responses(
        &self,
        _predicate: &(dyn Fn(&ResponseRequestContext, ResponseOrigin) -> bool + Send + Sync),
    ) -> bool {
        false
    }
    async fn close(&self) {}
}

#[derive(Debug)]
struct Availability {
    configured: bool,
    ok: bool,
    known: bool,
}

impl BackendAvailability for Availability {
    fn configured(&self) -> bool {
        self.configured
    }
    fn ok(&self) -> bool {
        self.ok
    }
    fn known(&self) -> bool {
        self.known
    }
}

#[derive(Debug)]
struct Runners {
    delegation: Arc<GatedRunner>,
    status_query: Arc<GatedRunner>,
    seen_delegations: std::sync::Mutex<Vec<DelegationRequest>>,
    seen_queries: std::sync::Mutex<Vec<StatusQueryRequest>>,
}

impl Runners {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            delegation: Arc::new(GatedRunner::new()),
            status_query: Arc::new(GatedRunner::new()),
            seen_delegations: std::sync::Mutex::new(Vec::new()),
            seen_queries: std::sync::Mutex::new(Vec::new()),
        })
    }
}

impl DelegationRunners for Runners {
    fn delegation_runner(&self, request: &DelegationRequest) -> Arc<dyn via_work::WorkRunner> {
        if let Ok(mut seen) = self.seen_delegations.lock() {
            seen.push(request.clone());
        }
        Arc::clone(&self.delegation) as Arc<dyn via_work::WorkRunner>
    }
    fn delegation_canceler(&self, _request: &DelegationRequest) -> Arc<dyn via_work::WorkCanceler> {
        Arc::new(via_work::testing::ScriptedCanceler::requesting().aborting(true))
    }
    fn status_query_runner(&self, request: &StatusQueryRequest) -> Arc<dyn via_work::WorkRunner> {
        if let Ok(mut seen) = self.seen_queries.lock() {
            seen.push(request.clone());
        }
        Arc::clone(&self.status_query) as Arc<dyn via_work::WorkRunner>
    }
    fn status_query_canceler(
        &self,
        _request: &StatusQueryRequest,
    ) -> Arc<dyn via_work::WorkCanceler> {
        Arc::new(via_work::testing::ScriptedCanceler::requesting().aborting(true))
    }
    fn scheduled_task_runner(&self) -> Option<Arc<dyn via_work::WorkRunner>> {
        Some(Arc::new(ImmediateRunner::completing("done")))
    }
}

#[derive(Debug, Default)]
struct Responder {
    fails: AtomicBool,
    calls: std::sync::Mutex<Vec<(String, PermissionDecision)>>,
}

#[async_trait::async_trait]
impl PermissionResponder for Responder {
    async fn respond(
        &self,
        authorization_id: &str,
        decision: PermissionDecision,
        _owner_id: &str,
    ) -> Result<(), String> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push((authorization_id.to_owned(), decision));
        }
        if self.fails.load(Ordering::SeqCst) {
            return Err("socket closed".to_owned());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// harness
// ---------------------------------------------------------------------------

const OWNER: &str = "user_personal";
const SESSION: &str = "main";
const TURN: &str = "voice-1";

struct Harness {
    handler: ToolCallHandler,
    frontend: Arc<RecordingFrontend>,
    work: Arc<WorkManager>,
    transcripts: TurnTranscripts,
    assets: InputAssetRegistry,
    runners: Arc<Runners>,
    responder: Arc<Responder>,
    policy: Arc<Mutex<SessionPermissionPolicy>>,
    client_states: Arc<std::sync::Mutex<Vec<String>>>,
    _dir: tempfile::TempDir,
}

fn harness(availability: Availability, mode: SessionMode) -> Harness {
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let user = MarkdownContextStore::builder()
        .file_path(dir.path().join("USER.md"))
        .scope("user")
        .template("# USER")
        .build();
    let memory_document = MarkdownContextStore::builder()
        .file_path(dir.path().join("MEMORY.md"))
        .scope("memory")
        .template("# MEMORY")
        .build();
    let work = Arc::new(WorkManager::in_memory());
    let transcripts = TurnTranscripts::builder().wait_ms(20).build();
    let assets = InputAssetRegistry::new(Locale::Zh);
    let runners = Runners::new();
    let responder = Arc::new(Responder::default());
    let policy = Arc::new(Mutex::new(SessionPermissionPolicy::new()));
    let client_states: Arc<std::sync::Mutex<Vec<String>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let frontend = RecordingFrontend::new();
    let handler = ToolCallHandler::new(ToolCallHandlerConfig {
        locale: Locale::Zh,
        owner_id: OWNER.to_owned(),
        session_id: SESSION.to_owned(),
        mode: ModePlan::new(mode, true),
        work: Arc::clone(&work),
        transcripts: transcripts.clone(),
        assets: assets.clone(),
        memory: MemoryTool::new(
            Some(FrontendMemoryService::new(
                Some(user),
                Some(memory_document),
            )),
            Locale::Zh,
        ),
        notes: NotesTool::new(Some(FrontendNotesStore::in_memory()), Locale::Zh),
        availability: Arc::new(availability),
        runners: Arc::clone(&runners) as Arc<dyn DelegationRunners>,
        permission_responder: Some(Arc::clone(&responder) as Arc<dyn PermissionResponder>),
        policy: Arc::clone(&policy),
        request_client_state: Some(Arc::new({
            let client_states = Arc::clone(&client_states);
            move |state: &str| {
                if let Ok(mut calls) = client_states.lock() {
                    calls.push(state.to_owned());
                }
            }
        })),
    });
    handler.set_client_context(ClientContext {
        time_zone: "Asia/Shanghai".to_owned(),
        locale: "zh-CN".to_owned(),
        working_directory: String::new(),
        states: Vec::new(),
    });
    Harness {
        handler,
        frontend,
        work,
        transcripts,
        assets,
        runners,
        responder,
        policy,
        client_states,
        _dir: dir,
    }
}

fn healthy() -> Availability {
    Availability {
        configured: true,
        ok: true,
        known: true,
    }
}

fn call(name: &str, arguments: Value) -> ToolCall {
    ToolCall {
        call_id: format!("call_{name}"),
        name: name.to_owned(),
        arguments: arguments.to_string(),
        response_id: "resp_1".to_owned(),
        turn_id: Some(TURN.to_owned()),
        turn_generation: Some(1),
    }
}

fn committed() -> CommittedTurn {
    CommittedTurn {
        turn_id: TURN.to_owned(),
        turn_generation: 1,
    }
}

async fn answer(harness: &Harness, call: &ToolCall) -> Value {
    let outcome = harness
        .handler
        .handle(call, &committed(), harness.frontend.as_ref())
        .await;
    outcome
        .output()
        .cloned()
        .unwrap_or_else(|| panic!("expected an answered call, got {outcome:?}"))
}

// ---------------------------------------------------------------------------
// dispatch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_call_with_no_call_id_cannot_be_answered_at_all() {
    let harness = harness(healthy(), SessionMode::Agent);
    let mut call = call(catalog::GET_CURRENT_TIME, json!({}));
    call.call_id.clear();
    let outcome = harness
        .handler
        .handle(&call, &committed(), harness.frontend.as_ref())
        .await;
    assert_eq!(outcome, ToolCallOutcome::MissingCallId);
    assert!(harness.frontend.sent.lock().await.is_empty());
}

#[tokio::test]
async fn the_same_call_id_is_answered_once() {
    let harness = harness(healthy(), SessionMode::Agent);
    let call = call(catalog::GET_CURRENT_TIME, json!({}));
    answer(&harness, &call).await;
    let repeat = harness
        .handler
        .handle(&call, &committed(), harness.frontend.as_ref())
        .await;
    assert_eq!(repeat, ToolCallOutcome::Duplicate);
    assert_eq!(harness.frontend.sent.lock().await.len(), 1);
}

#[tokio::test]
async fn a_stale_call_is_closed_as_superseded_and_says_nothing() {
    let harness = harness(healthy(), SessionMode::Agent);
    let mut call = call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" }));
    call.turn_generation = Some(0);
    let outcome = harness
        .handler
        .handle(&call, &committed(), harness.frontend.as_ref())
        .await;
    assert_eq!(outcome, ToolCallOutcome::Superseded);
    let sent = harness.frontend.last().await;
    assert_eq!(
        sent.output,
        json!({
            "status": "superseded",
            "message": t(Locale::Zh, keys::VOICE_RESULT_SUPERSEDED),
        }),
    );
    assert!(
        !sent.create_response,
        "the model must not speak about a turn the user moved past",
    );
    assert!(harness.work.list(WorkQuery::owner(OWNER)).await.is_empty());
}

#[tokio::test]
async fn a_call_that_predates_a_committed_turn_is_not_stale() {
    let harness = harness(healthy(), SessionMode::Agent);
    let mut call = call(catalog::GET_CURRENT_TIME, json!({}));
    call.turn_id = None;
    let output = answer(&harness, &call).await;
    assert_eq!(output["status"], "ok");
}

#[tokio::test]
async fn an_unknown_tool_is_refused_with_the_catalogued_code() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(&harness, &call("rm_rf", json!({}))).await;
    assert_eq!(output["error_code"], "unsupported_tool");
    assert_eq!(output["status"], "failed");
    assert_eq!(output["error"], true);
    assert_eq!(output["retryable"], false);
    assert_eq!(
        output["user_message"],
        json!(t(Locale::Zh, keys::VOICE_ERROR_OPERATION_UNAVAILABLE)),
    );
}

#[tokio::test]
async fn spawn_thinking_is_not_declared_in_direct_mode_and_is_refused() {
    let harness = harness(healthy(), SessionMode::Direct);
    let output = answer(
        &harness,
        &call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" })),
    )
    .await;
    assert_eq!(output["error_code"], "unsupported_tool");
    assert!(harness.work.list(WorkQuery::owner(OWNER)).await.is_empty());
}

#[tokio::test]
async fn dictation_declares_nothing_so_every_tool_is_refused() {
    let harness = harness(healthy(), SessionMode::Dictation);
    for name in catalog::ALL_TOOL_NAMES {
        let output = answer(&harness, &call(name, json!({ "action": "lists" }))).await;
        assert_eq!(output["error_code"], "unsupported_tool", "{name}");
    }
}

// ---------------------------------------------------------------------------
// get_current_time
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_current_time_answers_the_snapshot_in_field_order() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(&harness, &call(catalog::GET_CURRENT_TIME, json!({}))).await;
    let Value::Object(object) = &output else {
        panic!("an object");
    };
    assert_eq!(
        object.keys().collect::<Vec<_>>(),
        vec!["status", "iso_utc", "local_time", "time_zone", "locale"],
    );
    assert_eq!(output["status"], "ok");
    assert_eq!(output["time_zone"], "Asia/Shanghai");
    assert_eq!(output["locale"], "zh-CN");
}

// ---------------------------------------------------------------------------
// schedule_reminder
// ---------------------------------------------------------------------------

#[tokio::test]
async fn schedule_reminder_answers_the_catalogued_success_shape() {
    let harness = harness(healthy(), SessionMode::Agent);
    let at = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let output = answer(
        &harness,
        &call(
            catalog::SCHEDULE_REMINDER,
            json!({ "execute_at": at, "reminder": "喝水", "type": "reminder" }),
        ),
    )
    .await;
    assert_eq!(output["status"], "scheduled");
    assert_eq!(output["execute_at"], json!(at));
    assert_eq!(output["type"], "reminder");
    assert_eq!(output["recurrence"], "once", "the default recurrence");
    let reminder_id = output["reminder_id"].as_str().expect("an id");
    assert!(reminder_id.starts_with("work_"));

    let sent = harness.frontend.last().await;
    assert_eq!(
        sent.instructions,
        Some(t(Locale::Zh, keys::VOICE_INSTRUCTIONS_REMINDER_CONFIRMATION).to_owned()),
    );
    assert_eq!(sent.context.task_id.as_deref(), Some(reminder_id));

    let stored = harness
        .work
        .get(reminder_id, Some(OWNER))
        .await
        .expect("the Work exists");
    assert_eq!(stored.kind, WorkKind::Reminder);
    assert_eq!(stored.status, WorkStatus::Scheduled);
}

#[tokio::test]
async fn a_scheduled_task_becomes_a_scheduled_task_kind() {
    let harness = harness(healthy(), SessionMode::Agent);
    let at = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let output = answer(
        &harness,
        &call(
            catalog::SCHEDULE_REMINDER,
            json!({
                "execute_at": at,
                "reminder": "跑测试再告诉我",
                "type": "task",
                "recurrence": "daily",
            }),
        ),
    )
    .await;
    assert_eq!(output["type"], "task");
    assert_eq!(output["recurrence"], "daily");
    let stored = harness
        .work
        .get(
            output["reminder_id"].as_str().unwrap_or_default(),
            Some(OWNER),
        )
        .await
        .expect("the Work exists");
    assert_eq!(stored.kind, WorkKind::ScheduledTask);
}

#[tokio::test]
async fn every_unusable_execute_at_is_invalid_time() {
    let harness = harness(healthy(), SessionMode::Agent);
    let past = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
    let cases = [
        json!({ "execute_at": past, "reminder": "x" }),
        json!({ "execute_at": "not a time", "reminder": "x" }),
        json!({ "execute_at": "", "reminder": "x" }),
        json!({ "reminder": "x" }),
        json!({ "execute_at": "1970-01-01T00:00:00Z", "reminder": "x" }),
    ];
    for (index, arguments) in cases.into_iter().enumerate() {
        let mut request = call(catalog::SCHEDULE_REMINDER, arguments);
        request.call_id = format!("call_{index}");
        let output = answer(&harness, &request).await;
        assert_eq!(output["status"], "error", "{index}");
        assert_eq!(output["error"], true, "{index}");
        assert_eq!(output["error_code"], "invalid_time", "{index}");
        assert_eq!(
            output["user_message"],
            json!(t(Locale::Zh, keys::VOICE_ERROR_INVALID_TIME)),
            "{index}",
        );
        // The failure shape has exactly these four keys — no `retryable`.
        let Value::Object(object) = &output else {
            panic!("an object");
        };
        assert_eq!(
            object.keys().collect::<Vec<_>>(),
            vec!["status", "error", "error_code", "user_message"],
            "{index}",
        );
    }
    assert!(harness.work.list(WorkQuery::owner(OWNER)).await.is_empty());
}

// ---------------------------------------------------------------------------
// spawn_thinking
// ---------------------------------------------------------------------------

#[tokio::test]
async fn spawn_thinking_accepts_and_carries_the_thinking_marker() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(
        &harness,
        &call(
            catalog::SPAWN_THINKING,
            json!({ "objective": "查一下 天气" }),
        ),
    )
    .await;
    assert_eq!(output["status"], "accepted");
    assert_eq!(output["marker"], "[thinking]");
    let work_id = output["work_id"].as_str().expect("a work id");
    assert!(work_id.starts_with("work_"));

    let sent = harness.frontend.last().await;
    assert!(sent.create_response);
    assert!(
        sent.instructions
            .as_deref()
            .is_some_and(|text| text.contains(t(
                Locale::Zh,
                keys::VOICE_INSTRUCTIONS_ACCEPTED_NOT_FINISHED
            ))),
    );

    let seen = harness
        .runners
        .seen_delegations
        .lock()
        .expect("not poisoned")
        .clone();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].objective, "查一下 天气", "whitespace is collapsed");
    assert_eq!(seen[0].turn_id, TURN);
}

#[tokio::test]
async fn a_second_spawn_thinking_on_the_same_turn_is_a_duplicate() {
    let harness = harness(healthy(), SessionMode::Agent);
    let first = answer(
        &harness,
        &call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" })),
    )
    .await;
    let mut second = call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" }));
    second.call_id = "call_second".to_owned();
    let output = answer(&harness, &second).await;
    assert_eq!(output["status"], "duplicate");
    assert_eq!(output["work_id"], first["work_id"]);
    assert_eq!(
        output["message"],
        json!(t(Locale::Zh, keys::VOICE_RESULT_DUPLICATE)),
    );
    assert!(output.get("marker").is_none(), "a duplicate has no marker");
    assert_eq!(harness.work.list(WorkQuery::owner(OWNER)).await.len(), 1);
}

#[tokio::test]
async fn an_absent_objective_falls_back_to_the_transcript() {
    let harness = harness(healthy(), SessionMode::Agent);
    harness.transcripts.record(TURN, "帮我看看昨天的日志").await;
    let output = answer(&harness, &call(catalog::SPAWN_THINKING, json!({}))).await;
    assert_eq!(output["status"], "accepted");
    let seen = harness
        .runners
        .seen_delegations
        .lock()
        .expect("not poisoned")
        .clone();
    assert_eq!(seen[0].objective, "帮我看看昨天的日志");
}

#[tokio::test]
async fn an_objective_that_cannot_be_recovered_is_missing_objective() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(&harness, &call(catalog::SPAWN_THINKING, json!({}))).await;
    assert_eq!(output["error_code"], "missing_objective");
    assert_eq!(output["retryable"], true);
    assert_eq!(
        output["user_message"],
        json!(t(Locale::Zh, keys::VOICE_ERROR_OBJECTIVE_INCOMPLETE)),
    );
}

#[tokio::test]
async fn malformed_arguments_are_read_as_missing_fields() {
    let harness = harness(healthy(), SessionMode::Agent);
    let mut request = call(catalog::SPAWN_THINKING, json!({}));
    request.arguments = "{\"objective\": \"查天".to_owned();
    let output = answer(&harness, &request).await;
    assert_eq!(
        output["error_code"], "missing_objective",
        "truncated JSON is the same as an omitted field",
    );
}

#[tokio::test]
async fn an_unconfigured_backend_refuses_without_retryable() {
    let harness = harness(
        Availability {
            configured: false,
            ok: false,
            known: true,
        },
        SessionMode::Agent,
    );
    let output = answer(
        &harness,
        &call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" })),
    )
    .await;
    assert_eq!(output["error_code"], "backend_unavailable");
    assert_eq!(output["retryable"], false, "there is nothing to wait for");
    assert_eq!(
        output["user_message"],
        json!(t(Locale::Zh, keys::VOICE_ERROR_BACKEND_NOT_CONFIGURED)),
    );
    let sent = harness.frontend.last().await;
    assert_eq!(
        sent.instructions,
        Some(t(Locale::Zh, keys::VOICE_INSTRUCTIONS_BACKEND_NOT_CONFIGURED).to_owned()),
    );
    assert!(harness.work.list(WorkQuery::owner(OWNER)).await.is_empty());
}

#[tokio::test]
async fn a_disconnected_backend_refuses_but_is_retryable() {
    let harness = harness(
        Availability {
            configured: true,
            ok: false,
            known: true,
        },
        SessionMode::Agent,
    );
    let output = answer(
        &harness,
        &call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" })),
    )
    .await;
    assert_eq!(output["error_code"], "backend_unavailable");
    assert_eq!(output["retryable"], true, "the backend may come back");
    assert_eq!(
        output["user_message"],
        json!(t(Locale::Zh, keys::VOICE_ERROR_BACKEND_DISCONNECTED)),
    );
}

#[tokio::test]
async fn an_unsettled_probe_accepts_optimistically() {
    let harness = harness(
        Availability {
            configured: true,
            ok: false,
            known: false,
        },
        SessionMode::Agent,
    );
    let output = answer(
        &harness,
        &call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" })),
    )
    .await;
    assert_eq!(
        output["status"], "accepted",
        "an unknown probe must not refuse every delegation",
    );
}

#[tokio::test]
async fn the_default_availability_is_optimistic() {
    assert!(AssumeAvailable.configured());
    assert!(AssumeAvailable.ok());
    assert!(!AssumeAvailable.known());
}

#[tokio::test]
async fn an_expired_input_reference_is_refused_before_any_work_is_created() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(
        &harness,
        &call(
            catalog::SPAWN_THINKING,
            json!({ "objective": "看看这张图", "input_refs": ["input_9"] }),
        ),
    )
    .await;
    assert_eq!(output["error_code"], "invalid_input_ref");
    assert_eq!(output["retryable"], true);
    assert_eq!(
        output["user_message"],
        json!(t(Locale::Zh, keys::VOICE_ERROR_INPUT_REF_EXPIRED)),
    );
    assert!(harness.work.list(WorkQuery::owner(OWNER)).await.is_empty());
}

#[tokio::test]
async fn a_resolvable_input_reference_travels_with_the_delegation() {
    let harness = harness(healthy(), SessionMode::Agent);
    let registered = harness.assets.register_parts(
        OWNER,
        SESSION,
        "voice-0",
        &[InputPart::file("image/png", "data:image/png;base64,AAAA")],
    );
    assert_eq!(registered[0].reference(), "input_1");
    let output = answer(
        &harness,
        &call(
            catalog::SPAWN_THINKING,
            json!({ "objective": "看看这张图", "input_refs": ["input_1"] }),
        ),
    )
    .await;
    assert_eq!(output["status"], "accepted");
    let seen = harness
        .runners
        .seen_delegations
        .lock()
        .expect("not poisoned")
        .clone();
    assert_eq!(seen[0].input_parts.len(), 1);
    assert_eq!(seen[0].input_parts[0].reference(), "input_1");
}

#[tokio::test]
async fn this_turns_attachments_are_merged_with_the_referenced_ones_without_duplicates() {
    let harness = harness(healthy(), SessionMode::Agent);
    let this_turn = harness.assets.register_parts(
        OWNER,
        SESSION,
        TURN,
        &[InputPart::file("image/png", "data:image/png;base64,AAAA")],
    );
    harness.transcripts.record_parts(TURN, this_turn).await;
    let output = answer(
        &harness,
        &call(
            catalog::SPAWN_THINKING,
            json!({ "objective": "看看这张图", "input_refs": ["input_1"] }),
        ),
    )
    .await;
    assert_eq!(output["status"], "accepted");
    let seen = harness
        .runners
        .seen_delegations
        .lock()
        .expect("not poisoned")
        .clone();
    assert_eq!(
        seen[0].input_parts.len(),
        1,
        "the same asset referenced twice is delegated once",
    );
}

// ---------------------------------------------------------------------------
// permission
// ---------------------------------------------------------------------------

async fn work_with_pending_permission(harness: &Harness) -> String {
    let acceptance = harness
        .work
        .create(
            NewWork::new("跑测试", OWNER)
                .session(SESSION)
                .turn(TURN)
                .runner(Arc::clone(&harness.runners.delegation) as Arc<dyn via_work::WorkRunner>),
        )
        .await
        .expect("the manager is running");
    let permission = via_work::PendingPermission::pending(
        "auth_0123456789abcdef0123456789abcdef",
        "shell",
        "shell：rm -rf /tmp/x",
    );
    harness
        .runners
        .delegation
        .emit(via_work::RunnerEvent::PermissionRequested(permission))
        .await;
    // Let the manager fold the event into the record.
    for _ in 0..20 {
        tokio::task::yield_now().await;
        if harness
            .work
            .get(&acceptance.work.id, Some(OWNER))
            .await
            .and_then(|work| work.authorization)
            .is_some()
        {
            break;
        }
    }
    acceptance.work.id
}

#[tokio::test]
async fn a_pending_permission_blocks_a_new_delegation() {
    let harness = harness(healthy(), SessionMode::Agent);
    let work_id = work_with_pending_permission(&harness).await;
    let output = answer(
        &harness,
        &call(
            catalog::SPAWN_THINKING,
            json!({ "objective": "再查一件事" }),
        ),
    )
    .await;
    assert_eq!(output["status"], "authorization_pending");
    assert_eq!(output["error"], true);
    assert_eq!(output["error_code"], "permission_decision_required");
    assert_eq!(
        output["authorization_id"],
        "auth_0123456789abcdef0123456789abcdef"
    );
    assert_eq!(output["operation"], "shell：rm -rf /tmp/x");
    assert_eq!(output["retryable"], true);
    assert_eq!(
        output["user_message"],
        json!(t(
            Locale::Zh,
            keys::VOICE_ERROR_PERMISSION_DECISION_REQUIRED
        )),
    );
    let sent = harness.frontend.last().await;
    assert_eq!(sent.context.task_id.as_deref(), Some(work_id.as_str()));
    assert_eq!(
        sent.instructions,
        Some(t(Locale::Zh, keys::VOICE_INSTRUCTIONS_PERMISSION_PENDING).to_owned()),
    );
    assert_eq!(
        harness.work.list(WorkQuery::owner(OWNER)).await.len(),
        1,
        "no second Work was created",
    );
}

#[tokio::test]
async fn a_permission_answer_is_submitted_and_takes_effect_locally_at_once() {
    let harness = harness(healthy(), SessionMode::Agent);
    let work_id = work_with_pending_permission(&harness).await;
    harness.transcripts.record(TURN, "可以").await;
    let output = answer(
        &harness,
        &call(
            catalog::RESPOND_AGENT_PERMISSION,
            json!({
                "authorization_id": "auth_0123456789abcdef0123456789abcdef",
                "decision": "always",
            }),
        ),
    )
    .await;
    assert_eq!(
        output,
        json!({
            "status": "submitted",
            "authorization_id": "auth_0123456789abcdef0123456789abcdef",
        }),
    );
    let sent = harness.frontend.last().await;
    assert_eq!(sent.context.task_id.as_deref(), Some(work_id.as_str()));
    assert_eq!(
        sent.instructions,
        Some(
            t(
                Locale::Zh,
                keys::VOICE_INSTRUCTIONS_PERMISSION_SUBMITTED_ALWAYS
            )
            .to_owned()
        ),
    );
    assert!(
        harness
            .policy
            .lock()
            .await
            .should_auto_allow(OWNER, SESSION),
        "the relaxation is immediate, not held behind the round trip",
    );
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    let calls = harness
        .responder
        .calls
        .lock()
        .expect("not poisoned")
        .clone();
    assert_eq!(
        calls,
        vec![(
            "auth_0123456789abcdef0123456789abcdef".to_owned(),
            PermissionDecision::Always,
        )]
    );
}

#[tokio::test]
async fn a_rejection_does_not_relax_the_session() {
    let harness = harness(healthy(), SessionMode::Agent);
    work_with_pending_permission(&harness).await;
    harness.transcripts.record(TURN, "不要").await;
    let output = answer(
        &harness,
        &call(
            catalog::RESPOND_AGENT_PERMISSION,
            json!({
                "authorization_id": "auth_0123456789abcdef0123456789abcdef",
                "decision": "reject",
            }),
        ),
    )
    .await;
    assert_eq!(output["status"], "submitted");
    assert!(
        !harness
            .policy
            .lock()
            .await
            .should_auto_allow(OWNER, SESSION)
    );
    let sent = harness.frontend.last().await;
    assert_eq!(
        sent.instructions,
        Some(
            t(
                Locale::Zh,
                keys::VOICE_INSTRUCTIONS_PERMISSION_SUBMITTED_REJECT
            )
            .to_owned()
        ),
    );
}

#[tokio::test]
async fn a_failed_delivery_rolls_the_local_relaxation_back() {
    let harness = harness(healthy(), SessionMode::Agent);
    work_with_pending_permission(&harness).await;
    harness.responder.fails.store(true, Ordering::SeqCst);
    harness.transcripts.record(TURN, "可以").await;
    answer(
        &harness,
        &call(
            catalog::RESPOND_AGENT_PERMISSION,
            json!({
                "authorization_id": "auth_0123456789abcdef0123456789abcdef",
                "decision": "always",
            }),
        ),
    )
    .await;
    for _ in 0..50 {
        tokio::task::yield_now().await;
        if !harness
            .policy
            .lock()
            .await
            .should_auto_allow(OWNER, SESSION)
        {
            break;
        }
    }
    assert!(
        !harness
            .policy
            .lock()
            .await
            .should_auto_allow(OWNER, SESSION),
        "a decision that never reached the backend must not stay in force",
    );
}

#[tokio::test]
async fn a_permission_answer_on_a_silent_turn_is_refused() {
    let harness = harness(healthy(), SessionMode::Agent);
    work_with_pending_permission(&harness).await;
    // No transcript: the model may not approve on the user's behalf.
    let output = answer(
        &harness,
        &call(
            catalog::RESPOND_AGENT_PERMISSION,
            json!({
                "authorization_id": "auth_0123456789abcdef0123456789abcdef",
                "decision": "always",
            }),
        ),
    )
    .await;
    assert_eq!(output["error_code"], "invalid_permission_response");
    assert!(
        !harness
            .policy
            .lock()
            .await
            .should_auto_allow(OWNER, SESSION)
    );
    assert!(
        harness
            .responder
            .calls
            .lock()
            .expect("not poisoned")
            .is_empty(),
    );
}

#[tokio::test]
async fn an_invalid_decision_or_missing_id_is_refused() {
    let harness = harness(healthy(), SessionMode::Agent);
    work_with_pending_permission(&harness).await;
    harness.transcripts.record(TURN, "可以").await;
    let cases = [
        json!({ "authorization_id": "auth_x", "decision": "maybe" }),
        json!({ "authorization_id": "", "decision": "always" }),
        json!({ "decision": "always" }),
        json!({ "authorization_id": "auth_x" }),
    ];
    for (index, arguments) in cases.into_iter().enumerate() {
        let mut request = call(catalog::RESPOND_AGENT_PERMISSION, arguments);
        request.call_id = format!("call_{index}");
        let output = answer(&harness, &request).await;
        assert_eq!(
            output["error_code"], "invalid_permission_response",
            "{index}"
        );
    }
}

#[tokio::test]
async fn an_authorization_that_is_no_longer_pending_says_so() {
    let harness = harness(healthy(), SessionMode::Agent);
    harness.transcripts.record(TURN, "可以").await;
    let output = answer(
        &harness,
        &call(
            catalog::RESPOND_AGENT_PERMISSION,
            json!({ "authorization_id": "auth_gone", "decision": "always" }),
        ),
    )
    .await;
    assert_eq!(output["error_code"], "permission_not_pending");
    assert_eq!(output["retryable"], false);
    assert_eq!(
        output["user_message"],
        json!(t(
            Locale::Zh,
            keys::VOICE_INSTRUCTIONS_PERMISSION_ALREADY_HANDLED
        )),
    );
}

// ---------------------------------------------------------------------------
// cancel_agent_task
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cancel_with_nothing_running_answers_not_found() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(&harness, &call(catalog::CANCEL_AGENT_TASK, json!({}))).await;
    assert_eq!(
        output,
        json!({
            "status": "not_found",
            "message": t(Locale::Zh, keys::VOICE_RESULT_CANCEL_NOT_FOUND),
        }),
    );
}

#[tokio::test]
async fn cancel_targets_the_most_recent_cancellable_work_when_given_no_id() {
    let harness = harness(healthy(), SessionMode::Agent);
    let accepted = answer(
        &harness,
        &call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" })),
    )
    .await;
    let work_id = accepted["work_id"].as_str().expect("a work id").to_owned();
    let output = answer(&harness, &call(catalog::CANCEL_AGENT_TASK, json!({}))).await;
    assert_eq!(output["status"], "cancelled");
    assert_eq!(output["work_id"], json!(work_id));
    assert_eq!(
        output["message"],
        json!(t(Locale::Zh, keys::VOICE_RESULT_CANCELLED)),
    );
}

#[tokio::test]
async fn cancelling_a_finished_work_answers_not_active() {
    let harness = harness(healthy(), SessionMode::Agent);
    let acceptance = harness
        .work
        .create(
            NewWork::new("done already", OWNER)
                .session(SESSION)
                .runner(Arc::new(ImmediateRunner::completing("ok"))),
        )
        .await
        .expect("the manager is running");
    harness.work.wait(&acceptance.work.id).await;
    let output = answer(
        &harness,
        &call(
            catalog::CANCEL_AGENT_TASK,
            json!({ "work_id": acceptance.work.id }),
        ),
    )
    .await;
    assert_eq!(output["status"], "not_active");
    assert_eq!(output["work_id"], json!(acceptance.work.id));
    assert_eq!(
        output["message"],
        json!(t(Locale::Zh, keys::VOICE_RESULT_CANCEL_NOT_ACTIVE)),
    );
}

#[tokio::test]
async fn cancelling_an_unknown_id_answers_not_active() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(
        &harness,
        &call(
            catalog::CANCEL_AGENT_TASK,
            json!({ "work_id": "work_nope" }),
        ),
    )
    .await;
    assert_eq!(output["status"], "not_active");
}

// ---------------------------------------------------------------------------
// get_agent_task_status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn status_with_no_work_answers_not_found() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(&harness, &call(catalog::GET_AGENT_TASK_STATUS, json!({}))).await;
    assert_eq!(
        output,
        json!({
            "status": "not_found",
            "message": t(Locale::Zh, keys::VOICE_RESULT_NO_QUERYABLE_WORK),
        }),
    );
}

#[tokio::test]
async fn list_all_answers_the_catalogued_row_shape_and_caps_at_twenty() {
    let harness = harness(healthy(), SessionMode::Agent);
    for index in 0..25 {
        harness
            .work
            .create(
                NewWork::new(&format!("objective {index}"), OWNER)
                    .session(SESSION)
                    .runner(
                        Arc::clone(&harness.runners.delegation) as Arc<dyn via_work::WorkRunner>
                    ),
            )
            .await
            .expect("the manager is running");
    }
    let output = answer(
        &harness,
        &call(catalog::GET_AGENT_TASK_STATUS, json!({ "list_all": true })),
    )
    .await;
    assert_eq!(output["status"], "ok");
    assert_eq!(output["count"], 20, "the list is capped");
    let rows = output["tasks"].as_array().expect("an array");
    assert_eq!(rows.len(), 20);
    let Value::Object(row) = &rows[0] else {
        panic!("an object");
    };
    assert_eq!(
        row.keys().collect::<Vec<_>>(),
        vec![
            "work_id",
            "status",
            "kind",
            "objective",
            "execute_at",
            "recurrence"
        ],
    );
    assert_eq!(rows[0]["execute_at"], Value::Null, "an unscheduled Work");
    assert_eq!(rows[0]["recurrence"], Value::Null);
}

#[tokio::test]
async fn list_all_with_nothing_answers_empty() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(
        &harness,
        &call(catalog::GET_AGENT_TASK_STATUS, json!({ "list_all": true })),
    )
    .await;
    assert_eq!(output["status"], "empty");
    assert_eq!(output["count"], 0);
    assert_eq!(output["tasks"], json!([]));
}

#[tokio::test]
async fn list_all_clips_a_long_objective_at_three_hundred() {
    let harness = harness(healthy(), SessionMode::Agent);
    harness
        .work
        .create(
            NewWork::new(&"阿".repeat(500), OWNER)
                .session(SESSION)
                .runner(Arc::clone(&harness.runners.delegation) as Arc<dyn via_work::WorkRunner>),
        )
        .await
        .expect("the manager is running");
    let output = answer(
        &harness,
        &call(catalog::GET_AGENT_TASK_STATUS, json!({ "list_all": true })),
    )
    .await;
    let objective = output["tasks"][0]["objective"].as_str().expect("a string");
    assert_eq!(objective.chars().count(), 300);
}

#[tokio::test]
async fn a_scheduled_row_carries_its_iso_trigger_and_recurrence() {
    let harness = harness(healthy(), SessionMode::Agent);
    let at = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    answer(
        &harness,
        &call(
            catalog::SCHEDULE_REMINDER,
            json!({ "execute_at": at, "reminder": "喝水", "recurrence": "daily" }),
        ),
    )
    .await;
    let mut request = call(catalog::GET_AGENT_TASK_STATUS, json!({ "list_all": true }));
    request.call_id = "call_list".to_owned();
    let output = answer(&harness, &request).await;
    let row = &output["tasks"][0];
    assert_eq!(row["kind"], "reminder");
    assert_eq!(row["recurrence"], "daily");
    assert!(
        row["execute_at"]
            .as_str()
            .is_some_and(|value| value.ends_with('Z')),
        "the trigger is rendered as an ISO instant",
    );
}

#[tokio::test]
async fn a_single_status_answers_the_catalogued_field_set() {
    let harness = harness(healthy(), SessionMode::Agent);
    let accepted = answer(
        &harness,
        &call(catalog::SPAWN_THINKING, json!({ "objective": "查天气" })),
    )
    .await;
    let work_id = accepted["work_id"].as_str().expect("a work id").to_owned();
    let mut request = call(
        catalog::GET_AGENT_TASK_STATUS,
        json!({ "work_id": work_id }),
    );
    request.call_id = "call_status".to_owned();
    let output = answer(&harness, &request).await;
    let Value::Object(object) = &output else {
        panic!("an object");
    };
    assert_eq!(
        object.keys().collect::<Vec<_>>(),
        vec![
            "status",
            "work_id",
            "work_status",
            "objective",
            "elapsed_ms",
            "delegation",
            "authorization_pending",
            "last_activity",
            "result",
            "error",
        ],
    );
    assert_eq!(output["status"], "ok");
    assert_eq!(output["authorization_pending"], false);
    assert_eq!(output["delegation"], Value::Null);
    assert_eq!(output["last_activity"], Value::Null);
    assert_eq!(output["result"], Value::Null, "not completed yet");
    assert_eq!(output["error"], Value::Null);
}

#[tokio::test]
async fn a_completed_status_clips_the_result_at_five_hundred() {
    let harness = harness(healthy(), SessionMode::Agent);
    let acceptance = harness
        .work
        .create(
            NewWork::new("查天气", OWNER)
                .session(SESSION)
                .runner(Arc::new(ImmediateRunner::completing(&"阿".repeat(900)))),
        )
        .await
        .expect("the manager is running");
    harness.work.wait(&acceptance.work.id).await;
    let output = answer(&harness, &call(catalog::GET_AGENT_TASK_STATUS, json!({}))).await;
    assert_eq!(output["work_status"], "completed");
    let result = output["result"].as_str().expect("a string");
    assert_eq!(result.chars().count(), 500);
}

#[tokio::test]
async fn answering_a_status_query_for_a_pending_notification_consumes_it() {
    let harness = harness(healthy(), SessionMode::Agent);
    let acceptance = harness
        .work
        .create(
            NewWork::new("查天气", OWNER)
                .session(SESSION)
                .runner(Arc::new(ImmediateRunner::completing("晴"))),
        )
        .await
        .expect("the manager is running");
    harness.work.wait(&acceptance.work.id).await;
    answer(&harness, &call(catalog::GET_AGENT_TASK_STATUS, json!({}))).await;
    let sent = harness.frontend.last().await;
    assert!(
        sent.context.consumes_task_notification,
        "the user just heard the result; the announcement must not repeat it",
    );
}

#[tokio::test]
async fn a_delegated_work_opens_a_status_query_and_says_so() {
    let harness = harness(healthy(), SessionMode::Agent);
    let acceptance = harness
        .work
        .create(
            NewWork::new("跑一个长项目", OWNER)
                .session(SESSION)
                .runner(Arc::clone(&harness.runners.delegation) as Arc<dyn via_work::WorkRunner>),
        )
        .await
        .expect("the manager is running");
    harness
        .runners
        .delegation
        .emit(via_work::RunnerEvent::Delegated(
            via_work::DelegationRef::new("deleg_1", "跑一个长项目"),
        ))
        .await;
    for _ in 0..40 {
        tokio::task::yield_now().await;
        if harness
            .work
            .get(&acceptance.work.id, Some(OWNER))
            .await
            .is_some_and(|work| work.status == WorkStatus::Delegated)
        {
            break;
        }
    }

    let output = answer(
        &harness,
        &call(
            catalog::GET_AGENT_TASK_STATUS,
            json!({ "question": "现在到哪一步了" }),
        ),
    )
    .await;
    assert_eq!(output["status"], "querying");
    assert_eq!(output["work_id"], json!(acceptance.work.id));
    let query_id = output["query_work_id"].as_str().expect("a query id");
    assert_ne!(query_id, acceptance.work.id);
    assert_eq!(
        output["message"],
        json!(t(Locale::Zh, keys::VOICE_RESULT_QUERYING)),
    );

    let seen = harness
        .runners
        .seen_queries
        .lock()
        .expect("not poisoned")
        .clone();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].question, "现在到哪一步了");
    assert_eq!(seen[0].parent_work_id, acceptance.work.id);

    let query = harness
        .work
        .get(query_id, Some(OWNER))
        .await
        .expect("the query exists");
    assert_eq!(query.kind, WorkKind::Control);
    assert_eq!(
        query.parent_work_id.as_deref(),
        Some(acceptance.work.id.as_str())
    );
    assert!(
        query.objective.contains("跑一个长项目"),
        "the query names what it is asking about: {}",
        query.objective,
    );

    // A second query while the first is in flight reports the existing one.
    let mut again = call(catalog::GET_AGENT_TASK_STATUS, json!({}));
    again.call_id = "call_again".to_owned();
    let output = answer(&harness, &again).await;
    assert_eq!(output["status"], "querying");
    assert_eq!(output["query_work_id"], json!(query_id));
    assert_eq!(
        output["message"],
        json!(t(Locale::Zh, keys::VOICE_RESULT_QUERY_ALREADY_IN_FLIGHT)),
    );
    assert_eq!(
        harness
            .runners
            .seen_queries
            .lock()
            .expect("not poisoned")
            .len(),
        1,
        "no second query was submitted",
    );
}

#[tokio::test]
async fn a_delegated_query_clips_the_quoted_objective_at_two_hundred() {
    let harness = harness(healthy(), SessionMode::Agent);
    let long = "阿".repeat(400);
    let acceptance = harness
        .work
        .create(
            NewWork::new(&long, OWNER)
                .session(SESSION)
                .runner(Arc::clone(&harness.runners.delegation) as Arc<dyn via_work::WorkRunner>),
        )
        .await
        .expect("the manager is running");
    harness
        .runners
        .delegation
        .emit(via_work::RunnerEvent::Delegated(
            via_work::DelegationRef::new("deleg_1", "t"),
        ))
        .await;
    for _ in 0..40 {
        tokio::task::yield_now().await;
        if harness
            .work
            .get(&acceptance.work.id, Some(OWNER))
            .await
            .is_some_and(|work| work.status == WorkStatus::Delegated)
        {
            break;
        }
    }
    let output = answer(&harness, &call(catalog::GET_AGENT_TASK_STATUS, json!({}))).await;
    let query = harness
        .work
        .get(
            output["query_work_id"].as_str().unwrap_or_default(),
            Some(OWNER),
        )
        .await
        .expect("the query exists");
    let quoted = query.objective.matches('阿').count();
    assert_eq!(quoted, 200, "the quoted objective is clipped");
}

#[tokio::test]
async fn a_delegated_query_with_no_question_falls_back_to_the_default() {
    let harness = harness(healthy(), SessionMode::Agent);
    let acceptance = harness
        .work
        .create(
            NewWork::new("项目", OWNER)
                .session(SESSION)
                .runner(Arc::clone(&harness.runners.delegation) as Arc<dyn via_work::WorkRunner>),
        )
        .await
        .expect("the manager is running");
    harness
        .runners
        .delegation
        .emit(via_work::RunnerEvent::Delegated(
            via_work::DelegationRef::new("deleg_1", "t"),
        ))
        .await;
    for _ in 0..40 {
        tokio::task::yield_now().await;
        if harness
            .work
            .get(&acceptance.work.id, Some(OWNER))
            .await
            .is_some_and(|work| work.status == WorkStatus::Delegated)
        {
            break;
        }
    }
    answer(&harness, &call(catalog::GET_AGENT_TASK_STATUS, json!({}))).await;
    let seen = harness
        .runners
        .seen_queries
        .lock()
        .expect("not poisoned")
        .clone();
    assert_eq!(
        seen[0].question,
        t(Locale::Zh, keys::VOICE_INSTRUCTIONS_DELEGATED_STATUS_QUERY),
    );
}

// ---------------------------------------------------------------------------
// enter_sleep
// ---------------------------------------------------------------------------

#[tokio::test]
async fn enter_sleep_is_refused_by_a_client_that_cannot_sleep() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(&harness, &call(catalog::ENTER_SLEEP, json!({}))).await;
    assert_eq!(output["error_code"], "unsupported_client_state");
    assert_eq!(
        output["user_message"],
        json!(t(Locale::Zh, keys::VOICE_ERROR_SLEEP_UNSUPPORTED)),
    );
    assert!(harness.frontend.last().await.create_response);
    assert!(
        harness
            .client_states
            .lock()
            .expect("not poisoned")
            .is_empty(),
        "a refused call never reaches requestClientState",
    );
}

#[tokio::test]
async fn enter_sleep_answers_silently_for_a_client_that_can() {
    let harness = harness(healthy(), SessionMode::Agent);
    harness.handler.set_client_context(ClientContext {
        states: vec!["sleeping".to_owned()],
        ..ClientContext::default()
    });
    let output = answer(&harness, &call(catalog::ENTER_SLEEP, json!({}))).await;
    assert_eq!(output, json!({ "status": "sleeping" }));
    assert!(
        !harness.frontend.last().await.create_response,
        "a spoken confirmation would be cut off by the transition it confirms",
    );
}

#[tokio::test]
async fn enter_sleep_echoes_the_state_back_through_request_client_state() {
    let harness = harness(healthy(), SessionMode::Agent);
    harness.handler.set_client_context(ClientContext {
        states: vec!["sleeping".to_owned()],
        ..ClientContext::default()
    });
    answer(&harness, &call(catalog::ENTER_SLEEP, json!({}))).await;
    assert_eq!(
        *harness.client_states.lock().expect("not poisoned"),
        vec!["sleeping".to_owned()],
        "tool-call-handler.mjs:664 — requestClientState('sleeping'), after the answer",
    );
}

// ---------------------------------------------------------------------------
// memory and notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn several_memory_calls_in_one_turn_produce_one_response() {
    let harness = harness(healthy(), SessionMode::Agent);
    for (index, content) in ["- 称呼：老大", "- 语言：中文"].into_iter().enumerate() {
        let mut request = call(
            catalog::MEMORY,
            json!({ "action": "append", "document": "user", "content": content }),
        );
        request.call_id = format!("call_memory_{index}");
        let output = answer(&harness, &request).await;
        assert_eq!(output["status"], "updated", "{content}");
    }
    let sent = harness.frontend.sent.lock().await.clone();
    assert_eq!(sent.len(), 2);
    assert!(
        sent.iter().all(|entry| !entry.create_response),
        "each deferred output stays silent",
    );
    assert_eq!(
        harness.frontend.ensured.load(Ordering::SeqCst),
        0,
        "nothing is spoken until the source response finishes",
    );

    harness
        .handler
        .finish_tool_response("resp_1", false, harness.frontend.as_ref())
        .await;
    assert_eq!(
        harness.frontend.ensured.load(Ordering::SeqCst),
        1,
        "one response covers the whole batch",
    );
}

#[tokio::test]
async fn a_suppressed_source_response_produces_no_extra_response() {
    let harness = harness(healthy(), SessionMode::Agent);
    answer(
        &harness,
        &call(
            catalog::MEMORY,
            json!({ "action": "append", "document": "memory", "content": "住在上海" }),
        ),
    )
    .await;
    harness
        .handler
        .finish_tool_response("resp_1", true, harness.frontend.as_ref())
        .await;
    assert_eq!(harness.frontend.ensured.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_memory_call_on_a_response_with_no_id_is_answered_directly() {
    let harness = harness(healthy(), SessionMode::Agent);
    let mut request = call(
        catalog::MEMORY,
        json!({ "action": "read", "document": "all" }),
    );
    request.response_id.clear();
    answer(&harness, &request).await;
    assert!(
        harness.frontend.last().await.create_response,
        "with nothing to flush against, the output creates its own response",
    );
}

#[tokio::test]
async fn a_read_with_no_document_reads_everything() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(
        &harness,
        &call(catalog::MEMORY, json!({ "action": "read" })),
    )
    .await;
    assert!(output["documents"].is_array());
}

/// `docs/reference/contracts.json`, `json-field` / *memory read/write
/// outputs*: `read: {"status":"ok|not_found","count":N,"documents":[...]}` and
/// `write: {"status":"updated|unchanged","changed":N,"documents":[...]}`, in
/// that key order — exactly the envelope `MemoryToolOutcome::to_value` builds,
/// forwarded unchanged by `ToolCallHandler::memory`.
#[tokio::test]
async fn memory_read_and_write_outputs_match_the_catalogued_envelope() {
    let harness = harness(healthy(), SessionMode::Agent);

    // A read against an owner with nothing written: `not_found`, `count: 0`,
    // an empty `documents` array — and no other keys.
    let empty = answer(
        &harness,
        &call(
            catalog::MEMORY,
            json!({ "action": "read", "document": "all" }),
        ),
    )
    .await;
    assert_eq!(
        empty
            .as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        vec!["status", "count", "documents"],
    );
    assert_eq!(empty["status"], "not_found");
    assert_eq!(empty["count"], 0);
    assert_eq!(empty["documents"], json!([]));

    // A write: `updated`, `changed` carries the number of edits this call made
    // (one), and `documents` is the post-write state.
    let mut write = call(
        catalog::MEMORY,
        json!({ "action": "append", "document": "memory", "content": "住在上海" }),
    );
    write.call_id = "call_memory_write".to_owned();
    let written = answer(&harness, &write).await;
    assert_eq!(
        written
            .as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        vec!["status", "changed", "documents"],
    );
    assert_eq!(written["status"], "updated");
    assert_eq!(written["changed"], 1);
    assert!(
        written["documents"]
            .as_array()
            .is_some_and(|docs| !docs.is_empty())
    );

    // A no-op replace — the old text swapped for itself — changes nothing:
    // `unchanged`, `changed: 0`.
    let mut noop = call(
        catalog::MEMORY,
        json!({
            "action": "replace",
            "document": "memory",
            "old_text": "住在上海",
            "new_text": "住在上海",
        }),
    );
    noop.call_id = "call_memory_noop".to_owned();
    let unchanged = answer(&harness, &noop).await;
    assert_eq!(unchanged["status"], "unchanged");
    assert_eq!(unchanged["changed"], 0);

    // A read now finds the document just written: `ok`, `count: 1`.
    let mut read_back = call(
        catalog::MEMORY,
        json!({ "action": "read", "document": "memory" }),
    );
    read_back.call_id = "call_memory_read_back".to_owned();
    let found = answer(&harness, &read_back).await;
    assert_eq!(found["status"], "ok");
    assert_eq!(found["count"], 1);
    assert!(
        found["documents"][0]["content"]
            .as_str()
            .is_some_and(|content| content.contains("住在上海")),
    );
}

#[tokio::test]
async fn an_absent_new_text_is_a_refusal_and_an_empty_one_is_a_deletion() {
    let harness = harness(healthy(), SessionMode::Agent);
    answer(
        &harness,
        &call(
            catalog::MEMORY,
            json!({ "action": "append", "document": "memory", "content": "- 喜欢苹果" }),
        ),
    )
    .await;

    let mut absent = call(
        catalog::MEMORY,
        json!({ "action": "replace", "document": "memory", "old_text": "- 喜欢苹果" }),
    );
    absent.call_id = "call_absent".to_owned();
    let output = answer(&harness, &absent).await;
    assert_eq!(output["error_code"], "invalid_memory_edit");

    let mut deletion = call(
        catalog::MEMORY,
        json!({
            "action": "replace",
            "document": "memory",
            "old_text": "- 喜欢苹果",
            "new_text": "",
        }),
    );
    deletion.call_id = "call_delete".to_owned();
    let output = answer(&harness, &deletion).await;
    assert_eq!(output["status"], "updated");
}

#[tokio::test]
async fn a_sensitive_memory_write_is_rejected() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(
        &harness,
        &call(
            catalog::MEMORY,
            json!({ "action": "append", "document": "memory", "content": "密码是 hunter2" }),
        ),
    )
    .await;
    assert_eq!(output["status"], "rejected");
    assert_eq!(output["error_code"], "sensitive_memory");
}

#[tokio::test]
async fn notes_round_trips_through_the_shipped_implementation() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(
        &harness,
        &call(
            catalog::NOTES,
            json!({ "action": "add", "list": "购物清单", "items": ["牛奶", "  ", "鸡蛋"] }),
        ),
    )
    .await;
    assert_eq!(output["status"], "ok");

    let mut show = call(
        catalog::NOTES,
        json!({ "action": "show", "list": "购物清单" }),
    );
    show.call_id = "call_show".to_owned();
    let output = answer(&harness, &show).await;
    let items = output["items"].as_array().expect("an array");
    assert_eq!(items.len(), 2, "the blank item was dropped");
}

#[tokio::test]
async fn notes_items_are_capped_at_twenty() {
    let harness = harness(healthy(), SessionMode::Agent);
    let items: Vec<String> = (0..30).map(|index| format!("item {index}")).collect();
    answer(
        &harness,
        &call(
            catalog::NOTES,
            json!({ "action": "add", "list": "长清单", "items": items }),
        ),
    )
    .await;
    let mut show = call(
        catalog::NOTES,
        json!({ "action": "show", "list": "长清单" }),
    );
    show.call_id = "call_show".to_owned();
    let output = answer(&harness, &show).await;
    assert_eq!(output["items"].as_array().map(Vec::len), Some(20));
}

#[tokio::test]
async fn an_unknown_notes_action_is_refused() {
    let harness = harness(healthy(), SessionMode::Agent);
    let output = answer(
        &harness,
        &call(
            catalog::NOTES,
            json!({ "action": "obliterate", "list": "x" }),
        ),
    )
    .await;
    assert_eq!(output["error_code"], "invalid_notes_action");
}
