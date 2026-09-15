//! Every contract value this crate owns, asserted against
//! `docs/reference/contracts.json`.
//!
//! Nothing here is retyped. The timing budgets are scanned out of the
//! catalogue's prose, the status codes are split out of its enumeration, and
//! the environment allow-lists are reconstructed from the contract's own
//! shorthand and pushed through `docs/rebrand.md`'s rename rule
//! programmatically. What the tests compare against is what
//! `via-process` actually publishes.

mod common;

use std::collections::BTreeSet;

use common::{
    contract_value, contract_why, env, in_gateway_definition, rebranded, registry,
    screaming_tokens, service_driver, timing_ms,
};
use pretty_assertions::assert_eq;
use via_catalog::Ownership;
use via_i18n::{Locale, keys};
use via_process::{
    ADDRESS_PROBE_TIMEOUT, AcpConnection, BACKEND_PERMISSION_MODES, BackendRuntimeDriver,
    BackendRuntimeRegistry, BackendRuntimeState, BackendRuntimeStateOptions, BackendStatusCode,
    BackendStatusKind, DEFAULT_BACKOFF_MS, ENV_LOADED_NAME, ENV_LOADED_VALUE, INTERNAL_NAMES,
    PERMISSION_MODE_FULL, PERMISSION_MODE_NATIVE, ProcessError, STOP_GRACE, SYSTEM_NAMES,
    SYSTEM_PREFIXES, ServiceAddress, TRANSPORT_ACP, normalize_backend_runtime_protocol,
    resolve_managed_backend, validate_runtime_driver,
};

// ── timing ──────────────────────────────────────────────────────────────────

#[test]
fn the_address_probe_budget_is_the_catalogued_one() {
    assert_eq!(
        i64::try_from(ADDRESS_PROBE_TIMEOUT.as_millis()).unwrap(),
        timing_ms("backendAddressInUse probe"),
    );
}

#[test]
fn the_stop_grace_is_the_catalogued_one() {
    assert_eq!(
        i64::try_from(STOP_GRACE.as_millis()).unwrap(),
        timing_ms("ManagedBackendRuntime stop grace"),
    );
}

#[test]
fn the_failure_backoff_is_the_catalogued_one() {
    assert_eq!(
        DEFAULT_BACKOFF_MS,
        timing_ms("BackendRuntimeState DEFAULT_BACKOFF_MS="),
    );
}

// ── the status vocabulary ───────────────────────────────────────────────────

/// Every `SCREAMING_SNAKE_CASE` code named by the two contracts that carry the
/// status vocabulary, minus the ones that belong to another module.
fn catalogued_codes() -> BTreeSet<String> {
    let mut codes = screaming_tokens(&contract_value("state-name", "BackendRuntimeState codes"));
    codes.extend(screaming_tokens(&contract_value(
        "error-code",
        "backend status codes",
    )));
    // `NOT_CONFIGURED` is `agent-client.mjs`'s answer for "no backend at all",
    // not a `BackendRuntimeState` value — the contract lists it because both
    // reach `/api/health.backend`.
    codes.remove("NOT_CONFIGURED");
    // Prose, not a code.
    codes.remove("DEFAULT_BACKOFF_MS");
    codes
}

#[test]
fn every_catalogued_status_code_exists_and_no_others() {
    let published: BTreeSet<String> = BackendStatusCode::all()
        .iter()
        .map(|code| code.as_str().to_owned())
        .collect();
    assert_eq!(published, catalogued_codes());
}

#[test]
fn the_four_status_strings_are_the_catalogued_ones() {
    let catalogue = contract_value("error-code", "backend status codes");
    let published = [
        BackendStatusKind::Stopped,
        BackendStatusKind::Starting,
        BackendStatusKind::Ready,
        BackendStatusKind::Failed,
    ];
    for kind in published {
        assert!(
            catalogue.contains(&format!("'{}'", kind.as_str())),
            "status `{}` is not in the catalogue",
            kind.as_str(),
        );
    }
    // `not_configured` is deliberately absent from this type; the catalogue
    // lists it because another module publishes it into the same field.
    assert!(catalogue.contains("'not_configured'"));
    assert!(
        !published
            .iter()
            .any(|kind| kind.as_str() == "not_configured")
    );
}

#[test]
fn exactly_three_codes_read_as_a_transient_cold_start() {
    // The set is pinned in the contract's rationale, not its value:
    // "gateway-application.mjs:461 treats exactly
    //  ['NOT_STARTED','STARTING','BACKEND_STARTING'] […] as a transient cold
    //  start".
    let why = contract_why("error-code", "backend status codes");
    let opening = why
        .find('[')
        .expect("the rationale names the transient set");
    let closing = why[opening..]
        .find(']')
        .expect("the rationale closes the transient set")
        + opening;
    let catalogued: BTreeSet<String> = screaming_tokens(&why[opening..=closing]);

    let published: BTreeSet<String> = BackendStatusCode::all()
        .iter()
        .filter(|code| code.is_transient())
        .map(|code| code.as_str().to_owned())
        .collect();
    assert_eq!(published, catalogued);
}

#[test]
fn every_value_carries_the_four_envelope_fields() {
    let catalogue = contract_value("state-name", "BackendRuntimeState codes");
    for field in ["protocol", "ownership", "transport", "acpConnection"] {
        assert!(catalogue.contains(field), "the envelope has no `{field}`");
    }
    assert!(catalogue.contains("transport:'acp'"));
    assert!(catalogue.contains("'process'"));
    assert_eq!(TRANSPORT_ACP, "acp");
    assert_eq!(AcpConnection::Process.as_str(), "process");

    let state = BackendRuntimeState::new(BackendRuntimeStateOptions::new(
        "fixture",
        Ownership::Owned,
        Some(AcpConnection::Process),
        "Fixture",
    ));
    let value = state.value();
    assert_eq!(value["transport"], "acp");
    assert_eq!(value["acpConnection"], "process");
    assert_eq!(value["ownership"], "owned");
    assert_eq!(value["protocol"], "fixture");
}

#[test]
fn the_idle_status_deep_equals_the_catalogued_shape() {
    // `error-code/backend adapter status codes` pins the idle value exactly.
    let catalogue = contract_value("error-code", "backend adapter status codes");
    let opening = catalogue.find('{').expect("the contract shows the shape");
    let shape = &catalogue[opening..];

    let state = BackendRuntimeState::new(BackendRuntimeStateOptions::new(
        "fixture",
        Ownership::Owned,
        Some(AcpConnection::Process),
        "Fixture",
    ));
    let value = state.status(true);

    assert_eq!(value["ok"], false);
    assert_eq!(value["status"], "stopped");
    assert_eq!(value["code"], "NOT_STARTED");
    assert_eq!(value["ownership"], "owned");
    assert_eq!(value["transport"], "acp");
    assert_eq!(value["acpConnection"], "process");
    // No error, no retryAfterMs, no capabilities on an idle value.
    assert_eq!(
        value.keys().map(String::as_str).collect::<Vec<_>>(),
        vec![
            "ok",
            "status",
            "code",
            "protocol",
            "ownership",
            "transport",
            "acpConnection"
        ],
    );
    for key in value.keys() {
        assert!(
            shape.contains(key.as_str()),
            "`{key}` is not in the catalogued idle shape",
        );
    }
}

#[test]
fn the_configured_status_publishes_only_catalogued_fields() {
    // `json-field//api/health.backend` lists what a configured status() may
    // carry. Every key this crate can emit has to be in that list.
    let catalogue = contract_value("json-field", "/api/health.backend");
    let mut state = BackendRuntimeState::new(BackendRuntimeStateOptions::new(
        "fixture",
        Ownership::Owned,
        Some(AcpConnection::Process),
        "Fixture",
    ));
    let mut emitted: BTreeSet<String> = state.value().keys().cloned().collect();
    state.ready(None);
    emitted.extend(state.value().keys().cloned());
    state.failed(&via_process::BackendFailure::message("boom"));
    emitted.extend(state.status(true).keys().cloned());
    state.starting("");
    emitted.extend(state.value().keys().cloned());
    state.waiting("waiting");
    emitted.extend(state.value().keys().cloned());
    state.stopped();
    emitted.extend(state.value().keys().cloned());

    for key in &emitted {
        assert!(
            catalogue.contains(key.as_str()),
            "`{key}` is published but is not a catalogued /api/health.backend field",
        );
    }
    for required in ["ok", "status", "code", "error", "retryAfterMs", "agentInfo"] {
        assert!(
            emitted.contains(required),
            "the catalogued field `{required}` is never published",
        );
    }
}

// ── the child environment trust boundary ────────────────────────────────────

/// The names inside the first parenthesised list of the allow-list contract.
fn catalogued_system_names() -> Vec<String> {
    let catalogue = contract_value("env-var", "backend child environment allowlist");
    let opening = catalogue
        .find('(')
        .expect("the contract lists the OS names");
    let closing = catalogue.find(')').expect("the list closes");
    catalogue[opening + 1..closing]
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_system_allow_list_is_the_catalogued_one_in_order() {
    assert_eq!(
        SYSTEM_NAMES
            .iter()
            .map(|n| (*n).to_owned())
            .collect::<Vec<_>>(),
        catalogued_system_names(),
    );
}

#[test]
fn the_system_prefixes_are_the_catalogued_ones() {
    let catalogue = contract_value("env-var", "backend child environment allowlist");
    let marker = catalogue
        .find("prefixes ")
        .expect("the contract names the prefixes");
    let tail = &catalogue[marker + "prefixes ".len()..];
    let listed = tail
        .split(" plus ")
        .next()
        .expect("the prefix list is terminated");
    let catalogued: Vec<String> = listed
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect();
    assert_eq!(
        SYSTEM_PREFIXES
            .iter()
            .map(|p| (*p).to_owned())
            .collect::<Vec<_>>(),
        catalogued,
    );
}

/// Expand the contract's shorthand for the internal name set.
///
/// The catalogue writes
/// `QWEN_AUDIO_AGENT_BACKEND_AGENT/_MODEL/_OWNERSHIP/_PERMISSION_MODE,
///  QWEN_AUDIO_AGENT_DESKTOP, _DESKTOP_INSTALLED_ONLY, …`. Two rules make that
/// unambiguous: a `/`-continuation replaces the final `_`-segment of the full
/// name it follows, and a comma-separated token that opens with `_` is
/// `QWEN_AUDIO_AGENT` plus that token.
fn catalogued_internal_names() -> Vec<String> {
    let catalogue = contract_value("env-var", "child-process environment filter");
    let marker = catalogue
        .find("INTERNAL_NAMES set (")
        .expect("the contract names the internal set");
    let opening = marker + "INTERNAL_NAMES set (".len();
    let closing = catalogue[opening..]
        .find(')')
        .expect("the internal set closes")
        + opening;

    let mut names = Vec::new();
    for token in catalogue[opening..closing].split(',').map(str::trim) {
        let mut pieces = token.split('/');
        let Some(head) = pieces.next() else { continue };
        let full = if head.starts_with('_') {
            format!("QWEN_AUDIO_AGENT{head}")
        } else {
            head.to_owned()
        };
        let stem = full
            .rsplit_once('_')
            .map(|(stem, _)| stem.to_owned())
            .unwrap_or_else(|| full.clone());
        names.push(full);
        for suffix in pieces {
            names.push(format!("{stem}{suffix}"));
        }
    }
    names.sort_unstable();
    names
}

#[test]
fn the_internal_allow_list_is_the_catalogued_one_after_the_rename() {
    let catalogued: Vec<String> = catalogued_internal_names()
        .iter()
        .map(|name| rebranded(name))
        .collect();
    let mut published: Vec<String> = INTERNAL_NAMES.iter().map(|n| (*n).to_owned()).collect();
    published.sort_unstable();
    assert_eq!(published, catalogued);
}

#[test]
fn the_env_loaded_stamp_is_the_catalogued_one_after_the_rename() {
    let catalogue = contract_value("env-var", "child-process environment filter");
    assert!(catalogue.contains("QWEN_AUDIO_AGENT_ENV_LOADED='1'"));
    assert_eq!(ENV_LOADED_NAME, rebranded("QWEN_AUDIO_AGENT_ENV_LOADED"));
    assert_eq!(ENV_LOADED_VALUE, "1");
}

#[test]
fn the_gateway_auth_secret_is_not_on_any_allow_list() {
    // The security half of `env-var/backend and security environment`:
    // "QWEN_AUDIO_AGENT_AUTH_SECRET must NEVER appear in a spawned backend's
    // environment".
    let catalogue = contract_value("env-var", "backend and security environment");
    let secret = rebranded(
        screaming_tokens(&catalogue)
            .into_iter()
            .find(|token| token.ends_with("_AUTH_SECRET"))
            .expect("the contract names the auth secret")
            .as_str(),
    );
    assert!(!SYSTEM_NAMES.contains(&secret.as_str()));
    assert!(!INTERNAL_NAMES.contains(&secret.as_str()));
    assert!(
        !SYSTEM_PREFIXES
            .iter()
            .any(|prefix| secret.starts_with(prefix))
    );
}

// ── configuration vocabulary ────────────────────────────────────────────────

#[test]
fn the_permission_modes_are_the_catalogued_ones_with_native_first() {
    let catalogue = contract_value("env-var", "QWEN_AUDIO_AGENT_BACKEND_PERMISSION_MODE");
    assert!(catalogue.contains("'native' (default)"));
    assert!(catalogue.contains("'full'"));
    assert_eq!(
        BACKEND_PERMISSION_MODES,
        &[PERMISSION_MODE_NATIVE, PERMISSION_MODE_FULL],
    );
    // The default is security-relevant and must stay `native`.
    assert_eq!(
        via_process::permission_mode(&env(&[])).unwrap(),
        PERMISSION_MODE_NATIVE,
    );
}

#[test]
fn the_ownership_values_are_the_catalogued_ones() {
    let catalogue = contract_value("env-var", "QWEN_AUDIO_AGENT_BACKEND_OWNERSHIP");
    assert!(catalogue.contains("'' (auto) | 'owned' | 'external'"));
    assert_eq!(Ownership::Owned.as_str(), "owned");
    assert_eq!(Ownership::External.as_str(), "external");
}

#[test]
fn the_none_sentinel_normalises_to_frontend_only() {
    let catalogue = contract_value("env-var", "AGENT_PROTOCOL");
    assert!(catalogue.contains("'none' (normalised to '')"));
    assert_eq!(normalize_backend_runtime_protocol("none"), "");
    assert_eq!(normalize_backend_runtime_protocol("  NONE  "), "");
    assert_eq!(normalize_backend_runtime_protocol(""), "");
    assert!(
        resolve_managed_backend(
            &BackendRuntimeRegistry::new(),
            &env(&[("AGENT_PROTOCOL", "none")])
        )
        .unwrap()
        .is_none()
    );
}

// ── the three declarations of a service address ─────────────────────────────

/// `(id, default base URL, base-URL variable, port variable)` for every
/// service backend the catalogue names.
fn catalogued_service_addresses() -> Vec<(String, String, String, String)> {
    let catalogue = contract_value("default-value", "backend default base URLs");
    catalogue
        .split(';')
        .filter_map(|row| {
            let row = row.trim();
            let (id, rest) = row.split_once(' ')?;
            let (url, rest) = rest.split_once(" (env ")?;
            let (base_env, rest) = rest.split_once(", port env ")?;
            let port_env = rest.trim_end_matches(')');
            Some((
                id.to_owned(),
                url.to_owned(),
                base_env.to_owned(),
                port_env.to_owned(),
            ))
        })
        .collect()
}

#[test]
fn a_service_driver_must_agree_with_the_catalogued_address() {
    let rows = catalogued_service_addresses();
    assert!(!rows.is_empty(), "the catalogue names no service backend");
    for (id, url, base_env, port_env) in rows {
        let definition =
            via_catalog::backend_definition(&id).expect("a catalogued service backend");
        assert_eq!(definition.base_url_environment, Some(base_env.as_str()));
        assert_eq!(definition.default_base_url, Some(url.as_str()));

        let mut driver = BackendRuntimeDriver::in_gateway(definition);
        driver.separate_managed_process = true;
        driver.service = Some(ServiceAddress::websocket(
            base_env.clone(),
            url.clone(),
            port_env.clone(),
        ));
        driver.launch = Some(via_process::ManagedLaunch::new("fixture", Vec::new()));
        validate_runtime_driver(&driver, definition)
            .unwrap_or_else(|error| panic!("`{id}` should validate: {error}"));

        // …and a driver that disagrees with the catalogue must not register.
        let mut wrong = driver.clone();
        wrong.service = Some(ServiceAddress::websocket(
            base_env,
            format!("{url}0"),
            port_env,
        ));
        assert!(
            matches!(
                validate_runtime_driver(&wrong, definition),
                Err(ProcessError::DriverIdMismatch { .. })
            ),
            "`{id}` validated against a default base URL the catalogue does not declare",
        );
    }
}

// ── the refusals ────────────────────────────────────────────────────────────

/// A refusal, rendered with a sentinel in place of its one interpolation.
const SENTINEL: &str = "\u{2603}";

fn rendered(error: &ProcessError, placeholder: &str) -> String {
    error.message(Locale::Zh).replace(SENTINEL, placeholder)
}

#[test]
fn every_runtime_and_ownership_refusal_is_the_catalogued_sentence() {
    let catalogue = contract_value("error-code", "runtime/ownership errors");
    let cases: Vec<(ProcessError, &str)> = vec![
        (
            ProcessError::UnsupportedPermissionMode {
                requested: SENTINEL.to_owned(),
            },
            "<mode>",
        ),
        (
            ProcessError::UnsupportedOwnership {
                requested: SENTINEL.to_owned(),
            },
            "<v>",
        ),
        (
            ProcessError::ExternalServiceUnsupported {
                label: SENTINEL.to_owned(),
            },
            "<label>",
        ),
        (ProcessError::FullPermissionRequiresOwned, ""),
        (
            ProcessError::MustBeGatewayStarted {
                label: SENTINEL.to_owned(),
            },
            "<label>",
        ),
        (
            ProcessError::LocalOnly {
                url: SENTINEL.to_owned(),
            },
            "<url>",
        ),
        (
            ProcessError::UnsupportedServiceProtocol {
                protocol: SENTINEL.to_owned(),
            },
            "<protocol>",
        ),
        (ProcessError::ServiceUrlHasCredentials, ""),
        (
            ProcessError::FullPermissionUnsafe {
                label: SENTINEL.to_owned(),
            },
            "<label>",
        ),
    ];
    for (error, placeholder) in cases {
        let sentence = rendered(&error, placeholder);
        assert!(
            catalogue.contains(&sentence),
            "`{sentence}` is not one of the catalogued runtime/ownership refusals",
        );
    }
}

#[test]
fn every_driver_validation_refusal_is_the_catalogued_sentence() {
    let catalogue = contract_value("error-code", "driver validation errors");
    let cases = vec![
        ProcessError::UnsupportedBackend {
            protocol: SENTINEL.to_owned(),
        },
        ProcessError::DriverIdMismatch {
            id: SENTINEL.to_owned(),
        },
        ProcessError::DriverExternalServiceMismatch {
            id: SENTINEL.to_owned(),
        },
        ProcessError::DriverMissingManagedLaunch {
            id: SENTINEL.to_owned(),
        },
    ];
    for error in cases {
        let sentence = rendered(&error, "<id>");
        assert!(
            catalogue.contains(&sentence),
            "`{sentence}` is not one of the catalogued driver validation refusals",
        );
    }
}

#[test]
fn the_three_service_endpoint_rejections_are_catalogued() {
    let catalogue = contract_value("error-code", "service endpoint rejections");
    for fragment in [
        "不支持的后台服务地址协议",
        "不能包含用户名或密码",
        "不支持连接外部后台服务",
    ] {
        assert!(
            catalogue.contains(fragment),
            "`{fragment}` is not catalogued"
        );
    }
    assert!(
        rendered(
            &ProcessError::UnsupportedServiceProtocol {
                protocol: SENTINEL.to_owned()
            },
            "",
        )
        .contains("不支持的后台服务地址协议")
    );
    assert!(
        ProcessError::ServiceUrlHasCredentials
            .message(Locale::Zh)
            .contains("不能包含用户名或密码")
    );
    assert!(
        rendered(
            &ProcessError::ExternalServiceUnsupported {
                label: SENTINEL.to_owned()
            },
            "",
        )
        .contains("不支持连接外部后台服务")
    );
}

#[test]
fn a_driver_may_carry_its_own_refusal_key() {
    // `DriverRefused` is the escape hatch a backend-specific driver uses for a
    // sentence only it can write. The key travels, not the rendered string, so
    // the refusal is still localized where it is displayed.
    let error = ProcessError::DriverRefused {
        key: keys::BACKEND_FULL_PERMISSION_UNSAFE,
        arguments: vec![("label".to_owned(), SENTINEL.to_owned())],
    };
    let catalogue = contract_value("error-code", "runtime/ownership errors");
    assert!(catalogue.contains(&rendered(&error, "<label>")));
    // Every locale renders, and none of them is the key name.
    for locale in [Locale::En, Locale::Zh, Locale::Ko] {
        let message = error.message(locale);
        assert!(
            !message.contains("backend.full_permission_unsafe"),
            "{message}"
        );
        assert!(!message.contains('{'), "{message}");
    }
}

// ── "no readiness probe, no restart" ────────────────────────────────────────

#[tokio::test]
async fn a_failed_spawn_is_reported_once_and_never_retried() {
    // `state-name/managed backend readiness / restart / shutdown`:
    // "There is NO readiness probe and NO restart in managed-backend.mjs".
    let catalogue = contract_value(
        "state-name",
        "managed backend readiness / restart / shutdown",
    );
    assert!(catalogue.contains("NO readiness probe"));
    assert!(catalogue.contains("NO restart"));

    let definition = common::owned_service_definition();
    let registry = registry(vec![service_driver(definition)]);
    let spawner = common::RecordingSpawner::failing("no such file");
    let allocator = common::ScriptedAllocator::new(false, "");
    let mut environment = env(&[("AGENT_PROTOCOL", definition.id)]);
    let options = via_process::StartOptions {
        root: std::path::PathBuf::from("/repo"),
        registry: &registry,
        platform: via_core::search_path::Platform::Posix,
        allocator: &allocator,
        spawner: &spawner,
        resolver: &common::FixedResolver::new("/opt/fixture/bin"),
        logger: None,
    };
    let error = via_process::start_managed_backend(&mut environment, &options)
        .await
        .unwrap_err();
    assert!(matches!(error, ProcessError::Io { .. }));
    assert_eq!(
        spawner.specs.lock().unwrap().len(),
        1,
        "a failed spawn must not be retried",
    );
}

#[test]
fn an_in_gateway_backend_spawns_nothing_and_has_no_address() {
    let definition = in_gateway_definition();
    let registry = registry(vec![BackendRuntimeDriver::in_gateway(definition)]);
    let backend = resolve_managed_backend(&registry, &env(&[("AGENT_PROTOCOL", definition.id)]))
        .unwrap()
        .expect("a backend");
    assert_eq!(backend.ownership, Ownership::Owned);
    assert_eq!(backend.base_url, None);
    assert_eq!(backend.permission_mode, PERMISSION_MODE_NATIVE);
}
