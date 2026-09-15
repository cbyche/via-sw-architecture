//! `BackendRuntimeState` — the health surface a backend's lifecycle is read
//! through.
//!
//! Ported from `server/src/agent/backend-runtime-state.mjs:1-136`.
//!
//! This is the object `/api/health.backend` is built from, so its *shape* is
//! the contract and not merely its values. `docs/reference/contracts.json`
//! records it three times — `state-name/BackendRuntimeState codes`,
//! `error-code/backend status codes` and `json-field//api/health.backend` —
//! and one of those notes the consequence of getting it wrong:
//! `gateway-application.mjs:461` treats exactly
//! `['NOT_STARTED', 'STARTING', 'BACKEND_STARTING']` (or `status === 'starting'`)
//! as a transient cold start, so a code outside that set makes the availability
//! probe reject a receipt it should have accepted.
//!
//! # Why the value is a JSON map and not a struct
//!
//! Upstream builds each value by object spread, and JavaScript keeps a
//! re-assigned key in its *original* insertion position. That makes the key
//! order differ between transitions — [`BackendRuntimeState::waiting`] emits
//! `error` before `protocol`, [`BackendRuntimeState::starting`] emits it after
//! `acpConnection` — and `/api/health` publishes that order. The workspace
//! configures `serde_json` with `preserve_order` precisely so this is
//! reproducible; a struct with `#[serde(skip_serializing_if)]` would emit one
//! fixed order and quietly lose it.
//!
//! Typed accessors ([`BackendRuntimeState::code`],
//! [`BackendRuntimeState::status_kind`], [`BackendRuntimeState::ok`]) exist so
//! callers branch on an enum rather than on a string.

use std::sync::Arc;

use serde_json::{Map, Value, json};
use via_i18n::{Locale, format, keys};

use crate::error::ProcessError;

/// The backoff a failed backend is held under before another start is
/// attempted.
///
/// **External contract** — `server/src/agent/backend-runtime-state.mjs:1`
/// (`DEFAULT_BACKOFF_MS`), catalogued under `default-value/timing constants`.
pub const DEFAULT_BACKOFF_MS: i64 = 30_000;

/// The `transport` every value carries.
///
/// **External contract** — `server/src/agent/backend-runtime-state.mjs:44`. A
/// fixed literal: every backend VIA speaks to speaks ACP, whatever process
/// shape carries it.
pub const TRANSPORT_ACP: &str = "acp";

/// How the ACP connection to a backend is carried.
///
/// **External contract** — `state-name/BackendRuntimeState codes`, which
/// records the field as `acpConnection: <'process'|null>`. Modelled as an
/// `Option<AcpConnection>`: `None` is the JSON `null`.
///
/// This is the *second* of two independent axes. Service ownership
/// ([`via_catalog::Ownership`]) says who started the backend **service**; this
/// says how VIA reaches it over ACP. An `external` service can still be
/// reached through a locally spawned `process` adapter, which is why the two
/// are separate fields rather than one enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AcpConnection {
    /// A child process VIA speaks ACP to over its stdio.
    Process,
}

impl AcpConnection {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Process => "process",
        }
    }
}

/// The coarse lifecycle state a client renders.
///
/// **External contract** — `error-code/backend status codes`:
/// `'stopped' | 'starting' | 'ready' | 'failed'`, plus `'not_configured'`,
/// which is `agent-client.mjs`'s and not this type's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendStatusKind {
    /// Not running, and nothing is being attempted.
    Stopped,
    /// A start is under way.
    Starting,
    /// Connected and usable.
    Ready,
    /// A start was attempted and did not succeed.
    Failed,
}

impl BackendStatusKind {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

/// The precise `code` a status carries.
///
/// **External contract** — `state-name/BackendRuntimeState codes` and
/// `error-code/backend status codes`. The first six are lifecycle positions;
/// the last seven are failure classifications produced by
/// [`backend_failure_code`]. [`Self::ProcessExited`] is in both sets, exactly
/// as upstream has it: it is reached both by classifying an error message and
/// by [`BackendRuntimeState::status`] observing a dead client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BackendStatusCode {
    /// Nothing has been attempted yet — the initial value.
    NotStarted,
    /// VIA is starting the ACP client.
    Starting,
    /// The ACP client is up and waiting for the backend service behind it.
    BackendStarting,
    /// Connected and initialised.
    Ready,
    /// Deliberately stopped.
    Stopped,
    /// The process is gone.
    ProcessExited,
    /// The executable is not installed (`ENOENT`).
    NotInstalled,
    /// A credential or setting the backend needs is missing.
    ConfigRequired,
    /// The backend needs an interactive login.
    AuthRequired,
    /// The ACP protocol versions do not overlap.
    ProtocolMismatch,
    /// The start did not finish in time.
    StartTimeout,
    /// Anything else.
    StartFailed,
}

impl BackendStatusCode {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
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

    /// Every code, in catalogue order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
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
        ]
    }

    /// Whether this code means "still coming up", so a caller should wait
    /// rather than refuse.
    ///
    /// **External contract** — `server/src/app/gateway-application.mjs:461`,
    /// quoted in `error-code/backend status codes`: exactly
    /// `NOT_STARTED`, `STARTING` and `BACKEND_STARTING`. A caller must also
    /// treat `status === 'starting'` as transient, which
    /// [`BackendStatusKind::Starting`] covers.
    #[must_use]
    pub const fn is_transient(self) -> bool {
        matches!(
            self,
            Self::NotStarted | Self::Starting | Self::BackendStarting
        )
    }
}

/// A failure as `BackendRuntimeState` sees it.
///
/// Upstream classifies a JavaScript `Error`, reading `error.code`,
/// `error.cause.code` and `error.message`. Rust has no such shape, so the two
/// inputs the classifier actually uses are carried explicitly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BackendFailure {
    /// An OS or library error code, e.g. `ENOENT`. Upstream reads
    /// `error.code || error.cause.code`; either source lands here.
    pub code: Option<String>,
    /// The failure message, already localized by whoever produced it.
    pub message: String,
}

impl BackendFailure {
    /// A failure carrying only a message.
    #[must_use]
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
        }
    }

    /// A failure from an I/O error, mapping `NotFound` to the `ENOENT` that
    /// makes [`backend_failure_code`] answer [`BackendStatusCode::NotInstalled`].
    ///
    /// That mapping is the whole reason the `code` field exists: upstream
    /// reaches `NOT_INSTALLED` from Node's `ENOENT` on `spawn`, and
    /// `io::ErrorKind::NotFound` is the same event.
    #[must_use]
    pub fn from_io(error: &std::io::Error) -> Self {
        let code = match error.kind() {
            std::io::ErrorKind::NotFound => Some("ENOENT".to_owned()),
            _ => None,
        };
        Self {
            code,
            message: error.to_string(),
        }
    }

    /// A failure from a [`ProcessError`], rendered in `locale`.
    #[must_use]
    pub fn from_process_error(error: &ProcessError, locale: Locale) -> Self {
        let code = match error {
            ProcessError::CommandNotFound { .. } => Some("ENOENT".to_owned()),
            ProcessError::Io { source, .. } => Self::from_io(source).code,
            _ => None,
        };
        Self {
            code,
            message: error.message(locale),
        }
    }
}

/// Classify a failure into one of the seven failure codes.
///
/// **External contract** — `server/src/agent/backend-runtime-state.mjs:7-21`.
/// The order of the tests is the contract: a message matching both
/// `not logged in` and `timed out` is `AUTH_REQUIRED`, because authentication
/// is tested first.
///
/// The patterns are upstream's, in both languages upstream ships. VIA also
/// ships `ko`, and a Korean failure message matches none of them and lands on
/// [`BackendStatusCode::StartFailed`] — upstream has no `ko`, so widening the
/// patterns would be a behaviour change rather than a port. Recorded in
/// `docs/deviations/phase-2.md`.
#[must_use]
pub fn backend_failure_code(failure: &BackendFailure) -> BackendStatusCode {
    let code = failure
        .code
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_uppercase();
    let message = failure.message.trim().to_lowercase();
    if code == "ENOENT" {
        return BackendStatusCode::NotInstalled;
    }
    if contains_any(
        &message,
        &["api key", "credential", "not configured", "未配置", "凭据"],
    ) {
        return BackendStatusCode::ConfigRequired;
    }
    if contains_any(
        &message,
        &["unauth", "not logged in", "login", "认证", "登录"],
    ) {
        return BackendStatusCode::AuthRequired;
    }
    if contains_any(&message, &["protocol version", "协议版本"]) {
        return BackendStatusCode::ProtocolMismatch;
    }
    if contains_any(&message, &["timeout", "timed out", "超时"]) {
        return BackendStatusCode::StartTimeout;
    }
    if contains_any(&message, &["exited", "退出"]) {
        return BackendStatusCode::ProcessExited;
    }
    BackendStatusCode::StartFailed
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

/// What an ACP `initialize` answered, as [`BackendRuntimeState::ready`] reads
/// it.
///
/// Upstream passes the whole `initialized` response and picks two fields off
/// it. Carrying just those two keeps `via-process` free of any ACP dependency.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InitializedAgent {
    /// `initialized.agentInfo`, or `null`.
    pub agent_info: Option<Value>,
    /// `initialized.agentCapabilities`, or `{}`.
    pub agent_capabilities: Map<String, Value>,
}

/// A clock, so the backoff is testable without sleeping.
type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;

/// A stderr tail, read lazily at the moment a failure is recorded.
type StderrTail = Arc<dyn Fn() -> String + Send + Sync>;

/// Construction parameters for [`BackendRuntimeState`].
///
/// Mirrors upstream's constructor options object
/// (`server/src/agent/backend-runtime-state.mjs:23-31`).
#[derive(Clone)]
pub struct BackendRuntimeStateOptions {
    /// The backend protocol id, echoed in every value.
    pub protocol: String,
    /// Who owns the backend **service**.
    pub ownership: via_catalog::Ownership,
    /// How the ACP connection is carried, or `None` for the JSON `null`.
    pub connection_kind: Option<AcpConnection>,
    /// The backend's catalog label, interpolated into the
    /// process-exited message.
    pub label: String,
    /// The locale that message is rendered in.
    pub locale: Locale,
    /// Reads the child's captured stderr. Default: empty.
    pub stderr: StderrTail,
    /// Reads the clock in epoch milliseconds. Default: the system clock.
    pub now: Clock,
    /// The backoff window. Default: [`DEFAULT_BACKOFF_MS`].
    pub backoff_ms: i64,
}

impl std::fmt::Debug for BackendRuntimeStateOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendRuntimeStateOptions")
            .field("protocol", &self.protocol)
            .field("ownership", &self.ownership)
            .field("connection_kind", &self.connection_kind)
            .field("label", &self.label)
            .field("locale", &self.locale)
            .field("backoff_ms", &self.backoff_ms)
            .finish_non_exhaustive()
    }
}

impl BackendRuntimeStateOptions {
    /// Options with upstream's defaults for everything but identity.
    #[must_use]
    pub fn new(
        protocol: impl Into<String>,
        ownership: via_catalog::Ownership,
        connection_kind: Option<AcpConnection>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            protocol: protocol.into(),
            ownership,
            connection_kind,
            label: label.into(),
            locale: Locale::En,
            stderr: Arc::new(String::new),
            now: Arc::new(system_now_ms),
            backoff_ms: DEFAULT_BACKOFF_MS,
        }
    }
}

/// Epoch milliseconds. Saturates rather than panicking on a clock before the
/// epoch, which is the only way `duration_since` can fail.
fn system_now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        })
}

/// The recorded failure a backoff is measured from.
#[derive(Debug, Clone)]
struct LastFailure {
    at: i64,
    #[allow(dead_code)]
    result: Map<String, Value>,
}

/// One backend's lifecycle state.
///
/// Ported from `server/src/agent/backend-runtime-state.mjs:23-136`. Every
/// mutator mirrors one upstream method; there are deliberately **no** others.
/// In particular there is no `restart()` and no readiness probe:
/// `state-name/managed backend readiness / restart / shutdown` records that
/// *"There is NO readiness probe and NO restart in managed-backend.mjs"*, and
/// restart is the embedding host's job.
pub struct BackendRuntimeState {
    options: BackendRuntimeStateOptions,
    value: Map<String, Value>,
    last_failure: Option<LastFailure>,
}

impl std::fmt::Debug for BackendRuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendRuntimeState")
            .field("options", &self.options)
            .field("value", &self.value)
            .field("last_failure", &self.last_failure.as_ref().map(|f| f.at))
            .finish()
    }
}

impl BackendRuntimeState {
    /// A fresh state, sitting at `stopped` / `NOT_STARTED`.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:38-42`
    /// and `error-code/backend adapter status codes`, which pins the idle value
    /// to `{ok:false, status:'stopped', code:'NOT_STARTED', protocol,
    /// ownership:'owned', transport:'acp', acpConnection:'process'}`.
    #[must_use]
    pub fn new(options: BackendRuntimeStateOptions) -> Self {
        let mut state = Self {
            options,
            value: Map::new(),
            last_failure: None,
        };
        state.value = state.base(
            [
                ("ok".to_owned(), json!(false)),
                (
                    "status".to_owned(),
                    json!(BackendStatusKind::Stopped.as_str()),
                ),
                (
                    "code".to_owned(),
                    json!(BackendStatusCode::NotStarted.as_str()),
                ),
            ]
            .into_iter()
            .collect(),
        );
        state
    }

    /// Stamp the four identity fields onto a set of caller fields.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:40-47`.
    /// `serde_json::Map` is insertion-ordered under the workspace's
    /// `preserve_order` feature and, like JavaScript, keeps an existing key in
    /// its original position when it is re-assigned. That is what reproduces
    /// the per-transition key order described in the module docs.
    fn base(&self, mut fields: Map<String, Value>) -> Map<String, Value> {
        fields.insert("protocol".to_owned(), json!(self.options.protocol));
        fields.insert(
            "ownership".to_owned(),
            json!(self.options.ownership.as_str()),
        );
        fields.insert("transport".to_owned(), json!(TRANSPORT_ACP));
        fields.insert(
            "acpConnection".to_owned(),
            match self.options.connection_kind {
                Some(kind) => json!(kind.as_str()),
                None => Value::Null,
            },
        );
        fields
    }

    /// Whether a fresh start attempt should be held back.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:49-54`.
    #[must_use]
    pub fn should_backoff(&self) -> bool {
        self.last_failure
            .as_ref()
            .is_some_and(|failure| (self.options.now)() - failure.at < self.options.backoff_ms)
    }

    /// The published status.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:56-73`.
    /// Two overlays, in this order:
    ///
    /// 1. A value that says `ok` while the ACP client is gone is rewritten to
    ///    `stopped` / `PROCESS_EXITED` with a message naming the backend. This
    ///    is the guard that stops `/api/health` claiming a dead backend is
    ///    ready.
    /// 2. Otherwise, if a failure has been recorded, `retryAfterMs` is added,
    ///    floored at zero.
    ///
    /// The two are exclusive, as upstream's early return makes them.
    #[must_use]
    pub fn status(&self, client_ready: bool) -> Map<String, Value> {
        let mut value = self.value.clone();
        if self.ok() && !client_ready {
            value.insert("ok".to_owned(), json!(false));
            value.insert(
                "status".to_owned(),
                json!(BackendStatusKind::Stopped.as_str()),
            );
            value.insert(
                "code".to_owned(),
                json!(BackendStatusCode::ProcessExited.as_str()),
            );
            value.insert(
                "error".to_owned(),
                json!(format(
                    self.options.locale,
                    keys::ACP_PROCESS_EXITED_LABEL,
                    &[("label", self.options.label.as_str())],
                )),
            );
            return value;
        }
        if let Some(failure) = &self.last_failure {
            let remaining = self.options.backoff_ms - ((self.options.now)() - failure.at);
            value.insert("retryAfterMs".to_owned(), json!(remaining.max(0)));
        }
        value
    }

    /// The current value, without the [`Self::status`] overlays.
    #[must_use]
    pub fn value(&self) -> &Map<String, Value> {
        &self.value
    }

    /// The current lifecycle code.
    #[must_use]
    pub fn code(&self) -> BackendStatusCode {
        let raw = self.value.get("code").and_then(Value::as_str);
        BackendStatusCode::all()
            .iter()
            .copied()
            .find(|candidate| Some(candidate.as_str()) == raw)
            .unwrap_or(BackendStatusCode::StartFailed)
    }

    /// The current coarse status.
    #[must_use]
    pub fn status_kind(&self) -> BackendStatusKind {
        match self.value.get("status").and_then(Value::as_str) {
            Some("starting") => BackendStatusKind::Starting,
            Some("ready") => BackendStatusKind::Ready,
            Some("failed") => BackendStatusKind::Failed,
            _ => BackendStatusKind::Stopped,
        }
    }

    /// Whether the backend is currently usable.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.value
            .get("ok")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    /// VIA is starting the ACP client.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:75-84`.
    /// Spreads the *previous* value, so a `retryAfterMs`-free value keeps
    /// whatever extra fields the last transition left behind — including
    /// `agentInfo` and `capabilities` from a previous `ready`.
    pub fn starting(&mut self, error: impl Into<String>) {
        let mut fields = self.value.clone();
        fields.insert("ok".to_owned(), json!(false));
        fields.insert(
            "status".to_owned(),
            json!(BackendStatusKind::Starting.as_str()),
        );
        fields.insert(
            "code".to_owned(),
            json!(BackendStatusCode::Starting.as_str()),
        );
        fields.insert("error".to_owned(), json!(error.into()));
        self.value = self.base(fields);
    }

    /// The ACP client is up; the backend behind it is not yet answering.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:86-93`.
    /// Unlike [`Self::starting`] this does **not** spread the previous value,
    /// so any `agentInfo` or `capabilities` from an earlier `ready` are
    /// dropped — which is why the two methods produce different key orders.
    pub fn waiting(&mut self, message: impl Into<String>) {
        let fields: Map<String, Value> = [
            ("ok".to_owned(), json!(false)),
            (
                "status".to_owned(),
                json!(BackendStatusKind::Starting.as_str()),
            ),
            (
                "code".to_owned(),
                json!(BackendStatusCode::BackendStarting.as_str()),
            ),
            ("error".to_owned(), json!(message.into())),
        ]
        .into_iter()
        .collect();
        self.value = self.base(fields);
    }

    /// Connected and initialised. Clears the recorded failure.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:95-104`.
    /// `agentInfo` defaults to `null` and `capabilities` to `{}`.
    pub fn ready(&mut self, initialized: Option<&InitializedAgent>) {
        self.last_failure = None;
        let fields: Map<String, Value> = [
            ("ok".to_owned(), json!(true)),
            (
                "status".to_owned(),
                json!(BackendStatusKind::Ready.as_str()),
            ),
            ("code".to_owned(), json!(BackendStatusCode::Ready.as_str())),
            (
                "agentInfo".to_owned(),
                initialized
                    .and_then(|value| value.agent_info.clone())
                    .unwrap_or(Value::Null),
            ),
            (
                "capabilities".to_owned(),
                Value::Object(
                    initialized
                        .map(|value| value.agent_capabilities.clone())
                        .unwrap_or_default(),
                ),
            ),
        ]
        .into_iter()
        .collect();
        self.value = self.base(fields);
    }

    /// A start attempt failed. Records the failure, which arms the backoff.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:106-115`.
    /// The captured stderr is appended to the message when there is any, which
    /// is what turns "process exited unexpectedly (1)" into something an
    /// operator can act on.
    pub fn failed(&mut self, failure: &BackendFailure) {
        let stderr = (self.options.stderr)();
        let stderr = stderr.trim();
        let message = if stderr.is_empty() {
            failure.message.clone()
        } else {
            format!(
                "{message}{separator}{stderr}",
                message = failure.message,
                separator = stderr_separator(self.options.locale),
            )
        };
        let fields: Map<String, Value> = [
            ("ok".to_owned(), json!(false)),
            (
                "status".to_owned(),
                json!(BackendStatusKind::Failed.as_str()),
            ),
            (
                "code".to_owned(),
                json!(backend_failure_code(failure).as_str()),
            ),
            ("error".to_owned(), json!(message)),
        ]
        .into_iter()
        .collect();
        self.value = self.base(fields);
        self.last_failure = Some(LastFailure {
            at: (self.options.now)(),
            result: self.value.clone(),
        });
    }

    /// Deliberately stopped. Clears the recorded failure, so the next start is
    /// not held back.
    ///
    /// **External contract** — `server/src/agent/backend-runtime-state.mjs:117-127`.
    /// Spreads the previous value and blanks `error` rather than removing it.
    pub fn stopped(&mut self) {
        self.last_failure = None;
        let mut fields = self.value.clone();
        fields.insert("ok".to_owned(), json!(false));
        fields.insert(
            "status".to_owned(),
            json!(BackendStatusKind::Stopped.as_str()),
        );
        fields.insert(
            "code".to_owned(),
            json!(BackendStatusCode::Stopped.as_str()),
        );
        fields.insert("error".to_owned(), json!(""));
        self.value = self.base(fields);
    }
}

/// The separator upstream writes between a failure message and the captured
/// stderr.
///
/// Upstream writes the full-width colon `：`
/// (`server/src/agent/backend-runtime-state.mjs:112`), which is Chinese
/// punctuation inside an otherwise localized sentence. `via-i18n` has no
/// punctuation-only key and is shipped, so the separator lives here rather
/// than as a literal at the join site. Recorded in
/// `docs/deviations/phase-2.md`.
const fn stderr_separator(locale: Locale) -> &'static str {
    match locale {
        Locale::Zh => "：",
        Locale::En | Locale::Ko => ": ",
    }
}
