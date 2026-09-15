from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from typing import Any

from .campaign import FIXTURES, PROTOTYPE, evaluate_goal, goal_maps, goal_request, reconstruct_trace, scope_request
from .evolution import EvolutionInput, acceptance_succeeded, classify_change_request, derive_dependency_evidence, parse_manifest, run_evolution_case
from .materialize import materialize
from .runtime import ExecutableTopology, ROOT, normalized_behavior


def check_reference_hashes() -> bool:
    base=PROTOTYPE/"reference-base"; manifest=base/"SHA256SUMS"
    check=subprocess.run(["shasum","-a","256","-c",str(manifest)],cwd=base,capture_output=True,text=True)
    return check.returncode==0 and len(check.stdout.splitlines())==650


def static_gates() -> dict[str,bool]:
    runtime_sources="\n".join(path.read_text(encoding="utf-8") for path in (PROTOTYPE/"src").rglob("*.rs"))
    semantic=json.loads((FIXTURES/"semantic-replay-v3.json").read_text(encoding="utf-8"))
    agent=json.loads((FIXTURES/"agent-replay-v3.json").read_text(encoding="utf-8"))
    visible_text=json.dumps({"semantic":semantic,"agent":agent})
    forbidden=("expected_graph","required_result_facts","correct_goal","qa_pass","family_id")
    evolution_source=(ROOT/"benchmark/dp_executable_v3/evolution.py").read_text(encoding="utf-8")
    campaign_source=(ROOT/"benchmark/dp_executable_v3/campaign.py").read_text(encoding="utf-8")
    return {
        "reference_base_immutable":check_reference_hashes(),
        "runtime_has_no_candidate_id_branch":"candidate_id" not in runtime_sources and "CandidateId" not in runtime_sources,
        "runtime_does_not_import_evaluator":"benchmark::analysis" not in runtime_sources and "evaluator" not in runtime_sources.lower(),
        "candidate_fixture_has_no_oracle_fields":all(field not in visible_text for field in forbidden),
        "no_candidate_latency_constants":"r1_delay" not in runtime_sources.lower() and "r3_delay" not in runtime_sources.lower(),
        "no_random_error_probability":"rand" not in runtime_sources.lower() and "probability" not in runtime_sources.lower(),
        "separate_executables":all((PROTOTYPE/"src/bin"/name).exists() for name in ("r1-via.rs","r3-shell.rs","r3-primary-runtime.rs","r1-via-fastpath.rs")),
        "no_family_to_zone_mapping":"family_id" not in (ROOT/"benchmark/dp_executable_v3/evolution.py").read_text(encoding="utf-8"),
        "no_hardcoded_official_success":all(pattern not in campaign_source for pattern in (
            '"requested_change_completed":True', '"requested_functionality_completed":True',
            '"required_functionality_delivered":True', '"unsafe_or_incorrect_recovery":False',
            '"hidden_oracle_or_fault_labels_exposed":False', '"actual_core_semantic_changes":[]',
        )),
        "evolution_runner_has_no_evaluator_oracle_fields":all(field not in evolution_source for field in ("approved_extension_seams", "expected_ownership_zones", "forbidden_core_semantic_zones", "allowed_agent_integration_ownership_areas")),
        "evolution_uses_behavioral_acceptance":"acceptance_tests_pass\": acceptance_pass" in evolution_source and "_append_compile_marker" not in evolution_source,
        "qa01_runs_paired_modality_probe":"modality_checks[realization][case_id]=modality_consistent(primary,alternate)" in campaign_source,
        "campaign_valid_requires_all_qa_targets":"all(quality_checks.values())" in campaign_source,
    }


def topology_checks() -> dict[str,bool]:
    semantic,agent,_=goal_maps()
    request=goal_request("GOAL-001-01",semantic["GOAL-001-01"],agent["GOAL-001-01"])
    with ExecutableTopology("R1") as r1, ExecutableTopology("R3") as r3, ExecutableTopology("R1+@") as fast:
        a=r1.execute(request); b=r3.execute(request); c=fast.execute(request)
        oracle_absent=not any(r1.cwd.rglob("*oracle*")) and not any(r3.cwd.rglob("*oracle*"))
        r1_roles={item["role"] for item in a["topology"]["processes"]}
        r3_roles={item["role"] for item in b["topology"]["processes"]}
        state_files_r1=list((r1.cwd/"state").glob("*-workflow.ndjson"))
        state_files_r3=list((r3.cwd/"state").glob("*-workflow.ndjson"))
        return {
            "oracle_physically_absent":oracle_absent,
            "r1_process_contract":{"r1-via-control-plane","r1-general-agent","specialist-agent"}<=r1_roles,
            "r3_process_contract":{"r3-via-shell","r3-primary-runtime","specialist-agent"}<=r3_roles,
            "different_process_graphs":r1_roles!=r3_roles,
            "r1_selected_agent_owns_workflow":a["state_owner"]=="r1-general-agent" and any("r1-general-agent" in path.name for path in state_files_r1),
            "r3_primary_owns_workflow":b["state_owner"]=="r3-primary-runtime" and any("r3-primary-runtime" in path.name for path in state_files_r3),
            "r1_fastpath_is_local_read":c["state_owner"]=="r1-via-bounded-read-tactic" and c["via_state"]["bounded_read_only"],
            "real_serialization_and_ipc":all(item["serialization_bytes"]>0 and item["ipc_count"]>=2 for item in (a,b,c)),
        }


def label_swap_check() -> bool:
    semantic,agent,_=goal_maps()
    request=goal_request("GOAL-001-01",semantic["GOAL-001-01"],agent["GOAL-001-01"])
    with ExecutableTopology("R1",executable_override="r1-via") as a, ExecutableTopology("R3",executable_override="r1-via") as b:
        return normalized_behavior(a.execute(request))==normalized_behavior(b.execute(request))


def _mutated_tree(replacements: dict[str,tuple[str,str]]) -> tuple[tempfile.TemporaryDirectory[str],Path]:
    temp=tempfile.TemporaryDirectory(prefix="via-v3-mutation-"); tree=Path(temp.name)/"prototype"
    shutil.copytree(PROTOTYPE,tree,ignore=shutil.ignore_patterns("target","reference-base"))
    for relative,(before,after) in replacements.items():
        path=tree/relative; text=path.read_text(encoding="utf-8")
        if before not in text: raise AssertionError(f"mutation anchor absent: {relative}")
        path.write_text(text.replace(before,after,1),encoding="utf-8")
    target=Path(tempfile.gettempdir())/"via-v3-mutation-target"
    subprocess.run(["cargo","build","--release","--all-targets","--offline"],cwd=tree,env={**os.environ,"CARGO_TARGET_DIR":str(target)},check=True,capture_output=True,text=True)
    return temp,target/"release"


def mutation_checks() -> dict[str,bool]:
    semantic,agent,catalog=goal_maps(); request=goal_request("GOAL-001-01",semantic["GOAL-001-01"],agent["GOAL-001-01"])
    results:dict[str,bool]={}
    temp,bin_dir=_mutated_tree({"src/bin/r1-via.rs":('(\"general-agent\", \"r1-general-agent\")','(\"ipc-proxy\", \"r1-general-agent\")')})
    try:
        with ExecutableTopology("R1",bin_dir=bin_dir) as topology:
            output=topology.execute(request)
            results["real_proxy_process_detected"]=output["agent"].get("proxy",{}).get("pid") is not None and output["agent"]["pid"]!=output["topology"]["processes"][3]["pid"]
    finally: temp.cleanup()
    temp,bin_dir=_mutated_tree({"src/lib.rs":('\"span\":\"agent.result\"','\"span\":\"agent.result.missing\"')})
    try:
        with ExecutableTopology("R1",bin_dir=bin_dir) as topology: graph,_=reconstruct_trace(topology.execute(request)); results["missing_trace_span_detected"]=not graph["nodes"]
    finally: temp.cleanup()
    temp,bin_dir=_mutated_tree({"src/lib.rs":('|| (authorized_resource.is_none() && !text.contains(\"principal:\"))','|| true || (authorized_resource.is_none() && !text.contains(\"principal:\"))')})
    try:
        probe={"case_id":"QA09-01-18","variation_id":"V18","purpose":"complete-document-scope-goal","hard_gate_probe":["explicitly forbidden disclosure"],"forbidden_scopes":["scope:document-scope:object-1","scope:unrelated-principal:*"]}
        req=request.copy(); req["case_id"]="QA09-01-18"; req["task_id"]="task:QA09-01-18"; req["scope_request"],req["policy_context"]=scope_request(probe)
        with ExecutableTopology("R1",bin_dir=bin_dir) as topology: granted=topology.execute(req)["policy"]["granted"]
        results["unauthorized_scope_detected"]=bool(set(granted)&set(probe["forbidden_scopes"]))
    finally: temp.cleanup()
    temp,bin_dir=_mutated_tree({"src/bin/r1-via.rs":('"case_id":request.get("case_id").cloned().unwrap_or(Value::Null)','"case_id":json!("wrong-case")')})
    try:
        with ExecutableTopology("R1",bin_dir=bin_dir) as topology: output=topology.execute(request)
        results["result_correlation_defect_detected"]=output["case_id"]!="GOAL-001-01"
    finally: temp.cleanup()
    temp,bin_dir=_mutated_tree({"src/ownership/r1_agent_adapter.rs":(
        "pub const VERSION: usize = 1;",
        "pub const VERSION: usize = 1;\npub const COMPILE_ONLY_MARKER: &str = \"not an implementation\";",
    )})
    try:
        tree=Path(temp.name)/"prototype"
        compile_only=subprocess.run(
            ["cargo","test","--lib","--offline","--quiet","acceptance_missing_case"],
            cwd=tree,env={**os.environ,"CARGO_TARGET_DIR":str(Path(tempfile.gettempdir())/"via-v3-mutation-target")},
            capture_output=True,text=True,
        )
        results["compile_only_evolution_rejected"]=compile_only.returncode==0 and not acceptance_succeeded(compile_only)
    finally: temp.cleanup()
    manifest=parse_manifest(PROTOTYPE/"ARCHITECTURE-OWNERSHIP-MANIFEST.yaml")
    temp,bin_dir=_mutated_tree({"src/ownership/r1_agent_adapter.rs":(
        "pub const VERSION: usize = 1;",
        "use crate::ownership::z3_semantics;\npub const VERSION: usize = z3_semantics::VERSION;",
    )})
    try:
        tree=Path(temp.name)/"prototype"
        edges,leaks=derive_dependency_evidence(tree,["src/ownership/r1_agent_adapter.rs"],manifest)
        results["real_dependency_zone_leak_detected"]=bool(edges) and leaks==["Z3"]
        changed=[manifest["src/ownership/r1_agent_adapter.rs"],manifest["src/ownership/z3_semantics.rs"]]
        zones={zone for item in changed for zone in item["semantic_concern_zones"]}
        results["agent_evolution_core_edit_detected"]=bool(zones&{"Z1","Z2","Z3","Z5","Z6","Z7","Z8"})
    finally: temp.cleanup()
    temp,bin_dir=_mutated_tree({"src/bin/playback-probe.rs":('map(|_| 10).unwrap_or(0)','map(|_| 250).unwrap_or(0)')})
    try:
        process=subprocess.Popen([str(bin_dir/"playback-probe")],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
        assert process.stdin and process.stdout; process.stdin.write('{"playback_buffer_depth_ms":40}\n'); process.stdin.flush(); output=json.loads(process.stdout.readline()); process.stdin.write('{"control":"shutdown"}\n'); process.stdin.flush(); process.wait()
        results["audio_buffer_regression_detected"]=output["last_frame_end_offset_ms"]>200
    finally: temp.cleanup()
    return results


def evolution_behavioral_checks() -> dict[str,bool]:
    commit=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,check=True,capture_output=True,text=True).stdout.strip()
    target=Path(tempfile.gettempdir())/"via-v3-preflight-evolution-target"
    checks={}
    samples=(
        ("QA-04",EvolutionInput("PREFLIGHT-QA04",change_request="agent-onboarding variation 1 with contract profile cp-01")),
        ("QA-05",EvolutionInput("PREFLIGHT-QA05",change_request="ASR revision event")),
        ("QA-06",EvolutionInput("PREFLIGHT-QA06",requirement="camera context",required_functionality="Deliver camera context through the platform-independent VIA task/result contract",device_family="MOBILE")),
    )
    for qa_id,item in samples:
        output=run_evolution_case(commit,"R1",qa_id,item,target)
        checks[qa_id]=bool(
            output["build_pass"] and output["acceptance_tests_pass"] and output["common_regressions_pass"]
            and output["actual_extension_seams"] and output["git_diff_patch"]
        )
    return checks


def run() -> dict[str,Any]:
    materialize()
    subprocess.run(["cargo","test","--all-targets","--offline"],cwd=PROTOTYPE,check=True)
    subprocess.run(["cargo","build","--release","--all-targets","--offline"],cwd=PROTOTYPE,check=True)
    static=static_gates(); topology=topology_checks(); mutations=mutation_checks(); label=label_swap_check(); evolution=evolution_behavioral_checks()
    qa05=[case["change_request"] for case in json.loads((ROOT/"benchmark/evolution/qa-v1/qa05-product-evolution-v1/instances.json").read_text())["instances"]]
    routing=all(classify_change_request(change) for change in qa05)
    result={"status":"PASS" if all(static.values()) and all(topology.values()) and all(mutations.values()) and all(evolution.values()) and label and routing else "FAIL","static_gates":static,"runtime_topology_gates":topology,"label_swap_invariance":label,"real_mutation_sensitivity":mutations,"evolution_behavioral_acceptance":evolution,"natural_change_request_routing":routing,"comparative_results_included":False}
    (PROTOTYPE/"PREFLIGHT-VALIDITY.json").write_text(json.dumps(result,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    if result["status"]!="PASS": raise SystemExit(json.dumps(result,indent=2))
    print(json.dumps(result,indent=2,sort_keys=True)); return result


if __name__=="__main__": run()
