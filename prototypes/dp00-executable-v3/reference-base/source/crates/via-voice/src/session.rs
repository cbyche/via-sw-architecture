//! The voice session's own state: turns, timing constants, and the two
//! upgrade decisions.
//!
//! Ported from the per-connection state of
//! `server/src/voice/realtime-gateway.mjs`.
//!
//! # Turn identity is two values, not one
//!
//! Every stale check in the gateway compares an id **and** a generation, and
//! both are needed:
//!
//! - the **id** distinguishes two different utterances;
//! - the **generation** distinguishes *the same utterance before and after a
//!   barge-in*. `interrupt`, `mute` and the socket close all bump it without
//!   minting a new id, so a tool result that was already in flight when the
//!   user cut in is recognized as stale even though its turn id still matches.
//!
//! [`TurnTracker`] owns both, plus the *committed* pair — the turn the model
//! has actually been told about, which lags the current one until the ASR
//! settles.

use uuid::Uuid;

use crate::turn::TurnRef;

/// How many audio chunks are buffered while the socket is down.
///
/// **External contract** — `realtime-gateway.mjs:55`
/// (`MAX_PENDING_AUDIO_CHUNKS = 30`). Oldest-first eviction: a reconnect that
/// replays a minute of stale speech is worse than one that replays the last
/// second.
pub const MAX_PENDING_AUDIO_CHUNKS: usize = 30;

/// How long the gateway waits for a response to start before it reconnects.
///
/// **External contract** — `realtime-gateway.mjs:56`
/// (`RESPONSE_START_WATCHDOG_MS = 12000`). A provider may override it through
/// [`crate::provider::ProviderView::response_start_timeout_ms`].
pub const RESPONSE_START_WATCHDOG_MS: u64 = 12_000;

/// How long a permission answer waits for the model to act on it.
///
/// **External contract** — `realtime-gateway.mjs:57`
/// (`PERMISSION_RESPONSE_GRACE_MS = 800`). The grace exists because the model
/// usually answers on its own; the timer only covers the case where it does
/// not.
pub const PERMISSION_RESPONSE_GRACE_MS: u64 = 800;

/// How long a retired response context survives as a tombstone.
///
/// **External contract** — `realtime-gateway.mjs:58`
/// (`RESPONSE_CONTEXT_CLEANUP_MS = 30000`). Late provider audio and late client
/// receipts must not resurrect a cancelled response, and thirty seconds
/// outlives both.
pub const RESPONSE_CONTEXT_CLEANUP_MS: u64 = 30_000;

/// How long a connection must hold before the reconnect backoff resets.
///
/// **External contract** — `realtime-gateway.mjs:59`
/// (`REALTIME_STABLE_CONNECTION_MS = 10000`).
pub const REALTIME_STABLE_CONNECTION_MS: u64 = 10_000;

/// How many times a wake re-attempts a connect refused for capacity.
///
/// **External contract** — `realtime-gateway.mjs:1702`
/// (`WAKE_CONNECT_MAX_ATTEMPTS = 3`).
pub const WAKE_CONNECT_MAX_ATTEMPTS: u32 = 3;

/// How long a wake waits between those attempts.
///
/// **External contract** — `realtime-gateway.mjs:1703`
/// (`WAKE_CONNECT_RETRY_BACKOFF_MS = 350`).
pub const WAKE_CONNECT_RETRY_BACKOFF_MS: u64 = 350;

/// The WebSocket route the realtime gateway accepts.
///
/// **External contract** — `realtime-gateway.mjs:71`. Any other path is
/// destroyed without an HTTP response at all, because an upgrade to an unknown
/// route is not a client the Gateway has anything to say to.
pub const REALTIME_ROUTE: &str = "/api/realtime";

/// The refusal for an upgrade from a disallowed origin.
///
/// **External contract** — `realtime-gateway.mjs:161-163`.
pub const UPGRADE_FORBIDDEN: (&str, &str) = ("403 Forbidden", "origin not allowed");

/// The refusal for an upgrade with no resolvable identity.
///
/// **External contract** — `realtime-gateway.mjs:166-168`.
pub const UPGRADE_UNAUTHORIZED: (&str, &str) = ("401 Unauthorized", "identity required");

/// The `voice-` turn id prefix.
///
/// **External contract** — `realtime-gateway.mjs:983`
/// (`` `voice-${Date.now()}-${turnGeneration}` ``). `docs/rebrand.md` lists it
/// as brand-free and keeps it.
pub const VOICE_TURN_PREFIX: &str = "voice-";

/// The `text_` turn id prefix.
///
/// **External contract** — `realtime-gateway.mjs:1771`
/// (`` `text_${randomUUID().replaceAll('-', '')}` ``).
pub const TEXT_TURN_PREFIX: &str = "text_";

/// The `voice_` notification-claimant prefix.
///
/// **External contract** — `realtime-gateway.mjs:220`
/// (`` `voice_${randomUUID()}` ``). One claimant id per connection is what
/// makes the notification claim a per-frontend lease.
pub const CLAIMANT_PREFIX: &str = "voice_";

/// What an upgrade request may be answered with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeDecision {
    /// Accept the upgrade.
    Accept,
    /// Close the socket with no HTTP response at all.
    ///
    /// **External contract** — `realtime-gateway.mjs:70-74`: an upgrade to a
    /// path this Gateway does not serve gets `socket.destroy()`, not a status
    /// line.
    Destroy,
    /// Refuse with a status line and a plain-text body.
    Reject {
        /// The HTTP status line.
        status: &'static str,
        /// The body.
        message: &'static str,
    },
}

/// Decide what to do with a WebSocket upgrade.
///
/// **External contract** — `realtime-gateway.mjs:70-74, 158-177`, in that
/// order: the route first, then the origin, then the identity. The order
/// matters — an unknown route must not leak whether an origin would have been
/// allowed.
#[must_use]
pub const fn upgrade_decision(
    path_matches: bool,
    origin_allowed: bool,
    has_identity: bool,
) -> UpgradeDecision {
    if !path_matches {
        return UpgradeDecision::Destroy;
    }
    if !origin_allowed {
        return UpgradeDecision::Reject {
            status: UPGRADE_FORBIDDEN.0,
            message: UPGRADE_FORBIDDEN.1,
        };
    }
    if !has_identity {
        return UpgradeDecision::Reject {
            status: UPGRADE_UNAUTHORIZED.0,
            message: UPGRADE_UNAUTHORIZED.1,
        };
    }
    UpgradeDecision::Accept
}

/// A new claimant id for one connection.
#[must_use]
pub fn new_claimant_id() -> String {
    format!("{CLAIMANT_PREFIX}{}", Uuid::new_v4())
}

/// A new typed-input turn id.
///
/// **External contract** — `realtime-gateway.mjs:1771`: a UUID with its dashes
/// stripped.
#[must_use]
pub fn new_text_turn_id() -> String {
    format!("{TEXT_TURN_PREFIX}{}", Uuid::new_v4().simple())
}

/// The current and committed turns of one connection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TurnTracker {
    current: TurnRef,
    committed: TurnRef,
    sequence: i64,
}

impl TurnTracker {
    /// A tracker with no turn.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The turn currently being captured.
    #[must_use]
    pub const fn current(&self) -> &TurnRef {
        &self.current
    }

    /// The turn the model has been told about.
    #[must_use]
    pub const fn committed(&self) -> &TurnRef {
        &self.committed
    }

    /// The current generation.
    #[must_use]
    pub const fn generation(&self) -> i64 {
        self.current.turn_generation
    }

    /// Begin a voice turn at `now_ms`, minting its id.
    ///
    /// **External contract** — `realtime-gateway.mjs:982-984`:
    /// `voice-<epoch_ms>-<generation>`, where the generation is the
    /// monotonically increasing per-connection sequence. Both halves are
    /// needed: two utterances in the same millisecond differ by generation.
    pub fn begin_voice_turn(&mut self, now_ms: i64) -> TurnRef {
        self.sequence += 1;
        self.current = TurnRef::new(
            format!("{VOICE_TURN_PREFIX}{now_ms}-{}", self.sequence),
            self.sequence,
        );
        self.current.clone()
    }

    /// Begin a typed turn with an already-minted id.
    pub fn begin_typed_turn(&mut self, turn_id: &str) -> TurnRef {
        self.sequence += 1;
        self.current = TurnRef::new(turn_id, self.sequence);
        self.current.clone()
    }

    /// Adopt a turn the correlation already knew about.
    ///
    /// **External contract** — `realtime-gateway.mjs:975-981`: a provider that
    /// re-announces speech on a known item must reuse that item's turn rather
    /// than minting a second one for the same utterance.
    pub fn adopt(&mut self, turn: TurnRef) {
        self.current = turn;
    }

    /// Bump the generation without minting a turn.
    ///
    /// Used by `interrupt`, `mute` and the socket close. The committed
    /// generation is bumped with it, which is what makes every in-flight tool
    /// call stale at once.
    pub fn interrupt(&mut self) -> i64 {
        self.sequence += 1;
        self.current.turn_generation = self.sequence;
        self.committed.turn_generation = self.sequence;
        self.sequence
    }

    /// Commit `turn` as the one the model has been told about.
    ///
    /// **External contract** — `realtime-gateway.mjs:441-452`. Two guards:
    /// an empty id commits nothing, and an **older generation never
    /// overwrites a newer one** — an ASR result that settles after the user has
    /// already started the next turn must not drag the committed turn
    /// backwards.
    pub fn commit(&mut self, turn: &TurnRef) -> bool {
        if turn.turn_id.is_empty() {
            return false;
        }
        if self.committed.turn_id == turn.turn_id
            && self.committed.turn_generation == turn.turn_generation
        {
            return false;
        }
        if turn.turn_generation < self.committed.turn_generation {
            return false;
        }
        self.committed = turn.clone();
        true
    }

    /// The fallback correlation for a response that carried none.
    ///
    /// **External contract** — `realtime-gateway.mjs:592-599`
    /// (`fallbackResponseContext`): the committed turn when there is one,
    /// otherwise the current one — because an uncommitted turn is the only
    /// candidate a spontaneous response could belong to.
    #[must_use]
    pub fn fallback_turn(&self) -> TurnRef {
        if self.committed.turn_id.is_empty() {
            self.current.clone()
        } else {
            self.committed.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn an_unknown_route_is_destroyed_without_a_status_line() {
        assert_eq!(
            upgrade_decision(false, true, true),
            UpgradeDecision::Destroy,
        );
        // And the route is checked first, so a bad origin on a bad route still
        // leaks nothing.
        assert_eq!(
            upgrade_decision(false, false, false),
            UpgradeDecision::Destroy,
        );
    }

    #[test]
    fn the_two_refusals_carry_their_catalogued_bodies() {
        assert_eq!(
            upgrade_decision(true, false, true),
            UpgradeDecision::Reject {
                status: "403 Forbidden",
                message: "origin not allowed",
            },
        );
        assert_eq!(
            upgrade_decision(true, true, false),
            UpgradeDecision::Reject {
                status: "401 Unauthorized",
                message: "identity required",
            },
        );
        assert_eq!(upgrade_decision(true, true, true), UpgradeDecision::Accept);
    }

    #[test]
    fn a_voice_turn_id_carries_the_instant_and_the_generation() {
        let mut tracker = TurnTracker::new();
        let first = tracker.begin_voice_turn(1_700_000_000_000);
        assert_eq!(first.turn_id, "voice-1700000000000-1");
        assert_eq!(first.turn_generation, 1);
        // Two utterances in the same millisecond still differ.
        let second = tracker.begin_voice_turn(1_700_000_000_000);
        assert_eq!(second.turn_id, "voice-1700000000000-2");
        assert_ne!(first.turn_id, second.turn_id);
    }

    #[test]
    fn a_typed_turn_id_is_a_dashless_uuid() {
        let id = new_text_turn_id();
        assert!(id.starts_with("text_"));
        assert_eq!(id.len(), "text_".len() + 32);
        assert!(!id.contains('-'));
        assert_ne!(id, new_text_turn_id());
    }

    #[test]
    fn a_claimant_id_is_unique_per_connection() {
        let id = new_claimant_id();
        assert!(id.starts_with("voice_"));
        assert_ne!(id, new_claimant_id());
    }

    #[test]
    fn an_interrupt_bumps_the_generation_without_minting_a_turn() {
        let mut tracker = TurnTracker::new();
        let turn = tracker.begin_voice_turn(1_000);
        tracker.commit(&turn);
        let before = tracker.generation();
        tracker.interrupt();
        assert_eq!(tracker.generation(), before + 1);
        assert_eq!(
            tracker.current().turn_id,
            turn.turn_id,
            "the id is unchanged: it is the same utterance",
        );
        assert_eq!(
            tracker.committed().turn_generation,
            before + 1,
            "every in-flight tool call is stale at once",
        );
    }

    #[test]
    fn an_older_generation_never_drags_the_committed_turn_backwards() {
        let mut tracker = TurnTracker::new();
        let first = tracker.begin_voice_turn(1_000);
        assert!(tracker.commit(&first));
        let second = tracker.begin_voice_turn(2_000);
        assert!(tracker.commit(&second));
        // A late ASR result for the first turn arrives.
        assert!(!tracker.commit(&first), "the stale commit is refused");
        assert_eq!(tracker.committed().turn_id, second.turn_id);
    }

    #[test]
    fn committing_the_same_turn_twice_changes_nothing() {
        let mut tracker = TurnTracker::new();
        let turn = tracker.begin_voice_turn(1_000);
        assert!(tracker.commit(&turn));
        assert!(!tracker.commit(&turn));
    }

    #[test]
    fn an_empty_turn_id_commits_nothing() {
        let mut tracker = TurnTracker::new();
        assert!(!tracker.commit(&TurnRef::default()));
        assert!(tracker.committed().turn_id.is_empty());
    }

    #[test]
    fn the_fallback_prefers_the_committed_turn_but_falls_back_to_the_current() {
        let mut tracker = TurnTracker::new();
        let turn = tracker.begin_voice_turn(1_000);
        assert_eq!(
            tracker.fallback_turn(),
            turn,
            "an uncommitted turn is the only candidate",
        );
        tracker.commit(&turn);
        let next = tracker.begin_voice_turn(2_000);
        assert_eq!(tracker.fallback_turn(), turn);
        assert_ne!(tracker.fallback_turn(), next);
    }

    #[test]
    fn adopting_a_known_turn_does_not_mint_a_second_one() {
        let mut tracker = TurnTracker::new();
        let first = tracker.begin_voice_turn(1_000);
        tracker.adopt(first.clone());
        assert_eq!(tracker.current(), &first);
        assert_eq!(tracker.generation(), 1, "the sequence did not advance");
    }

    #[test]
    fn the_timing_constants_are_the_catalogued_ones() {
        assert_eq!(MAX_PENDING_AUDIO_CHUNKS, 30);
        assert_eq!(RESPONSE_START_WATCHDOG_MS, 12_000);
        assert_eq!(PERMISSION_RESPONSE_GRACE_MS, 800);
        assert_eq!(RESPONSE_CONTEXT_CLEANUP_MS, 30_000);
        assert_eq!(REALTIME_STABLE_CONNECTION_MS, 10_000);
        assert_eq!(WAKE_CONNECT_MAX_ATTEMPTS, 3);
        assert_eq!(WAKE_CONNECT_RETRY_BACKOFF_MS, 350);
        assert_eq!(REALTIME_ROUTE, "/api/realtime");
    }
}
