//! The port of `server/test/managed-backend.test.mjs`.
//!
//! Upstream's file mixes the generic managed-process behaviour with two
//! backend-specific credential fixtures. Only the generic half belongs here;
//! the OpenClaw credential tests are `via-backends`' (see the module docs on
//! [`via_process::driver`]). Everything upstream asserts about ownership, port
//! reallocation, the child environment and the shutdown ladder is reproduced,
//! selecting backends by property rather than by name.

mod common;

use std::path::PathBuf;
use std::time::Duration;

use common::{
    FixedResolver, MissingResolver, RecordingSpawner, ScriptedAllocator, always_full_definition,
    env, explicit_list_definition, external_service_definition, in_gateway_definition,
    no_full_permission_definition, owned_service_definition, registry, service_driver,
};
use pretty_assertions::assert_eq;
use via_catalog::Ownership;
use via_core::EnvMap;
use via_core::config::names;
use via_core::search_path::Platform;
use via_i18n::{Locale, keys};
use via_process::{
    BackendRuntimeDriver, BackendRuntimeHooks, BackendRuntimeRegistry, ENV_LOADED_NAME,
    ENV_LOADED_VALUE, ManagedBackend, ManagedBackendStart, PERMISSION_MODE_FULL,
    PERMISSION_MODE_NATIVE, ProcessError, ResolveRequest, StartOptions, StopSignal,
    resolve_managed_backend, start_managed_backend,
};

const ROOT: &str = "/repo";
const RESOLVED_BIN: &str = "/opt/fixture/bin";

/// Run a start with the collaborators a test supplies.
async fn start(
    registry: &BackendRuntimeRegistry,
    environment: &mut EnvMap,
    allocator: &ScriptedAllocator,
    spawner: &RecordingSpawner,
) -> Result<ManagedBackendStart, ProcessError> {
    let resolver = FixedResolver::new(RESOLVED_BIN);
    let options = StartOptions {
        root: PathBuf::from(ROOT),
        registry,
        platform: Platform::Posix,
        allocator,
        spawner,
        resolver: &resolver,
        logger: None,
    };
    start_managed_backend(environment, &options).await
}

// ── ownership ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_gateway_owned_backend_moves_away_from_an_occupied_port() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_OWNERSHIP, "owned"),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
        ("FIXTURE_TOKEN", "reaches the child"),
        // Gateway secrets that must not cross the boundary.
        (names::AUTH_SECRET, "must not reach the backend"),
        ("SPEECH_TO_SPEECH_AUTH_TOKEN", "must not reach the backend"),
    ]);
    let allocator = ScriptedAllocator::new(true, "http://127.0.0.1:45678");
    let spawner = RecordingSpawner::new();
    let started = start(&registry, &mut environment, &allocator, &spawner)
        .await
        .expect("the backend starts");

    // The move happened, and both variables were republished.
    assert_eq!(allocator.probed(), vec!["http://127.0.0.1:18789"]);
    assert_eq!(allocator.allocated(), vec!["http://127.0.0.1:18789"]);
    assert_eq!(
        environment.get(&service.base_url_environment),
        Some("http://127.0.0.1:45678"),
    );
    assert_eq!(environment.get(&service.port_environment), Some("45678"));
    assert_eq!(
        started.backend.as_ref().and_then(|b| b.base_url.as_deref()),
        Some("http://127.0.0.1:45678"),
    );

    // The spawn spec is the catalogued shape.
    let spec = spawner.only_spec();
    assert_eq!(
        spec.command,
        PathBuf::from(format!("{RESOLVED_BIN}/fixture-runner")),
    );
    assert_eq!(spec.arguments, vec!["--serve".to_owned()]);
    assert_eq!(spec.working_directory, PathBuf::from(ROOT));

    // The trust boundary.
    assert_eq!(
        spec.environment.get(ENV_LOADED_NAME),
        Some(ENV_LOADED_VALUE)
    );
    assert_eq!(
        spec.environment.get("FIXTURE_TOKEN"),
        Some("reaches the child")
    );
    assert_eq!(spec.environment.get(names::AUTH_SECRET), None);
    assert_eq!(spec.environment.get("SPEECH_TO_SPEECH_AUTH_TOKEN"), None);

    // And the ladder signals the group.
    let mut runtime = started.runtime;
    assert!(runtime.owns_process());
    runtime.close(StopSignal::Term);
    assert_eq!(spawner.signals(), vec![StopSignal::Term]);
}

#[tokio::test]
async fn a_managed_backend_that_ignores_graceful_shutdown_is_force_stopped() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_OWNERSHIP, "owned"),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
    ]);
    let allocator = ScriptedAllocator::new(false, "");
    let spawner = RecordingSpawner::new();
    let started = start(&registry, &mut environment, &allocator, &spawner)
        .await
        .expect("the backend starts");

    // Nothing moved: the port was free.
    assert!(allocator.allocated().is_empty());
    assert_eq!(
        environment.get(&service.base_url_environment),
        Some("http://127.0.0.1:18789"),
    );

    let mut runtime = started.runtime;
    runtime
        .stop(StopSignal::Term, Duration::from_millis(1))
        .await;
    assert_eq!(spawner.signals(), vec![StopSignal::Term, StopSignal::Kill]);
}

#[tokio::test]
async fn an_explicitly_configured_service_is_external_and_is_left_alone() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
    ]);
    assert_eq!(
        resolve_managed_backend(&registry, &environment).unwrap(),
        Some(ManagedBackend {
            protocol: definition.id.to_owned(),
            ownership: Ownership::External,
            permission_mode: PERMISSION_MODE_NATIVE.to_owned(),
            base_url: Some("http://127.0.0.1:18789".to_owned()),
        }),
    );

    // "external Gateway ownership must not probe or move the port" and
    // "must not spawn" — both collaborators panic if touched.
    let allocator = ScriptedAllocator::forbidden();
    let spawner = RecordingSpawner::forbidden();
    let started = start(&registry, &mut environment, &allocator, &spawner)
        .await
        .expect("an external backend resolves");

    assert!(!started.runtime.owns_process());
    assert_eq!(
        environment.get(names::BACKEND_OWNERSHIP),
        Some(Ownership::External.as_str()),
    );
    assert_eq!(
        environment.get(&service.base_url_environment),
        Some("http://127.0.0.1:18789"),
        "an external address must not be rewritten",
    );
    assert!(environment.get(&service.port_environment).is_none());
}

#[test]
fn a_secure_remote_websocket_is_accepted_without_taking_ownership() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let backend = resolve_managed_backend(
        &registry,
        &env(&[
            ("AGENT_PROTOCOL", definition.id),
            (
                &service.base_url_environment,
                "wss://agent.example.com/gateway",
            ),
        ]),
    )
    .unwrap();
    assert_eq!(
        backend,
        Some(ManagedBackend {
            protocol: definition.id.to_owned(),
            ownership: Ownership::External,
            permission_mode: PERMISSION_MODE_NATIVE.to_owned(),
            base_url: Some("wss://agent.example.com".to_owned()),
        }),
    );
}

#[test]
fn credentials_embedded_in_an_external_address_are_refused() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    for address in [
        "wss://user:secret@agent.example.com",
        "http://user@127.0.0.1:18789",
    ] {
        let error = resolve_managed_backend(
            &registry,
            &env(&[
                ("AGENT_PROTOCOL", definition.id),
                (&service.base_url_environment, address),
            ]),
        )
        .unwrap_err();
        assert!(
            matches!(error, ProcessError::ServiceUrlHasCredentials),
            "{address} was accepted",
        );
    }
}

#[test]
fn a_scheme_the_driver_does_not_admit_is_refused() {
    let definition = owned_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let error = resolve_managed_backend(
        &registry,
        &env(&[
            ("AGENT_PROTOCOL", definition.id),
            (&service.base_url_environment, "wss://127.0.0.1:4096"),
        ]),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProcessError::UnsupportedServiceProtocol { ref protocol } if protocol == "wss:"
    ));
}

#[tokio::test]
async fn no_backend_selected_means_no_managed_process() {
    let registry = BackendRuntimeRegistry::new();
    for environment in [env(&[]), env(&[("AGENT_PROTOCOL", "none")])] {
        assert_eq!(
            resolve_managed_backend(&registry, &environment).unwrap(),
            None
        );
        let mut environment = environment;
        let started = start(
            &registry,
            &mut environment,
            &ScriptedAllocator::forbidden(),
            &RecordingSpawner::forbidden(),
        )
        .await
        .expect("frontend-only operation is not an error");
        assert!(started.backend.is_none());
        assert!(!started.runtime.owns_process());
        assert!(
            environment.get(names::BACKEND_OWNERSHIP).is_none(),
            "frontend-only operation must not write an ownership back",
        );
    }
}

#[test]
fn the_selected_backend_address_is_reduced_to_its_origin() {
    let definition = owned_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let backend = resolve_managed_backend(
        &registry,
        &env(&[
            ("AGENT_PROTOCOL", definition.id),
            (&service.base_url_environment, "http://localhost:4096/path"),
        ]),
    )
    .unwrap()
    .expect("a backend");
    assert_eq!(backend.base_url.as_deref(), Some("http://localhost:4096"));
    // A driver that cannot be external stays owned even with an address set.
    assert_eq!(backend.ownership, Ownership::Owned);
}

#[tokio::test]
async fn an_in_gateway_backend_is_managed_without_a_separate_server() {
    let definition = in_gateway_definition();
    let registry = registry(vec![BackendRuntimeDriver::in_gateway(definition)]);
    let mut environment = env(&[("AGENT_PROTOCOL", definition.id)]);
    let started = start(
        &registry,
        &mut environment,
        &ScriptedAllocator::forbidden(),
        &RecordingSpawner::forbidden(),
    )
    .await
    .expect("an in-Gateway backend resolves");
    assert!(!started.runtime.owns_process());
    assert_eq!(
        started.backend.expect("a backend"),
        ManagedBackend {
            protocol: definition.id.to_owned(),
            ownership: Ownership::Owned,
            permission_mode: PERMISSION_MODE_NATIVE.to_owned(),
            base_url: None,
        },
    );
    // The write-back happens even when nothing is spawned: the child that the
    // ACP layer starts still has to see the resolved value.
    assert_eq!(
        environment.get(names::BACKEND_OWNERSHIP),
        Some(Ownership::Owned.as_str()),
    );
}

#[test]
fn an_in_gateway_backend_may_not_be_external() {
    let definition = in_gateway_definition();
    let registry = registry(vec![BackendRuntimeDriver::in_gateway(definition)]);
    let error = resolve_managed_backend(
        &registry,
        &env(&[
            ("AGENT_PROTOCOL", definition.id),
            (names::BACKEND_OWNERSHIP, "external"),
        ]),
    )
    .unwrap_err();
    // Upstream interpolates the driver *id* here, not the catalog label.
    assert!(matches!(
        error,
        ProcessError::ExternalServiceUnsupported { ref label } if label == definition.id
    ));
    assert!(
        error.message(Locale::Zh).starts_with(definition.id),
        "the refusal must name the driver id, not the label: {}",
        error.message(Locale::Zh),
    );
}

// ── permission mode ─────────────────────────────────────────────────────────

#[test]
fn an_always_full_backend_reports_full_whatever_is_configured() {
    let definition = always_full_definition();
    let registry = registry(vec![BackendRuntimeDriver::in_gateway(definition)]);
    for configured in [None, Some(PERMISSION_MODE_NATIVE)] {
        let mut environment = env(&[("AGENT_PROTOCOL", definition.id)]);
        if let Some(mode) = configured {
            environment.set(names::BACKEND_PERMISSION_MODE, mode);
        }
        let backend = resolve_managed_backend(&registry, &environment)
            .unwrap()
            .expect("a backend");
        assert_eq!(
            backend.permission_mode, PERMISSION_MODE_FULL,
            "configured {configured:?} must still report full",
        );
    }
}

#[test]
fn full_permission_is_refused_for_a_backend_that_cannot_support_it_safely() {
    let definition = no_full_permission_definition();
    let registry = registry(vec![BackendRuntimeDriver::in_gateway(definition)]);
    let error = resolve_managed_backend(
        &registry,
        &env(&[
            ("AGENT_PROTOCOL", definition.id),
            (names::BACKEND_PERMISSION_MODE, PERMISSION_MODE_FULL),
        ]),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProcessError::FullPermissionUnsafe { ref label } if label == definition.label
    ));
}

/// A driver with a refusal only it can express.
#[derive(Debug)]
struct RefusingHooks;

impl BackendRuntimeHooks for RefusingHooks {
    fn guard(&self, request: &ResolveRequest<'_>) -> Result<(), ProcessError> {
        if request.permission_mode == PERMISSION_MODE_FULL {
            return Err(ProcessError::DriverRefused {
                key: keys::BACKEND_FULL_PERMISSION_UNSAFE,
                arguments: vec![("label".to_owned(), "Fixture".to_owned())],
            });
        }
        Ok(())
    }
}

#[test]
fn a_drivers_own_refusal_wins_over_the_generic_one() {
    let definition = no_full_permission_definition();
    let mut driver = BackendRuntimeDriver::in_gateway(definition);
    driver.hooks = Some(std::sync::Arc::new(RefusingHooks));
    let registry = registry(vec![driver]);
    let error = resolve_managed_backend(
        &registry,
        &env(&[
            ("AGENT_PROTOCOL", definition.id),
            (names::BACKEND_PERMISSION_MODE, PERMISSION_MODE_FULL),
        ]),
    )
    .unwrap_err();
    assert!(
        matches!(error, ProcessError::DriverRefused { .. }),
        "the driver's own refusal must run before the generic one",
    );
}

#[test]
fn full_permission_requires_a_gateway_owned_service() {
    let definition = external_service_definition();
    let mut driver = service_driver(definition);
    // A driver that *would* accept full permission, so the ownership check is
    // the only thing that can refuse.
    driver.supports_full_permission = true;
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let error = resolve_managed_backend(
        &registry,
        &env(&[
            ("AGENT_PROTOCOL", definition.id),
            (names::BACKEND_PERMISSION_MODE, PERMISSION_MODE_FULL),
            (&service.base_url_environment, "http://127.0.0.1:18789"),
        ]),
    )
    .unwrap_err();
    assert!(matches!(error, ProcessError::FullPermissionRequiresOwned));
}

#[test]
fn apply_backend_permission_mode_re_checks_ownership() {
    // The second of upstream's two copies of the same refusal. It fires even
    // when handed a `ManagedBackend` that never went through the resolver,
    // which is the point: it mutates an environment a child will inherit.
    let definition = owned_service_definition();
    let driver = service_driver(definition);
    let mut environment = env(&[]);
    let error = via_process::apply_backend_permission_mode(
        &driver,
        &mut environment,
        &ManagedBackend {
            protocol: definition.id.to_owned(),
            ownership: Ownership::External,
            permission_mode: PERMISSION_MODE_FULL.to_owned(),
            base_url: None,
        },
    )
    .unwrap_err();
    assert!(matches!(error, ProcessError::FullPermissionRequiresOwned));
}

#[test]
fn an_unsupported_permission_mode_is_refused_and_is_not_trimmed() {
    for configured in ["danger", " full ", "FULL ", "native "] {
        let error =
            via_process::permission_mode(&env(&[(names::BACKEND_PERMISSION_MODE, configured)]))
                .unwrap_err();
        assert!(
            matches!(error, ProcessError::UnsupportedPermissionMode { .. }),
            "`{configured}` was accepted",
        );
    }
    // Case *is* folded, though.
    assert_eq!(
        via_process::permission_mode(&env(&[(names::BACKEND_PERMISSION_MODE, "FULL")])).unwrap(),
        PERMISSION_MODE_FULL,
    );
    // An empty assignment reads as unset.
    assert_eq!(
        via_process::permission_mode(&env(&[(names::BACKEND_PERMISSION_MODE, "")])).unwrap(),
        PERMISSION_MODE_NATIVE,
    );
}

#[test]
fn an_unsupported_ownership_is_refused_but_is_trimmed() {
    let definition = in_gateway_definition();
    let driver = BackendRuntimeDriver::in_gateway(definition);
    let error =
        via_process::backend_ownership(&driver, &env(&[(names::BACKEND_OWNERSHIP, "shared")]))
            .unwrap_err();
    assert!(matches!(
        error,
        ProcessError::UnsupportedOwnership { ref requested } if requested == "shared"
    ));
    // Unlike the permission mode, this one *is* trimmed and lowercased.
    assert_eq!(
        via_process::backend_ownership(&driver, &env(&[(names::BACKEND_OWNERSHIP, "  OWNED  ")]))
            .unwrap(),
        Ownership::Owned,
    );
}

// ── the loopback boundary ───────────────────────────────────────────────────

#[tokio::test]
async fn the_gateway_refuses_to_launch_a_backend_on_another_machine() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_OWNERSHIP, "owned"),
        (&service.base_url_environment, "https://agent.example.com"),
    ]);
    // The refusal must come *before* any socket is opened.
    let allocator = ScriptedAllocator::forbidden();
    let spawner = RecordingSpawner::forbidden();
    let error = start(&registry, &mut environment, &allocator, &spawner)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ProcessError::LocalOnly { ref url } if url == "https://agent.example.com"
    ));
    assert!(allocator.probed().is_empty());
}

// ── driver hooks ────────────────────────────────────────────────────────────

/// A driver that mints a credential and records that it ran.
#[derive(Debug)]
struct PreparingHooks;

impl BackendRuntimeHooks for PreparingHooks {
    fn prepare_environment(
        &self,
        env: &mut EnvMap,
        backend: &ManagedBackend,
    ) -> Result<(), ProcessError> {
        env.set("FIXTURE_TOKEN", format!("minted-for-{}", backend.protocol));
        Ok(())
    }

    fn apply_permission_mode(
        &self,
        env: &mut EnvMap,
        _backend: &ManagedBackend,
    ) -> Result<(), ProcessError> {
        env.set("FIXTURE_PERMISSION", "allow");
        Ok(())
    }
}

#[tokio::test]
async fn a_driver_prepares_the_environment_before_the_child_is_spawned() {
    let definition = owned_service_definition();
    let mut driver = service_driver(definition);
    driver.hooks = Some(std::sync::Arc::new(PreparingHooks));
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_PERMISSION_MODE, PERMISSION_MODE_FULL),
        (&service.base_url_environment, "http://127.0.0.1:4096"),
    ]);
    let allocator = ScriptedAllocator::new(false, "");
    let spawner = RecordingSpawner::new();
    start(&registry, &mut environment, &allocator, &spawner)
        .await
        .expect("the backend starts");

    let spec = spawner.only_spec();
    assert_eq!(
        spec.environment.get("FIXTURE_TOKEN"),
        Some(format!("minted-for-{}", definition.id).as_str()),
    );
    assert_eq!(spec.environment.get("FIXTURE_PERMISSION"), Some("allow"));
}

#[tokio::test]
async fn an_external_service_never_reaches_the_driver_environment_hook() {
    let definition = external_service_definition();
    let mut driver = service_driver(definition);
    driver.hooks = Some(std::sync::Arc::new(PreparingHooks));
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
    ]);
    start(
        &registry,
        &mut environment,
        &ScriptedAllocator::forbidden(),
        &RecordingSpawner::forbidden(),
    )
    .await
    .expect("an external backend resolves");
    assert_eq!(
        environment.get("FIXTURE_TOKEN"),
        None,
        "an external service is someone else's to configure",
    );
}

// ── the explicit environment allow-list ─────────────────────────────────────

#[tokio::test]
async fn a_user_supplied_allow_list_opens_the_boundary_only_where_asked() {
    let definition = explicit_list_definition();
    let allow_list = definition
        .environment
        .explicit_list_environment
        .expect("this backend declares one");
    let mut driver = BackendRuntimeDriver::in_gateway(definition);
    driver.separate_managed_process = true;
    driver.service = None;
    // A driver with no service cannot spawn; assert the projection directly.
    driver.separate_managed_process = false;

    let projected = via_process::backend_environment(
        &driver.environment,
        &env(&[
            (allow_list, "WANTED_ONE,WANTED_TWO"),
            ("WANTED_ONE", "1"),
            ("WANTED_TWO", "2"),
            ("UNWANTED", "3"),
            (names::AUTH_SECRET, "must not cross"),
        ]),
        &[],
    );
    assert_eq!(projected.get("WANTED_ONE"), Some("1"));
    assert_eq!(projected.get("WANTED_TWO"), Some("2"));
    assert_eq!(projected.get("UNWANTED"), None);
    assert_eq!(projected.get(names::AUTH_SECRET), None);
}

// ── the launch command ──────────────────────────────────────────────────────

#[tokio::test]
async fn a_launch_command_that_is_not_on_path_is_reported_as_not_installed() {
    let definition = owned_service_definition();
    let registry = registry(vec![service_driver(definition)]);
    let mut environment = env(&[("AGENT_PROTOCOL", definition.id)]);
    let allocator = ScriptedAllocator::new(false, "");
    let spawner = RecordingSpawner::forbidden();
    let options = StartOptions {
        root: PathBuf::from(ROOT),
        registry: &registry,
        platform: Platform::Posix,
        allocator: &allocator,
        spawner: &spawner,
        resolver: &MissingResolver,
        logger: None,
    };
    let error = start_managed_backend(&mut environment, &options)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ProcessError::CommandNotFound { ref command } if command == "fixture-runner"
    ));
    assert_eq!(
        via_process::backend_failure_code(&via_process::BackendFailure::from_process_error(
            &error,
            Locale::En
        )),
        via_process::BackendStatusCode::NotInstalled,
    );
}

#[test]
fn an_unknown_agent_protocol_is_refused() {
    let registry = BackendRuntimeRegistry::new();
    let error = resolve_managed_backend(&registry, &env(&[("AGENT_PROTOCOL", "no-such-backend")]))
        .unwrap_err();
    assert!(matches!(
        error,
        ProcessError::UnsupportedBackend { ref protocol } if protocol == "no-such-backend"
    ));
}
