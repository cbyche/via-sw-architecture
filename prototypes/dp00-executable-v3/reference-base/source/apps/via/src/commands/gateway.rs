//! `via gateway` — run the Gateway in the foreground.
//!
//! The sequence, and every step is here because the next one depends on it:
//!
//! 1. **Normalise the arguments.** `--url` and `--backend-url` become origins;
//!    the backend id is resolved against the catalog; `full` permission is
//!    checked against the backend that would run under it.
//! 2. **Apply them to the environment**, in upstream's order
//!    (`cli/src/launcher.mjs:55-86`), so the resolver and the setup gate see
//!    one environment rather than two.
//! 3. **Resolve the configuration**, which is where a malformed one is
//!    refused.
//! 4. **Run the setup gate.** `docs/architecture.md` §15 ends phase 1 exactly
//!    here: *"`via gateway` refuses to start unconfigured with the exact
//!    message in all three locales"*.
//! 5. **Take the single-instance lease and hold it** — and only then, because
//!    upstream gates *before* touching the lease (`server/src/index.mjs:50`)
//!    so that a misconfigured start never disturbs a running one.
//! 6. **Serve** until SIGINT or SIGTERM, then close in upstream's order.
//!
//! Steps 1-3 are this module. Steps 4-6 are [`crate::gateway`], which is the
//! process around `via-app`; phase 5 replaced the phase-1 ending — take the
//! lease, give it straight back, and refuse naming the phase — with exactly
//! that.

use std::io::Write;

use via_catalog::{
    BackendDefinition, backend_definition, backend_names, normalize_backend_protocol,
    resolve_backend_ownership,
};
use via_core::assert_gateway_setup;
use via_core::config::{BackendSelection, GatewayOptions, names};
use via_i18n::{Locale, format, keys, t};
use via_lock::{LeaseError, VIA_GATEWAY_ALREADY_RUNNING};

use crate::cli::GatewayArgs;
use crate::error::{CODE_INVALID_ARGUMENT, CliError};
use crate::host::Host;
use crate::origin::{UrlLabel, clean_origin, listen_address};

/// The `owner` this binary stamps into the lease.
///
/// **External contract** — `server/src/index.mjs:51-53` passes `"cli"` or
/// `"desktop"`; VIA has no desktop host, so the Gateway is always CLI-owned.
/// `via-lock`'s own default is `"gateway"`, which is the value for an embedder
/// that does not say.
pub const LEASE_OWNER: &str = "cli";

/// The permission mode used when nothing selects one.
///
/// **External contract** — `cli/src/arguments.mjs:109-111`.
pub const DEFAULT_BACKEND_PERMISSION_MODE: &str = "native";

/// The permission mode that needs a Gateway-owned backend.
///
/// **External contract** — `cli/src/arguments.mjs:27,269`.
pub const FULL_PERMISSION_MODE: &str = "full";

/// The arguments, normalised and validated against the catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatewayPlan {
    /// The Gateway origin — scheme, host, non-default port.
    pub url: String,
    /// The backend id, empty for voice chat only.
    pub backend: String,
    /// `native` or `full`, lowercased.
    pub permission_mode: String,
    /// The backend's base URL as an origin, empty when it declares none.
    pub backend_url: String,
    /// Whether `--backend-url` was given on the command line.
    pub backend_url_specified: bool,
    /// The coordinator agent id, trimmed.
    pub backend_agent: String,
}

/// Validate `args` against the catalog and the environment.
///
/// **External contract** — `cli/src/arguments.mjs:136-155,215-231,259-275`.
/// The order matters and is upstream's: the backend id is resolved first
/// (because everything else is keyed on its definition), then the permission
/// mode, then the URLs, then the full-permission check — which is last because
/// it needs both the definition and the mode.
///
/// # Errors
///
/// * [`CliError::Core`] for a backend id the catalog does not have, or an
///   ownership the backend cannot take.
/// * [`CliError::Refused`] for an unparseable or non-`http(s)` URL, and for
///   `full` permission on a backend that does not support it.
pub fn plan(args: &GatewayArgs, host: &Host) -> Result<GatewayPlan, CliError> {
    let locale = host.locale();
    let url = clean_origin(&args.url, UrlLabel::Gateway, locale)?;

    let backend = normalize_backend_protocol(args.backend.as_deref().unwrap_or_default());
    let definition = resolve_definition(&backend)?;

    let permission_mode = args
        .backend_permission_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_BACKEND_PERMISSION_MODE)
        .to_lowercase();

    let backend_url_specified = args.backend_url.is_some();
    let backend_url = resolve_backend_url(args, host, definition, locale)?;

    // `!['setup', 'install'].includes(command) && definition && mode ===
    // 'full' && !definition.supportsFullPermission`
    // (`cli/src/arguments.mjs:266-275`). Neither excluded command is a Gateway
    // run, so the guard applies unconditionally here.
    if let Some(definition) = definition
        && permission_mode == FULL_PERMISSION_MODE
        && !definition.supports_full_permission
    {
        return Err(CliError::refused_with(
            CODE_INVALID_ARGUMENT,
            locale,
            keys::CLI_BACKEND_NO_FULL_PERMISSION,
            &[("label", definition.label)],
        ));
    }

    Ok(GatewayPlan {
        url,
        backend,
        permission_mode,
        backend_url,
        backend_url_specified,
        backend_agent: args
            .backend_agent
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_owned(),
    })
}

/// Look up a backend id, refusing one the catalog does not have.
fn resolve_definition(backend: &str) -> Result<Option<&'static BackendDefinition>, CliError> {
    if backend.is_empty() {
        return Ok(None);
    }
    backend_definition(backend).map(Some).ok_or_else(|| {
        CliError::Core(via_core::CoreError::Catalog(
            via_catalog::CatalogError::UnsupportedBackend {
                protocol: backend.to_owned(),
            },
        ))
    })
}

/// The backend's base URL, as an origin.
///
/// **External contract** — `cli/src/arguments.mjs:260-265`: only a backend
/// that declares a `baseUrlEnvironment` has one at all, its default comes from
/// that variable and then from the catalog's `defaultBaseUrl`, and the result
/// is normalised to an origin. A backend without one gets the empty string —
/// not the Gateway's URL, and not a guess.
fn resolve_backend_url(
    args: &GatewayArgs,
    host: &Host,
    definition: Option<&'static BackendDefinition>,
    locale: Locale,
) -> Result<String, CliError> {
    let Some(variable) = definition.and_then(|d| d.base_url_environment) else {
        return Ok(String::new());
    };
    let configured = host
        .env()
        .get_truthy(variable)
        .or_else(|| definition.and_then(|d| d.default_base_url))
        .unwrap_or_default();
    let requested = args
        .backend_url
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(configured);
    clean_origin(requested, UrlLabel::Backend, locale)
}

/// Write the plan into the environment.
///
/// **External contract** — `cli/src/launcher.mjs:55-86` (*env-var/frontend-only
/// override*) plus the `HOST`/`PORT` half of *env-var/gateway child spawn
/// environment* (`cli/src/runtime.mjs:294-316`).
///
/// Two details are load-bearing:
///
/// * **`AGENT_PROTOCOL` is set to the empty string, never deleted.** The
///   catalogue says why: *"Deleting the key would let the child Gateway reload
///   `AGENT_PROTOCOL` from `config.env` and silently start a backend the user
///   asked to disable."* `via-core`'s `BackendSelection::None` is that empty
///   string; `BackendSelection` having no third "unset" variant is what makes
///   the distinction impossible to lose.
/// * **Ownership is computed before the base URL is written.** Upstream reads
///   the *pre-existing* value of the backend's `*_BASE_URL` to decide whether
///   the backend is external, and only afterwards overwrites it with the
///   resolved origin. Doing it the other way round makes every backend that
///   declares a default base URL look externally configured.
///
/// # Errors
///
/// [`CliError::Core`] when the backend cannot take the ownership its
/// configuration implies.
pub fn apply(plan: &GatewayPlan, host: &mut Host) -> Result<(), CliError> {
    let definition = resolve_definition(&plan.backend)?;

    let ownership = if plan.backend.is_empty() {
        None
    } else {
        let base_url_configured = plan.backend_url_specified
            || definition
                .and_then(|d| d.base_url_environment)
                .is_some_and(|variable| !host.env().get_trimmed(variable).is_empty());
        Some(resolve_backend_ownership(
            &plan.backend,
            base_url_configured,
            host.env().get(names::BACKEND_OWNERSHIP).unwrap_or_default(),
        )?)
    };

    let mut options = GatewayOptions {
        backend: Some(if plan.backend.is_empty() {
            BackendSelection::None
        } else {
            BackendSelection::Named(plan.backend.clone())
        }),
        ..GatewayOptions::default()
    };
    if let Some((listen_host, port)) = listen_address(&plan.url) {
        options.host = Some(listen_host);
        options.port = Some(port);
    }
    host.apply_gateway_options(&options);

    match ownership {
        Some(ownership) => {
            host.set_env(names::BACKEND_OWNERSHIP, ownership.as_str());
            host.set_env(names::BACKEND_PERMISSION_MODE, plan.permission_mode.clone());
        }
        None => {
            host.remove_env(names::BACKEND_OWNERSHIP);
            host.remove_env(names::BACKEND_PERMISSION_MODE);
        }
    }

    if !plan.backend.is_empty() && !plan.backend_agent.is_empty() {
        host.set_env(names::BACKEND_AGENT, plan.backend_agent.clone());
    } else {
        host.remove_env(names::BACKEND_AGENT);
    }

    if let Some(variable) = definition.and_then(|d| d.base_url_environment) {
        host.set_env(variable, plan.backend_url.clone());
        // **External contract** — `env-var/backend env derived from base URL`
        // (`cli/src/runtime.mjs:282-291`). Upstream publishes the port beside
        // the base URL, derived from the URL itself, and assigns it
        // unconditionally: the base URL is the single source of truth, so a
        // stale `<X>_PORT` from the environment cannot outlive the address it
        // no longer describes.
        //
        // `via_backends::managed_service_port` derives the same value when this
        // variable is absent, so the two agree by construction rather than by
        // coincidence — but publishing it here is what makes the Gateway's own
        // environment match upstream's, which is what the contract names.
        if let Ok(port) = via_process::service_endpoint_port(&plan.backend_url) {
            host.set_env(via_backends::port_environment(variable), port.to_string());
        }
    }
    Ok(())
}

/// Run `via gateway`.
///
/// Blocks until SIGINT or SIGTERM. The reactor is built here rather than by a
/// `#[tokio::main]` on `main`, because five of the six verbs need no reactor at
/// all and paying for one on `via config` would be paying for nothing.
///
/// # Errors
///
/// * [`CliError::Refused`] for an argument the catalog rejects, and for
///   `VIA_GATEWAY_ALREADY_RUNNING` when another Gateway holds the lease.
/// * [`CliError::Core`] for a malformed configuration and for the setup gate.
pub fn run(args: &GatewayArgs, host: &mut Host, out: &mut dyn Write) -> Result<(), CliError> {
    let plan = plan(args, host)?;
    apply(&plan, host)?;
    let config = host.resolve_config()?;
    // Run the gate *here* as well as inside `gateway::boot`, so an
    // unconfigured start is refused before a reactor is ever built. The gate is
    // a pure function of the environment, so running it twice cannot disagree
    // with itself.
    assert_gateway_setup(host.env(), host.locale())?;

    // The Gateway logger is built from **this host's** environment rather than
    // from the process's, for the same reason `Host` exists at all: the
    // resolver is a pure function of the environment it was handed, and a
    // logger that read `std::env` instead would write into the developer's real
    // log directory when a test asked for a synthetic one.
    let mut options = via_log::LoggerOptions::gateway(host.env(), host.home_directory());
    if via_log::is_test_process(host.env(), std::env::args()) {
        options.console_enabled = false;
        options.file_enabled = false;
    }
    let composition = crate::gateway::Composition {
        logger: Some(std::sync::Arc::new(via_log::Logger::new(options))),
        ..crate::gateway::Composition::default()
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| CliError::io("start the runtime", "<tokio>", error))?;
    runtime.block_on(crate::gateway::serve(&config, host.env(), composition, out))
}

/// Render a [`LeaseError`] as a localized refusal.
///
/// **External contract** — *error-code/`VIA_GATEWAY_ALREADY_RUNNING`*,
/// `shared/gateway-instance-lock.mjs:154-159`: the message is
/// `已有 Gateway 正在运行` with `：<origin>` appended when the incumbent
/// published one. `via-lock` carries those literals as its `Display` because
/// it predates `via-i18n`; here they come from the catalog, so `en` and `ko`
/// are real sentences rather than Chinese ones.
pub(crate) fn lease_error(error: LeaseError, locale: Locale) -> CliError {
    match &error {
        LeaseError::AlreadyRunning { lease } => {
            let message = if lease.origin.is_empty() {
                t(locale, keys::LOCK_GATEWAY_ALREADY_RUNNING).to_owned()
            } else {
                format(
                    locale,
                    keys::LOCK_GATEWAY_ALREADY_RUNNING_AT,
                    &[("origin", lease.origin.as_str())],
                )
            };
            CliError::Refused {
                code: VIA_GATEWAY_ALREADY_RUNNING,
                message,
            }
        }
        LeaseError::Exhausted => CliError::refused(
            VIA_GATEWAY_ALREADY_RUNNING,
            locale,
            keys::LOCK_LEASE_EXHAUSTED,
        ),
        LeaseError::Io { path, .. } => CliError::io(
            "gateway lease",
            path.clone(),
            std::io::Error::other(error.to_string()),
        ),
    }
}

/// Every backend id the catalog knows, joined for a message.
///
/// **External contract** — `backendNames().join('、')`
/// (`shared/backend-catalog.mjs:435-437`), in catalog order.
#[must_use]
pub fn backend_name_list(locale: Locale) -> String {
    super::join_list(locale, &backend_names())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command, parse_from};
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    fn gateway_args(args: &[&str]) -> GatewayArgs {
        let cli: Cli = parse_from(args.iter().map(|s| (*s).to_owned())).expect("parses");
        let Command::Gateway(gateway) = cli.command else {
            panic!("{args:?} is not a gateway invocation");
        };
        gateway
    }

    /// Every refusal carries the sentence *as rendered when it was built*,
    /// from the host's locale — so a test that wants to assert upstream's own
    /// `zh` wording has to build a `zh` host, which is also the only way to
    /// prove the locale actually reaches the refusal.
    fn host_in(locale: Locale, pairs: &[(&str, &str)]) -> Host {
        let mut env: Vec<(&str, &str)> = vec![("VIA_LOCALE", locale.as_str())];
        env.extend_from_slice(pairs);
        Host::new(
            env.into_iter().collect(),
            PathBuf::from("/home/via"),
            PathBuf::from("/srv/via"),
        )
    }

    fn host(pairs: &[(&str, &str)]) -> Host {
        host_in(Locale::Zh, pairs)
    }

    /// `--backend` carries a `clap` `env` fallback, so a developer with
    /// `AGENT_PROTOCOL` exported would otherwise see different parses. Every
    /// test that cares builds its `GatewayArgs` explicitly instead.
    fn explicit(backend: Option<&str>) -> GatewayArgs {
        GatewayArgs {
            url: crate::origin::DEFAULT_GATEWAY_URL.to_owned(),
            backend: backend.map(str::to_owned),
            backend_permission_mode: None,
            backend_url: None,
            backend_agent: None,
        }
    }

    #[test]
    fn a_frontend_only_plan_has_no_backend_and_no_backend_url() {
        let plan = plan(&explicit(None), &host(&[])).expect("planned");
        assert_eq!(plan.backend, "");
        assert_eq!(plan.backend_url, "");
        assert_eq!(plan.permission_mode, DEFAULT_BACKEND_PERMISSION_MODE);
        assert_eq!(plan.url, crate::origin::DEFAULT_GATEWAY_URL);
    }

    #[test]
    fn none_in_any_case_means_frontend_only() {
        for spelling in ["none", "NONE", "None", "  none  "] {
            let plan = plan(&explicit(Some(spelling)), &host(&[])).expect("planned");
            assert_eq!(plan.backend, "", "{spelling}");
        }
    }

    #[test]
    fn a_backend_that_declares_a_base_url_gets_the_catalog_default() {
        let plan = plan(&explicit(Some("openclaw")), &host(&[])).expect("planned");
        assert_eq!(plan.backend, "openclaw");
        assert_eq!(plan.backend_url, "http://127.0.0.1:18789");
        assert!(!plan.backend_url_specified);
    }

    #[test]
    fn the_backends_own_variable_beats_the_catalog_default() {
        let plan = plan(
            &explicit(Some("opencode")),
            &host(&[("OPENCODE_BASE_URL", "http://127.0.0.1:5000/ignored")]),
        )
        .expect("planned");
        assert_eq!(plan.backend_url, "http://127.0.0.1:5000");
    }

    #[test]
    fn the_flag_beats_the_variable_and_marks_the_backend_external() {
        let mut args = explicit(Some("openclaw"));
        args.backend_url = Some("http://localhost:18888/path".to_owned());
        let plan = plan(&args, &host(&[])).expect("planned");
        assert_eq!(plan.backend_url, "http://localhost:18888");
        assert!(plan.backend_url_specified);
    }

    #[test]
    fn a_backend_without_a_base_url_variable_gets_no_url_at_all() {
        for backend in [
            "qoder",
            "acp",
            "kimi",
            "hermes",
            "codebuddy",
            "codex",
            "claude",
            "pi",
        ] {
            let plan = plan(&explicit(Some(backend)), &host(&[])).expect("planned");
            assert_eq!(plan.backend, backend);
            assert_eq!(plan.backend_url, "", "{backend}");
        }
    }

    #[test]
    fn an_unknown_backend_is_refused_by_the_catalog() {
        let error = plan(&explicit(Some("nonesuch")), &host(&[])).expect_err("not in the catalog");
        assert_eq!(error.code(), "VIA_BACKEND_UNSUPPORTED");
        assert_eq!(error.message(Locale::Zh), "不支持的后台 Agent：nonesuch");
    }

    #[test]
    fn full_permission_is_refused_for_a_backend_that_cannot_take_it() {
        let mut args = explicit(Some("openclaw"));
        args.backend_permission_mode = Some("FULL".to_owned());
        let error = plan(&args, &host(&[])).expect_err("OpenClaw has no safe mapping");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
        assert_eq!(
            error.message(Locale::Zh),
            "OpenClaw 不支持 Gateway 统一最高权限模式"
        );
    }

    #[test]
    fn full_permission_is_accepted_for_a_backend_that_can() {
        let mut args = explicit(Some("qoder"));
        args.backend_permission_mode = Some("full".to_owned());
        let plan = plan(&args, &host(&[])).expect("Qoder supports it");
        assert_eq!(plan.permission_mode, FULL_PERMISSION_MODE);
    }

    #[test]
    fn full_permission_without_a_backend_is_not_checked_at_all() {
        let mut args = explicit(None);
        args.backend_permission_mode = Some("full".to_owned());
        let plan = plan(&args, &host(&[])).expect("no backend, nothing to refuse");
        assert_eq!(plan.permission_mode, FULL_PERMISSION_MODE);
    }

    #[test]
    fn an_unparseable_gateway_url_is_refused_before_anything_else() {
        let mut args = explicit(Some("nonesuch"));
        args.url = "not a url".to_owned();
        let error = plan(&args, &host(&[])).expect_err("the URL is checked first");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
        assert_eq!(error.message(Locale::Zh), "无效的 Gateway URL：not a url");
        // English, from an English host: the same refusal, a different
        // sentence.
        let error = plan(&args, &host_in(Locale::En, &[])).expect_err("still refused");
        assert_eq!(error.message(Locale::En), "invalid  Gateway URL: not a url");
    }

    #[test]
    fn a_websocket_backend_url_is_refused_with_the_backend_label() {
        let mut args = explicit(Some("openclaw"));
        args.backend_url = Some("ws://127.0.0.1:18789".to_owned());
        let error = plan(&args, &host(&[])).expect_err("ws is not http");
        assert_eq!(error.message(Locale::Zh), "后台地址只支持 http 或 https");
    }

    #[test]
    fn applying_a_frontend_only_plan_blanks_the_protocol_and_deletes_the_rest() {
        let mut host = host(&[
            ("VIA_BACKEND_OWNERSHIP", "external"),
            ("VIA_BACKEND_PERMISSION_MODE", "full"),
            ("VIA_BACKEND_AGENT", "build"),
        ]);
        let plan = plan(&explicit(None), &host).expect("planned");
        apply(&plan, &mut host).expect("applied");
        assert_eq!(
            host.env().get("AGENT_PROTOCOL"),
            Some(""),
            "blanked, not deleted — a deleted key would be refilled from config.env"
        );
        assert_eq!(host.env().get("VIA_BACKEND_OWNERSHIP"), None);
        assert_eq!(host.env().get("VIA_BACKEND_PERMISSION_MODE"), None);
        assert_eq!(host.env().get("VIA_BACKEND_AGENT"), None);
    }

    #[test]
    fn applying_a_backend_plan_writes_ownership_mode_and_base_url() {
        let mut host = host(&[]);
        let mut args = explicit(Some("opencode"));
        args.backend_agent = Some("  build  ".to_owned());
        let plan = plan(&args, &host).expect("planned");
        apply(&plan, &mut host).expect("applied");
        assert_eq!(host.env().get("AGENT_PROTOCOL"), Some("opencode"));
        assert_eq!(host.env().get("VIA_BACKEND_OWNERSHIP"), Some("owned"));
        assert_eq!(
            host.env().get("VIA_BACKEND_PERMISSION_MODE"),
            Some("native")
        );
        assert_eq!(host.env().get("VIA_BACKEND_AGENT"), Some("build"));
        assert_eq!(
            host.env().get("OPENCODE_BASE_URL"),
            Some("http://127.0.0.1:4096")
        );
        assert_eq!(
            host.env().get("OPENCODE_PORT"),
            Some("4096"),
            "the port is published beside the base URL, derived from it",
        );
    }

    /// `env-var/backend env derived from base URL` — `cli/src/runtime.mjs:282-291`.
    ///
    /// The regression: `apply` wrote `<X>_BASE_URL` and nothing derived
    /// `<X>_PORT`, so `--backend-url http://127.0.0.1:9999` left VIA routing to
    /// `9999` while the OpenCode child it launched listened on `4096`.
    #[test]
    fn a_custom_backend_url_publishes_a_matching_port() {
        let mut host = host(&[]);
        let mut args = explicit(Some("opencode"));
        args.backend_url = Some("http://127.0.0.1:9999".to_owned());
        let plan = plan(&args, &host).expect("planned");
        apply(&plan, &mut host).expect("applied");
        assert_eq!(
            host.env().get("OPENCODE_BASE_URL"),
            Some("http://127.0.0.1:9999")
        );
        assert_eq!(host.env().get("OPENCODE_PORT"), Some("9999"));
        // And the argv the managed launch will carry agrees with it, which is
        // the property the mismatch broke. Both halves are checked: the
        // placeholder reads the same variable `apply` just published, and its
        // fallback is the same port — so the child listens on 9999 whether or
        // not `apply_backend_address` runs before the spawn.
        assert_eq!(
            via_backends::managed_launch("opencode", host.env())
                .expect("opencode spawns a service")
                .arguments[4],
            "${OPENCODE_PORT:-9999}",
        );
    }

    /// Upstream assigns the port unconditionally, so the base URL is the single
    /// source of truth and a stale `<X>_PORT` cannot outlive it.
    #[test]
    fn the_published_port_overwrites_a_stale_one() {
        let mut host = host(&[("OPENCODE_PORT", "5000")]);
        let mut args = explicit(Some("opencode"));
        args.backend_url = Some("http://127.0.0.1:9999".to_owned());
        let plan = plan(&args, &host).expect("planned");
        apply(&plan, &mut host).expect("applied");
        assert_eq!(host.env().get("OPENCODE_PORT"), Some("9999"));
    }

    #[test]
    fn an_explicit_backend_url_makes_openclaw_external() {
        let mut host = host(&[]);
        let mut args = explicit(Some("openclaw"));
        args.backend_url = Some("http://10.0.0.2:18789".to_owned());
        let plan = plan(&args, &host).expect("planned");
        apply(&plan, &mut host).expect("applied");
        assert_eq!(host.env().get("VIA_BACKEND_OWNERSHIP"), Some("external"));
        assert_eq!(
            host.env().get("OPENCLAW_BASE_URL"),
            Some("http://10.0.0.2:18789")
        );
    }

    #[test]
    fn ownership_is_decided_before_the_base_url_is_overwritten() {
        // The regression: `applyGatewayOptions` writes the resolved origin into
        // the backend's own variable. If ownership were computed afterwards,
        // every backend with a catalogue `defaultBaseUrl` would look
        // externally configured, and OpenClaw would never be Gateway-owned.
        let mut host = host(&[]);
        let plan = plan(&explicit(Some("openclaw")), &host).expect("planned");
        apply(&plan, &mut host).expect("applied");
        assert_eq!(
            host.env().get("VIA_BACKEND_OWNERSHIP"),
            Some("owned"),
            "the catalog default must not read as an external configuration"
        );
    }

    #[test]
    fn an_empty_backend_agent_is_deleted_rather_than_blanked() {
        let mut host = host(&[("VIA_BACKEND_AGENT", "stale")]);
        let mut args = explicit(Some("qoder"));
        args.backend_agent = Some("   ".to_owned());
        let plan = plan(&args, &host).expect("planned");
        apply(&plan, &mut host).expect("applied");
        assert_eq!(host.env().get("VIA_BACKEND_AGENT"), None);
    }

    #[test]
    fn the_gateway_url_becomes_host_and_port() {
        let mut host = host(&[("DASHSCOPE_API_KEY", "k")]);
        let mut args = explicit(None);
        args.url = "http://localhost:9999".to_owned();
        let plan = plan(&args, &host).expect("planned");
        apply(&plan, &mut host).expect("applied");
        let config = host.resolve_config().expect("resolvable");
        assert_eq!(config.host, "127.0.0.1", "localhost is pinned to IPv4");
        assert_eq!(config.port, 9999);
    }

    #[test]
    fn the_lease_owner_is_the_cli() {
        assert_eq!(LEASE_OWNER, "cli");
        assert_ne!(LEASE_OWNER, via_lock::DEFAULT_LEASE_OWNER);
    }

    #[test]
    fn the_backend_name_list_is_the_catalog_in_order() {
        assert_eq!(
            backend_name_list(Locale::Zh),
            via_catalog::backend_names().join("、")
        );
    }

    #[test]
    fn parsing_the_documented_flags_produces_the_documented_plan() {
        let args = gateway_args(&[
            "gateway",
            "--url",
            "https://voice.example.com/path",
            "--backend-url",
            "http://localhost:18888/path",
        ]);
        assert_eq!(args.url, "https://voice.example.com/path");
        assert_eq!(
            args.backend_url.as_deref(),
            Some("http://localhost:18888/path")
        );
    }
}
