"""Resume only missing grades with exact emitted-reference schema feedback."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import time

CONTROL = Path(__file__).resolve().parent
CAMPAIGN = CONTROL.parent
TOOLS = CAMPAIGN / "evaluator-tools-v4"
BUDGET = CAMPAIGN.parent / "dp41-p21-mac-20261012-preparation/global-budgets/openai.json"

def stage(kind, **details):
    row = {"utc": datetime.now(timezone.utc).isoformat(), "kind": kind, **details}
    with (CONTROL / "stages.jsonl").open("a") as handle:
        handle.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(json.dumps(row, ensure_ascii=False), flush=True)

def alive(pid):
    import os
    try:
        os.kill(pid, 0)
        # A completed child can remain a zombie while its old controller is stopped.
        status = subprocess.run(["/bin/ps", "-p", str(pid), "-o", "state="], capture_output=True, text=True)
        if status.returncode == 0:
            return bool(status.stdout.strip()) and not status.stdout.strip().startswith("Z")
        return False if status.returncode == 1 else True
    except ProcessLookupError:
        return False

stage("V4_EXACT_EMISSION_EVALUATOR_START", script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), original_adjudications_preserved=True, interrupted_provider_calls=0)
assert not alive(6694) and not alive(3684)
assert json.loads((CONTROL / "exact-emission-feedback-repair-tests.json").read_text())["status"] == "PASS"
manifest = json.loads((CAMPAIGN / "manifest.json").read_text())
tools = json.loads((CAMPAIGN / "evaluation-tool-manifest-v4.json").read_text())
assert json.loads((CONTROL / "validator-repair-tests.json").read_text())["status"] == "PASS"
for relative, expected in tools["file_hashes"].items():
    assert hashlib.sha256((TOOLS / relative).read_bytes()).hexdigest() == expected, relative
for relative, expected in manifest["frozen_source_hashes"].items():
    assert hashlib.sha256((CAMPAIGN / "frozen-source" / relative).read_bytes()).hexdigest() == expected, relative
workers = [CAMPAIGN / "workers" / f"{variant}{repeat}" for variant in ("A", "B") for repeat in range(1, 6)]
command = [manifest["python_executable"], "-u", "-m", "benchmark.architecture.dp41_45_poc.evaluate",
           "--output", str(CAMPAIGN / "evaluation"), "--workers", *map(str, workers),
           "--budget-file", str(BUDGET), "--max-cost", "20"]
stage("RESUME_MISSING_JUDGEMENTS_WITH_VALIDATOR_REPAIR", source_digest=tools["source_digest"], measured_source_digest=tools["baseline_source_digest"])
for attempt in range(1, 4):
    stage("REPAIRED_EVALUATION_ATTEMPT", attempt=attempt)
    with (CONTROL / "driver-v4.log").open("a") as log:
        process = subprocess.Popen(command, cwd=TOOLS, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        stage("REPAIRED_EVALUATOR_CHILD", pid=process.pid, attempt=attempt)
        for line in process.stdout:
            log.write(line)
            log.flush()
            print(line.rstrip(), flush=True)
        code = process.wait()
    stage("REPAIRED_EVALUATION_ATTEMPT_END", attempt=attempt, exit_code=code)
    if code == 0:
        stage("EVALUATION_COMPLETE")
        break
else:
    stage("EVALUATION_NEEDS_AUDIT_AFTER_RECORDED_RETRIES")
    raise SystemExit(1)
