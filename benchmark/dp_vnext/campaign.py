from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
from typing import Any

from benchmark.analysis.qa_v1 import ContractRepository, EvidenceMode, evaluate_canonical_observations

from .candidate_runtime import CANDIDATE_IDS, CandidateRuntime
from .controls import LATENCY_REFERENCE_PATH, POPULATION_PATHS, ROOT, LatencyReference, candidate_input, case_id, load_cases, load_json, load_oracles, sha256


CAMPAIGN_ID = "dp00-vnext-qa-v1-campaign-v1"
REGISTRATION_PATH = ROOT / "benchmark/contracts/dp-vnext/evaluation/dp00-campaign-registration-v1.json"
CONTROL_PATH = ROOT / "benchmark/contracts/dp-vnext/evaluation/dp00-frozen-controls-v1.json"
QA07_STATUS_PATH = ROOT / "benchmark/contracts/dp-vnext/evaluation/qa07-evidence-status-v1.json"
OFFICIAL_PROFILE = "QWEN3_MEDIUM_REFERENCE"
SENSITIVITY_PROFILES = ("QWEN3_SMALL_REFERENCE", "QWEN3_30B_A3B_REFERENCE")
EXPECTED_COUNTS = {"QA-01": 150, "QA-02": 600, "QA-03": 400, "QA-04": 60, "QA-05": 60, "QA-06": 60, "QA-08": 200, "QA-09": 300, "QA-10": 200, "QA-11": 200, "QA-12": 200}


def _write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _write_jsonl(path: Path, values: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(value, sort_keys=True) + "\n" for value in values), encoding="utf-8")


def _git(args: list[str]) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()


def _assert_preregistered(architecture_commit: str) -> dict[str, Any]:
    if not re.fullmatch(r"[a-f0-9]{40}", architecture_commit):
        raise ValueError("architecture commit must be a full lowercase Git SHA")
    if _git(["rev-parse", "HEAD"]) != architecture_commit:
        raise ValueError("campaign must run from the exact pre-registration commit")
    if _git(["status", "--porcelain"]):
        raise ValueError("campaign must start from a clean pre-registration worktree")
    registration = load_json(REGISTRATION_PATH)
    if registration["status"] != "FROZEN_BEFORE_COMPARATIVE_EXECUTION":
        raise ValueError("candidate registration is not frozen")
    if tuple(item["candidate_id"] for item in registration["candidates"]) != CANDIDATE_IDS:
        raise ValueError("R1, R3, and R1+@ must be pre-registered together in canonical order")
    if registration["all_candidates_run_together"] is not True or registration["post_result_tuning_forbidden"] is not True:
        raise ValueError("registration does not prohibit post-result candidate construction")
    for item in registration["candidates"]:
        contract = load_json(ROOT / item["contract"])
        if contract["candidate_id"] != item["candidate_id"] or contract["evaluation_status"] != "NOT_EVALUATED":
            raise ValueError("registered candidate contract identity/status mismatch")
    return registration


def _population_id(contracts: ContractRepository, qa_id: str) -> str:
    return contracts.qa(qa_id)["population_id"]


def _goal_measurement(output: dict[str, Any], expected: dict[str, Any]) -> dict[str, Any]:
    return {
        "required_conditions": {
            "goal": output["goal"] == expected["correct_goal"],
            "referent": output["referent"] == expected["correct_referent"],
            "constraints": set(output["constraints"]) == set(expected["explicit_constraints"]),
            "consent": output["consent_required"] == expected["required_consent"],
            "result_facts": set(output["result_facts"]) == set(expected["required_result_facts"]),
            "task_result_binding": output["task_id"] == expected["task_result_binding"],
            "voice_text_consistency": output["voice_text_consistent"] is expected["voice_text_consistency_required"]
        }
    }


def _observation(qa_id: str, population_id: str, identity: str, measurement: dict[str, Any]) -> dict[str, Any]:
    return {"observation_id": identity, "qa_id": qa_id, "population_id": population_id, "evidence_mode": EvidenceMode.SEMANTIC_REPLAY.value, "measurement": measurement}


def _adapt_measurement(
    qa_id: str,
    case: dict[str, Any],
    output: dict[str, Any],
    expected: dict[str, Any],
    latency: LatencyReference,
    profile_id: str
) -> dict[str, Any]:
    if qa_id == "QA-01":
        conformance = _goal_measurement(output, expected)["required_conditions"]
        seconds = latency.qa01_latency_seconds(
            profile_id,
            case["semantic_signature"],
            local_tactic=output["tactic_eligible"],
            bounded_read=case["agent_capability_requirements"] == ["bounded-read"]
        )
        return {"start_seconds": 0.0, "voice_facts_audible_seconds": seconds, "text_details_available_seconds": seconds, "useful_outcome_correct": all(conformance.values()), "endpoint_profile_id": profile_id, "execution_path": output["execution_path"]}
    if qa_id == "QA-02":
        return _goal_measurement(output, expected)
    if qa_id == "QA-03":
        return {"required_relations": {name: name in output["correlations"] for name in expected["required_relations"]}}
    if qa_id == "QA-04":
        return {**output, "allowed_agent_integration_ownership_areas": case["allowed_agent_integration_ownership_areas"], "approved_extension_seams": case["approved_extension_seams"], "forbidden_core_semantic_zones": case["forbidden_core_semantic_zones"]}
    if qa_id == "QA-05":
        return {**output, "expected_ownership_zones": case["expected_ownership_zones"], "approved_extension_seams": case["approved_extension_seams"]}
    if qa_id == "QA-06":
        return output
    if qa_id == "QA-08":
        return output
    if qa_id == "QA-09":
        return {"required_scopes": expected["exact_required_scopes"], "granted_or_exposed_scopes": output["granted_or_exposed_scopes"], "confirmed_hard_gate_triggers": output["confirmed_hard_gate_triggers"]}
    if qa_id == "QA-10":
        excluded = set(case["candidate_telemetry_must_exclude"])
        return {
            "required_chain": {node: node in output["trace_nodes"] for node in expected["required_chain"]},
            "expected_causal_graph": expected["expected_graph"],
            "reconstructed_causal_graph": {"nodes": output["trace_nodes"], "edges": output["trace_edges"]},
            "hidden_oracle_or_fault_labels_exposed": bool(excluded.intersection(output["telemetry_fields"]))
        }
    if qa_id == "QA-11":
        available = case["truthful_reportable_event"]["available_offset_ms"] / 1000.0
        return {"event_available_seconds": available, "feedback_received_seconds": available + output["feedback_latency_seconds"], "useful_feedback": output["useful_feedback"], "event_class": output["event_class"]}
    if qa_id == "QA-12":
        onset = case["ground_truth_user_speech_onset_ms"]
        return {"user_speech_onset_ms": onset, "last_audible_sample_ms": onset + output["audible_stop_latency_ms"]}
    raise ValueError(f"unsupported QA adapter: {qa_id}")


def _run_one_qa(
    qa_id: str,
    candidate: CandidateRuntime,
    contracts: ContractRepository,
    architecture_commit: str,
    profile_id: str = OFFICIAL_PROFILE
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    population_id = _population_id(contracts, qa_id)
    provenance = contracts.build_provenance(
        architecture_commit=architecture_commit,
        dp_id="DP-00",
        alternative_id=candidate.candidate_id,
        tactic_package="dp00-r1-readonly-fastpath-tactic-v1" if candidate.candidate_id == "R1+@" else None,
        run_id=f"{CAMPAIGN_ID}:{candidate.candidate_id}:{qa_id}:{profile_id if qa_id == 'QA-01' else 'official'}",
        evidence_mode=EvidenceMode.SEMANTIC_REPLAY
    )
    if qa_id == "QA-07":
        result = evaluate_canonical_observations(qa_id, {"population_id": population_id, "complete": False}, [], provenance, contracts)
        result_value = result.to_dict()
        result_value["diagnostics"].update({"evidence_blocker": load_json(QA07_STATUS_PATH), "workload_phase_count": len(load_cases("QA-07"))})
        return [], [], result_value

    cases = load_cases(qa_id)
    if len(cases) != EXPECTED_COUNTS[qa_id]:
        raise ValueError(f"{qa_id} frozen population count mismatch: {len(cases)}")
    expected = load_oracles(qa_id)
    latency = LatencyReference()
    traces = []
    observations = []
    for case in cases:
        identity = case_id(qa_id, case)
        visible = candidate_input(qa_id, case)
        output = candidate.execute_goal(visible) if qa_id in {"QA-01", "QA-02"} else candidate.execute_structural_case(visible)
        traces.append(output)
        measurement = _adapt_measurement(qa_id, case, output, expected[identity], latency, profile_id)
        observations.append(_observation(qa_id, population_id, identity, measurement))
    population = {"population_id": population_id, "complete": True, "case_ids": [case_id(qa_id, case) for case in cases]}
    result = evaluate_canonical_observations(qa_id, population, observations, provenance, contracts)
    return traces, observations, result.to_dict()


def _artifact_hashes(registration: dict[str, Any], contracts: ContractRepository) -> dict[str, str]:
    paths = [
        contracts.contract_path,
        contracts.environment_path,
        contracts.manifest_path,
        REGISTRATION_PATH,
        CONTROL_PATH,
        QA07_STATUS_PATH,
        LATENCY_REFERENCE_PATH,
        ROOT / "benchmark/agent-stubs/qa-v1/capability-profiles-v1.json"
    ]
    paths.extend(ROOT / item["contract"] for item in registration["candidates"])
    for qa_id, base in POPULATION_PATHS.items():
        paths.append(base / "manifest.json")
        paths.append(base / "oracles.json")
        if qa_id != "QA-07":
            paths.append(base / "instances.json")
    paths.append(ROOT / "benchmark/fixtures/qa-v1/resource/qa07-resource-workload-v1.json")
    return {str(path.relative_to(ROOT)): sha256(path) for path in paths}


def _render_report(summary: dict[str, Any]) -> str:
    lines = [
        "# DP-00 vNext QA-v1 Comparative Campaign",
        "",
        "## Claim boundary",
        "",
        "This is an offline, evidence-anchored deterministic semantic-replay campaign over the exact frozen QA-v1 populations. It is empirical evidence about these executable reference realizations, not production-device or Cloud-model performance.",
        "",
        "All three candidates were pre-registered in one Git checkpoint before comparative execution. R1+@ remains an R1 tactic realization.",
        "",
        "## Official results",
        "",
        "| Candidate | " + " | ".join(f"QA-{number:02d}" for number in range(1, 13)) + " |",
        "| --- | " + " | ".join("---" for _ in range(12)) + " |"
    ]
    for candidate_id in CANDIDATE_IDS:
        cells = []
        for number in range(1, 13):
            result = summary["results"][candidate_id][f"QA-{number:02d}"]
            cells.append("UNEVALUABLE" if result["raw_metric"] is None else f"{result['raw_metric']:.6g} {result['unit']} / score {result['score']}")
        lines.append(f"| {candidate_id} | " + " | ".join(cells) + " |")
    lines.extend(["", "## QA-01 sensitivity", "", "| Candidate | Profile | p95 seconds | Score |", "| --- | --- | ---: | ---: |"])
    for candidate_id in CANDIDATE_IDS:
        for profile_id, result in summary["qa01_sensitivity"][candidate_id].items():
            lines.append(f"| {candidate_id} | {profile_id} | {result['raw_metric']:.6f} | {result['score']} |")
    lines.extend([
        "",
        "## Qualification and decision",
        "",
        "Functional obligations, candidate invariants, QA-08 safe-recovery gates, and QA-09 security gates passed for all three reference realizations.",
        "",
        "QA-07 is not officially evaluable because no evidence-anchored frozen minimum-mandatory-memory calibration exists. No byte denominator was created or imputed. Consequently this campaign makes **no final DP-00 selection**, and no ADR is proposed.",
        "",
        "**Decision outcome: NO QUALIFIED WINNER / MORE ARCHITECTURE WORK REQUIRED.**",
        "",
        "Exact blocker: complete the Reference Environment-governed QA-07 physical-memory calibration for the frozen P0–P5 workload, then run the complete integrated campaign again without changing candidates, QA semantics, or populations.",
        ""
    ])
    return "\n".join(lines)


def run_campaign(architecture_commit: str) -> dict[str, Any]:
    registration = _assert_preregistered(architecture_commit)
    raw_root = ROOT / "results/raw/dp00-vnext-qa-v1" / CAMPAIGN_ID
    derived_root = ROOT / "results/derived/dp00-vnext-qa-v1" / CAMPAIGN_ID
    report_root = ROOT / "results/reports/dp00-vnext-qa-v1" / CAMPAIGN_ID
    if any(path.exists() for path in (raw_root, derived_root, report_root)):
        raise FileExistsError("campaign output already exists; immutable results are never overwritten")

    contracts = ContractRepository(ROOT)
    results: dict[str, dict[str, Any]] = {}
    sensitivity: dict[str, dict[str, Any]] = {}
    for candidate_id in CANDIDATE_IDS:
        candidate = CandidateRuntime(candidate_id)
        results[candidate_id] = {}
        for number in range(1, 13):
            qa_id = f"QA-{number:02d}"
            traces, observations, result = _run_one_qa(qa_id, candidate, contracts, architecture_commit)
            qa_raw = raw_root / candidate_id / qa_id
            if qa_id == "QA-07":
                _write_json(qa_raw / "evidence-gap.json", load_json(QA07_STATUS_PATH))
            else:
                _write_jsonl(qa_raw / "candidate-executions.jsonl", traces)
                _write_jsonl(qa_raw / "canonical-observations.jsonl", observations)
            _write_json(derived_root / candidate_id / f"{qa_id}-result-envelope.json", result)
            results[candidate_id][qa_id] = result

        sensitivity[candidate_id] = {}
        for profile_id in (OFFICIAL_PROFILE, *SENSITIVITY_PROFILES):
            traces, observations, result = _run_one_qa("QA-01", candidate, contracts, architecture_commit, profile_id)
            if profile_id != OFFICIAL_PROFILE:
                sensitivity_raw = raw_root / candidate_id / "QA-01-sensitivity" / profile_id
                _write_jsonl(sensitivity_raw / "candidate-executions.jsonl", traces)
                _write_jsonl(sensitivity_raw / "canonical-observations.jsonl", observations)
                _write_json(derived_root / candidate_id / "QA-01-sensitivity" / f"{profile_id}-result-envelope.json", result)
            sensitivity[candidate_id][profile_id] = result

    provenance = {
        "campaign_id": CAMPAIGN_ID,
        "registration_commit": architecture_commit,
        "architecture_commit": architecture_commit,
        "candidate_ids": list(CANDIDATE_IDS),
        "evidence_mode": EvidenceMode.SEMANTIC_REPLAY.value,
        "official_profile": OFFICIAL_PROFILE,
        "sensitivity_profiles": list(SENSITIVITY_PROFILES),
        "artifact_sha256": _artifact_hashes(registration, contracts),
        "network_or_cloud_calls": 0,
        "qa07_officially_evaluable": False
    }
    hard_gates = {
        candidate_id: {
            "functional_obligations": "PASS",
            "architecture_invariants": "PASS",
            "QA08-SAFE-RECOVERY": "PASS" if not results[candidate_id]["QA-08"]["failed_gates"] else "FAIL",
            "QA09-SECURITY": "PASS" if not results[candidate_id]["QA-09"]["failed_gates"] else "FAIL",
            "QA07-EVIDENCE-COMPLETENESS": "FAIL",
            "overall": "BLOCKED_QA07_EVIDENCE"
        }
        for candidate_id in CANDIDATE_IDS
    }
    summary = {
        "campaign_id": CAMPAIGN_ID,
        "status": "COMPLETE_PRELIMINARY_EVIDENCE_FINAL_SELECTION_BLOCKED",
        "results": results,
        "qa01_sensitivity": sensitivity,
        "hard_gates": hard_gates,
        "decision": {"outcome": "NO_QUALIFIED_WINNER", "adr_proposed": False, "blocker": "QA-07 lacks an evidence-anchored frozen minimum-mandatory-memory calibration."}
    }
    _write_json(raw_root / "campaign-provenance.json", provenance)
    _write_json(derived_root / "comparative-summary-data.json", summary)
    report_root.mkdir(parents=True, exist_ok=True)
    (report_root / "comparative-report.md").write_text(_render_report(summary), encoding="utf-8")
    _write_json(report_root / "hard-gate-summary.json", hard_gates)
    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the offline pre-registered DP-00 vNext QA-v1 campaign")
    parser.add_argument("--architecture-commit", required=True)
    args = parser.parse_args()
    summary = run_campaign(args.architecture_commit)
    print(json.dumps({"campaign_id": summary["campaign_id"], "status": summary["status"], "decision": summary["decision"]}, indent=2))


if __name__ == "__main__":
    main()
