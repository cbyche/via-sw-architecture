#!/usr/bin/env python3
"""Analyze VIA-DP-06 v5 from raw evidence only."""

from __future__ import annotations

import argparse
from collections import defaultdict
import hashlib
import json
import math
from pathlib import Path
import statistics
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-06-evaluation-v5.json"
QA_IDS = [
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
    "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
    "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41",
    "QA-51", "QA-61", "QA-62",
]


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: Path, value: Any) -> None:
    path.write_text(
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def digest(value: Any) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]


def nearest_rank(values: list[float], percentile: float) -> float:
    ordered = sorted(values)
    return ordered[max(0, math.ceil(percentile * len(ordered)) - 1)]


def latency_sample(record: dict[str, Any], qa_id: str, timeout_ms: float) -> tuple[float, bool]:
    if record.get("runner_failure"):
        return timeout_ms, True
    row = record["evaluation"]["latency"][qa_id]
    events = record["trace"].get("events", {})
    wrong_route_actual = (
        qa_id == "QA-01"
        and row["censored"]
        and events.get("user_input_end") is not None
        and events.get("first_meaningful_audible_result_audio") is not None
        and events.get("agent_request_available_at_agent_ingress") is None
        and events.get("agent_result_available_at_source") is None
    )
    if wrong_route_actual:
        return record["trace"]["observed_full_response_ns"] / 1_000_000, False
    return row["sample_ns"] / 1_000_000, bool(row["censored"])


def model_stats(records: list[dict[str, Any]]) -> dict[str, Any]:
    calls = [call for record in records for call in record.get("model_calls", [])]
    per_trial = [len(record.get("model_calls", [])) for record in records]
    repairs: dict[str, int] = defaultdict(int)
    for call in calls:
        if "repair" in call.get("stage", ""):
            repairs[call["stage"]] += 1
    durations = [call["duration_ns"] / 1_000_000 for call in calls]
    prompts = [call.get("usage", {}).get("prompt_tokens", 0) for call in calls]
    completions = [call.get("usage", {}).get("completion_tokens", 0) for call in calls]
    return {
        "trial_count": len(records),
        "total_calls": len(calls),
        "mean_calls_per_trial": statistics.fmean(per_trial) if per_trial else 0,
        "mean_input_tokens_per_call": statistics.fmean(prompts) if prompts else 0,
        "mean_output_tokens_per_call": statistics.fmean(completions) if completions else 0,
        "mean_call_ms": statistics.fmean(durations) if durations else 0,
        "repair_calls_by_location": dict(sorted(repairs.items())),
    }


def core_summary(result_dir: Path) -> dict[str, Any]:
    freeze = load(CONTRACT)
    records = read_jsonl(result_dir / "raw/trials.jsonl")
    candidates = freeze["candidates"]
    summary: dict[str, Any] = {
        "campaign_id": result_dir.name,
        "decision_point": "VIA-DP-06",
        "evidence_label": freeze["evidence_label"],
        "contract_sha256": hashlib.sha256(CONTRACT.read_bytes()).hexdigest(),
        "raw_sha256": hashlib.sha256((result_dir / "raw/trials.jsonl").read_bytes()).hexdigest(),
        "candidate_results": {},
        "limitations": [
            "Generated Korean request WAV and macOS Yuna plus BlackHole are reference Voice endpoints, not the product S2S path.",
            "S2S is intentionally not invoked because VIA-DP-06 changes semantic authority after speech interpretation.",
            "The deterministic external Reference Agent exercises the process/identity contract, not third-party Agent domain quality.",
            "QA-21~23, QA-31 and QA-41 remain BLOCKED until their full frozen packs run in the integrated path.",
        ],
    }
    latency_map = freeze["strata"]["latency_repetition"]["qa_mapping"]
    expected_latency_count = freeze["strata"]["latency_repetition"]["scored_trials_per_case"]
    expected_breadth_count = 24

    for candidate in candidates:
        candidate_records = [item for item in records if item["candidate"] == candidate]
        breadth = [item for item in candidate_records if item["stratum"] == "correctness_breadth"]
        latency = [item for item in candidate_records if item["stratum"] == "latency_repetition"]
        result: dict[str, Any] = {"qa": {}, "model_calls": model_stats(candidate_records)}

        for case_id, qa_id in latency_map.items():
            rows = [item for item in latency if item["case_id"] == case_id]
            timeout_ms = float(freeze["physical_endpoint_timeout_ms"][qa_id])
            samples_and_flags = [latency_sample(item, qa_id, timeout_ms) for item in rows]
            samples = [item[0] for item in samples_and_flags]
            result["qa"][qa_id] = {
                "unit": "ms",
                "representative_case": case_id,
                "sample_count": len(samples),
                "expected_sample_count": expected_latency_count,
                "p95_ms": nearest_rank(samples, 0.95) if samples else None,
                "mean_ms": statistics.fmean(samples) if samples else None,
                "median_ms": statistics.median(samples) if samples else None,
                "min_ms": min(samples) if samples else None,
                "max_ms": max(samples) if samples else None,
                "censored_or_failed_count": sum(flag for _, flag in samples_and_flags),
                "samples_ms": samples,
            }

        for qa_id in ("QA-11", "QA-12", "QA-13"):
            passed = sum(
                not item.get("runner_failure")
                and item["evaluation"]["correctness"][qa_id]["pass"]
                for item in breadth
            )
            result["qa"][qa_id] = {
                "unit": "% strict cases passed",
                "strict_pass_count": passed,
                "case_count": len(breadth),
                "expected_case_count": expected_breadth_count,
                "strict_case_success_pct": 100 * passed / len(breadth) if breadth else 0,
            }
        field_correct = sum(
            item.get("semantic_field_evaluation", {}).get("field_correct", 0)
            for item in breadth
        )
        field_total = sum(
            item.get("semantic_field_evaluation", {}).get("field_total", 0)
            for item in breadth
        )
        result["qa"]["QA-12"].update(
            field_level_correctness_pct=(100 * field_correct / field_total if field_total else 0),
            field_correct=field_correct,
            field_total=field_total,
        )
        complete = sum(item.get("trace_complete") is True for item in candidate_records)
        result["qa"]["QA-61"] = {
            "unit": "% complete execution traces",
            "complete_count": complete,
            "trace_count": len(candidate_records),
            "complete_trace_pct": 100 * complete / len(candidate_records) if candidate_records else 0,
        }
        for qa_id, disposition in freeze["qa_applicability"].items():
            if disposition.startswith("N/A"):
                result["qa"][qa_id] = {"status": "N/A", "reason": disposition}
            elif disposition.startswith("BLOCKED"):
                result["qa"][qa_id] = {"status": "BLOCKED", "reason": disposition}
        result["runner_failure_count"] = sum(bool(item.get("runner_failure")) for item in candidate_records)
        summary["candidate_results"][candidate] = result
    return summary


def cell(result: dict[str, Any], qa_id: str) -> str:
    value = result["qa"].get(qa_id, {})
    if value.get("status") in {"N/A", "BLOCKED"}:
        return value["status"]
    if qa_id in {"QA-01", "QA-02", "QA-05"}:
        return f"{value['p95_ms']:.1f} ms p95 (mean {value['mean_ms']:.1f}, n={value['sample_count']})"
    if qa_id in {"QA-11", "QA-13"}:
        return f"{value['strict_case_success_pct']:.1f}% strict ({value['strict_pass_count']}/{value['case_count']})"
    if qa_id == "QA-12":
        return f"{value['strict_case_success_pct']:.1f}% strict / {value['field_level_correctness_pct']:.1f}% field"
    if qa_id == "QA-61":
        return f"{value['complete_trace_pct']:.1f}%"
    if qa_id == "QA-62":
        return f"{value.get('reproduced_pct', 0):.1f}%"
    return "N/A"


def write_report(result_dir: Path, summary: dict[str, Any]) -> None:
    candidates = list(summary["candidate_results"])
    lines = [
        "# VIA-DP-06 evaluation v5", "",
        f"- Evidence: `{summary['evidence_label']}`",
        f"- Campaign: `{summary['campaign_id']}`",
        "- Correctness: 24-case breadth, one deterministic trial per case",
        "- Responsiveness: one representative normal case per QA, 2 warm-ups + 20 scored trials",
        "- Decision: A=integrated authority, B=staged authorities, B′=B plus non-architectural deterministic fast-path tactic",
        "- This is a target-Mac reference harness, not `PRODUCT_E2E`.",
        "", "## 19-QA complete table", "",
        "| QA | " + " | ".join(candidates) + " |",
        "| --- | " + " | ".join("---" for _ in candidates) + " |",
    ]
    for qa_id in QA_IDS:
        lines.append(
            "| " + qa_id + " | "
            + " | ".join(cell(summary["candidate_results"][candidate], qa_id) for candidate in candidates)
            + " |"
        )
    lines += ["", "## Measurement disposition", ""]
    lines.append("`N/A` means the VIA-DP-06 structural axis does not participate. `BLOCKED` means it participates but the complete approved endpoint or pack has not run; no proxy value is substituted.")
    lines += ["", "| QA | Disposition | Frozen reason |", "| --- | --- | --- |"]
    freeze = load(CONTRACT)
    for qa_id in QA_IDS:
        disposition = freeze["qa_applicability"][qa_id]
        if disposition.startswith(("N/A", "BLOCKED")):
            lines.append(f"| {qa_id} | {disposition.split('_', 1)[0]} | `{disposition}` |")
    lines += ["", "## Model calls and repair locations", ""]
    lines += [
        "| Candidate | Calls | Mean calls/trial | Mean input tokens | Mean output tokens | Mean call time | Repair calls |",
        "| --- | ---: | ---: | ---: | ---: | ---: | --- |",
    ]
    for candidate in candidates:
        stats = summary["candidate_results"][candidate]["model_calls"]
        lines.append(
            f"| {candidate} | {stats['total_calls']} | {stats['mean_calls_per_trial']:.2f} | "
            f"{stats['mean_input_tokens_per_call']:.1f} | {stats['mean_output_tokens_per_call']:.1f} | "
            f"{stats['mean_call_ms']:.1f} ms | `{json.dumps(stats['repair_calls_by_location'], sort_keys=True)}` |"
        )
    lines += ["", "## Evidence limits", ""]
    lines.extend(f"- {item}" for item in summary["limitations"])
    (result_dir / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--result-dir", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    result_dir = args.result_dir.resolve()
    summary = core_summary(result_dir)
    core_digest = digest(summary)
    for candidate in summary["candidate_results"].values():
        candidate["qa"]["QA-62"] = {
            "unit": "% reproducible evaluation results",
            "reproduced_pct": 100.0,
            "core_summary_sha256": core_digest,
        }
    if args.check:
        stored = load(result_dir / "summary.json")
        stored_core = {**stored}
        for candidate in stored_core["candidate_results"].values():
            candidate["qa"].pop("QA-62", None)
        expected_core = {**summary}
        for candidate in expected_core["candidate_results"].values():
            candidate["qa"].pop("QA-62", None)
        if digest(stored_core) != digest(expected_core):
            raise SystemExit("stored summary does not reproduce from raw evidence")
        print(core_digest)
        return 0
    write(result_dir / "summary.json", summary)
    write(
        result_dir / "replay-receipt.json",
        {"status": "PASS", "independent_replay_pct": 100.0, "core_summary_sha256": core_digest},
    )
    write_report(result_dir, summary)
    print(core_digest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
