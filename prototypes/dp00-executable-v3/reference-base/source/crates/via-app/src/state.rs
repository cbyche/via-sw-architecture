//! The state every handler shares.
//!
//! One `Arc`-backed struct rather than a dozen extractors, because upstream's
//! handlers close over one lexical scope and reproducing that literally is what
//! keeps the diff readable. It is `Clone` and cheap: every field inside
//! [`crate::Services`] is already an `Arc`.

use std::sync::Arc;

use via_i18n::Locale;

use crate::health::{HealthInputs, HealthPayload, HealthSnapshot, RealtimeHealth, health_payload};
use crate::realtime::{EngineFactory, NoEngineFactory, VoiceClientRegistry};
use crate::services::Services;

/// What `/api/health` needs that the configuration does not supply.
#[derive(Debug, Clone, Default)]
pub struct InstanceIdentity {
    /// The lease's `instanceId`, echoed as `gatewayInstanceId`.
    ///
    /// **External contract** — `gateway-application.mjs:237`. Upstream reads
    /// `process.env.QWEN_AUDIO_GATEWAY_INSTANCE_ID`, which
    /// `server/src/index.mjs:56` writes from the lease. VIA passes the value
    /// instead of round-tripping it through the environment: `std::env::set_var`
    /// is `unsafe` in edition 2024, and a process-global would make two
    /// Gateways in one test binary report each other's identity.
    pub instance_id: Option<String>,
    /// When this instance started, ISO-8601.
    ///
    /// **External contract** — `gateway-application.mjs:238`
    /// (`QWEN_AUDIO_GATEWAY_STARTED_AT`).
    pub started_at: Option<String>,
}

/// The shared handler state.
#[derive(Clone)]
pub struct AppState {
    services: Services,
    identity: InstanceIdentity,
    clients: Arc<VoiceClientRegistry>,
    engines: Arc<dyn EngineFactory>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("services", &self.services)
            .field("identity", &self.identity)
            .field("engines", &self.engines)
            .finish()
    }
}

impl AppState {
    /// Wrap `services`.
    ///
    /// The engine factory defaults to [`NoEngineFactory`]: a Gateway with no
    /// realtime binding still serves every route and every non-model frame.
    /// See [`crate::realtime::engine`].
    #[must_use]
    pub fn new(services: Services, identity: InstanceIdentity) -> Self {
        Self {
            services,
            identity,
            clients: Arc::new(VoiceClientRegistry::default()),
            engines: Arc::new(NoEngineFactory),
        }
    }

    /// Replace the engine factory.
    #[must_use]
    pub fn with_engines(mut self, engines: Arc<dyn EngineFactory>) -> Self {
        self.engines = engines;
        self
    }

    /// The engine factory one connection opens its model session through.
    #[must_use]
    pub fn engines(&self) -> Arc<dyn EngineFactory> {
        self.engines.clone()
    }

    /// The injected services.
    #[must_use]
    pub const fn services(&self) -> &Services {
        &self.services
    }

    /// The connected voice clients.
    #[must_use]
    pub fn clients(&self) -> &Arc<VoiceClientRegistry> {
        &self.clients
    }

    /// The locale every user-facing string is rendered in.
    #[must_use]
    pub const fn locale(&self) -> Locale {
        self.services.locale
    }

    /// Which lease this Gateway holds.
    #[must_use]
    pub const fn instance(&self) -> &InstanceIdentity {
        &self.identity
    }

    /// Assemble `/api/health`.
    ///
    /// # Errors
    ///
    /// [`via_realtime::RealtimeError::UnsupportedProvider`] when the configured
    /// default provider is not registered. Upstream throws the same way
    /// (`describeActiveRealtime` → `registry.resolve`), and the route turns it
    /// into the terminal error handler's 500 rather than serving a health
    /// payload that lies about which provider is active.
    pub async fn health(&self) -> Result<HealthPayload, via_realtime::RealtimeError> {
        let services = &self.services;
        let active = services
            .realtime_registry
            .describe_active_realtime(services.realtime_provider.as_deref())?;
        let inputs = HealthInputs {
            gateway_instance_id: self.identity.instance_id.clone(),
            gateway_started_at: self.identity.started_at.clone(),
            announce_into_context: services.config.announce_into_context,
            result_context_max_chars: services.config.result_context_max_chars,
            announcement_batch_ms: services.config.announcement_batch_ms,
            announcement_quiet_ms: services.config.announcement_quiet_ms,
            identity_mode: services.config.identity_mode,
            harness_configured: services.backend.enabled(),
            local_realtime: services.local_realtime.clone(),
        };
        Ok(health_payload(
            &inputs,
            RealtimeHealth::from(active),
            HealthSnapshot {
                input_suspension: services.input_arbitration.status().await,
                frontend_memory: services.memory.health(),
                notes: services.notes.health(),
                task_store: services.work.store_health().await,
                voice_clients: self.clients.health(),
            },
            &services.backend.describe(),
            &services.backend.status(),
        ))
    }
}
