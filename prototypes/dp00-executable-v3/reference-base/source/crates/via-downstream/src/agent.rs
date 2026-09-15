//! The seam itself.
//!
//! `docs/architecture.md` §6 states the rule this crate exists to enforce:
//!
//! > **VIA ships no agent harness of its own.** It does not port ARGO's agent
//! > loop, and it does not reimplement one. Layer 3 is a single extension point
//! > — `trait DownstreamAgent` — and every harness, ARGO's included, is a
//! > plugin behind it.
//!
//! and the corollary that keeps it one seam rather than a suggestion:
//!
//! > **One dispatcher.** A new shape adds a `DownstreamAgent` impl, never a
//! > second dispatch path.
//!
//! Upstream has no single trait over its backends. It has a *validated plugin
//! contract* spread across four files — `backends/registry.mjs` (validation and
//! lookup), `backends/shared.mjs` (the connection description),
//! `backend-adapter.mjs` (the error shape) and `acp-backend-profile.mjs` (the
//! entry point) — and every one of those pieces is lifted into the types these
//! two traits are written in terms of.
//!
//! # What is above this seam, and stays there
//!
//! `docs/architecture.md` §6 again: *"Sessions, events, cancellation and
//! permissions come from Layer 2. A harness answers prompts and emits events;
//! it never owns a queue, a `work_id`, or a permission decision."* That is why
//! [`PromptRequest::work_id`](crate::PromptRequest::work_id) is correlation
//! only, why [`SessionEvent`] carries no permission payload, and why
//! [`HarnessSession::cancel`] reports what happened rather than deciding what
//! it means.

use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::cancel::{CancelOutcome, CancelScope};
use crate::descriptor::HarnessDescriptor;
use crate::error::HarnessError;
use crate::event::SessionEvent;
use crate::health::HarnessHealth;
use crate::prompt::{PromptOutcome, PromptRequest};
use crate::session_key::SessionKey;

/// One harness: something VIA can delegate work to.
///
/// An implementation is registered by id and resolved through
/// [`HarnessRegistry`](crate::HarnessRegistry); nothing above Layer 3 names a
/// concrete harness type.
#[async_trait]
pub trait DownstreamAgent: Send + Sync {
    /// Who this harness is, and what it can do.
    ///
    /// Already validated — a [`HarnessDescriptor`] cannot be built any other
    /// way — so a caller may trust the flags without re-checking them.
    fn descriptor(&self) -> &HarnessDescriptor;

    /// Open, or re-attach to, the session named by `key`.
    ///
    /// `key` is the whole address: which backend, and whether this is the
    /// per-owner coordinator session or one delegated project session. A
    /// harness that keeps its own history
    /// ([`native_session_history`](crate::BackendCapabilities::native_session_history))
    /// re-attaches; one that does not starts fresh.
    ///
    /// # Errors
    ///
    /// Any [`HarnessError`]. Configuration faults arrive as
    /// [`HarnessError::Configuration`]; anything the transport reports arrives
    /// as [`HarnessError::Agent`].
    async fn open(&self, key: &SessionKey) -> Result<Box<dyn HarnessSession>, HarnessError>;

    /// Whether the harness can take a turn right now.
    ///
    /// Polled by the availability probe rather than by a turn, so it must not
    /// block on a start-up it could instead report as
    /// [`HarnessHealth::is_transient`].
    async fn health(&self) -> HarnessHealth;
}

/// One open session against a harness.
///
/// Sessions are long-lived: `docs/reference/contracts.json` calls the
/// coordinator key *"THE fixed identity that survives voice sessions, Work IDs,
/// and Gateway restarts"*. A session is therefore not a turn, and
/// [`Self::prompt`] may be called on it many times — one at a time, which is
/// what [`HarnessError::SessionBusy`] answers when something races that.
#[async_trait]
pub trait HarnessSession: Send + Sync {
    /// The backend's own id for this session.
    ///
    /// Empty is not a valid answer: it is the one way a `Box<dyn
    /// HarnessSession>` can still be unusable, and
    /// [`HarnessRegistry::open`](crate::HarnessRegistry::open) rejects it with
    /// [`HarnessError::InvalidSession`] — upstream's
    /// `后台 Driver 返回了无效 Profile`.
    fn session_id(&self) -> &str;

    /// Take one turn.
    ///
    /// # Errors
    ///
    /// [`HarnessError::Cancelled`] when the turn was cancelled — never `Ok`;
    /// see [`PromptOutcome::new`]. [`HarnessError::SessionBusy`] when a turn is
    /// already in flight. Anything else the transport reports as
    /// [`HarnessError::Agent`].
    async fn prompt(&self, request: PromptRequest) -> Result<PromptOutcome, HarnessError>;

    /// Subscribe to this session's normalised progress events.
    ///
    /// The stream is `'static` so a subscriber can outlive the borrow, and it
    /// yields [`SessionEvent`]s only — the projection in [`crate::event`] has
    /// already happened, so there is no raw notification for a subscriber to
    /// mishandle. Progress is observability: `docs/architecture.md` §6 —
    /// *"activity never produces spoken status updates and never affects the
    /// queue."*
    fn events(&self) -> BoxStream<'static, SessionEvent>;

    /// Ask for work to stop.
    ///
    /// Returns what actually happened. A harness that can only *send* a cancel
    /// returns [`CancelOutcome::Requested`] and lets the confirmation arrive
    /// later; it must not return [`CancelOutcome::Confirmed`] on the strength
    /// of having sent one.
    ///
    /// # Errors
    ///
    /// [`HarnessError::CancelUnsupported`] when the scope needs a capability
    /// this harness does not declare — see [`CancelScope::is_supported_by`].
    /// [`HarnessError::NotCancellable`] when nothing matched.
    async fn cancel(&self, scope: CancelScope) -> Result<CancelOutcome, HarnessError>;
}
