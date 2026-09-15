//! The backend agent catalog.
//!
//! Ported from `shared/backend-catalog.mjs`. Twelve backends in catalog order,
//! plus the `none` sentinel that [`normalize_backend_protocol`] maps to the empty
//! string (frontend-only operation).
//!
//! # The environment allow-policy is a security boundary
//!
//! [`EnvironmentPolicy`] is the credential-namespace boundary, not a
//! convenience list. A child agent process receives exactly the names and
//! prefixes declared here, plus the shared system names and VIA-internal names
//! that `via-process` adds. Gateway secrets — the auth secret, the realtime API
//! key, the memory-extractor key, the speech-to-speech token — reach no backend.
//! `docs/fidelity.md` lists these allow-lists under "reproduced verbatim in
//! code"; the only edits are the identity renames mandated by
//! `docs/rebrand.md` (`QWEN_AUDIO_AGENT_*` → `VIA_*`, `QWAUDIO_*` → `VIA_*`).
//! Every vendor-owned name — `DASHSCOPE_API_KEY`, `QWEN_CODE_`, `ANTHROPIC_*`,
//! `OPENAI_*` — is KEEP.
//!
//! `qwen` here is the third-party Qwen Code CLI, not VIA: its id, label,
//! executable and `QWEN_CODE_` namespace are all KEEP.
//!
//! # What this table deliberately does not carry
//!
//! [`InstallationSpec::steps`] is declared and empty. The pinned npm
//! coordinates and shell installers are execution machinery and live with the
//! executor, in `via_backends::install`; the *shape* is present here so the
//! catalog's `installation: null` distinction — `acp` has no installer at all,
//! and that is contract — stays representable. [`Onboarding`] likewise carries
//! the command and the auth probe but not upstream's `hint`, which is a
//! user-facing Chinese sentence: `via_backends::onboarding` resolves the
//! backend id to a `via-i18n` key. Both were recorded as deferred in
//! `docs/deviations/phase-0.md` and both landed in phase 2, in those places.

use serde::Serialize;

use crate::error::CatalogError;

/// The sentinel a user writes to run frontend-only.
///
/// External contract — `shared/backend-catalog.mjs:462-465`. Case-insensitive;
/// normalises to the empty string.
pub const BACKEND_NONE_SENTINEL: &str = "none";

/// How the gateway talks to a backend's executable.
///
/// External contract — `shared/backend-catalog.mjs`. Consumed as a discriminant
/// by CLI rendering and by desktop settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Integration {
    /// The backend speaks ACP itself.
    Native,
    /// Reached through a bridge process (OpenClaw).
    Bridge,
    /// Reached through a separate ACP adapter binary.
    Adapter,
    /// A user-supplied ACP agent named entirely by configuration.
    Generic,
}

/// Who owns the backend's model configuration.
///
/// External contract — `shared/backend-catalog.mjs`
/// (`lifecycle.configuration.mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationMode {
    /// The backend keeps its own model configuration; VIA does not write it.
    BackendOwned,
    /// VIA may write a Bailian (DashScope) configuration, else the backend owns it.
    BailianOrBackendOwned,
    /// The user configures everything.
    UserManaged,
}

/// How a backend's authentication state is inspected.
///
/// External contract — `shared/backend-catalog.mjs`; `backend-auth-status.mjs`
/// dispatches on exactly these values, so a spelling change makes a backend
/// silently report `unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProbeKind {
    /// Run the backend's own command and parse its output with
    /// [`AuthProbe::parser`].
    Command,
    /// Read Qwen Code's `~/.qwen/settings.json`.
    QwenSettings,
    /// Pi's auth check.
    PiAuthCheck,
    /// DeepSeek Harness credential file.
    DeepseekCredentials,
    /// CodeBuddy credential file.
    CodebuddyCredentials,
    /// OpenClaw's own state directory.
    OpenclawState,
}

/// How [`ProbeKind::Command`] output is read.
///
/// External contract — `shared/backend-catalog.mjs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandParser {
    /// Count credentials listed by the backend.
    CredentialCount,
    /// Qoder's `status` output.
    QoderStatus,
    /// Codex's `login status` output.
    CodexStatus,
}

/// One backend's authentication probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthProbe {
    /// Which probe to run.
    pub kind: ProbeKind,
    /// Arguments for [`ProbeKind::Command`]; empty otherwise.
    pub args: &'static [&'static str],
    /// Output parser for [`ProbeKind::Command`]; `None` otherwise.
    pub parser: Option<CommandParser>,
}

/// First-run onboarding for a backend.
///
/// Upstream's `onboarding.hint` — a Chinese user-facing sentence — is
/// deliberately absent: per `docs/fidelity.md` those live in `via-i18n`, keyed by
/// backend id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Onboarding {
    /// The command a user runs to authenticate.
    pub command: &'static str,
    /// How to check whether that already happened.
    pub probe: Option<AuthProbe>,
}

/// How a backend's executable and ACP adapter are located.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setup {
    /// The backend's own command name, when it has a fixed one.
    pub command: Option<&'static str>,
    /// Environment variable naming the command, for backends that have no fixed
    /// one (`acp`).
    pub command_environment: Option<&'static str>,
    /// Environment variables that override the executable path, in priority
    /// order. Qoder declares two.
    pub executable_environment: &'static [&'static str],
    /// How the gateway talks to it.
    pub integration: Integration,
    /// Minimum acceptable version, where one is enforced.
    pub minimum_version: Option<&'static str>,
    /// The ACP adapter command, for [`Integration::Adapter`] backends.
    pub adapter_command: Option<&'static str>,
    /// Environment variable overriding the adapter path.
    pub adapter_environment: Option<&'static str>,
    /// Environment variable overriding the adapter's runtime.
    pub adapter_runtime_environment: Option<&'static str>,
    /// Whether a missing adapter may fall back to a package-runner invocation.
    ///
    /// Upstream expresses this as the *absence* of `managedAdapterFallback` and
    /// tests it with `spec.managedAdapterFallback === false`
    /// (`shared/backend-setup.mjs:329,337`), so absent means `true`. Only
    /// DeepSeek declares `false`.
    pub managed_adapter_fallback: bool,
    /// Whether the adapter is inspected even when the backend itself is not
    /// ready (`shared/backend-setup.mjs:448`). Only DeepSeek declares `true`.
    pub inspect_adapter_independently: bool,
}

/// One installation step.
///
/// Declared but never constructed in phase 0 — see the module docs. The variants
/// mirror upstream's `{ kind: 'npm' | 'script', … }` step objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum InstallStep {
    /// A pinned npm package installed globally.
    Npm {
        /// The exact `name@version` coordinate.
        package: &'static str,
        /// Environment variable overriding that coordinate.
        package_env: Option<&'static str>,
        /// Which half of a composed backend this step installs.
        component: Option<InstallComponent>,
        /// Registry override.
        registry: Option<&'static str>,
    },
    /// A shell installer.
    Script {
        /// The command line, run verbatim.
        command: &'static str,
        /// Node `process.platform` values this step applies to.
        platforms: &'static [&'static str],
    },
}

/// Which half of a composed backend an [`InstallStep`] belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallComponent {
    /// The backend agent itself.
    Backend,
    /// Its ACP adapter.
    Adapter,
}

/// A backend's installation plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationSpec {
    /// Whether installed package versions are verified after the run. Only
    /// DeepSeek declares it.
    pub verify_installed_packages: bool,
    /// The ordered steps.
    ///
    /// **Empty in phase 0.** The pinned coordinates move to `via-backends`; see
    /// the module docs. `acp`'s genuine absence of an installer is modelled as
    /// [`Lifecycle::installation`] being `None`, which stays distinguishable from
    /// an empty step list.
    pub steps: &'static [InstallStep],
}

/// Installation and configuration ownership for a backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lifecycle {
    /// `None` means VIA never installs this backend (`acp`).
    pub installation: Option<InstallationSpec>,
    /// Who owns the model configuration.
    pub configuration: ConfigurationMode,
}

/// A backend reachable as an already-running service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalService {
    /// Environment variable carrying the credential for that service.
    pub credential_environment: &'static str,
}

/// A backend's `skills.sh` declaration.
///
/// Upstream makes this a *required* field and throws at registration when it is
/// missing (`validateBackendSkillsSpec`, `shared/backend-catalog.mjs:405-419`) so
/// that a new backend cannot silently skip the SKILL support decision. In Rust the
/// field is non-optional and this enum has no "unset" variant, which is the same
/// guarantee enforced at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillsSpec {
    /// The backend's agent id on `skills.sh`.
    Installer(&'static str),
    /// No dedicated installer; the backend reads the shared `~/.agents/skills`
    /// directory and is covered passively (upstream `{ installer: null }`).
    NoInstaller,
    /// No skills convention at all — a user-supplied ACP agent (upstream
    /// `skills: null`).
    NoConvention,
}

impl SkillsSpec {
    /// The `skills.sh` installer id, when there is one.
    pub const fn installer(&self) -> Option<&'static str> {
        match self {
            Self::Installer(id) => Some(*id),
            Self::NoInstaller | Self::NoConvention => None,
        }
    }
}

/// Which environment variables cross into a spawned backend process.
///
/// **A security boundary.** External contract —
/// `shared/backend-catalog.mjs:30-388`, identity renames applied per
/// `docs/rebrand.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentPolicy {
    /// Exact variable names that are forwarded.
    pub names: &'static [&'static str],
    /// Prefixes whose every variable is forwarded.
    pub prefixes: &'static [&'static str],
    /// Variable naming a user-supplied comma-separated allow-list, for the
    /// generic `acp` backend which has no vendor namespace of its own.
    pub explicit_list_environment: Option<&'static str>,
}

impl EnvironmentPolicy {
    /// Whether this policy forwards `name`.
    ///
    /// Matching is exact for [`Self::names`] and a literal prefix test for
    /// [`Self::prefixes`] — case-sensitive, as upstream's is. The
    /// [`Self::explicit_list_environment`] opt-in is deliberately *not* consulted
    /// here: its contents come from the user's environment at spawn time, which
    /// is `via-process`'s to read.
    pub fn allows(&self, name: &str) -> bool {
        self.names.contains(&name) || self.prefixes.iter().any(|p| name.starts_with(p))
    }
}

/// One backend agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendDefinition {
    /// The protocol id, as accepted by configuration and echoed in
    /// `/api/health`.
    pub id: &'static str,
    /// The human label, interpolated into CLI output and error messages.
    pub label: &'static str,
    /// Environment variable overriding this backend's workspace directory.
    pub workspace_environment: &'static str,
    /// `skills.sh` declaration.
    pub skills: SkillsSpec,
    /// Executable and adapter resolution.
    pub setup: Setup,
    /// Installation and configuration ownership.
    pub lifecycle: Lifecycle,
    /// First-run authentication.
    pub onboarding: Option<Onboarding>,
    /// Environment variable overriding the backend's HTTP base URL.
    pub base_url_environment: Option<&'static str>,
    /// The base URL used when nothing overrides it.
    pub default_base_url: Option<&'static str>,
    /// Whether the backend may be an already-running external service.
    pub supports_external_service: bool,
    /// Credential for that external service.
    pub external_service: Option<ExternalService>,
    /// Whether the gateway may switch this backend into full-permission mode.
    pub supports_full_permission: bool,
    /// Whether the backend is *always* effectively full-permission, whatever the
    /// user configures. A safety disclosure, not a setting — see
    /// [`effective_backend_permission_mode`].
    pub always_full_permission: bool,
    /// The credential-namespace boundary.
    pub environment: EnvironmentPolicy,
}

/// A [`Setup`] with every optional field cleared, for terser table entries.
const SETUP: Setup = Setup {
    command: None,
    command_environment: None,
    executable_environment: &[],
    integration: Integration::Native,
    minimum_version: None,
    adapter_command: None,
    adapter_environment: None,
    adapter_runtime_environment: None,
    managed_adapter_fallback: true,
    inspect_adapter_independently: false,
};

/// A backend VIA installs, with no step list yet. See the module docs.
const INSTALLS: Option<InstallationSpec> = Some(InstallationSpec {
    verify_installed_packages: false,
    steps: &[],
});

/// The twelve backends, in catalog order.
///
/// External contract — `shared/backend-catalog.mjs:6-390`. The *order* is
/// contract: `backendNames().join('、')` is interpolated into several error
/// messages, and `--backend` documents the ids in this order.
static BACKEND_DEFINITIONS: [BackendDefinition; 12] = [
    BackendDefinition {
        id: "opencode",
        label: "OpenCode",
        workspace_environment: "OPENCODE_WORKSPACE",
        skills: SkillsSpec::Installer("opencode"),
        setup: Setup {
            command: Some("opencode"),
            executable_environment: &["OPENCODE_BIN"],
            integration: Integration::Native,
            minimum_version: Some("1.18.0"),
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BailianOrBackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "opencode auth login",
            probe: Some(AuthProbe {
                kind: ProbeKind::Command,
                args: &["auth", "list"],
                parser: Some(CommandParser::CredentialCount),
            }),
        }),
        base_url_environment: Some("OPENCODE_BASE_URL"),
        default_base_url: Some("http://127.0.0.1:4096"),
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &[
                "DASHSCOPE_API_KEY",
                // Upstream QWEN_AUDIO_AGENT_OPENCODE_ISOLATE_USER_CONFIG.
                "VIA_OPENCODE_ISOLATE_USER_CONFIG",
                // Upstream QWEN_AUDIO_AGENT_OPENCODE_XDG_CONFIG_HOME.
                "VIA_OPENCODE_XDG_CONFIG_HOME",
            ],
            prefixes: &["OPENCODE_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "openclaw",
        label: "OpenClaw",
        // Upstream QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE.
        workspace_environment: "VIA_OPENCLAW_WORKSPACE",
        skills: SkillsSpec::Installer("openclaw"),
        setup: Setup {
            command: Some("openclaw"),
            executable_environment: &["OPENCLAW_BIN"],
            integration: Integration::Bridge,
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BailianOrBackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "openclaw onboard",
            probe: Some(AuthProbe {
                kind: ProbeKind::OpenclawState,
                args: &[],
                parser: None,
            }),
        }),
        base_url_environment: Some("OPENCLAW_BASE_URL"),
        default_base_url: Some("http://127.0.0.1:18789"),
        supports_external_service: true,
        external_service: Some(ExternalService {
            credential_environment: "OPENCLAW_GATEWAY_TOKEN",
        }),
        // OpenClaw's full permission needs exec approvals, elevated and host
        // configured separately; the gateway's single switch cannot enable it
        // safely.
        supports_full_permission: false,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &[
                "DASHSCOPE_API_KEY",
                "AGENT_API_KEY",
                // Upstream QWAUDIO_CONFIG_DIR.
                "VIA_CONFIG_DIR",
                // Upstream QWEN_AUDIO_AGENT_OPENCLAW_*.
                "VIA_OPENCLAW_MODEL",
                "VIA_OPENCLAW_MODEL_ID",
                "VIA_OPENCLAW_STATE_DIR",
                "VIA_OPENCLAW_WORKSPACE",
            ],
            prefixes: &["OPENCLAW_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "qoder",
        label: "Qoder",
        workspace_environment: "QODER_WORKSPACE",
        skills: SkillsSpec::Installer("qoder"),
        setup: Setup {
            command: Some("qodercli"),
            // Two names, in priority order.
            executable_environment: &["QODERCLI_PATH", "QODER_CLI_PATH"],
            integration: Integration::Native,
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "qodercli login",
            probe: Some(AuthProbe {
                kind: ProbeKind::Command,
                args: &["status"],
                parser: Some(CommandParser::QoderStatus),
            }),
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &[],
            prefixes: &["QODER_", "QODERCLI_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        // The third-party Qwen Code CLI. Id, label, command and QWEN_CODE_
        // namespace are all KEEP — they name Alibaba's product, not VIA.
        id: "qwen",
        label: "Qwen Code",
        workspace_environment: "QWEN_CODE_WORKSPACE",
        skills: SkillsSpec::Installer("qwen-code"),
        setup: Setup {
            command: Some("qwen"),
            executable_environment: &["QWEN_CODE_BIN"],
            integration: Integration::Native,
            minimum_version: Some("0.21.6"),
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "qwen",
            probe: Some(AuthProbe {
                kind: ProbeKind::QwenSettings,
                args: &[],
                parser: None,
            }),
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &["DASHSCOPE_API_KEY", "OPENAI_API_KEY", "OPENAI_BASE_URL"],
            prefixes: &["QWEN_CODE_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "kimi",
        label: "Kimi Code",
        workspace_environment: "KIMI_WORKSPACE",
        // Kimi reads the shared ~/.agents/skills directory; skills.sh maps the
        // same id.
        skills: SkillsSpec::Installer("kimi-code-cli"),
        setup: Setup {
            command: Some("kimi"),
            executable_environment: &["KIMI_CODE_BIN"],
            integration: Integration::Native,
            minimum_version: Some("0.31.0"),
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "kimi login",
            probe: None,
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &[],
            prefixes: &["KIMI_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "hermes",
        label: "Hermes",
        workspace_environment: "HERMES_WORKSPACE",
        skills: SkillsSpec::Installer("hermes-agent"),
        setup: Setup {
            command: Some("hermes"),
            executable_environment: &["HERMES_BIN"],
            integration: Integration::Native,
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "hermes setup --portal",
            probe: None,
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &[],
            prefixes: &["HERMES_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "codebuddy",
        label: "CodeBuddy",
        workspace_environment: "CODEBUDDY_WORKSPACE",
        skills: SkillsSpec::Installer("codebuddy"),
        setup: Setup {
            command: Some("codebuddy"),
            executable_environment: &["CODEBUDDY_BIN"],
            integration: Integration::Native,
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "codebuddy",
            probe: Some(AuthProbe {
                kind: ProbeKind::CodebuddyCredentials,
                args: &[],
                parser: None,
            }),
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &[],
            prefixes: &["CODEBUDDY_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "codex",
        label: "Codex",
        workspace_environment: "CODEX_WORKSPACE",
        skills: SkillsSpec::Installer("codex"),
        setup: Setup {
            command: Some("codex"),
            executable_environment: &["CODEX_PATH"],
            integration: Integration::Adapter,
            adapter_command: Some("codex-acp"),
            adapter_environment: Some("CODEX_ACP_BIN"),
            adapter_runtime_environment: Some("CODEX_ACP_RUNTIME"),
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "codex login",
            probe: Some(AuthProbe {
                kind: ProbeKind::Command,
                args: &["login", "status"],
                parser: Some(CommandParser::CodexStatus),
            }),
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &["DASHSCOPE_API_KEY", "OPENAI_API_KEY", "OPENAI_BASE_URL"],
            prefixes: &["CODEX_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "claude",
        label: "Claude Code",
        workspace_environment: "CLAUDE_WORKSPACE",
        skills: SkillsSpec::Installer("claude-code"),
        setup: Setup {
            command: Some("claude"),
            executable_environment: &["CLAUDE_CODE_EXECUTABLE"],
            integration: Integration::Adapter,
            adapter_command: Some("claude-code-acp"),
            adapter_environment: Some("CLAUDE_CODE_ACP_BIN"),
            adapter_runtime_environment: Some("CLAUDE_CODE_ACP_RUNTIME"),
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "claude",
            probe: None,
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &["ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL", "CLAUDE_API_KEY"],
            prefixes: &["CLAUDE_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "deepseek",
        label: "DeepSeek",
        workspace_environment: "DEEPSEEK_HARNESS_WORKSPACE",
        // skills.sh has no dsh-specific installer yet; dsh reads the shared
        // ~/.agents/skills directory and benefits passively.
        skills: SkillsSpec::NoInstaller,
        setup: Setup {
            command: Some("dsh"),
            executable_environment: &["DEEPSEEK_HARNESS_BIN"],
            integration: Integration::Native,
            adapter_command: Some("dsh-acp-demo"),
            adapter_environment: Some("DEEPSEEK_HARNESS_ACP_BIN"),
            adapter_runtime_environment: Some("DEEPSEEK_HARNESS_ACP_RUNTIME"),
            managed_adapter_fallback: false,
            inspect_adapter_independently: true,
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: Some(InstallationSpec {
                verify_installed_packages: true,
                steps: &[],
            }),
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "dsh web",
            probe: Some(AuthProbe {
                kind: ProbeKind::DeepseekCredentials,
                args: &[],
                parser: None,
            }),
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &["DEEPSEEK_API_KEY"],
            prefixes: &["DEEPSEEK_", "DSH_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "pi",
        label: "Pi",
        workspace_environment: "PI_WORKSPACE",
        skills: SkillsSpec::Installer("pi"),
        setup: Setup {
            command: Some("pi"),
            executable_environment: &["PI_BIN"],
            integration: Integration::Adapter,
            minimum_version: Some("0.80.4"),
            adapter_command: Some("pi-acp"),
            adapter_environment: Some("PI_ACP_BIN"),
            adapter_runtime_environment: Some("PI_ACP_RUNTIME"),
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: INSTALLS,
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "pi",
            probe: Some(AuthProbe {
                kind: ProbeKind::PiAuthCheck,
                args: &[],
                parser: None,
            }),
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: true,
        // Pi has no approval gate in any configuration; the declaration is a
        // safety disclosure surfaced to the user, not a setting.
        always_full_permission: true,
        environment: EnvironmentPolicy {
            names: &[
                "ANTHROPIC_API_KEY",
                "ANTHROPIC_BASE_URL",
                "OPENAI_API_KEY",
                "OPENAI_BASE_URL",
                "GEMINI_API_KEY",
                "GOOGLE_API_KEY",
            ],
            prefixes: &["PI_"],
            explicit_list_environment: None,
        },
    },
    BackendDefinition {
        id: "acp",
        label: "ACP Agent",
        workspace_environment: "ACP_WORKSPACE",
        // A user-supplied agent: its skills directory convention is unknown, and
        // upstream declares that explicitly rather than leaving it unset.
        skills: SkillsSpec::NoConvention,
        setup: Setup {
            command_environment: Some("ACP_COMMAND"),
            integration: Integration::Generic,
            ..SETUP
        },
        lifecycle: Lifecycle {
            installation: None,
            configuration: ConfigurationMode::UserManaged,
        },
        onboarding: None,
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: false,
        always_full_permission: false,
        environment: EnvironmentPolicy {
            names: &[],
            prefixes: &["ACP_"],
            // Upstream QWEN_AUDIO_AGENT_ACP_FORWARD_ENV.
            explicit_list_environment: Some("VIA_ACP_FORWARD_ENV"),
        },
    },
];

/// The backend catalog, in catalog order.
pub fn backend_definitions() -> &'static [BackendDefinition] {
    &BACKEND_DEFINITIONS
}

/// Backend ids in catalog order.
///
/// Upstream `backendNames()` (`shared/backend-catalog.mjs:435-437`); the joined
/// form appears inside several operator-facing error messages.
pub fn backend_names() -> Vec<&'static str> {
    BACKEND_DEFINITIONS.iter().map(|d| d.id).collect()
}

/// Normalise a backend protocol value.
///
/// External contract — `shared/backend-catalog.mjs:462-465`: trim, lowercase, and
/// map the `none` sentinel to the empty string (frontend-only). Any other value
/// is returned as-is; validity is [`backend_definition`]'s to decide.
pub fn normalize_backend_protocol(value: &str) -> String {
    let protocol = value.trim().to_lowercase();
    if protocol == BACKEND_NONE_SENTINEL {
        String::new()
    } else {
        protocol
    }
}

/// Look up a backend by protocol id, normalising first.
///
/// Returns `None` for the empty string and for the `none` sentinel, matching
/// upstream's `backendDefinition()` (`shared/backend-catalog.mjs:392-394`).
pub fn backend_definition(protocol: &str) -> Option<&'static BackendDefinition> {
    let id = normalize_backend_protocol(protocol);
    if id.is_empty() {
        return None;
    }
    BACKEND_DEFINITIONS.iter().find(|d| d.id == id)
}

/// The distinct `skills.sh` installer agent ids, in catalog order.
///
/// Upstream `skillsInstallerAgents()` (`shared/backend-catalog.mjs:429-433`),
/// which builds the explicit `-a` name list rather than trusting `skills.sh`'s
/// own host detection.
pub fn skills_installer_agents() -> Vec<&'static str> {
    let mut agents: Vec<&'static str> = Vec::new();
    for definition in &BACKEND_DEFINITIONS {
        if let Some(installer) = definition.skills.installer()
            && !agents.contains(&installer)
        {
            agents.push(installer);
        }
    }
    agents
}

/// Whether `value` is a well-formed `skills.sh` agent id.
///
/// External contract — `SKILLS_INSTALLER_PATTERN`
/// (`shared/backend-catalog.mjs:398`), `/^[a-z0-9]+(?:-[a-z0-9]+)*$/`. The
/// restricted character set exists to stop an installer declaration from
/// smuggling extra flags into a command line, so it is reproduced exactly rather
/// than loosened.
pub fn is_valid_skills_installer(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Whether the gateway launches the backend or connects to a running one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Ownership {
    /// The gateway starts and supervises the process.
    Owned,
    /// The backend is already running; the gateway connects to it.
    External,
}

impl Ownership {
    /// The wire string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Owned => "owned",
            Self::External => "external",
        }
    }
}

/// Decide whether a backend is gateway-owned or an external service.
///
/// External contract — `shared/backend-catalog.mjs:443-460`. An explicit request
/// wins if the backend supports it; otherwise a backend that both supports
/// external service *and* has a base URL configured defaults to `external`.
///
/// # Errors
///
/// - [`CatalogError::UnsupportedBackend`] — no such backend.
/// - [`CatalogError::UnsupportedBackendOwnership`] — a requested value other than
///   `owned` or `external`.
/// - [`CatalogError::ExternalServiceUnsupported`] — `external` requested for a
///   backend the gateway must launch itself.
pub fn resolve_backend_ownership(
    protocol: &str,
    base_url_configured: bool,
    requested_ownership: &str,
) -> Result<Ownership, CatalogError> {
    let definition =
        backend_definition(protocol).ok_or_else(|| CatalogError::UnsupportedBackend {
            protocol: normalize_backend_protocol(protocol),
        })?;
    let requested = requested_ownership.trim().to_lowercase();
    if !requested.is_empty() {
        let ownership = match requested.as_str() {
            "owned" => Ownership::Owned,
            "external" => Ownership::External,
            _ => return Err(CatalogError::UnsupportedBackendOwnership { requested }),
        };
        if ownership == Ownership::External && !definition.supports_external_service {
            return Err(CatalogError::ExternalServiceUnsupported {
                label: definition.label,
            });
        }
        return Ok(ownership);
    }
    Ok(
        if definition.supports_external_service && base_url_configured {
            Ownership::External
        } else {
            Ownership::Owned
        },
    )
}

/// The permission mode that is actually in effect for a backend.
///
/// External contract — `shared/backend-catalog.mjs:471-476`, test-locked at
/// `test/backend-catalog.test.mjs:16-28`. A backend that declares
/// [`BackendDefinition::always_full_permission`] reports `full` whatever the user
/// configured; everything else is trimmed, lowercased, and defaults to `native`.
///
/// Upstream performs no validity check here ("本函数不做合法性校验") and neither
/// does this: an unrecognised mode is passed through for the caller to reject in
/// its own context, which is why the return type is a `String` rather than an
/// enum.
pub fn effective_backend_permission_mode(protocol: &str, mode: &str) -> String {
    let normalized = mode.trim().to_lowercase();
    let normalized = if normalized.is_empty() {
        "native".to_owned()
    } else {
        normalized
    };
    match backend_definition(protocol) {
        Some(definition) if definition.always_full_permission => "full".to_owned(),
        _ => normalized,
    }
}
