//! The tool-call handler: dispatch, and every catalogued output shape.
//!
//! Ported from `server/src/voice/tools/tool-call-handler.mjs`.
//!
//! # Receipt-based acceptance
//!
//! Every branch here answers the model **fast**. A tool result is a receipt for
//! intake, not a report of completion: `spawn_thinking` answers `accepted`
//! without waiting for the backend, `respond_agent_permission` answers
//! `submitted` while the ACP round trip is still in flight, and
//! `schedule_reminder` answers `scheduled` before the timer is armed. The
//! reason is that the model is holding a conversational turn open while it
//! waits, and a slow tool is heard as a pause in a sentence.
//!
//! What that costs is bookkeeping: a permission decision that never reaches the
//! backend has to roll the local policy back and re-announce
//! ([`ToolCallHandler::respond_agent_permission`]), and a backend that looked
//! healthy at receipt but fails at dispatch surfaces through the failed-Work
//! announcement path instead.
//!
//! # Ordering
//!
//! [`ToolCallHandler::handle`] is `&self` and internally sequenced by the
//! frontend's own output queue, so two tool calls in one response are answered
//! in arrival order. The three bounded caches — processed calls, per-turn Work,
//! deferred responses — are insertion-ordered and evict oldest-first.

use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};
use serde_json::{Map, Value, json};
use tokio::sync::Mutex;
use via_conversation::{
    MemoryTool, MemoryToolRequest, NotesTool, NotesToolRequest, RawClientContext, ToolFailure,
    current_time_snapshot,
};
use via_i18n::{Locale, format as i18n_format, keys, t};
use via_protocol::{WorkKind, WorkStatus};
use via_work::{NewScheduledWork, NewWork, PublicWork, Schedule, WorkManager, WorkQuery};

use crate::assets::InputAssetRegistry;
use crate::frontend::{FunctionOutputOptions, ResponseRequestContext, VoiceFrontend};
use crate::input::{InputPart, merge_input_parts};
use crate::mode::ModePlan;
use crate::permission::{PermissionDecision, SessionPermissionPolicy};
use crate::text::{bounded_utf16, clean, trim};
use crate::tools::catalog;
use crate::tools::instructions;
use crate::tools::transcripts::TurnTranscripts;

/// How many answered call ids are remembered, for idempotence.
///
/// **External contract** — `tool-call-handler.mjs:347`.
pub const MAX_PROCESSED_CALLS: usize = 500;

/// How many turns keep their submitted Work id, for duplicate suppression.
///
/// **External contract** — `tool-call-handler.mjs:268`.
pub const MAX_TURN_TASKS: usize = 100;

/// How many response ids may have deferred tool output at once.
///
/// **External contract** — `tool-call-handler.mjs:124`.
pub const MAX_DEFERRED_RESPONSES: usize = 100;

/// The clip on an objective echoed back to the model.
///
/// **External contract** — `tool-call-handler.mjs:836,955` (`.slice(0, 300)`),
/// a JavaScript `.length`, so a UTF-16 bound.
pub const OBJECTIVE_CLIP: usize = 300;

/// The clip on an objective quoted inside a delegated status query.
///
/// **External contract** — `tool-call-handler.mjs:889` (`.slice(0, 200)`).
pub const QUERY_OBJECTIVE_CLIP: usize = 200;

/// The clip on a completed Work's result.
///
/// **External contract** — `tool-call-handler.mjs:972` (`.slice(0, 500)`).
pub const RESULT_CLIP: usize = 500;

/// The clip on the last activity's detail.
///
/// **External contract** — `tool-call-handler.mjs:968` (`.slice(0, 160)`).
pub const ACTIVITY_DETAIL_CLIP: usize = 160;

/// How many Work items `list_all` reports.
///
/// **External contract** — `tool-call-handler.mjs:832` (`.slice(0, 20)`).
pub const LIST_ALL_CAP: usize = 20;

/// The scheduler priority a delegated status query is submitted at.
///
/// **External contract** — `tool-call-handler.mjs:888`. High enough to overtake
/// ordinary queued Work in the same lane, because the user is waiting for the
/// answer in conversation.
pub const STATUS_QUERY_PRIORITY: i64 = 100;

/// The marker on an accepted `spawn_thinking`.
///
/// **External contract** — `tool-call-handler.mjs:626`.
pub const THINKING_MARKER: &str = "[thinking]";

/// The five statuses `cancel_agent_task` will target when given no id.
///
/// **External contract** — `tool-call-handler.mjs:781-787`.
pub const CANCELLABLE_STATUSES: [WorkStatus; 5] = [
    WorkStatus::Scheduled,
    WorkStatus::Queued,
    WorkStatus::Running,
    WorkStatus::Delegated,
    WorkStatus::Finalizing,
];

/// A cached view of whether the backend can take work.
///
/// `via-backends` is Layer 3 and Layer 1 may not reach it
/// (`docs/architecture.md` §9), so the voice layer takes the snapshot behind a
/// trait and `via-app` bridges `via_backends::BackendAvailability` to it.
pub trait BackendAvailability: Send + Sync + std::fmt::Debug {
    /// Whether a backend is configured.
    fn configured(&self) -> bool;
    /// Whether it can take work.
    fn ok(&self) -> bool;
    /// Whether this answer rests on a completed probe.
    ///
    /// `false` means *accept optimistically and let dispatch report failures*.
    fn known(&self) -> bool;
}

/// The default when no probe is installed.
///
/// **External contract** — `tool-call-handler.mjs:463-464`:
/// `{ configured: true, ok: true, known: false }`. Optimistic on purpose — a
/// Gateway with no probe must not refuse every delegation.
#[derive(Clone, Copy, Debug, Default)]
pub struct AssumeAvailable;

impl BackendAvailability for AssumeAvailable {
    fn configured(&self) -> bool {
        true
    }
    fn ok(&self) -> bool {
        true
    }
    fn known(&self) -> bool {
        false
    }
}

/// Relays a permission decision to the backend.
#[async_trait::async_trait]
pub trait PermissionResponder: Send + Sync {
    /// Deliver `decision` for `authorization_id`.
    ///
    /// # Errors
    ///
    /// Any message; the handler rolls the local policy back and re-announces.
    async fn respond(
        &self,
        authorization_id: &str,
        decision: PermissionDecision,
        owner_id: &str,
    ) -> Result<(), String>;
}

/// Supplies the runners a delegation and a status query need.
///
/// The bodies live in `via-app`, which can see both `via-coordinator` and the
/// harness; this crate only decides *when* one is created.
pub trait DelegationRunners: Send + Sync + std::fmt::Debug {
    /// The runner for a `spawn_thinking` delegation.
    fn delegation_runner(&self, request: &DelegationRequest) -> Arc<dyn via_work::WorkRunner>;
    /// The canceler for a `spawn_thinking` delegation.
    fn delegation_canceler(&self, request: &DelegationRequest) -> Arc<dyn via_work::WorkCanceler>;
    /// The runner for a delegated-status query.
    fn status_query_runner(&self, request: &StatusQueryRequest) -> Arc<dyn via_work::WorkRunner>;
    /// The canceler for a delegated-status query.
    fn status_query_canceler(
        &self,
        request: &StatusQueryRequest,
    ) -> Arc<dyn via_work::WorkCanceler>;
    /// The runner for a scheduled `type: 'task'`.
    ///
    /// `None` for a `type: 'reminder'`, which speaks its objective back through
    /// [`via_work::reminder_runner`].
    fn scheduled_task_runner(&self) -> Option<Arc<dyn via_work::WorkRunner>>;
}

/// What a delegation was submitted with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelegationRequest {
    /// The owner.
    pub owner_id: String,
    /// The session.
    pub session_id: String,
    /// The turn.
    pub turn_id: String,
    /// The model's objective, cleaned.
    pub objective: String,
    /// The attachments that travel with it.
    pub input_parts: Vec<InputPart>,
}

/// What a delegated-status query was submitted with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusQueryRequest {
    /// The owner.
    pub owner_id: String,
    /// The Work being asked about.
    pub parent_work_id: String,
    /// The user's question, or the default.
    pub question: String,
}

/// The client's environment, as the handler needs it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientContext {
    /// The IANA zone the client reported.
    pub time_zone: String,
    /// The BCP-47 tag the client reported.
    pub locale: String,
    /// The client's launch directory.
    pub working_directory: String,
    /// The states the client declared, e.g. `sleeping`.
    pub states: Vec<String>,
}

impl ClientContext {
    /// Whether the client declared `state`.
    #[must_use]
    pub fn declares(&self, state: &str) -> bool {
        self.states.iter().any(|declared| declared == state)
    }

    fn raw(&self) -> RawClientContext {
        RawClientContext {
            time_zone: self.time_zone.clone(),
            locale: self.locale.clone(),
            working_directory: self.working_directory.clone(),
        }
    }
}

/// One realtime function call, normalized.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolCall {
    /// `call_id` — the id the output must be written back against.
    pub call_id: String,
    /// The tool name.
    pub name: String,
    /// The raw JSON argument string.
    pub arguments: String,
    /// The response this call belongs to.
    pub response_id: String,
    /// The turn the response was correlated to.
    pub turn_id: Option<String>,
    /// That turn's generation.
    pub turn_generation: Option<i64>,
}

/// What the handler did with one call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolCallOutcome {
    /// The output was written.
    Answered {
        /// The output object, exactly as the model sees it.
        output: Value,
    },
    /// The call id had already been answered.
    Duplicate,
    /// The turn moved on; the call was closed as superseded.
    Superseded,
    /// The call carried no `call_id` and could not be answered at all.
    MissingCallId,
}

impl ToolCallOutcome {
    /// The output object, when there was one.
    #[must_use]
    pub const fn output(&self) -> Option<&Value> {
        match self {
            Self::Answered { output } => Some(output),
            _ => None,
        }
    }
}

/// A synchronous callback fired to echo a client-declared state back to it.
///
/// See [`ToolCallHandlerConfig::request_client_state`].
pub type ClientStateNotifier = Arc<dyn Fn(&str) + Send + Sync>;

/// What the handler needs to answer a tool call.
pub struct ToolCallHandlerConfig {
    /// The locale every sentence is rendered in.
    pub locale: Locale,
    /// Whose session this is.
    pub owner_id: String,
    /// Which session.
    pub session_id: String,
    /// What this mode allows.
    pub mode: ModePlan,
    /// The Work manager.
    pub work: Arc<WorkManager>,
    /// The transcripts for this connection.
    pub transcripts: TurnTranscripts,
    /// The input asset registry.
    pub assets: InputAssetRegistry,
    /// The memory tool.
    pub memory: MemoryTool,
    /// The notes tool.
    pub notes: NotesTool,
    /// The backend availability snapshot.
    pub availability: Arc<dyn BackendAvailability>,
    /// The runners a delegation needs.
    pub runners: Arc<dyn DelegationRunners>,
    /// The permission relay, when one is configured.
    pub permission_responder: Option<Arc<dyn PermissionResponder>>,
    /// The per-session permission policy.
    pub policy: Arc<Mutex<SessionPermissionPolicy>>,
    /// Echo a client-declared state back to it. Only the `enter_sleep` tool
    /// calls this, and only once the state is already confirmed declared —
    /// upstream's own guard (`clientContext.states?.includes(state)`) is
    /// therefore redundant here and not reproduced.
    ///
    /// **External contract** — `tool-call-handler.mjs:65,664`
    /// (`requestClientState`). Upstream's own callback additionally enters
    /// sleep when `state === 'sleeping'`; that half is the wake-word
    /// lifecycle's, not this crate's, and is left for the stage that owns it.
    pub request_client_state: Option<ClientStateNotifier>,
}

impl std::fmt::Debug for ToolCallHandlerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolCallHandlerConfig")
            .field("locale", &self.locale)
            .field("owner_id", &self.owner_id)
            .field("session_id", &self.session_id)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
struct HandlerState {
    processed_calls: IndexSet<String>,
    turn_tasks: IndexMap<String, String>,
    deferred: IndexMap<String, DeferredBatch>,
    gateway_approved: IndexSet<String>,
}

#[derive(Clone, Debug, Default)]
struct DeferredBatch {
    pending: usize,
    source_done: bool,
    failed: bool,
    suppress_response: bool,
    turn_id: Option<String>,
    turn_generation: Option<i64>,
}

/// The turn the gateway has committed to, as the handler sees it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommittedTurn {
    /// The committed turn id.
    pub turn_id: String,
    /// Its generation.
    pub turn_generation: i64,
}

/// Answers a realtime function call.
pub struct ToolCallHandler {
    config: ToolCallHandlerConfig,
    state: Mutex<HandlerState>,
    /// The client's environment, replaced whenever a `connect` frame
    /// re-declares it. A `std::sync::Mutex` rather than a `tokio` one because
    /// every reader is synchronous and holds it for one clone.
    client_context: std::sync::Mutex<ClientContext>,
}

impl std::fmt::Debug for ToolCallHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolCallHandler")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ToolCallHandler {
    /// Build a handler.
    #[must_use]
    pub fn new(config: ToolCallHandlerConfig) -> Self {
        Self {
            config,
            state: Mutex::new(HandlerState::default()),
            client_context: std::sync::Mutex::new(ClientContext::default()),
        }
    }

    /// The locale.
    #[must_use]
    pub const fn locale(&self) -> Locale {
        self.config.locale
    }

    /// Whether this call is for a turn the gateway has moved past.
    ///
    /// **External contract** — `tool-call-handler.mjs:93-98`. A generation
    /// mismatch always means stale. A turn-id mismatch means stale only when
    /// **both** ids are non-empty: a call that arrived before the gateway
    /// committed a turn has nothing to be stale against.
    #[must_use]
    pub fn is_stale(&self, call: &ToolCall, committed: &CommittedTurn) -> bool {
        let generation = call.turn_generation.unwrap_or(committed.turn_generation);
        if generation != committed.turn_generation {
            return true;
        }
        let turn_id = call.turn_id.clone().unwrap_or_default();
        !turn_id.is_empty() && !committed.turn_id.is_empty() && turn_id != committed.turn_id
    }

    /// Answer one tool call.
    ///
    /// The whole dispatch, in upstream's order — the ordering is contract,
    /// because the stale check runs **before** any tool body and the
    /// duplicate-call guard runs before that.
    pub async fn handle(
        &self,
        call: &ToolCall,
        committed: &CommittedTurn,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        if call.call_id.is_empty() {
            return ToolCallOutcome::MissingCallId;
        }
        {
            let mut state = self.state.lock().await;
            if !state.processed_calls.insert(call.call_id.clone()) {
                return ToolCallOutcome::Duplicate;
            }
            while state.processed_calls.len() > MAX_PROCESSED_CALLS {
                state.processed_calls.shift_remove_index(0);
            }
        }

        let turn_id = call
            .turn_id
            .clone()
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| committed.turn_id.clone());
        let args = parse_arguments(&call.arguments);

        if self.is_stale(call, committed) {
            self.close_stale_call(&call.call_id, &turn_id, frontend)
                .await;
            return ToolCallOutcome::Superseded;
        }

        if !self.config.mode.declares_tool(&call.name) {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "unsupported_tool",
                        t(self.locale(), keys::VOICE_ERROR_OPERATION_UNAVAILABLE),
                    )
                    .to_value(),
                    &turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        }

        match call.name.as_str() {
            catalog::GET_CURRENT_TIME => self.get_current_time(call, &turn_id, frontend).await,
            catalog::MEMORY => self.memory(call, &turn_id, &args, frontend).await,
            catalog::NOTES => self.notes(call, &turn_id, &args, frontend).await,
            catalog::SCHEDULE_REMINDER => {
                self.schedule_reminder(call, &turn_id, &args, frontend)
                    .await
            }
            catalog::CANCEL_AGENT_TASK => {
                self.cancel_agent_task(call, &turn_id, &args, frontend)
                    .await
            }
            catalog::GET_AGENT_TASK_STATUS => {
                self.get_agent_task_status(call, &turn_id, &args, frontend)
                    .await
            }
            catalog::RESPOND_AGENT_PERMISSION => {
                self.respond_agent_permission(call, &turn_id, &args, frontend)
                    .await
            }
            catalog::ENTER_SLEEP => self.enter_sleep(call, &turn_id, frontend).await,
            catalog::SPAWN_THINKING => {
                self.spawn_thinking(call, &turn_id, &args, committed, frontend)
                    .await
            }
            _ => {
                self.answer(
                    &call.call_id,
                    ToolFailure::new(
                        "unsupported_tool",
                        t(self.locale(), keys::VOICE_ERROR_OPERATION_UNAVAILABLE),
                    )
                    .to_value(),
                    &turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await
            }
        }
    }

    /// Write a stale call's output without letting the model answer it.
    ///
    /// **External contract** — `tool-call-handler.mjs:159-170`. `createResponse`
    /// is false: the model must see that the call was closed, but nothing
    /// should be said about a turn the user has already moved past.
    async fn close_stale_call(&self, call_id: &str, turn_id: &str, frontend: &dyn VoiceFrontend) {
        let output = json!({
            "status": "superseded",
            "message": t(self.locale(), keys::VOICE_RESULT_SUPERSEDED),
        });
        self.answer(
            call_id,
            output,
            turn_id,
            None,
            FunctionOutputOptions::silent(),
            frontend,
        )
        .await;
    }

    async fn answer(
        &self,
        call_id: &str,
        output: Value,
        turn_id: &str,
        task_id: Option<String>,
        options: FunctionOutputOptions,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let context = ResponseRequestContext {
            turn_id: (!turn_id.is_empty()).then(|| turn_id.to_owned()),
            task_id,
            ..ResponseRequestContext::default()
        };
        frontend
            .send_function_output(call_id, &output, context, options)
            .await;
        ToolCallOutcome::Answered { output }
    }

    async fn answer_with_context(
        &self,
        call_id: &str,
        output: Value,
        context: ResponseRequestContext,
        options: FunctionOutputOptions,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        frontend
            .send_function_output(call_id, &output, context, options)
            .await;
        ToolCallOutcome::Answered { output }
    }

    // ---- get_current_time --------------------------------------------------

    async fn get_current_time(
        &self,
        call: &ToolCall,
        turn_id: &str,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let snapshot = current_time_snapshot(&self.client_context().raw(), chrono::Utc::now());
        // `{status: 'ok', ...snapshot}` — the status first, then the four
        // snapshot fields in their own order (`tool-call-handler.mjs:983-986`).
        let mut output = Map::new();
        output.insert("status".to_owned(), json!("ok"));
        output.insert("iso_utc".to_owned(), json!(snapshot.iso_utc));
        output.insert("local_time".to_owned(), json!(snapshot.local_time));
        output.insert("time_zone".to_owned(), json!(snapshot.time_zone));
        output.insert("locale".to_owned(), json!(snapshot.locale));
        self.answer(
            &call.call_id,
            Value::Object(output),
            turn_id,
            None,
            FunctionOutputOptions::with_response(),
            frontend,
        )
        .await
    }

    /// The client's environment as last declared.
    #[must_use]
    pub fn client_context(&self) -> ClientContext {
        match self.client_context.lock() {
            Ok(context) => context.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    // ---- memory ------------------------------------------------------------

    async fn memory(
        &self,
        call: &ToolCall,
        turn_id: &str,
        args: &Map<String, Value>,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        // Several `memory` calls can arrive inside one response — upstream's
        // "same utterance, several persistent changes, one call each" rule. The
        // deferred batch is what merges them: each call writes its output with
        // `createResponse: false`, and the response is created once, after the
        // last one, by `finish_tool_response`.
        let deferred = self
            .begin_deferred_tool_response(&call.response_id, turn_id, call.turn_generation)
            .await;
        let request = memory_request(args);
        let outcome = self.config.memory.handle(&self.config.owner_id, &request);
        let options = if deferred {
            FunctionOutputOptions::silent()
        } else {
            FunctionOutputOptions::with_response()
        };
        let answered = self
            .answer(
                &call.call_id,
                outcome.to_value(),
                turn_id,
                None,
                options,
                frontend,
            )
            .await;
        self.complete_deferred_tool_response(&call.response_id, false, frontend)
            .await;
        answered
    }

    // ---- notes -------------------------------------------------------------

    async fn notes(
        &self,
        call: &ToolCall,
        turn_id: &str,
        args: &Map<String, Value>,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let request = notes_request(args);
        let outcome = self.config.notes.handle(&self.config.owner_id, &request);
        self.answer(
            &call.call_id,
            outcome.to_value(),
            turn_id,
            None,
            FunctionOutputOptions::with_response(),
            frontend,
        )
        .await
    }

    // ---- schedule_reminder -------------------------------------------------

    async fn schedule_reminder(
        &self,
        call: &ToolCall,
        turn_id: &str,
        args: &Map<String, Value>,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let raw_execute_at = string_arg(args, "execute_at");
        let execute_at = parse_execute_at(&raw_execute_at);
        let now = chrono::Utc::now().timestamp_millis();
        // `!executeAt || executeAt <= Date.now()` — an unparseable time, the
        // epoch itself, and any past instant are all `invalid_time`.
        let Some(execute_at) = execute_at.filter(|at| *at > now && *at != 0) else {
            let output = json!({
                "status": "error",
                "error": true,
                "error_code": "invalid_time",
                "user_message": t(self.locale(), keys::VOICE_ERROR_INVALID_TIME),
            });
            return self
                .answer(
                    &call.call_id,
                    output,
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        let kind = string_arg(args, "type");
        let is_task = kind == "task";
        let kind = if is_task { "task" } else { "reminder" };
        let recurrence = {
            let supplied = string_arg(args, "recurrence");
            if supplied.is_empty() {
                "once".to_owned()
            } else {
                supplied
            }
        };
        let objective = string_arg(args, "reminder");

        let schedule = Schedule {
            kind: "at".to_owned(),
            at: execute_at,
            recurrence: recurrence.clone(),
        };
        let mut request = if is_task {
            NewScheduledWork::task(&objective, &self.config.owner_id, schedule)
        } else {
            NewScheduledWork::reminder(&objective, &self.config.owner_id, schedule)
        }
        .session(&self.config.session_id)
        .turn(turn_id);
        if is_task && let Some(runner) = self.config.runners.scheduled_task_runner() {
            request = request.runner(runner);
        }

        let Ok(acceptance) = self.config.work.create_scheduled(request).await else {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "work_submission_failed",
                        t(self.locale(), keys::VOICE_ERROR_SUBMIT_FAILED),
                    )
                    .retryable()
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        let output = json!({
            "status": "scheduled",
            "reminder_id": acceptance.work.id,
            "execute_at": raw_execute_at,
            "type": kind,
            "recurrence": recurrence,
        });
        self.answer(
            &call.call_id,
            output,
            turn_id,
            Some(acceptance.work.id.clone()),
            FunctionOutputOptions::with_response().instructions(
                instructions::reminder_confirmation_instructions(self.locale()),
            ),
            frontend,
        )
        .await
    }

    // ---- cancel_agent_task -------------------------------------------------

    async fn cancel_agent_task(
        &self,
        call: &ToolCall,
        turn_id: &str,
        args: &Map<String, Value>,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let requested = trim(&string_arg(args, "work_id")).to_owned();
        let target = if requested.is_empty() {
            self.config
                .work
                .list(WorkQuery::owner(&self.config.owner_id).session(&self.config.session_id))
                .await
                .into_iter()
                .find(|work| CANCELLABLE_STATUSES.contains(&work.status))
                .map(|work| work.id)
        } else {
            Some(requested)
        };
        let Some(target) = target else {
            let output = json!({
                "status": "not_found",
                "message": t(self.locale(), keys::VOICE_RESULT_CANCEL_NOT_FOUND),
            });
            return self
                .answer(
                    &call.call_id,
                    output,
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        // A status query about the Work being cancelled is cancelled with it:
        // otherwise the query answers about work that no longer exists.
        let related: Vec<String> = self
            .config
            .work
            .list(
                WorkQuery::owner(&self.config.owner_id)
                    .active()
                    .with_control(),
            )
            .await
            .into_iter()
            .filter(|work| {
                work.kind == WorkKind::Control && work.parent_work_id.as_deref() == Some(&target)
            })
            .map(|work| work.id)
            .collect();
        for control in related {
            self.config
                .work
                .cancel(&control, Some(&self.config.owner_id))
                .await;
        }

        let Some(work) = self
            .config
            .work
            .cancel(&target, Some(&self.config.owner_id))
            .await
        else {
            let output = json!({
                "status": "not_active",
                "work_id": target,
                "message": t(self.locale(), keys::VOICE_RESULT_CANCEL_NOT_ACTIVE),
            });
            return self
                .answer(
                    &call.call_id,
                    output,
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        let output = if work.status == WorkStatus::Cancelled {
            json!({
                "status": work.status.as_str(),
                "work_id": work.id,
                "message": t(self.locale(), keys::VOICE_RESULT_CANCELLED),
            })
        } else {
            ToolFailure::new(
                "work_cancellation_failed",
                work.error.clone().unwrap_or_else(|| {
                    t(self.locale(), keys::VOICE_RESULT_CANCEL_FAILED).to_owned()
                }),
            )
            .to_value()
        };
        self.answer(
            &call.call_id,
            output,
            turn_id,
            Some(work.id.clone()),
            FunctionOutputOptions::with_response(),
            frontend,
        )
        .await
    }

    // ---- get_agent_task_status ---------------------------------------------

    async fn get_agent_task_status(
        &self,
        call: &ToolCall,
        turn_id: &str,
        args: &Map<String, Value>,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        if args.get("list_all") == Some(&Value::Bool(true)) {
            return self.list_all(call, turn_id, frontend).await;
        }
        let requested = trim(&string_arg(args, "work_id")).to_owned();
        let work = if requested.is_empty() {
            self.config
                .work
                .list(WorkQuery::owner(&self.config.owner_id).session(&self.config.session_id))
                .await
                .into_iter()
                .next()
        } else {
            self.config
                .work
                .get(&requested, Some(&self.config.owner_id))
                .await
        };
        let Some(work) = work else {
            let output = json!({
                "status": "not_found",
                "message": t(self.locale(), keys::VOICE_RESULT_NO_QUERYABLE_WORK),
            });
            return self
                .answer(
                    &call.call_id,
                    output,
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        if work.status == WorkStatus::Delegated {
            return self
                .query_delegated(call, turn_id, args, &work, frontend)
                .await;
        }
        self.answer_single_status(call, turn_id, &work, frontend)
            .await
    }

    async fn list_all(
        &self,
        call: &ToolCall,
        turn_id: &str,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let works = self
            .config
            .work
            .list(WorkQuery::owner(&self.config.owner_id).session(&self.config.session_id))
            .await;
        let tasks: Vec<Value> = works
            .iter()
            .take(LIST_ALL_CAP)
            .map(|work| {
                json!({
                    "work_id": work.id,
                    "status": work.status.as_str(),
                    "kind": work.kind.as_str(),
                    "objective": bounded_utf16(&work.objective, OBJECTIVE_CLIP),
                    "execute_at": work
                        .schedule
                        .as_ref()
                        .and_then(|schedule| iso_instant(schedule.at))
                        .map_or(Value::Null, Value::String),
                    "recurrence": work
                        .schedule
                        .as_ref()
                        .map_or(Value::Null, |schedule| json!(schedule.recurrence)),
                })
            })
            .collect();
        let output = json!({
            "status": if tasks.is_empty() { "empty" } else { "ok" },
            "count": tasks.len(),
            "tasks": tasks,
        });
        self.answer(
            &call.call_id,
            output,
            turn_id,
            None,
            FunctionOutputOptions::with_response(),
            frontend,
        )
        .await
    }

    async fn query_delegated(
        &self,
        call: &ToolCall,
        turn_id: &str,
        args: &Map<String, Value>,
        work: &PublicWork,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let existing = self
            .config
            .work
            .list(
                WorkQuery::owner(&self.config.owner_id)
                    .session(&self.config.session_id)
                    .active()
                    .with_control(),
            )
            .await
            .into_iter()
            .find(|item| {
                item.kind == WorkKind::Control
                    && item.parent_work_id.as_deref() == Some(work.id.as_str())
            });
        if let Some(existing) = existing {
            let output = json!({
                "status": "querying",
                "work_id": work.id,
                "query_work_id": existing.id,
                "message": t(self.locale(), keys::VOICE_RESULT_QUERY_ALREADY_IN_FLIGHT),
            });
            return self
                .answer(
                    &call.call_id,
                    output,
                    turn_id,
                    Some(work.id.clone()),
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        }

        let supplied = trim(&string_arg(args, "question")).to_owned();
        let question = if supplied.is_empty() {
            self.config.transcripts.transcript(turn_id).await
        } else {
            supplied
        };
        let question = trim(&question).to_owned();
        let objective = i18n_format(
            self.locale(),
            keys::VOICE_RESULT_QUERY_PROMPT,
            &[
                (
                    "objective",
                    &bounded_utf16(&work.objective, QUERY_OBJECTIVE_CLIP),
                ),
                (
                    "question",
                    if question.is_empty() {
                        t(self.locale(), keys::VOICE_RESULT_QUERY_PROMPT_DEFAULT)
                    } else {
                        &question
                    },
                ),
            ],
        );
        let request = StatusQueryRequest {
            owner_id: self.config.owner_id.clone(),
            parent_work_id: work.id.clone(),
            question: if question.is_empty() {
                t(
                    self.locale(),
                    keys::VOICE_INSTRUCTIONS_DELEGATED_STATUS_QUERY,
                )
                .to_owned()
            } else {
                question
            },
        };
        let new_work = NewWork::new(&objective, &self.config.owner_id)
            .session(&self.config.session_id)
            .turn(turn_id)
            .kind(WorkKind::Control)
            .parent(&work.id)
            .priority(STATUS_QUERY_PRIORITY)
            .lane(
                &via_coordinator::coordinator_session_lane(&self.config.owner_id),
                1,
            )
            .runner(self.config.runners.status_query_runner(&request))
            .canceler(self.config.runners.status_query_canceler(&request));
        let Ok(acceptance) = self.config.work.create(new_work).await else {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "work_submission_failed",
                        t(self.locale(), keys::VOICE_ERROR_SUBMIT_FAILED),
                    )
                    .retryable()
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        let output = json!({
            "status": "querying",
            "work_id": work.id,
            "query_work_id": acceptance.work.id,
            "message": t(self.locale(), keys::VOICE_RESULT_QUERYING),
        });
        self.answer(
            &call.call_id,
            output,
            turn_id,
            Some(work.id.clone()),
            FunctionOutputOptions::with_response(),
            frontend,
        )
        .await
    }

    async fn answer_single_status(
        &self,
        call: &ToolCall,
        turn_id: &str,
        work: &PublicWork,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let last_activity = work.activity.last();
        // Answering a status query about a Work whose result is still queued
        // for announcement *is* the delivery: the user just heard it. The flag
        // travels on the response context so the announcement manager can
        // confirm the notification when playback starts.
        let consumes = matches!(work.status, WorkStatus::Completed | WorkStatus::Failed)
            && matches!(
                work.notification_status,
                via_work::NotificationStatus::Pending | via_work::NotificationStatus::Delivering
            );
        let output = json!({
            "status": "ok",
            "work_id": work.id,
            "work_status": work.status.as_str(),
            "objective": bounded_utf16(&work.objective, OBJECTIVE_CLIP),
            "elapsed_ms": work.elapsed_ms,
            "delegation": work.delegation.as_ref().map_or(Value::Null, |delegation| {
                json!({ "status": delegation.status, "title": delegation.title })
            }),
            "authorization_pending": work
                .authorization
                .as_ref()
                .is_some_and(|permission| permission.status == via_work::PermissionStatus::Pending),
            "last_activity": last_activity.map_or(Value::Null, |activity| {
                json!({
                    "category": activity.category.clone().unwrap_or_else(|| activity.kind.clone()),
                    "status": activity.status,
                    "detail": bounded_utf16(
                        activity.detail.as_deref().unwrap_or_default(),
                        ACTIVITY_DETAIL_CLIP,
                    ),
                })
            }),
            "result": if work.status == WorkStatus::Completed {
                json!(bounded_utf16(work.result.as_deref().unwrap_or_default(), RESULT_CLIP))
            } else {
                Value::Null
            },
            "error": if matches!(work.status, WorkStatus::Failed | WorkStatus::Cancelled) {
                work.error.clone().map_or(Value::Null, Value::String)
            } else {
                Value::Null
            },
        });
        let context = ResponseRequestContext {
            turn_id: (!turn_id.is_empty()).then(|| turn_id.to_owned()),
            task_id: Some(work.id.clone()),
            consumes_task_notification: consumes,
            ..ResponseRequestContext::default()
        };
        self.answer_with_context(
            &call.call_id,
            output,
            context,
            FunctionOutputOptions::with_response(),
            frontend,
        )
        .await
    }

    // ---- respond_agent_permission -----------------------------------------

    async fn respond_agent_permission(
        &self,
        call: &ToolCall,
        turn_id: &str,
        args: &Map<String, Value>,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        let authorization_id = trim(&string_arg(args, "authorization_id")).to_owned();
        let decision = PermissionDecision::from_wire(trim(&string_arg(args, "decision")));
        let transcript = self.config.transcripts.transcript(turn_id).await;
        let transcript = trim(&transcript);
        // The transcript requirement is the anti-hallucination gate: the model
        // may not approve a permission on a turn where the user said nothing.
        let Some(decision) =
            decision.filter(|_| !authorization_id.is_empty() && !transcript.is_empty())
        else {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "invalid_permission_response",
                        t(self.locale(), keys::VOICE_ERROR_PERMISSION_NOT_FOUND),
                    )
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        let pending = self
            .config
            .work
            .list(
                WorkQuery::owner(&self.config.owner_id)
                    .session(&self.config.session_id)
                    .active(),
            )
            .await
            .into_iter()
            .find(|work| {
                work.authorization
                    .as_ref()
                    .is_some_and(|permission| permission.id == authorization_id)
            });
        let Some(pending) = pending else {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "permission_not_pending",
                        t(
                            self.locale(),
                            keys::VOICE_INSTRUCTIONS_PERMISSION_ALREADY_HANDLED,
                        ),
                    )
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        let Some(responder) = self.config.permission_responder.clone() else {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "permission_unavailable",
                        t(
                            self.locale(),
                            keys::VOICE_ERROR_PERMISSION_BACKEND_UNAVAILABLE,
                        ),
                    )
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };

        // Receipt-based: the local policy takes effect at once so the spoken
        // confirmation is not held behind an ACP round trip. On delivery
        // failure the policy rolls back and the authorization is still pending
        // on the backend, so the standard re-announce path asks again.
        let previous = {
            let mut policy = self.config.policy.lock().await;
            let previous = policy.mode(&self.config.owner_id, &self.config.session_id);
            policy.apply_decision(&self.config.owner_id, &self.config.session_id, decision);
            previous
        };
        let policy = Arc::clone(&self.config.policy);
        let owner_id = self.config.owner_id.clone();
        let session_id = self.config.session_id.clone();
        let authorization = authorization_id.clone();
        tokio::spawn(async move {
            if let Err(error) = responder.respond(&authorization, decision, &owner_id).await {
                policy
                    .lock()
                    .await
                    .set_mode(&owner_id, &session_id, previous);
                tracing::warn!(
                    target: "via::voice",
                    authorization_id = %authorization,
                    error = %error,
                    "permission.delivery_failed",
                );
            }
        });

        let output = json!({
            "status": "submitted",
            "authorization_id": authorization_id,
        });
        let instructions = match decision {
            PermissionDecision::Always => {
                instructions::permission_submitted_always_instructions(self.locale())
            }
            PermissionDecision::Reject => {
                instructions::permission_submitted_reject_instructions(self.locale())
            }
        };
        self.answer(
            &call.call_id,
            output,
            turn_id,
            Some(pending.id.clone()),
            FunctionOutputOptions::with_response().instructions(instructions),
            frontend,
        )
        .await
    }

    // ---- enter_sleep -------------------------------------------------------

    async fn enter_sleep(
        &self,
        call: &ToolCall,
        turn_id: &str,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        if !self
            .client_context()
            .declares(catalog::SLEEPING_CLIENT_STATE)
        {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "unsupported_client_state",
                        t(self.locale(), keys::VOICE_ERROR_SLEEP_UNSUPPORTED),
                    )
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        }
        // `createResponse: false` — the entry point is about to go quiet, and a
        // spoken confirmation would be cut off by the transition it confirms.
        let outcome = self
            .answer(
                &call.call_id,
                json!({ "status": "sleeping" }),
                turn_id,
                None,
                FunctionOutputOptions::silent(),
                frontend,
            )
            .await;
        // `tool-call-handler.mjs:664` — fired after the answer, not before.
        if let Some(notify) = &self.config.request_client_state {
            notify(catalog::SLEEPING_CLIENT_STATE);
        }
        outcome
    }

    // ---- spawn_thinking ----------------------------------------------------

    async fn spawn_thinking(
        &self,
        call: &ToolCall,
        turn_id: &str,
        args: &Map<String, Value>,
        committed: &CommittedTurn,
        frontend: &dyn VoiceFrontend,
    ) -> ToolCallOutcome {
        if let Some(pending) = self.pending_permission_work().await {
            let permission = pending
                .authorization
                .as_ref()
                .map(|permission| (permission.id.clone(), permission.summary.clone()))
                .unwrap_or_default();
            let output = json!({
                "status": "authorization_pending",
                "error": true,
                "error_code": "permission_decision_required",
                "authorization_id": permission.0,
                "operation": permission.1,
                "user_message": t(self.locale(), keys::VOICE_ERROR_PERMISSION_DECISION_REQUIRED),
                "retryable": true,
            });
            return self
                .answer(
                    &call.call_id,
                    output,
                    turn_id,
                    Some(pending.id.clone()),
                    FunctionOutputOptions::with_response()
                        .instructions(instructions::permission_pending_instructions(self.locale())),
                    frontend,
                )
                .await;
        }

        if !self.config.availability.configured() {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "backend_unavailable",
                        t(self.locale(), keys::VOICE_ERROR_BACKEND_NOT_CONFIGURED),
                    )
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response().instructions(
                        instructions::backend_not_configured_instructions(self.locale()),
                    ),
                    frontend,
                )
                .await;
        }
        if self.config.availability.known() && !self.config.availability.ok() {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "backend_unavailable",
                        t(self.locale(), keys::VOICE_ERROR_BACKEND_DISCONNECTED),
                    )
                    .retryable()
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response().instructions(
                        instructions::backend_disconnected_instructions(self.locale()),
                    ),
                    frontend,
                )
                .await;
        }

        let mut objective = clean(&string_arg(args, "objective"));
        if objective.is_empty() {
            // The rare model slip. Only this path waits for the transcript.
            let resolved = self
                .config
                .transcripts
                .resolve_delegation(turn_id, "")
                .await;
            if self.is_stale(call, committed) {
                self.close_stale_call(&call.call_id, turn_id, frontend)
                    .await;
                return ToolCallOutcome::Superseded;
            }
            objective = trim(&resolved.original_request).to_owned();
        }
        if objective.is_empty() {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "missing_objective",
                        t(self.locale(), keys::VOICE_ERROR_OBJECTIVE_INCOMPLETE),
                    )
                    .retryable()
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        }

        if let Some(existing) = self.turn_task(turn_id).await {
            let output = json!({
                "status": "duplicate",
                "work_id": existing,
                "message": t(self.locale(), keys::VOICE_RESULT_DUPLICATE),
            });
            return self
                .answer(
                    &call.call_id,
                    output,
                    turn_id,
                    Some(existing),
                    FunctionOutputOptions::with_response().instructions(
                        instructions::duplicate_submission_instructions(self.locale()),
                    ),
                    frontend,
                )
                .await;
        }

        let references: Vec<String> = args
            .get("input_refs")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let historical = match self.config.assets.resolve(
            &self.config.owner_id,
            &self.config.session_id,
            &references,
        ) {
            Ok(parts) => parts,
            Err(_) => {
                return self
                    .answer(
                        &call.call_id,
                        ToolFailure::new(
                            "invalid_input_ref",
                            t(self.locale(), keys::VOICE_ERROR_INPUT_REF_EXPIRED),
                        )
                        .retryable()
                        .to_value(),
                        turn_id,
                        None,
                        FunctionOutputOptions::with_response(),
                        frontend,
                    )
                    .await;
            }
        };
        let turn_parts = self.config.transcripts.parts(turn_id).await;
        let input_parts = merge_input_parts(&[&turn_parts, &historical]);

        let request = DelegationRequest {
            owner_id: self.config.owner_id.clone(),
            session_id: self.config.session_id.clone(),
            turn_id: turn_id.to_owned(),
            objective: objective.clone(),
            input_parts,
        };
        // The duplicate-submission key is per session and per turn: the same
        // turn cannot submit twice even if the model calls the tool twice.
        let submission_key = format!(
            "delegation:{}:{}",
            self.config.session_id,
            if turn_id.is_empty() {
                &call.call_id
            } else {
                turn_id
            },
        );
        let new_work = NewWork::new(&objective, &self.config.owner_id)
            .session(&self.config.session_id)
            .turn(turn_id)
            .submission_key(&submission_key)
            .lane(
                &via_coordinator::coordinator_session_lane(&self.config.owner_id),
                1,
            )
            .runner(self.config.runners.delegation_runner(&request))
            .canceler(self.config.runners.delegation_canceler(&request));
        let Ok(acceptance) = self.config.work.create(new_work).await else {
            return self
                .answer(
                    &call.call_id,
                    ToolFailure::new(
                        "work_submission_failed",
                        t(self.locale(), keys::VOICE_ERROR_SUBMIT_FAILED),
                    )
                    .retryable()
                    .to_value(),
                    turn_id,
                    None,
                    FunctionOutputOptions::with_response(),
                    frontend,
                )
                .await;
        };
        self.remember_turn_task(turn_id, &acceptance.work.id).await;

        let output = if acceptance.reused {
            json!({
                "status": "duplicate",
                "work_id": acceptance.work.id,
                "message": t(self.locale(), keys::VOICE_RESULT_DUPLICATE),
            })
        } else {
            json!({
                "status": "accepted",
                "marker": THINKING_MARKER,
                "work_id": acceptance.work.id,
            })
        };
        self.answer(
            &call.call_id,
            output,
            turn_id,
            Some(acceptance.work.id.clone()),
            FunctionOutputOptions::with_response()
                .instructions(instructions::accepted_instructions(self.locale())),
            frontend,
        )
        .await
    }

    async fn pending_permission_work(&self) -> Option<PublicWork> {
        self.config
            .work
            .list(
                WorkQuery::owner(&self.config.owner_id)
                    .session(&self.config.session_id)
                    .active(),
            )
            .await
            .into_iter()
            .find(|work| {
                work.authorization.as_ref().is_some_and(|permission| {
                    permission.status == via_work::PermissionStatus::Pending
                })
            })
    }

    async fn turn_task(&self, turn_id: &str) -> Option<String> {
        self.state.lock().await.turn_tasks.get(turn_id).cloned()
    }

    async fn remember_turn_task(&self, turn_id: &str, work_id: &str) {
        let mut state = self.state.lock().await;
        state
            .turn_tasks
            .insert(turn_id.to_owned(), work_id.to_owned());
        while state.turn_tasks.len() > MAX_TURN_TASKS {
            state.turn_tasks.shift_remove_index(0);
        }
    }

    // ---- the deferred response batch --------------------------------------

    /// Register one deferred tool output for `response_id`.
    ///
    /// **External contract** — `tool-call-handler.mjs:113-130`. Answers whether
    /// deferral is in force; an empty response id cannot be deferred, because
    /// there is nothing to flush against.
    pub async fn begin_deferred_tool_response(
        &self,
        response_id: &str,
        turn_id: &str,
        turn_generation: Option<i64>,
    ) -> bool {
        if response_id.is_empty() {
            return false;
        }
        let mut state = self.state.lock().await;
        if !state.deferred.contains_key(response_id)
            && state.deferred.len() >= MAX_DEFERRED_RESPONSES
        {
            state.deferred.shift_remove_index(0);
        }
        let batch = state
            .deferred
            .entry(response_id.to_owned())
            .or_insert_with(|| DeferredBatch {
                turn_id: (!turn_id.is_empty()).then(|| turn_id.to_owned()),
                turn_generation,
                ..DeferredBatch::default()
            });
        batch.pending += 1;
        true
    }

    /// One deferred output finished.
    pub async fn complete_deferred_tool_response(
        &self,
        response_id: &str,
        failed: bool,
        frontend: &dyn VoiceFrontend,
    ) {
        let flush = {
            let mut state = self.state.lock().await;
            let Some(batch) = state.deferred.get_mut(response_id) else {
                return;
            };
            batch.pending = batch.pending.saturating_sub(1);
            batch.failed |= failed;
            Self::take_flushable(&mut state, response_id)
        };
        Self::flush(flush, frontend).await;
    }

    /// The source response finished; flush when every deferred output has.
    ///
    /// **External contract** — `tool-call-handler.mjs:140-157`. `suppress`
    /// carries the four reasons the gateway already knows the model has
    /// nothing more to say: the response failed, it was suppressed, it emitted
    /// audio, or it produced a transcript.
    pub async fn finish_tool_response(
        &self,
        response_id: &str,
        suppress: bool,
        frontend: &dyn VoiceFrontend,
    ) {
        let flush = {
            let mut state = self.state.lock().await;
            let Some(batch) = state.deferred.get_mut(response_id) else {
                return;
            };
            batch.source_done = true;
            batch.suppress_response |= suppress;
            Self::take_flushable(&mut state, response_id)
        };
        Self::flush(flush, frontend).await;
    }

    fn take_flushable(state: &mut HandlerState, response_id: &str) -> Option<DeferredBatch> {
        let batch = state.deferred.get(response_id)?;
        if !batch.source_done || batch.pending > 0 {
            return None;
        }
        state.deferred.shift_remove(response_id)
    }

    async fn flush(batch: Option<DeferredBatch>, frontend: &dyn VoiceFrontend) {
        let Some(batch) = batch else { return };
        if batch.failed || batch.suppress_response {
            return;
        }
        frontend
            .ensure_response(
                ResponseRequestContext {
                    turn_id: batch.turn_id,
                    turn_generation: batch.turn_generation,
                    ..ResponseRequestContext::default()
                },
                None,
            )
            .await;
    }

    /// Whether the Gateway itself auto-approved `permission_id`.
    ///
    /// **External contract** — `tool-call-handler.mjs:172-211`. The set is what
    /// stops the Gateway's own approval being re-announced to the user as if a
    /// human had to decide it.
    pub async fn mark_gateway_approved(&self, permission_id: &str) {
        self.state
            .lock()
            .await
            .gateway_approved
            .insert(permission_id.to_owned());
    }

    /// Take the gateway-approval mark, if there is one.
    pub async fn take_gateway_approval(&self, permission_id: &str) -> bool {
        self.state
            .lock()
            .await
            .gateway_approved
            .shift_remove(permission_id)
    }

    /// Whether this session auto-approves backend permissions.
    pub async fn should_auto_allow(&self) -> bool {
        self.config
            .policy
            .lock()
            .await
            .should_auto_allow(&self.config.owner_id, &self.config.session_id)
    }

    /// Replace the client context this handler reads.
    pub fn set_client_context(&self, context: ClientContext) {
        match self.client_context.lock() {
            Ok(mut slot) => *slot = context,
            Err(poisoned) => *poisoned.into_inner() = context,
        }
    }
}

fn parse_arguments(arguments: &str) -> Map<String, Value> {
    // Invalid arguments are handled as missing fields, exactly as upstream's
    // `try { JSON.parse } catch {}` does. A model that emits truncated JSON
    // gets the same answer as one that omitted the field.
    serde_json::from_str::<Value>(arguments)
        .ok()
        .and_then(|value| match value {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default()
}

fn string_arg(args: &Map<String, Value>, key: &str) -> String {
    args.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// `Date.parse(value)`, restricted to what a model can produce.
///
/// **External contract** — `tool-call-handler.mjs:275`. `schedule_reminder`'s
/// schema says ISO 8601, and RFC 3339 is the interoperable subset of it; a
/// value without an offset is read as local-naive in UTC, which is what
/// `Date.parse` does for `YYYY-MM-DDTHH:MM:SS` in a UTC host.
fn parse_execute_at(value: &str) -> Option<i64> {
    let value = trim(value);
    if value.is_empty() {
        return None;
    }
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|instant| instant.timestamp_millis())
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
                .map(|naive| naive.and_utc().timestamp_millis())
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
                .map(|naive| naive.and_utc().timestamp_millis())
        })
        .ok()
}

fn iso_instant(epoch_ms: i64) -> Option<String> {
    use chrono::TimeZone;
    chrono::Utc
        .timestamp_millis_opt(epoch_ms)
        .single()
        .map(|instant| instant.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

/// Build a [`MemoryToolRequest`] from raw arguments.
///
/// **External contract** — `tool-call-handler.mjs:990-996`. `document`
/// defaults to `all` for a read and to `''` for a write, which is what makes
/// "read with no document" mean *both* while "append with no document" is a
/// refusal rather than a guess.
fn memory_request(args: &Map<String, Value>) -> MemoryToolRequest {
    let action = trim(&string_arg(args, "action")).to_lowercase();
    let document = match args.get("document").and_then(Value::as_str) {
        Some(document) => Some(document.to_owned()),
        // `read` with no document means every document; a write with none is a
        // refusal rather than a guess, so it stays absent.
        None if action == "read" => Some(via_core::memory_scopes::ALL_SCOPE.to_owned()),
        None => None,
    };
    MemoryToolRequest {
        action,
        document,
        old_text: args
            .get("old_text")
            .and_then(Value::as_str)
            .map(str::to_owned),
        // `hasOwnProperty` — an absent `new_text` is a refusal, an explicit
        // empty one is a deletion, so presence is preserved rather than
        // collapsed to `""`.
        new_text: args
            .get("new_text")
            .map(|value| value.as_str().unwrap_or_default().to_owned()),
        content: args
            .get("content")
            .and_then(Value::as_str)
            .map(|content| trim(content).to_owned()),
    }
}

/// Build a [`NotesToolRequest`] from raw arguments.
///
/// **External contract** — `tool-call-handler.mjs:1069-1073`. Items are
/// trimmed, blanks dropped, and the list re-capped at
/// [`catalog::MAX_NOTES_ITEMS`] — the schema's `maxItems` is advisory to the
/// model, and this is the enforcement.
fn notes_request(args: &Map<String, Value>) -> NotesToolRequest {
    let items: Vec<String> = args
        .get("items")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(|value| trim(value.as_str().unwrap_or_default()).to_owned())
                .filter(|item| !item.is_empty())
                .take(catalog::MAX_NOTES_ITEMS)
                .collect()
        })
        .unwrap_or_default();
    NotesToolRequest {
        action: trim(&string_arg(args, "action")).to_lowercase(),
        list: args
            .get("list")
            .and_then(Value::as_str)
            .map(|list| trim(list).to_owned()),
        items,
    }
}
