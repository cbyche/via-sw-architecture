//! One-step backend installation: the pinned specs, and the executor.
//!
//! Ported from `shared/backend-install.mjs`, with the step table itself from
//! `shared/backend-catalog.mjs`. `via-catalog` carries the *shape* of an
//! installation spec but deliberately not the coordinates — those name npm
//! packages belonging to twelve third-party products, and only this crate is
//! allowed to name a backend (`docs/architecture.md` §9).
//!
//! # The version policy, restated because it is load-bearing
//!
//! Upstream's own comment: *"npm packages are pinned without exception (the same
//! discipline as the managed launcher scripts), overridable through each
//! `packageEnv`. The similarly-named `kimi-code` / `codebuddy` / `hermes-agent`
//! packages on npm are third-party or placeholder packages and must never be
//! written into a spec."* Every coordinate below is
//! `default-value/backend pinned install packages and scripts`, asserted
//! against `docs/reference/contracts.json` by `tests/contracts.rs`.
//!
//! **DeepSeek's step order is a contract, not a convenience.** Its ACP
//! executable package is last so that a partial failure leaves setup visibly
//! incomplete and a retry fills the whole composition rather than skipping it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use via_catalog::backend::InstallComponent;
use via_catalog::backend_definition;
use via_core::EnvMap;
use via_core::search_path::merge_search_path;
use via_i18n::{Locale, format, keys, t};

use crate::detect::ExecutableFinder;
use crate::onboarding::{AuthenticationSupport, backend_authentication_support};
use crate::platform::HostPlatform;

/// How long one installation step may run.
///
/// **External contract** — `shared/backend-install.mjs:34`
/// (`DEFAULT_STEP_TIMEOUT_MS = 10 * 60 * 1000`).
pub const DEFAULT_STEP_TIMEOUT: Duration = Duration::from_millis(600_000);

/// How much of a step's output is retained for the failure message.
///
/// **External contract** — `shared/backend-install.mjs:35`
/// (`MAX_INSTALL_OUTPUT_CHARS = 64 * 1024`). A tail, not a head: the useful
/// part of a failed `npm install` is at the end.
pub const MAX_INSTALL_OUTPUT_CHARS: usize = 65_536;

/// The variable that makes `npm` non-interactive.
///
/// **External contract** — `shared/backend-install.mjs:208`
/// (`npm_config_yes: 'true'`). It rides the `npm_config_` prefix already on the
/// child environment allow-list.
pub const NPM_CONFIG_YES: (&str, &str) = ("npm_config_yes", "true");

/// The shell a `script` step runs under on POSIX.
///
/// **External contract** — `shared/backend-install.mjs:581`.
pub const POSIX_SHELL: &str = "/bin/sh";

/// The shell a `script` step runs under on Windows, with its flags.
///
/// **External contract** — `shared/backend-install.mjs:566`.
pub const WINDOWS_SHELL: &str = "powershell.exe";
/// `powershell.exe`'s flags, in upstream's order.
pub const WINDOWS_SHELL_FLAGS: [&str; 3] = ["-ExecutionPolicy", "Bypass", "-Command"];

/// Whether a step installs an npm package or runs a shell command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    /// `npm install -g [--registry=R] <package>`.
    Npm,
    /// A vendor's own installer, run through a shell.
    Script,
}

/// A step's display label.
///
/// Upstream writes four literals. Two are translated product-neutral phrases
/// and live in `via-i18n`; the other two are proper nouns upstream leaves
/// untranslated in every locale, so they stay literals here for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StepLabel {
    /// `ACP 适配器` — `backend-catalog.mjs:215,248,345`.
    AcpAdapter,
    /// `运行组件` — `backend-catalog.mjs:307`.
    RuntimeComponents,
    /// `DeepSeek CLI` — `backend-catalog.mjs:289`, a product name.
    DeepSeekCli,
    /// `ACP Runtime` — `backend-catalog.mjs:307`, untranslated upstream.
    AcpRuntime,
}

impl StepLabel {
    /// The rendered label.
    #[must_use]
    pub fn text(self, locale: Locale) -> String {
        match self {
            Self::AcpAdapter => t(locale, keys::BACKEND_INSTALL_STEP_LABEL_ACP_ADAPTER).to_owned(),
            Self::RuntimeComponents => {
                t(locale, keys::BACKEND_INSTALL_STEP_LABEL_RUNTIME_COMPONENTS).to_owned()
            }
            Self::DeepSeekCli => "DeepSeek CLI".to_owned(),
            Self::AcpRuntime => "ACP Runtime".to_owned(),
        }
    }
}

/// One installation step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallStepSpec {
    /// npm or script.
    pub kind: StepKind,
    /// Display label, when the step has one.
    pub label: Option<StepLabel>,
    /// Which half of a composed backend this installs.
    pub component: Option<InstallComponent>,
    /// The pinned `name@version` coordinate. Empty for a script step.
    pub package: &'static str,
    /// The variable that overrides that coordinate.
    pub package_env: Option<&'static str>,
    /// A registry override.
    pub registry: Option<&'static str>,
    /// The shell command. Empty for an npm step.
    pub command: &'static str,
    /// The platforms this step applies to. Empty means every platform, which
    /// is upstream's absent `platforms` key.
    pub platforms: &'static [HostPlatform],
}

/// A blank step, so the table below states only what differs.
const STEP: InstallStepSpec = InstallStepSpec {
    kind: StepKind::Npm,
    label: None,
    component: None,
    package: "",
    package_env: None,
    registry: None,
    command: "",
    platforms: &[],
};

/// Hermes's POSIX installer.
///
/// **External contract** — `shared/backend-catalog.mjs:1`.
pub const HERMES_INSTALL_COMMAND: &str =
    "curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash";

/// Hermes's Windows installer.
///
/// **External contract** — `shared/backend-catalog.mjs:164`.
pub const HERMES_INSTALL_COMMAND_WINDOWS: &str =
    "iex (irm https://hermes-agent.nousresearch.com/install.ps1)";

/// The registry every DeepSeek component is pulled from.
///
/// **External contract** — `shared/backend-catalog.mjs:292,310`. Explicit
/// because a Developer Preview build must not be resolved from a mirror that
/// has not synced it.
pub const DEEPSEEK_REGISTRY: &str = "https://registry.npmjs.org/";

const OPENCODE_STEPS: &[InstallStepSpec] = &[InstallStepSpec {
    package: "opencode-ai@1.18.5",
    package_env: Some("OPENCODE_PACKAGE"),
    ..STEP
}];

const OPENCLAW_STEPS: &[InstallStepSpec] = &[InstallStepSpec {
    package: "openclaw@2026.6.33",
    package_env: Some("OPENCLAW_PACKAGE"),
    ..STEP
}];

const QODER_STEPS: &[InstallStepSpec] = &[InstallStepSpec {
    package: "@qoder-ai/qodercli@1.1.13",
    package_env: Some("QODERCLI_PACKAGE"),
    ..STEP
}];

const QWEN_STEPS: &[InstallStepSpec] = &[InstallStepSpec {
    package: "@qwen-code/qwen-code@0.21.6",
    package_env: Some("QWEN_CODE_PACKAGE"),
    ..STEP
}];

const KIMI_STEPS: &[InstallStepSpec] = &[InstallStepSpec {
    package: "@moonshot-ai/kimi-code@0.32.0",
    package_env: Some("KIMI_CODE_PACKAGE"),
    ..STEP
}];

const HERMES_STEPS: &[InstallStepSpec] = &[
    InstallStepSpec {
        kind: StepKind::Script,
        command: HERMES_INSTALL_COMMAND,
        platforms: &[HostPlatform::Darwin, HostPlatform::Linux],
        ..STEP
    },
    InstallStepSpec {
        kind: StepKind::Script,
        command: HERMES_INSTALL_COMMAND_WINDOWS,
        platforms: &[HostPlatform::Windows],
        ..STEP
    },
];

const CODEBUDDY_STEPS: &[InstallStepSpec] = &[InstallStepSpec {
    package: "@tencent-ai/codebuddy-code@2.132.0",
    package_env: Some("CODEBUDDY_PACKAGE"),
    ..STEP
}];

const CODEX_STEPS: &[InstallStepSpec] = &[
    InstallStepSpec {
        package: "@openai/codex@0.146.0",
        package_env: Some("CODEX_PACKAGE"),
        ..STEP
    },
    InstallStepSpec {
        label: Some(StepLabel::AcpAdapter),
        component: Some(InstallComponent::Adapter),
        package: "@agentclientprotocol/codex-acp@1.1.7",
        package_env: Some("CODEX_ACP_PACKAGE"),
        ..STEP
    },
];

const CLAUDE_STEPS: &[InstallStepSpec] = &[
    InstallStepSpec {
        package: "@anthropic-ai/claude-code@2.1.221",
        package_env: Some("CLAUDE_CODE_PACKAGE"),
        ..STEP
    },
    InstallStepSpec {
        label: Some(StepLabel::AcpAdapter),
        component: Some(InstallComponent::Adapter),
        package: "@zed-industries/claude-code-acp@0.16.2",
        package_env: Some("CLAUDE_CODE_ACP_PACKAGE"),
        ..STEP
    },
];

/// A DeepSeek component step, which differ only by package and label.
const fn deepseek_component(package: &'static str, label: StepLabel) -> InstallStepSpec {
    InstallStepSpec {
        kind: StepKind::Npm,
        label: Some(label),
        component: Some(InstallComponent::Adapter),
        package,
        package_env: None,
        registry: Some(DEEPSEEK_REGISTRY),
        command: "",
        platforms: &[],
    }
}

const DEEPSEEK_STEPS: &[InstallStepSpec] = &[
    InstallStepSpec {
        label: Some(StepLabel::DeepSeekCli),
        component: Some(InstallComponent::Backend),
        package: "@deepseek-ai/dsh@0.1.0-rc.6",
        registry: Some(DEEPSEEK_REGISTRY),
        ..STEP
    },
    deepseek_component(
        "@deepseek-ai/dsh-llm-deepseek@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    deepseek_component(
        "@deepseek-ai/dsh-sandbox-local@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    deepseek_component(
        "@deepseek-ai/dsh-subprocess-local@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    deepseek_component(
        "@deepseek-ai/dsh-bash-sandbox@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    deepseek_component(
        "@deepseek-ai/dsh-token-meter@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    deepseek_component(
        "@deepseek-ai/dsh-compaction-basic@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    deepseek_component(
        "@deepseek-ai/dsh-fs-sandbox@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    deepseek_component(
        "@deepseek-ai/dsh-fs-observation-policy@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    deepseek_component(
        "@deepseek-ai/dsh-tool-fs@0.1.0-rc.6",
        StepLabel::RuntimeComponents,
    ),
    // Last on purpose — see the module docs.
    deepseek_component(
        "@deepseek-ai/dsh-acp-demo@0.1.0-rc.6",
        StepLabel::AcpRuntime,
    ),
];

const PI_STEPS: &[InstallStepSpec] = &[
    InstallStepSpec {
        package: "@earendil-works/pi-coding-agent@0.84.1",
        package_env: Some("PI_PACKAGE"),
        ..STEP
    },
    InstallStepSpec {
        label: Some(StepLabel::AcpAdapter),
        component: Some(InstallComponent::Adapter),
        package: "pi-acp@0.0.33",
        package_env: Some("PI_ACP_PACKAGE"),
        ..STEP
    },
];

/// A backend's steps and whether installed versions are verified afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallSteps {
    /// `verifyInstalledPackages` — only DeepSeek declares it.
    pub verify_installed_packages: bool,
    /// The steps that apply on this platform, in order.
    pub steps: Vec<&'static InstallStepSpec>,
}

/// The unfiltered step table for a catalogued id.
///
/// `None` for `acp`, whose catalog entry has `installation: null` — a genuine
/// absence, distinct from an empty step list.
#[must_use]
pub fn install_spec(id: &str) -> Option<(&'static [InstallStepSpec], bool)> {
    Some(match id {
        "opencode" => (OPENCODE_STEPS, false),
        "openclaw" => (OPENCLAW_STEPS, false),
        "qoder" => (QODER_STEPS, false),
        "qwen" => (QWEN_STEPS, false),
        "kimi" => (KIMI_STEPS, false),
        "hermes" => (HERMES_STEPS, false),
        "codebuddy" => (CODEBUDDY_STEPS, false),
        "codex" => (CODEX_STEPS, false),
        "claude" => (CLAUDE_STEPS, false),
        "deepseek" => (DEEPSEEK_STEPS, true),
        "pi" => (PI_STEPS, false),
        _ => return None,
    })
}

/// `specSteps(id, platform)` — `shared/backend-install.mjs:46-57`.
#[must_use]
pub fn install_steps(id: &str, platform: HostPlatform) -> InstallSteps {
    let (steps, verify) = install_spec(id).unwrap_or((&[], false));
    InstallSteps {
        verify_installed_packages: verify,
        steps: steps
            .iter()
            .filter(|step| step.platforms.is_empty() || step.platforms.contains(&platform))
            .collect(),
    }
}

/// `stepPackage(step, env)` — `shared/backend-install.mjs:59-61`.
#[must_use]
pub fn step_package(step: &InstallStepSpec, env: &EnvMap) -> String {
    step.package_env
        .map(|name| env.get_trimmed(name))
        .filter(|value| !value.is_empty())
        .unwrap_or(step.package)
        .to_owned()
}

/// `stepDisplay(step, env)` — `shared/backend-install.mjs:63-69`.
///
/// The exact command line a user is shown before it runs, and the one echoed
/// back in the timeout and failure messages.
#[must_use]
pub fn step_display(step: &InstallStepSpec, env: &EnvMap) -> String {
    if step.kind == StepKind::Script {
        return step.command.to_owned();
    }
    let registry = step
        .registry
        .map(|value| std::format!(" --registry={value}"))
        .unwrap_or_default();
    std::format!("npm install -g{registry} {}", step_package(step, env))
}

/// `npmStepArgs(step, env)` — `shared/backend-install.mjs:71-79`.
#[must_use]
pub fn npm_step_args(step: &InstallStepSpec, env: &EnvMap) -> Vec<String> {
    let mut args = vec!["install".to_owned(), "-g".to_owned()];
    if let Some(registry) = step.registry {
        args.push(std::format!("--registry={registry}"));
    }
    args.push(step_package(step, env));
    args
}

/// `stepTitle(step, index)` — `shared/backend-install.mjs:81-83`.
///
/// One-based, as upstream's `index + 1` is.
#[must_use]
pub fn step_title(step: &InstallStepSpec, index: usize, locale: Locale) -> String {
    let position = (index + 1).to_string();
    match step.label {
        None => format(locale, keys::INSTALL_STEP_INDEX, &[("index", &position)]),
        Some(label) => format(
            locale,
            keys::INSTALL_STEP_INDEX_LABELLED,
            &[("index", &position), ("label", &label.text(locale))],
        ),
    }
}

/// `packageIdentity(spec)` — `shared/backend-setup.mjs:157-165`.
///
/// Splits on the **last** `@` so scoped names survive: `@scope/pkg@1.2.3`
/// becomes `("@scope/pkg", "1.2.3")`, while a bare `@scope/pkg` has no version.
#[must_use]
pub fn package_identity(spec: &str) -> (String, String) {
    let value = spec.trim();
    match value.rfind('@') {
        Some(index) if index > 0 => (value[..index].to_owned(), value[index + 1..].to_owned()),
        _ => (value.to_owned(), String::new()),
    }
}

/// One rendered step, as a settings UI shows it before anything runs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallStepSummary {
    /// npm or script.
    pub kind: StepKind,
    /// `步骤 2（ACP 适配器）`.
    pub title: String,
    /// The command line.
    pub display: String,
}

/// Whether a backend can be installed in one step here, and what that would run.
///
/// **External contract** — `installSupport(id, { env, platform })`,
/// `shared/backend-install.mjs:114-148`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallSupport {
    /// Whether one-step installation is offered.
    pub supported: bool,
    /// Why not, when it is not.
    pub reason: Option<String>,
    /// Whether any step runs a vendor script and therefore needs a human `y`.
    pub requires_confirmation: bool,
    /// The backend's own sign-in, so a caller can render both at once.
    pub authentication: Option<AuthenticationSupport>,
    /// The steps.
    pub steps: Vec<InstallStepSummary>,
}

/// `installSupport(id, { env, platform })`.
#[must_use]
pub fn install_support(
    id: &str,
    env: &EnvMap,
    platform: HostPlatform,
    locale: Locale,
) -> InstallSupport {
    let unsupported = |reason: String| InstallSupport {
        supported: false,
        reason: Some(reason),
        requires_confirmation: false,
        authentication: None,
        steps: Vec::new(),
    };
    let Some(definition) = backend_definition(id) else {
        return unsupported(format(
            locale,
            keys::INSTALL_UNSUPPORTED_BACKEND,
            &[("id", id.trim())],
        ));
    };
    if definition.lifecycle.installation.is_none() {
        return unsupported(t(locale, keys::INSTALL_GENERIC_ACP_MANUAL).to_owned());
    }
    let plan = install_steps(definition.id, platform);
    if plan.steps.is_empty() {
        return unsupported(t(locale, keys::INSTALL_PLATFORM_UNSUPPORTED).to_owned());
    }
    InstallSupport {
        supported: true,
        reason: None,
        requires_confirmation: plan.steps.iter().any(|step| step.kind == StepKind::Script),
        authentication: Some(backend_authentication_support(
            definition.id,
            env,
            Some(platform),
            locale,
        )),
        steps: plan
            .steps
            .iter()
            .enumerate()
            .map(|(index, step)| InstallStepSummary {
                kind: step.kind,
                title: step_title(step, index, locale),
                display: step_display(step, env),
            })
            .collect(),
    }
}

/// The closed set of installer failures.
///
/// **External contract** — `error-code/installBackend error codes (closed set)`
/// and `error-code/backend installer error codes`. CLI and desktop branch on
/// these, so the set is closed and the spellings are exact.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum InstallError {
    /// No one-step installation for this backend on this platform.
    #[error("{reason}")]
    Unsupported {
        /// Rendered explanation.
        reason: String,
    },
    /// `npm` is not on `PATH`.
    #[error("npm was not found")]
    NpmMissing,
    /// A `script` step was shown and the human said no.
    #[error("installation declined")]
    Declined,
    /// The caller cancelled mid-run.
    #[error("installation cancelled")]
    Cancelled,
    /// A step exceeded [`DEFAULT_STEP_TIMEOUT`].
    #[error("installation command timed out: {display}")]
    StepTimeout {
        /// The command line that timed out.
        display: String,
    },
    /// A step exited non-zero.
    #[error("installation command failed ({exit_code}): {display}")]
    StepFailed {
        /// The command line that failed.
        display: String,
        /// Its exit code, or `-1` when it never produced one.
        exit_code: i32,
        /// The spawn error, or the tail of the step's output.
        cause: String,
    },
    /// Every step succeeded and the backend still does not check out.
    #[error("{detail}")]
    VerifyFailed {
        /// The first issue from the post-install inspection.
        detail: String,
    },
}

impl InstallError {
    /// The catalogued code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Unsupported { .. } => "UNSUPPORTED",
            Self::NpmMissing => "NPM_MISSING",
            Self::Declined => "DECLINED",
            Self::Cancelled => "CANCELLED",
            Self::StepTimeout { .. } => "STEP_TIMEOUT",
            Self::StepFailed { .. } => "STEP_FAILED",
            Self::VerifyFailed { .. } => "VERIFY_FAILED",
        }
    }

    /// The user-facing sentence.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::Unsupported { reason } | Self::VerifyFailed { detail: reason } => reason.clone(),
            Self::NpmMissing => t(locale, keys::INSTALL_NPM_MISSING).to_owned(),
            Self::Declined => t(locale, keys::INSTALL_CANCELLED).to_owned(),
            Self::Cancelled => t(locale, keys::INSTALL_ABORTED).to_owned(),
            Self::StepTimeout { display } => format(
                locale,
                keys::INSTALL_COMMAND_TIMEOUT,
                &[("command", display)],
            ),
            Self::StepFailed {
                display, exit_code, ..
            } => format(
                locale,
                keys::INSTALL_COMMAND_FAILED,
                &[("code", &exit_code.to_string()), ("command", display)],
            ),
        }
    }
}

/// What phase of a step a progress event reports.
///
/// **External contract** — `default-value/install execution parameters and
/// progress phases`: `'skip' | 'start' | 'output' | 'done'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressPhase {
    /// The component was already present.
    Skip,
    /// The step is starting.
    Start,
    /// A chunk of the step's output.
    Output,
    /// The step finished successfully.
    Done,
}

/// Which stream a chunk came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

impl OutputStream {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

/// One progress event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    /// Zero-based step index, as upstream's `step` is.
    pub step: usize,
    /// The phase.
    pub phase: ProgressPhase,
    /// The step title, on `skip`, `start` and `done`.
    pub title: Option<String>,
    /// The command line, on `skip` and `start`.
    pub display: Option<String>,
    /// The stream, on `output`.
    pub stream: Option<OutputStream>,
    /// The chunk, on `output`.
    pub chunk: Option<String>,
}

/// Somewhere to send progress.
pub trait InstallObserver: Send + Sync {
    /// Report one event. Must not block.
    fn progress(&self, event: &ProgressEvent);
}

/// An observer that discards everything.
#[derive(Debug, Clone, Copy, Default)]
pub struct SilentObserver;

impl InstallObserver for SilentObserver {
    fn progress(&self, _event: &ProgressEvent) {}
}

/// Confirming a `script` step with a human.
#[async_trait]
pub trait StepConfirmer: Send + Sync {
    /// Whether to run `command`. `false` produces [`InstallError::Declined`].
    async fn confirm(&self, index: usize, display: &str, command: &str) -> bool;
}

/// The default: never run a vendor script.
///
/// **External contract** — `shared/backend-install.mjs:441`
/// (`confirmStep = async () => false`). A caller that does not wire a prompt
/// gets a refusal, not an unattended `curl | bash`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeclineScripts;

#[async_trait]
impl StepConfirmer for DeclineScripts {
    async fn confirm(&self, _index: usize, _display: &str, _command: &str) -> bool {
        false
    }
}

/// A cooperative cancel flag.
///
/// Upstream takes an `AbortSignal`. Rust's idiom would be to drop the future,
/// but the installer must distinguish *cancelled* from *failed* in its result,
/// so the flag is explicit. Recorded as a deviation.
#[derive(Debug, Clone, Default)]
pub struct InstallCancel {
    flag: Arc<AtomicBool>,
}

impl InstallCancel {
    /// A flag that has not been raised.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raise it. Every subsequent step check, and the running step, gives up.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// Whether it is raised.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

/// What one step did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StepOutcome {
    /// The exit code, or `-1` when there was none.
    pub code: i32,
    /// The bounded output tail.
    pub output: String,
    /// Whether it ran out of time.
    pub timeout: bool,
    /// Whether it was cancelled.
    pub aborted: bool,
    /// A spawn error, if the process never started.
    pub error: Option<String>,
}

/// One step's command line, environment and deadline.
#[derive(Debug, Clone)]
pub struct StepRequest {
    /// The executable.
    pub command: String,
    /// Its arguments.
    pub arguments: Vec<String>,
    /// The child environment.
    pub environment: EnvMap,
    /// The deadline.
    pub timeout: Duration,
}

/// Running one installation step.
///
/// Injected because upstream injects `spawnImpl`: the installer's branching —
/// skip, confirm, timeout, tail, verify — is the part worth testing, and it
/// must be testable without installing anything.
#[async_trait]
pub trait StepRunner: Send + Sync {
    /// Run one step to completion.
    async fn run(
        &self,
        request: StepRequest,
        cancel: &InstallCancel,
        on_output: &(dyn for<'a> Fn(OutputStream, &'a str) + Send + Sync),
    ) -> StepOutcome;
}

/// Whether a component the installer would install is already present.
///
/// The half of upstream's setup report `stepComponentReady`
/// (`shared/backend-install.mjs:331-343`) actually reads. Keeping it this
/// narrow is what lets the installer be exercised without a filesystem.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComponentReadiness {
    /// Whether the backend itself checks out.
    pub backend_ready: bool,
    /// Whether its ACP adapter checks out.
    pub adapter_ready: bool,
    /// Whether the whole backend checks out.
    pub ready: bool,
    /// Per-package readiness, by package name, when the report carries it.
    pub packages: Vec<(String, bool)>,
    /// The first issue, used for [`InstallError::VerifyFailed`].
    pub first_issue: Option<String>,
}

/// Inspecting a backend before and after installation.
#[async_trait]
pub trait SetupInspector: Send + Sync {
    /// Inspect `id` against `env`.
    async fn inspect(&self, id: &str, env: &EnvMap) -> ComponentReadiness;
}

/// `stepComponentReady(step, item, env)` — `shared/backend-install.mjs:331-343`.
///
/// A per-package answer wins when the report carries one; otherwise the step's
/// component decides. A report with no detail at all is conservatively *not*
/// ready, so the step runs.
#[must_use]
pub fn step_component_ready(
    step: &InstallStepSpec,
    readiness: &ComponentReadiness,
    env: &EnvMap,
) -> bool {
    if step.kind == StepKind::Npm && !readiness.packages.is_empty() {
        let (name, _) = package_identity(&step_package(step, env));
        if let Some((_, ready)) = readiness
            .packages
            .iter()
            .find(|(observed, _)| *observed == name)
        {
            return *ready;
        }
    }
    match step.component {
        Some(InstallComponent::Adapter) => readiness.adapter_ready,
        _ => readiness.backend_ready,
    }
}

/// What a successful installation reports back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallReport {
    /// Whether every step was skipped because everything was already present.
    pub already_installed: bool,
    /// The backend's own sign-in guidance, when one is still needed.
    pub configuration_hint: Option<String>,
    /// The backend's sign-in support, as resolved after installation.
    pub authentication: AuthenticationSupport,
}

/// Everything `install_backend` needs that is not a value.
pub struct InstallRequest<'a> {
    /// The backend id.
    pub id: &'a str,
    /// The Gateway environment. `PATH` is extended in place as global installs
    /// publish new bin directories.
    pub env: EnvMap,
    /// The platform whose step filter and shell apply.
    pub platform: HostPlatform,
    /// The locale every message is rendered in.
    pub locale: Locale,
    /// Locating `npm`.
    pub finder: &'a dyn ExecutableFinder,
    /// Running steps.
    pub runner: &'a dyn StepRunner,
    /// Inspecting before and after.
    pub inspector: &'a dyn SetupInspector,
    /// Confirming script steps.
    pub confirmer: &'a dyn StepConfirmer,
    /// Reporting progress.
    pub observer: &'a dyn InstallObserver,
    /// Cancellation.
    pub cancel: InstallCancel,
    /// Per-step deadline.
    pub step_timeout: Duration,
}

/// Install one backend.
///
/// **External contract** — `installBackend(id, options)`,
/// `shared/backend-install.mjs:436-681`. The sequence: inspect once, run only
/// the steps whose component is missing, confirm every `script` step with a
/// human, then inspect again and refuse to claim success if the backend still
/// does not check out.
///
/// # Errors
///
/// One of the seven [`InstallError`] codes.
pub async fn install_backend(
    mut request: InstallRequest<'_>,
) -> Result<InstallReport, InstallError> {
    let support = install_support(request.id, &request.env, request.platform, request.locale);
    if !support.supported {
        return Err(InstallError::Unsupported {
            reason: support.reason.unwrap_or_default(),
        });
    }
    let definition = backend_definition(request.id).ok_or_else(|| InstallError::Unsupported {
        reason: format(
            request.locale,
            keys::INSTALL_UNSUPPORTED_BACKEND,
            &[("id", request.id.trim())],
        ),
    })?;
    let plan = install_steps(definition.id, request.platform);

    let before = request.inspector.inspect(definition.id, &request.env).await;
    let pending: Vec<bool> = plan
        .steps
        .iter()
        .map(|step| !step_component_ready(step, &before, &request.env))
        .collect();

    if !pending.iter().any(|value| *value) && before.ready {
        return Ok(finish(&request, definition.id, true));
    }

    let npm = if plan
        .steps
        .iter()
        .zip(&pending)
        .any(|(step, run)| *run && step.kind == StepKind::Npm)
    {
        let resolved = resolve_npm(request.finder, request.platform);
        if resolved.is_empty() {
            return Err(InstallError::NpmMissing);
        }
        resolved
    } else {
        String::new()
    };

    for (index, step) in plan.steps.iter().enumerate() {
        let display = step_display(step, &request.env);
        let title = step_title(step, index, request.locale);
        if !pending[index] {
            request.observer.progress(&ProgressEvent {
                step: index,
                phase: ProgressPhase::Skip,
                title: Some(title),
                display: Some(display),
                stream: None,
                chunk: None,
            });
            continue;
        }
        request.observer.progress(&ProgressEvent {
            step: index,
            phase: ProgressPhase::Start,
            title: Some(title.clone()),
            display: Some(display.clone()),
            stream: None,
            chunk: None,
        });
        if step.kind == StepKind::Script
            && !request
                .confirmer
                .confirm(index, &display, step.command)
                .await
        {
            return Err(InstallError::Declined);
        }
        let step_request = match step.kind {
            StepKind::Npm => StepRequest {
                command: npm.clone(),
                arguments: npm_step_args(step, &request.env),
                environment: npm_run_env(&request.env, &npm, request.platform),
                timeout: request.step_timeout,
            },
            StepKind::Script if request.platform.is_windows() => StepRequest {
                command: WINDOWS_SHELL.to_owned(),
                arguments: WINDOWS_SHELL_FLAGS
                    .iter()
                    .map(|flag| (*flag).to_owned())
                    .chain(std::iter::once(step.command.to_owned()))
                    .collect(),
                environment: request.env.clone(),
                timeout: request.step_timeout,
            },
            StepKind::Script => StepRequest {
                command: POSIX_SHELL.to_owned(),
                arguments: vec!["-c".to_owned(), step.command.to_owned()],
                environment: request.env.clone(),
                timeout: request.step_timeout,
            },
        };
        let observer = request.observer;
        let outcome = request
            .runner
            .run(step_request, &request.cancel, &|stream, chunk| {
                observer.progress(&ProgressEvent {
                    step: index,
                    phase: ProgressPhase::Output,
                    title: None,
                    display: None,
                    stream: Some(stream),
                    chunk: Some(chunk.to_owned()),
                });
            })
            .await;

        if outcome.aborted {
            return Err(InstallError::Cancelled);
        }
        if outcome.timeout {
            return Err(InstallError::StepTimeout { display });
        }
        if outcome.code != 0 {
            return Err(InstallError::StepFailed {
                display,
                exit_code: outcome.code,
                cause: outcome
                    .error
                    .unwrap_or_else(|| outcome.output.trim_end().to_owned()),
            });
        }
        if step.kind == StepKind::Npm && !npm.is_empty() {
            // A freshly installed global binary lives in npm's global bin
            // directory, which may not have been on `PATH` when the Gateway
            // started. Without this the verification pass below cannot see what
            // was just installed.
            let global_bin = npm_global_bin(&request.env, request.platform);
            let merged = merge_search_path(
                request.env.get_trimmed("PATH"),
                &global_bin,
                request.platform.paths(),
                false,
            );
            request.env.set("PATH", merged);
        }
        request.observer.progress(&ProgressEvent {
            step: index,
            phase: ProgressPhase::Done,
            title: Some(title),
            display: None,
            stream: None,
            chunk: None,
        });
    }

    let after = request.inspector.inspect(definition.id, &request.env).await;
    if !after.ready {
        return Err(InstallError::VerifyFailed {
            detail: after.first_issue.unwrap_or_else(|| {
                t(request.locale, keys::INSTALL_COMPLETED_BUT_UNAVAILABLE).to_owned()
            }),
        });
    }
    Ok(finish(&request, definition.id, false))
}

fn finish(request: &InstallRequest<'_>, id: &str, already_installed: bool) -> InstallReport {
    let authentication =
        backend_authentication_support(id, &request.env, Some(request.platform), request.locale);
    InstallReport {
        already_installed,
        configuration_hint: crate::onboarding::onboarding_hint_key(id)
            .filter(|_| authentication.supported)
            .map(|key| t(request.locale, key).to_owned()),
        authentication,
    }
}

/// `npm.cmd` on Windows, `npm` elsewhere, with upstream's fallback to the bare
/// name — `shared/backend-install.mjs:499`.
fn resolve_npm(finder: &dyn ExecutableFinder, platform: HostPlatform) -> String {
    let primary = if platform.is_windows() {
        "npm.cmd"
    } else {
        "npm"
    };
    let found = finder.find(primary);
    if found.is_empty() {
        finder.find("npm")
    } else {
        found
    }
}

/// npm's global bin directory, when `npm config get prefix` is unavailable.
///
/// **External contract** — `shared/backend-install.mjs:632-637`. Upstream asks
/// npm first and falls back to these; VIA uses the fallbacks directly rather
/// than spawning a second npm just to read a path, and records the deviation.
fn npm_global_bin(env: &EnvMap, platform: HostPlatform) -> String {
    if platform.is_windows() {
        let base = env
            .first_truthy(&["APPDATA", "USERPROFILE", "HOME"])
            .unwrap_or_default();
        std::format!("{base}\\npm")
    } else {
        "/usr/local/bin".to_owned()
    }
}

/// `npmRunEnv(baseEnv, npmCommand, platform)` — `shared/backend-install.mjs:207-231`.
///
/// The `PATH` composition is `via-process`'s
/// [`compose_child_search_path`](via_process::compose_child_search_path): drop
/// `PATH` entries that are executables rather than directories, then prepend the
/// directory `npm` was found in, so npm's own `postinstall` hooks can find the
/// matching `node`.
#[must_use]
pub fn npm_run_env(base: &EnvMap, npm_command: &str, platform: HostPlatform) -> EnvMap {
    let mut env = base.clone();
    env.set(NPM_CONFIG_YES.0, NPM_CONFIG_YES.1);
    if npm_command.is_empty() {
        return env;
    }
    let composed = via_process::compose_child_search_path(
        base.get_trimmed("PATH"),
        npm_command,
        platform.paths(),
    );
    env.set("PATH", composed);
    env
}

/// Append a chunk to a bounded output tail.
///
/// **External contract** — `shared/backend-install.mjs:297-301`: each chunk is
/// prefixed with its stream name and the whole buffer is truncated from the
/// front to [`MAX_INSTALL_OUTPUT_CHARS`].
#[must_use]
pub fn append_output(current: &str, stream: OutputStream, chunk: &str) -> String {
    let combined = std::format!("{current}[{}] {chunk}", stream.as_str());
    if combined.chars().count() <= MAX_INSTALL_OUTPUT_CHARS {
        return combined;
    }
    let skip = combined.chars().count() - MAX_INSTALL_OUTPUT_CHARS;
    combined.chars().skip(skip).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    #[test]
    fn the_generic_acp_backend_is_never_installed_for_the_user() {
        let support = install_support("acp", &EnvMap::new(), HostPlatform::Darwin, Locale::Zh);
        assert!(!support.supported);
        assert_eq!(
            support.reason.as_deref(),
            Some("通用 ACP 接入的 Agent 需自行安装，并通过 ACP_COMMAND 配置")
        );
        assert!(install_spec("acp").is_none());
    }

    #[test]
    fn hermes_installs_from_a_different_script_per_platform() {
        for (platform, expected) in [
            (HostPlatform::Darwin, HERMES_INSTALL_COMMAND),
            (HostPlatform::Linux, HERMES_INSTALL_COMMAND),
            (HostPlatform::Windows, HERMES_INSTALL_COMMAND_WINDOWS),
        ] {
            let plan = install_steps("hermes", platform);
            assert_eq!(plan.steps.len(), 1, "{platform:?}");
            assert_eq!(plan.steps[0].command, expected);
            assert_eq!(plan.steps[0].kind, StepKind::Script);
        }
    }

    #[test]
    fn a_script_step_needs_confirmation_and_an_npm_step_does_not() {
        assert!(
            install_support("hermes", &EnvMap::new(), HostPlatform::Darwin, Locale::En)
                .requires_confirmation
        );
        assert!(
            !install_support("codex", &EnvMap::new(), HostPlatform::Darwin, Locale::En)
                .requires_confirmation
        );
    }

    #[test]
    fn deepseek_installs_its_acp_runtime_last() {
        let plan = install_steps("deepseek", HostPlatform::Linux);
        assert_eq!(plan.steps.len(), 11);
        assert!(plan.verify_installed_packages);
        assert_eq!(plan.steps[0].package, "@deepseek-ai/dsh@0.1.0-rc.6");
        assert_eq!(plan.steps[0].component, Some(InstallComponent::Backend));
        let last = plan.steps[plan.steps.len() - 1];
        assert_eq!(last.package, "@deepseek-ai/dsh-acp-demo@0.1.0-rc.6");
        assert_eq!(last.label, Some(StepLabel::AcpRuntime));
        for step in &plan.steps {
            assert_eq!(step.registry, Some(DEEPSEEK_REGISTRY));
            assert_eq!(step.package_env, None, "{}", step.package);
        }
    }

    #[test]
    fn a_package_env_overrides_the_pinned_coordinate() {
        let step = &OPENCODE_STEPS[0];
        assert_eq!(step_package(step, &EnvMap::new()), "opencode-ai@1.18.5");
        assert_eq!(
            step_package(step, &env(&[("OPENCODE_PACKAGE", "opencode-ai@9.9.9")])),
            "opencode-ai@9.9.9"
        );
        // A blank override does not win: upstream's `clean(...) || step.package`.
        assert_eq!(
            step_package(step, &env(&[("OPENCODE_PACKAGE", "   ")])),
            "opencode-ai@1.18.5"
        );
    }

    #[test]
    fn the_displayed_command_line_carries_the_registry() {
        assert_eq!(
            step_display(&DEEPSEEK_STEPS[0], &EnvMap::new()),
            "npm install -g --registry=https://registry.npmjs.org/ @deepseek-ai/dsh@0.1.0-rc.6"
        );
        assert_eq!(
            step_display(&OPENCODE_STEPS[0], &EnvMap::new()),
            "npm install -g opencode-ai@1.18.5"
        );
        assert_eq!(
            npm_step_args(&DEEPSEEK_STEPS[0], &EnvMap::new()),
            vec![
                "install",
                "-g",
                "--registry=https://registry.npmjs.org/",
                "@deepseek-ai/dsh@0.1.0-rc.6"
            ]
        );
        assert_eq!(
            step_display(&HERMES_STEPS[0], &EnvMap::new()),
            HERMES_INSTALL_COMMAND
        );
    }

    #[test]
    fn step_titles_are_one_based_and_label_the_adapter() {
        assert_eq!(step_title(&CODEX_STEPS[0], 0, Locale::Zh), "步骤 1");
        assert_eq!(
            step_title(&CODEX_STEPS[1], 1, Locale::Zh),
            "步骤 2（ACP 适配器）"
        );
    }

    #[test]
    fn a_scoped_package_keeps_its_scope() {
        assert_eq!(
            package_identity("@deepseek-ai/dsh@0.1.0-rc.6"),
            ("@deepseek-ai/dsh".to_owned(), "0.1.0-rc.6".to_owned())
        );
        assert_eq!(
            package_identity("opencode-ai@1.18.5"),
            ("opencode-ai".to_owned(), "1.18.5".to_owned())
        );
        assert_eq!(
            package_identity("@scope/pkg"),
            ("@scope/pkg".to_owned(), String::new())
        );
    }

    #[test]
    fn every_error_carries_its_catalogued_code() {
        assert_eq!(InstallError::NpmMissing.code(), "NPM_MISSING");
        assert_eq!(InstallError::Declined.code(), "DECLINED");
        assert_eq!(InstallError::Cancelled.code(), "CANCELLED");
        assert_eq!(
            InstallError::StepTimeout {
                display: "x".to_owned()
            }
            .code(),
            "STEP_TIMEOUT"
        );
        assert_eq!(
            InstallError::StepFailed {
                display: "x".to_owned(),
                exit_code: 1,
                cause: String::new(),
            }
            .code(),
            "STEP_FAILED"
        );
        assert_eq!(
            InstallError::VerifyFailed {
                detail: "x".to_owned()
            }
            .code(),
            "VERIFY_FAILED"
        );
        assert_eq!(
            InstallError::Unsupported {
                reason: "x".to_owned()
            }
            .code(),
            "UNSUPPORTED"
        );
    }

    #[test]
    fn declining_and_cancelling_read_differently() {
        // Upstream keeps two sentences for these two states.
        assert_eq!(InstallError::Declined.message(Locale::Zh), "已取消安装");
        assert_eq!(InstallError::Cancelled.message(Locale::Zh), "安装已取消");
    }

    #[test]
    fn a_missing_per_package_answer_falls_back_to_the_component() {
        let step = &CODEX_STEPS[1];
        let readiness = ComponentReadiness {
            backend_ready: true,
            adapter_ready: false,
            ..ComponentReadiness::default()
        };
        assert!(!step_component_ready(step, &readiness, &EnvMap::new()));
        assert!(step_component_ready(
            &CODEX_STEPS[0],
            &readiness,
            &EnvMap::new()
        ));
    }

    #[test]
    fn a_per_package_answer_wins_over_the_component() {
        let readiness = ComponentReadiness {
            backend_ready: true,
            adapter_ready: true,
            packages: vec![("@deepseek-ai/dsh-acp-demo".to_owned(), false)],
            ..ComponentReadiness::default()
        };
        let last = &DEEPSEEK_STEPS[DEEPSEEK_STEPS.len() - 1];
        assert!(
            !step_component_ready(last, &readiness, &EnvMap::new()),
            "the package report says it is missing even though the adapter looks ready"
        );
    }

    #[test]
    fn an_empty_report_is_conservatively_not_ready() {
        let readiness = ComponentReadiness::default();
        for step in DEEPSEEK_STEPS {
            assert!(!step_component_ready(step, &readiness, &EnvMap::new()));
        }
    }

    #[test]
    fn npm_runs_non_interactively_with_its_own_directory_first() {
        let base = env(&[("PATH", "/usr/bin:/bin")]);
        let composed = npm_run_env(&base, "/opt/node/bin/npm", HostPlatform::Linux);
        assert_eq!(composed.get("npm_config_yes"), Some("true"));
        assert_eq!(composed.get("PATH"), Some("/opt/node/bin:/usr/bin:/bin"));
    }

    #[test]
    fn a_path_entry_that_is_an_executable_is_dropped_before_npm_runs() {
        // `npmRunEnv` exists because `npm` spawns `node`; a `PATH` entry that is
        // itself `npm.cmd` would make that lookup fail.
        let base = env(&[("PATH", "C:\\tools\\nodejs\\npm.cmd;C:\\Windows")]);
        let composed = npm_run_env(&base, "C:\\tools\\nodejs\\npm.cmd", HostPlatform::Windows);
        assert_eq!(composed.get("PATH"), Some("C:\\tools\\nodejs;C:\\Windows"));
    }

    #[test]
    fn npm_run_env_without_a_command_only_adds_the_yes_flag() {
        let base = env(&[("PATH", "/usr/bin")]);
        let composed = npm_run_env(&base, "", HostPlatform::Linux);
        assert_eq!(composed.get("npm_config_yes"), Some("true"));
        assert_eq!(composed.get("PATH"), Some("/usr/bin"));
    }

    #[test]
    fn the_output_tail_is_bounded_and_labelled() {
        assert_eq!(
            append_output("", OutputStream::Stderr, "boom"),
            "[stderr] boom"
        );
        let long = "x".repeat(MAX_INSTALL_OUTPUT_CHARS * 2);
        let tail = append_output("", OutputStream::Stdout, &long);
        assert_eq!(tail.chars().count(), MAX_INSTALL_OUTPUT_CHARS);
        assert!(tail.ends_with('x'));
    }
}
