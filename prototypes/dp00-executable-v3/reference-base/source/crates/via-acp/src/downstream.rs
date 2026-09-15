//! The Layer-3 plug: an ACP process backend behind
//! [`via_downstream::DownstreamAgent`].
//!
//! `docs/architecture.md` §6 — *"VIA ships no agent harness of its own. Layer 3
//! is a single extension point, and every harness is a plugin behind it."* This
//! module is the `acp` shape of that plug, and it is the whole of what the
//! layers above ever see of ACP.
//!
//! # A profile is data, not a subclass
//!
//! `via-acp` names no backend — not in code, not in a match arm, not in a test
//! fixture (`docs/architecture.md` §9, and upstream's
//! `dependency-boundaries.test.mjs` asserts the same rule with a regex over
//! source text). Everything that distinguishes one ACP agent from another
//! arrives on an [`AcpBackendProfile`]: a command, its arguments, a working
//! directory, a projected environment, the capability flags it declares, and
//! two optional hooks for rewriting its output and its errors. Adding a backend
//! is a `via-backends` table entry; it is never a change here.
//!
//! # What this seam deliberately does not carry
//!
//! Sessions, the Work queue, permissions policy and cancellation *state* come
//! from Layer 2. A harness answers prompts and emits events; it never owns a
//! `work_id` or a permission decision. So [`AcpHarnessSession::cancel`] reports
//! [`CancelOutcome::Requested`] for a transport cancel rather than claiming
//! confirmation it does not have — *"cancellation is confirmed, not
//! optimistic"* (`docs/architecture.md` §4).

use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use via_downstream::{
    BackendCapabilities, CancelOutcome, CancelRoute, CancelScope, CancelTarget, DownstreamAgent,
    HarnessDescriptor, HarnessError, HarnessHealth, HarnessSession, PromptOutcome, PromptRequest,
    SessionEvent, SessionKey, StopReason,
};
use via_i18n::Locale;

use crate::client::{
    AcpProcessClient, FormatRequestError, PromptOptions, SanitizeOutput, SessionDetails,
    SessionRole,
};
use crate::content::prompt_with_input_parts;
use crate::error::AcpError;
use crate::process::SpawnSpec;
use crate::registry::AcpSessionRegistry;

/// Everything that distinguishes one ACP backend from another.
///
/// The Rust shape of `acpBackendProfile()`
/// (`server/src/agent/acp-backend-profile.mjs`), which upstream builds by
/// asking a named driver for one. Here the driver is the caller: `via-backends`
/// resolves an id to a profile and hands it over, and this crate never learns
/// which id it was.
#[derive(Clone)]
pub struct AcpBackendProfile {
    /// The catalogued backend id, used only to build [`SessionKey`]s and to
    /// look the descriptor up. Opaque to this crate.
    pub protocol: String,
    /// The display label, interpolated into every message.
    pub label: String,
    /// How to start the child, including its projected environment.
    pub spawn: SpawnSpec,
    /// The seven flags this harness declares.
    pub capabilities: BackendCapabilities,
    /// Per-request deadline. `None` takes [`crate::limits::DEFAULT_TIMEOUT`].
    pub timeout: Option<std::time::Duration>,
    /// The locale messages are rendered in.
    pub locale: Locale,
    /// Rewrites the child's output before it is stored or shown.
    pub sanitize_process_output: Option<SanitizeOutput>,
    /// Rewrites a JSON-RPC failure into an actionable sentence.
    pub format_request_error: Option<FormatRequestError>,
}

impl std::fmt::Debug for AcpBackendProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpBackendProfile")
            .field("protocol", &self.protocol)
            .field("label", &self.label)
            .field("spawn", &self.spawn)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

impl AcpBackendProfile {
    /// A profile with no hooks and the default deadline.
    #[must_use]
    pub fn new(
        protocol: impl Into<String>,
        label: impl Into<String>,
        spawn: SpawnSpec,
        capabilities: BackendCapabilities,
    ) -> Self {
        Self {
            protocol: protocol.into(),
            label: label.into(),
            spawn,
            capabilities,
            timeout: None,
            locale: Locale::default(),
            sanitize_process_output: None,
            format_request_error: None,
        }
    }
}

/// An ACP process backend, as Layer 2 sees it.
#[derive(Debug)]
pub struct AcpDownstreamAgent {
    descriptor: HarnessDescriptor,
    profile: AcpBackendProfile,
    client: AcpProcessClient,
    registry: Arc<AcpSessionRegistry>,
}

impl AcpDownstreamAgent {
    /// Declare and validate an ACP harness.
    ///
    /// The descriptor is checked against `via-catalog` at construction, not
    /// mid-turn: `docs/architecture.md` §6 — *"a descriptor that is incomplete
    /// or internally inconsistent is rejected at startup"*.
    ///
    /// # Errors
    ///
    /// [`HarnessError::DriverNotRegistered`],
    /// [`HarnessError::DriverLabelMismatch`] or
    /// [`HarnessError::IncompleteCapabilities`].
    pub fn declare(
        profile: AcpBackendProfile,
        registry: Arc<AcpSessionRegistry>,
    ) -> Result<Self, HarnessError> {
        let descriptor =
            HarnessDescriptor::declare(&profile.protocol, &profile.label, profile.capabilities)?;
        let mut builder = AcpProcessClient::builder()
            .label(profile.label.clone())
            .locale(profile.locale)
            .spawn_spec(profile.spawn.clone());
        if let Some(timeout) = profile.timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(sanitize) = profile.sanitize_process_output.clone() {
            builder = builder.sanitize_process_output(sanitize);
        }
        if let Some(format) = profile.format_request_error.clone() {
            builder = builder.format_request_error(format);
        }
        Ok(Self {
            descriptor,
            profile,
            client: builder.build(),
            registry,
        })
    }

    /// The underlying process client.
    ///
    /// Exposed so a caller that needs an ACP method this seam does not model —
    /// `session/list`, `session/set_config_option` — can reach it without a
    /// second dispatch path (`docs/architecture.md` §6: *one dispatcher*).
    #[must_use]
    pub fn client(&self) -> &AcpProcessClient {
        &self.client
    }

    /// The profile this harness was declared from.
    #[must_use]
    pub fn profile(&self) -> &AcpBackendProfile {
        &self.profile
    }
}

#[async_trait]
impl DownstreamAgent for AcpDownstreamAgent {
    fn descriptor(&self) -> &HarnessDescriptor {
        &self.descriptor
    }

    /// Open, or re-attach to, the session `key` names.
    ///
    /// The registry is what makes re-attaching possible: a recorded session id
    /// is resumed (with its recorded `cwd`, because `session/resume` needs one
    /// and the caller no longer knows it), and a resume that the agent refuses
    /// falls back to a fresh session rather than failing the turn. A backend
    /// that has forgotten a session VIA remembers is a normal event — the user
    /// cleared its state, or it was reinstalled — and the right answer is to
    /// start talking again, not to report an error.
    async fn open(&self, key: &SessionKey) -> Result<Box<dyn HarnessSession>, HarnessError> {
        let cwd = self
            .profile
            .spawn
            .cwd
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let role = if key.is_coordinator() {
            SessionRole::Coordinator
        } else {
            SessionRole::Project
        };
        let details = SessionDetails {
            owner_id: key.owner_id().map(str::to_owned),
            role: Some(role),
            ..SessionDetails::default()
        };

        if let Some(record) = self.registry.get(key) {
            let recorded_cwd = if record.cwd.is_empty() {
                cwd.clone()
            } else {
                std::path::PathBuf::from(&record.cwd)
            };
            match self
                .client
                .resume_session(
                    &record.session_id,
                    recorded_cwd.clone(),
                    Vec::new(),
                    None,
                    details.clone(),
                )
                .await
            {
                Ok(_) => {
                    return Ok(Box::new(AcpHarnessSession {
                        session_id: record.session_id,
                        client: self.client.clone(),
                        capabilities: self.profile.capabilities,
                        locale: self.profile.locale,
                        protocol: self.profile.protocol.clone(),
                    }));
                }
                Err(error) => {
                    tracing::warn!(
                        event = "acp.session_resume_failed",
                        backend = self.profile.label,
                        error = %error,
                    );
                    self.registry.delete(key);
                }
            }
        }

        let session = self
            .client
            .new_session(cwd, Vec::new(), None, details)
            .await
            .map_err(|error| error.into_harness(self.profile.locale))?;
        let session_id = session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .session_id
            .clone();
        if session_id.is_empty() {
            return Err(HarnessError::InvalidSession {
                id: self.profile.protocol.clone(),
            });
        }
        self.registry.set(
            key,
            &session_id,
            &self
                .profile
                .spawn
                .cwd
                .clone()
                .unwrap_or_default()
                .to_string_lossy(),
        );
        Ok(Box::new(AcpHarnessSession {
            session_id,
            client: self.client.clone(),
            capabilities: self.profile.capabilities,
            locale: self.profile.locale,
            protocol: self.profile.protocol.clone(),
        }))
    }

    /// Whether the agent process is up and initialized.
    ///
    /// Never starts one: the probe is polled, and a probe that spawned a
    /// backend would make "is it running?" and "run it" the same question.
    async fn health(&self) -> HarnessHealth {
        if self.client.is_ready().await {
            HarnessHealth::ready()
        } else {
            HarnessHealth::not_started()
        }
    }
}

/// One open ACP session.
#[derive(Debug)]
pub struct AcpHarnessSession {
    session_id: String,
    client: AcpProcessClient,
    capabilities: BackendCapabilities,
    locale: Locale,
    protocol: String,
}

#[async_trait]
impl HarnessSession for AcpHarnessSession {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    async fn prompt(&self, request: PromptRequest) -> Result<PromptOutcome, HarnessError> {
        let prompt = prompt_with_input_parts(&request.text, &request.attachments);
        let timeout = Some(request.timeout_ms.map(std::time::Duration::from_millis));
        let turn = self
            .client
            .prompt(
                &self.session_id,
                prompt,
                PromptOptions {
                    timeout,
                    ..PromptOptions::default()
                },
            )
            .await
            .map_err(|error| error.into_harness(self.locale))?;
        PromptOutcome::new(
            &turn.content,
            StopReason::from_wire(&stop_reason_wire(&turn.response.stop_reason)),
        )
    }

    /// Progress events for this session.
    ///
    /// Empty for now, and deliberately so rather than silently: the projection
    /// itself is [`via_downstream::ActivityTracker`], and wiring it to a
    /// broadcast fan-out is `via-backends`' job, because it is the crate that
    /// knows which Work a session's activity belongs to. A subscriber gets a
    /// stream that ends rather than one that lies about being live.
    fn events(&self) -> BoxStream<'static, SessionEvent> {
        Box::pin(futures::stream::empty())
    }

    /// Ask the agent to stop.
    ///
    /// `session/cancel` is a **notification**: nothing comes back, so the
    /// honest answer is [`CancelOutcome::Requested`]. Confirmation arrives
    /// later, as a prompt that returns `stopReason: "cancelled"`.
    async fn cancel(&self, scope: CancelScope) -> Result<CancelOutcome, HarnessError> {
        if !scope.is_supported_by(self.capabilities) {
            return Err(HarnessError::CancelUnsupported);
        }
        self.client
            .cancel_session(&self.session_id)
            .await
            .map_err(|error| error.into_harness(self.locale))?;
        Ok(CancelOutcome::requested(
            CancelRoute::Adapter,
            match scope.delegation_id() {
                Some(delegation_id) => CancelTarget {
                    delegation_id: Some(delegation_id.to_owned()),
                    session_id: Some(self.session_id.clone()),
                },
                None => CancelTarget::session(&self.session_id),
            },
        ))
    }
}

impl AcpHarnessSession {
    /// The backend id this session belongs to.
    #[must_use]
    pub fn protocol(&self) -> &str {
        &self.protocol
    }
}

/// The wire spelling of an SDK stop reason.
///
/// The SDK's enum and `via-downstream`'s are two spellings of the same protocol
/// value; the wire string is what they agree on, so going through it means a
/// stop reason either crate grows later still arrives intact rather than being
/// flattened into `end_turn`.
///
/// The SDK enum is `#[non_exhaustive]`, which is exactly why this serializes
/// rather than matching: a hand-written match would need a catch-all arm, and
/// the only honest value for that arm is "not a normal end of turn". Serializing
/// has no such arm — every variant, present and future, renders as itself.
fn stop_reason_wire(reason: &agent_client_protocol::schema::v1::StopReason) -> String {
    serde_json::to_value(reason)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        // Unreachable for a fieldless enum. `unknown` rather than `end_turn`
        // because [`StopReason::Other`] reports `is_end_turn() == false`, and a
        // turn whose ending nobody understood must not be recorded as a clean
        // one.
        .unwrap_or_else(|| UNKNOWN_STOP_REASON.to_owned())
}

/// What an unreadable stop reason is reported as.
///
/// Deliberately not a protocol value: it parses to
/// [`StopReason::Other`](via_downstream::StopReason::Other), which is not an
/// end of turn.
const UNKNOWN_STOP_REASON: &str = "unknown";

impl AcpError {
    /// Project an ACP failure onto the Layer-3 seam.
    ///
    /// Two variants have a named counterpart and must use it, because Layer 2
    /// dispatches on them: a busy session is
    /// [`HarnessError::SessionBusy`] (HTTP 409) and a cancelled turn is
    /// [`HarnessError::Cancelled`]. Everything else becomes
    /// [`HarnessError::Agent`], which is upstream's `AgentError` lifted intact
    /// — message already localized, `status`, `body` and `protocol` preserved.
    #[must_use]
    pub fn into_harness(self, locale: Locale) -> HarnessError {
        match &self {
            Self::SessionBusy { label, id } => HarnessError::SessionBusy {
                label: label.clone(),
                session_id: id.clone(),
            },
            Self::Cancelled { .. } => HarnessError::Cancelled,
            _ => HarnessError::Agent {
                message: self.message(locale),
                status: self.status(),
                body: self.body().to_owned(),
                protocol: self.protocol().to_owned(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_busy_session_keeps_its_named_variant_and_its_409() {
        let error = AcpError::SessionBusy {
            label: "Example".into(),
            id: "s-1".into(),
        };
        let harness = error.into_harness(Locale::En);
        assert!(matches!(harness, HarnessError::SessionBusy { .. }));
        assert_eq!(harness.http_status(), via_downstream::SESSION_BUSY_STATUS);
    }

    #[test]
    fn a_cancelled_turn_never_becomes_a_generic_agent_error() {
        let harness = AcpError::Cancelled { reason: None }.into_harness(Locale::En);
        assert!(harness.is_cancelled());
    }

    #[test]
    fn everything_else_arrives_as_an_agent_error_with_its_body_intact() {
        let error = AcpError::RequestFailed {
            label: "Example".into(),
            method: "session/prompt".into(),
            detail: "Internal error".into(),
            stderr: String::new(),
            body: "missing scope".into(),
        };
        let HarnessError::Agent {
            message,
            status,
            body,
            protocol,
        } = error.into_harness(Locale::En)
        else {
            panic!("an unnamed failure lifts to HarnessError::Agent");
        };
        assert_eq!(message, "Example ACP session/prompt failed: Internal error");
        assert_eq!(status, 0);
        assert_eq!(body, "missing scope");
        assert_eq!(protocol, "acp");
    }

    #[test]
    fn stop_reasons_round_trip_through_their_wire_spelling() {
        use agent_client_protocol::schema::v1::StopReason as Acp;
        for (acp, expected) in [
            (Acp::EndTurn, StopReason::EndTurn),
            (Acp::MaxTokens, StopReason::MaxTokens),
            (Acp::MaxTurnRequests, StopReason::MaxTurnRequests),
            (Acp::Refusal, StopReason::Refusal),
            (Acp::Cancelled, StopReason::Cancelled),
        ] {
            assert_eq!(StopReason::from_wire(&stop_reason_wire(&acp)), expected);
        }
    }

    #[test]
    fn an_unreadable_stop_reason_is_not_a_clean_end_of_turn() {
        let unknown = StopReason::from_wire(UNKNOWN_STOP_REASON);
        assert!(
            !unknown.is_cancelled(),
            "it is not a cancellation either — nobody said so"
        );
        let outcome = PromptOutcome::new("", unknown).expect("an outcome, but not an end turn");
        assert!(
            !outcome.is_end_turn(),
            "a turn whose ending nobody understood must not be recorded as clean"
        );
    }

    #[test]
    fn a_cancelled_stop_reason_can_never_produce_an_outcome() {
        // The rule `PromptOutcome::new` enforces, asserted here because this
        // module is the caller that would otherwise complete cancelled Work.
        assert!(PromptOutcome::new("", StopReason::Cancelled).is_err());
    }
}
