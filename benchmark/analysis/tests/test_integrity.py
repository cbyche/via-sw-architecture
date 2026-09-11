from conftest import success_events
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
    assert "MULTIPLE_INITIAL_ROUTE_COMMITS" in codes(events)
    no_terminal = success_events()[:-1]
    assert "MISSING_OR_CONFLICTING_TERMINAL" in codes(no_terminal)
    bad = success_events()
    bad[6]["monotonic_timestamp"] = bad[4]["monotonic_timestamp"] - 1
    bad.sort(key=lambda x: x["monotonic_timestamp"])
    assert "OUTCOME_BEFORE_EXECUTION" in codes(bad)


def test_cross_stream_identity_and_result_reference_detected():
    events = success_events()
    events[0]["run_id"] = "wrong"
    events[5]["product_correlation"]["execution_id"] = "unknown"
    assert {"CROSS_STREAM_IDENTITY_MISMATCH", "INCONSISTENT_RESULT_BINDING"} <= codes(events)
