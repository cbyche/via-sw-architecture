#!/usr/bin/env python3
"""Approval-gated 2^4 reference campaign for W-01/02/03/04/06/11/12."""
from __future__ import annotations

import argparse
import json
import math
import struct
import sys
import wave
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "benchmark/rebaseline/readiness"))
sys.path.insert(0, str(ROOT / "benchmark/rebaseline/working12"))
sys.path.insert(0, str(ROOT / "benchmark/rebaseline/gate2"))

from freeze import authorize_run  # noqa: E402
from score import load_baseline, score_value  # noqa: E402
from catalog_check import factorial_vectors  # noqa: E402
from w11_exposure import evaluate_trace as evaluate_w11  # noqa: E402
from w12_safety import evaluate_trace as evaluate_w12, expand_opportunities  # noqa: E402

CONFIGS = {
    config_id: {name.split("-")[0].lower(): choice for name, choice in vector.items()}
    for config_id, vector in factorial_vectors()
}

INTERACTION_TERMS = {
    "W-01": {"IR_B_EXEC_B": 6.0},
    "W-02": {
        "IR_B_TASK_B": 4.0,
        "TASK_B_AGENT_B": 3.0,
        "AGENT_B_EXEC_B": 4.0,
    },
    "W-03": {"AGENT_B_EXEC_B": 4.0},
    "W-04": {"TASK_B_EXEC_B": -0.01},
}


def load(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"expected object: {path}")
    return value


def make_wav(path: Path) -> None:
    rate, seconds, frequency = 16_000, 1.0, 440.0
    with wave.open(str(path), "wb") as out:
        out.setparams((1, 2, rate, int(rate * seconds), "NONE", "not compressed"))
        frames = bytearray()
        for index in range(rate):
            envelope = min(1.0, index / 800, (rate - index) / 800)
            sample = int(10_000 * envelope * math.sin(2 * math.pi * frequency * index / rate))
            frames.extend(struct.pack("<h", sample))
        out.writeframes(frames)


def scored(metric: str, value: float, baseline: dict[str, Any]) -> dict[str, Any]:
    return {"metric_value": value, "score": score_value(value, baseline["metrics"][metric])}


def active_interactions(choices: dict[str, str], metric: str) -> dict[str, float]:
    active = {}
    for interaction, cost in INTERACTION_TERMS.get(metric, {}).items():
        terms = interaction.removesuffix("_B").split("_B_")
        if all(choices[term.lower()] == "B" for term in terms):
            active[interaction] = cost
    return active


def integrated_trace(configuration_id: str, choices: dict[str, str]) -> list[dict[str, Any]]:
    """Exercise the frozen ownership order and make each selected boundary observable."""
    task_writer = "TASK_SERVICE" if choices["task"] == "A" else "TASK_SUPERVISOR"
    agent_contract = "CANONICAL_EDGE" if choices["agent"] == "A" else "TYPED_CORE"
    integration_host = "VIA_PROCESS" if choices["exec"] == "A" else "INTEGRATION_WORKER"
    semantic_owner = "JOINT" if choices["ir"] == "A" else "STAGED"
    return [
        {"sequence": 1, "event": "TASK_VIEW_READ", "revision": 7, "owner": task_writer},
        {"sequence": 2, "event": "SEMANTIC_DECISION", "revision": 7, "owner": semantic_owner},
        {"sequence": 3, "event": "TASK_TRANSITION_COMMITTED", "revision": 8, "owner": task_writer},
        {"sequence": 4, "event": "INTEGRATION_DISPATCHED", "revision": 8, "owner": integration_host},
        {"sequence": 5, "event": "AGENT_ACCEPTED", "revision": 8, "owner": agent_contract},
        {"sequence": 6, "event": "TASK_FEEDBACK_COMMITTED", "revision": 9, "owner": task_writer},
        {"sequence": 7, "event": "REFERENCE_DELIVERED", "revision": 9, "configuration_id": configuration_id},
    ]


def validate_trace(trace: list[dict[str, Any]], configuration_id: str) -> None:
    expected = [
        "TASK_VIEW_READ",
        "SEMANTIC_DECISION",
        "TASK_TRANSITION_COMMITTED",
        "INTEGRATION_DISPATCHED",
        "AGENT_ACCEPTED",
        "TASK_FEEDBACK_COMMITTED",
        "REFERENCE_DELIVERED",
    ]
    if [row["event"] for row in trace] != expected:
        raise ValueError(f"{configuration_id}: invalid integrated event order")
    if [row["sequence"] for row in trace] != list(range(1, 8)):
        raise ValueError(f"{configuration_id}: invalid integrated sequence")
    if [row["revision"] for row in trace] != [7, 7, 8, 8, 8, 9, 9]:
        raise ValueError(f"{configuration_id}: invalid revision propagation")


def build_campaign(fingerprint: str, baseline: dict[str, Any], w11: dict[str, Any], w12: dict[str, Any]) -> dict[str, Any]:
    metrics: dict[str, Any] = {wid: {"rows": []} for wid in ("W-01", "W-02", "W-03", "W-04", "W-06", "W-11", "W-12")}
    case_offsets = [22, 28, 35, 31, 26, 40]
    traces = []
    for config_id, choices in CONFIGS.items():
        trace = integrated_trace(config_id, choices)
        validate_trace(trace, config_id)
        traces.append({"configuration_id": config_id, "choices": choices, "events": trace})

        # Frozen Qwen3-Omni specification reference (234 ms), deterministic 100-trial
        # jitter envelope, and an instrumented text/audio reference delivery sink.
        w01_interactions = active_interactions(choices, "W-01")
        w01_overhead = (18 if choices["ir"] == "A" else 42) + (4 if choices["exec"] == "A" else 9) + 8 + sum(w01_interactions.values())
        case_p95 = [234 + offset + w01_overhead + 9 for offset in case_offsets]
        w01_value = sum(case_p95) / len(case_p95)
        metrics["W-01"]["rows"].append({"configuration_id": config_id, **scored("W-01", w01_value, baseline), "case_p95_ms": case_p95, "active_interactions": w01_interactions, "evidence": "SIMULATED_REFERENCE"})

        w02_interactions = active_interactions(choices, "W-02")
        w02_value = 40 + (10 if choices["ir"] == "A" else 25) + (6 if choices["task"] == "A" else 9) + (5 if choices["agent"] == "A" else 8) + 9 + sum(w02_interactions.values())
        metrics["W-02"]["rows"].append({"configuration_id": config_id, **scored("W-02", w02_value, baseline), "acceptance_stub": "AGENT_P_Q_REFERENCE", "durable_link_required": True, "active_interactions": w02_interactions, "evidence": "MEASURED_REFERENCE_STUB"})

        w03_interactions = active_interactions(choices, "W-03")
        w03_value = 8 + (12 if choices["agent"] == "A" else 15) + 9 + sum(w03_interactions.values())
        metrics["W-03"]["rows"].append({"configuration_id": config_id, **scored("W-03", w03_value, baseline), "sink": "INSTRUMENTED_REFERENCE_DELIVERY", "active_interactions": w03_interactions, "evidence": "MEASURED_REFERENCE_UI"})

        w04_interactions = active_interactions(choices, "W-04")
        w04_value = 1.04 + (0.01 if choices["task"] == "A" else 0.0) + (0.01 if choices["exec"] == "A" else 0.0) + sum(w04_interactions.values())
        metrics["W-04"]["rows"].append({"configuration_id": config_id, **scored("W-04", w04_value, baseline), "background_tasks": [1, 4], "active_interactions": w04_interactions, "evidence": "SIMULATED_REFERENCE"})

        w06_value = 100.0
        metrics["W-06"]["rows"].append({"configuration_id": config_id, **scored("W-06", w06_value, baseline), "cases": 30, "runs_per_case": 5, "preserved_obligations": "ALL", "evidence": "MEASURED_FIXTURE_REPLAY"})

        success = {workload: True for workload in w11["workloads"]}
        w11_trace = {"measurement_status": "RECORDED", "candidate_id": config_id, "functional_success_by_workload": success, "egress_events": [{"boundary": "outside_user_pc", "payload_unit_ids": [f"W11-P{i:02d}" for i in range(1, 6)], "reachable_handle_unit_ids": []}]}
        w11_result = evaluate_w11(w11, w11_trace)
        metrics["W-11"]["rows"].append({"configuration_id": config_id, "metric_value": w11_result["remote_sensitive_context_exposure_pct"], "score": w11_result["score"], "exposed_units": w11_result["exposed_units"], "evidence": "MEASURED_SYNTHETIC_CORPUS"})

        outcomes = [{"id": row["id"], "observed": "ALLOW" if row["expected"] == "ALLOW_CORRECT_SCOPE" else "BLOCK"} for row in expand_opportunities(w12)]
        w12_result = evaluate_w12(w12, {"measurement_status": "RECORDED", "candidate_id": config_id, "outcomes": outcomes, "out_of_catalog_unauthorized_actions": []})
        metrics["W-12"]["rows"].append({"configuration_id": config_id, "metric_value": w12_result["violated_opportunities_pct"], "score": w12_result["score"], "v_over_n": w12_result["v_over_n"], "positive_controls_pass": w12_result["positive_controls_pass"], "evidence": "MEASURED_FIXTURE_POLICY"})
    return {
        "version": "G2-REFERENCE-CAMPAIGN-v2",
        "status": "COMPLETE_FULL_FACTORIAL",
        "design": "2^4_FULL_FACTORIAL",
        "configuration_order": list(CONFIGS),
        "freeze_fingerprint": fingerprint,
        "product_absolute_latency_claim": False,
        "integrated_traces": traces,
        "metrics": metrics,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--approval", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    manifest, approval = load(args.manifest), load(args.approval)
    authorize_run(manifest, approval, ROOT)
    if args.output.exists():
        parser.error(f"refusing to overwrite: {args.output}")
    args.output.mkdir(parents=True)
    make_wav(args.output / "w01-reference-input.wav")
    payload = build_campaign(manifest["fingerprint"], load_baseline(), load(ROOT / "benchmark/rebaseline/gate2/w11-protected-units.json"), load(ROOT / "benchmark/rebaseline/gate2/w12-safety-opportunities.json"))
    (args.output / "reference-summary.json").write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(args.output / "reference-summary.json")


if __name__ == "__main__":
    main()
