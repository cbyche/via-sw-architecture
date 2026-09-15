//! The Work record's two published vocabularies.
//!
//! `server/src/task/task-manager.mjs` carries both a `status` and a `workState`
//! on every record, and both reach the wire: `status` in the `publicTask`
//! payload and in every `task.*` frame, `workState` as the collapsed projection
//! consumers branch on when they do not care which of the five active statuses
//! a Work is in.
//!
//! The upstream file belongs to `via-work`, which does not exist yet — but the
//! *vocabularies* are already shipped by `via-protocol`, because they are wire
//! values rather than queue behaviour. So these two contracts are owned here,
//! and the rest of `task-manager.mjs` stays pending on `via-work`.

use pretty_assertions::assert_eq;
use via_conformance::expect_contract;
use via_conformance::value::list;
use via_protocol::{WorkState, WorkStatus};

/// Sorted, for comparing two vocabularies as sets.
///
/// Neither of these two vocabularies is a declared, ordered object upstream —
/// `status` is a set of string literals assigned across `task-manager.mjs` and
/// `workState` is computed — so *membership* is the contract and the order the
/// catalogue happens to list them in is not. That is the opposite of the wire
/// event vocabularies, which are declaration-ordered frozen objects and are
/// asserted in order.
fn sorted(values: &[&str]) -> Vec<String> {
    let mut sorted: Vec<String> = values.iter().map(|v| (*v).to_owned()).collect();
    sorted.sort();
    sorted
}

#[test]
fn work_status_values() {
    let contract = expect_contract("state-name", "Work status values");
    let expected = list(&contract.exact_value);
    let shipped: Vec<&str> = WorkStatus::ALL.iter().map(|s| s.as_str()).collect();

    assert_eq!(
        sorted(&expected),
        sorted(&shipped),
        "the nine Work statuses ({})",
        contract.file
    );
    assert_eq!(shipped.len(), 9);
    // VIA declares them in lifecycle order (`docs/architecture.md` §4), which
    // is also the order the catalogue lists them in. Locked because it is
    // VIA's own choice and worth keeping stable, not because upstream declares
    // it.
    assert_eq!(expected, shipped, "declaration order is lifecycle order");

    for status in WorkStatus::ALL {
        assert_eq!(WorkStatus::from_wire(status.as_str()), Some(*status));
    }
}

#[test]
fn work_state_values() {
    let contract = expect_contract("state-name", "workState values");
    let expected = list(&contract.exact_value);
    let shipped: Vec<&str> = WorkState::ALL.iter().map(|s| s.as_str()).collect();

    // Membership only. `workState` is `ACTIVE.has(task.status) ? 'active' :
    // task.status` (`task-manager.mjs:58`) — there is no declared list
    // upstream, so the catalogue's listing order is the surveyor's and not a
    // contract. Asserting it would lock a value nobody promised.
    assert_eq!(
        sorted(&expected),
        sorted(&shipped),
        "the five workState values ({})",
        contract.file
    );
    assert_eq!(shipped.len(), 5);

    // The projection itself: `ACTIVE.has(status) ? 'active' : status`
    // (`task-manager.mjs:58`). Every status must project into the vocabulary,
    // and the five active ones must project onto `active`.
    for status in WorkStatus::ALL {
        let projected = status.state();
        assert!(
            expected.contains(&projected.as_str()),
            "`{}` projects to `{}`, which is not a workState value",
            status.as_str(),
            projected.as_str()
        );
        if WorkStatus::ACTIVE.contains(status) {
            assert_eq!(
                projected,
                WorkState::Active,
                "{} is active",
                status.as_str()
            );
        } else {
            assert_eq!(
                projected.as_str(),
                status.as_str(),
                "a status outside ACTIVE projects to itself"
            );
        }
    }
    assert_eq!(WorkStatus::ACTIVE.len(), 5);
}
