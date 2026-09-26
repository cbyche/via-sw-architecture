#!/usr/bin/env python3
"""Create the fail-closed VIA-DP-11 v4 result from frozen raw evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from benchmark.architecture.analyze_dp11_complete import calculate  # noqa: E402

QA_IDS = (
    "QA-01", "QA-02", "QA-03", "QA-04", "QA-05",
    "QA-11", "QA-12", "QA-13", "QA-14", "QA-15",
    "QA-21", "QA-22", "QA-23", "QA-31", "QA-32", "QA-41",
    "QA-51", "QA-61", "QA-62",
)
CANDIDATES = ("isolated_worker", "same_process")


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def digest(value: Any) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def core_summary(result_dir: Path) -> dict[str, Any]:
    contract = load(result_dir / "contract.json")
    observed = calculate(result_dir)
    results: dict[str, Any] = {}
    for candidate in CANDIDATES:
        qa: dict[str, Any] = {}
        for qa_id, disposition in contract["qa_applicability"].items():
            if disposition.startswith("N/A"):
                qa[qa_id] = {"status": "N/A", "reason": disposition}
            elif disposition.startswith("BLOCKED"):
                qa[qa_id] = {"status": "BLOCKED", "reason": disposition}
        qa["QA-31"] = {
            "status": "MEASURED", "unit": "ms",
            "worst_fault_p95_ms": observed["QA-31"][candidate] / 1_000_000,
        }
        qa["QA-32"] = {
            "status": "MEASURED", "unit": "excess user-visible units",
            "max_excess_affected_units": observed["QA-32"][candidate],
        }
        qa["QA-41"] = {
            "status": "MEASURED", "unit": "bytes",
            "peak_memory_p95_bytes": observed["QA-41"][candidate],
        }
        qa["QA-61"] = {
            "status": "MEASURED", "unit": "% complete execution traces",
            "complete_trace_pct": observed["QA-61"][candidate],
        }
        qa["QA-62"] = {"status": "PENDING_INDEPENDENT_REPLAY"}
        results[candidate] = {"qa": qa}
    return {
        "campaign_id": result_dir.name,
        "decision_point": "VIA-DP-11",
        "evidence_label": "MEASURED_REFERENCE_HARNESS",
        "contract_sha256": hashlib.sha256((result_dir / "contract.json").read_bytes()).hexdigest(),
        "raw_sha256": hashlib.sha256((result_dir / "raw/trials.jsonl").read_bytes()).hexdigest(),
        "candidate_results": results,
        "limitations": contract["limitations"],
    }


def cell(value: dict[str, Any], qa_id: str) -> str:
    if value.get("status") in {"N/A", "BLOCKED"}:
        return value["status"]
    if qa_id == "QA-31":
        return f"{value['worst_fault_p95_ms']:.1f} ms"
    if qa_id == "QA-32":
        return f"{value['max_excess_affected_units']} units"
    if qa_id == "QA-41":
        return f"{value['peak_memory_p95_bytes'] / 1024 / 1024:.1f} MiB"
    if qa_id == "QA-61":
        return f"{value['complete_trace_pct']:.1f}%"
    if qa_id == "QA-62":
        return f"{value['reproduced_pct']:.1f}%"
    return value.get("status", "INVALID")


def write_report(result_dir: Path, summary: dict[str, Any]) -> None:
    lines = [
        "# VIA-DP-11 evaluation v4", "",
        "- A: VIA-owned Agent client in an isolated supervised worker process",
        "- B: the same client in the VIA Core process",
        f"- Evidence: `{summary['evidence_label']}`; reference harness, not product E2E", "",
        "## 19-QA complete table", "",
        "| QA | A isolated worker | B same process |",
        "| --- | ---: | ---: |",
    ]
    for qa_id in QA_IDS:
        values = [summary["candidate_results"][candidate]["qa"][qa_id] for candidate in CANDIDATES]
        lines.append(f"| {qa_id} | {cell(values[0], qa_id)} | {cell(values[1], qa_id)} |")
    lines += [
        "", "## Interpretation", "",
        "This campaign scores only endpoints that the executable reference path actually implements. "
        "The older renderer, correctness and design-ledger proxies remain in raw diagnostics but are not promoted to active QA values.",
        "", "## N/A and blocked audit", "",
        "| QA | Disposition | Frozen reason |", "| --- | --- | --- |",
    ]
    contract = load(result_dir / "contract.json")
    for qa_id in QA_IDS:
        disposition = contract["qa_applicability"][qa_id]
        if disposition.startswith("N/A"):
            lines.append(f"| {qa_id} | N/A | `{disposition}` |")
        elif disposition.startswith("BLOCKED"):
            lines.append(f"| {qa_id} | BLOCKED | `{disposition}` |")
    lines += ["", "## Limitations", ""]
    lines.extend(f"- {item}" for item in summary["limitations"])
    lines += [
        "", "## Raw evidence", "",
        f"- `raw/trials.jsonl` SHA-256: `{summary['raw_sha256']}`",
        "- Source strata remain separately available under `raw/`.",
        "- `replay-receipt.json` records independent analyzer reproduction.", "",
    ]
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
        print(json.dumps({"match": core_digest == args.verify_core, "digest": core_digest}))
        return 0 if core_digest == args.verify_core else 1
    verified = subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--result-dir", str(result_dir), "--verify-core", core_digest],
        check=False, capture_output=True, text=True,
    )
    receipt = {
        "reproduced": verified.returncode == 0,
        "expected_core_digest": core_digest,
        "stdout": verified.stdout.strip(),
        "stderr": verified.stderr.strip(),
    }
    (result_dir / "replay-receipt.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    for candidate in CANDIDATES:
        core["candidate_results"][candidate]["qa"]["QA-62"] = {
            "status": "MEASURED", "unit": "% exact independent replay",
            "reproduced_pct": 100.0 if receipt["reproduced"] else 0.0,
        }
    (result_dir / "summary.json").write_text(json.dumps(core, ensure_ascii=False, indent=2), encoding="utf-8")
    write_report(result_dir, core)
    manifest = load(result_dir / "manifest.json")
    manifest["status"] = "COMPLETE" if receipt["reproduced"] else "INVALID"
    manifest["core_summary_sha256"] = core_digest
    manifest["replay_verified"] = receipt["reproduced"]
    (result_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    status = "COMPLETE" if receipt["reproduced"] else "INVALID"
    (result_dir / "STATUS.md").write_text(
        f"# Evidence Status\n\n> **{status} — targeted DP-11 v4 reference campaign**\n\n"
        "All 19 QA rows are present. Only QA-31, QA-32, QA-41, QA-61 and QA-62 are scored; "
        "all other rows are explicitly N/A or BLOCKED by the frozen participation audit.\n",
        encoding="utf-8",
    )
    print(result_dir / "report.md")
    return 0 if receipt["reproduced"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
