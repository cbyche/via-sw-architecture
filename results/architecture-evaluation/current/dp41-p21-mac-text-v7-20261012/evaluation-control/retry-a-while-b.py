"""Preserved evaluator retry after timed workers, with a separate writer scope."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

C = Path(__file__).resolve().parents[1]
OUTPUT = C / "evaluation/a-preserved-retry"


def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def key(row):
    return row["variant"], row["repeat"], row["case_id"]


def append_atomic(path, record):
    data = (json.dumps(record, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()
    descriptor = os.open(path, os.O_APPEND | os.O_WRONLY | os.O_CREAT, 0o644)
    try:
        if os.write(descriptor, data) != len(data):
            raise OSError("Incomplete append")
    finally:
        os.close(descriptor)


def run():
    if OUTPUT.exists():
        raise FileExistsError("A retry writer scope must not be overwritten")
    manifest = json.loads((C / "manifest.json").read_text())
    if not any(row.get("kind") == "ALL_WORKERS_COMPLETE" for row in rows(C / "campaign-progress.jsonl")):
        raise ValueError("No evaluator concurrency during timing")
    tools = json.loads((C / "evaluation-tool-manifest.json").read_text())
    for relative, expected in tools["file_hashes"].items():
        if hashlib.sha256((C / "evaluator-tools" / relative).read_bytes()).hexdigest() != expected:
            raise ValueError("Frozen evaluator changed")
    seed = [row for row in rows(C / "evaluation/adjudications.jsonl") if row["variant"] == "A"]
    seed_keys = {key(row) for row in seed}
    missing = sorted({("A", repeat, case) for repeat in range(1, 6) for case in manifest["sequence"]} - seed_keys)
    if not missing:
        raise ValueError("No A records require retry")
    OUTPUT.mkdir()
    seed_path = OUTPUT / "adjudications.jsonl"
    seed_path.write_text("".join(json.dumps(row, ensure_ascii=False) + "\n" for row in seed))
    (OUTPUT / "seed-provenance.json").write_text(json.dumps({
        "copied_original_adjudications": len(seed), "copied_records_are_not_new_judge_calls": True,
        "seed_sha256": hashlib.sha256(seed_path.read_bytes()).hexdigest(), "missing_trial_keys": missing,
        "source_digest": tools["source_digest"], "timing_workers_complete": True,
        "evaluation_concurrency_only": "Separate 4-call writer; formal candidate concurrency unchanged"
    }, ensure_ascii=False, indent=2) + "\n")
    budget = C.parent / "dp41-p21-mac-20261012-preparation/global-budgets/openai.json"
    command = [manifest["python_executable"], "-u", "-B", "-m", "benchmark.architecture.dp41_45_poc.evaluate",
               "--output", str(OUTPUT), "--workers", *[str(C / "workers" / f"A{repeat}") for repeat in range(1, 6)],
               "--budget-file", str(budget), "--max-cost", "20"]
    print(json.dumps({"stage": "SEPARATE_A_RETRY", "missing": len(missing)}), flush=True)
    with (OUTPUT / "driver.log").open("w") as log:
        process = subprocess.Popen(command, cwd=C / "evaluator-tools", stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            log.write(line); log.flush(); print(line.rstrip(), flush=True)
        code = process.wait()
    stages = rows(C / "evaluation-control/stages.jsonl")
    first_attempt_active = not any(row.get("kind") == "EVALUATION_ATTEMPT" and row.get("attempt", 0) > 1 for row in stages)
    original_keys = {key(row) for row in rows(C / "evaluation/adjudications.jsonl")}
    merged = []
    for path in sorted(OUTPUT.glob("*.jsonl")):
        for row in rows(path):
            if path.name == "adjudications.jsonl":
                if key(row) in seed_keys or key(row) in original_keys or not first_attempt_active:
                    continue
                merged.append(key(row)); original_keys.add(key(row))
            copied = {**row, "evaluation_retry_origin": str(path.relative_to(C)),
                      "source_row_sha256": hashlib.sha256(json.dumps(row, ensure_ascii=False, sort_keys=True,
                                                                  separators=(",", ":")).encode()).hexdigest()}
            for field in ("context", "output"):
                if isinstance(copied.get(field), dict) and "path" in copied[field]:
                    copied[field] = {**copied[field], "path": "a-preserved-retry/" + copied[field]["path"]}
            append_atomic(C / "evaluation" / path.name, copied)
    report = {"stage": "SEPARATE_A_RETRY_FINISHED", "exit_code": code, "new_adjudications_merged": merged,
              "main_first_attempt_active": first_attempt_active, "seed_records_not_merged": len(seed),
              "frozen_candidate_changes": 0, "source_script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    (OUTPUT / "merge-provenance.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False), flush=True)


if __name__ == "__main__":
    run()
