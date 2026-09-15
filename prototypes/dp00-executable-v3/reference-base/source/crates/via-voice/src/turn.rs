//! Correlating a provider's conversation-item ids to VIA turns.
//!
//! Ported from `server/src/voice/turn-correlation.mjs`.
//!
//! An OpenAI Realtime item id identifies one conversation item for the lifetime
//! of the session, and the provider may reopen it: a late transcript delta, a
//! duplicate `completed`, a retried commit. The map is therefore **kept after
//! completion** rather than cleared, so a second event about the same item
//! updates the same frontend turn instead of creating a second message.
//!
//! It is bounded, because the ids are the provider's to mint and a long session
//! would otherwise grow one entry per utterance forever. Eviction is
//! oldest-**inserted** first, which is why the backing map is insertion-ordered
//! and not a `BTreeMap`.

use indexmap::{IndexMap, IndexSet};

/// How many item ids one correlation holds.
///
/// **External contract** — `turn-correlation.mjs:2` (`maxItems = 100`).
pub const DEFAULT_MAX_ITEMS: usize = 100;

/// The turn one conversation item belongs to.
///
/// Upstream stores the gateway's `{ turnId, turnGeneration }` object by
/// reference and compares it with `===`, which is how
/// `responseTurnCandidate === transcriptTurn` works. Here it is a value, and
/// identity comparisons become equality comparisons — the generation is what
/// makes that safe, because it is bumped on every new turn and on every
/// barge-in.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TurnRef {
    /// `voice-<epoch_ms>-<generation>` or `text_<uuid>`.
    pub turn_id: String,
    /// Monotonic within one connection. Bumped by every new turn, every
    /// `interrupt`, every `mute` and the socket close.
    pub turn_generation: i64,
}

impl TurnRef {
    /// A turn reference.
    #[must_use]
    pub fn new(turn_id: impl Into<String>, turn_generation: i64) -> Self {
        Self {
            turn_id: turn_id.into(),
            turn_generation,
        }
    }
}

/// What [`TurnCorrelation::complete`] answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletedTurn {
    /// The correlated turn, or the supplied fallback.
    pub context: TurnRef,
    /// Whether the item was invalidated before it completed.
    pub invalid: bool,
}

/// A bounded item-id → turn map with an invalidation set beside it.
#[derive(Clone, Debug)]
pub struct TurnCorrelation {
    max_items: usize,
    turns: IndexMap<String, TurnRef>,
    invalid_items: IndexSet<String>,
}

impl Default for TurnCorrelation {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ITEMS)
    }
}

impl TurnCorrelation {
    /// A correlation holding at most `max_items` ids.
    #[must_use]
    pub fn new(max_items: usize) -> Self {
        Self {
            max_items,
            turns: IndexMap::new(),
            invalid_items: IndexSet::new(),
        }
    }

    /// Bind `item_id` to `context`, or answer the binding that already exists.
    ///
    /// **External contract** — `turn-correlation.mjs:8-19`. First writer wins:
    /// a provider that re-announces an item must not be able to move an
    /// in-flight turn under the transcript that is already accruing for it.
    /// An empty id binds nothing and echoes the context back.
    pub fn remember(&mut self, item_id: &str, context: TurnRef) -> TurnRef {
        if item_id.is_empty() {
            return context;
        }
        if let Some(existing) = self.turns.get(item_id) {
            return existing.clone();
        }
        self.turns.insert(item_id.to_owned(), context.clone());
        while self.turns.len() > self.max_items {
            // `shift_remove_index(0)` and not `swap_remove`: eviction is
            // oldest-inserted first, and `swap_remove` would evict by moving
            // the newest entry into the hole.
            if let Some((oldest, _)) = self.turns.shift_remove_index(0) {
                self.invalid_items.shift_remove(&oldest);
            }
        }
        context
    }

    /// The turn bound to `item_id`, or `fallback`.
    #[must_use]
    pub fn resolve(&self, item_id: &str, fallback: TurnRef) -> TurnRef {
        self.turns.get(item_id).cloned().unwrap_or(fallback)
    }

    /// The turn bound to `item_id`, if any.
    #[must_use]
    pub fn lookup(&self, item_id: &str) -> Option<&TurnRef> {
        self.turns.get(item_id)
    }

    /// Mark `item_id`'s transcript void.
    ///
    /// Set by `input_audio_buffer.speech_stopped` with
    /// `reason: 'turn_invalid'` — the provider decided the segment was not
    /// speech. Every later transcript event for that item is dropped.
    pub fn invalidate(&mut self, item_id: &str) {
        if !item_id.is_empty() {
            self.invalid_items.insert(item_id.to_owned());
        }
    }

    /// Whether `item_id` was invalidated.
    #[must_use]
    pub fn is_invalid(&self, item_id: &str) -> bool {
        self.invalid_items.contains(item_id)
    }

    /// Resolve `item_id` and report whether it was invalidated.
    ///
    /// **External contract** — `turn-correlation.mjs:33-41`. The binding is
    /// deliberately **not** removed: see the module documentation.
    #[must_use]
    pub fn complete(&self, item_id: &str, fallback: TurnRef) -> CompletedTurn {
        CompletedTurn {
            context: self.resolve(item_id, fallback),
            invalid: self.is_invalid(item_id),
        }
    }

    /// Drop every binding and every invalidation.
    pub fn clear(&mut self) {
        self.turns.clear();
        self.invalid_items.clear();
    }

    /// How many ids are bound.
    #[must_use]
    pub fn len(&self) -> usize {
        self.turns.len()
    }

    /// Whether nothing is bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn turn(id: &str, generation: i64) -> TurnRef {
        TurnRef::new(id, generation)
    }

    #[test]
    fn the_first_binding_wins_and_a_second_remember_does_not_move_it() {
        let mut correlation = TurnCorrelation::default();
        let first = correlation.remember("item_1", turn("voice-1", 1));
        let second = correlation.remember("item_1", turn("voice-2", 2));
        assert_eq!(first, turn("voice-1", 1));
        assert_eq!(second, turn("voice-1", 1));
        assert_eq!(correlation.len(), 1);
    }

    #[test]
    fn an_empty_item_id_binds_nothing_and_echoes_the_context() {
        let mut correlation = TurnCorrelation::default();
        assert_eq!(
            correlation.remember("", turn("voice-1", 1)),
            turn("voice-1", 1)
        );
        assert!(correlation.is_empty());
    }

    #[test]
    fn a_completed_item_stays_bound_so_a_reopened_item_finds_the_same_turn() {
        let mut correlation = TurnCorrelation::default();
        correlation.remember("item_1", turn("voice-1", 1));
        let completed = correlation.complete("item_1", turn("fallback", 9));
        assert_eq!(completed.context, turn("voice-1", 1));
        assert!(!completed.invalid);
        // The provider reopens the item; the same turn is still found.
        assert_eq!(
            correlation.resolve("item_1", turn("fallback", 9)),
            turn("voice-1", 1)
        );
    }

    #[test]
    fn eviction_is_oldest_inserted_first_and_takes_the_invalidation_with_it() {
        let mut correlation = TurnCorrelation::new(2);
        correlation.remember("a", turn("voice-a", 1));
        correlation.remember("b", turn("voice-b", 2));
        correlation.invalidate("a");
        assert!(correlation.is_invalid("a"));
        correlation.remember("c", turn("voice-c", 3));
        assert_eq!(correlation.len(), 2);
        // `a` was evicted, so its stale invalidation went with it. Otherwise a
        // recycled id would inherit a void transcript it never had.
        assert!(!correlation.is_invalid("a"));
        assert_eq!(
            correlation.resolve("a", turn("fallback", 9)),
            turn("fallback", 9)
        );
        assert_eq!(
            correlation.resolve("b", turn("fallback", 9)),
            turn("voice-b", 2)
        );
        assert_eq!(
            correlation.resolve("c", turn("fallback", 9)),
            turn("voice-c", 3)
        );
    }

    #[test]
    fn an_invalidated_item_completes_as_invalid_with_its_context_intact() {
        let mut correlation = TurnCorrelation::default();
        correlation.remember("item_1", turn("voice-1", 1));
        correlation.invalidate("item_1");
        let completed = correlation.complete("item_1", turn("fallback", 9));
        assert_eq!(completed.context, turn("voice-1", 1));
        assert!(completed.invalid);
    }

    #[test]
    fn invalidating_an_unknown_item_still_records_it() {
        // The provider may invalidate an item the gateway never saw open.
        let mut correlation = TurnCorrelation::default();
        correlation.invalidate("ghost");
        assert!(correlation.is_invalid("ghost"));
        correlation.invalidate("");
        assert!(!correlation.is_invalid(""));
    }

    #[test]
    fn clear_drops_both_maps() {
        let mut correlation = TurnCorrelation::default();
        correlation.remember("item_1", turn("voice-1", 1));
        correlation.invalidate("item_1");
        correlation.clear();
        assert!(correlation.is_empty());
        assert!(!correlation.is_invalid("item_1"));
    }
}
