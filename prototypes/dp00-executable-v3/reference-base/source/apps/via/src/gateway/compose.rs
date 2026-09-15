//! Building every Gateway service from one resolved configuration.
//!
//! `via_app::Services` is *"`createGatewayApplication`'s parameter object as a
//! type"* and takes every service by injection, with a default for each. This
//! module supplies the ones whose default is not good enough for a Gateway a
//! person actually runs:
//!
//! | Field | Default | What this supplies |
//! | --- | --- | --- |
//! | `realtime_registry` | empty | every shipped provider — `dashscope`, `speech-to-speech`, `openai`, `local-omni` — plus whatever the host registered |
//! | `backend` | frontend-only | the configured harness, behind [`via_app::HarnessBackend`] |
//! | `work` | in-memory | `tasks.json`, retention and the clock, from the configuration |
//! | `memory` / `notes` | in-memory | the files under the data directory |
//! | `local_realtime` | `null` | `via_realtime_local::local_health(config)`, serialized — `via-app` cannot compute this itself |
//!
//! # `agent` degrades to `direct`, and says so
//!
//! `docs/architecture.md` §2: *"`agent` degrades to `direct` when no harness is
//! configured, rather than failing … The degradation is reported on
//! `/api/health`, never silent."* `Services::backend` is
//! [`via_app::FrontendOnlyBackend`] in that case, `Services::backend.enabled()`
//! answers `false`, and `via_voice::ModePlan` does the rest — so the
//! degradation is one value rather than a branch repeated per call site.
//!
//! # What a harness costs to build
//!
//! [`via_backends::create_downstream_agent`] resolves a catalogued backend id
//! into a launch spec and validates the descriptor *at composition*, which is
//! the property `docs/architecture.md` §6 asks for: *"A descriptor that is
//! incomplete or internally inconsistent is rejected at startup, not discovered
//! mid-turn."* Nothing is spawned here — the ACP child starts on the first turn
//! that needs it.

use std::sync::Arc;

use async_trait::async_trait;
use tokio_util::task::TaskTracker;
use via_acp::AcpSessionRegistry;
use via_app::{FrontendOnlyBackend, GatewayBackend, HarnessBackend, Services};
use via_backends::{LaunchContext, SystemFinder};
use via_coordinator::{Coordinator, CoordinatorProfile};
use via_core::{Config, EnvMap};
use via_downstream::DownstreamAgent;
use via_i18n::Locale;
use via_log::Logger;
use via_voice::PermissionDecision;
use via_voice::tools::{BackendAvailability, DelegationRunners, PermissionResponder};
use via_work::WorkManager;

use crate::error::CliError;
use crate::gateway::delegation::{CoordinatorRecovery, CoordinatorRunners};
use crate::gateway::engine::{EngineServices, RealtimeEngineFactory};
use crate::gateway::opener::{ConnectOpener, SessionOpener};

/// Everything one Gateway process runs on.
pub struct Composed {
    /// The injected services `via_app::GatewayApplication::build` takes.
    pub services: Services,
    /// The realtime binding, or `None` when the Gateway runs no model at all.
    pub engines: Arc<RealtimeEngineFactory>,
    /// The coordinator, when a harness is configured. Held so shutdown can
    /// close it.
    pub coordinator: Option<Arc<Coordinator>>,
    /// Every task the Gateway spawned, so shutdown can wait on them.
    pub tracker: TaskTracker,
}

impl std::fmt::Debug for Composed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Composed")
            .field("services", &self.services)
            .field("has_coordinator", &self.coordinator.is_some())
            .finish_non_exhaustive()
    }
}

/// How a Gateway is composed, before anything is built.
///
/// Every field is a seam a test replaces; a `via gateway` run replaces none of
/// them. That asymmetry is the point — the milestone test drives the *same*
/// composition root a user does, with the realtime transport and Layer 3
/// swapped at their own seams rather than around them.
pub struct Composition {
    /// Where a realtime session comes from.
    pub opener: Arc<dyn SessionOpener>,
    /// Extra realtime providers to register beside the built-in two.
    pub providers: Vec<Arc<dyn via_realtime::RealtimeProvider>>,
    /// Layer 3, when the caller has already built one.
    ///
    /// `None` resolves the configured backend id through `via-backends`, which
    /// is what a `via gateway` run does.
    pub harness: Option<Arc<dyn DownstreamAgent>>,
    /// The structured logger.
    pub logger: Option<Arc<Logger>>,
}

impl std::fmt::Debug for Composition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Composition")
            .field("opener", &self.opener)
            .field("providers", &self.providers.len())
            .field("has_harness", &self.harness.is_some())
            .finish_non_exhaustive()
    }
}

impl Default for Composition {
    fn default() -> Self {
        Self {
            opener: Arc::new(ConnectOpener),
            providers: Vec::new(),
            harness: None,
            logger: None,
        }
    }
}

impl Composition {
    /// Build every service.
    ///
    /// Must be called from inside a Tokio runtime: the Work queue, the
    /// conversation record and the input arbitration are each an owning task
    /// (`docs/architecture.md` §11), and this method itself awaits one of
    /// them — [`WorkManager::recover_delegated`], the last step, so that a
    /// composed Gateway never starts serving before its restart recovery has
    /// actually run.
    ///
    /// # Errors
    ///
    /// * [`CliError::Core`] for a configuration `via-core` or `via-catalog`
    ///   refuses — including an `VIA_AUTH_SECRET` under the 32-character
    ///   minimum.
    /// * [`CliError::Refused`] for a realtime provider that will not register
    ///   and for a backend id whose descriptor does not validate.
    pub async fn compose(
        self,
        config: &Config,
        environment: &EnvMap,
    ) -> Result<Composed, CliError> {
        let locale = config.locale;
        let tracker = TaskTracker::new();
        let logger = self
            .logger
            .unwrap_or_else(|| Arc::new(via_log::gateway_logger()));

        // `dashscope` and `speech-to-speech` first, upstream's own order, then
        // the two Rust-only providers phases 6 and 8 shipped but never wired
        // into a composition root: `openai` (the litellm/Azure candidate walk)
        // and `local-omni` (the on-device pipeline and endpoint modes). All
        // four register unconditionally — an unconfigured one still resolves
        // by name and still appears in the catalogue as unconfigured, it is
        // only excluded from the *picker* list
        // (`RealtimeProviderRegistry::describe_providers`) — exactly as
        // `builtin_provider_registry` already treats an unconfigured
        // DashScope. `via-realtime-mock` is deliberately not one of the four:
        // it is a test double that already reaches tests through
        // `via_app::testing::test_registry`, and has no place in a
        // composition root a person actually runs.
        let mut registry = via_realtime_dashscope::builtin_provider_registry(config)
            .map_err(|refusal| realtime_refusal(&refusal))?;
        via_realtime_openai::register_openai_provider(&mut registry, config)
            .map_err(|refusal| realtime_refusal(&refusal))?;
        via_realtime_local::register_local_provider(&mut registry, config)
            .map_err(|refusal| realtime_refusal(&refusal))?;
        for provider in self.providers {
            registry
                .register(provider)
                .map_err(|refusal| realtime_refusal(&refusal))?;
        }

        let work = Arc::new(
            WorkManager::builder()
                .configured(config)
                .logger((*logger).clone())
                .tracker(tracker.clone())
                .build(),
        );

        let harness = match self.harness {
            Some(harness) => Some(harness),
            None => build_harness(config, environment)?,
        };

        let (backend, coordinator): (Arc<dyn GatewayBackend>, Option<Arc<Coordinator>>) =
            match harness {
                Some(harness) => {
                    let descriptor = *harness.descriptor();
                    let profile = CoordinatorProfile::default_for(
                        descriptor.id(),
                        descriptor.label(),
                        locale,
                    )
                    .configured(config);
                    let coordinator = Arc::new(
                        Coordinator::builder(harness, profile)
                            .locale(locale)
                            .tracker(tracker.clone())
                            .build(),
                    );
                    let backend = HarnessBackend::new(
                        descriptor,
                        Arc::new(coordinator.permissions().clone()),
                        locale,
                    );
                    (Arc::new(backend), Some(coordinator))
                }
                None => (Arc::new(FrontendOnlyBackend::new(locale)), None),
            };

        // `via-app` stays provider-agnostic and does not depend on
        // `via-realtime-local`, so the on-device `/api/health` note is
        // computed here, at the composition root, and handed over already
        // serialized — `via_realtime_local`'s own module docs ask for exactly
        // this: *"the Gateway embeds [it] beside the realtime block."*
        let local_realtime = serde_json::to_value(via_realtime_local::local_health(config))
            .unwrap_or(serde_json::Value::Null);

        // The wake-word half of every connection. `via-app` never names
        // `via-wake-word` itself; this composition root is where the switch
        // in `config`, an (empty, until something installs models) keyword
        // table and the detector opener become one
        // [`via_voice::WakeWordLifecycle`]. `via_voice::NoWakeWordEngine`
        // is the opener every build without `--features sherpa` composes —
        // see that crate's own docs for why the engine stays opt-in.
        // `VIA_WAKE_WORD_ENABLED` with no keyword table installed resolves
        // to a clean `DisabledReason::NoPhraseConfigured`, never a panic.
        let wake_word = via_voice::WakeWordLifecycle::from_config(
            config,
            via_voice::KeywordSet::new(),
            Arc::new(via_voice::NoWakeWordEngine),
        );

        let services = Services::builder(config.clone())
            .logger(Arc::clone(&logger))
            .realtime_registry(Arc::new(registry))
            .realtime_provider(config.realtime.provider)
            .work(Arc::clone(&work))
            .backend(Arc::clone(&backend))
            .local_realtime(local_realtime)
            .wake_word(wake_word)
            .build()?;

        let runners: Arc<dyn DelegationRunners> = match &coordinator {
            Some(coordinator) => Arc::new(CoordinatorRunners::new(Arc::clone(coordinator), locale)),
            None => Arc::new(NoRunners),
        };

        // `via_work::WorkManager::recover_delegated` documents itself as
        // "call once, at composition: the candidate list is drained" — this
        // is that call. Skipping it when there is no coordinator would leave
        // a `delegated` Work from a *previous*, harness-configured run
        // stranded `queued` in `recovery_candidates` forever
        // (`docs/deviations/phase-9-via-e2e.md` #4), so [`NoRecovery`] runs
        // the same drain and declines every candidate instead.
        let recovery: Arc<dyn via_work::DelegatedWorkRecovery> = match &coordinator {
            Some(coordinator) => {
                Arc::new(CoordinatorRecovery::new(Arc::clone(coordinator), locale))
            }
            None => Arc::new(NoRecovery),
        };
        work.recover_delegated(recovery).await;

        let engines = Arc::new(RealtimeEngineFactory::new(EngineServices {
            services: services.clone(),
            opener: self.opener,
            runners,
            availability: Arc::new(GatewayAvailability {
                backend: Arc::clone(&backend),
            }),
            permissions: coordinator
                .as_ref()
                .map(|_| Arc::new(BackendRelay { backend }) as Arc<dyn PermissionResponder>),
            tracker: tracker.clone(),
        }));

        Ok(Composed {
            services,
            engines,
            coordinator,
            tracker,
        })
    }
}

/// Resolve the configured backend id into a harness, or answer `None` for
/// voice chat only.
///
/// **External contract** — `cli/src/launcher.mjs:55-86`: an *empty*
/// `AGENT_PROTOCOL` is the deliberate "no backend" value, which is why
/// `via_core::BackendSelection` has no third variant and why this is a plain
/// emptiness check rather than a catalog miss.
fn build_harness(
    config: &Config,
    environment: &EnvMap,
) -> Result<Option<Arc<dyn DownstreamAgent>>, CliError> {
    let protocol = config.agent_protocol.trim();
    if protocol.is_empty() {
        return Ok(None);
    }
    // A platform outside upstream's closed set has no launch conventions to
    // follow, so the backend is refused rather than launched with the wrong
    // ones.
    let platform = via_backends::HostPlatform::host().ok_or_else(|| {
        CliError::refused_with(
            crate::error::CODE_INVALID_ARGUMENT,
            config.locale,
            via_i18n::keys::CLI_SERVICE_PLATFORM_UNSUPPORTED,
            &[("platform", std::env::consts::OS)],
        )
    })?;
    let finder = SystemFinder::new(environment.clone(), platform);
    let context = LaunchContext {
        config,
        env: environment,
        ownership: config.backend_ownership,
        permission_mode: &config.backend_permission_mode,
        owner_id: &config.personal_owner_id,
        finder: &finder,
    };
    let sessions = Arc::new(
        AcpSessionRegistry::builder()
            // `<config>/state/acp-sessions.json` — `via-core` owns the path,
            // catalogued as *"backend session state file"*.
            .file_path(config.paths.backend_session_state_file())
            .locale(config.locale)
            .build(),
    );
    let agent =
        via_backends::create_downstream_agent(protocol, &context, sessions).map_err(|refusal| {
            CliError::Refused {
                code: crate::error::CODE_INVALID_ARGUMENT,
                message: refusal.to_string(),
            }
        })?;
    Ok(Some(Arc::new(agent)))
}

/// A realtime registration refusal, as a CLI refusal.
fn realtime_refusal(error: &via_realtime::RealtimeError) -> CliError {
    CliError::Refused {
        code: crate::error::CODE_INVALID_ARGUMENT,
        message: error.to_string(),
    }
}

/// [`via_voice::tools::BackendAvailability`] over the Gateway's backend seam.
///
/// **External contract** — `tool-call-handler.mjs:463-464`'s default is
/// `{configured: true, ok: true, known: false}`, *optimistic on purpose*. This
/// keeps the optimism where upstream has it and only tightens `configured`,
/// which is the one fact the Gateway knows for certain: whether a harness was
/// composed at all.
#[derive(Debug)]
struct GatewayAvailability {
    backend: Arc<dyn GatewayBackend>,
}

impl BackendAvailability for GatewayAvailability {
    fn configured(&self) -> bool {
        self.backend.enabled()
    }

    fn ok(&self) -> bool {
        self.backend.enabled()
    }

    /// `false`: *accept optimistically and let dispatch report failures*. The
    /// probe that would make this `true` is `via-backends`' availability probe,
    /// which is the reminder scheduler's and the backend runtime's to own.
    fn known(&self) -> bool {
        false
    }
}

/// [`PermissionResponder`] over the same seam `POST /api/permissions/{id}` uses.
///
/// Routing the model's `respond_agent_permission` through
/// [`via_app::GatewayBackend`] rather than straight to the broker is what keeps
/// *one* relay: a decision a person clicks and a decision the model speaks land
/// on the same code, so they cannot diverge.
#[derive(Debug)]
struct BackendRelay {
    backend: Arc<dyn GatewayBackend>,
}

#[async_trait]
impl PermissionResponder for BackendRelay {
    async fn respond(
        &self,
        authorization_id: &str,
        decision: PermissionDecision,
        owner_id: &str,
    ) -> Result<(), String> {
        self.backend
            .respond_permission(authorization_id, decision, owner_id)
            .await
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

/// The runners a Gateway with no harness has.
///
/// Unreachable in practice: `via_voice::ModePlan` does not declare
/// `spawn_thinking` when no harness is configured, so the handler refuses the
/// tool before a runner is ever asked for. It exists so that fact is expressed
/// as a type rather than as an `Option` every call site has to unwrap.
#[derive(Debug)]
struct NoRunners;

impl DelegationRunners for NoRunners {
    fn delegation_runner(
        &self,
        _request: &via_voice::tools::DelegationRequest,
    ) -> Arc<dyn via_work::WorkRunner> {
        Arc::new(refusing_runner as fn(String, via_work::WorkContext) -> _)
    }

    fn delegation_canceler(
        &self,
        _request: &via_voice::tools::DelegationRequest,
    ) -> Arc<dyn via_work::WorkCanceler> {
        Arc::new(nothing_to_cancel as fn(via_work::CancelRequest) -> _)
    }

    fn status_query_runner(
        &self,
        _request: &via_voice::tools::StatusQueryRequest,
    ) -> Arc<dyn via_work::WorkRunner> {
        Arc::new(refusing_runner as fn(String, via_work::WorkContext) -> _)
    }

    fn status_query_canceler(
        &self,
        _request: &via_voice::tools::StatusQueryRequest,
    ) -> Arc<dyn via_work::WorkCanceler> {
        Arc::new(nothing_to_cancel as fn(via_work::CancelRequest) -> _)
    }

    fn scheduled_task_runner(&self) -> Option<Arc<dyn via_work::WorkRunner>> {
        None
    }
}

async fn refusing_runner(
    _objective: String,
    _context: via_work::WorkContext,
) -> Result<via_work::WorkOutcome, via_work::RunFailure> {
    Err(via_work::RunFailure {
        message: via_i18n::t(
            Locale::En,
            via_i18n::keys::VOICE_ERROR_BACKEND_NOT_CONFIGURED,
        )
        .to_owned(),
    })
}

async fn nothing_to_cancel(
    _request: via_work::CancelRequest,
) -> Result<via_downstream::CancelOutcome, via_work::RunFailure> {
    Ok(via_downstream::CancelOutcome::NotFound)
}

/// The recovery a Gateway with no coordinator has.
///
/// There is no backend to reattach a delegation to, so every candidate
/// declines — the same outcome [`CoordinatorRecovery`] reaches today anyway
/// (VIA's composition wires no native-delegation adapter), reached here
/// without needing a coordinator to ask. `run`/`cancel` reuse [`NoRunners`]'s
/// own refusal and no-op, since [`via_work::WorkManager::recover_delegated`]
/// never calls either once `can_recover` has answered `false`.
#[derive(Debug)]
struct NoRecovery;

#[async_trait]
impl via_work::DelegatedWorkRecovery for NoRecovery {
    fn can_recover(&self, _work: &via_work::WorkSnapshot) -> bool {
        false
    }

    async fn run(
        &self,
        _work: via_work::WorkSnapshot,
        context: via_work::WorkContext,
    ) -> Result<via_work::WorkOutcome, via_work::RunFailure> {
        refusing_runner(String::new(), context).await
    }

    async fn cancel(
        &self,
        _work: via_work::WorkSnapshot,
        request: via_work::CancelRequest,
    ) -> Result<via_downstream::CancelOutcome, via_work::RunFailure> {
        nothing_to_cancel(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn config() -> Config {
        let env: EnvMap = [
            ("DASHSCOPE_API_KEY", "sk-test"),
            ("VIA_AUTH_SECRET", &"a".repeat(64)),
            ("PORT", "0"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
        let overrides = via_core::config::Overrides {
            home_directory: std::env::temp_dir().join("via-compose-tests"),
            working_directory: std::env::temp_dir().join("via-compose-tests"),
            ..via_core::config::Overrides::default()
        };
        via_core::config::resolve(&env, None, &overrides).expect("a resolvable configuration")
    }

    #[tokio::test]
    async fn a_gateway_with_no_backend_composes_frontend_only() {
        let composed = Composition::default()
            .compose(&config(), &EnvMap::default())
            .await
            .expect("composes");
        assert!(!composed.services.backend.enabled());
        assert!(composed.coordinator.is_none());
    }

    #[tokio::test]
    async fn every_shipped_realtime_provider_is_registered() {
        let composed = Composition::default()
            .compose(&config(), &EnvMap::default())
            .await
            .expect("composes");
        assert_eq!(
            composed.services.realtime_registry.provider_keys(),
            ["dashscope", "speech-to-speech", "openai", "local-omni"],
            "dashscope and speech-to-speech first (upstream's own order), then \
             the two Rust-only providers phase 6 and phase 8 shipped but never \
             wired into a composition root — via-realtime-mock is deliberately \
             not one of the four, it is a test double",
        );
    }

    /// `via_realtime_local::local_health` "and nothing consumes it" no
    /// longer holds: `/api/health` (via `Services::local_realtime`) carries
    /// the reason an operator's local-omni provider is or is not usable.
    #[tokio::test]
    async fn the_local_realtime_health_note_reaches_services() {
        let composed = Composition::default()
            .compose(&config(), &EnvMap::default())
            .await
            .expect("composes");
        assert_eq!(
            composed.services.local_realtime["mode"], "local-omni:pipeline",
            "a bare configuration selects the componentized pipeline, which \
             runs on the dev box — `docs/architecture.md` §16",
        );
        assert_eq!(composed.services.local_realtime["verified"], true);
    }

    #[tokio::test]
    async fn an_unconfigured_provider_is_listed_but_unconfigured_rather_than_absent() {
        // No realtime credential at all — `via-realtime-openai` reads the same
        // `VIA_REALTIME_API_KEY`/`DASHSCOPE_API_KEY` fallback dashscope does,
        // so `config()`'s own `DASHSCOPE_API_KEY=sk-test` would configure it
        // too — and no local weights on disk. Both providers still resolve by
        // name, so the Gateway can report why they are unusable, and neither
        // shows up in the picker list a client sees.
        let env: EnvMap = [
            ("VIA_AUTH_SECRET".to_owned(), "a".repeat(64)),
            ("PORT".to_owned(), "0".to_owned()),
        ]
        .into_iter()
        .collect();
        let overrides = via_core::config::Overrides {
            home_directory: std::env::temp_dir().join("via-compose-tests-unconfigured"),
            working_directory: std::env::temp_dir().join("via-compose-tests-unconfigured"),
            ..via_core::config::Overrides::default()
        };
        let config = via_core::config::resolve(&env, None, &overrides)
            .expect("a resolvable configuration with no realtime credential at all");
        let composed = Composition::default()
            .compose(&config, &EnvMap::default())
            .await
            .expect("composes");
        let registry = &composed.services.realtime_registry;
        assert_eq!(
            registry.resolve(Some("openai")).expect("resolves").key(),
            "openai"
        );
        assert_eq!(
            registry
                .resolve(Some("local-omni"))
                .expect("resolves")
                .key(),
            "local-omni",
        );
        let picker: Vec<String> = registry
            .describe_providers(false)
            .into_iter()
            .map(|descriptor| descriptor.key)
            .collect();
        assert!(!picker.iter().any(|key| key == "openai"), "{picker:?}");
        assert!(!picker.iter().any(|key| key == "local-omni"), "{picker:?}");
    }

    /// Selecting a newly-registered provider as `realtimeProvider` composes,
    /// which is what makes it reachable through `/api/health`'s
    /// `realtimeProvider` field rather than only through `realtimeProviders`.
    #[tokio::test]
    async fn selecting_either_new_provider_as_the_active_one_composes() {
        for provider in ["openai", "local-omni"] {
            let env: EnvMap = [
                ("DASHSCOPE_API_KEY", "sk-test"),
                ("VIA_AUTH_SECRET", &"a".repeat(64)),
                ("PORT", "0"),
                ("VIA_REALTIME_PROVIDER", provider),
            ]
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect();
            let overrides = via_core::config::Overrides {
                home_directory: std::env::temp_dir().join("via-compose-tests"),
                working_directory: std::env::temp_dir().join("via-compose-tests"),
                ..via_core::config::Overrides::default()
            };
            let config = via_core::config::resolve(&env, None, &overrides)
                .expect("a resolvable configuration");
            let composed = Composition::default()
                .compose(&config, &EnvMap::default())
                .await
                .unwrap_or_else(|error| panic!("{provider} composes: {error}"));
            assert_eq!(
                composed.services.realtime_provider.as_deref(),
                Some(provider)
            );
        }
    }

    #[tokio::test]
    async fn an_unavailable_backend_still_answers_optimistically() {
        let composed = Composition::default()
            .compose(&config(), &EnvMap::default())
            .await
            .expect("composes");
        let availability = GatewayAvailability {
            backend: Arc::clone(&composed.services.backend),
        };
        assert!(!availability.configured());
        assert!(
            !availability.known(),
            "`known` stays false: accept optimistically and let dispatch report failures",
        );
    }

    /// The end-to-end shape of `docs/deviations/README.md`'s closed
    /// "`WorkManager::recover_delegated` is never called" gap: a real
    /// `compose()`, with a real (harness-configured) coordinator, actually
    /// drains `recovery_candidates` and reaches the catalogued *unrecoverable
    /// delegated work* restart sentence — not the *interactive* one, and not
    /// a Work stranded `queued` forever.
    #[tokio::test]
    async fn an_unrecoverable_delegated_work_reaches_the_restart_sentence_at_composition() {
        use via_downstream::DownstreamAgent;
        use via_downstream::testing::ScriptedHarness;
        use via_protocol::WorkStatus;
        use via_work::WorkStore;

        let env: EnvMap = [
            ("DASHSCOPE_API_KEY", "sk-test"),
            ("VIA_AUTH_SECRET", &"a".repeat(64)),
            ("PORT", "0"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
        let overrides = via_core::config::Overrides {
            home_directory: std::env::temp_dir().join("via-compose-tests-recovery"),
            working_directory: std::env::temp_dir().join("via-compose-tests-recovery"),
            ..via_core::config::Overrides::default()
        };
        let config =
            via_core::config::resolve(&env, None, &overrides).expect("a resolvable configuration");

        // Seed the restart fixture directly at the path `compose()`'s own
        // `WorkManager::builder().configured(config)` will read — no process
        // actually crashed here, but the file on disk cannot tell the
        // difference.
        let record: via_work::PersistedWork = serde_json::from_value(serde_json::json!({
            "id": "work-delegated",
            "status": "delegated",
            "objective": "继续项目",
            "ownerId": "owner-one",
            "delegation": {
                "id": "run-one",
                "sessionId": "agent:child:one",
                "directory": "/project",
                "title": "项目任务",
            },
        }))
        .expect("a valid persisted record");
        assert!(
            WorkStore::builder()
                .file_path(&config.task_state_path)
                .build()
                .save(&[record]),
            "seeds the restart fixture",
        );

        // A harness-configured coordinator, exactly like a real `via gateway`
        // with `AGENT_PROTOCOL` set — and, like every shipped composition,
        // with no `NativeDelegationAdapter`. The script carries no turn at
        // all: a correctly declined recovery never prompts it.
        let harness = ScriptedHarness::builder("opencode")
            .build()
            .expect("opencode is catalogued");
        let composition = Composition {
            harness: Some(Arc::new(harness) as Arc<dyn DownstreamAgent>),
            ..Composition::default()
        };

        let composed = composition
            .compose(&config, &EnvMap::default())
            .await
            .expect("composes");
        assert!(
            composed.coordinator.is_some(),
            "a harness-configured Composition builds a real coordinator",
        );

        let recovered = composed
            .services
            .work
            .get("work-delegated", None)
            .await
            .expect("restored");
        assert_eq!(recovered.status, WorkStatus::Failed);
        assert_eq!(
            recovered.error.as_deref(),
            Some(via_i18n::t(
                config.locale,
                via_i18n::keys::WORK_RESTART_DELEGATED_LOST
            )),
            "the *unrecoverable delegated work* sentence, not the interactive one \
             `Actor::restore` would have used for a Work with no addressable \
             delegation",
        );
        assert_ne!(
            recovered.error.as_deref(),
            Some(via_i18n::t(
                config.locale,
                via_i18n::keys::WORK_RESTART_INTERACTIVE_INCOMPLETE
            )),
        );
    }
}
