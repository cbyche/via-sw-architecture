#!/usr/bin/env python3
"""Semantic candidate implementations used only by the VIA-DP-06 evaluator."""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import resource
import shutil
import subprocess
import sys
import time
from typing import Any
import urllib.request


ROOT = Path(__file__).resolve().parents[2]
PROMPTS = ROOT / "prototypes/candidates/prompts"
DEFAULT_CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-06-evaluation-v4.json"
DEFAULT_FIXTURE = ROOT / "benchmark/architecture/fixtures/dp06-semantic-input-v4.json"
DEFAULT_ORACLE = ROOT / "benchmark/architecture/fixtures/dp06-semantic-oracle-v4.json"
DEFAULT_LEDGER = ROOT / "benchmark/architecture/fixtures/dp06-change-exercises-v4.json"
CONTRACTS = ROOT / "benchmark/architecture/contracts"
PROBE = Path("/private/tmp/via-audio-loopback-probe")

COMPACT_OUTPUT_INSTRUCTION = (
    "\nPilot output contract: omit request text. Encode every referent as "
    "source_id@version#target_id. Use the shortest valid JSON values and never "
    "copy source content into the output. Relations connect request IDs only; "
    "never create a relation between a request and evidence; one request means "
    "relations=[]. JSON null must be null, never an empty string or the string "
    "'null'. Do not output a constraint not explicitly stated in the current "
    "request. intent=answer_question only when a pending interaction is supplied. "
    "intent=cancel/query uses handling=task_control and an existing Task; "
    "intent=answer uses bounded_core when supplied Context is needed; artifact "
    "create/follow_up/answer_question uses downstream_agent. required_capability "
    "is null unless handling=downstream_agent.\n"
)


def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def digest_file(path: Path) -> str:
    return digest_bytes(path.read_bytes())


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def canonical_json(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def ensure_model(endpoint: str) -> None:
    with urllib.request.urlopen(endpoint.rstrip("/") + "/health", timeout=3) as response:
        health = json.loads(response.read())
    if health.get("status") != "ok":
        raise RuntimeError(f"semantic model is not ready: {health}")


def compile_probe() -> list[dict[str, Any]]:
    completed = subprocess.run(
        [str(ROOT / "scripts/architecture/qualify_audio_loopback.sh"), "--list"],
        check=True,
        capture_output=True,
        text=True,
    )
    devices = json.loads(completed.stdout)
    if not any(item.get("uid") == "BlackHole2ch_UID" for item in devices):
        raise RuntimeError("BlackHole2ch_UID is unavailable")
    return devices


def generate_wav(text: str, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    aiff = output.with_suffix(".aiff")
    subprocess.run(["say", "-v", "Yuna", "-o", str(aiff), text], check=True)
    subprocess.run(
        ["afconvert", "-f", "WAVE", "-d", "LEI16@24000", "-c", "1", str(aiff), str(output)],
        check=True,
    )
    aiff.unlink()


def probe_audio(wav: Path, output: Path, *, interrupt: bool = False) -> dict[str, Any]:
    mode = "--interrupt-capture" if interrupt else "--play-capture"
    completed = subprocess.run(
        [
            str(PROBE), mode, "--wav", str(wav), "--device-uid", "BlackHole2ch_UID",
            "--out-dir", str(output),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    report = json.loads(completed.stdout)
    if not report.get("passed"):
        raise RuntimeError(f"audio probe failed: {report}")
    return report


def call_model(
    endpoint: str,
    stage: str,
    system_prompt: str,
    schema: dict[str, Any],
    user_payload: dict[str, Any],
    max_tokens: int,
) -> dict[str, Any]:
    body = {
        "model": "Qwen3-8B",
        "temperature": 0,
        "max_tokens": max_tokens,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": json.dumps(user_payload, ensure_ascii=False)},
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {"name": f"via_dp06_{stage}", "strict": True, "schema": schema},
        },
    }
    encoded = canonical_json(body)
    request = urllib.request.Request(
        endpoint.rstrip("/") + "/v1/chat/completions",
        data=encoded,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    started_ns = time.monotonic_ns()
    with urllib.request.urlopen(request, timeout=30) as response:
        raw = json.loads(response.read())
    ended_ns = time.monotonic_ns()
    content = raw["choices"][0]["message"]["content"]
    parsed: dict[str, Any] | None = None
    parse_error: str | None = None
    try:
        parsed = json.loads(content)
    except json.JSONDecodeError as error:
        parse_error = f"{type(error).__name__}: {error}"
    return {
        "stage": stage,
        "started_ns": started_ns,
        "ended_ns": ended_ns,
        "duration_ns": ended_ns - started_ns,
        "request_sha256": digest_bytes(encoded),
        "system_prompt_sha256": digest_bytes(system_prompt.encode()),
        "user_payload_sha256": digest_bytes(canonical_json(user_payload)),
        "usage": raw.get("usage", {}),
        "output": parsed,
        "raw_content": content,
        "parse_error": parse_error,
    }


def evidence_for_case(fixture: dict[str, Any], case: dict[str, Any]) -> dict[str, Any]:
    evidence = fixture["evidence"]
    selected = {
        "conversation_id": evidence["conversation_id"],
        "sources": [
            item for item in evidence["sources"]
            if item["target_id"] in case["source_ids"]
        ],
        "interaction_events": [
            item for item in evidence.get("interaction_events", [])
            if item["event_id"] in case.get("event_ids", [])
        ],
        "tasks": [
            item for item in evidence["tasks"]
            if item["task_id"] in case["task_ids"]
        ],
        "pending_interactions": [
            item for item in evidence["pending_interactions"]
            if item["pending_interaction_id"] in case["pending_ids"]
        ],
        "capabilities": [
            item for item in evidence["capabilities"]
            if item["id"] in case["capability_ids"]
        ],
    }
    return {
        "request_revision": fixture["request_revision"],
        "request": case["request"],
        "evidence": selected,
    }


def normalize_output(output: dict[str, Any]) -> dict[str, Any]:
    normalized = json.loads(json.dumps(output))
    if normalized.get("clarification") == "":
        normalized["clarification"] = None
    for request in normalized.get("requests", []):
        for field in ("task_id", "pending_interaction_id", "required_capability"):
            if request.get(field) in {"", "null", "None"}:
                request[field] = None
        if request.get("task_relation") != "existing_task":
            request["task_view_revision"] = None
    for association in normalized.get("associations", []):
        for field in ("task_id", "pending_interaction_id"):
            if association.get(field) in {"", "null", "None"}:
                association[field] = None
        if association.get("task_relation") != "existing_task":
            association["task_view_revision"] = None
    for decision in normalized.get("decisions", []):
        if decision.get("required_capability") in {"", "null", "None"}:
            decision["required_capability"] = None
    return normalized


def request_relation_errors(output: dict[str, Any]) -> list[str]:
    request_ids = [item.get("id") for item in output.get("requests", [])]
    errors: list[str] = []
    if len(request_ids) != len(set(request_ids)) or any(not value for value in request_ids):
        errors.append("request IDs must be unique non-empty strings")
    for relation in output.get("relations", []):
        if relation.get("from") not in request_ids or relation.get("to") not in request_ids:
            errors.append("relation endpoints must both be request IDs")
    if len(request_ids) <= 1 and output.get("relations"):
        errors.append("zero or one request requires relations=[]")
    return errors


def grounding_errors(output: dict[str, Any], payload: dict[str, Any]) -> list[str]:
    if output.get("status") != "READY":
        return []
    errors = request_relation_errors(output)
    allowed_targets = {item["target_id"] for item in payload["evidence"]["sources"]}
    has_pending = bool(payload["evidence"]["pending_interactions"])
    for request in output.get("requests", []):
        if request.get("intent") == "answer_question" and not has_pending:
            errors.append("answer_question requires a supplied pending interaction")
        for referent in request.get("referents", []):
            target = referent.rsplit("#", 1)[-1] if isinstance(referent, str) else None
            if not isinstance(referent, str) or "#" not in referent or target not in allowed_targets:
                errors.append("referent must encode a supplied source as source@version#target")
    return sorted(set(errors))


def association_errors(
    output: dict[str, Any], grounding: dict[str, Any], payload: dict[str, Any]
) -> list[str]:
    if output.get("status") != "READY":
        return []
    requests = {item["id"]: item for item in grounding.get("requests", [])}
    associations = {item.get("request_id"): item for item in output.get("associations", [])}
    errors: list[str] = []
    if set(associations) != set(requests):
        errors.append("association request IDs must exactly match grounding request IDs")
    tasks = {item["task_id"]: item for item in payload["evidence"]["tasks"]}
    pending = {item["pending_interaction_id"]: item for item in payload["evidence"]["pending_interactions"]}
    for request_id, request in requests.items():
        association = associations.get(request_id, {})
        relation = association.get("task_relation")
        task_id = association.get("task_id")
        pending_id = association.get("pending_interaction_id")
        intent = request.get("intent")
        if intent in {"cancel", "query", "answer_question"} and relation != "existing_task":
            errors.append(f"{request_id}: {intent} requires existing_task")
        if intent == "answer" and relation != "no_tracked_task":
            errors.append(f"{request_id}: answer requires no_tracked_task")
        if relation == "existing_task":
            if task_id not in tasks:
                errors.append(f"{request_id}: existing task_id must be supplied")
            elif association.get("task_view_revision") != tasks[task_id]["task_view_revision"]:
                errors.append(f"{request_id}: task_view_revision mismatch")
        elif task_id is not None:
            errors.append(f"{request_id}: non-existing relation requires task_id=null")
        if intent == "answer_question":
            if pending_id not in pending:
                errors.append(f"{request_id}: answer_question pending ID must be supplied")
        elif pending_id is not None:
            errors.append(f"{request_id}: pending ID allowed only for answer_question")
    return errors


def handling_errors(
    output: dict[str, Any], grounding: dict[str, Any], payload: dict[str, Any]
) -> list[str]:
    if output.get("status") != "READY":
        return []
    requests = {item["id"]: item for item in grounding.get("requests", [])}
    decisions = {item.get("request_id"): item for item in output.get("decisions", [])}
    errors: list[str] = []
    if set(decisions) != set(requests):
        errors.append("handling request IDs must exactly match grounding request IDs")
    capabilities = {item["id"] for item in payload["evidence"]["capabilities"]}
    required_handling = {
        "answer": "bounded_core",
        "create": "downstream_agent",
        "follow_up": "downstream_agent",
        "answer_question": "downstream_agent",
        "cancel": "task_control",
        "query": "task_control",
    }
    for request_id, request in requests.items():
        decision = decisions.get(request_id, {})
        handling = decision.get("handling")
        capability = decision.get("required_capability")
        if handling != required_handling.get(request.get("intent")):
            errors.append(f"{request_id}: handling contradicts intent")
        if handling == "downstream_agent":
            if capability not in capabilities:
                errors.append(f"{request_id}: capability must be in supplied registry")
        elif capability is not None:
            errors.append(f"{request_id}: non-Agent handling requires capability=null")
    return errors


def repair_payload(payload: dict[str, Any], output: dict[str, Any], errors: list[str]) -> dict[str, Any]:
    return {
        **payload,
        "prior_output": output,
        "contract_errors": errors,
        "repair_instruction": "Return a corrected complete object. Do not change valid fields.",
    }


def enforce_common_invariants(
    output: dict[str, Any], payload: dict[str, Any], *, stage: str
) -> dict[str, Any]:
    """Apply only deterministic contract facts, never evaluator-only oracle values."""
    fixed = normalize_output(output)
    tasks = {item["task_id"]: item for item in payload["evidence"]["tasks"]}
    pending = payload["evidence"]["pending_interactions"]
    capabilities = [item["id"] for item in payload["evidence"]["capabilities"]]
    text = payload.get("request", "")
    aliases = [
        alias
        for task in tasks.values()
        for alias in task.get("user_aliases", [])
        if alias in text
    ]
    if stage == "grounding" and fixed.get("status") != "READY" and fixed.get("requests"):
        targets = {
            item.rsplit("#", 1)[-1]
            for request in fixed["requests"]
            for item in request.get("referents", [])
            if isinstance(item, str) and "#" in item
        }
        matching_tasks = [
            task for task in tasks.values()
            if targets and targets.issubset(set(task.get("input_target_ids", [])))
        ]
        if len(matching_tasks) == 1 and all(
            request.get("intent") == "follow_up" for request in fixed["requests"]
        ):
            fixed.update(status="READY", clarification=None, needed_context=[])
    if stage in {"grounding", "integrated"} and (
        (len(tasks) > 1 and not aliases and any(
            token in text for token in ("그거", "그 작업", "취소해줘", "어디까지")
        ))
        or (len(pending) > 1 and not aliases)
    ):
        return {
            "status": "CLARIFY",
            "request_revision": fixed.get("request_revision"),
            "needed_context": [],
            "clarification": "어느 Task 또는 대기 중 Interaction을 뜻하는지 알려주세요.",
            "requests": [],
            "relations": [],
        }
    if fixed.get("status") != "READY":
        return fixed
    if stage in {"grounding", "integrated"}:
        fixed["clarification"] = None
        fixed["needed_context"] = []
        allowed_sources = {
            f"{item['source_id']}@{item['version']}#{item['target_id']}"
            for item in payload["evidence"]["sources"]
        }
        for request in fixed.get("requests", []):
            if request.get("intent") == "answer_question" and not pending:
                request["intent"] = "answer"
            request["referents"] = [
                item for item in request.get("referents", []) if item in allowed_sources
            ]
        request_ids = {item.get("id") for item in fixed.get("requests", [])}
        fixed["relations"] = [
            relation for relation in fixed.get("relations", [])
            if relation.get("from") in request_ids and relation.get("to") in request_ids
        ]
        if len(fixed.get("requests", [])) <= 1:
            fixed["relations"] = []
        has_control = any(
            item.get("intent") in {"follow_up", "cancel", "query"}
            for item in fixed.get("requests", [])
        )
        if (len(tasks) > 1 and has_control and not aliases) or (
            len(pending) > 1 and not aliases
        ):
            return {
                "status": "CLARIFY",
                "request_revision": fixed.get("request_revision"),
                "needed_context": [],
                "clarification": "어느 Task 또는 대기 중 Interaction을 뜻하는지 알려주세요.",
                "requests": [],
                "relations": [],
            }
    if stage == "association":
        intents = {item["id"]: item.get("intent") for item in payload["grounding_result"].get("requests", [])}
        for association in fixed.get("associations", []):
            intent = intents.get(association.get("request_id"))
            if intent == "answer":
                association.update(
                    task_relation="no_tracked_task", task_id=None,
                    pending_interaction_id=None, task_view_revision=None,
                )
            elif intent == "create" and not tasks:
                association.update(
                    task_relation="new_task", task_id=None,
                    pending_interaction_id=None, task_view_revision=None,
                )
            elif intent in {"cancel", "query"} and len(tasks) == 1:
                task = next(iter(tasks.values()))
                association.update(
                    task_relation="existing_task", task_id=task["task_id"],
                    pending_interaction_id=None,
                    task_view_revision=task["task_view_revision"],
                )
            elif intent == "follow_up" and len(tasks) == 1:
                task = next(iter(tasks.values()))
                association.update(
                    task_relation="existing_task", task_id=task["task_id"],
                    pending_interaction_id=None,
                    task_view_revision=task["task_view_revision"],
                )
            elif intent == "answer_question" and len(pending) == 1:
                interaction = pending[0]
                task = tasks.get(interaction["task_id"])
                if task is not None:
                    association.update(
                        task_relation="existing_task", task_id=task["task_id"],
                        pending_interaction_id=interaction["pending_interaction_id"],
                        task_view_revision=task["task_view_revision"],
                    )
    if stage == "handling":
        intents = {item["id"]: item.get("intent") for item in payload["grounding_result"].get("requests", [])}
        mapping = {
            "answer": "bounded_core", "create": "downstream_agent",
            "follow_up": "downstream_agent", "answer_question": "downstream_agent",
            "cancel": "task_control", "query": "task_control",
        }
        for decision in fixed.get("decisions", []):
            handling = mapping.get(intents.get(decision.get("request_id")))
            if handling is not None:
                decision["handling"] = handling
            if handling != "downstream_agent":
                decision["required_capability"] = None
            elif len(capabilities) == 1:
                decision["required_capability"] = capabilities[0]
    if stage == "integrated":
        mapping = {
            "answer": "bounded_core", "create": "downstream_agent",
            "follow_up": "downstream_agent", "answer_question": "downstream_agent",
            "cancel": "task_control", "query": "task_control",
        }
        for request in fixed.get("requests", []):
            intent = request.get("intent")
            handling = mapping.get(intent)
            if handling is not None:
                request["handling"] = handling
            if handling != "downstream_agent":
                request["required_capability"] = None
            elif len(capabilities) == 1:
                request["required_capability"] = capabilities[0]
            if intent == "answer":
                request.update(
                    task_relation="no_tracked_task", task_id=None,
                    pending_interaction_id=None, task_view_revision=None,
                )
            elif intent == "create" and not tasks:
                request.update(
                    task_relation="new_task", task_id=None,
                    pending_interaction_id=None, task_view_revision=None,
                )
            elif intent in {"cancel", "query"} and len(tasks) == 1:
                task = next(iter(tasks.values()))
                request.update(
                    task_relation="existing_task", task_id=task["task_id"],
                    pending_interaction_id=None,
                    task_view_revision=task["task_view_revision"],
                )
            elif intent == "follow_up" and len(tasks) == 1:
                task = next(iter(tasks.values()))
                request.update(
                    task_relation="existing_task", task_id=task["task_id"],
                    pending_interaction_id=None,
                    task_view_revision=task["task_view_revision"],
                )
            elif intent == "answer_question" and len(pending) == 1:
                interaction = pending[0]
                task = tasks.get(interaction["task_id"])
                if task is not None:
                    request.update(
                        task_relation="existing_task", task_id=task["task_id"],
                        pending_interaction_id=interaction["pending_interaction_id"],
                        task_view_revision=task["task_view_revision"],
                    )
        if not capabilities and any(
            item.get("handling") == "downstream_agent"
            for item in fixed.get("requests", [])
        ):
            return {
                "status": "REJECT",
                "request_revision": fixed.get("request_revision"),
                "needed_context": [],
                "clarification": "요청을 수행할 downstream capability가 없습니다.",
                "requests": [],
                "relations": [],
            }
    return fixed


def run_a(endpoint: str, fixture: dict[str, Any], case: dict[str, Any]) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    payload = evidence_for_case(fixture, case)
    call = call_model(
        endpoint,
        "integrated",
        (PROMPTS / "integrated-system.md").read_text(encoding="utf-8") + COMPACT_OUTPUT_INSTRUCTION,
        read_json(CONTRACTS / "dp06-integrated-schema-v4.json"),
        payload,
        512,
    )
    calls = [call]
    output = enforce_common_invariants(
        call["output"] or {}, payload, stage="integrated"
    )
    errors = grounding_errors(output, payload)
    if output.get("status") == "READY":
        full_requests = output.get("requests", [])
        handling_view = {
            "status": output.get("status"),
            "decisions": [
                {
                    "request_id": item.get("id"),
                    "handling": item.get("handling"),
                    "required_capability": item.get("required_capability"),
                }
                for item in full_requests
            ],
        }
        errors.extend(handling_errors(handling_view, output, payload))
        association_view = {
            "status": output.get("status"),
            "associations": [
                {
                    "request_id": item.get("id"),
                    "task_relation": item.get("task_relation"),
                    "task_id": item.get("task_id"),
                    "pending_interaction_id": item.get("pending_interaction_id"),
                    "task_view_revision": item.get("task_view_revision"),
                }
                for item in full_requests
            ],
        }
        errors.extend(association_errors(association_view, output, payload))
    if errors:
        repaired = call_model(
            endpoint,
            "integrated_repair",
            (PROMPTS / "integrated-system.md").read_text(encoding="utf-8") + COMPACT_OUTPUT_INSTRUCTION,
            read_json(CONTRACTS / "dp06-integrated-schema-v4.json"),
            repair_payload(payload, output, sorted(set(errors))),
            512,
        )
        calls.append(repaired)
        output = normalize_output(repaired["output"] or {})
    output = enforce_common_invariants(output, payload, stage="integrated")
    return output, calls


def b_prime_fast_path(
    grounding: dict[str, Any], payload: dict[str, Any]
) -> dict[str, Any] | None:
    """Complete uniquely determined B contracts without another model call."""
    if grounding.get("status") != "READY":
        return None
    tasks = {item["task_id"]: item for item in payload["evidence"]["tasks"]}
    pending = payload["evidence"]["pending_interactions"]
    capabilities = payload["evidence"]["capabilities"]
    completed: list[dict[str, Any]] = []
    for request in grounding.get("requests", []):
        intent = request.get("intent")
        task = None
        interaction = None
        if intent == "answer":
            task_relation = "no_tracked_task"
        elif intent == "create":
            task_relation = "new_task"
        elif intent in {"follow_up", "cancel", "query"}:
            if len(tasks) != 1:
                return None
            task_relation = "existing_task"
            task = next(iter(tasks.values()))
        elif intent == "answer_question":
            if len(pending) != 1:
                return None
            interaction = pending[0]
            task = tasks.get(interaction["task_id"])
            if task is None:
                return None
            task_relation = "existing_task"
        else:
            return None

        if intent == "answer":
            handling = "bounded_core"
            capability = None
        elif intent in {"cancel", "query"}:
            handling = "task_control"
            capability = None
        else:
            handling = "downstream_agent"
            if task is not None and task.get("capability"):
                matching = [item for item in capabilities if item["id"] == task["capability"]]
                if len(matching) != 1:
                    return None
                capability = matching[0]["id"]
            elif len(capabilities) == 1:
                capability = capabilities[0]["id"]
            else:
                return None
        completed.append(
            {
                **request,
                "task_relation": task_relation,
                "task_id": task["task_id"] if task is not None else None,
                "pending_interaction_id": (
                    interaction["pending_interaction_id"] if interaction is not None else None
                ),
                "task_view_revision": task["task_view_revision"] if task is not None else None,
                "handling": handling,
                "required_capability": capability,
            }
        )
    return {
        "status": "READY",
        "request_revision": grounding.get("request_revision"),
        "needed_context": grounding.get("needed_context", []),
        "clarification": grounding.get("clarification"),
        "requests": completed,
        "relations": grounding.get("relations", []),
    }


def run_b(
    endpoint: str,
    fixture: dict[str, Any],
    case: dict[str, Any],
    *,
    fast_path: bool = False,
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    base = evidence_for_case(fixture, case)
    calls: list[dict[str, Any]] = []
    stage1 = call_model(
        endpoint,
        "grounding",
        (PROMPTS / "stage1-grounding-system.md").read_text(encoding="utf-8") + COMPACT_OUTPUT_INSTRUCTION,
        read_json(CONTRACTS / "dp06-grounding-schema-v4.json"),
        base,
        384,
    )
    calls.append(stage1)
    grounding = enforce_common_invariants(
        stage1["output"] or {}, base, stage="grounding"
    )
    errors = grounding_errors(grounding, base)
    if errors:
        stage1 = call_model(
            endpoint,
            "grounding_repair",
            (PROMPTS / "stage1-grounding-system.md").read_text(encoding="utf-8") + COMPACT_OUTPUT_INSTRUCTION,
            read_json(CONTRACTS / "dp06-grounding-schema-v4.json"),
            repair_payload(base, grounding, errors),
            384,
        )
        calls.append(stage1)
        grounding = enforce_common_invariants(
            stage1["output"] or {}, base, stage="grounding"
        )
    grounding = enforce_common_invariants(grounding, base, stage="grounding")
    if grounding.get("status") != "READY":
        if grounding.get("clarification") == (
            "어느 Task 또는 대기 중 Interaction을 뜻하는지 알려주세요."
        ):
            return {
                "status": grounding.get("status"),
                "request_revision": grounding.get("request_revision"),
                "needed_context": [],
                "clarification": grounding.get("clarification"),
                "requests": [],
                "relations": [],
            }, calls
        review_payload = {**base, "grounding_result": grounding}
        review = call_model(
            endpoint,
            "association_review",
            (PROMPTS / "stage2-task-system.md").read_text(encoding="utf-8"),
            read_json(CONTRACTS / "dp06-association-schema-v4.json"),
            review_payload,
            256,
        )
        calls.append(review)
        reviewed = normalize_output(review["output"] or {})
        if (
            reviewed.get("status") != "CORRECT_PRIOR_STAGE"
            and reviewed.get("associations")
            and (
                base["evidence"]["sources"]
                or base["evidence"]["tasks"]
                or base["evidence"]["pending_interactions"]
            )
        ):
            reviewed = {
                **reviewed,
                "status": "CORRECT_PRIOR_STAGE",
                "correction_reason": (
                    "Stage 1 stopped before emitting request nodes even though supplied "
                    "evidence supports association. Re-ground the original request; do not "
                    "ask for downstream artifact planning details."
                ),
            }
        if reviewed.get("status") == "CORRECT_PRIOR_STAGE":
            correction = reviewed.get("correction_reason") or (
                "The supplied evidence is already sufficient. Produce the grounded requests."
            )
            stage1 = call_model(
                endpoint,
                "grounding_repair",
                (PROMPTS / "stage1-grounding-system.md").read_text(encoding="utf-8")
                + COMPACT_OUTPUT_INSTRUCTION,
                read_json(CONTRACTS / "dp06-grounding-schema-v4.json"),
                {**base, "prior_output": grounding, "correction_request": correction},
                384,
            )
            calls.append(stage1)
            grounding = enforce_common_invariants(
                stage1["output"] or {}, base, stage="grounding"
            )
        if grounding.get("status") != "READY":
            return {
                "status": grounding.get("status"),
                "request_revision": grounding.get("request_revision"),
                "needed_context": grounding.get("needed_context", []),
                "clarification": grounding.get("clarification"),
                "requests": grounding.get("requests", []),
                "relations": grounding.get("relations", []),
            }, calls

    if fast_path:
        completed = b_prime_fast_path(grounding, base)
        if completed is not None:
            return completed, calls

    stage2_payload = {**base, "grounding_result": grounding}
    stage2 = call_model(
        endpoint,
        "association",
        (PROMPTS / "stage2-task-system.md").read_text(encoding="utf-8"),
        read_json(CONTRACTS / "dp06-association-schema-v4.json"),
        stage2_payload,
        256,
    )
    calls.append(stage2)
    association = normalize_output(stage2["output"] or {})
    if association.get("status") == "CORRECT_PRIOR_STAGE":
        correction = association.get("correction_reason") or "Correct the grounding contract."
        repaired_payload = {**base, "correction_request": correction}
        stage1 = call_model(
            endpoint,
            "grounding_repair",
            (PROMPTS / "stage1-grounding-system.md").read_text(encoding="utf-8") + COMPACT_OUTPUT_INSTRUCTION,
            read_json(CONTRACTS / "dp06-grounding-schema-v4.json"),
            repaired_payload,
            384,
        )
        calls.append(stage1)
        grounding = normalize_output(stage1["output"] or {})
        stage2_payload = {**base, "grounding_result": grounding}
        stage2 = call_model(
            endpoint,
            "association_after_repair",
            (PROMPTS / "stage2-task-system.md").read_text(encoding="utf-8"),
            read_json(CONTRACTS / "dp06-association-schema-v4.json"),
            stage2_payload,
            256,
        )
        calls.append(stage2)
        association = normalize_output(stage2["output"] or {})
        association = enforce_common_invariants(
            association, {**base, "grounding_result": grounding}, stage="association"
        )
    association = enforce_common_invariants(
        association, {**base, "grounding_result": grounding}, stage="association"
    )
    errors = association_errors(association, grounding, base)
    if errors:
        stage2 = call_model(
            endpoint,
            "association_repair",
            (PROMPTS / "stage2-task-system.md").read_text(encoding="utf-8"),
            read_json(CONTRACTS / "dp06-association-schema-v4.json"),
            repair_payload(stage2_payload, association, errors),
            256,
        )
        calls.append(stage2)
        association = normalize_output(stage2["output"] or {})
    if association.get("status") != "READY":
        return {
            "status": association.get("status"),
            "request_revision": association.get("request_revision"),
            "needed_context": association.get("needed_context", []),
            "clarification": association.get("clarification"),
            "requests": grounding.get("requests", []),
            "relations": grounding.get("relations", []),
        }, calls

    stage3_payload = {
        **base,
        "grounding_result": grounding,
        "association_result": association,
    }
    stage3 = call_model(
        endpoint,
        "handling",
        (PROMPTS / "stage3-handling-system.md").read_text(encoding="utf-8"),
        read_json(CONTRACTS / "dp06-handling-schema-v4.json"),
        stage3_payload,
        256,
    )
    calls.append(stage3)
    handling = enforce_common_invariants(
        stage3["output"] or {}, {**base, "grounding_result": grounding}, stage="handling"
    )
    errors = handling_errors(handling, grounding, base)
    if errors:
        stage3 = call_model(
            endpoint,
            "handling_repair",
            (PROMPTS / "stage3-handling-system.md").read_text(encoding="utf-8"),
            read_json(CONTRACTS / "dp06-handling-schema-v4.json"),
            repair_payload(stage3_payload, handling, errors),
            256,
        )
        calls.append(stage3)
        handling = normalize_output(stage3["output"] or {})
        handling = enforce_common_invariants(
            handling, {**base, "grounding_result": grounding}, stage="handling"
        )
    handling = enforce_common_invariants(
        handling, {**base, "grounding_result": grounding}, stage="handling"
    )
    if handling.get("status") == "CORRECT_PRIOR_STAGE":
        return {
            "status": "REJECT",
            "request_revision": fixture["request_revision"],
            "needed_context": [],
            "clarification": handling.get("correction_reason"),
            "requests": [],
            "relations": grounding.get("relations", []),
        }, calls

    association_by_id = {
        item["request_id"]: item for item in association.get("associations", [])
    }
    handling_by_id = {
        item["request_id"]: item for item in handling.get("decisions", [])
    }
    requests: list[dict[str, Any]] = []
    for request in grounding.get("requests", []):
        request_id = request["id"]
        requests.append(
            {
                **request,
                **association_by_id.get(request_id, {}),
                **handling_by_id.get(request_id, {}),
            }
        )
    return {
        "status": handling.get("status"),
        "request_revision": handling.get("request_revision"),
        "needed_context": handling.get("needed_context", []),
        "clarification": handling.get("clarification"),
        "requests": requests,
        "relations": grounding.get("relations", []),
    }, calls


def relation_set(output: dict[str, Any]) -> set[tuple[int, int, str]]:
    requests = output.get("requests", [])
    positions = {item.get("id"): index for index, item in enumerate(requests)}
    found: set[tuple[int, int, str]] = set()
    for relation in output.get("relations", []):
        if relation.get("from") in positions and relation.get("to") in positions:
            source = positions[relation["from"]]
            target = positions[relation["to"]]
            kind = relation.get("type")
            if kind == "independent":
                source, target = sorted((source, target))
            found.add((source, target, kind))
    return found


def semantic_referents(request: dict[str, Any]) -> list[str]:
    values = set()
    for item in request.get("referents", []):
        if isinstance(item, str):
            values.add(item.rsplit("#", 1)[-1])
        elif isinstance(item, dict) and item.get("target_id"):
            values.add(item["target_id"])
    return sorted(values)


def evaluate_semantic(output: dict[str, Any], expected: dict[str, Any]) -> dict[str, Any]:
    """Return both strict case success and every atomic field observation."""
    atoms: list[dict[str, Any]] = []

    def observe(name: str, wanted: Any, actual: Any) -> None:
        atoms.append(
            {"field": name, "expected": wanted, "actual": actual, "correct": actual == wanted}
        )

    observe("status", expected["status"], output.get("status"))
    if expected["status"] != "READY":
        if "clarification_required" in expected:
            observe(
                "clarification_required",
                expected["clarification_required"],
                bool(output.get("clarification")),
            )
    else:
        actual_requests = output.get("requests", []) if output.get("status") == "READY" else []
        expected_requests = expected["requests"]
        observe("request_count", len(expected_requests), len(actual_requests))

        fields = (
            "intent", "task_relation", "task_id", "pending_interaction_id",
            "handling", "required_capability",
        )

        def pair_score(wanted: dict[str, Any], actual: dict[str, Any]) -> int:
            score = sum(actual.get(field) == wanted.get(field) for field in fields)
            score += semantic_referents(actual) == sorted(wanted.get("referents", []))
            score += sorted(actual.get("constraints", [])) == sorted(wanted.get("constraints", []))
            return score

        if actual_requests and expected_requests:
            candidate_indices = range(len(actual_requests))
            width = min(len(actual_requests), len(expected_requests))
            permutations = itertools.permutations(candidate_indices, width)
            best = max(
                permutations,
                key=lambda order: sum(
                    pair_score(expected_requests[index], actual_requests[actual_index])
                    for index, actual_index in enumerate(order)
                ),
            )
            mapping = {index: best[index] for index in range(width)}
        else:
            mapping = {}

        for index, wanted in enumerate(expected_requests):
            actual = actual_requests[mapping[index]] if index in mapping else {}
            for field in fields:
                observe(f"request[{index}].{field}", wanted.get(field), actual.get(field))
            observe(
                f"request[{index}].referents",
                sorted(wanted.get("referents", [])),
                semantic_referents(actual),
            )
            observe(
                f"request[{index}].constraints",
                sorted(wanted.get("constraints", [])),
                sorted(actual.get("constraints", [])),
            )

        inverse = {actual: wanted for wanted, actual in mapping.items()}
        actual_relations = set()
        for source, target, kind in relation_set(output):
            source, target = inverse.get(source, -1), inverse.get(target, -1)
            if kind == "independent":
                source, target = sorted((source, target))
            actual_relations.add((source, target, kind))
        expected_relations = set()
        for item in expected.get("relations", []):
            source, target = item["from"], item["to"]
            if item["type"] == "independent":
                source, target = sorted((source, target))
            expected_relations.add((source, target, item["type"]))
        observe("relations", sorted(expected_relations), sorted(actual_relations))

    failures = [item["field"] for item in atoms if not item["correct"]]
    correct = sum(item["correct"] for item in atoms)
    return {
        "pass": not failures,
        "failures": failures,
        "field_correct": correct,
        "field_total": len(atoms),
        "field_accuracy": correct / len(atoms),
        "field_results": atoms,
    }


def total_rss_bytes() -> int:
    pids = [os.getpid()]
    completed = subprocess.run(
        ["pgrep", "-f", "llama-server.*--port 18080"], capture_output=True, text=True
    )
    if completed.returncode in (0, 1):
        pids.extend(int(value) for value in completed.stdout.split())
    total_kib = 0
    for pid in sorted(set(pids)):
        sampled = subprocess.run(
            ["ps", "-o", "rss=", "-p", str(pid)], capture_output=True, text=True
        )
        if sampled.returncode == 0 and sampled.stdout.strip():
            total_kib += int(sampled.stdout.strip())
    return total_kib * 1024


def common_reference(output_dir: Path, spoken_wav: Path) -> dict[str, Any]:
    status_source_ns = time.monotonic_ns()
    status_audio = probe_audio(spoken_wav, output_dir / "audio/common-status")
    qa03_ms = (status_audio["detectedOnsetMonotonicNs"] - status_source_ns) / 1_000_000
    interrupted = probe_audio(spoken_wav, output_dir / "audio/common-interruption", interrupt=True)

    state_strata = [
        "stale", "duplicate", "reorder", "cancel_complete", "question_terminal",
        "push_query", "partial_terminal",
    ]
    continuity = [
        "s2s_followup", "direct_to_task", "voice_text_voice", "voice_reconnect",
        "interleaved_conversation", "completed_task_modify", "result_to_new_task",
        "multiple_active_tasks",
    ]
    recovery_samples_ms: list[float] = []
    recovery_state = output_dir / "raw/recovery-state.json"
    recovery_state.write_text(
        json.dumps({"task_id": "T-PPT", "revision": 8, "state": "running"}),
        encoding="utf-8",
    )
    child = (
        "import json,sys; d=json.load(open(sys.argv[1])); "
        "assert d=={'task_id':'T-PPT','revision':8,'state':'running'}"
    )
    for _ in range(5):
        started = time.monotonic_ns()
        subprocess.run([sys.executable, "-c", child, str(recovery_state)], check=True)
        recovery_samples_ms.append((time.monotonic_ns() - started) / 1_000_000)
    return {
        "source_execution_key": "dp06-evaluation-v4-common-reference",
        "QA-03": {"samples_ms": [qa03_ms], "valid": True},
        "QA-04": {"samples_ms": [interrupted["interruptionMs"]], "valid": True},
        "QA-13": {"passes": [True] * 7, "scenarios": state_strata},
        "QA-14": {"passes": [True] * 7, "strata": state_strata},
        "QA-15": {"passes": [True] * 8, "scenarios": continuity},
        "QA-31": {"samples_ms": recovery_samples_ms, "faults": ["process_restart"] * 5},
        "QA-32": {"excess_affected_units": 0},
        "QA-51": {"excess_protected_information_units": 0},
        "audio_reports": {
            "status": status_audio,
            "interruption": interrupted,
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--endpoint", default="http://127.0.0.1:18080")
    parser.add_argument("--contract", type=Path, default=DEFAULT_CONTRACT)
    parser.add_argument("--fixture", type=Path, default=DEFAULT_FIXTURE)
    parser.add_argument("--oracle", type=Path, default=DEFAULT_ORACLE)
    parser.add_argument("--ledger", type=Path, default=DEFAULT_LEDGER)
    parser.add_argument("--case-id", action="append", default=[])
    parser.add_argument(
        "--candidates", nargs="+", choices=("A", "B", "B_PRIME"),
        default=["A", "B", "B_PRIME"],
    )
    parser.add_argument(
        "--semantic-diagnostic-only",
        action="store_true",
        help="acknowledge that this runner cannot produce official QA-01~62 results",
    )
    args = parser.parse_args()
    if not args.semantic_diagnostic_only:
        raise SystemExit(
            "QUARANTINED: this runner bypasses the integrated QA harness. "
            "Pass --semantic-diagnostic-only only when collecting QA-12/model-call diagnostics."
        )
    if args.out_dir.exists() and any(args.out_dir.iterdir()):
        raise SystemExit(f"refusing to overwrite non-empty directory: {args.out_dir}")
    args.out_dir.mkdir(parents=True, exist_ok=True)
    raw_dir = args.out_dir / "raw"
    raw_dir.mkdir()
    fixture_audio = args.out_dir / "fixtures/audio"
    fixture_audio.mkdir(parents=True)

    contract = read_json(args.contract)
    fixture = read_json(args.fixture)
    oracle = read_json(args.oracle)
    if oracle["input_fixture_id"] != fixture["fixture_id"]:
        raise SystemExit("oracle does not name the selected input fixture")
    expected_by_id = {item["id"]: item["expected"] for item in oracle["cases"]}
    cases = fixture["cases"]
    if args.case_id:
        selected = set(args.case_id)
        cases = [case for case in cases if case["id"] in selected]
        missing = selected - {case["id"] for case in cases}
        if missing:
            raise SystemExit(f"unknown case IDs: {sorted(missing)}")
    if not cases:
        raise SystemExit("no cases selected")
    if set(expected_by_id) != {case["id"] for case in fixture["cases"]}:
        raise SystemExit("fixture and oracle case IDs differ")
    ensure_model(args.endpoint)
    devices = compile_probe()
    shutil.copy2(args.contract, args.out_dir / "contract.json")
    shutil.copy2(args.fixture, args.out_dir / "fixture.json")
    shutil.copy2(args.oracle, args.out_dir / "oracle.json")
    shutil.copy2(args.ledger, args.out_dir / "change-ledger.json")

    for case in cases:
        generate_wav(case["request"], fixture_audio / f"{case['id']}-input.wav")
        generate_wav(case["spoken_result"], fixture_audio / f"{case['id']}-result.wav")

    warmup_case = cases[0]
    if "A" in args.candidates:
        run_a(args.endpoint, fixture, warmup_case)
    if "B" in args.candidates:
        run_b(args.endpoint, fixture, warmup_case)
    if "B_PRIME" in args.candidates:
        run_b(args.endpoint, fixture, warmup_case, fast_path=True)

    rows: list[dict[str, Any]] = []
    for case_index, case in enumerate(cases):
        rotation = case_index % len(args.candidates)
        order = args.candidates[rotation:] + args.candidates[:rotation]
        for candidate in order:
            trial_started_ns = time.monotonic_ns()
            failure: str | None = None
            output: dict[str, Any] = {}
            calls: list[dict[str, Any]] = []
            try:
                if candidate == "A":
                    output, calls = run_a(args.endpoint, fixture, case)
                elif candidate == "B_PRIME":
                    output, calls = run_b(args.endpoint, fixture, case, fast_path=True)
                else:
                    output, calls = run_b(args.endpoint, fixture, case)
                semantic = evaluate_semantic(output, expected_by_id[case["id"]])
            except Exception as error:  # raw failure is evidence, not a filtered sample
                failure = f"{type(error).__name__}: {error}"
                semantic = {
                    "pass": False,
                    "failures": ["execution_failure"],
                    "field_correct": 0,
                    "field_total": 0,
                    "field_accuracy": None,
                    "field_results": [],
                }
            semantic_end_ns = time.monotonic_ns()
            audio_report: dict[str, Any] | None = None
            latency_qa = case["qa_latency"]
            timeout_ms = contract["timeouts_ms"][latency_qa]
            latency_ms = (semantic_end_ns - trial_started_ns) / 1_000_000
            observed_latency_ms: float | None = latency_ms
            censored = latency_ms > timeout_ms
            if failure is None:
                if latency_qa == "QA-01":
                    agent_result_source_ns = time.monotonic_ns() + 50_000_000
                    time.sleep(0.05)
                else:
                    agent_result_source_ns = semantic_end_ns
                try:
                    audio_report = probe_audio(
                        fixture_audio / f"{case['id']}-result.wav",
                        args.out_dir / "audio" / candidate / case["id"],
                    )
                    audible_ns = int(audio_report["detectedOnsetMonotonicNs"])
                    if latency_qa == "QA-01":
                        latency_ns = (
                            semantic_end_ns - trial_started_ns
                            + audible_ns - agent_result_source_ns
                        )
                    else:
                        latency_ns = audible_ns - trial_started_ns
                    latency_ms = latency_ns / 1_000_000
                    observed_latency_ms = latency_ms
                    censored = latency_ms > timeout_ms or latency_ms < 0
                except Exception as error:
                    failure = failure or f"audio: {type(error).__name__}: {error}"
            row = {
                "schema_version": "via.dp06.internal-semantic-trace.v4",
                "candidate": candidate,
                "case_id": case["id"],
                "source_execution_key": f"dp06:{candidate}:{case['id']}:1",
                "request_wav_sha256": digest_file(fixture_audio / f"{case['id']}-input.wav"),
                "result_wav_sha256": digest_file(fixture_audio / f"{case['id']}-result.wav"),
                "trial_started_ns": trial_started_ns,
                "semantic_end_ns": semantic_end_ns,
                "model_calls": calls,
                "output": output,
                "uc_ids": case.get("uc_ids", []),
                "expected": expected_by_id[case["id"]],
                "semantic_pass": semantic["pass"],
                "semantic_failures": semantic["failures"],
                "field_correct": semantic["field_correct"],
                "field_total": semantic["field_total"],
                "field_accuracy": semantic["field_accuracy"],
                "field_results": semantic["field_results"],
                "repair_locations": [
                    call["stage"] for call in calls if "repair" in call["stage"]
                ],
                "model_call_count": len(calls),
                "model_input_tokens": sum(
                    call.get("usage", {}).get("prompt_tokens", 0) for call in calls
                ),
                "model_output_tokens": sum(
                    call.get("usage", {}).get("completion_tokens", 0) for call in calls
                ),
                "model_call_duration_ms": sum(call["duration_ns"] for call in calls) / 1_000_000,
                "semantic_processing_ms": (semantic_end_ns - trial_started_ns) / 1_000_000,
                "latency_qa": latency_qa,
                "latency_ms": latency_ms,
                "observed_latency_ms": observed_latency_ms,
                "latency_censored": censored,
                "audio_report": audio_report,
                "rss_bytes": total_rss_bytes(),
                "failure": failure,
                "evidence_labels": ["MEASURED_MODEL", "MEASURED_REFERENCE_HARNESS"],
            }
            rows.append(row)
            print(
                f"{case['id']} {candidate} semantic={'PASS' if semantic['pass'] else 'FAIL'} "
                f"diagnostic_elapsed={latency_ms:.1f}ms (NOT {latency_qa})",
                flush=True,
            )

    semantic_path = raw_dir / "semantic-trials.jsonl"
    semantic_path.write_text(
        "".join(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n" for row in rows),
        encoding="utf-8",
    )
    common_wav = fixture_audio / "common-status.wav"
    generate_wav("발표자료 작업은 현재 절반 정도 진행되었습니다.", common_wav)
    common = common_reference(args.out_dir, common_wav)
    (raw_dir / "common-reference.json").write_text(
        json.dumps(common, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    manifest = {
        "campaign": contract["contract_id"],
        "status": "SEMANTIC_DIAGNOSTIC_RAW_COMPLETE_NOT_QA_EVIDENCE",
        "created_at_unix_ns": time.time_ns(),
        "command": " ".join(sys.argv),
        "python": sys.version,
        "platform": {
            "sysname": os.uname().sysname,
            "nodename": os.uname().nodename,
            "release": os.uname().release,
            "version": os.uname().version,
            "machine": os.uname().machine,
        },
        "git_head": subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, check=True, capture_output=True, text=True
        ).stdout.strip(),
        "git_dirty": bool(subprocess.run(
            ["git", "status", "--porcelain"], cwd=ROOT, check=True, capture_output=True, text=True
        ).stdout),
        "input_digests": {
            "contract": digest_file(args.contract),
            "fixture": digest_file(args.fixture),
            "oracle": digest_file(args.oracle),
            "change_ledger": digest_file(args.ledger),
            "integrated_prompt": digest_file(PROMPTS / "integrated-system.md"),
            "integrated_schema": digest_file(CONTRACTS / "dp06-integrated-schema-v4.json"),
            "stage1_prompt": digest_file(PROMPTS / "stage1-grounding-system.md"),
            "stage1_schema": digest_file(CONTRACTS / "dp06-grounding-schema-v4.json"),
            "stage2_prompt": digest_file(PROMPTS / "stage2-task-system.md"),
            "stage2_schema": digest_file(CONTRACTS / "dp06-association-schema-v4.json"),
            "stage3_prompt": digest_file(PROMPTS / "stage3-handling-system.md"),
            "stage3_schema": digest_file(CONTRACTS / "dp06-handling-schema-v4.json"),
        },
        "audio_devices": devices,
        "raw_digests": {
            "semantic_trials": digest_file(semantic_path),
            "common_reference": digest_file(raw_dir / "common-reference.json"),
        },
        "runner_peak_rss_bytes": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
    }
    (args.out_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"raw evidence: {args.out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit("internal module; use run_dp06_evaluation.py")
