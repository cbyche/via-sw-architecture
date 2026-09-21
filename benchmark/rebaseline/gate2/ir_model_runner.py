"""Prepare or execute frozen IR-DP01 model calls against a local OpenAI-compatible server.

Default is dry-run. --execute makes actual model calls and saves raw responses, but this
script never computes W-05, p95 scores, or a winner. Stage 2/3 consume actual prior-stage
model output only in --execute mode.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import time
import urllib.request
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
PROMPTS = ROOT / "prototype/gate2/prompts"
GENERATED = ROOT / "benchmark/rebaseline"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def ensure_assets() -> None:
    needed = [GENERATED / "cases.json", GENERATED / "fixtures/base.json", GENERATED / "fixtures/patches.json"]
    if not all(path.exists() for path in needed):
        subprocess.run(["python3", str(GENERATED / "build_assets.py")], cwd=ROOT, check=True)


def load_inputs(case_ids: list[str]) -> list[dict[str, Any]]:
    ensure_assets()
    cases = json.loads((GENERATED / "cases.json").read_text(encoding="utf-8"))
    base = json.loads((GENERATED / "fixtures/base.json").read_text(encoding="utf-8"))
    patches = json.loads((GENERATED / "fixtures/patches.json").read_text(encoding="utf-8"))
    by_id = {case["id"]: case for case in cases}
    rows = []
    for case_id in case_ids:
        case = by_id[case_id]
        patch = patches[case["input"]["patch_key"]]
        rows.append({
            "case_id": case_id,
            "request_revision": 1,
            "user_request": case["input"]["utterance"],
            "scripted_followups": case["input"]["scripted_followups"],
            "evidence": {
                "sources": base["sources"],
                "conversation": base["conversation"],
                "tasks": base["tasks"],
                "agent_catalog": base["agent_catalog"],
                "policy": base["policy"],
                "memory": base["memory"],
                "case_patch": patch,
            },
        })
    return rows


def json_file(name: str) -> dict[str, Any]:
    return json.loads((PROMPTS / name).read_text(encoding="utf-8"))


def text_file(name: str) -> str:
    return (PROMPTS / name).read_text(encoding="utf-8")


def payload(model: str, system: str, user: dict[str, Any], schema: dict[str, Any]) -> dict[str, Any]:
    return {
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": json.dumps(user, ensure_ascii=False, separators=(",", ":"))},
        ],
        "temperature": 0.7,
        "top_p": 0.8,
        "top_k": 20,
        "min_p": 0.0,
        "presence_penalty": 1.5,
        "response_format": {"type": "json_object"},
        "json_schema": schema,
    }


def post(url: str, body: dict[str, Any]) -> tuple[int, dict[str, Any]]:
    data = json.dumps(body, ensure_ascii=False).encode()
    request = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"}, method="POST")
    start = time.perf_counter_ns()
    with urllib.request.urlopen(request, timeout=180) as response:
        parsed = json.loads(response.read())
    return time.perf_counter_ns() - start, parsed


def content(response: dict[str, Any]) -> dict[str, Any]:
    raw = response["choices"][0]["message"]["content"]
    if isinstance(raw, dict):
        return raw
    return json.loads(raw)


def integrated_input(row: dict[str, Any]) -> dict[str, Any]:
    return row


def stage1_input(row: dict[str, Any]) -> dict[str, Any]:
    e = row["evidence"]
    return {
        "case_id": row["case_id"], "request_revision": row["request_revision"],
        "user_request": row["user_request"], "scripted_followups": row["scripted_followups"],
        "evidence": {"sources": e["sources"], "conversation": e["conversation"], "case_patch": e["case_patch"]},
    }


def stage2_input(row: dict[str, Any], stage1: Any) -> dict[str, Any]:
    e = row["evidence"]
    return {
        "case_id": row["case_id"], "request_revision": row["request_revision"], "user_request": row["user_request"],
        "stage1": stage1, "tasks": e["tasks"],
        "pending_interactions": e["case_patch"].get("pending_questions", []) + e["case_patch"].get("pending_approvals", []),
    }


def stage3_input(row: dict[str, Any], stage1: Any, stage2: Any) -> dict[str, Any]:
    e = row["evidence"]
    return {
        "case_id": row["case_id"], "request_revision": row["request_revision"], "user_request": row["user_request"],
        "stage1": stage1, "stage2": stage2,
        "bounded_core_capabilities": ["bounded.explain", "bounded.summarize", "source.read"],
        "agent_catalog": e["agent_catalog"], "policy": e["policy"],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--server", default="http://127.0.0.1:8080")
    parser.add_argument("--model", default="Qwen3-8B-Q4_K_M")
    parser.add_argument("--case", action="append", dest="cases")
    parser.add_argument("--output", type=Path, default=ROOT / "results/gate2-local/ir-requests.json")
    args = parser.parse_args()

    case_ids = args.cases or ["TC-04.2", "TC-06.2", "TC-09.3", "TC-09.4", "TC-14.5"]
    rows = load_inputs(case_ids)
    systems = {
        "integrated": text_file("integrated-system.md"),
        "stage1": text_file("stage1-grounding-system.md"),
        "stage2": text_file("stage2-task-system.md"),
        "stage3": text_file("stage3-handling-system.md"),
    }
    schemas = {
        "integrated": json_file("integrated-schema.json"),
        "stage1": json_file("stage1-schema.json"),
        "stage2": json_file("stage2-schema.json"),
        "stage3": json_file("stage3-schema.json"),
    }
    result: dict[str, Any] = {
        "status": "MODEL_CALLS_EXECUTED_RAW_ONLY" if args.execute else "DRY_RUN_MODEL_NOT_CALLED",
        "scoring": "NOT_RUN", "model": args.model,
        "prompt_hashes": {p.name: sha(p) for p in sorted(PROMPTS.glob("*")) if p.is_file()},
        "cases": [],
    }

    endpoint = args.server.rstrip("/") + "/v1/chat/completions"
    for row in rows:
        integrated_body = payload(args.model, systems["integrated"], integrated_input(row), schemas["integrated"])
        record: dict[str, Any] = {"case_id": row["case_id"], "integrated_request": integrated_body}
        if args.execute:
            ns, raw = post(endpoint, integrated_body)
            record["integrated_response"] = {"latency_ns": ns, "raw": raw}

        s1_body = payload(args.model, systems["stage1"], stage1_input(row), schemas["stage1"])
        if args.execute:
            ns1, raw1 = post(endpoint, s1_body)
            s1 = content(raw1)
            s2_body = payload(args.model, systems["stage2"], stage2_input(row, s1), schemas["stage2"])
            ns2, raw2 = post(endpoint, s2_body)
            s2 = content(raw2)
            s3_body = payload(args.model, systems["stage3"], stage3_input(row, s1, s2), schemas["stage3"])
            ns3, raw3 = post(endpoint, s3_body)
            record["staged"] = [
                {"stage": 1, "request": s1_body, "latency_ns": ns1, "raw": raw1},
                {"stage": 2, "request": s2_body, "latency_ns": ns2, "raw": raw2},
                {"stage": 3, "request": s3_body, "latency_ns": ns3, "raw": raw3},
            ]
        else:
            placeholder1 = {"$MODEL_OUTPUT": "stage1"}
            placeholder2 = {"$MODEL_OUTPUT": "stage2"}
            record["staged_requests"] = [
                {"stage": 1, "request": s1_body},
                {"stage": 2, "request": payload(args.model, systems["stage2"], stage2_input(row, placeholder1), schemas["stage2"])},
                {"stage": 3, "request": payload(args.model, systems["stage3"], stage3_input(row, placeholder1, placeholder2), schemas["stage3"])},
            ]
        result["cases"].append(record)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(args.output)


if __name__ == "__main__":
    main()
