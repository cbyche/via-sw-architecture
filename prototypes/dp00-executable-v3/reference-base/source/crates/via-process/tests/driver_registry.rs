//! The runtime half of `server/test/backend-driver-registry.test.mjs`.
//!
//! Upstream's file asserts both halves of the driver contract — the Agent
//! driver and the Runtime driver. Only the second is this crate's;
//! `every advertised backend has Agent and Runtime drivers` is reproduced here
//! for the runtime half, and the capability-contract assertions belong with
//! `via-downstream`.
//!
//! The refusals are exercised against synthetic definitions rather than real
//! ones, which is what lets a crate that may not name a backend still test
//! every branch of its validator.

mod common;

use common::{
    external_service_definition, in_gateway_definition, owned_service_definition, registry,
    service_driver, synthetic_definition,
};
use pretty_assertions::assert_eq;
use via_process::{
    BackendRuntimeDriver, BackendRuntimeRegistry, ManagedLaunch, ProcessError, ServiceAddress,
    normalize_backend_runtime_protocol, validate_runtime_driver,
};

/// The registry a complete `via-backends` would build: a service driver for
/// every backend the catalog gives an address, the manufactured in-Gateway
/// driver for the rest.
fn complete_registry() -> BackendRuntimeRegistry {
    registry(
        via_catalog::backend_definitions()
            .iter()
            .map(|definition| {
                if definition.base_url_environment.is_some() {
                    service_driver(definition)
                } else {
                    BackendRuntimeDriver::in_gateway(definition)
                }
            })
            .collect(),
    )
}

#[test]
fn every_catalogued_backend_resolves_to_a_runtime_driver() {
    // Upstream: "every advertised backend has Agent and Runtime drivers", and
    // the runtime driver's address declarations must agree with the catalog.
    let registry = complete_registry();
    for definition in via_catalog::backend_definitions() {
        let driver = registry
            .driver(definition.id)
            .unwrap_or_else(|error| panic!("`{}` has no runtime driver: {error}", definition.id));
        assert_eq!(driver.id, definition.id);
        assert_eq!(driver.label, definition.label);
        assert_eq!(
            driver.supports_external_service,
            definition.supports_external_service,
        );
        assert_eq!(
            driver
                .service
                .as_ref()
                .map(|s| s.base_url_environment.as_str()),
            definition.base_url_environment,
        );
        assert_eq!(
            driver.service.as_ref().map(|s| s.default_base_url.as_str()),
            definition.default_base_url,
        );
    }
}

#[test]
fn the_manufactured_in_gateway_driver_spawns_nothing() {
    // Upstream's `managedProcessDriver` factory: `separateManagedProcess:
    // false`, no `managedScript`, no address.
    for definition in via_catalog::backend_definitions() {
        assert_eq!(
            BackendRuntimeDriver::in_gateway(definition).environment,
            definition.environment,
            "the manufactured driver must carry the catalog's own policy",
        );
        let driver = BackendRuntimeDriver::in_gateway(definition);
        assert!(!driver.separate_managed_process, "{}", definition.id);
        assert!(driver.launch.is_none(), "{}", definition.id);
        assert!(driver.service.is_none(), "{}", definition.id);
        assert!(driver.hooks.is_none(), "{}", definition.id);
    }
}

#[test]
fn a_service_backend_may_not_fall_back_to_the_manufactured_driver() {
    // A strengthening over upstream. `managedProcessDriver` manufactures a
    // driver with no address, and upstream never reaches it for a backend that
    // has one only because its driver map is a module literal. VIA's registry
    // is built by the caller, so an unregistered service backend is reachable —
    // and silently dropping its address would leave the Gateway connecting
    // nowhere. It is refused instead.
    let registry = BackendRuntimeRegistry::new();
    for definition in via_catalog::backend_definitions() {
        if definition.base_url_environment.is_none() {
            continue;
        }
        assert!(
            matches!(
                registry.driver(definition.id),
                Err(ProcessError::DriverIdMismatch { .. })
            ),
            "`{}` fell back to an addressless driver",
            definition.id,
        );
    }
}

#[test]
fn a_registered_driver_wins_over_the_manufactured_one() {
    let definition = owned_service_definition();
    let registry = registry(vec![service_driver(definition)]);
    let resolved = registry
        .driver(definition.id)
        .expect("the registered driver");
    assert!(resolved.separate_managed_process);
    assert!(resolved.launch.is_some());
    assert_eq!(registry.registered(), vec![definition.id]);

    // Anything not registered still gets the manufactured one.
    let other = in_gateway_definition();
    assert!(
        !registry
            .driver(other.id)
            .expect("a manufactured driver")
            .separate_managed_process
    );
}

#[test]
fn the_registry_normalises_before_it_looks_up() {
    let registry = BackendRuntimeRegistry::new();
    let definition = in_gateway_definition();
    let spelled = format!("  {}  ", definition.id.to_uppercase());
    assert_eq!(registry.driver(&spelled).unwrap().id, definition.id);
    assert_eq!(normalize_backend_runtime_protocol(&spelled), definition.id);
}

#[test]
fn the_none_sentinel_and_an_unknown_id_are_both_refused_by_the_registry() {
    let registry = BackendRuntimeRegistry::new();
    for spelling in ["", "none", "NONE", "no-such-backend"] {
        let error = registry.driver(spelling).unwrap_err();
        assert!(
            matches!(error, ProcessError::UnsupportedBackend { .. }),
            "`{spelling}` resolved to a driver",
        );
    }
}

// ── validation ──────────────────────────────────────────────────────────────

/// A driver that agrees with a synthetic definition in every respect.
fn matching_driver(definition: &via_catalog::BackendDefinition) -> BackendRuntimeDriver {
    BackendRuntimeDriver {
        id: definition.id.to_owned(),
        label: definition.label.to_owned(),
        separate_managed_process: false,
        supports_external_service: definition.supports_external_service,
        supports_full_permission: definition.supports_full_permission,
        service: None,
        launch: None,
        environment: definition.environment,
        hooks: None,
    }
}

#[test]
fn a_consistent_driver_validates() {
    let definition = synthetic_definition("fixture", "Fixture");
    assert!(validate_runtime_driver(&matching_driver(&definition), &definition).is_ok());
}

#[test]
fn a_driver_whose_id_disagrees_is_refused() {
    let definition = synthetic_definition("fixture", "Fixture");
    let mut driver = matching_driver(&definition);
    driver.id = "other".to_owned();
    assert!(matches!(
        validate_runtime_driver(&driver, &definition),
        Err(ProcessError::DriverIdMismatch { .. })
    ));
}

#[test]
fn a_driver_whose_label_disagrees_is_refused() {
    let definition = synthetic_definition("fixture", "Fixture");
    let mut driver = matching_driver(&definition);
    driver.label = "Something Else".to_owned();
    assert!(matches!(
        validate_runtime_driver(&driver, &definition),
        Err(ProcessError::DriverIdMismatch { .. })
    ));
}

#[test]
fn a_driver_that_claims_an_external_service_it_has_no_right_to_is_refused() {
    let definition = synthetic_definition("fixture", "Fixture");
    let mut driver = matching_driver(&definition);
    driver.supports_external_service = true;
    assert!(matches!(
        validate_runtime_driver(&driver, &definition),
        Err(ProcessError::DriverExternalServiceMismatch { .. })
    ));
}

#[test]
fn a_driver_that_invents_a_service_address_is_refused() {
    // The synthetic definition has no base-URL variable, so a driver that
    // declares one disagrees with the catalog. This is the check upstream
    // makes in its registry *test* rather than in `validateRuntimeDriver`.
    let definition = synthetic_definition("fixture", "Fixture");
    let mut driver = matching_driver(&definition);
    driver.service = Some(ServiceAddress::http(
        "FIXTURE_BASE_URL",
        "http://127.0.0.1:1",
        "FIXTURE_PORT",
    ));
    assert!(matches!(
        validate_runtime_driver(&driver, &definition),
        Err(ProcessError::DriverIdMismatch { .. })
    ));
}

#[test]
fn a_driver_that_declares_a_process_without_a_launch_is_refused() {
    let definition = external_service_definition();
    let mut driver = service_driver(definition);
    driver.launch = None;
    assert!(matches!(
        validate_runtime_driver(&driver, definition),
        Err(ProcessError::DriverMissingManagedLaunch { .. })
    ));
}

#[test]
fn a_driver_that_declares_a_process_without_an_address_is_refused() {
    let definition = synthetic_definition("fixture", "Fixture");
    let mut driver = matching_driver(&definition);
    driver.separate_managed_process = true;
    driver.launch = Some(ManagedLaunch::new("fixture-runner", Vec::new()));
    assert!(matches!(
        validate_runtime_driver(&driver, &definition),
        Err(ProcessError::DriverMissingManagedLaunch { .. })
    ));
}

#[test]
fn registration_refuses_an_id_the_catalog_does_not_know() {
    let definition = synthetic_definition("fixture", "Fixture");
    let mut registry = BackendRuntimeRegistry::new();
    let error = registry.register(matching_driver(&definition)).unwrap_err();
    assert!(matches!(
        error,
        ProcessError::UnsupportedBackend { ref protocol } if protocol == "fixture"
    ));
    assert!(registry.registered().is_empty());
}

#[test]
fn registration_refuses_an_inconsistent_driver_before_it_is_stored() {
    let definition = in_gateway_definition();
    let mut driver = BackendRuntimeDriver::in_gateway(definition);
    driver.supports_external_service = !definition.supports_external_service;
    let mut registry = BackendRuntimeRegistry::new();
    assert!(matches!(
        registry.register(driver),
        Err(ProcessError::DriverExternalServiceMismatch { .. })
    ));
    assert!(
        registry.registered().is_empty(),
        "an invalid driver must not reach the registry",
    );
}

// ── service address helpers ─────────────────────────────────────────────────

#[test]
fn the_two_service_shapes_admit_the_schemes_they_advertise() {
    let http = ServiceAddress::http("X_BASE_URL", "http://127.0.0.1:1", "X_PORT");
    assert_eq!(
        http.protocols,
        vec!["http:".to_owned(), "https:".to_owned()]
    );

    let websocket = ServiceAddress::websocket("X_BASE_URL", "http://127.0.0.1:1", "X_PORT");
    assert_eq!(
        websocket.protocols,
        via_process::DEFAULT_SERVICE_PROTOCOLS
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    );
}
