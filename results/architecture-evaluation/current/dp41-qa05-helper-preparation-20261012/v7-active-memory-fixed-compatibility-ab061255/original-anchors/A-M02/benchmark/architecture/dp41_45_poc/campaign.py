"""Immutable campaign source, paired isolated workers and a durable progress log."""
from __future__ import annotations

import argparse
import asyncio
import datetime
import hashlib
import json
import platform
from pathlib import Path
import shutil
import subprocess
import sys
import time

from .fixtures import load_manifest, validate_fixtures
from prototypes.candidates.dp41_45_poc.shared.providers import PRICES

ROOT = Path(__file__).resolve().parents[3]
SOURCE_TREES = ("benchmark/architecture/dp41_45_poc", "prototypes/candidates/dp41_45_poc")
IGNORE = shutil.ignore_patterns("__pycache__", "*.pyc", ".pytest_cache")


def snapshot(directory):
    return {str(p.relative_to(directory)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(directory.rglob("*")) if p.is_file() and "__pycache__" not in p.parts and p.suffix != ".pyc"}


def freeze(output):
    output = output.resolve()
    if output.exists():
        raise FileExistsError("A campaign is never overwritten")
    problems = validate_fixtures()
    if problems:
        raise ValueError(problems)
    source = output / "frozen-source"
    source.mkdir(parents=True)
    fixture_manifest = load_manifest()
    for tree in SOURCE_TREES:
        shutil.copytree(ROOT / tree, source / tree, ignore=IGNORE)
    for relative in fixture_manifest["canonical_sources"]:
        target = source / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / relative, target)
    scoring_input = "docs/architecture/12-decisions/decision-packages/03-02-poc-inputs.json"
    scoring_target = source / scoring_input
    scoring_target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ROOT / scoring_input, scoring_target)
    hashes = snapshot(source)
    manifest = {
        "campaign_contract": "DP41-P21-MAC-TEXT-v7", "created_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "qa_contract_version": "ASR-QA-v6", "variants": ["A", "B"], "repeat_ids": [1,2,3,4,5],
        "sequence": fixture_manifest["sequence"], "case_ids": fixture_manifest["sequence"],
        "independent_repeats": 5, "pair_order": [["A","B"],["B","A"],["A","B"],["B","A"],["A","B"]],
        "configuration": {"A":"41A+44A+45A+42A", "B":"41B+44A+45A+42A"},
        "strict_coverage": True, "speech_mode":"SIMULATED_PROFILE", "input_mode":"TRANSCRIPT",
        "evidence_level":"MEASURED_MODEL_WITH_SIMULATED_SPEECH", "qa04_primary_metric":"ACTUAL_VIA_PROVIDER_USAGE_X_PRICE",
        "memory_scoring_metric":"phys_footprint", "memory_scope":"ONE_ISOLATED_WORKER_VIA_COMMON_STUB_AND_RAW_RECORDER",
        "memory_sample_interval_seconds":.05, "host": {"system":platform.system(),"release":platform.release(),"macos":platform.mac_ver()[0],"machine":platform.machine(),"python":platform.python_version()},
        "python_executable":sys.executable,"prices":PRICES,
        "semantic_model":{"id":"qwen3.6-35b-a3b","temperature":0,"enable_thinking":False,"max_request_calls":8,"max_concurrency":4},
        "judge_model":{"id":"gpt-6-luna","reasoning":"high","protocol":"16_CALIBRATIONS_THEN_TWO_BLIND_PASSES_THIRD_ON_DISAGREEMENT_AND_TRACE_REVIEW"},
        "budget_usd":{"qwen":100,"openai":100},"canonical_sources":fixture_manifest["canonical_sources"],
        "frozen_source_hashes":hashes,
        "frozen_source_digest":hashlib.sha256(json.dumps(hashes,sort_keys=True,separators=(",",":")).encode()).hexdigest(),
        "qa_input_path":str(source/"docs/architecture/12-decisions/decision-packages/03-02-poc-inputs.json"),
        "workload_path":str(source/"docs/architecture/12-decisions/decision-packages/03-02-common-workload.json"),
        "timeouts":{"automatic_episode_s":60,"whole_watchdog_s":600,"qa02_failure_cap_s":5,"qa03_failure_cap_s":30},
        "execution":"SEQUENTIAL_ISOLATED_WORKERS_ALTERNATING_PAIR_ORDER_NO_JUDGE_CONTENDING_WITH_TIMING",
        "response_endpoint":"EXACT_JUDGE_CORE_CLAUSE_START_AT_OBSERVED_SIMULATED_CHARACTER_SINK_NODE",
        "service_failure_policy": "PRESERVE_INVALID_A_B_ACTIVITY_PAIR_THEN_REPLAY_SAME_FROZEN_PAIR_MAX_THREE_ATTEMPTS",
    }
    manifest["canonical_sources"][scoring_input] = hashlib.sha256(scoring_target.read_bytes()).hexdigest()
    (output/"manifest.json").write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+"\n")
    return manifest


def event(output, record):
    record = {"utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),**record}
    with (output/"campaign-progress.jsonl").open("a") as handle:
        handle.write(json.dumps(record,ensure_ascii=False)+"\n")
        handle.flush()
    print(json.dumps(record,ensure_ascii=False),flush=True)


def service_failure_calls(worker):
    """Classify missing usage from common service failure, never bad semantics.

    A provider's successful response with invalid model JSON/schema remains a
    candidate observation. The benign-fixture input-filter rejection was also
    observed to pass when its exact API context was replayed in preparation.
    """
    failures = []
    path = worker / "model-calls.jsonl"
    if not path.exists():
        return failures
    for line in path.read_text().splitlines():
        call = json.loads(line)
        if call.get("usage") is not None and call.get("cost_usd") is not None:
            continue
        status, http = call.get("status"), call.get("http_status")
        retryable = status == "TRANSPORT_ERROR" or status == "HTTP_ERROR" and http in (429, 500, 502, 503, 504)
        if status == "HTTP_ERROR" and http == 400 and call.get("output"):
            response = json.loads((worker / call["output"]["path"]).read_text())
            try:
                body = json.loads(response.get("body", "{}"))
            except (TypeError, json.JSONDecodeError):
                body = {}
            retryable = (body.get("error") or {}).get("code") == "data_inspection_failed"
        if not retryable:
            raise ValueError("Unclassified missing usage needs an evidence audit: " + call["call_id"])
        failures.append({"call_id": call["call_id"], "status": status, "http_status": http,
                         "case_id": call.get("case_id"), "request_id": call.get("request_id"),
                         "context": call.get("context"), "output": call.get("output")})
    return failures


def verify_input_filter_transience(worker, failure, output, budget_file):
    """Require an identical accepted API context before masking a filter event.

    The health response is never adopted as candidate meaning. Persistent input
    rejection remains for an audit instead of becoming an automatic INVALID.
    """
    if failure.get("http_status") != 400:
        return None
    from prototypes.candidates.dp41_45_poc.shared.evidence import Evidence
    from prototypes.candidates.dp41_45_poc.shared.providers import ModelClient
    directory = output / "service-health" / failure["call_id"]
    summary = directory / "summary.json"
    original_path = worker / failure["context"]["path"]
    original = json.loads(original_path.read_text())
    original_hash = hashlib.sha256(original_path.read_bytes()).hexdigest()
    if summary.exists():
        record = json.loads(summary.read_text())
        if record["original_context_file_sha256"] != original_hash:
            raise ValueError("Service-health context changed")
    else:
        evidence = Evidence(directory / "raw", worker.name[0], int(worker.name[1:]))
        async def replay():
            client = ModelClient(evidence, "qwen", max_cost_usd=1, budget_file=budget_file)
            for attempt in range(2):
                try:
                    await client.call(original["messages"], "CINTERPRET", request_id="SERVICE_HEALTH:" + failure["call_id"],
                                      case_id=failure.get("case_id", ""), max_tokens=original["max_tokens"],
                                      purpose="SERVICE_HEALTH")
                except Exception:
                    # Both the terminal response and known/unknown usage are
                    # recorded by ModelClient; no response is substituted.
                    pass
                calls = [json.loads(line) for line in (directory / "raw/model-calls.jsonl").read_text().splitlines()]
                for call in calls:
                    context = json.loads((directory / "raw" / call["context"]["path"]).read_text())
                    if (context == original and call.get("http_status") == 200
                            and call.get("usage") is not None and call.get("cost_usd") is not None):
                        return call["call_id"]
            return None
        accepted = asyncio.run(replay())
        record = {"status": "CONFIRMED_TRANSIENT_IDENTICAL_CONTEXT" if accepted else "PERSISTENT_REJECTION_NEEDS_AUDIT",
                  "original_call_id": failure["call_id"], "original_context_file_sha256": original_hash,
                  "accepted_identical_context_call_id": accepted, "candidate_meaning_adopted": False,
                  "included_in_final_population": False, "evidence_level": "SERVICE_HEALTH_REPLAY"}
        summary.write_text(json.dumps(record, indent=2) + "\n")
    if record["status"] != "CONFIRMED_TRANSIENT_IDENTICAL_CONTEXT":
        raise ValueError("Persistent provider input rejection requires an audit: " + failure["call_id"])
    return str(summary)


def run_workers(output, budget_file):
    manifest = json.loads((output/"manifest.json").read_text())
    source = output/"frozen-source"
    def run_one(repeat, variant):
        worker = output/"workers"/f"{variant}{repeat}"
        summary = worker/"worker-summary.jsonl"
        if summary.exists() and any(json.loads(line).get("status") == "COMPLETE" for line in summary.read_text().splitlines()):
            event(output,{"kind":"ALREADY_COMPLETE","variant":variant,"repeat":repeat})
            return
        if worker.exists():
            raise FileExistsError(f"Partial worker evidence requires explicit audit: {worker}")
        if snapshot(source) != manifest["frozen_source_hashes"]:
            raise ValueError("Frozen source changed before a worker")
        event(output,{"kind":"WORKER_START","variant":variant,"repeat":repeat})
        command=[manifest["python_executable"],"-m","benchmark.architecture.dp41_45_poc.worker","--variant",variant,"--repeat",str(repeat),"--output",str(worker),"--max-cost","15","--budget-file",str(budget_file.resolve())]
        started=time.monotonic()
        process=subprocess.Popen(command,cwd=source,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
        with (output/f"worker-{variant}{repeat}.log").open("a") as log:
            for line in process.stdout:
                log.write(line); log.flush(); print(line.rstrip(),flush=True)
        code=process.wait()
        event(output,{"kind":"WORKER_END","variant":variant,"repeat":repeat,"exit_code":code,"seconds":time.monotonic()-started})
        if code:
            raise RuntimeError(f"Worker {variant}{repeat} failed; retained for audit")
    for repeat, order in enumerate(manifest["pair_order"], 1):
        recorded = len(list((output / "invalid-worker-attempts").glob(f"pair-{repeat}-attempt-*/invalid-pair.json")))
        if recorded >= 3:
            raise RuntimeError("Three common-failure pairs already retained; audit before restarting")
        for attempt in range(recorded + 1, 4):
            for variant in order:
                run_one(repeat, variant)
            failures = {variant: service_failure_calls(output / "workers" / f"{variant}{repeat}")
                        for variant in order}
            if not any(failures.values()):
                event(output, {"kind": "PAIR_ACCEPTED", "repeat": repeat, "attempt": attempt})
                break
            for variant, records in failures.items():
                for failure in records:
                    failure["identical_context_transience_proof"] = verify_input_filter_transience(
                        output / "workers" / f"{variant}{repeat}", failure, output, budget_file)
            preserved = output / "invalid-worker-attempts" / f"pair-{repeat}-attempt-{attempt}"
            preserved.mkdir(parents=True)
            relocation = {}
            for variant in order:
                worker = output / "workers" / f"{variant}{repeat}"
                target = preserved / worker.name
                shutil.move(str(worker), target)
                relocation[str(worker)] = str(target)
                log = output / f"worker-{variant}{repeat}.log"
                if log.exists():
                    shutil.move(str(log), preserved / log.name)
            invalid = {"status": "INVALID_COMMON_PROVIDER_FAILURE_PAIR", "repeat": repeat,
                       "attempt": attempt, "failures": failures, "artifact_relocation": relocation,
                       "preserved_file_hashes": snapshot(preserved), "included_in_final_population": False,
                       "reason": "ASR-QA-v6 §3.5 preserves/replays the complete A/B activity pair for a common provider failure; candidate failures never trigger this path."}
            (preserved / "invalid-pair.json").write_text(json.dumps(invalid, ensure_ascii=False, indent=2) + "\n")
            event(output, {"kind": "PAIR_INVALIDATED_COMMON_PROVIDER_FAILURE", "repeat": repeat,
                           "attempt": attempt, "record": str(preserved / "invalid-pair.json")})
            if attempt == 3:
                raise RuntimeError("Common provider failure persisted across three preserved A/B pairs")
    event(output,{"kind":"ALL_WORKERS_COMPLETE","trials":210})


if __name__ == "__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("mode",choices=("freeze","run"))
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--budget-file",type=Path)
    args=parser.parse_args()
    if args.mode == "freeze":
        m=freeze(args.output); print(json.dumps({"frozen_source_digest":m["frozen_source_digest"],"files":len(m["frozen_source_hashes"])}))
    else:
        run_workers(args.output.resolve(),args.budget_file)
