#!/usr/bin/env python3
"""Run VIA-DP-14 through actual Task writers, model, Agent and audio endpoints."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from benchmark.architecture._dp06_semantic_candidates import (  # noqa: E402
    compile_probe, ensure_model, evaluate_semantic, generate_wav, probe_audio, run_a,
)
from benchmark.architecture.integrated_runtime import RustAgentHost  # noqa: E402
from benchmark.architecture.run_dp06_evaluation import (  # noqa: E402
    ReferenceAgent, load_json, reply_run_id,
)
from benchmark.architecture.task_host_client import (  # noqa: E402
    TASK_HOST, TaskHostClient, accept_command, cancel_command, cancelled_command,
    create_command, progress_command, result_command,
)


CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-14-evaluation-v1.json"
DP06_FIXTURE = ROOT / "benchmark/architecture/fixtures/dp06-semantic-input-v4.json"
DP06_ORACLE = ROOT / "benchmark/architecture/fixtures/dp06-semantic-oracle-v4.json"
MODEL_CASES = {"T-CREATE": "TC-W01", "T-CANCEL": "TC-C03"}


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def append_jsonl(path: Path, value: dict[str, Any]) -> None:
    with path.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n")


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def agent_reply_run_id(response: dict[str, Any]) -> str:
    return reply_run_id(response)


def event_time(events: list[dict[str, Any]], name: str) -> int | None:
    values = [int(item["source_monotonic_ns"]) for item in events if item.get("event") == name]
    return values[-1] if values else None


def render(text: str, directory: Path) -> dict[str, Any]:
    directory.mkdir(parents=True, exist_ok=True)
    wav = directory / "response.wav"
    generate_wav(text, wav)
    return probe_audio(wav, directory / "loopback")


def input_audio(text: str, directory: Path) -> dict[str, Any]:
    directory.mkdir(parents=True, exist_ok=True)
    wav = directory / "request.wav"
    generate_wav(text, wav)
    return probe_audio(wav, directory / "loopback")


def model_call(
    endpoint: str,
    fixture: dict[str, Any],
    oracle: dict[str, Any],
    logical_case: str,
) -> tuple[dict[str, Any], list[dict[str, Any]], dict[str, Any]]:
    case_id = MODEL_CASES[logical_case]
    case = next(item for item in fixture["cases"] if item["id"] == case_id)
    expected = next(item["expected"] for item in oracle["cases"] if item["id"] == case_id)
    output, calls = run_a(endpoint, fixture, case)
    return output, calls, evaluate_semantic(output, expected)


def setup_running_task(
    task: TaskHostClient,
    agent: RustAgentHost,
    task_id: str,
    key: str,
    goal: str,
) -> tuple[dict[str, Any], str, list[dict[str, Any]]]:
    events: list[dict[str, Any]] = []
    view, _ = task.apply(create_command(task_id, f"{key}:create", goal))
    accepted = agent.request({
        "op": "submit",
        "request": {"task_id": task_id, "submission_key": key, "goal": goal},
    })
    run_id = agent_reply_run_id(accepted)
    view, _ = task.apply(accept_command(task_id, f"{key}:accept", view["revision"], run_id))
    events.extend(agent.drain_events())
    return view, run_id, events


def trace_complete(record: dict[str, Any]) -> bool:
    required = {
        "candidate", "case_id", "trial", "stratum", "source_execution_key",
        "component_path", "events", "oracle", "outcome", "memory_observation",
    }
    return required.issubset(record) and all(record.get(key) is not None for key in required)


def execute_case(
    *,
    endpoint: str,
    semantic_pid: int,
    candidate: str,
    case_id: str,
    trial: int,
    stratum: str,
    warmup: bool,
    task: TaskHostClient,
    agent: RustAgentHost,
    fixture: dict[str, Any],
    oracle_fixture: dict[str, Any],
    output_dir: Path,
) -> dict[str, Any]:
    directory = output_dir / "artifacts" / stratum / ("warmup" if warmup else "scored") / candidate / case_id / f"trial-{trial:02d}"
    task_id = f"{candidate}-{case_id}-{stratum}-{trial}"
    prefix = f"{candidate}:{case_id}:{stratum}:{trial}"
    events: list[dict[str, Any]] = []
    model_calls: list[dict[str, Any]] = []
    semantic_fields: dict[str, Any] | None = None
    request_audio = None
    output_audio = None
    latency_qa = None
    latency_ns = None
    field_results: dict[str, bool] = {}
    final_state = None
    error = None
    started = time.monotonic_ns()
    try:
        if case_id == "T-CREATE":
            text = "이 예산안으로 5장 발표자료를 만들어줘"
            request_audio = input_audio(text, directory / "input")
            semantic, model_calls, semantic_fields = model_call(endpoint, fixture, oracle_fixture, case_id)
            field_results["semantic"] = semantic_fields["pass"]
            agent.drain_events()
            view, run_id, setup_events = setup_running_task(task, agent, task_id, prefix, text)
            events.extend(setup_events)
            agent.request({"op": "complete", "run_id": run_id, "artifact": f"artifact-{task_id}"})
            events.extend(agent.drain_events())
            view, _ = task.apply(result_command(
                task_id, f"{prefix}:result", view["revision"], run_id, 2, f"artifact-{task_id}"
            ))
            output_audio = render("발표자료 작업을 완료했습니다.", directory / "output")
            ingress = event_time(events, "agent_request_available_at_agent_ingress")
            result_source = event_time(events, "agent_result_available_at_source")
            input_end = int(request_audio["detectedLastMonotonicNs"])
            onset = int(output_audio["detectedOnsetMonotonicNs"])
            latency_qa = "QA-01"
            latency_ns = (
                (ingress - input_end) + (onset - result_source)
                if None not in (ingress, result_source) else None
            )
            final_state = view["state"]
            field_results.update(task_completed=final_state == "completed", binding=view["run_id"] == run_id)
        elif case_id == "T-STATUS":
            view, run_id, setup_events = setup_running_task(task, agent, task_id, prefix, "status Task")
            events.extend(setup_events)
            agent.drain_events()
            agent.request({"op": "emit_progress", "run_id": run_id, "percent": 40})
            events.extend(agent.drain_events())
            agent.request({"op": "events_since", "run_id": run_id, "after_revision": 1})
            view, _ = task.apply(progress_command(
                task_id, f"{prefix}:progress", view["revision"], run_id, 2, 40
            ))
            output_audio = render("작업은 40퍼센트 진행 중입니다.", directory / "output")
            status_source = event_time(events, "agent_status_available_at_source")
            latency_qa = "QA-03"
            latency_ns = (
                int(output_audio["detectedOnsetMonotonicNs"]) - status_source
                if status_source is not None else None
            )
            final_state = view["state"]
            field_results.update(progress_committed=final_state == "running", binding=view["run_id"] == run_id)
        elif case_id == "T-CANCEL":
            view, run_id, setup_events = setup_running_task(task, agent, task_id, prefix, "cancel Task")
            events.extend(setup_events)
            text = "아니, 메일 말고 발표자료 작업을 취소해줘"
            request_audio = input_audio(text, directory / "input")
            semantic, model_calls, semantic_fields = model_call(endpoint, fixture, oracle_fixture, case_id)
            field_results["semantic"] = semantic_fields["pass"]
            view, _ = task.apply(cancel_command(task_id, f"{prefix}:cancel", view["revision"]))
            agent.request({"op": "cancel", "run_id": run_id})
            agent.request({"op": "confirm_cancel", "run_id": run_id})
            view, _ = task.apply(cancelled_command(
                task_id, f"{prefix}:cancelled", view["revision"], run_id, 3
            ))
            output_audio = render("발표자료 작업을 취소했습니다.", directory / "output")
            latency_qa = "QA-05"
            latency_ns = int(output_audio["detectedOnsetMonotonicNs"]) - int(request_audio["detectedLastMonotonicNs"])
            final_state = view["state"]
            field_results.update(task_cancelled=final_state == "cancelled", binding=view["run_id"] == run_id)
        elif case_id == "T-RACE":
            view, run_id, setup_events = setup_running_task(task, agent, task_id, prefix, "race Task")
            events.extend(setup_events)
            race, _ = task.request({
                "op": "race_cancel_result", "task_id": task_id, "run_id": run_id,
                "expected_revision": view["revision"], "command_prefix": f"{prefix}:race",
                "source_revision": 2, "artifact": f"artifact-{task_id}",
            })
            if race.get("status") != "race":
                raise RuntimeError(f"race failed: {race}")
            view = race["final_view"]
            one_winner = int(race["cancel"]["ok"]) + int(race["result"]["ok"]) == 1
            if view["state"] == "cancel_requested":
                agent.request({"op": "confirm_cancel", "run_id": run_id})
                view, _ = task.apply(cancelled_command(
                    task_id, f"{prefix}:race-cancelled", view["revision"], run_id, 3
                ))
            final_state = view["state"]
            field_results.update(one_committed_race_winner=one_winner, terminal=final_state in {"completed", "cancelled"})
            events.append({"event": "task_race", "race": race})
        elif case_id == "T-MULTI":
            views = []
            for index in range(4):
                view, run_id, setup_events = setup_running_task(
                    task, agent, f"{task_id}-{index}", f"{prefix}:{index}", f"Task {index}"
                )
                views.append((view, run_id))
                events.extend(setup_events)
            for index, (view, run_id) in enumerate(views[1:], start=1):
                updated, _ = task.apply(progress_command(
                    view["task_id"], f"{prefix}:{index}:progress", view["revision"], run_id, 2, 40
                ))
                views[index] = (updated, run_id)
            final_state = "running"
            field_results.update(
                all_four_bound=len({view["run_id"] for view, _ in views}) == 4,
                unrelated_tasks_progressed=all(view["state"] == "running" for view, _ in views[1:]),
            )
        else:
            raise ValueError(f"unknown DP-14 case: {case_id}")
    except Exception as exc:
        error = f"{type(exc).__name__}: {exc}"

    candidate_pids = [task.process.pid]
    candidate_rss = subprocess.check_output(
        ["ps", "-o", "rss=", "-p", str(task.process.pid)], text=True
    ).strip()
    model_rss = subprocess.check_output(["ps", "-o", "rss=", "-p", str(semantic_pid)], text=True).strip()
    memory = (int(candidate_rss or 0) + int(model_rss or 0)) * 1024
    record = {
        "schema_version": "via.dp14.trial.v1",
        "candidate": candidate, "case_id": case_id, "trial": trial,
        "stratum": stratum, "warmup": warmup,
        "source_execution_key": f"{stratum}:{candidate}:{case_id}:{trial}",
        "component_path": ["voice_input", "shared_semantic_model", "rust_task_host", "rust_agent_worker", "external_reference_agent", "voice_output"],
        "events": events, "model_calls": model_calls, "semantic_fields": semantic_fields,
        "input_audio_report": request_audio, "output_audio_report": output_audio,
        "latency_qa": latency_qa, "latency_ns": latency_ns,
        "field_results": field_results,
        "oracle": {"all_fields_must_pass": True, "terminal_or_running_state": final_state},
        "final_state": final_state,
        "outcome": "PASS" if error is None and field_results and all(field_results.values()) else "FAIL",
        "error": error, "elapsed_ns": time.monotonic_ns() - started,
        "memory_observation": {"candidate_pids": candidate_pids, "total_rss_bytes": memory},
    }
    record["trace_complete"] = trace_complete(record)
    return record


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--semantic-endpoint", default="http://127.0.0.1:18080")
    parser.add_argument("--semantic-pid", type=int, required=True)
    parser.add_argument("--qualification", action="store_true")
    args = parser.parse_args()
    ensure_model(args.semantic_endpoint)
    compile_probe()
    if not TASK_HOST.exists():
        raise RuntimeError(f"missing {TASK_HOST}")
    contract = load_json(CONTRACT)
    fixture = load_json(DP06_FIXTURE)
    oracle = load_json(DP06_ORACLE)
    output_dir = args.output_dir.resolve()
    if output_dir.exists() and any(output_dir.iterdir()):
        raise RuntimeError(f"refusing to overwrite non-empty result directory: {output_dir}")
    (output_dir / "raw").mkdir(parents=True, exist_ok=True)
    raw_path = output_dir / "raw/trials.jsonl"
    warmup_path = output_dir / "raw/warmups.jsonl"
    raw_path.touch()
    warmup_path.touch()
    manifest = {
        "campaign_id": output_dir.name, "status": "RUNNING",
        "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "contract_sha256": sha256_file(CONTRACT), "fixture_sha256": sha256_file(DP06_FIXTURE),
        "oracle_sha256": sha256_file(DP06_ORACLE), "semantic_pid": args.semantic_pid,
        "qualification_only": args.qualification,
    }
    write_json(output_dir / "manifest.json", manifest)
    candidates = contract["candidates"]
    cases = list(contract["normal_cases"])
    breadth = ["T-CREATE", "T-STATUS", "T-CANCEL"] if args.qualification else cases
    agent_source = ReferenceAgent()
    agent = RustAgentHost("isolated", agent_source)
    hosts: dict[str, TaskHostClient] = {}
    try:
        for candidate in candidates:
            hosts[candidate] = TaskHostClient(candidate, output_dir / "raw" / f"{candidate}.sqlite")
        for case_id in breadth:
            for candidate in candidates:
                print(f"[breadth] {candidate} {case_id}", flush=True)
                append_jsonl(raw_path, execute_case(
                    endpoint=args.semantic_endpoint, semantic_pid=args.semantic_pid,
                    candidate=candidate, case_id=case_id, trial=1,
                    stratum="correctness_breadth", warmup=False,
                    task=hosts[candidate], agent=agent, fixture=fixture,
                    oracle_fixture=oracle, output_dir=output_dir,
                ))
        latency_cases = list(contract["latency_case_qa_mapping"])
        warmups = 0 if args.qualification else contract["repetitions"]["latency_warmup_per_case_per_candidate"]
        scored = 1 if args.qualification else contract["repetitions"]["latency_scored_per_case_per_candidate"]
        for warmup, count, destination in ((True, warmups, warmup_path), (False, scored, raw_path)):
            for trial in range(1, count + 1):
                for case_index, case_id in enumerate(latency_cases):
                    order = list(candidates)
                    if (trial + case_index) % 2:
                        order.reverse()
                    for candidate in order:
                        print(f"[{'warmup' if warmup else 'latency'}] {candidate} {case_id} {trial}", flush=True)
                        append_jsonl(destination, execute_case(
                            endpoint=args.semantic_endpoint, semantic_pid=args.semantic_pid,
                            candidate=candidate, case_id=case_id, trial=trial,
                            stratum="latency_repetition", warmup=warmup,
                            task=hosts[candidate], agent=agent, fixture=fixture,
                            oracle_fixture=oracle, output_dir=output_dir,
                        ))
    finally:
        for host in hosts.values():
            host.close()
        agent.close()
        agent_source.close()

    # Fresh-state restart/reopen recovery. The same durable DB survives host replacement.
    fault_count = 1 if args.qualification else contract["repetitions"]["recovery_faults_per_candidate"]
    for candidate in candidates:
        database = output_dir / "raw" / f"fault-{candidate}.sqlite"
        for trial in range(1, fault_count + 1):
            task_id = f"FAULT-{candidate}-{trial}"
            host = TaskHostClient(candidate, database)
            view, _ = host.apply(create_command(task_id, f"fault:{trial}:create", "recoverable Task"))
            view, _ = host.apply(accept_command(task_id, f"fault:{trial}:accept", view["revision"], f"run-{trial}"))
            started = time.monotonic_ns()
            host.close()
            host = TaskHostClient(candidate, database)
            recovered, _ = host.get(task_id)
            controlled, _ = host.apply(cancel_command(task_id, f"fault:{trial}:cancel", recovered["revision"]))
            elapsed = time.monotonic_ns() - started
            host.close()
            record = {
                "schema_version": "via.dp14.fault.v1", "candidate": candidate,
                "case_id": "F-RESTART", "trial": trial, "stratum": "fault", "warmup": False,
                "source_execution_key": f"fault:{candidate}:{trial}",
                "component_path": ["rust_task_host", "sqlite_repository"], "events": [],
                "oracle": {"state_before": "running", "state_after_control": "cancel_requested"},
                "outcome": "PASS" if recovered["state"] == "running" and controlled["state"] == "cancel_requested" else "FAIL",
                "recovery_ns": elapsed, "excess_affected_units": 0,
                "memory_observation": {"candidate_pids": [], "total_rss_bytes": 0},
            }
            record["trace_complete"] = trace_complete(record)
            append_jsonl(raw_path, record)
    manifest["status"] = "RAW_COMPLETE_PENDING_ANALYSIS"
    manifest["raw_sha256"] = sha256_file(raw_path)
    manifest["warmup_sha256"] = sha256_file(warmup_path)
    write_json(output_dir / "manifest.json", manifest)
    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
