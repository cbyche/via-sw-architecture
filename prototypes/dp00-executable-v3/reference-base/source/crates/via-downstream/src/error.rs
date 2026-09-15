//! Everything Layer 3 refuses to do.
//!
//! Upstream carries two error shapes across this boundary: bare `Error`s with
//! interpolated Chinese sentences thrown by the driver registry
//! (`server/src/agent/backends/registry.mjs:22-63`), and `AgentError`
//! (`server/src/agent/backend-adapter.mjs:1-9`) which adds `{status, body,
//! protocol}` for anything a running harness reports.
//!
//! Per `docs/fidelity.md` — *"ad-hoc `{ ok, error, code }` returns → `thiserror`
//! enums with a `code()` accessor"* — the discriminant and the interpolation
//! values live here and the sentence lives in `via-i18n`. [`Self::message`]
//! renders it. The `Display` impls are developer-facing English and are not the
//! string an operator reads.
//!
//! Every refusal upstream words for the *agent* driver registry has a variant:
//!
//! | Upstream | Variant | Key |
//! | --- | --- | --- |
//! | `后台 Driver 未在目录注册：<id>` | [`Self::DriverNotRegistered`] | `backend.driver_not_registered` |
//! | `后台 Driver 标签不一致：<id>` | [`Self::DriverLabelMismatch`] | `backend.driver_label_mismatch` |
//! | `后台 Driver 能力声明不完整：<id>` | [`Self::IncompleteCapabilities`] | `backend.driver_incomplete_capabilities` |
//! | `后台 Driver 返回了无效 Profile：<id>` | [`Self::InvalidSession`] | `backend.driver_invalid_profile` |
//! | `不支持的后台 Agent：<id>` | [`Self::UnsupportedBackend`] | `backend.unsupported_agent` |
//!
//! Two of the catalogued ten have **no runtime analogue in Rust** and are
//! recorded in `docs/deviations/phase-2.md` rather than reproduced:
//! `后台 Driver 缺少 createProfile` — [`DownstreamAgent`](crate::DownstreamAgent)
//! cannot be implemented without `open`, so a driver missing it does not
//! compile — and the two `skills` refusals, which
//! [`via_catalog::SkillsSpec`] already makes unrepresentable. The remaining
//! five are the *runtime* driver's (`server/src/process/backend-drivers/registry.mjs`)
//! and belong to `via-process`.

use via_core::CoreError;
use via_i18n::{Locale, format, keys, t};

use crate::capability::CapabilityFault;

/// The HTTP status upstream attaches to a busy session.
///
/// `server/src/agent/acp-process-client.mjs:442-447` throws `AgentError` with
/// `{ status: 409 }` when a second prompt reaches a session that already has
/// one in flight.
pub const SESSION_BUSY_STATUS: u16 = 409;

/// Anything a harness, or the registry in front of it, refuses.
///
/// Deliberately **not** `#[non_exhaustive]`, matching
/// [`via_core::CoreError`] and [`via_catalog::CatalogError`]: a crate above
/// this seam should be made to recompile when a new refusal appears, rather
/// than fold it into a wildcard arm that already existed.
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    /// No backend is configured at all.
    ///
    /// Upstream's frontend-only mode (`AGENT_PROTOCOL=''`), where
    /// `server/src/agent/agent-client.mjs:164-175` answers with a disabled
    /// descriptor instead of a driver. `docs/architecture.md` §2: `agent` mode
    /// degrades to `direct` here rather than failing, and the degradation is
    /// reported on `/api/health`.
    #[error("no backend agent is configured")]
    NotConfigured,

    /// A backend id that no registered harness answers to.
    ///
    /// Upstream `backendDriver()`, `server/src/agent/backends/registry.mjs:58`.
    #[error("unsupported backend agent: {id}")]
    UnsupportedBackend {
        /// The id as requested, trimmed and lower-cased.
        id: String,
    },

    /// A harness declared an id the backend catalog has never heard of.
    ///
    /// Upstream `validateBackendDriver()`,
    /// `server/src/agent/backends/registry.mjs:24`.
    #[error("backend driver is not registered in the catalog: {id}")]
    DriverNotRegistered {
        /// The id the descriptor declared.
        id: String,
    },

    /// A harness's label is not the label the catalog gives that backend.
    ///
    /// Upstream `server/src/agent/backends/registry.mjs:28-30`. The label
    /// reaches the operator inside a dozen other messages, so two spellings of
    /// one backend is a real defect rather than a cosmetic one.
    #[error(
        "backend driver label does not match for {id}: declared {declared}, catalog {expected}"
    )]
    DriverLabelMismatch {
        /// The backend id.
        id: String,
        /// What the descriptor said.
        declared: String,
        /// What `via-catalog` says.
        expected: &'static str,
    },

    /// A capability declaration contradicts itself or the catalog.
    ///
    /// Upstream `server/src/agent/backends/registry.mjs:32-43`. See
    /// [`CapabilityFault`] for the individual rules.
    #[error("backend driver capability declaration is incomplete for {id}: {}", render_faults(.faults))]
    IncompleteCapabilities {
        /// The backend id.
        id: String,
        /// Every rule that failed, in a stable order.
        faults: Vec<CapabilityFault>,
    },

    /// A harness opened a session that is not usable.
    ///
    /// Upstream's shape check on what `createProfile` returned,
    /// `server/src/agent/backends/registry.mjs:76-78`: `!profile ||
    /// typeof profile !== 'object' || Array.isArray(profile)`. In Rust the type
    /// carries all of that; what it cannot carry is a session that reports an
    /// empty [`session_id`](crate::HarnessSession::session_id), which is the
    /// one way a `Box<dyn HarnessSession>` can still be unusable.
    #[error("backend driver returned an invalid session for {id}")]
    InvalidSession {
        /// The backend id.
        id: String,
    },

    /// A second prompt reached a session that already has one in flight.
    ///
    /// Upstream `server/src/agent/acp-process-client.mjs:442-447`, HTTP
    /// [`SESSION_BUSY_STATUS`]. `docs/architecture.md` §11: one item per owner
    /// inside the backend session at a time — this is what that invariant
    /// looks like when something races it anyway.
    #[error("{label} session {session_id} already has a request in flight")]
    SessionBusy {
        /// The harness label.
        label: String,
        /// The session id that is busy.
        session_id: String,
    },

    /// The turn ended in cancellation.
    ///
    /// Upstream `server/src/agent/acp-process-client.mjs:500-502` throws rather
    /// than returning when `stopReason === 'cancelled'`, and
    /// `docs/reference/contracts.json` spells out why: *"a Rust port that
    /// returns `Ok` on `'cancelled'` would silently complete cancelled Work."*
    /// [`PromptOutcome::new`](crate::PromptOutcome::new) is where that is
    /// enforced.
    #[error("the session was cancelled")]
    Cancelled,

    /// The harness cannot cancel Layer-3 work.
    ///
    /// Upstream `via_i18n::keys::AGENT_CANCEL_UNSUPPORTED`; reached when
    /// [`BackendCapabilities::delegation`](crate::BackendCapabilities::delegation)
    /// is false and a cancel arrives anyway.
    #[error("the current backend agent does not support cancelling a layer 3 session")]
    CancelUnsupported,

    /// There was no cancellable project task under that id for that owner.
    ///
    /// Upstream `server/src/agent/acp-backend-adapter.mjs:1376-1379`.
    #[error("no cancellable {label} project task was found")]
    NotCancellable {
        /// The harness label.
        label: String,
    },

    /// A delegation id that names nothing.
    ///
    /// Upstream `via_i18n::keys::ACP_DELEGATION_NOT_FOUND`.
    #[error("no matching {label} project task was found")]
    DelegationNotFound {
        /// The harness label.
        label: String,
    },

    /// Anything a running harness reports, with the transport's own detail.
    ///
    /// This is upstream's `AgentError`
    /// (`server/src/agent/backend-adapter.mjs:1-9`) lifted intact: `message`
    /// has already been localized by the driver that raised it, `status` is an
    /// HTTP status or `0`, `body` is the transport's raw payload and `protocol`
    /// is the backend id. It exists so that a driver has one place to put a
    /// failure this enum does not name, instead of each driver growing an
    /// error type the seam cannot see through.
    #[error("{message}")]
    Agent {
        /// The localized sentence.
        message: String,
        /// An HTTP status, or `0` when the transport had none.
        status: u16,
        /// The transport's raw response body, if any.
        body: String,
        /// The backend id that raised it.
        protocol: String,
    },

    /// Configuration was wrong in a way `via-core` already words.
    ///
    /// A harness reaches this while opening: the generic ACP backend with no
    /// `ACP_COMMAND`, a backend with no workspace, an unsupported permission
    /// mode.
    #[error(transparent)]
    Configuration(#[from] CoreError),
}

/// Renders a fault list for a `Display` impl.
fn render_faults(faults: &[CapabilityFault]) -> String {
    if faults.is_empty() {
        return "no fault recorded".to_owned();
    }
    faults
        .iter()
        .map(|fault| fault.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

impl HarnessError {
    /// A stable machine-readable code.
    ///
    /// VIA's own — upstream throws untyped `Error`s for every one of these
    /// except the `AgentError` family, which carries a status rather than a
    /// code. [`Self::Configuration`] delegates to `via-core` so a configuration
    /// failure keeps the code it already had.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured => "VIA_BACKEND_NOT_CONFIGURED",
            Self::UnsupportedBackend { .. } => "VIA_BACKEND_UNSUPPORTED",
            Self::DriverNotRegistered { .. } => "VIA_BACKEND_DRIVER_NOT_REGISTERED",
            Self::DriverLabelMismatch { .. } => "VIA_BACKEND_DRIVER_LABEL_MISMATCH",
            Self::IncompleteCapabilities { .. } => "VIA_BACKEND_DRIVER_CAPABILITIES_INCOMPLETE",
            Self::InvalidSession { .. } => "VIA_BACKEND_DRIVER_INVALID_PROFILE",
            Self::SessionBusy { .. } => "VIA_HARNESS_SESSION_BUSY",
            Self::Cancelled => "VIA_HARNESS_CANCELLED",
            Self::CancelUnsupported => "VIA_HARNESS_CANCEL_UNSUPPORTED",
            Self::NotCancellable { .. } => "VIA_HARNESS_NOT_CANCELLABLE",
            Self::DelegationNotFound { .. } => "VIA_HARNESS_DELEGATION_NOT_FOUND",
            Self::Agent { .. } => "VIA_HARNESS_AGENT",
            Self::Configuration(error) => error.code(),
        }
    }

    /// The sentence an operator reads, in their locale.
    ///
    /// Every string comes from `via-i18n`; there is no literal in this crate.
    /// [`Self::Agent`] and [`Self::Configuration`] carry a sentence that was
    /// already rendered by whoever raised them, and are passed through.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::NotConfigured => t(locale, keys::AGENT_NOT_CONFIGURED).to_owned(),
            Self::UnsupportedBackend { id } => {
                format(locale, keys::BACKEND_UNSUPPORTED_AGENT, &[("id", id)])
            }
            Self::DriverNotRegistered { id } => {
                format(locale, keys::BACKEND_DRIVER_NOT_REGISTERED, &[("id", id)])
            }
            Self::DriverLabelMismatch { id, .. } => {
                format(locale, keys::BACKEND_DRIVER_LABEL_MISMATCH, &[("id", id)])
            }
            Self::IncompleteCapabilities { id, .. } => format(
                locale,
                keys::BACKEND_DRIVER_INCOMPLETE_CAPABILITIES,
                &[("id", id)],
            ),
            Self::InvalidSession { id } => {
                format(locale, keys::BACKEND_DRIVER_INVALID_PROFILE, &[("id", id)])
            }
            Self::SessionBusy { label, session_id } => format(
                locale,
                keys::ACP_SESSION_BUSY,
                &[("label", label), ("id", session_id)],
            ),
            Self::Cancelled => t(locale, keys::ACP_SESSION_CANCELLED).to_owned(),
            Self::CancelUnsupported => t(locale, keys::AGENT_CANCEL_UNSUPPORTED).to_owned(),
            Self::NotCancellable { label } => format(
                locale,
                keys::ACP_DELEGATION_NOT_CANCELLABLE,
                &[("label", label)],
            ),
            Self::DelegationNotFound { label } => {
                format(locale, keys::ACP_DELEGATION_NOT_FOUND, &[("label", label)])
            }
            Self::Agent { message, .. } => message.clone(),
            Self::Configuration(error) => error.to_string(),
        }
    }

    /// The HTTP status this failure carries, or `0` when it has none.
    ///
    /// Upstream's `AgentError` defaults `status` to `0`
    /// (`server/src/agent/backend-adapter.mjs:3`); only the busy-session refusal
    /// and whatever a transport reports carry a real one.
    #[must_use]
    pub fn http_status(&self) -> u16 {
        match self {
            Self::SessionBusy { .. } => SESSION_BUSY_STATUS,
            Self::Agent { status, .. } => *status,
            _ => 0,
        }
    }

    /// Whether this failure is the turn having been cancelled.
    ///
    /// Callers branch on it to record `cancelled` rather than `failed`; see
    /// [`CancelOutcome`](crate::CancelOutcome).
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_refusals_render_upstreams_chinese_verbatim() {
        let error = HarnessError::DriverNotRegistered {
            id: "ghost".to_owned(),
        };
        assert_eq!(error.message(Locale::Zh), "后台 Driver 未在目录注册：ghost");
        assert_eq!(error.code(), "VIA_BACKEND_DRIVER_NOT_REGISTERED");
    }

    #[test]
    fn a_busy_session_carries_409() {
        let error = HarnessError::SessionBusy {
            label: "Codex".to_owned(),
            session_id: "s-1".to_owned(),
        };
        assert_eq!(error.http_status(), SESSION_BUSY_STATUS);
        assert_eq!(
            error.message(Locale::Zh),
            "Codex Session s-1 已有正在执行的请求",
        );
    }

    #[test]
    fn everything_else_has_no_status() {
        assert_eq!(HarnessError::Cancelled.http_status(), 0);
        assert!(HarnessError::Cancelled.is_cancelled());
        assert!(!HarnessError::NotConfigured.is_cancelled());
    }
}
