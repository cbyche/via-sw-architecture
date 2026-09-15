//! The layered configuration resolver.
//!
//! Upstream evaluates `process.env.X ?? 'default'` at module scope, scattered
//! across `server/src/core/config.mjs`, `shared/runtime-environment.mjs`,
//! `shared/realtime-provider-catalog.mjs` and `shared/logger.mjs`. The defaults
//! that fall out of that are contract — `docs/reference/contracts.json`
//! catalogues 75 of them — so VIA collects them into **one**
//! [`impl Default for Config`] and layers everything else on top of it.
//! `tests/config_snapshot.rs` snapshots the result under an empty environment,
//! which makes any change to any default a reviewable diff.
//!
//! # Layering, lowest to highest
//!
//! 1. [`Config::default`] — the literals.
//! 2. The configuration **file**, `<data>/config.env`. It fills only keys the
//!    environment has not already defined, so an empty shell assignment masks
//!    it (`shared/runtime-environment.mjs:86-88`).
//! 3. The `VIA_*` **environment**.
//! 4. Explicit **overrides** — [`GatewayOptions`], the host-facing options an
//!    embedding process passes. They are applied as environment entries through
//!    the catalogued `gatewayOptionsEnvironment` mapping
//!    (`shared/gateway-options.mjs:14-41`), so a host option and the variable it
//!    corresponds to cannot drift apart.
//!
//! # Purity
//!
//! [`resolve`] reads no environment, no filesystem and no clock. The real
//! environment is sampled once by the binary ([`EnvMap::from_process`]), and the
//! two host facts a pure function cannot invent — the home directory and the
//! working directory — are parameters on [`Overrides`]. `std::env::set_var` is
//! `unsafe` in edition 2024; a pure loader means the configuration tests never
//! need it, and never need to serialize against each other.
//!
//! # Structural deviation
//!
//! Upstream flattens the realtime frontend into `config.audioProvider`,
//! `config.audioModel`, `config.audioVoice`, `config.audioRealtimeBaseUrl`,
//! `config.dashscopeApiKey`, `config.realtimeConfigSignature`,
//! `config.speechToSpeech*`. VIA nests them under [`Config::realtime`] because
//! they are resolved together by one function and are meaningful only together.
//! Recorded in `docs/deviations/phase-1.md`.

pub mod backend;
pub mod names;
pub mod realtime;

use std::path::{Path, PathBuf};

use via_catalog::{
    BACKEND_NONE_SENTINEL, Ownership, backend_definition, effective_backend_permission_mode,
    normalize_backend_protocol, resolve_backend_ownership,
};
use via_i18n::Locale;

use crate::env::{Bounds, EnvMap, integer_setting, lowercase_equals};
use crate::envfile::parse_env_map;
use crate::error::CoreError;
use crate::identity::IdentityMode;
use crate::paths::{InstallPaths, resolve_path};
use crate::secret::Secret;
use crate::security::parse_allowed_origins;

pub use backend::{
    AcpOptions, BackendModels, BackendOptions, ClaudeOptions, CliBackendOptions, CodeBuddyOptions,
    CodexOptions, DeepSeekOptions, OpenClawOptions, OpenCodeOptions, QoderOptions,
    resolve_acp_args, resolve_backend_models, resolve_backend_workspace,
    resolve_opencode_coordinator_agent,
};
pub use realtime::{RealtimeFrontend, resolve_realtime_frontend};

/// The two backend permission modes.
///
/// **External contract** — `server/src/core/config.mjs:133`. Kept as strings
/// rather than an enum because
/// [`effective_backend_permission_mode`](via_catalog::effective_backend_permission_mode)
/// deliberately passes an unrecognised mode through for its caller to reject in
/// context.
pub const BACKEND_PERMISSION_MODES: &[&str] = &["native", "full"];

/// Host-facing Gateway options.
///
/// **External contract** — `shared/gateway-options.mjs:14-41`. The mapping to
/// environment entries is the single translation from what a host asks for to
/// what the Gateway reads, and **an option that was not passed emits nothing**,
/// so "unset means the environment decides" holds for every embedding shape.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GatewayOptions {
    /// `VIA_CONFIG_DIR`.
    pub config_dir: Option<PathBuf>,
    /// `HOST`.
    pub host: Option<String>,
    /// `PORT`.
    pub port: Option<u16>,
    /// `AGENT_PROTOCOL`.
    pub backend: Option<BackendSelection>,
    /// `VIA_WAKE_WORD_ENABLED`, as the literal `true` or `false`.
    pub wake_word: Option<bool>,
    /// `VIA_GATEWAY_OWNER`.
    pub owner: Option<String>,
    /// `VIA_LOG_CONSOLE`, as the literal `1` or `0`.
    pub log_console: Option<bool>,
}

/// What a host asked for when it named a backend.
///
/// The three-way distinction is load-bearing: *not passed* leaves
/// `AGENT_PROTOCOL` alone, while [`Self::None`] sets it to the **empty string**
/// rather than deleting it. `docs/reference/contracts.json` explains why:
/// "Deleting the key would let the child Gateway reload AGENT_PROTOCOL from
/// config.env and silently start a backend the user asked to disable."
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendSelection {
    /// No backend: frontend-only.
    None,
    /// A named backend id.
    Named(String),
}

impl GatewayOptions {
    /// The environment entries these options imply.
    ///
    /// **External contract** — `shared/gateway-options.mjs:14-41`.
    #[must_use]
    pub fn to_environment(&self) -> EnvMap {
        let mut env = EnvMap::new();
        if let Some(directory) = &self.config_dir {
            env.set(names::CONFIG_DIR, directory.display().to_string());
        }
        if let Some(host) = &self.host {
            env.set(names::HOST, host.clone());
        }
        if let Some(port) = self.port {
            env.set(names::PORT, port.to_string());
        }
        if let Some(backend) = &self.backend {
            env.set(
                names::AGENT_PROTOCOL,
                match backend {
                    BackendSelection::None => String::new(),
                    BackendSelection::Named(id) => id.clone(),
                },
            );
        }
        if let Some(enabled) = self.wake_word {
            env.set(
                names::WAKE_WORD_ENABLED,
                if enabled { "true" } else { "false" },
            );
        }
        if let Some(owner) = &self.owner {
            env.set(names::GATEWAY_OWNER, owner.clone());
        }
        if let Some(console) = self.log_console {
            env.set(via_log::ENV_LOG_CONSOLE, if console { "1" } else { "0" });
        }
        env
    }
}

/// Everything [`resolve`] needs that is not an environment variable.
///
/// Two of these are host facts rather than settings. Node reaches them through
/// `os.homedir()` and `process.cwd()`, neither of which is an environment read;
/// a pure resolver has to be handed them. [`Overrides::from_process`] is the
/// impure convenience that fills them in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Overrides {
    /// The OS home directory, the base of `~/.config/via`.
    pub home_directory: PathBuf,
    /// The process working directory. A relative `VIA_CONFIG_DIR`,
    /// `QODER_CONFIG_DIR`, `CLAUDE_CONFIG_DIR` or `VIA_WAKE_WORD_MODEL_DIR`
    /// resolves against it, matching Node's single-argument `path.resolve`.
    pub working_directory: PathBuf,
    /// The installation root — upstream's `sourceRoot`
    /// (`server/src/core/config.mjs:19`). `VIA_RUNTIME_ROOT` overrides it, as
    /// it does upstream. `None` means "the working directory".
    pub runtime_root: Option<PathBuf>,
    /// Host-facing options, applied last.
    pub gateway: GatewayOptions,
}

impl Overrides {
    /// Sample the host facts from the real process.
    ///
    /// Falls back to `.` where the OS declines to answer, which is the one
    /// branch Node cannot reach (`os.homedir()` throws instead).
    #[must_use]
    pub fn from_process() -> Self {
        Self {
            home_directory: dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            runtime_root: None,
            gateway: GatewayOptions::default(),
        }
    }

    /// Replace the host-facing options.
    #[must_use]
    pub fn with_gateway(mut self, gateway: GatewayOptions) -> Self {
        self.gateway = gateway;
        self
    }
}

/// The resolved configuration.
///
/// Field order is the order upstream's object literal declares them, so a
/// reader can put the two side by side.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// The installation root every relative path override resolves against.
    pub root: PathBuf,
    /// Where runtime state and user assets live.
    pub paths: InstallPaths,
    /// Listen address.
    pub host: String,
    /// Listen port; `0` asks the OS to choose.
    pub port: u16,
    /// The realtime frontend, resolved together.
    pub realtime: RealtimeFrontend,
    /// Origins allowed to reach the Gateway from a browser. Empty by default.
    pub allowed_origins: Vec<String>,
    /// HMAC key for the owner cookie. Generated into `state.env` on first run.
    pub auth_secret: Secret,
    /// Whether owners come from a cookie or are collapsed to one.
    pub identity_mode: IdentityMode,
    /// The owner every request resolves to in [`IdentityMode::Personal`].
    pub personal_owner_id: String,
    /// The selected backend agent id, or empty for frontend-only.
    pub agent_protocol: String,
    /// Whether the Gateway launches the backend or connects to a running one.
    pub backend_ownership: Ownership,
    /// The permission mode actually in effect, after
    /// `always_full_permission` normalisation.
    pub backend_permission_mode: String,
    /// Backend request timeout.
    pub agent_timeout_ms: i64,
    /// The twelve per-backend namespaces.
    pub backends: BackendOptions,
    /// Whether finished work is injected into the model's context.
    pub announce_into_context: bool,
    /// Character cap on an injected result.
    pub result_context_max_chars: i64,
    /// Coalescing window for announcements.
    pub announcement_batch_ms: i64,
    /// Cap on how many results one announcement batches.
    pub announcement_max_batch_items: i64,
    /// Silence required before an announcement may speak.
    pub announcement_quiet_ms: i64,
    /// How long to wait for a client to acknowledge an announcement.
    pub announcement_acknowledgement_timeout_ms: i64,
    /// Cap on announcement retries.
    pub announcement_max_retry_attempts: i64,
    /// Directory holding `PROMPT.md` and `ASSISTANT.md`.
    pub frontend_prompt_dir: PathBuf,
    /// The user's overriding assistant persona.
    pub assistant_profile_path: PathBuf,
    /// The long-term memory document.
    pub frontend_memory_path: PathBuf,
    /// The notes store.
    pub frontend_notes_path: PathBuf,
    /// The user personalization document.
    pub user_model_path: PathBuf,
    /// The Work store.
    pub task_state_path: PathBuf,
    /// The ACP session index.
    pub backend_session_state_path: PathBuf,
    /// How long a terminal Work is retained.
    pub task_terminal_ttl_ms: i64,
    /// How long an undelivered notification is retained.
    pub task_pending_notification_ttl_ms: i64,
    /// How long a delivery claim is held.
    pub task_notification_claim_ttl_ms: i64,
    /// Cap on terminal Work per owner.
    pub max_terminal_tasks_per_owner: i64,
    /// Global concurrent-Work cap.
    pub task_max_concurrent: i64,
    /// Per-owner concurrent-Work cap.
    pub task_max_concurrent_per_owner: i64,
    /// How long a conversation session is retained.
    pub conversation_session_ttl_ms: i64,
    /// Cap on retained conversation sessions.
    pub max_conversation_sessions: i64,
    /// How long an owner's memories are retained; `0` keeps them forever.
    pub frontend_memory_owner_ttl_ms: i64,
    /// Cap on retained memory owners.
    pub max_frontend_memory_owners: i64,
    /// Whether end-of-session memory extraction runs.
    pub memory_auto_enabled: bool,
    /// The text model the extractor calls.
    pub memory_model: String,
    /// The OpenAI-compatible endpoint the extractor calls.
    pub memory_base_url: String,
    /// The extractor's credential. Without one it stays silently disabled.
    pub memory_api_key: Secret,
    /// The extractor's audit log.
    pub memory_audit_path: PathBuf,
    /// Whether the reminder scheduler runs.
    pub reminder_scheduler_enabled: bool,
    /// Cap on reminders per owner.
    pub reminder_max_per_owner: i64,
    /// Timeout for a scheduled task.
    pub scheduled_task_timeout_ms: i64,
    /// Cadence of background-work progress checks.
    pub background_task_progress_check_ms: i64,
    /// Delay before an offline notification is raised.
    pub offline_notification_delay_ms: i64,
    /// Spread applied to overdue reminders on restart.
    pub reminder_stagger_ms: i64,
    /// Idle time before the Gateway sleeps; `0` disables sleeping.
    pub sleep_timeout_ms: i64,
    /// Whether wake-word detection runs.
    pub wake_word_enabled: bool,
    /// The wake phrase. See [`names::WAKE_WORD`] for why VIA has no default.
    pub wake_word: String,
    /// Where the wake-word model is installed.
    pub wake_word_model_directory: PathBuf,
    /// Whether the bundled computer-use MCP server is offered.
    pub computer_use: bool,
    /// Who launched this Gateway: `cli`, `desktop` or `service`.
    pub gateway_owner: String,
    /// The locale every user-facing message is rendered in.
    #[serde(serialize_with = "serialize_locale")]
    pub locale: Locale,
}

impl Default for Config {
    /// **The one place a configuration default literal appears.**
    ///
    /// Every value here is an external contract catalogued in
    /// `docs/reference/contracts.json`; `tests/contracts.rs` asserts each
    /// against the catalogue rather than against a retyped copy.
    ///
    /// Path fields are left empty: they are *derived* from the resolved
    /// directories, and the directories depend on the host. There is no correct
    /// literal for them, and inventing one would be a lie the snapshot would
    /// then enshrine. [`resolve`] fills them in; the filenames they are built
    /// from are file-path contracts owned by [`crate::paths`].
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            paths: InstallPaths::new(PathBuf::new(), PathBuf::new()),
            // `server/src/core/config.mjs:172`. Loopback-only, which is what
            // makes the implicit same-origin path safe.
            host: "127.0.0.1".to_owned(),
            // `server/src/core/config.mjs:175-177`.
            port: 3101,
            realtime: RealtimeFrontend::default(),
            // `server/src/core/config.mjs:196-199`. Empty, never `*`.
            allowed_origins: Vec::new(),
            // `server/src/core/config.mjs:200`.
            auth_secret: Secret::default(),
            // `server/src/core/config.mjs:201-203`.
            identity_mode: IdentityMode::Personal,
            // `server/src/core/config.mjs:204`.
            personal_owner_id: "user_personal".to_owned(),
            // `server/src/core/config.mjs:100-109`: unset means frontend-only.
            agent_protocol: String::new(),
            // `server/src/core/config.mjs:110-120`.
            backend_ownership: Ownership::Owned,
            // `server/src/core/config.mjs:128-130`. Security-relevant: `native`
            // means the backend asks before acting.
            backend_permission_mode: "native".to_owned(),
            // `server/src/core/config.mjs:208`.
            agent_timeout_ms: 300_000,
            backends: BackendOptions::default(),
            // `server/src/core/config.mjs:333-336`.
            announce_into_context: true,
            // `server/src/core/config.mjs:337-341`.
            result_context_max_chars: 6_000,
            // `server/src/core/config.mjs:342-346`.
            announcement_batch_ms: 120,
            // `server/src/core/config.mjs:347-351`.
            announcement_max_batch_items: 8,
            // `server/src/core/config.mjs:352-356`.
            announcement_quiet_ms: 350,
            // `server/src/core/config.mjs:357-361`.
            announcement_acknowledgement_timeout_ms: 120_000,
            // `server/src/core/config.mjs:362-366`.
            announcement_max_retry_attempts: 8,
            frontend_prompt_dir: PathBuf::new(),
            assistant_profile_path: PathBuf::new(),
            frontend_memory_path: PathBuf::new(),
            frontend_notes_path: PathBuf::new(),
            user_model_path: PathBuf::new(),
            task_state_path: PathBuf::new(),
            backend_session_state_path: PathBuf::new(),
            // `server/src/core/config.mjs:392-396`.
            task_terminal_ttl_ms: 86_400_000,
            // `server/src/core/config.mjs:397-401`.
            task_pending_notification_ttl_ms: 604_800_000,
            // `server/src/core/config.mjs:402-406`.
            task_notification_claim_ttl_ms: 60_000,
            // `server/src/core/config.mjs:407-411`.
            max_terminal_tasks_per_owner: 100,
            // `server/src/core/config.mjs:412-416`.
            task_max_concurrent: 4,
            // `server/src/core/config.mjs:417-421`.
            task_max_concurrent_per_owner: 2,
            // `server/src/core/config.mjs:422-426`.
            conversation_session_ttl_ms: 21_600_000,
            // `server/src/core/config.mjs:427-431`.
            max_conversation_sessions: 500,
            // `server/src/core/config.mjs:433-437`. Zero keeps explicit
            // personal memories until the user removes them.
            frontend_memory_owner_ttl_ms: 0,
            // `server/src/core/config.mjs:438-442`.
            max_frontend_memory_owners: 1_000,
            // `server/src/core/config.mjs:447-449`.
            memory_auto_enabled: true,
            // `server/src/core/config.mjs:450-451`. A DashScope model id: KEEP.
            memory_model: "qwen-flash".to_owned(),
            // `server/src/core/config.mjs:452-455`.
            memory_base_url: backend::DASHSCOPE_COMPATIBLE_BASE_URL.to_owned(),
            // `server/src/core/config.mjs:456-458`.
            memory_api_key: Secret::default(),
            memory_audit_path: PathBuf::new(),
            // `server/src/core/config.mjs:463-465`.
            reminder_scheduler_enabled: true,
            // `server/src/core/config.mjs:466-470`.
            reminder_max_per_owner: 50,
            // `server/src/core/config.mjs:471-475`.
            scheduled_task_timeout_ms: 1_800_000,
            // `server/src/core/config.mjs:476-481`.
            background_task_progress_check_ms: 300_000,
            // `server/src/core/config.mjs:482-486`.
            offline_notification_delay_ms: 5_000,
            // `server/src/core/config.mjs:487-491`.
            reminder_stagger_ms: 30_000,
            // `server/src/core/config.mjs:495-499`. Zero disables sleeping.
            sleep_timeout_ms: 0,
            // `server/src/core/config.mjs:500-502`.
            wake_word_enabled: false,
            // VIA-owned. Upstream hard-codes a phrase; `docs/architecture.md`
            // §16 makes it configuration and defers the choice to phase 6, so
            // the default is "unset" rather than a placeholder that would ship.
            wake_word: String::new(),
            wake_word_model_directory: PathBuf::new(),
            // `server/src/agent/builtin-mcp.mjs:22-26`: an unusual default-on
            // toggle — unset means enabled.
            computer_use: true,
            // `shared/gateway-options.mjs:37`: unset until a host says.
            gateway_owner: String::new(),
            // `docs/architecture.md` §16: `VIA_LOCALE` → OS locale → `en`.
            locale: Locale::default(),
        }
    }
}

impl Config {
    /// Where runtime state lives.
    #[must_use]
    pub fn config_directory(&self) -> &Path {
        self.paths.config_directory()
    }

    /// Where user assets live.
    #[must_use]
    pub fn data_directory(&self) -> &Path {
        self.paths.data_directory()
    }

    /// Whether a backend agent is configured at all.
    ///
    /// `agent` mode degrades to `direct` when this is false
    /// (`docs/architecture.md` §2), rather than failing.
    #[must_use]
    pub fn has_backend(&self) -> bool {
        !self.agent_protocol.is_empty()
    }
}

/// Resolve a configuration.
///
/// `file` is the contents of `<data>/config.env`, already read — [`resolve`]
/// itself touches no filesystem. Pass `None` when there is no file.
///
/// # Errors
///
/// * [`CoreError::Catalog`] — an unsupported backend id, an unsupported backend
///   ownership, `external` for a backend that cannot be external, an
///   unsupported realtime provider, or an unknown realtime model id.
/// * [`CoreError::UnsupportedBackendPermissionMode`] — a mode other than
///   `native` or `full`, **only** when a backend is configured. Upstream checks
///   it under the same condition (`server/src/core/config.mjs:131-138`), so a
///   frontend-only install is not refused for a stale variable.
/// * [`CoreError::AcpArgsNotJson`] / [`CoreError::AcpArgsNotStringArray`].
/// * [`CoreError::BackendWorkspaceMissing`] — a catalog entry with no workspace
///   variable, which no shipped backend has.
pub fn resolve(
    env: &EnvMap,
    file: Option<&str>,
    overrides: &Overrides,
) -> Result<Config, CoreError> {
    let mut effective = env.clone();
    if let Some(contents) = file {
        // `parseEnv` builds an object, so a key repeated inside one file keeps
        // its last assignment; the *file* then loses to the environment.
        for (key, value) in parse_env_map(contents) {
            effective.set_if_absent(key, value);
        }
    }
    effective.overlay(&overrides.gateway.to_environment());
    resolve_effective(&effective, overrides)
}

/// The tail of [`resolve`], after layering. Split out so a caller that has
/// already flattened its own layers can reuse it.
fn resolve_effective(env: &EnvMap, overrides: &Overrides) -> Result<Config, CoreError> {
    let defaults = Config::default();
    let cwd = overrides.working_directory.as_path();
    let installation_root = overrides
        .runtime_root
        .clone()
        .unwrap_or_else(|| cwd.to_path_buf());
    let root = match env.get_truthy(names::RUNTIME_ROOT) {
        Some(configured) => resolve_path(cwd, configured),
        None => installation_root,
    };
    let paths = InstallPaths::from_env(env, &overrides.home_directory, cwd);
    let data_directory = paths.data_directory().to_path_buf();

    let realtime = resolve_realtime_frontend(env)?;

    // ── backend selection ───────────────────────────────────────────────────
    let agent_protocol = normalize_backend_protocol(env.get(names::AGENT_PROTOCOL).unwrap_or(""));
    let definition = if agent_protocol.is_empty() {
        None
    } else {
        Some(backend_definition(&agent_protocol).ok_or_else(|| {
            CoreError::Catalog(via_catalog::CatalogError::UnsupportedBackend {
                protocol: agent_protocol.clone(),
            })
        })?)
    };

    let backend_ownership = match definition {
        None => defaults.backend_ownership,
        Some(definition) => {
            let base_url_configured = definition
                .base_url_environment
                .is_some_and(|name| !env.get_trimmed(name).is_empty());
            resolve_backend_ownership(
                &agent_protocol,
                base_url_configured,
                env.get(names::BACKEND_OWNERSHIP).unwrap_or(""),
            )?
        }
    };

    let requested_permission_mode = env
        .get_truthy(names::BACKEND_PERMISSION_MODE)
        .unwrap_or(defaults.backend_permission_mode.as_str())
        .to_lowercase();
    if definition.is_some()
        && !BACKEND_PERMISSION_MODES.contains(&requested_permission_mode.as_str())
    {
        return Err(CoreError::UnsupportedBackendPermissionMode {
            requested: requested_permission_mode,
        });
    }
    let backend_permission_mode =
        effective_backend_permission_mode(&agent_protocol, &requested_permission_mode);

    let models = resolve_backend_models(env);
    let backends = resolve_backend_options(env, &root, cwd, &data_directory, &paths, &models)?;

    // ── path overrides ──────────────────────────────────────────────────────
    let rooted = |keys: &[&str], fallback: PathBuf| -> PathBuf {
        match env.first_truthy(keys) {
            Some(configured) => resolve_path(&root, configured),
            None => fallback,
        }
    };

    Ok(Config {
        root: root.clone(),
        host: env
            .get_truthy(names::HOST)
            .unwrap_or(defaults.host.as_str())
            .to_owned(),
        port: resolve_port(env, defaults.port),
        realtime,
        allowed_origins: parse_allowed_origins(env.get(names::ALLOWED_ORIGINS)),
        auth_secret: Secret::new(env.get_truthy(names::AUTH_SECRET).unwrap_or_default()),
        identity_mode: IdentityMode::from_env_value(env.get(names::IDENTITY_MODE)),
        personal_owner_id: env
            .get_truthy(names::PERSONAL_OWNER_ID)
            .unwrap_or(defaults.personal_owner_id.as_str())
            .to_owned(),
        agent_protocol,
        backend_ownership,
        backend_permission_mode,
        agent_timeout_ms: integer_setting(
            env.get(names::AGENT_TIMEOUT_MS),
            defaults.agent_timeout_ms,
            Bounds::min(10_000.0),
        ),
        backends,
        announce_into_context: lowercase_equals(
            env.get(names::ANNOUNCE_INTO_CONTEXT),
            "true",
            "true",
        ),
        result_context_max_chars: integer_setting(
            env.get(names::RESULT_CONTEXT_MAX_CHARS),
            defaults.result_context_max_chars,
            Bounds::min(256.0),
        ),
        announcement_batch_ms: integer_setting(
            env.get(names::ANNOUNCEMENT_BATCH_MS),
            defaults.announcement_batch_ms,
            Bounds::range(0.0, 1_000.0),
        ),
        announcement_max_batch_items: integer_setting(
            env.get(names::ANNOUNCEMENT_MAX_BATCH_ITEMS),
            defaults.announcement_max_batch_items,
            Bounds::range(1.0, 32.0),
        ),
        announcement_quiet_ms: integer_setting(
            env.get(names::ANNOUNCEMENT_QUIET_MS),
            defaults.announcement_quiet_ms,
            Bounds::range(0.0, 2_000.0),
        ),
        announcement_acknowledgement_timeout_ms: integer_setting(
            env.get(names::ANNOUNCEMENT_ACK_TIMEOUT_MS),
            defaults.announcement_acknowledgement_timeout_ms,
            Bounds::min(10_000.0),
        ),
        announcement_max_retry_attempts: integer_setting(
            env.get(names::ANNOUNCEMENT_MAX_RETRIES),
            defaults.announcement_max_retry_attempts,
            Bounds::range(1.0, 32.0),
        ),
        frontend_prompt_dir: rooted(
            &[names::FRONTEND_PROMPT_DIR],
            resolve_path(&root, crate::paths::FRONTEND_PROMPT_DIRECTORY),
        ),
        assistant_profile_path: rooted(
            &[names::ASSISTANT_PROFILE_PATH],
            paths.assistant_profile_file(),
        ),
        // Two-name fallback chains: the legacy spelling is still honoured, and
        // the modern one wins (`server/src/core/config.mjs:373-385`).
        frontend_memory_path: rooted(
            &[names::MEMORY_PATH, names::FRONTEND_MEMORY_PATH],
            paths.memory_file(),
        ),
        frontend_notes_path: rooted(&[names::FRONTEND_NOTES_PATH], paths.frontend_notes_file()),
        user_model_path: rooted(
            &[names::USER_MODEL_PATH, names::USER_PROFILE_PATH],
            paths.user_model_file(),
        ),
        task_state_path: rooted(&[names::TASK_STATE_PATH], paths.task_state_file()),
        backend_session_state_path: rooted(
            &[names::BACKEND_SESSION_STATE_PATH],
            paths.backend_session_state_file(),
        ),
        task_terminal_ttl_ms: integer_setting(
            env.get(names::TASK_TERMINAL_TTL_MS),
            defaults.task_terminal_ttl_ms,
            Bounds::min(60_000.0),
        ),
        task_pending_notification_ttl_ms: integer_setting(
            env.get(names::TASK_NOTIFICATION_TTL_MS),
            defaults.task_pending_notification_ttl_ms,
            Bounds::min(60_000.0),
        ),
        task_notification_claim_ttl_ms: integer_setting(
            env.get(names::TASK_NOTIFICATION_CLAIM_TTL_MS),
            defaults.task_notification_claim_ttl_ms,
            Bounds::min(5_000.0),
        ),
        max_terminal_tasks_per_owner: integer_setting(
            env.get(names::MAX_TERMINAL_TASKS_PER_OWNER),
            defaults.max_terminal_tasks_per_owner,
            Bounds::min(10.0),
        ),
        task_max_concurrent: integer_setting(
            env.get(names::TASK_MAX_CONCURRENT),
            defaults.task_max_concurrent,
            Bounds::range(1.0, 64.0),
        ),
        task_max_concurrent_per_owner: integer_setting(
            env.get(names::TASK_MAX_CONCURRENT_PER_OWNER),
            defaults.task_max_concurrent_per_owner,
            Bounds::range(1.0, 16.0),
        ),
        conversation_session_ttl_ms: integer_setting(
            env.get(names::SESSION_TTL_MS),
            defaults.conversation_session_ttl_ms,
            Bounds::min(60_000.0),
        ),
        max_conversation_sessions: integer_setting(
            env.get(names::MAX_SESSIONS),
            defaults.max_conversation_sessions,
            Bounds::min(10.0),
        ),
        frontend_memory_owner_ttl_ms: integer_setting(
            env.get(names::MEMORY_OWNER_TTL_MS),
            defaults.frontend_memory_owner_ttl_ms,
            Bounds::min(0.0),
        ),
        max_frontend_memory_owners: integer_setting(
            env.get(names::MAX_MEMORY_OWNERS),
            defaults.max_frontend_memory_owners,
            Bounds::min(10.0),
        ),
        // Disabled *only* by the literal `off`; anything else leaves it on.
        memory_auto_enabled: !lowercase_equals(env.get(names::MEMORY_AUTO), "on", "off"),
        memory_model: {
            let configured = env.get_trimmed(names::MEMORY_MODEL);
            if configured.is_empty() {
                defaults.memory_model.clone()
            } else {
                configured.to_owned()
            }
        },
        // Not trimmed: upstream applies only `.replace(/\/+$/, '')` here, and a
        // value with surrounding whitespace keeps it.
        memory_base_url: backend::strip_trailing_slashes(
            env.get_truthy(names::MEMORY_BASE_URL)
                .unwrap_or(defaults.memory_base_url.as_str()),
        )
        .to_owned(),
        memory_api_key: Secret::new(
            env.first_truthy(&[names::MEMORY_API_KEY, names::DASHSCOPE_API_KEY])
                .unwrap_or_default(),
        ),
        memory_audit_path: paths.memory_audit_file(),
        reminder_scheduler_enabled: lowercase_equals(
            env.get(names::REMINDER_SCHEDULER),
            "true",
            "true",
        ),
        reminder_max_per_owner: integer_setting(
            env.get(names::REMINDER_MAX_PER_OWNER),
            defaults.reminder_max_per_owner,
            Bounds::range(1.0, 500.0),
        ),
        scheduled_task_timeout_ms: integer_setting(
            env.get(names::SCHEDULED_TASK_TIMEOUT_MS),
            defaults.scheduled_task_timeout_ms,
            Bounds::min(60_000.0),
        ),
        background_task_progress_check_ms: integer_setting(
            env.first_truthy(&[
                names::BACKGROUND_TASK_PROGRESS_CHECK_MS,
                names::SCHEDULED_TASK_PROGRESS_CHECK_MS,
            ]),
            defaults.background_task_progress_check_ms,
            Bounds::min(30_000.0),
        ),
        offline_notification_delay_ms: integer_setting(
            env.get(names::OFFLINE_NOTIFICATION_DELAY_MS),
            defaults.offline_notification_delay_ms,
            Bounds::range(1_000.0, 120_000.0),
        ),
        reminder_stagger_ms: integer_setting(
            env.get(names::REMINDER_STAGGER_MS),
            defaults.reminder_stagger_ms,
            Bounds::range(0.0, 300_000.0),
        ),
        // Configured in seconds, published in milliseconds.
        sleep_timeout_ms: integer_setting(
            env.get(names::SLEEP_TIMEOUT_SECONDS),
            0,
            Bounds::range(0.0, 86_400.0),
        ) * 1_000,
        wake_word_enabled: lowercase_equals(env.get(names::WAKE_WORD_ENABLED), "", "true"),
        wake_word: env
            .get_truthy(names::WAKE_WORD)
            .unwrap_or(defaults.wake_word.as_str())
            .to_owned(),
        // Resolved against the working directory, not the runtime root —
        // upstream uses single-argument `resolve()` here
        // (`server/src/core/config.mjs:504-506`).
        wake_word_model_directory: match env.get_truthy(names::WAKE_WORD_MODEL_DIR) {
            Some(configured) => resolve_path(cwd, configured),
            None => paths.wake_word_model_directory(),
        },
        computer_use: resolve_computer_use(env.get(names::COMPUTER_USE)),
        gateway_owner: env.get_trimmed(names::GATEWAY_OWNER).to_owned(),
        locale: via_i18n::resolve_locale(&env.reader()),
        paths,
    })
}

/// `PORT=0` means "let the OS choose"; anything else is clamped into
/// `[1, 65535]`.
///
/// **External contract** — `server/src/core/config.mjs:175-177`. An
/// out-of-range value is clamped, never rejected.
fn resolve_port(env: &EnvMap, fallback: u16) -> u16 {
    if env.get(names::PORT).unwrap_or_default().trim() == "0" {
        return 0;
    }
    let resolved = integer_setting(
        env.get(names::PORT),
        i64::from(fallback),
        Bounds::range(1.0, 65_535.0),
    );
    u16::try_from(resolved).unwrap_or(fallback)
}

/// The bundled computer-use MCP server's unusual default-**on** toggle.
///
/// **External contract** — `server/src/agent/builtin-mcp.mjs:22-26`: disabled
/// when the trimmed, lowercased value is one of `false`, `off`, `0`, `no`,
/// `disabled`; empty or unset means enabled.
#[must_use]
pub fn resolve_computer_use(value: Option<&str>) -> bool {
    const DISABLED: &[&str] = &["false", "off", "0", "no", "disabled"];
    let normalized = value.unwrap_or_default().trim().to_lowercase();
    !DISABLED.contains(&normalized.as_str())
}

/// Whether `full` permission may be used with this ownership.
///
/// **External contract** — `server/src/process/managed-backend.mjs:61-69`.
/// [`resolve`] deliberately does **not** apply it: upstream checks it in
/// `managed-backend`, at spawn time, and refusing at configuration time would
/// reject a startup upstream accepts. `via-process` calls this.
///
/// # Errors
///
/// [`CoreError::FullPermissionRequiresOwnedBackend`].
pub fn assert_full_permission_allowed(
    permission_mode: &str,
    ownership: Ownership,
) -> Result<(), CoreError> {
    if permission_mode == "full" && ownership != Ownership::Owned {
        return Err(CoreError::FullPermissionRequiresOwnedBackend);
    }
    Ok(())
}

/// Build the twelve per-backend namespaces.
#[allow(clippy::too_many_lines)]
fn resolve_backend_options(
    env: &EnvMap,
    root: &Path,
    working_directory: &Path,
    data_directory: &Path,
    paths: &InstallPaths,
    models: &BackendModels,
) -> Result<BackendOptions, CoreError> {
    let workspace = |protocol: &str| resolve_backend_workspace(protocol, env, root, data_directory);
    let trimmed = |name: &str| env.get_trimmed(name).to_owned();
    // Upstream uses single-argument `resolve()` for these two, so they anchor
    // to the working directory rather than the runtime root
    // (`server/src/core/config.mjs:254-256,306-308`).
    let absolute = |name: &str| match env.get_truthy(name) {
        Some(configured) => resolve_path(working_directory, configured),
        None => PathBuf::new(),
    };

    // Upstream enters managed-Bailian mode only when a model is configured, a
    // DashScope key is present, and the user has not pointed OpenClaw at their
    // own config (`server/src/core/config.mjs:122-127`).
    let managed_openclaw_bailian = !models.common.is_empty()
        && env.get_truthy(names::DASHSCOPE_API_KEY).is_some()
        && env.get_truthy(names::OPENCLAW_CONFIG_PATH).is_none();

    let shared_backend_agent = trimmed(names::BACKEND_AGENT);
    let openclaw_coordinator = if !shared_backend_agent.is_empty() {
        shared_backend_agent
    } else {
        let legacy = backend::legacy_backend_agent(
            env.get(names::OPENCLAW_COORDINATOR_AGENT),
            backend::LEGACY_OPENCLAW_COORDINATOR_AGENT,
        );
        if !legacy.is_empty() {
            legacy
        } else if managed_openclaw_bailian {
            backend::VIA_BACKEND_AGENT_ID.to_owned()
        } else {
            String::new()
        }
    };

    let workspace_id = env.get_truthy(names::DASHSCOPE_WORKSPACE_ID);
    let derived_model_url = |suffix: &str| -> String {
        if models.common.is_empty() {
            String::new()
        } else {
            format!(
                "{}{suffix}",
                backend::dashscope_compatible_base_url(workspace_id)
            )
        }
    };

    Ok(BackendOptions {
        opencode: OpenCodeOptions {
            base_url: backend::strip_trailing_slashes(
                env.get_truthy(names::OPENCODE_BASE_URL)
                    .or_else(|| backend_definition("opencode").and_then(|d| d.default_base_url))
                    .unwrap_or_default(),
            )
            .to_owned(),
            model: models.open_code.clone(),
            directory: workspace("opencode")?,
            coordinator_agent: resolve_opencode_coordinator_agent(env),
        },
        openclaw: OpenClawOptions {
            base_url: backend::strip_trailing_slashes(
                env.get_truthy(names::OPENCLAW_BASE_URL)
                    .or_else(|| backend_definition("openclaw").and_then(|d| d.default_base_url))
                    .unwrap_or_default(),
            )
            .to_owned(),
            token: Secret::new(
                env.first_truthy(&[names::OPENCLAW_GATEWAY_TOKEN, names::AGENT_API_KEY])
                    .unwrap_or_default(),
            ),
            token_file: match env.get_truthy(names::OPENCLAW_GATEWAY_TOKEN_FILE) {
                Some(configured) => resolve_path(root, configured),
                None => openclaw_state_directory(env, root, paths)
                    .join(crate::paths::OPENCLAW_TOKEN_FILE_NAME),
            },
            model: models.open_claw.clone(),
            directory: workspace("openclaw")?,
            cli_path: trimmed(names::OPENCLAW_ACP_BIN),
            coordinator_agent: openclaw_coordinator,
        },
        qoder: QoderOptions {
            model: models.qoder.trim().to_owned(),
            directory: workspace("qoder")?,
            cli_path: env
                .first_truthy(&[names::QODERCLI_PATH, names::QODER_CLI_PATH])
                .unwrap_or_default()
                .trim()
                .to_owned(),
            config_directory: absolute(names::QODER_CONFIG_DIR),
        },
        qwen: CliBackendOptions {
            model: models.qwen.trim().to_owned(),
            directory: workspace("qwen")?,
            cli_path: trimmed(names::QWEN_CODE_BIN),
        },
        kimi: CliBackendOptions {
            model: models.kimi.trim().to_owned(),
            directory: workspace("kimi")?,
            cli_path: trimmed(names::KIMI_CODE_BIN),
        },
        hermes: CliBackendOptions {
            model: models.hermes.trim().to_owned(),
            directory: workspace("hermes")?,
            cli_path: trimmed(names::HERMES_BIN),
        },
        codebuddy: CodeBuddyOptions {
            model: models.code_buddy.trim().to_owned(),
            model_url: match env.get_truthy(names::CODEBUDDY_MODEL_URL) {
                Some(configured) => configured.to_owned(),
                None => derived_model_url(backend::CHAT_COMPLETIONS_SUFFIX),
            },
            directory: workspace("codebuddy")?,
            cli_path: trimmed(names::CODEBUDDY_BIN),
        },
        codex: CodexOptions {
            model: models.codex.trim().to_owned(),
            model_url: backend::strip_trailing_slashes(
                &match env.get_truthy(names::CODEX_BASE_URL) {
                    Some(configured) => configured.to_owned(),
                    None => derived_model_url(""),
                },
            )
            .to_owned(),
            directory: workspace("codex")?,
            cli_path: trimmed(names::CODEX_ACP_BIN),
        },
        claude: ClaudeOptions {
            model: models.claude.trim().to_owned(),
            directory: workspace("claude")?,
            cli_path: trimmed(names::CLAUDE_CODE_ACP_BIN),
            claude_executable: trimmed(names::CLAUDE_CODE_EXECUTABLE),
            config_directory: absolute(names::CLAUDE_CONFIG_DIR),
        },
        deepseek: DeepSeekOptions {
            model: models.deep_seek_harness.clone(),
            directory: workspace("deepseek")?,
            cli_path: trimmed(names::DEEPSEEK_HARNESS_ACP_BIN),
            session_root: paths.deepseek_session_root(),
        },
        pi: CliBackendOptions {
            model: models.pi.trim().to_owned(),
            directory: workspace("pi")?,
            cli_path: trimmed(names::PI_ACP_BIN),
        },
        acp: AcpOptions {
            model: models.acp.trim().to_owned(),
            directory: workspace("acp")?,
            cli_path: trimmed(names::ACP_COMMAND),
            args: resolve_acp_args(env.get(names::ACP_ARGS))?,
            label: {
                let configured = env.get_trimmed(names::ACP_LABEL);
                if configured.is_empty() {
                    // `server/src/core/config.mjs:329`. Applied both when the
                    // variable is unset and when it trims to empty.
                    "ACP Agent".to_owned()
                } else {
                    configured.to_owned()
                }
            },
            coordinator_agent: trimmed(names::ACP_COORDINATOR_AGENT),
        },
    })
}

/// OpenClaw's state directory, honouring `VIA_OPENCLAW_STATE_DIR`.
///
/// **External contract** — `shared/runtime-environment.mjs:518-520`.
fn openclaw_state_directory(env: &EnvMap, root: &Path, paths: &InstallPaths) -> PathBuf {
    match env.get_truthy(names::OPENCLAW_STATE_DIR) {
        Some(configured) => resolve_path(root, configured),
        None => paths.openclaw_state_directory(),
    }
}

/// Whether a backend id names "no backend at all".
///
/// **External contract** — `shared/backend-catalog.mjs`: `none` normalises to
/// the empty string.
#[must_use]
pub fn is_no_backend(value: &str) -> bool {
    let normalized = value.trim().to_lowercase();
    normalized.is_empty() || normalized == BACKEND_NONE_SENTINEL
}

/// Serialize a locale as its two-letter wire code.
///
/// Written out rather than derived so `via-core` does not depend on `via-i18n`
/// deriving `Serialize` for a type whose wire spelling is already public API.
fn serialize_locale<S: serde::Serializer>(
    locale: &Locale,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(locale.as_str())
}
