//! One utterance, N intents.
//!
//! `docs/architecture.md` §4 draws this box and says exactly what it is:
//!
//! > The reference architecture draws **IntentSet fan-out** — one utterance
//! > yielding N intents — where qwen delegates a single objective string.
//! > […] VIA widens `INTENT_SCHEMA` to an array and returns
//! > `Vec<DelegationIntent>` […] and the Scheduler admits each intent
//! > independently.
//!
//! # The constraint that makes this safe to ship
//!
//! Upstream has 707 catalogued contracts and every one of them was written
//! against a **single** objective string. So the rule here is that a
//! one-element [`IntentSet`] must produce exactly what upstream produces —
//! byte for byte, including the submission key, which is what
//! `via-work`'s duplicate-submission suppression is keyed on. Widening a
//! contract is only safe if the narrow case is unchanged, and
//! [`IntentSet::work_requests`] is where that is enforced:
//!
//! * one intent → one [`NewWork`] with the caller's own submission key,
//!   indistinguishable from the single-objective path;
//! * N intents → N [`NewWork`]s, each with a **suffixed** key, so an intent can
//!   never collide with a single-objective submission and duplicate suppression
//!   keeps working in both worlds.
//!
//! # Independent admission, one lane
//!
//! Every intent carries [`via_work::coordinator_lane`] with width 1, exactly as
//! upstream's single objective does
//! (`server/src/voice/tools/tool-call-handler.mjs:227-228`). That is not a
//! contradiction of *"admitted independently"*: admission is per-Work — each
//! intent gets its own `work_id`, its own cancellation scope and its own
//! retention — while the lane keeps one owner's Work **serial inside the
//! backend coordinator session**, which is `docs/architecture.md` §11's first
//! invariant and is a property of the session, not of the utterance.
//!
//! Cancelling one intent therefore cancels one Work. `docs/architecture.md`
//! §11 records the ARGO defect this avoids: a shared cancellation token let one
//! barge-in kill independent background work, and here there is no shared
//! token to capture.

use via_downstream::text::clean;
use via_work::{NewWork, coordinator_lane};

/// The lane width one owner's coordinator lane has.
///
/// **External contract** — `tool-call-handler.mjs:228` (`laneLimit: 1`), which
/// is `docs/architecture.md` §4's *"coordinator lane = 1"*. Restated as a name
/// here rather than as a bare `1` at the call site.
pub const COORDINATOR_LANE_LIMIT: usize = 1;

/// What separates a submission key from an intent's index.
///
/// VIA's own — there is no upstream equivalent because upstream has no fan-out.
/// It is deliberately a character that cannot appear in a `work_<uuid>`, so a
/// suffixed key can never be mistaken for an unsuffixed one.
pub const INTENT_KEY_SEPARATOR: char = '#';

/// A set of intents that could not be built.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IntentSetError {
    /// An utterance yielded no intent at all.
    ///
    /// Not the same as *"the user asked a question"*: a direct answer never
    /// reaches this type, because `docs/architecture.md` §4 answers it on the
    /// fast path with no Work created. An empty set here means a planner
    /// produced nothing, which is a planner bug rather than a user outcome.
    #[error("an intent set must carry at least one intent")]
    Empty,

    /// One intent's objective is blank.
    #[error("intent {index} has a blank objective")]
    BlankObjective {
        /// Which intent, counting from zero.
        index: usize,
    },
}

impl IntentSetError {
    /// A stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Empty => "VIA_COORDINATOR_INTENT_SET_EMPTY",
            Self::BlankObjective { .. } => "VIA_COORDINATOR_INTENT_BLANK",
        }
    }
}

/// One thing the user asked for.
///
/// Upstream's `objective` is the whole of this: *"the conservative summary the
/// voice frontend made of what the user said"*
/// ([`via_i18n::keys::COORDINATOR_INTERFACE_NOTE`]). The type exists so the
/// fan-out has something to be a set **of**, and so an intent can grow a field
/// without every caller changing shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intent {
    objective: String,
}

impl Intent {
    /// An intent to pursue `objective`.
    ///
    /// # Errors
    ///
    /// [`IntentSetError::BlankObjective`] when `objective` cleans to nothing.
    /// The index is `0` here and is rewritten by
    /// [`IntentSet::new`] to the intent's real position.
    pub fn new(objective: &str) -> Result<Self, IntentSetError> {
        let objective = clean(objective);
        if objective.is_empty() {
            return Err(IntentSetError::BlankObjective { index: 0 });
        }
        Ok(Self {
            objective: objective.to_owned(),
        })
    }

    /// The objective, cleaned.
    #[must_use]
    pub fn objective(&self) -> &str {
        &self.objective
    }
}

/// What one utterance asked for, in order.
///
/// Never empty: [`IntentSet::new`] refuses an empty vector, and there is no
/// `Default`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentSet {
    intents: Vec<Intent>,
}

impl IntentSet {
    /// The single-objective set — upstream's whole world.
    ///
    /// # Errors
    ///
    /// [`IntentSetError::BlankObjective`] when `objective` cleans to nothing.
    pub fn single(objective: &str) -> Result<Self, IntentSetError> {
        Ok(Self {
            intents: vec![Intent::new(objective)?],
        })
    }

    /// A set of N intents, in the order the planner produced them.
    ///
    /// # Errors
    ///
    /// [`IntentSetError::Empty`] for an empty vector.
    pub fn new(intents: Vec<Intent>) -> Result<Self, IntentSetError> {
        if intents.is_empty() {
            return Err(IntentSetError::Empty);
        }
        Ok(Self { intents })
    }

    /// Build a set from objectives, reporting which one was blank.
    ///
    /// # Errors
    ///
    /// [`IntentSetError::Empty`] or [`IntentSetError::BlankObjective`] carrying
    /// the offending index.
    pub fn from_objectives(objectives: &[&str]) -> Result<Self, IntentSetError> {
        if objectives.is_empty() {
            return Err(IntentSetError::Empty);
        }
        let intents = objectives
            .iter()
            .enumerate()
            .map(|(index, objective)| {
                Intent::new(objective).map_err(|_| IntentSetError::BlankObjective { index })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { intents })
    }

    /// How many intents there are. Always at least one.
    #[must_use]
    pub fn len(&self) -> usize {
        self.intents.len()
    }

    /// Always `false`; present because clippy asks for it beside `len`.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// Whether this is the single-objective case.
    #[must_use]
    pub fn is_single(&self) -> bool {
        self.intents.len() == 1
    }

    /// The intents, in planner order.
    #[must_use]
    pub fn intents(&self) -> &[Intent] {
        &self.intents
    }

    /// The single objective, for the case upstream's contracts were written
    /// against.
    ///
    /// `None` for a fan-out: there is no *the* objective then, and returning
    /// the first would let a caller silently drop the rest.
    #[must_use]
    pub fn objective(&self) -> Option<&str> {
        self.is_single().then(|| self.intents[0].objective())
    }

    /// The submission key for intent `index`.
    ///
    /// **The single-intent case returns `base` unchanged**, which is what makes
    /// a one-element set byte-identical to upstream: `via-work`'s
    /// duplicate-submission suppression is keyed on this string, so a suffix
    /// here would make every re-submission look new.
    #[must_use]
    pub fn submission_key(&self, base: &str, index: usize) -> String {
        let base = clean(base);
        if self.is_single() || base.is_empty() {
            return base.to_owned();
        }
        format!("{base}{INTENT_KEY_SEPARATOR}{index}")
    }

    /// One [`NewWork`] per intent, each ready for
    /// [`via_work::WorkManager::create`].
    ///
    /// Every request carries `owner_id`'s coordinator lane at width
    /// [`COORDINATOR_LANE_LIMIT`], so the whole set runs one at a time inside
    /// the backend coordinator session while a *different* owner's Work runs
    /// beside it.
    ///
    /// `submission_key` may be blank, in which case no key is set at all and
    /// duplicate suppression does not apply — upstream's behaviour for a
    /// submission that named none.
    #[must_use]
    pub fn work_requests(&self, submission: &IntentSubmission<'_>) -> Vec<NewWork> {
        let lane = coordinator_lane(submission.owner_id);
        self.intents
            .iter()
            .enumerate()
            .map(|(index, intent)| {
                let mut request = NewWork::new(intent.objective(), submission.owner_id)
                    .lane(&lane, COORDINATOR_LANE_LIMIT);
                if !submission.session_id.is_empty() {
                    request = request.session(submission.session_id);
                }
                if !submission.turn_id.is_empty() {
                    request = request.turn(submission.turn_id);
                }
                let key = self.submission_key(submission.submission_key, index);
                if !key.is_empty() {
                    request = request.submission_key(&key);
                }
                request
            })
            .collect()
    }
}

/// The identity every intent in one utterance shares.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IntentSubmission<'a> {
    /// Whose utterance this was.
    pub owner_id: &'a str,
    /// Which voice session.
    pub session_id: &'a str,
    /// Which turn.
    pub turn_id: &'a str,
    /// The duplicate-suppression key, or blank for none.
    pub submission_key: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn submission() -> IntentSubmission<'static> {
        IntentSubmission {
            owner_id: "owner-one",
            session_id: "voice-one",
            turn_id: "turn-one",
            submission_key: "voice-one:turn-one",
        }
    }

    #[test]
    fn a_single_intent_set_is_the_upstream_case() {
        let set = IntentSet::single("  继续修改此前讨论的页面  ").expect("valid");
        assert!(set.is_single());
        assert_eq!(set.len(), 1);
        assert_eq!(set.objective(), Some("继续修改此前讨论的页面"));
        assert_eq!(
            set.submission_key("voice-one:turn-one", 0),
            "voice-one:turn-one",
            "the key is untouched, so duplicate suppression is unchanged",
        );
    }

    #[test]
    fn a_single_intent_produces_exactly_one_laned_work_request() {
        let set = IntentSet::single("build the thing").expect("valid");
        let requests = set.work_requests(&submission());
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].objective, "build the thing");
        assert_eq!(requests[0].owner_id, "owner-one");
        assert_eq!(requests[0].session_id.as_deref(), Some("voice-one"));
        assert_eq!(requests[0].turn_id.as_deref(), Some("turn-one"));
        assert_eq!(
            requests[0].submission_key.as_deref(),
            Some("voice-one:turn-one"),
        );
        assert_eq!(
            requests[0].lane_key.as_deref(),
            Some(coordinator_lane("owner-one").as_str()),
        );
        assert_eq!(requests[0].lane_limit, COORDINATOR_LANE_LIMIT);
    }

    #[test]
    fn a_fan_out_suffixes_every_key_including_the_first() {
        let set = IntentSet::from_objectives(&["one", "two", "three"]).expect("valid");
        assert!(!set.is_single());
        assert_eq!(set.objective(), None, "a fan-out has no single objective");
        assert_eq!(set.submission_key("k", 0), "k#0");
        assert_eq!(set.submission_key("k", 2), "k#2");
        let requests = set.work_requests(&submission());
        assert_eq!(requests.len(), 3);
        let keys: Vec<String> = (0..3).map(|index| set.submission_key("k", index)).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 3, "no two intents share a key");
        assert!(
            keys.iter().all(|key| key != "k"),
            "a fanned-out key can never be mistaken for a single-objective one",
        );
    }

    #[test]
    fn an_empty_submission_key_stays_empty_for_any_arity() {
        let single = IntentSet::single("one").expect("valid");
        let many = IntentSet::from_objectives(&["one", "two"]).expect("valid");
        assert_eq!(single.submission_key("", 0), "");
        assert_eq!(many.submission_key("   ", 1), "");
        let requests = many.work_requests(&IntentSubmission {
            submission_key: "",
            ..submission()
        });
        assert_eq!(requests.len(), 2);
        assert!(
            requests
                .iter()
                .all(|request| request.submission_key.is_none())
        );
    }

    #[test]
    fn every_intent_in_a_fan_out_shares_the_owners_lane() {
        let set = IntentSet::from_objectives(&["one", "two"]).expect("valid");
        let requests = set.work_requests(&submission());
        let lane = coordinator_lane("owner-one");
        for request in &requests {
            assert_eq!(request.lane_key.as_deref(), Some(lane.as_str()));
            assert_eq!(request.lane_limit, COORDINATOR_LANE_LIMIT);
        }
        assert_eq!(
            requests
                .iter()
                .map(|request| request.submission_key.clone())
                .collect::<Vec<_>>(),
            [
                Some("voice-one:turn-one#0".to_owned()),
                Some("voice-one:turn-one#1".to_owned()),
            ],
        );
    }

    #[test]
    fn a_blank_objective_is_refused_with_its_index() {
        assert_eq!(
            IntentSet::single("   ").expect_err("blank"),
            IntentSetError::BlankObjective { index: 0 },
        );
        assert_eq!(
            IntentSet::from_objectives(&["one", "  ", "three"]).expect_err("blank"),
            IntentSetError::BlankObjective { index: 1 },
        );
        assert_eq!(
            IntentSet::from_objectives(&[]).expect_err("empty"),
            IntentSetError::Empty,
        );
        assert_eq!(
            IntentSet::new(Vec::new()).expect_err("empty"),
            IntentSetError::Empty,
        );
        assert_eq!(
            IntentSetError::Empty.code(),
            "VIA_COORDINATOR_INTENT_SET_EMPTY",
        );
        assert_eq!(
            IntentSetError::BlankObjective { index: 0 }.code(),
            "VIA_COORDINATOR_INTENT_BLANK",
        );
    }

    #[test]
    fn a_set_is_never_empty() {
        let set = IntentSet::single("one").expect("valid");
        assert!(!set.is_empty());
        assert_eq!(set.intents().len(), 1);
    }
}
