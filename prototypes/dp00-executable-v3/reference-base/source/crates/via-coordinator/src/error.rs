//! Everything the coordination layer refuses.
//!
//! Upstream throws two kinds of thing across this boundary: `AgentError`
//! (`server/src/agent/backend-adapter.mjs`) with an already-localized Chinese
//! sentence and a `{status, protocol}` pair, and — inside `coordinator.mjs`
//! itself — five bare `Error`s carrying developer English. Per
//! `docs/fidelity.md` the discriminant and the interpolation values live here
//! and the sentence lives in `via-i18n`; [`Self::message`] renders it, and the
//! `Display` impls are developer-facing English that no operator reads.
//!
//! # Four of upstream's five bare errors are already worded
//!
//! | Upstream | Variant | Key |
//! | --- | --- | --- |
//! | `Coordinator backend is unavailable` | [`Self::NotConfigured`] | `agent.not_configured` |
//! | `Coordinator backend returned an empty response` | [`Self::EmptyResponse`] | `acp.session_returned_nothing` |
//! | `Coordinator backend cannot cancel delegated work` | [`Self::CancelUnsupported`] | `agent.cancel_unsupported` |
//! | `Coordinator backend cannot query delegated work` | [`Self::QueryUnsupported`] | `agent.query_unsupported` |
//! | `Coordinator did not return a final result (state=…)` | [`Self::NotFinalResult`] | `coordinator.not_final_result` |
//!
//! The first four reuse keys `via-acp` and `via-downstream` already ship,
//! because they are the same refusal observed one layer up: upstream has two
//! empty-response checks — one in the adapter and one in the Coordinator — and
//! only the adapter's carries the recovery predicate, so VIA keeps one variant
//! with the predicate on it. The fifth is the one key this crate adds, and it
//! is added because a Work that failed this way becomes `task.error`, is
//! persisted, and is spoken aloud.

use via_downstream::HarnessError;
use via_i18n::{Locale, format, keys, t};

use crate::executor::ExecutorStopped;

/// Anything the coordinator refuses.
///
/// Deliberately **not** `#[non_exhaustive]`, matching
/// [`via_downstream::HarnessError`] and [`via_core::CoreError`]: a crate above
/// this one should be made to recompile when a new refusal appears rather than
/// fold it into a wildcard arm that already existed.
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    /// There is no backend agent to coordinate with.
    ///
    /// Upstream `new Error('Coordinator backend is unavailable')`,
    /// `server/src/agent/coordinator.mjs:262`, reached when the client has no
    /// `runCoordinator`. `docs/architecture.md` §2: `agent` mode degrades to
    /// `direct` here rather than failing.
    #[error("no backend agent is configured")]
    NotConfigured,

    /// The coordinator session answered with nothing.
    ///
    /// Upstream `AgentError(`${label} ACP Session 未返回任何内容`, {status: 502})`,
    /// `acp-backend-adapter.mjs:1051-1060`.
    ///
    /// `recoverable` is the catalogued four-part purity predicate
    /// (`error-code` / *empty coordinator response*):
    /// `!receivedUpdate && !delegation && nativeToolCalls.size === 0 &&
    /// toolCalls.size === 0`. It decides whether the coordinator session is
    /// discarded and the turn retried **exactly once**; retrying after any
    /// observed activity would duplicate side effects.
    #[error("{label} returned no content")]
    EmptyResponse {
        /// The harness label.
        label: String,
        /// Whether the turn may be retried in a fresh coordinator session.
        recoverable: bool,
    },

    /// The reply's `state` is not one the Gateway may deliver.
    ///
    /// Upstream
    /// ``new Error(`Coordinator did not return a final result (state=${state})`)``,
    /// `coordinator.mjs:280`, reached only after the retry ladder has already
    /// asked twice.
    #[error("the coordinator did not return a final result (state={state})")]
    NotFinalResult {
        /// The state it returned instead, lower-cased, verbatim.
        state: String,
    },

    /// This backend cannot cancel a Layer-3 Session.
    #[error("the current backend agent does not support cancelling a layer 3 session")]
    CancelUnsupported,

    /// This backend cannot be asked about a Layer-3 Session.
    #[error("the current backend agent does not support querying a layer 3 session")]
    QueryUnsupported,

    /// A second delegation was attempted inside one coordination turn.
    ///
    /// Upstream `AgentError('当前协调轮次已经启动了一个第三层任务')`,
    /// `acp-backend-adapter.mjs:744-748`. One turn delegates once; the model is
    /// told so in [`via_i18n::keys::COORDINATOR_DELEGATION_NOTE`], and this is
    /// what happens when it does it anyway.
    #[error("this coordination turn has already started a layer 3 task")]
    DelegationAlreadyStarted,

    /// Nothing cancellable answers to that Work id **for that owner**.
    ///
    /// Upstream `AgentError(`没有找到可取消的 ${label} 项目任务`)`,
    /// `acp-backend-adapter.mjs:1377-1379`. The catalogue is explicit about the
    /// second half: *"a delegation belonging to another owner is reported as
    /// not-found, never as forbidden."*
    #[error("no cancellable {label} project task was found")]
    NotCancellable {
        /// The harness label.
        label: String,
    },

    /// Nothing answers to that Work id for that owner.
    ///
    /// Upstream `AgentError(`没有找到对应的 ${label} 项目任务`)`,
    /// `acp-backend-adapter.mjs:1429-1431`. Owner-scoped for the same reason.
    #[error("no matching {label} project task was found")]
    DelegationNotFound {
        /// The harness label.
        label: String,
    },

    /// A persisted delegation cannot be reattached to.
    ///
    /// Upstream `AgentError(`${label} 无法恢复这项第三层任务`)`,
    /// `acp-backend-adapter.mjs:1286-1288`.
    #[error("{label} cannot recover this layer 3 task")]
    NotRecoverable {
        /// The harness label.
        label: String,
    },

    /// A Session was named for continuation but its directory is unknown.
    ///
    /// Upstream
    /// `AgentError(`${label} Session 的项目目录未知，请先查询 Session 列表后再继续`)`,
    /// `acp-backend-adapter.mjs:856-859`.
    #[error("the project directory of this {label} session is unknown")]
    SessionDirectoryUnknown {
        /// The harness label.
        label: String,
    },

    /// A permission decision named a request that is not this owner's.
    ///
    /// Upstream
    /// `AgentError('权限请求不存在、已经失效或不属于当前用户')`,
    /// `permission-broker.mjs:108-110`, which the HTTP route maps to 404. All
    /// three causes share one sentence deliberately: telling a caller *which*
    /// of the three it was would let them enumerate another owner's pending
    /// permissions.
    #[error("that permission request does not exist, has expired, or is not this owner's")]
    PermissionRequestUnknown,

    /// Whatever the harness itself refused.
    #[error(transparent)]
    Harness(#[from] HarnessError),

    /// An owning task this crate depends on has shut down.
    #[error(transparent)]
    Stopped(#[from] ExecutorStopped),
}

impl CoordinatorError {
    /// A stable machine-readable code.
    ///
    /// VIA's own for everything the coordinator raises; [`Self::Harness`] and
    /// [`Self::Stopped`] delegate, so a refusal keeps the code it already had.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured => "VIA_BACKEND_NOT_CONFIGURED",
            Self::EmptyResponse { .. } => "VIA_COORDINATOR_EMPTY_RESPONSE",
            Self::NotFinalResult { .. } => "VIA_COORDINATOR_NOT_FINAL_RESULT",
            Self::CancelUnsupported => "VIA_HARNESS_CANCEL_UNSUPPORTED",
            Self::QueryUnsupported => "VIA_COORDINATOR_QUERY_UNSUPPORTED",
            Self::DelegationAlreadyStarted => "VIA_COORDINATOR_DELEGATION_ALREADY_STARTED",
            Self::NotCancellable { .. } => "VIA_HARNESS_NOT_CANCELLABLE",
            Self::DelegationNotFound { .. } => "VIA_HARNESS_DELEGATION_NOT_FOUND",
            Self::NotRecoverable { .. } => "VIA_COORDINATOR_DELEGATION_NOT_RECOVERABLE",
            Self::SessionDirectoryUnknown { .. } => "VIA_COORDINATOR_SESSION_DIRECTORY_UNKNOWN",
            Self::PermissionRequestUnknown => "VIA_COORDINATOR_PERMISSION_UNKNOWN",
            Self::Harness(error) => error.code(),
            Self::Stopped(error) => error.code(),
        }
    }

    /// The sentence an operator — or the user, through a spoken failure —
    /// reads.
    ///
    /// Every string comes from `via-i18n`; there is no user-facing literal in
    /// this crate.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        match self {
            Self::NotConfigured => t(locale, keys::AGENT_NOT_CONFIGURED).to_owned(),
            Self::EmptyResponse { label, .. } => format(
                locale,
                keys::ACP_SESSION_RETURNED_NOTHING,
                &[("label", label)],
            ),
            Self::NotFinalResult { state } => format(
                locale,
                keys::COORDINATOR_NOT_FINAL_RESULT,
                &[("state", state)],
            ),
            Self::CancelUnsupported => t(locale, keys::AGENT_CANCEL_UNSUPPORTED).to_owned(),
            Self::QueryUnsupported => t(locale, keys::AGENT_QUERY_UNSUPPORTED).to_owned(),
            Self::DelegationAlreadyStarted => {
                t(locale, keys::ACP_DELEGATION_ALREADY_STARTED).to_owned()
            }
            Self::NotCancellable { label } => format(
                locale,
                keys::ACP_DELEGATION_NOT_CANCELLABLE,
                &[("label", label)],
            ),
            Self::DelegationNotFound { label } => {
                format(locale, keys::ACP_DELEGATION_NOT_FOUND, &[("label", label)])
            }
            Self::NotRecoverable { label } => format(
                locale,
                keys::ACP_DELEGATION_NOT_RECOVERABLE,
                &[("label", label)],
            ),
            Self::SessionDirectoryUnknown { label } => format(
                locale,
                keys::ACP_SESSION_DIRECTORY_UNKNOWN,
                &[("label", label)],
            ),
            Self::PermissionRequestUnknown => {
                t(locale, keys::GATEWAY_PERMISSION_REQUEST_UNKNOWN).to_owned()
            }
            Self::Harness(error) => error.message(locale),
            Self::Stopped(error) => error.to_string(),
        }
    }

    /// The HTTP status this failure carries, or `0` when it has none.
    ///
    /// Upstream attaches `502` to the empty-response `AgentError`
    /// (`acp-backend-adapter.mjs:1053`) and nothing to the rest;
    /// [`Self::Harness`] keeps whatever the harness reported.
    #[must_use]
    pub fn http_status(&self) -> u16 {
        match self {
            Self::EmptyResponse { .. } => EMPTY_RESPONSE_STATUS,
            Self::Harness(error) => error.http_status(),
            _ => 0,
        }
    }

    /// Whether this turn may be retried in a **fresh** coordinator session.
    ///
    /// True only for an [`Self::EmptyResponse`] whose four-part purity
    /// predicate held. Every other failure — including an empty response after
    /// any observed activity — is final for the turn.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::EmptyResponse {
                recoverable: true,
                ..
            }
        )
    }

    /// Whether this failure is the turn having been cancelled.
    ///
    /// Delegates to [`HarnessError::is_cancelled`]; callers branch on it to
    /// record `cancelled` rather than `failed`.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Harness(error) if error.is_cancelled())
    }
}

/// The HTTP status upstream attaches to an empty coordinator response.
///
/// **External contract** — `acp-backend-adapter.mjs:1053`.
pub const EMPTY_RESPONSE_STATUS: u16 = 502;

impl From<CoordinatorError> for via_work::RunFailure {
    /// A coordination refusal is a Work failure whose message is the
    /// coordinator's.
    ///
    /// `via_work::RunFailure` carries an **already-localized** sentence, so the
    /// conversion needs a locale; this one uses the developer-facing `Display`
    /// so that a caller who forgot to localize gets something diagnosable
    /// rather than a Chinese sentence in an English deployment. Prefer
    /// [`CoordinatorError::into_failure`], which takes the locale.
    fn from(error: CoordinatorError) -> Self {
        Self::new(error.to_string())
    }
}

impl CoordinatorError {
    /// This refusal as a Work failure, in `locale`.
    #[must_use]
    pub fn into_failure(self, locale: Locale) -> via_work::RunFailure {
        via_work::RunFailure::new(self.message(locale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_refusal_renders_the_catalogued_chinese() {
        assert_eq!(
            CoordinatorError::DelegationAlreadyStarted.message(Locale::Zh),
            "当前协调轮次已经启动了一个第三层任务",
        );
        assert_eq!(
            CoordinatorError::NotCancellable {
                label: "OpenCode".to_owned(),
            }
            .message(Locale::Zh),
            "没有找到可取消的 OpenCode 项目任务",
        );
        assert_eq!(
            CoordinatorError::DelegationNotFound {
                label: "OpenCode".to_owned(),
            }
            .message(Locale::Zh),
            "没有找到对应的 OpenCode 项目任务",
        );
        assert_eq!(
            CoordinatorError::NotRecoverable {
                label: "OpenClaw".to_owned(),
            }
            .message(Locale::Zh),
            "OpenClaw 无法恢复这项第三层任务",
        );
        assert_eq!(
            CoordinatorError::SessionDirectoryUnknown {
                label: "Qoder".to_owned(),
            }
            .message(Locale::Zh),
            "Qoder Session 的项目目录未知，请先查询 Session 列表后再继续",
        );
        assert_eq!(
            CoordinatorError::PermissionRequestUnknown.message(Locale::Zh),
            "权限请求不存在、已经失效或不属于当前用户",
        );
        assert_eq!(
            CoordinatorError::EmptyResponse {
                label: "OpenClaw".to_owned(),
                recoverable: true,
            }
            .message(Locale::Zh),
            "OpenClaw ACP Session 未返回任何内容",
        );
    }

    #[test]
    fn the_not_final_result_message_names_the_state_in_every_locale() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let rendered = CoordinatorError::NotFinalResult {
                state: "delegated".to_owned(),
            }
            .message(locale);
            assert!(rendered.contains("state=delegated"), "{rendered}");
            assert!(!rendered.contains("<via-i18n:"), "{rendered}");
        }
    }

    #[test]
    fn only_a_pure_empty_response_is_recoverable() {
        assert!(
            CoordinatorError::EmptyResponse {
                label: "L".to_owned(),
                recoverable: true,
            }
            .is_recoverable()
        );
        assert!(
            !CoordinatorError::EmptyResponse {
                label: "L".to_owned(),
                recoverable: false,
            }
            .is_recoverable()
        );
        assert!(!CoordinatorError::NotConfigured.is_recoverable());
        assert!(
            !CoordinatorError::NotFinalResult {
                state: "active".to_owned(),
            }
            .is_recoverable()
        );
    }

    #[test]
    fn only_the_empty_response_carries_a_status_of_its_own() {
        assert_eq!(
            CoordinatorError::EmptyResponse {
                label: "L".to_owned(),
                recoverable: false,
            }
            .http_status(),
            EMPTY_RESPONSE_STATUS,
        );
        assert_eq!(CoordinatorError::NotConfigured.http_status(), 0);
        assert_eq!(
            CoordinatorError::Harness(HarnessError::SessionBusy {
                label: "L".to_owned(),
                session_id: "s".to_owned(),
            })
            .http_status(),
            via_downstream::SESSION_BUSY_STATUS,
        );
    }

    #[test]
    fn a_cancelled_harness_turn_is_reported_as_cancelled() {
        assert!(CoordinatorError::Harness(HarnessError::Cancelled).is_cancelled());
        assert!(!CoordinatorError::NotConfigured.is_cancelled());
    }

    #[test]
    fn every_code_is_distinct_and_prefixed() {
        let codes = [
            CoordinatorError::NotConfigured.code(),
            CoordinatorError::EmptyResponse {
                label: String::new(),
                recoverable: false,
            }
            .code(),
            CoordinatorError::NotFinalResult {
                state: String::new(),
            }
            .code(),
            CoordinatorError::CancelUnsupported.code(),
            CoordinatorError::QueryUnsupported.code(),
            CoordinatorError::DelegationAlreadyStarted.code(),
            CoordinatorError::NotCancellable {
                label: String::new(),
            }
            .code(),
            CoordinatorError::DelegationNotFound {
                label: String::new(),
            }
            .code(),
            CoordinatorError::NotRecoverable {
                label: String::new(),
            }
            .code(),
            CoordinatorError::SessionDirectoryUnknown {
                label: String::new(),
            }
            .code(),
            CoordinatorError::PermissionRequestUnknown.code(),
        ];
        let mut sorted = codes.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), codes.len(), "codes must be distinct");
        assert!(codes.iter().all(|code| code.starts_with("VIA_")));
    }

    #[test]
    fn a_refusal_becomes_a_localized_work_failure() {
        let failure = CoordinatorError::DelegationAlreadyStarted.into_failure(Locale::Zh);
        assert_eq!(failure.message, "当前协调轮次已经启动了一个第三层任务");
    }
}
