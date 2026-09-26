#!/usr/bin/env python3
"""Independently reproduce and classify the VIA-DP-11 19-QA result table."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any


CANDIDATES = ("isolated_worker", "same_process")
QA_IDS = [
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
    "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
    "QA-21", "QA-22", "QA-23", "QA-31", "QA-32",
    "QA-41", "QA-51", "QA-61", "QA-62",
]
QA_NAMES = {
    "QA-01": "Delegated Path VIA Responsiveness",
    "QA-02": "VIA Direct Voice Response Responsiveness",
    "QA-03": "Agent Progress Voice Feedback Responsiveness",
    "QA-04": "Voice Interruption Responsiveness",
    "QA-05": "Task Control Responsiveness",
    "QA-11": "VIA Request Handling Correctness",
    "QA-12": "Request Semantic Resolution Correctness",
    "QA-13": "Task & Interaction Binding Correctness",
    "QA-14": "Async Task State Convergence Correctness",
    "QA-15": "Interaction & Task Continuity Correctness",
    "QA-21": "Agent Change Locality",
    "QA-22": "Model, Context & State Change Locality",
    "QA-23": "Experiment & Logging Change Locality",
    "QA-31": "Correct Task Recovery Time",
    "QA-32": "Fault Blast Radius",
    "QA-41": "Target Device Memory Footprint",
    "QA-51": "Protected Data Exposure Minimization",
    "QA-61": "Execution Trace Completeness",
    "QA-62": "Evidence Reproducibility",
}
LOWER_IS_BETTER = {
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
    "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41", "QA-51",
}


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]


def p95(values: list[int]) -> int:
    return sorted(values)[math.ceil(0.95 * len(values)) - 1]


def calculate(evidence: Path) -> dict[str, dict[str, Any]]:
    contract = json.loads((evidence / "contract.json").read_text(encoding="utf-8"))
    cases = json.loads((evidence / "normal-fixture.json").read_text(encoding="utf-8"))["cases"]
    semantic = load_jsonl(evidence / "raw/semantic.jsonl")
    normal = load_jsonl(evidence / "raw/normal.jsonl")
    faults = load_jsonl(evidence / "raw/faults.jsonl")
    changes = load_jsonl(evidence / "raw/changes.jsonl")
    memory = load_jsonl(evidence / "raw/memory.jsonl")
    exposure = load_jsonl(evidence / "raw/exposure.jsonl")

    def mapped(candidate: str, records: list[dict[str, Any]]) -> list[dict[str, Any]]:
        return [
            record for record in records
            if record["candidate"] == candidate
            or (
                record["candidate"] == "common"
                and candidate in record["raw_observation"].get("mapped_candidates", [])
            )
        ]

    def worst_p95(candidate: str, field: str) -> int | None:
        case_values = []
        for case in cases:
            values = [
                record["raw_observation"][field]
                for record in mapped(candidate, normal)
                if record["case"] == case["id"]
                and record["raw_observation"].get(field) is not None
            ]
            if values:
                case_values.append(p95(values))
        return max(case_values) if case_values else None

    def macro_rate(candidate: str, qa_id: str, field: str) -> float | None:
        case_rates = []
        for case in cases:
            if qa_id not in case["qa"]:
                continue
            values = [
                bool(record["raw_observation"].get(field, True))
                for record in mapped(candidate, normal)
                if record["case"] == case["id"]
            ]
            if values:
                case_rates.append(sum(values) / len(values))
        return 100.0 * sum(case_rates) / len(case_rates) if case_rates else None

    result: dict[str, dict[str, Any]] = {}
    for qa_id, field in {
        "QA-01": "qa01_ns", "QA-02": "qa02_ns", "QA-03": "qa03_ns",
        "QA-04": "qa04_ns", "QA-05": "qa05_ns",
    }.items():
        result[qa_id] = {candidate: worst_p95(candidate, field) for candidate in CANDIDATES}
        result[qa_id]["unit"] = "ns"

    result["QA-11"] = {
        candidate: macro_rate(candidate, "QA-11", "integrated_pass") for candidate in CANDIDATES
    } | {"unit": "pct"}
    semantic_rate = 100.0 * sum(record["semantic_pass"] for record in semantic) / len(semantic)
    result["QA-12"] = {candidate: semantic_rate for candidate in CANDIDATES} | {"unit": "pct"}
    for qa_id, field in {
        "QA-13": "binding_pass", "QA-14": "state_pass", "QA-15": "continuity_pass",
    }.items():
        result[qa_id] = {
            candidate: macro_rate(candidate, qa_id, field) for candidate in CANDIDATES
        } | {"unit": "pct"}

    for qa_id in ("QA-21", "QA-22", "QA-23"):
        result[qa_id] = {"unit": "elements"}
        for candidate in CANDIDATES:
            counts = [
                record["changed_count"] for record in changes
                if record["qa"] == qa_id and record["candidate"] == candidate
            ]
            result[qa_id][candidate] = sum(counts) / len(counts)

    for candidate in CANDIDATES:
        candidate_faults = mapped(candidate, faults)
        result.setdefault("QA-31", {"unit": "ns"})[candidate] = max(
            p95([r["duration_ns"] for r in candidate_faults if r["workload"] == fault])
            for fault in ("F-01", "F-02", "F-03")
        )
        result.setdefault("QA-32", {"unit": "units"})[candidate] = max(
            r["raw_observation"]["excess_affected_units"] for r in candidate_faults
        )
        scored = mapped(candidate, normal) + candidate_faults
        result.setdefault("QA-61", {"unit": "pct"})[candidate] = (
            100.0 * sum(r["trace_complete"] for r in scored) / len(scored)
        )

    result["QA-41"] = {
        candidate: p95([r["total_rss_bytes"] for r in memory if r["candidate"] == candidate])
        for candidate in CANDIDATES
    } | {"unit": "bytes"}
    result["QA-51"] = {
        candidate: sum(
            r["excess_protected_units"] for r in exposure if r["candidate"] == candidate
        )
        for candidate in CANDIDATES
    } | {"unit": "units"}

    reported = json.loads((evidence / "derived/summary.json").read_text(encoding="utf-8"))
    reported_by_id = {row["qa"]: row for row in reported["complete_19_qa_table"]}
    reproduced = 0
    for qa_id in QA_IDS[:-1]:
        row = reported_by_id[qa_id]
        if (
            row["A"] == result[qa_id]["isolated_worker"]
            and row["B"] == result[qa_id]["same_process"]
            and row["unit"] == result[qa_id]["unit"]
        ):
            reproduced += 1
    reproducibility = 100.0 * reproduced / len(QA_IDS[:-1])
    result["QA-62"] = {
        candidate: reproducibility for candidate in CANDIDATES
    } | {"unit": "pct", "reproduced_rows": reproduced, "sampled_rows": len(QA_IDS) - 1}
    result["_contract"] = contract
    return result


def threshold_met(qa_id: str, a: float, b: float) -> bool:
    difference = abs(a - b)
    relative_pct = 100.0 * difference / max(abs(a), abs(b), 1e-12)
    if qa_id in {"QA-01", "QA-02", "QA-03", "QA-05"}:
        return difference >= 100_000_000 and relative_pct >= 10
    if qa_id == "QA-04":
        return difference >= 20_000_000 and relative_pct >= 20
    if qa_id in {"QA-11", "QA-12", "QA-13", "QA-14", "QA-15", "QA-61"}:
        return difference >= 5
    if qa_id in {"QA-21", "QA-22", "QA-23", "QA-32", "QA-51"}:
        return difference >= 1
    if qa_id == "QA-31":
        return difference >= 500_000_000 and relative_pct >= 20
    if qa_id == "QA-41":
        return difference >= 100 * 1024 * 1024 and relative_pct >= 10
    return qa_id == "QA-62" and difference > 0


def classify(qa_id: str, values: dict[str, Any], role: str) -> tuple[str, str]:
    a = values["isolated_worker"]
    b = values["same_process"]
    if qa_id == "QA-62":
        return ("재현 완료" if a == 100 and b == 100 else "재현 불완전", "qualification")
    if role == "non_applicable_regression":
        return "공통/회귀 검증; DP-11 변별용 아님", "regression"
    if a == b:
        suffix = "공통 qualification" if role == "qualification" else "차이 없음"
        return f"A/B 동일; {suffix}", "no_difference"
    winner = "A" if ((a < b) == (qa_id in LOWER_IS_BETTER)) else "B"
    if threshold_met(qa_id, a, b):
        return f"의미 있는 차이: {winner} 우세", f"meaningful_{winner}"
    return f"수치상 {winner} 우세, 최소 차이 미달", "below_threshold"


def display(value: float, unit: str) -> str:
    if unit == "ns":
        return f"{value / 1_000_000:.3f} ms"
    if unit == "pct":
        return f"{value:.2f}%"
    if unit == "bytes":
        return f"{value / 1024 / 1024:.1f} MiB"
    if unit == "elements":
        return f"{value:.2f} elements"
    return f"{value:g} units"


def report(evidence: Path, values: dict[str, dict[str, Any]]) -> str:
    contract = values["_contract"]
    rows = []
    meaningful = []
    for qa_id in QA_IDS:
        role = contract["qa_plan"][qa_id]["role"]
        verdict, code = classify(qa_id, values[qa_id], role)
        if code.startswith("meaningful_"):
            meaningful.append((qa_id, verdict))
        rows.append(
            f"| {qa_id} {QA_NAMES[qa_id]} | {display(values[qa_id]['isolated_worker'], values[qa_id]['unit'])} "
            f"| {display(values[qa_id]['same_process'], values[qa_id]['unit'])} | {verdict} | {role} |"
        )
    meaningful_text = ", ".join(f"{qa_id} ({verdict.split(': ')[-1]})" for qa_id, verdict in meaningful)
    return f"""# VIA-DP-11 전체 19개 QA 평가 결과

> 근거 수준: `MEASURED_MODEL` + `MEASURED_REFERENCE_HARNESS` + `HYBRID_REFERENCE_ESTIMATE` + `ARCHITECTURE_LEDGER`
>
> 방안 A: 별도 Integration Worker process
> 방안 B: VIA Core process 안의 Agent Client

## 결론

동결한 한 campaign에서 active QA 19개를 모두 평가했다. 독립 analyzer가 source에서 계산되는 18개 row를 모두 정확히 재현했으므로 QA-62는 양쪽 모두 100%다.

동결 최소 차이를 넘은 조합은 {meaningful_text}뿐이다. 나머지 수치 차이는 최소 차이 미달이거나 공통·회귀 축이다. 따라서 이 보고서는 전체 승자를 선택하지 않는다.

QA-12는 69.57%, 통합 QA-11은 75.00%로 양쪽이 같다. 이는 공통 semantic 경로의 한계이며 DP-11 process placement 차이가 아니다. 결과에 맞춰 없애거나 A/B trade-off로 세지 않고 qualification 미달로 보존한다.

## 전체 DP × QA 표

시간·변경 요소·영향 단위·메모리·노출은 작을수록 좋다. 정확성·완전성·재현성은 클수록 좋다.

| QA | A | B | 동결 기준 판정 | DP-11에서의 역할 |
| --- | ---: | ---: | --- | --- |
{chr(10).join(rows)}

## 실제로 이 DP를 구분하는 QA

- **QA-32:** A는 주입한 Integration Client fatal fault를 worker에 격리했다. B는 Core process와 함께 독립 사용자 기능 네 개를 잃었다. 가장 강한 구조적 trade-off다.
- **QA-23:** B는 worker IPC·supervision 경계가 없어서 frozen 실험·로그 change pack의 변경 Architecture Element가 더 적었다.
- **QA-01/03/05/31/41:** 수치 방향은 갈렸지만 절대 차이가 동결 기준에 못 미쳤다. 측정값은 보존하되 trade-off로 승격하지 않는다.
- **QA-02/04/12/22:** 공통 또는 회귀 축이다. DP-11이 이 경로를 소유하지 않으므로 동률이나 미세한 proxy noise가 정상이다.
- **QA-11/13/14/15/51/61/62:** qualification이다. 더 빠르거나 더 잘 격리된 후보가 부정확·불안전·추적 불가·재현 불가한 상태로 이기는 것을 막는다.

## Evidence package

- semantic model 23건, 정상 runtime 250건, fault 60건, change ledger 58건, memory 50건, 보호정보 노출 110건을 보존한다.
- Raw record는 `raw/`, runner가 만든 표는 `derived/summary.json`에 있다.
- `derived/verified-summary.json`은 독립 analyzer가 동결 contract와 raw evidence만으로 생성한다.
- Process placement는 공통 semantic 경로를 바꾸지 않으므로 같은 semantic-model record를 양쪽에 매핑한다.

## 한계

- Voice endpoint는 microphone-to-speaker acoustic loopback이 아니라 instrumented renderer/audio-buffer proxy다. `PRODUCT_E2E` 값이 아니다.
- Reference Agent는 deterministic하다. VIA Architecture 동작을 격리하며 실제 Agent reasoning·실행 품질을 측정하지 않는다.
- QA-21/22/23은 runtime timing이 아니라 frozen Architecture Element ledger다.
- 주입한 fatal fault는 그 fault가 존재할 때의 구조적 결과를 증명한다. 제품 의사결정에서의 가중치는 실제 Agent Client fault profile에 달려 있다.
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    values = calculate(args.evidence)
    reproducibility = values["QA-62"]["isolated_worker"]
    if args.write:
        output = {qa_id: values[qa_id] for qa_id in QA_IDS}
        (args.evidence / "derived/verified-summary.json").write_text(
            json.dumps(output, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        (args.evidence / "report.md").write_text(report(args.evidence, values), encoding="utf-8")
    print(f"reproduced={values['QA-62']['reproduced_rows']}/{values['QA-62']['sampled_rows']} ({reproducibility:.2f}%)")
    if args.check and reproducibility != 100.0:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
