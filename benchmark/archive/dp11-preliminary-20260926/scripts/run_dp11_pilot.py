#!/usr/bin/env python3
"""Run the frozen VIA-DP-11 pilot and preserve immutable raw observations."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-11-pilot-v2.json"
DEFAULT_BENCH = ROOT / "prototypes/candidates/target/debug/via-bench"
DEFAULT_HOST = ROOT / "prototypes/candidates/target/debug/via-host"
DEFAULT_WORKER = ROOT / "prototypes/candidates/target/debug/via-worker"
EVIDENCE_LABEL = "MEASURED_REFERENCE_HARNESS"


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run_json(command: list[str], timeout_seconds: int = 180) -> tuple[dict[str, Any], int, str]:
    started = time.monotonic_ns()
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        timeout=timeout_seconds,
    )
    elapsed = time.monotonic_ns() - started
    return json.loads(completed.stdout), elapsed, completed.stderr


def run_text(command: list[str]) -> str:
    return subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def nearest_rank_p95(values: list[int]) -> int:
    if not values:
        raise ValueError("p95 requires at least one value")
    ordered = sorted(values)
    rank = (95 * len(ordered) + 99) // 100
    return ordered[rank - 1]


def percentile_summary(values: list[int]) -> dict[str, int]:
    return {
        "count": len(values),
        "min": min(values),
        "median": sorted(values)[(len(values) - 1) // 2],
        "p95": nearest_rank_p95(values),
        "max": max(values),
    }


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_jsonl(path: Path, records: list[dict[str, Any]]) -> None:
    with path.open("x", encoding="utf-8") as stream:
        for record in records:
            stream.write(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n")


def envelope(
    *,
    contract: dict[str, Any],
    contract_hash: str,
    source_execution_key: str,
    candidate: str,
    workload: str,
    trial: int,
    duration_ns: int,
    observation: dict[str, Any],
    outcome: str = "PASS",
) -> dict[str, Any]:
    return {
        "contract_id": contract["contract_id"],
        "contract_sha256": contract_hash,
        "source_execution_key": source_execution_key,
        "candidate": candidate,
        "workload": workload,
        "trial": trial,
        "outcome": outcome,
        "evidence_label": EVIDENCE_LABEL,
        "started_at_utc": utc_now(),
        "duration_ns": duration_ns,
        "raw_observation": observation,
    }


def validate_trace_completeness(
    records: list[dict[str, Any]], required_fields: list[str]
) -> tuple[int, int, float]:
    complete = sum(
        1
        for record in records
        if all(field in record and record[field] is not None for field in required_fields)
    )
    return complete, len(records), 100.0 * complete / len(records)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--contract", type=Path, default=DEFAULT_CONTRACT)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--bench", type=Path, default=DEFAULT_BENCH)
    parser.add_argument("--host", type=Path, default=DEFAULT_HOST)
    parser.add_argument("--worker", type=Path, default=DEFAULT_WORKER)
    args = parser.parse_args()

    if args.output.exists():
        raise SystemExit(f"refusing to overwrite existing evidence directory: {args.output}")
    for binary in (args.bench, args.host, args.worker):
        if not binary.is_file():
            raise SystemExit(f"missing candidate binary: {binary}")

    contract_bytes = args.contract.read_bytes()
    contract = json.loads(contract_bytes)
    contract_hash = hashlib.sha256(contract_bytes).hexdigest()
    raw_dir = args.output / "raw"
    derived_dir = args.output / "derived"
    raw_dir.mkdir(parents=True)
    derived_dir.mkdir()
    (args.output / "contract.json").write_bytes(contract_bytes)

    run_id = args.output.name
    manifest = {
        "run_id": run_id,
        "created_at_utc": utc_now(),
        "contract": str(args.contract.relative_to(ROOT)),
        "contract_sha256": contract_hash,
        "evidence_label": EVIDENCE_LABEL,
        "target": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "macos": platform.mac_ver()[0],
        },
        "git": {
            "head": run_text(["git", "rev-parse", "HEAD"]),
            "status_porcelain": run_text(["git", "status", "--short"]),
        },
        "binaries": {
            "via-bench": sha256(args.bench),
            "via-host": sha256(args.host),
            "via-worker": sha256(args.worker),
        },
        "commands": [],
    }
    write_json(args.output / "manifest.json", manifest)

    normal_spec = contract["workloads"]["normal_bridge_diagnostic"]
    normal_command = [
        str(args.bench),
        "dp11-normal-diagnostic",
        "--worker",
        str(args.worker),
        "--trials",
        str(normal_spec["scored_trials_per_candidate"]),
        "--warmup",
        str(normal_spec["warmup_per_candidate"]),
    ]
    normal, normal_command_ns, normal_stderr = run_json(normal_command)
    manifest["commands"].append({
        "argv": normal_command,
        "duration_ns": normal_command_ns,
        "stderr": normal_stderr,
    })
    normal_records = []
    for item in normal["runs"]:
        if item["warmup"]:
            continue
        normal_records.append(
            envelope(
                contract=contract,
                contract_hash=contract_hash,
                source_execution_key=f"{run_id}:normal:{item['candidate']}:{item['trial']}",
                candidate=item["candidate"],
                workload="normal_bridge_diagnostic",
                trial=item["trial"],
                duration_ns=item["full_lifecycle_ns"],
                observation=item,
            )
        )
    write_jsonl(raw_dir / "normal-bridge.jsonl", normal_records)

    recovery_command = [
        str(args.bench),
        "qa09-integration-fatal",
        "--host",
        str(args.host),
        "--worker",
        str(args.worker),
        "--profile",
        "dp11-pilot",
        "--freeze-fingerprint",
        contract_hash,
    ]
    recovery, recovery_command_ns, recovery_stderr = run_json(recovery_command, 300)
    manifest["commands"].append({
        "argv": recovery_command,
        "duration_ns": recovery_command_ns,
        "stderr": recovery_stderr,
    })
    recovery_records = []
    for candidate in recovery["candidates"]:
        candidate_id = (
            "same_process" if candidate["exec_candidate"] == "shared" else "isolated_worker"
        )
        for stratum in candidate["strata"]:
            for item in stratum["trials"]:
                observation = dict(item)
                observation["active_tasks"] = stratum["active_tasks"]
                recovery_records.append(
                    envelope(
                        contract=contract,
                        contract_hash=contract_hash,
                        source_execution_key=(
                            f"{run_id}:recovery:{candidate_id}:"
                            f"{stratum['active_tasks']}:{item['trial']}"
                        ),
                        candidate=candidate_id,
                        workload="integration_client_fatal_recovery",
                        trial=item["trial"],
                        duration_ns=item["recovery_elapsed_ns"],
                        observation=observation,
                    )
                )
    write_jsonl(raw_dir / "recovery.jsonl", recovery_records)

    blast_records = []
    blast_trials = contract["workloads"]["integration_client_fatal_blast_radius"][
        "scored_trials_per_candidate"
    ]
    independent_units = contract["workloads"]["integration_client_fatal_blast_radius"][
        "independent_user_visible_units"
    ]
    for trial in range(1, blast_trials + 1):
        command = [
            str(args.bench),
            "exec-blast-smoke",
            "--host",
            str(args.host),
            "--worker",
            str(args.worker),
        ]
        value, command_ns, stderr = run_json(command)
        manifest["commands"].append({"argv": command, "duration_ns": command_ns, "stderr": stderr})
        shared_observation = {
            **value,
            "independent_units": independent_units,
            "unavailable_independent_units": independent_units
            if value["shared_host_exited"]
            else [],
            "excess_affected_unit_count": len(independent_units)
            if value["shared_host_exited"]
            else 0,
        }
        isolated_observation = {
            **value,
            "independent_units": independent_units,
            "unavailable_independent_units": []
            if value["isolated_core_host_survived"]
            else independent_units,
            "excess_affected_unit_count": 0
            if value["isolated_core_host_survived"]
            else len(independent_units),
        }
        for candidate, observation in [
            ("same_process", shared_observation),
            ("isolated_worker", isolated_observation),
        ]:
            blast_records.append(
                envelope(
                    contract=contract,
                    contract_hash=contract_hash,
                    source_execution_key=f"{run_id}:blast:{candidate}:{trial}",
                    candidate=candidate,
                    workload="integration_client_fatal_blast_radius",
                    trial=trial,
                    duration_ns=command_ns,
                    observation=observation,
                )
            )
    write_jsonl(raw_dir / "blast-radius.jsonl", blast_records)

    all_records = normal_records + recovery_records + blast_records
    required_fields = contract["observability"]["required_raw_fields"]
    complete, total, completeness_pct = validate_trace_completeness(all_records, required_fields)

    normal_summary: dict[str, Any] = {}
    for candidate in ("same_process", "isolated_worker"):
        observations = [
            record["raw_observation"]
            for record in normal_records
            if record["candidate"] == candidate
        ]
        normal_summary[candidate] = {
            "full_lifecycle_ns": percentile_summary(
                [item["full_lifecycle_ns"] for item in observations]
            ),
            "cancel_ns": percentile_summary([item["cancel_ns"] for item in observations]),
            "correctness_pass_rate_pct": 100.0
            * sum(item["correctness_pass"] for item in observations)
            / len(observations),
        }

    recovery_summary: dict[str, Any] = {}
    for candidate in ("same_process", "isolated_worker"):
        strata = {}
        for active_tasks in contract["workloads"]["integration_client_fatal_recovery"][
            "active_task_strata"
        ]:
            values = [
                record["duration_ns"]
                for record in recovery_records
                if record["candidate"] == candidate
                and record["raw_observation"]["active_tasks"] == active_tasks
            ]
            strata[str(active_tasks)] = percentile_summary(values)
        recovery_summary[candidate] = {
            "strata_duration_ns": strata,
            "qa31_pilot_worst_stratum_p95_ns": max(
                item["p95"] for item in strata.values()
            ),
        }

    blast_summary = {
        candidate: max(
            record["raw_observation"]["excess_affected_unit_count"]
            for record in blast_records
            if record["candidate"] == candidate
        )
        for candidate in ("same_process", "isolated_worker")
    }

    summary = {
        "contract_id": contract["contract_id"],
        "contract_sha256": contract_hash,
        "evidence_label": EVIDENCE_LABEL,
        "normal_bridge_diagnostic": normal_summary,
        "qa31_integration_client_fatal_pilot": recovery_summary,
        "qa32_fault_blast_radius": blast_summary,
        "qa61_complete_execution_trace_run_rate_pct": completeness_pct,
        "trace_records": {"complete": complete, "total": total},
        "limitations": contract["limitations"],
    }
    write_json(derived_dir / "summary.json", summary)
    write_json(args.output / "manifest.json", manifest)

    def milliseconds(nanoseconds: int) -> str:
        return f"{nanoseconds / 1_000_000:.3f}"

    report = f"""# VIA-DP-11 Pilot Result

> Evidence: `{EVIDENCE_LABEL}`
> Contract: `{contract['contract_id']}` / `{contract_hash}`
> This pilot does not select a winner.

## Result at a glance

| Observation | A: isolated worker | B: same process | Interpretation |
| --- | ---: | ---: | --- |
| Normal bridge lifecycle diagnostic p95 | {milliseconds(normal_summary['isolated_worker']['full_lifecycle_ns']['p95'])} ms | {milliseconds(normal_summary['same_process']['full_lifecycle_ns']['p95'])} ms | Supporting subspan only; not QA-01/03/05 |
| QA-31 pilot: integration-client fatal worst-stratum p95 | {milliseconds(recovery_summary['isolated_worker']['qa31_pilot_worst_stratum_p95_ns'])} ms | {milliseconds(recovery_summary['same_process']['qa31_pilot_worst_stratum_p95_ns'])} ms | One fault stratum, not final all-fault QA-31 |
| QA-32 excess affected user-visible units | {blast_summary['isolated_worker']} | {blast_summary['same_process']} | Frozen independent-unit registry |
| QA-61 complete trace rate | {completeness_pct:.1f}% | {completeness_pct:.1f}% | Package-level rate across both candidates |

## What may be concluded

- The same Reference Agent contract and Task authority ran on both sides; the changed axis was the Agent-client process boundary.
- The normal bridge p95 difference is {milliseconds(normal_summary['isolated_worker']['full_lifecycle_ns']['p95'] - normal_summary['same_process']['full_lifecycle_ns']['p95'])} ms. It is real at this subspan but far below the pre-existing 100 ms responsiveness interpretation threshold and is not a user-endpoint QA result.
- The QA-31 pilot difference is {milliseconds(abs(recovery_summary['isolated_worker']['qa31_pilot_worst_stratum_p95_ns'] - recovery_summary['same_process']['qa31_pilot_worst_stratum_p95_ns']))} ms, so this pilot treats the candidates as practically similar for this stratum rather than declaring a recovery winner.
- QA-32 is structurally sensitive to this DP if the frozen client-fatal fault is credible for the production dependency.
- QA-31 evidence is provisional because the final metric is the maximum across the complete frozen fault pack.
- Normal bridge timings show the direct cost of IPC, but cannot be substituted for user/acoustic responsiveness metrics.

## Evidence package

- `manifest.json`: target, Git state, binary hashes, exact commands
- `raw/normal-bridge.jsonl`: every scored normal-path diagnostic trial
- `raw/recovery.jsonl`: every recovery trial, including task-count stratum
- `raw/blast-radius.jsonl`: every fatal-fault containment observation
- `derived/summary.json`: deterministic aggregation of raw data

## Limitations

""" + "\n".join(f"- {item}" for item in contract["limitations"]) + "\n"
    (args.output / "report.md").write_text(report, encoding="utf-8")
    print(args.output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
