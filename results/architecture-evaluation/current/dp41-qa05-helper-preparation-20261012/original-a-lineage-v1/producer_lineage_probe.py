"""Same actual producer-revision probe for original and corrected M04 copies.

No API. Correct semantic documents are scripted solely to isolate the execution
contract. Failures are preserved; this probe never repairs the candidate source.
"""
from __future__ import annotations
import argparse
import asyncio
import copy
import hashlib
import json
from pathlib import Path
import sys
import time
import traceback


def source_manifest(source):
    return {str(p.relative_to(source)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(source.rglob("*")) if p.is_file() and "__pycache__" not in p.parts and p.suffix != ".pyc"}


async def run(source, variant, representation, mapping_scope):
    source = Path(source).resolve()
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(source))
    from prototypes.candidates.dp41_45_poc.shared import runtime as rm
    from prototypes.candidates.dp41_45_poc.dp41 import contract
    assert Path(rm.__file__).resolve().is_relative_to(source)
    assert Path(contract.__file__).resolve().is_relative_to(source)
    before = source_manifest(source)
    observations, checks = [], {}

    class Evidence:
        def __init__(self): self.origin, self.rows = time.monotonic(), []
        def seconds(self): return time.monotonic() - self.origin
        def write(self, table, row):
            self.rows.append({"table": table, **copy.deepcopy(row)})
            return row

    def proposed():
        predicate = {"field": "STATE", "comparison": "EQ", "value": "FAILED"}
        if representation == "original": predicate["producer_goal_id"] = "report"
        else: predicate["reference_id"] = {"producer_goal_id": "report"}
        return {"version": contract.MEANING_VERSION,
            "goals": [{"id": gid, "kind": "NEW_TASK", "operation": operation, "target": None,
                "arguments": {"source_refs": ["doc@v1"]}, "instruction": "구조 검증 입력"}
                for gid, operation in [("report", "CREATE_REPORT"), ("draft", "DRAFT_MAIL")]],
            "relations": [{"op": "WHEN", "from_goal": "report", "to_goal": "draft", "predicate": predicate}],
            "constraints": [{"op": "FORBID_ACTION", "scope": ["report", "draft"], "action": "SEND"}],
            "preservations": [], "response": None, "evidence_refs": [], "unresolved": []}

    def frame():
        item = proposed()
        item.update(version=contract.FRAME_VERSION, references=[{"id": "source", "selector": {
            "op": "KNOWN_REF", "source_ref": "doc@v1"}, "cardinality": "ONE"}])
        for goal in item["goals"]: goal["arguments"]["source_refs"] = ["source"]
        return item

    class Model:
        def __init__(self): self.calls = []
        async def call(self, messages, role, **kwargs):
            output = frame() if role == "CINTERPRET" else {"text": "실제 실행 사실 전달"}
            self.calls.append({"messages": copy.deepcopy(messages), "role": role, "output": copy.deepcopy(output)})
            return output

    def build():
        evidence, model = Evidence(), Model()
        runtime = rm.LocalVIA({"sources": [{"source_id": "doc", "revision": 1, "kind": "DOCUMENT",
            "title": "doc", "text": "실제 원문", "permission": {"read": True, "model": True, "agent": True},
            "metadata": {}}], "tasks": [], "voice": {"on": False}}, model, evidence, variant,
            stub_profile={"acceptance_receipt_seconds": .001, "final_result_seconds": .06})
        return runtime, evidence, model

    async def collect(runtime, evidence, model, name):
        for task in list(runtime.jobs): task.cancel()
        await asyncio.gather(*list(runtime.jobs), return_exceptions=True)
        observations.append({"probe": name, "events": evidence.rows, "effects": runtime.effects,
            "owners": copy.deepcopy(runtime.owners.tasks), "scripted_model_calls": model.calls,
            "mapping": [{"key": list(key), "binding": copy.deepcopy(value)} for key, value in runtime.goal_tasks.items()],
            "command_fingerprints": [{"key": list(key), "command": copy.deepcopy(value)}
                                     for key, value in runtime.sent_commands.items()]})

    async def perform_meaning(runtime, ports):
        meaning = proposed()
        if variant == "B":
            from prototypes.candidates.dp41_45_poc.dp41.b import resolve
            contract.validate_document(frame(), frame=True)
            meaning = await resolve(runtime.interpretation_input(ports), ports)
            assert meaning["relations"] == proposed()["relations"], "scripted Frame lost the producer condition"
            assert all(g["arguments"]["source_refs"] == ["doc@v1"] for g in meaning["goals"])
        runtime.validate_adoption(meaning, ports)
        return meaning

    async def same_request_two_revisions():
        runtime, evidence, model = build()
        executions = []
        try:
            predicate = proposed()["relations"][0]["predicate"]
            missing_scope = ("never-dispatched", 1) if mapping_scope == "revision" else "never-dispatched"
            assert runtime._predicate(predicate, missing_scope) is None, "missing producer was treated as false"
            ports = [rm.Ports(runtime, "same-request", "revision-input-" + str(rev), "실행 검증", revision_number=rev)
                     for rev in [1, 2]]
            meanings = [await perform_meaning(runtime, port) for port in ports]
            for meaning, port in zip(meanings, ports):
                executions.append(asyncio.create_task(runtime.execute(meaning, port)))
            deadline = time.monotonic() + .3
            while len([e for e in runtime.effects if e["operation"] == "CREATE_REPORT"]) < 2 and time.monotonic() < deadline:
                await asyncio.sleep(.001)
            first = runtime.sent_commands[("same-request", 1, "report")]
            second = runtime.sent_commands[("same-request", 2, "report")]
            assert first["task_id"] != second["task_id"], "distinct Request revisions reused a producer Task"
            await runtime.external_event({"event_id": "actual-v1-failure", "type": "FAILURE", "payload": {
                "task_id": first["task_id"], "execution_id": first["execution_id"],
                "request_revision": first["request_revision"], "reason": "ACTUAL_PRODUCER_FAILURE"}})
            await asyncio.wait_for(asyncio.gather(*executions), .6)
            while runtime.jobs:
                await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .6)
                await asyncio.sleep(0)
            assert runtime.owners.tasks[first["task_id"]]["status"] == "FAILED"
            assert runtime.owners.tasks[second["task_id"]]["status"] == "COMPLETED"
            drafts = [e for e in runtime.effects if e["operation"] == "DRAFT_MAIL"]
            assert len(drafts) == 1, "rev1 failure continuation was lost or rev2 success incorrectly continued"
            expected = runtime.sent_commands.get(("same-request", 1, "draft"))
            assert expected and expected["command_id"] == drafts[0]["command_id"], "draft belonged to the wrong Request revision"
            assert ("same-request", 2, "draft") not in runtime.sent_commands
            assert not [e for e in runtime.effects if e["operation"] == "SEND_MAIL"]
        finally:
            for execution in executions:
                if not execution.done(): execution.cancel()
            await asyncio.gather(*executions, return_exceptions=True)
            await collect(runtime, evidence, model, "same_request_two_revisions")

    async def actual_owner_revision_and_stale_execution():
        runtime, evidence, model = build()
        execution = None
        try:
            ports = rm.Ports(runtime, "owner-lineage-request", "owner-lineage-input", "검증", revision_number=3)
            meaning = await perform_meaning(runtime, ports)
            execution = asyncio.create_task(runtime.execute(meaning, ports))
            deadline = time.monotonic() + .3
            while not runtime.effects and time.monotonic() < deadline: await asyncio.sleep(.001)
            first = runtime.sent_commands[(ports.request_id, ports.revision, "report")]
            for mismatch in [{"execution_id": "wrong-execution"}, {"request_revision": 99}]:
                await runtime.external_event({"event_id": "foreign-lineage-" + str(mismatch), "type": "FAILURE", "payload": {
                    "task_id": first["task_id"], "execution_id": first["execution_id"],
                    "request_revision": first["request_revision"], "reason": "WRONG_LINEAGE", **mismatch}})
            assert not any(e.get("kind") == "TASK_FAILED" for e in evidence.rows)
            await runtime.external_event({"event_id": "valid-lineage-failure", "type": "FAILURE", "payload": {
                "task_id": first["task_id"], "execution_id": first["execution_id"],
                "request_revision": first["request_revision"], "reason": "ACTUAL_PRODUCER_FAILURE"}})
            await asyncio.wait_for(execution, .6)
            while runtime.jobs:
                await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .6)
                await asyncio.sleep(0)
            predicate = meaning["relations"][0]["predicate"]
            scope = (ports.request_id, ports.revision) if mapping_scope == "revision" else ports.request_id
            assert runtime._predicate(predicate, scope) is True
            update = {"id": "update", "kind": "UPDATE_TASK", "operation": "SET_PROPERTY", "target": first["task_id"],
                "arguments": {"property": "FORMAT", "value": "TABLE"}, "instruction": "실제 owner revision 변경"}
            await runtime.perform_goal(update, {"goals": [update], "constraints": []},
                rm.Ports(runtime, "owner-update", "owner-update-input", "표로 변경"))
            assert runtime.owners.tasks[first["task_id"]]["request_revision"] != first["request_revision"]
            assert runtime._predicate(predicate, scope) is None, "old producer consumed a changed Task revision"
            if mapping_scope == "revision":
                key = (ports.request_id, ports.revision, "report")
                binding = copy.deepcopy(runtime.goal_tasks[key])
                runtime.goal_tasks[key]["execution_id"] = "fault-injected-stale-execution"
                assert runtime._predicate(predicate, scope) is None
                runtime.goal_tasks[key] = binding
                observations.append({"probe": "mapping_execution_guard", "injection_kind": "EVALUATOR_MAPPING_FAULT"})
        finally:
            if execution and not execution.done(): execution.cancel(); await asyncio.gather(execution, return_exceptions=True)
            await collect(runtime, evidence, model, "actual_owner_revision_and_stale_execution")

    for name, function in [("same_request_two_revisions", same_request_two_revisions),
                           ("actual_owner_revision_and_stale_execution", actual_owner_revision_and_stale_execution)]:
        try:
            await asyncio.wait_for(function(), 3)
            checks[name] = {"passed": True}
        except Exception as error:
            checks[name] = {"passed": False, "error": f"{type(error).__name__}: {error}", "traceback": traceback.format_exc()}
    after = source_manifest(source)
    checks["source_unchanged"] = {"passed": before == after}
    return {"status": "PASS" if all(c["passed"] for c in checks.values()) else "FAIL", "checks": checks,
        "observations": observations, "variant": variant, "representation": representation, "mapping_scope": mapping_scope,
        "source": str(source), "source_manifest": before, "imported_runtime": str(Path(rm.__file__).resolve()),
        "helper_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION", "actual_model_semantic_accuracy_claim": False, "api_calls": 0}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--variant", choices=["A", "B"], required=True)
    parser.add_argument("--representation", choices=["original", "typed"], required=True)
    parser.add_argument("--mapping-scope", choices=["request", "revision"], required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try: result = asyncio.run(run(args.source, args.variant, args.representation, args.mapping_scope))
    except Exception as error: result = {"status": "FAIL", "error": f"{type(error).__name__}: {error}", "traceback": traceback.format_exc(), "api_calls": 0}
    if args.output.exists(): raise FileExistsError("never overwrite lineage evidence")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2, default=str) + "\n")
    print(json.dumps({k: result[k] for k in ["status", "checks", "api_calls"] if k in result}, ensure_ascii=False))
    sys.exit(0 if result["status"] == "PASS" else 1)
