//! Admission, and nothing else.
//!
//! `server/src/task/task-scheduler.mjs` in full — 44 lines, three questions:
//!
//! 1. is the global cap free?
//! 2. is this owner's cap free?
//! 3. is this Work's lane free?
//!
//! It answers `canStart`; it does not run anything, own a queue, or know what a
//! runner is. `docs/architecture.md` §4: *"Admission only: global cap,
//! per-owner cap, **coordinator lane = 1**."*
//!
//! # Queue position is internal
//!
//! `queued` and `running` are both "processing" to a client — there is no
//! position field on the Work record and no API that reports one. The ordering
//! the scheduler enforces is real, and it is deliberately unobservable.
//!
//! # The lane is what makes one voice session serial
//!
//! Every Work a voice session submits carries
//! `laneKey: coordinator:<ownerId>` with `laneLimit: 1`
//! (`server/src/voice/tools/tool-call-handler.mjs:227-228,895-896`), so one
//! owner's Work runs one at a time inside the backend coordinator session —
//! while a *different* owner, or a Work with no lane at all, runs beside it.
//! That is `docs/architecture.md` §11's first invariant, and it is why the
//! manager that drives this scheduler is an owning task rather than a mutex:
//! exclusion is this file's job, **order** is the manager's.

use indexmap::IndexMap;

/// The global concurrency cap.
///
/// Contract — `server/src/task/task-scheduler.mjs:8` and
/// `task-manager.mjs:105` (`maxConcurrent = 4`), catalogued under *TaskManager
/// / TaskScheduler / TaskStore defaults*.
pub const DEFAULT_MAX_CONCURRENT: usize = 4;

/// The per-owner concurrency cap.
///
/// Contract — `server/src/task/task-scheduler.mjs:9` and
/// `task-manager.mjs:106` (`maxConcurrentPerOwner = 2`).
pub const DEFAULT_MAX_CONCURRENT_PER_OWNER: usize = 2;

/// The lane width a Work gets when it names a lane but no width.
///
/// Contract — `server/src/task/task-scheduler.mjs:33`
/// (`limit(task.laneLimit, 1)`), catalogued as *TaskScheduler laneLimit
/// fallback=1*. This is the "coordinator lane = 1" of
/// `docs/architecture.md` §4: the lane is one wide unless something asks for
/// more, and nothing does.
pub const COORDINATOR_LANE_LIMIT: usize = 1;

/// The lane key prefix a coordinator lane is built from.
///
/// `` `coordinator:${this.ownerId}` `` —
/// `server/src/voice/tools/tool-call-handler.mjs:227,895`. It is an in-process
/// key, not a wire value, but it is the same string on both call sites and a
/// third caller that spelled it differently would silently get its own lane.
pub const COORDINATOR_LANE_PREFIX: &str = "coordinator:";

/// The lane key for `owner_id`'s coordinator session.
#[must_use]
pub fn coordinator_lane(owner_id: &str) -> String {
    format!("{COORDINATOR_LANE_PREFIX}{owner_id}")
}

/// `limit(value, fallback)` — `server/src/task/task-scheduler.mjs:1-4`.
///
/// A value that is not a finite positive number falls back. In Rust the "not
/// finite" half is unrepresentable, so what remains is: zero and negatives fall
/// back, which is upstream's `parsed > 0`.
#[must_use]
pub fn limit(value: i64, fallback: usize) -> usize {
    if value > 0 {
        usize::try_from(value).unwrap_or(fallback)
    } else {
        fallback
    }
}

/// What the scheduler needs to know about a Work to admit it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admission {
    /// The Work id — the key of the active set.
    pub work_id: String,
    /// Whose Work it is; the per-owner cap counts these.
    pub owner_id: String,
    /// The serialization lane, or `None` for unlaned Work.
    pub lane_key: Option<String>,
    /// How wide that lane is.
    pub lane_limit: usize,
}

/// The admission controller.
#[derive(Debug, Clone)]
pub struct AdmissionScheduler {
    max_concurrent: usize,
    max_concurrent_per_owner: usize,
    active: IndexMap<String, Admission>,
}

impl Default for AdmissionScheduler {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_CONCURRENT as i64,
            DEFAULT_MAX_CONCURRENT_PER_OWNER as i64,
        )
    }
}

impl AdmissionScheduler {
    /// A scheduler with the two caps, each falling back to its default when it
    /// is not a positive number.
    #[must_use]
    pub fn new(max_concurrent: i64, max_concurrent_per_owner: i64) -> Self {
        Self {
            max_concurrent: limit(max_concurrent, DEFAULT_MAX_CONCURRENT),
            max_concurrent_per_owner: limit(
                max_concurrent_per_owner,
                DEFAULT_MAX_CONCURRENT_PER_OWNER,
            ),
            active: IndexMap::new(),
        }
    }

    /// The global cap in force.
    #[must_use]
    pub const fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// The per-owner cap in force.
    #[must_use]
    pub const fn max_concurrent_per_owner(&self) -> usize {
        self.max_concurrent_per_owner
    }

    /// How many Work items hold a slot.
    #[must_use]
    pub fn active(&self) -> usize {
        self.active.len()
    }

    /// How many active Work items satisfy `predicate`.
    #[must_use]
    pub fn count(&self, predicate: impl Fn(&Admission) -> bool) -> usize {
        self.active.values().filter(|item| predicate(item)).count()
    }

    /// Whether `candidate` may start right now.
    ///
    /// `canStart` — `server/src/task/task-scheduler.mjs:24-35`, in its exact
    /// order. The three tests are asked in sequence and the first refusal wins,
    /// which matters because it decides *which* cap a queued Work is waiting
    /// on.
    #[must_use]
    pub fn can_start(&self, candidate: &Admission) -> bool {
        if self.active.len() >= self.max_concurrent {
            return false;
        }
        if self.count(|item| item.owner_id == candidate.owner_id) >= self.max_concurrent_per_owner {
            return false;
        }
        let Some(lane_key) = candidate.lane_key.as_deref() else {
            return true;
        };
        self.count(|item| item.lane_key.as_deref() == Some(lane_key))
            < limit(
                i64::try_from(candidate.lane_limit).unwrap_or(0),
                COORDINATOR_LANE_LIMIT,
            )
    }

    /// Take a slot. Re-acquiring an id already held replaces its ticket, which
    /// is upstream's `Map.set`.
    pub fn acquire(&mut self, admission: Admission) {
        self.active.insert(admission.work_id.clone(), admission);
    }

    /// Give a slot back. Releasing an id that holds none is a no-op.
    pub fn release(&mut self, work_id: &str) {
        self.active.shift_remove(work_id);
    }

    /// Whether `work_id` currently holds a slot.
    #[must_use]
    pub fn holds(&self, work_id: &str) -> bool {
        self.active.contains_key(work_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Admission, AdmissionScheduler, COORDINATOR_LANE_LIMIT, DEFAULT_MAX_CONCURRENT,
        DEFAULT_MAX_CONCURRENT_PER_OWNER, coordinator_lane, limit,
    };

    fn work(id: &str, owner: &str) -> Admission {
        Admission {
            work_id: id.to_owned(),
            owner_id: owner.to_owned(),
            lane_key: None,
            lane_limit: COORDINATOR_LANE_LIMIT,
        }
    }

    fn laned(id: &str, owner: &str, lane: &str) -> Admission {
        Admission {
            lane_key: Some(lane.to_owned()),
            ..work(id, owner)
        }
    }

    #[test]
    fn the_defaults_are_four_and_two() {
        let scheduler = AdmissionScheduler::default();
        assert_eq!(scheduler.max_concurrent(), DEFAULT_MAX_CONCURRENT);
        assert_eq!(
            scheduler.max_concurrent_per_owner(),
            DEFAULT_MAX_CONCURRENT_PER_OWNER
        );
    }

    #[test]
    fn a_non_positive_cap_falls_back() {
        assert_eq!(limit(0, 4), 4);
        assert_eq!(limit(-1, 4), 4);
        assert_eq!(limit(7, 4), 7);
        let scheduler = AdmissionScheduler::new(0, -3);
        assert_eq!(scheduler.max_concurrent(), DEFAULT_MAX_CONCURRENT);
        assert_eq!(
            scheduler.max_concurrent_per_owner(),
            DEFAULT_MAX_CONCURRENT_PER_OWNER
        );
    }

    #[test]
    fn the_global_cap_is_checked_first() {
        let mut scheduler = AdmissionScheduler::new(2, 8);
        scheduler.acquire(work("a", "one"));
        scheduler.acquire(work("b", "two"));
        assert!(!scheduler.can_start(&work("c", "three")));
        scheduler.release("a");
        assert!(scheduler.can_start(&work("c", "three")));
    }

    #[test]
    fn the_per_owner_cap_bounds_one_owner_without_bounding_another() {
        let mut scheduler = AdmissionScheduler::new(8, 2);
        scheduler.acquire(work("a", "one"));
        scheduler.acquire(work("b", "one"));
        assert!(!scheduler.can_start(&work("c", "one")));
        assert!(scheduler.can_start(&work("c", "two")));
    }

    #[test]
    fn the_coordinator_lane_is_one_wide() {
        let mut scheduler = AdmissionScheduler::new(8, 8);
        let lane = coordinator_lane("owner");
        scheduler.acquire(laned("a", "owner", &lane));
        assert!(!scheduler.can_start(&laned("b", "owner", &lane)));
        assert!(
            scheduler.can_start(&laned("b", "owner", &coordinator_lane("other"))),
            "a different owner's lane is a different lane",
        );
        assert!(
            scheduler.can_start(&work("b", "owner")),
            "unlaned Work is not held by somebody else's lane",
        );
    }

    #[test]
    fn a_zero_lane_limit_falls_back_to_one() {
        let mut scheduler = AdmissionScheduler::new(8, 8);
        let mut candidate = laned("a", "owner", "lane");
        candidate.lane_limit = 0;
        scheduler.acquire(candidate.clone());
        assert!(!scheduler.can_start(&candidate));
    }

    #[test]
    fn a_wider_lane_admits_more() {
        let mut scheduler = AdmissionScheduler::new(8, 8);
        let mut candidate = laned("a", "owner", "lane");
        candidate.lane_limit = 3;
        scheduler.acquire(candidate.clone());
        scheduler.acquire(Admission {
            work_id: "b".to_owned(),
            ..candidate.clone()
        });
        assert!(scheduler.can_start(&candidate));
        scheduler.acquire(Admission {
            work_id: "c".to_owned(),
            ..candidate.clone()
        });
        assert!(!scheduler.can_start(&candidate));
    }

    #[test]
    fn releasing_an_unheld_id_is_a_no_op() {
        let mut scheduler = AdmissionScheduler::default();
        scheduler.release("nothing");
        assert_eq!(scheduler.active(), 0);
        scheduler.acquire(work("a", "one"));
        assert!(scheduler.holds("a"));
        scheduler.acquire(work("a", "one"));
        assert_eq!(scheduler.active(), 1, "re-acquiring replaces the ticket");
        scheduler.release("a");
        assert!(!scheduler.holds("a"));
    }

    #[test]
    fn the_coordinator_lane_key_is_the_upstream_spelling() {
        assert_eq!(coordinator_lane("owner"), "coordinator:owner");
        assert_eq!(coordinator_lane(""), "coordinator:");
    }
}
