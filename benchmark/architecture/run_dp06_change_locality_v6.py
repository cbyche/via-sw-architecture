#!/usr/bin/env python3
"""Execute the frozen VIA-DP-06 QA-21~23 Architecture Element ledger."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-06-change-locality-v6.json"


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


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_revision() -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()


def candidate_elements(ledger: dict[str, Any], candidate: str) -> list[dict[str, Any]]:
    candidate_value = ledger["candidate_elements"][candidate]
    if isinstance(candidate_value, dict) and "same_as" in candidate_value:
        candidate_value = ledger["candidate_elements"][candidate_value["same_as"]]
    return copy.deepcopy(ledger["shared_elements"] + candidate_value)


def validate_inputs(contract: dict[str, Any], ledger: dict[str, Any], pack: dict[str, Any]) -> None:
    candidates = contract["candidates"]
    if pack["candidates"] != candidates:
        raise ValueError("change-pack candidate order differs from frozen contract")
    expected = contract["required_change_ids"]
    actual = {qa_id: [] for qa_id in expected}
    for change in pack["changes"]:
        if change["qa_id"] not in actual:
            raise ValueError(f"unexpected QA in change pack: {change['qa_id']}")
        actual[change["qa_id"]].append(change["id"])
        if set(change["impacts"]) != set(candidates):
            raise ValueError(f"candidate coverage is incomplete: {change['id']}")
    if actual != expected:
        raise ValueError(f"change pack differs from frozen membership: {actual}")

    required_fields = {
        "id", "type", "name", "responsibility", "owner", "users", "boundary",
        "independence", "change_when", "design_source", "anchors",
    }
    for candidate in candidates:
        elements = candidate_elements(ledger, candidate)
        ids = [element["id"] for element in elements]
        if len(ids) != len(set(ids)):
            raise ValueError(f"duplicate element ID in candidate {candidate}")
        for element in elements:
            missing = required_fields - set(element)
            if missing:
                raise ValueError(f"{candidate}/{element.get('id')} missing {sorted(missing)}")
            if element["type"] not in contract["element_types"]:
                raise ValueError(f"invalid element type: {element['id']}")
            if not element["anchors"]:
                raise ValueError(f"element has no source anchor: {element['id']}")
            for anchor in element["anchors"]:
                if not (ROOT / anchor).exists():
                    raise ValueError(f"missing source anchor: {element['id']} -> {anchor}")


def apply_change(
    baseline: list[dict[str, Any]], candidate: str, change: dict[str, Any], shared_capabilities: list[str]
) -> dict[str, Any]:
    before = copy.deepcopy(baseline)
    after = copy.deepcopy(baseline)
    by_id = {element["id"]: element for element in after}
    modified: list[str] = []
    added: list[str] = []
    removed: list[str] = []
    seen: set[str] = set()

    for operation in change["impacts"][candidate]:
        op, element_id = operation.split(":", 1)
        if element_id in seen:
            raise ValueError(f"duplicate impact {candidate}/{change['id']}/{element_id}")
        seen.add(element_id)
        if op == "M":
            if element_id not in by_id:
                raise ValueError(f"modified element absent from baseline: {element_id}")
            by_id[element_id]["change_application"] = change["id"]
            modified.append(element_id)
        elif op == "A":
            if element_id in by_id:
                raise ValueError(f"added element already exists: {element_id}")
            element_type = element_id.split("-", 1)[0]
            if element_type not in {"C", "I", "S", "D"}:
                raise ValueError(f"cannot infer added element type: {element_id}")
            new_element = {
                "id": element_id,
                "type": element_type,
                "name": f"{change['name']} 추가 요소",
                "responsibility": change["before_after"],
                "owner": "candidate change design",
                "users": ["preserved VIA capability"],
                "boundary": change["completion"],
                "independence": "Registered as an independent element by the frozen impact ledger.",
                "change_when": f"The {change['id']} adaptation changes.",
                "design_source": "benchmark/architecture/fixtures/dp06-change-locality-pack-v6.json",
                "anchors": ["benchmark/architecture/fixtures/dp06-change-locality-pack-v6.json"],
                "change_application": change["id"],
            }
            after.append(new_element)
            by_id[element_id] = new_element
            added.append(element_id)
        elif op == "R":
            if element_id not in by_id:
                raise ValueError(f"removed element absent from baseline: {element_id}")
            after = [element for element in after if element["id"] != element_id]
            by_id.pop(element_id)
            removed.append(element_id)
        else:
            raise ValueError(f"unknown operation: {operation}")

    changed = modified + added + removed
    breakdown = {element_type: 0 for element_type in ("C", "I", "S", "D")}
    before_by_id = {element["id"]: element for element in before}
    for element_id in changed:
        source = by_id.get(element_id) or before_by_id[element_id]
        breakdown[source["type"]] += 1
    if sum(breakdown.values()) != len(changed):
        raise AssertionError("C/I/S/D breakdown does not match changed-element count")

    return {
        "schema_version": "via.dp06.change-locality-trial.v6",
        "evidence_label": "HYBRID_REFERENCE_ESTIMATE",
        "measurement_scope": "SOURCE_ANCHORED_EXECUTABLE_ARCHITECTURE_LEDGER",
        "candidate": candidate,
        "qa_id": change["qa_id"],
        "change_id": change["id"],
        "change_name": change["name"],
        "applicability": change["applicability"],
        "source_execution_key": f"dp06-change-locality-v6:{candidate}:{change['id']}",
        "baseline_reset": True,
        "before_ledger_sha256": digest(before),
        "after_ledger_sha256": digest(after),
        "modified_element_ids": modified,
        "added_element_ids": added,
        "removed_element_ids": removed,
        "changed_element_count": len(changed),
        "element_type_breakdown": breakdown,
        "preserved_capabilities": shared_capabilities,
        "completion_contract": change["completion"],
        "functional_evidence": "DESIGN_CONTRACT_REVIEWED_NOT_PRODUCT_RUNTIME_TESTED",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()

    contract = load(CONTRACT)
    ledger_path = ROOT / contract["element_ledger"]
    pack_path = ROOT / contract["change_pack"]
    ledger = load(ledger_path)
    pack = load(pack_path)
    validate_inputs(contract, ledger, pack)

    output_dir = args.output_dir.resolve()
    if output_dir.exists() and any(output_dir.iterdir()):
        raise RuntimeError(f"refusing to overwrite non-empty result directory: {output_dir}")
    (output_dir / "raw").mkdir(parents=True, exist_ok=True)

    records: list[dict[str, Any]] = []
    baseline_digests: dict[str, str] = {}
    for candidate in contract["candidates"]:
        baseline = candidate_elements(ledger, candidate)
        baseline_digests[candidate] = digest(baseline)
        for change in pack["changes"]:
            records.append(
                apply_change(baseline, candidate, change, ledger["shared_capabilities"])
            )

    raw_path = output_dir / "raw/change-locality.jsonl"
    raw_path.write_text(
        "".join(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n" for record in records),
        encoding="utf-8",
    )
    source_path = ROOT / contract["source_campaign"]["path"]
    source_summary = source_path / "summary.json"
    source_manifest = source_path / "manifest.json"
    source_map = {
        "reuse_mode": "REFERENCE_ONLY_NOT_INDEPENDENT_SAMPLES",
        "source_campaign": contract["source_campaign"]["path"],
        "source_summary_sha256": sha256_file(source_summary),
        "source_manifest_sha256": sha256_file(source_manifest),
        "source_raw_sha256": load(source_manifest)["raw_sha256"],
        "reused_qa_ids": ["QA-01", "QA-02", "QA-03", "QA-04", "QA-05", "QA-11", "QA-12", "QA-13", "QA-14", "QA-15", "QA-31", "QA-32", "QA-41", "QA-51", "QA-61", "QA-62"],
        "newly_executed_qa_ids": ["QA-21", "QA-22", "QA-23"],
    }
    write(output_dir / "source-evidence-map.json", source_map)
    write(
        output_dir / "manifest.json",
        {
            "campaign_id": output_dir.name,
            "status": "RAW_COMPLETE_PENDING_ANALYSIS",
            "started_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "completed_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "source_revision": git_revision(),
            "evidence_label": contract["evidence_label"],
            "contract_sha256": sha256_file(CONTRACT),
            "element_ledger_sha256": sha256_file(ledger_path),
            "change_pack_sha256": sha256_file(pack_path),
            "source_evidence_map_sha256": sha256_file(output_dir / "source-evidence-map.json"),
            "raw_sha256": sha256_file(raw_path),
            "record_count": len(records),
            "baseline_ledger_sha256": baseline_digests,
            "environment": {
                "platform": platform.platform(),
                "python": platform.python_version(),
                "machine": platform.machine(),
            },
        },
    )
    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
