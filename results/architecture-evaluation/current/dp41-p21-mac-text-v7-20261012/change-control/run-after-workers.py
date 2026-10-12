"""Preserved QA05 control procedure; no API calls and no timing contention."""
from __future__ import annotations

from datetime import datetime, timezone
from functools import partial
import hashlib
import json
from pathlib import Path
import sys
import os
import subprocess
import time
import traceback


CONTROL = Path(__file__).resolve().parent
CAMPAIGN = CONTROL.parent
TOOLS = CAMPAIGN / "evaluator-tools"
BASELINE = CAMPAIGN / "frozen-source"
EXPECTED_TOOL_DIGEST = "44f725057e7589c35aa59cb23537a9a8c59f48a40944b70f03e3906e931f08ce"
HELPER = CAMPAIGN / "qa05-additional-helper-v1"
EXPECTED_HELPER_DIGEST = 'c5c337286fa4b64edb971f6849cddcc1d22d9b0374715ca8a2989e6b1a4c8333'
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


    if summary["status"] != "AWAITING_RESPONSIBILITY_REVIEW":
        stage("ADDITIONAL_JOBS_NOT_STARTED_ORIGINAL_FAILURES", api_calls=0)
        return
    helper_manifest = json.loads((HELPER / "helper-manifest.json").read_text())
    if helper_manifest["source_digest"] != EXPECTED_HELPER_DIGEST:
        raise ValueError("unexpected additional helper generation")
    verify_files(HELPER, helper_manifest["file_hashes"])
    additional = []
    for name, script in [("CORRECTED_A_M04", HELPER / "original-a-lineage-v1/run_corrected_a.py"),
                         ("TYPED_B_M04_AND_M05_RESTART", HELPER / "run_additional_v2.py"),
                         ("M06_EXACT_PREFERENCE_BEFORE_AFTER", HELPER / "run_m06_preservation_v3.py")]:
        invalidated = CAMPAIGN / "invalidated.json"
        if invalidated.exists():
            stage("ADDITIONAL_JOBS_ABORTED_CAMPAIGN_INVALIDATED", invalidation_ref=str(invalidated), api_calls=0)
            return
        verify_files(HELPER, helper_manifest["file_hashes"])
        stage("ADDITIONAL_JOB_START", name=name, script=str(script), helper_digest=EXPECTED_HELPER_DIGEST, api_calls=0)
        stdout_ref, stderr_ref = CONTROL / (name.lower() + "-stdout.log"), CONTROL / (name.lower() + "-stderr.log")
        with stdout_ref.open("x", encoding="utf-8") as out, stderr_ref.open("x", encoding="utf-8") as err:
            process = subprocess.run([sys.executable, str(script), "--campaign", str(CAMPAIGN), "--helper", str(HELPER)],
                stdout=out, stderr=err, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}, timeout=3600)
        lines = stdout_ref.read_text().splitlines()
        try: result = json.loads(lines[-1])
        except (IndexError, json.JSONDecodeError): result = {"status": "INVALID_STDOUT", "raw_stdout_ref": str(stdout_ref)}
        if name == "CORRECTED_A_M04":
            passed = result.get("status") == "AWAITING_RESPONSIBILITY_REVIEW"
        elif name == "M06_EXACT_PREFERENCE_BEFORE_AFTER":
            passed = result.get("status") == "PASSED" and len(result.get("records", [])) == 2 and all(
                row.get("passed") is True and row.get("zero_change_eligible_by_preserved_oracle") is True
                for row in result.get("records", []))
        else:
            passed = result.get("alternative", {}).get("status") == "AWAITING_RESPONSIBILITY_REVIEW" and all(
                row.get("status") == "PASSED" for row in result.get("m05_supplements", [])) and len(result.get("m05_supplements", [])) == 2
        passed = passed and process.returncode == 0
        row = {"name": name, "passed": passed, "returncode": process.returncode, "result": result,
               "stdout_ref": str(stdout_ref), "stderr_ref": str(stderr_ref), "helper_digest": EXPECTED_HELPER_DIGEST, "api_calls": 0}
        additional.append(row)
        stage("ADDITIONAL_JOB_END", **row)
        verify_files(HELPER, helper_manifest["file_hashes"])
        verify_files(BASELINE, manifest["frozen_source_hashes"])
        verify_files(TOOLS, tool_manifest["file_hashes"])
    pipeline = {"status": "AWAITING_RESPONSIBILITY_REVIEW" if all(row["passed"] for row in additional) else "EXECUTION_FAILURES",
                "original_job_count": 12, "additional": additional, "count": None, "api_calls": 0,
                "selected_job_count": None, "selection": "ROOT_EXPLICIT_SELECTION_MANIFEST_REQUIRED",
                "M06_exact_oracle_supplement": str(CAMPAIGN / "change-control/m06-exact-preservation-summary.json"), "author_responsibility_review_applied": False}
    with (CONTROL / "additional-pipeline-summary.json").open("x", encoding="utf-8") as handle:
        handle.write(json.dumps(pipeline, ensure_ascii=False, indent=2) + "\n")
    stage("ADDITIONAL_PIPELINE_EXECUTION_END", status=pipeline["status"], api_calls=0,
          summary_ref=str(CONTROL / "additional-pipeline-summary.json"))


if __name__ == "__main__":
    try:
        run()
    except BaseException as error:
        stage("DRIVER_FAILED", error=f"{type(error).__name__}: {error}", traceback=traceback.format_exc(), api_calls=0)
        raise
