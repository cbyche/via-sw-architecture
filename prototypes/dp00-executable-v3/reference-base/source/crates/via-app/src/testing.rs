//! Deterministic doubles for the composition root.
//!
//! Upstream's own Gateway test constructs an application with an isolated task
//! manager, an isolated coordinator, a stand-in input-asset registry and a
//! **private** realtime provider that is `gateway-only`, then asserts the
//! private provider is active and is *not* in `/api/health.realtimeProviders`
//! (`server/test/gateway-application.test.mjs`). Everything that test needs
//! lives here so `via-e2e` and `via-conformance` do not each build their own.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use via_core::config::{Overrides, resolve};
use via_core::{Config, EnvMap, IdentityManager};
use via_downstream::HarnessHealth;
use via_realtime::{RealtimeProviderRegistry, Visibility, testing::TestProvider};
use via_voice::PermissionDecision;

use crate::backend::{BackendDescription, GatewayBackend, PermissionRelayError};
use crate::services::Services;

/// The provider key the test registry serves under.
pub const TEST_PROVIDER: &str = "mock";

/// The `gateway-only` provider `server/test/gateway-application.test.mjs`
/// registers to prove `realtimeProviders` filters it out.
pub const PRIVATE_PROVIDER: &str = "private-realtime";

/// A configuration with a long-enough auth secret and an ephemeral port.
///
/// The secret is not a credential — it is 64 `a`s — but it is long enough for
/// [`IdentityManager::new`], which is the only thing the tests need from it.
#[must_use]
pub fn test_config() -> Config {
    let env: EnvMap = [
        ("DASHSCOPE_API_KEY", "sk-test"),
        ("VIA_AUTH_SECRET", &"a".repeat(64)),
        ("PORT", "0"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect();
    let overrides = Overrides {
        home_directory: std::env::temp_dir().join("via-app-tests"),
        working_directory: std::env::temp_dir().join("via-app-tests"),
        ..Overrides::default()
    };
    resolve(&env, None, &overrides).unwrap_or_default()
}

/// A registry with one public provider and one `gateway-only` provider.
#[must_use]
pub fn test_registry() -> RealtimeProviderRegistry {
    let mut registry = RealtimeProviderRegistry::with_default(TEST_PROVIDER);
    let _ = registry.register(Arc::new(TestProvider::new(TEST_PROVIDER)));
    let _ = registry.register(Arc::new(
        TestProvider::new(PRIVATE_PROVIDER).with_visibility(Visibility::GatewayOnly),
    ));
    registry
}

/// Services with every default, an in-memory Work queue and the test registry.
#[must_use]
pub fn test_services() -> Services {
    let config = test_config();
    let identity = IdentityManager::new(
        &"a".repeat(64),
        config.identity_mode,
        &config.personal_owner_id,
    )
    .expect("a 64-character secret clears the 32-character minimum");
    Services::builder(config)
        .identity(Arc::new(identity))
        .realtime_registry(Arc::new(test_registry()))
        .realtime_provider(TEST_PROVIDER)
        .logger(Arc::new(via_log::Logger::with_sinks(
            via_log::LoggerOptions::detached("gateway"),
            Vec::new(),
        )))
        .build()
        .expect("every default is constructible from the test configuration")
}

/// A backend that answers whatever it was constructed with.
#[derive(Debug, Clone)]
pub struct ScriptedBackend {
    enabled: bool,
    description: BackendDescription,
    status: HarnessHealth,
    ui_url: Option<String>,
    permission: Option<Value>,
}

impl ScriptedBackend {
    /// A configured backend that declares `backendUi`.
    #[must_use]
    pub fn with_web_ui(url: &str) -> Self {
        let mut fields = serde_json::Map::new();
        fields.insert("enabled".to_owned(), Value::Bool(true));
        fields.insert("protocol".to_owned(), Value::String("opencode".into()));
        fields.insert(
            "capabilities".to_owned(),
            serde_json::json!({ "backendUi": true }),
        );
        Self {
            enabled: true,
            description: BackendDescription::new(fields),
            status: HarnessHealth::ready(),
            ui_url: Some(url.to_owned()),
            permission: Some(serde_json::json!({
                "id": "auth_1",
                "status": "approved",
                "category": "shell",
                "summary": "ls",
            })),
        }
    }

    /// A configured backend that declares `backendUi` but resolves no URL —
    /// upstream's second 404 branch.
    #[must_use]
    pub fn with_unresolvable_web_ui() -> Self {
        Self {
            ui_url: None,
            ..Self::with_web_ui("http://127.0.0.1:1/ui")
        }
    }

    /// A configured backend that declares no web UI at all.
    #[must_use]
    pub fn without_web_ui() -> Self {
        let mut fields = serde_json::Map::new();
        fields.insert("enabled".to_owned(), Value::Bool(true));
        fields.insert(
            "capabilities".to_owned(),
            serde_json::json!({ "backendUi": false }),
        );
        Self {
            enabled: true,
            description: BackendDescription::new(fields),
            status: HarnessHealth::ready(),
            ui_url: None,
            permission: None,
        }
    }

    /// Refuse every permission relay with a 404-shaped error.
    #[must_use]
    pub fn refusing_permissions(mut self) -> Self {
        self.permission = None;
        self
    }
}

#[async_trait]
impl GatewayBackend for ScriptedBackend {
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn describe(&self) -> BackendDescription {
        self.description.clone()
    }

    fn status(&self) -> HarnessHealth {
        self.status.clone()
    }

    async fn ui_url(&self, _owner_id: &str) -> Result<Option<String>, PermissionRelayError> {
        Ok(self.ui_url.clone())
    }

    async fn respond_permission(
        &self,
        _authorization_id: &str,
        _decision: PermissionDecision,
        _owner_id: &str,
    ) -> Result<Value, PermissionRelayError> {
        self.permission
            .clone()
            .ok_or_else(|| PermissionRelayError::NotFound("not found".to_owned()))
    }
}
