from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "benchmark/fixtures/dp00-executable-v3"


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def materialize() -> dict[str, str]:
    catalog_path = ROOT / "benchmark/scenarios/qa-v1/goals/instances/catalog.json"
    catalog = load(catalog_path)["instances"]
    semantic: list[dict[str, Any]] = []
    agents: list[dict[str, Any]] = []
    for item in catalog:
        semantics = item["expected_task_semantics"]
        committed = item["user_input"]["committed_representation"].split(":")
        semantic.append({
            "case_id": item["goal_id"],
            "goal": semantics["intent"],
            "referent": ":".join(committed[1:3]),
            "constraints": semantics["constraints"],
            "consent_required": semantics["consent_required"],
            "read_only": semantics["read_only"],
            "capability": item["agent_capability_requirements"][0],
            "modality": item["modality"],
            "semantic_family": item["qa02_stratum"],
            "revision_signal": any("revision" in event["event"] for event in item["temporal_interaction_events"]),
        })
        agents.append({
            "case_id": item["goal_id"],
            "status": "success",
            "result_version": 1,
            "facts": item["required_result_facts"],
            "events": ["accepted", "progress", "completion"],
        })
    semantic_path = OUT / "semantic-replay-v3.json"
    agent_path = OUT / "agent-replay-v3.json"
    write(semantic_path, {"artifact_class": "SemanticReplay", "version": 3, "candidate_neutral": True, "records": semantic})
    write(agent_path, {"artifact_class": "AgentReplay", "version": 3, "candidate_neutral": True, "records": agents})
    manifest = {
        "artifact_class_separation": ["RuntimeInput", "SemanticReplay", "AgentReplay", "EvaluatorOracle"],
        "candidate_visible": [semantic_path.relative_to(ROOT).as_posix(), agent_path.relative_to(ROOT).as_posix()],
        "candidate_excluded": ["benchmark/**/oracles.json"],
        "hashes": {
            semantic_path.relative_to(ROOT).as_posix(): sha256(semantic_path),
            agent_path.relative_to(ROOT).as_posix(): sha256(agent_path),
            catalog_path.relative_to(ROOT).as_posix(): sha256(catalog_path),
        },
    }
    manifest_path = OUT / "manifest.json"
    write(manifest_path, manifest)
    return {**manifest["hashes"], manifest_path.relative_to(ROOT).as_posix(): sha256(manifest_path)}


if __name__ == "__main__":
    print(json.dumps(materialize(), indent=2, sort_keys=True))

