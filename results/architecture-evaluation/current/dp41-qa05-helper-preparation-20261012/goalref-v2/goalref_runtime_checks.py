"""API-free state probes against one independently copied candidate source tree.

This file is evaluator code. Scripted meanings below test the changed execution
contract and never claim model semantic accuracy. Candidate source is imported
only from the explicit source argument, not the working repository.
"""
from __future__ import annotations

import asyncio
import copy
import json
from pathlib import Path
import sys
import time
import traceback


async def run_checks(source_root: Path, variant: str, task_id: str) -> dict:
    source_root = source_root.resolve()
    sys.path.insert(0, str(source_root))
    from prototypes.candidates.dp41_45_poc.shared import runtime as runtime_module
    from prototypes.candidates.dp41_45_poc.dp41 import contract
    assert Path(runtime_module.__file__).resolve().is_relative_to(source_root), "probe imported non-isolated source"
    LocalVIA, Ports, PolicyFenceError = runtime_module.LocalVIA, runtime_module.Ports, runtime_module.PolicyFenceError
    observations, checks = [], {}

    class Evidence:
        def __init__(self):
            self.origin, self.rows = time.monotonic(), []
        def seconds(self):
            return time.monotonic() - self.origin
        def write(self, table, row):
            self.rows.append({"table": table, **copy.deepcopy(row)})
            return row

    class Model:
        def __init__(self, hold=None):
            self.calls, self.hold = [], hold
        async def call(self, messages, role, **kwargs):
            self.calls.append({"messages": copy.deepcopy(messages), "role": role})
            if self.hold is not None:
                await self.hold.wait()
            return {"text": "모사 모델 문장: 실제 제공된 실행 사실만 사용"}

    def document(name="doc", revision=1, text="실제 원문 문장.", **extra):
        return {"source_id": name, "revision": revision, "kind": "DOCUMENT", "title": name,
                "text": text, "permission": {"read": True, "model": True, "agent": True}, "metadata": {}, **extra}

    def goal(gid, op, args=None, target=None, kind="NEW_TASK"):
        return {"id": gid, "kind": kind, "operation": op, "target": target,
                "arguments": args if args is not None else {"source_refs": ["doc@v1"]}, "instruction": "구조 계약 검증 입력"}

    def meaning(goals, relations=None):
        return {"version": contract.MEANING_VERSION, "goals": goals, "relations": relations or [],
                "constraints": [{"op": "FORBID_ACTION", "scope": [g["id"] for g in goals], "action": "SEND"}],
                "preservations": [], "response": None, "evidence_refs": [], "unresolved": []}

    def build(initial=None, model=None, final=.015):
        evidence = Evidence()
        runtime = LocalVIA(initial or {"sources": [document()], "tasks": [], "voice": {"on": False}},
            model or Model(), evidence, variant, voice_profile={"output_onset_seconds": .001,
            "characters_per_second": 100000., "stop_seconds": .001}, stub_profile={
            "acceptance_receipt_seconds": .001, "bounded_result_seconds": .001,
            "summary_result_seconds": .004, "final_result_seconds": final})
        return runtime, Ports(runtime, "probe-request", "probe-input", "측정 대상 구조 계약"), evidence

    async def cleanup(runtime, evidence, label):
        for job in list(runtime.jobs):
            job.cancel()
        await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .5)
        observations.append({"probe": label, "events": evidence.rows, "effects": runtime.effects,
            "tasks": runtime.owners.tasks, "deliveries": runtime.deliveries,
            "model_calls": runtime.model_client.calls, "timing": runtime.timeline.snapshot()})

    async def settle(runtime):
        # Preserve normal runtime behavior while yielding to its completion
        # callbacks even when a Python 3.14 gather contains only done tasks.
        while runtime.jobs:
            await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), .5)
            await asyncio.sleep(0)

    async def probe(names, execute):
        print("QA05 probe start " + names[0], file=sys.stderr, flush=True)
        try:
            await execute()
            for name in names:
                checks[name] = {"passed": True, "evidence": ["runtime-requirement-checks.json#" + name]}
        except Exception as error:
            for name in names:
                checks[name] = {"passed": False, "error": f"{type(error).__name__}: {error}",
                    "traceback": traceback.format_exc(), "evidence": ["runtime-requirement-checks.json#" + name]}
        print("QA05 probe end " + names[0], file=sys.stderr, flush=True)

    if task_id == "M01":
        async def schema_rejection():
            from prototypes.candidates.dp41_45_poc.shared.adapters import ProviderWireAdapter, AdapterError
            try:
                ProviderWireAdapter("qwen", "fixture").decode({"result": {"text": "result"}, "metering": {"input_tokens": 4}})
                raise AssertionError("incomplete new provider usage accepted")
            except AdapterError:
                pass
            bad = goal("bad", "ARBITRARY_EXEC")
            try:
                contract.validate_document(meaning([bad]))
                raise AssertionError("invalid upper semantic contract accepted")
            except contract.ContractError:
                pass
            observations.append({"probe": "schema_rejection", "new_wire_incomplete_usage_rejected": True,
                                 "invalid_semantic_operation_rejected": True})
        await probe(["schema_rejection"], schema_rejection)
    if task_id == "M02":
        async def duplicate_command():
            runtime, ports, evidence = build()
            try:
                report = goal("report", "CREATE_REPORT")
                await runtime.perform_goal(report, meaning([report]), ports)
                first_tasks = set(runtime.owners.tasks)
                await runtime.perform_goal(report, meaning([report]), ports)
                await settle(runtime)
                assert len(runtime.effects) == 1, "logical command repeated"
                assert set(runtime.owners.tasks) == first_tasks, "duplicate command created orphan Task"
                assert any(e.get("kind") == "COMMAND_DUPLICATE_SUPPRESSED" for e in evidence.rows)
            finally:
                await cleanup(runtime, evidence, "duplicate_command")
        await probe(["duplicate_command"], duplicate_command)
    if task_id == "M05":
        async def duplicate_answer():
            initial = {"sources": [document()], "tasks": [{"task_id": "work", "execution_id": "work-exec", "request_revision": 1,
                "status": "RUNNING", "instruction": "업무", "source_refs": ["doc@v1"], "properties": {"PERIOD": "Q3"}}], "voice": {"on": False}}
            runtime, ports, evidence = build(initial)
            try:
                question = {"question_id": "q", "revision": 2, "task_id": "work", "execution_id": "work-exec",
                    "request_revision": 1, "status": "VALID", "text": "상반기 범위를 사용합니까?",
                    "scope": {"property": "PERIOD", "proposed_value": "H1"}}
                await runtime.external_event({"event_id": "question", "type": "QUESTION", "payload": {"question": question}})
                actual = runtime.owners.questions["q"]
                assert actual.get("allowed_answer_scope_token"), "Task did not issue new token"
                answer = goal("answer", "ANSWER_QUESTION", {"question_ref": "q", "question_revision": 2,
                    "answer": "yes", "properties": {"PERIOD": "H1"}}, "work", "ANSWER_QUESTION")
                await runtime.perform_goal(answer, meaning([answer]), ports)
                assert runtime.effects[0]["payload"]["question"]["allowed_answer_scope_token"] == actual["allowed_answer_scope_token"]
                try:
                    duplicate_ports = Ports(runtime, "another-answer-request", "another-answer-input", "중복 응답")
                    await runtime.perform_goal(answer, meaning([answer]), duplicate_ports)
                    raise AssertionError("duplicate answer accepted")
                except (ValueError, runtime_module.PolicyFenceError):
                    pass
                await settle(runtime)
                assert len(runtime.effects) == 1 and actual["status"] == "ANSWERED"
            finally:
                await cleanup(runtime, evidence, "duplicate_answer_actual_owner_token")
        await probe(["duplicate_answer"], duplicate_answer)
        async def restored_initial_question():
            question = {"question_id": "restored-q", "revision": 2, "task_id": "work", "execution_id": "work-exec",
                "request_revision": 1, "status": "VALID", "text": "상반기로 바꿀까요?", "delivered_receipt": True,
                "text_presented": True, "voice_presented": False, "scope": {"property": "PERIOD", "proposed_value": "H1"}}
            initial = {"sources": [document()], "tasks": [{"task_id": "work", "execution_id": "work-exec", "request_revision": 1,
                "status": "RUNNING", "instruction": "업무", "source_refs": ["doc@v1"], "properties": {"PERIOD": "Q3"},
                "questions": [question]}], "voice": {"on": False}}
            runtime, ports, evidence = build(initial)
            try:
                actual = runtime.owners.questions["restored-q"]
                assert actual.get("allowed_answer_scope_token")
                answer = goal("answer", "ANSWER_QUESTION", {"question_ref": "restored-q", "question_revision": 2,
                    "answer": "yes", "properties": {"PERIOD": "H1"}}, "work", "ANSWER_QUESTION")
                await runtime.perform_goal(answer, meaning([answer]), ports)
                await settle(runtime)
                assert len(runtime.effects) == 1 and runtime.owners.tasks["work"]["properties"]["PERIOD"] == "H1"
                assert runtime.effects[0]["payload"]["question"]["allowed_answer_scope_token"] == actual["allowed_answer_scope_token"]
            finally:
                await cleanup(runtime, evidence, "restore_existing_actual_presented_question")
        await probe(["restored_initial_question_with_token"], restored_initial_question)
    if task_id == "M03":
        async def new_calendar_port():
            from prototypes.candidates.dp41_45_poc.shared.adapters import CalendarSourceAdapter
            for state in ("BUSY", "FREE"):
                raw = {"resource_type": "CALENDAR_V2", "calendar_uid": "calendar", "etag_revision": 2,
                    "original_text": "실제 원천 일정 정보", "access": {"read": True, "model": True, "agent": True},
                    "calendar": {"day": "2026-10-16", "zone": "Asia/Seoul", "availability": state,
                        "start_utc": "2026-10-16T06:00:00+00:00", "end_utc": "2026-10-16T07:00:00+00:00", "appointments": []}}
                normalized = CalendarSourceAdapter().normalize(raw)
                assert normalized["source_id"] == "calendar", "new raw identity not normalized"
                runtime, ports, evidence = build({"sources": [normalized], "tasks": [], "voice": {"on": False}})
                try:
                    conditional = goal("calendar", "CALENDAR_CONDITIONAL", {"source_refs": ["calendar@v2"],
                        "date": "2026-10-16", "start": "15:00", "end": "16:00", "if_free": "CREATE_EVENT",
                        "otherwise": "AVAILABLE_ONLY"}, kind="DIRECT_RESPONSE")
                    result = await runtime.perform_goal(conditional, meaning([conditional]), ports)
                    await settle(runtime)
                    assert result["condition_state"] == state
                    assert len(runtime.effects) == (1 if state == "FREE" else 0)
                    observations.append({"probe": "new_calendar_port", "raw": raw, "candidate_normalized": normalized})
                finally:
                    await cleanup(runtime, evidence, "new_calendar_" + state)
        await probe(["new_calendar_port_busy_and_free"], new_calendar_port)

    if task_id == "M04":
        async def chain(failure=True, relation="WHEN", comparison="IN", terminal_value=None):
            runtime, ports, evidence = build(final=.02)
            pending = None
            try:
                first, second = goal("report", "CREATE_REPORT"), goal("draft", "DRAFT_MAIL")
                link = {"op": relation, "from_goal": "report", "to_goal": "draft"}
                if relation == "WHEN":
                    value = terminal_value if terminal_value is not None else ["FAILED", "COMPLETED"]
                    link["predicate"] = {"reference_id": {"producer_goal_id": "report"}, "field": "STATE", "comparison": comparison, "value": value}
                proposed = meaning([first, second], [link])
                if variant == "B":
                    from prototypes.candidates.dp41_45_poc.dp41.b import resolve
                    frame = copy.deepcopy(proposed)
                    frame.update({"version": contract.FRAME_VERSION,
                        "references": [{"id": "source", "selector": {"op": "KNOWN_REF", "source_ref": "doc@v1"},
                                        "cardinality": "ONE"}], "input_ref": ports.input_id})
                    for frame_goal in frame["goals"]:
                        frame_goal["arguments"]["source_refs"] = ["source"]
                    # Exercise the model-frame validation entry point before
                    # the real resolver; a direct binder call misses this gate.
                    contract.validate_document(frame, frame=True)
                    if relation == "WHEN":
                        for invalid in (
                            {"reference_id": {"producer_goal_id": "draft"}}, {"producer_goal_id": "report"},
                            {"comparison": "NE"}, {"value": "RUNNING"},
                            {"comparison": "IN", "value": []}, {"comparison": "IN", "value": ["FAILED", "FAILED"]},
                            {"comparison": "IN", "value": ["FAILED", "RUNNING"]},
                            {"comparison": "IN", "value": "FAILED"}):
                            rejected = copy.deepcopy(frame)
                            rejected["relations"][0]["predicate"].update(invalid)
                            try:
                                contract.validate_document(rejected, frame=True)
                            except contract.ContractError:
                                continue
                            raise AssertionError("invalid current-producer frame predicate accepted: " + str(invalid))
                    class FrameModel(Model):
                        async def call(self, messages, role, **kwargs):
                            if role == "CINTERPRET":
                                self.calls.append({"messages": copy.deepcopy(messages), "role": role,
                                                   "scripted_frame": copy.deepcopy(frame)})
                                return copy.deepcopy(frame)
                            return await super().call(messages, role, **kwargs)
                    runtime.model_client = FrameModel()
                    proposed = await resolve(runtime.interpretation_input(ports), ports)
                    assert [g["operation"] for g in proposed["goals"]] == ["CREATE_REPORT", "DRAFT_MAIL"], "frame rejected or repaired"
                    assert proposed["relations"] == [link], "resolver changed terminal predicate"
                    assert all(g["arguments"]["source_refs"] == ["doc@v1"] for g in proposed["goals"])
                    assert any(e.get("kind") == "dp41_b_frame_produced" for e in evidence.rows)
                    assert not any(e.get("kind") == "dp41_output_rejected" for e in evidence.rows)
                    observations.append({"probe": "actual_b_frame_resolver", "frame": frame,
                        "resolved_meaning": copy.deepcopy(proposed), "scripted_semantic_accuracy_claim": False})
                runtime.validate_adoption(proposed, ports)
                pending = asyncio.create_task(runtime.execute(proposed, ports))
                deadline = time.monotonic() + .2
                while not runtime.effects and time.monotonic() < deadline:
                    await asyncio.sleep(.001)
                assert runtime.effects, "actual report command was not dispatched"
                effect = runtime.effects[0]
                await asyncio.sleep(.003)
                task_id = effect["task_id"]
                foreign = {"event_id": "foreign", "type": "FAILURE", "payload": {
                    "task_id": "unrelated", "execution_id": effect["execution_id"], "request_revision": 1, "reason": "OTHER"}}
                await runtime.external_event(foreign)
                if failure:
                    event = {"event_id": "actual-report-failure", "type": "FAILURE", "payload": {
                        "task_id": task_id, "execution_id": effect["execution_id"], "request_revision": 1, "reason": "REPORT_SOURCE_REJECTED"}}
                    await runtime.external_event(event)
                    await runtime.external_event(event)
                    await runtime.external_event({**event, "event_id": "same-terminal-new-delivery"})
                await asyncio.wait_for(pending, .3)
                print("QA05 chain execution returned " + relation, file=sys.stderr, flush=True)
                await asyncio.wait_for(settle(runtime), .5)
                print("QA05 chain drained " + relation, file=sys.stderr, flush=True)
                report_task = runtime.owners.tasks[task_id]
                drafts = [e for e in runtime.effects if e["operation"] == "DRAFT_MAIL"]
                expected_state = "FAILED" if failure else "COMPLETED"
                expected_drafts = int(relation == "WHEN" and
                    (link["predicate"]["value"] == expected_state if comparison == "EQ"
                     else expected_state in link["predicate"]["value"]))
                assert len(drafts) == expected_drafts, "wrong terminal continuation count"
                for draft in drafts:
                    assert draft["payload"]["goal"]["arguments"]["source_refs"] == ["doc@v1"]
                    assert "body_artifact" not in draft["payload"], "successful report artifact was injected"
                    assert len(draft["payload"]["sources"]) == 1
                    draft_source = draft["payload"]["sources"][0]
                    assert (draft_source["source_id"], draft_source["revision"], draft_source["text"]) == ("doc", 1, "실제 원문 문장.")
                if failure:
                    assert report_task["status"] == "FAILED", "actual failure did not update Task owner"
                    assert "result" not in report_task, "failed report invented successful result"
                    assert sum(e.get("table") == "events" and e.get("kind") == "TASK_FAILED" for e in evidence.rows) == 1
                    await runtime.external_event({"event_id": "late-success", "type": "RESULT", "payload": {
                        "task_id": task_id, "execution_id": effect["execution_id"], "request_revision": 1,
                        "status": "COMPLETED", "artifact": {"artifact_id": "late", "text": "늦은 성공", "source_refs": ["doc@v1"]}}})
                    assert report_task["status"] == "FAILED" and "result" not in report_task
                    assert len([e for e in runtime.effects if e["operation"] == "DRAFT_MAIL"]) == len(drafts)
                else:
                    assert report_task["status"] == "COMPLETED" and report_task.get("result")
                assert not [e for e in runtime.effects if e["operation"] == "SEND_MAIL"]
                assert "unrelated" not in runtime.owners.tasks
                assert runtime.owners.tasks[task_id]["execution_id"] == effect["execution_id"]
            finally:
                if pending and not pending.done():
                    pending.cancel()
                    await asyncio.gather(pending, return_exceptions=True)
                await cleanup(runtime, evidence, f"{relation}-{comparison}-{terminal_value}-{'failure' if failure else 'success'}")
        await probe(["failure_continues_draft_once", "SEND_zero", "duplicate_failure", "late_success", "unrelated_task"], lambda: chain(True))
        await probe(["success_branch_preserved"], lambda: chain(False))
        await probe(["failure_only_success_suppressed"], lambda: chain(False, comparison="EQ", terminal_value="FAILED"))
        await probe(["failure_only_continuation"], lambda: chain(True, comparison="EQ", terminal_value="FAILED"))
        await probe(["completed_only_continuation"], lambda: chain(False, comparison="EQ", terminal_value="COMPLETED"))
        await probe(["completed_only_failure_blocked"], lambda: chain(True, comparison="EQ", terminal_value="COMPLETED"))
        await probe(["AFTER"], lambda: chain(True, "AFTER"))
        await probe(["USE_RESULT"], lambda: chain(True, "USE_RESULT"))
    elif task_id == "M06":
        async def fences():
            gate = asyncio.Event()
            runtime, ports, evidence = build({"sources": [document(), document("other", text="별도 허용 원문")],
                                             "tasks": [], "voice": {"on": False}}, Model(gate))
            pending = publication = None
            try:
                await ports.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "doc@v1"}})
                runtime.artifacts["derived"] = {"artifact_id": "derived", "text": "파생 내용", "source_refs": ["doc@v1"]}
                runtime.goal_results[(ports.request_id, "derived")] = runtime.artifacts["derived"]
                runtime.memory_records["derived"] = {"value": "파생 관계", "source_refs": ["doc@v1"]}
                runtime.memory_records["other"] = {"value": "허용 관계", "source_refs": ["other@v1"]}
                await runtime.input_started("hold-publication", advance_generation=False)
                publication = asyncio.create_task(runtime.deliver("철회 전 준비 문장", request_id=ports.request_id,
                    facts={"sources": [copy.deepcopy(runtime.owners.lookup("doc@v1"))]}))
                messages = [{"role": "system", "content": "규칙"}, {"role": "user", "content": "실제 원문 문장. doc@v1"}]
                pending = asyncio.create_task(ports.model(messages))
                await asyncio.sleep(.003)
                await runtime.external_event({"event_id": "revoke", "type": "PERMISSION_REVOKED", "payload": {"source_id": "doc"}})
                gate.set()
                try:
                    await pending
                    raise AssertionError("pending late model return was accepted")
                except PolicyFenceError:
                    pass
                await runtime.speech_end("hold-publication")
                output = await publication
                assert not output["delivered_receipt"] and output["text_presented_s"] is None
                assert runtime.artifacts["derived"]["text"] == "" and runtime.memory_records["derived"]["value"] is None
                assert runtime.memory_records["other"]["value"] == "허용 관계"
                assert runtime.owners.lookup("other@v1")["text"] == "별도 허용 원문"
                assert not runtime.owners.cache and "REVOKED" in messages[1]["content"]
                receipt = [e for e in evidence.rows if e.get("kind") == "REVOCATION_FENCE_APPLIED"][-1]
                assert receipt["receipt"] == "LOCAL_CONSUMERS_FENCED_REMOTE_SENT_COPIES_NOT_RETRACTED"
                for context in ("read", "agent", "publication"):
                    try:
                        runtime.source_payload(["doc@v1"], roles=("read", "agent"), context=context)
                        raise AssertionError("revoked original source was consumed")
                    except PolicyFenceError:
                        pass
                runtime.memory_records["kept-preference"] = {"value": "다른 명시 선호", "source_refs": []}
                pref = goal("preference", "REGISTER_MEMORY", {"key": "explicit", "value": "짧은 문장", "source_refs": []})
                await runtime.perform_goal(pref, meaning([pref]), ports)
                assert runtime.memory_records["explicit"]["value"] == "짧은 문장"
                assert runtime.memory_records["kept-preference"]["value"] == "다른 명시 선호"
                await runtime.external_event({"event_id": "restore-consent", "type": "SOURCE_CHANGE", "payload": {"source": document(revision=2, text="재허용 원문")}})
                restored = await Ports(runtime, "restored-request", "restored-input", "재허용").read({
                    "tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "doc@v2"}})
                assert restored.get("items") and restored["items"][0].get("content") == "재허용 원문"
                assert runtime.artifacts["derived"]["text"] == "", "restoring consent resurrected revoked derivatives"
            finally:
                for p in (pending, publication):
                    if p and not p.done():
                        p.cancel()
                        await asyncio.gather(p, return_exceptions=True)
                await cleanup(runtime, evidence, "baseline-use-fence-and-restore")
        await probe(["original_derived_pending_late_use_blocked", "honest_processing_receipt", "other_data_retained",
                     "cache_invalidation", "late_publication", "explicit_preference_registration", "consent_restore"], fences)
    else:
        # Existing runtime contract probes are run from the isolated tree and
        # paired with the new adapter's actual transport tests by the caller.
        from prototypes.candidates.dp41_45_poc.shared.tests import test_runtime
        import unittest
        mapping = {
            "M01": {"late_cancel_usage": ["test_revocation_fences_late_model_and_clears_local_derived_content"]},
            "M02": {"reverse_receipt": ["test_post_cancel_original_receipt_does_not_claim_termination_or_restore_running"],
                    "cancel_unknown": ["test_post_cancel_original_receipt_does_not_claim_termination_or_restore_running"]},
            "M03": {"S3-03_BUSY_NO_CREATE": ["test_calendar_guard_uses_actual_current_owner_after_source_change"],
                    "stale_source": ["test_calendar_guard_uses_actual_current_owner_after_source_change"],
                    "permission_denied": ["test_model_prohibition_cannot_be_bypassed_by_execution_source_read"]},
            "M05": {"reverse_event": ["test_question_withdrawn_during_input_hold_has_no_presentation_receipt"],
                    "input_hold": ["test_question_withdrawn_during_input_hold_has_no_presentation_receipt"]},
        }
        # unittest's async methods need their own event loop, so the synchronous
        # suite executes in another thread outside this probe loop.
        async def suite(names):
            def execute():
                result = unittest.TestResult()
                unittest.TestSuite([test_runtime.RuntimeTests(n) for n in names]).run(result)
                return result
            result = await asyncio.to_thread(execute)
            observations.append({"probe": names, "tests_run": result.testsRun,
                "errors": [(str(t), e) for t, e in result.errors], "failures": [(str(t), e) for t, e in result.failures]})
            assert result.wasSuccessful(), str(result.errors + result.failures)
        for check, names in mapping[task_id].items():
            await probe([check], lambda names=names: suite(names))
    return {"checks": checks, "observations": observations, "variant": variant, "task_id": task_id,
            "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION", "source_root": str(source_root),
            "imported_runtime": str(Path(runtime_module.__file__).resolve()), "api_calls": 0}


if __name__ == "__main__":
    async def combined():
        result = await run_checks(Path(sys.argv[1]), sys.argv[2], sys.argv[3])
        from goalref_extras import run_goalref_extras
        extra = await run_goalref_extras(Path(sys.argv[1]))
        result["checks"].update(extra["checks"])
        result["observations"].extend(extra["observations"])
        result["representation_alternative"] = "TYPED-GOALREF-v2"
        return result
    print(json.dumps(asyncio.run(combined()), ensure_ascii=False, default=str))
