"""Immutable campaign source, paired isolated workers and a durable progress log."""
from __future__ import annotations

import argparse
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
    hashes = snapshot(source)
    manifest = {
        "campaign_contract": "DP41-P21-MAC-TEXT-v1", "created_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
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
    }
    (output/"manifest.json").write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+"\n")
    return manifest


def event(output, record):
    record = {"utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),**record}
    with (output/"campaign-progress.jsonl").open("a") as handle:
        handle.write(json.dumps(record,ensure_ascii=False)+"\n")
        handle.flush()
    print(json.dumps(record,ensure_ascii=False),flush=True)


def run_workers(output, budget_file):
    manifest = json.loads((output/"manifest.json").read_text())
    source = output/"frozen-source"
    for repeat, order in enumerate(manifest["pair_order"],1):
        for variant in order:
            worker = output/"workers"/f"{variant}{repeat}"
            summary = worker/"worker-summary.jsonl"
            if summary.exists() and any(json.loads(line).get("status") == "COMPLETE" for line in summary.read_text().splitlines()):
                event(output,{"kind":"ALREADY_COMPLETE","variant":variant,"repeat":repeat})
                continue
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
