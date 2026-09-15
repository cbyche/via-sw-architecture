from __future__ import annotations

import json
from pathlib import Path
import sys

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))

from benchmark.dp_executable_v3.campaign import event_script, goal_maps, goal_request, modality_consistent, reconstruct_trace, scope_request
from benchmark.dp_executable_v3.evolution import EvolutionInput, ZONE_TERMS, classify_change_request, derive_dependency_evidence, derive_extension_seam, parse_manifest
from benchmark.dp_executable_v3.preflight import check_reference_hashes, static_gates
from benchmark.dp_executable_v3.runtime import ExecutableTopology, PROTOTYPE, normalized_behavior


def test_reference_snapshot_manifest_verifies() -> None:
    assert check_reference_hashes()


def test_ownership_manifest_maps_all_evolution_zones() -> None:
    manifest=parse_manifest(PROTOTYPE/"ARCHITECTURE-OWNERSHIP-MANIFEST.yaml")
    zones={zone for item in manifest.values() for zone in item["semantic_concern_zones"]}
    assert zones==set(ZONE_TERMS)


@pytest.mark.parametrize(("change_text","zone"),[("ASR revision event","Z1"),("referent ranking policy","Z2"),("semantic decision schema","Z3"),("Agent protocol change","Z4"),("Task persistence evolution","Z5"),("voice summary policy","Z6"),("buffer allocator","Z7"),("robot sensor adapter","Z8")])
def test_change_request_routes_by_semantics(change_text: str, zone: str) -> None:
    assert classify_change_request(change_text)==zone


def test_evolution_seams_are_derived_without_evaluator_fields() -> None:
    assert derive_extension_seam("QA-04",EvolutionInput(case_id="c",change_request="agent-onboarding variation 1 with contract profile cp-01"))=="agent-integration:agent-onboarding"
    assert derive_extension_seam("QA-05",EvolutionInput(case_id="c",change_request="ASR revision event"))=="z1:asr-revision-event"
    assert derive_extension_seam("QA-06",EvolutionInput(case_id="c",requirement="camera context",device_family="MOBILE"))=="mobile-context-provider"


def test_dependency_leak_is_derived_from_rust_source(tmp_path: Path) -> None:
    prototype=tmp_path/"prototype"; module=prototype/"src/ownership/r1_agent_adapter.rs"
    module.parent.mkdir(parents=True)
    module.write_text("use crate::ownership::z3_semantics;\npub const VERSION: usize = z3_semantics::VERSION;\n")
    manifest=parse_manifest(PROTOTYPE/"ARCHITECTURE-OWNERSHIP-MANIFEST.yaml")
    edges,leaks=derive_dependency_evidence(prototype,["src/ownership/r1_agent_adapter.rs"],manifest)
    assert edges==[{"source":"src/ownership/r1_agent_adapter.rs","target":"src/ownership/z3_semantics.rs"}]
    assert leaks==["Z3"]


def test_event_script_is_chronological_and_executable() -> None:
    case={"case_id":"C1","event_interleaving":"approval-after-progress"}
    kinds=[item["type"] for item in event_script(case)]
    assert kinds.index("TASK_START")<kinds.index("AGENT_APPROVAL_REQUIRED")<kinds.index("AGENT_RESULT")


def test_concurrency_episode_has_two_live_tasks() -> None:
    case={"case_id":"C1","event_interleaving":"two-task-burst"}
    starts=[item["task_id"] for item in event_script(case) if item["type"]=="TASK_START"]
    assert len(set(starts))==2


def test_scope_request_is_derived_from_runtime_input() -> None:
    case={"case_id":"QA09-02-03","variation_id":"V03","purpose":"complete-file-scope-goal","hard_gate_probe":[],"forbidden_scopes":[]}
    requested,context=scope_request(case)
    assert requested==["scope:file-scope:object-2","scope:task:QA09-02-03"]
    assert context["resource_kind"]=="file-scope"


def test_static_anti_cheating_gates_pass() -> None:
    assert all(static_gates().values())


def test_r1_and_r3_are_different_process_graphs() -> None:
    semantic,agent,_=goal_maps(); request=goal_request("GOAL-001-01",semantic["GOAL-001-01"],agent["GOAL-001-01"])
    with ExecutableTopology("R1") as r1, ExecutableTopology("R3") as r3:
        a=r1.execute(request); b=r3.execute(request)
    assert {p["role"] for p in a["topology"]["processes"]}!={p["role"] for p in b["topology"]["processes"]}
    assert a["state_owner"]=="r1-general-agent"
    assert b["state_owner"]=="r3-primary-runtime"


def test_external_label_swap_does_not_change_behavior() -> None:
    semantic,agent,_=goal_maps(); request=goal_request("GOAL-001-01",semantic["GOAL-001-01"],agent["GOAL-001-01"])
    with ExecutableTopology("R1",executable_override="r1-via") as a, ExecutableTopology("R3",executable_override="r1-via") as b:
        assert normalized_behavior(a.execute(request))==normalized_behavior(b.execute(request))


def test_voice_text_consistency_is_derived_from_two_executions() -> None:
    semantic,agent,_=goal_maps(); primary_semantic=dict(semantic["GOAL-001-01"]); alternate_semantic=dict(primary_semantic)
    alternate_semantic["modality"]="text" if primary_semantic["modality"]=="voice" else "voice"
    with ExecutableTopology("R1") as topology:
        primary=topology.execute(goal_request("GOAL-001-01",primary_semantic,agent["GOAL-001-01"]))
        alternate=topology.execute(goal_request("GOAL-001-01",alternate_semantic,agent["GOAL-001-01"]))
    assert modality_consistent(primary,alternate)


def test_r1_fastpath_rejects_general_work() -> None:
    semantic,agent,_=goal_maps(); item=dict(semantic["GOAL-001-01"]); item.update({"capability":"planning","read_only":False})
    with ExecutableTopology("R1+@") as topology:
        output=topology.execute(goal_request("GOAL-001-01",item,agent["GOAL-001-01"]))
    assert output["state_owner"]=="r1-general-agent"
    assert not output["via_state"]["bounded_read_only"]


def test_normal_trace_reconstructs_only_from_spans() -> None:
    semantic,agent,_=goal_maps(); request=goal_request("GOAL-001-01",semantic["GOAL-001-01"],agent["GOAL-001-01"])
    with ExecutableTopology("R1") as topology: output=topology.execute(request)
    graph,chain=reconstruct_trace(output)
    assert len(graph["nodes"])==10
    assert all(chain.values())


def test_candidate_directory_contains_no_oracle() -> None:
    with ExecutableTopology("R1") as topology:
        assert not list(topology.cwd.rglob("*oracle*"))


def test_semantic_and_agent_replay_classes_are_distinct() -> None:
    fixture=PROTOTYPE.parents[1]/"benchmark/fixtures/dp00-executable-v3"
    semantic=json.loads((fixture/"semantic-replay-v3.json").read_text())
    agent=json.loads((fixture/"agent-replay-v3.json").read_text())
    assert semantic["artifact_class"]=="SemanticReplay"
    assert agent["artifact_class"]=="AgentReplay"
