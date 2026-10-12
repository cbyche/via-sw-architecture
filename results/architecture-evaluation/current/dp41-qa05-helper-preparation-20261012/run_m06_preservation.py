"""API-free exact S1-03 preservation evidence for original M06 jobs.

Original job sources and records are read-only. The supplemental result must
pass before an honest zero-change M06 responsibility review can be accepted.
"""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys


def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def save(path, value):
    with Path(path).open("x", encoding="utf-8") as out:
        out.write(json.dumps(value, ensure_ascii=False, indent=2, default=str) + "\n")


def run(campaign, helper):
    campaign, helper = Path(campaign).resolve(), Path(helper).resolve()
    if (campaign / "invalidated.json").exists(): raise ValueError("campaign invalidated")
    progress = [json.loads(line) for line in (campaign / "campaign-progress.jsonl").read_text().splitlines()]
    if not any(row.get("kind") == "ALL_WORKERS_COMPLETE" for row in progress):
        raise ValueError("timing workers must finish before exact functional probes")
    original = json.loads((campaign / "change-control/execution-summary.json").read_text())
    if len(original["jobs"]) != 12: raise ValueError("all original independent jobs must be preserved first")
    manifest = json.loads((helper / "helper-manifest.json").read_text())
    for name, expected in manifest["file_hashes"].items():
        path = (helper / name).resolve()
        if not path.is_relative_to(helper) or sha(path) != expected:
            raise ValueError("exact preservation helper changed")
    sys.dont_write_bytecode = True
    tools = campaign / "evaluator-tools"
    sys.path.insert(0, str(tools))
    from benchmark.architecture.dp41_45_poc import change_experiments as changes
    assert Path(changes.__file__).resolve().is_relative_to(tools)
    baseline = campaign / "frozen-source"
    baseline_hash = changes.snapshot_digest(changes.source_snapshot(baseline))
    spec = importlib.util.spec_from_file_location("exact_memory_preservation", helper / "m06-exact-preference-v2/m06_exact_probe.py")
    probe = importlib.util.module_from_spec(spec); spec.loader.exec_module(probe)
    output = campaign / "changes-supplemental/M06-EXACT-S103-v2"
    output.mkdir(parents=True, exist_ok=False)
    records = []
    for variant in ["A", "B"]:
        if (campaign / "invalidated.json").exists(): raise ValueError("campaign invalidated during supplements")
        original_ref = campaign / f"changes/CH-DP41-{variant}-M06/result.json"
        original_job = json.loads(original_ref.read_text())
        source = original_ref.parent / "source"
        if changes.snapshot_digest(changes.source_snapshot(source)) != original_job["after_source_hash"]:
            raise ValueError("original M06 source changed before supplemental")
        before = probe.run_supplement(baseline, variant, output / (variant + "-before"))
        after = probe.run_supplement(source, variant, output / (variant + "-after"))
        passed = before["status"] == after["status"] == "PASSED" and before["source_unchanged"] and after["source_unchanged"]
        records.append({"variant": variant, "job_id": f"CH-DP41-{variant}-M06",
            "original_result_ref": str(original_ref), "before": before, "after": after,
            "passed": passed, "zero_change_eligible_by_preserved_oracle": passed and
                original_job.get("changed_paths") == [] and original_job.get("completion_passed") is True and
                original_job.get("regression_passed") is True and original_job.get("baseline_support_audit_passed") is True,
            "count": None, "responsibility_review": "TWO_INDEPENDENT_REVIEWERS_REQUIRED"})
    if changes.snapshot_digest(changes.source_snapshot(baseline)) != baseline_hash:
        raise ValueError("original baseline modified by functional probe")
    result = {"status": "PASSED" if all(row["passed"] for row in records) else "FAILED",
        "records": records, "api_calls": 0, "helper_source_digest": manifest["source_digest"],
        "baseline_source_digest": baseline_hash, "source_unchanged": True,
        "P21_accuracy_denominator_addition": 0, "actual_model_semantic_accuracy_claim": False,
        "author_responsibility_review_applied": False, "count": None}
    save(output / "summary.json", result)
    save(campaign / "change-control/m06-exact-preservation-summary.json", result)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--campaign", required=True)
    parser.add_argument("--helper", default=str(Path(__file__).resolve().parent))
    args = parser.parse_args()
    result = run(args.campaign, args.helper)
    print(json.dumps(result, ensure_ascii=False, default=str))
    sys.exit(0 if result["status"] == "PASSED" else 1)
