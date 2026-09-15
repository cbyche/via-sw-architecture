//! Every service the Gateway is built from, taken by injection.
//!
//! This is `createGatewayApplication`'s parameter object
//! (`server/src/app/gateway-application.mjs:41-54`) as a type. Upstream's
//! default for each field is a module-level singleton; here the default is
//! spelled out by [`ServicesBuilder`] and every field can be replaced, which is
//! what `server/test/gateway-application.test.mjs` needs in order to construct a
//! Gateway with an isolated task manager, an isolated coordinator, a private
//! realtime provider and a stand-in input-asset registry.
//!
//! # Importing the factory must not bind a port
//!
//! Upstream separates *construct* from *listen* with an `autoStart` flag
//! defaulting to `true`, and its test passes `autoStart: false` and then asserts
//! `application.server.listening === false`
//! (`gateway-application.test.mjs:41`). In Rust the separation is structural
//! rather than a flag: [`crate::GatewayApplication::build`] constructs and binds
//! nothing, and [`crate::GatewayApplication::bind`] is a separate,
//! `async`, fallible call. There is no way to spell "constructing it also bound
//! a socket", so `bootstrap.mjs`'s module-level side effect has no counterpart
//! and needs none.

use std::sync::Arc;

use via_conversation::{ConversationSyncHandle, FrontendMemoryService, FrontendNotesStore};
use via_core::{Config, IdentityManager};
use via_i18n::Locale;
use via_log::Logger;
use via_realtime::RealtimeProviderRegistry;
use via_voice::{InputArbitration, InputAssetRegistry, SessionPermissionPolicy};
use via_work::WorkManager;

use crate::backend::{FrontendOnlyBackend, GatewayBackend};

/// The services one Gateway is composed of.
///
/// Cheap to clone: every field is an `Arc`, and the Gateway holds one clone per
/// request handler.
#[derive(Clone)]
pub struct Services {
    /// The resolved configuration.
    pub config: Arc<Config>,
    /// Layer 3, behind the four-question seam.
    pub backend: Arc<dyn GatewayBackend>,
    /// The conversation record.
    pub conversation_sync: ConversationSyncHandle,
    /// The staged input parts, per session.
    pub input_assets: Arc<InputAssetRegistry>,
    /// The microphone control plane.
    pub input_arbitration: Arc<InputArbitration>,
    /// The Work queue.
    pub work: Arc<WorkManager>,
    /// `USER.md` and `MEMORY.md`.
    pub memory: Arc<FrontendMemoryService>,
    /// The volatile named lists.
    pub notes: Arc<FrontendNotesStore>,
    /// The per-session backend-permission policy.
    ///
    /// A `tokio::Mutex` rather than an owning task: the policy is a bounded LRU
    /// whose operations are pure map reads and writes with no ordering
    /// requirement between owners, so `docs/architecture.md` §11's *"ordering
    /// invariants get an owning task"* does not apply. The four invariants that
    /// do are elsewhere — the Work FIFO, the keyed serial executor, the
    /// announcement window and delegation correlation — and each already has
    /// one in the crate that owns it.
    pub permission_policy: Arc<tokio::sync::Mutex<SessionPermissionPolicy>>,
    /// Who this request is.
    pub identity: Arc<IdentityManager>,
    /// The realtime providers a client may select at connect time.
    pub realtime_registry: Arc<RealtimeProviderRegistry>,
    /// The provider a client gets when it names none.
    pub realtime_provider: Option<String>,
    /// The structured logger.
    pub logger: Arc<Logger>,
    /// The locale every user-facing string is rendered in.
    pub locale: Locale,
    /// The on-device realtime path's own `/api/health` note, or `null`.
    ///
    /// Untyped for the same reason [`crate::backend::BackendDescription`] is:
    /// `via-app` is provider-agnostic and does not depend on
    /// `via-realtime-local`, whose `LocalHealth` this carries. The composition
    /// root computes it — `serde_json::to_value(local_health(config))` — and
    /// hands over the already-serialized value; `via-app` only carries it
    /// through to `/api/health`'s `localRealtime` field.
    pub local_realtime: serde_json::Value,
    /// The wake-word half of every connection: settings plus the engine that
    /// opens a detector. `via-app` stays provider-agnostic here too — it
    /// never names `via-wake-word` itself, only [`via_voice::WakeWordLifecycle`]
    /// — exactly as [`Self::local_realtime`] keeps `via-realtime-local` out
    /// of this crate's own dependency graph.
    pub wake_word: via_voice::WakeWordLifecycle,
}

impl std::fmt::Debug for Services {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Services")
            .field("backend", &self.backend)
            .field("realtime_provider", &self.realtime_provider)
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl Services {
    /// Start from `config`, with every service defaulted.
    #[must_use]
    pub fn builder(config: Config) -> ServicesBuilder {
        ServicesBuilder::new(config)
    }
}

/// Builds [`Services`], one replaced default at a time.
///
/// Every `with_*` call is one line of upstream's parameter object. A field left
/// alone gets the default `createGatewayApplication` would have used, built
/// from the configuration.
pub struct ServicesBuilder {
    config: Config,
    backend: Option<Arc<dyn GatewayBackend>>,
    conversation_sync: Option<ConversationSyncHandle>,
    input_assets: Option<Arc<InputAssetRegistry>>,
    input_arbitration: Option<Arc<InputArbitration>>,
    work: Option<Arc<WorkManager>>,
    memory: Option<Arc<FrontendMemoryService>>,
    notes: Option<Arc<FrontendNotesStore>>,
    permission_policy: Option<Arc<tokio::sync::Mutex<SessionPermissionPolicy>>>,
    identity: Option<Arc<IdentityManager>>,
    realtime_registry: Option<Arc<RealtimeProviderRegistry>>,
    realtime_provider: Option<String>,
    logger: Option<Arc<Logger>>,
    local_realtime: Option<serde_json::Value>,
    wake_word: Option<via_voice::WakeWordLifecycle>,
}

impl std::fmt::Debug for ServicesBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServicesBuilder")
            .field("realtime_provider", &self.realtime_provider)
            .finish_non_exhaustive()
    }
}

impl ServicesBuilder {
    fn new(config: Config) -> Self {
        Self {
            config,
            backend: None,
            conversation_sync: None,
            input_assets: None,
            input_arbitration: None,
            work: None,
            memory: None,
            notes: None,
            permission_policy: None,
            identity: None,
            realtime_registry: None,
            realtime_provider: None,
            logger: None,
            local_realtime: None,
            wake_word: None,
        }
    }

    /// Replace Layer 3.
    #[must_use]
    pub fn backend(mut self, backend: Arc<dyn GatewayBackend>) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Replace the conversation record.
    #[must_use]
    pub fn conversation_sync(mut self, sync: ConversationSyncHandle) -> Self {
        self.conversation_sync = Some(sync);
        self
    }

    /// Replace the input-asset registry — upstream's `inputAssets` parameter,
    /// which its own test replaces with a bare marker object.
    #[must_use]
    pub fn input_assets(mut self, assets: Arc<InputAssetRegistry>) -> Self {
        self.input_assets = Some(assets);
        self
    }

    /// Replace the microphone control plane.
    #[must_use]
    pub fn input_arbitration(mut self, arbitration: Arc<InputArbitration>) -> Self {
        self.input_arbitration = Some(arbitration);
        self
    }

    /// Replace the Work queue — upstream's `taskManager` **and** `taskStore`,
    /// which are one object here because [`WorkManager`] owns its store.
    #[must_use]
    pub fn work(mut self, work: Arc<WorkManager>) -> Self {
        self.work = Some(work);
        self
    }

    /// Replace the memory service.
    #[must_use]
    pub fn memory(mut self, memory: Arc<FrontendMemoryService>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Replace the notes store.
    #[must_use]
    pub fn notes(mut self, notes: Arc<FrontendNotesStore>) -> Self {
        self.notes = Some(notes);
        self
    }

    /// Replace the per-session permission policy.
    #[must_use]
    pub fn permission_policy(
        mut self,
        policy: Arc<tokio::sync::Mutex<SessionPermissionPolicy>>,
    ) -> Self {
        self.permission_policy = Some(policy);
        self
    }

    /// Replace the identity manager.
    #[must_use]
    pub fn identity(mut self, identity: Arc<IdentityManager>) -> Self {
        self.identity = Some(identity);
        self
    }

    /// Replace the realtime provider registry.
    #[must_use]
    pub fn realtime_registry(mut self, registry: Arc<RealtimeProviderRegistry>) -> Self {
        self.realtime_registry = Some(registry);
        self
    }

    /// Select the default realtime provider — upstream's `realtimeProvider`
    /// parameter, defaulting to `config.audioProvider`.
    #[must_use]
    pub fn realtime_provider(mut self, provider: impl Into<String>) -> Self {
        self.realtime_provider = Some(provider.into());
        self
    }

    /// Replace the logger.
    #[must_use]
    pub fn logger(mut self, logger: Arc<Logger>) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Set the on-device realtime path's `/api/health` note.
    ///
    /// Left at the default (`null`) by every caller that has no
    /// `via-realtime-local` provider registered.
    #[must_use]
    pub fn local_realtime(mut self, note: serde_json::Value) -> Self {
        self.local_realtime = Some(note);
        self
    }

    /// Replace the wake-word lifecycle.
    ///
    /// Left at the default — settings built from `config` with an empty
    /// keyword table and [`via_voice::NoWakeWordEngine`] — by every caller
    /// that has not composed a real detector opener or discovered installed
    /// keyword models. That default is what a deployment with
    /// `VIA_WAKE_WORD_ENABLED` unset, or set with no phrase chosen, resolves
    /// to: a clean "wake word disabled", never a panic.
    #[must_use]
    pub fn wake_word(mut self, wake_word: via_voice::WakeWordLifecycle) -> Self {
        self.wake_word = Some(wake_word);
        self
    }

    /// Fill in every default and hand back the services.
    ///
    /// Must be called from inside a Tokio runtime: the defaults for the Work
    /// queue, the conversation record and the input arbitration are each an
    /// owning task (`docs/architecture.md` §11), and spawning one needs a
    /// reactor.
    ///
    /// # Errors
    ///
    /// [`CoreError::AuthSecretTooShort`](via_core::CoreError::AuthSecretTooShort)
    /// when the default identity manager is built from a `VIA_AUTH_SECRET`
    /// shorter than 32 characters. Supplying an [`IdentityManager`] through
    /// [`identity`](Self::identity) skips the check, because then the secret is
    /// the caller's.
    pub fn build(self) -> Result<Services, via_core::CoreError> {
        let config = self.config;
        let locale = config.locale;
        let logger = self
            .logger
            .unwrap_or_else(|| Arc::new(via_log::gateway_logger()));

        let input_assets = self.input_assets.unwrap_or_else(|| {
            Arc::new(
                InputAssetRegistry::new(locale)
                    .session_ttl_ms(config.conversation_session_ttl_ms)
                    .max_sessions(usize::try_from(config.max_conversation_sessions).unwrap_or(0)),
            )
        });
        let input_arbitration = self
            .input_arbitration
            .unwrap_or_else(|| Arc::new(InputArbitration::new(locale)));
        let identity = match self.identity {
            Some(identity) => identity,
            None => Arc::new(IdentityManager::new(
                config.auth_secret.expose(),
                config.identity_mode,
                &config.personal_owner_id,
            )?),
        };
        let permission_policy = self.permission_policy.unwrap_or_else(|| {
            Arc::new(tokio::sync::Mutex::new(
                SessionPermissionPolicy::new()
                    .ttl_ms(config.conversation_session_ttl_ms)
                    .max_sessions(usize::try_from(config.max_conversation_sessions).unwrap_or(0)),
            ))
        });
        let backend = self
            .backend
            .unwrap_or_else(|| Arc::new(FrontendOnlyBackend::new(locale)));

        Ok(Services {
            backend,
            conversation_sync: self.conversation_sync.unwrap_or_else(|| {
                let mut sync = via_conversation::ConversationSync::default();
                sync.set_retention(via_conversation::Retention {
                    session_ttl_ms: config.conversation_session_ttl_ms,
                    max_sessions: usize::try_from(config.max_conversation_sessions).unwrap_or(0),
                    ..via_conversation::Retention::default()
                });
                // The join handle is deliberately dropped: the actor stops when
                // the last handle does, which is what `close()` relies on.
                ConversationSyncHandle::spawn(sync).0
            }),
            input_assets,
            input_arbitration,
            work: self
                .work
                .unwrap_or_else(|| Arc::new(WorkManager::in_memory())),
            memory: self
                .memory
                .unwrap_or_else(|| Arc::new(FrontendMemoryService::default())),
            notes: self
                .notes
                .unwrap_or_else(|| Arc::new(FrontendNotesStore::in_memory())),
            permission_policy,
            identity,
            realtime_registry: self
                .realtime_registry
                .unwrap_or_else(|| Arc::new(RealtimeProviderRegistry::new())),
            realtime_provider: self.realtime_provider,
            logger,
            locale,
            local_realtime: self.local_realtime.unwrap_or(serde_json::Value::Null),
            wake_word: self.wake_word.unwrap_or_else(|| {
                via_voice::WakeWordLifecycle::from_config(
                    &config,
                    via_voice::KeywordSet::new(),
                    Arc::new(via_voice::NoWakeWordEngine),
                )
            }),
            config: Arc::new(config),
        })
    }
}
