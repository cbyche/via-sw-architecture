//! Whether a harness can take a turn right now.
//!
//! The vocabulary is `BackendRuntimeState`'s
//! (`server/src/agent/backend-runtime-state.mjs:7-135`), catalogued twice in
//! `docs/reference/contracts.json` — as `state-name` / *BackendRuntimeState
//! codes* and `error-code` / *backend status codes*. Five statuses, thirteen
//! codes, and one predicate that matters more than either.
//!
//! # The predicate
//!
//! [`HarnessHealth::is_transient`] is `BackendAvailability`'s `transient` flag
//! (`server/src/app/gateway-application.mjs:447-465`). The catalogue says what
//! it is for: *"the `transient` flag is what keeps receipt-based
//! `spawn_thinking` from being rejected during a cold start."* A harness that
//! is merely starting is not a harness that is broken, and conflating the two
//! makes the first thing a user says after launch fail.
//!
//! # What is deliberately not here
//!
//! `backendFailureCode()` — the classifier that turns a thrown error into
//! `NOT_INSTALLED` / `AUTH_REQUIRED` / `START_TIMEOUT` / … by matching its
//! message — is **not** ported into this crate. It reads a transport's error
//! text, so it belongs beside the state machine that produces that text, in
//! `via-acp`. What belongs to the seam is the vocabulary those codes come from,
//! which is here and is closed: a harness picks a [`HarnessStatusCode`], it
//! does not invent a string.

use serde::Serialize;
use serde::ser::SerializeMap;

/// The five coarse states a harness reports.
///
/// `docs/reference/contracts.json` (`error-code` / *backend status codes*):
/// `'stopped' | 'starting' | 'ready' | 'failed' | 'not_configured'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HarnessStatus {
    /// Not running, and not trying to.
    Stopped,
    /// Coming up.
    Starting,
    /// Able to take a turn.
    Ready,
    /// Tried and failed; [`HarnessHealth::retry_after_ms`] says when to try
    /// again.
    Failed,
    /// No harness is configured at all — `docs/architecture.md` §2's
    /// degradation of `agent` mode to `direct`.
    NotConfigured,
}

impl HarnessStatus {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::NotConfigured => "not_configured",
        }
    }

    /// Parse a wire spelling.
    #[must_use]
    pub fn from_wire(value: &str) -> Option<Self> {
        [
            Self::Stopped,
            Self::Starting,
            Self::Ready,
            Self::Failed,
            Self::NotConfigured,
        ]
        .into_iter()
        .find(|status| status.as_str() == value)
    }
}

/// The thirteen codes a harness reports alongside its status.
///
/// `docs/reference/contracts.json` (`error-code` / *backend status codes*).
/// The first seven are lifecycle; the last six are
/// `backendFailureCode()`'s classifications, kept in the vocabulary here so
/// that a driver picks one rather than coining a code the CLI cannot branch on.
/// [`Self::ProcessExited`] appears in both roles upstream and is one value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HarnessStatusCode {
    /// No harness configured.
    NotConfigured,
    /// Configured, never started.
    NotStarted,
    /// Starting.
    Starting,
    /// Waiting on a managed backend service that is itself starting.
    BackendStarting,
    /// Up.
    Ready,
    /// Stopped on purpose.
    Stopped,
    /// The process is gone.
    ProcessExited,
    /// The executable is not installed.
    NotInstalled,
    /// Credentials or configuration are missing.
    ConfigRequired,
    /// The backend needs its own sign-in.
    AuthRequired,
    /// The harness speaks a protocol version VIA does not.
    ProtocolMismatch,
    /// Start-up took too long.
    StartTimeout,
    /// Start-up failed for a reason none of the above names.
    StartFailed,
}

impl HarnessStatusCode {
    /// Every code, in catalogue order.
    pub const ALL: [Self; 13] = [
        Self::NotConfigured,
        Self::NotStarted,
        Self::Starting,
        Self::BackendStarting,
        Self::Ready,
        Self::Stopped,
        Self::ProcessExited,
        Self::NotInstalled,
        Self::ConfigRequired,
        Self::AuthRequired,
        Self::ProtocolMismatch,
        Self::StartTimeout,
        Self::StartFailed,
    ];

    /// The three codes `BackendAvailability` treats as a cold start.
    ///
    /// `server/src/app/gateway-application.mjs:461`.
    pub const TRANSIENT: [Self; 3] = [Self::NotStarted, Self::Starting, Self::BackendStarting];

    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotConfigured => "NOT_CONFIGURED",
            Self::NotStarted => "NOT_STARTED",
            Self::Starting => "STARTING",
            Self::BackendStarting => "BACKEND_STARTING",
            Self::Ready => "READY",
            Self::Stopped => "STOPPED",
            Self::ProcessExited => "PROCESS_EXITED",
            Self::NotInstalled => "NOT_INSTALLED",
            Self::ConfigRequired => "CONFIG_REQUIRED",
            Self::AuthRequired => "AUTH_REQUIRED",
            Self::ProtocolMismatch => "PROTOCOL_MISMATCH",
            Self::StartTimeout => "START_TIMEOUT",
            Self::StartFailed => "START_FAILED",
        }
    }

    /// Parse a wire spelling.
    #[must_use]
    pub fn from_wire(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|code| code.as_str() == value)
    }
}

/// A harness's answer to "can you take a turn?".
///
/// Serializes as `{ ok, status, code, error?, retryAfterMs? }` — the seam's
/// share of `BackendRuntimeState.base()`. The transport-specific fields
/// upstream also carries (`protocol`, `ownership`, `transport`,
/// `acpConnection`) are added by the crate that knows them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessHealth {
    ok: bool,
    status: HarnessStatus,
    code: HarnessStatusCode,
    error: Option<String>,
    retry_after_ms: Option<u64>,
}

impl HarnessHealth {
    /// `{ok: false, status: 'stopped', code: 'NOT_STARTED'}` — the state a
    /// harness is constructed in (`backend-runtime-state.mjs:41-45`).
    #[must_use]
    pub const fn not_started() -> Self {
        Self::new(false, HarnessStatus::Stopped, HarnessStatusCode::NotStarted)
    }

    /// `{ok: true, status: 'ready', code: 'READY'}`.
    #[must_use]
    pub const fn ready() -> Self {
        Self::new(true, HarnessStatus::Ready, HarnessStatusCode::Ready)
    }

    /// `{ok: false, status: 'starting', code: 'STARTING'}`.
    #[must_use]
    pub const fn starting() -> Self {
        Self::new(false, HarnessStatus::Starting, HarnessStatusCode::Starting)
    }

    /// `{ok: false, status: 'starting', code: 'BACKEND_STARTING'}` — waiting on
    /// a managed service (`backend-runtime-state.mjs:95-101`).
    #[must_use]
    pub const fn backend_starting() -> Self {
        Self::new(
            false,
            HarnessStatus::Starting,
            HarnessStatusCode::BackendStarting,
        )
    }

    /// `{ok: false, status: 'stopped', code: 'STOPPED'}`.
    #[must_use]
    pub const fn stopped() -> Self {
        Self::new(false, HarnessStatus::Stopped, HarnessStatusCode::Stopped)
    }

    /// `{enabled: false, ok: true, status: 'not_configured'}` —
    /// `server/src/agent/agent-client.mjs:176-189`.
    ///
    /// `ok` is **true**: a Gateway with no harness is not a broken Gateway, it
    /// is a Gateway in frontend-only mode.
    #[must_use]
    pub const fn not_configured() -> Self {
        Self::new(
            true,
            HarnessStatus::NotConfigured,
            HarnessStatusCode::NotConfigured,
        )
    }

    /// `{ok: false, status: 'failed', code: <classification>}`.
    ///
    /// `code` is whichever of the failure codes the harness's own classifier
    /// arrived at — see the module docs on why that classifier is not here.
    #[must_use]
    pub fn failed(code: HarnessStatusCode, error: &str) -> Self {
        Self {
            ok: false,
            status: HarnessStatus::Failed,
            code,
            error: Some(error.to_owned()),
            retry_after_ms: None,
        }
    }

    /// Attach the backoff window a retry should wait out.
    ///
    /// `backend-runtime-state.mjs:73-80`: `retryAfterMs` appears only once
    /// there has been a failure, and counts down from the 30 s backoff.
    #[must_use]
    pub const fn with_retry_after_ms(mut self, retry_after_ms: u64) -> Self {
        self.retry_after_ms = Some(retry_after_ms);
        self
    }

    /// Attach an explanatory message. Already localized by its author.
    #[must_use]
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_owned());
        self
    }

    const fn new(ok: bool, status: HarnessStatus, code: HarnessStatusCode) -> Self {
        Self {
            ok,
            status,
            code,
            error: None,
            retry_after_ms: None,
        }
    }

    /// Whether the harness can take a turn.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.ok
    }

    /// The coarse state.
    #[must_use]
    pub const fn status(&self) -> HarnessStatus {
        self.status
    }

    /// The code the CLI and desktop branch on.
    #[must_use]
    pub const fn code(&self) -> HarnessStatusCode {
        self.code
    }

    /// The message, when there is one.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// How long to wait before retrying, when a backoff is running.
    #[must_use]
    pub const fn retry_after_ms(&self) -> Option<u64> {
        self.retry_after_ms
    }

    /// Whether this is a cold start rather than a fault.
    ///
    /// `health.status === 'starting' || ['NOT_STARTED', 'STARTING',
    /// 'BACKEND_STARTING'].includes(health.code)`
    /// (`server/src/app/gateway-application.mjs:461`).
    #[must_use]
    pub fn is_transient(&self) -> bool {
        self.status == HarnessStatus::Starting || HarnessStatusCode::TRANSIENT.contains(&self.code)
    }
}

impl Default for HarnessHealth {
    /// [`Self::not_started`] — upstream's constructed state.
    fn default() -> Self {
        Self::not_started()
    }
}

impl Serialize for HarnessHealth {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("ok", &self.ok)?;
        map.serialize_entry("status", self.status.as_str())?;
        map.serialize_entry("code", self.code.as_str())?;
        if let Some(error) = &self.error {
            map.serialize_entry("error", error)?;
        }
        if let Some(retry_after_ms) = self.retry_after_ms {
            map.serialize_entry("retryAfterMs", &retry_after_ms)?;
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cold_start_is_transient_and_a_failure_is_not() {
        assert!(HarnessHealth::not_started().is_transient());
        assert!(HarnessHealth::starting().is_transient());
        assert!(HarnessHealth::backend_starting().is_transient());
        assert!(!HarnessHealth::ready().is_transient());
        assert!(!HarnessHealth::stopped().is_transient());
        assert!(!HarnessHealth::failed(HarnessStatusCode::AuthRequired, "sign in").is_transient());
    }

    #[test]
    fn an_unconfigured_gateway_is_healthy() {
        let health = HarnessHealth::not_configured();
        assert!(health.is_ok());
        assert_eq!(health.status(), HarnessStatus::NotConfigured);
        assert!(!health.is_transient());
    }

    #[test]
    fn every_code_round_trips() {
        for code in HarnessStatusCode::ALL {
            assert_eq!(HarnessStatusCode::from_wire(code.as_str()), Some(code));
        }
        assert_eq!(HarnessStatusCode::from_wire("READY "), None);
    }
}
