"""Preserved QA05 control procedure; no API calls and no timing contention."""
from __future__ import annotations

from datetime import datetime, timezone
from functools import partial
import hashlib
import json
from pathlib import Path
import sys
import time
import traceback


CONTROL = Path(__file__).resolve().parent
CAMPAIGN = CONTROL.parent
TOOLS = CAMPAIGN / "evaluator-tools"
BASELINE = CAMPAIGN / "frozen-source"
EXPECTED_TOOL_DIGEST = "c2dfc26fa6afbce8df4af3405913031ca440fefa150d6a0739c010b3ed4796a7"
sys.dont_write_bytecode = True


def stage(kind, **details):
    row = {"utc": datetime.now(timezone.utc).isoformat(), "kind": kind, **details}
    message = json.dumps(row, ensure_ascii=False)
    with (CONTROL / "stages.jsonl").open("a", encoding="utf-8") as handle:
        handle.write(message + "\n")
    with (CONTROL / "driver.log").open("a", encoding="utf-8") as handle:
        handle.write(message + "\n")
    print(message, flush=True)


def verify_files(root, hashes):
    for relative, expected in hashes.items():
        path = (root / relative).resolve()
        if not path.is_relative_to(root.resolve()):
            raise ValueError("manifest path escapes source root")
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError("source hash mismatch: " + str(path))


def all_workers_event():
    progress = CAMPAIGN / "campaign-progress.jsonl"
    if not progress.exists():
        return None
    for line in progress.read_text().splitlines():
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue  # A concurrently appended incomplete last line is reread.
        if row.get("kind") == "ALL_WORKERS_COMPLETE":
            return row
    return None


def run():
    stage("WAITING_FOR_ALL_WORKERS_COMPLETE", campaign=str(CAMPAIGN), baseline=str(BASELINE),
          evaluator_tools=str(TOOLS), expected_tool_digest=EXPECTED_TOOL_DIGEST,
          driver_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), api_calls=0)
    while True:
        invalidated = CAMPAIGN / "invalidated.json"
        if invalidated.exists():
            stage("WAIT_ABORTED_CAMPAIGN_INVALIDATED", invalidation_ref=str(invalidated),
                  invalidation_record=invalidated.read_text(), change_jobs_started=False, api_calls=0)
            return
        completed = all_workers_event()
        if completed is not None:
            break
        time.sleep(15)
    stage("ALL_WORKERS_COMPLETE_OBSERVED", source_event=completed)
    tool_manifest = json.loads((CAMPAIGN / "evaluation-tool-manifest.json").read_text())
    manifest = json.loads((CAMPAIGN / "manifest.json").read_text())
    if tool_manifest["source_digest"] != EXPECTED_TOOL_DIGEST:
        raise ValueError("unexpected evaluator tool generation")
    verify_files(TOOLS, tool_manifest["file_hashes"])
    verify_files(BASELINE, manifest["frozen_source_hashes"])
    sys.path.insert(0, str(TOOLS))
    from benchmark.architecture.dp41_45_poc import change_experiments as changes, replay
    for module in (changes, replay):
        if not Path(module.__file__).resolve().is_relative_to(TOOLS):
            raise ValueError("import did not use the evaluator snapshot")
    original_run_job = changes.run_change_job
    original_replay = partial(replay.regression_runner, campaign=CAMPAIGN)

    def logged_replay(source, variant, task_id, *, audit=False):
        stage("BASELINE_P21_REPLAY_START" if audit else "P21_REPLAY_START",
              variant=variant, task_id=task_id, source=str(source), api_calls=0)
        checks = original_replay(source, variant, task_id, audit=audit)
        p21 = checks["P21"]
        stage("BASELINE_P21_REPLAY_END" if audit else "P21_REPLAY_END", variant=variant,
              task_id=task_id, passed=p21.get("passed"), case_count=len(p21.get("cases", [])),
              raw_evidence=p21.get("evidence"), api_calls=0)
        return checks

    def logged_run_job(*args, **kwargs):
        invalidated = CAMPAIGN / "invalidated.json"
        if invalidated.exists():
            stage("CHANGE_JOBS_ABORTED_CAMPAIGN_INVALIDATED", invalidation_ref=str(invalidated),
                  invalidation_record=invalidated.read_text(), api_calls=0)
            raise RuntimeError("Campaign invalidated; preserved changes are not final QA05 evidence")
        variant, task_id = args[2], args[3]
        stage("CHANGE_JOB_START", job_id=f"CH-DP41-{variant}-{task_id}", variant=variant,
              task_id=task_id, evaluator_tools_digest=EXPECTED_TOOL_DIGEST, api_calls=0)
        record = original_run_job(*args, **kwargs)
        stage("CHANGE_JOB_END", job_id=record["job_id"], variant=variant, task_id=task_id,
              status=record["status"], completion_passed=record["completion_passed"],
              regression_passed=record["regression_passed"], result_ref=record["result_ref"],
              baseline_source_hash=record["baseline_source_hash"],
              after_source_hash=record.get("after_source_hash"),
              changed_unit_set=record["changed_unit_set"], count=record.get("count"),
              error=record.get("error"), api_calls=0)
        return record

    changes.run_change_job = logged_run_job
    records = changes.run_dp41_change_jobs(BASELINE, CAMPAIGN / "changes", logged_replay)
    verify_files(TOOLS, tool_manifest["file_hashes"])
    verify_files(BASELINE, manifest["frozen_source_hashes"])
    summary = {"status": "AWAITING_RESPONSIBILITY_REVIEW" if len(records) == 12 and
               all(r["status"] == "AWAITING_RESPONSIBILITY_REVIEW" for r in records) else "EXECUTION_FAILURES",
               "jobs": records, "api_calls": 0, "baseline_source_digest": manifest["frozen_source_digest"],
               "evaluator_tools_digest": EXPECTED_TOOL_DIGEST,
               "independent_responsibility_reviewers_required": 2, "author_review_applied": False}
    with (CONTROL / "execution-summary.json").open("x", encoding="utf-8") as handle:
        handle.write(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    stage("CHANGE_CAMPAIGN_EXECUTION_END", status=summary["status"], job_count=len(records),
          awaiting_review=sum(r["status"] == "AWAITING_RESPONSIBILITY_REVIEW" for r in records),
          failed=sum(r["status"] == "FAILED" for r in records),
          summary_ref=str(CONTROL / "execution-summary.json"), api_calls=0)


if __name__ == "__main__":
    try:
        run()
    except BaseException as error:
        stage("DRIVER_FAILED", error=f"{type(error).__name__}: {error}", traceback=traceback.format_exc(), api_calls=0)
        raise
