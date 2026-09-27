#!/usr/bin/env python3
"""Run the frozen VIA-DP-06 v5 breadth plus repeated-latency campaign."""

from __future__ import annotations

import argparse
import json
import platform
from pathlib import Path
import subprocess
import time
from typing import Any

from run_dp06_evaluation import (
    AGENT_BIN,
    CHANGE_EXERCISES,
    FIXTURE,
    FOUNDATION,
    ORACLE,
    compile_probe,
    ensure_model,
    generate_wav,
    jsonl_append,
    load_foundation,
    load_json,
    run_change_exercises,
    run_trial,
    sha256_file,
)


ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-06-evaluation-v5.json"


def write_json(path: Path, value: Any) -> None:
    path.write_text(
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def git_revision() -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()


def run_record(
    *,
    endpoint: str,
    candidate: str,
    case: dict[str, Any],
    trial: int,
    stratum: str,
    warmup: bool,
    fixture: dict[str, Any],
    expected_by_case: dict[str, dict[str, Any]],
    foundation: dict[str, Any],
    output_dir: Path,
) -> dict[str, Any]:
    started = time.monotonic_ns()
    try:
        record = run_trial(
            endpoint=endpoint,
            candidate=candidate,
            fixture=fixture,
            case=dict(case),
            expected=expected_by_case[case["id"]],
            foundation=foundation,
            output_dir=output_dir,
            trial=trial,
            stratum=stratum,
            warmup=warmup,
        )
        record["schema_version"] = "via.dp06.evaluation-trial.v5"
        record["runner_elapsed_ns"] = time.monotonic_ns() - started
        record["runner_failure"] = None
        return record
    except Exception as error:
        # Failures remain first-class raw samples. The campaign continues so a
        # single bad endpoint cannot silently remove a tail observation.
        return {
            "schema_version": "via.dp06.evaluation-trial.v5",
            "evidence_label": "MEASURED_REFERENCE_HARNESS",
            "candidate": candidate,
            "case_id": case["id"],
            "trial": trial,
            "stratum": stratum,
            "warmup": warmup,
            "source_execution_key": (
                f"{stratum}:{'warmup' if warmup else 'scored'}:"
                f"{candidate}:{case['id']}:{trial}"
            ),
            "runner_elapsed_ns": time.monotonic_ns() - started,
            "runner_failure": f"{type(error).__name__}: {error}",
            "trace_complete": False,
            "trace_missing": ["trial_execution_failed"],
        }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--endpoint", default="http://127.0.0.1:18080")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--candidate", action="append", choices=["A", "B", "B_PRIME"])
    args = parser.parse_args()

    freeze = load_json(CONTRACT)
    fixture = load_json(FIXTURE)
    oracle = load_json(ORACLE)
    foundation = load_foundation(freeze)
    cases_by_id = {item["id"]: item for item in fixture["cases"]}
    expected_by_case = {item["id"]: item["expected"] for item in oracle["cases"]}
    candidates = args.candidate or freeze["candidates"]

    breadth_cases = list(fixture["cases"])
    latency_ids = freeze["strata"]["latency_repetition"]["cases"]
    latency_cases = [cases_by_id[case_id] for case_id in latency_ids]
    warmups = freeze["strata"]["latency_repetition"]["warmup_trials_per_case"]
    latency_trials = freeze["strata"]["latency_repetition"]["scored_trials_per_case"]

    ensure_model(args.endpoint)
    compile_probe()
    if not AGENT_BIN.exists():
        raise RuntimeError(f"build Reference Agent first: {AGENT_BIN}")

    output_dir = args.output_dir.resolve()
    if output_dir.exists() and any(output_dir.iterdir()):
        raise RuntimeError(f"refusing to overwrite non-empty result directory: {output_dir}")
    (output_dir / "raw").mkdir(parents=True, exist_ok=True)
    (output_dir / "stimuli").mkdir(parents=True, exist_ok=True)

    for case in fixture["cases"]:
        generate_wav(case["request"], output_dir / "stimuli" / f"{case['id']}.wav")
    run_change_exercises(output_dir)

    raw_path = output_dir / "raw/trials.jsonl"
    warmup_path = output_dir / "raw/warmups.jsonl"
    manifest = {
        "campaign_id": output_dir.name,
        "status": "RUNNING",
        "started_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "source_revision": git_revision(),
        "contract_sha256": sha256_file(CONTRACT),
        "foundation_sha256": sha256_file(FOUNDATION),
        "fixture_sha256": sha256_file(FIXTURE),
        "oracle_sha256": sha256_file(ORACLE),
        "change_exercises_sha256": sha256_file(CHANGE_EXERCISES),
        "candidates": candidates,
        "correctness_breadth_cases": [item["id"] for item in breadth_cases],
        "latency_cases": latency_ids,
        "latency_warmups_per_case": warmups,
        "latency_scored_trials_per_case": latency_trials,
        "evidence_label": freeze["evidence_label"],
        "environment": {
            "platform": platform.platform(),
            "python": platform.python_version(),
            "machine": platform.machine(),
            "semantic_endpoint": args.endpoint,
        },
    }
    write_json(output_dir / "manifest.json", manifest)

    total_breadth = len(candidates) * len(breadth_cases)
    index = 0
    for candidate in candidates:
        for case in breadth_cases:
            index += 1
            print(f"[breadth {index}/{total_breadth}] {candidate} {case['id']}", flush=True)
            record = run_record(
                endpoint=args.endpoint,
                candidate=candidate,
                case=case,
                trial=1,
                stratum="correctness_breadth",
                warmup=False,
                fixture=fixture,
                expected_by_case=expected_by_case,
                foundation=foundation,
                output_dir=output_dir,
            )
            jsonl_append(raw_path, record)

    total_warmups = len(candidates) * len(latency_cases) * warmups
    index = 0
    for candidate in candidates:
        for case in latency_cases:
            for trial in range(1, warmups + 1):
                index += 1
                print(
                    f"[warmup {index}/{total_warmups}] {candidate} {case['id']} {trial}",
                    flush=True,
                )
                record = run_record(
                    endpoint=args.endpoint,
                    candidate=candidate,
                    case=case,
                    trial=trial,
                    stratum="latency_repetition",
                    warmup=True,
                    fixture=fixture,
                    expected_by_case=expected_by_case,
                    foundation=foundation,
                    output_dir=output_dir,
                )
                jsonl_append(warmup_path, record)

    total_latency = len(candidates) * len(latency_cases) * latency_trials
    index = 0
    for trial in range(1, latency_trials + 1):
        for case in latency_cases:
            for candidate in candidates:
                index += 1
                print(
                    f"[latency {index}/{total_latency}] {candidate} {case['id']} {trial}",
                    flush=True,
                )
                record = run_record(
                    endpoint=args.endpoint,
                    candidate=candidate,
                    case=case,
                    trial=trial,
                    stratum="latency_repetition",
                    warmup=False,
                    fixture=fixture,
                    expected_by_case=expected_by_case,
                    foundation=foundation,
                    output_dir=output_dir,
                )
                jsonl_append(raw_path, record)

    manifest["status"] = "RAW_COMPLETE_PENDING_ANALYSIS"
    manifest["completed_at"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    manifest["raw_sha256"] = sha256_file(raw_path)
    manifest["warmup_sha256"] = sha256_file(warmup_path)
    write_json(output_dir / "manifest.json", manifest)
    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
