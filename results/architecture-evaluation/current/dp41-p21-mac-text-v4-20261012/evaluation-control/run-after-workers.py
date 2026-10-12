"""Run the frozen judge only after all timing workers have exited."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import time

CONTROL = Path(__file__).resolve().parent
CAMPAIGN = CONTROL.parent
TOOLS = CAMPAIGN / "evaluator-tools"
BUDGET = CAMPAIGN.parent / "dp41-p21-mac-20261012-preparation/global-budgets/openai.json"


def stage(kind, **details):
    row = {"utc": datetime.now(timezone.utc).isoformat(), "kind": kind, **details}
    line = json.dumps(row, ensure_ascii=False)
    with (CONTROL / "stages.jsonl").open("a") as handle:
        handle.write(line + "\n")
    print(line, flush=True)


def completed():
    progress = CAMPAIGN / "campaign-progress.jsonl"
    if not progress.exists():
        return False
    for line in progress.read_text().splitlines():
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        if row.get("kind") == "ALL_WORKERS_COMPLETE":
            return True
    return False


stage("WAITING_FOR_ALL_WORKERS_COMPLETE", script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
while not completed():
    if (CAMPAIGN / "invalidated.json").exists():
        stage("STOPPED_INVALIDATED_CAMPAIGN")
        raise SystemExit(1)
    time.sleep(15)
manifest = json.loads((CAMPAIGN / "manifest.json").read_text())
tools = json.loads((CAMPAIGN / "evaluation-tool-manifest.json").read_text())
for relative, expected in tools["file_hashes"].items():
    if hashlib.sha256((TOOLS / relative).read_bytes()).hexdigest() != expected:
        raise ValueError("Evaluator source changed: " + relative)
for relative, expected in manifest["frozen_source_hashes"].items():
    if hashlib.sha256((CAMPAIGN / "frozen-source" / relative).read_bytes()).hexdigest() != expected:
        raise ValueError("Measured source changed: " + relative)
workers = [CAMPAIGN / "workers" / f"{variant}{repeat}" for variant in ("A", "B") for repeat in range(1, 6)]
command = [manifest["python_executable"], "-u", "-m", "benchmark.architecture.dp41_45_poc.evaluate",
           "--output", str(CAMPAIGN / "evaluation"), "--workers", *map(str, workers),
           "--budget-file", str(BUDGET), "--max-cost", "20"]
stage("START_JUDGING", source_digest=tools["source_digest"], trials=210)
for attempt in range(1, 4):
    stage("EVALUATION_ATTEMPT", attempt=attempt)
    with (CONTROL / "driver.log").open("a") as log:
        process = subprocess.Popen(command, cwd=TOOLS, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            log.write(line)
            log.flush()
            print(line.rstrip(), flush=True)
        code = process.wait()
    stage("EVALUATION_ATTEMPT_END", attempt=attempt, exit_code=code)
    if code == 0:
        stage("EVALUATION_COMPLETE")
        break
else:
    stage("EVALUATION_NEEDS_AUDIT_AFTER_RECORDED_RETRIES")
    raise SystemExit(1)
