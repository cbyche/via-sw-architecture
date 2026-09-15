//! One dispatcher: configured backend id → harness.
//!
//! Upstream's is `server/src/agent/backends/registry.mjs:47-70` — a `Map` built
//! by running nine hard-coded driver objects through `validateBackendDriver`,
//! and three lookups over it (`backendDriver`, `hasBackendDriver`,
//! `backendIds`). VIA keeps the validation and the lookups and drops the
//! hard-coding, because `docs/architecture.md` §6 requires it:
//!
//! > **Registration is data, not a code change.** A harness is named in
//! > configuration by id; `via-backends` resolves the id to a launch spec.
//! > Adding a harness that speaks ACP or a CLI protocol requires no VIA code at
//! > all.
//!
//! The id is not free-form for that reason: it has to resolve through
//! [`via_catalog`]'s twelve definitions, which is checked by
//! [`HarnessDescriptor::declare`] and checked again here.

use std::sync::Arc;

use indexmap::IndexMap;

use crate::agent::{DownstreamAgent, HarnessSession};
use crate::descriptor::HarnessDescriptor;
use crate::error::HarnessError;
use crate::session_key::SessionKey;

/// Every harness this Gateway can dispatch to.
///
/// Iteration order is registration order, so `ids()` and any diagnostic built
/// from it read the same on every run.
#[derive(Default)]
pub struct HarnessRegistry {
    agents: IndexMap<&'static str, Arc<dyn DownstreamAgent>>,
}

impl HarnessRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a harness under the id its descriptor declares.
    ///
    /// The descriptor is validated **again** here. It was already validated at
    /// construction, and that is not redundant: `descriptor()` is a trait
    /// method, so the object handing one over is the object being checked, and
    /// a registry that trusted it would be trusting the thing it is screening.
    /// The cost is five boolean comparisons and a table lookup, once per
    /// process.
    ///
    /// Returns the harness this one displaced, if any. Upstream's
    /// `new Map([...].map(d => [d.id, d]))` is last-wins and silent
    /// (`registry.mjs:47-56`); this keeps that behaviour and hands back the
    /// evidence, because when registration is configuration rather than a
    /// hard-coded list, two harnesses claiming one id is a wiring bug the
    /// caller should be able to name in its own words.
    ///
    /// # Errors
    ///
    /// Every refusal [`HarnessDescriptor::declare`] makes:
    /// [`HarnessError::DriverNotRegistered`],
    /// [`HarnessError::DriverLabelMismatch`],
    /// [`HarnessError::IncompleteCapabilities`].
    pub fn register(
        &mut self,
        agent: Arc<dyn DownstreamAgent>,
    ) -> Result<Option<Arc<dyn DownstreamAgent>>, HarnessError> {
        let descriptor = *agent.descriptor();
        let revalidated = HarnessDescriptor::declare(
            descriptor.id(),
            descriptor.label(),
            descriptor.capabilities(),
        )?;
        Ok(self.agents.insert(revalidated.id(), agent))
    }

    /// Look a harness up by configured id.
    ///
    /// `backendDriver(protocol)`, `server/src/agent/backends/registry.mjs:58-62`
    /// — the id is trimmed and lower-cased before the lookup, so
    /// `AGENT_PROTOCOL=" Codex "` resolves.
    ///
    /// # Errors
    ///
    /// [`HarnessError::NotConfigured`] when `protocol` is empty, and
    /// [`HarnessError::UnsupportedBackend`] when it names nothing registered.
    ///
    /// Upstream answers `不支持的后台 Agent：` — with an empty id — for both.
    /// VIA splits them because they are different situations with different
    /// remedies: nothing is configured, which `docs/architecture.md` §2 makes a
    /// *supported* mode that degrades `agent` to `direct`, versus something is
    /// configured and is wrong. Recorded in `docs/deviations/phase-2.md`.
    pub fn resolve(&self, protocol: &str) -> Result<&Arc<dyn DownstreamAgent>, HarnessError> {
        let id = protocol.trim().to_lowercase();
        if id.is_empty() {
            return Err(HarnessError::NotConfigured);
        }
        self.agents
            .get(id.as_str())
            .ok_or(HarnessError::UnsupportedBackend { id })
    }

    /// Whether an id resolves. `hasBackendDriver(protocol)`,
    /// `server/src/agent/backends/registry.mjs:64-66`.
    #[must_use]
    pub fn contains(&self, protocol: &str) -> bool {
        self.agents
            .contains_key(protocol.trim().to_lowercase().as_str())
    }

    /// Every registered id, in registration order. `backendIds()`,
    /// `server/src/agent/backends/registry.mjs:68-70`.
    #[must_use]
    pub fn ids(&self) -> Vec<&'static str> {
        self.agents.keys().copied().collect()
    }

    /// The descriptor registered under an id.
    #[must_use]
    pub fn descriptor(&self, protocol: &str) -> Option<HarnessDescriptor> {
        self.resolve(protocol).ok().map(|agent| *agent.descriptor())
    }

    /// How many harnesses are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Whether nothing is registered — the frontend-only Gateway.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Open the session `key` names, on the harness `key` names.
    ///
    /// The protocol comes from the key rather than from a second argument, so
    /// there is no way to open a `codex` key on an `opencode` harness.
    ///
    /// The session that comes back is checked before it is handed on: a
    /// [`HarnessSession`] whose `session_id()` is blank is the Rust analogue of
    /// upstream's `后台 Driver 返回了无效 Profile`
    /// (`server/src/agent/backends/registry.mjs:76-78`) — the one shape of
    /// unusable-but-well-typed result the type system cannot catch.
    ///
    /// # Errors
    ///
    /// Anything [`Self::resolve`] or
    /// [`DownstreamAgent::open`](crate::DownstreamAgent::open) returns, plus
    /// [`HarnessError::InvalidSession`].
    pub async fn open(&self, key: &SessionKey) -> Result<Box<dyn HarnessSession>, HarnessError> {
        let agent = self.resolve(key.protocol())?;
        let session = agent.open(key).await?;
        if session.session_id().trim().is_empty() {
            return Err(HarnessError::InvalidSession {
                id: agent.descriptor().id().to_owned(),
            });
        }
        Ok(session)
    }
}

impl std::fmt::Debug for HarnessRegistry {
    /// Lists the registered ids. The harnesses themselves are trait objects
    /// with no `Debug` bound, and adding one to the seam to make a diagnostic
    /// prettier would be the tail wagging the dog.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HarnessRegistry")
            .field("ids", &self.ids())
            .finish()
    }
}
