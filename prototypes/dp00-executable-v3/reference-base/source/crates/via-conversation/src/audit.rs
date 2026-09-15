//! [`MemoryAudit`] — the append-only record of automatic memory writes.
//!
//! Ported from `server/src/conversation/memory-audit.mjs`.
//!
//! A voice-only product has no confirmation dialog, so a user cannot watch the
//! extractor decide. This file is the substitute: one JSON line per automatic
//! operation, saying whether a patch ran, was skipped and why, or failed. It
//! is what a person reads when they want to know why the assistant started
//! calling them something new.
//!
//! Two properties are load-bearing:
//!
//! - **Memory text stays out of it.** A `patch` line records which documents,
//!   how many edits, and the before/after revisions — never the content. The
//!   audit is a diagnostic surface; copying durable personal facts into a
//!   second file with different retention would widen the blast radius of
//!   anything that reads logs.
//! - **A failure here never fails a memory write.** The first write error
//!   disables auditing for the process, raises one warning, and returns
//!   `false`. Nothing above it branches on that.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use via_i18n::{Locale, format as i18n_format, keys};

/// Epoch-millisecond clock.
pub type NowFn = Arc<dyn Fn() -> i64 + Send + Sync>;

/// Warning sink.
pub type WarningSink = Arc<dyn Fn(&AuditWarning) + Send + Sync>;

/// The diagnostic raised when the audit file could not be written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditWarning {
    /// The rendered `memory.audit_disabled` sentence.
    pub message: String,
    /// Epoch milliseconds.
    pub at: i64,
}

/// The audit's health.
///
/// **External contract** — `memory-audit.mjs:46-53`:
/// `{ok, configured, enabled, warning}`, in that order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditHealth {
    /// `true` while no warning stands.
    pub ok: bool,
    /// Whether a path is configured at all.
    pub configured: bool,
    /// Whether a write would currently be attempted.
    pub enabled: bool,
    /// The most recent warning, or `null`.
    pub warning: Option<AuditWarning>,
}

/// Why an automatic run wrote nothing.
///
/// **External contract** — the catalogued *memory extractor skip reasons*.
/// These land in the audit line's `reason` field verbatim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// The model proposed no changes.
    NoChange,
    /// The proposal named an unknown document, repeated a document, or carried
    /// a change with neither edits nor an append.
    InvalidChange,
    /// The proposed text tripped the sensitive-content gate.
    Sensitive,
    /// Directive-shaped content was routed to `MEMORY.md`, or non-directive
    /// content to `USER.md`.
    DocumentBoundary,
    /// A `USER.md` write was proposed with no explicit directive anywhere in
    /// the user's own turns.
    UserDirectiveNotExplicit,
}

impl SkipReason {
    /// The wire spelling written into the audit line.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoChange => "no_change",
            Self::InvalidChange => "invalid_change",
            Self::Sensitive => "sensitive",
            Self::DocumentBoundary => "document_boundary",
            Self::UserDirectiveNotExplicit => "user_directive_not_explicit",
        }
    }
}

/// One audit line's body, before `at` is prepended.
///
/// **External contract** — the catalogued *memory-audit.jsonl line format*.
/// `op` is the first key of every variant, and field order below is the field
/// order on disk.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuditEvent {
    /// A patch was applied.
    Patch {
        /// Always `patch`.
        op: &'static str,
        /// The owner whose documents changed.
        #[serde(rename = "ownerId")]
        owner_id: String,
        /// Which documents the change named, in order.
        documents: Vec<String>,
        /// The sum of every document's `changed` count.
        changed: usize,
        /// Each document's revision before the write, or `null`.
        #[serde(rename = "beforeRevisions")]
        before_revisions: serde_json::Map<String, serde_json::Value>,
        /// Each document's revision after the write.
        #[serde(rename = "afterRevisions")]
        after_revisions: serde_json::Map<String, serde_json::Value>,
        /// How many exact edits were applied.
        edits: usize,
        /// Whether any change carried an append.
        appended: bool,
    },
    /// A run decided to write nothing.
    Skip {
        /// Always `skip`.
        op: &'static str,
        /// The owner the run was for.
        #[serde(rename = "ownerId")]
        owner_id: String,
        /// Why.
        reason: &'static str,
    },
    /// A run failed.
    Error {
        /// Always `error`.
        op: &'static str,
        /// The owner the run was for.
        #[serde(rename = "ownerId")]
        owner_id: String,
        /// The failure, as a string.
        error: String,
    },
}

impl AuditEvent {
    /// A `skip` line.
    #[must_use]
    pub fn skip(owner_id: &str, reason: SkipReason) -> Self {
        Self::Skip {
            op: "skip",
            owner_id: owner_id.to_owned(),
            reason: reason.as_str(),
        }
    }

    /// An `error` line.
    #[must_use]
    pub fn error(owner_id: &str, error: impl Into<String>) -> Self {
        Self::Error {
            op: "error",
            owner_id: owner_id.to_owned(),
            error: error.into(),
        }
    }

    /// This line's `op`.
    #[must_use]
    pub fn op(&self) -> &'static str {
        match self {
            Self::Patch { op, .. } | Self::Skip { op, .. } | Self::Error { op, .. } => op,
        }
    }
}

struct Inner {
    file_path: Option<PathBuf>,
    locale: Locale,
    now: NowFn,
    on_warning: WarningSink,
    state: Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    disabled: bool,
    warning: Option<AuditWarning>,
}

/// The append-only JSONL audit trail.
///
/// Cheap to clone; every clone is the same audit.
#[derive(Clone)]
pub struct MemoryAudit {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for MemoryAudit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryAudit")
            .field("file_path", &self.inner.file_path)
            .field("health", &self.health())
            .finish()
    }
}

impl Default for MemoryAudit {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Builder for [`MemoryAudit`].
#[derive(Default)]
pub struct MemoryAuditBuilder {
    file_path: Option<PathBuf>,
    locale: Locale,
    now: Option<NowFn>,
    on_warning: Option<WarningSink>,
}

impl std::fmt::Debug for MemoryAuditBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryAuditBuilder")
            .field("file_path", &self.file_path)
            .finish_non_exhaustive()
    }
}

impl MemoryAuditBuilder {
    /// Where the JSONL file lives. `None` makes every record a silent no-op —
    /// upstream's default, and what a Gateway with no config directory gets.
    #[must_use]
    pub fn file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Which locale the disable warning is rendered in.
    #[must_use]
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Override the clock. Every line's `at` comes from it.
    #[must_use]
    pub fn now(mut self, now: NowFn) -> Self {
        self.now = Some(now);
        self
    }

    /// Where the disable warning goes.
    #[must_use]
    pub fn on_warning(mut self, on_warning: WarningSink) -> Self {
        self.on_warning = Some(on_warning);
        self
    }

    /// Finish the audit.
    #[must_use]
    pub fn build(self) -> MemoryAudit {
        MemoryAudit {
            inner: Arc::new(Inner {
                file_path: self.file_path,
                locale: self.locale,
                now: self.now.unwrap_or_else(|| Arc::new(system_now_ms)),
                on_warning: self
                    .on_warning
                    .unwrap_or_else(|| Arc::new(|_: &AuditWarning| {})),
                state: Mutex::new(State::default()),
            }),
        }
    }
}

fn system_now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// `new Date(ms).toISOString()` — always three fractional digits and a `Z`.
#[must_use]
pub fn iso_timestamp(epoch_ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(epoch_ms)
        .unwrap_or_default()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

impl MemoryAudit {
    /// Start building an audit.
    #[must_use]
    pub fn builder() -> MemoryAuditBuilder {
        MemoryAuditBuilder::default()
    }

    /// Append one line. Returns whether it reached the disk.
    ///
    /// **External contract** — `memory-audit.mjs:21-44`:
    /// `` `${JSON.stringify({ at, ...event })}\n` ``, parent directory at
    /// `0700`, file at `0600`. `at` is first; the event's own keys follow in
    /// their declaration order.
    ///
    /// The first failure disables auditing for the life of the process and
    /// raises one warning. Memory writes never branch on the result.
    pub fn record<E: Serialize>(&self, event: &E) -> bool {
        let Some(path) = self.inner.file_path.as_deref() else {
            return false;
        };
        if self.state().disabled {
            return false;
        }
        match self.append(path, event) {
            Ok(()) => true,
            Err(error) => {
                self.disable(&error.to_string());
                false
            }
        }
    }

    fn append<E: Serialize>(&self, path: &Path, event: &E) -> std::io::Result<()> {
        let mut line = serde_json::Map::new();
        line.insert("at".into(), iso_timestamp((self.inner.now)()).into());
        // A payload that is not a JSON object contributes no keys, which is
        // what spreading a primitive does in JavaScript.
        if let Ok(serde_json::Value::Object(body)) = serde_json::to_value(event) {
            for (key, value) in body {
                line.insert(key, value);
            }
        }
        let body = serde_json::to_string(&serde_json::Value::Object(line))
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

        if let Some(parent) = path.parent() {
            let mut builder = std::fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt as _;
                builder.mode(0o700);
            }
            // `mkdirSync(..., { recursive: true })` succeeds on an existing
            // directory; Rust's recursive builder does too.
            builder.create(parent)?;
        }

        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        let mut file = options.open(path)?;
        writeln!(file, "{body}")
    }

    fn disable(&self, detail: &str) {
        let warning = AuditWarning {
            message: i18n_format(
                self.inner.locale,
                keys::MEMORY_AUDIT_DISABLED,
                &[("detail", detail)],
            ),
            at: (self.inner.now)(),
        };
        {
            let mut state = self.state();
            state.disabled = true;
            state.warning = Some(warning.clone());
        }
        // "Diagnostics must not prevent memory operations" —
        // `memory-audit.mjs:37-41` swallows a throwing sink.
        let on_warning = self.inner.on_warning.clone();
        let _ =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || on_warning(&warning)));
    }

    /// The audit's health.
    #[must_use]
    pub fn health(&self) -> AuditHealth {
        let state = self.state();
        AuditHealth {
            ok: state.warning.is_none(),
            configured: self.inner.file_path.is_some(),
            enabled: self.inner.file_path.is_some() && !state.disabled,
            warning: state.warning.clone(),
        }
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_timestamps_always_carry_three_fractional_digits() {
        assert_eq!(iso_timestamp(1000), "1970-01-01T00:00:01.000Z");
        assert_eq!(iso_timestamp(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso_timestamp(1_780_000_000_123), "2026-05-28T20:26:40.123Z");
    }

    #[test]
    fn every_skip_reason_has_its_catalogued_spelling() {
        assert_eq!(SkipReason::NoChange.as_str(), "no_change");
        assert_eq!(SkipReason::InvalidChange.as_str(), "invalid_change");
        assert_eq!(SkipReason::Sensitive.as_str(), "sensitive");
        assert_eq!(SkipReason::DocumentBoundary.as_str(), "document_boundary");
        assert_eq!(
            SkipReason::UserDirectiveNotExplicit.as_str(),
            "user_directive_not_explicit"
        );
    }

    #[test]
    fn an_unconfigured_audit_is_silent_but_healthy() {
        let audit = MemoryAudit::default();
        assert!(!audit.record(&AuditEvent::skip("owner", SkipReason::NoChange)));
        let health = audit.health();
        assert!(health.ok);
        assert!(!health.configured);
        assert!(!health.enabled);
    }
}
