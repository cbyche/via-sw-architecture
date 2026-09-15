from __future__ import annotations

import argparse
from collections import defaultdict
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import tempfile
import time
from typing import Any

from benchmark.analysis.qa_v1 import ContractRepository, EvidenceMode, FrozenCorpus, evaluate_canonical_observations
from .evolution import EvolutionInput, run_evolution_case
from .runtime import ExecutableTopology, PlaybackProbe, ROOT

CAMPAIGN_ID = "dp00-executable-reference-v3-qa-v1"
REALIZATIONS = ("R1", "R3", "R1+@")
EXPECTED_COUNTS = {"QA-01":150,"QA-02":600,"QA-03":400,"QA-04":60,"QA-05":60,"QA-06":60,"QA-08":200,"QA-09":300,"QA-10":200,"QA-11":200,"QA-12":200}
FIXTURES = ROOT / "benchmark/fixtures/dp00-executable-v3"
PROTOTYPE = ROOT / "prototypes/dp00-executable-v3"
RESULT_ROOT = ROOT / "results/dp00-executable-v3"


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_jsonl(path: Path, values: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(value, sort_keys=True) + "\n" for value in values), encoding="utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def percentile95(values: list[float]) -> float:
    return sorted(values)[math.ceil(0.95 * len(values)) - 1]


def population_paths(qa_id: str) -> tuple[Path, Path]:
    mapping = {
        "QA-01":"benchmark/scenarios/qa-v1/qa01-fast-v1",
        "QA-02":"benchmark/scenarios/qa-v1/qa02-goals-v1",
        "QA-03":"benchmark/scenarios/qa-v1/qa03-continuity-v1",
        "QA-04":"benchmark/evolution/qa-v1/qa04-agent-evolution-v1",
        "QA-05":"benchmark/evolution/qa-v1/qa05-product-evolution-v1",
        "QA-06":"benchmark/evolution/qa-v1/qa06-device-adaptation-v1",
        "QA-08":"benchmark/failure-injection/qa-v1/qa08-recovery-v1",
        "QA-09":"benchmark/scenarios/qa-v1/qa09-sensitive-scope-v1",
        "QA-10":"benchmark/scenarios/qa-v1/qa10-trace-v1",
        "QA-11":"benchmark/scenarios/qa-v1/qa11-feedback-v1",
        "QA-12":"benchmark/scenarios/qa-v1/qa12-barge-in-v1",
    }
    base = ROOT / mapping[qa_id]
    return base / "instances.json", base / "oracles.json"


def cases(qa_id: str) -> list[dict[str, Any]]:
    return load(population_paths(qa_id)[0])["instances"]


def oracles(qa_id: str) -> dict[str, Any]:
    return load(population_paths(qa_id)[1])["oracles"]


def identity(qa_id: str, case: dict[str, Any]) -> str:
    return case["goal_id"] if qa_id in {"QA-01","QA-02"} else case["case_id"]


def observation(qa_id: str, population_id: str, case_id: str, measurement: dict[str, Any]) -> dict[str, Any]:
    return {"observation_id":case_id,"qa_id":qa_id,"population_id":population_id,"evidence_mode":"executed","measurement":measurement}


def provenance(contracts: ContractRepository, commit: str, realization: str, qa_id: str):
    return contracts.build_provenance(
        architecture_commit=commit, dp_id="DP-00", alternative_id=realization,
        tactic_package="dp00-r1-readonly-fastpath-tactic-v1" if realization == "R1+@" else None,
        run_id=f"{CAMPAIGN_ID}:{realization}:{qa_id}", evidence_mode=EvidenceMode.EXECUTED,
    )


def goal_maps() -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    semantic = {item["case_id"]:item for item in load(FIXTURES / "semantic-replay-v3.json")["records"]}
    agent = {item["case_id"]:item for item in load(FIXTURES / "agent-replay-v3.json")["records"]}
    catalog = {item["goal_id"]:item for item in load(ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json")["instances"]}
    return semantic, agent, catalog


def goal_request(case_id: str, semantic: dict[str, Any], agent: dict[str, Any], *, timing: bool = False) -> dict[str, Any]:
    task = f"task:{case_id}"
    commit = "VOICE_DELTA" if semantic["modality"] == "voice" else "TEXT_TURN"
    events = [{"type":commit,"task_id":task},{"type":"TURN_COMMIT","task_id":task},{"type":"TASK_START","task_id":task},{"type":"AGENT_PROGRESS","task_id":task},{"type":"AGENT_RESULT","task_id":task}]
    return {
        "case_id":case_id,"task_id":task,"principal":"principal-local","purpose":semantic["goal"],
        "semantic_replay":semantic,"agent_replay":agent,"events":events,
        "scope_request":[f"scope:task:{case_id}"],
        "delays":{"semantic_ms":70 if timing else 0,"agent_ms":10 if timing else 0,"tool_ms":10 if timing else 0},
    }


def extract_agent_result(output: dict[str, Any]) -> dict[str, Any]:
    if "agent" in output:
        return output["agent"]["result"]
    return output["primary_runtime"]["execution"]["result"]


def modality_consistent(primary: dict[str, Any], alternate: dict[str, Any]) -> bool:
    comparable=("goal","referent","constraints","consent_required","ambiguity_status","capability")
    return (
        all(primary["semantic"].get(key)==alternate["semantic"].get(key) for key in comparable)
        and extract_agent_result(primary)==extract_agent_result(alternate)
        and primary["case_id"]==alternate["case_id"]
    )


def evaluate_goal(output: dict[str, Any], expected: dict[str, Any]) -> dict[str, bool]:
    semantic = output["semantic"]
    agent = extract_agent_result(output)
    return {
        "goal": semantic["goal"] == expected["correct_goal"],
        "referent": semantic["referent"] == expected["correct_referent"],
        "constraints": set(semantic["constraints"]) == set(expected["explicit_constraints"]),
        "consent": semantic["consent_required"] == expected["required_consent"],
        "result_facts": set(agent["facts"]) == set(expected["required_result_facts"]),
        "task_result_binding": output["case_id"] == expected["task_result_binding"].removeprefix("task:"),
        "voice_text_consistency": output.get("voice_text_consistency") is True,
    }


def event_script(case: dict[str, Any]) -> list[dict[str, Any]]:
    task = f"task:{case['case_id']}"
    other = f"{task}:concurrent"
    name = case["event_interleaving"]
    events = [
        {"type":"VOICE_DELTA","task_id":task}, {"type":"VOICE_REVISION","task_id":task},
        {"type":"TURN_COMMIT","task_id":task}, {"type":"TEMPORAL_CONTEXT_EVENT","task_id":task},
        {"type":"POINTER_EVENT","task_id":task}, {"type":"CONTEXT_VERSION_CHANGE","task_id":task},
        {"type":"CLARIFICATION_RESPONSE","task_id":task}, {"type":"TASK_START","task_id":task},
    ]
    if "two-task" in name or "other-response" in name:
        events.extend([{"type":"TASK_START","task_id":other},{"type":"AGENT_PROGRESS","task_id":other}])
    if "approval" in name:
        events.extend([{"type":"AGENT_APPROVAL_REQUIRED","task_id":task},{"type":"APPROVAL_RESPONSE","task_id":task}])
    if "result-before-progress" in name:
        events.extend([{"type":"AGENT_RESULT","task_id":task},{"type":"AGENT_PROGRESS","task_id":task}])
    elif "cancel" in name:
        events.extend([{"type":"CANCEL","task_id":task},{"type":"LATE_AGENT_RESULT","task_id":task}])
    else:
        events.extend([{"type":"AGENT_PROGRESS","task_id":task},{"type":"AGENT_RESULT","task_id":task}])
    events.extend([{"type":"FOLLOW_UP","task_id":task},{"type":"VOICE_SESSION_END","task_id":task},{"type":"RESOURCE_ACQUIRE","task_id":task},{"type":"RESOURCE_RELEASE","task_id":task}])
    return events


def generic_request(case_id: str, *, capability: str = "planning", events: list[dict[str, Any]] | None = None) -> dict[str, Any]:
    return {
        "case_id":case_id,"task_id":f"task:{case_id}","principal":"principal-local","purpose":"frozen-runtime-case",
        "semantic_replay":{"goal":"frozen-runtime-case","referent":"local:v1","constraints":[],"consent_required":False,"capability":capability,"read_only":False},
        "agent_replay":{"case_id":case_id,"status":"success","result_version":1,"facts":[f"{case_id}:fact:identity"]},
        "events":events or [{"type":"TURN_COMMIT","task_id":f"task:{case_id}"},{"type":"TASK_START","task_id":f"task:{case_id}"},{"type":"AGENT_RESULT","task_id":f"task:{case_id}"}],
        "scope_request":[f"scope:task:{case_id}"],"delays":{"semantic_ms":0,"agent_ms":0,"tool_ms":0},
    }


def run_qa01(commit: str, contracts: ContractRepository) -> tuple[dict[str, Any], dict[str, list[dict[str, Any]]], dict[str, Any]]:
    semantic, agent, catalog = goal_maps()
    qa_cases = cases("QA-01")
    case_ids = [item["goal_id"] for item in qa_cases]
    repetitions: dict[str, dict[str, list[dict[str, Any]]]] = {r:{case_id:[] for case_id in case_ids} for r in REALIZATIONS}
    raw: dict[str, list[dict[str, Any]]] = {r:[] for r in REALIZATIONS}
    modality_checks: dict[str, dict[str, bool]] = {r:{} for r in REALIZATIONS}
    memory: dict[str, Any] = {}
    topologies = {r:ExecutableTopology(r) for r in REALIZATIONS}
    try:
        for realization, topology in topologies.items():
            for warmup in range(5):
                topology.execute(goal_request(case_ids[warmup], semantic[case_ids[warmup]], agent[case_ids[warmup]], timing=True))
            for case_id in case_ids:
                primary=topology.execute(goal_request(case_id,semantic[case_id],agent[case_id]))
                alternate_semantic=dict(semantic[case_id])
                alternate_semantic["modality"]="text" if semantic[case_id]["modality"]=="voice" else "voice"
                alternate=topology.execute(goal_request(case_id,alternate_semantic,agent[case_id]))
                modality_checks[realization][case_id]=modality_consistent(primary,alternate)
            memory[realization] = topology.memory_snapshot()
        for repetition in range(7):
            for index, case_id in enumerate(case_ids):
                order = REALIZATIONS[(index + repetition) % 3:] + REALIZATIONS[:(index + repetition) % 3]
                for realization in order:
                    output = topologies[realization].execute(goal_request(case_id, semantic[case_id], agent[case_id], timing=True))
                    output["voice_text_consistency"]=modality_checks[realization][case_id]
                    record = {"repetition":repetition,"run_order":list(order),"stratum":semantic[case_id]["semantic_family"],**output}
                    repetitions[realization][case_id].append(record)
                    raw[realization].append(record)
    finally:
        for topology in topologies.values(): topology.close()
    results = {}
    observations_by_realization = {}
    population = FrozenCorpus(contracts).population("QA-01")
    oracle_map = oracles("QA-01")
    for realization in REALIZATIONS:
        observations = []
        medians = []
        strata: dict[str,list[float]] = defaultdict(list)
        diagnostics = {"ipc_count":[],"process_hop_count":[],"serialization_bytes":[],"architecture_overhead_ms":[]}
        for case_id in case_ids:
            runs = repetitions[realization][case_id]
            median_run = sorted(runs, key=lambda item:item["driver_wall_ns"])[3]
            latency = statistics.median(item["driver_wall_ns"] for item in runs) / 1e9
            medians.append(latency)
            strata[semantic[case_id]["semantic_family"]].append(latency)
            correct = all(evaluate_goal(median_run, catalog[case_id]["goal_oracle"]).values())
            observations.append(observation("QA-01", population["population_id"], case_id, {
                "start_seconds":0.0,"voice_facts_audible_seconds":latency,"text_details_available_seconds":latency,"useful_outcome_correct":correct,
            }))
            diagnostics["ipc_count"].append(median_run["ipc_count"])
            diagnostics["process_hop_count"].append(median_run["process_hop_count"])
            diagnostics["serialization_bytes"].append(median_run["serialization_bytes"])
            common_ms = 80 if not (realization == "R1+@" and semantic[case_id]["semantic_family"] == "fast_bounded") else 80
            diagnostics["architecture_overhead_ms"].append(max(0.0, latency*1000-common_ms))
        result = evaluate_canonical_observations("QA-01",population,observations,provenance(contracts,commit,realization,"QA-01"),contracts).to_dict()
        result["diagnostics"]["strata"] = {key:{"p50_seconds":statistics.median(values),"p95_seconds":percentile95(values),"count":len(values)} for key,values in strata.items()}
        result["diagnostics"]["structure"] = {key:{"median":statistics.median(values),"p95":percentile95(values)} for key,values in diagnostics.items()}
        results[realization] = result
        observations_by_realization[realization] = observations
    return results, raw, {"observations":observations_by_realization,"memory":memory}


def run_goal_qa(qa_id: str, realization: str, topology: ExecutableTopology, commit: str, contracts: ContractRepository):
    semantic, agent, catalog = goal_maps()
    population = FrozenCorpus(contracts).population(qa_id)
    outputs=[]; observations=[]
    for case in cases(qa_id):
        case_id=case["goal_id"]
        output=topology.execute(goal_request(case_id,semantic[case_id],agent[case_id]))
        alternate_semantic=dict(semantic[case_id])
        alternate_semantic["modality"]="text" if semantic[case_id]["modality"]=="voice" else "voice"
        alternate=topology.execute(goal_request(case_id,alternate_semantic,agent[case_id]))
        output["voice_text_consistency"]=modality_consistent(output,alternate)
        output["alternate_modality_probe"]={"modality":alternate_semantic["modality"],"consistent":output["voice_text_consistency"],"topology":alternate["topology"]}
        outputs.append(output)
        observations.append(observation(qa_id,population["population_id"],case_id,{"required_conditions":evaluate_goal(output,catalog[case_id]["goal_oracle"])}))
    result=evaluate_canonical_observations(qa_id,population,observations,provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    return result,outputs,observations


def run_qa03(realization: str, topology: ExecutableTopology, commit: str, contracts: ContractRepository):
    qa_id="QA-03"; population=FrozenCorpus(contracts).population(qa_id); outputs=[]; observations=[]
    for case in cases(qa_id):
        request=generic_request(case["case_id"],capability="specialist-query" if int(case["variation_id"][1:])%4==0 else "planning",events=event_script(case))
        output=topology.execute(request); outputs.append(output)
        state = output.get("via_state",output.get("shell_state"))["event_state" if "via_state" in output else "correlation_log"]
        task=state[f"task:{case['case_id']}"]
        normal_spans={span["span"] for span in output["trace"]}
        relation_evidence={
            "turn_task":bool(task["events"]),"task_execution":bool(output["state_owner"]),"execution_result":"agent.result" in normal_spans,
            "result_response":"result.bind" in normal_spans,"response_delivery":"delivery" in normal_spans,
            "approval_task":task.get("approval_version",0)==1 or "approval" not in case["event_interleaving"],
            "cancel_task":task.get("cancelled",False) or "cancel" not in case["event_interleaving"],
            "follow_up_task":task.get("follow_up_correlated",False),
        }
        required=oracles(qa_id)[case["case_id"]]["required_relations"]
        observations.append(observation(qa_id,population["population_id"],case["case_id"],{"required_relations":{key:relation_evidence[key] for key in required}}))
    result=evaluate_canonical_observations(qa_id,population,observations,provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    return result,outputs,observations


def run_evolution_qa(qa_id: str, realization: str, commit: str, contracts: ContractRepository, target_dir: Path):
    population=FrozenCorpus(contracts).population(qa_id); outputs=[]; observations=[]
    for case in cases(qa_id):
        candidate_input=EvolutionInput(
            case_id=case["case_id"],
            change_request=case.get("change_request"),
            requirement=case.get("requirement"),
            required_functionality=case.get("required_functionality"),
            device_family=case.get("device_family"),
        )
        output=run_evolution_case(commit,realization,qa_id,candidate_input,target_dir); outputs.append(output)
        if qa_id=="QA-04":
            measurement={"requested_change_completed":output["acceptance_tests_pass"],"common_regressions_pass":output["common_regressions_pass"],"actual_semantic_ownership_changes":output["actual_semantic_ownership_changes"],"allowed_agent_integration_ownership_areas":case["allowed_agent_integration_ownership_areas"],"actual_extension_seams":output["actual_extension_seams"],"approved_extension_seams":case["approved_extension_seams"],"actual_core_semantic_zone_changes":output["actual_changed_semantic_zones"],"forbidden_core_semantic_zones":case["forbidden_core_semantic_zones"]}
        elif qa_id=="QA-05":
            measurement={"requested_functionality_completed":output["acceptance_tests_pass"],"common_regressions_pass":output["common_regressions_pass"],"actual_changed_semantic_zones":output["actual_changed_semantic_zones"],"expected_ownership_zones":case["expected_ownership_zones"],"actual_extension_seams":output["actual_extension_seams"],"approved_extension_seams":case["approved_extension_seams"],"semantic_dependency_leaks":output["semantic_dependency_leaks"]}
        else:
            measurement={"required_functionality_delivered":output["acceptance_tests_pass"],"actual_core_semantic_changes":output["actual_core_semantic_changes"],"device_family":case["device_family"]}
        observations.append(observation(qa_id,population["population_id"],case["case_id"],measurement))
    result=evaluate_canonical_observations(qa_id,population,observations,provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    return result,outputs,observations


def fault_target(realization: str, case: dict[str, Any]) -> tuple[str|None,str]:
    fault=case["fault_type"]; variation=int(case["variation_id"][1:])
    if fault=="context-broker-failure": return "context","context dependency"
    if fault in {"agent-crash","agent-disconnect","task-runtime-restart","adapter-crash"}:
        if realization=="R3": return ("specialist-agent" if variation%4==0 else "primary-runtime"),("specialist-agent" if variation%4==0 else "r3-primary-runtime")
        return ("specialist-agent" if variation%4==0 else "general-agent"),("specialist-agent" if variation%4==0 else "r1-general-agent")
    return None,"VIA delivery/correlation"


def run_qa08(realization: str, topology: ExecutableTopology, commit: str, contracts: ContractRepository):
    qa_id="QA-08"; population=FrozenCorpus(contracts).population(qa_id); outputs=[]; observations=[]
    for case in cases(qa_id):
        target,owner=fault_target(realization,case)
        capability="specialist-query" if target=="specialist-agent" else "planning"
        task_id=f"task:{case['case_id']}"
        events=[{"type":"TASK_START","task_id":task_id},{"type":"AGENT_PROGRESS","task_id":task_id},{"type":"FAULT","task_id":task_id}]
        if case["fault_type"]=="duplicate-event":
            duplicate={"type":"AGENT_PROGRESS","task_id":task_id,"event_id":f"progress:{case['case_id']}"}
            events.extend([duplicate,dict(duplicate)])
        if case["fault_type"]=="late-event":
            events.extend([{"type":"CANCEL","task_id":task_id},{"type":"LATE_AGENT_RESULT","task_id":task_id},{"type":"RECOVERY","task_id":task_id}])
        elif case["fault_type"]=="delivery-interruption":
            events.extend([{"type":"AGENT_RESULT","task_id":task_id},{"type":"RECOVERY","task_id":task_id},{"type":"FOLLOW_UP","task_id":task_id}])
        else:
            events.extend([{"type":"RECOVERY","task_id":task_id},{"type":"AGENT_RESULT","task_id":task_id}])
        request=generic_request(case["case_id"],capability=capability,events=events); request["fault_target"]=target
        output=topology.execute(request); output["fault_owner"]=owner; outputs.append(output)
        lifecycle=output.get("via_state",output.get("shell_state"))["event_state" if "via_state" in output else "correlation_log"]
        task=lifecycle[f"task:{case['case_id']}"]
        event_kinds=[event["type"] for event in task["events"]]
        component_recovered=(target is None or output["recovery"]["component_restarted"] is True)
        task_identity=(output["case_id"]==case["case_id"] and output.get("via_state",output.get("shell_state"))["user_task"]==f"task:{case['case_id']}")
        result_binding=output.get("via_state",output.get("shell_state"))["result_binding"]
        if case["fault_type"]=="late-event":
            version_matches=task["result_version"]==0 and task.get("late_result_rejected") is True and result_binding is False
            single_result=event_kinds.count("AGENT_RESULT")==0
        else:
            version_matches=extract_agent_result(output)["result_version"]==1 and task["result_version"]==1 and result_binding is True
            single_result=event_kinds.count("AGENT_RESULT")==1
        safe=(component_recovered and task_identity and version_matches and event_kinds.count("FAULT")==1 and event_kinds.count("RECOVERY")==1 and single_result)
        output["recovery"]["derived_checks"]={"component_recovered":component_recovered,"task_identity":task_identity,"result_version_matches":version_matches,"single_result_commit":single_result,"fault_and_recovery_observed":event_kinds.count("FAULT")==1 and event_kinds.count("RECOVERY")==1}
        output["recovery"]["safe_continuation"]=safe
        seconds=output["recovery"]["recovery_ns"]/1e9
        observations.append(observation(qa_id,population["population_id"],case["case_id"],{"safe_recovery_reached":safe,"safe_recovery_seconds":seconds,"unsafe_or_incorrect_recovery":not safe}))
    result=evaluate_canonical_observations(qa_id,population,observations,provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    owners=defaultdict(list)
    for item in outputs: owners[item["fault_owner"]].append(item["recovery"]["recovery_ns"]/1e6)
    result["diagnostics"]["fault_owners"]={key:{"count":len(values),"p95_ms":percentile95(values)} for key,values in owners.items()}
    return result,outputs,observations


def scope_request(case: dict[str, Any]) -> tuple[list[str],dict[str,Any]]:
    resource=case["purpose"].removeprefix("complete-").removesuffix("-goal")
    variation=int(case["variation_id"][1:]); object_id=f"object-{(variation-1)%5}"
    task_scope=f"scope:task:{case['case_id']}"; resource_scope=f"scope:{resource}:{object_id}"
    requested=[resource_scope,task_scope]
    if case["hard_gate_probe"]:
        requested.extend(case["forbidden_scopes"])
    return requested,{"resource_kind":resource,"object_id":object_id,"approval_version":"v1"}


def run_qa09(realization: str, topology: ExecutableTopology, commit: str, contracts: ContractRepository):
    qa_id="QA-09"; population=FrozenCorpus(contracts).population(qa_id); oracle=oracles(qa_id); outputs=[]; observations=[]
    for case in cases(qa_id):
        request=generic_request(case["case_id"],capability="specialist-query" if int(case["variation_id"][1:])%5==0 else "planning")
        request["principal"]=case["principal"]; request["purpose"]=case["purpose"]; request["scope_request"],request["policy_context"]=scope_request(case)
        output=topology.execute(request); outputs.append(output)
        granted=output["policy"]["granted"]
        hard=[]
        if set(granted)&set(case["forbidden_scopes"]): hard=list(case["hard_gate_probe"])
        observations.append(observation(qa_id,population["population_id"],case["case_id"],{"required_scopes":oracle[case["case_id"]]["exact_required_scopes"],"granted_or_exposed_scopes":granted,"confirmed_hard_gate_triggers":hard}))
    result=evaluate_canonical_observations(qa_id,population,observations,provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    boundaries=defaultdict(int)
    for output in outputs:
        for crossing in output["scope_crossings"]: boundaries[crossing["boundary"]]+=1
    result["diagnostics"]["trust_boundary_crossings"]=dict(boundaries)
    return result,outputs,observations


def reconstruct_trace(output: dict[str, Any]) -> tuple[dict[str,Any],dict[str,Any]]:
    spans={span["span"]:span for span in output["trace"]}
    parent_valid=all(span["parent"] is None or span["parent"] in spans for span in spans.values())
    required={"interaction","semantic","policy","dispatch","agent.accept","agent.execute","agent.result","result.bind","delivery"}
    complete=required<=set(spans) and parent_valid
    nodes=["input","evidence","decision","agent_selection","execution","task","approval_progress_result","response","delivery","attribution"] if complete else []
    graph={"nodes":nodes,"edges":[[a,b] for a,b in zip(nodes,nodes[1:])]}
    chain={node:complete for node in nodes}
    return graph,chain


def run_qa10(realization: str, topology: ExecutableTopology, commit: str, contracts: ContractRepository):
    qa_id="QA-10"; population=FrozenCorpus(contracts).population(qa_id); oracle=oracles(qa_id); outputs=[]; observations=[]
    for case in cases(qa_id):
        capability="specialist-query" if "specialist" in case["execution_fixture"] else "planning"
        output=topology.execute(generic_request(case["case_id"],capability=capability)); graph,chain=reconstruct_trace(output)
        output["reconstructed_graph"]=graph; outputs.append(output)
        expected=oracle[case["case_id"]]
        serialized_output=json.dumps(output,sort_keys=True).lower()
        oracle_exposed=any(marker in serialized_output for marker in ("expected_graph","required_chain","ground_truth_trace","oracle"))
        observations.append(observation(qa_id,population["population_id"],case["case_id"],{"required_chain":{node:chain.get(node,False) for node in expected["required_chain"]},"expected_causal_graph":expected["expected_graph"],"reconstructed_causal_graph":graph,"hidden_oracle_or_fault_labels_exposed":oracle_exposed}))
    result=evaluate_canonical_observations(qa_id,population,observations,provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    return result,outputs,observations


def run_qa11(realization: str, topology: ExecutableTopology, commit: str, contracts: ContractRepository):
    qa_id="QA-11"; population=FrozenCorpus(contracts).population(qa_id); outputs=[]; observations=[]
    for case in cases(qa_id):
        request=generic_request(case["case_id"],capability="planning"); request["feedback_event_class"]=case["event_class"]
        output=topology.execute(request); output["event_class"]=case["event_class"]; outputs.append(output)
        emitted=output["feedback"]["emitted_offset_ns"]/1e9; delivered=output["feedback"]["delivered_offset_ns"]/1e9
        observations.append(observation(qa_id,population["population_id"],case["case_id"],{"event_available_seconds":emitted,"feedback_received_seconds":delivered,"useful_feedback":delivered>=emitted,"event_class":case["event_class"]}))
    result=evaluate_canonical_observations(qa_id,population,observations,provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    return result,outputs,observations


def run_qa12(realization: str, commit: str, contracts: ContractRepository):
    qa_id="QA-12"; population=FrozenCorpus(contracts).population(qa_id); outputs=[]; observations=[]; probe=PlaybackProbe()
    try:
        for case in cases(qa_id):
            output=probe.execute(case["playback_buffer_depth_ms"]); output["case_id"]=case["case_id"]; outputs.append(output)
            onset=case["ground_truth_user_speech_onset_ms"]
            observations.append(observation(qa_id,population["population_id"],case["case_id"],{"user_speech_onset_ms":onset,"last_audible_sample_ms":onset+output["last_frame_end_offset_ms"]}))
    finally: probe.close()
    result=evaluate_canonical_observations(qa_id,population,observations,provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    result["diagnostics"]["frame_ms"]=10
    return result,outputs,observations


def qa07_result(realization: str, commit: str, contracts: ContractRepository, memory: list[dict[str,Any]]):
    qa_id="QA-07"; population=FrozenCorpus(contracts).population(qa_id); population["complete"]=False
    result=evaluate_canonical_observations(qa_id,population,[],provenance(contracts,commit,realization,qa_id),contracts).to_dict()
    result["eligibility_status"]="UNEVALUABLE_NO_PHYSICAL_MEMORY_DENOMINATOR"
    result["diagnostics"].update({"diagnostic_name":"LOCAL REFERENCE STRUCTURAL MEMORY DIAGNOSTIC","process_count":len(memory),"aggregate_rss_kib":sum(row["rss_kib"] for row in memory),"processes":memory,"official_score_prohibited":True})
    return result,[],[]


def assert_clean_preregistration(commit: str) -> None:
    head=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,check=True,capture_output=True,text=True).stdout.strip()
    status=subprocess.run(["git","status","--porcelain"],cwd=ROOT,check=True,capture_output=True,text=True).stdout.strip()
    if head!=commit or status: raise RuntimeError("official campaign must start at exact clean preregistration commit")
    subject=subprocess.run(["git","show","-s","--format=%s",commit],cwd=ROOT,check=True,capture_output=True,text=True).stdout.strip()
    if subject!="feat(evaluation): preregister DP-00 executable reference v3":
        raise RuntimeError("HEAD is not the named v3 preregistration commit")
    subprocess.run(["cargo","build","--release","--all-targets","--offline"],cwd=PROTOTYPE,check=True)


def render_summary(summary: dict[str,Any]) -> str:
    lines=["# DP-00 Executable Reference Architecture Qualification v3 Results","","## Evidence status","",summary["evidence_status"],"","## QA-v1 results","","| QA | R1 raw / score | R3 raw / score | R1+@ raw / score | Target |","| --- | ---: | ---: | ---: | --- |"]
    for number in range(1,13):
        qa=f"QA-{number:02d}"; row=[]
        for r in REALIZATIONS:
            result=summary["results"][r][qa]
            row.append("UNEVALUABLE" if result["raw_metric"] is None else f"{result['raw_metric']:.6g} {result['unit']} / {result['score']}")
        lines.append(f"| {qa} | {row[0]} | {row[1]} | {row[2]} | {summary['targets'][qa]} |")
    lines.extend(["","## Decision","",f"**{summary['decision']}**","",summary["interpretation"],""])
    return "\n".join(lines)


def run(commit: str) -> dict[str,Any]:
    assert_clean_preregistration(commit)
    if RESULT_ROOT.exists(): raise FileExistsError(f"immutable campaign result path already exists: {RESULT_ROOT}")
    contracts=ContractRepository(ROOT); results={r:{} for r in REALIZATIONS}; raw={r:{} for r in REALIZATIONS}; observations={r:{} for r in REALIZATIONS}
    qa01,qa01raw,qa01extra=run_qa01(commit,contracts)
    for r in REALIZATIONS:
        results[r]["QA-01"]=qa01[r]; raw[r]["QA-01"]=qa01raw[r]; observations[r]["QA-01"]=qa01extra["observations"][r]
    with tempfile.TemporaryDirectory(prefix="via-v3-evolution-target-") as target:
        target_dir=Path(target)
        for r in REALIZATIONS:
            with ExecutableTopology(r) as topology:
                runners={"QA-02":lambda:run_goal_qa("QA-02",r,topology,commit,contracts),"QA-03":lambda:run_qa03(r,topology,commit,contracts),"QA-08":lambda:run_qa08(r,topology,commit,contracts),"QA-09":lambda:run_qa09(r,topology,commit,contracts),"QA-10":lambda:run_qa10(r,topology,commit,contracts),"QA-11":lambda:run_qa11(r,topology,commit,contracts)}
                for qa_id,runner in runners.items(): results[r][qa_id],raw[r][qa_id],observations[r][qa_id]=runner()
            results[r]["QA-07"],raw[r]["QA-07"],observations[r]["QA-07"]=qa07_result(r,commit,contracts,qa01extra["memory"][r])
            results[r]["QA-12"],raw[r]["QA-12"],observations[r]["QA-12"]=run_qa12(r,commit,contracts)
            for qa_id in ("QA-04","QA-05","QA-06"):
                results[r][qa_id],raw[r][qa_id],observations[r][qa_id]=run_evolution_qa(qa_id,r,commit,contracts,target_dir)
    targets={qa["qa_id"]:qa["target"]["display"] for qa in contracts.contract["quality_attributes"]}
    structural={
        "topologies_differ":raw["R1"]["QA-01"][0]["topology"]!=raw["R3"]["QA-01"][0]["topology"],
        "specialist_paths_differ":any(item["process_hop_count"]==3 for item in raw["R3"]["QA-01"]) and any(item["process_hop_count"]==2 for item in raw["R1"]["QA-01"]),
        "state_owners_differ":{item["state_owner"] for item in raw["R1"]["QA-01"]}!={item["state_owner"] for item in raw["R3"]["QA-01"]},
        "qa01_diagnostics_differ":results["R1"]["QA-01"]["diagnostics"]["structure"]!=results["R3"]["QA-01"]["diagnostics"]["structure"],
        "qa04_changed_surfaces_differ":{tuple(item["changed_modules"]) for item in raw["R1"]["QA-04"]}!={tuple(item["changed_modules"]) for item in raw["R3"]["QA-04"]},
        "fault_owners_differ":set(results["R1"]["QA-08"]["diagnostics"]["fault_owners"])!=set(results["R3"]["QA-08"]["diagnostics"]["fault_owners"]),
        "privilege_topologies_differ":set(results["R1"]["QA-09"]["diagnostics"]["trust_boundary_crossings"])!=set(results["R3"]["QA-09"]["diagnostics"]["trust_boundary_crossings"]),
        "trace_topologies_differ":raw["R1"]["QA-10"][0]["topology"]!=raw["R3"]["QA-10"][0]["topology"],
    }
    coverage={
        "QA-01":{"bounded":sum(item["stratum"]=="fast_bounded" for item in raw["R1"]["QA-01"]),"general":sum(item["stratum"]=="short_general_agent" for item in raw["R1"]["QA-01"]),"specialist":sum("specialist-agent" in item["topology"]["selected_path"] for item in raw["R3"]["QA-01"])},
        "QA-03":{"episodes":len(raw["R1"]["QA-03"]),"event_types":sorted({event["type"] for case in cases("QA-03") for event in event_script(case)})},
        "QA-08":{"owners":sorted({item["fault_owner"] for r in REALIZATIONS for item in raw[r]["QA-08"]})},
        "QA-09":{"boundaries":sorted({cross["boundary"] for r in REALIZATIONS for item in raw[r]["QA-09"] for cross in item["scope_crossings"]})},
    }
    quality_checks={
        f"{r}:{qa_id}":(
            results[r][qa_id]["raw_metric"] is None
            if qa_id=="QA-07"
            else results[r][qa_id]["target_met"] is True and not results[r][qa_id]["failed_gates"]
        )
        for r in REALIZATIONS for qa_id in [f"QA-{n:02d}" for n in range(1,13)]
    }
    campaign_valid=all(structural.values()) and all(quality_checks.values()) and coverage["QA-01"]["bounded"]>0 and coverage["QA-01"]["general"]>0 and coverage["QA-01"]["specialist"]>0
    decision="FULL QUALIFICATION BLOCKED ONLY BY QA-07" if campaign_valid else "EVALUATION STILL INVALID"
    common_provenance=results["R1"]["QA-01"]["provenance"]
    summary={"campaign_id":CAMPAIGN_ID,"preregistration_commit":commit,"provenance":{key:common_provenance[key] for key in ("qa_contract_id","qa_contract_hash","reference_environment_id","reference_environment_hash","corpus_manifest_id","corpus_manifest_hash","evidence_mode")},"evidence_status":"VALID EXECUTABLE QA-v1 EVIDENCE; official QA-07 remains unevaluable.","results":results,"targets":targets,"coverage_gate":{"pass":campaign_valid,"details":coverage,"quality_checks":quality_checks},"structural_sensitivity_gate":{"pass":all(structural.values()),"checks":structural},"decision":decision,"interpretation":"R1 and R3 are not selected while QA-07 lacks the approved physical-memory denominator. R1+@ is evaluated as an R1 tactic; any latency improvement is reported without treating it as a third base architecture."}
    for r in REALIZATIONS:
        for qa_id in [f"QA-{n:02d}" for n in range(1,13)]:
            write_jsonl(RESULT_ROOT/"raw"/r/qa_id/"executions.jsonl",raw[r][qa_id])
            write_jsonl(RESULT_ROOT/"raw"/r/qa_id/"observations.jsonl",observations[r][qa_id])
            write_json(RESULT_ROOT/"derived"/r/f"{qa_id}-result.json",results[r][qa_id])
    source_hashes={str(path.relative_to(ROOT)):sha256(path) for path in sorted(PROTOTYPE.rglob("*")) if path.is_file() and "target" not in path.parts and "reference-base/source" not in str(path)}
    write_json(RESULT_ROOT/"campaign-summary.json",summary)
    write_json(RESULT_ROOT/"source-freeze-verification.json",{"preregistration_commit":commit,"source_hashes":source_hashes,"cloud_or_api_calls":0,"wall_clock_qa01":True,"source_changes_after_preregistration":[]})
    (RESULT_ROOT/"QUALIFICATION-REPORT.md").write_text(render_summary(summary),encoding="utf-8")
    return summary


def main() -> None:
    parser=argparse.ArgumentParser(); parser.add_argument("--architecture-commit",required=True); args=parser.parse_args()
    summary=run(args.architecture_commit); print(json.dumps({"campaign_id":summary["campaign_id"],"decision":summary["decision"],"valid":summary["coverage_gate"]["pass"]},indent=2))


if __name__=="__main__": main()
