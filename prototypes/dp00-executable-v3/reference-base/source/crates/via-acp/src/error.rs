//! Everything the ACP client refuses to do, and the sentence a person sees.
//!
//! Ported from `server/src/agent/backend-adapter.mjs` (`AgentError`) and the
//! error sites in `server/src/agent/acp-process-client.mjs`. Upstream throws a
//! single `AgentError` class carrying an already-interpolated Chinese sentence
//! plus `{status, body, protocol}`; per `docs/deviations/phase-1.md` the VIA
//! convention is a `thiserror` enum whose `Display` is a structural English
//! string, with the machine-readable [`AcpError::code`] beside it and the
//! user-facing sentence rendered from `via-i18n` at the surface that shows it.
//!
//! The three `AgentError` fields survive as accessors, because all three are
//! externally observable: [`AcpError::status`] becomes the HTTP status the
//! Gateway answers with (409 for a concurrent prompt is a catalogued
//! contract), [`AcpError::body`] carries the backend's own diagnostic, and
//! [`AcpError::protocol`] is echoed on `/api/health` and in error payloads.

use std::fmt;

use via_i18n::{Locale, format, keys};

/// The `protocol` field every error this crate raises carries.
///
/// **External contract** — `AgentError(..., { protocol: 'acp' })` throughout
/// `acp-process-client.mjs`; the value reaches API clients.
pub const PROTOCOL: &str = "acp";

/// HTTP 409, the one non-zero status upstream's ACP client sets.
///
/// **External contract** — *"concurrent prompt on one session"*,
/// `acp-process-client.mjs:442-447`.
pub const STATUS_CONFLICT: u16 = 409;

/// The POSIX error name a spawn failure preserves.
///
/// Upstream copies `error.code` and `error.cause` off the Node spawn error so
/// `ENOENT` survives to the UI, and `acp-process-client.test.mjs:67-71` asserts
/// exactly that. Rust's [`std::io::ErrorKind`] is the same information under a
/// different name; [`SpawnErrno::from_io`] translates the two cases that drive
/// user-visible behaviour and reports everything else as
/// [`SpawnErrno::Other`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnErrno {
    /// The executable was not found — "the CLI is not installed" in the UI.
    NoEnt,
    /// The executable was found but is not executable by this user.
    Access,
    /// Anything else; the message still carries the operating system's text.
    Other,
}

impl SpawnErrno {
    /// Classify a spawn failure.
    #[must_use]
    pub fn from_io(error: &std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Self::NoEnt,
            std::io::ErrorKind::PermissionDenied => Self::Access,
            _ => Self::Other,
        }
    }

    /// The POSIX name, as upstream's `error.code` spells it.
    ///
    /// `None` for [`Self::Other`], because inventing a name for an
    /// unclassified failure would be worse than admitting there isn't one.
    #[must_use]
    pub fn as_str(self) -> Option<&'static str> {
        match self {
            Self::NoEnt => Some("ENOENT"),
            Self::Access => Some("EACCES"),
            Self::Other => None,
        }
    }
}

impl fmt::Display for SpawnErrno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str().unwrap_or("UNKNOWN"))
    }
}

/// Anything `via-acp` refuses to do.
///
/// Cloneable on purpose: one failure is reported to the caller, logged, and —
/// for a prompt — also delivered as the abort reason to whatever was waiting
/// on the cancellation signal.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AcpError {
    /// The child process could not be spawned.
    ///
    /// Contract — *"process lifecycle errors"*,
    /// `acp-process-client.mjs:188-195`: `进程启动失败（{message}）` with the
    /// spawn error's `code` and `cause` preserved.
    #[error("{label} ACP process failed to start ({detail})")]
    ProcessSpawnFailed {
        /// The backend's display label.
        label: String,
        /// The operating system's own message.
        detail: String,
        /// The classified errno, so "not installed" stays distinguishable.
        errno: SpawnErrno,
    },

    /// The child exited before or during the session.
    ///
    /// Contract — `进程意外退出（{signal || code || 'unknown'}）`,
    /// `acp-process-client.mjs:201-205`.
    #[error("{label} ACP process exited unexpectedly ({code})")]
    ProcessExited {
        /// The backend's display label.
        label: String,
        /// The signal name, else the exit code, else `unknown`.
        code: String,
        /// The bounded stderr tail, appended to the message when non-empty.
        stderr: String,
    },

    /// `initialize` failed, and the child wrote something useful to stderr.
    ///
    /// Contract — `初始化失败` plus stderr, `acp-process-client.mjs:261`. The
    /// whole point of this variant is that stdout may close a tick before the
    /// child's `exit`, in which case the SDK reports only "connection closed"
    /// while the backend already explained itself on stderr.
    #[error("{label} ACP initialization failed")]
    InitializeFailed {
        /// The backend's display label.
        label: String,
        /// The bounded stderr tail.
        stderr: String,
    },

    /// The agent answered `initialize` with a protocol version we do not speak.
    ///
    /// Contract — `协议版本不兼容（Agent={x}，Client={y}）`,
    /// `acp-process-client.mjs:236-242`. The client closes the connection.
    #[error("{label} ACP protocol version mismatch (agent={agent}, client={client})")]
    ProtocolVersionMismatch {
        /// The backend's display label.
        label: String,
        /// What the agent said.
        agent: String,
        /// What VIA speaks.
        client: String,
    },

    /// A request was attempted after the connection had gone away.
    #[error("the {label} ACP process has exited")]
    ProcessGone {
        /// The backend's display label.
        label: String,
    },

    /// A JSON-RPC request failed.
    ///
    /// Contract — *"ACP request failure wrapper"*,
    /// `acp-process-client.mjs:311-338`. `detail` is the already-composed
    /// `requestErrorMessage`, `stderr` is empty for a JSON-RPC error response
    /// and populated for a transport failure, and `body` is
    /// `stderr || requestErrorDetails(error)`.
    #[error("{label} ACP {method} failed: {detail}")]
    RequestFailed {
        /// The backend's display label.
        label: String,
        /// The JSON-RPC method that failed.
        method: String,
        /// The composed failure detail.
        detail: String,
        /// The bounded stderr tail, or empty.
        stderr: String,
        /// `AgentError.body`.
        body: String,
    },

    /// A second prompt was submitted for a session that already has one.
    ///
    /// Contract — *"concurrent prompt on one session"*, status
    /// [`STATUS_CONFLICT`]. This is what enforces one in-flight prompt per ACP
    /// session, and it surfaces through the Gateway API as HTTP 409.
    #[error("{label} session {id} already has a request in flight")]
    SessionBusy {
        /// The backend's display label.
        label: String,
        /// The ACP session id.
        id: String,
    },

    /// The agent declares neither `session/resume` nor `session/load`.
    ///
    /// Contract — `{label} ACP 不支持继续已有 Session`,
    /// `acp-process-client.mjs:400-403`.
    #[error("{label} ACP does not support continuing an existing session")]
    ResumeUnsupported {
        /// The backend's display label.
        label: String,
    },

    /// The prompt turn was cancelled.
    ///
    /// Contract — `session/prompt` returning `stopReason: "cancelled"` is a
    /// **hard error**, not a normal return: the client throws
    /// `combined.reason || Error('ACP Session 已取消')`. A port that returned
    /// `Ok` here would silently complete cancelled Work.
    #[error("the ACP session was cancelled")]
    Cancelled {
        /// The abort signal's own reason, when it had one.
        reason: Option<String>,
    },

    /// The prompt was an empty `ContentBlock` array.
    ///
    /// Contract — `ACP Prompt 不能为空`, `acp-content.mjs:81`.
    #[error("an ACP prompt cannot be empty")]
    PromptEmpty,

    /// A prompt block was not an object with a `type`.
    ///
    /// Contract — `ACP Prompt 包含无效的 ContentBlock`, `acp-content.mjs:84`.
    #[error("the ACP prompt contains an invalid ContentBlock")]
    PromptInvalidBlock,

    /// An image block was offered to an agent that did not declare
    /// `promptCapabilities.image`.
    #[error("the current backend agent does not declare ACP image input")]
    ImageInputUndeclared,

    /// An audio block was offered to an agent that did not declare
    /// `promptCapabilities.audio`.
    #[error("the current backend agent does not declare ACP audio input")]
    AudioInputUndeclared,

    /// An embedded resource was offered to an agent that did not declare
    /// `promptCapabilities.embeddedContext`.
    #[error("the current backend agent does not declare ACP embedded files")]
    EmbeddedFileUndeclared,

    /// The factory was handed a connection kind it does not implement.
    ///
    /// Contract — `不支持的 ACP 连接方式：{kind}`,
    /// `acp-client-factory.mjs:18-21`. An unconfigured connection reports the
    /// empty kind, which renders as the localized "not configured" token.
    #[error("unsupported ACP connection kind: {kind}")]
    UnsupportedConnection {
        /// The requested kind, trimmed and lowercased; empty when unset.
        kind: String,
    },
}

impl AcpError {
    /// A stable machine-readable code.
    ///
    /// VIA's own namespace: upstream has no code on `AgentError` at all, only
    /// a `status`, so nothing here is inherited and nothing may be mistaken
    /// for a contract a client already knows.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::ProcessSpawnFailed { .. } => "VIA_ACP_PROCESS_SPAWN_FAILED",
            Self::ProcessExited { .. } => "VIA_ACP_PROCESS_EXITED",
            Self::InitializeFailed { .. } => "VIA_ACP_INITIALIZE_FAILED",
            Self::ProtocolVersionMismatch { .. } => "VIA_ACP_PROTOCOL_VERSION_MISMATCH",
            Self::ProcessGone { .. } => "VIA_ACP_PROCESS_GONE",
            Self::RequestFailed { .. } => "VIA_ACP_REQUEST_FAILED",
            Self::SessionBusy { .. } => "VIA_ACP_SESSION_BUSY",
            Self::ResumeUnsupported { .. } => "VIA_ACP_RESUME_UNSUPPORTED",
            Self::Cancelled { .. } => "VIA_ACP_SESSION_CANCELLED",
            Self::PromptEmpty => "VIA_ACP_PROMPT_EMPTY",
            Self::PromptInvalidBlock => "VIA_ACP_PROMPT_INVALID_BLOCK",
            Self::ImageInputUndeclared => "VIA_ACP_IMAGE_INPUT_UNDECLARED",
            Self::AudioInputUndeclared => "VIA_ACP_AUDIO_INPUT_UNDECLARED",
            Self::EmbeddedFileUndeclared => "VIA_ACP_EMBEDDED_FILE_UNDECLARED",
            Self::UnsupportedConnection { .. } => "VIA_ACP_UNSUPPORTED_CONNECTION",
        }
    }

    /// `AgentError.status` — 0 for everything but a concurrent prompt.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            Self::SessionBusy { .. } => STATUS_CONFLICT,
            _ => 0,
        }
    }

    /// `AgentError.body` — the backend's own diagnostic, when there is one.
    #[must_use]
    pub fn body(&self) -> &str {
        match self {
            Self::RequestFailed { body, .. } => body,
            Self::ProcessExited { stderr, .. } | Self::InitializeFailed { stderr, .. } => stderr,
            _ => "",
        }
    }

    /// `AgentError.protocol` — always [`PROTOCOL`] for this crate.
    #[must_use]
    pub fn protocol(&self) -> &'static str {
        PROTOCOL
    }

    /// The POSIX spawn errno, for the one variant that has one.
    ///
    /// This is upstream's `error.code` / `error.cause.code` pair collapsed into
    /// one accessor: "is the CLI missing?" is the only question anyone asks of
    /// it, and it drives the install UX.
    #[must_use]
    pub fn spawn_errno(&self) -> Option<SpawnErrno> {
        match self {
            Self::ProcessSpawnFailed { errno, .. } => Some(*errno),
            _ => None,
        }
    }

    /// The sentence a person reads, in their locale.
    ///
    /// Every string comes from `via-i18n`; there is no literal here, including
    /// the separator between a message and a stderr tail, which is fullwidth in
    /// `zh` and an ASCII colon in `en`/`ko`.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::ProcessSpawnFailed { label, detail, .. } => Self::process_error(
                locale,
                label,
                &format(
                    locale,
                    keys::ACP_PROCESS_SPAWN_FAILED,
                    &[("detail", detail)],
                ),
                "",
            ),
            Self::ProcessExited {
                label,
                code,
                stderr,
            } => Self::process_error(
                locale,
                label,
                &format(locale, keys::ACP_PROCESS_EXITED, &[("code", code)]),
                stderr,
            ),
            Self::InitializeFailed { label, stderr } => Self::process_error(
                locale,
                label,
                via_i18n::t(locale, keys::ACP_INITIALIZE_FAILED),
                stderr,
            ),
            Self::ProtocolVersionMismatch {
                label,
                agent,
                client,
            } => Self::process_error(
                locale,
                label,
                &format(
                    locale,
                    keys::ACP_PROTOCOL_VERSION_MISMATCH,
                    &[("agent", agent), ("client", client)],
                ),
                "",
            ),
            Self::ProcessGone { label } => {
                format(locale, keys::ACP_PROCESS_EXITED_LABEL, &[("label", label)])
            }
            Self::RequestFailed {
                label,
                method,
                detail,
                stderr,
                ..
            } => {
                if stderr.is_empty() {
                    format(
                        locale,
                        keys::ACP_REQUEST_FAILED,
                        &[("label", label), ("method", method), ("detail", detail)],
                    )
                } else {
                    format(
                        locale,
                        keys::ACP_REQUEST_FAILED_WITH_STDERR,
                        &[
                            ("label", label),
                            ("method", method),
                            ("detail", detail),
                            ("stderr", stderr),
                        ],
                    )
                }
            }
            Self::SessionBusy { label, id } => format(
                locale,
                keys::ACP_SESSION_BUSY,
                &[("label", label), ("id", id)],
            ),
            Self::ResumeUnsupported { label } => {
                format(locale, keys::ACP_RESUME_UNSUPPORTED, &[("label", label)])
            }
            Self::Cancelled { reason } => reason
                .clone()
                .unwrap_or_else(|| via_i18n::t(locale, keys::ACP_SESSION_CANCELLED).to_owned()),
            Self::PromptEmpty => via_i18n::t(locale, keys::ACP_PROMPT_EMPTY).to_owned(),
            Self::PromptInvalidBlock => {
                via_i18n::t(locale, keys::ACP_PROMPT_INVALID_BLOCK).to_owned()
            }
            Self::ImageInputUndeclared => {
                via_i18n::t(locale, keys::ACP_IMAGE_INPUT_UNDECLARED).to_owned()
            }
            Self::AudioInputUndeclared => {
                via_i18n::t(locale, keys::ACP_AUDIO_INPUT_UNDECLARED).to_owned()
            }
            Self::EmbeddedFileUndeclared => {
                via_i18n::t(locale, keys::ACP_EMBEDDED_FILE_UNDECLARED).to_owned()
            }
            Self::UnsupportedConnection { kind } => {
                // Upstream renders `kind || '未配置'`; the fallback token is a
                // catalog key, not a literal.
                let kind = if kind.is_empty() {
                    via_i18n::t(locale, keys::GATEWAY_NOT_CONFIGURED)
                } else {
                    kind
                };
                format(locale, keys::ACP_UNSUPPORTED_CONNECTION, &[("kind", kind)])
            }
        }
    }

    /// `` `${label} ACP ${message}${stderr ? `：${stderr}` : ''}` `` —
    /// `acp-process-client.mjs:56-61`.
    fn process_error(locale: Locale, label: &str, detail: &str, stderr: &str) -> String {
        if stderr.is_empty() {
            format(
                locale,
                keys::ACP_PROCESS_ERROR,
                &[("label", label), ("detail", detail)],
            )
        } else {
            format(
                locale,
                keys::ACP_PROCESS_ERROR_WITH_STDERR,
                &[("label", label), ("detail", detail), ("stderr", stderr)],
            )
        }
    }
}

/// The result type every fallible entry point in this crate returns.
pub type Result<T> = std::result::Result<T, AcpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_concurrent_prompt_carries_a_status() {
        let busy = AcpError::SessionBusy {
            label: "Example".into(),
            id: "s-1".into(),
        };
        assert_eq!(busy.status(), 409);
        assert_eq!(
            AcpError::PromptEmpty.status(),
            0,
            "every other failure is a plain agent error"
        );
    }

    #[test]
    fn every_error_reports_the_acp_protocol() {
        assert_eq!(AcpError::PromptEmpty.protocol(), "acp");
    }

    #[test]
    fn spawn_errno_translates_the_two_cases_the_ui_depends_on() {
        let missing = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert_eq!(SpawnErrno::from_io(&missing).as_str(), Some("ENOENT"));
        let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert_eq!(SpawnErrno::from_io(&denied).as_str(), Some("EACCES"));
        let other = std::io::Error::from(std::io::ErrorKind::BrokenPipe);
        assert_eq!(SpawnErrno::from_io(&other).as_str(), None);
    }
}
