"""Validate Gate 2 design inventories; generate unmeasured review ledgers.

This is NOT a VIA implementation or an accuracy/latency benchmark.
Only Python's standard library is used. Outputs never contain invented scores.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
from itertools import product
from pathlib import Path
from typing import Any

CORE = ("IR-DP01", "TASK-DP01", "AGENT-DP01", "EXEC-DP01")
WORKING = tuple(f"W-{i:02d}" for i in range(1, 13))
CHANGES = tuple(f"{p}-{i:02d}" for p, n in (("M", 9), ("A", 9), ("C", 6)) for i in range(1, n + 1))
ID_RE = re.compile(r"G2-([CISD])-[A-Z0-9-]+")
META_RE = re.compile(r"<!-- gate2: (.+?) -->")

def elements(text: str, source: str) -> dict[str, dict[str, str]]:
    """Read the normative element table, not IDs mentioned in prose."""
    found: dict[str, dict[str, str]] = {}
    for line in text.splitlines():
        if not line.startswith("| G2-"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != 4 or not all(cells):
            raise ValueError(f"Invalid element row in {source}: {line}")
        eid, responsibility, boundary, trigger = cells
        match = ID_RE.fullmatch(eid)
        if not match or eid in found:
            raise ValueError(f"Invalid or duplicate element ID: {source}: {eid}")
        found[eid] = {"id": eid, "type": match.group(1), "responsibility": responsibility,
                      "owner_consumers_lifetime": boundary, "change_trigger": trigger, "source": source}
    if not found:
        raise ValueError(f"No element definitions: {source}")
    return found


def current_revision(root: Path) -> str:
    env_sha = os.environ.get("GITHUB_SHA", "").strip().lower()
    if re.fullmatch(r"[0-9a-f]{40}", env_sha):
        return env_sha
    try:
        value = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip().lower()
    except (OSError, subprocess.CalledProcessError):
        return "UNAVAILABLE"
    return value if re.fullmatch(r"[0-9a-f]{40}", value) else "UNAVAILABLE"


def catalog_fingerprint(catalog: dict[str, Any]) -> str:
    structural = {
        "common": catalog["common"],
        "elements": catalog["elements"],
        "cards": {
            dp: {
                "reference": card["reference"],
                "alternatives": card["alternatives"],
                "hypotheses": card["hypotheses"],
            }
            for dp, card in sorted(catalog["cards"].items())
        },
    }
    payload = json.dumps(
        structural,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode()
    return hashlib.sha256(payload).hexdigest()


def load(root: Path) -> dict[str, Any]:
    base = root / "docs/rebaseline/12-gate2"
    common_path = base / "common-contract.md"
    common = elements(common_path.read_text(encoding="utf-8"), str(common_path.relative_to(root)))
    all_elements = dict(common)
    cards: dict[str, dict[str, Any]] = {}
    hashes: dict[str, str] = {}
    for path in sorted(base.glob("*.md")):
        hashes[str(path.relative_to(root))] = hashlib.sha256(path.read_bytes()).hexdigest()
    for dp in CORE:
        path = base / f"{dp}.md"
        text = path.read_text(encoding="utf-8")
        matches = META_RE.findall(text)
        if len(matches) != 1:
            raise ValueError(f"Expected exactly one gate2 metadata object: {dp}")
        meta = json.loads(matches[0])
        if meta["dp"] != dp or set(meta["alternatives"]) != {"A", "B"}:
            raise ValueError(f"Invalid alternative identity: {dp}")
        if meta["reference"] not in meta["alternatives"]:
            raise ValueError(f"Invalid reference alternative: {dp}")
        hypotheses = meta["hypotheses"]
        if not 1 <= len(hypotheses) <= 4 or len(hypotheses) != len(set(hypotheses)) or not set(hypotheses) <= set(WORKING):
            raise ValueError(f"Invalid hypothesis set: {dp}")
        local = elements(text, str(path.relative_to(root)))
        if set(local) & set(all_elements):
            raise ValueError(f"Element definition duplicated across documents: {dp}")
        selections: set[str] = set()
        for choice, ids in meta["alternatives"].items():
            if not ids or len(ids) != len(set(ids)) or not set(ids) <= set(local):
                raise ValueError(f"Unregistered or duplicated alternative element: {dp}/{choice}")
            selections.update(ids)
        if selections != set(local):
            raise ValueError(f"Unselected element definitions: {dp}")
        if set(meta["alternatives"]["A"]) == set(meta["alternatives"]["B"]):
            raise ValueError(f"No registered structural delta: {dp}")
        if text.count("```mermaid") != 2:
            raise ValueError(f"Expected two candidate diagrams: {dp}")
        cards[dp] = meta
        all_elements.update(local)
    return {"root": str(root), "common": common, "elements": all_elements, "cards": cards, "source_hashes": hashes}


def compose(catalog: dict[str, Any], vector: dict[str, str]) -> list[str]:
    if set(vector) != set(CORE):
        raise ValueError("Configuration must choose every Core DP exactly once")
    active = set(catalog["common"])
    for dp in CORE:
        choice = vector[dp]
        if choice not in {"A", "B"}:
            raise ValueError(f"Unknown alternative: {dp}/{choice}")
        active.update(catalog["cards"][dp]["alternatives"][choice])
    if not all(any(catalog["elements"][eid]["type"] == t for eid in active) for t in "CISD"):
        raise ValueError("Incomplete C/I/S/D inventory")
    return sorted(active)


def bindings(vector: dict[str, str]) -> dict[str, Any]:
    task_writer = "G2-C-TASKSERVICE" if vector["TASK-DP01"] == "A" else "G2-C-TASKACTOR"
    return {
        "voice_direct_release_owner": "G2-C-VOICE",
        "task_state_writer": task_writer,
        "execution_link_writer": task_writer,
        "normal_sync_owner": "G2-C-AGENTSYNC",
        "agent_semantics_owner": "G2-C-EDGESEM" if vector["AGENT-DP01"] == "A" else "G2-C-TYPEDHANDLER",
        "integration_host": "G2-D-VIA" if vector["EXEC-DP01"] == "A" else "G2-D-INTEGRATION",
        "task_host": "G2-D-VIA",
        "repository_host": "G2-D-VIA",
        "integration_db_write_permission": False,
    }


def factorial_vectors() -> list[tuple[str, dict[str, str]]]:
    """Return the complete 2^4 design in the canonical IR/TASK/AGENT/EXEC order."""
    return [
        ("".join(choices), dict(zip(CORE, choices, strict=True)))
        for choices in product("AB", repeat=len(CORE))
    ]


def make_report(catalog: dict[str, Any]) -> dict[str, Any]:
    reference = {dp: catalog["cards"][dp]["reference"] for dp in CORE}
    configs: dict[str, dict[str, Any]] = {}
    pair_records = []
    metric_ledger = []
    change_ledger = []

    for key, vector in factorial_vectors():
        active = compose(catalog, vector)
        owner_bindings = bindings(vector)
        if not {
            value
            for value in owner_bindings.values()
            if isinstance(value, str) and value.startswith("G2-")
        } <= set(active):
            raise ValueError(f"Binding to inactive element: {key}")
        configs[key] = {
            "id": key,
            "choices": vector,
            "element_ids": active,
            "bindings": owner_bindings,
            "element_type_counts": {
                kind: sum(catalog["elements"][eid]["type"] == kind for eid in active)
                for kind in "CISD"
            },
        }

    for dp in CORE:
        for alt in ("A", "B"):
            vector = dict(reference)
            vector[dp] = alt
            key = "".join(vector[k] for k in CORE)
            cid = f"{dp}/{alt}"
            pair_records.append({"candidate_id": cid, "configuration_id": key, "hypotheses": catalog["cards"][dp]["hypotheses"]})
            for w in WORKING:
                metric_ledger.append({"candidate_id": cid, "configuration_id": key, "working_asr": w,
                                      "prior_role": "H" if w in catalog["cards"][dp]["hypotheses"] else "R",
                                      "metric_value": None, "score": None, "evidence": "NOT_RUN"})
            for change in CHANGES:
                change_ledger.append({"candidate_id": cid, "configuration_id": key, "change_id": change,
                                      "modified": None, "added": None, "removed": None,
                                      "unique_count": None, "functional_preservation": None, "evidence": "NOT_ANALYZED"})
    if len(configs) != 16 or len(pair_records) != 8 or len(metric_ledger) != 96 or len(change_ledger) != 192:
        raise ValueError("Unexpected complete-configuration or ledger cardinality")
    return {"version": "G2-DESIGN-v1.2",
            "source_revision": current_revision(Path(catalog["root"])),
            "catalog_fingerprint": catalog_fingerprint(catalog),
            "status": "DESIGN_REVIEW_ONLY", "reference_choices": reference,
            "source_hashes": catalog["source_hashes"], "elements": catalog["elements"],
            "configurations": configs, "pair_candidates": pair_records,
            "metrics": metric_ledger, "changes": change_ledger,
            "warning": "Static design metadata only; no candidate execution, model accuracy, latency or winner."}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[3])
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = make_report(load(args.root.resolve()))
    summary = {"status": "PASS_STATIC_DESIGN_CHECKS", "dp_pairs": len(CORE),
               "pair_candidates": len(report["pair_candidates"]), "unique_configurations": len(report["configurations"]),
               "registered_elements": len(report["elements"]), "unmeasured_metric_rows": len(report["metrics"]),
               "unanalyzed_change_rows": len(report["changes"]), "candidate_execution": "NOT_RUN"}
    if args.output:
        args.output.mkdir(parents=True, exist_ok=True)
        for name, data in (("design-manifest.json", report), ("validation-summary.json", summary)):
            (args.output / name).write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
