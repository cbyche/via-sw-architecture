//! VIA's composition root and its HTTP/WebSocket server.
//!
//! Ported from `qwen-audio-agent` v1.11.0
//! `server/src/app/{gateway-application, bootstrap, offline-notifications}.mjs`
//! and `server/src/index.mjs`.
//!
//! Everything below this crate answers one question well. This crate is where
//! the answers are wired together, put behind a socket, and taken down again in
//! the right order. It adds no policy of its own: where a rule exists, the
//! crate that owns it is called.
//!
//! | Module | What | Upstream |
//! | --- | --- | --- |
//! | [`services`] | every service, by injection | `gateway-application.mjs:41-54` |
//! | [`application`] | construct, bind, serve, close | `gateway-application.mjs:482-554`, `bootstrap.mjs` |
//! | [`http`] | the route table | `gateway-application.mjs:216-436` |
//! | [`health`] | `/api/health` and its **field order** | `gateway-application.mjs:224-269` |
//! | [`realtime`] | `WS /api/realtime` and the 52-name vocabulary | `voice/realtime-gateway.mjs` (transport half) |
//! | [`serve`] | the accept loop, and the destroy contract | `voice/realtime-gateway.mjs:70-74` |
//! | [`offline`] | the delayed hand-off to the host | `offline-notifications.mjs` |
//! | [`runtime`] | startup and shutdown, in order | `index.mjs` |
//! | [`backend`] | Layer 3, as four questions | `agent/agent-client.mjs:155-206` |
//!
//! # The four things this crate is responsible for
//!
//! **Importing the factory must not bind a port.** Upstream needs an
//! `autoStart` flag for that; here it is structural —
//! [`GatewayApplication::build`] owns no socket and
//! [`GatewayApplication::bind`] is a separate fallible call. See
//! [`application`].
//!
//! **`/api/health`'s field order is a contract.** Twenty-six keys, in one
//! order, asserted against [`health::HEALTH_FIELD_ORDER`]. It is a struct
//! rather than a map so the compiler holds the order. See [`health`].
//!
//! **A WebSocket upgrade to the wrong path gets no HTTP response at all.**
//! Not a 404 — the socket is destroyed. That is why this crate drives hyper
//! itself instead of calling `axum::serve`. See [`serve`].
//!
//! **The setup gate runs before the lease is touched.** A misconfigured start
//! must never disturb a running Gateway. See [`runtime`].
//!
//! # What this crate builds on rather than restating
//!
//! - [`via_protocol`] owns the 52 wire names, [`SessionMode`](via_protocol::SessionMode),
//!   the protocol version and the capability list. This crate **serves** them.
//! - [`via_core`] owns the origin allow-list, the DNS-rebinding defence, the
//!   signed identity cookie and the setup gate; [`http::middleware`] is the
//!   wiring, not a second copy of the rules.
//! - [`via_voice`] owns the input arbitration, the voice slot, the Injection
//!   Gate, the mode plan and the upgrade decision.
//! - [`via_work`] owns the Work record and its event vocabulary;
//!   [`via_realtime`] owns the provider registry and `/api/health`'s realtime
//!   block; [`via_i18n`] owns every sentence a person reads.
//!
//! # Layering
//!
//! `docs/architecture.md` §9 gives the App band
//! `app → agent, app, conversation, core, task, voice` plus the entry-file row
//! `root → app, process, shared`. This crate is the only one that may see all
//! of them at once, which is what lets it bridge Layer 1's three traits —
//! [`via_voice::tools::BackendAvailability`],
//! [`via_voice::tools::PermissionResponder`] and
//! [`via_voice::tools::DelegationRunners`] — onto Layer 3.
//!
//! # Fidelity
//!
//! Every value that reproduces an upstream literal names the file it came from
//! in its own documentation, and `docs/reference/contracts.json` is the
//! acceptance spec — `tests/contracts.rs` parses it rather than retyping it.
//! Deviations are recorded in `docs/deviations/phase-5-via-app.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod application;
pub mod backend;
pub mod error;
pub mod health;
pub mod http;
pub mod offline;
pub mod realtime;
pub mod runtime;
pub mod serve;
pub mod services;
pub mod state;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use application::{GatewayApplication, Serving};
pub use backend::{
    BackendDescription, FrontendOnlyBackend, GatewayBackend, HarnessBackend, PermissionRelayError,
};
pub use error::AppError;
pub use health::{
    HEALTH_FIELD_ORDER, HEALTH_STATUS_READY, HealthInputs, HealthPayload, HealthSnapshot,
    LivenessProbe, ReadinessProbe, RealtimeHealth, SessionModeHealth, health_payload,
};
pub use http::{JSON_BODY_LIMIT, NOT_FOUND_BODY, REALTIME_ROUTE, router};
pub use offline::{
    OFFLINE_NOTIFICATION_TYPE, OfflineNotification, OfflineNotifications, OfflineTask,
    PROGRESS_STATUS,
};
pub use realtime::{
    ClientDescriptor, ClientFrame, EngineContext, EngineFactory, MAX_PAYLOAD_BYTES,
    NoEngineFactory, NoModelEngine, ServerFrame, SharedGate, VoiceClientRegistry, VoiceEngine,
};
pub use runtime::{
    EXIT_TIMEOUT, HEARTBEAT_INTERVAL, Runtime, Startup, Stoppable, heartbeat, shutdown,
    shutdown_with_timeout,
};
pub use serve::{BoundServer, destroys_socket, is_websocket_upgrade};
pub use services::{Services, ServicesBuilder};
pub use state::{AppState, InstanceIdentity};
