"""Canonical 94-case -> Gate 2 executable-slice adapter plan.

This module does not fabricate candidate observations. It converts every reviewed
canonical Test Case into an explicit list of runtime/harness responsibilities and
reports which responsibilities are currently executable, fixture-only, external,
or blocked. It is a pre-measurement coverage gate, not W-05/W-06 scoring.
"""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
REBASELINE = ROOT / "benchmark/rebaseline"
CASES = REBASELINE / "cases.json"

STATUS = {
    "IR_MODEL": "HARNESS_READY_EXTERNAL_MODEL_PENDING",
    "TASK_RUNTIME": "IMPLEMENTED",
    "AGENT_FIXTURE": "IMPLEMENTED_FIXTURE",
    "EXEC_HOST": "IMPLEMENTED",
    "CONTEXT_FIXTURE": "FIXTURE_ONLY",
    "CONVERSATION_STATE": "FIXTURE_ONLY",
    "UI_INTERACTION": "FIXTURE_ONLY",
    "VOICE_RUNTIME": "NOT_IMPLEMENTED",
    "PRESENTATION": "NOT_IMPLEMENTED",
    "POLICY_SERVICE": "FIXTURE_ONLY",
    "MEMORY_SERVICE": "FIXTURE_ONLY",
    "COMPOUND_RUNTIME": "PARTIAL",
}

BLOCKING = {"NOT_IMPLEMENTED", "PARTIAL"}
EXTERNAL = {"HARNESS_READY_EXTERNAL_MODEL_PENDING"}


def ensure_assets() -> None:
    if CASES.exists():
        return
    subprocess.run(
        ["python3", str(REBASELINE / "build_assets.py")],
        cwd=ROOT,
        check=True,
    )


def uc_family(case: dict[str, Any]) -> int:
    raw = case["uc"]
    try:
        return int(raw.split("-", 1)[1].split(".", 1)[0])
    except (IndexError, ValueError) as exc:
        raise ValueError(f"invalid UC id: {raw}") from exc


def has_user_utterance(case: dict[str, Any]) -> bool:
    value = str(case["input"].get("utterance", "")).strip()
    return bool(value) and not value.startswith("[")


def responsibilities(case: dict[str, Any]) -> list[str]:
    uc = uc_family(case)
    required: set[str] = set()

    if has_user_utterance(case):
        required.add("IR_MODEL")

    if uc == 1:
        required |= {"CONVERSATION_STATE", "PRESENTATION"}
    elif uc == 2:
        required |= {"CONTEXT_FIXTURE", "PRESENTATION"}
    elif uc == 3:
        required |= {"CONTEXT_FIXTURE", "UI_INTERACTION", "PRESENTATION"}
    elif uc == 4:
        required |= {"CONTEXT_FIXTURE", "UI_INTERACTION", "VOICE_RUNTIME", "PRESENTATION"}
    elif uc == 5:
        required |= {"CONTEXT_FIXTURE", "CONVERSATION_STATE", "PRESENTATION"}
    elif uc == 6:
        required |= {"CONVERSATION_STATE", "PRESENTATION"}
    elif uc == 7:
        required |= {
            "VOICE_RUNTIME",
            "CONVERSATION_STATE",
            "TASK_RUNTIME",
            "AGENT_FIXTURE",
            "PRESENTATION",
        }
    elif uc == 8:
        required |= {"TASK_RUNTIME", "AGENT_FIXTURE", "PRESENTATION"}
    elif uc == 9:
        required |= {"COMPOUND_RUNTIME", "TASK_RUNTIME", "AGENT_FIXTURE", "PRESENTATION"}
    elif uc == 10:
        required |= {"TASK_RUNTIME", "AGENT_FIXTURE", "PRESENTATION"}
    elif uc == 11:
        required |= {"VOICE_RUNTIME", "CONVERSATION_STATE", "PRESENTATION"}
    elif uc == 12:
        required |= {"TASK_RUNTIME", "AGENT_FIXTURE", "PRESENTATION"}
    elif uc == 13:
        required |= {"TASK_RUNTIME", "AGENT_FIXTURE", "PRESENTATION"}
    elif uc == 14:
        required |= {"TASK_RUNTIME", "AGENT_FIXTURE", "PRESENTATION"}
    elif uc == 15:
        required |= {"VOICE_RUNTIME", "CONVERSATION_STATE", "TASK_RUNTIME", "PRESENTATION"}
    elif uc == 16:
        required |= {"POLICY_SERVICE", "PRESENTATION"}
    elif uc == 17:
        required |= {"MEMORY_SERVICE", "PRESENTATION"}
    elif uc == 18:
        required |= {"PRESENTATION"}
        patch = case["input"]["patch_key"]
        if patch == "missing_source":
            required.add("CONTEXT_FIXTURE")
        elif patch == "model_down":
            required.add("IR_MODEL")
        elif patch in {"agent_unavailable", "unknown_status", "partial"}:
            required |= {"TASK_RUNTIME", "AGENT_FIXTURE"}
        elif patch == "restart":
            required |= {"EXEC_HOST", "TASK_RUNTIME", "AGENT_FIXTURE"}
        else:
            raise ValueError(f"unmapped UC-18 patch: {patch}")
    else:
        raise ValueError(f"unmapped UC family: {uc}")

    # TC-14.3 exercises scoped pending approval/question identity even though UC-14
    # is primarily multi-Task continuity.
    if case["id"] == "TC-14.3":
        required.add("POLICY_SERVICE")

    return sorted(required)


def plan_case(case: dict[str, Any]) -> dict[str, Any]:
    required = responsibilities(case)
    component_status = {name: STATUS[name] for name in required}
    blockers = [
        name
        for name, status in component_status.items()
        if status in BLOCKING
    ]
    external = [
        name
        for name, status in component_status.items()
        if status in EXTERNAL
    ]
    fixture_only = [
        name
        for name, status in component_status.items()
        if status == "FIXTURE_ONLY"
    ]

    if blockers:
        readiness = "BLOCKED_RUNTIME_GAP"
    elif external:
        readiness = "EXECUTION_DEPENDENCY_PENDING"
    else:
        readiness = "STRUCTURAL_ADAPTER_READY"

    return {
        "case_id": case["id"],
        "uc": case["uc"],
        "patch_key": case["input"]["patch_key"],
        "primary_asrs": case["primary_asrs"],
        "responsibilities": required,
        "component_status": component_status,
        "runtime_blockers": blockers,
        "external_execution_dependencies": external,
        "fixture_only_boundaries": fixture_only,
        "adapter_readiness": readiness,
        "candidate_observation": "NOT_RUN",
        "score": None,
    }


def build_plan() -> dict[str, Any]:
    ensure_assets()
    cases = json.loads(CASES.read_text(encoding="utf-8"))
    plans = [plan_case(case) for case in cases]
    ids = [plan["case_id"] for plan in plans]
    if len(plans) != 94:
        raise ValueError(f"expected 94 canonical cases, found {len(plans)}")
    if len(set(ids)) != len(ids):
        raise ValueError("duplicate canonical case id")
    if any(not plan["responsibilities"] for plan in plans):
        raise ValueError("canonical case without an execution responsibility")

    counts: dict[str, int] = {}
    for plan in plans:
        counts[plan["adapter_readiness"]] = counts.get(plan["adapter_readiness"], 0) + 1
    component_counts: dict[str, int] = {}
    for plan in plans:
        for component in plan["responsibilities"]:
            component_counts[component] = component_counts.get(component, 0) + 1

    return {
        "schema_version": 1,
        "purpose": "PRE_MEASUREMENT_CANONICAL_ADAPTER_COVERAGE",
        "canonical_case_count": len(plans),
        "readiness_counts": dict(sorted(counts.items())),
        "component_case_counts": dict(sorted(component_counts.items())),
        "candidate_observations": "NOT_RUN",
        "scoring": "NOT_RUN",
        "cases": plans,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    plan = build_plan()
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(
            json.dumps(plan, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
    if args.check or not args.output:
        print(
            json.dumps(
                {
                    "status": "PASS",
                    "canonical_case_count": plan["canonical_case_count"],
                    "readiness_counts": plan["readiness_counts"],
                    "candidate_observations": "NOT_RUN",
                },
                ensure_ascii=False,
                indent=2,
            )
        )


if __name__ == "__main__":
    main()
