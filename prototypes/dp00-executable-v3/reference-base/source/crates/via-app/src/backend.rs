//! The Gateway's view of Layer 3, as one injectable seam.
//!
//! Upstream's composition root reaches a module-level singleton, `agent`
//! (`server/src/agent/agent-client.mjs:155-206`), and asks it four questions
//! from inside the HTTP handlers: [`describe`](GatewayBackend::describe),
//! [`status`](GatewayBackend::status), [`ui_url`](GatewayBackend::ui_url) and
//! [`respond_permission`](GatewayBackend::respond_permission). That singleton
//! is exactly what makes `gateway-application.mjs` hard to test, which is why
//! the factory takes `agent` by injection in the first place
//! (`server/test/gateway-application.test.mjs:33`).
//!
//! Here it is a trait with two shipped implementations:
//!
//! | | When | Reproduces |
//! | --- | --- | --- |
//! | [`FrontendOnlyBackend`] | `AGENT_PROTOCOL` unset or `none` | `agent-client.mjs:164-189`'s literal fallback objects |
//! | [`HarnessBackend`] | a harness is configured | `agent-client.mjs:163-186`'s `requireAgent()` arm |
//!
//! # Why `describe` returns a `Value`
//!
//! `/api/health.backend` is `{...agent.describe(), ...agent.status()}` — a
//! *spread* of two objects whose key sets are the driver's, not the Gateway's
//! (`gateway-application.mjs:264-267`). A struct here would freeze a shape that
//! upstream deliberately leaves to the adapter, and `docs/rebrand.md` keeps the
//! adapter keys (`kind`, `baseUrl`, `acpConnection`, `sessionModel`, …)
//! verbatim. The one key the Gateway itself reads is
//! `capabilities.backendUi`, and [`BackendDescription::backend_ui`] is that
//! read expressed once.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Map, Value, json};
use via_downstream::{HarnessDescriptor, HarnessHealth};
use via_i18n::{Locale, keys, t};
use via_voice::PermissionDecision;

/// `agent.describe()` — the static half of `/api/health.backend`.
///
/// **External contract** — `server/src/agent/acp-backend-adapter.mjs:254-272`
/// for the configured shape, `agent-client.mjs:164-176` for the unconfigured
/// one. The map is ordered (`serde_json/preserve_order`), because the two
/// objects are spread into one and the reader sees insertion order.
#[derive(Clone, Debug, PartialEq)]
pub struct BackendDescription {
    fields: Map<String, Value>,
}

impl BackendDescription {
    /// Wrap a driver's own object.
    #[must_use]
    pub const fn new(fields: Map<String, Value>) -> Self {
        Self { fields }
    }

    /// `agent.describe()` when `config.agentProtocol` is falsy.
    ///
    /// **External contract** — `agent-client.mjs:164-176`, key for key. The
    /// label is upstream's `'仅前台聊天'`, which `docs/rebrand.md` classifies as
    /// product prose and `via-i18n` therefore owns.
    #[must_use]
    pub fn frontend_only(locale: Locale) -> Self {
        let mut fields = Map::new();
        fields.insert("enabled".to_owned(), Value::Bool(false));
        fields.insert("protocol".to_owned(), Value::Null);
        fields.insert("kind".to_owned(), Value::Null);
        fields.insert(
            "label".to_owned(),
            Value::String(t(locale, keys::GATEWAY_FRONTEND_ONLY_LABEL).to_owned()),
        );
        fields.insert("status".to_owned(), Value::String("not_configured".into()));
        fields.insert("capabilities".to_owned(), json!({ "backendUi": false }));
        Self { fields }
    }

    /// The description a validated [`HarnessDescriptor`] produces.
    ///
    /// The seven capability flags are the descriptor's; `backendUi` is the one
    /// the Gateway itself branches on
    /// (`gateway-application.mjs:299`).
    #[must_use]
    pub fn from_descriptor(descriptor: &HarnessDescriptor) -> Self {
        let capabilities = descriptor.capabilities();
        let mut fields = Map::new();
        fields.insert("enabled".to_owned(), Value::Bool(true));
        fields.insert(
            "protocol".to_owned(),
            Value::String(descriptor.id().to_owned()),
        );
        fields.insert("kind".to_owned(), Value::String(descriptor.id().to_owned()));
        fields.insert(
            "label".to_owned(),
            Value::String(descriptor.label().to_owned()),
        );
        fields.insert(
            "capabilities".to_owned(),
            serde_json::to_value(capabilities).unwrap_or(Value::Null),
        );
        Self { fields }
    }

    /// The raw object, as it is spread into `/api/health.backend`.
    #[must_use]
    pub const fn fields(&self) -> &Map<String, Value> {
        &self.fields
    }

    /// `describe().capabilities.backendUi`.
    ///
    /// **External contract** — `gateway-application.mjs:299`. A missing or
    /// non-boolean value reads as `false`, which is JavaScript's own answer for
    /// `!undefined`.
    #[must_use]
    pub fn backend_ui(&self) -> bool {
        self.fields
            .get("capabilities")
            .and_then(Value::as_object)
            .and_then(|capabilities| capabilities.get("backendUi"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
}

/// Resolves the harness's own web address for one owner.
///
/// A named type rather than an inline closure so `GET /api/backend/ui`'s one
/// injection point reads as what it is.
pub type UiUrlResolver = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// Supplies the live runtime state `agent.status()` reads.
pub type HealthSource = Arc<dyn Fn() -> HarnessHealth + Send + Sync>;

/// Why a permission relay failed.
///
/// The route maps [`NotFound`](PermissionRelayError::NotFound) to a 404 and
/// everything else to the terminal error handler, reproducing
/// `gateway-application.mjs:399-402`'s `error?.status === 404` branch.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PermissionRelayError {
    /// The id is unknown, expired, or belongs to another owner.
    #[error("{0}")]
    NotFound(String),
    /// Anything else the harness reported.
    #[error("{0}")]
    Failed(String),
}

/// The four questions the Gateway asks Layer 3.
///
/// See the module documentation for why this is a trait rather than a direct
/// dependency on a harness.
#[async_trait]
pub trait GatewayBackend: Send + Sync + std::fmt::Debug {
    /// Whether a harness is configured at all — upstream's `agent.enabled`
    /// (`agent-client.mjs:155-157`), which is `Boolean(config.agentProtocol)`.
    fn enabled(&self) -> bool;

    /// `agent.describe()`.
    fn describe(&self) -> BackendDescription;

    /// `agent.status()` — the cached runtime state, answered synchronously.
    fn status(&self) -> HarnessHealth;

    /// `agent.health()` — a live probe that may start the harness.
    async fn health(&self) -> HarnessHealth {
        self.status()
    }

    /// `agent.uiUrl({ownerId})`.
    ///
    /// `None` is the 404 branch; upstream's `?.` makes a driver without the
    /// method resolve `null`, which is the default here.
    async fn ui_url(&self, _owner_id: &str) -> Result<Option<String>, PermissionRelayError> {
        Ok(None)
    }

    /// `agent.respondPermission(id, decision, {ownerId})`.
    ///
    /// # Errors
    ///
    /// [`PermissionRelayError::NotFound`] for an unknown, expired or foreign
    /// id — the route turns it into a 404 with the message as the body.
    async fn respond_permission(
        &self,
        authorization_id: &str,
        decision: PermissionDecision,
        owner_id: &str,
    ) -> Result<Value, PermissionRelayError>;
}

/// The `AGENT_PROTOCOL=none` Gateway: voice chat, no delegation.
///
/// `docs/architecture.md` §2 — *"`agent` degrades to `direct` when no harness
/// is configured, rather than failing. […] The degradation is reported on
/// `/api/health`, never silent."* This is the object that makes that true.
#[derive(Clone, Copy, Debug)]
pub struct FrontendOnlyBackend {
    locale: Locale,
}

impl FrontendOnlyBackend {
    /// A frontend-only backend that speaks `locale`.
    #[must_use]
    pub const fn new(locale: Locale) -> Self {
        Self { locale }
    }
}

impl Default for FrontendOnlyBackend {
    fn default() -> Self {
        Self::new(Locale::En)
    }
}

#[async_trait]
impl GatewayBackend for FrontendOnlyBackend {
    fn enabled(&self) -> bool {
        false
    }

    fn describe(&self) -> BackendDescription {
        BackendDescription::frontend_only(self.locale)
    }

    /// **External contract** — `agent-client.mjs:183-189`:
    /// `{enabled:false, ok:true, status:'not_configured', code:'NOT_CONFIGURED'}`.
    /// Note `ok: true`: a Gateway with no backend is healthy, not broken.
    fn status(&self) -> HarnessHealth {
        HarnessHealth::not_configured()
    }

    async fn respond_permission(
        &self,
        _authorization_id: &str,
        _decision: PermissionDecision,
        _owner_id: &str,
    ) -> Result<Value, PermissionRelayError> {
        Err(PermissionRelayError::NotFound(
            t(self.locale, keys::GATEWAY_PERMISSION_REQUEST_UNKNOWN).to_owned(),
        ))
    }
}

/// A configured harness, seen through `via-coordinator`'s permission broker.
///
/// The descriptor supplies `describe()`; the broker supplies the permission
/// relay. Both are Layer 3 facts that Layer 1 may not reach
/// (`docs/architecture.md` §9), which is why the bridge lives here.
#[derive(Clone)]
pub struct HarnessBackend {
    descriptor: HarnessDescriptor,
    health: HealthSource,
    broker: Arc<via_coordinator::PermissionBroker>,
    ui_url: Option<UiUrlResolver>,
    locale: Locale,
}

impl std::fmt::Debug for HarnessBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HarnessBackend")
            .field("descriptor", &self.descriptor.id())
            .field("has_ui_url", &self.ui_url.is_some())
            .finish()
    }
}

impl HarnessBackend {
    /// Bridge `descriptor` and `broker` into the Gateway's backend seam.
    #[must_use]
    pub fn new(
        descriptor: HarnessDescriptor,
        broker: Arc<via_coordinator::PermissionBroker>,
        locale: Locale,
    ) -> Self {
        Self {
            descriptor,
            health: Arc::new(HarnessHealth::not_started),
            broker,
            ui_url: None,
            locale,
        }
    }

    /// Supply the live runtime state that `agent.status()` reads.
    #[must_use]
    pub fn health_source(mut self, health: Arc<dyn Fn() -> HarnessHealth + Send + Sync>) -> Self {
        self.health = health;
        self
    }

    /// Supply the backend's own web address resolver.
    ///
    /// Absent, `GET /api/backend/ui` answers 404 even when the descriptor
    /// declares `backendUi` — upstream's second 404 branch
    /// (`gateway-application.mjs:303-307`), whose body is byte-identical to the
    /// first.
    #[must_use]
    pub fn ui_url_source(mut self, resolve: UiUrlResolver) -> Self {
        self.ui_url = Some(resolve);
        self
    }
}

#[async_trait]
impl GatewayBackend for HarnessBackend {
    fn enabled(&self) -> bool {
        true
    }

    fn describe(&self) -> BackendDescription {
        BackendDescription::from_descriptor(&self.descriptor)
    }

    fn status(&self) -> HarnessHealth {
        (self.health)()
    }

    async fn ui_url(&self, owner_id: &str) -> Result<Option<String>, PermissionRelayError> {
        Ok(self.ui_url.as_ref().and_then(|resolve| resolve(owner_id)))
    }

    async fn respond_permission(
        &self,
        authorization_id: &str,
        decision: PermissionDecision,
        owner_id: &str,
    ) -> Result<Value, PermissionRelayError> {
        let response = match decision {
            PermissionDecision::Always => via_coordinator::PermissionResponse::Always,
            PermissionDecision::Reject => via_coordinator::PermissionResponse::Reject,
        };
        // Upstream maps `error?.status === 404` to a 404 and everything else to
        // `next(error)` (`gateway-application.mjs:399-402`). The broker's one
        // refusal is the 404; a stopped broker is the 500.
        let resolved = self
            .broker
            .respond(authorization_id, response, owner_id)
            .await
            .map_err(|error| match error {
                via_coordinator::CoordinatorError::PermissionRequestUnknown => {
                    PermissionRelayError::NotFound(
                        t(self.locale, keys::GATEWAY_PERMISSION_REQUEST_UNKNOWN).to_owned(),
                    )
                }
                other => PermissionRelayError::Failed(other.to_string()),
            })?;
        serde_json::to_value(&resolved)
            .map_err(|error| PermissionRelayError::Failed(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_unconfigured_description_is_upstreams_literal_object() {
        let description = BackendDescription::frontend_only(Locale::Zh);
        let keys: Vec<&str> = description.fields().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "enabled",
                "protocol",
                "kind",
                "label",
                "status",
                "capabilities"
            ],
            "key order is spread into /api/health.backend"
        );
        assert_eq!(description.fields()["status"], "not_configured");
        assert_eq!(description.fields()["label"], "仅前台聊天");
        assert!(!description.backend_ui());
    }

    #[test]
    fn a_missing_capability_block_reads_as_no_backend_ui() {
        let description = BackendDescription::new(Map::new());
        assert!(!description.backend_ui());
        let description = BackendDescription::new(
            json!({ "capabilities": { "backendUi": "yes" } })
                .as_object()
                .cloned()
                .expect("object"),
        );
        assert!(
            !description.backend_ui(),
            "a non-boolean is not a capability"
        );
    }

    #[tokio::test]
    async fn the_frontend_only_backend_is_healthy_and_has_no_web_address() {
        let backend = FrontendOnlyBackend::new(Locale::En);
        assert!(!backend.enabled());
        let status = backend.status();
        assert!(status.is_ok(), "no backend configured is not a failure");
        assert_eq!(backend.ui_url("user_1").await.expect("resolves"), None);
        let refusal = backend
            .respond_permission("perm_1", PermissionDecision::Always, "user_1")
            .await
            .expect_err("nothing to respond to");
        assert!(matches!(refusal, PermissionRelayError::NotFound(_)));
    }
}
