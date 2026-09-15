//! The two backends the Gateway starts as a service, as `via-process` data.
//!
//! Ported from `server/src/process/backend-drivers/{registry,opencode,openclaw}.mjs`.
//! `via-process` may not name a backend — `via-arch-test` fails the build if it
//! ever depends on this crate — so upstream's two driver literals become
//! [`via_process::BackendRuntimeDriver`] values built here, and the three
//! behaviours a driver adds beyond data arrive as
//! [`via_process::BackendRuntimeHooks`] implementations.
//!
//! `state-name/managed backend spawn spec` records the shape this preserves:
//! *"Only two backends spawn a supervised process; the rest are in-process ACP
//! stdio clients started elsewhere."* The other ten reach
//! `BackendRuntimeDriver::in_gateway` through the registry's fallback, which is
//! upstream's `managedProcessDriver(definition)`.
//!
//! # The managed argv carries the port, and resolves it at spawn time
//!
//! Upstream spawns `node scripts/openclaw-gateway.mjs`, and that shim reads
//! `process.env.OPENCLAW_PORT` at start-up — after the Gateway has published a
//! possibly-reallocated port into the environment. VIA spawns the binary, so
//! the port is on the argv instead, and that is two problems rather than one.
//!
//! [`managed_service_port`] answers the *configuration* half: the port is
//! derived from the service's own `<X>_BASE_URL` rather than from an unrelated
//! constant, so a custom base URL and the child's listen port cannot disagree.
//!
//! The *reallocation* half is answered by emitting a placeholder —
//! `${OPENCLAW_PORT:-<derived>}` — which [`via_process::spawn_spec`] expands
//! from the child's own environment at spawn time, which is the same moment
//! upstream's shim would read it. `start_managed_backend` may find the
//! requested address busy, allocate another, and publish it through
//! `via_process::apply_backend_address`; because the argv is resolved after
//! that, the child is told the port VIA actually reserved.
//!
//! The fallback in the placeholder is what this derived before the placeholder
//! existed, so a path that never reaches `apply_backend_address` still launches
//! on the configured port.

use std::sync::Arc;

use serde_json::{Map, Value};
use via_catalog::{BackendDefinition, backend_definition, backend_definitions};
use via_core::EnvMap;
use via_core::config::backend::resolve_opencode_coordinator_agent;
use via_i18n::keys;
use via_process::{
    BackendRuntimeDriver, BackendRuntimeHooks, BackendRuntimeRegistry, ManagedBackend,
    ManagedLaunch, PERMISSION_MODE_FULL, ProcessError, ResolveRequest, ServiceAddress,
    service_endpoint_port,
};

use crate::openclaw;

/// `OPENCODE_PORT` — where the resolved OpenCode port is published.
pub const OPENCODE_PORT: &str = "OPENCODE_PORT";
/// `OPENCLAW_PORT` — where the resolved OpenClaw port is published.
pub const OPENCLAW_PORT: &str = "OPENCLAW_PORT";

/// OpenCode's default managed port.
///
/// The port half of `default-value/backend default base URLs`
/// (`http://127.0.0.1:4096`); the shim's own default
/// (`scripts/opencode.mjs:16`).
///
/// Kept as the last-resort fallback for [`managed_service_port`], not as the
/// first answer: it and the default base URL's port are the same number by
/// construction, so the derivation subsumes it.
pub const OPENCODE_DEFAULT_PORT: &str = "4096";

/// `<X>_BASE_URL` → `<X>_PORT`.
///
/// **External contract** — `env-var/backend env derived from base URL`
/// (`cli/src/runtime.mjs:284-287`):
/// `definition.baseUrlEnvironment.replace(/_BASE_URL$/, '_PORT')`. The regex is
/// anchored at the end, so a variable that merely *contains* `_BASE_URL` is
/// returned unchanged — as `String.prototype.replace` with a non-matching
/// pattern does.
#[must_use]
pub fn port_environment(base_url_environment: &str) -> String {
    match base_url_environment.strip_suffix("_BASE_URL") {
        Some(prefix) => format!("{prefix}_PORT"),
        None => base_url_environment.to_owned(),
    }
}

/// The port a managed service should listen on, derived from its own base URL.
///
/// **External contract** — `env-var/backend env derived from base URL`
/// (`cli/src/runtime.mjs:282-291`). Upstream computes the value from the base
/// URL itself: the URL's explicit port, else `443` for `https:`/`wss:` and `80`
/// otherwise. [`via_process::service_endpoint_port`] is that exact table.
///
/// The order is: an explicit `<X>_PORT` wins, then the configured
/// `<X>_BASE_URL`, then the catalogued default base URL.
///
/// # Why this is a derivation and not a constant
///
/// VIA shipped both halves — the base URL and a hardcoded default port — with
/// no rule joining them, so `OPENCODE_BASE_URL=http://127.0.0.1:9999` with no
/// matching `OPENCODE_PORT` left VIA routing to `9999` while the child it
/// launched listened on `4096`. That is a functional mismatch, not a
/// conformance gap.
///
/// Deriving does not move the default: the hardcoded constants and their
/// default base URLs' ports are the same numbers by construction, so with
/// nothing overridden this still answers `4096` for OpenCode and `18789` for
/// OpenClaw. The constants remain as the last resort for a definition that
/// carries no default base URL, which neither shipped service does.
#[must_use]
pub fn managed_service_port(
    definition: &BackendDefinition,
    env: &EnvMap,
    fallback: &str,
) -> String {
    let Some(base_url_environment) = definition.base_url_environment else {
        return fallback.to_owned();
    };
    if let Some(explicit) = non_empty(env.get_trimmed(&port_environment(base_url_environment))) {
        return explicit;
    }
    let configured = non_empty(env.get_trimmed(base_url_environment))
        .or_else(|| non_empty(definition.default_base_url.unwrap_or_default()));
    configured
        .and_then(|base_url| service_endpoint_port(&base_url).ok())
        .map_or_else(|| fallback.to_owned(), |port| port.to_string())
}

/// The loopback host the managed OpenCode server binds.
///
/// **External contract** — `scripts/opencode.mjs:74`
/// (`serve --hostname 127.0.0.1 --port <PORT>`). Binding loopback is a security
/// property: an OpenCode server on `0.0.0.0` is an unauthenticated code
/// execution surface.
pub const OPENCODE_BIND_HOST: &str = "127.0.0.1";

/// `OPENCODE_CONFIG_CONTENT` — the inline OpenCode configuration.
pub const OPENCODE_CONFIG_CONTENT: &str = "OPENCODE_CONFIG_CONTENT";
/// `OPENCODE_TASK_AGENT` — the sub-agent OpenCode delegates tasks to.
pub const OPENCODE_TASK_AGENT: &str = "OPENCODE_TASK_AGENT";

/// OpenCode's default task agent.
///
/// **External contract** — `env-var/OPENCODE_CONFIG_CONTENT / OPENCODE_TASK_AGENT`:
/// *"`OPENCODE_TASK_AGENT` default `build`"*, consumed by OpenCode itself.
pub const OPENCODE_DEFAULT_TASK_AGENT: &str = "build";

/// The permission value written into OpenCode's configuration in `full` mode.
///
/// **External contract** — the literal `allow`, consumed by OpenCode.
pub const OPENCODE_PERMISSION_ALLOW: &str = "allow";

/// The managed launch for one backend, or `None` for the ten with no service.
///
/// Reads the port from `env` — see the module docs on why that is a deviation.
#[must_use]
pub fn managed_launch(protocol: &str, env: &EnvMap) -> Option<ManagedLaunch> {
    match protocol {
        "opencode" => {
            // The port is a **placeholder**, not a value: `via-process`
            // expands it in `spawn_spec` from the child's own environment, so
            // a port `start_managed_backend` reallocated reaches the command
            // line and not just the environment. The fallback is what this
            // derived before the placeholder existed, so a path that never
            // reaches `apply_backend_address` still gets the configured port.
            let fallback = backend_definition(protocol).map_or_else(
                || OPENCODE_DEFAULT_PORT.to_owned(),
                |definition| managed_service_port(definition, env, OPENCODE_DEFAULT_PORT),
            );
            Some(ManagedLaunch::new(
                "opencode",
                vec![
                    "serve".to_owned(),
                    "--hostname".to_owned(),
                    OPENCODE_BIND_HOST.to_owned(),
                    "--port".to_owned(),
                    format!("${{{OPENCODE_PORT}:-{fallback}}}"),
                ],
            ))
        }
        "openclaw" => Some(ManagedLaunch::new(
            "openclaw",
            openclaw::managed_gateway_arguments(env),
        )),
        _ => None,
    }
}

/// OpenCode's runtime driver — `backend-drivers/opencode.mjs:32-84`.
#[must_use]
pub fn opencode_runtime_driver(
    definition: &BackendDefinition,
    env: &EnvMap,
) -> BackendRuntimeDriver {
    BackendRuntimeDriver {
        separate_managed_process: true,
        service: Some(ServiceAddress::http(
            definition.base_url_environment.unwrap_or_default(),
            definition.default_base_url.unwrap_or_default(),
            OPENCODE_PORT,
        )),
        launch: managed_launch(definition.id, env),
        hooks: Some(Arc::new(OpenCodeHooks)),
        ..BackendRuntimeDriver::in_gateway(definition)
    }
}

/// OpenClaw's runtime driver — `backend-drivers/openclaw.mjs:7-52`.
///
/// The only driver that admits `ws:`/`wss:`, and the only one where an explicit
/// base URL flips ownership to `external`.
#[must_use]
pub fn openclaw_runtime_driver(
    definition: &BackendDefinition,
    env: &EnvMap,
) -> BackendRuntimeDriver {
    BackendRuntimeDriver {
        separate_managed_process: true,
        service: Some(ServiceAddress::websocket(
            definition.base_url_environment.unwrap_or_default(),
            definition.default_base_url.unwrap_or_default(),
            OPENCLAW_PORT,
        )),
        launch: managed_launch(definition.id, env),
        hooks: Some(Arc::new(OpenClawHooks)),
        ..BackendRuntimeDriver::in_gateway(definition)
    }
}

/// Every catalogued backend's runtime driver, registered and validated.
///
/// The two service backends get their dedicated drivers; the other ten reach
/// `BackendRuntimeDriver::in_gateway` through
/// [`BackendRuntimeRegistry::driver`]'s fallback, which is exactly upstream's
/// `managedProcessDriver`. Registering them explicitly would be the same values
/// twice.
///
/// # Errors
///
/// Whatever [`via_process::validate_runtime_driver`] refuses with — which is
/// the point of building the registry rather than the drivers: a driver whose
/// address disagrees with the catalog is rejected here, at startup.
pub fn runtime_registry(env: &EnvMap) -> Result<BackendRuntimeRegistry, ProcessError> {
    let mut registry = BackendRuntimeRegistry::new();
    for definition in backend_definitions() {
        let driver = match definition.id {
            "opencode" => opencode_runtime_driver(definition, env),
            "openclaw" => openclaw_runtime_driver(definition, env),
            _ => continue,
        };
        registry.register(driver)?;
    }
    Ok(registry)
}

/// OpenCode's one hook: rewriting its inline configuration for `full` mode.
#[derive(Debug, Clone, Copy)]
struct OpenCodeHooks;

impl BackendRuntimeHooks for OpenCodeHooks {
    /// `applyPermissionMode(env, backend)` —
    /// `backend-drivers/opencode.mjs:53-77`.
    ///
    /// In `full` mode the inline configuration gains `permission: 'allow'` at
    /// the top level and on two named agents: the configured coordinator agent,
    /// and OpenCode's task agent. Both are needed — OpenCode resolves
    /// permission per agent, so a top-level `allow` alone still leaves a
    /// sub-agent asking.
    fn apply_permission_mode(
        &self,
        env: &mut EnvMap,
        backend: &ManagedBackend,
    ) -> Result<(), ProcessError> {
        if backend.permission_mode != PERMISSION_MODE_FULL {
            return Ok(());
        }
        let mut config = inline_config(env.get_trimmed(OPENCODE_CONFIG_CONTENT))?;
        let mut agents = config
            .get("agent")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let task_agent = non_empty(env.get_trimmed(OPENCODE_TASK_AGENT))
            .unwrap_or_else(|| OPENCODE_DEFAULT_TASK_AGENT.to_owned());
        let mut names = vec![resolve_opencode_coordinator_agent(env), task_agent];
        names.dedup();
        for name in names {
            if name.is_empty() {
                continue;
            }
            let mut existing = agents
                .get(&name)
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            existing.insert(
                "permission".to_owned(),
                OPENCODE_PERMISSION_ALLOW.to_owned().into(),
            );
            agents.insert(name, Value::Object(existing));
        }
        config.insert(
            "permission".to_owned(),
            OPENCODE_PERMISSION_ALLOW.to_owned().into(),
        );
        config.insert("agent".to_owned(), Value::Object(agents));
        env.set(OPENCODE_CONFIG_CONTENT, Value::Object(config).to_string());
        Ok(())
    }
}

/// `inlineConfig(value)` — `backend-drivers/opencode.mjs:6-18`.
///
/// An empty value is an empty object; anything that is not a JSON object is a
/// hard error, because silently discarding a user's configuration would turn a
/// typo into an unexplained permission change.
fn inline_config(value: &str) -> Result<Map<String, Value>, ProcessError> {
    if value.trim().is_empty() {
        return Ok(Map::new());
    }
    let parsed: Value = serde_json::from_str(value).map_err(|_| ProcessError::DriverRefused {
        key: keys::BACKEND_OPENCODE_CONFIG_NOT_JSON,
        arguments: Vec::new(),
    })?;
    match parsed {
        Value::Object(object) => Ok(object),
        _ => Err(ProcessError::DriverRefused {
            key: keys::BACKEND_OPENCODE_CONFIG_NOT_OBJECT,
            arguments: Vec::new(),
        }),
    }
}

/// OpenClaw's two hooks: a bespoke refusal and a minted transport credential.
#[derive(Debug, Clone, Copy)]
struct OpenClawHooks;

impl BackendRuntimeHooks for OpenClawHooks {
    /// `resolve`'s first statement — `backend-drivers/openclaw.mjs:17-20`.
    ///
    /// OpenClaw's own full permission is three separate settings inside a
    /// third-party product (`exec approvals`, `elevated`, `host`). The
    /// Gateway's single switch cannot stand in for them, so this is a refusal
    /// rather than a best effort.
    fn guard(&self, request: &ResolveRequest<'_>) -> Result<(), ProcessError> {
        if request.permission_mode == PERMISSION_MODE_FULL {
            return Err(ProcessError::DriverRefused {
                key: keys::BACKEND_OPENCLAW_FULL_PERMISSION_UNSAFE,
                arguments: Vec::new(),
            });
        }
        Ok(())
    }

    /// `prepareEnvironment(env, backend)` —
    /// `backend-drivers/openclaw.mjs:41-51`.
    ///
    /// When VIA is managing OpenClaw against Bailian and no transport token is
    /// configured, one is minted. It is deliberately **not** the Gateway's
    /// identity secret: *"Gateway transport authentication is independent from
    /// the local user identity signing secret and remains private to this
    /// backend plugin."*
    fn prepare_environment(
        &self,
        env: &mut EnvMap,
        backend: &ManagedBackend,
    ) -> Result<(), ProcessError> {
        if !managed_bailian(env, backend) {
            return Ok(());
        }
        if !env
            .get_trimmed(via_core::config::names::OPENCLAW_GATEWAY_TOKEN)
            .is_empty()
        {
            return Ok(());
        }
        let token = mint_gateway_token()?;
        env.set(via_core::config::names::OPENCLAW_GATEWAY_TOKEN, token);
        Ok(())
    }
}

/// Mint an OpenClaw Gateway transport token.
///
/// **External contract** — `backend-drivers/openclaw.mjs:49`
/// (`randomBytes(32).toString('hex')`).
///
/// Deliberately **not** [`via_core::runtime::generate_auth_secret`], despite
/// producing the same shape: upstream's comment is explicit that *"Gateway
/// transport authentication is independent from the local user identity signing
/// secret and remains private to this backend plugin"*, and sharing one
/// generator would invite sharing one value.
///
/// # Errors
///
/// [`ProcessError::Io`] when the operating system declines to supply
/// randomness. Refusing to start is the only safe answer: a predictable token
/// authenticates anyone to a loopback Gateway that can run commands.
fn mint_gateway_token() -> Result<String, ProcessError> {
    let mut bytes = [0u8; GATEWAY_TOKEN_BYTES];
    getrandom::fill(&mut bytes)
        .map_err(|error| ProcessError::io("read randomness for", std::io::Error::other(error)))?;
    let mut hex = String::with_capacity(GATEWAY_TOKEN_BYTES * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        // Writing to a `String` cannot fail.
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}

/// How many random bytes a minted Gateway token carries.
///
/// **External contract** — `backend-drivers/openclaw.mjs:49`
/// (`randomBytes(32).toString('hex')`), i.e. 32 bytes rendered as 64 hex
/// characters.
pub const GATEWAY_TOKEN_BYTES: usize = 32;

/// The condition under which VIA manages OpenClaw's own configuration.
///
/// **External contract** — `backend-drivers/openclaw.mjs:41-47`: VIA owns the
/// service, the user has not pointed at their own config, a DashScope key is
/// present, and a concrete backend model is selected. `auto` is the sentinel
/// for *"let the backend decide"* and does not count.
#[must_use]
pub fn managed_bailian(env: &EnvMap, backend: &ManagedBackend) -> bool {
    let model = env
        .get_trimmed(via_core::config::names::BACKEND_MODEL)
        .to_lowercase();
    backend.ownership == via_catalog::Ownership::Owned
        && env
            .get_trimmed(via_core::config::names::OPENCLAW_CONFIG_PATH)
            .is_empty()
        && !env
            .get_trimmed(via_core::config::names::DASHSCOPE_API_KEY)
            .is_empty()
        && !model.is_empty()
        && model != via_core::config::backend::AUTO_MODEL_SENTINEL
}

/// Whether the Gateway spawns a separate service process for this backend.
///
/// **External contract** — `state-name/managed backend spawn spec`. Exposed so
/// a caller can answer the question without building a driver.
#[must_use]
pub fn spawns_separate_process(protocol: &str) -> bool {
    matches!(
        backend_definition(protocol).map(|definition| definition.id),
        Some("opencode" | "openclaw")
    )
}

fn non_empty(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.trim().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_catalog::Ownership;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    fn backend(permission_mode: &str, ownership: Ownership) -> ManagedBackend {
        ManagedBackend {
            protocol: "opencode".to_owned(),
            ownership,
            permission_mode: permission_mode.to_owned(),
            base_url: Some("http://127.0.0.1:4096".to_owned()),
        }
    }

    #[test]
    fn only_two_backends_spawn_a_service() {
        for definition in backend_definitions() {
            assert_eq!(
                spawns_separate_process(definition.id),
                matches!(definition.id, "opencode" | "openclaw"),
                "{}",
                definition.id
            );
            assert_eq!(
                managed_launch(definition.id, &EnvMap::new()).is_some(),
                spawns_separate_process(definition.id),
                "{}",
                definition.id
            );
        }
    }

    #[test]
    fn the_registry_validates_both_service_drivers_against_the_catalog() {
        let registry = runtime_registry(&EnvMap::new()).expect("both drivers validate");
        // `registered()` answers in id order, not registration order.
        assert_eq!(registry.registered(), vec!["openclaw", "opencode"]);
        // Every other id still resolves, through the in-Gateway fallback.
        for definition in backend_definitions() {
            let driver = registry.driver(definition.id).expect("resolves");
            assert_eq!(driver.id, definition.id);
            assert_eq!(
                driver.separate_managed_process,
                spawns_separate_process(definition.id)
            );
        }
    }

    #[test]
    fn the_managed_argv_binds_loopback_on_both() {
        assert_eq!(
            managed_launch("opencode", &EnvMap::new())
                .expect("has one")
                .arguments,
            [
                "serve",
                "--hostname",
                "127.0.0.1",
                "--port",
                "${OPENCODE_PORT:-4096}"
            ]
        );
        assert_eq!(
            managed_launch("openclaw", &EnvMap::new())
                .expect("has one")
                .arguments,
            [
                "gateway",
                "run",
                "--port",
                "${OPENCLAW_PORT:-18789}",
                "--bind",
                "loopback"
            ]
        );
        assert_eq!(
            managed_launch("opencode", &env(&[(OPENCODE_PORT, "5000")]))
                .expect("has one")
                .arguments[4],
            "${OPENCODE_PORT:-5000}"
        );
    }

    /// `env-var/backend env derived from base URL` — `cli/src/runtime.mjs:284-287`.
    #[test]
    fn the_port_variable_is_the_base_url_variable_with_the_suffix_swapped() {
        assert_eq!(port_environment("OPENCODE_BASE_URL"), "OPENCODE_PORT");
        assert_eq!(port_environment("OPENCLAW_BASE_URL"), "OPENCLAW_PORT");
        // The regex is anchored, so an interior match is not a match.
        assert_eq!(
            port_environment("OPENCODE_BASE_URL_EXTRA"),
            "OPENCODE_BASE_URL_EXTRA"
        );
        // `String.prototype.replace` with no match returns the input.
        assert_eq!(port_environment("OPENCODE"), "OPENCODE");
    }

    /// The regression this derivation exists for.
    ///
    /// Before it, `managed_launch` read `<X>_PORT` and fell back to a hardcoded
    /// constant, so a custom `OPENCODE_BASE_URL` with no matching
    /// `OPENCODE_PORT` left VIA routing to the custom port while the child it
    /// launched listened on `4096`.
    #[test]
    fn the_managed_port_tracks_a_custom_base_url_with_no_explicit_port() {
        assert_eq!(
            managed_launch(
                "opencode",
                &env(&[("OPENCODE_BASE_URL", "http://127.0.0.1:9999")])
            )
            .expect("has one")
            .arguments[4],
            "${OPENCODE_PORT:-9999}",
            "the fallback tracks the base URL; `via-process` expands the \
             placeholder from the child's own environment at spawn time",
        );
        assert_eq!(
            managed_launch(
                "openclaw",
                &env(&[("OPENCLAW_BASE_URL", "http://127.0.0.1:9999")])
            )
            .expect("has one")
            .arguments[3],
            "${OPENCLAW_PORT:-9999}",
        );
    }

    /// Deriving does not move the default: the constants and the default base
    /// URLs' ports are the same numbers by construction.
    #[test]
    fn deriving_reproduces_the_hardcoded_defaults_when_nothing_is_overridden() {
        let opencode = backend_definition("opencode").expect("catalogued");
        let openclaw = backend_definition("openclaw").expect("catalogued");
        assert_eq!(
            managed_service_port(opencode, &EnvMap::new(), "unused"),
            OPENCODE_DEFAULT_PORT,
        );
        assert_eq!(
            managed_service_port(openclaw, &EnvMap::new(), "unused"),
            openclaw::DEFAULT_GATEWAY_PORT,
        );
    }

    /// An explicit `<X>_PORT` still wins — upstream writes that variable
    /// itself, and `via_process::apply_backend_address` republishes it after a
    /// reallocation.
    #[test]
    fn an_explicit_port_override_beats_the_base_url() {
        let opencode = backend_definition("opencode").expect("catalogued");
        assert_eq!(
            managed_service_port(
                opencode,
                &env(&[
                    ("OPENCODE_BASE_URL", "http://127.0.0.1:9999"),
                    (OPENCODE_PORT, "5000"),
                ]),
                "unused",
            ),
            "5000",
        );
    }

    /// The default-port table is `via_process::service_endpoint_port`'s, which
    /// is upstream's: 443 for `https:`/`wss:`, 80 for everything else.
    #[test]
    fn a_base_url_with_no_port_falls_back_to_the_scheme_default() {
        let openclaw = backend_definition("openclaw").expect("catalogued");
        for (base_url, expected) in [
            ("https://gateway.example", "443"),
            ("wss://gateway.example", "443"),
            ("http://gateway.example", "80"),
            ("ws://gateway.example", "80"),
        ] {
            assert_eq!(
                managed_service_port(openclaw, &env(&[("OPENCLAW_BASE_URL", base_url)]), "unused"),
                expected,
                "{base_url}",
            );
        }
    }

    /// A base URL that does not parse falls through to the caller's constant
    /// rather than panicking or emitting an empty `--port`.
    #[test]
    fn an_unparseable_base_url_falls_through_to_the_fallback() {
        let opencode = backend_definition("opencode").expect("catalogued");
        assert_eq!(
            managed_service_port(
                opencode,
                &env(&[("OPENCODE_BASE_URL", "not a url")]),
                OPENCODE_DEFAULT_PORT,
            ),
            OPENCODE_DEFAULT_PORT,
        );
        assert!(
            !managed_launch("opencode", &env(&[("OPENCODE_BASE_URL", "not a url")]))
                .expect("has one")
                .arguments[4]
                .is_empty(),
            "an empty --port would be worse than a wrong one",
        );
    }

    #[test]
    fn openclaw_refuses_full_permission_with_its_own_sentence() {
        let registry = runtime_registry(&EnvMap::new()).expect("validates");
        let driver = registry.driver("openclaw").expect("registered");
        let environment = EnvMap::new();
        let error = driver
            .resolve(&ResolveRequest {
                env: &environment,
                ownership: Ownership::Owned,
                permission_mode: PERMISSION_MODE_FULL,
            })
            .expect_err("full is refused");
        assert_eq!(
            error.message(via_i18n::Locale::Zh),
            "OpenClaw 的最高权限需要单独配置 exec approvals、elevated 和 host，不能由 Gateway 的统一权限开关安全启用"
        );
    }

    #[test]
    fn opencode_rewrites_its_inline_config_only_in_full_mode() {
        let mut environment = env(&[(OPENCODE_CONFIG_CONTENT, r#"{"theme":"dark"}"#)]);
        OpenCodeHooks
            .apply_permission_mode(&mut environment, &backend("native", Ownership::Owned))
            .expect("native changes nothing");
        assert_eq!(
            environment.get(OPENCODE_CONFIG_CONTENT),
            Some(r#"{"theme":"dark"}"#)
        );

        OpenCodeHooks
            .apply_permission_mode(
                &mut environment,
                &backend(PERMISSION_MODE_FULL, Ownership::Owned),
            )
            .expect("full rewrites");
        let rewritten: Value =
            serde_json::from_str(environment.get(OPENCODE_CONFIG_CONTENT).unwrap_or_default())
                .expect("valid JSON");
        assert_eq!(rewritten["theme"], Value::from("dark"));
        assert_eq!(rewritten["permission"], Value::from("allow"));
        assert_eq!(
            rewritten["agent"]["build"]["permission"],
            Value::from("allow")
        );
    }

    #[test]
    fn the_configured_coordinator_agent_is_allowed_alongside_the_task_agent() {
        let mut environment = env(&[
            ("VIA_BACKEND_AGENT", "reviewer"),
            (OPENCODE_TASK_AGENT, "worker"),
        ]);
        OpenCodeHooks
            .apply_permission_mode(
                &mut environment,
                &backend(PERMISSION_MODE_FULL, Ownership::Owned),
            )
            .expect("full rewrites");
        let rewritten: Value =
            serde_json::from_str(environment.get(OPENCODE_CONFIG_CONTENT).unwrap_or_default())
                .expect("valid JSON");
        assert_eq!(
            rewritten["agent"]["reviewer"]["permission"],
            Value::from("allow")
        );
        assert_eq!(
            rewritten["agent"]["worker"]["permission"],
            Value::from("allow")
        );
    }

    #[test]
    fn a_reserved_agent_id_names_no_agent() {
        // `via-backend` means "use OpenCode's own default", so it must not be
        // written into the agent map as though it were a real agent.
        let mut environment = env(&[("VIA_BACKEND_AGENT", "via-backend")]);
        OpenCodeHooks
            .apply_permission_mode(
                &mut environment,
                &backend(PERMISSION_MODE_FULL, Ownership::Owned),
            )
            .expect("full rewrites");
        let rewritten: Value =
            serde_json::from_str(environment.get(OPENCODE_CONFIG_CONTENT).unwrap_or_default())
                .expect("valid JSON");
        assert!(rewritten["agent"].get("via-backend").is_none());
        assert_eq!(
            rewritten["agent"]["build"]["permission"],
            Value::from("allow")
        );
    }

    #[test]
    fn a_malformed_inline_config_is_a_hard_error() {
        let mut environment = env(&[(OPENCODE_CONFIG_CONTENT, "{not json")]);
        let error = OpenCodeHooks
            .apply_permission_mode(
                &mut environment,
                &backend(PERMISSION_MODE_FULL, Ownership::Owned),
            )
            .expect_err("refused");
        assert_eq!(
            error.message(via_i18n::Locale::Zh),
            "OPENCODE_CONFIG_CONTENT 不是有效的 JSON"
        );

        let mut environment = env(&[(OPENCODE_CONFIG_CONTENT, "[1,2]")]);
        let error = OpenCodeHooks
            .apply_permission_mode(
                &mut environment,
                &backend(PERMISSION_MODE_FULL, Ownership::Owned),
            )
            .expect_err("refused");
        assert_eq!(
            error.message(via_i18n::Locale::Zh),
            "OPENCODE_CONFIG_CONTENT 必须是 JSON 对象"
        );
    }

    #[test]
    fn a_token_is_minted_only_for_a_managed_bailian_gateway() {
        let openclaw = ManagedBackend {
            protocol: "openclaw".to_owned(),
            ownership: Ownership::Owned,
            permission_mode: "native".to_owned(),
            base_url: Some("http://127.0.0.1:18789".to_owned()),
        };
        let mut environment = env(&[
            ("DASHSCOPE_API_KEY", "key"),
            ("VIA_BACKEND_MODEL", "qwen3.7-max"),
        ]);
        OpenClawHooks
            .prepare_environment(&mut environment, &openclaw)
            .expect("mints");
        let token = environment
            .get("OPENCLAW_GATEWAY_TOKEN")
            .unwrap_or_default();
        assert_eq!(token.len(), GATEWAY_TOKEN_BYTES * 2, "32 bytes of hex");
        assert!(token.chars().all(|value| value.is_ascii_hexdigit()));

        // A second call leaves the existing token alone.
        let existing = token.to_owned();
        OpenClawHooks
            .prepare_environment(&mut environment, &openclaw)
            .expect("keeps");
        assert_eq!(
            environment.get("OPENCLAW_GATEWAY_TOKEN"),
            Some(existing.as_str())
        );
    }

    #[test]
    fn no_token_is_minted_without_the_managed_bailian_conditions() {
        let openclaw = ManagedBackend {
            protocol: "openclaw".to_owned(),
            ownership: Ownership::Owned,
            permission_mode: "native".to_owned(),
            base_url: Some("http://127.0.0.1:18789".to_owned()),
        };
        for pairs in [
            vec![("VIA_BACKEND_MODEL", "qwen3.7-max")],
            vec![("DASHSCOPE_API_KEY", "key")],
            vec![("DASHSCOPE_API_KEY", "key"), ("VIA_BACKEND_MODEL", "auto")],
            vec![
                ("DASHSCOPE_API_KEY", "key"),
                ("VIA_BACKEND_MODEL", "qwen3.7-max"),
                ("OPENCLAW_CONFIG_PATH", "/home/u/.openclaw/openclaw.json"),
            ],
        ] {
            let mut environment = env(&pairs);
            OpenClawHooks
                .prepare_environment(&mut environment, &openclaw)
                .expect("no-op");
            assert!(
                environment.get("OPENCLAW_GATEWAY_TOKEN").is_none(),
                "{pairs:?}"
            );
        }
    }

    #[test]
    fn an_external_gateway_is_never_given_a_minted_token() {
        let external = ManagedBackend {
            protocol: "openclaw".to_owned(),
            ownership: Ownership::External,
            permission_mode: "native".to_owned(),
            base_url: Some("wss://openclaw.example.test".to_owned()),
        };
        let mut environment = env(&[
            ("DASHSCOPE_API_KEY", "key"),
            ("VIA_BACKEND_MODEL", "qwen3.7-max"),
        ]);
        OpenClawHooks
            .prepare_environment(&mut environment, &external)
            .expect("no-op");
        assert!(environment.get("OPENCLAW_GATEWAY_TOKEN").is_none());
    }
}
