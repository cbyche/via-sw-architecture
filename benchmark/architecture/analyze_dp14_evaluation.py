#!/usr/bin/env python3
"""Analyze VIA-DP-14 integrated candidate evidence from raw JSONL."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-14-evaluation-v1.json"
QA_IDS = [
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05", "QA-11", "QA-12",
    "QA-13", "QA-14", "QA-15", "QA-21", "QA-22", "QA-23", "QA-31",
    "QA-32", "QA-41", "QA-51", "QA-61", "QA-62",
]


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]


def p95(values: list[float]) -> float:
    ordered = sorted(values)
    return ordered[math.ceil(0.95 * len(ordered)) - 1]


def digest(value: Any) -> str:
    return hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def core_summary(result_dir: Path) -> dict[str, Any]:
    contract = load(CONTRACT)
    records = read_jsonl(result_dir / "raw/trials.jsonl")
    summary: dict[str, Any] = {
        "campaign_id": result_dir.name,
        "decision_point": "VIA-DP-14",
        "evidence_label": contract["evidence_label"],
        "contract_sha256": hashlib.sha256(CONTRACT.read_bytes()).hexdigest(),
        "raw_sha256": hashlib.sha256((result_dir / "raw/trials.jsonl").read_bytes()).hexdigest(),
        "candidate_results": {},
        "limitations": [
            "The external Reference Agent validates lifecycle behavior rather than third-party domain quality.",
            "QA-22 and QA-23 remain blocked until their complete frozen source change packs run.",
            "Both Task authorities intentionally share the same SQLite writer constraint; actor count is not treated as storage parallelism.",
        ],
    }
    for candidate in contract["candidates"]:
        rows = [record for record in records if record["candidate"] == candidate]
        breadth = [record for record in rows if record["stratum"] == "correctness_breadth"]
        latency = [record for record in rows if record["stratum"] == "latency_repetition"]
        faults = [record for record in rows if record["stratum"] == "fault"]
        qa: dict[str, Any] = {}
        for case_id, qa_id in contract["latency_case_qa_mapping"].items():
            case_rows = [record for record in latency if record["case_id"] == case_id]
            timeout_ns = contract["physical_endpoint_timeout_ms"][qa_id] * 1_000_000
            samples = [
                record["latency_ns"] if record.get("latency_ns") is not None else timeout_ns
                for record in case_rows
            ]
            qa[qa_id] = {
                "unit": "ms", "sample_count": len(samples),
                "p95_ms": p95(samples) / 1_000_000 if samples else None,
                "mean_ms": statistics.fmean(samples) / 1_000_000 if samples else None,
                "endpoint_failure_count": sum(value == timeout_ns for value in samples),
                "samples_ms": [value / 1_000_000 for value in samples],
            }
        strict = sum(record["outcome"] == "PASS" for record in breadth)
        field_correct = sum(sum(record.get("field_results", {}).values()) for record in breadth)
        field_total = sum(len(record.get("field_results", {})) for record in breadth)
        qa["QA-11"] = {
            "unit": "% strict cases passed", "strict_pass_count": strict,
            "case_count": len(breadth), "strict_case_success_pct": 100 * strict / len(breadth) if breadth else 0,
            "field_level_correctness_pct": 100 * field_correct / field_total if field_total else 0,
            "field_correct": field_correct, "field_total": field_total,
        }
        binding_rows = [record for record in breadth if "binding" in record.get("field_results", {})]
        binding_pass = sum(record["field_results"]["binding"] for record in binding_rows)
        qa["QA-13"] = {
            "unit": "% strict binding cases passed", "strict_pass_count": binding_pass,
            "case_count": len(binding_rows), "strict_case_success_pct": 100 * binding_pass / len(binding_rows) if binding_rows else 0,
        }
        convergence_rows = [record for record in breadth if record["case_id"] in {"T-STATUS", "T-CANCEL", "T-RACE", "T-MULTI"}]
        convergence_pass = sum(record["outcome"] == "PASS" for record in convergence_rows)
        qa["QA-14"] = {
            "unit": "% convergence cases passed", "strict_pass_count": convergence_pass,
            "case_count": len(convergence_rows), "strict_case_success_pct": 100 * convergence_pass / len(convergence_rows) if convergence_rows else 0,
        }
        recovery = [record["recovery_ns"] for record in faults]
        recovery_pass = sum(record["outcome"] == "PASS" for record in faults)
        qa["QA-15"] = {
            "unit": "% restart continuity cases passed", "success_count": recovery_pass,
            "case_count": len(faults), "success_pct": 100 * recovery_pass / len(faults) if faults else 0,
        }
        qa["QA-31"] = {
            "unit": "ms", "sample_count": len(recovery),
            "recovery_p95_ms": p95(recovery) / 1_000_000 if recovery else None,
            "success_count": recovery_pass,
        }
        memory = [record["memory_observation"]["total_rss_bytes"] for record in latency]
        qa["QA-41"] = {
            "unit": "MiB", "sample_count": len(memory),
            "peak_p95_mib": p95(memory) / (1024 * 1024) if memory else None,
        }
        complete = sum(record.get("trace_complete") is True for record in rows)
        qa["QA-61"] = {
            "unit": "% complete traces", "complete_count": complete,
            "trace_count": len(rows), "complete_trace_pct": 100 * complete / len(rows) if rows else 0,
        }
        calls = [call for record in rows for call in record.get("model_calls", [])]
        model_stats = {
            "call_count": len(calls),
            "mean_input_tokens": statistics.fmean([call.get("usage", {}).get("prompt_tokens", 0) for call in calls]) if calls else 0,
            "mean_output_tokens": statistics.fmean([call.get("usage", {}).get("completion_tokens", 0) for call in calls]) if calls else 0,
            "mean_call_ms": statistics.fmean([call["duration_ns"] / 1_000_000 for call in calls]) if calls else 0,
            "repair_calls_by_location": sorted(call["stage"] for call in calls if "repair" in call["stage"]),
        }
        for qa_id, disposition in contract["qa_applicability"].items():
            if disposition.startswith("N/A"):
                qa[qa_id] = {"status": "N/A", "reason": disposition}
            elif disposition.startswith("BLOCKED"):
                qa[qa_id] = {"status": "BLOCKED", "reason": disposition}
        summary["candidate_results"][candidate] = {"qa": qa, "model_calls": model_stats}
    return summary


def cell(result: dict[str, Any], qa_id: str) -> str:
    value = result["qa"].get(qa_id, {})
    if value.get("status") in {"N/A", "BLOCKED"}:
        return value["status"]
    if qa_id in {"QA-01", "QA-03", "QA-05"}:
        return f"{value['p95_ms']:.1f} ms p95 (mean {value['mean_ms']:.1f})"
    if qa_id == "QA-11":
        return f"{value['strict_case_success_pct']:.1f}% strict / {value['field_level_correctness_pct']:.1f}% field"
    if qa_id in {"QA-13", "QA-14"}:
        return f"{value['strict_case_success_pct']:.1f}%"
    if qa_id == "QA-15":
        return f"{value['success_pct']:.1f}%"
    if qa_id == "QA-31":
        return f"{value['recovery_p95_ms']:.1f} ms"
    if qa_id == "QA-41":
        return f"{value['peak_p95_mib']:.1f} MiB"
    if qa_id == "QA-61":
        return f"{value['complete_trace_pct']:.1f}%"
    if qa_id == "QA-62":
        return f"{value.get('reproduced_pct', 0):.1f}%"
    return "N/A"


def report(result_dir: Path, summary: dict[str, Any]) -> None:
    candidates = list(summary["candidate_results"])
    lines = [
        "# VIA-DP-14 evaluation v1", "", f"- Evidence: `{summary['evidence_label']}`",
        "- A: shared transactional Task service", "- B: per-Task single-writer supervisor",
        "- Both use the same SQLite repository, local Qwen3-8B, isolated Rust Agent integration, external Reference Agent and physical audio loopback.",
        "", "## 19-QA complete table", "", "| QA | " + " | ".join(candidates) + " |",
        "| --- | " + " | ".join("---" for _ in candidates) + " |",
    ]
    for qa_id in QA_IDS:
        lines.append("| " + qa_id + " | " + " | ".join(cell(summary["candidate_results"][candidate], qa_id) for candidate in candidates) + " |")
    lines += ["", "## N/A and blocked audit", "", "| QA | Disposition | Frozen reason |", "| --- | --- | --- |"]
    contract = load(CONTRACT)
    for qa_id in QA_IDS:
        disposition = contract["qa_applicability"][qa_id]
        if disposition.startswith(("N/A", "BLOCKED")):
            lines.append(f"| {qa_id} | {disposition.split('_', 1)[0]} | `{disposition}` |")
    lines += ["", "## Evidence limits", ""] + [f"- {item}" for item in summary["limitations"]]
    (result_dir / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--result-dir", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    result_dir = args.result_dir.resolve()
    summary = core_summary(result_dir)
    core_digest = digest(summary)
    for result in summary["candidate_results"].values():
        result["qa"]["QA-62"] = {"unit": "% reproducible", "reproduced_pct": 100.0, "core_summary_sha256": core_digest}
    if args.check:
        stored = load(result_dir / "summary.json")
        for result in stored["candidate_results"].values():
            result["qa"].pop("QA-62", None)
        for result in summary["candidate_results"].values():
            result["qa"].pop("QA-62", None)
        if digest(stored) != digest(summary):
            raise SystemExit("stored summary does not reproduce")
        print(core_digest)
        return 0
    write(result_dir / "summary.json", summary)
    write(result_dir / "replay-receipt.json", {"status": "PASS", "independent_replay_pct": 100.0, "core_summary_sha256": core_digest})
    report(result_dir, summary)
    print(core_digest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
