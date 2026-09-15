//! Which providers exist, what they are called, and what `/api/health` says
//! about them.
//!
//! Ported from the registry half of
//! `server/src/voice/providers/provider-registry.mjs` plus the whole of
//! `server/src/voice/providers/registry.mjs`.
//!
//! Three things here are external contract rather than convenience:
//!
//! **Alias resolution.** `qwen` → `dashscope` and `s2s` → `speech-to-speech`, both
//! **KEEP** under `docs/rebrand.md`: `qwen` names the vendor's realtime runtime
//! rather than VIA, clients already send it on the `connect` frame, and users
//! already have it in `config.env`. Lookup is `trim().toLowerCase()`, so
//! `resolve("QWEN")` works.
//!
//! **A name is claimed once.** A second provider that wants a key or alias
//! another already answers to is refused at registration — not silently
//! shadowed — because the alias table is what a client's provider selection is
//! resolved through.
//!
//! **The descriptor's field order.** [`ActiveRealtime`] is published on
//! `/api/health`, and its field order is upstream's object-literal order
//! (`providers/registry.mjs:47-70`). `via-catalog` already owns the hash under
//! `configurationSignature`; what this crate owns is the shape around it.

use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use via_catalog::{ModelCapabilities, ModelProfile, TransportCapabilities};

use crate::error::RealtimeError;
use crate::provider::{RealtimeProvider, Visibility, clean_key, validate_realtime_provider};

/// How [`RealtimeProviderRegistry::list`] filters.
///
/// Upstream `list({ includeGatewayOnly = false, configuredOnly = false })`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListOptions {
    /// Include providers whose visibility is
    /// [`GatewayOnly`](Visibility::GatewayOnly).
    pub include_gateway_only: bool,
    /// Exclude providers that are not configured.
    pub configured_only: bool,
}

/// One entry in `/api/health.realtimeProviders`.
///
/// External contract — `providers/registry.mjs:26-37`. Five fields, in this
/// order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDescriptor {
    /// The canonical key.
    pub key: String,
    /// The human label.
    pub label: String,
    /// The active model id, or `null`.
    pub model: Option<String>,
    /// Every model this provider can be pointed at, or `null` when it has no
    /// catalog of its own.
    ///
    /// `null` rather than `[]` is deliberate and is upstream's:
    /// `provider.modelCatalog?.().map(…) ?? null`. A provider whose model is
    /// chosen outside VIA — `speech-to-speech` — publishes `null`, and a client
    /// renders that as "no model picker" rather than as "an empty picker".
    pub realtime_model_ids: Option<Vec<String>>,
    /// Whether its credentials and endpoint are present.
    pub configured: bool,
}

/// What `/api/health` says about the active realtime frontend.
///
/// External contract — `json-field` / *describeActiveRealtime response shape*
/// (`providers/registry.mjs:47-70`). Twelve fields, in this order; consumed by
/// the desktop settings window and by every client's configuration-drift check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRealtime {
    /// The resolved provider key.
    pub provider: String,
    /// Its label.
    pub label: String,
    /// The active model id, or `null`.
    pub model: Option<String>,
    /// The active model's profile, or `null`.
    pub model_profile: Option<ModelProfile>,
    /// The active model's capabilities, or `null`. Lifted out of the profile
    /// because upstream publishes it twice and clients read the flat copy.
    pub model_capabilities: Option<ModelCapabilities>,
    /// The transport's capabilities, or `null`.
    pub transport_capabilities: Option<TransportCapabilities>,
    /// Every model this provider can be pointed at. `[]`, never `null`.
    pub model_catalog: Vec<ModelProfile>,
    /// The active voice id, or `null`.
    pub voice: Option<String>,
    /// The rate the client must capture at.
    pub input_sample_rate: u32,
    /// Whether the provider is configured.
    pub configured: bool,
    /// `sha256` of the configuration identity — `via-catalog` owns the input and
    /// its key order.
    pub configuration_signature: String,
    /// The configured, publicly visible providers.
    pub providers: Vec<ProviderDescriptor>,
}

/// The providers a Gateway knows about.
///
/// Registration order is preserved: [`list`](Self::list) returns it, and the
/// unsupported-provider message lists it.
#[derive(Default)]
pub struct RealtimeProviderRegistry {
    providers: IndexMap<String, Arc<dyn RealtimeProvider>>,
    aliases: HashMap<String, String>,
    default_provider: String,
}

impl core::fmt::Debug for RealtimeProviderRegistry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RealtimeProviderRegistry")
            .field("providers", &self.provider_keys())
            .field("default_provider", &self.default_provider)
            .finish()
    }
}

impl RealtimeProviderRegistry {
    /// An empty registry with no default.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty registry whose default provider is `default_provider`.
    ///
    /// The default is cleaned but **not** checked: upstream lets a registry be
    /// constructed with a default it does not have yet, so providers can be
    /// registered afterwards, and `resolve()` is where the miss is reported.
    #[must_use]
    pub fn with_default(default_provider: &str) -> Self {
        Self {
            default_provider: clean_key(default_provider),
            ..Self::default()
        }
    }

    /// Register a provider, after validating it.
    ///
    /// # Errors
    ///
    /// Whatever [`validate_realtime_provider`] refuses, or
    /// [`RealtimeError::ProviderNameTaken`] when the key or one of the aliases
    /// is already claimed.
    pub fn register(
        &mut self,
        provider: Arc<dyn RealtimeProvider>,
    ) -> Result<Arc<dyn RealtimeProvider>, RealtimeError> {
        validate_realtime_provider(provider.as_ref())?;
        let key = clean_key(provider.key());
        let mut names = vec![key.clone()];
        names.extend(provider.aliases().iter().map(|alias| clean_key(alias)));
        // Every name is checked *before* any is claimed, so a refused
        // registration leaves the table exactly as it was.
        for name in &names {
            if self.aliases.contains_key(name) {
                return Err(RealtimeError::ProviderNameTaken { name: name.clone() });
            }
        }
        self.providers.insert(key.clone(), Arc::clone(&provider));
        for name in names {
            self.aliases.insert(name, key.clone());
        }
        Ok(provider)
    }

    /// The provider a key or alias names, or the default when `requested` is
    /// `None` or empty.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::UnsupportedProvider`], carrying every registered key.
    pub fn resolve(
        &self,
        requested: Option<&str>,
    ) -> Result<Arc<dyn RealtimeProvider>, RealtimeError> {
        let requested = requested.unwrap_or_default();
        let name = if clean_key(requested).is_empty() {
            self.default_provider.clone()
        } else {
            clean_key(requested)
        };
        self.aliases
            .get(&name)
            .and_then(|key| self.providers.get(key))
            .map(Arc::clone)
            .ok_or_else(|| RealtimeError::UnsupportedProvider {
                requested: if name.is_empty() {
                    clean_key(requested)
                } else {
                    name
                },
                available: self
                    .provider_keys()
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            })
    }

    /// The providers a filter admits, in registration order.
    #[must_use]
    pub fn list(&self, options: ListOptions) -> Vec<Arc<dyn RealtimeProvider>> {
        self.providers
            .values()
            .filter(|provider| {
                (options.include_gateway_only || provider.visibility() == Visibility::Public)
                    && (!options.configured_only || provider.is_configured())
            })
            .map(Arc::clone)
            .collect()
    }

    /// Registered keys, in registration order. Aliases are not keys.
    #[must_use]
    pub fn provider_keys(&self) -> Vec<&str> {
        self.providers.keys().map(String::as_str).collect()
    }

    /// Every name — key or alias — that resolves, sorted.
    #[must_use]
    pub fn resolvable_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.aliases.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// The key `resolve(None)` would answer with.
    #[must_use]
    pub fn default_provider(&self) -> &str {
        &self.default_provider
    }

    /// How many providers are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Whether nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// The configured providers a client may choose between.
    ///
    /// Upstream `listRealtimeProviders`, which always filters
    /// `configuredOnly: true` — an unconfigured provider is invisible in a
    /// picker rather than a choice that fails.
    #[must_use]
    pub fn describe_providers(&self, include_gateway_only: bool) -> Vec<ProviderDescriptor> {
        self.list(ListOptions {
            include_gateway_only,
            configured_only: true,
        })
        .iter()
        .map(|provider| describe_provider(provider.as_ref()))
        .collect()
    }

    /// The `/api/health` realtime block.
    ///
    /// Upstream `describeActiveRealtime(requested)`.
    ///
    /// # Errors
    ///
    /// [`RealtimeError::UnsupportedProvider`] when `requested` names nothing.
    pub fn describe_active_realtime(
        &self,
        requested: Option<&str>,
    ) -> Result<ActiveRealtime, RealtimeError> {
        let provider = self.resolve(requested)?;
        let profile = provider.model_profile();
        Ok(ActiveRealtime {
            provider: provider.key().to_owned(),
            label: provider.label().to_owned(),
            model: provider.model().map(str::to_owned),
            model_capabilities: profile.as_ref().map(|profile| profile.model_capabilities),
            transport_capabilities: profile
                .as_ref()
                .map(|profile| profile.transport_capabilities),
            model_profile: profile,
            model_catalog: provider.model_catalog().to_vec(),
            voice: provider.voice().map(str::to_owned),
            input_sample_rate: provider.input_sample_rate(),
            configured: provider.is_configured(),
            configuration_signature: provider.configuration_signature(),
            providers: self.describe_providers(false),
        })
    }
}

/// One provider's `/api/health` entry.
#[must_use]
pub fn describe_provider(provider: &dyn RealtimeProvider) -> ProviderDescriptor {
    let catalog = provider.model_catalog();
    ProviderDescriptor {
        key: provider.key().to_owned(),
        label: provider.label().to_owned(),
        model: provider.model().map(str::to_owned),
        realtime_model_ids: (!catalog.is_empty()).then(|| {
            catalog
                .iter()
                .map(|profile| profile.id.to_string())
                .collect()
        }),
        configured: provider.is_configured(),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::testing::TestProvider;

    fn registry() -> RealtimeProviderRegistry {
        RealtimeProviderRegistry::with_default("custom")
    }

    fn register(registry: &mut RealtimeProviderRegistry, provider: TestProvider) {
        registry
            .register(Arc::new(provider))
            .map(|_| ())
            .expect("registers");
    }

    fn keys(providers: &[Arc<dyn RealtimeProvider>]) -> Vec<String> {
        providers
            .iter()
            .map(|provider| provider.key().to_owned())
            .collect()
    }

    #[test]
    fn a_custom_provider_resolves_by_key_and_by_alias() {
        let mut registry = registry();
        register(
            &mut registry,
            TestProvider::new("custom-provider").with_aliases(["custom"]),
        );
        assert_eq!(
            registry.resolve(None).expect("default").key(),
            "custom-provider"
        );
        assert_eq!(
            registry.resolve(Some("CUSTOM")).expect("alias").key(),
            "custom-provider"
        );
        assert_eq!(
            registry
                .resolve(Some("  custom-provider  "))
                .expect("key")
                .key(),
            "custom-provider"
        );
    }

    #[test]
    fn a_claimed_name_cannot_be_claimed_twice() {
        let mut registry = registry();
        register(
            &mut registry,
            TestProvider::new("custom-provider").with_aliases(["custom"]),
        );
        let clash = registry.register(Arc::new(
            TestProvider::new("other").with_aliases(["custom"]),
        ));
        assert_eq!(
            clash.map(|_| ()),
            Err(RealtimeError::ProviderNameTaken {
                name: "custom".into()
            })
        );
        // The refused registration left nothing behind.
        assert_eq!(registry.provider_keys(), ["custom-provider"]);
        assert!(registry.resolve(Some("other")).is_err());
    }

    #[test]
    fn a_provider_key_that_clashes_with_an_alias_is_refused() {
        let mut registry = registry();
        register(&mut registry, TestProvider::new("a").with_aliases(["b"]));
        assert_eq!(
            registry
                .register(Arc::new(TestProvider::new("b")))
                .map(|_| ()),
            Err(RealtimeError::ProviderNameTaken { name: "b".into() })
        );
    }

    #[test]
    fn an_invalid_provider_never_reaches_the_table() {
        let mut registry = registry();
        assert!(
            registry
                .register(Arc::new(TestProvider::new("Bad")))
                .is_err()
        );
        assert!(registry.is_empty());
    }

    #[test]
    fn an_unknown_name_reports_what_is_available() {
        let mut registry = registry();
        register(&mut registry, TestProvider::new("dashscope"));
        register(&mut registry, TestProvider::new("speech-to-speech"));
        let error = registry.resolve(Some("qwen3")).expect_err("unknown");
        assert_eq!(
            error,
            RealtimeError::UnsupportedProvider {
                requested: "qwen3".into(),
                available: vec!["dashscope".into(), "speech-to-speech".into()],
            }
        );
    }

    #[test]
    fn an_empty_request_falls_back_to_the_default() {
        let mut registry = RealtimeProviderRegistry::with_default("dashscope");
        register(&mut registry, TestProvider::new("dashscope"));
        assert_eq!(
            registry.resolve(Some("")).expect("default").key(),
            "dashscope"
        );
        assert_eq!(
            registry.resolve(Some("   ")).expect("default").key(),
            "dashscope"
        );
        assert_eq!(registry.default_provider(), "dashscope");
    }

    #[test]
    fn a_registry_with_no_default_reports_the_empty_request_as_unknown() {
        let registry = RealtimeProviderRegistry::new();
        let error = registry.resolve(None).expect_err("no default");
        assert_eq!(
            error,
            RealtimeError::UnsupportedProvider {
                requested: String::new(),
                available: Vec::new(),
            }
        );
    }

    #[test]
    fn gateway_only_providers_resolve_but_are_never_listed() {
        let mut registry = registry();
        register(&mut registry, TestProvider::new("public-provider"));
        register(
            &mut registry,
            TestProvider::new("private-provider").with_visibility(Visibility::GatewayOnly),
        );

        assert_eq!(
            keys(&registry.list(ListOptions::default())),
            ["public-provider"]
        );
        assert_eq!(
            keys(&registry.list(ListOptions {
                include_gateway_only: true,
                ..ListOptions::default()
            })),
            ["public-provider", "private-provider"]
        );

        assert_eq!(
            registry
                .resolve(Some("private-provider"))
                .expect("resolves")
                .key(),
            "private-provider"
        );
    }

    #[test]
    fn an_unconfigured_provider_is_invisible_in_a_picker() {
        let mut registry = registry();
        register(&mut registry, TestProvider::new("ready"));
        register(
            &mut registry,
            TestProvider::new("unready").with_configured(false),
        );
        let descriptors = registry.describe_providers(false);
        assert_eq!(
            descriptors
                .iter()
                .map(|d| d.key.as_str())
                .collect::<Vec<_>>(),
            ["ready"]
        );
        // …but it still resolves, so a misconfigured choice fails with its own
        // message rather than with "unsupported".
        assert!(registry.resolve(Some("unready")).is_ok());
    }

    #[test]
    fn registration_order_survives() {
        let mut registry = registry();
        for key in ["c", "a", "b"] {
            register(&mut registry, TestProvider::new(key));
        }
        assert_eq!(registry.provider_keys(), ["c", "a", "b"]);
        assert_eq!(registry.len(), 3);
        assert_eq!(registry.resolvable_names(), ["a", "b", "c"]);
    }

    #[test]
    fn a_provider_with_no_catalog_publishes_null_model_ids() {
        let descriptor = describe_provider(&TestProvider::new("s2s-like").with_model(None));
        assert_eq!(descriptor.realtime_model_ids, None);
        assert_eq!(descriptor.model, None);
        assert_eq!(
            serde_json::to_value(&descriptor).expect("serialize")["realtimeModelIds"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn the_descriptor_field_order_is_the_published_one() {
        let descriptor = describe_provider(&TestProvider::new("a"));
        let json = serde_json::to_value(&descriptor).expect("serialize");
        assert_eq!(
            json.as_object()
                .expect("object")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["key", "label", "model", "realtimeModelIds", "configured"]
        );
    }

    #[test]
    fn the_active_realtime_field_order_is_the_published_one() {
        let mut registry = registry();
        register(&mut registry, TestProvider::new("custom"));
        let active = registry.describe_active_realtime(None).expect("describes");
        let json = serde_json::to_value(&active).expect("serialize");
        assert_eq!(
            json.as_object()
                .expect("object")
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [
                "provider",
                "label",
                "model",
                "modelProfile",
                "modelCapabilities",
                "transportCapabilities",
                "modelCatalog",
                "voice",
                "inputSampleRate",
                "configured",
                "configurationSignature",
                "providers",
            ]
        );
    }

    #[test]
    fn a_provider_with_no_profile_publishes_nulls_and_an_empty_catalog() {
        let mut registry = registry();
        register(
            &mut registry,
            TestProvider::new("custom")
                .with_model_profile(None)
                .with_model(None),
        );
        let active = registry.describe_active_realtime(None).expect("describes");
        assert_eq!(active.model_profile, None);
        assert_eq!(active.model_capabilities, None);
        assert_eq!(active.transport_capabilities, None);
        assert_eq!(active.model_catalog, Vec::new());
        assert_eq!(active.model, None);
    }

    #[test]
    fn the_capability_blocks_are_lifted_out_of_the_profile() {
        let profile = via_catalog::local_realtime_model_profile("local-model");
        let mut registry = registry();
        register(
            &mut registry,
            TestProvider::new("custom").with_model_profile(Some(profile.clone())),
        );
        let active = registry.describe_active_realtime(None).expect("describes");
        assert_eq!(active.model_capabilities, Some(profile.model_capabilities));
        assert_eq!(
            active.transport_capabilities,
            Some(profile.transport_capabilities)
        );
        assert_eq!(active.model_profile, Some(profile));
    }

    #[test]
    fn describing_an_unknown_provider_is_an_error_not_an_empty_block() {
        let registry = registry();
        assert!(registry.describe_active_realtime(Some("nope")).is_err());
    }
}
