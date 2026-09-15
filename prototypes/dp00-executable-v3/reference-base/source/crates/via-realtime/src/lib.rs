//! VIA's realtime transport layer — Layer 1's provider seam.
//!
//! Ported from `qwen-audio-agent` v1.11.0 `server/src/voice/`:
//! `realtime-provider.mjs`, `realtime-provider-extension.mjs`,
//! `providers/{provider-registry, registry, ga-protocol,
//! openai-compatible-protocol}.mjs`, `realtime-connection-status.mjs`,
//! `realtime-errors.mjs`, `reconnect-backoff.mjs` and the half of
//! `response-lifecycle.mjs` the session branches on.
//!
//! It ships **no provider**. `via-realtime-dashscope`, `via-realtime-mock` and
//! the local providers each own one; what lives here is the shape they share.
//!
//! # The two-trait split
//!
//! Upstream's single biggest structural idea is that *which service* and *which
//! wire dialect* are different questions:
//!
//! ```text
//! RealtimeProvider  ── endpoint, credential, model, voice, session payload,
//!                      error corpus, capability declaration
//!         │
//!         └─ protocol() ─► RealtimeProtocol  ── envelope, event names,
//!                                               id namespaces, correlation
//! ```
//!
//! Two dialects ship — [`openai_compatible_protocol`] (the beta OpenAI Realtime
//! envelope, which DashScope speaks) and [`ga_realtime_protocol`] (the GA one,
//! which huggingface/speech-to-speech speaks). The split is what will let a local
//! model reuse a dialect rather than reimplement it, and it is why the Gateway
//! above this crate never sees a wire format at all: it consumes only what
//! [`RealtimeProtocol::normalize_incoming`] returns.
//!
//! # Capabilities instead of provider names
//!
//! [`ProviderCapabilities`] is five flags, and the reason it exists is upstream's
//! own: *"optional features require opt-in and providers declare known
//! constraints, without the frontend ever branching on a provider name."* Every
//! behavioural difference between the two shipped providers — whether
//! `session.update` is acknowledged, whether there is one response slot,
//! whether response metadata is echoed, whether a client-assigned item id
//! survives — is one of those flags, and [`RealtimeSession`] reads the flag
//! rather than the key.
//!
//! # `dictation` has no model turn
//!
//! `docs/architecture.md` §2 makes this a hard requirement on the trait rather
//! than a runtime detail: in `dictation` the provider *may be a plain streaming
//! ASR*. So [`RealtimeProvider::model`], [`RealtimeProvider::voice`] and
//! [`RealtimeProvider::model_profile`] are all `Option`, the model catalog may be
//! empty, and nothing in [`RealtimeSession`] requires a `response.*` event ever
//! to arrive. A session in that mode answers every response-creating call with
//! `skipped / no_model_turn` without touching the socket.
//!
//! # What is here
//!
//! | Module | Upstream | What |
//! | --- | --- | --- |
//! | [`provider`] | `provider-registry.mjs` (provider half) | [`RealtimeProvider`], validation |
//! | [`protocol`] | `openai-compatible-protocol.mjs`, `ga-protocol.mjs` | [`RealtimeProtocol`] and the two dialects |
//! | [`capabilities`] | `realtime-provider.mjs:61-88` | the five flags |
//! | [`registry`] | `provider-registry.mjs`, `registry.mjs` | aliases, list filters, `/api/health` |
//! | [`session`] | `realtime-provider.mjs` | one socket, correlation, the output queue |
//! | [`status`] | `realtime-connection-status.mjs` | the six states and their precedence |
//! | [`event_error`] | `realtime-errors.mjs`, `realtime-provider.mjs:46-59` | the composed message, the classification vocabulary |
//! | [`backoff`] | `reconnect-backoff.mjs` | the reconnect ladder |
//! | [`lifecycle`] | `response-lifecycle.mjs` | which events prove a response exists |
//! | [`extension`] | `realtime-provider-extension.mjs` | the host-extension seam |
//! | [`testing`] | `realtime-provider-registry.test.mjs:11-30` | a provider double and an in-memory socket |
//!
//! # What it builds on rather than restating
//!
//! - [`via_catalog`] owns the model profiles, the provider keys and aliases, and
//!   the configuration-signature hash and its key order.
//! - [`via_i18n`] owns every sentence a person reads; there is no user-facing
//!   literal in this crate.
//! - [`via_protocol`] owns [`SessionMode`](via_protocol::SessionMode).
//!
//! Deviations are recorded in `docs/deviations/phase-5-via-realtime.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod backoff;
pub mod capabilities;
pub mod error;
pub mod event_error;
pub mod extension;
pub mod lifecycle;
pub mod protocol;
pub mod provider;
pub mod registry;
pub mod session;
pub mod status;
pub mod testing;

pub use backoff::{
    BackoffConfig, DEFAULT_BASE_MS, DEFAULT_JITTER_RATIO, DEFAULT_MAX_MS, JitterSource,
    MAX_ATTEMPT_EXPONENT, ReconnectBackoff, STABLE_CONNECTION_MS, SystemJitter,
};
pub use capabilities::{CAPABILITY_FLAGS, ProviderCapabilities};
pub use error::RealtimeError;
pub use event_error::{
    ErrorClass, is_recoverable_realtime_inactivity_error, realtime_event_classification_text,
    realtime_event_error_message, realtime_event_error_message_with_fallback,
};
pub use lifecycle::{
    FAILED_RESPONSE_STATUSES, RESPONSE_ACTIVITY_TYPES, is_completed_status,
    is_response_activity_event, realtime_response_id,
};
pub use protocol::{
    GaRealtimeProtocol, OpenAiCompatibleProtocol, PROTOCOL_METHODS, RESPONSE_CORRELATION_KEY,
    RealtimeProtocol, ga_realtime_protocol, openai_compatible_protocol,
};
pub use provider::{
    AgentContext, AgentContextPatch, ConnectionInfo, Injection, InputProjection,
    InputProjectionRequest, PROVIDER_METHODS, PermissionRequest, RealtimeProvider, SessionRequest,
    Visibility, clean_key, define_realtime_provider, is_valid_provider_key,
    validate_realtime_provider,
};
pub use registry::{
    ActiveRealtime, ListOptions, ProviderDescriptor, RealtimeProviderRegistry, describe_provider,
};
pub use session::{
    BUSY_RETRY_DELAYS, CONNECT_TIMEOUT, DEFAULT_RESPONSE_INACTIVITY_TIMEOUT,
    DEFAULT_RESPONSE_START_TIMEOUT, DIAGNOSTIC_RESPONSE_TIMEOUT, Diagnostic, FunctionOutputOptions,
    Guard, InjectOutcome, MAX_BUSY_RETRIES, OutcomeKind, OutcomePhase, POST_CANCEL_RECOVERY,
    ProviderEvent, RealtimeSession, ResponseContext, ResponseOrigin, ResponseOutcome, SessionEvent,
    SessionEvents, SessionOptions, Transport,
};
pub use status::{
    ConnectionState, RealtimeConnectionInputs, RealtimeConnectionStatus, realtime_connection_status,
};
