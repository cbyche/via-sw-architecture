#!/usr/bin/env python3
"""Recompute the VIA-DP-11 pilot summary from preserved raw evidence only."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from run_dp11_pilot import percentile_summary, validate_trace_completeness


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    with path.open(encoding="utf-8") as stream:
        return [json.loads(line) for line in stream if line.strip()]


def compute(evidence: Path) -> dict[str, Any]:
    contract_path = evidence / "contract.json"
    contract_bytes = contract_path.read_bytes()
    contract = json.loads(contract_bytes)
    contract_hash = hashlib.sha256(contract_bytes).hexdigest()
    manifest = json.loads((evidence / "manifest.json").read_text(encoding="utf-8"))
    if manifest["contract_sha256"] != contract_hash:
        raise ValueError("contract hash does not match manifest")

    normal_records = read_jsonl(evidence / "raw/normal-bridge.jsonl")
    recovery_records = read_jsonl(evidence / "raw/recovery.jsonl")
    blast_records = read_jsonl(evidence / "raw/blast-radius.jsonl")
    all_records = normal_records + recovery_records + blast_records
    for record in all_records:
        if record["contract_sha256"] != contract_hash:
            raise ValueError(
                f"raw record contract mismatch: {record.get('source_execution_key')}"
            )

    required_fields = contract["observability"]["required_raw_fields"]
    complete, total, completeness_pct = validate_trace_completeness(
        all_records, required_fields
    )

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
    strata_ids = contract["workloads"]["integration_client_fatal_recovery"][
        "active_task_strata"
    ]
    for candidate in ("same_process", "isolated_worker"):
        strata = {}
        for active_tasks in strata_ids:
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

    return {
        "contract_id": contract["contract_id"],
        "contract_sha256": contract_hash,
        "evidence_label": contract["evidence_label"],
        "normal_bridge_diagnostic": normal_summary,
        "qa31_integration_client_fatal_pilot": recovery_summary,
        "qa32_fault_blast_radius": blast_summary,
        "qa61_complete_execution_trace_run_rate_pct": completeness_pct,
        "trace_records": {"complete": complete, "total": total},
        "limitations": contract["limitations"],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    computed = compute(args.evidence)
    if args.check:
        reported = json.loads(
            (args.evidence / "derived/summary.json").read_text(encoding="utf-8")
        )
        if computed != reported:
            raise SystemExit("recomputed summary differs from reported summary")
        print("PASS: raw evidence exactly reproduces derived/summary.json")
    elif args.output:
        args.output.write_text(
            json.dumps(computed, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
    else:
        print(json.dumps(computed, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
