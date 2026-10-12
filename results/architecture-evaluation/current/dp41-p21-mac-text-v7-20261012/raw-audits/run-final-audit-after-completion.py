"""Wait for complete final exports, then run the API-free independent audit."""
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
import sys
import time

campaign = Path(__file__).resolve().parent.parent
audit_directory = campaign / "raw-audits"
required = ["final/completion.json", "final/qa-results.json", "final/normalized-trials.jsonl",
            "final/conditions.csv", "final/reactions.csv", "final/cycles.csv", "final/changes.csv",
            "final/repeat-results.csv", "evaluation/evaluation-origin-manifest.json"]


def record(stage, **fields):
    row = {"utc": datetime.now(timezone.utc).isoformat(), "stage": stage,
           "campaign": str(campaign), "api_calls": 0, **fields}
    with (audit_directory / "final-audit-driver-log.jsonl").open("a") as handle:
        handle.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(json.dumps(row, ensure_ascii=False), flush=True)


record("WAITING_FINAL_COMPLETION_AND_ORIGIN")
while not all((campaign / relative).is_file() for relative in required):
    if (campaign / "invalidated.json").exists():
        record("INVALIDATED_NO_AUDIT_EXECUTED")
        sys.exit(2)
    time.sleep(15)
record("INDEPENDENT_RECALCULATION_STARTED")
log = audit_directory / ("final-audit-execution-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + ".log")
with log.open("x") as handle:
    result = subprocess.run([sys.executable, "-B", str(audit_directory / "recompute-final-independent.py"),
                             "--campaign", str(campaign)], stdout=handle, stderr=subprocess.STDOUT)
record("INDEPENDENT_RECALCULATION_TERMINAL", returncode=result.returncode, log=str(log))
sys.exit(result.returncode)
