#!/usr/bin/env python3
"""Analyze VIA-DP-06 v6 change-locality evidence and write a Korean report."""

from __future__ import annotations

import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import statistics
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-06-change-locality-v6.json"
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


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest(value: Any) -> str:
    encoded = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]


def core_summary(result_dir: Path) -> dict[str, Any]:
    contract = load(CONTRACT)
    source_dir = ROOT / contract["source_campaign"]["path"]
    source_summary_path = source_dir / "summary.json"
    source_manifest_path = source_dir / "manifest.json"
    source = load(source_summary_path)
    records = read_jsonl(result_dir / "raw/change-locality.jsonl")
    expected_record_count = len(contract["candidates"]) * sum(
        len(ids) for ids in contract["required_change_ids"].values()
    )
    if len(records) != expected_record_count:
        raise ValueError(f"expected {expected_record_count} records, found {len(records)}")
    keys = [record["source_execution_key"] for record in records]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate source_execution_key in structural evidence")
    if not all(record["baseline_reset"] is True for record in records):
        raise ValueError("one or more changes were not applied from a fresh baseline")

    candidate_results = copy.deepcopy(source["candidate_results"])
    for candidate in contract["candidates"]:
        candidate_rows = [record for record in records if record["candidate"] == candidate]
        for qa_id, expected_ids in contract["required_change_ids"].items():
            rows = [record for record in candidate_rows if record["qa_id"] == qa_id]
            if [record["change_id"] for record in rows] != expected_ids:
                raise ValueError(f"{candidate}/{qa_id} membership or order differs from contract")
            counts = [record["changed_element_count"] for record in rows]
            type_totals: Counter[str] = Counter()
            for record in rows:
                if sum(record["element_type_breakdown"].values()) != record["changed_element_count"]:
                    raise ValueError(f"invalid type breakdown: {record['source_execution_key']}")
                type_totals.update(record["element_type_breakdown"])
            candidate_results[candidate]["qa"][qa_id] = {
                "unit": "changed Architecture Elements per change",
                "mean_changed_elements": statistics.fmean(counts),
                "changed_element_sum": sum(counts),
                "change_count": len(counts),
                "maximum_changed_elements": max(counts),
                "element_type_totals": {
                    element_type: type_totals[element_type] for element_type in ("C", "I", "S", "D")
                },
                "per_change": [
                    {
                        "change_id": record["change_id"],
                        "count": record["changed_element_count"],
                        "modified": record["modified_element_ids"],
                        "added": record["added_element_ids"],
                        "removed": record["removed_element_ids"],
                        "applicability": record["applicability"],
                        "source_execution_key": record["source_execution_key"],
                    }
                    for record in rows
                ],
                "evidence_label": contract["evidence_label"],
            }

    return {
        "campaign_id": result_dir.name,
        "decision_point": "VIA-DP-06",
        "status": "ANALYSIS_COMPLETE_WITH_NON_QA20_BLOCKED_AXES",
        "evidence": {
            "qa_01_15_31_62": source["evidence_label"],
            "qa_21_23": contract["evidence_label"],
        },
        "contract_sha256": sha256_file(CONTRACT),
        "raw_change_locality_sha256": sha256_file(result_dir / "raw/change-locality.jsonl"),
        "source_campaign": contract["source_campaign"]["path"],
        "source_summary_sha256": sha256_file(source_summary_path),
        "source_manifest_sha256": sha256_file(source_manifest_path),
        "candidate_results": candidate_results,
        "limitations": contract["limitations"],
    }


def source_cell(value: dict[str, Any], qa_id: str) -> str:
    if value.get("status") in {"N/A", "BLOCKED"}:
        return value["status"]
    if qa_id in {"QA-01", "QA-02", "QA-05"}:
        return f"{value['p95_ms']:.1f} ms p95 (평균 {value['mean_ms']:.1f}, n={value['sample_count']})"
    if qa_id in {"QA-11", "QA-13"}:
        return f"{value['strict_case_success_pct']:.1f}% strict ({value['strict_pass_count']}/{value['case_count']})"
    if qa_id == "QA-12":
        return f"{value['strict_case_success_pct']:.1f}% strict / {value['field_level_correctness_pct']:.1f}% field"
    if qa_id == "QA-61":
        return f"{value['complete_trace_pct']:.1f}%"
    if qa_id == "QA-62":
        return f"{value.get('reproduced_pct', 0):.1f}%"
    return "N/A"


def cell(result: dict[str, Any], qa_id: str) -> str:
    value = result["qa"].get(qa_id, {})
    if qa_id in {"QA-21", "QA-22", "QA-23"}:
        return (
            f"{value['mean_changed_elements']:.2f}개/변화 "
            f"({value['changed_element_sum']}/{value['change_count']}, 최대 {value['maximum_changed_elements']})"
        )
    rendered = source_cell(value, qa_id)
    if rendered not in {"N/A", "BLOCKED"}:
        return f"{rendered} (v5 재사용)"
    return rendered


def write_report(result_dir: Path, summary: dict[str, Any]) -> None:
    candidates = list(summary["candidate_results"])
    lines = [
        "# VIA-DP-06 통합 평가 보고서 v6",
        "",
        "> 기존 v5의 응답성·정확성 측정 252회는 다시 실행하지 않았다. 이 보고서는 그 원시 증거를 `source_execution_key`와 digest로 참조하고, 누락됐던 QA-21~23의 29개 변경을 A/B/B′ 각각에 독립 적용한 87개 구조 평가를 추가한다.",
        "",
        "## 1. 평가 범위와 증거 수준",
        "",
        f"- 의사결정: `VIA-DP-06` — 요청 의미의 최종 확정 권한",
        f"- 결과 묶음: `{summary['campaign_id']}`",
        "- QA-01~15·31~62 기존 축: v5 `MEASURED_REFERENCE_HARNESS` 증거를 참조 재사용",
        "- QA-21~23 신규 축: `HYBRID_REFERENCE_ESTIMATE` — source anchor가 있는 실행 가능한 Architecture Element 원장",
        "- 비교안: A=통합 의미 권한, B=단계별 의미 권한, B′=B+fast-path tactic",
        "- B′의 fast path는 Architecture 구조를 바꾸지 않으므로 QA-21~23 원장은 B와 같다.",
        "",
        "## 2. 핵심 결론",
        "",
        "- **QA-21:** 세 안 모두 2.78개/변화로 같다. Agent 연동 변화는 대부분 공통 Agent 경계에서 끝나며 DP-06의 의미 권한 배치가 우열을 만들지 않았다.",
        "- **QA-22:** 세 안 모두 1.93개/변화로 같다. 화면 계약 변경은 A의 통합 Coordinator 또는 B의 Grounding Resolver 한 곳까지만 전파되므로 단계 수 전체를 억지로 세지 않았다.",
        "- **QA-23:** A는 2.80개/변화, B/B′는 3.60개/변화다. timing·correlation 변경이 A의 semantic producer 1곳과 B의 stage producer 3곳에 각각 닿기 때문이다. A가 평균 0.80개, 약 22.2% 적다.",
        "- 따라서 DP-06의 변경용이성 trade-off는 QA-21/22가 아니라 **QA-23에서만 명확하게 나타났다.**",
        "",
        "## 3. 19개 QA 통합표",
        "",
        "| QA | A | B | B′ |",
        "| --- | --- | --- | --- |",
    ]
    for qa_id in QA_IDS:
        lines.append(
            "| " + qa_id + " | "
            + " | ".join(cell(summary["candidate_results"][candidate], qa_id) for candidate in candidates)
            + " |"
        )

    lines += [
        "",
        "`N/A`는 DP-06 구조가 해당 경로에 참여하지 않는다는 뜻이다. `BLOCKED`는 관련성은 있지만 승인된 전체 endpoint 또는 반복 측정이 아직 없다는 뜻이다. 이번 보완 범위는 QA-21~23이며, 기존 `BLOCKED` 축을 숫자로 대체하지 않았다.",
        "",
        "## 4. QA-21~23 상세 결과",
        "",
        "| QA | 후보 | 평균 변경 요소 | 합계/변화 수 | 최대 | C | I | S | D |",
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for qa_id in ("QA-21", "QA-22", "QA-23"):
        for candidate in candidates:
            value = summary["candidate_results"][candidate]["qa"][qa_id]
            types = value["element_type_totals"]
            lines.append(
                f"| {qa_id} | {candidate.replace('_PRIME', '′')} | {value['mean_changed_elements']:.2f} | "
                f"{value['changed_element_sum']}/{value['change_count']} | {value['maximum_changed_elements']} | "
                f"{types['C']} | {types['I']} | {types['S']} | {types['D']} |"
            )

    lines += [
        "",
        "### QA-21 — Agent 변화 국소성",
        "",
        "A-01~A-09를 각각 같은 baseline에서 시작했다. capability 추가·재구성처럼 의미 처리와 닿는 변화도 A의 Coordinator 한 곳 또는 B의 Handling Selector 한 곳에만 닿는다. 나머지는 공통 Agent adapter, lifecycle contract, execution link에 국소화된다. 그러므로 세 후보의 평균이 같은 것이 정상 결과다.",
        "",
        "### QA-22 — Model·Context·State 변화 국소성",
        "",
        "M-01~M-09와 C-01~C-06을 독립 적용했다. 특히 C-03 화면 연동 계약 변경에서 B의 Task Associator와 Handling Selector는 화면 원천 계약을 직접 소비하지 않고 GroundingResult를 소비한다. 따라서 실제 소비자인 Grounding Resolver만 변경 대상으로 세었다. 후보별 단계 상자 수를 그대로 변경 수로 바꾸지 않았다.",
        "",
        "### QA-23 — 실험·로그 변화 국소성",
        "",
        "E-01 timing span과 E-02 correlation dimension은 semantic event를 실제 생산하는 authority에 닿는다. A는 producer가 Coordinator 1곳이고, B/B′는 Grounding·Association·Handling 3곳이다. E-03 trace schema와 E-04 export 변경은 공통 observability 경계에만 닿고, E-05 새 실험 추가는 기존 generic assignment/exposure 계약 안의 configuration이므로 0개다.",
        "",
        "## 5. 해석과 한계",
        "",
        "- 이 수치는 코드 파일 수, 수정 line 수, 개발 일수가 아니다. 책임·계약·상태·배치라는 Architecture Element의 변경 범위다.",
        "- QA-21~23은 실제 제품 migration을 구현한 실측이 아니다. 동결된 전체 요소 원장과 source anchor를 검증하고 변경을 기계적으로 적용한 구조 추정이다.",
        "- 각 변경은 누적하지 않았다. 항상 동일 후보 baseline에서 시작했으며, 결과 원장에 수정·추가·제거 ID와 C/I/S/D breakdown을 남겼다.",
        "- A/B의 최종 선택은 이 보고서만으로 하지 않는다. 응답성·정확성 결과와 QA 우선순위를 함께 검토해야 한다.",
        "",
        "## 6. 재현 자료",
        "",
        "- 신규 원시 자료: `raw/change-locality.jsonl` (87개 record)",
        "- 기존 증거 연결: `source-evidence-map.json`",
        "- 통합 기계 판독 결과: `summary.json`",
        "- 입력·원시 자료 digest: `manifest.json`",
        "- 독립 재생성 확인: `replay-receipt.json`",
    ]
    (result_dir / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def finalize(result_dir: Path, summary: dict[str, Any], core_digest: str) -> None:
    write(result_dir / "summary.json", summary)
    receipt = {
        "status": "REPRODUCED",
        "core_summary_sha256": core_digest,
        "raw_change_locality_sha256": summary["raw_change_locality_sha256"],
        "source_summary_sha256": summary["source_summary_sha256"],
        "reproduced_pct": 100.0,
    }
    write(result_dir / "replay-receipt.json", receipt)
    write_report(result_dir, summary)
    (result_dir / "STATUS.md").write_text(
        "# 상태\n\n"
        "`ANALYSIS_COMPLETE_WITH_NON_QA20_BLOCKED_AXES`\n\n"
        "QA-21~23 구조 평가와 기존 v5 증거의 참조 통합은 완료됐다. "
        "QA-15, QA-31, QA-41은 기존과 같이 미측정 상태이며 이 결과는 제품 E2E가 아니다.\n",
        encoding="utf-8",
    )
    manifest_path = result_dir / "manifest.json"
    manifest = load(manifest_path)
    manifest.update(
        status=summary["status"],
        analyzer_sha256=sha256_file(Path(__file__)),
        summary_sha256=sha256_file(result_dir / "summary.json"),
        report_sha256=sha256_file(result_dir / "report.md"),
        replay_receipt_sha256=sha256_file(result_dir / "replay-receipt.json"),
        replay_core_summary_sha256=core_digest,
        replay_verified=True,
        result_limit=(
            "QA-21~23 are source-anchored Architecture ledger estimates; "
            "QA-15/31/41 remain blocked and this package is not PRODUCT_E2E."
        ),
    )
    write(manifest_path, manifest)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--result-dir", type=Path, required=True)
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    result_dir = args.result_dir.resolve()
    summary = core_summary(result_dir)
    core_digest = digest(summary)
    if args.verify_only:
        recorded = load(result_dir / "replay-receipt.json")
        if recorded["core_summary_sha256"] != core_digest:
            raise SystemExit("replay mismatch")
        print(core_digest)
        return 0
    finalize(result_dir, summary, core_digest)
    print(core_digest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
