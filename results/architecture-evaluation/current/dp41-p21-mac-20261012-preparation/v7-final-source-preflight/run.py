"""Actual model preflight, excluded from the final P21 population."""
import datetime
import hashlib
import json
from pathlib import Path
import subprocess

CONTROL = Path(__file__).resolve().parent
PREPARATION = CONTROL.parent
CAMPAIGN = PREPARATION.parent / "dp41-p21-mac-text-v7-20261012"
manifest = json.loads((CAMPAIGN / "manifest.json").read_text())
for relative, expected in manifest["frozen_source_hashes"].items():
    assert hashlib.sha256((CAMPAIGN / "frozen-source" / relative).read_bytes()).hexdigest() == expected, relative
rows = []
for variant in ("A", "B"):
    output = PREPARATION / f"v7-final-source-preflight-{variant}"
    if output.exists():
        raise FileExistsError("Preflight evidence cannot be overwritten")
    command = [manifest["python_executable"], "-B", "-u", "-m", "benchmark.architecture.dp41_45_poc.worker",
               "--variant", variant, "--repeat", "0", "--cases", "S6-01", "--output", str(output),
               "--max-cost", "2", "--budget-file", str(PREPARATION / "global-budgets/qwen.json")]
    with (CONTROL / f"worker-{variant}.log").open("x") as log:
        process = subprocess.Popen(command, cwd=CAMPAIGN / "frozen-source", stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            log.write(line)
            log.flush()
            print(line.rstrip(), flush=True)
        code = process.wait()
    rows.append({"variant": variant, "exit_code": code, "raw_directory": str(output),
                 "included_in_final_population": False})
    if code:
        raise RuntimeError(f"Actual preflight worker {variant} failed: {code}")
record = {"status": "EXECUTION_COMPLETE", "source_digest": manifest["frozen_source_digest"],
          "utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "workers": rows,
          "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
          "candidate_failures_preserved": True, "included_in_final_population": False}
(CONTROL / "execution-summary.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record), flush=True)
