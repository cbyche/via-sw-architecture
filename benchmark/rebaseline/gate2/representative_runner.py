#!/usr/bin/env python3
"""Approval-gated W-09/W-10 representative measurement orchestrator."""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
READINESS = ROOT / "benchmark/rebaseline/readiness"
WORKING12 = ROOT / "benchmark/rebaseline/working12"
sys.path.insert(0, str(READINESS))
sys.path.insert(0, str(WORKING12))

from freeze import authorize_run  # noqa: E402
from score import load_baseline, score_value  # noqa: E402
from catalog_check import factorial_vectors  # noqa: E402

CONFIGURATIONS = {
    config_id: {
        "task": "shared" if vector["TASK-DP01"] == "A" else "per_task",
        "exec": "shared" if vector["EXEC-DP01"] == "A" else "isolated",
    }
    for config_id, vector in factorial_vectors()
}


def load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"expected JSON object: {path}")
    return value


def run_json(command: list[str]) -> dict[str, Any]:
    print(f"RUN {' '.join(command)}", flush=True)
    result = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=None,
        check=True,
    )
    value = json.loads(result.stdout)
    if not isinstance(value, dict) or value.get("status") != "PASS":
        raise RuntimeError(f"representative command did not return PASS: {command}")
    return value


def index_candidates(payload: dict[str, Any], key: str) -> dict[str, dict[str, Any]]:
    candidates = payload.get("candidates")
    if not isinstance(candidates, list):
        raise ValueError("representative result lacks candidates")
    result = {candidate[key]: candidate for candidate in candidates}
    if len(result) != len(candidates):
        raise ValueError(f"duplicate representative candidate key: {key}")
    return result


def assemble_w09(
    whole: dict[str, Any], fatal: dict[str, Any], baseline: dict[str, Any]
) -> dict[str, Any]:
    if whole.get("profile") != "frozen" or fatal.get("profile") != "frozen":
        raise ValueError("W-09 representative assembly requires frozen profiles")
    if whole.get("w09_strata_metric_eligible") is not True:
        raise ValueError("whole-process W-09 result is not metric eligible")
    if fatal.get("w09_strata_metric_eligible") is not True:
        raise ValueError("integration-fatal W-09 result is not metric eligible")
    fingerprint = whole.get("freeze_fingerprint")
    if not isinstance(fingerprint, str) or len(fingerprint) != 64:
        raise ValueError("whole-process W-09 result lacks freeze fingerprint")
    if fatal.get("freeze_fingerprint") != fingerprint:
        raise ValueError("W-09 strata were produced under different freeze fingerprints")
    task = index_candidates(whole, "task_candidate")
    execution = index_candidates(fatal, "exec_candidate")
    rows = []
    for configuration, choices in CONFIGURATIONS.items():
        task_strata = task[choices["task"]]["strata"]
        exec_strata = execution[choices["exec"]]["strata"]
        if len(task_strata) != 4 or len(exec_strata) != 2:
            raise ValueError(f"{configuration}: W-09 requires exactly six strata")
        p95_values = [float(row["p95_recovery_ms"]) for row in task_strata + exec_strata]
        metric = sum(p95_values) / 6
        rows.append(
            {
                "configuration_id": configuration,
                "task_candidate": choices["task"],
                "exec_candidate": choices["exec"],
                "stratum_p95_recovery_ms": p95_values,
                "metric_value": metric,
                "score": score_value(metric, baseline["metrics"]["W-09"]),
                "factorial_axes_exercised": ["TASK-DP01", "EXEC-DP01"],
                "evidence": "MEASURED_STRUCTURAL",
            }
        )
    return {"working_asr": "W-09", "rows": rows}


def assemble_w10(payload: dict[str, Any], baseline: dict[str, Any]) -> dict[str, Any]:
    if payload.get("profile") != "frozen" or payload.get("w10_metric_eligible") is not True:
        raise ValueError("W-10 representative assembly requires a metric-eligible frozen profile")
    fingerprint = payload.get("freeze_fingerprint")
    if not isinstance(fingerprint, str) or len(fingerprint) != 64:
        raise ValueError("W-10 result lacks freeze fingerprint")
    execution = index_candidates(payload, "exec_candidate")
    rows = []
    for configuration, choices in CONFIGURATIONS.items():
        candidate = execution[choices["exec"]]
        if len(candidate.get("external_cells", [])) != 24 or len(candidate.get("fatal_cells", [])) != 4:
            raise ValueError(f"{configuration}: W-10 requires exactly 28 cells")
        metric = float(candidate["controller_retention_pct"])
        rows.append(
            {
                "configuration_id": configuration,
                "exec_candidate": choices["exec"],
                "passed_cells": int(candidate["passed_cells"]),
                "metric_value": metric,
                "score": score_value(metric, baseline["metrics"]["W-10"]),
                "factorial_axes_exercised": ["EXEC-DP01"],
                "evidence": "MEASURED_STRUCTURAL",
            }
        )
    return {"working_asr": "W-10", "rows": rows}


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--approval", type=Path, required=True)
    parser.add_argument("--bin-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--suite", choices=("w09", "w10", "all"), default="all")
    args = parser.parse_args()

    manifest = load_json(args.manifest)
    approval = load_json(args.approval)
    authorize_run(manifest, approval, ROOT)
    if args.output.exists():
        parser.error(f"refusing to overwrite output directory: {args.output}")
    args.output.mkdir(parents=True)

    bench = (args.bin_dir / "gate2-bench").resolve()
    host = (args.bin_dir / "gate2-host").resolve()
    worker = (args.bin_dir / "gate2-worker").resolve()
    for binary in (bench, host, worker):
        if not binary.is_file():
            parser.error(f"missing built binary: {binary}")

    baseline = load_baseline()
    summary: dict[str, Any] = {
        "status": "REPRESENTATIVE_MEASUREMENT_COMPLETE",
        "freeze_fingerprint": manifest["fingerprint"],
        "evidence": "MEASURED_STRUCTURAL",
        "target_windows_absolute_performance_claim": False,
        "metrics": {},
    }
    if args.suite in {"w09", "all"}:
        whole = run_json(
            [
                str(bench),
                "w09-whole-process",
                "--profile",
                "frozen",
                "--freeze-fingerprint",
                manifest["fingerprint"],
            ]
        )
        fatal = run_json(
            [
                str(bench),
                "w09-integration-fatal",
                "--host",
                str(host),
                "--worker",
                str(worker),
                "--profile",
                "frozen",
                "--freeze-fingerprint",
                manifest["fingerprint"],
            ]
        )
        write_json(args.output / "w09-whole-process.json", whole)
        write_json(args.output / "w09-integration-fatal.json", fatal)
        summary["metrics"]["W-09"] = assemble_w09(whole, fatal, baseline)

    if args.suite in {"w10", "all"}:
        containment = run_json(
            [
                str(bench),
                "w10-containment",
                "--spec",
                str(ROOT / "benchmark/rebaseline/gate2/w10-containment-cells.json"),
                "--host",
                str(host),
                "--worker",
                str(worker),
                "--profile",
                "frozen",
                "--freeze-fingerprint",
                manifest["fingerprint"],
            ]
        )
        write_json(args.output / "w10-containment.json", containment)
        summary["metrics"]["W-10"] = assemble_w10(containment, baseline)

    write_json(args.output / "representative-summary.json", summary)
    print(args.output / "representative-summary.json")


if __name__ == "__main__":
    main()
