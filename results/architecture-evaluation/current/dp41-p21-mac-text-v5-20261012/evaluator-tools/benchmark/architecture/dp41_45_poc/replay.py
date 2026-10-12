"""API-free replay of frozen DP41 ports for QA05 source-change regression.

Recorded model answers remain recorded answers, including failures. No oracle is
loaded. This checks execution and upper-port preservation, not semantic accuracy.
The evaluator launches the copied source in a fresh process; it never imports a
candidate from the parent evaluator's module cache.
"""
from __future__ import annotations

import argparse
import asyncio
import collections
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import traceback
import uuid


CASE_IDS = [f"S{group}-{number:02d}" for group, count in ((1, 2), (2, 5), (3, 3), (4, 3), (5, 4), (6, 4))
            for number in range(1, count + 1)]
LEVEL = "RECORDED_PORT_REPLAY_NOT_SEMANTIC_REMEASUREMENT"


def _canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def _hash(value):
    return hashlib.sha256(_canonical(value).encode()).hexdigest()


def _read_jsonl(path):
    return [json.loads(line) for line in Path(path).read_text().splitlines() if line.strip()]


def _messages(value):
    result = []
    for message in value:
        row = copy.deepcopy(message)
        if row.get("role") == "system":
            row["content"] = "<SYSTEM_DELTA_ALLOWED>"
        elif isinstance(row.get("content"), str):
            try:
                row["content"] = json.loads(row["content"])
            except json.JSONDecodeError:
                pass
        result.append(row)
    return result


class ReplayMismatch(AssertionError):
    pass


class RawWorker:
    """Read only actual worker streams and referenced model artifacts."""
    def __init__(self, directory):
        self.directory = Path(directory).resolve()
        self.trials = _read_jsonl(self.directory / "trials-unjudged.jsonl")
        self.calls = _read_jsonl(self.directory / "model-calls.jsonl")
        self.starts = _read_jsonl(self.directory / "model-call-starts.jsonl")
        self.artifacts = {}
        self.stream_hashes = [{"path": str(self.directory / name),
                               "file_sha256": hashlib.sha256((self.directory / name).read_bytes()).hexdigest()}
                              for name in ("trials-unjudged.jsonl", "model-calls.jsonl", "model-call-starts.jsonl")]

    def artifact(self, ref):
        path = (self.directory / ref["path"]).resolve()
        if not path.is_relative_to(self.directory):
            raise ReplayMismatch("model artifact escapes its frozen worker")
        data = path.read_bytes()
        value = json.loads(data)
        if ref.get("sha256") != _hash(value):
            raise ReplayMismatch("frozen model artifact hash mismatch: " + str(path))
        self.artifacts[str(path)] = {"path": str(path), "json_sha256": _hash(value),
                                     "file_sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
        return value

    def model_groups(self, trial, role):
        calls = [c for c in self.calls if c.get("case_id") == trial["case_id"] and c.get("role") == role
                 and c.get("repeat") == trial["repeat"]]
        calls.sort(key=lambda c: c.get("enqueue_ns", c.get("monotonic_ns", 0)))
        groups = []
        for call in calls:
            request = call.get("request_id")
            if call.get("attempt", 1) == 1:
                groups.append({"request_id": request, "attempts": []})
            matches = [g for g in groups if g["request_id"] == request]
            if not matches:
                raise ReplayMismatch("model retry lacks its original attempt")
            group = matches[-1]
            context = self.artifact(call["context"])
            messages = context.get("messages", context.get("input"))
            if not isinstance(messages, list):
                raise ReplayMismatch("recorded provider context lacks messages")
            attempt = {"call": call, "messages": messages, "context": context}
            if call.get("output"):
                output = self.artifact(call["output"])
                if "choices" in output:
                    text = output["choices"][0]["message"].get("content") or ""
                elif "output" in output:
                    text = "".join(part.get("text", "") for row in output["output"]
                                   for part in row.get("content", []) if part.get("type") == "output_text")
                else:
                    text = None
                try:
                    if text is not None and text.strip().startswith("```"):
                        lines = text.strip().splitlines()
                        text = "\n".join(lines[1:-1]) if lines[-1].strip() == "```" else text
                    attempt["parsed"] = json.loads(text) if text is not None else None
                except json.JSONDecodeError:
                    attempt["parsed"] = None
            group["attempts"].append(attempt)
        return groups

    def sessions(self, trial):
        events = trial["events"]
        groups = self.model_groups(trial, "CINTERPRET")
        requested = [e for e in events if e.get("kind") == "dp41_model_requested"]
        sessions = {}
        for event in requested:
            key = (event.get("request_id"), event.get("input_id"))
            session = sessions.setdefault(key, {"key": key, "models": [], "reads": [], "clarifications": []})
            start = event.get("monotonic_ns", 0)
            later = [e.get("monotonic_ns", 0) for e in requested
                     if e.get("request_id") == key[0] and e.get("monotonic_ns", 0) > start]
            end = min(later) if later else float("inf")
            matches = [g for g in groups if g["request_id"] == key[0]
                       and start <= g["attempts"][0]["call"].get("enqueue_ns", 0) < end]
            if len(matches) != 1:
                raise ReplayMismatch(f"model port invocation does not have one physical-attempt group: {key}")
            group = matches[0]
            group["requested_event"] = event
            first = group["attempts"][0]
            if "input" not in session:
                content = next(m["content"] for m in first["messages"] if m.get("role") == "user")
                session["input"] = json.loads(content)
                if (session["input"].get("request_id"), session["input"].get("input_id")) != key:
                    raise ReplayMismatch("original first input IDs do not match the recorded job")
            available = [e for e in events if e.get("kind") == "MODEL_RETURN_AVAILABLE"
                         and e.get("request_id") == key[0] and e.get("input_id") == key[1]
                         and e.get("role") == "CINTERPRET" and start <= e.get("monotonic_ns", 0) < end]
            if available:
                if len(available) != 1:
                    raise ReplayMismatch("model invocation has multiple local return events")
                group["returned"] = available[0]["payload"]
                if group["attempts"][-1].get("parsed") != group["returned"]:
                    raise ReplayMismatch("local model return differs from actual raw provider output")
            else:
                failures = [e for e in events if e.get("kind") == "dp41_resolution_failed"
                            and e.get("request_id") == key[0] and e.get("input_id") == key[1]
                            and start <= e.get("monotonic_ns", 0) < end]
                if failures:
                    group["exception"] = {"type": failures[-1].get("type", "RuntimeError"),
                                          "reason": failures[-1].get("reason", "recorded model failure")}
                elif any(a["call"].get("discard_reason") == "LOCAL_CONTINUATION_CANCELLED" for a in group["attempts"]):
                    group["cancelled"] = True
                else:
                    raise ReplayMismatch("model invocation has no actual return/failure/cancellation evidence")
            session["models"].append(group)
        assigned = sum(len(s["models"]) for s in sessions.values())
        if assigned != len(groups):
            raise ReplayMismatch("unassigned CINTERPRET physical call group")
        owner_receipts = {e["receipt"]["read_id"]: e for e in events
                          if e.get("kind") == "OWNER_READ" and e.get("receipt", {}).get("read_id")}
        for key, session in sessions.items():
            own = [e for e in events if (e.get("request_id"), e.get("input_id")) == key]
            reads = [e for e in own if e.get("kind") == "dp41_read_requested"]
            returned = [e for e in own if e.get("kind") == "dp41_read_returned"]
            for request in reads:
                match = next((e for e in returned if e.get("ordinal") == request.get("ordinal")), None)
                if match is None:
                    raise ReplayMismatch("actual read has no returned receipt")
                owner = owner_receipts.get(match["result"].get("read_id"))
                if owner is None or owner["receipt"] != match["result"] or owner["spec"] != request["spec"]:
                    raise ReplayMismatch("candidate read differs from actual OWNER_READ evidence")
                session["reads"].append({"spec": request["spec"], "result": match["result"],
                                         "evidence_id": owner.get("evidence_id")})
            questions = [e for e in own if e.get("kind") == "dp41_clarification_requested"]
            answers = [e for e in own if e.get("kind") == "dp41_clarification_returned"]
            if len(questions) != len(answers):
                raise ReplayMismatch("clarification has no actual returned answer")
            session["clarifications"] = [{"question": q["question"], "scope": q.get("scope"),
                                            "answer": a["answer"], "evidence_id": a.get("evidence_id")}
                                           for q, a in zip(questions, answers)]
            proposed = [e for e in events if e.get("kind") == "MEANING_PROPOSED" and e.get("request_id") == key[0]]
            # A logical Request can have follow-up input jobs. The interval of
            # this actual job, rather than the last meaning of the Request, wins.
            own_start = session["models"][0]["requested_event"].get("monotonic_ns", 0)
            other = [s["models"][0]["requested_event"].get("monotonic_ns", 0)
                     for k, s in sessions.items() if k[0] == key[0] and k != key
                     and s["models"][0]["requested_event"].get("monotonic_ns", 0) > own_start]
            own_end = min(other) if other else float("inf")
            proposed = [e for e in proposed if own_start <= e.get("monotonic_ns", 0) < own_end]
            if len(proposed) == 1:
                session["expected"] = proposed[0]["meaning"]
                session["expected_evidence_id"] = proposed[0].get("evidence_id")
            elif not proposed and any(g.get("cancelled") for g in session["models"]):
                session["expected_cancelled"] = True
            else:
                raise ReplayMismatch("job lacks a unique actual proposed meaning")
        return list(sessions.values())


class ReplayPorts:
    def __init__(self, session):
        self.session = session
        self.models = collections.deque(session["models"])
        self.reads = collections.deque(session["reads"])
        self.clarifications = collections.deque(session["clarifications"])
        self.observations, self.errors = [], []
        self.pending_purpose = None

    def mismatch(self, message):
        self.errors.append(message)
        raise ReplayMismatch(message)

    def emit(self, kind, payload):
        self.observations.append({"kind": kind, "payload": copy.deepcopy(payload)})
        if kind == "dp41_model_requested":
            self.pending_purpose = payload.get("purpose")

    async def model(self, messages, role="CINTERPRET"):
        if role != "CINTERPRET" or not self.models:
            self.mismatch("additional model invocation or non-CINTERPRET role")
        group = self.models.popleft()
        if self.pending_purpose != group["requested_event"].get("purpose"):
            self.mismatch("interpretation model purpose/route changed")
        if _messages(messages) != _messages(group["attempts"][0]["messages"]):
            self.mismatch("user/request/source/assistant/answer model context changed")
        self.observations.append({"port": "model", "call_ids": [a["call"]["call_id"] for a in group["attempts"]],
                                  "original_context_sha256": _hash(group["attempts"][0]["messages"]),
                                  "replay_context_sha256": _hash(messages), "non_system_context_identical": True,
                                  "system_delta": messages[0].get("content") != group["attempts"][0]["messages"][0].get("content")})
        await asyncio.sleep(0)
        if group.get("cancelled"):
            raise asyncio.CancelledError
        if group.get("exception"):
            error = group["exception"]
            exception_type = type(error["type"], (RuntimeError,), {})
            raise exception_type(error["reason"])
        return copy.deepcopy(group["returned"])

    async def read(self, spec):
        if not self.reads:
            self.mismatch("additional bounded owner read")
        read = self.reads.popleft()
        if spec != read["spec"]:
            self.mismatch("bounded owner read spec changed")
        self.observations.append({"port": "read", "spec": copy.deepcopy(spec),
                                  "receipt_sha256": _hash(read["result"]), "evidence_id": read["evidence_id"]})
        return copy.deepcopy(read["result"])

    async def clarify(self, question, scope=None):
        if not self.clarifications:
            self.mismatch("additional clarification")
        row = self.clarifications.popleft()
        if question != row["question"] or scope != row["scope"]:
            self.mismatch("clarification text/scope changed")
        self.observations.append({"port": "clarify", "question": question, "scope": copy.deepcopy(scope),
                                  "answer_sha256": _hash(row["answer"]), "evidence_id": row["evidence_id"]})
        return copy.deepcopy(row["answer"])

    def complete(self):
        if self.errors or self.models or self.reads or self.clarifications:
            raise ReplayMismatch("replay did not preserve all actual ports: " + _canonical({
                "errors": self.errors, "unconsumed_models": len(self.models), "unconsumed_reads": len(self.reads),
                "unconsumed_clarifications": len(self.clarifications)}))


class ProbeEvidence:
    def __init__(self):
        self.origin, self.rows = time.monotonic(), []
    def seconds(self):
        return time.monotonic() - self.origin
    def write(self, table, row):
        saved = {"table": table, **copy.deepcopy(row)}
        self.rows.append(saved)
        return saved


async def _external_probe(trial, groups, runtime_module, variant):
    """Real copied-runtime event/publication probe; no semantic model is needed."""
    evidence, observations = ProbeEvidence(), []
    start = time.monotonic()
    scale = 100.
    queues = collections.defaultdict(collections.deque)
    for group in groups:
        queues[group["request_id"]].append(group)

    class ResponseReplay:
        async def call(self, messages, role, **kwargs):
            if role != "CRESPONSE":
                raise ReplayMismatch("external-only runtime requested semantic interpretation")
            request = kwargs.get("request_id")
            if not queues[request]:
                raise ReplayMismatch("external runtime has an additional response model call")
            group = queues[request].popleft()
            terminal = group["attempts"][-1]
            target = (terminal["call"]["at_seconds"] - trial["scene_origin_evidence_seconds"]) / scale
            await asyncio.sleep(max(0., target - (time.monotonic() - start)))
            if terminal.get("parsed") is None:
                raise ReplayMismatch("external response lacks actual parsed provider output")
            observations.append({"port": "CRESPONSE_SEPARATE", "call_ids": [a["call"]["call_id"] for a in group["attempts"]],
                                  "non_system_context_identical": _messages(messages) == _messages(group["attempts"][0]["messages"]),
                                  "recorded_context_sha256": _hash(group["attempts"][0]["messages"]),
                                  "replay_context_sha256": _hash(messages),
                                  "scope": "RESPONSE_REPLAY_NOT_SEMANTIC_ACCURACY"})
            return copy.deepcopy(terminal["parsed"])

    runtime = runtime_module.LocalVIA(copy.deepcopy(trial["initial"]), ResponseReplay(), evidence, variant,
        voice_profile={"output_onset_seconds": .001, "characters_per_second": 100000., "stop_seconds": .001})
    pending = []
    try:
        external = [e for e in trial["events"] if e.get("kind") == "EXTERNAL_EVENT_AVAILABLE"]
        for event in external:
            target = event.get("scene_seconds", 0.) / scale
            await asyncio.sleep(max(0., target - (time.monotonic() - start)))
            envelope = {"event_id": event["source_event_id"], "type": event["external_type"], "payload": event["payload"]}
            pending.append(asyncio.create_task(runtime.external_event(copy.deepcopy(envelope))))
        if pending:
            await asyncio.wait_for(asyncio.gather(*pending), 5.)
        await asyncio.wait_for(runtime.drain(), 5.)
        def publications(rows):
            return [{k: row.get(k) for k in ("source_event_id", "kind", "text", "voice_on", "voice_text", "delivered_receipt")}
                    | {"text_presented": row.get("text_presented_s") is not None,
                       "discard_reason": row.get("discard_reason")} for row in rows]
        expected, actual = publications(trial["deliveries"]), publications(runtime.deliveries)
        if expected != actual:
            raise ReplayMismatch("external-only publication content/currentness/delivery changed")
        def dispositions(rows):
            return [{k: row.get(k) for k in ("kind", "source_event_id", "reason")}
                    for row in rows if row.get("kind") in ("DUPLICATE_IGNORED", "RESULT_REJECTED", "QUESTION_REJECTED")]
        if dispositions(trial["events"]) != dispositions(evidence.rows):
            raise ReplayMismatch("external-only duplicate/currentness disposition changed")
        def question_states(rows):
            return {q["question_id"]: {k: q.get(k) for k in ("task_id", "execution_id", "revision", "status",
                    "presentation_status", "delivered_receipt", "answer_status")} for q in rows}
        if question_states(trial["final_questions"]) != question_states(runtime.owners.questions.values()):
            raise ReplayMismatch("external-only final question authority/currentness changed")
        if {t["task_id"]: t for t in trial["final_tasks"]} != runtime.owners.tasks:
            raise ReplayMismatch("external-only final Task/result state changed")
        if any(queues.values()):
            raise ReplayMismatch("external runtime omitted a recorded response model call")
        return {"passed": True, "name": "ACTUAL_EXTERNAL_EVENT_PUBLICATION_CURRENTNESS",
                "observations": observations, "events": evidence.rows, "publications": actual,
                "timing_scope": "ORDERED_COMPRESSED_EXECUTION_NOT_QA02_QA03_MEASUREMENT"}
    finally:
        for job in [*pending, *list(runtime.jobs)]:
            if not job.done():
                job.cancel()
        await asyncio.gather(*pending, *list(runtime.jobs), return_exceptions=True)


async def _late_probe(trial, actual_meanings, runtime_module, candidate, variant):
    """Probe the actual runtime's superseded-job guard with a replayed old meaning."""
    discarded = [e for e in trial["events"] if e.get("kind") == "MEANING_DISCARDED"]
    if not trial.get("old_job_created"):
        return {"passed": True, "exercised": False, "reason": "NO_ACTUAL_OLD_JOB_BRANCH_NOT_FORCED"}
    if not discarded:
        return {"passed": True, "exercised": False, "reason": "NO_RECORDED_DISCARD; cancellation covered by named runtime probe"}
    from unittest.mock import patch
    observations = []
    for event in discarded:
        expected = event["meaning"]
        if expected not in actual_meanings:
            raise ReplayMismatch("late runtime probe has no actually reproduced discarded meaning")
        evidence, gate = ProbeEvidence(), asyncio.Event()
        class NoModel:
            async def call(self, *args, **kwargs):
                raise ReplayMismatch("discarded old meaning reached response execution")
        runtime = runtime_module.LocalVIA(copy.deepcopy(trial["initial"]), NoModel(), evidence, variant,
            voice_profile={"output_onset_seconds": .001, "characters_per_second": 100000., "stop_seconds": .001})
        async def recorded_old_resolution(inp, ports):
            await gate.wait()
            return copy.deepcopy(expected)
        with patch.object(candidate, "resolve", recorded_old_resolution):
            pending = asyncio.create_task(runtime.resolve("recorded old job guard", request_id=event["request_id"],
                input_id="actual-old-currentness-probe", generation=0))
            await asyncio.sleep(0)
            root = next(e for e in trial["events"] if e.get("kind") == "INPUT_STARTED")
            await runtime.input_started(root["input_id"])
            await runtime.speech_end(root["input_id"])
            gate.set()
            result = await pending
        if result.get("status") != "DISCARDED" or runtime.effects or runtime.deliveries:
            raise ReplayMismatch("superseded old result was adopted or executed")
        observations.append({"actual_discard_evidence": event.get("evidence_id"), "result": result, "events": evidence.rows})
    return {"passed": True, "exercised": True, "observations": observations,
            "scope": "ACTUAL_RECORDED_MEANING_CURRENTNESS_PROBE_SEMANTIC_RESOLVE_REPLAYED_SEPARATELY"}


async def replay_worker(source, worker, variant, *, require_full_p21=True):
    source = Path(source).resolve()
    sys.path.insert(0, str(source))
    import importlib
    candidate = importlib.import_module("prototypes.candidates.dp41_45_poc.dp41." + variant.lower())
    runtime_module = importlib.import_module("prototypes.candidates.dp41_45_poc.shared.runtime")
    if not all(Path(module.__file__).resolve().is_relative_to(source) for module in (candidate, runtime_module)):
        raise ReplayMismatch("replay imported candidate outside the explicit copied source")
    raw = RawWorker(worker)
    by_id = {t["case_id"]: t for t in raw.trials}
    if len(by_id) != len(raw.trials):
        raise ReplayMismatch("worker must have one frozen trial per case")
    cases = []
    for case_id in CASE_IDS:
        trial = by_id.get(case_id)
        if trial is None:
            cases.append({"case_id": case_id, "passed": False, "status": "MISSING_FROZEN_TRIAL"})
            continue
        row = {"case_id": case_id, "trial_sha256": _hash(trial), "passed": False, "sessions": [],
               "evidence_level": LEVEL, "api_calls": 0}
        try:
            sessions = raw.sessions(trial)
            responses = raw.model_groups(trial, "CRESPONSE")
            row["response_calls_separate"] = [{"request_id": group["request_id"],
                "call_ids": [a["call"]["call_id"] for a in group["attempts"]],
                "scope": "CRESPONSE_EXCLUDED_FROM_CINTERPRET_SEMANTIC_REPLAY"} for group in responses]
            meanings = []
            for session in sessions:
                ports = ReplayPorts(session)
                session_row = {"request_id": session["key"][0], "input_id": session["key"][1],
                               "passed": False, "original_input_sha256": _hash(session["input"]),
                               "expected_evidence_id": session.get("expected_evidence_id"),
                               "ports": ports.observations}
                row["sessions"].append(session_row)
                cancelled = False
                try:
                    actual = await candidate.resolve(copy.deepcopy(session["input"]), ports)
                except asyncio.CancelledError:
                    actual, cancelled = None, True
                ports.complete()
                if cancelled != session.get("expected_cancelled", False) or (not cancelled and actual != session.get("expected")):
                    raise ReplayMismatch("actual resolve output differs from the recorded meaning, including failure state")
                if not cancelled:
                    meanings.append(actual)
                session_row.update({"passed": True, "actual_meaning": actual,
                    "actual_meaning_sha256": _hash(actual), "cancelled": cancelled,
                    "port_errors": ports.errors})
            if case_id in ("S5-01", "S5-02"):
                if sessions:
                    raise ReplayMismatch("external-only case unexpectedly has a semantic-model session")
                row["semantic_replay"] = "NOT_APPLICABLE_NO_USER_REQUEST_INTERPRETATION"
                row["external_runtime_probe"] = await _external_probe(trial, responses, runtime_module, variant)
            elif not sessions:
                raise ReplayMismatch("user-request case has no actual semantic-model execution")
            row["late_runtime_probe"] = await _late_probe(trial, meanings, runtime_module, candidate, variant)
            row["passed"], row["status"] = True, "RECORDED_BEHAVIOUR_PRESERVED"
        except Exception as error:
            row["error"] = f"{type(error).__name__}: {error}"
            row["traceback"] = traceback.format_exc()
            row["status"] = "REGRESSION_FAILURE"
        cases.append(row)
    present = [row for row in cases if row["status"] != "MISSING_FROZEN_TRIAL"]
    full = len(present) == 21
    summary_path = raw.directory / "worker-summary.jsonl"
    worker_complete = summary_path.exists() and any(r.get("status") == "COMPLETE" and r.get("case_count") == 21
                                                   for r in _read_jsonl(summary_path))
    return {"passed": full and worker_complete and all(row["passed"] for row in cases),
            "format_check_passed": bool(present) and all(row["passed"] for row in present),
            "full_p21": full, "worker_complete": worker_complete, "requires_full_p21": require_full_p21, "cases": cases,
            "source_root": str(source), "imported_candidate": str(Path(candidate.__file__).resolve()),
            "worker": str(Path(worker).resolve()), "artifacts": list(raw.artifacts.values()),
            "raw_stream_hashes": raw.stream_hashes,
            "evidence_level": LEVEL, "semantic_accuracy_remeasured": False, "api_calls": 0}


def regression_runner(source, variant, task_id, *, campaign=None, require_full_p21=True, audit=False):
    """Return the P21 entry consumed by run_dp41_change_jobs.

    Set VIA_DP41_REPLAY_CAMPAIGN to the frozen campaign directory, or pass it
    explicitly. Pilot subsets remain format checks and cannot pass full P21.
    """
    source = Path(source).resolve()
    campaign = campaign or os.environ.get("VIA_DP41_REPLAY_CAMPAIGN")
    if variant not in ("A", "B") or not campaign:
        raise ValueError("variant A/B and an explicit frozen replay campaign are required")
    worker = Path(campaign).resolve() / "workers" / (variant + "1")
    prefix = "baseline-p21-replay" if audit else "p21-replay"
    result_file = source.parent / (f"{prefix}-{task_id}-{variant}-{uuid.uuid4().hex}.json")
    try:
        process = subprocess.run([sys.executable, "-B", str(Path(__file__).resolve()), "--source", str(source),
            "--worker", str(worker), "--variant", variant] + ([] if require_full_p21 else ["--format-only"]),
            cwd=source, capture_output=True, text=True, timeout=60)
        if process.returncode:
            result = {"passed": False, "status": "REPLAY_PROCESS_FAILED", "returncode": process.returncode,
                      "stdout": process.stdout, "stderr": process.stderr, "api_calls": 0, "evidence_level": LEVEL}
        else:
            result = json.loads(process.stdout)
            result["stderr"] = process.stderr
    except subprocess.TimeoutExpired:
        result = {"passed": False, "status": "REPLAY_PROCESS_TIMEOUT", "api_calls": 0, "evidence_level": LEVEL}
    except (OSError, json.JSONDecodeError) as error:
        result = {"passed": False, "status": "REPLAY_PROCESS_COLLECTION_ERROR", "error": str(error),
                  "api_calls": 0, "evidence_level": LEVEL}
    result["task_id"] = task_id
    result["baseline_audit"] = audit
    result["live_checker_file_sha256"] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    with result_file.open("x", encoding="utf-8") as handle:
        handle.write(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    checks = {"P21": {**result, "evidence": [str(result_file)], "raw_result_sha256": _hash(result)}}
    if task_id == "M05":
        for case_id in ("S6-04", "S6-02"):
            row = next((c for c in result.get("cases", []) if c["case_id"] == case_id), {})
            checks[case_id] = {"passed": row.get("passed") is True,
                               "evidence": [str(result_file) + "#" + case_id], "evidence_level": LEVEL,
                               "scope": "RECORDED_SEMANTIC_PORTS; token/restore/duplicate/hold have separate execution probes",
                               "trial_sha256": row.get("trial_sha256")}
    return checks


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--worker", type=Path, required=True)
    parser.add_argument("--variant", choices=("A", "B"), required=True)
    parser.add_argument("--format-only", action="store_true")
    args = parser.parse_args()
    print(_canonical(asyncio.run(replay_worker(args.source, args.worker, args.variant,
                                              require_full_p21=not args.format_only))))
