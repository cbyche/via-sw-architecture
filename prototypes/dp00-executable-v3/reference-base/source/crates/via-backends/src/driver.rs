//! The twelve drivers, and the registry that resolves one.
//!
//! Ported from `server/src/agent/backends/registry.mjs` and the ten driver
//! modules beside it. This is the file the whole crate exists for:
//! `via-acp` speaks ACP to *a* child process and `via-process` supervises *a*
//! managed service, and neither is allowed to know that OpenClaw or Codex or Pi
//! exist. Everything that distinguishes them is a table entry here.
//!
//! # What replaces `process.execPath scripts/<name>.mjs`
//!
//! Upstream launches six of the twelve through a Node shim: `process.execPath`
//! plus a `scripts/*.mjs` launcher that resolves a runtime, sets a few
//! variables and `exec`s the real binary. `docs/architecture.md` §10 is explicit
//! that those disappear — *"Node's `npm install -g`, the `npx` shims and
//! `scripts/*-acp.mjs` disappear: a Rust binary is its own installer, and the
//! shims become `via-backends` launch specs"* — so each driver below launches
//! the executable the shim would have `exec`'d, with the variables the shim
//! would have set.
//!
//! Two consequences are recorded as deviations. `ELECTRON_RUN_AS_NODE=1` is not
//! stamped: it tells an **Electron** binary to behave as Node, and it is
//! meaningful only because upstream's command *is* `process.execPath`. Neither
//! is `VIA_NODE`, for the reason `via-acp` already records — VIA has no
//! interpreter of its own to publish. Every other variable in
//! `env-var/per-driver launch env injections` is reproduced.
//!
//! # The credential boundary is applied here
//!
//! Each driver projects the Gateway environment through **its own** catalogued
//! [`EnvironmentPolicy`](via_catalog::EnvironmentPolicy) with
//! [`via_acp::BackendEnv::project`]. Gateway identity, realtime credentials and
//! other backends' secrets are absent from the child, not merely unused by it;
//! `tests/backend_environment.rs` is the verbatim port of
//! `server/test/backend-environment.test.mjs`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use via_acp::{AcpBackendProfile, AcpDownstreamAgent, AcpSessionRegistry, BackendEnv, SpawnSpec};
use via_catalog::{
    BackendDefinition, Ownership, backend_definition, backend_names, normalize_backend_protocol,
};
use via_core::Config;
use via_core::EnvMap;
use via_core::config::names;
use via_downstream::{BackendCapabilities, DownstreamAgent, HarnessRegistry};
use via_i18n::{Locale, t};

use crate::capability::backend_capabilities;
use crate::detect::ExecutableFinder;
use crate::error::BackendsError;
use crate::openclaw;
use crate::opencode;
use crate::profile::{
    BackendProfile, BackendUi, ControlInstructions, PrepareAction, SessionConfigOption,
    SessionInstructions,
};

/// The permission mode that turns a backend's own approval gate off.
///
/// **External contract** — `via_process::PERMISSION_MODE_FULL`; restated here
/// only so the driver table reads against a name rather than a literal.
pub const PERMISSION_MODE_FULL: &str = via_process::PERMISSION_MODE_FULL;

/// The `model_provider` VIA registers inside Codex.
///
/// **External contract** — `server/src/agent/backends/codex.mjs:4`
/// (`CODEX_PROVIDER = 'qwen-audio-agent'`), renamed per `docs/rebrand.md` row
/// *"`CODEX_PROVIDER = 'qwen-audio-agent'` → `via`"*. It is written into
/// `MODEL_PROVIDER` and used as the `model_providers` key, so both move
/// together.
pub const CODEX_PROVIDER: &str = "via";

/// DeepSeek Harness's default model.
///
/// **External contract** — `env-var/per-driver launch env injections`:
/// *"`DSH_MODEL`'s default `deepseek-v4-pro` … asserted in
/// `server/test/backend-driver-launch.test.mjs`"*. This is the same default
/// [`DEEPSEEK_HARNESS_CORDIS_YAML`] carries in its own `acp-agent` block
/// (`model: !!js "process.env.DSH_MODEL ?? 'deepseek-v4-pro'"`);
/// `tests/contracts.rs::the_deepseek_default_model_matches_the_shipped_cordis_asset`
/// asserts the constant against the shipped literal so the two cannot drift.
pub const DEEPSEEK_DEFAULT_MODEL: &str = "deepseek-v4-pro";

/// DeepSeek Harness's permission mode in `full`.
pub const DSH_PERMISSION_FULL: &str = "danger-full-access";
/// DeepSeek Harness's permission mode otherwise.
pub const DSH_PERMISSION_WORKSPACE: &str = "workspace-write";

/// The harness configuration asset DeepSeek is pointed at.
///
/// **External contract** — `file-path/deepseek harness config asset`
/// (`config/deepseek-harness/cordis.yml`), resolved against the install root.
pub const DEEPSEEK_HARNESS_CONFIG_PATH: &str = "config/deepseek-harness/cordis.yml";

/// The DeepSeek Harness composition, shipped verbatim.
///
/// **Copied byte for byte** from `config/deepseek-harness/cordis.yml`, with the
/// one identity mention `docs/rebrand.md`'s general rule covers — the header
/// comment's product name, `qwen-audio-agent` → `VIA` — and nothing else.
/// Every plugin id, npm package coordinate, `DSH_*` / `DEEPSEEK_HARNESS_*`
/// environment name and the `deepseek-v4-pro` / `deepseek-v4-flash` model ids
/// are third-party or vendor literals and stay (`docs/rebrand.md`: "DeepSeek
/// and CodeBuddy model ids, `DEEPSEEK_*` and `CODEBUDDY_*` env names, and
/// third-party package coordinates are KEEP"). `docs/fidelity.md` lists this
/// file under *"Copied verbatim, byte for byte"*; `tests/contracts.rs` is what
/// asserts it.
///
/// The `!!js` tags are YAML this crate never parses — they are JavaScript the
/// DeepSeek Harness process itself evaluates. Written to
/// [`DEEPSEEK_HARNESS_CONFIG_PATH`] untouched, the harness reads and
/// interprets them; a Rust YAML parser would reject the tags outright.
pub const DEEPSEEK_HARNESS_CORDIS_YAML: &str =
    include_str!("../assets/deepseek-harness/cordis.yml");

/// Everything a driver needs to build a profile.
///
/// The Rust shape of upstream's `createProfile(options)` argument, which is
/// assembled at `server/src/agent/acp-backend-profile.mjs` from the same
/// configuration this carries.
pub struct LaunchContext<'a> {
    /// The resolved configuration. Every per-backend namespace comes from
    /// `config.backends`, already normalised by `via-core`.
    pub config: &'a Config,
    /// The Gateway environment, the source the credential projection filters.
    pub env: &'a EnvMap,
    /// Whether VIA owns the backend service or is connecting to someone
    /// else's.
    pub ownership: Ownership,
    /// The permission mode already normalised through
    /// [`via_catalog::effective_backend_permission_mode`].
    pub permission_mode: &'a str,
    /// The owner whose coordinator session this is.
    pub owner_id: &'a str,
    /// Locating executables a launcher shim used to find.
    pub finder: &'a dyn ExecutableFinder,
}

impl LaunchContext<'_> {
    fn locale(&self) -> Locale {
        self.config.locale
    }

    fn is_full_permission(&self) -> bool {
        self.permission_mode == PERMISSION_MODE_FULL
    }
}

/// One backend driver: an id, a label, seven flags, and a way to build a
/// profile.
///
/// Upstream's driver object minus `createProfile`'s closure, which in Rust is a
/// function pointer on the table rather than a method on a literal.
#[derive(Clone, Copy)]
pub struct BackendDriver {
    /// The catalogued id.
    pub id: &'static str,
    /// The catalogued label. `validateBackendDriver` refuses a mismatch, and so
    /// does [`via_downstream::HarnessDescriptor::declare`].
    pub label: &'static str,
    /// The seven flags this driver declares.
    pub capabilities: BackendCapabilities,
    build: fn(&BackendDefinition, &LaunchContext<'_>) -> Result<BackendProfile, BackendsError>,
}

impl std::fmt::Debug for BackendDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendDriver")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

impl BackendDriver {
    /// Build this backend's profile.
    ///
    /// # Errors
    ///
    /// [`BackendsError::GenericAcpRequiresCommand`] and
    /// [`BackendsError::GenericAcpFullPermissionUnsafe`] for the generic ACP
    /// entry point; nothing else refuses here, because every other refusal is
    /// the runtime layer's (`via-process`) or the seam's (`via-downstream`).
    pub fn create_profile(
        &self,
        context: &LaunchContext<'_>,
    ) -> Result<BackendProfile, BackendsError> {
        let definition =
            backend_definition(self.id).ok_or_else(|| BackendsError::UnsupportedBackend {
                protocol: self.id.to_owned(),
            })?;
        (self.build)(definition, context)
    }
}

/// `backendDriver(protocol)` — `server/src/agent/backends/registry.mjs:58-62`.
///
/// The id is trimmed and lower-cased, and `none` resolves to nothing.
///
/// # Errors
///
/// [`BackendsError::UnsupportedBackend`].
pub fn backend_driver(protocol: &str) -> Result<BackendDriver, BackendsError> {
    let id = normalize_backend_protocol(protocol);
    DRIVERS
        .iter()
        .find(|driver| driver.id == id)
        .copied()
        .ok_or(BackendsError::UnsupportedBackend { protocol: id })
}

/// `hasBackendDriver(protocol)` — `registry.mjs:64-66`.
#[must_use]
pub fn has_backend_driver(protocol: &str) -> bool {
    backend_driver(protocol).is_ok()
}

/// `backendIds()` — `registry.mjs:68-70`, in catalog order.
#[must_use]
pub fn backend_ids() -> Vec<&'static str> {
    DRIVERS.iter().map(|driver| driver.id).collect()
}

/// Every driver, in catalog order.
#[must_use]
pub fn backend_drivers() -> &'static [BackendDriver] {
    &DRIVERS
}

/// `createBackendProfile(protocol, options)` — `registry.mjs:72-86`.
///
/// # Errors
///
/// [`BackendsError::UnsupportedBackend`], plus whatever the driver refuses
/// with.
pub fn create_backend_profile(
    protocol: &str,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    backend_driver(protocol)?.create_profile(context)
}

/// Build the [`DownstreamAgent`] for one backend.
///
/// The descriptor is validated against `via-catalog` inside
/// [`AcpDownstreamAgent::declare`], so a driver whose declaration contradicts
/// the catalog is refused here rather than mid-turn.
///
/// # Errors
///
/// Whatever [`create_backend_profile`] refuses with, or
/// [`BackendsError::Harness`] when the declaration does not validate.
pub fn create_downstream_agent(
    protocol: &str,
    context: &LaunchContext<'_>,
    sessions: Arc<AcpSessionRegistry>,
) -> Result<AcpDownstreamAgent, BackendsError> {
    let profile = create_backend_profile(protocol, context)?;
    Ok(AcpDownstreamAgent::declare(profile.acp, sessions)?)
}

/// Register every catalogued backend that this configuration can build.
///
/// Upstream's registry is a module literal of nine driver objects, validated at
/// import (`registry.mjs:47-56`). VIA's is built here from the same table, and
/// the generic ACP entry point is skipped when it has no `ACP_COMMAND` — the one
/// backend whose *existence* depends on configuration, and which upstream would
/// only fail on at `createProfile` time.
///
/// # Errors
///
/// Whatever [`create_downstream_agent`] refuses with, other than the generic
/// ACP backend's own configuration refusals.
pub fn register_backends(
    context: &LaunchContext<'_>,
    sessions: &Arc<AcpSessionRegistry>,
) -> Result<HarnessRegistry, BackendsError> {
    let mut registry = HarnessRegistry::new();
    for driver in DRIVERS {
        match create_downstream_agent(driver.id, context, Arc::clone(sessions)) {
            Ok(agent) => {
                registry.register(Arc::new(agent) as Arc<dyn DownstreamAgent>)?;
            }
            Err(
                BackendsError::GenericAcpRequiresCommand
                | BackendsError::GenericAcpFullPermissionUnsafe,
            ) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(registry)
}

// ── the table ───────────────────────────────────────────────────────────────

/// The twelve, in `shared/backend-catalog.mjs` order.
static DRIVERS: [BackendDriver; 12] = [
    BackendDriver {
        id: "opencode",
        label: "OpenCode",
        capabilities: crate::capability::OPENCODE,
        build: build_opencode,
    },
    BackendDriver {
        id: "openclaw",
        label: "OpenClaw",
        capabilities: crate::capability::OPENCLAW,
        build: build_openclaw,
    },
    BackendDriver {
        id: "qoder",
        label: "Qoder",
        capabilities: crate::capability::SESSION_MCP,
        build: build_qoder,
    },
    BackendDriver {
        // The third-party Qwen Code CLI. Id, label, command and `QWEN_CODE_`
        // namespace are KEEP — they name Alibaba's product, not VIA.
        id: "qwen",
        label: "Qwen Code",
        capabilities: crate::capability::SESSION_MCP,
        build: build_qwen,
    },
    BackendDriver {
        id: "kimi",
        label: "Kimi Code",
        capabilities: crate::capability::SESSION_MCP,
        build: build_kimi,
    },
    BackendDriver {
        id: "hermes",
        label: "Hermes",
        capabilities: crate::capability::SESSION_MCP,
        build: build_hermes,
    },
    BackendDriver {
        id: "codebuddy",
        label: "CodeBuddy",
        capabilities: crate::capability::SESSION_MCP,
        build: build_codebuddy,
    },
    BackendDriver {
        id: "codex",
        label: "Codex",
        capabilities: crate::capability::SESSION_MCP,
        build: build_codex,
    },
    BackendDriver {
        id: "claude",
        label: "Claude Code",
        capabilities: crate::capability::SESSION_MCP,
        build: build_claude,
    },
    BackendDriver {
        id: "deepseek",
        label: "DeepSeek",
        capabilities: crate::capability::DEEPSEEK,
        build: build_deepseek,
    },
    BackendDriver {
        id: "pi",
        label: "Pi",
        capabilities: crate::capability::PI,
        build: build_pi,
    },
    BackendDriver {
        id: "acp",
        label: "ACP Agent",
        capabilities: crate::capability::SESSION_MCP,
        build: build_generic_acp,
    },
];

// ── shared construction ─────────────────────────────────────────────────────

/// A profile with every optional field at its default, so each driver states
/// only what it overrides.
fn base_profile(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
    label: String,
    spawn: SpawnSpec,
) -> BackendProfile {
    let capabilities =
        backend_capabilities(definition.id).unwrap_or(crate::capability::SESSION_MCP);
    let locale = context.locale();
    BackendProfile {
        acp: AcpBackendProfile {
            protocol: definition.id.to_owned(),
            label: label.clone(),
            spawn,
            capabilities,
            timeout: timeout(context),
            locale,
            sanitize_process_output: None,
            format_request_error: None,
        },
        capabilities,
        session_config_options: Vec::new(),
        session_instructions: None,
        control_instructions: ControlInstructions::SessionTools,
        delegation_title: BackendProfile::default_delegation_title(&label, locale),
        readiness_message: None,
        coordinator_meta: None,
        backend_ui: None,
        process_model_configuration: false,
        prepare: None,
    }
}

/// The per-request deadline, from `AGENT_TIMEOUT_MS`.
fn timeout(context: &LaunchContext<'_>) -> Option<std::time::Duration> {
    u64::try_from(context.config.agent_timeout_ms)
        .ok()
        .map(std::time::Duration::from_millis)
}

/// Project the Gateway environment through this backend's policy.
///
/// The single place a child environment is built in this crate, and the reason
/// it takes a [`BackendDefinition`] rather than a name: the policy is the
/// catalog's, not the driver's.
fn child_env(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
    additions: &[(String, String)],
) -> BackendEnv {
    let borrowed: Vec<(&str, &str)> = additions
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    BackendEnv::project(&definition.environment, context.env, &borrowed)
}

/// `clean(value) || fallback` — `backends/shared.mjs:12-14`.
fn command_or(configured: &str, fallback: &str) -> String {
    let trimmed = configured.trim();
    if trimmed.is_empty() {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn addition(additions: &mut Vec<(String, String)>, name: &str, value: &str) {
    if !value.trim().is_empty() {
        additions.push((name.to_owned(), value.trim().to_owned()));
    }
}

/// A local ACP backend: the catalogued command, a fixed argument list, and
/// nothing but the policy environment.
///
/// `localAcpBackend({ id, label, command, args })` —
/// `server/src/agent/backends/local-acp.mjs:3-38`.
fn local_acp(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
    cli_path: &str,
    directory: &Path,
    arguments: Vec<String>,
) -> BackendProfile {
    let command = command_or(cli_path, definition.setup.command.unwrap_or_default());
    let spawn = SpawnSpec::new(command)
        .args(arguments)
        .cwd(directory)
        .env(child_env(definition, context, &[]));
    base_profile(definition, context, definition.label.to_owned(), spawn)
}

// ── the ten named drivers ───────────────────────────────────────────────────

/// `server/src/agent/backends/opencode.mjs:32-63`.
///
/// Upstream runs `node scripts/opencode.mjs acp`, whose `auto` runtime ends at
/// `opencode acp`. VIA launches that directly and performs the two environment
/// steps the shim performed: the Bailian model override and the optional
/// configuration isolation.
fn build_opencode(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.opencode;
    let mut additions = Vec::new();
    // `scripts/opencode.mjs:61-64`: a configured backend model becomes
    // OpenCode's own, unless the user already set one.
    if context.env.get_trimmed("OPENCODE_MODEL").is_empty() {
        addition(&mut additions, "OPENCODE_MODEL", &options.model);
    }
    if let Some(home) = opencode::isolated_xdg_config_home(context.env, &context.config.root) {
        addition(&mut additions, "XDG_CONFIG_HOME", &home);
    }
    let command = command_or(
        context.env.get_trimmed("OPENCODE_BIN"),
        definition.setup.command.unwrap_or_default(),
    );
    let spawn = SpawnSpec::new(command)
        .args(args(&["acp"]))
        .cwd(&options.directory)
        .env(child_env(definition, context, &additions));
    let mut profile = base_profile(definition, context, definition.label.to_owned(), spawn);
    profile.backend_ui = Some(BackendUi::OpenCode);
    Ok(profile)
}

/// `server/src/agent/backends/openclaw.mjs:49-155`.
fn build_openclaw(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.openclaw;
    let direct_bridge = options.cli_path.trim();
    let token = options.token.expose().trim().to_owned();
    let token_file = options.token_file.to_string_lossy().trim().to_owned();

    let mut arguments = vec!["acp".to_owned(), "--url".to_owned()];
    arguments.push(
        openclaw::bridge_websocket_url(&options.base_url)
            .unwrap_or_else(|_| options.base_url.clone()),
    );
    if !token_file.is_empty() {
        arguments.push("--token-file".to_owned());
        arguments.push(token_file.clone());
    }
    arguments.push("--verbose".to_owned());

    let mut additions = Vec::new();
    addition(&mut additions, names::OPENCLAW_GATEWAY_TOKEN, &token);
    if !token_file.is_empty() {
        // Keep the ACP bridge's device identity separate from the user's normal
        // OpenClaw CLI identity. A loopback bridge presenting the Gateway's
        // shared token can then use OpenClaw's silent local pairing instead of
        // inheriting a stale, narrowly scoped user device token.
        let state = Path::new(&token_file)
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();
        addition(&mut additions, "OPENCLAW_STATE_DIR", &state);
    }

    let command = command_or(direct_bridge, definition.setup.command.unwrap_or_default());
    let spawn = SpawnSpec::new(command)
        .args(arguments)
        .cwd(&options.directory)
        .env(child_env(definition, context, &additions));

    let locale = context.locale();
    let mut profile = base_profile(definition, context, definition.label.to_owned(), spawn);
    profile.acp.sanitize_process_output = Some(Arc::new(|value: &str| {
        openclaw::sanitize_process_output(value)
    }));
    profile.acp.format_request_error = Some(Arc::new(move |context| {
        openclaw::format_request_error(&context.details, locale)
    }));
    profile.session_instructions = Some(SessionInstructions::OpenClaw);
    profile.control_instructions = ControlInstructions::OpenClawNative;
    profile.delegation_title = BackendProfile::openclaw_delegation_title(locale);
    // An owned Gateway is started concurrently and may need a short warm-up.
    // For an external Gateway, let the official bridge establish the real
    // connection so TLS, authentication, routing and remote-network errors are
    // reported accurately instead of being hidden by a local probe.
    profile.readiness_message = match context.ownership {
        Ownership::External => None,
        Ownership::Owned => Some(t(locale, BackendProfile::readiness_key()).to_owned()),
    };
    profile.coordinator_meta =
        openclaw::coordinator_meta(&options.coordinator_agent, context.owner_id);
    profile.backend_ui = Some(BackendUi::OpenClaw {
        token: token.clone(),
    });
    profile.prepare = (!direct_bridge.is_empty() && !token.is_empty() && !token_file.is_empty())
        .then(|| PrepareAction::WritePrivateFile {
            path: PathBuf::from(&token_file),
            contents: openclaw::token_file_contents(&token),
        });
    Ok(profile)
}

/// `local-acp.mjs:41-49` — Qoder.
fn build_qoder(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.qoder;
    let mut arguments = args(&["--acp"]);
    if context.is_full_permission() {
        arguments.push("--dangerously-skip-permissions".to_owned());
    }
    Ok(local_acp(
        definition,
        context,
        &options.cli_path,
        &options.directory,
        arguments,
    ))
}

/// `local-acp.mjs:50-54` — the third-party Qwen Code CLI.
fn build_qwen(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.qwen;
    Ok(local_acp(
        definition,
        context,
        &options.cli_path,
        &options.directory,
        args(&["--acp"]),
    ))
}

/// `local-acp.mjs:55-65` — Kimi Code, the one backend with a session config
/// option.
fn build_kimi(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.kimi;
    let mut profile = local_acp(
        definition,
        context,
        &options.cli_path,
        &options.directory,
        args(&["acp"]),
    );
    if context.is_full_permission() {
        profile.session_config_options = vec![SessionConfigOption {
            id: "mode",
            value: "auto",
        }];
    }
    Ok(profile)
}

/// `local-acp.mjs:66-70` — Hermes.
fn build_hermes(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.hermes;
    Ok(local_acp(
        definition,
        context,
        &options.cli_path,
        &options.directory,
        args(&["acp", "--accept-hooks"]),
    ))
}

/// The CodeBuddy model table, shipped verbatim.
///
/// **Copied byte for byte** from
/// `config/codebuddy/workspace/.codebuddy/models.json` — no substitution at
/// all, because every string in it is third-party or vendor-owned: the
/// DashScope model id and display name (`docs/rebrand.md`'s `qwen3.7-max` /
/// "Qwen 3.7 Max" KEEP row), the vendor name, and the two `${…}` references
/// ([`via_core::config::names::DASHSCOPE_API_KEY`] and
/// [`via_core::config::names::CODEBUDDY_MODEL_URL`], asserted against this
/// literal so the placeholder names cannot drift from the environment names
/// VIA actually sets). `docs/fidelity.md` lists this file under *"Copied
/// verbatim, byte for byte"*; `tests/contracts.rs` is what asserts it.
///
/// Read by the external CodeBuddy CLI at
/// `<codeBuddyWorkspace>/.codebuddy/models.json`
/// ([`via_core::config::backend::codebuddy_models_json_path`]); the `${…}`
/// references are expanded by **CodeBuddy**, not by VIA.
pub const CODEBUDDY_MODELS_JSON_TEMPLATE: &str =
    include_str!("../assets/codebuddy/workspace/.codebuddy/models.json");

/// `server/src/agent/backends/codebuddy.mjs:16-49`.
fn build_codebuddy(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.codebuddy;
    let mut arguments = args(&["--acp"]);
    if !options.model.trim().is_empty() {
        arguments.push("--model".to_owned());
        arguments.push(options.model.trim().to_owned());
    }
    if context.is_full_permission() {
        arguments.push("--dangerously-skip-permissions".to_owned());
    }
    let mut additions = Vec::new();
    addition(
        &mut additions,
        names::CODEBUDDY_MODEL_URL,
        &options.model_url,
    );
    let command = command_or(
        &options.cli_path,
        definition.setup.command.unwrap_or_default(),
    );
    let spawn = SpawnSpec::new(command)
        .args(arguments)
        .cwd(&options.directory)
        .env(child_env(definition, context, &additions));
    Ok(base_profile(
        definition,
        context,
        definition.label.to_owned(),
        spawn,
    ))
}

/// `server/src/agent/backends/codex.mjs:6-72`, plus the two variables
/// `scripts/codex-acp.mjs:16-25` sets.
fn build_codex(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.codex;
    let mut additions = Vec::new();
    addition(&mut additions, names::CODEX_ACP_BIN, &options.cli_path);
    // `scripts/codex-acp.mjs:16` — the adapter must not try to open a browser
    // on a headless Gateway. `NO_BROWSER` is already on the shared allow-list,
    // so a user value wins; this only supplies a default.
    if context.env.get_trimmed("NO_BROWSER").is_empty() {
        addition(&mut additions, "NO_BROWSER", "1");
    }
    // `scripts/codex-acp.mjs:19-25` — the adapter drives the `codex` CLI and
    // needs to be told where it is.
    let codex_path = context.env.get_trimmed("CODEX_PATH").to_owned();
    let codex_path = if codex_path.is_empty() {
        context.finder.find("codex")
    } else {
        codex_path
    };
    addition(&mut additions, "CODEX_PATH", &codex_path);

    if !options.model_url.trim().is_empty() {
        addition(&mut additions, "MODEL_PROVIDER", CODEX_PROVIDER);
        addition(&mut additions, "CODEX_CONFIG", &codex_config(options));
    }
    if context.is_full_permission() {
        addition(&mut additions, "INITIAL_AGENT_MODE", "agent-full-access");
    }
    let command = command_or(
        &options.cli_path,
        definition.setup.adapter_command.unwrap_or_default(),
    );
    let spawn = SpawnSpec::new(command)
        .cwd(&options.directory)
        .env(child_env(definition, context, &additions));
    Ok(base_profile(
        definition,
        context,
        definition.label.to_owned(),
        spawn,
    ))
}

/// The inline `CODEX_CONFIG` blob — `backends/codex.mjs:40-56`.
///
/// Key order is JavaScript object-literal order and is observable, because
/// Codex reads the blob back as TOML-ish configuration; `serde_json`'s
/// `preserve_order` is what keeps it reproducible.
fn codex_config(options: &via_core::config::backend::CodexOptions) -> String {
    let mut provider = serde_json::Map::new();
    provider.insert("name".to_owned(), CODEX_PROVIDER.into());
    provider.insert(
        "base_url".to_owned(),
        options.model_url.trim().to_owned().into(),
    );
    provider.insert("env_key".to_owned(), names::DASHSCOPE_API_KEY.into());
    provider.insert("wire_api".to_owned(), "responses".into());

    let mut providers = serde_json::Map::new();
    providers.insert(CODEX_PROVIDER.to_owned(), provider.into());

    let mut config = serde_json::Map::new();
    if !options.model.trim().is_empty() {
        config.insert("model".to_owned(), options.model.trim().to_owned().into());
    }
    config.insert("model_provider".to_owned(), CODEX_PROVIDER.into());
    config.insert("model_providers".to_owned(), providers.into());
    serde_json::Value::Object(config).to_string()
}

/// `server/src/agent/backends/claude.mjs:5-46`, plus the two variables
/// `scripts/claude-code-acp.mjs:12-25` sets.
fn build_claude(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.claude;
    let mut additions = Vec::new();
    addition(
        &mut additions,
        names::CLAUDE_CODE_ACP_BIN,
        &options.cli_path,
    );
    // `scripts/claude-code-acp.mjs:19-25` — the adapter drives the `claude` CLI.
    let executable = if options.claude_executable.trim().is_empty() {
        context.finder.find("claude")
    } else {
        options.claude_executable.trim().to_owned()
    };
    addition(&mut additions, names::CLAUDE_CODE_EXECUTABLE, &executable);
    // `scripts/claude-code-acp.mjs:12-15` — the adapter reads
    // `ANTHROPIC_API_KEY`; upstream's convention copies `CLAUDE_API_KEY` into
    // it when only the latter is set. Both are on Claude's own allow-list, so
    // this crosses no boundary it was not already crossing.
    if context.env.get_trimmed("ANTHROPIC_API_KEY").is_empty() {
        addition(
            &mut additions,
            "ANTHROPIC_API_KEY",
            context.env.get_trimmed("CLAUDE_API_KEY"),
        );
    }
    let command = command_or(
        &options.cli_path,
        definition.setup.adapter_command.unwrap_or_default(),
    );
    let spawn = SpawnSpec::new(command)
        .cwd(&options.directory)
        .env(child_env(definition, context, &additions));
    Ok(base_profile(
        definition,
        context,
        definition.label.to_owned(),
        spawn,
    ))
}

/// `server/src/agent/backends/deepseek-harness.mjs:6-61`.
fn build_deepseek(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.deepseek;
    let config_path = context
        .config
        .root
        .join(DEEPSEEK_HARNESS_CONFIG_PATH)
        .to_string_lossy()
        .into_owned();
    let mut additions = Vec::new();
    addition(&mut additions, "DEEPSEEK_HARNESS_CONFIG", &config_path);
    addition(
        &mut additions,
        "DEEPSEEK_HARNESS_SESSION_ROOT",
        &options.session_root.to_string_lossy(),
    );
    addition(
        &mut additions,
        names::DEEPSEEK_HARNESS_ACP_BIN,
        &options.cli_path,
    );
    additions.push((
        "DSH_MODEL".to_owned(),
        command_or(&options.model, DEEPSEEK_DEFAULT_MODEL),
    ));
    additions.push((
        "DSH_PERMISSION_MODE".to_owned(),
        if context.is_full_permission() {
            DSH_PERMISSION_FULL.to_owned()
        } else {
            DSH_PERMISSION_WORKSPACE.to_owned()
        },
    ));
    let command = command_or(
        &options.cli_path,
        definition.setup.adapter_command.unwrap_or_default(),
    );
    // `scripts/deepseek-harness-acp.mjs:82` — the harness takes its composition
    // on the command line, not from the environment alone.
    let spawn = SpawnSpec::new(command)
        .args(vec!["--config".to_owned(), config_path])
        .cwd(&options.directory)
        .env(child_env(definition, context, &additions));
    let mut profile = base_profile(definition, context, definition.label.to_owned(), spawn);
    profile.session_instructions = Some(SessionInstructions::DeepSeek);
    profile.process_model_configuration = true;
    Ok(profile)
}

/// `server/src/agent/backends/pi.mjs:6-47`, plus the adapter variables
/// `scripts/pi-acp.mjs:12-22` sets.
fn build_pi(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.pi;
    let mut additions = Vec::new();
    addition(&mut additions, names::PI_ACP_BIN, &options.cli_path);
    // `scripts/pi-acp.mjs:12-22` — pi-acp reads `PI_ACP_PI_COMMAND`; upstream
    // respects that adapter-native override first, then `PI_BIN`, then PATH.
    let pi_command = ["PI_ACP_PI_COMMAND", "PI_BIN"]
        .into_iter()
        .map(|name| context.env.get_trimmed(name).to_owned())
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| context.finder.find("pi"));
    addition(&mut additions, "PI_BIN", &pi_command);
    addition(&mut additions, "PI_ACP_PI_COMMAND", &pi_command);
    let command = command_or(
        &options.cli_path,
        definition.setup.adapter_command.unwrap_or_default(),
    );
    let spawn = SpawnSpec::new(command)
        .cwd(&options.directory)
        .env(child_env(definition, context, &additions));
    let mut profile = base_profile(definition, context, definition.label.to_owned(), spawn);
    profile.session_instructions = Some(SessionInstructions::Pi);
    Ok(profile)
}

/// `server/src/agent/backends/generic-acp.mjs:7-51`.
///
/// The two refusals are the whole of this driver's logic. The second is a
/// safety property, not a limitation: a user-supplied agent has no declared
/// approval gate, so VIA cannot claim one switch turns full permission on
/// safely.
fn build_generic_acp(
    definition: &BackendDefinition,
    context: &LaunchContext<'_>,
) -> Result<BackendProfile, BackendsError> {
    let options = &context.config.backends.acp;
    let command = options.cli_path.trim();
    if command.is_empty() {
        return Err(BackendsError::GenericAcpRequiresCommand);
    }
    if context.is_full_permission() {
        return Err(BackendsError::GenericAcpFullPermissionUnsafe);
    }
    let label = command_or(&options.label, definition.label);
    let spawn = SpawnSpec::new(command)
        .args(options.args.clone())
        .cwd(&options.directory)
        .env(child_env(definition, context, &[]));
    Ok(base_profile(definition, context, label, spawn))
}

/// Every catalogued id has a driver, and every driver a catalogued id.
///
/// Upstream gets this from `validateBackendDriver` running over a hard-coded
/// list at import (`registry.mjs:47-56`); here it is a table invariant, asserted
/// by `tests/registry.rs` rather than trusted.
#[must_use]
pub fn table_matches_catalog() -> bool {
    backend_ids() == backend_names()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect::MissingFinder;

    fn config() -> Config {
        Config {
            root: PathBuf::from("/opt/via"),
            agent_timeout_ms: 60_000,
            ..Config::default()
        }
    }

    fn context<'a>(
        config: &'a Config,
        env: &'a EnvMap,
        permission_mode: &'a str,
    ) -> LaunchContext<'a> {
        LaunchContext {
            config,
            env,
            ownership: Ownership::Owned,
            permission_mode,
            owner_id: "user_personal",
            finder: &MissingFinder,
        }
    }

    #[test]
    fn the_table_is_the_catalog() {
        assert!(table_matches_catalog());
        assert_eq!(backend_ids().len(), 12);
    }

    #[test]
    fn every_driver_declares_the_catalog_label() {
        for driver in backend_drivers() {
            let definition = backend_definition(driver.id).expect("catalogued");
            assert_eq!(driver.label, definition.label, "{}", driver.id);
            assert_eq!(
                driver.capabilities,
                backend_capabilities(driver.id).expect("declared"),
                "{}",
                driver.id
            );
        }
    }

    #[test]
    fn lookup_normalises_and_refuses() {
        assert_eq!(backend_driver("  CODEX ").expect("normalised").id, "codex");
        assert!(has_backend_driver("openclaw"));
        assert!(!has_backend_driver("none"));
        assert!(!has_backend_driver(""));
        match backend_driver("nope") {
            Err(BackendsError::UnsupportedBackend { protocol }) => assert_eq!(protocol, "nope"),
            other => panic!("wrong result: {other:?}"),
        }
    }

    #[test]
    fn the_generic_acp_backend_refuses_without_a_command() {
        let config = config();
        let env = EnvMap::new();
        let error = create_backend_profile("acp", &context(&config, &env, "native"))
            .expect_err("ACP_COMMAND is unset");
        assert!(matches!(error, BackendsError::GenericAcpRequiresCommand));
    }

    #[test]
    fn the_generic_acp_backend_refuses_full_permission() {
        let mut config = config();
        config.backends.acp.cli_path = "/opt/agent".to_owned();
        let env = EnvMap::new();
        let error = create_backend_profile("acp", &context(&config, &env, "full"))
            .expect_err("full is not offered");
        assert!(matches!(
            error,
            BackendsError::GenericAcpFullPermissionUnsafe
        ));
    }

    #[test]
    fn deepseek_carries_its_frozen_capability_set_onto_the_profile() {
        let config = config();
        let env = EnvMap::new();
        let profile = create_backend_profile("deepseek", &context(&config, &env, "native"))
            .expect("deepseek builds");
        assert_eq!(profile.capabilities, crate::capability::DEEPSEEK);
        assert_eq!(profile.acp.capabilities, profile.capabilities);
    }
}
