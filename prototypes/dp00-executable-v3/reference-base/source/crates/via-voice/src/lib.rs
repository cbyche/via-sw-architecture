//! VIA's voice session layer — Layer 1 above the realtime transport.
//!
//! Ported from `qwen-audio-agent` v1.11.0 `server/src/voice/` and
//! `shared/input-parts.mjs`. `via-realtime` owns the socket, the provider seam
//! and the wire dialects; **this crate owns everything a voice session decides**
//! once a frame has been normalized.
//!
//! ```
//! use via_voice::{
//!     announcement::AnnouncementWindow,
//!     gate::InjectionGate,
//!     response::ResponseOrigin,
//! };
//! use via_audio::SampleRate;
//!
//! let mut gate = InjectionGate::new(SampleRate::HZ_24000);
//! gate.set_output_enabled(true);
//! assert!(!gate.is_blocked(), "an idle session may speak a result");
//!
//! // The user starts talking; a finished Work must now wait.
//! gate.window_mut().begin_turn("voice-1");
//! assert!(gate.is_blocked());
//!
//! // They stop, and the model answers without audio.
//! gate.window_mut().end_speech();
//! gate.window_mut().response_done(&via_voice::announcement::ResponseDone {
//!     turn_id: "voice-1".to_owned(),
//!     origin: ResponseOrigin::Model,
//!     ..Default::default()
//! });
//! assert!(!gate.is_blocked());
//! ```
//!
//! # The Injection Gate
//!
//! The highest-value thing this port supplies (`docs/architecture.md` §3).
//! ARGO's realtime stack lists the problem under *"Absent by design — do not
//! debug these"*: **delegation results can land mid-sentence**, because
//! injection is queued the moment the result exists and holding it until
//! neither side is speaking needs playback position from the client. qwen's
//! client protocol already carries that position, as `playback.started` /
//! `playback.ended` / `playback.cancelled` — which is why porting its
//! WebSocket vocabulary whole is load-bearing rather than incidental.
//!
//! The predicate is
//! `sleeping || waking || !output_enabled || window.is_blocked()`, where
//! `window.is_blocked()` is
//! `user_speaking || turn_pending || audio_responses.nonEmpty`. A result is
//! marked **delivered only after playback finishes**, retries are bounded so one
//! malformed result cannot block every later completion, and a renewable claim
//! stops two live frontends presenting the same result.
//! [`via_audio::PlaybackCursor`] is the drain predicate.
//!
//! # The modules
//!
//! | Module | What |
//! | --- | --- |
//! | [`gate`] | **the Injection Gate** — the predicate, the cursor, the receipt admission |
//! | [`announcement`] | the blocking window, the delivery engine, the model-visible envelope |
//! | [`tools`] | the nine frontend tools, their instructions, the tool-call handler |
//! | [`prompt`] | `buildFrontendInstructions` and the packaged `PROMPT.md` / `ASSISTANT.md` |
//! | [`response`] | response lifecycle and the correlated response context |
//! | [`guards`] | the response guards, including the action-promise guard |
//! | [`turn`] | item-id → turn correlation |
//! | [`session`] | turn identity, the gateway's timing constants, the upgrade decision |
//! | [`mode`] | [`via_protocol::SessionMode`] dispatch and its two degradations |
//! | [`arbitration`] | the host input suspension state machine |
//! | [`assets`] | the per-session input asset registry |
//! | [`input`] | input parts: normalization, anchors, the model-visible projection |
//! | [`clients`] | the per-owner voice slot and the client capability resolution |
//! | [`permission`] | the per-session backend-permission policy |
//! | [`sleep`] | the inactivity timer and its `can_sleep` predicate |
//! | [`status`] | the realtime connection state and the `/api/health` aggregation |
//! | [`frontend`] | the realtime session as this layer consumes it |
//! | [`provider`] | the read-only provider view and the five capability flags |
//! | [`text`] | the string primitives, and which unit each bound counts in |
//!
//! # What it builds on rather than restating
//!
//! - [`via_protocol`] owns every wire event name, [`via_protocol::SessionMode`],
//!   [`via_protocol::WorkStatus`] and the coded errors.
//! - [`via_i18n`] owns every sentence a person or the model reads — the nine
//!   tool descriptions included. Its `zh` column *is* upstream's text.
//! - [`via_conversation`] owns the `memory` and `notes` **implementations**, the
//!   four context blocks and the conversation record. This crate owns those two
//!   tools' schemas and descriptions and calls into that crate.
//! - [`via_work`] owns the Work record, the admission scheduler and the
//!   notification claim the announcement manager renews.
//! - [`via_audio`] owns [`PlaybackCursor`](via_audio::PlaybackCursor).
//! - [`via_core`] owns [`Config`](via_core::Config) and the memory scope table.
//!
//! # Layering
//!
//! `docs/architecture.md` §9: `voice → conversation, core, shared, task, voice`.
//! There is **no Layer 3 dependency** — no `via-downstream`, `via-acp`,
//! `via-backends`, `via-process` or `via-mcp-tools`. Where this layer needs a
//! fact from Layer 3 it takes a trait and `via-app` bridges it:
//! [`tools::BackendAvailability`], [`tools::PermissionResponder`] and
//! [`tools::DelegationRunners`]. A harness is Layer 2's to open, and the
//! realtime layer never holds a session, a `work_id` or a permission decision.
//!
//! # Fidelity
//!
//! Every value that reproduces an upstream literal names the file it came from
//! in its own documentation, and `docs/reference/contracts.json` is the
//! acceptance spec. Deviations are recorded in `docs/deviations/phase-5.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod announcement;
pub mod arbitration;
pub mod assets;
pub mod clients;
pub mod frontend;
pub mod gate;
pub mod guards;
pub mod input;
pub mod mode;
pub mod permission;
pub mod prompt;
pub mod provider;
pub mod response;
pub mod session;
pub mod sleep;
pub mod status;
pub mod text;
pub mod tools;
pub mod turn;

pub use announcement::{
    Announcement, AnnouncementEvent, AnnouncementManager, AnnouncementManagerConfig,
    AnnouncementWindow, NoClaims, NotificationClaims, ResponseDone, format_progress,
    format_work_results, truncate_result,
};
pub use arbitration::{
    DEFAULT_INPUT_SUSPEND_TTL_MS, Holder, InputArbitration, MAX_INPUT_SUSPEND_TTL_MS,
    OwnerRequired, SuspensionChange, SuspensionState, SuspensionStatus,
};
pub use assets::{AssetError, AssetMetadata, INPUT_REF_PREFIX, InputAssetRegistry};
pub use clients::{
    ActivationResult, ActiveVoiceClients, DeclaredCapabilities, VoiceCapabilities, VoiceClient,
    client_voice_capabilities,
};
pub use frontend::{
    FunctionOutputOptions, ResponseOutcome, ResponseRequestContext, SettlePhase, VoiceFrontend,
};
pub use gate::{GateFlags, InjectionGate, accepts_playback_receipt};
pub use guards::{
    ACTION_PROMISE_GUARD_ID, ACTION_PROMISE_MAX_CHARS, ActionPromiseGuard, GuardDecision,
    GuardObservation, GuardTurnState, ResponseGuard, default_guards,
    evaluate_default_response_guards, evaluate_response_guards, is_response_guard_turn_current,
    promises_action,
};
pub use input::{
    ALLOWED_URL_PROTOCOLS, AttachmentMetadata, INPUT_REF_META_KEY, InputError, InputPart,
    MAX_INPUT_FILE_BYTES, MAX_INPUT_PARTS, MAX_INPUT_TOTAL_FILE_BYTES, PartSource, SourceKind,
    SourceText, display_input_text, frontend_input_projection, input_attachment_metadata,
    input_file_parts, input_part_label, input_text, merge_input_parts, normalize_input_parts,
    with_attachment_anchors,
};
pub use mode::{Degradation, ModePlan};
pub use permission::{PermissionDecision, PermissionMode, SessionPermissionPolicy};
pub use prompt::{
    ASSISTANT_PROFILE_CLOSE_TAG, ASSISTANT_PROFILE_HEADING, ASSISTANT_PROFILE_OPEN_TAG,
    assemble_frontend_instructions, build_frontend_instructions,
    build_packaged_frontend_instructions, packaged_assistant_profile, packaged_prompt,
};
pub use provider::{ErrorClassification, ProviderCapabilities, ProviderView};
pub use response::{
    CorrelatedContext, PendingTranscript, RESPONSE_ACTIVITY_TYPES, RESPONSE_FAILURE_STATUSES,
    ResponseContext, ResponseContexts, ResponseOrigin, ServerEvent, ensure_response_context,
    merge_response_context, response_activity_context_patch,
};
pub use session::{
    MAX_PENDING_AUDIO_CHUNKS, PERMISSION_RESPONSE_GRACE_MS, REALTIME_ROUTE,
    REALTIME_STABLE_CONNECTION_MS, RESPONSE_CONTEXT_CLEANUP_MS, RESPONSE_START_WATCHDOG_MS,
    TurnTracker, UpgradeDecision, WAKE_CONNECT_MAX_ATTEMPTS, WAKE_CONNECT_RETRY_BACKOFF_MS,
    new_claimant_id, new_text_turn_id, upgrade_decision,
};
pub use sleep::{
    CanSleepInputs, NoWakeWordEngine, SleepController, SleepControllerConfig, SleepingWakeWord,
    WakeWordDetectorOpener, WakeWordEnabled, WakeWordLifecycle, WakeWordPrepareInputs,
    WakeWordPrepareOutcome, can_sleep,
};
pub use status::{
    ClientTypeCounts, DegradedMode, RealtimeAggregate, RealtimeState, RealtimeStateInputs,
    RealtimeStatus, StateCounts, VoiceClientStatus, VoiceClientsHealth, aggregate,
    realtime_connection_status,
};
pub use tools::{
    ClientContext as ToolClientContext, ToolCall, ToolCallHandler, ToolCallHandlerConfig,
    ToolCallOutcome, TurnTranscripts, frontend_tools,
};
pub use turn::{CompletedTurn, TurnCorrelation, TurnRef};
/// The per-locale keyword table a [`WakeWordLifecycle`] is built from.
///
/// Re-exported so a composition root can build one — an empty table, or one
/// read off installed models — without naming `via-wake-word` itself.
pub use via_wake_word::KeywordSet;
