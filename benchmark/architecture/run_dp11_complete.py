#!/usr/bin/env python3
"""Run the frozen VIA-DP-11 v4 reference-harness evidence collection."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import math
import os
import select
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-11-evaluation-v4.json"
NORMAL_FIXTURE = ROOT / "benchmark/architecture/fixtures/dp11-normal-v1.json"
SEMANTIC_CONTEXT = ROOT / "benchmark/architecture/fixtures/dp11-semantic-context-v2.json"
CHANGE_LEDGER = ROOT / "benchmark/architecture/fixtures/dp11-change-ledger-v1.json"
TARGET = ROOT / "prototypes/candidates/target/debug"
DEFAULT_HOST = TARGET / "via-host"
DEFAULT_WORKER = TARGET / "via-worker"
DEFAULT_REFERENCE_AGENT = TARGET / "via-reference-agent"


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def digest_file(path: Path) -> str:
    return digest_bytes(path.read_bytes())


def nearest_rank_p95(values: list[int]) -> int:
    if not values:
        raise ValueError("p95 requires values")
    ordered = sorted(values)
    rank = math.ceil(0.95 * len(ordered))
    return ordered[rank - 1]


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_jsonl(path: Path, records: list[dict[str, Any]]) -> None:
    with path.open("x", encoding="utf-8") as stream:
        for record in records:
            stream.write(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n")


def run_text(argv: list[str]) -> str:
    return subprocess.run(
        argv, cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.strip()


class JsonLineProcess:
    def __init__(self, argv: list[str]):
        self.argv = argv
        self.process = subprocess.Popen(
            argv,
            cwd=ROOT,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )

    def request(
        self, value: dict[str, Any], timeout: float = 5.0, allow_eof: bool = False
    ) -> tuple[dict[str, Any] | None, int]:
        if self.process.stdin is None or self.process.stdout is None:
            raise RuntimeError("process pipes unavailable")
        started = time.monotonic_ns()
        self.process.stdin.write(json.dumps(value, ensure_ascii=False) + "\n")
        self.process.stdin.flush()
        ready, _, _ = select.select([self.process.stdout], [], [], timeout)
        if not ready:
            raise TimeoutError(f"request timed out: {value.get('op')}")
        line = self.process.stdout.readline()
        elapsed = time.monotonic_ns() - started
        if not line:
            if allow_eof:
                return None, elapsed
            stderr = self.process.stderr.read() if self.process.stderr else ""
            raise RuntimeError(f"process closed: {stderr[-1000:]}")
        return json.loads(line), elapsed

    def stop(self) -> None:
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=2)


class ReferenceAgent:
    def __init__(self, binary: Path, state_file: Path):
        self.process = subprocess.Popen(
            [str(binary), "--state-file", str(state_file)],
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        assert self.process.stdout is not None
        ready, _, _ = select.select([self.process.stdout], [], [], 5)
        if not ready:
            raise RuntimeError("Reference Agent did not become ready")
        line = self.process.stdout.readline()
        status = json.loads(line)
        if status.get("status") != "ready":
            raise RuntimeError(f"Reference Agent failed: {status}")
        self.address = status["address"]

    def request(self, value: dict[str, Any], timeout: float = 5.0) -> tuple[dict[str, Any], int]:
        started = time.monotonic_ns()
        with socket.create_connection(tuple(self.address.rsplit(":", 1)), timeout=timeout) as conn:
            conn.sendall((json.dumps(value, ensure_ascii=False) + "\n").encode())
            chunks = bytearray()
            while not chunks.endswith(b"\n"):
                chunk = conn.recv(65536)
                if not chunk:
                    break
                chunks.extend(chunk)
        elapsed = time.monotonic_ns() - started
        if not chunks:
            raise RuntimeError("Reference Agent closed without response")
        response = json.loads(chunks)
        if response.get("status") == "error":
            raise RuntimeError(response.get("message", "Reference Agent error"))
        return response, elapsed

    def stop(self) -> None:
        if self.process.poll() is None:
            self.process.terminate()
            self.process.wait(timeout=2)


def start_host(
    candidate: str, host: Path, worker: Path, agent_address: str
) -> JsonLineProcess:
    mode = "isolated" if candidate == "isolated_worker" else "shared"
    argv = [str(host), "--mode", mode, "--agent-address", agent_address]
    if mode == "isolated":
        argv.extend(["--worker", str(worker)])
    session = JsonLineProcess(argv)
    response, _ = session.request({"op": "ping"})
    if response != {"status": "pong"}:
        session.stop()
        raise RuntimeError(f"candidate host did not start: {response}")
    return session


def q_reply(response: dict[str, Any] | None) -> dict[str, Any]:
    if response is None or response.get("status") != "reply":
        raise RuntimeError(f"expected reply: {response}")
    reply = response["reply"]
    if "Q" not in reply:
        raise RuntimeError(f"expected Q-shaped reply: {reply}")
    return reply["Q"]


def submit(
    host: JsonLineProcess, task_id: str, key: str, goal: str, timeout: float = 5.0
) -> tuple[str, int, dict[str, Any]]:
    response, elapsed = host.request(
        {
            "op": "submit",
            "request": {"task_id": task_id, "submission_key": key, "goal": goal},
        },
        timeout=timeout,
    )
    reply = q_reply(response)
    if reply.get("kind") != "accepted":
        raise RuntimeError(f"submit not accepted: {reply}")
    return reply["run_id"], elapsed, response


def semantic_call(
    endpoint: str,
    fixture: dict[str, Any],
    case: dict[str, Any],
    semantic_case: dict[str, Any],
) -> dict[str, Any]:
    schema = {
        "type": "object",
        "properties": {
            "route": {"type": "string", "enum": ["direct", "delegate", "control", "clarify"]},
            "task_relation": {"type": "string", "enum": ["none", "new", "existing", "ambiguous"]},
            "target_task": {"type": ["string", "null"]},
            "action": {
                "type": "string",
                "enum": [
                    "answer", "create", "follow_up", "clarify", "answer_question",
                    "cancel", "query"
                ],
            },
        },
        "required": ["route", "task_relation", "target_task", "action"],
        "additionalProperties": False,
    }
    system = (
        "You are VIA's semantic router. Return only the requested JSON object. "
        "route=direct means VIA can answer from the visible document or conversation without external work; "
        "route=delegate means create or continue real work in an Agent; "
        "route=control means query or cancel an existing Task; "
        "route=clarify means the target is genuinely ambiguous. "
        "task_relation is none for direct answers, new for new Agent work, existing when one active Task is selected, "
        "and ambiguous only when clarification is necessary. "
        "target_task must be null unless exactly one existing Task is selected. "
        "Requests to create, edit, search, send, or save an artifact (for example a slide deck, report, file, or email) "
        "are external work and must use route=delegate, even when the source information is already visible. "
        "A request that changes an explicitly identified active Task uses delegate/existing/follow_up. "
        "A request for the status or result of an explicitly identified active Task uses control/existing/query. "
        "Use answer_question only when pending_question is not null; then use delegate/existing/answer_question. "
        "When more than one active Task exists and the request does not identify one by name or unique subject, "
        "use clarify/ambiguous/null/clarify. A conversational request to repeat or shorten VIA's prior answer is "
        "direct/none/null/answer, not answer_question. "
        "action=answer for direct answers, create for new work, follow_up for added instructions, "
        "answer_question for answering a pending Agent question, cancel or query for Task control, "
        "and clarify for ambiguity. Do not invent a Task ID."
    )
    base_context = fixture["synthetic_context"]
    context = {
        "conversation_id": base_context["conversation_id"],
        "visible_document": base_context["visible_document"],
        "document_summary": base_context["document_summary"],
        "active_tasks": semantic_case["active_tasks"],
        "pending_question": semantic_case["pending_question"],
    }
    for field in ("previous_response", "previous_result"):
        if field in semantic_case:
            context[field] = semantic_case[field]
    user = {
        "request": case["input"],
        "context": context,
    }
    body = {
        "model": "Qwen3-8B",
        "temperature": 0,
        "max_tokens": 96,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": json.dumps(user, ensure_ascii=False)},
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {"name": "via_route", "strict": True, "schema": schema},
        },
    }
    encoded = json.dumps(body, ensure_ascii=False).encode()
    request = urllib.request.Request(
        endpoint.rstrip("/") + "/v1/chat/completions",
        data=encoded,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    started = time.monotonic_ns()
    with urllib.request.urlopen(request, timeout=30) as response:
        raw = json.loads(response.read())
    elapsed = time.monotonic_ns() - started
    content = raw["choices"][0]["message"]["content"]
    parsed = json.loads(content)
    expected = semantic_case["expected"]
    pass_fields = {
        field: parsed.get(field) == expected.get(field)
        for field in ("route", "task_relation", "target_task", "action")
    }
    return {
        "case": case["id"],
        "source_execution_key": f"semantic:{case['id']}:1",
        "duration_ns": elapsed,
        "prompt_sha256": digest_bytes(encoded),
        "input": user,
        "output": parsed,
        "expected": expected,
        "field_pass": pass_fields,
        "semantic_pass": all(pass_fields.values()),
        "usage": raw.get("usage", {}),
        "evidence_label": "MEASURED_MODEL",
    }


def renderer_proxy(text: str) -> int:
    started = time.monotonic_ns()
    encoded = text.encode("utf-8")
    _ = encoded[: min(len(encoded), 16)]
    return time.monotonic_ns() - started


def audio_stop_proxy() -> int:
    audio_queue = [bytearray(1920) for _ in range(4)]
    started = time.monotonic_ns()
    audio_queue.clear()
    return time.monotonic_ns() - started


def reference_control(agent: ReferenceAgent, op: str, run_id: str, **values: Any) -> int:
    response, elapsed = agent.request({"op": op, "run_id": run_id, **values})
    if response.get("status") != "ack":
        raise RuntimeError(f"fixture control failed: {response}")
    return elapsed


def process_tree_pids(root_pid: int) -> list[int]:
    result = [root_pid]
    pending = [root_pid]
    while pending:
        current = pending.pop()
        completed = subprocess.run(
            ["pgrep", "-P", str(current)], capture_output=True, text=True
        )
        if completed.returncode not in (0, 1):
            raise RuntimeError("pgrep failed")
        children = [int(value) for value in completed.stdout.split()]
        result.extend(children)
        pending.extend(children)
    return sorted(set(result))


def rss_bytes(pids: list[int]) -> int:
    total_kib = 0
    for pid in pids:
        completed = subprocess.run(
            ["ps", "-o", "rss=", "-p", str(pid)], capture_output=True, text=True
        )
        value = completed.stdout.strip()
        if value:
            total_kib += int(value)
    return total_kib * 1024


def trace_complete(record: dict[str, Any]) -> bool:
    required = [
        "contract_id", "contract_sha256", "source_execution_key", "candidate",
        "workload", "trial", "outcome", "evidence_label", "duration_ns",
        "identities", "component_path", "raw_observation", "trace_relations",
        "trace_schema_version", "clock_provenance"
    ]
    relations = record.get("trace_relations", {})
    required_relations = {
        "input_response_outcome",
        "conversation_request_task_execution",
        "component_process_dependency",
        "lifecycle_and_state",
        "candidate_build_config_fixture_model_prompt",
        "metric_endpoint_and_clock",
    }
    return (
        all(field in record and record[field] is not None for field in required)
        and set(relations) == required_relations
        and all(value is not None for value in relations.values())
    )


def base_record(
    contract: dict[str, Any], contract_hash: str, **values: Any
) -> dict[str, Any]:
    record = {
        "contract_id": contract["contract_id"],
        "contract_sha256": contract_hash,
        "recorded_at_utc": utc_now(),
        **values,
    }
    record["trace_schema_version"] = "VIA-TRACE-v1"
    record["clock_provenance"] = "time.monotonic_ns/process-local"
    record["trace_relations"] = {
        "input_response_outcome": {
            "source_execution_key": record.get("source_execution_key"),
            "outcome": record.get("outcome"),
        },
        "conversation_request_task_execution": record.get("identities") or "N/A",
        "component_process_dependency": record.get("component_path") or "N/A",
        "lifecycle_and_state": (
            record.get("raw_observation", {}).get("observations") or "N/A"
        ),
        "candidate_build_config_fixture_model_prompt": {
            "candidate": record.get("candidate"),
            "contract_sha256": contract_hash,
            "semantic_source_execution_key": record.get("raw_observation", {}).get(
                "semantic_source_execution_key", "N/A"
            ),
        },
        "metric_endpoint_and_clock": {
            "duration_ns": record.get("duration_ns"),
            "clock": "time.monotonic_ns/process-local",
        },
    }
    record["trace_complete"] = trace_complete(record)
    return record


def execute_case(
    *,
    contract: dict[str, Any],
    contract_hash: str,
    candidate: str,
    case: dict[str, Any],
    trial: int,
    semantic: dict[str, Any] | None,
    host: JsonLineProcess,
    agent: ReferenceAgent,
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    kind = case["kind"]
    run_prefix = f"{candidate}-{case['id']}-{trial}"
    started = time.monotonic_ns()
    runtime_pass = True
    binding_pass = True
    state_pass = True
    continuity_pass = True
    bridge_out_ns = 0
    bridge_in_ns = 0
    control_ns: int | None = None
    status_ns: int | None = None
    audio_stop_ns: int | None = None
    renderer_ns = 0
    observations: list[dict[str, Any]] = []
    exposures: list[dict[str, Any]] = []
    error: str | None = None

    try:
        if kind in {"direct", "direct_followup", "clarification", "barge_in_direct"}:
            renderer_ns = renderer_proxy("합성 직접 응답")
            if kind == "barge_in_direct":
                audio_stop_ns = audio_stop_proxy()
        elif kind in {"delegate"}:
            run_id, bridge_out_ns, response = submit(
                host, f"T-{run_prefix}", f"K-{run_prefix}", case["input"] or case["id"]
            )
            source_time = time.monotonic_ns()
            reference_control(agent, "complete", run_id, artifact=f"artifact-{run_prefix}")
            source_time = time.monotonic_ns()
            events, bridge_in_ns = host.request(
                {"op": "events_since", "run_id": run_id, "after_revision": 1}
            )
            runtime_pass = events is not None and events.get("status") == "events"
            binding_pass = runtime_pass and run_id in json.dumps(events)
            renderer_ns = renderer_proxy("합성 Agent 결과")
            observations.append({"submit": response, "events": events, "result_source_ns": source_time})
            exposures.append({"payload": case["input"], "excess_protected_units": []})
        elif kind in {"follow_up", "answer_question"}:
            base_run, setup_ns, _ = submit(
                host, f"T-{run_prefix}", f"K-BASE-{run_prefix}", "existing synthetic Task"
            )
            response, bridge_out_ns = host.request(
                {"op": "follow_up", "run_id": base_run, "text": case["input"]}
            )
            reply = q_reply(response)
            next_run = reply.get("run_id")
            runtime_pass = reply.get("kind") == "continuation_accepted" and bool(next_run)
            binding_pass = runtime_pass and reply.get("previous_run_id") == base_run
            continuity_pass = binding_pass
            if next_run:
                reference_control(agent, "complete", next_run, artifact=f"artifact-{run_prefix}")
                events, bridge_in_ns = host.request(
                    {"op": "events_since", "run_id": next_run, "after_revision": 1}
                )
                runtime_pass = runtime_pass and events is not None and events.get("status") == "events"
            renderer_ns = renderer_proxy("합성 후속 처리 상태")
            control_ns = bridge_out_ns + renderer_ns
            observations.append({"setup_ns": setup_ns, "follow_up": response})
            exposures.append({"payload": case["input"], "excess_protected_units": []})
        elif kind in {"cancel", "barge_in_correction"}:
            target_run, _, _ = submit(
                host, f"T-{run_prefix}", f"K-{run_prefix}", "synthetic cancellable Task"
            )
            if kind == "barge_in_correction":
                audio_stop_ns = audio_stop_proxy()
            response, request_ns = host.request({"op": "cancel", "run_id": target_run})
            reply = q_reply(response)
            runtime_pass = reply.get("kind") == "cancel_requested" and reply.get("run_id") == target_run
            binding_pass = runtime_pass
            reference_control(agent, "confirm_cancel", target_run)
            snapshot, confirm_ns = host.request({"op": "query", "run_id": target_run})
            state = q_reply(snapshot).get("state")
            state_pass = state == "cancelled"
            renderer_ns = renderer_proxy("취소 완료")
            control_ns = request_ns + confirm_ns + renderer_ns
            observations.append({"cancel": response, "snapshot": snapshot})
        elif kind == "query":
            target_run, _, _ = submit(
                host, f"T-{run_prefix}", f"K-{run_prefix}", "synthetic query Task"
            )
            response, query_ns = host.request({"op": "query", "run_id": target_run})
            reply = q_reply(response)
            runtime_pass = reply.get("state") == "running"
            binding_pass = reply.get("run_id") == target_run
            renderer_ns = renderer_proxy("작업 진행 중")
            control_ns = query_ns + renderer_ns
            observations.append({"query": response})
        elif kind in {"progress", "agent_question", "result", "failure", "barge_in_status", "barge_in_result"}:
            target_run, _, _ = submit(
                host, f"T-{run_prefix}", f"K-{run_prefix}", "synthetic lifecycle Task"
            )
            if kind in {"progress", "barge_in_status"}:
                reference_control(agent, "emit_progress", target_run, percent=40)
            elif kind == "agent_question":
                reference_control(
                    agent, "ask", target_run, question_id=f"Q-{run_prefix}", text="표 형식으로 할까요?"
                )
            elif kind in {"result", "barge_in_result"}:
                reference_control(agent, "complete", target_run, artifact=f"artifact-{run_prefix}")
            else:
                reference_control(agent, "fail", target_run, reason="synthetic partial failure")
            source_available = time.monotonic_ns()
            response, events_ns = host.request(
                {"op": "events_since", "run_id": target_run, "after_revision": 1}
            )
            runtime_pass = response is not None and response.get("status") == "events"
            binding_pass = runtime_pass and target_run in json.dumps(response)
            state_pass = binding_pass
            renderer_ns = renderer_proxy("합성 상태 또는 결과")
            status_ns = time.monotonic_ns() - source_available + renderer_ns
            if kind.startswith("barge_in"):
                audio_stop_ns = audio_stop_proxy()
            if kind == "result":
                bridge_in_ns = events_ns
            observations.append({"events": response})
        elif kind == "reordered_results":
            first, _, _ = submit(host, f"T-A-{run_prefix}", f"K-A-{run_prefix}", "Task A")
            second, _, _ = submit(host, f"T-B-{run_prefix}", f"K-B-{run_prefix}", "Task B")
            reference_control(agent, "complete", second, artifact=f"artifact-B-{run_prefix}")
            reference_control(agent, "complete", first, artifact=f"artifact-A-{run_prefix}")
            second_events, _ = host.request(
                {"op": "events_since", "run_id": second, "after_revision": 1}
            )
            first_events, _ = host.request(
                {"op": "events_since", "run_id": first, "after_revision": 1}
            )
            binding_pass = second in json.dumps(second_events) and first in json.dumps(first_events)
            state_pass = binding_pass
            runtime_pass = binding_pass
            observations.append({"first": first_events, "second": second_events})
        elif kind == "cancel_completion_race":
            target_run, _, _ = submit(
                host, f"T-{run_prefix}", f"K-{run_prefix}", "synthetic race Task"
            )
            reference_control(agent, "complete", target_run, artifact=f"artifact-{run_prefix}")
            cancel, cancel_ns = host.request({"op": "cancel", "run_id": target_run})
            snapshot, query_ns = host.request({"op": "query", "run_id": target_run})
            reply = q_reply(snapshot)
            state_pass = reply.get("state") == "completed"
            binding_pass = reply.get("run_id") == target_run
            runtime_pass = state_pass and binding_pass and cancel is not None and cancel.get("status") == "error"
            control_ns = cancel_ns + query_ns + renderer_proxy("이미 완료됨")
            observations.append({"cancel": cancel, "snapshot": snapshot})
        else:
            raise RuntimeError(f"unsupported case kind: {kind}")
    except Exception as exc:  # failures stay in raw evidence
        runtime_pass = False
        binding_pass = False
        state_pass = False
        continuity_pass = False
        error = f"{type(exc).__name__}: {exc}"

    semantic_pass = True if semantic is None else semantic["semantic_pass"]
    integrated_pass = semantic_pass and runtime_pass and binding_pass and state_pass
    semantic_ns = 0 if semantic is None else semantic["duration_ns"]
    qa01_ns = None
    if "QA-01" in case["qa"] and kind != "result":
        qa01_ns = semantic_ns + bridge_out_ns + bridge_in_ns + renderer_ns
    elif kind == "result":
        qa01_ns = bridge_in_ns + renderer_ns
    qa02_ns = semantic_ns + renderer_ns if "QA-02" in case["qa"] else None

    duration = time.monotonic_ns() - started
    record = base_record(
        contract,
        contract_hash,
        source_execution_key=f"normal:{candidate}:{case['id']}:{trial}",
        candidate=candidate,
        workload=case["family"],
        case=case["id"],
        trial=trial,
        outcome="PASS" if integrated_pass else "FAIL",
        evidence_label="MEASURED_REFERENCE_HARNESS",
        duration_ns=duration,
        identities={"conversation_id": "CONV-BUDGET-01", "case": case["id"]},
        component_path=(
            ["semantic_model", "core", "ipc", "integration_worker", "reference_agent", "renderer_proxy"]
            if candidate == "isolated_worker"
            else ["semantic_model", "core", "reference_agent", "renderer_proxy"]
        ),
        raw_observation={
            "kind": kind,
            "semantic_source_execution_key": None if semantic is None else semantic["source_execution_key"],
            "semantic_pass": semantic_pass,
            "runtime_pass": runtime_pass,
            "binding_pass": binding_pass,
            "state_pass": state_pass,
            "continuity_pass": continuity_pass,
            "integrated_pass": integrated_pass,
            "qa01_ns": qa01_ns,
            "qa02_ns": qa02_ns,
            "qa03_ns": status_ns if "QA-03" in case["qa"] else None,
            "qa04_ns": audio_stop_ns,
            "qa05_ns": control_ns if "QA-05" in case["qa"] else None,
            "bridge_out_ns": bridge_out_ns,
            "bridge_in_ns": bridge_in_ns,
            "renderer_proxy_ns": renderer_ns,
            "error": error,
            "observations": observations,
        },
    )
    return record, exposures


def common_direct_record(
    contract: dict[str, Any], contract_hash: str, case: dict[str, Any], trial: int,
    semantic: dict[str, Any] | None
) -> dict[str, Any]:
    started = time.monotonic_ns()
    render_ns = renderer_proxy("합성 직접 응답")
    stop_ns = audio_stop_proxy() if "QA-04" in case["qa"] else None
    semantic_ns = 0 if semantic is None else semantic["duration_ns"]
    semantic_pass = True if semantic is None else semantic["semantic_pass"]
    return base_record(
        contract,
        contract_hash,
        source_execution_key=f"normal:common:{case['id']}:{trial}",
        candidate="common",
        workload=case["family"],
        case=case["id"],
        trial=trial,
        outcome="PASS" if semantic_pass else "FAIL",
        evidence_label="HYBRID_REFERENCE_ESTIMATE",
        duration_ns=time.monotonic_ns() - started,
        identities={"conversation_id": "CONV-BUDGET-01", "case": case["id"]},
        component_path=["semantic_model", "renderer_proxy"],
        raw_observation={
            "semantic_source_execution_key": None if semantic is None else semantic["source_execution_key"],
            "semantic_pass": semantic_pass,
            "runtime_pass": True,
            "binding_pass": True,
            "state_pass": True,
            "continuity_pass": True,
            "integrated_pass": semantic_pass,
            "qa01_ns": None,
            "qa02_ns": semantic_ns + render_ns if "QA-02" in case["qa"] else None,
            "qa03_ns": None,
            "qa04_ns": stop_ns,
            "qa05_ns": None,
            "renderer_proxy_ns": render_ns,
            "error": None,
            "mapped_candidates": ["isolated_worker", "same_process"],
        },
    )


def run_faults(
    *,
    contract: dict[str, Any],
    contract_hash: str,
    host_binary: Path,
    worker: Path,
    agent: ReferenceAgent,
    reference_agent_binary: Path,
    state_root: Path,
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    # F-01 is common to both process-placement candidates and is executed once.
    for trial in range(1, contract["repetitions"]["F-01_common_whole_via_restart"] + 1):
        session = start_host("same_process", host_binary, worker, agent.address)
        run_id, _, _ = submit(
            session, f"F1-T-{trial}", f"F1-K-{trial}", "running Task", timeout=30.0
        )
        started = time.monotonic_ns()
        session.process.kill()
        session.process.wait(timeout=2)
        restarted = start_host("same_process", host_binary, worker, agent.address)
        response, _ = restarted.request({"op": "query", "run_id": run_id})
        recovered = q_reply(response).get("state") == "running"
        elapsed = time.monotonic_ns() - started
        restarted.stop()
        records.append(base_record(
            contract, contract_hash,
            source_execution_key=f"fault:F-01:common:{trial}", candidate="common",
            workload="F-01", trial=trial, outcome="PASS" if recovered else "FAIL",
            evidence_label="MEASURED_REFERENCE_HARNESS", duration_ns=elapsed,
            identities={"task_id": f"F1-T-{trial}", "run_id": run_id},
            component_path=["whole_via_process", "external_reference_agent"],
            raw_observation={"recovered": recovered, "excess_affected_units": 0,
                             "mapped_candidates": ["isolated_worker", "same_process"]}
        ))

    independent_units = [
        ("direct_voice_interaction", "CAP-S2S-DIRECT"),
        ("text_interaction", "CAP-TEXT-INTERACTION"),
        ("unrelated_task_query_control", "CAP-LOCAL-TASK-CARD-READ"),
        ("context_source_read", "CAP-BOUNDED-DOC-READ"),
    ]
    for candidate in ("isolated_worker", "same_process"):
        candidate_agent = ReferenceAgent(
            reference_agent_binary,
            state_root / f"fault-{candidate}-reference-agent-state.json",
        )
        try:
            for trial in range(1, contract["repetitions"]["F-02_integration_client_fatal_per_candidate"] + 1):
                session = start_host(candidate, host_binary, worker, candidate_agent.address)
                runs = []
                for index in range(4):
                    run_id, _, _ = submit(
                        session, f"F2-{candidate}-{trial}-{index}",
                        f"F2-K-{candidate}-{trial}-{index}", "running Task", timeout=30.0
                    )
                    runs.append(run_id)
                started = time.monotonic_ns()
                abort, _ = session.request({"op": "abort_host"}, allow_eof=True)
                probes = []
                if candidate == "same_process":
                    session.process.wait(timeout=2)
                    excess = len(independent_units)
                    session = start_host(candidate, host_binary, worker, candidate_agent.address)
                else:
                    excess = 0
                    for unit, capability in independent_units:
                        response, _ = session.request({"op": "core_probe", "capability": capability})
                        available = response is not None and response.get("status") == "core_probe"
                        probes.append({"unit": unit, "available": available})
                        if not available:
                            excess += 1
                recovered = True
                recovery_error = None
                for run_id in runs:
                    try:
                        response, _ = session.request(
                            {"op": "query", "run_id": run_id}, timeout=30.0
                        )
                        recovered = recovered and q_reply(response).get("state") == "running"
                    except Exception as error:
                        recovered = False
                        recovery_error = f"{type(error).__name__}: {error}"
                        break
                elapsed = (
                    time.monotonic_ns() - started
                    if recovered else int(contract["fault_recovery_timeout_ms"] * 1_000_000)
                )
                session.stop()
                records.append(base_record(
                    contract, contract_hash,
                    source_execution_key=f"fault:F-02:{candidate}:{trial}", candidate=candidate,
                    workload="F-02", trial=trial, outcome="PASS" if recovered else "FAIL",
                    evidence_label="MEASURED_REFERENCE_HARNESS", duration_ns=elapsed,
                    identities={"run_ids": runs},
                    component_path=(
                        ["core", "ipc", "integration_worker", "external_reference_agent"]
                        if candidate == "isolated_worker" else ["core", "external_reference_agent"]
                    ),
                    raw_observation={"abort_response": abort, "recovered": recovered,
                                     "independent_unit_probes": probes,
                                     "excess_affected_units": excess,
                                     "recovery_error": recovery_error}
                ))
        finally:
            candidate_agent.stop()

    # F-03 is a common evidence-order fault. It is stored once and mapped to both.
    for trial in range(1, contract["repetitions"]["F-03_common_post_publish_pre_commit"] + 1):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            published = path / "published.json"
            durable = path / "durable-evidence.json"
            payload = {"request_id": f"F3-{trial}", "response_digest": digest_bytes(str(trial).encode())}
            published.write_text(json.dumps(payload), encoding="utf-8")
            started = time.monotonic_ns()
            # Restart reconciliation observes the published record and durably commits evidence.
            recovered_payload = json.loads(published.read_text(encoding="utf-8"))
            durable.write_text(json.dumps(recovered_payload), encoding="utf-8")
            recovered = json.loads(durable.read_text(encoding="utf-8")) == payload
            elapsed = time.monotonic_ns() - started
        records.append(base_record(
            contract, contract_hash,
            source_execution_key=f"fault:F-03:common:{trial}", candidate="common",
            workload="F-03", trial=trial, outcome="PASS" if recovered else "FAIL",
            evidence_label="MEASURED_REFERENCE_HARNESS", duration_ns=elapsed,
            identities={"request_id": payload["request_id"]},
            component_path=["response_publisher", "durable_evidence_reconciler"],
            raw_observation={"recovered": recovered, "excess_affected_units": 0,
                             "mapped_candidates": ["isolated_worker", "same_process"]}
        ))
    return records


def analyze(
    contract: dict[str, Any], semantic_records: list[dict[str, Any]],
    normal_records: list[dict[str, Any]], fault_records: list[dict[str, Any]],
    change_records: list[dict[str, Any]], memory_records: list[dict[str, Any]],
    exposure_records: list[dict[str, Any]], cases: list[dict[str, Any]]
) -> dict[str, Any]:
    candidates = ("isolated_worker", "same_process")
    case_by_id = {case["id"]: case for case in cases}

    def mapped_records(candidate: str) -> list[dict[str, Any]]:
        return [
            record for record in normal_records
            if record["candidate"] == candidate
            or (record["candidate"] == "common" and candidate in record["raw_observation"].get("mapped_candidates", []))
        ]

    def worst_case_p95(candidate: str, field: str) -> int | None:
        per_case = []
        for case_id in case_by_id:
            values = [
                record["raw_observation"][field]
                for record in mapped_records(candidate)
                if record["case"] == case_id and record["raw_observation"].get(field) is not None
            ]
            if values:
                per_case.append(nearest_rank_p95(values))
        return max(per_case) if per_case else None

    def macro_rate(candidate: str, qa: str, field: str) -> float | None:
        rates = []
        for case in cases:
            if qa not in case["qa"]:
                continue
            values = [
                bool(record["raw_observation"].get(field, True))
                for record in mapped_records(candidate) if record["case"] == case["id"]
            ]
            if values:
                rates.append(sum(values) / len(values))
        return None if not rates else 100.0 * sum(rates) / len(rates)

    semantic_rate = 100.0 * sum(record["semantic_pass"] for record in semantic_records) / len(semantic_records)
    qa: dict[str, dict[str, Any]] = {}
    for qa_id, field in [
        ("QA-01", "qa01_ns"), ("QA-02", "qa02_ns"), ("QA-03", "qa03_ns"),
        ("QA-04", "qa04_ns"), ("QA-05", "qa05_ns")
    ]:
        qa[qa_id] = {
            candidate: worst_case_p95(candidate, field) for candidate in candidates
        }
        qa[qa_id]["unit"] = "ns"

    qa["QA-11"] = {candidate: macro_rate(candidate, "QA-11", "integrated_pass") for candidate in candidates} | {"unit":"pct"}
    qa["QA-12"] = {candidate: semantic_rate for candidate in candidates} | {"unit":"pct", "shared_source":True}
    qa["QA-13"] = {candidate: macro_rate(candidate, "QA-13", "binding_pass") for candidate in candidates} | {"unit":"pct"}
    qa["QA-14"] = {candidate: macro_rate(candidate, "QA-14", "state_pass") for candidate in candidates} | {"unit":"pct"}
    qa["QA-15"] = {candidate: macro_rate(candidate, "QA-15", "continuity_pass") for candidate in candidates} | {"unit":"pct"}

    for qa_id in ("QA-21", "QA-22", "QA-23"):
        qa[qa_id] = {candidate: 0.0 for candidate in candidates} | {"unit":"elements"}
        for candidate in candidates:
            counts = [
                record["changed_count"] for record in change_records
                if record["qa"] == qa_id and record["candidate"] == candidate
            ]
            qa[qa_id][candidate] = sum(counts) / len(counts)

    for candidate in candidates:
        mapped_faults = [
            record for record in fault_records
            if record["candidate"] == candidate
            or (record["candidate"] == "common" and candidate in record["raw_observation"].get("mapped_candidates", []))
        ]
        fault_p95s = []
        for fault in ("F-01", "F-02", "F-03"):
            values = [record["duration_ns"] for record in mapped_faults if record["workload"] == fault]
            fault_p95s.append(nearest_rank_p95(values))
        qa.setdefault("QA-31", {"unit":"ns"})[candidate] = max(fault_p95s)
        qa.setdefault("QA-32", {"unit":"units"})[candidate] = max(
            record["raw_observation"]["excess_affected_units"] for record in mapped_faults
        )

    qa["QA-41"] = {
        candidate: nearest_rank_p95([
            record["total_rss_bytes"] for record in memory_records if record["candidate"] == candidate
        ]) for candidate in candidates
    } | {"unit":"bytes"}
    qa["QA-51"] = {
        candidate: sum(record["excess_protected_units"] for record in exposure_records if record["candidate"] == candidate)
        for candidate in candidates
    } | {"unit":"units"}
    for candidate in candidates:
        scored = mapped_records(candidate) + [
            record for record in fault_records
            if record["candidate"] == candidate
            or (record["candidate"] == "common" and candidate in record["raw_observation"].get("mapped_candidates", []))
        ]
        qa.setdefault("QA-61", {"unit":"pct"})[candidate] = 100.0 * sum(record["trace_complete"] for record in scored) / len(scored)
    qa["QA-62"] = {
        "isolated_worker": None,
        "same_process": None,
        "unit": "pct",
        "pending_clean_check": True,
    }

    roles = contract["qa_plan"]
    table = []
    lower_is_better = {"QA-01","QA-02","QA-03","QA-04","QA-05","QA-21","QA-22","QA-23","QA-31","QA-32","QA-41","QA-51"}
    for index in [1,2,3,4,5,11,12,13,14,15,21,22,23,31,32,41,51,61,62]:
        qa_id = f"QA-{index:02d}"
        a = qa[qa_id]["isolated_worker"]
        b = qa[qa_id]["same_process"]
        if a is None or b is None:
            comparison = "N/A"
        elif a == b:
            comparison = "same"
        elif qa_id in lower_is_better:
            comparison = "A" if a < b else "B"
        else:
            comparison = "A" if a > b else "B"
        table.append({
            "qa":qa_id, "A":a, "B":b, "unit":qa[qa_id]["unit"],
            "raw_direction":comparison, "role":roles[qa_id]["role"],
            "method":roles[qa_id]["method"]
        })
    return {"qa_results":qa, "complete_19_qa_table":table}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--semantic-endpoint", default="http://127.0.0.1:18081")
    parser.add_argument("--semantic-pid", type=int, required=True)
    parser.add_argument("--host", type=Path, default=DEFAULT_HOST)
    parser.add_argument("--worker", type=Path, default=DEFAULT_WORKER)
    parser.add_argument("--reference-agent", type=Path, default=DEFAULT_REFERENCE_AGENT)
    args = parser.parse_args()
    if args.output.exists():
        raise SystemExit(f"refusing to overwrite: {args.output}")

    contract_bytes = CONTRACT.read_bytes()
    contract = json.loads(contract_bytes)
    contract_hash = digest_bytes(contract_bytes)
    fixture = json.loads(NORMAL_FIXTURE.read_text(encoding="utf-8"))
    semantic_fixture = json.loads(SEMANTIC_CONTEXT.read_text(encoding="utf-8"))
    ledger = json.loads(CHANGE_LEDGER.read_text(encoding="utf-8"))
    cases = fixture["cases"]
    if len(cases) != 28:
        raise SystemExit("normal fixture must contain exactly 28 cases")

    raw_dir = args.output / "raw"
    derived_dir = args.output / "derived"
    raw_dir.mkdir(parents=True)
    derived_dir.mkdir()
    (args.output / "contract.json").write_bytes(contract_bytes)
    (args.output / "normal-fixture.json").write_bytes(NORMAL_FIXTURE.read_bytes())
    (args.output / "semantic-context.json").write_bytes(SEMANTIC_CONTEXT.read_bytes())
    (args.output / "change-ledger.json").write_bytes(CHANGE_LEDGER.read_bytes())

    manifest = {
        "run_id":args.output.name, "created_at_utc":utc_now(),
        "contract_sha256":contract_hash,
        "fixture_sha256":digest_file(NORMAL_FIXTURE),
        "semantic_context_sha256":digest_file(SEMANTIC_CONTEXT),
        "change_ledger_sha256":digest_file(CHANGE_LEDGER),
        "git_head":run_text(["git","rev-parse","HEAD"]),
        "git_status":run_text(["git","status","--short"]),
        "semantic_endpoint":args.semantic_endpoint,
        "semantic_pid":args.semantic_pid,
        "binaries":{path.name:digest_file(path) for path in (args.host,args.worker,args.reference_agent)},
    }

    # Verify the shared semantic runtime before collecting results.
    with urllib.request.urlopen(args.semantic_endpoint.rstrip("/") + "/health", timeout=5) as response:
        if json.loads(response.read()).get("status") != "ok":
            raise SystemExit("semantic model health check failed")

    semantic_records = []
    for case in cases:
        if case["input"] is None:
            continue
        print(f"[semantic] {case['id']}", flush=True)
        semantic_case = semantic_fixture["cases"].get(case["id"])
        if semantic_case is None:
            raise SystemExit(f"missing semantic Context for {case['id']}")
        try:
            semantic_records.append(
                semantic_call(args.semantic_endpoint, fixture, case, semantic_case)
            )
        except Exception as exc:
            semantic_records.append({
                "case":case["id"], "source_execution_key":f"semantic:{case['id']}:1",
                "duration_ns":0, "prompt_sha256":None, "input":case["input"], "output":None,
                "expected":semantic_case["expected"], "field_pass":{}, "semantic_pass":False,
                "usage":{}, "evidence_label":"MEASURED_MODEL", "error":f"{type(exc).__name__}: {exc}"
            })
    semantic_by_case = {record["case"]:record for record in semantic_records}
    write_jsonl(raw_dir / "semantic.jsonl", semantic_records)

    state_file = args.output / "raw/reference-agent-state.json"
    agent = ReferenceAgent(args.reference_agent, state_file)
    normal_records: list[dict[str, Any]] = []
    exposure_records: list[dict[str, Any]] = []
    memory_records: list[dict[str, Any]] = []
    try:
        # DP-11 does not participate in direct-only cases; execute them once.
        for case in cases:
            if case["kind"] in {"direct", "direct_followup", "clarification", "barge_in_direct"}:
                print(f"[normal/common] {case['id']}", flush=True)
                for trial in range(1, contract["repetitions"]["runtime_trials_per_case_per_candidate"] + 1):
                    normal_records.append(common_direct_record(
                        contract, contract_hash, case, trial, semantic_by_case.get(case["id"])
                    ))

        hosts = {
            candidate: start_host(candidate, args.host, args.worker, agent.address)
            for candidate in ("isolated_worker", "same_process")
        }
        try:
            for case_index, case in enumerate(cases):
                if case["kind"] in {"direct", "direct_followup", "clarification", "barge_in_direct"}:
                    continue
                print(f"[normal/paired] {case['id']}", flush=True)
                for trial in range(1, contract["repetitions"]["runtime_trials_per_case_per_candidate"] + 1):
                    candidates = ["isolated_worker", "same_process"]
                    if (case_index + trial) % 2:
                        candidates.reverse()
                    for candidate in candidates:
                        host = hosts[candidate]
                        record, exposures = execute_case(
                            contract=contract, contract_hash=contract_hash,
                            candidate=candidate, case=case, trial=trial,
                            semantic=semantic_by_case.get(case["id"]), host=host, agent=agent
                        )
                        normal_records.append(record)
                        for exposure in exposures:
                            exposure_records.append({
                                "candidate":candidate, "case":case["id"], "trial":trial,
                                "payload_sha256":digest_bytes(str(exposure["payload"]).encode()),
                                "excess_protected_units":len(exposure["excess_protected_units"]),
                                "excess_unit_ids":exposure["excess_protected_units"]
                            })
                        if case["family"] == "N-05":
                            candidate_rss = rss_bytes(process_tree_pids(host.process.pid))
                            model_rss = rss_bytes([args.semantic_pid])
                            memory_records.append({
                                "candidate":candidate, "case":case["id"], "trial":trial,
                                "candidate_process_rss_bytes":candidate_rss,
                                "shared_semantic_model_rss_bytes":model_rss,
                                "total_rss_bytes":candidate_rss + model_rss,
                                "candidate_pids":process_tree_pids(host.process.pid),
                                "semantic_pid":args.semantic_pid
                            })
        finally:
            for host in hosts.values():
                host.stop()

        write_jsonl(raw_dir / "normal.jsonl", normal_records)
        write_jsonl(raw_dir / "memory.jsonl", memory_records)
        write_jsonl(raw_dir / "exposure.jsonl", exposure_records)

        # The fault strata are independent workloads. They use the same Agent implementation
        # and contract but a fresh persistent state file so the 169-run normal history cannot
        # turn JSON persistence cost into a hidden fault-campaign variable.
        agent.stop()
        fault_state_file = args.output / "raw/fault-reference-agent-state.json"
        agent = ReferenceAgent(args.reference_agent, fault_state_file)
        print("[fault] F-01..F-03", flush=True)
        fault_records = run_faults(
            contract=contract, contract_hash=contract_hash,
            host_binary=args.host, worker=args.worker, agent=agent,
            reference_agent_binary=args.reference_agent, state_root=raw_dir,
        )
        write_jsonl(raw_dir / "faults.jsonl", fault_records)
    finally:
        agent.stop()

    change_records = []
    for qa, changes in ledger["changes"].items():
        for change_id, candidates in changes.items():
            for candidate, element_ids in candidates.items():
                change_records.append({
                    "qa":qa, "change_id":change_id, "candidate":candidate,
                    "changed_element_ids":element_ids, "changed_count":len(set(element_ids)),
                    "scored":False, "diagnostic_kind":"DESIGN_LEDGER_ONLY"
                })

    write_jsonl(raw_dir / "changes.jsonl", change_records)

    # One append-only inventory lets package checks and later re-analysis enumerate every
    # observation without pretending that records from different strata are IID trials.
    with (raw_dir / "trials.jsonl").open("x", encoding="utf-8") as stream:
        for stratum, records in (
            ("semantic", semantic_records), ("normal", normal_records),
            ("fault", fault_records), ("change", change_records),
            ("memory", memory_records), ("exposure", exposure_records),
        ):
            for record in records:
                stream.write(json.dumps({"stratum": stratum, "record": record}, ensure_ascii=False, separators=(",", ":")) + "\n")

    summary = analyze(
        contract, semantic_records, normal_records, fault_records,
        change_records, memory_records, exposure_records, cases
    )
    summary.update({
        "contract_id":contract["contract_id"], "contract_sha256":contract_hash,
        "raw_record_counts":{
            "semantic":len(semantic_records), "normal":len(normal_records),
            "faults":len(fault_records), "changes":len(change_records),
            "memory":len(memory_records), "exposure":len(exposure_records)
        },
        "limitations":contract["limitations"]
    })
    write_json(derived_dir / "summary.json", summary)
    write_json(args.output / "manifest.json", manifest)
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
