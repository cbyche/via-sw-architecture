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
                    "predicate": predicate or {"reference_id": {"producer_goal_id": "report"}, "field": "STATE",
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
            "tasks": runtime.owners.tasks, "goal_tasks": [{"request": k[0], "request_revision": k[1], "goal": k[2], "task_binding": v}
                for k, v in runtime.goal_tasks.items()], "model_calls": runtime.model_client.calls})
    async def probe(name, operation):
        try:
            await operation()
            checks[name] = {"passed": True, "evidence": ["goalref-runtime-checks.json#" + name]}
        except Exception as error:
            checks[name] = {"passed": False, "error": f"{type(error).__name__}: {error}",
                "traceback": traceback.format_exc(), "evidence": ["goalref-runtime-checks.json#" + name]}

    async def typed_reference_validation():
        valid = frame()
        contract.validate_document(valid, frame=True)
        invalid_refs = [{}, {"producer_goal_id": "draft"}, {"producer_goal_id": "report", "owner": "work"},
                        {"producer_goal_id": ["report"]}, {"producer_goal_id": {"id": "report"}},
                        {"producer_goal_id": "source"}, {"producer_goal_id": "another-request:report"},
                        ["report"], None, 17]
        for ref in invalid_refs:
            invalid = copy.deepcopy(valid)
            invalid["relations"][0]["predicate"]["reference_id"] = ref
            try: contract.validate_document(invalid, frame=True)
            except contract.ContractError: continue
            raise AssertionError("invalid producer tag accepted: " + str(ref))
        invalid = copy.deepcopy(valid)
        invalid["references"].append({"id": "report", "selector": {"op": "KNOWN_REF", "source_ref": "doc@v1"}})
        try: contract.validate_document(invalid, frame=True)
        except contract.ContractError: pass
        else: raise AssertionError("Frame reference and goal IDs overlap")
        observations.append({"probe": "strict_typed_reference_and_frame_disjointness", "invalid_refs": invalid_refs})
    await probe("strict_typed_reference_and_frame_disjointness", typed_reference_validation)

    async def request_scope():
        runtime, evidence = build()
        executions = []
        try:
            proposed = meaning({"reference_id": {"producer_goal_id": "report"}, "field": "STATE", "comparison": "EQ", "value": "FAILED"})
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
            assert runtime._predicate(proposed["relations"][0]["predicate"], ("never-dispatched-request", 1)) is None
            assert runtime._predicate(proposed["relations"][0]["predicate"], ("successful-request", 1)) is False
            assert runtime._predicate(proposed["relations"][0]["predicate"], ("failed-request", 1)) is True
            assert not [e for e in runtime.effects if e["operation"] == "SEND_MAIL"]
        finally:
            for execution in executions:
                if not execution.done(): execution.cancel()
            await asyncio.gather(*executions, return_exceptions=True)
            await clean(runtime, evidence, "actual_request_goal_disjoint_mapping")
    await probe("actual_request_goal_disjoint_mapping", request_scope)

    async def revision_lineage():
        runtime, evidence = build()
        executions = []
        try:
            proposed = meaning({"reference_id": {"producer_goal_id": "report"}, "field": "STATE", "comparison": "EQ", "value": "FAILED"})
            for rev in [1, 2]:
                ports = rm.Ports(runtime, "same-request", "input-revision-" + str(rev), "검증", revision_number=rev)
                executions.append(asyncio.create_task(runtime.execute(copy.deepcopy(proposed), ports)))
            deadline = time.monotonic() + .25
            while len(runtime.goal_tasks) < 2 and time.monotonic() < deadline:
                await asyncio.sleep(.001)
            first = runtime.goal_tasks[("same-request", 1, "report")]
            second = runtime.goal_tasks[("same-request", 2, "report")]
            assert first["task_id"] != second["task_id"]
            for mismatch in [{"execution_id": "another-execution"}, {"request_revision": 99}]:
                await runtime.external_event({"event_id": "bad-lineage-" + str(mismatch), "type": "FAILURE", "payload": {
                    **first, "reason": "BAD_LINEAGE", **mismatch}})
            assert not any(e.get("kind") == "TASK_FAILED" for e in evidence.rows)
            await runtime.external_event({"event_id": "actual-v1-failure", "type": "FAILURE", "payload": {
                **first, "reason": "ACTUAL_PRODUCER_FAILURE"}})
            await asyncio.wait_for(asyncio.gather(*executions), .5)
            while runtime.jobs:
                await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .5)
                await asyncio.sleep(0)
            predicate = proposed["relations"][0]["predicate"]
            assert runtime._predicate(predicate, ("same-request", 1)) is True
            assert runtime._predicate(predicate, ("same-request", 2)) is False
            assert runtime._predicate(predicate, ("different-request", 1)) is None
            assert len([e for e in runtime.effects if e["operation"] == "DRAFT_MAIL"]) == 1
            # Actual owner command advances the first Task's own revision. The
            # old producer tag must not consume the changed owner as its terminal.
            update = {"id": "update", "kind": "UPDATE_TASK", "operation": "SET_PROPERTY", "target": first["task_id"],
                      "arguments": {"property": "FORMAT", "value": "TABLE"}, "instruction": "실제 owner revision 변경"}
            ports = rm.Ports(runtime, "owner-update", "owner-update-input", "표로 변경")
            await runtime.perform_goal(update, {"goals": [update], "constraints": []}, ports)
            assert runtime.owners.tasks[first["task_id"]]["request_revision"] != first["request_revision"]
            assert runtime._predicate(predicate, ("same-request", 1)) is None
            # Explicit fault injection tests the stored dispatch Execution
            # guard without pretending that an event changed owner identity.
            current = copy.deepcopy(runtime.goal_tasks[("same-request", 2, "report")])
            runtime.goal_tasks[("same-request", 2, "report")]["execution_id"] = "injected-stale-execution"
            assert runtime._predicate(predicate, ("same-request", 2)) is None
            runtime.goal_tasks[("same-request", 2, "report")] = current
            observations.append({"probe": "stale_dispatch_execution", "injection_kind": "EVALUATOR_MAPPING_FAULT",
                                 "unknown_instead_of_another_execution_state": True})
        finally:
            for execution in executions:
                if not execution.done(): execution.cancel()
            await asyncio.gather(*executions, return_exceptions=True)
            await clean(runtime, evidence, "actual_request_revision_and_owner_execution_lineage")
    await probe("actual_request_revision_and_owner_execution_lineage", revision_lineage)

    async def owner_when():
        task = {"task_id": "GOAL:report", "execution_id": "GOAL:report-exec", "request_revision": 1,
                "status": "COMPLETED", "instruction": "완료 Task", "properties": {}}
        proposed = frame({"reference_id": "owner", "field": "STATE", "comparison": "EQ", "value": "COMPLETED"})
        proposed["references"].append({"id": "owner", "selector": {"op": "KNOWN_REF", "source_ref": "GOAL:report"}})
        model = Model(proposed)
        runtime, evidence = build(tasks=[task], model=model)
        try:
            ports = rm.Ports(runtime, "owner-when", "owner-input", "검증")
            contract.validate_document(proposed, frame=True)
            resolved = await resolve(runtime.interpretation_input(ports), ports)
            assert resolved["relations"][0]["predicate"]["reference_id"] == "GOAL:report"
            await asyncio.wait_for(runtime.execute(resolved, ports), .5)
            while runtime.jobs:
                await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .5)
                await asyncio.sleep(0)
            assert len([e for e in runtime.effects if e["operation"] == "DRAFT_MAIL"]) == 1
        finally: await clean(runtime, evidence, "legacy_GOAL_literal_owner_when_binder_regression")
    await probe("legacy_GOAL_literal_owner_when_binder_regression", owner_when)
    return {"checks": checks, "observations": observations, "api_calls": 0,
            "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION"}
