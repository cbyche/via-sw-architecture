#!/usr/bin/env python3
"""Executable A/B reference candidates for VIA-DP-02/05/09/12/13.

The candidates share fixtures and completion rules.  They differ only at the
authority, contract, ordering, or resource-ownership boundary named by the DP.
"""

from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor, wait
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import threading
import time
from typing import Any, Callable


def _elapsed_ms(start_ns: int) -> float:
    return (time.perf_counter_ns() - start_ns) / 1_000_000


def _validate_contract(value: Any, rounds: int = 48) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    digest = payload
    for _ in range(rounds):
        digest = hashlib.sha256(digest).digest()
    return digest.hex()


def _event(trace: list[dict[str, Any]], name: str, start_ns: int, **fields: Any) -> None:
    trace.append({"event": name, "offset_ns": time.perf_counter_ns() - start_ns, **fields})


def _dp05(candidate: str, case: dict[str, Any], _: Path) -> dict[str, Any]:
    start = time.perf_counter_ns()
    trace: list[dict[str, Any]] = []
    sources = {
        "doc:v1": {"budget": 120, "unit": "million_krw"},
        "doc:v2": {"budget": 125, "unit": "million_krw"},
        "mail:v1": {"quoted_budget": 118, "sender": "kim"},
        "calendar:v1": {"meeting": "16:00"},
    }
    required = list(case["required_refs"])
    initial = list(case["initial_refs"])
    revoked = set(case.get("revoked_refs", []))
    _event(trace, "request_received", start, case_id=case["id"])
    generation_count = 1
    read_set: list[str] = []
    denied = False

    if candidate == "A":
        manifest = {"generation": 1, "refs": initial}
        _validate_contract(manifest)
        _event(trace, "context_manifest_committed", start, generation=1, refs=initial)
        missing = [ref for ref in required if ref not in initial]
        if missing:
            generation_count += 1
            manifest = {"generation": 2, "refs": sorted(set(initial + missing))}
            _validate_contract(manifest)
            _event(trace, "context_manifest_committed", start, generation=2, refs=manifest["refs"])
        for ref in required:
            if ref in revoked:
                denied = True
                _event(trace, "context_read_denied", start, ref=ref)
                break
            read_set.append(ref)
            _event(trace, "context_value_read", start, ref=ref)
    else:
        capability = {"generation": 1, "allowed_prefixes": ["doc", "mail", "calendar"]}
        _validate_contract(capability)
        _event(trace, "context_capability_issued", start, generation=1)
        for ref in required:
            _validate_contract({"capability": capability, "ref": ref})
            if ref in revoked:
                denied = True
                _event(trace, "context_read_denied", start, ref=ref)
                break
            read_set.append(ref)
            _event(trace, "context_read_set_extended", start, ref=ref)

    values = [sources[ref] for ref in read_set]
    expected_denied = bool(revoked.intersection(required))
    semantic_correct = denied == expected_denied and (denied or read_set == required)
    continuity_correct = case.get("continuity", False) is False or (
        "doc:v1" in read_set and sources["doc:v1"]["budget"] == 120
    )
    strict = semantic_correct and continuity_correct
    _event(trace, "request_completed", start, denied=denied, refs=read_set)
    return {
        "execution_ms": _elapsed_ms(start),
        "strict_success": strict,
        "semantic_correct": semantic_correct,
        "continuity_correct": continuity_correct,
        "exposure_excess": 0,
        "trace_complete": {e["event"] for e in trace}.issuperset({"request_received", "request_completed"}),
        "generation_count": generation_count,
        "read_set": read_set,
        "values_digest": _validate_contract(values, rounds=1),
        "trace": trace,
    }


def _canonical_agent_meaning(native: dict[str, Any]) -> dict[str, Any]:
    kind = native["kind"]
    mapping = {
        "p_cancel_confirmed": "CANCELLED",
        "q_cancel_requested": "CANCEL_PENDING",
        "completion_after_cancel": "COMPLETED",
        "unsupported": "UNSUPPORTED",
        "question": "WAITING_USER",
        "artifact": "COMPLETED",
        "paused": "PAUSED",
        "progress": "RUNNING",
        "failed": "FAILED",
    }
    return {"task_state": mapping[kind], "native_revision": native["revision"]}


def _typed_agent_contract(native: dict[str, Any]) -> dict[str, Any]:
    return {
        "variant": native["kind"],
        "revision": native["revision"],
        "provider": native["provider"],
    }


def _core_lifecycle_meaning(typed: dict[str, Any]) -> dict[str, Any]:
    return _canonical_agent_meaning({
        "kind": typed["variant"],
        "revision": typed["revision"],
        "provider": typed["provider"],
    })


def _dp09(candidate: str, case: dict[str, Any], _: Path) -> dict[str, Any]:
    start = time.perf_counter_ns()
    trace: list[dict[str, Any]] = []
    native = {"kind": case["native_kind"], "revision": case.get("revision", 1), "provider": case["provider"]}
    _event(trace, "native_agent_event", start, **native)
    control_start = time.perf_counter_ns() if case.get("control") else None
    if candidate == "A":
        meaning = _canonical_agent_meaning(native)
        _event(trace, "edge_semantic_normalized", start, task_state=meaning["task_state"])
    else:
        typed = _typed_agent_contract(native)
        _event(trace, "typed_contract_received", start, variant=typed["variant"])
        meaning = _core_lifecycle_meaning(typed)
        _event(trace, "core_lifecycle_interpreted", start, task_state=meaning["task_state"])
    _event(trace, "task_disposition_presented", start, task_state=meaning["task_state"])
    expected = case["expected_state"]
    strict = meaning["task_state"] == expected
    common_change = native["kind"] not in {"paused", "failed"}
    if candidate == "A":
        changed = ["C-AGENT-SEMANTIC-ADAPTER"] if common_change else [
            "C-AGENT-SEMANTIC-ADAPTER", "I-AGENT-CANONICAL", "C-TASK-OWNER"
        ]
    else:
        changed = ["I-AGENT-TYPED", "C-CORE-LIFECYCLE-HANDLER"]
    return {
        "execution_ms": _elapsed_ms(start),
        "control_ms": _elapsed_ms(control_start) if control_start else None,
        "strict_success": strict,
        "binding_correct": strict,
        "converged": strict,
        "change_elements": changed,
        "exposure_excess": 0,
        "trace_complete": len(trace) >= 3,
        "trace": trace,
    }


def _open_db(path: Path) -> sqlite3.Connection:
    connection = sqlite3.connect(path)
    connection.execute("PRAGMA journal_mode=WAL")
    connection.execute("PRAGMA synchronous=FULL")
    return connection


def _dp02_schema(connection: sqlite3.Connection) -> None:
    connection.executescript(
        "CREATE TABLE IF NOT EXISTS state(k TEXT PRIMARY KEY,v TEXT NOT NULL);"
        "CREATE TABLE IF NOT EXISTS pending(k TEXT PRIMARY KEY,v TEXT NOT NULL);"
    )
    connection.commit()


def _put(connection: sqlite3.Connection, table: str, key: str, value: str) -> None:
    connection.execute(
        f"INSERT INTO {table}(k,v) VALUES(?,?) ON CONFLICT(k) DO UPDATE SET v=excluded.v",
        (key, value),
    )


def _dp02(candidate: str, case: dict[str, Any], work_dir: Path) -> dict[str, Any]:
    start = time.perf_counter_ns()
    control_start: int | None = None
    trace: list[dict[str, Any]] = []
    command = f"{case['id']}-command"
    task_id = case.get("task_id", "T-PPT")
    expected_state = case["expected_state"]
    recovery_ms: float | None = None
    _event(trace, "relation_command_received", start, command_id=command, task_id=task_id)

    if candidate == "A":
        db = _open_db(work_dir / "joint.sqlite")
        _dp02_schema(db)
        try:
            control_start = time.perf_counter_ns() if case.get("control") else None
            db.execute("BEGIN IMMEDIATE")
            if case.get("inject_crash"):
                db.rollback()
                db.close()
                recovery_start = time.perf_counter_ns()
                db = _open_db(work_dir / "joint.sqlite")
                _dp02_schema(db)
                db.execute("BEGIN IMMEDIATE")
                _put(db, "state", f"conversation:{command}", "linked")
                _put(db, "state", f"task:{task_id}", expected_state)
                _put(db, "state", f"relation:{command}", "confirmed")
                db.commit()
                recovery_ms = _elapsed_ms(recovery_start)
                _event(trace, "joint_commit_recovered", start, command_id=command)
            else:
                _put(db, "state", f"conversation:{command}", "linked")
                _put(db, "state", f"task:{task_id}", expected_state)
                _put(db, "state", f"relation:{command}", "confirmed")
                db.commit()
                _event(trace, "joint_commit_completed", start, command_id=command)
            state = dict(db.execute("SELECT k,v FROM state"))
        finally:
            db.close()
    else:
        conv = _open_db(work_dir / "conversation.sqlite")
        task = _open_db(work_dir / "task.sqlite")
        relation = _open_db(work_dir / "relation.sqlite")
        for db in (conv, task, relation):
            _dp02_schema(db)
        try:
            control_start = time.perf_counter_ns() if case.get("control") else None
            _put(conv, "state", f"conversation:{command}", "reserved")
            _put(conv, "pending", command, task_id)
            conv.commit()
            _event(trace, "conversation_reservation_committed", start, command_id=command)
            if case.get("inject_crash"):
                conv.close(); task.close(); relation.close()
                recovery_start = time.perf_counter_ns()
                _event(trace, "reconciler_restarted", start, command_id=command)
                conv = _open_db(work_dir / "conversation.sqlite")
                task = _open_db(work_dir / "task.sqlite")
                relation = _open_db(work_dir / "relation.sqlite")
                for db in (conv, task, relation):
                    _dp02_schema(db)
            _put(task, "state", f"task:{task_id}", expected_state)
            task.commit()
            _event(trace, "task_owner_committed", start, task_id=task_id)
            _put(relation, "state", f"relation:{command}", "confirmed")
            relation.commit()
            conv.execute("DELETE FROM pending WHERE k=?", (command,))
            _put(conv, "state", f"conversation:{command}", "linked")
            conv.commit()
            if case.get("inject_crash"):
                recovery_ms = _elapsed_ms(recovery_start)
            state = dict(conv.execute("SELECT k,v FROM state"))
            state.update(dict(task.execute("SELECT k,v FROM state")))
            state.update(dict(relation.execute("SELECT k,v FROM state")))
            _event(trace, "relation_reconciliation_completed", start, command_id=command)
        finally:
            conv.close(); task.close(); relation.close()

    binding = state.get(f"relation:{command}") == "confirmed" and state.get(f"conversation:{command}") == "linked"
    converged = state.get(f"task:{task_id}") == expected_state
    _event(trace, "task_disposition_presented", start, state=state.get(f"task:{task_id}"))
    return {
        "execution_ms": _elapsed_ms(start),
        "control_ms": _elapsed_ms(control_start) if control_start else None,
        "recovery_ms": recovery_ms,
        "strict_success": binding and converged,
        "binding_correct": binding,
        "converged": converged,
        "continuity_correct": binding,
        "excess_interrupted_units": 0,
        "exposure_excess": 0,
        "trace_complete": len(trace) >= 3,
        "trace": trace,
    }


def _sleep_work(ms: float) -> str:
    time.sleep(ms / 1000)
    return "completed"


def _dp13(candidate: str, case: dict[str, Any], _: Path) -> dict[str, Any]:
    start = time.perf_counter_ns()
    trace: list[dict[str, Any]] = []
    workers = int(case["workers"])
    jobs = int(case["jobs"])
    work_ms = float(case["work_ms"])
    control_delay_ms = float(case["control_delay_ms"])
    control_done = threading.Event()
    _event(trace, "workload_started", start, jobs=jobs, workers=workers)

    def control() -> str:
        _event(trace, "control_execution_started", start)
        control_done.set()
        return "correct_disposition"

    futures = []
    if candidate == "A":
        _event(trace, "reserved_control_slot_available", start)
        with ThreadPoolExecutor(max_workers=workers - 1) as work_pool, ThreadPoolExecutor(max_workers=1) as control_pool:
            for _index in range(jobs):
                futures.append(work_pool.submit(_sleep_work, work_ms))
            time.sleep(control_delay_ms / 1000)
            control_start = time.perf_counter_ns()
            control_future = control_pool.submit(control)
            disposition = control_future.result()
            control_ms = _elapsed_ms(control_start)
            wait(futures)
    else:
        _event(trace, "shared_pool_may_be_fully_occupied", start)
        with ThreadPoolExecutor(max_workers=workers) as shared:
            initial = min(workers, jobs)
            for _index in range(initial):
                futures.append(shared.submit(_sleep_work, work_ms))
            time.sleep(control_delay_ms / 1000)
            control_start = time.perf_counter_ns()
            control_future = shared.submit(control)
            # The control item is first in the pending queue: priority without a reserved slot.
            disposition = control_future.result()
            control_ms = _elapsed_ms(control_start)
            for _index in range(initial, jobs):
                futures.append(shared.submit(_sleep_work, work_ms))
            wait(futures)
    _event(trace, "workload_completed", start, jobs=jobs)
    strict = disposition == "correct_disposition" and all(f.result() == "completed" for f in futures)
    return {
        "execution_ms": _elapsed_ms(start),
        "control_ms": control_ms,
        "work_makespan_ms": _elapsed_ms(start),
        "strict_success": strict,
        "binding_correct": disposition == "correct_disposition",
        "converged": strict,
        "trace_complete": control_done.is_set() and len(trace) >= 3,
        "trace": trace,
    }


def _durable_append(path: Path, record: dict[str, Any]) -> None:
    with path.open("ab", buffering=0) as handle:
        handle.write((json.dumps(record, sort_keys=True) + "\n").encode())
        os.fsync(handle.fileno())


def _dp12(candidate: str, case: dict[str, Any], work_dir: Path) -> dict[str, Any]:
    start = time.perf_counter_ns()
    control_start = time.perf_counter_ns() if case.get("control") else None
    trace: list[dict[str, Any]] = []
    spool = work_dir / "evidence.jsonl"
    record = {"request_id": case["id"], "decision": "correct", "schema": 1}
    _event(trace, "response_ready", start, request_id=case["id"])
    published = False
    durable = False
    writer_failed = bool(case.get("writer_failure"))
    crash_before_flush = bool(case.get("crash_before_flush"))

    if candidate == "A":
        if not writer_failed:
            _durable_append(spool, record)
            durable = True
            _event(trace, "evidence_durable", start, request_id=case["id"])
            published = True
            _event(trace, "response_published", start, request_id=case["id"])
        else:
            _event(trace, "evidence_write_failed", start, request_id=case["id"])
            _event(trace, "response_withheld", start, request_id=case["id"])
    else:
        volatile_queue = [record]
        _event(trace, "evidence_enqueued", start, request_id=case["id"])
        published = True
        _event(trace, "response_published", start, request_id=case["id"])
        if not writer_failed and not crash_before_flush:
            _durable_append(spool, volatile_queue.pop())
            durable = True
            _event(trace, "evidence_durable", start, request_id=case["id"])
        elif crash_before_flush:
            volatile_queue.clear()
            _event(trace, "process_crashed_before_flush", start, request_id=case["id"])
        else:
            _event(trace, "evidence_write_failed", start, request_id=case["id"])

    # User completion is deliberately distinct from a truthful diagnostic record:
    # A's fail-closed withholding is observable as a request failure, while B can
    # complete the request and still fail trace completeness.
    strict = published
    control_ms = None
    if control_start is not None:
        control_ms = _elapsed_ms(control_start) if published else float(case["control_timeout_ms"])
    trace_complete = durable and published
    changed = (
        ["I-EVIDENCE-SCHEMA", "C-EVIDENCE-WRITER", "C-PUBLISH-GATE"]
        if candidate == "A"
        else ["I-EVIDENCE-SCHEMA", "C-EVIDENCE-WRITER"]
    )
    return {
        "execution_ms": _elapsed_ms(start),
        "control_ms": control_ms,
        "strict_success": strict,
        "published": published,
        "durable": durable,
        "change_elements": changed,
        "exposure_excess": 0,
        "trace_complete": trace_complete,
        "trace": trace,
    }


RUNNERS: dict[str, Callable[[str, dict[str, Any], Path], dict[str, Any]]] = {
    "VIA-DP-02": _dp02,
    "VIA-DP-05": _dp05,
    "VIA-DP-09": _dp09,
    "VIA-DP-12": _dp12,
    "VIA-DP-13": _dp13,
}


def run_candidate(dp: str, candidate: str, case: dict[str, Any], work_dir: Path) -> dict[str, Any]:
    if candidate not in {"A", "B"}:
        raise ValueError(f"unknown candidate: {candidate}")
    work_dir.mkdir(parents=True, exist_ok=True)
    result = RUNNERS[dp](candidate, case, work_dir)
    return {"dp": dp, "candidate": candidate, "case_id": case["id"], **result}
