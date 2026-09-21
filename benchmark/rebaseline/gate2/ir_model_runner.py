"""IR-DP01 pre-measurement runner.

Dry-run prepares only Integrated and Grounding requests. Later staged requests cannot be
prepared without actual preceding model outputs. --execute is blocked unless a reviewed
measurement-freeze manifest and approval match the current source inventory. Even in
execute mode this script saves raw semantic evidence only; it never computes W-05, p95,
0-5 scores, or a winner.
"""
from __future__ import annotations

import argparse
import copy
import json
import subprocess
import sys
import time
import urllib.request
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
GENERATED = ROOT / "benchmark/rebaseline"
READINESS = GENERATED / "readiness"
sys.path.insert(0, str(READINESS))

from freeze import authorize_run  # noqa: E402
from ir_contract import merge_stages, prepare, validate_result  # noqa: E402

MAX_CONTEXT_ROUNDS = 2
MAX_CORRECTION_ROUNDS = 2
MAX_VALIDATION_REPAIRS = 2
DEFAULT_CASES = ["TC-04.2", "TC-06.2", "TC-09.3", "TC-09.4", "TC-14.5"]


class SemanticRunError(RuntimeError):
    pass


def ensure_assets() -> None:
    needed = [
        GENERATED / "cases.json",
        GENERATED / "fixtures/base.json",
        GENERATED / "fixtures/patches.json",
    ]
    if not all(path.exists() for path in needed):
        subprocess.run(
            ["python3", str(GENERATED / "build_assets.py")],
            cwd=ROOT,
            check=True,
        )


def deep_merge_mapping(base: dict[str, Any], override: dict[str, Any]) -> dict[str, Any]:
    result = copy.deepcopy(base)
    for key, value in override.items():
        if isinstance(value, dict) and isinstance(result.get(key), dict):
            result[key] = deep_merge_mapping(result[key], value)
        else:
            result[key] = copy.deepcopy(value)
    return result


def materialize_tasks(base: dict[str, Any], patch: dict[str, Any]) -> list[dict[str, Any]]:
    values = {task["id"]: copy.deepcopy(task) for task in base["tasks"]}
    for task in patch.get("task_additions", []):
        values[task["id"]] = copy.deepcopy(task)
    for task_id, override in patch.get("task_overrides", {}).items():
        if task_id in values:
            values[task_id].update(copy.deepcopy(override))
    return [
        {
            "task_id": task_id,
            "revision": int(task.get("revision", 1)),
            "state": task.get("state"),
            "run_id": task.get("run"),
            "goal": task.get("goal"),
        }
        for task_id, task in sorted(values.items())
    ]


def materialize_agents(base: dict[str, Any], patch: dict[str, Any]) -> list[dict[str, Any]]:
    if "agent_catalog" in patch:
        agents = copy.deepcopy(patch["agent_catalog"])
    else:
        agents = copy.deepcopy(base["agent_catalog"])
        overrides = patch.get("agent_overrides", {})
        for agent in agents:
            agent.update(copy.deepcopy(overrides.get(agent["id"], {})))
    return agents


def pending_interactions(base: dict[str, Any], patch: dict[str, Any]) -> list[dict[str, Any]]:
    values: list[dict[str, Any]] = []
    for item in base["policy"].get("pending_approvals", []):
        values.append(
            {
                "id": item["id"],
                "task_id": item.get("task"),
                "kind": "approval",
                "revision": item.get("revision"),
            }
        )
    for kind, key in (("question", "pending_questions"), ("approval", "pending_approvals")):
        for item in patch.get(key, []):
            values.append(
                {
                    "id": item["id"],
                    "task_id": item.get("task"),
                    "kind": kind,
                    "revision": item.get("revision"),
                }
            )
    unique: dict[str, dict[str, Any]] = {}
    for value in values:
        unique[value["id"]] = value
    return list(unique.values())


def source_catalog(sources: dict[str, Any]) -> list[dict[str, Any]]:
    catalog = []
    for source_id, value in sorted(sources.items()):
        catalog.append(
            {
                "source_id": source_id,
                "version": "fixture-v1",
                "kind": value.get("kind"),
                "title": value.get("title") or value.get("subject"),
                "content_materialized": False,
            }
        )
    return catalog


def build_case(
    case: dict[str, Any],
    base: dict[str, Any],
    patch: dict[str, Any],
) -> tuple[dict[str, Any], dict[str, Any], list[str]]:
    sources = deep_merge_mapping(base["sources"], patch.get("source_additions", {}))
    windows = copy.deepcopy(patch.get("windows", base["windows"]))
    conversation = copy.deepcopy(base["conversation"])
    conversation.extend(copy.deepcopy(patch.get("append_history", [])))
    agents = materialize_agents(base, patch)
    capabilities = sorted(
        {
            capability
            for agent in agents
            if agent.get("status", True)
            for capability in agent.get("capability", [])
        }
    )
    policy = deep_merge_mapping(base["policy"], patch.get("policy_overrides", {}))
    value = {
        "case_id": case["id"],
        "request_revision": 1,
        "original_request": case["input"]["utterance"],
        "interaction_events": copy.deepcopy(patch.get("events", [])),
        "visible_context": [
            {"kind": "source_catalog", "items": source_catalog(sources)},
            {"kind": "windows", "items": windows},
        ],
        "conversation": conversation,
        "task_views": materialize_tasks(base, patch),
        "pending_interactions": pending_interactions(base, patch),
        "capabilities": capabilities,
        "policy_constraints": [policy],
    }
    return value, sources, list(case["input"].get("scripted_followups", []))


def load_cases(case_ids: list[str]) -> list[tuple[dict[str, Any], dict[str, Any], list[str]]]:
    ensure_assets()
    cases = json.loads((GENERATED / "cases.json").read_text(encoding="utf-8"))
    base = json.loads((GENERATED / "fixtures/base.json").read_text(encoding="utf-8"))
    patches = json.loads((GENERATED / "fixtures/patches.json").read_text(encoding="utf-8"))
    by_id = {case["id"]: case for case in cases}
    result = []
    for case_id in case_ids:
        if case_id not in by_id:
            raise KeyError(f"unknown canonical test case: {case_id}")
        case = by_id[case_id]
        patch = patches[case["input"]["patch_key"]]
        result.append(build_case(case, base, patch))
    return result


def response_content(response: dict[str, Any]) -> dict[str, Any]:
    raw = response["choices"][0]["message"]["content"]
    if isinstance(raw, dict):
        return raw
    if not isinstance(raw, str):
        raise SemanticRunError("model response content is not JSON text/object")
    return json.loads(raw)


def post_json(url: str, body: dict[str, Any]) -> tuple[int, dict[str, Any]]:
    data = json.dumps(body, ensure_ascii=False).encode()
    request = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    start = time.perf_counter_ns()
    with urllib.request.urlopen(request, timeout=180) as response:
        parsed = json.loads(response.read())
    return time.perf_counter_ns() - start, parsed


def call_stage(
    stage: str,
    case: dict[str, Any],
    model: str,
    endpoint: str,
    *,
    prior: dict[str, dict[str, Any]] | None = None,
    controller_feedback: str | None = None,
    seed: int,
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    feedback = controller_feedback
    attempts: list[dict[str, Any]] = []
    for repair in range(MAX_VALIDATION_REPAIRS + 1):
        prepared = prepare(
            stage,
            case,
            model,
            prior,
            controller_feedback=feedback,
            seed=seed,
        )
        latency_ns, raw = post_json(endpoint, prepared["request"])
        try:
            value = response_content(raw)
            validate_result(stage, value, case, prior)
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as exc:
            attempts.append(
                {
                    "repair": repair,
                    "request_sha256": prepared["request_sha256"],
                    "latency_ns": latency_ns,
                    "raw": raw,
                    "validation": f"REJECTED: {exc}",
                }
            )
            if repair == MAX_VALIDATION_REPAIRS:
                raise SemanticRunError(
                    f"{stage}: validator repair budget exhausted: {exc}"
                ) from exc
            feedback = f"VIA validator rejected the previous output: {exc}"
            continue
        attempts.append(
            {
                "repair": repair,
                "request_sha256": prepared["request_sha256"],
                "latency_ns": latency_ns,
                "raw": raw,
                "validation": "ACCEPTED",
            }
        )
        return value, attempts
    raise AssertionError("unreachable")


def foreground_source_id(case: dict[str, Any]) -> str | None:
    windows = next(
        (
            item["items"]
            for item in case["visible_context"]
            if item.get("kind") == "windows"
        ),
        [],
    )
    foreground = [item for item in windows if item.get("foreground")]
    if len(foreground) == 1:
        return foreground[0].get("document")
    return None


def materialize_needed_context(
    case: dict[str, Any],
    sources: dict[str, Any],
    needs: list[dict[str, Any]],
) -> tuple[dict[str, Any], list[str]]:
    updated = copy.deepcopy(case)
    already = {
        item.get("source_id")
        for item in updated["visible_context"]
        if item.get("kind") == "materialized_source"
    }
    materialized: list[str] = []
    for need in needs:
        source_id = need.get("source_id") or foreground_source_id(updated)
        if not source_id or source_id not in sources:
            raise SemanticRunError(
                f"cannot deterministically materialize requested context: {need}"
            )
        if source_id in already:
            continue
        updated["visible_context"].append(
            {
                "kind": "materialized_source",
                "source_id": source_id,
                "version": "fixture-v1",
                "scope": need["scope"],
                "content": copy.deepcopy(sources[source_id]),
            }
        )
        already.add(source_id)
        materialized.append(source_id)
    if not materialized and needs:
        raise SemanticRunError("NEED_CONTEXT requested only already-materialized evidence")
    return updated, materialized


def apply_scripted_followup(case: dict[str, Any], text: str) -> dict[str, Any]:
    updated = copy.deepcopy(case)
    updated["request_revision"] += 1
    updated["interaction_events"].append(
        {
            "kind": "scripted_user_reply",
            "text": text,
            "request_revision": updated["request_revision"],
        }
    )
    updated["conversation"].append(
        {
            "turn": f"scripted-followup-{updated['request_revision']}",
            "input": text,
            "response": None,
            "refs": [],
        }
    )
    return updated


def run_integrated(
    original_case: dict[str, Any],
    sources: dict[str, Any],
    followups: list[str],
    model: str,
    endpoint: str,
    seed: int,
) -> dict[str, Any]:
    case = copy.deepcopy(original_case)
    followups = list(followups)
    context_rounds = 0
    calls: list[dict[str, Any]] = []
    while True:
        value, attempts = call_stage(
            "integrated", case, model, endpoint, seed=seed
        )
        calls.extend({"stage": "integrated", **item} for item in attempts)
        if value["status"] == "READY":
            return {"status": "READY", "decision": value, "calls": calls}
        if value["status"] == "NEED_CONTEXT":
            if context_rounds >= MAX_CONTEXT_ROUNDS:
                raise SemanticRunError("integrated context acquisition budget exhausted")
            case, materialized = materialize_needed_context(
                case, sources, value["needed_context"]
            )
            context_rounds += 1
            calls.append(
                {
                    "stage": "controller",
                    "action": "materialize_context",
                    "sources": materialized,
                }
            )
            continue
        if value["status"] == "CLARIFY" and followups:
            case = apply_scripted_followup(case, followups.pop(0))
            calls.append(
                {
                    "stage": "controller",
                    "action": "scripted_followup",
                    "request_revision": case["request_revision"],
                }
            )
            continue
        return {"status": value["status"], "decision": value, "calls": calls}


def run_staged(
    original_case: dict[str, Any],
    sources: dict[str, Any],
    followups: list[str],
    model: str,
    endpoint: str,
    seed: int,
) -> dict[str, Any]:
    case = copy.deepcopy(original_case)
    followups = list(followups)
    context_rounds = 0
    correction_rounds = 0
    calls: list[dict[str, Any]] = []
    grounding_feedback: str | None = None

    while True:
        grounding, attempts = call_stage(
            "grounding",
            case,
            model,
            endpoint,
            controller_feedback=grounding_feedback,
            seed=seed,
        )
        grounding_feedback = None
        calls.extend({"stage": "grounding", **item} for item in attempts)

        if grounding["status"] == "NEED_CONTEXT":
            if context_rounds >= MAX_CONTEXT_ROUNDS:
                raise SemanticRunError("staged context acquisition budget exhausted")
            case, materialized = materialize_needed_context(
                case, sources, grounding["needed_context"]
            )
            context_rounds += 1
            calls.append(
                {
                    "stage": "controller",
                    "action": "materialize_context",
                    "sources": materialized,
                }
            )
            continue
        if grounding["status"] == "CLARIFY" and followups:
            case = apply_scripted_followup(case, followups.pop(0))
            calls.append(
                {
                    "stage": "controller",
                    "action": "scripted_followup",
                    "request_revision": case["request_revision"],
                }
            )
            continue
        if grounding["status"] != "READY":
            return {"status": grounding["status"], "grounding": grounding, "calls": calls}

        association_feedback: str | None = None
        while True:
            association, attempts = call_stage(
                "association",
                case,
                model,
                endpoint,
                prior={"grounding": grounding},
                controller_feedback=association_feedback,
                seed=seed,
            )
            association_feedback = None
            calls.extend({"stage": "association", **item} for item in attempts)

            if association["status"] == "CORRECT_PRIOR_STAGE":
                if correction_rounds >= MAX_CORRECTION_ROUNDS:
                    raise SemanticRunError("staged correction budget exhausted")
                correction_rounds += 1
                grounding_feedback = association["correction_reason"] or (
                    "Task association requested grounding correction"
                )
                calls.append(
                    {
                        "stage": "controller",
                        "action": "restart_grounding",
                        "reason": grounding_feedback,
                    }
                )
                break
            if association["status"] == "CLARIFY" and followups:
                case = apply_scripted_followup(case, followups.pop(0))
                calls.append(
                    {
                        "stage": "controller",
                        "action": "scripted_followup",
                        "request_revision": case["request_revision"],
                    }
                )
                grounding_feedback = None
                break
            if association["status"] != "READY":
                return {
                    "status": association["status"],
                    "grounding": grounding,
                    "association": association,
                    "calls": calls,
                }

            handling_feedback: str | None = None
            while True:
                handling, attempts = call_stage(
                    "handling",
                    case,
                    model,
                    endpoint,
                    prior={"grounding": grounding, "association": association},
                    controller_feedback=handling_feedback,
                    seed=seed,
                )
                handling_feedback = None
                calls.extend({"stage": "handling", **item} for item in attempts)

                if handling["status"] == "CORRECT_PRIOR_STAGE":
                    if correction_rounds >= MAX_CORRECTION_ROUNDS:
                        raise SemanticRunError("staged correction budget exhausted")
                    correction_rounds += 1
                    target = handling["correction_stage"]
                    reason = handling["correction_reason"] or (
                        "Handling stage requested prior-stage correction"
                    )
                    if target == "grounding":
                        grounding_feedback = reason
                        calls.append(
                            {
                                "stage": "controller",
                                "action": "restart_grounding",
                                "reason": reason,
                            }
                        )
                        break
                    association_feedback = reason
                    calls.append(
                        {
                            "stage": "controller",
                            "action": "restart_association",
                            "reason": reason,
                        }
                    )
                    continue
                if handling["status"] == "CLARIFY" and followups:
                    case = apply_scripted_followup(case, followups.pop(0))
                    calls.append(
                        {
                            "stage": "controller",
                            "action": "scripted_followup",
                            "request_revision": case["request_revision"],
                        }
                    )
                    grounding_feedback = None
                    break
                if handling["status"] != "READY":
                    return {
                        "status": handling["status"],
                        "grounding": grounding,
                        "association": association,
                        "handling": handling,
                        "calls": calls,
                    }
                final = merge_stages(case, grounding, association, handling)
                return {
                    "status": "READY",
                    "decision": final,
                    "grounding": grounding,
                    "association": association,
                    "handling": handling,
                    "calls": calls,
                }

            if grounding_feedback is not None:
                break
            if handling_feedback is not None:
                continue
            # A scripted follow-up or grounding correction restarts from Stage 1.
            if calls and calls[-1].get("action") in {
                "scripted_followup",
                "restart_grounding",
            }:
                break
        # Re-enter Stage 1 after a correction/follow-up.
        continue


def dry_run(case_ids: list[str], model: str, seed: int) -> dict[str, Any]:
    rows = load_cases(case_ids)
    output: dict[str, Any] = {
        "status": "DRY_RUN_MODEL_NOT_CALLED",
        "scoring": "NOT_RUN",
        "model": model,
        "seed": seed,
        "cases": [],
    }
    for case, _sources, _followups in rows:
        integrated = prepare("integrated", case, model, seed=seed)
        grounding = prepare("grounding", case, model, seed=seed)
        output["cases"].append(
            {
                "case_id": case["case_id"],
                "integrated": integrated,
                "staged": {
                    "grounding": grounding,
                    "association": "REQUIRES_ACTUAL_GROUNDING_OUTPUT",
                    "handling": "REQUIRES_ACTUAL_GROUNDING_AND_ASSOCIATION_OUTPUT",
                },
            }
        )
    return output


def execute(
    case_ids: list[str],
    model: str,
    seed: int,
    server: str,
) -> dict[str, Any]:
    endpoint = server.rstrip("/") + "/v1/chat/completions"
    output: dict[str, Any] = {
        "status": "MODEL_CALLS_EXECUTED_RAW_ONLY",
        "scoring": "NOT_RUN",
        "model": model,
        "seed": seed,
        "cases": [],
    }
    for case, sources, followups in load_cases(case_ids):
        output["cases"].append(
            {
                "case_id": case["case_id"],
                "integrated": run_integrated(
                    case, sources, followups, model, endpoint, seed
                ),
                "staged": run_staged(
                    case, sources, followups, model, endpoint, seed
                ),
            }
        )
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--server", default="http://127.0.0.1:8080")
    parser.add_argument("--model", default="Qwen3-8B-Q4_K_M")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--case", action="append", dest="cases")
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--approval", type=Path)
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / "results/gate2-local/ir-requests.json",
    )
    args = parser.parse_args()
    case_ids = args.cases or DEFAULT_CASES

    if args.execute:
        if args.manifest is None or args.approval is None:
            parser.error("--execute requires --manifest and --approval from Measurement Freeze Review")
        manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
        approval = json.loads(args.approval.read_text(encoding="utf-8"))
        authorize_run(manifest, approval, ROOT, real_model=True)
        result = execute(case_ids, args.model, args.seed, args.server)
        result["freeze_fingerprint"] = manifest["fingerprint"]
    else:
        result = dry_run(case_ids, args.model, args.seed)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(result, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(args.output)


if __name__ == "__main__":
    main()
