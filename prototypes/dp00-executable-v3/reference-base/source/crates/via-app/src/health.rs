//! `GET /api/health` — the discovery surface, and its field order.
//!
//! **The field order is the contract.** `docs/reference/contracts.json`
//! (`http-route` / *GET /api/health*) spells out twenty-six keys *"in this
//! order"*, and `docs/architecture.md` §14 lists `/api/health`'s field order
//! among the things reproduced verbatim in code. A `HashMap` cannot do it and
//! an `IndexMap` would only do it if every insertion site stayed in order
//! forever, so the payload is a **struct**: `serde` emits struct fields in
//! declaration order unconditionally, which turns the contract into something
//! the compiler holds rather than something a reviewer checks.
//!
//! [`HEALTH_FIELD_ORDER`] is the same list as data, so a test can assert the
//! serialized payload against it without retyping either.
//!
//! # Four things upstream's payload does not carry
//!
//! **`sessionModes`.** `docs/architecture.md` §2 requires the mode degradation
//! to be *"reported on `/api/health`, never silent"*: `agent` degrades to
//! `direct` with no harness configured. `interface` used to degrade too, until
//! `via-context` landed in phase 7; it now reports itself, and the row stays in
//! the payload because a client reads the table rather than branching on which
//! rows are present. Upstream has no `SessionMode` at all, so there is nothing
//! to reproduce; the block is appended **after** the twenty-six contract keys
//! so their order is untouched.
//!
//! **`voiceClients.degradedModes`.** The same fact per live session, which is
//! `via-voice`'s [`aggregate`](via_voice::aggregate) and lands inside the
//! `voiceClients` block it already owns.
//!
//! **No credential, anywhere.** `server/test/embedded-gateway.test.mjs:78-81`
//! asserts the serialized body contains neither the API key, nor a signed
//! token from the realtime base URL, nor the provider hostname.
//! [`RealtimeHealth`] is built only from
//! [`via_realtime::ActiveRealtime`], which carries a `sha256`
//! *signature* of the configuration rather than the configuration.
//!
//! **`localRealtime`.** `via_realtime_local`'s own module docs: *"`via-realtime`'s
//! `ActiveRealtime` is a closed struct with no free-form slot, and this crate
//! does not edit it. So the note is a typed value this crate publishes and the
//! Gateway embeds beside the realtime block."* `via-app` stays provider-agnostic
//! — it does not depend on `via-realtime-local` — so [`HealthPayload::local_realtime`]
//! carries whatever the composition root already serialized
//! ([`Services::local_realtime`](crate::Services::local_realtime)) rather than
//! computing it here; `null` when no on-device provider was registered at all.

use serde::Serialize;
use via_catalog::{ModelProfile, TransportCapabilities};
use via_core::IdentityMode;
use via_downstream::HarnessHealth;
use via_protocol::{GATEWAY_CAPABILITIES, GATEWAY_PROTOCOL_VERSION, SessionMode};
use via_realtime::{ActiveRealtime, ProviderDescriptor};
use via_store::StoreHealth;
use via_voice::{SuspensionStatus, VoiceClientsHealth, mode::ModePlan};

use crate::backend::BackendDescription;

/// The twenty-six contract keys of `/api/health`, in order.
///
/// **External contract** — `docs/reference/contracts.json`, `http-route` /
/// *GET /api/health*, and `server/src/app/gateway-application.mjs:230-268`.
/// `tests/health.rs` asserts the serialized payload's key order against this
/// list, and `via-conformance` asserts this list against the catalogue.
///
/// VIA's own additions are **not** in it — see the module documentation.
pub const HEALTH_FIELD_ORDER: [&str; 26] = [
    "ok",
    "status",
    "protocolVersion",
    "capabilities",
    "gatewayInstanceId",
    "gatewayStartedAt",
    "inputSuspension",
    "voiceConfigured",
    "realtimeProvider",
    "realtimeLabel",
    "realtimeModel",
    "realtimeModelProfile",
    "realtimeModelCatalog",
    "realtimeInputSampleRate",
    "realtimeConfigurationSignature",
    "realtimeProviders",
    "announceIntoContext",
    "resultContextMaxChars",
    "announcementBatchMs",
    "announcementQuietMs",
    "frontendMemory",
    "notes",
    "taskStore",
    "identityMode",
    "voiceClients",
    "backend",
];

/// The literal `status` of a healthy Gateway.
///
/// **External contract** — `gateway-application.mjs:233`. Gateway liveness is
/// independent of optional backend readiness: this is `'ready'` the moment the
/// socket is listening, whatever the harness is doing.
pub const HEALTH_STATUS_READY: &str = "ready";

/// `GET /livez` — `{"ok":true,"status":"live"}`.
///
/// **External contract** — `gateway-application.mjs:216-218`, deep-equality
/// asserted by `server/test/embedded-gateway.test.mjs:50-52`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LivenessProbe {
    /// Always `true`.
    pub ok: bool,
    /// Always `"live"`.
    pub status: &'static str,
}

impl Default for LivenessProbe {
    fn default() -> Self {
        Self {
            ok: true,
            status: "live",
        }
    }
}

/// `GET /readyz` — `{"ok":true,"status":"ready"}`.
///
/// **External contract** — `gateway-application.mjs:220-222`. It reports ready
/// as soon as the Gateway is listening and deliberately does **not** reflect
/// backend readiness; the catalogue says so in as many words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ReadinessProbe {
    /// Always `true`.
    pub ok: bool,
    /// Always `"ready"`.
    pub status: &'static str,
}

impl Default for ReadinessProbe {
    fn default() -> Self {
        Self {
            ok: true,
            status: HEALTH_STATUS_READY,
        }
    }
}

/// The realtime half of the payload, already stripped of credentials.
#[derive(Debug, Clone)]
pub struct RealtimeHealth {
    /// Whether the active provider is configured.
    pub configured: bool,
    /// The active provider key.
    pub provider: String,
    /// Its label.
    pub label: String,
    /// The active model id.
    pub model: Option<String>,
    /// The active model's profile.
    pub model_profile: Option<ModelProfile>,
    /// Every model the provider can be pointed at.
    pub model_catalog: Vec<ModelProfile>,
    /// The capture rate a client must use.
    pub input_sample_rate: u32,
    /// `sha256` of the configuration identity.
    pub configuration_signature: String,
    /// The providers a client may select at connect time.
    pub providers: Vec<ProviderDescriptor>,
}

impl From<ActiveRealtime> for RealtimeHealth {
    fn from(active: ActiveRealtime) -> Self {
        Self {
            configured: active.configured,
            provider: active.provider,
            label: active.label,
            model: active.model,
            model_profile: active.model_profile,
            model_catalog: active.model_catalog,
            input_sample_rate: active.input_sample_rate,
            configuration_signature: active.configuration_signature,
            providers: active.providers,
        }
    }
}

impl RealtimeHealth {
    /// The transport capabilities of the active model, when there is one.
    #[must_use]
    pub fn transport_capabilities(&self) -> Option<TransportCapabilities> {
        self.model_profile
            .as_ref()
            .map(|profile| profile.transport_capabilities)
    }
}

/// One session mode, and what it actually runs as on this Gateway.
///
/// **VIA's own** — see the module documentation. `reason` is
/// [`Degradation::as_str`](via_voice::Degradation::as_str), so the same code
/// appears here and in `voiceClients.degradedModes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionModeHealth {
    /// The mode a client may ask for.
    pub requested: &'static str,
    /// What a session in that mode actually runs as.
    pub effective: &'static str,
    /// Why they differ, or `null`.
    pub reason: Option<&'static str>,
}

impl SessionModeHealth {
    /// Resolve every mode against whether a harness is configured.
    ///
    /// The Context Engine ships in this workspace, so `via_voice::ModePlan::new`
    /// resolves `interface` to itself and this table reports it with no reason.
    /// Whether that engine has a *screen* bound is a per-session fact the engine
    /// answers itself (`via_context::SurfaceStatus`); it is deliberately not
    /// folded in here, because this table describes what a mode *is* on this
    /// Gateway and not what one session happens to be looking at.
    #[must_use]
    pub fn all(harness_configured: bool) -> Vec<Self> {
        SessionMode::ALL
            .iter()
            .map(|mode| {
                let plan = ModePlan::new(*mode, harness_configured);
                Self {
                    requested: plan.requested().as_str(),
                    effective: plan.effective().as_str(),
                    reason: plan.degradation().map(via_voice::Degradation::as_str),
                }
            })
            .collect()
    }
}

/// The `/api/health` body.
///
/// Field order is the contract; see the module documentation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthPayload {
    /// Always `true`. Gateway liveness is independent of backend readiness
    /// (`gateway-application.mjs:231-232`).
    pub ok: bool,
    /// [`HEALTH_STATUS_READY`].
    pub status: &'static str,
    /// [`GATEWAY_PROTOCOL_VERSION`]. Clients branch on a capability, never on a
    /// product version.
    pub protocol_version: &'static str,
    /// [`GATEWAY_CAPABILITIES`], in declaration order.
    pub capabilities: &'static [&'static str],
    /// The lease's instance id, or `null`.
    pub gateway_instance_id: Option<String>,
    /// When this instance started, ISO-8601, or `null`.
    pub gateway_started_at: Option<String>,
    /// The input-arbitration status, verbatim.
    pub input_suspension: SuspensionStatus,
    /// Whether the active realtime provider is configured.
    pub voice_configured: bool,
    /// The active provider key.
    pub realtime_provider: String,
    /// The active provider's label.
    pub realtime_label: String,
    /// The active model id, or `null`.
    pub realtime_model: Option<String>,
    /// The active model's profile, or `null`.
    pub realtime_model_profile: Option<ModelProfile>,
    /// Every model the active provider can be pointed at.
    pub realtime_model_catalog: Vec<ModelProfile>,
    /// The rate a client must capture at.
    pub realtime_input_sample_rate: u32,
    /// `sha256` of the realtime configuration identity — never the
    /// configuration.
    pub realtime_configuration_signature: String,
    /// The providers a client may select through the `connect` event.
    ///
    /// `gateway-only` providers are excluded, which
    /// `server/test/gateway-application.test.mjs:56-59` asserts directly.
    pub realtime_providers: Vec<ProviderDescriptor>,
    /// Whether finished Work is written back into the model's context.
    pub announce_into_context: bool,
    /// How much of a result may be.
    pub result_context_max_chars: i64,
    /// The announcement batching window.
    pub announcement_batch_ms: i64,
    /// The quiet period the Injection Gate waits for.
    pub announcement_quiet_ms: i64,
    /// `USER.md` / `MEMORY.md` persistence health.
    pub frontend_memory: via_conversation::memory_service::MemoryServiceHealth,
    /// `notes.json` persistence health.
    pub notes: via_conversation::NotesHealth,
    /// `tasks.json` persistence health.
    pub task_store: StoreHealth,
    /// `personal` or `multi`.
    pub identity_mode: &'static str,
    /// The connected voice clients, by type, by provider and by state.
    pub voice_clients: VoiceClientsHealth,
    /// `{...agent.describe(), ...agent.status()}`.
    pub backend: serde_json::Map<String, serde_json::Value>,
    /// **VIA's own**, appended after the twenty-six contract keys: what each
    /// [`SessionMode`] actually runs as here.
    pub session_modes: Vec<SessionModeHealth>,
    /// **VIA's own**, appended after `sessionModes`: the on-device realtime
    /// path's note — which mode is mounted, whether it is configured, and
    /// whether it has ever run on real hardware
    /// (`via_realtime_local::LocalHealth`, serialized). `null` when the
    /// composition root registered no `via-realtime-local` provider.
    pub local_realtime: serde_json::Value,
}

/// Everything the payload needs that is not a service handle.
#[derive(Debug, Clone)]
pub struct HealthInputs {
    /// The lease's instance id.
    pub gateway_instance_id: Option<String>,
    /// When this instance started.
    pub gateway_started_at: Option<String>,
    /// `config.announceIntoContext`.
    pub announce_into_context: bool,
    /// `config.resultContextMaxChars`.
    pub result_context_max_chars: i64,
    /// `config.announcementBatchMs`.
    pub announcement_batch_ms: i64,
    /// `config.announcementQuietMs`.
    pub announcement_quiet_ms: i64,
    /// `config.identityMode`.
    pub identity_mode: IdentityMode,
    /// Whether a harness is configured, for the `sessionModes` block.
    pub harness_configured: bool,
    /// [`Services::local_realtime`](crate::Services::local_realtime), verbatim.
    pub local_realtime: serde_json::Value,
}

/// The live state the payload is assembled from.
///
/// One struct rather than six positional arguments, because six health objects
/// of five different types in a row is exactly the signature a caller gets
/// wrong silently.
#[derive(Debug, Clone)]
pub struct HealthSnapshot {
    /// The input-arbitration status.
    pub input_suspension: SuspensionStatus,
    /// `USER.md` / `MEMORY.md` persistence health.
    pub frontend_memory: via_conversation::memory_service::MemoryServiceHealth,
    /// `notes.json` persistence health.
    pub notes: via_conversation::NotesHealth,
    /// `tasks.json` persistence health.
    pub task_store: StoreHealth,
    /// The connected voice clients.
    pub voice_clients: VoiceClientsHealth,
}

/// Assemble the payload.
///
/// The argument order is upstream's own read order
/// (`gateway-application.mjs:225-268`): the backend first, then the realtime
/// description, then the live per-request state.
#[must_use]
pub fn health_payload(
    inputs: &HealthInputs,
    realtime: RealtimeHealth,
    snapshot: HealthSnapshot,
    backend_description: &BackendDescription,
    backend_status: &HarnessHealth,
) -> HealthPayload {
    let HealthSnapshot {
        input_suspension,
        frontend_memory,
        notes,
        task_store,
        voice_clients,
    } = snapshot;
    // `{...describe(), ...status()}` — a JavaScript spread, so a key present in
    // both takes the *status* value and keeps the *describe* position.
    let mut backend = backend_description.fields().clone();
    if let Ok(serde_json::Value::Object(status)) = serde_json::to_value(backend_status) {
        for (key, value) in status {
            backend.insert(key, value);
        }
    }

    HealthPayload {
        ok: true,
        status: HEALTH_STATUS_READY,
        protocol_version: GATEWAY_PROTOCOL_VERSION,
        capabilities: GATEWAY_CAPABILITIES,
        gateway_instance_id: inputs.gateway_instance_id.clone(),
        gateway_started_at: inputs.gateway_started_at.clone(),
        input_suspension,
        voice_configured: realtime.configured,
        realtime_provider: realtime.provider,
        realtime_label: realtime.label,
        realtime_model: realtime.model,
        realtime_model_profile: realtime.model_profile,
        realtime_model_catalog: realtime.model_catalog,
        realtime_input_sample_rate: realtime.input_sample_rate,
        realtime_configuration_signature: realtime.configuration_signature,
        realtime_providers: realtime.providers,
        announce_into_context: inputs.announce_into_context,
        result_context_max_chars: inputs.result_context_max_chars,
        announcement_batch_ms: inputs.announcement_batch_ms,
        announcement_quiet_ms: inputs.announcement_quiet_ms,
        frontend_memory,
        notes,
        task_store,
        identity_mode: inputs.identity_mode.as_str(),
        voice_clients,
        backend,
        session_modes: SessionModeHealth::all(inputs.harness_configured),
        local_realtime: inputs.local_realtime.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn inputs() -> HealthInputs {
        HealthInputs {
            gateway_instance_id: Some("instance-1".to_owned()),
            gateway_started_at: Some("2026-08-22T00:00:00.000Z".to_owned()),
            announce_into_context: true,
            result_context_max_chars: 4000,
            announcement_batch_ms: 1200,
            announcement_quiet_ms: 600,
            identity_mode: IdentityMode::Personal,
            harness_configured: false,
            local_realtime: serde_json::Value::Null,
        }
    }

    fn realtime() -> RealtimeHealth {
        RealtimeHealth {
            configured: true,
            provider: "mock".to_owned(),
            label: "Mock".to_owned(),
            model: None,
            model_profile: None,
            model_catalog: Vec::new(),
            input_sample_rate: 16_000,
            configuration_signature: "0".repeat(64),
            providers: Vec::new(),
        }
    }

    fn payload() -> HealthPayload {
        payload_with(inputs())
    }

    fn payload_with(inputs: HealthInputs) -> HealthPayload {
        health_payload(
            &inputs,
            realtime(),
            HealthSnapshot {
                input_suspension: SuspensionStatus::default(),
                frontend_memory: via_conversation::memory_service::FrontendMemoryService::new(
                    None, None,
                )
                .health(),
                notes: via_conversation::FrontendNotesStore::in_memory().health(),
                task_store: via_store::VersionedJsonStore::builder().build().health(),
                voice_clients: VoiceClientsHealth::default(),
            },
            &BackendDescription::frontend_only(via_i18n::Locale::En),
            &HarnessHealth::not_configured(),
        )
    }

    #[test]
    fn the_twenty_six_contract_keys_come_first_and_in_order() {
        let value = serde_json::to_value(payload()).expect("serializes");
        let object = value.as_object().expect("an object");
        let keys: Vec<&str> = object.keys().map(String::as_str).collect();
        assert_eq!(
            keys[..HEALTH_FIELD_ORDER.len()],
            HEALTH_FIELD_ORDER,
            "the /api/health field order is an external contract"
        );
        assert_eq!(
            &keys[HEALTH_FIELD_ORDER.len()..],
            ["sessionModes", "localRealtime"],
            "VIA's own additions go after the contract keys, never inside them"
        );
    }

    #[test]
    fn local_realtime_carries_whatever_the_composition_root_serialized() {
        let mut inputs = inputs();
        inputs.local_realtime = serde_json::json!({
            "mode": "local-omni:pipeline",
            "configured": false,
            "verified": true,
        });
        let value = serde_json::to_value(payload_with(inputs)).expect("serializes");
        assert_eq!(value["localRealtime"]["mode"], "local-omni:pipeline");
        assert_eq!(value["localRealtime"]["configured"], false);
    }

    #[test]
    fn local_realtime_is_null_when_no_on_device_provider_was_registered() {
        let value = serde_json::to_value(payload()).expect("serializes");
        assert_eq!(value["localRealtime"], serde_json::Value::Null);
    }

    #[test]
    fn the_backend_block_is_describe_spread_by_status() {
        let value = serde_json::to_value(payload()).expect("serializes");
        let backend = value["backend"].as_object().expect("an object");
        // `describe()` supplies `status: 'not_configured'`; `status()` spreads
        // over it with `HarnessStatus::NotConfigured`, whose wire spelling is
        // the same string — so the observable result is upstream's.
        assert_eq!(backend["status"], "not_configured");
        assert_eq!(backend["code"], "NOT_CONFIGURED");
        assert_eq!(backend["ok"], true);
        assert_eq!(backend["enabled"], false);
        let keys: Vec<&str> = backend.keys().map(String::as_str).collect();
        assert_eq!(
            keys[0], "enabled",
            "a spread keeps the left object's key positions"
        );
    }

    #[test]
    fn a_gateway_without_a_harness_says_so_for_every_mode() {
        let value = serde_json::to_value(payload()).expect("serializes");
        let modes = value["sessionModes"].as_array().expect("an array");
        assert_eq!(modes.len(), 4);
        let agent = modes
            .iter()
            .find(|mode| mode["requested"] == "agent")
            .expect("the agent row");
        assert_eq!(agent["effective"], "direct");
        assert_eq!(agent["reason"], "no_harness");
        let interface = modes
            .iter()
            .find(|mode| mode["requested"] == "interface")
            .expect("the interface row");
        assert_eq!(interface["effective"], "interface");
        assert_eq!(interface["reason"], serde_json::Value::Null);
        let dictation = modes
            .iter()
            .find(|mode| mode["requested"] == "dictation")
            .expect("the dictation row");
        assert_eq!(dictation["reason"], serde_json::Value::Null);
    }

    #[test]
    fn interface_reports_itself_now_that_the_context_engine_ships() {
        let modes = SessionModeHealth::all(true);
        let agent = modes
            .iter()
            .find(|mode| mode.requested == "agent")
            .expect("the agent row");
        assert_eq!(agent.effective, "agent");
        assert_eq!(agent.reason, None);
        let interface = modes
            .iter()
            .find(|mode| mode.requested == "interface")
            .expect("the interface row");
        assert_eq!(interface.effective, "interface");
        assert_eq!(interface.reason, None);
    }

    #[test]
    fn the_two_probes_are_the_catalogued_literals() {
        assert_eq!(
            serde_json::to_value(LivenessProbe::default()).expect("serializes"),
            serde_json::json!({ "ok": true, "status": "live" }),
        );
        assert_eq!(
            serde_json::to_value(ReadinessProbe::default()).expect("serializes"),
            serde_json::json!({ "ok": true, "status": "ready" }),
        );
    }
}
