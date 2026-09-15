//! The cancellation facts a coordinator has to be told about, exactly once.
//!
//! When a delegated Work is cancelled and the coordinator session is *busy*,
//! the adapter cannot ask the model to call `session_cancel` — it sends
//! `session/cancel` down the transport itself, because cancellation is urgent
//! (`server/src/agent/acp-backend-adapter.mjs:1404`). The model therefore never
//! learns that the session it delegated to has stopped, and would happily go on
//! reporting progress for it.
//!
//! Upstream closes that hole with a per-owner fact queue
//! (`acp-backend-adapter.mjs:1407-1416`), drained into the *next* coordinator
//! turn as a `<…_reconciliation>` block and cleared only once that turn returns
//! non-empty content (`:1064`). This is that queue.
//!
//! # Why it lives here
//!
//! The facts are statements about **Work** — a `work_id`, the delegation it
//! was correlated to, and when the Gateway verified the stop. Injecting them
//! into a prompt is the coordinator's job and the block's wording is the
//! coordinator's contract; recording them, capping them and making the drain
//! once-only is this crate's.
//!
//! # Once-only, and what "once" costs
//!
//! [`ReconciliationLedger::pending`] *peeks*. A caller injects what it peeked,
//! and calls [`ReconciliationLedger::clear`] only after the turn actually
//! produced something. A turn that returned nothing leaves the facts in place,
//! so a coordinator that failed mid-turn is still told next time. That
//! asymmetry is deliberate and is upstream's.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// The `kind` a cancellation fact carries.
///
/// Contract — `docs/reference/contracts.json` (`prompt-text` /
/// *reconciliation block*): the fact shape is
/// `{"kind":"delegated_session_cancelled","work_id":…,"delegation_id":…,"target_session_id":…,"confirmed_at":…}`.
/// It is the only kind upstream produces.
pub const FACT_DELEGATED_SESSION_CANCELLED: &str = "delegated_session_cancelled";

/// How many facts one owner's queue holds.
///
/// Contract — `server/src/agent/acp-backend-adapter.mjs:1416`
/// (`facts.slice(-20)`), catalogued as *pendingCoordinatorFacts cap 20 per
/// owner*. The **newest** twenty survive.
pub const MAX_FACTS_PER_OWNER: usize = 20;

/// One thing the Gateway did and verified, which the coordinator does not know
/// about yet.
///
/// Field order is the catalogued order, and the field names are snake_case
/// because the model reads them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancellationFact {
    /// Always [`FACT_DELEGATED_SESSION_CANCELLED`] today.
    pub kind: String,
    /// The Work that was cancelled.
    pub work_id: String,
    /// The delegation the cancel was correlated to.
    pub delegation_id: String,
    /// The backend session that was told to stop.
    pub target_session_id: String,
    /// ISO-8601, from `new Date().toISOString()` — the instant the Gateway
    /// **verified** the stop, not the instant it asked for one.
    pub confirmed_at: String,
}

impl CancellationFact {
    /// A `delegated_session_cancelled` fact.
    #[must_use]
    pub fn delegated_session_cancelled(
        work_id: &str,
        delegation_id: &str,
        target_session_id: &str,
        confirmed_at: &str,
    ) -> Self {
        Self {
            kind: FACT_DELEGATED_SESSION_CANCELLED.to_owned(),
            work_id: work_id.to_owned(),
            delegation_id: delegation_id.to_owned(),
            target_session_id: target_session_id.to_owned(),
            confirmed_at: confirmed_at.to_owned(),
        }
    }
}

/// The per-owner queue of unreported facts.
#[derive(Debug, Clone, Default)]
pub struct ReconciliationLedger {
    facts: IndexMap<String, Vec<CancellationFact>>,
}

impl ReconciliationLedger {
    /// An empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a fact for `owner_id`, keeping the newest
    /// [`MAX_FACTS_PER_OWNER`].
    pub fn record(&mut self, owner_id: &str, fact: CancellationFact) {
        let queue = self.facts.entry(owner_id.to_owned()).or_default();
        queue.push(fact);
        if queue.len() > MAX_FACTS_PER_OWNER {
            queue.drain(..queue.len() - MAX_FACTS_PER_OWNER);
        }
    }

    /// What `owner_id` still owes the coordinator, oldest first.
    ///
    /// A **peek**: the facts stay until [`Self::clear`].
    #[must_use]
    pub fn pending(&self, owner_id: &str) -> &[CancellationFact] {
        self.facts.get(owner_id).map_or(&[], Vec::as_slice)
    }

    /// Forget `owner_id`'s facts.
    ///
    /// Call this only after a coordinator turn actually carried them — a turn
    /// that returned nothing has told the model nothing.
    ///
    /// Returns how many facts were dropped.
    pub fn clear(&mut self, owner_id: &str) -> usize {
        self.facts
            .shift_remove(owner_id)
            .map_or(0, |facts| facts.len())
    }

    /// Whether any owner has an unreported fact.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.values().all(Vec::is_empty)
    }

    /// Every owner with an unreported fact, in the order they first recorded
    /// one.
    #[must_use]
    pub fn owners(&self) -> Vec<&str> {
        self.facts
            .iter()
            .filter(|(_, facts)| !facts.is_empty())
            .map(|(owner, _)| owner.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CancellationFact, FACT_DELEGATED_SESSION_CANCELLED, MAX_FACTS_PER_OWNER,
        ReconciliationLedger,
    };
    use serde_json::json;

    fn fact(index: usize) -> CancellationFact {
        CancellationFact::delegated_session_cancelled(
            &format!("work_{index}"),
            &format!("run-{index}"),
            &format!("target-{index}"),
            "2026-08-22T00:00:00.000Z",
        )
    }

    #[test]
    fn a_fact_serializes_in_the_catalogued_shape() {
        let rendered = serde_json::to_value(fact(1)).expect("serializes");
        assert_eq!(
            rendered,
            json!({
                "kind": FACT_DELEGATED_SESSION_CANCELLED,
                "work_id": "work_1",
                "delegation_id": "run-1",
                "target_session_id": "target-1",
                "confirmed_at": "2026-08-22T00:00:00.000Z",
            })
        );
        let keys: Vec<&String> = rendered.as_object().expect("object").keys().collect();
        assert_eq!(
            keys,
            [
                "kind",
                "work_id",
                "delegation_id",
                "target_session_id",
                "confirmed_at"
            ]
        );
    }

    #[test]
    fn peeking_does_not_drain() {
        let mut ledger = ReconciliationLedger::new();
        ledger.record("owner", fact(1));
        assert_eq!(ledger.pending("owner").len(), 1);
        assert_eq!(ledger.pending("owner").len(), 1, "a peek is not a take");
        assert_eq!(ledger.clear("owner"), 1);
        assert!(ledger.pending("owner").is_empty());
        assert!(ledger.is_empty());
    }

    #[test]
    fn the_newest_twenty_survive() {
        let mut ledger = ReconciliationLedger::new();
        for index in 0..(MAX_FACTS_PER_OWNER + 5) {
            ledger.record("owner", fact(index));
        }
        let pending = ledger.pending("owner");
        assert_eq!(pending.len(), MAX_FACTS_PER_OWNER);
        assert_eq!(pending[0].work_id, "work_5");
        assert_eq!(pending[MAX_FACTS_PER_OWNER - 1].work_id, "work_24");
    }

    #[test]
    fn owners_do_not_see_each_others_facts() {
        let mut ledger = ReconciliationLedger::new();
        ledger.record("one", fact(1));
        ledger.record("two", fact(2));
        assert_eq!(ledger.owners(), ["one", "two"]);
        assert_eq!(ledger.pending("one").len(), 1);
        assert_eq!(ledger.pending("one")[0].work_id, "work_1");
        ledger.clear("one");
        assert_eq!(ledger.owners(), ["two"]);
        assert!(!ledger.is_empty());
    }

    #[test]
    fn an_unknown_owner_owes_nothing() {
        let ledger = ReconciliationLedger::new();
        assert!(ledger.pending("nobody").is_empty());
        assert_eq!(ReconciliationLedger::new().clear("nobody"), 0);
    }
}
