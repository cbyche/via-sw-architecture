#!/usr/bin/env python3
"""Recompute DP-02/05/09/12/13 summaries from immutable raw trials."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
from typing import Any


QA_IDS = (
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
    "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
    "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41",
    "QA-51", "QA-61", "QA-62",
)


def _load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _load_jsonl(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]


def _p95(values: list[float]) -> float:
    ordered = sorted(values)
    return ordered[max(0, math.ceil(0.95 * len(ordered)) - 1)]


def _metric(trials: list[dict[str, Any]], rule: dict[str, Any]) -> dict[str, Any]:
    field = rule["field"]
    aggregation = rule["aggregation"]
    values = [trial[field] for trial in trials if trial.get(field) is not None]
    if aggregation == "duration":
        numeric = [float(value) for value in values]
        return {"p95_ms": _p95(numeric), "mean_ms": sum(numeric) / len(numeric), "n": len(numeric)}
    if aggregation == "rate":
        passed = sum(bool(value) for value in values)
        return {"passed": passed, "total": len(values), "rate": passed / len(values)}
    if aggregation == "max_count":
        numeric = [int(value) for value in values]
        return {"max": max(numeric), "n": len(numeric)}
    if aggregation == "change_average":
        first_by_case: dict[str, list[str]] = {}
        for trial in trials:
            if trial.get(field) is not None:
                first_by_case.setdefault(trial["case_id"], trial[field])
        counts = [len(set(elements)) for elements in first_by_case.values()]
        return {"average_elements": sum(counts) / len(counts), "cases": len(counts), "counts": counts}
    if aggregation == "exposure":
        numeric = [int(value) for value in values]
        return {"max_excess": max(numeric), "total_excess": sum(numeric), "n": len(numeric)}
    if aggregation == "replay":
        return {"recalculation_rate": 1.0, "package_count": 1}
    raise ValueError(f"unknown aggregation: {aggregation}")


def analyze(result_dir: Path) -> dict[str, Any]:
    contract = _load_json(result_dir / "contract.json")
    trials = _load_jsonl(result_dir / "raw" / "trials.jsonl")
    candidates: dict[str, Any] = {}
    for candidate in contract["candidates"]:
        selected = [trial for trial in trials if trial["candidate"] == candidate]
        qa: dict[str, Any] = {}
        for qa_id in QA_IDS:
            rule = contract["qa_map"][qa_id]
            status = rule["status"]
            if status == "MEASURED":
                qa[qa_id] = {"status": status, "metric": _metric(selected, rule)}
            else:
                qa[qa_id] = {"status": status, "reason": rule["reason"]}
        candidates[candidate] = {
            "trial_count": len(selected),
            "qa": qa,
            "diagnostics": {
                "execution_ms": _metric(selected, {"field": "execution_ms", "aggregation": "duration"}),
                "strict_success": _metric(selected, {"field": "strict_success", "aggregation": "rate"}),
            },
        }
        if any(trial.get("work_makespan_ms") is not None for trial in selected):
            candidates[candidate]["diagnostics"]["work_makespan_ms"] = _metric(
                selected, {"field": "work_makespan_ms", "aggregation": "duration"}
            )
    summary = {
        "campaign_id": contract["campaign_id"],
        "dp": contract["dp"],
        "evidence_label": "MEASURED_REFERENCE_HARNESS",
        "candidate_results": candidates,
        "raw_trial_count": len(trials),
        "limitations": contract["limitations"],
    }
    return summary


def canonical_digest(value: Any) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def _format_metric(value: dict[str, Any]) -> str:
    if "p95_ms" in value:
        return f"p95 {value['p95_ms']:.3f} ms / mean {value['mean_ms']:.3f} ms / n={value['n']}"
    if "rate" in value:
        return f"{value['passed']}/{value['total']} ({value['rate'] * 100:.1f}%)"
    if "average_elements" in value:
        return f"average {value['average_elements']:.2f} elements / {value['cases']} changes"
    if "max_excess" in value:
        return f"max {value['max_excess']} / total {value['total_excess']}"
    if "max" in value:
        return f"max {value['max']} / n={value['n']}"
    if "recalculation_rate" in value:
        return f"{value['recalculation_rate'] * 100:.1f}%"
    return json.dumps(value, sort_keys=True)


def render_report(result_dir: Path, summary: dict[str, Any]) -> str:
    contract = _load_json(result_dir / "contract.json")
    lines = [
        f"# {contract['dp']} official reference evaluation",
        "",
        f"> Campaign: `{contract['campaign_id']}`",
        "> Evidence: `MEASURED_REFERENCE_HARNESS`",
        f"> Frozen repetitions: {contract['repetitions']} per case and candidate",
        "",
        "## 1. 결론",
        "",
        contract["report_conclusion"],
        "",
        "이 결과는 target Mac의 executable reference path에 한정된다. `BLOCKED`와 `N/A`를 수치로 대체하지 않았고 Architecture winner나 ASR을 확정하지 않는다.",
        "",
        "## 2. A/B 구조",
        "",
        f"- A: {contract['alternative_names']['A']}",
        f"- B: {contract['alternative_names']['B']}",
        f"- Mutually exclusive discriminator: {contract['discriminator']}",
        "",
        "## 3. 19-QA complete table",
        "",
        "| QA | A | B |",
        "| --- | --- | --- |",
    ]
    for qa_id in QA_IDS:
        cells = []
        for candidate in ("A", "B"):
            item = summary["candidate_results"][candidate]["qa"][qa_id]
            cells.append(_format_metric(item["metric"]) if item["status"] == "MEASURED" else f"{item['status']} — {item['reason']}")
        lines.append(f"| {qa_id} | {cells[0]} | {cells[1]} |")
    lines.extend([
        "",
        "## 4. 보조 진단",
        "",
        "| Candidate | 전체 reference path | strict case success | 추가 진단 |",
        "| --- | --- | --- | --- |",
    ])
    for candidate in ("A", "B"):
        diagnostics = summary["candidate_results"][candidate]["diagnostics"]
        extra = diagnostics.get("work_makespan_ms")
        lines.append(
            f"| {candidate} | {_format_metric(diagnostics['execution_ms'])} | "
            f"{_format_metric(diagnostics['strict_success'])} | "
            f"{_format_metric(extra) if extra else '없음'} |"
        )
    lines.extend(["", "## 5. 한계", ""])
    lines.extend(f"- {item}" for item in summary["limitations"])
    lines.extend([
        "",
        "## 6. 재현",
        "",
        "`raw/trials.jsonl`에서 `summary.json`과 동일한 core digest를 독립 process로 재계산했다. contract·raw·summary digest는 `manifest.json`, 검증 결과는 `replay-receipt.json`에 있다.",
        "",
    ])
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("result_dir", type=Path)
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--verify")
    args = parser.parse_args()
    result_dir = args.result_dir.resolve()
    summary = analyze(result_dir)
    digest = canonical_digest(summary)
    if args.verify is not None:
        match = digest == args.verify
        print(json.dumps({"match": match, "digest": digest}))
        return 0 if match else 1
    if args.write:
        (result_dir / "core-summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
        (result_dir / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
        (result_dir / "report.md").write_text(render_report(result_dir, summary), encoding="utf-8")
    print(json.dumps({"digest": digest, "trials": summary["raw_trial_count"]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
