"""Two fresh-process probes of a published M05 token's actual persistence.

This is evaluator code. It reads the original independent M05 change source
without changing it. Scripted response text cannot establish semantic accuracy.
"""
from __future__ import annotations
import argparse
import asyncio
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import traceback


def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def write(path, record):
    path = Path(path)
    if path.exists(): raise FileExistsError("never overwrite supplemental evidence: " + str(path))
    path.write_text(json.dumps(record, ensure_ascii=False, indent=2, default=str) + "\n")


async def phase(source, variant, output, restart=False):
    source, output = Path(source).resolve(), Path(output).resolve()
    sys.path.insert(0, str(source))
    from prototypes.candidates.dp41_45_poc.shared import runtime as rm
    from prototypes.candidates.dp41_45_poc.shared import adapters
    assert Path(rm.__file__).resolve().is_relative_to(source)
    assert Path(adapters.__file__).resolve().is_relative_to(source)
    class Evidence:
        def __init__(self): self.origin, self.rows = time.monotonic(), []
        def seconds(self): return time.monotonic() - self.origin
        def write(self, table, row):
            self.rows.append({"table": table, **copy.deepcopy(row)})
            return row
    class Model:
        def __init__(self): self.calls = []
        async def call(self, messages, role, **kwargs):
            self.calls.append({"role": role, "messages": copy.deepcopy(messages)})
            return {"text": "상반기 범위를 사용합니까?"}
    evidence, model = Evidence(), Model()
    snapshot_ref = output / "published-owner-and-binding-snapshot.json"
    if restart:
        persisted = json.loads(snapshot_ref.read_text())
        initial = persisted["owner_initial"]
    else:
        initial = {"sources": [], "tasks": [{"task_id": "work", "execution_id": "work-exec",
            "request_revision": 1, "status": "RUNNING", "instruction": "실제 Task", "source_refs": [],
            "properties": {"PERIOD": "Q3"}}], "voice": {"on": False}}
    runtime = rm.LocalVIA(initial, model, evidence, variant, stub_profile={
        "acceptance_receipt_seconds": .001, "final_result_seconds": .005})
    try:
        if not restart:
            await runtime.external_event({"event_id": "new-owner-question", "type": "QUESTION", "payload": {
                "question": {"question_id": "published-q", "revision": 2, "task_id": "work",
                    "execution_id": "work-exec", "request_revision": 1, "status": "VALID",
                    "text": "상반기 범위를 사용합니까?", "scope": {"property": "PERIOD", "proposed_value": "H1"}}}})
            while runtime.jobs:
                await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .5)
                await asyncio.sleep(0)
            owner = runtime.owners.questions["published-q"]
            token = owner.get("allowed_answer_scope_token")
            assert isinstance(token, str) and token, "actual Task owner did not issue a token"
            presented = runtime.presented_questions.snapshot()
            actual = presented["published-q"]
            assert actual["question"]["allowed_answer_scope_token"] == token
            assert actual["receipt"]["publication_id"] in {d["publication_id"] for d in runtime.deliveries}
            assert actual["receipt"]["delivered_receipt"] is True and owner["presentation_status"] == "PRESENTED"
            assert any(e.get("kind") == "QUESTION_PRESENTED" for e in evidence.rows)
            owner_initial = {"sources": copy.deepcopy(runtime.owners.sources),
                "tasks": copy.deepcopy(list(runtime.owners.tasks.values())),
                "questions": copy.deepcopy(list(runtime.owners.questions.values())),
                "interactions": copy.deepcopy(runtime.owners.interactions), "voice": {"on": False}}
            write(snapshot_ref, {"owner_initial": owner_initial, "dialogue_binding": presented,
                "question_id": "published-q", "task_id": "work", "token": token,
                "actual_publication_id": actual["receipt"]["publication_id"], "producer_process_pid": os.getpid()})
            result = {"passed": True, "token_sha256": hashlib.sha256(token.encode()).hexdigest(),
                "publication_id": actual["receipt"]["publication_id"], "snapshot_sha256": sha(snapshot_ref)}
        else:
            assert persisted["producer_process_pid"] != os.getpid(), "restart did not use a fresh process"
            owner = runtime.owners.questions["published-q"]
            token = owner.get("allowed_answer_scope_token")
            assert token == persisted["token"], "owner restore replaced the actually published token"
            # Constructor's owner reload must retain the token before the
            # existing generic binding persistence interface is exercised.
            constructor_binding = runtime.presented_questions.snapshot()["published-q"]
            assert constructor_binding["question"]["allowed_answer_scope_token"] == token
            runtime.presented_questions.restore(persisted["dialogue_binding"])
            restored = runtime.presented_questions.snapshot()["published-q"]
            assert restored["question"]["allowed_answer_scope_token"] == token
            assert restored["receipt"]["publication_id"] == persisted["actual_publication_id"]
            assert not runtime.deliveries, "restart pretended to present a new question"
            ports = rm.Ports(runtime, "answer-after-restart", "answer-input", "네")
            goal = {"id": "answer", "kind": "ANSWER_QUESTION", "operation": "ANSWER_QUESTION", "target": "work",
                    "arguments": {"question_ref": "published-q", "question_revision": 2,
                        "answer": "yes", "properties": {"PERIOD": "H1"}}, "instruction": "실제 제시 질문 응답"}
            meaning = {"version": "dp41-meaning-runtime-1", "goals": [goal], "relations": [],
                       "constraints": [], "preservations": [], "response": None, "evidence_refs": [], "unresolved": []}
            # A corrupted dialogue token is rejected by the real Task answer
            # gate, before any external command or question consumption.
            wrong = copy.deepcopy(persisted["dialogue_binding"])
            wrong["published-q"]["question"]["allowed_answer_scope_token"] = "not-the-owner-issued-token"
            runtime.presented_questions.restore(wrong)
            try: await runtime.perform_goal(copy.deepcopy(goal), meaning, ports)
            except ValueError: pass
            else: raise AssertionError("Task accepted a token absent from its actual published record")
            assert not runtime.effects and owner["status"] == "VALID"
            runtime.presented_questions.restore(persisted["dialogue_binding"])
            await runtime.perform_goal(copy.deepcopy(goal), meaning, ports)
            while runtime.jobs:
                await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .5)
                await asyncio.sleep(0)
            assert len(runtime.effects) == 1
            effect = runtime.effects[0]
            assert effect["operation"] == "ANSWER_QUESTION" and effect["task_id"] == "work"
            assert effect["payload"]["question"]["allowed_answer_scope_token"] == token
            assert owner["status"] == "ANSWERED" and runtime.owners.tasks["work"]["properties"]["PERIOD"] == "H1"
            duplicate_ports = rm.Ports(runtime, "duplicate-after-restart", "duplicate-input", "네")
            try: await runtime.perform_goal(copy.deepcopy(goal), meaning, duplicate_ports)
            except ValueError: pass
            else: raise AssertionError("restored question was consumed twice")
            assert len(runtime.effects) == 1
            result = {"passed": True, "same_published_token": True, "corrupt_dialogue_token_rejected": True,
                "one_actual_answer_command": True, "duplicate_rejected": True, "owner_status": owner["status"],
                "token_sha256": hashlib.sha256(token.encode()).hexdigest(), "snapshot_sha256": sha(snapshot_ref)}
        return {**result, "pid": os.getpid(), "variant": variant, "events": evidence.rows,
            "effects": runtime.effects, "owner_questions": runtime.owners.questions,
            "binding": runtime.presented_questions.snapshot(), "deliveries": runtime.deliveries,
            "model_calls": model.calls, "api_calls": 0, "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION",
            "imported_runtime": str(Path(rm.__file__).resolve()), "imported_adapters": str(Path(adapters.__file__).resolve()),
            "source_sha256": {str(p.relative_to(source)): sha(p) for p in [Path(rm.__file__), Path(adapters.__file__)]}}
    finally:
        for task in list(runtime.jobs): task.cancel()
        await asyncio.gather(*list(runtime.jobs), return_exceptions=True)


def run_supplement(source, variant, output):
    source, output = Path(source).resolve(), Path(output).resolve()
    if output.exists(): raise FileExistsError("never overwrite supplemental output")
    output.mkdir(parents=True)
    before = {str(p.relative_to(source)): sha(p) for p in source.rglob("*")
              if p.is_file() and "__pycache__" not in p.parts}
    records = []
    env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}
    for name in ["publish", "restart"]:
        try:
            process = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--source", str(source),
                "--variant", variant, "--output", str(output), "--phase", name], env=env, cwd=source,
                capture_output=True, text=True, timeout=15)
            stdout, stderr = process.stdout, process.stderr
            if process.returncode == 0: record = json.loads(stdout)
            else: record = {"passed": False, "returncode": process.returncode}
        except subprocess.TimeoutExpired as error:
            stdout, stderr = str(error.stdout), str(error.stderr)
            record = {"passed": False, "error": "PROCESS_WATCHDOG_TIMEOUT"}
        write(output / (name + "-raw.json"), {"record": record, "stdout": stdout, "stderr": stderr})
        records.append(record)
        if record.get("passed") is not True: break
    after = {str(p.relative_to(source)): sha(p) for p in source.rglob("*")
             if p.is_file() and "__pycache__" not in p.parts}
    result = {"status": "PASSED" if len(records) == 2 and all(r.get("passed") for r in records) and before == after else "FAILED",
              "source_unchanged": before == after, "source": str(source), "variant": variant, "api_calls": 0,
              "helper_sha256": sha(__file__), "source_manifest": before, "raw_refs": [str(output / (name + "-raw.json"))
                  for name in ["publish", "restart"] if (output / (name + "-raw.json")).is_file()],
              "responsibility_review": "AWAITING_INDEPENDENT_REVIEW", "records": records}
    write(output / "result.json", result)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True)
    parser.add_argument("--variant", choices=["A", "B"], required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--phase", choices=["publish", "restart"])
    args = parser.parse_args()
    try:
        result = asyncio.run(phase(args.source, args.variant, args.output, args.phase == "restart")) if args.phase else run_supplement(args.source, args.variant, args.output)
        print(json.dumps(result, ensure_ascii=False, default=str))
        sys.exit(0 if result.get("passed", result.get("status") == "PASSED") else 1)
    except Exception as error:
        print(json.dumps({"passed": False, "error": f"{type(error).__name__}: {error}", "traceback": traceback.format_exc(),
            "api_calls": 0, "phase": args.phase}, ensure_ascii=False))
        sys.exit(1)
