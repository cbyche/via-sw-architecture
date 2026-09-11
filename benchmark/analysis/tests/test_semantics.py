from conftest import event, effect, model_call, route, success_events
from dp00_analysis.semantics import reconstruct_actual


def test_t13_p01_50_to_35_is_decreased_and_t14_reverse_is_not():
    assert "DECREASED" in reconstruct_actual(success_events(before=50, after=35)).derived_predicates
    assert "DECREASED" not in reconstruct_actual(success_events(before=20, after=35)).derived_predicates


def test_p04_p05_p06_actual_reconstruction_needs_no_oracle():
    events = [
        event(1, {"Architecture": {"ClarificationRequested": {"prompt": "which?", "reason": "AMBIGUOUS_REFERENT"}}}),
        event(2, {"Architecture": {"ClarificationResolved": {"request_turn_id": "U1", "response_turn_id": "U2"}}}),
        event(3, {"Architecture": {"ReferentBound": {"referent_role": "SOURCE", "resolved_referent_id": "doc-right"}}}),
        event(4, {"Architecture": {"TaskAssociated": {"task_relation": "FOLLOW_UP"}}}),
        event(5, {"ExecutionStarted": {"invocation": {"capability_id": "network.dns_check", "executor_id": "NetworkAgent"}}}, emitter="AGENT_FIXTURE", corr={"turn_id":"U2","task_id":"T1","execution_id":"X1","dispatch_id":None,"result_id":None,"clarification_id":None}),
        event(6, {"Architecture": "ResultBound"}, corr={"turn_id":"U2","task_id":"T1","execution_id":"X1","dispatch_id":None,"result_id":"R1","clarification_id":None}),
        event(7, {"UsefulOutcomeObserved": {"effect": effect("DNS_CHECK_OBSERVED", "network.dns_check", None, None, "OBSERVED", "NetworkAgent", "T1")}}, emitter="OUTCOME_PROBE"),
    ]
    actual = reconstruct_actual(events)
    assert actual.referent_bindings[0]["object_id"] == "doc-right"
    assert actual.task_associations[0]["task_relation"] == "FOLLOW_UP"
    assert actual.result_bindings[0] == {
        "result_id": "R1",
        "task_id": "T1",
        "parent_task_id": None,
        "child_task_id": None,
        "subgoal_id": None,
        "execution_id": "X1",
    }
    assert actual.clarifications[0]["request_turn_id"] == "U1"
    assert actual.clarifications[0]["response_turn_id"] == "U2"


def test_p09_wrong_candidate_and_p08_p10_failures():
    failed = [event(1, {"Architecture": "ProcessingStarted"}), event(2, {"EpisodeFailed": {"reason": "INVALID_ROUTE"}}, emitter="BENCHMARK")]
    actual = reconstruct_actual(failed, [model_call(candidate="MailAgent", included=False)])
    assert "MAIL_AGENT_REJECTED_OR_NOT_COMMITTED" in actual.derived_predicates
    assert "SAFE_FAILURE:WRONG_CANDIDATE" in actual.derived_predicates
    for reason, predicate in [("MODEL_MALFORMED", "SAFE_FAILURE:MALFORMED"), ("MODEL_TIMEOUT", "SAFE_FAILURE:TIMEOUT"), ("MODEL_NO_RESPONSE", "SAFE_FAILURE:NO_RESPONSE")]:
        actual = reconstruct_actual([event(1, {"EpisodeFailed": {"reason": reason}}, emitter="BENCHMARK")])
        assert predicate in actual.derived_predicates


def test_p07_false_fast_uses_actual_route_invocation_and_effect():
    events = [
        event(1, {"Architecture": {"RouteCommitted": {"route": route("LOCAL_DIRECT", "VIA_LOCAL_VOLUME", "VIA_LOCAL_VOLUME"), "subgoal_id": None}}}),
        event(2, {"ExecutionStarted": {"invocation": {"capability_id":"downloads.organize", "executor_id":"VIA_LOCAL_VOLUME"}}}, emitter="TOOL_FIXTURE", corr={"turn_id":"U1","task_id":"T1","execution_id":"X1","dispatch_id":None,"result_id":None,"clarification_id":None}),
        event(3, {"UsefulOutcomeObserved": {"effect": effect("DOWNLOADS_ORGANIZED", "downloads.organize", None, None, "ORGANIZED", "VIA_LOCAL_VOLUME", "downloads")}}, emitter="OUTCOME_PROBE"),
    ]
    actual = reconstruct_actual(events)
    assert actual.execution_routes[0]["identity"] == "LOCAL_DIRECT:VIA_FAST"
    assert actual.execution_invocations[0]["executor_id"] == "VIA_LOCAL_VOLUME"
    assert "LOCAL_FAST_CLAIMED_DOMAIN_PLANNING_SUCCESS" in actual.derived_predicates


def test_local_media_subgoal_does_not_claim_argo_downloads_effect():
    events = [
        event(
            1,
            {"ExecutionStarted": {"invocation": {"capability_id": "media.pause", "executor_id": "VIA_LOCAL_MEDIA"}}},
            emitter="TOOL_FIXTURE",
            corr={"turn_id": "U1", "task_id": "C1", "execution_id": "X1", "dispatch_id": None, "result_id": None, "clarification_id": None},
        ),
        event(
            2,
            {"UsefulOutcomeObserved": {"effect": effect("DOWNLOADS_ORGANIZED", "downloads.organize", None, None, "ORGANIZED", "ARGO", "downloads")}},
            emitter="OUTCOME_PROBE",
        ),
    ]
    actual = reconstruct_actual(events)
    assert "LOCAL_FAST_CLAIMED_DOMAIN_PLANNING_SUCCESS" not in actual.derived_predicates
