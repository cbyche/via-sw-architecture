"""Post-timing A correction job, with original sources preserved read-only."""
from __future__ import annotations
from functools import partial
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys


def save(path, data):
    if path.exists(): raise FileExistsError("never overwrite correction evidence")
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2, default=str) + "\n")


def run_corrected_a(campaign, helper):
    campaign, helper = Path(campaign).resolve(), Path(helper).resolve()
    if (campaign / "invalidated.json").exists(): raise ValueError("invalidated campaign")
    progress = [json.loads(line) for line in (campaign / "campaign-progress.jsonl").read_text().splitlines()]
    if not any(e.get("kind") == "ALL_WORKERS_COMPLETE" for e in progress): raise ValueError("timing is not complete")
    jobs = json.loads((campaign / "change-control/execution-summary.json").read_text())["jobs"]
    if len(jobs) != 12: raise ValueError("original 12 jobs must be preserved first")
    manifest = json.loads((helper / "helper-manifest.json").read_text())
    for name, expected in manifest["file_hashes"].items():
        path = (helper / name).resolve()
        if not path.is_relative_to(helper) or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError("helper generation changed")
    tools, baseline = campaign / "evaluator-tools", campaign / "frozen-source"
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(tools))
    from benchmark.architecture.dp41_45_poc import change_experiments as changes, replay
    assert Path(changes.__file__).resolve().is_relative_to(tools)
    replay_runner = partial(replay.regression_runner, campaign=campaign)
    lineage_file = helper / "original-a-lineage-v1/producer_lineage_probe.py"
    spec = importlib.util.spec_from_file_location("corrected_a_lineage_patch", helper / "original-a-lineage-v1/lineage_patch.py")
    patch_module = importlib.util.module_from_spec(spec); spec.loader.exec_module(patch_module)

    def lineage_probe(source, variant, representation, mapping_scope, ref):
        if (campaign / "invalidated.json").exists(): raise ValueError("campaign invalidated")
        process = subprocess.run([sys.executable, str(lineage_file), "--source", str(source),
            "--variant", variant, "--representation", representation, "--mapping-scope", mapping_scope,
            "--output", str(ref)], capture_output=True, text=True, timeout=15,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        raw = json.loads(ref.read_text()) if ref.exists() else {"status": "FAIL", "checks": {}, "collection_error": "MISSING_RAW_RECORD"}
        launch = ref.with_name(ref.stem + "-launch.json")
        save(launch, {"returncode": process.returncode, "stdout": process.stdout, "stderr": process.stderr,
                      "api_calls": 0, "source": str(source)})
        return {name: {**row, "evidence": [str(ref) + "#" + name, str(launch)]}
                for name, row in raw.get("checks", {}).items()}

    original_witnesses = campaign / "changes-supplemental/original-M04-lineage-collision-v1"
    if original_witnesses.exists(): raise FileExistsError("never overwrite original collision witnesses")
    original_witnesses.mkdir(parents=True)
    witnesses = {}
    for variant in ["A", "B"]:
        record_ref = campaign / f"changes/CH-DP41-{variant}-M04/result.json"
        record = json.loads(record_ref.read_text())
        source = record_ref.parent / "source"
        if changes.snapshot_digest(changes.source_snapshot(source)) != record["after_source_hash"]:
            raise ValueError("original M04 source changed")
        witnesses[variant] = lineage_probe(source, variant, "original", "request", original_witnesses / (variant + "-raw.json"))
    save(original_witnesses / "summary.json", {"checks": witnesses, "api_calls": 0,
        "interpretation": "Original failures are raw defect evidence; they are not counted as a DP semantic accuracy difference."})
    required = [*changes.TASKS["M04"]["completion"], *changes.TASKS["M04"]["regressions"]]
    changes.TASKS["M04"] = {**changes.TASKS["M04"], "regressions": [*changes.TASKS["M04"]["regressions"],
        "same_request_two_revisions", "actual_owner_revision_and_stale_execution", "source_unchanged"]}
    metadata = {}
    def patch(source):
        record = patch_module.patch_corrected_a_failure(source, tools)
        metadata.update(record)
        save(source.parent / "a-semantic-integrity.json", record["a_semantic_sha256"])
        return record
    def regressions(source):
        checks = replay_runner(source, "A", "M04")
        checks.update(changes.execute_runtime_requirement(source, "A", "M04"))
        checks.update(lineage_probe(source, "A", "original", "revision", source.parent / "producer-lineage-checks.json"))
        return checks
    def audit(source):
        checks = replay_runner(source, "A", "M04", audit=True)
        checks.update(changes.execute_runtime_requirement(source, "A", "M04", audit=True))
        ref = source.parent / "baseline-runtime-checks.json"; save(ref, checks)
        supports = all(isinstance(checks.get(name), dict) and checks[name].get("passed") is True
            and bool(checks[name].get("evidence")) for name in required)
        return {"supports_requirement": supports, "evidence": [str(ref)], "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION"}
    output = campaign / "changes-alternatives/ORIGINAL-A-LINEAGE-v1"
    record = changes.run_change_job(baseline, output, "A", "M04", patch, regressions, lambda source, record: [], audit)
    save(output / "alternative-manifest.json", {"alternative_job_id": "CH-DP41-A-M04-ORIGINAL-LINEAGE-v1",
        "original_result_ref": str(campaign / "changes/CH-DP41-A-M04/result.json"), "alternative_result_ref": record["result_ref"],
        "helper_source_digest": manifest["source_digest"], "original_collision_witnesses": str(original_witnesses),
        "correction_relative_to_original": metadata.get("correction_relative_to_original_a_m04"),
        "status": record["status"], "selection": "ROOT_EXPLICIT_SELECTION_MANIFEST_REQUIRED",
        "author_responsibility_review_applied": False, "api_calls": 0})
    return record


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--campaign", required=True)
    parser.add_argument("--helper", default=str(Path(__file__).resolve().parent.parent))
    args = parser.parse_args()
    record = run_corrected_a(args.campaign, args.helper)
    print(json.dumps({k: record[k] for k in ["status", "result_ref", "completion_passed", "regression_passed"]}, ensure_ascii=False))
