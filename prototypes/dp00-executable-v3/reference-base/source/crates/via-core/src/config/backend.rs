//! Backend selection and the twelve per-backend option namespaces.
//!
//! Ported from `server/src/core/config.mjs:35-331`. The static half of this —
//! which backends exist, what their environment allow-lists are, whether they
//! support an external service — is `via-catalog`'s. What lives here is the
//! *environment reading* on top of it: which model string each backend gets,
//! where its workspace is, and which executable to run.
//!
//! # One model name, twelve spellings
//!
//! `VIA_BACKEND_MODEL` is a single operator-facing setting, and each backend
//! wants it differently: OpenCode wants `alibaba-cn/<name>`, OpenClaw wants
//! `bailian/<name>`, Qoder wants the bare `<name>`, Kimi wants the whole value
//! including any provider prefix, and DeepSeek wants it only if it looks like a
//! DeepSeek model. Both prefixes are literal strings the third-party backends
//! parse, so [`BackendModels`] reproduces the mapping exactly and
//! `resolve_backend_models` is asserted against upstream's own test table.

use std::path::{Path, PathBuf};

use via_catalog::backend_definition;

use crate::config::names;
use crate::env::EnvMap;
use crate::error::CoreError;
use crate::paths::resolve_path;
use crate::secret::Secret;

/// The OpenAI-compatible DashScope endpoint.
///
/// **External contract** and KEEP — `server/src/core/config.mjs:293` and
/// `:454`. It is both the memory extractor's default base URL and the value
/// Codex is pointed at when a Bailian model is configured, so it is named once
/// rather than typed twice.
pub const DASHSCOPE_COMPATIBLE_BASE_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1";

/// Path appended for CodeBuddy, which wants the chat-completions endpoint
/// rather than the base.
///
/// **External contract** — `server/src/core/config.mjs:280`.
pub const CHAT_COMPLETIONS_SUFFIX: &str = "/chat/completions";

/// Host-and-path suffix of the per-workspace OpenAI-compatible endpoint.
///
/// **External contract** — `server/src/core/config.mjs:279,292`.
pub const DASHSCOPE_WORKSPACE_COMPATIBLE_SUFFIX: &str =
    ".cn-beijing.maas.aliyuncs.com/compatible-mode/v1";

/// The coordinator agent id VIA registers inside a third-party backend.
///
/// **External contract** — upstream `qwen-audio-agent-backend`
/// (`server/src/core/config.mjs:151,161`), renamed per `docs/rebrand.md`.
/// Renaming it is a user-visible migration: the id is written into OpenCode's
/// and OpenClaw's own configuration.
pub const VIA_BACKEND_AGENT_ID: &str = "via-backend";

/// A second reserved agent id OpenCode blanks to its own default.
///
/// **External contract** — upstream `qwen-audio-agent-coordinator`
/// (`server/src/core/config.mjs:162`), renamed per `docs/rebrand.md`.
pub const VIA_COORDINATOR_AGENT_ID: &str = "via-coordinator";

/// The pre-1.11 OpenClaw coordinator agent id.
///
/// **External contract** — `server/src/core/config.mjs:243`. Brand-free, so it
/// is not renamed; a user who still has it configured is mapped forward to
/// [`VIA_BACKEND_AGENT_ID`].
pub const LEGACY_OPENCLAW_COORDINATOR_AGENT: &str = "voice-coordinator";

/// The literal that means "no explicit model override".
///
/// **External contract** — `server/src/core/config.mjs:78`, compared
/// case-insensitively.
pub const AUTO_MODEL_SENTINEL: &str = "auto";

/// `VIA_BACKEND_MODEL`, spelled the way each backend needs it.
///
/// **External contract** — `server/src/core/config.mjs:74-98`, asserted field
/// for field in `server/test/config.test.mjs:78-168`.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendModels {
    /// The configured value, with `auto` normalised to empty.
    pub common: String,
    /// `alibaba-cn/<name>`, or empty.
    pub open_code: String,
    /// `bailian/<name>`, or empty.
    pub open_claw: String,
    /// The bare name, i.e. everything after the first `/`.
    pub qoder: String,
    /// The bare name.
    pub qwen: String,
    /// The whole configured value, prefix included.
    pub kimi: String,
    /// The whole configured value.
    pub hermes: String,
    /// The bare name.
    pub code_buddy: String,
    /// The bare name.
    pub codex: String,
    /// The whole configured value.
    pub claude: String,
    /// `DEEPSEEK_HARNESS_MODEL`, else the bare name when it starts with
    /// `deepseek-`, else empty.
    pub deep_seek_harness: String,
    /// The whole configured value.
    pub pi: String,
    /// The whole configured value.
    pub acp: String,
}

/// Everything after the first `/`, or the whole string when there is none.
///
/// **External contract** — `server/src/core/config.mjs:68-72`.
#[must_use]
pub fn backend_model_name(value: &str) -> &str {
    let model = value.trim();
    match model.find('/') {
        Some(separator) => &model[separator + 1..],
        None => model,
    }
}

/// Derive every backend's model string from `VIA_BACKEND_MODEL`.
#[must_use]
pub fn resolve_backend_models(env: &EnvMap) -> BackendModels {
    let configured = env.get_trimmed(names::BACKEND_MODEL);
    let common = if configured.to_lowercase() == AUTO_MODEL_SENTINEL {
        ""
    } else {
        configured
    };
    let name = backend_model_name(common);
    let prefixed = |prefix: &str| {
        if common.is_empty() {
            String::new()
        } else {
            format!("{prefix}{name}")
        }
    };
    let deep_seek_harness = match env.get_truthy(names::DEEPSEEK_HARNESS_MODEL) {
        Some(explicit) => explicit.trim().to_owned(),
        None if name.starts_with("deepseek-") => name.to_owned(),
        None => String::new(),
    };
    BackendModels {
        common: common.to_owned(),
        open_code: prefixed("alibaba-cn/"),
        open_claw: prefixed("bailian/"),
        qoder: name.to_owned(),
        qwen: name.to_owned(),
        kimi: common.to_owned(),
        hermes: common.to_owned(),
        code_buddy: name.to_owned(),
        codex: name.to_owned(),
        claude: common.to_owned(),
        deep_seek_harness,
        pi: common.to_owned(),
        acp: common.to_owned(),
    }
}

/// Where a backend's workspace is.
///
/// **External contract** — `server/src/core/config.mjs:35-48`. An explicit
/// `<BACKEND>_WORKSPACE` resolves against the runtime root; otherwise **every**
/// backend shares `<data>/workspace`. The historical per-backend
/// `workspaces/<id>/` directories are only ever reported, never migrated.
///
/// # Errors
///
/// [`CoreError::BackendWorkspaceMissing`] when the id names no backend, or one
/// with no workspace variable.
pub fn resolve_backend_workspace(
    protocol: &str,
    env: &EnvMap,
    root: &Path,
    data_directory: &Path,
) -> Result<PathBuf, CoreError> {
    let variable = backend_definition(protocol)
        .map(|definition| definition.workspace_environment)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| CoreError::BackendWorkspaceMissing {
            protocol: protocol.to_owned(),
        })?;
    Ok(match env.get_truthy(variable) {
        Some(configured) => resolve_path(root, configured),
        None => data_directory.join(crate::paths::SHARED_WORKSPACE_DIRECTORY_NAME),
    })
}

/// Map the pre-1.11 OpenClaw coordinator id forward.
///
/// **External contract** — `server/src/core/config.mjs:149-152`.
#[must_use]
pub fn legacy_backend_agent(value: Option<&str>, legacy_default: &str) -> String {
    let selected = value.unwrap_or_default().trim();
    if selected == legacy_default {
        VIA_BACKEND_AGENT_ID.to_owned()
    } else {
        selected.to_owned()
    }
}

/// OpenCode's coordinator agent, or empty to use OpenCode's own default.
///
/// **External contract** — `server/src/core/config.mjs:154-164`, test-locked at
/// `server/test/config.test.mjs:46-58`. `VIA_BACKEND_AGENT` wins over
/// `OPENCODE_COORDINATOR_AGENT`, and either of the two reserved VIA ids means
/// "use the backend's default" rather than naming an agent.
#[must_use]
pub fn resolve_opencode_coordinator_agent(env: &EnvMap) -> String {
    let selected = env
        .first_truthy(&[names::BACKEND_AGENT, names::OPENCODE_COORDINATOR_AGENT])
        .unwrap_or_default()
        .trim();
    if selected == VIA_BACKEND_AGENT_ID || selected == VIA_COORDINATOR_AGENT_ID {
        String::new()
    } else {
        selected.to_owned()
    }
}

/// Options every stdio-launched backend shares.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliBackendOptions {
    /// Model string for this backend.
    pub model: String,
    /// Working directory the agent runs in.
    pub directory: PathBuf,
    /// Explicit executable path, or empty to resolve from `PATH`.
    pub cli_path: String,
}

/// OpenCode's namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeOptions {
    /// HTTP base URL, trailing slashes stripped.
    pub base_url: String,
    /// Model string.
    pub model: String,
    /// Working directory.
    pub directory: PathBuf,
    /// Coordinator agent id, or empty for OpenCode's default.
    pub coordinator_agent: String,
}

/// OpenClaw's namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawOptions {
    /// HTTP or WebSocket base URL, trailing slashes stripped.
    pub base_url: String,
    /// Gateway token. Redacted in `Debug` and in serialization.
    pub token: Secret,
    /// File the token is read from when it is not set directly.
    pub token_file: PathBuf,
    /// Model string.
    pub model: String,
    /// Working directory.
    pub directory: PathBuf,
    /// Explicit executable path.
    pub cli_path: String,
    /// Coordinator agent id.
    pub coordinator_agent: String,
}

/// Qoder's namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QoderOptions {
    /// Model string.
    pub model: String,
    /// Working directory.
    pub directory: PathBuf,
    /// Explicit executable path.
    pub cli_path: String,
    /// Qoder's own configuration directory, absolute, or empty.
    pub config_directory: PathBuf,
}

/// CodeBuddy's namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuddyOptions {
    /// Model string.
    pub model: String,
    /// Chat-completions endpoint, derived when a Bailian model is configured.
    pub model_url: String,
    /// Working directory.
    pub directory: PathBuf,
    /// Explicit executable path.
    pub cli_path: String,
}

/// Codex's namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexOptions {
    /// Model string.
    pub model: String,
    /// OpenAI-compatible base URL, trailing slashes stripped.
    pub model_url: String,
    /// Working directory.
    pub directory: PathBuf,
    /// Explicit ACP adapter path.
    pub cli_path: String,
}

/// Claude Code's namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeOptions {
    /// Model string.
    pub model: String,
    /// Working directory.
    pub directory: PathBuf,
    /// Explicit ACP adapter path.
    pub cli_path: String,
    /// Path to the `claude` executable the adapter should drive.
    pub claude_executable: String,
    /// Claude Code's own configuration directory, absolute, or empty.
    pub config_directory: PathBuf,
}

/// DeepSeek Harness's namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepSeekOptions {
    /// Model string.
    pub model: String,
    /// Working directory.
    pub directory: PathBuf,
    /// Explicit ACP adapter path.
    pub cli_path: String,
    /// Where the harness keeps its sessions.
    pub session_root: PathBuf,
}

/// The generic ACP backend's namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpOptions {
    /// Model string.
    pub model: String,
    /// Working directory.
    pub directory: PathBuf,
    /// The command to run. Required when `AGENT_PROTOCOL=acp`.
    pub cli_path: String,
    /// Arguments, from `ACP_ARGS`.
    pub args: Vec<String>,
    /// Display label.
    pub label: String,
    /// Coordinator agent id.
    pub coordinator_agent: String,
}

/// All twelve namespaces.
///
/// Upstream keys these by driver id in one object literal
/// (`server/src/core/config.mjs:212-332`); named fields make a typo a compile
/// error instead of a silent `undefined`.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendOptions {
    /// OpenCode.
    pub opencode: OpenCodeOptions,
    /// OpenClaw.
    pub openclaw: OpenClawOptions,
    /// Qoder.
    pub qoder: QoderOptions,
    /// Qwen Code — a third-party CLI; the id is KEEP.
    pub qwen: CliBackendOptions,
    /// Kimi Code.
    pub kimi: CliBackendOptions,
    /// Hermes.
    pub hermes: CliBackendOptions,
    /// CodeBuddy.
    pub codebuddy: CodeBuddyOptions,
    /// Codex.
    pub codex: CodexOptions,
    /// Claude Code.
    pub claude: ClaudeOptions,
    /// DeepSeek Harness.
    pub deepseek: DeepSeekOptions,
    /// Pi.
    pub pi: CliBackendOptions,
    /// The generic ACP entry point.
    pub acp: AcpOptions,
}

/// Parse `ACP_ARGS`.
///
/// **External contract** — `server/src/core/config.mjs:50-66`. A value opening
/// with `[` must be a JSON array of strings; anything else is split on runs of
/// whitespace. The whitespace fallback means **quoted arguments are not
/// honoured** unless the JSON form is used, which is why `.env.example`
/// recommends it.
///
/// # Errors
///
/// [`CoreError::AcpArgsNotJson`] or [`CoreError::AcpArgsNotStringArray`].
pub fn resolve_acp_args(value: Option<&str>) -> Result<Vec<String>, CoreError> {
    let source = value.unwrap_or_default().trim();
    if source.is_empty() {
        return Ok(Vec::new());
    }
    if source.starts_with('[') {
        let parsed: serde_json::Value =
            serde_json::from_str(source).map_err(|_| CoreError::AcpArgsNotJson)?;
        let items = parsed
            .as_array()
            .ok_or(CoreError::AcpArgsNotStringArray)?
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect::<Option<Vec<String>>>()
            .ok_or(CoreError::AcpArgsNotStringArray)?;
        return Ok(items);
    }
    Ok(source.split_whitespace().map(str::to_owned).collect())
}

/// Strip trailing `/` characters.
///
/// **External contract** — `server/src/core/config.mjs:217,226,295`
/// (`.replace(/\/+$/, '')`).
#[must_use]
pub fn strip_trailing_slashes(value: &str) -> &str {
    value.trim_end_matches('/')
}

/// The OpenAI-compatible endpoint for a workspace, or the shared one.
#[must_use]
pub fn dashscope_compatible_base_url(workspace_id: Option<&str>) -> String {
    match workspace_id {
        Some(id) => format!("https://{id}{DASHSCOPE_WORKSPACE_COMPATIBLE_SUFFIX}"),
        None => DASHSCOPE_COMPATIBLE_BASE_URL.to_owned(),
    }
}

/// Where the CodeBuddy model table lives inside its workspace.
///
/// **External contract** — `file-path/CodeBuddy models.json target`
/// (`shared/runtime-environment.mjs:366-410,:550-556`):
/// `<codeBuddyWorkspace>/.codebuddy/models.json`. The write itself —
/// directory mode `0o700`, file mode `0o600`, the exclusive-create flag
/// upstream calls `'wx'` — is [`crate::runtime::seed_codebuddy_models_json`],
/// which resolves the path through this function so the two cannot disagree.
#[must_use]
pub fn codebuddy_models_json_path(codebuddy_workspace: &Path) -> PathBuf {
    codebuddy_workspace.join(".codebuddy").join("models.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    #[test]
    fn one_model_name_becomes_twelve() {
        let models = resolve_backend_models(&env(&[("VIA_BACKEND_MODEL", "qwen3.7-plus")]));
        assert_eq!(models.open_code, "alibaba-cn/qwen3.7-plus");
        assert_eq!(models.open_claw, "bailian/qwen3.7-plus");
        assert_eq!(models.qoder, "qwen3.7-plus");
        assert_eq!(models.deep_seek_harness, "");
    }

    #[test]
    fn auto_means_no_override_in_any_case() {
        for value in ["auto", "AUTO", "Auto"] {
            let models = resolve_backend_models(&env(&[("VIA_BACKEND_MODEL", value)]));
            assert_eq!(models, BackendModels::default());
        }
    }

    #[test]
    fn acp_args_reject_a_non_string_array() {
        assert!(matches!(
            resolve_acp_args(Some("[1,2]")),
            Err(CoreError::AcpArgsNotStringArray)
        ));
        assert!(matches!(
            resolve_acp_args(Some("[--acp")),
            Err(CoreError::AcpArgsNotJson)
        ));
        assert_eq!(
            resolve_acp_args(Some("--acp  --verbose")).expect("whitespace form always parses"),
            vec!["--acp".to_owned(), "--verbose".to_owned()]
        );
    }
}
