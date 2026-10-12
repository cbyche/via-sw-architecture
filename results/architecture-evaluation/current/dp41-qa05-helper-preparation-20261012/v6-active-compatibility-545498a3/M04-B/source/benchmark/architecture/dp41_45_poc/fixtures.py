"""P21 source fixtures and separately loaded evaluator oracles.

Only ``public_inputs`` may cross into the candidate/model request boundary.
The replay plan and oracle deliberately live outside that returned object.
"""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any

FIXTURE_DIR = Path(__file__).with_name("fixtures")
_REPO_ROOT = Path(__file__).resolve().parents[3]


def _read(name: str) -> dict[str, Any]:
    return json.loads((FIXTURE_DIR / name).read_text(encoding="utf-8"))


def load_cases() -> list[dict[str, Any]]:
    """Return harness inputs/plans; no evaluator oracle is loaded here."""
    return copy.deepcopy(_read("inputs.json")["cases"])


def load_oracles() -> dict[str, dict[str, Any]]:
    """Evaluator-only entry point. Never invoke from the candidate process."""
    return copy.deepcopy(_read("oracles.json")["oracles"])


def load_manifest() -> dict[str, Any]:
    return copy.deepcopy(_read("manifest.json"))


def public_inputs(case: dict[str, Any]) -> dict[str, Any]:
    """Allowlist candidate input without the replay plan or case selection key.

    Source bodies belong to the candidate's owner repository, not the initial
    model context. The candidate must use the read port to receive their text.
    """
    return copy.deepcopy({
        "transcript": case.get("transcript"),
        "root_voice_seconds": case["root_voice_seconds"],
        "initial": case["initial"],
        "input_evidence": case.get("input_evidence", {}),
    })


def validate_fixtures() -> list[str]:
    """Validate source provenance, observation populations and privacy boundaries."""
    errors: list[str] = []
    manifest = load_manifest()
    for relative, expected in manifest["canonical_sources"].items():
        path = _REPO_ROOT / relative
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            errors.append(f"canonical source changed: {relative}")
    cases, oracles = load_cases(), load_oracles()
    case_ids = [case["case_id"] for case in cases]
    if case_ids != manifest["sequence"] or len(set(case_ids)) != 21:
        errors.append("P21 case membership/order differs from manifest")
    if set(case_ids) != set(oracles):
        errors.append("case/oracle membership mismatch")
    reaction_ids = [item["id"] for case in cases for item in case["reactions"]]
    cycle_ids = [case["cycle"]["id"] for case in cases if case["cycle"]]
    if len(reaction_ids) != 27 or len(set(reaction_ids)) != 27:
        errors.append("reaction population must contain 27 distinct observations")
    if len(cycle_ids) != 19 or len(set(cycle_ids)) != 19:
        errors.append("cycle population must contain 19 distinct observations")
    for case in cases:
        case_id = case["case_id"]
        if set(oracles[case_id]["conditions"]) != {f"C{i}" for i in range(1, 7)}:
            errors.append(f"{case_id}: missing C1..C6")
        for condition in oracles[case_id]["conditions"].values():
            if not condition["mandatory_facts"]:
                errors.append(f"{case_id}: empty mandatory condition")
        public = public_inputs(case)
        if any(key in public for key in ("case_id", "oracle", "events", "followups", "stub_profile", "preparation")):
            errors.append(f"{case_id}: private harness key leaked")
        for source in public["initial"]["sources"]:
            if source["permission"]["read"] == "DENIED" and source.get("text"):
                errors.append(f"{case_id}: denied source body leaked")
        previous = float("-inf")
        for event in case["events"]:
            if event["at_seconds"] < previous:
                errors.append(f"{case_id}: fixed events out of source order")
            previous = event["at_seconds"]
    return errors


def build_activity_baseline() -> dict[str, Any]:
    """Build the canonical raw background, without learned relationships.

    The 20 MiB contract covers source text, original turn text and task text.
    JSON/DB/index/runtime overhead is extra. Empty C slots receive scene source
    records from the harness; they are never pre-filled with current meanings.
    """
    spec = load_manifest()["initial_workload"]
    sources = [{
        "source_id": f"B{i:02d}", "revision": 1, "kind": "DOCUMENT",
        "title": f"P21 background source B{i:02d}",
        "text": f"index={i:02d}; value={100 + i}",
        "permission": {"read": "ALLOWED", "model": "ALLOWED", "agent": "ALLOWED"},
        "metadata": {},
    } for i in range(1, 21)]
    sources.extend({
        "source_id": f"C{i:02d}", "revision": 1, "kind": "SOURCE_SLOT",
        "title": f"P21 canonical source slot C{i:02d}", "text": "",
        "permission": {"read": "ALLOWED", "model": "ALLOWED", "agent": "ALLOWED"},
        "metadata": {"unpopulated": True},
    } for i in range(1, 11))
    interactions = []
    for i in range(1, spec["history_turns"] + 1):
        source_index = (i - 1) % 20 + 1
        text = f"배경 기록 H{i:04d}: B{source_index:02d}@v1의 값은 {100 + source_index}이다."
        interactions.append({
            "publication_id": f"H{i:04d}", "text": text,
            "delivered_voice_text": text, "delivered_receipt": True,
            "source_refs": [{"source_id": f"B{source_index:02d}", "revision": 1}],
        })
    tasks = []
    for i in range(1, spec["completed_tasks"] + 1):
        source_index = (i - 1) % 20 + 1
        text = f"B{source_index:02d}@v1 값 한정 조회"
        tasks.append({
            "task_id": f"HT{i:03d}", "execution_id": f"HX{i:03d}",
            "request_revision": 1, "status": "COMPLETED", "operation": "READ_FIELD",
            "instruction": text, "source_refs": [{"source_id": f"B{source_index:02d}", "revision": 1}],
            "properties": {}, "result": {"revision": 1, "text": f"값={100 + source_index}"},
            "state_history": ["ACCEPTED", "COMPLETED", "DELIVERED"],
        })
    measured = sum(len(s["text"].encode("utf-8")) for s in sources)
    measured += sum(len(i["text"].encode("utf-8")) for i in interactions)
    measured += sum(len(t["instruction"].encode("utf-8")) + len(t["result"]["text"].encode("utf-8")) for t in tasks)
    target = int(spec["source_payload_mib"]) * 1024 * 1024
    if measured > target:
        raise ValueError("canonical raw baseline exceeds payload budget before padding")
    sources[19]["text"] += " " * (target - measured)
    return {"sources": sources, "tasks": tasks, "interactions": interactions,
            "payload_bytes": target, "payload_size_basis": "source.text+interaction.text+task.instruction+task.result.text"}


def build_observation_buffers() -> list[bytearray]:
    """Allocate and touch the common raw-buffer workload; this is not vision evidence."""
    spec = load_manifest()["initial_workload"]
    pixels = int(spec["raw_frame_width"]) * int(spec["raw_frame_height"])
    return [bytearray(_rgba_pixel(index) * pixels) for index in range(int(spec["raw_rgba_frames"]))]


def _rgba_pixel(index: int) -> bytes:
    return bytes((index % 256, (17 * index) % 256, (31 * index) % 256, 255))


def observation_buffer_records(buffers: list[bytearray]) -> list[dict[str, Any]]:
    """Return reproducible dimensions, actual byte counts and hashes without saving pixels."""
    spec = load_manifest()["initial_workload"]
    width, height = int(spec["raw_frame_width"]), int(spec["raw_frame_height"])
    expected_size = width * height * 4
    if len(buffers) != int(spec["raw_rgba_frames"]) or len({id(frame) for frame in buffers}) != len(buffers):
        raise ValueError("Common observation buffers require distinct actual frames")
    records = []
    for index, frame in enumerate(buffers):
        if not isinstance(frame, bytearray) or len(frame) != expected_size:
            raise ValueError("Common observation frame type/dimensions differ from the contract")
        pixel = _rgba_pixel(index)
        expected_hash = hashlib.sha256()
        row = pixel * width
        for _ in range(height):
            expected_hash.update(row)
        actual_hash = hashlib.sha256(frame).hexdigest()
        if actual_hash != expected_hash.hexdigest():
            raise ValueError("Common observation frame differs from its deterministic touched contents")
        records.append({"frame_index": index, "width": width, "height": height,
                        "channels": 4, "byte_count": len(frame), "sha256": actual_hash,
                        "pixel_rgba": list(pixel), "generator": "P21_SYNTHETIC_RGBA_V1",
                        "contents_touched": True,
                        "scope": "COMMON_MEMORY_WORKLOAD_NOT_PIXEL_VISION_EVIDENCE"})
    return records
