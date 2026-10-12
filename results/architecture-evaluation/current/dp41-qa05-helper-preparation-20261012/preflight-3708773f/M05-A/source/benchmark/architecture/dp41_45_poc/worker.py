"""Measured candidate process. This module never imports an evaluator oracle."""
from __future__ import annotations

import argparse
import asyncio
import copy
import gzip
import hashlib
import json
import time
import uuid
from pathlib import Path

from .fixtures import build_activity_baseline, build_observation_buffers, observation_buffer_records, load_cases, load_manifest, public_inputs
from prototypes.candidates.dp41_45_poc.shared.evidence import Evidence, canonical, digest
from prototypes.candidates.dp41_45_poc.shared.memory import MemorySampler
from prototypes.candidates.dp41_45_poc.shared.providers import ModelClient
from prototypes.candidates.dp41_45_poc.shared.runtime import LocalVIA


async def wait_until(runtime, offset):
    await asyncio.sleep(max(0., offset - runtime.now()))


class AnswerSource:
    def __init__(self, case):
        self.plans = copy.deepcopy(case.get("followups", []))
        self.unexpected = case.get("unexpected_question_reply", {})
        self.unexpected_count = 0
        self.release = case.get("consent_source_release")
        self.current_transcript = case.get("transcript", "") or ""
        self.current_report_refs = [t["task_id"] for t in case.get("initial",{}).get("tasks",[])
                                    if "보고서" in t.get("instruction", "")]

    def report_scope_question(self, text, scope):
        # A scripted user's response is tied to the question's actual subject,
        # not to one literal sentence or a required occurrence of '보고서'.
        scope_text = json.dumps(scope,ensure_ascii=False)
        related_report = (any(w in text for w in ("보고서","3분기","4분기","분기"))
                          or any(ref in scope_text for ref in self.current_report_refs))
        asks_revision = any(w in text for w in ("수정","바꾸","바꿀","형식","포맷","대상","어느 업무","어떤 업무"))
        asks_scope = any(w in text for w in ("어떤","어느","무엇","확인","알려","말씀","할까요","?"))
        return "보고서" in self.current_transcript and (related_report or asks_revision) and asks_scope

    async def __call__(self, text, scope, delivery, runtime):
        delivered_question = delivery.get("facts", {}).get("question", {})
        qid = delivered_question.get("question_id")
        actual_question = runtime.owners.questions.get(qid, {})
        actual_receipt = delivery.get("delivered_receipt") is True and (
            actual_question.get("status") == "VALID" and actual_question.get("delivered_receipt") is True
            and actual_question.get("revision") == delivered_question.get("revision"))
        if not actual_receipt:
            return None
        plan = next((p for p in self.plans if p["trigger"] == "QUESTION_PRESENTED" and not p.get("used")), None)
        eligible = False
        if plan:
            match = plan.get("match", {})
            if match.get("purpose") == "CLARIFY_TASK_OR_REVISION_SCOPE":
                eligible = (self.report_scope_question(text, actual_question.get("scope"))
                            and actual_question.get("kind") not in ("ACTION_APPROVAL", "INFORMATION_CONSENT"))
            elif match.get("kind") == "INFORMATION_CONSENT":
                actual_scope = actual_question.get("scope", {})
                eligible = (qid == match["question_id"] and actual_question.get("revision") == match["revision"]
                            and actual_question.get("kind") == match["kind"]
                            and actual_question.get("task_id") == "T메일"
                            and actual_scope.get("task_id") == "T메일"
                            and actual_scope.get("source_id") == "민수"
                            and set(actual_scope.get("uses", [])) == {"READ_CONTACT", "PROVIDE_TO_TASK"})
            else:
                eligible = True
        if eligible:
            plan["used"] = True
        else:
            if self.unexpected_count >= self.unexpected.get("max_occurrences", 2):
                return None
            self.unexpected_count += 1
            plan = self.unexpected
        start = runtime.now()
        runtime.followup_pause_start = start
        wait_start = runtime.timeline.now()
        await asyncio.sleep(plan.get("wait_seconds", 1))
        wait_end = runtime.timeline.now()
        predecessor = delivery.get("voice_end_node_id") if delivery.get("voice_on") else delivery.get("text_node_id")
        wait_node = runtime.timeline.span("EXTERNAL_USER_WAIT", "USER", wait_start, wait_end,
            parents=[predecessor] if predecessor else [], evidence=[delivery["publication_id"]])
        input_id = "input:" + str(uuid.uuid4())
        await runtime.input_started(input_id, advance_generation=False, causal_parents=[wait_node])
        await asyncio.sleep(plan.get("speech_seconds", 3))
        await runtime.speech_end(input_id)
        speech_end = runtime.now()
        await asyncio.sleep(runtime.profile["transcript_delay_seconds"])
        runtime.external_intervals.append({"kind": "EXTERNAL_USER_WAIT", "start_s": start,
                                           "end_s": speech_end - plan.get("speech_seconds", 3)})
        runtime.external_intervals.append({"kind": "EXTERNAL_USER_SPEECH", "start_s": speech_end - plan.get("speech_seconds", 3),
                                           "end_s": speech_end})
        runtime.followup_pause_start = None
        answer = {"transcript": plan["transcript"], "input_id": input_id,
                  "speech_end_s": speech_end, "transcript_final_s": runtime.now(),
                  "eligible_planned_reply": bool(eligible), "unexpected_reply": not eligible,
                  "question_id": qid, "question_revision": actual_question.get("revision"),
                  "question_publication_id": delivery["publication_id"]}
        runtime.evidence.write("inputs-and-source", {"scene_key": runtime.scene_key, "kind": "FOLLOWUP_INPUT", **answer})
        # The contact remains private until the actual consent adoption invokes its release.
        if eligible and self.release:
            runtime.pending_consent_source = copy.deepcopy(self.release)
        return answer


async def run_scene(case, variant, client, evidence, baseline, retained, previous=None):
    public = public_inputs(case)
    initial = copy.deepcopy(public["initial"])
    case_sources = initial.get("sources", [])
    source_count = len({s["source_id"] for s in case_sources})
    if source_count > 10:
        raise ValueError("Canonical source-slot capacity exceeded")
    initial["sources"] = [*[s for s in baseline["sources"] if s.get("kind") != "SOURCE_SLOT"],
                          *case_sources,
                          *[s for s in baseline["sources"] if s.get("kind") == "SOURCE_SLOT"][source_count:]]
    for key in ("tasks", "interactions"):
        initial[key] = [*baseline[key], *initial.get(key, [])]
    added_bytes = sum(len(s.get('text','').encode()) for s in case_sources)
    added_bytes += sum(len(t.get('instruction','').encode()) + len(t.get('result',{}).get('text','').encode())
                       for t in public['initial'].get('tasks',[]))
    added_bytes += sum(len(i.get('text','').encode()) for i in public['initial'].get('interactions',[]))
    padding_index = next(i for i,s in enumerate(initial['sources']) if s['source_id'] == 'B20')
    padding_source = copy.deepcopy(initial['sources'][padding_index])
    initial['sources'][padding_index] = padding_source
    if added_bytes:
        padding_source['text'] = padding_source['text'][:-added_bytes]
    evidence.write('allocations',{'scene_key':case['case_id'],'kind':'SCOPED_ORIGINALS_MATERIALIZED',
        'canonical_payload_bytes':baseline['payload_bytes'],'replaced_padding_bytes':added_bytes,
        'scope':'SOURCE_TEXT_PLUS_ORIGINAL_INTERACTION_AND_TASK_TEXT','logical_sources':len({s['source_id'] for s in initial['sources']})})
    runtime = LocalVIA(initial, client, evidence, variant, stub_profile=case["stub_profile"],
                       answer_source=AnswerSource(case), scene_key=case["case_id"])
    runtime.input_evidence = public.get("input_evidence", {})
    if case.get("preparation", {}).get("restore") is False and previous is not None:
        runtime.owners = previous.owners
        runtime.owners.emit = runtime.emit
    elif case.get("preparation", {}).get("restore") is False:
        raise ValueError("Canonical actual predecessor missing; standalone repeated-source case is invalid")
    evidence.write("inputs-and-source", {"scene_key": case["case_id"], "kind": "SCENE_INPUT",
                                          "public": public, "source_hash": digest(public)})
    preparation = case.get("preparation", {})
    preparation_origin = evidence.seconds()
    for event in preparation.get("events", []):
        # Preparation is real source publication in the same owner ports.
        offset = event.get("at_seconds", 0.)
        await asyncio.sleep(max(0., preparation_origin + offset - evidence.seconds()))
        evidence.write("inputs-and-source", {"scene_key": case["case_id"], "kind": "PREPARATION_AVAILABILITY",
            "event": event, "scheduled_source_offset_seconds": offset,
            "actual_source_offset_seconds": evidence.seconds() - preparation_origin})
        await runtime.external_event(event)
    if preparation.get("settle_seconds"):
        await asyncio.sleep(preparation["settle_seconds"])
    baseline_ids = {key: {row.get("source_id", row.get("task_id", row.get("publication_id"))) for row in baseline[key]}
                    for key in ("sources", "tasks", "interactions")}
    actual_initial = copy.deepcopy(public["initial"])
    actual_initial.update({
        "sources": copy.deepcopy([s for s in runtime.owners.sources if s.get("source_id") not in baseline_ids["sources"]]),
        "tasks": copy.deepcopy([t for key, t in runtime.owners.tasks.items() if key not in baseline_ids["tasks"]]),
        "interactions": copy.deepcopy([i for i in runtime.owners.interactions if i.get("publication_id") not in baseline_ids["interactions"]]),
        "questions": copy.deepcopy(list(runtime.owners.questions.values()))})
    evidence.write("inputs-and-source", {"scene_key": case["case_id"], "kind": "ACTUAL_INITIAL_OWNER_STATE",
        "state": actual_initial, "sha256": digest(actual_initial)})
    old_job = None
    release = asyncio.Event()
    old = preparation.get("pending_interpretation")
    if old:
        old_job = runtime.track(runtime.resolve(old["transcript"], request_id=old["request_id"],
                                                input_id="old:" + str(uuid.uuid4()), generation=0,
                                                held_return=release))
        await asyncio.sleep(0)
    first_offset = min([float(e["at_seconds"]) for e in case["events"]] + [0.])
    runtime.origin = evidence.seconds() + max(0., -first_offset)
    request_id, input_id = "R:" + str(uuid.uuid4()), "I:" + str(uuid.uuid4())
    started = evidence.seconds()
    observed_inputs = []

    async def replay():
        for event in case["events"]:
            await wait_until(runtime, event["at_seconds"])
            actual = runtime.now()
            evidence.write("inputs-and-source", {"scene_key": case["case_id"], "kind": "SOURCE_AVAILABILITY",
                                                  "event": event, "scheduled_source_offset_seconds": event["at_seconds"],
                                                  "actual_source_offset_seconds": actual,
                                                  "injection_error_seconds": actual - event["at_seconds"]})
            if event["type"] == "RELEASE_HELD_JOB":
                release.causal_node_id = runtime.timeline.point("HELD_RETURN_RELEASED", evidence=[event["event_id"]])
                release.set()
                runtime.emit("HELD_RETURN_RELEASED", {"actual_old_job_created": old_job is not None,
                    "causal_node_id":release.causal_node_id})
            else:
                runtime.track(runtime.external_event(event))

    async def user_input(transcript, duration, *, voice_state=None, causal_parents=None):
        if voice_state:
            runtime.voice_on = voice_state["on"]
            runtime.voice_connection = voice_state.get("connection_id", runtime.voice_connection)
        await runtime.input_started(input_id, causal_parents=causal_parents)
        await asyncio.sleep(duration)
        await runtime.speech_end(input_id)
        end = runtime.now()
        await asyncio.sleep(runtime.profile["transcript_delay_seconds"])
        observed_inputs.append({"input_id": input_id, "request_id": request_id, "transcript": transcript,
                                "speech_end_s": end, "transcript_final_s": runtime.now()})
        await runtime.resolve(transcript, request_id=request_id, input_id=input_id)

    async def reconnect():
        plan = next(p for p in case["followups"] if p["trigger"] == "TEXT_PRESENTED")
        while runtime.now() < case["episode_limits"]["automatic_episode_seconds"]:
            actual = next((d for d in runtime.deliveries if d["kind"] == "RESULT" and not d["voice_on"]), None)
            if actual:
                wait_start = runtime.timeline.now()
                await asyncio.sleep(plan["wait_seconds"])
                wait_node = runtime.timeline.span("EXTERNAL_USER_WAIT", "USER", wait_start, runtime.timeline.now(),
                    parents=[actual["text_node_id"]], evidence=[actual["publication_id"]])
                await user_input(plan["transcript"], plan["speech_seconds"], voice_state=plan["voice_state"], causal_parents=[wait_node])
                return
            await asyncio.sleep(.02)

    async def episode():
        replay_job = asyncio.create_task(replay())
        await wait_until(runtime, 0.)
        if case.get("transcript") is not None:
            await user_input(case["transcript"], case["root_voice_seconds"])
        elif any(p["trigger"] == "TEXT_PRESENTED" for p in case["followups"]):
            await reconnect()
        await replay_job
        await runtime.drain()
        if old_job:
            release.set()
            await old_job
        await asyncio.sleep(case["episode_limits"]["post_terminal_observation_seconds"])
        await runtime.drain()

    async def watch_episode():
        job = asyncio.create_task(episode())
        limit = case["episode_limits"]["automatic_episode_seconds"]
        while not job.done():
            elapsed = max(0., runtime.now() - case.get("root_voice_seconds", 0.))
            paused = sum(max(0., i["end_s"] - i["start_s"]) for i in runtime.external_intervals
                         if i["kind"] in ("EXTERNAL_USER_WAIT", "EXTERNAL_USER_SPEECH"))
            if getattr(runtime, "followup_pause_start", None) is not None:
                paused += max(0., runtime.now() - runtime.followup_pause_start)
            if elapsed - paused >= limit:
                runtime.emit("AUTOMATIC_EPISODE_TIMEOUT", {"automatic_seconds": elapsed - paused,
                    "limit_seconds": limit, "actual_wall_seconds": runtime.now(),
                    "manual_input_pause_seconds": paused})
                job.cancel()
                await asyncio.gather(job, return_exceptions=True)
                raise asyncio.TimeoutError
            await asyncio.sleep(.05)
        return await job

    try:
        await asyncio.wait_for(watch_episode(), timeout=case["episode_limits"]["whole_episode_seconds"])
        status = "OBSERVED"
    except asyncio.TimeoutError:
        status = "EPISODE_TIMEOUT"
        runtime.emit("EPISODE_TIMEOUT", {"whole_watchdog_seconds": case["episode_limits"]["whole_episode_seconds"],
                                        "automatic_watchdog_seconds": case["episode_limits"]["automatic_episode_seconds"]})
        for task in runtime.jobs:
            task.cancel()
        await runtime.drain()
    event_records = [r for r in evidence.records.get("events", []) if r.get("scene_key") == case["case_id"]]
    calls = [r for r in evidence.records.get("model-calls", []) if r.get("case_id") == case["case_id"]]
    trial = {"case_id": case["case_id"], "variant": variant, "repeat": evidence.repeat,
             "status": status, "request_id": request_id, "initial": actual_initial,
             "fixture_initial_seed": public["initial"],
             "transcript": case.get("transcript"), "observed_inputs": observed_inputs + [
                 {k: v for k, v in r.items() if k not in ("variant", "repeat", "schema_version")}
                 for r in evidence.records.get("inputs-and-source", [])
                 if r.get("scene_key") == case["case_id"] and r.get("kind") == "FOLLOWUP_INPUT"],
             "meanings": [{"request_id": key, "meaning": value["meaning"]} for key, value in runtime.requests.items()],
             "effects": runtime.effects, "deliveries": runtime.deliveries, "events": event_records,
             "final_tasks": [t for key, t in runtime.owners.tasks.items() if not key.startswith("HT")],
             "final_questions": list(runtime.owners.questions.values()),
             "external_intervals": runtime.external_intervals, "model_calls": calls,
             "observed_source_availability": [r for r in evidence.records.get("inputs-and-source", [])
                 if r.get("scene_key") == case["case_id"] and r.get("kind") == "SOURCE_AVAILABILITY"],
             "duration": evidence.seconds() - started, "old_job_created": old_job is not None,
             "late_return_exercised": bool(old_job is not None and
                  any(e["kind"] == "MODEL_RETURN_AVAILABLE" and e.get("request_id") == old["request_id"] for e in event_records)
                  and any(e["kind"] == "MEANING_DISCARDED" and e.get("request_id") == old["request_id"] for e in event_records)),
             "reaction_blueprint": case["reactions"], "cycle_blueprint": case["cycle"],
             "voice_profile": runtime.profile, "evidence_level": "MEASURED_MODEL_WITH_SIMULATED_SPEECH"}
    trial["scene_origin_monotonic_ns"] = evidence.origin + int(runtime.origin * 1e9)
    trial["scene_origin_evidence_seconds"] = runtime.origin
    trial["causal_timeline"] = runtime.timeline.snapshot()
    evidence.write("trials-unjudged", trial)
    retained.append(runtime)
    return runtime, trial


async def main(arguments):
    directory = arguments.output.resolve()
    evidence = Evidence(directory, arguments.variant, arguments.repeat)
    sampler = MemorySampler(evidence)
    sampler.start()
    client = ModelClient(evidence, "qwen", max_cost_usd=arguments.max_cost,
                         budget_file=arguments.budget_file or directory.parents[1] / "qwen-budget.json")
    observation_buffers = build_observation_buffers()
    observation_records = observation_buffer_records(observation_buffers)
    evidence.artifact("raw-inputs", "observation-buffer-manifest", {"frames": observation_records,
        "count": len(observation_buffers), "actual_bytes": sum(len(b) for b in observation_buffers),
        "semantics": "SYNTHETIC_RAW_RGBA_RETENTION_INPUT_WITH_SEPARATE_DECLARED_TEXT_EVIDENCE",
        "retention": "ENTIRE_ACTIVITY", "vision_model_test": False})
    evidence.write("allocations", {"kind": "COMMON_RAW_OBSERVATION_BUFFERS", "frames": observation_records,
        "actual_bytes": sum(len(b) for b in observation_buffers), "accounting": "REAL_TOUCHED_ALLOCATION_NOT_ADDED_TO_OS_MEMORY"})
    baseline = build_activity_baseline()
    encoded = canonical(baseline).encode()
    with gzip.open(directory / "activity-baseline.json.gz", "wb") as handle:
        handle.write(encoded)
    evidence.write("allocations", {"kind": "ACTIVITY_ORIGINALS_CREATED", "payload_bytes": baseline["payload_bytes"],
                                   "actual_serialization_copy_bytes": len(encoded), "buffer_sha256": hashlib.sha256(encoded).hexdigest(),
                                   "retention": "ACTIVITY", "accounting": "EXPLANATION_NOT_ADDED_TO_OS_MEMORY"})
    del encoded
    retained = []
    completed_scenes = {}
    previous = None
    cases = load_cases()
    if arguments.cases:
        cases = [c for c in cases if c["case_id"] in arguments.cases.split(",")]
    available = set()
    required_predecessors = {d for c in cases for d in c.get('preparation',{}).get('depends_on',[])}
    for case in cases:
        missing = set(case.get("preparation", {}).get("depends_on", [])) - available
        if missing:
            raise ValueError(f"Missing canonical predecessors for {case['case_id']}: {sorted(missing)}")
        available.add(case["case_id"])
    try:
        sampler.phase = "ACTIVITY"
        for case in cases:
            dependency = case.get("preparation", {}).get("depends_on", [])
            previous, trial = await run_scene(case, arguments.variant, client, evidence, baseline, retained,
                                              previous=completed_scenes.get(dependency[0]) if dependency else None)
            if case['case_id'] in required_predecessors:
                completed_scenes[case["case_id"]] = previous
            # Raw traces are durable. Keep the current live episode and actual
            # reuse predecessors, rather than retaining 21 independent owner replicas.
            retained[:] = list(dict.fromkeys([*completed_scenes.values(),previous]))
            print(json.dumps({"case": case["case_id"], "variant": arguments.variant,
                              "repeat": arguments.repeat, "status": trial["status"],
                              "models": len(trial["model_calls"]), "effects": len(trial["effects"]),
                              "deliveries": len(trial["deliveries"]), "seconds": round(trial["duration"], 2)}, ensure_ascii=False), flush=True)
            if case["case_id"] == "S2-04":
                activity_origin = evidence.seconds()
                for event in load_manifest()["activity_events"]:
                    await asyncio.sleep(max(0., activity_origin + event["at_seconds"] - evidence.seconds()))
                    if event["type"] == "SOURCE_CHANGE":
                        source = copy.deepcopy(event["payload"]["source"])
                        index = next(i for i, s in enumerate(baseline["sources"]) if s["source_id"] == source["source_id"])
                        baseline["sources"][index] = source
                        previous.owners.apply_source(source)
                    evidence.write("inputs-and-source", {"kind": "ACTIVITY_EVENT", "event": event,
                        "actual_offset_seconds": evidence.seconds() - activity_origin})
                    if event["type"] == "MAINTENANCE":
                        evidence.write("events", {"kind": "MAINTENANCE_TICK", "tick": 1})
        evidence.write("events", {"kind": "MAINTENANCE_TICK", "tick": 2})
        for runtime in retained:
            runtime.owners.revoke("B04")
        sampler.phase = "DRAIN"
        await asyncio.gather(*(runtime.drain() for runtime in retained))
        await asyncio.sleep(30)
        sampler.phase = "IDLE"
        await asyncio.sleep(10)
        evidence.write("worker-summary", {"status": "COMPLETE", "case_count": len(cases),
                                           "provider_usage_price_usd": client.spent})
    finally:
        sampler.stop()
        evidence.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--variant", choices=("A", "B"), required=True)
    parser.add_argument("--repeat", type=int, required=True)
    parser.add_argument("--max-cost", type=float, default=10.)
    parser.add_argument("--cases")
    parser.add_argument("--budget-file", type=Path)
    asyncio.run(main(parser.parse_args()))
