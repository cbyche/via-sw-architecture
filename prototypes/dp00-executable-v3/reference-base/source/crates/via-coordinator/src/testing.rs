//! Deterministic doubles for the three seams this crate declares.
//!
//! [`via_downstream::testing::ScriptedHarness`] already doubles the harness, so
//! nothing here re-does it. What is here is the three things a coordinator
//! needs that a harness does not supply: somewhere for the two delegation
//! events to go, somewhere for the two permission events to go, a project
//! Session directory, and a native-delegation adapter.
//!
//! Everything is deterministic: no clock, no randomness, no task scheduling.
//!
//! Behind the `testing` feature, which is **on by default** so this crate's own
//! integration tests reach it under a bare `cargo test -p via-coordinator` and
//! so a sibling can depend on this crate without restating the feature. A build
//! that wants none of it takes `default-features = false`.

use std::sync::{Arc, Mutex, MutexGuard};

use tokio_util::sync::CancellationToken;
use via_downstream::HarnessError;
use via_work::{DelegationRef, PendingPermission};

use crate::error::CoordinatorError;
use crate::permission::PermissionObserver;
use crate::runtime::{CoordinationObserver, NativeDelegationAdapter, ProjectSessionDirectory};

/// Lock, treating poisoning as "the value is still there".
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// One thing a coordinator announced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordedEvent {
    /// `backend.delegated`.
    Delegated(DelegationRef),
    /// `backend.delegation.completed`.
    DelegationCompleted(DelegationRef),
    /// `backend.permission.requested`.
    PermissionRequested(PendingPermission),
    /// `backend.permission.resolved`.
    PermissionResolved(PendingPermission),
}

impl RecordedEvent {
    /// The event name, as the wire spells it.
    ///
    /// The four names are `via-work`'s and `via-downstream`'s; they are spelled
    /// here so a test can assert the *sequence* without matching on shapes.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Delegated(_) => "backend.delegated",
            Self::DelegationCompleted(_) => "backend.delegation.completed",
            Self::PermissionRequested(_) => "backend.permission.requested",
            Self::PermissionResolved(_) => "backend.permission.resolved",
        }
    }

    /// The delegation this event is about, if it is about one.
    #[must_use]
    pub const fn delegation(&self) -> Option<&DelegationRef> {
        match self {
            Self::Delegated(delegation) | Self::DelegationCompleted(delegation) => Some(delegation),
            _ => None,
        }
    }

    /// The permission this event is about, if it is about one.
    #[must_use]
    pub const fn permission(&self) -> Option<&PendingPermission> {
        match self {
            Self::PermissionRequested(permission) | Self::PermissionResolved(permission) => {
                Some(permission)
            }
            _ => None,
        }
    }
}

/// Records everything a coordinator announces, in order.
///
/// One recorder implements both observer traits, because the ordering *between*
/// a permission and a delegation is itself worth asserting.
#[derive(Debug, Default)]
pub struct RecordingObserver {
    events: Mutex<Vec<RecordedEvent>>,
}

impl RecordingObserver {
    /// An empty recorder.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Everything announced so far, in order.
    #[must_use]
    pub fn events(&self) -> Vec<RecordedEvent> {
        lock(&self.events).clone()
    }

    /// The event names, in order.
    #[must_use]
    pub fn names(&self) -> Vec<&'static str> {
        self.events().iter().map(RecordedEvent::name).collect()
    }

    /// Whether an event with `name` was announced.
    #[must_use]
    pub fn saw(&self, name: &str) -> bool {
        self.names().contains(&name)
    }

    fn push(&self, event: RecordedEvent) {
        lock(&self.events).push(event);
    }
}

impl CoordinationObserver for RecordingObserver {
    fn delegated(&self, delegation: &DelegationRef) {
        self.push(RecordedEvent::Delegated(delegation.clone()));
    }

    fn delegation_completed(&self, delegation: &DelegationRef) {
        self.push(RecordedEvent::DelegationCompleted(delegation.clone()));
    }
}

impl PermissionObserver for RecordingObserver {
    fn requested(&self, permission: &PendingPermission) {
        self.push(RecordedEvent::PermissionRequested(permission.clone()));
    }

    fn resolved(&self, permission: &PendingPermission) {
        self.push(RecordedEvent::PermissionResolved(permission.clone()));
    }
}

/// A project-Session directory that answers from a fixed table.
#[derive(Debug, Default, Clone)]
pub struct FixedSessionDirectory {
    sessions: Vec<via_mcp_tools::SessionSummary>,
}

impl FixedSessionDirectory {
    /// A directory holding these Sessions, newest first.
    #[must_use]
    pub fn new(sessions: Vec<via_mcp_tools::SessionSummary>) -> Arc<Self> {
        Arc::new(Self { sessions })
    }

    /// A directory holding one Session.
    #[must_use]
    pub fn with_session(session_id: &str, title: &str, directory: &str) -> Arc<Self> {
        Self::new(vec![via_mcp_tools::SessionSummary::new(
            session_id,
            title,
            directory,
            "2026-01-01T00:00:00Z",
        )])
    }
}

#[async_trait::async_trait]
impl ProjectSessionDirectory for FixedSessionDirectory {
    async fn list(
        &self,
        limit: i64,
    ) -> Result<Vec<via_mcp_tools::SessionSummary>, CoordinatorError> {
        let limit = usize::try_from(limit).unwrap_or(self.sessions.len());
        Ok(self.sessions.iter().take(limit).cloned().collect())
    }

    async fn directory_of(&self, session_id: &str) -> Result<Option<String>, CoordinatorError> {
        Ok(self
            .sessions
            .iter()
            .find(|summary| summary.session_id == session_id)
            .map(|summary| summary.directory.clone()))
    }
}

/// A native-delegation adapter that answers with a fixed result.
///
/// It parks until cancelled when built with [`Self::pending`], which is how a
/// test observes the window between `backend.delegated` and
/// `backend.delegation.completed` without a clock.
#[derive(Debug)]
pub struct ScriptedNativeDelegation {
    result: Option<String>,
    cancels: Mutex<Vec<(String, String)>>,
}

impl ScriptedNativeDelegation {
    /// An adapter whose delegations finish at once with `content`.
    #[must_use]
    pub fn completing(content: &str) -> Arc<Self> {
        Arc::new(Self {
            result: Some(content.to_owned()),
            cancels: Mutex::new(Vec::new()),
        })
    }

    /// An adapter whose delegations never finish on their own.
    #[must_use]
    pub fn pending() -> Arc<Self> {
        Arc::new(Self {
            result: None,
            cancels: Mutex::new(Vec::new()),
        })
    }

    /// Every `(delegation_id, session_id)` this adapter was asked to cancel.
    #[must_use]
    pub fn cancels(&self) -> Vec<(String, String)> {
        lock(&self.cancels).clone()
    }
}

#[async_trait::async_trait]
impl NativeDelegationAdapter for ScriptedNativeDelegation {
    async fn wait(
        &self,
        _delegation_id: &str,
        _session_id: &str,
        signal: CancellationToken,
    ) -> Result<String, CoordinatorError> {
        match self.result.clone() {
            Some(content) if !signal.is_cancelled() => Ok(content),
            _ => {
                signal.cancelled().await;
                Err(CoordinatorError::Harness(HarnessError::Cancelled))
            }
        }
    }

    async fn cancel(&self, delegation_id: &str, session_id: &str) -> Result<(), CoordinatorError> {
        lock(&self.cancels).push((delegation_id.to_owned(), session_id.to_owned()));
        Ok(())
    }
}
