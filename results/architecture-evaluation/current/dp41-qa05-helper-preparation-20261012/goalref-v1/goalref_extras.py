"""API-free, isolated GOALREF namespace and Request-scope regressions."""
from __future__ import annotations
import asyncio
import copy
from pathlib import Path
import sys
import time
import traceback


async def run_goalref_extras(source):
    source = Path(source).resolve()
    sys.path.insert(0, str(source))
    from prototypes.candidates.dp41_45_poc.shared import runtime as rm
    from prototypes.candidates.dp41_45_poc.dp41 import contract
    from prototypes.candidates.dp41_45_poc.dp41.b import resolve
    assert Path(rm.__file__).resolve().is_relative_to(source)
    checks, observations = {}, []

    class Evidence:
        def __init__(self): self.origin, self.rows = time.monotonic(), []
        def seconds(self): return time.monotonic() - self.origin
        def write(self, table, row):
            self.rows.append({"table": table, **copy.deepcopy(row)})
            return row
    class Model:
        def __init__(self, frame=None): self.frame, self.calls = frame, []
        async def call(self, messages, role, **kwargs):
            self.calls.append({"messages": copy.deepcopy(messages), "role": role})
            return copy.deepcopy(self.frame) if role == "CINTERPRET" else {"text": "실제 실행 사실 전달"}
    def doc(name="doc"):
        return {"source_id": name, "revision": 1, "kind": "DOCUMENT", "text": "원문", "title": name,
                "permission": {"read": True, "model": True, "agent": True}, "metadata": {}}
    def meaning(predicate=None):
        goals = [{"id": g, "kind": "NEW_TASK", "operation": op, "target": None,
                  "arguments": {"source_refs": ["doc@v1"]}, "instruction": "계약 검증"}
                 for g, op in [("report", "CREATE_REPORT"), ("draft", "DRAFT_MAIL")]]
        return {"version": contract.MEANING_VERSION, "goals": goals,
                "relations": [{"op": "WHEN", "from_goal": "report", "to_goal": "draft",
                    "predicate": predicate or {"reference_id": "GOAL:report", "field": "STATE",
                        "comparison": "IN", "value": ["FAILED", "COMPLETED"]}}],
                "constraints": [{"op": "FORBID_ACTION", "scope": ["report", "draft"], "action": "SEND"}],
                "preservations": [], "response": None, "evidence_refs": [], "unresolved": []}
    def frame(predicate=None):
        item = meaning(predicate)
        item.update(version=contract.FRAME_VERSION, references=[{"id": "source", "selector": {
            "op": "KNOWN_REF", "source_ref": "doc@v1"}, "cardinality": "ONE"}])
        for g in item["goals"]: g["arguments"]["source_refs"] = ["source"]
        return item
    def build(extra_sources=None, tasks=None, model=None):
        evidence = Evidence()
        runtime = rm.LocalVIA({"sources": [doc(), *(extra_sources or [])], "tasks": tasks or [],
            "voice": {"on": False}}, model or Model(), evidence, "B", stub_profile={
            "acceptance_receipt_seconds": .001, "final_result_seconds": .045})
        return runtime, evidence
    async def clean(runtime, evidence, name):
        for task in list(runtime.jobs): task.cancel()
        await asyncio.gather(*list(runtime.jobs), return_exceptions=True)
        observations.append({"probe": name, "events": evidence.rows, "effects": runtime.effects,
            "tasks": runtime.owners.tasks, "goal_tasks": [{"request": k[0], "goal": k[1], "task": v}
                for k, v in runtime.goal_tasks.items()], "model_calls": runtime.model_client.calls})
    async def probe(name, operation):
        try:
            await operation()
            checks[name] = {"passed": True, "evidence": ["goalref-runtime-checks.json#" + name]}
        except Exception as error:
            checks[name] = {"passed": False, "error": f"{type(error).__name__}: {error}",
                "traceback": traceback.format_exc(), "evidence": ["goalref-runtime-checks.json#" + name]}

    async def reserved_namespace():
        valid = frame()
        contract.validate_document(valid, frame=True)
        for reference in ["GOAL:", "goal:report", "GOAL:draft", "GOAL:missing", "GOAL:another-request:report", "GOAL:report@v1"]:
            invalid = copy.deepcopy(valid)
            invalid["relations"][0]["predicate"]["reference_id"] = reference
            try: contract.validate_document(invalid, frame=True)
            except contract.ContractError: continue
            raise AssertionError("invalid namespace accepted: " + reference)
        for reference in ["GOAL:report", "goal:report"]:
            invalid = copy.deepcopy(valid)
            invalid["references"].append({"id": reference, "selector": {"op": "KNOWN_REF", "source_ref": "doc@v1"}})
            try: contract.validate_document(invalid, frame=True)
            except contract.ContractError: continue
            raise AssertionError("reserved Frame binding key accepted")
        observations.append({"probe": "strict_namespace_and_frame_collision", "invalid_references_rejected": 8})
    await probe("strict_namespace_and_frame_collision", reserved_namespace)

    async def owner_collisions():
        for owner_type in ["SOURCE", "TASK"]:
            task = {"task_id": "GOAL:report", "execution_id": "foreign-exec", "request_revision": 1,
                "status": "COMPLETED", "instruction": "외부 Task", "properties": {}}
            runtime, evidence = build([doc("GOAL:report")] if owner_type == "SOURCE" else [],
                                      [task] if owner_type == "TASK" else [])
            try:
                proposed = meaning()
                contract.validate_document(proposed)
                try: await runtime.execute(proposed, rm.Ports(runtime, "collision", "input", "검증"))
                except ValueError as error: assert "collides" in str(error)
                else: raise AssertionError("actual owner collision accepted")
                assert not runtime.effects, "collision rejected after a side effect"
            finally: await clean(runtime, evidence, "actual_owner_collision_" + owner_type)
    await probe("actual_owner_collision_before_effect", owner_collisions)

    async def request_scope():
        runtime, evidence = build()
        executions = []
        try:
            proposed = meaning({"reference_id": "GOAL:report", "field": "STATE", "comparison": "EQ", "value": "FAILED"})
            contract.validate_document(proposed)
            for request in ["failed-request", "successful-request"]:
                ports = rm.Ports(runtime, request, request + "-input", "검증")
                executions.append(asyncio.create_task(runtime.execute(copy.deepcopy(proposed), ports)))
            deadline = time.monotonic() + .25
            while len([e for e in runtime.effects if e["operation"] == "CREATE_REPORT"]) < 2 and time.monotonic() < deadline:
                await asyncio.sleep(.001)
            reports = {e["request_id"]: e for e in runtime.effects if e["operation"] == "CREATE_REPORT"}
            assert set(reports) == {"failed-request", "successful-request"}
            assert reports["failed-request"]["task_id"] != reports["successful-request"]["task_id"]
            effect = reports["failed-request"]
            await runtime.external_event({"event_id": "scope-failure", "type": "FAILURE", "payload": {
                "task_id": effect["task_id"], "execution_id": effect["execution_id"], "request_revision": 1,
                "reason": "ACTUAL_FAILURE"}})
            await asyncio.wait_for(asyncio.gather(*executions), .5)
            while runtime.jobs:
                await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .5)
                await asyncio.sleep(0)
            drafts = [e for e in runtime.effects if e["operation"] == "DRAFT_MAIL"]
            assert len(drafts) == 1 and drafts[0]["request_id"] == "failed-request"
            assert runtime._predicate(proposed["relations"][0]["predicate"], "never-dispatched-request") is None
            assert runtime._predicate(proposed["relations"][0]["predicate"], "successful-request") is False
            assert runtime._predicate(proposed["relations"][0]["predicate"], "failed-request") is True
            assert not [e for e in runtime.effects if e["operation"] == "SEND_MAIL"]
        finally:
            for execution in executions:
                if not execution.done(): execution.cancel()
            await asyncio.gather(*executions, return_exceptions=True)
            await clean(runtime, evidence, "actual_request_goal_disjoint_mapping")
    await probe("actual_request_goal_disjoint_mapping", request_scope)

    async def owner_when():
        task = {"task_id": "old-work", "execution_id": "old-work-exec", "request_revision": 1,
                "status": "COMPLETED", "instruction": "완료 Task", "properties": {}}
        proposed = frame({"reference_id": "owner", "field": "STATE", "comparison": "EQ", "value": "COMPLETED"})
        proposed["references"].append({"id": "owner", "selector": {"op": "KNOWN_REF", "source_ref": "old-work"}})
        model = Model(proposed)
        runtime, evidence = build(tasks=[task], model=model)
        try:
            ports = rm.Ports(runtime, "owner-when", "owner-input", "검증")
            contract.validate_document(proposed, frame=True)
            resolved = await resolve(runtime.interpretation_input(ports), ports)
            assert resolved["relations"][0]["predicate"]["reference_id"] == "old-work"
            await asyncio.wait_for(runtime.execute(resolved, ports), .5)
            while runtime.jobs:
                await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .5)
                await asyncio.sleep(0)
            assert len([e for e in runtime.effects if e["operation"] == "DRAFT_MAIL"]) == 1
        finally: await clean(runtime, evidence, "ordinary_owner_when_binder_regression")
    await probe("ordinary_owner_when_binder_regression", owner_when)
    return {"checks": checks, "observations": observations, "api_calls": 0,
            "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION"}
