from conftest import correlation, event, route, success_events
from dp00_analysis.validation import cross_stream_issues
from conftest import provenance


def codes(events):
    return {x.code for x in cross_stream_issues(provenance(event_count=len(events)), events, [], [])}


def test_duplicate_sequence_and_timestamp_reversal_detected():
    events = success_events()
    events[1]["sequence_number"] = events[0]["sequence_number"]
    events[2]["monotonic_timestamp"] = 0
    assert {"DUPLICATE_CANONICAL_SEQUENCE", "TIMESTAMP_REVERSAL"} <= codes(events)


def test_multiple_commit_missing_terminal_and_outcome_order_detected():
    events = success_events()
    events.insert(4, dict(events[3], event_id="duplicate", sequence_number=40))
    assert "DUPLICATE_INITIAL_ROUTE_COMMIT" in codes(events)
    no_terminal = success_events()[:-1]
    assert "MISSING_OR_CONFLICTING_TERMINAL" in codes(no_terminal)


def test_terminal_must_be_final_for_closed_world_evidence():
    events = success_events()
    events[-1], events[-2] = events[-2], events[-1]
    for index, event in enumerate(events):
        event["sequence_number"] = index
        event["monotonic_timestamp"] = index
    assert "TERMINAL_NOT_FINAL" in codes(events)
    bad = success_events()
    bad[6]["monotonic_timestamp"] = bad[4]["monotonic_timestamp"] - 1
    bad.sort(key=lambda x: x["monotonic_timestamp"])
    assert "OUTCOME_BEFORE_EXECUTION" in codes(bad)


def test_cross_stream_identity_and_result_reference_detected():
    events = success_events()
    events[0]["run_id"] = "wrong"
    events[5]["product_correlation"]["execution_id"] = "unknown"
    assert {"CROSS_STREAM_IDENTITY_MISMATCH", "INCONSISTENT_RESULT_BINDING"} <= codes(events)


def test_compound_commits_are_unique_per_subgoal_and_cross_plan_barrier():
    events = success_events(scenario="P12")
    first = event(
        4,
        {"Architecture": {"RouteCommitted": {"route": route(), "subgoal_id": "S1"}}},
        timestamp=1_300_000,
        scenario="P12",
        corr=correlation(task="C1") | {
            "parent_task_id": "PARENT", "child_task_id": "C1", "subgoal_id": "S1"
        },
    )
    second = event(
        5,
        {"Architecture": {"RouteCommitted": {"route": route(), "subgoal_id": "S2"}}},
        timestamp=1_350_000,
        scenario="P12",
        corr=correlation(task="C2") | {
            "parent_task_id": "PARENT", "child_task_id": "C2", "subgoal_id": "S2"
        },
    )
    events[3:4] = [first, second]
    for index, row in enumerate(events, 1):
        row["sequence_number"] = index
    assert "DUPLICATE_INITIAL_ROUTE_COMMIT" not in codes(events)
    assert "EXECUTION_BEFORE_INITIAL_ROUTE_PLAN_BARRIER" not in codes(events)

    premature = [dict(row) for row in events]
    premature[4], premature[5] = premature[5], premature[4]
    for index, row in enumerate(premature, 1):
        row["sequence_number"] = index
        row["monotonic_timestamp"] = index * 10
    assert "EXECUTION_BEFORE_INITIAL_ROUTE_PLAN_BARRIER" in codes(premature)

    duplicate = [dict(row) for row in events]
    duplicate[4] = {
        **duplicate[4],
        "event": {
            "Architecture": {
                "RouteCommitted": {"route": route(), "subgoal_id": "S1"}
            }
        },
    }
    assert "DUPLICATE_INITIAL_ROUTE_COMMIT" in codes(duplicate)
