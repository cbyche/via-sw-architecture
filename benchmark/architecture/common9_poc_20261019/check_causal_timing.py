"""Deterministic evaluator checks, explicitly requested before model freeze.

Run: python3 benchmark/architecture/common9_poc_20261019/check_causal_timing.py
No API/model/parser invocation. Synthetic seconds are not performance evidence.
"""
import json
from causal_timing import build_graph
from evaluate import substantive, expression_match, evaluate, summarize, memory_sample_state

SECOND = 1_000_000_000


class Trace:
    def __init__(self):
        self.trace = {"events": [], "calls": [], "source_nodes": []}

    def source(self, name, start, end, parents=(), role="USER_WAIT"):
        self.trace["source_nodes"].append({"id": name, "start_ns": int(start*SECOND),
            "end_ns": int(end*SECOND), "parents": list(parents), "role": role})
        return name

    def operation(self, name, start, finish, parents=(), role="VIA", enqueue=None, send=None, terminal=None, predecessor=None):
        data = {"node_id": name, "parent_ids": list(parents), "role": role, "stage": name}
        for kind, at in [("OPERATION_START", start), ("OPERATION_END", finish)]:
            self.trace["events"].append({"kind": kind, "event_id": kind+name, "data": data,
                "parent_ids": list(parents), "observed_ns": int(at*SECOND)})
        self.trace["calls"].append({"call_uuid": name, "causal_trace": data,
            "enqueue_ns": int((start if enqueue is None else enqueue)*SECOND),
            "send_ns": int((start if send is None else send)*SECOND),
            "terminal_ns": int((finish if terminal is None else terminal)*SECOND),
            "role": role, "resource_predecessor_call_uuid": predecessor})
        return name

    def endpoint(self, name, at, parents):
        self.trace["events"].append({"kind": "GOAL_COMPLETE", "event_id": name,
            "data": {}, "parent_ids": list(parents), "observed_ns": int(at*SECOND)})
        return "point:" + name

    def measure(self, end, root=0):
        return build_graph(self.trace).measure(end, int(root*SECOND))


def run_checks():
    checks = []
    def equal(name, actual, expected):
        assert actual == expected, (name, actual, expected)
        checks.append({"name": name, "actual": actual, "expected": expected, "status": "PASS"})
    # (a) Serial external service is zero, VIA before and after survives.
    t=Trace();t.source("root",0,0);t.operation("a",0,2,["root"])
    t.source("external",2,10,["a"],"EXTERNAL");t.operation("b",10,11,["external"])
    end=t.endpoint("done",11,["b"])
    equal("a_serial_2_external8_1",t.measure(end)["via_ns"],3*SECOND)
    # (b) Preserve VIA running during the external interval; no wall subtraction.
    t=Trace();t.source("root",0,0);t.operation("a",0,3,["root"])
    t.source("external",0,8,["root"],"EXTERNAL");t.operation("b",8,9,["a","external"])
    equal("b_parallel_3_external8_1",t.measure(t.endpoint("done",9,["b"]))["via_ns"],4*SECOND)
    # (c) Logical state consumption explicitly joins previous VIA and later speech.
    t=Trace();t.source("root",0,0);t.operation("a",0,4,["root"])
    t.source("speech",0,6,["root"]);end=t.endpoint("done",6,["a","speech"])
    equal("c_user6_parallel_via4",t.measure(end)["via_ns"],4*SECOND)
    # (d) A's actual boundary interruption adds the child dependency to resume.
    t=Trace();t.source("root",0,0);t.operation("old",0,2,["root"])
    t.source("new",2,2);t.operation("child",2,5,["old","new"])
    t.operation("resume",5,6,["old","child"])
    equal("d_boundary_child_dependency",t.measure(t.endpoint("done",6,["resume"]))["via_ns"],6*SECOND)
    # (e1) Resource predecessor and full queue compete by MAX, not serial sum.
    t=Trace();t.source("root",0,0);t.operation("owner",0,3,["root"])
    t.operation("queued",0,4,["root"],send=3,predecessor="owner")
    equal("e1_slot_queue_not_double_counted",t.measure(t.endpoint("done",4,["queued"]))["via_ns"],4*SECOND)
    t=Trace();t.source("root",0,0)
    t.operation("owner",0,5,["root"],terminal=3)
    t.operation("queued",0,4,["root"],send=3,predecessor="owner")
    equal("slot_released_before_owner_decode_end",t.measure(t.endpoint("done",4,["queued"]))["via_ns"],4*SECOND)
    t=Trace();t.source("root",0,0);t.operation("owner",0,3,["root"])
    t.operation("queued",2,5,["root"],send=4,predecessor="owner")
    equal("slot_release_to_acquire_scheduling_tail",t.measure(t.endpoint("done",5,["queued"]))["via_ns"],5*SECOND)
    t=Trace();t.source("root",0,0);t.operation("unrelated_owner",0,3,["root"])
    t.source("later_user",0,100,["root"])
    t.operation("current",100,101,["later_user"],predecessor="unrelated_owner")
    equal("historical_idle_slot_owner_not_causal",t.measure(t.endpoint("done",101,["current"]))["via_ns"],SECOND)
    t=Trace();t.source("root",0,0);t.operation("owner",0,3,["root"])
    t.operation("queued",2,5,["root"],send=4.1,predecessor="owner")
    t.trace['calls'][-1].update(before_slot_wait_ns=int(2.2*SECOND),slot_acquired_ns=4*SECOND)
    equal("actual_serialization_wait_acquire_dispatch_split",t.measure(t.endpoint("done",5,["queued"]))["via_ns"],5*SECOND)
    # (e2) RECEIVER service is zero; actual VIA wait for its occupied slot is
    # still a resource queue. A successor consuming its result without queue
    # owes only the following VIA service. Both paths are separately asserted.
    t=Trace();t.source("root",0,0);t.operation("receiver",0,8,["root"],role="RECEIVER")
    t.source("external",0,8,["root","receiver"],"EXTERNAL")
    t.operation("after",8,9,["external"])
    equal("e2_receiver_and_external_overlap_zero",t.measure(t.endpoint("done",9,["after"]))["via_ns"],SECOND)
    t.operation("queued",0,9,["root"],send=8,predecessor="receiver")
    equal("e3_actual_receiver_slot_queue_retained",t.measure(t.endpoint("queued_done",9,["queued"]))["via_ns"],9*SECOND)
    # (f) Correction keeps original root; a new goal clips inherited prior work.
    t=Trace();t.source("root",0,0);t.operation("prior",0,2,["root"])
    t.source("correction",2,6,["prior"]);t.operation("final",6,7,["prior","correction"])
    end=t.endpoint("done",7,["final"])
    equal("f1_corrected_goal_original_root",t.measure(end,0)["via_ns"],3*SECOND)
    equal("f2_new_goal_own_root",t.measure(end,6)["via_ns"],SECOND)
    # Receipt parent can occur inside an external interval. Endpoint join with
    # zero duration avoids interval-order false errors and duplicate8s.
    t=Trace();t.source("root",0,0);t.operation("receiver",2,5,["root"],role="RECEIVER")
    t.source("external",2,10,["receiver"],"EXTERNAL");t.operation("after",10,11,["external"])
    # Actual VIA dispatch delay before receiver start (2s) remains owned VIA.
    equal("receipt_inside_external_interval",t.measure(t.endpoint("done",11,["after"]))["via_ns"],3*SECOND)
    t=Trace();t.operation("a",0,1,["missing"])
    equal("missing_parent_not_fabricated_zero",t.measure(t.endpoint("done",1,["a"]))["status"],"UNRESOLVED")
    t=Trace();t.source("root",0,0);t.operation("a",0,2,["root"])
    t.operation("bad",1,3,["a"])
    equal("nonmonotonic_dependency_rejected",t.measure(t.endpoint("done",3,["bad"]))["status"],"UNRESOLVED")
    equal("generic_ack_excluded",substantive({"text":"알겠습니다."})[0],False)
    equal("generic_intention_excluded",substantive({"text":"지정한 요청의 실제 내용을 확인해 답변하겠습니다."})[0],False)
    equal("unspecified_work_in_progress_excluded",substantive({"text":"요청하신 작업을 처리하고 있습니다."})[0],False)
    equal("raw_user_echo_excluded",substantive({"text":"이 문단을 수정해줘."},"이 문단을 수정해줘.")[0],False)
    equal("unresolved_deictic_excluded",substantive({"text":"이 문단을 수정하겠습니다."})[0],False)
    equal("resolved_target_card_included",substantive({"text":"이 문단을 수정하겠습니다.","targets":["DOC-A:v1:P2"]})[0],True)
    equal("wrong_but_concrete_answer_keeps_time",substantive({"text":"Cedar의 가격은 월999달러입니다."})[0],True)
    equal("legitimate_clarification_included",substantive({"text":"TA와 TB 중 어느 업무를 승인하셨습니까?"})[0],True)
    equal("case_particle_match",expression_match("이 표들을","이 표들"),True)
    equal("plural_scope_not_erased",expression_match("이 표들","이 표"),False)
    # Fixed planned denominators persist even for a failed/no-output episode.
    r=evaluate({"episode":"S08","configuration":"AAAA","rep":1,"phase":"OFFLINE",
        "events":[],"calls":[],"source_nodes":[],"provided_sources":{},"memory":[]})
    equal("no_output_fixed_episode_responses",len(r["qa01"]),3)
    equal("no_output_fixed_episode_goals",len(r["qa03"]),1)
    equal("no_output_fixed_episode_bindings",len(r["qa02"]),4)
    summary=summarize([r])["OFFLINE:AAAA"]
    equal("campaign_response_denominator",summary["qa01"]["planned"],215)
    equal("campaign_goal_denominator",summary["qa03"]["planned"],115)
    equal("campaign_binding_denominator",summary["qa02"]["planned"],75)
    duplicate=summarize([r,r])["OFFLINE:AAAA"]
    equal("duplicate_population_not_scored",duplicate["qa02"]["fixed_denominator_accuracy"],None)
    invalid=evaluate({"episode":"S08","events":[],"calls":[],"memory":[{
        "at_ns":0,"concurrent_sum_bytes":0,"all_registered_observed":False,
        "registered_processes":[{"pid":1,"role":"python_bridge"},{"pid":2,"role":"rust_candidate"}],
        "processes":[]}],"host_failures":[{"kind":"MEMORY_SAMPLER_FAILED"}]})
    equal("failed_ps_zero_not_a_memory_measurement",invalid["qa06"]["status"],"INVALID")
    equal("failed_ps_zero_not_a_peak",invalid["qa06"]["peak_concurrent_sum_bytes"],None)
    unknown=evaluate({"episode":"S08","events":[],"memory":[],"calls":[{
        "cost_usd":None,"reservation_usd":0.03,"input_tokens":None,"output_tokens":None}]})
    equal("unknown_usage_not_free",unknown["qa05"]["status"],"INCOMPLETE_COST")
    equal("unknown_reservation_retained",unknown["qa05"]["unknown_reservation_usd"],0.03)
    missing=evaluate({"episode":"S08","events":[{"kind":"GOAL_COMPLETE","event_id":"done",
        "observed_ns":11,"data":{"origin_event_id":"S08-U01","stimulus_id":"S08-A01","revision":1}}],
        "calls":[],"memory":[],"provided_sources":{"S08-U01":{"source_ns":0},"S08-A01":{"source_ns":10}}})
    equal("all_goal_member_sources_required",missing["qa03"][0]["status"],"NOT_OBSERVED")
    t=Trace();t.source("root",0,0);end=t.endpoint("unconnected",1,[])
    equal("missing_original_source_dependency_not_zero",build_graph(t.trace).measure(end,0,"root")["status"],"UNRESOLVED")
    boundary={"at_ns":0,"concurrent_sum_bytes":100,"all_registered_observed":False,
        "registered_processes":[{"pid":1,"role":"python_bridge"},{"pid":2,"role":"rust_candidate"}],
        "processes":[{"pid":1,"rss_bytes":100}],"candidate_lifecycle":{"pid":2,"alive_at_sample_start":True,
        "exit_confirmed_by_sample_end":True,"exitcode":0},"confirmed_exited_pids":[2]}
    equal("confirmed_exit_boundary_not_zero_normal_sample",memory_sample_state(boundary),"CONFIRMED_EXIT_BOUNDARY")
    boundary['registered_processes'].append({"pid":3,"role":"actual_parser_helper"})
    equal("missing_live_helper_not_exit_waived",memory_sample_state(boundary),"INVALID")
    boundary['registered_processes'].pop();boundary['candidate_lifecycle']['exit_confirmed_by_sample_end']=False
    equal("uncertain_exit_remains_invalid",memory_sample_state(boundary),"INVALID")
    boundary['candidate_lifecycle']['exit_confirmed_by_sample_end']=True;boundary['ps_returncode']=1
    equal("ps_failure_not_exit_waived",memory_sample_state(boundary),"INVALID")
    startup={"at_ns":10,"sample_end_ns":11,"concurrent_sum_bytes":100,"all_registered_observed":True,
        "registered_processes":[{"pid":1,"role":"python_bridge"},{"pid":2,"role":"rust_candidate"}],
        "processes":[{"pid":1,"rss_bytes":100},{"pid":2,"rss_bytes":0}],"ps_returncode":0,
        "process_start_metadata":{"2":{"role":"rust_candidate","spawn_begin_ns":1,"spawn_return_ns":2,"registered_at_ns":3}}}
    equal("startup_zero_requires_later_positive",memory_sample_state(startup,{1:10,2:20}),"STARTUP_ZERO_RSS")
    equal("startup_never_positive_is_incomplete",memory_sample_state(startup,{1:10}),"INVALID")
    equal("active_zero_after_positive_is_invalid",memory_sample_state(startup,{1:1,2:5}),"INVALID")
    startup['process_start_metadata']={}
    equal("startup_zero_without_spawn_proof_invalid",memory_sample_state(startup,{1:10,2:20}),"INVALID")
    boundary['ps_returncode']=0
    equal("never_positive_exit_boundary_is_incomplete",memory_sample_state(boundary,{1:0}),"INVALID")
    return checks


if __name__ == "__main__":
    print(json.dumps({"scope":"synthetic causal/endpoint evaluator checks, no API", "checks":run_checks()},ensure_ascii=False,indent=2))
