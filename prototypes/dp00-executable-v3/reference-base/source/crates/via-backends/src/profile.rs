//! What a driver returns: everything about one backend that is not ACP.
//!
//! Upstream's `createProfile(options)` returns an object mixing three unrelated
//! things — the ACP connection description, the seven capability flags, and a
//! grab-bag of per-backend behaviour that Layer 2 reads (session instructions, a
//! UI URL builder, a coordinator `_meta`, a retry policy). `registry.mjs:72-86`
//! then freezes the flags on top.
//!
//! Here the ACP half is [`via_acp::AcpBackendProfile`] — the same value
//! `via-acp` takes, because it *is* ACP's — and everything else is
//! [`BackendProfile`]. The split is the crate boundary: `via-acp` may not know
//! what OpenClaw is, so the parts that name one live here.

use std::path::PathBuf;

use via_acp::AcpBackendProfile;
use via_downstream::BackendCapabilities;
use via_i18n::{Key, Locale, format, keys, t};

/// One `session/set_config_option` VIA sends after opening a session.
///
/// **External contract** — `server/src/agent/backends/local-acp.mjs:69-73`,
/// where Kimi's full-permission mode is `{ id: 'mode', value: 'auto' }`. Only
/// Kimi declares any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfigOption {
    /// The option id the agent advertises.
    pub id: &'static str,
    /// The value to select.
    pub value: &'static str,
}

/// How a backend's own web interface is addressed.
///
/// **External contract** — `opencode.mjs:52-60` and `openclaw.mjs:138-153`, the
/// two `uiUrl` builders behind `GET /api/backend/ui`. Only the two backends that
/// declare
/// [`backend_ui`](via_downstream::BackendCapabilities::backend_ui) have one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendUi {
    /// OpenCode: a per-server, per-session deep link.
    OpenCode,
    /// OpenClaw: its dashboard, with the gateway address and token in the URL
    /// fragment.
    OpenClaw {
        /// The gateway token, carried in the fragment when there is one.
        token: String,
    },
}

/// What a coordinator session tells the backend about itself.
///
/// **External contract** — `state-name/openclaw coordinator _meta.sessionKey`.
/// Sent inside `session/new._meta`, and *persisted by OpenClaw*: renaming the
/// product segment orphans every existing coordinator session on an installed
/// OpenClaw.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinatorMeta {
    /// `agent:<coordinatorAgent>:via:<owner>:backend`.
    pub session_key: String,
}

/// A side effect a driver needs performed just before its child is spawned.
///
/// Upstream's `prepare` hook (`backends/shared.mjs:24`), whose one instance is
/// OpenClaw writing its bridge token file (`openclaw.mjs:88-90`). It is data
/// rather than a closure so that a test can assert *what would be written*
/// without a filesystem, and so the profile stays `Clone` and `Debug`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrepareAction {
    /// Write `contents` to `path` with owner-only permissions.
    WritePrivateFile {
        /// Where.
        path: PathBuf,
        /// What. Includes its trailing newline — the format is part of the
        /// contract.
        contents: String,
    },
}

impl PrepareAction {
    /// Carry the action out.
    ///
    /// # Errors
    ///
    /// Any [`std::io::Error`] from creating the directory, writing the
    /// temporary file, or renaming it into place.
    pub fn run(&self) -> std::io::Result<()> {
        match self {
            Self::WritePrivateFile { path, contents } => {
                crate::openclaw::write_private_file(path, contents)
            }
        }
    }
}

/// The model-facing paragraph that replaces VIA's default session instructions.
///
/// # Why these are literals rather than `via-i18n` keys
///
/// All three are **model-facing English** and all three are pinned byte for byte
/// in `docs/reference/contracts.json` (`prompt-text/deepseek sessionInstructions`,
/// `/pi sessionInstructions`, `/openclaw sessionInstructions`). Upstream does not
/// localize them — there is one string, in English, in a codebase whose other
/// prompts are Chinese. Routing them through `via-i18n` would require inventing
/// `zh` and `ko` variants that upstream does not have, and the catalogued
/// contract is the English text, so translating it would break the assertion it
/// exists to make. Recorded as a deviation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionInstructions {
    /// OpenClaw drives delegation through its own session tools.
    OpenClaw,
    /// DeepSeek Harness exposes no session management at all.
    DeepSeek,
    /// Pi's ACP adapter does not wire VIA's session tools through.
    Pi,
}

/// OpenClaw's replacement paragraph — `backends/openclaw.mjs:120-126`.
pub const OPENCLAW_SESSION_INSTRUCTIONS: &str = "For a separate or previous project, use OpenClaw native session tools: sessions_spawn to create work, sessions_list to locate prior Sessions, sessions_send to continue one, and sessions_history for status. These are third-layer tasks. After spawn/send is accepted, return the delegated response required by the request envelope and stop this turn.";

/// DeepSeek's replacement paragraph — `backends/deepseek-harness.mjs:54-59`.
pub const DEEPSEEK_SESSION_INSTRUCTIONS: &str = "DeepSeek Harness ACP does not expose project Session management to the Gateway. Complete the requested work in this Session with the tools available to you. Do not claim to have opened or resumed a separate Session.";

/// Pi's replacement paragraph — `backends/pi.mjs:41-45`.
pub const PI_SESSION_INSTRUCTIONS: &str = "The current Pi ACP adapter does not expose Gateway Session tools. Complete the requested work in this Session with Pi's own tools. Do not claim to have opened or resumed a separate background Session.";

impl SessionInstructions {
    /// The paragraph.
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::OpenClaw => OPENCLAW_SESSION_INSTRUCTIONS,
            Self::DeepSeek => DEEPSEEK_SESSION_INSTRUCTIONS,
            Self::Pi => PI_SESSION_INSTRUCTIONS,
        }
    }
}

/// How a backend is told to cancel or report on a delegated task.
///
/// **External contract** — `prompt-text/openclaw cancelInstruction /
/// statusInstruction`. OpenClaw is the one backend that overrides the defaults,
/// because its delegation runs through its own session tools rather than VIA's
/// MCP ones, so the default sentences would name tools it has never been given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlInstructions {
    /// `via_session_cancel` / `via_session_status` — every `sessionMcp`
    /// backend.
    SessionTools,
    /// OpenClaw's own `sessions_history` and native stop.
    OpenClawNative,
}

impl ControlInstructions {
    /// The cancel sentence for a delegated task.
    ///
    /// `identifier` is the delegation id for [`Self::SessionTools`] and the
    /// OpenClaw `sessionKey` for [`Self::OpenClawNative`] — upstream passes
    /// `record.id` and `record.sessionId` respectively.
    #[must_use]
    pub fn cancel(self, identifier: &str, locale: Locale) -> String {
        match self {
            Self::SessionTools => format(
                locale,
                keys::ACP_CANCEL_INSTRUCTION_DEFAULT,
                &[("delegation_id", identifier)],
            ),
            Self::OpenClawNative => format(
                locale,
                keys::OPENCLAW_CANCEL_INSTRUCTION,
                &[("session_key", identifier)],
            ),
        }
    }

    /// The status sentence for a delegated task.
    #[must_use]
    pub fn status(self, identifier: &str, locale: Locale) -> String {
        match self {
            Self::SessionTools => format(
                locale,
                keys::ACP_STATUS_INSTRUCTION_DEFAULT,
                &[("delegation_id", identifier)],
            ),
            Self::OpenClawNative => format(
                locale,
                keys::OPENCLAW_STATUS_INSTRUCTION,
                &[("session_key", identifier)],
            ),
        }
    }
}

/// One backend, fully described.
///
/// The ACP half is handed straight to `via-acp`; everything else is read by
/// Layer 2.
#[derive(Debug, Clone)]
pub struct BackendProfile {
    /// The ACP connection, label and capability flags.
    pub acp: AcpBackendProfile,
    /// The seven flags again, as the immutable contract.
    ///
    /// Upstream publishes `Object.freeze({ ...driver.capabilities })` alongside
    /// the spread copy (`registry.mjs:82-84`); `BackendCapabilities` is `Copy`
    /// with no interior mutability, which is the same guarantee by construction.
    pub capabilities: BackendCapabilities,
    /// Options to select on a new session.
    pub session_config_options: Vec<SessionConfigOption>,
    /// A replacement for VIA's default session-tools paragraph.
    pub session_instructions: Option<SessionInstructions>,
    /// Which control sentences this backend understands.
    pub control_instructions: ControlInstructions,
    /// The label a delegated project task is announced under.
    pub delegation_title: String,
    /// A message shown while a managed service warms up.
    ///
    /// Empty for an external Gateway: upstream deliberately lets the official
    /// bridge establish the real connection so TLS, authentication, routing and
    /// remote-network errors are reported accurately instead of being hidden
    /// behind a local warm-up notice (`openclaw.mjs:129-136`).
    pub readiness_message: Option<String>,
    /// The `_meta` a coordinator session carries.
    pub coordinator_meta: Option<CoordinatorMeta>,
    /// The backend's own web interface, when it has one.
    pub backend_ui: Option<BackendUi>,
    /// Whether the model is configured through the process environment rather
    /// than through ACP session config (`deepseek-harness.mjs:52`).
    pub process_model_configuration: bool,
    /// A side effect to perform before spawning.
    pub prepare: Option<PrepareAction>,
}

impl BackendProfile {
    /// The default project-task label, `<label> 项目任务`.
    ///
    /// **External contract** — `acp.project_task_label`. OpenClaw overrides it
    /// with its own (`openclaw.mjs:119`).
    #[must_use]
    pub fn default_delegation_title(label: &str, locale: Locale) -> String {
        format(locale, keys::ACP_PROJECT_TASK_LABEL, &[("label", label)])
    }

    /// OpenClaw's delegation title — `backends/openclaw.mjs:119`.
    #[must_use]
    pub fn openclaw_delegation_title(locale: Locale) -> String {
        t(locale, keys::OPENCLAW_PROJECT_TASK_LABEL).to_owned()
    }

    /// The i18n key a readiness message is rendered from, when there is one.
    #[must_use]
    pub const fn readiness_key() -> Key {
        keys::OPENCLAW_GATEWAY_STARTING
    }
}

/// `encodeURIComponent(value)`.
///
/// Needed for one string and one string only: the OpenClaw coordinator
/// `sessionKey`, whose owner segment is percent-encoded inside a colon-delimited
/// key (`json-field/OpenClaw coordinator session key`, asserted at
/// `server/test/acp-backend-adapter.test.mjs:1900` as
/// `owner%20one`). The unreserved set is JavaScript's, not the URL crate's:
/// `A-Z a-z 0-9 - _ . ! ~ * ' ( )`.
#[must_use]
pub fn encode_uri_component(value: &str) -> String {
    const UNRESERVED: &[u8] = b"-_.!~*'()";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        if byte.is_ascii_alphanumeric() || UNRESERVED.contains(byte) {
            encoded.push(*byte as char);
        } else {
            use std::fmt::Write as _;
            // Writing to a `String` cannot fail.
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_replacement_paragraphs_are_upstreams_own_words() {
        assert!(
            SessionInstructions::DeepSeek
                .text()
                .starts_with("DeepSeek Harness ACP does not expose project Session management")
        );
        assert!(SessionInstructions::Pi.text().contains("Pi's own tools"));
        assert!(
            SessionInstructions::OpenClaw
                .text()
                .contains("sessions_spawn to create work")
        );
    }

    #[test]
    fn encode_uri_component_matches_javascript() {
        assert_eq!(encode_uri_component("owner one"), "owner%20one");
        assert_eq!(encode_uri_component("user_personal"), "user_personal");
        assert_eq!(encode_uri_component("a-b.c!d~e*f'g(h)"), "a-b.c!d~e*f'g(h)");
        assert_eq!(encode_uri_component("a/b?c=d&e"), "a%2Fb%3Fc%3Dd%26e");
        assert_eq!(encode_uri_component("é"), "%C3%A9");
    }

    #[test]
    fn openclaw_replaces_both_control_sentences() {
        assert_eq!(
            ControlInstructions::SessionTools.cancel("work_1", Locale::Zh),
            "请调用 via_session_cancel 取消 delegation_id=work_1。"
        );
        assert_eq!(
            ControlInstructions::OpenClawNative.cancel("agent:x:via:y:backend", Locale::Zh),
            "请用 OpenClaw 原生 Session 工具立即停止 sessionKey=agent:x:via:y:backend 对应的第三层任务。"
        );
        assert_eq!(
            ControlInstructions::OpenClawNative.status("k", Locale::Zh),
            "请调用 OpenClaw 原生 sessions_history 查询 sessionKey=k 的真实状态和阶段结果。"
        );
    }

    #[test]
    fn the_default_delegation_title_interpolates_the_backend_label() {
        assert_eq!(
            BackendProfile::default_delegation_title("Codex", Locale::Zh),
            "Codex 项目任务"
        );
        assert_eq!(
            BackendProfile::openclaw_delegation_title(Locale::Zh),
            "OpenClaw 项目任务"
        );
    }
}
