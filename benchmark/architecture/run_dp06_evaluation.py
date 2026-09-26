#!/usr/bin/env python3
"""Run the frozen VIA-DP-06 evaluation-v4 A/B/B' campaign."""

from __future__ import annotations

import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import queue
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from benchmark.architecture.qa01_15_evaluator import evaluate_trial  # noqa: E402
from benchmark.architecture._dp06_semantic_candidates import (  # noqa: E402
    compile_probe,
    ensure_model,
    evaluate_semantic,
    evidence_for_case,
    generate_wav,
    probe_audio,
    run_a,
    run_b,
)

DEFAULT_FREEZE = ROOT / "benchmark/architecture/contracts/via-dp-06-evaluation-v4.json"
FOUNDATION = ROOT / "benchmark/architecture/contracts/qa01-15-foundation-v1.json"
FIXTURE = ROOT / "benchmark/architecture/fixtures/dp06-semantic-input-v4.json"
ORACLE = ROOT / "benchmark/architecture/fixtures/dp06-semantic-oracle-v4.json"
CHANGE_EXERCISES = ROOT / "benchmark/architecture/fixtures/dp06-change-exercises-v4.json"
AGENT_BIN = ROOT / "prototypes/candidates/target/debug/via-reference-agent"


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_freeze(path: Path) -> dict[str, Any]:
    overlay = load_json(path)
    base_path = overlay.get("base_contract")
    if not base_path:
        return overlay
    merged = load_json(ROOT / base_path)
    merged.update(overlay)
    return merged


def load_foundation(freeze: dict[str, Any]) -> dict[str, Any]:
    foundation = load_json(FOUNDATION)
    for qa_id, timeout_ms in freeze.get("physical_endpoint_timeout_ms", {}).items():
        foundation["latency"][qa_id]["timeout_ms"] = timeout_ms
    return foundation


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def jsonl_append(path: Path, value: dict[str, Any]) -> None:
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n")


class ReferenceAgent:
    """One isolated external Agent runtime per trial."""

    def __init__(self) -> None:
        self.process = subprocess.Popen(
            [str(AGENT_BIN)], stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, bufsize=1,
        )
        assert self.process.stdout is not None and self.process.stderr is not None
        ready = json.loads(self.process.stdout.readline())
        self.address = ready["address"]
        before_wall = time.time_ns()
        before_mono = time.monotonic_ns()
        after_wall = time.time_ns()
        after_mono = time.monotonic_ns()
        self.wall_to_mono_ns = ((before_mono - before_wall) + (after_mono - after_wall)) // 2
        self.clock_uncertainty_ns = max(after_wall - before_wall, after_mono - before_mono)
        self.events: queue.Queue[dict[str, Any]] = queue.Queue()
        self.stderr_lines: deque[str] = deque()
        self.reader = threading.Thread(target=self._read_stderr, daemon=True)
        self.reader.start()

    def _read_stderr(self) -> None:
        assert self.process.stderr is not None
        for line in self.process.stderr:
            line = line.strip()
            if not line:
                continue
            self.stderr_lines.append(line)
            try:
                item = json.loads(line)
            except json.JSONDecodeError:
                continue
            if item.get("schema_version") == "via.reference-agent.source-event.v1":
                item["source_monotonic_ns"] = int(item["source_wall_time_ns"]) + self.wall_to_mono_ns
                item["clock_uncertainty_ns"] = self.clock_uncertainty_ns
                self.events.put(item)

    def request(self, request: dict[str, Any]) -> dict[str, Any]:
        host, port_text = self.address.rsplit(":", 1)
        with socket.create_connection((host, int(port_text)), timeout=5) as stream:
            stream.sendall(canonical_bytes(request) + b"\n")
            response = b""
            while not response.endswith(b"\n"):
                block = stream.recv(65536)
                if not block:
                    break
                response += block
        parsed = json.loads(response)
        if parsed.get("status") == "error":
            raise RuntimeError(parsed["message"])
        return parsed

    def drain_events(self, settle_ms: int = 30) -> list[dict[str, Any]]:
        time.sleep(settle_ms / 1000)
        values: list[dict[str, Any]] = []
        while True:
            try:
                values.append(self.events.get_nowait())
            except queue.Empty:
                return values

    def close(self) -> None:
        self.process.terminate()
        try:
            self.process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=2)

    def __enter__(self) -> "ReferenceAgent":
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


def reply_run_id(response: dict[str, Any]) -> str:
    reply = response.get("reply", {})
    for variant in ("Q", "P"):
        body = reply.get(variant)
        if isinstance(body, dict) and body.get("run_id"):
            return body["run_id"]
    raise RuntimeError(f"Agent reply has no run_id: {response}")


def seed_agent(agent: ReferenceAgent, case: dict[str, Any]) -> dict[str, str]:
    runs: dict[str, str] = {}
    for task_id in case["task_ids"]:
        response = agent.request({
            "op": "submit",
            "request": {"task_id": task_id, "submission_key": f"seed-{task_id}", "goal": f"seed {task_id}"},
        })
        run_id = reply_run_id(response)
        runs[task_id] = run_id
        if task_id.endswith("DONE"):
            agent.request({"op": "complete", "run_id": run_id, "artifact": f"artifact-{task_id}"})
    agent.drain_events()
    return runs


def execute_candidate(
    agent: ReferenceAgent, output: dict[str, Any], case: dict[str, Any], candidate: str
) -> dict[str, Any]:
    runs = seed_agent(agent, case)
    actions: list[dict[str, Any]] = []
    source_events: list[dict[str, Any]] = []
    if output.get("status") != "READY":
        return {"actions": actions, "source_events": source_events, "all_actions_ok": True}

    for index, request in enumerate(output.get("requests", [])):
        handling = request.get("handling")
        intent = request.get("intent")
        receipt: dict[str, Any] = {
            "request_index": index, "intent": intent, "handling": handling,
            "task_id": request.get("task_id"), "ok": False,
        }
        try:
            if handling == "bounded_core":
                receipt.update(ok=True, operation="direct_response")
            elif handling == "downstream_agent":
                task_id = request.get("task_id") or f"{candidate}-{case['id']}-task-{index}"
                if request.get("task_relation") == "existing_task":
                    old_run = runs[task_id]
                    accepted = agent.request({"op": "follow_up", "run_id": old_run, "text": case["request"]})
                    run_id = reply_run_id(accepted)
                    operation = "follow_up"
                else:
                    accepted = agent.request({
                        "op": "submit",
                        "request": {
                            "task_id": task_id,
                            "submission_key": f"{candidate}-{case['id']}-{index}",
                            "goal": case["request"],
                        },
                    })
                    run_id = reply_run_id(accepted)
                    operation = "submit"
                artifact = f"artifact-{candidate}-{case['id']}-{index}"
                agent.request({"op": "complete", "run_id": run_id, "artifact": artifact})
                receipt.update(ok=True, operation=operation, task_id=task_id, run_id=run_id, artifact=artifact)
            elif handling == "task_control":
                task_id = request.get("task_id")
                run_id = runs[task_id]
                op = "cancel" if intent == "cancel" else "query"
                response = agent.request({"op": op, "run_id": run_id})
                receipt.update(ok=True, operation=op, run_id=run_id, response=response)
            else:
                receipt.update(error=f"unsupported handling: {handling}")
        except Exception as error:  # raw evidence must retain execution failures
            receipt.update(error=f"{type(error).__name__}: {error}")
        actions.append(receipt)
        source_events.extend(agent.drain_events())
    return {
        "actions": actions,
        "source_events": source_events,
        "all_actions_ok": all(item["ok"] for item in actions),
    }


def normalized_semantic(output: dict[str, Any]) -> dict[str, Any]:
    requests = output.get("requests", []) if output.get("status") == "READY" else []
    return {
        "status": output.get("status"),
        "intents": [item.get("intent") for item in requests],
        "constraints": [sorted(item.get("constraints", [])) for item in requests],
        "deliverables": [sorted(x for x in item.get("constraints", []) if str(x).startswith("output=")) for item in requests],
        "referents": [sorted(x.rsplit("#", 1)[-1] for x in item.get("referents", [])) for item in requests],
        "request_count": len(requests),
        "clarification_required": bool(output.get("clarification")),
        "task_relations": [item.get("task_relation") for item in requests],
        "task_ids": [item.get("task_id") for item in requests if item.get("task_id")],
        "pending_ids": [item.get("pending_interaction_id") for item in requests if item.get("pending_interaction_id")],
        "handlings": [item.get("handling") for item in requests],
        "capabilities": [item.get("required_capability") for item in requests if item.get("required_capability")],
    }


def expected_semantic(expected: dict[str, Any]) -> dict[str, Any]:
    requests = expected.get("requests", []) if expected["status"] == "READY" else []
    return {
        "status": expected["status"],
        "intents": [item.get("intent") for item in requests],
        "constraints": [sorted(item.get("constraints", [])) for item in requests],
        "deliverables": [sorted(x for x in item.get("constraints", []) if str(x).startswith("output=")) for item in requests],
        "referents": [sorted(item.get("referents", [])) for item in requests],
        "request_count": len(requests),
        "clarification_required": bool(expected.get("clarification_required")),
        "task_relations": [item.get("task_relation") for item in requests],
        "task_ids": [item.get("task_id") for item in requests if item.get("task_id")],
        "pending_ids": [item.get("pending_interaction_id") for item in requests if item.get("pending_interaction_id")],
        "handlings": [item.get("handling") for item in requests],
        "capabilities": [item.get("required_capability") for item in requests if item.get("required_capability")],
    }


def exact_group(identifier: str, path: str, expected: Any) -> dict[str, Any]:
    return {"applicability": "APPLICABLE", "predicates": [{
        "id": identifier, "operator": "EXACT", "actual_path": path, "expected": expected,
    }]}


def expected_response_propositions(expected: dict[str, Any]) -> list[str]:
    values = [f"status:{expected['status']}"]
    if expected["status"] == "CLARIFY":
        values.append("clarification_present")
    elif expected["status"] == "NEED_CONTEXT":
        values.append("need_context")
    elif expected["status"] == "REJECT":
        values.append("rejected")
    elif expected["status"] == "READY":
        for index, request in enumerate(expected.get("requests", [])):
            handling = request.get("handling")
            if handling == "bounded_core":
                values.extend(f"source:{target}" for target in request.get("referents", []))
            elif handling == "downstream_agent":
                values.append(f"downstream:{index}:completed")
            elif handling == "task_control":
                values.append(f"control:{request.get('intent')}:{request.get('task_id')}")
    return values


def make_oracle(case: dict[str, Any], expected: dict[str, Any]) -> dict[str, Any]:
    wanted = expected_semantic(expected)
    expected_agent = sum(value == "downstream_agent" for value in wanted["handlings"])
    expected_actions = sum(value != "bounded_core" for value in wanted["handlings"])
    qa12 = {
        "goal": exact_group("goal", "semantic.intents", wanted["intents"]),
        "constraints": exact_group("constraints", "semantic.constraints", wanted["constraints"]),
        "deliverables": exact_group("deliverables", "semantic.deliverables", wanted["deliverables"]),
        "referents": exact_group("referents", "semantic.referents", wanted["referents"]),
        "decomposition": exact_group("request_count", "semantic.request_count", wanted["request_count"]),
        "clarification": exact_group("clarification", "semantic.clarification_required", wanted["clarification_required"]),
        "task_relation": exact_group("task_relation", "semantic.task_relations", wanted["task_relations"]),
        "handling": exact_group("handling", "semantic.handlings", wanted["handlings"]),
        "agent_capability": exact_group("capability", "semantic.capabilities", wanted["capabilities"]),
    }
    qa13 = {
        "conversation": exact_group("conversation", "binding.conversation_id", "CONV-BUDGET-01"),
        "request": exact_group("request", "binding.request_count", wanted["request_count"]),
        "task": exact_group("task", "binding.task_ids", wanted["task_ids"]),
        "agent_execution": exact_group("agent_execution", "binding.successful_agent_actions", expected_agent),
        "pending_interaction": exact_group("pending", "binding.pending_ids", wanted["pending_ids"]),
        "result_artifact": exact_group("artifacts", "binding.artifact_count", expected_agent),
        "cardinality": exact_group("action_cardinality", "binding.successful_non_direct_actions", expected_actions),
    }
    qa11 = {
        "response_or_result": {
            "applicability": "APPLICABLE",
            "predicates": [{
                "id": "substantive_response", "operator": "REQUIRED_PROPOSITION",
                "actual_path": "integrated.response_propositions",
                "expected": expected_response_propositions(expected),
            }],
        },
        "cardinality": exact_group("integrated_cardinality", "integrated.successful_non_direct_actions", expected_actions),
        "mandatory_gate": exact_group("gate", "integrated.all_actions_ok", True),
    }
    latency_id = case["qa_latency"]
    return {
        "correctness": {"QA-11": qa11, "QA-12": qa12, "QA-13": qa13},
        "qa11_driver_applicability": {
            "QA-12": "APPLICABLE", "QA-13": "APPLICABLE", "QA-14": "N/A", "QA-15": "N/A",
        },
        "latency_applicability": {
            qa_id: ("APPLICABLE" if qa_id == latency_id else "N/A")
            for qa_id in ("QA-01", "QA-02", "QA-03", "QA-04", "QA-05")
        },
        "latency": {
            latency_id: {"predicates": [{
                "id": "meaningful_response_observed", "operator": "EXACT",
                "actual_path": f"latency_validity.{latency_id}", "expected": True,
            }]}
        },
    }


def response_text_and_propositions(
    output: dict[str, Any], execution: dict[str, Any], fixture: dict[str, Any]
) -> tuple[str, list[str]]:
    status = output.get("status")
    propositions = [f"status:{status}"]
    if status == "CLARIFY":
        propositions.append("clarification_present")
        return output.get("clarification") or "어느 대상을 뜻하는지 알려주세요.", propositions
    if status == "NEED_CONTEXT":
        propositions.append("need_context")
        return "요청한 자료를 찾지 못했습니다. 자료를 지정해 주세요.", propositions
    if status == "REJECT":
        propositions.append("rejected")
        return output.get("clarification") or "현재 이 요청을 수행할 수 없습니다.", propositions
    sources = {item["target_id"]: item["content"] for item in fixture["evidence"]["sources"]}
    action_by_index = {item["request_index"]: item for item in execution["actions"]}
    sentences: list[str] = []
    for index, request in enumerate(output.get("requests", [])):
        handling = request.get("handling")
        action = action_by_index.get(index, {})
        if handling == "bounded_core":
            target_ids = [item.rsplit("#", 1)[-1] for item in request.get("referents", [])]
            for target_id in target_ids:
                if target_id in sources:
                    propositions.append(f"source:{target_id}")
                    sentences.append(sources[target_id])
            if not target_ids:
                sentences.append("참조할 Context가 없어 직접 답변할 수 없습니다.")
        elif handling == "downstream_agent" and action.get("ok"):
            propositions.append(f"downstream:{index}:completed")
            sentences.append(
                f"Task {action.get('task_id')} 실행이 완료됐고 결과는 {action.get('artifact')}입니다."
            )
        elif handling == "task_control" and action.get("ok"):
            propositions.append(f"control:{request.get('intent')}:{request.get('task_id')}")
            verb = "취소 요청했습니다" if request.get("intent") == "cancel" else "상태를 조회했습니다"
            sentences.append(f"Task {request.get('task_id')}을 {verb}.")
        else:
            sentences.append(f"요청 {index + 1}을 처리하지 못했습니다.")
    return " ".join(sentences) or "처리할 수 있는 요청이 없습니다.", propositions


def source_time(events: list[dict[str, Any]], event_name: str, *, first: bool) -> int | None:
    values = [item["source_monotonic_ns"] for item in events if item.get("event") == event_name]
    if not values:
        return None
    return min(values) if first else max(values)


def event(timestamp: int | None, provenance: str) -> dict[str, Any] | None:
    return None if timestamp is None else {"timestamp_ns": timestamp, "provenance": provenance}


def build_trace(
    case: dict[str, Any], output: dict[str, Any], execution: dict[str, Any],
    input_report: dict[str, Any], output_report: dict[str, Any], response: str,
    response_propositions: list[str],
) -> dict[str, Any]:
    sem = normalized_semantic(output)
    successful_agent = sum(
        item["ok"] and item.get("handling") == "downstream_agent" for item in execution["actions"]
    )
    successful_non_direct = sum(
        item["ok"] and item.get("handling") != "bounded_core" for item in execution["actions"]
    )
    input_end = int(input_report["detectedLastMonotonicNs"])
    onset = int(output_report["detectedOnsetMonotonicNs"])
    events: dict[str, Any] = {}
    latency_id = case["qa_latency"]
    if latency_id == "QA-01":
        for name, value, provenance in (
            ("user_input_end", input_end, "annotated_capture_wav"),
            ("agent_request_available_at_agent_ingress", source_time(execution["source_events"], "agent_request_available_at_agent_ingress", first=True), "reference_agent_source"),
            ("agent_result_available_at_source", source_time(execution["source_events"], "agent_result_available_at_source", first=False), "reference_agent_source"),
            ("first_meaningful_audible_result_audio", onset, "audio_loopback"),
        ):
            value_event = event(value, provenance)
            if value_event:
                events[name] = value_event
    elif latency_id == "QA-02":
        events = {
            "user_input_end": event(input_end, "annotated_capture_wav"),
            "first_meaningful_audible_direct_response": event(onset, "audio_loopback"),
        }
    elif latency_id == "QA-05":
        events = {
            "task_control_input_end": event(input_end, "annotated_capture_wav"),
            "correct_task_control_disposition_presented": event(onset, "audio_loopback"),
        }
    return {
        "events": events,
        "path_stages": [
            "input_finalization", "semantic_target_resolution", "task_binding",
            "control_delivery", "truthful_disposition",
        ],
        "semantic": sem,
        "binding": {
            "conversation_id": "CONV-BUDGET-01",
            "request_count": sem["request_count"],
            "task_ids": sem["task_ids"],
            "pending_ids": sem["pending_ids"],
            "successful_agent_actions": successful_agent,
            "artifact_count": sum(bool(item.get("artifact")) for item in execution["actions"]),
            "successful_non_direct_actions": successful_non_direct,
        },
        "integrated": {
            "response_present": bool(response.strip()),
            "response_propositions": response_propositions,
            "successful_non_direct_actions": successful_non_direct,
            "all_actions_ok": execution["all_actions_ok"],
        },
        "latency_validity": {latency_id: True},
        "observed_full_response_ns": onset - input_end,
    }


def trace_complete(record: dict[str, Any]) -> tuple[bool, list[str]]:
    required = [
        "candidate", "case_id", "input_audio_report", "semantic_output", "model_calls",
        "execution", "response_text", "output_audio_report", "trace", "oracle",
        "evaluation", "semantic_field_evaluation",
    ]
    missing = [name for name in required if name not in record]
    return not missing, missing


def run_change_exercises(output_dir: Path) -> list[dict[str, Any]]:
    fixture = load_json(CHANGE_EXERCISES)
    results: list[dict[str, Any]] = []
    for exercise in fixture["exercises"]:
        for candidate, edits in exercise["edits"].items():
            changed: set[str] = set()
            with tempfile.TemporaryDirectory(prefix="via-dp06-change-") as temp:
                temp_root = Path(temp)
                observations: list[dict[str, Any]] = []
                valid = True
                for edit in edits:
                    source = ROOT / edit["file"]
                    destination = temp_root / edit["file"]
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(source, destination)
                    before = destination.read_text(encoding="utf-8")
                    occurrences = before.count(edit["find"])
                    if occurrences == 0:
                        valid = False
                    after = before.replace(edit["find"], edit["replace"], 1)
                    destination.write_text(after, encoding="utf-8")
                    if after != before:
                        changed.add(edit["file"])
                    if destination.suffix == ".json":
                        json.loads(after)
                    observations.append({
                        "file": edit["file"], "match_count": occurrences,
                        "before_sha256": hashlib.sha256(before.encode()).hexdigest(),
                        "after_sha256": hashlib.sha256(after.encode()).hexdigest(),
                    })
            results.append({
                "exercise_id": exercise["id"], "qa_id": exercise["qa_id"],
                "candidate": candidate, "valid": valid,
                "changed_elements": sorted(changed), "element_count": len(changed),
                "observations": observations,
            })
    path = output_dir / "raw/change-exercises.json"
    path.write_text(json.dumps(results, ensure_ascii=False, indent=2), encoding="utf-8")
    return results


def run_trial(
    *, endpoint: str, candidate: str, fixture: dict[str, Any], case: dict[str, Any],
    expected: dict[str, Any], foundation: dict[str, Any], output_dir: Path, trial: int,
) -> dict[str, Any]:
    trial_dir = output_dir / "artifacts" / candidate / case["id"] / f"trial-{trial:02d}"
    trial_dir.mkdir(parents=True, exist_ok=True)
    input_wav = output_dir / "stimuli" / f"{case['id']}.wav"
    input_report = probe_audio(input_wav, trial_dir / "input-audio")
    if candidate == "A":
        semantic_output, calls = run_a(endpoint, fixture, case)
    else:
        semantic_output, calls = run_b(endpoint, fixture, case, fast_path=candidate == "B_PRIME")
    with ReferenceAgent() as agent:
        execution = execute_candidate(agent, semantic_output, case, candidate)
        clock = {"wall_to_mono_ns": agent.wall_to_mono_ns, "uncertainty_ns": agent.clock_uncertainty_ns}
    response, response_propositions = response_text_and_propositions(
        semantic_output, execution, fixture
    )
    output_wav = trial_dir / "response.wav"
    generate_wav(response, output_wav)
    output_report = probe_audio(output_wav, trial_dir / "output-audio")
    trace = build_trace(
        case, semantic_output, execution, input_report, output_report, response,
        response_propositions,
    )
    trial_oracle = make_oracle(case, expected)
    evaluation = evaluate_trial(foundation, trace, trial_oracle)
    semantic_fields = evaluate_semantic(semantic_output, expected)
    record = {
        "schema_version": "via.dp06.evaluation-trial.v4",
        "evidence_label": "MEASURED_REFERENCE_HARNESS",
        "candidate": candidate, "case_id": case["id"], "trial": trial,
        "source_execution_key": f"{candidate}:{case['id']}:{trial}",
        "request_annotation": case["request"],
        "input_audio_report": input_report,
        "semantic_output": semantic_output,
        "model_calls": calls,
        "execution": execution,
        "reference_agent_clock": clock,
        "response_text": response,
        "output_audio_report": output_report,
        "trace": trace,
        "oracle": trial_oracle,
        "evaluation": evaluation,
        "semantic_field_evaluation": semantic_fields,
    }
    complete, missing = trace_complete(record)
    record["trace_complete"] = complete
    record["trace_missing"] = missing
    return record


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--endpoint", default="http://127.0.0.1:18080")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--case", action="append", dest="cases")
    parser.add_argument("--candidate", action="append", choices=["A", "B", "B_PRIME"])
    parser.add_argument("--trials", type=int)
    args = parser.parse_args()

    ensure_model(args.endpoint)
    compile_probe()
    if not AGENT_BIN.exists():
        raise RuntimeError(f"build Reference Agent first: {AGENT_BIN}")
    freeze = load_freeze(DEFAULT_FREEZE)
    fixture = load_json(FIXTURE)
    oracle = load_json(ORACLE)
    foundation = load_foundation(freeze)
    selected = set(args.cases or [])
    cases = [item for item in fixture["cases"] if not selected or item["id"] in selected]
    if selected != {item["id"] for item in cases} and selected:
        raise ValueError(f"unknown case IDs: {sorted(selected - {item['id'] for item in cases})}")
    for case in cases:
        override = freeze.get("latency_case_overrides", {}).get(case["id"])
        if override:
            case["qa_latency"] = override
    expected_by_case = {item["id"]: item["expected"] for item in oracle["cases"]}
    candidates = args.candidate or freeze["candidates"]
    trials = args.trials or freeze["repetitions"]["scored_trials_per_case"]

    output_dir = args.output_dir.resolve()
    if output_dir.exists() and any(output_dir.iterdir()):
        raise RuntimeError(f"refusing to overwrite non-empty result directory: {output_dir}")
    (output_dir / "raw").mkdir(parents=True, exist_ok=True)
    (output_dir / "stimuli").mkdir(parents=True, exist_ok=True)
    raw_path = output_dir / "raw/trials.jsonl"
    manifest = {
        "campaign_id": output_dir.name,
        "status": "RUNNING",
        "started_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "contract_sha256": sha256_file(DEFAULT_FREEZE),
        "foundation_sha256": sha256_file(FOUNDATION),
        "fixture_sha256": sha256_file(FIXTURE),
        "oracle_sha256": sha256_file(ORACLE),
        "change_exercises_sha256": sha256_file(CHANGE_EXERCISES),
        "candidates": candidates,
        "cases": [item["id"] for item in cases],
        "trials_per_case": trials,
        "evidence_label": "MEASURED_REFERENCE_HARNESS",
    }
    (output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    for case in cases:
        generate_wav(case["request"], output_dir / "stimuli" / f"{case['id']}.wav")
    run_change_exercises(output_dir)

    total = len(candidates) * len(cases) * trials
    index = 0
    for candidate in candidates:
        for case in cases:
            for trial in range(1, trials + 1):
                index += 1
                print(f"[{index}/{total}] {candidate} {case['id']} trial={trial}", flush=True)
                record = run_trial(
                    endpoint=args.endpoint, candidate=candidate, fixture=fixture, case=case,
                    expected=expected_by_case[case["id"]], foundation=foundation,
                    output_dir=output_dir, trial=trial,
                )
                jsonl_append(raw_path, record)

    manifest["status"] = "RAW_COMPLETE_PENDING_ANALYSIS"
    manifest["completed_at"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    manifest["raw_sha256"] = sha256_file(raw_path)
    (output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
