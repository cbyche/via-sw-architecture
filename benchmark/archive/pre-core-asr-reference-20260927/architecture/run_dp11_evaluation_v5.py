#!/usr/bin/env python3
"""Run VIA-DP-11 through the integrated Voice/model/Rust-host/Agent path."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

import benchmark.architecture.run_dp11_complete as legacy  # noqa: E402
from benchmark.architecture._dp06_semantic_candidates import (  # noqa: E402
    compile_probe,
    ensure_model,
    generate_wav,
    probe_audio,
)
from benchmark.architecture.run_dp06_evaluation import ReferenceAgent as EventReferenceAgent  # noqa: E402


CONTRACT = ROOT / "benchmark/architecture/contracts/via-dp-11-evaluation-v5.json"
NORMAL_FIXTURE = ROOT / "benchmark/architecture/fixtures/dp11-normal-v1.json"
SEMANTIC_CONTEXT = ROOT / "benchmark/architecture/fixtures/dp11-semantic-context-v2.json"
CHANGE_LEDGER = ROOT / "benchmark/architecture/fixtures/dp11-change-ledger-v1.json"
TARGET = ROOT / "prototypes/candidates/target/debug"
HOST_BIN = TARGET / "via-host"
WORKER_BIN = TARGET / "via-worker"
REFERENCE_AGENT_BIN = TARGET / "via-reference-agent"


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value: Any) -> None:
    path.write_text(
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def append_jsonl(path: Path, value: dict[str, Any]) -> None:
    with path.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n")


class LegacyAgentAdapter:
    """Expose source events while preserving the existing fixture-control API."""

    def __init__(self) -> None:
        self.source = EventReferenceAgent()
        self.address = self.source.address
        self.process = self.source.process

    def request(self, value: dict[str, Any], timeout: float = 5.0) -> tuple[dict[str, Any], int]:
        del timeout
        started = time.monotonic_ns()
        response = self.source.request(value)
        return response, time.monotonic_ns() - started

    def drain_events(self, settle_ms: int = 30) -> list[dict[str, Any]]:
        return self.source.drain_events(settle_ms)

    def stop(self) -> None:
        self.source.close()


class PhysicalRenderer:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.counter = 0
        self.reports: list[dict[str, Any]] = []

    def __call__(self, text: str) -> int:
        self.counter += 1
        directory = self.root / f"render-{self.counter:05d}"
        directory.mkdir(parents=True, exist_ok=True)
        wav = directory / "response.wav"
        started = time.monotonic_ns()
        generate_wav(text, wav)
        report = probe_audio(wav, directory / "loopback")
        report["renderer_start_monotonic_ns"] = started
        report["text_sha256"] = hashlib.sha256(text.encode()).hexdigest()
        self.reports.append(report)
        return int(report["detectedOnsetMonotonicNs"]) - started

    def take_last(self) -> dict[str, Any] | None:
        return self.reports[-1] if self.reports else None


def relevant_cases(cases: list[dict[str, Any]]) -> list[dict[str, Any]]:
    bypass = {"direct", "direct_followup", "clarification", "barge_in_direct"}
    return [case for case in cases if case["kind"] not in bypass]


def event_time(events: list[dict[str, Any]], name: str) -> int | None:
    values = [int(item["source_monotonic_ns"]) for item in events if item.get("event") == name]
    return values[-1] if values else None


def exact_endpoint_patch(
    record: dict[str, Any],
    case: dict[str, Any],
    input_report: dict[str, Any] | None,
    output_report: dict[str, Any] | None,
    source_events: list[dict[str, Any]],
) -> None:
    raw = record["raw_observation"]
    raw.pop("renderer_proxy_ns", None)
    raw["input_audio_report"] = input_report
    raw["output_audio_report"] = output_report
    raw["source_events"] = source_events
    raw["audible_endpoint_provenance"] = "audio_loopback" if output_report else None
    record["component_path"] = [
        "audio_loopback" if item == "renderer_proxy" else item
        for item in record["component_path"]
    ]
    if output_report is None:
        return
    onset = int(output_report["detectedOnsetMonotonicNs"])
    input_end = int(input_report["detectedLastMonotonicNs"]) if input_report else None
    ingress = event_time(source_events, "agent_request_available_at_agent_ingress")
    result_source = event_time(source_events, "agent_result_available_at_source")
    status_source = event_time(source_events, "agent_status_available_at_source")
    if case["id"] == "N2-01":
        raw["qa01_ns"] = (
            (ingress - input_end) + (onset - result_source)
            if None not in (ingress, input_end, result_source) else None
        )
    if case["id"] == "N4-01":
        raw["qa03_ns"] = onset - status_source if status_source is not None else None
    if case["id"] == "N5-01":
        raw["qa05_ns"] = onset - input_end if input_end is not None else None


def atomic_correctness(record: dict[str, Any], semantic: dict[str, Any] | None) -> dict[str, bool]:
    raw = record["raw_observation"]
    fields: dict[str, bool] = {}
    if semantic is not None:
        fields.update({f"semantic.{name}": bool(value) for name, value in semantic["field_pass"].items()})
    fields.update({
        "runtime.operation": bool(raw["runtime_pass"]),
        "binding.identity": bool(raw["binding_pass"]),
        "state.convergence": bool(raw["state_pass"]),
        "continuity": bool(raw["continuity_pass"]),
    })
    return fields


def execute_normal_trial(
    *,
    contract: dict[str, Any],
    contract_hash: str,
    fixture: dict[str, Any],
    semantic_fixture: dict[str, Any],
    endpoint: str,
    candidate: str,
    case: dict[str, Any],
    trial: int,
    stratum: str,
    warmup: bool,
    host: legacy.JsonLineProcess,
    agent: LegacyAgentAdapter,
    output_dir: Path,
    semantic_pid: int,
) -> dict[str, Any]:
    case_dir = output_dir / "artifacts" / stratum / ("warmup" if warmup else "scored") / candidate / case["id"] / f"trial-{trial:02d}"
    case_dir.mkdir(parents=True, exist_ok=True)
    input_report = None
    semantic = None
    if case["input"] is not None:
        input_wav = case_dir / "request.wav"
        generate_wav(case["input"], input_wav)
        input_report = probe_audio(input_wav, case_dir / "input-loopback")
        semantic_case = semantic_fixture["cases"].get(case["id"])
        if semantic_case is None:
            raise RuntimeError(f"missing semantic Context for {case['id']}")
        semantic = legacy.semantic_call(endpoint, fixture, case, semantic_case)
    agent.drain_events()
    renderer_count = len(_RENDERER.reports)
    record, exposures = legacy.execute_case(
        contract=contract,
        contract_hash=contract_hash,
        candidate=candidate,
        case=case,
        trial=trial,
        semantic=semantic,
        host=host,
        agent=agent,
        execution_namespace=f"{stratum}-{'warmup' if warmup else 'scored'}",
    )
    source_events = agent.drain_events()
    output_report = _RENDERER.take_last() if len(_RENDERER.reports) > renderer_count else None
    exact_endpoint_patch(record, case, input_report, output_report, source_events)
    record.update(
        schema_version="via.dp11.integrated-trial.v5",
        stratum=stratum,
        warmup=warmup,
        source_execution_key=f"{stratum}:{candidate}:{case['id']}:{trial}",
        atomic_correctness=atomic_correctness(record, semantic),
        semantic_call=semantic,
        exposure_observations=exposures,
    )
    record["trace_relations"]["input_response_outcome"]["source_execution_key"] = record[
        "source_execution_key"
    ]
    record["trace_relations"]["metric_endpoint_and_clock"].update(
        endpoint_provenance="audio_loopback" if output_report else "software_event_only",
        input_audio_observed=input_report is not None,
        output_audio_observed=output_report is not None,
    )
    candidate_pids = legacy.process_tree_pids(host.process.pid)
    candidate_rss = legacy.rss_bytes(candidate_pids)
    model_rss = legacy.rss_bytes([semantic_pid])
    record["memory_observation"] = {
        "candidate_pids": candidate_pids,
        "candidate_process_rss_bytes": candidate_rss,
        "shared_semantic_model_pid": semantic_pid,
        "shared_semantic_model_rss_bytes": model_rss,
        "total_rss_bytes": candidate_rss + model_rss,
    }
    record["strict_integrated_pass"] = all(record["atomic_correctness"].values())
    record["trace_complete"] = legacy.trace_complete(record)
    return record


_RENDERER: PhysicalRenderer


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--semantic-endpoint", default="http://127.0.0.1:18080")
    parser.add_argument("--semantic-pid", type=int, required=True)
    parser.add_argument("--qualification", action="store_true")
    args = parser.parse_args()

    ensure_model(args.semantic_endpoint)
    compile_probe()
    for binary in (HOST_BIN, WORKER_BIN, REFERENCE_AGENT_BIN):
        if not binary.exists():
            raise RuntimeError(f"missing candidate binary: {binary}")

    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    contract_hash = sha256_file(CONTRACT)
    fixture = json.loads(NORMAL_FIXTURE.read_text(encoding="utf-8"))
    semantic_fixture = json.loads(SEMANTIC_CONTEXT.read_text(encoding="utf-8"))
    cases = fixture["cases"]
    cases_by_id = {case["id"]: case for case in cases}
    candidates = contract["candidates"]
    output_dir = args.output_dir.resolve()
    if output_dir.exists() and any(output_dir.iterdir()):
        raise RuntimeError(f"refusing to overwrite non-empty result directory: {output_dir}")
    (output_dir / "raw").mkdir(parents=True, exist_ok=True)

    global _RENDERER
    _RENDERER = PhysicalRenderer(output_dir / "audio")
    original_renderer = legacy.renderer_proxy
    legacy.renderer_proxy = _RENDERER

    manifest = {
        "campaign_id": output_dir.name,
        "status": "RUNNING",
        "started_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "contract_sha256": contract_hash,
        "fixture_sha256": sha256_file(NORMAL_FIXTURE),
        "semantic_context_sha256": sha256_file(SEMANTIC_CONTEXT),
        "change_ledger_sha256": sha256_file(CHANGE_LEDGER),
        "binary_sha256": {binary.name: sha256_file(binary) for binary in (HOST_BIN, WORKER_BIN, REFERENCE_AGENT_BIN)},
        "semantic_pid": args.semantic_pid,
        "semantic_endpoint": args.semantic_endpoint,
        "qualification_only": args.qualification,
    }
    write_json(output_dir / "manifest.json", manifest)
    raw_path = output_dir / "raw/trials.jsonl"
    warmup_path = output_dir / "raw/warmups.jsonl"
    raw_path.touch()
    warmup_path.touch()

    agent = LegacyAgentAdapter()
    hosts = {
        candidate: legacy.start_host(candidate, HOST_BIN, WORKER_BIN, agent.address)
        for candidate in candidates
    }
    try:
        breadth = (
            [cases_by_id[value] for value in contract["strata"]["latency_repetition"]["case_qa_mapping"]]
            if args.qualification else relevant_cases(cases)
        )
        total = len(breadth) * len(candidates)
        index = 0
        for case in breadth:
            for candidate in candidates:
                index += 1
                print(f"[breadth {index}/{total}] {candidate} {case['id']}", flush=True)
                record = execute_normal_trial(
                    contract=contract, contract_hash=contract_hash, fixture=fixture,
                    semantic_fixture=semantic_fixture, endpoint=args.semantic_endpoint,
                    candidate=candidate, case=case, trial=1,
                    stratum="correctness_breadth", warmup=False,
                    host=hosts[candidate], agent=agent, output_dir=output_dir,
                    semantic_pid=args.semantic_pid,
                )
                append_jsonl(raw_path, record)

        latency = contract["strata"]["latency_repetition"]
        latency_cases = [cases_by_id[value] for value in latency["case_qa_mapping"]]
        warmups = 0 if args.qualification else latency["warmup_trials_per_case_per_candidate"]
        scored = 1 if args.qualification else latency["scored_trials_per_case_per_candidate"]
        for warmup in (True, False):
            count = warmups if warmup else scored
            destination = warmup_path if warmup else raw_path
            total = count * len(latency_cases) * len(candidates)
            index = 0
            for trial in range(1, count + 1):
                for case_index, case in enumerate(latency_cases):
                    order = list(candidates)
                    if (trial + case_index) % 2:
                        order.reverse()
                    for candidate in order:
                        index += 1
                        print(f"[{'warmup' if warmup else 'latency'} {index}/{total}] {candidate} {case['id']} {trial}", flush=True)
                        record = execute_normal_trial(
                            contract=contract, contract_hash=contract_hash, fixture=fixture,
                            semantic_fixture=semantic_fixture, endpoint=args.semantic_endpoint,
                            candidate=candidate, case=case, trial=trial,
                            stratum="latency_repetition", warmup=warmup,
                            host=hosts[candidate], agent=agent, output_dir=output_dir,
                            semantic_pid=args.semantic_pid,
                        )
                        append_jsonl(destination, record)
    finally:
        legacy.renderer_proxy = original_renderer
        for host in hosts.values():
            host.stop()
        agent.stop()

    fault_contract = dict(contract)
    faults = contract["strata"]["faults"]
    if args.qualification:
        faults = {key: 1 for key in faults}
    fault_contract["repetitions"] = faults
    fault_agent = legacy.ReferenceAgent(
        REFERENCE_AGENT_BIN, output_dir / "raw/fault-common-agent-state.json"
    )
    try:
        fault_records = legacy.run_faults(
            contract=fault_contract,
            contract_hash=contract_hash,
            host_binary=HOST_BIN,
            worker=WORKER_BIN,
            agent=fault_agent,
            reference_agent_binary=REFERENCE_AGENT_BIN,
            state_root=output_dir / "raw",
        )
    finally:
        fault_agent.stop()
    for record in fault_records:
        record["schema_version"] = "via.dp11.fault-trial.v5"
        record["stratum"] = "fault"
        append_jsonl(raw_path, record)

    manifest["status"] = "RAW_COMPLETE_PENDING_ANALYSIS"
    manifest["completed_at"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    manifest["raw_sha256"] = sha256_file(raw_path)
    manifest["warmup_sha256"] = sha256_file(warmup_path)
    write_json(output_dir / "manifest.json", manifest)
    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
