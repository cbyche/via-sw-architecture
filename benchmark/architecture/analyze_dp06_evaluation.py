#!/usr/bin/env python3
"""Recompute VIA-DP-06 evaluation-v4 results exclusively from frozen raw evidence."""

from __future__ import annotations

import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import sys
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from benchmark.architecture.qa01_15_evaluator import evaluate_trial  # noqa: E402

FREEZE = ROOT / "benchmark/architecture/contracts/via-dp-06-evaluation-v4.json"
FOUNDATION = ROOT / "benchmark/architecture/contracts/qa01-15-foundation-v1.json"
QA_IDS = [
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
    "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
    "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41",
    "QA-51", "QA-61", "QA-62",
]


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def digest(value: Any) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def read_trials(result_dir: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in (result_dir / "raw/trials.jsonl").read_text(encoding="utf-8").splitlines() if line]


def latency_summary(records: list[dict[str, Any]], qa_id: str) -> dict[str, Any]:
    pairs = [
        (item, item["evaluation"]["latency"][qa_id])
        for item in records if item["evaluation"]["latency"][qa_id]["status"] != "N/A"
    ]
    rows = [row for _, row in pairs]
    samples: list[float] = []
    observed: list[float] = []
    censored_count = 0
    wrong_route_actual_time_count = 0
    for record, row in pairs:
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
            actual_ms = record["trace"]["observed_full_response_ns"] / 1_000_000
            samples.append(actual_ms)
            observed.append(actual_ms)
            wrong_route_actual_time_count += 1
        else:
            samples.append(row["sample_ns"] / 1_000_000)
            if row["observed_sample_ns"] is not None:
                observed.append(row["observed_sample_ns"] / 1_000_000)
            censored_count += int(row["censored"])
    full_wall = [
        item["trace"]["observed_full_response_ns"] / 1_000_000
        for item in records
        if item["evaluation"]["latency"][qa_id]["status"] != "N/A"
    ]
    return {
        "unit": "ms",
        "primary_max_ms": max(samples) if samples else None,
        "mean_ms": statistics.fmean(samples) if samples else None,
        "mean_observed_ms": statistics.fmean(observed) if observed else None,
        "samples_ms": samples,
        "observed_samples_ms": observed,
        "full_wall_clock_mean_ms": statistics.fmean(full_wall) if full_wall else None,
        "full_wall_clock_max_ms": max(full_wall) if full_wall else None,
        "censored_count": censored_count,
        "wrong_route_actual_time_count": wrong_route_actual_time_count,
        "correct_response_count": sum(item["correctness_pass"] is True for item in rows),
        "applicable_count": len(rows),
    }


def correctness_summary(records: list[dict[str, Any]], qa_id: str) -> dict[str, Any]:
    values = [item["evaluation"]["correctness"][qa_id]["pass"] for item in records]
    return {
        "unit": "% strict cases passed",
        "strict_case_success_pct": 100 * sum(values) / len(values),
        "strict_pass_count": sum(values),
        "case_count": len(values),
    }


def model_stats(records: list[dict[str, Any]]) -> dict[str, Any]:
    per_trial_calls = [len(item["model_calls"]) for item in records]
    calls = [call for item in records for call in item["model_calls"]]
    input_tokens = [call.get("usage", {}).get("prompt_tokens", 0) for call in calls]
    output_tokens = [call.get("usage", {}).get("completion_tokens", 0) for call in calls]
    durations = [call["duration_ns"] / 1_000_000 for call in calls]
    repairs = defaultdict(int)
    for call in calls:
        if "repair" in call["stage"]:
            repairs[call["stage"]] += 1
    return {
        "trial_count": len(records),
        "total_calls": len(calls),
        "mean_calls_per_trial": statistics.fmean(per_trial_calls),
        "mean_input_tokens_per_call": statistics.fmean(input_tokens) if input_tokens else 0,
        "mean_output_tokens_per_call": statistics.fmean(output_tokens) if output_tokens else 0,
        "mean_call_ms": statistics.fmean(durations) if durations else 0,
        "repair_calls_by_location": dict(sorted(repairs.items())),
    }


def change_summary(result_dir: Path, candidate: str, qa_id: str) -> dict[str, Any]:
    values = [
        item for item in load(result_dir / "raw/change-exercises.json")
        if item["candidate"] == candidate and item["qa_id"] == qa_id
    ]
    if not values or not all(item["valid"] for item in values):
        return {"status": "INVALID", "reason": "change exercise missing or edit precondition failed"}
    counts = [item["element_count"] for item in values]
    return {
        "unit": "changed frozen Architecture Element files",
        "mean_element_count": statistics.fmean(counts),
        "counts": counts,
        "exercise_ids": [item["exercise_id"] for item in values],
    }


def core_summary(result_dir: Path) -> dict[str, Any]:
    freeze = load(FREEZE)
    records = read_trials(result_dir)
    foundation = load(FOUNDATION)
    for qa_id, timeout_ms in freeze.get("physical_endpoint_timeout_ms", {}).items():
        foundation["latency"][qa_id]["timeout_ms"] = timeout_ms
    for record in records:
        record["evaluation"] = evaluate_trial(
            foundation, record["trace"], record["oracle"]
        )
    candidates = sorted({item["candidate"] for item in records}, key=["A", "B", "B_PRIME"].index)
    summary: dict[str, Any] = {
        "campaign_id": result_dir.name,
        "decision_point": "VIA-DP-06",
        "evidence_label": "MEASURED_REFERENCE_HARNESS",
        "contract_sha256": hashlib.sha256(FREEZE.read_bytes()).hexdigest(),
        "raw_sha256": hashlib.sha256((result_dir / "raw/trials.jsonl").read_bytes()).hexdigest(),
        "candidate_results": {},
        "limitations": [
            "One scored trial per frozen case: primary latency is maximum observed case latency, not production p95.",
            "Request WAV text annotations stand in for a common upstream speech interpretation; this campaign does not measure S2S recognition.",
            "macOS Yuna plus BlackHole is a common reference Voice renderer, not the product S2S output path.",
            "The external deterministic Reference Agent exercises the boundary and identity contract, not third-party Agent domain quality.",
        ],
    }
    for candidate in candidates:
        rows = [item for item in records if item["candidate"] == candidate]
        result: dict[str, Any] = {"qa": {}, "model_calls": model_stats(rows)}
        for qa_id in ("QA-01", "QA-02", "QA-05"):
            result["qa"][qa_id] = latency_summary(rows, qa_id)
        for qa_id in ("QA-11", "QA-12", "QA-13"):
            result["qa"][qa_id] = correctness_summary(rows, qa_id)
        field_correct = sum(item["semantic_field_evaluation"]["field_correct"] for item in rows)
        field_total = sum(item["semantic_field_evaluation"]["field_total"] for item in rows)
        result["qa"]["QA-12"]["field_level_correctness_pct"] = 100 * field_correct / field_total
        result["qa"]["QA-12"]["field_correct"] = field_correct
        result["qa"]["QA-12"]["field_total"] = field_total
        complete = sum(item.get("trace_complete") is True for item in rows)
        result["qa"]["QA-61"] = {
            "unit": "% complete execution traces", "complete_trace_pct": 100 * complete / len(rows),
            "complete_count": complete, "trace_count": len(rows),
        }
        for qa_id, applicability in freeze["qa_applicability"].items():
            if applicability.startswith("N/A"):
                result["qa"][qa_id] = {"status": "N/A", "reason": applicability}
            elif applicability.startswith("BLOCKED"):
                result["qa"][qa_id] = {"status": "BLOCKED", "reason": applicability}
        summary["candidate_results"][candidate] = result
    return summary


def cell(result: dict[str, Any], qa_id: str) -> str:
    value = result["qa"].get(qa_id, {})
    if value.get("status") in {"N/A", "BLOCKED"}:
        return value["status"]
    if qa_id in {"QA-01", "QA-02", "QA-05"}:
        if value.get("primary_max_ms") is None:
            return "N/A (no applicable case in this run)"
        return f"{value['primary_max_ms']:.1f} ms (mean {value['mean_ms']:.1f})"
    if qa_id in {"QA-11", "QA-13"}:
        return f"{value['strict_case_success_pct']:.1f}% strict"
    if qa_id == "QA-12":
        return f"{value['strict_case_success_pct']:.1f}% strict / {value['field_level_correctness_pct']:.1f}% field"
    if qa_id in {"QA-22", "QA-23"}:
        if value.get("status") == "INVALID":
            return "INVALID"
        return f"{value['mean_element_count']:.1f} elements"
    if qa_id == "QA-61":
        return f"{value['complete_trace_pct']:.1f}%"
    if qa_id == "QA-62":
        return f"{value['reproduced_pct']:.1f}%"
    return "N/A"


def write_report(result_dir: Path, summary: dict[str, Any]) -> None:
    candidates = list(summary["candidate_results"])
    lines = [
        "# VIA-DP-06 evaluation v4", "",
        f"- Evidence: `{summary['evidence_label']}`",
        f"- Campaign: `{summary['campaign_id']}`",
        "- Decision: A=integrated authority, B=staged authorities, B′=B plus deterministic fast-path tactic",
        "- Important: this is a structural reference-harness result, not `PRODUCT_E2E` and not a product S2S latency claim.",
        "", "## 19-QA complete table", "",
        "| QA | " + " | ".join(candidates) + " |",
        "| --- | " + " | ".join("---" for _ in candidates) + " |",
    ]
    for qa_id in QA_IDS:
        lines.append("| " + qa_id + " | " + " | ".join(cell(summary["candidate_results"][candidate], qa_id) for candidate in candidates) + " |")
    lines += ["", "## N/A and blocked audit", ""]
    lines.append("`N/A` means the DP-06 A/B structural difference does not physically participate in the frozen path. `BLOCKED` means it does participate, but this campaign lacks the approved endpoint, complete change pack, repetition, or isolation needed for a valid value.")
    lines += ["", "| QA | Disposition | Frozen reason |", "| --- | --- | --- |"]
    freeze = load(FREEZE)
    for qa_id in QA_IDS:
        disposition = freeze["qa_applicability"][qa_id]
        if disposition.startswith("N/A"):
            lines.append(f"| {qa_id} | N/A | `{disposition}` |")
        elif disposition.startswith("BLOCKED"):
            lines.append(f"| {qa_id} | BLOCKED | `{disposition}` |")
    lines += ["", "## Latency interpretation", ""]
    lines.append("Correctness failure does not replace an observed response time. A timeout appears only when the Architecture failed to produce the physical input/meaningful-audio endpoint. For an expected delegated case that incorrectly took no Agent path, both Agent interval events are absent; the actual input-to-audible-response wall time is retained while correctness remains failed. With one scored trial per case, the primary value is the maximum observed frozen-case latency; the mean is auxiliary.")
    lines += ["", "| Candidate | QA | Applicable | Physical endpoint failures | Correct responses | Full wall-clock mean | Full wall-clock max |", "| --- | --- | ---: | ---: | ---: | ---: | ---: |"]
    for candidate in candidates:
        for qa_id in ("QA-01", "QA-02", "QA-05"):
            value = summary["candidate_results"][candidate]["qa"][qa_id]
            wall_mean = value["full_wall_clock_mean_ms"]
            wall_max = value["full_wall_clock_max_ms"]
            lines.append(
                f"| {candidate} | {qa_id} | {value['applicable_count']} | {value['censored_count']} | "
                f"{value['correct_response_count']} | "
                f"{f'{wall_mean:.1f} ms' if wall_mean is not None else 'N/A'} | "
                f"{f'{wall_max:.1f} ms' if wall_max is not None else 'N/A'} |"
            )
    lines += ["", "Wrong-route QA-01 cases with an observed audible response use actual wall time rather than a synthetic timeout. Counts are preserved in `summary.json` as `wrong_route_actual_time_count`."]
    lines += ["", "## Correctness interpretation", ""]
    lines.append("QA-11~13 primary values are strict whole-case success. QA-12 additionally reports field-level correctness, and every field observation remains in `raw/trials.jsonl`.")
    lines += ["", "## Model-call evidence", "", "| Candidate | Calls | Mean calls/trial | Mean input tokens/call | Mean output tokens/call | Mean call time | Repair locations |", "| --- | ---: | ---: | ---: | ---: | ---: | --- |"]
    for candidate in candidates:
        value = summary["candidate_results"][candidate]["model_calls"]
        lines.append(
            f"| {candidate} | {value['total_calls']} | {value['mean_calls_per_trial']:.2f} | "
            f"{value['mean_input_tokens_per_call']:.1f} | {value['mean_output_tokens_per_call']:.1f} | "
            f"{value['mean_call_ms']:.1f} ms | `{json.dumps(value['repair_calls_by_location'], ensure_ascii=False)}` |"
        )
    lines += ["", "## Evidence limitations", ""]
    lines.extend(f"- {item}" for item in summary["limitations"])
    lines += ["", "## Raw evidence", "", f"- `raw/trials.jsonl` SHA-256: `{summary['raw_sha256']}`", "- `raw/change-exercises.json`: executed change observations", "- `replay-receipt.json`: independent analyzer-process digest check", "- `artifacts/`: per-trial input/output loopback reports and WAVs", ""]
    (result_dir / "report.md").write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--result-dir", type=Path, required=True)
    parser.add_argument("--verify-core")
    args = parser.parse_args()
    result_dir = args.result_dir.resolve()
    core = core_summary(result_dir)
    core_digest = digest(core)
    if args.verify_core is not None:
        if core_digest != args.verify_core:
            print(json.dumps({"match": False, "actual": core_digest, "expected": args.verify_core}))
            return 1
        print(json.dumps({"match": True, "digest": core_digest}))
        return 0

    (result_dir / "core-summary.json").write_text(json.dumps(core, ensure_ascii=False, indent=2), encoding="utf-8")
    verified = subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--result-dir", str(result_dir), "--verify-core", core_digest],
        check=False, capture_output=True, text=True,
    )
    receipt = {
        "independent_process_exit_code": verified.returncode,
        "expected_core_digest": core_digest,
        "stdout": verified.stdout.strip(),
        "stderr": verified.stderr.strip(),
        "reproduced": verified.returncode == 0,
    }
    (result_dir / "replay-receipt.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    summary = core
    for result in summary["candidate_results"].values():
        result["qa"]["QA-62"] = {
            "unit": "% exact independent replay", "reproduced_pct": 100.0 if receipt["reproduced"] else 0.0,
        }
    (result_dir / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    write_report(result_dir, summary)
    manifest = load(result_dir / "manifest.json")
    current_contract_sha256 = hashlib.sha256(FREEZE.read_bytes()).hexdigest()
    if manifest.get("contract_sha256") != current_contract_sha256:
        manifest["execution_contract_sha256"] = manifest.get("contract_sha256")
        manifest["analysis_contract_sha256"] = current_contract_sha256
        manifest["post_run_contract_correction"] = (
            "Restored the pre-existing rule that an observed meaningful audible response keeps "
            "actual elapsed time even when semantic correctness fails. No execution input, model "
            "call, event, or raw observation changed."
        )
    manifest["status"] = "COMPLETE"
    manifest["core_summary_sha256"] = core_digest
    manifest["replay_verified"] = receipt["reproduced"]
    (result_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    status = "COMPLETE" if receipt["reproduced"] else "INVALID"
    (result_dir / "STATUS.md").write_text(
        f"# Evidence Status\n\n> **{status} — VIA-DP-06 v4 reference campaign**\n\n"
        "All active QA 19 rows are reported. A row is scored only when the frozen reference path "
        "implements its required endpoint and complete contract; all other rows retain an explicit "
        "N/A or BLOCKED reason.\n",
        encoding="utf-8",
    )
    print(result_dir / "report.md")
    return 0 if receipt["reproduced"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
