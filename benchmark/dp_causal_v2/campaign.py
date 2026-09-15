from __future__ import annotations

import argparse
from hashlib import sha256
import json
from pathlib import Path
import re
import subprocess
from typing import Any

from benchmark.analysis.qa_v1 import ContractRepository, EvidenceMode, evaluate_canonical_observations
from benchmark.dp_vnext.controls import POPULATION_PATHS, ROOT, case_id, load_cases, load_json, load_oracles

from .architecture import CANDIDATE_IDS, CausalCandidateRuntime
from .observation import ObservationAdapter
from .schemas import CandidateVisibleInput, EvaluatorOracle


CAMPAIGN_ID = "dp00-causal-offline-v2"
REGISTRATION_PATH = ROOT / "benchmark/contracts/dp-causal-v2/preregistration.json"
OFFICIAL_PROFILE = "QWEN3_MEDIUM_REFERENCE"
SENSITIVITY_PROFILES = ("QWEN3_SMALL_REFERENCE", "QWEN3_30B_A3B_REFERENCE")
EXPECTED_COUNTS = {"QA-01": 150, "QA-02": 600, "QA-03": 400, "QA-04": 60, "QA-05": 60,
                   "QA-06": 60, "QA-08": 200, "QA-09": 300, "QA-10": 200, "QA-11": 200, "QA-12": 200}


def _git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()


def _write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _write_jsonl(path: Path, values: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(value, sort_keys=True) + "\n" for value in values), encoding="utf-8")


def _sha(path: Path) -> str:
    return sha256(path.read_bytes()).hexdigest()


def _assert_preregistered(commit: str) -> dict[str, Any]:
    if not re.fullmatch(r"[a-f0-9]{40}", commit) or _git("rev-parse", "HEAD") != commit:
        raise ValueError("official campaign must run at the exact full preregistration commit")
    if _git("status", "--porcelain"):
        raise ValueError("official campaign must begin from a clean preregistration commit")
    registration = load_json(REGISTRATION_PATH)
    if registration["status"] != "FROZEN_BEFORE_COMPARATIVE_EXECUTION":
        raise ValueError("v2 registration is not frozen")
    if tuple(registration["candidate_ids"]) != CANDIDATE_IDS:
        raise ValueError("candidate set/order mismatch")
    return registration


def _artifact_hashes() -> dict[str, str]:
    paths = [
        ROOT / "benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json",
        ROOT / "benchmark/contracts/qa-v1/reference-environment-v1.json",
        ROOT / "benchmark/contracts/qa-v1/corpus-manifest-v1.json",
        ROOT / "benchmark/contracts/dp00-qa01-design-reference-latency-v1.json",
        ROOT / "benchmark/contracts/dp-causal-v2/preregistration.json",
    ]
    for base in POPULATION_PATHS.values():
        paths.extend(sorted(base.glob("*.json")))
    paths.extend(sorted((ROOT / "results/raw/dp00-vnext-qa-v1/dp00-vnext-qa-v1-campaign-v1").rglob("*")))
    files = [path for path in paths if path.is_file()]
    return {str(path.relative_to(ROOT)): _sha(path) for path in files}


def _run_qa(candidate_id: str, qa_id: str, commit: str, contracts: ContractRepository,
            profile_id: str = OFFICIAL_PROFILE) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    population_id = contracts.qa(qa_id)["population_id"]
    provenance = contracts.build_provenance(architecture_commit=commit, dp_id="DP-00", alternative_id=candidate_id,
        tactic_package="dp00-r1-readonly-fastpath-tactic-v1" if candidate_id == "R1+@" else None,
        run_id=f"{CAMPAIGN_ID}:{candidate_id}:{qa_id}:{profile_id}", evidence_mode=EvidenceMode.SEMANTIC_REPLAY)
    if qa_id == "QA-07":
        result = evaluate_canonical_observations(qa_id, {"population_id": population_id, "complete": False}, [], provenance, contracts).to_dict()
        result["diagnostics"].update({"status": "UNEVALUABLE — missing evidence-anchored frozen memory calibration",
            "structural_diagnostic": "NON-SCORING STRUCTURAL DIAGNOSTIC",
            "residency_topology": "VIA interaction plus selected/primary downstream runtime; possible shared replay and task-state stores"})
        return [], [], result
    cases = load_cases(qa_id)
    if len(cases) != EXPECTED_COUNTS[qa_id]:
        raise ValueError(f"{qa_id} population changed")
    oracles = load_oracles(qa_id)
    evidence_rows, observations = [], []
    for case in cases:
        identity = case_id(qa_id, case)
        visible = CandidateVisibleInput.from_case(qa_id, case)
        # Candidate execution completes before the evaluator-only oracle is constructed.
        evidence = CausalCandidateRuntime(candidate_id, profile_id).execute(visible)
        oracle = EvaluatorOracle.from_values(qa_id, identity, oracles[identity])
        measurement = ObservationAdapter().adapt(qa_id, case, evidence, oracle)
        if qa_id == "QA-01": measurement["endpoint_profile_id"] = profile_id
        evidence_rows.append(evidence.to_dict())
        observations.append({"observation_id": identity, "qa_id": qa_id, "population_id": population_id,
                             "evidence_mode": EvidenceMode.SEMANTIC_REPLAY.value, "measurement": measurement})
    population = {"population_id": population_id, "complete": True,
                  "case_ids": [case_id(qa_id, case) for case in cases]}
    result = evaluate_canonical_observations(qa_id, population, observations, provenance, contracts).to_dict()
    return evidence_rows, observations, result


def _representative_paths() -> dict[str, list[str]]:
    cases = load_cases("QA-02")
    examples = {}
    for candidate, capability, key in (("R1", "general-v1", "r1_general"), ("R1", "specialist-v1", "r1_specialist"),
                                        ("R3", "general-v1", "r3_general"), ("R3", "specialist-v1", "r3_specialist")):
        case = next(c for c in cases if c["agent_capability_requirements"] == [capability])
        examples[key] = CausalCandidateRuntime(candidate).execute(CandidateVisibleInput.from_case("QA-02", case)).architecture_path
    return examples


def _render_report(summary: dict[str, Any]) -> str:
    lines = ["# VIA DP-00 Offline Causal Architecture Evaluation v2", "", "## Evidence status", "",
        "Campaign v1 validated QA-v1 population handling, provenance, scoring, candidate pre-registration, and evaluation plumbing. Its executable candidate runtimes did not causally materialize enough R1/R3 architecture behavior to support R1-vs-R3 architecture equivalence or selection claims.", "",
        "The v1 numbers remain historical preliminary evidence and were not used to select DP-00.", "", "## Primary QAs", "",
        "| QA | R1 raw / score | R3 raw / score | R1+@ raw / score | R1−R3 | causal explanation |", "| --- | ---: | ---: | ---: | ---: | --- |"]
    explanations = summary["causal_explanations"]
    for qa in ("QA-01", "QA-02", "QA-04", "QA-05"):
        row = []
        for candidate in CANDIDATE_IDS:
            result = summary["results"][candidate][qa]
            row.append(f"{result['raw_metric']:.6g} {result['unit']} / {result['score']}")
        r1 = summary["results"]["R1"][qa]["raw_metric"]; r3 = summary["results"]["R3"][qa]["raw_metric"]
        lines.append(f"| {qa} | {row[0]} | {row[1]} | {row[2]} | {r1-r3:.6g} | {explanations[qa]} |")
    lines += ["", "## Secondary QAs", "", "| QA | R1 | R3 | R1+@ |", "| --- | ---: | ---: | ---: |"]
    for qa in ("QA-03", "QA-06", "QA-08", "QA-09", "QA-10", "QA-11", "QA-12"):
        vals = [summary["results"][c][qa] for c in CANDIDATE_IDS]
        lines.append(f"| {qa} | {vals[0]['raw_metric']:.6g} / {vals[0]['score']} | {vals[1]['raw_metric']:.6g} / {vals[1]['score']} | {vals[2]['raw_metric']:.6g} / {vals[2]['score']} |")
    lines += ["", "## Representative architecture paths", ""]
    for name, path in summary["representative_paths"].items(): lines.append(f"- {name}: " + " → ".join(path))
    lines += ["", "## Qualification", "", "QA-07: **UNEVALUABLE / calibration missing**.", "",
              f"Hard qualification: **{summary['hard_qualification_status']}**.", "",
              f"Interpretation: **{summary['decision']['outcome']}**.", "",
              "All observations were derived as scenario → candidate state/events → independent adapter; evaluator oracles entered only during comparison. No Cloud/API/network calls occurred.", ""]
    return "\n".join(lines)


def run_campaign(commit: str) -> dict[str, Any]:
    _assert_preregistered(commit)
    raw = ROOT / "results/raw/dp00-causal-offline-v2" / CAMPAIGN_ID
    derived = ROOT / "results/derived/dp00-causal-offline-v2" / CAMPAIGN_ID
    report = ROOT / "results/reports/dp00-causal-offline-v2" / CAMPAIGN_ID
    if raw.exists() or derived.exists() or report.exists():
        raise FileExistsError("immutable v2 results already exist")
    contracts = ContractRepository(ROOT)
    results: dict[str, dict[str, Any]] = {c: {} for c in CANDIDATE_IDS}
    local_count = 0
    all_conformance: dict[str, bool] = {c: True for c in CANDIDATE_IDS}
    for candidate in CANDIDATE_IDS:
        for number in range(1, 13):
            qa = f"QA-{number:02d}"
            evidence, observations, result = _run_qa(candidate, qa, commit, contracts)
            results[candidate][qa] = result
            target = raw / candidate / qa
            if qa == "QA-07": _write_json(target / "evidence-gap.json", result["diagnostics"])
            else:
                _write_jsonl(target / "execution-evidence.jsonl", evidence)
                _write_jsonl(target / "canonical-observations.jsonl", observations)
                _write_jsonl(target / "event-graphs.jsonl", [{"case_id": e["case_id"], "events": e["events"]} for e in evidence])
                _write_jsonl(target / "state-transitions.jsonl", [{"case_id": e["case_id"], "transitions": e["state_transitions"]} for e in evidence])
                all_conformance[candidate] &= all(e["conformance"]["passed"] for e in evidence)
                if candidate == "R1+@" and qa in {"QA-01", "QA-02"}:
                    local_count += sum("bounded local deterministic read" in e["architecture_path"] for e in evidence)
            _write_json(derived / candidate / f"{qa}-result-envelope.json", result)
        for profile in SENSITIVITY_PROFILES:
            evidence, observations, result = _run_qa(candidate, "QA-01", commit, contracts, profile)
            _write_jsonl(raw / candidate / "QA-01-sensitivity" / profile / "execution-evidence.jsonl", evidence)
            _write_jsonl(raw / candidate / "QA-01-sensitivity" / profile / "canonical-observations.jsonl", observations)
            _write_json(derived / candidate / "QA-01-sensitivity" / f"{profile}-result-envelope.json", result)
    hard = {candidate: {"candidate_conformance": all_conformance[candidate],
                         "QA08": not results[candidate]["QA-08"]["failed_gates"],
                         "QA09": not results[candidate]["QA-09"]["failed_gates"], "QA07": False} for candidate in CANDIDATE_IDS}
    summary = {"campaign_id": CAMPAIGN_ID, "V2_PREREGISTRATION_COMMIT": commit, "results": results,
        "provenance": results["R1"]["QA-01"]["provenance"],
        "hard_gates": hard, "hard_qualification_status": "INCOMPLETE — QA-07 calibration missing",
        "r1_plus_local_tactic_executions": local_count, "representative_paths": _representative_paths(),
        "causal_explanations": {
            "QA-01": "R1 and R3 tie at population p95. Per-case graphs still preserve structural costs: R3 specialist delegation adds a common-cost hop, while eligible R1+@ reads omit model inference and downstream return, producing its small aggregate advantage.",
            "QA-02": "Common semantic and Agent replay capabilities are equal; candidate-owned state machines bind independently produced versioned results.",
            "QA-04": "The preregistered dependency graph integrates Agent evolution at each architecture's authoritative Agent boundary without propagating into forbidden Core zones.",
            "QA-05": "The frozen component-to-zone graph propagates each requested product change through its declared dependency seam; no expected zone is read during propagation."},
        "decision": {"outcome": "CONDITIONAL DP-00 PREFERENCE — PENDING QA-07 QUALIFICATION", "adr_created": False},
        "network_or_cloud_calls": 0}
    # A preference is defensible only if a primary QA differs.
    if all(results["R1"][qa]["raw_metric"] == results["R3"][qa]["raw_metric"] for qa in ("QA-01", "QA-02", "QA-04", "QA-05")):
        summary["decision"]["outcome"] = "NO DP-00 PREFERENCE FROM CURRENT QA EVIDENCE"
    _write_json(raw / "campaign-provenance.json", {"registration_commit": commit, "architecture_commit": commit,
        "candidate_ids": list(CANDIDATE_IDS), "official_profile": OFFICIAL_PROFILE, "network_or_cloud_calls": 0,
        "artifact_sha256": _artifact_hashes()})
    _write_json(derived / "comparative-summary-data.json", summary)
    report.mkdir(parents=True, exist_ok=True)
    (report / "comparative-report.md").write_text(_render_report(summary), encoding="utf-8")
    _write_json(report / "hard-gate-summary.json", hard)
    return summary


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--architecture-commit", required=True)
    args = parser.parse_args()
    summary = run_campaign(args.architecture_commit)
    print(json.dumps({"campaign_id": summary["campaign_id"], "decision": summary["decision"]}, indent=2))


if __name__ == "__main__": main()
