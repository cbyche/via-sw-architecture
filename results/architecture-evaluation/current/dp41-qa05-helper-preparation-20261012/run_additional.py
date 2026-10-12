"""Post-timing, post-original-jobs supplemental executor; API-free.

Freeze this entire helper directory in its own immutable generation before use.
Original change sources and their evidence are read-only inputs.
"""
from __future__ import annotations
import argparse
from functools import partial
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys


def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def save(path, data):
    if path.exists(): raise FileExistsError("never overwrite additional evidence: " + str(path))
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2, default=str) + "\n")
def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run(campaign, helper):
    campaign, helper = Path(campaign).resolve(), Path(helper).resolve()
    if (campaign / "invalidated.json").exists(): raise ValueError("campaign is invalidated")
    events = [json.loads(line) for line in (campaign / "campaign-progress.jsonl").read_text().splitlines()]
    if not any(e.get("kind") == "ALL_WORKERS_COMPLETE" for e in events):
        raise ValueError("all timing workers must finish first")
    original = json.loads((campaign / "change-control/execution-summary.json").read_text())
    if len(original["jobs"]) != 12 or not all(j["status"] in {"AWAITING_RESPONSIBILITY_REVIEW", "COMPLETED"} for j in original["jobs"]):
        raise ValueError("all original 12 jobs must be functionally complete before alternatives")
    helper_manifest = json.loads((helper / "helper-manifest.json").read_text())
    for relative, expected in helper_manifest["file_hashes"].items():
        path = (helper / relative).resolve()
        if not path.is_relative_to(helper) or sha(path) != expected:
            raise ValueError("supplemental helper hash mismatch")
    tools = campaign / "evaluator-tools"
    tool_manifest = json.loads((campaign / "evaluation-tool-manifest.json").read_text())
    for relative, expected in tool_manifest["file_hashes"].items():
        if sha(tools / relative) != expected: raise ValueError("original evaluator tool changed")
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(tools))
    from benchmark.architecture.dp41_45_poc import change_experiments as changes, replay
    assert Path(changes.__file__).resolve().is_relative_to(tools)
    assert Path(replay.__file__).resolve().is_relative_to(tools)
    baseline = campaign / "frozen-source"
    patch_module = load_module("goalref_alternative_patch", helper / "goalref-v1/goalref_patch.py")
    replay_runner = partial(replay.regression_runner, campaign=campaign)
    extras = ["strict_namespace_and_frame_collision", "actual_owner_collision_before_effect",
              "actual_request_goal_disjoint_mapping", "ordinary_owner_when_binder_regression", "b_binder_byte_identical"]
    original_required = [*changes.TASKS["M04"]["completion"], *changes.TASKS["M04"]["regressions"]]
    # This process adds explicit stricter alternative support checks. It does
    # not modify the original TASKS file or the original 12-job requirements.
    changes.TASKS["M04"] = {**changes.TASKS["M04"],
        "regressions": [*changes.TASKS["M04"]["regressions"], *extras]}
    patch_metadata = {}
    def patch(source):
        result = patch_module.patch_goalref_continuation(source, tools)
        patch_metadata.update(result)
        save(source.parent / "b-binder-integrity.json", result["b_binder_sha256"])
        return result
    def checks(source):
        if (campaign / "invalidated.json").exists(): raise ValueError("campaign invalidated before alternative")
        checks = replay_runner(source, "B", "M04")
        process = subprocess.run([sys.executable, str(helper / "goalref-v1/goalref_runtime_checks.py"),
            str(source), "B", "M04"], capture_output=True, text=True, timeout=30,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        try: raw = json.loads(process.stdout)
        except json.JSONDecodeError: raw = {"checks": {}, "collection_error": "INVALID_PROBE_OUTPUT"}
        raw.update(returncode=process.returncode, stdout=process.stdout, stderr=process.stderr)
        ref = source.parent / "goalref-runtime-checks.json"
        save(ref, raw)
        checks.update({name: {**value, "evidence": [str(ref) + "#" + name]}
                       for name, value in raw.get("checks", {}).items()})
        checks["b_binder_byte_identical"] = {"passed": sha(source / "prototypes/candidates/dp41_45_poc/dp41/b.py") ==
            sha(baseline / "prototypes/candidates/dp41_45_poc/dp41/b.py"),
            "evidence": [str(source.parent / "b-binder-integrity.json")]}
        return checks
    def audit(source):
        audited = replay_runner(source, "B", "M04", audit=True)
        audited.update(changes.execute_runtime_requirement(source, "B", "M04", audit=True))
        ref = source.parent / "baseline-runtime-checks.json"
        save(ref, audited)
        supports = all(isinstance(audited.get(name), dict) and audited[name].get("passed") is True
                       and bool(audited[name].get("evidence")) for name in original_required)
        return {"supports_requirement": supports, "evidence": [str(ref)],
                "reason": "Original failure-continuation baseline support is computed from the actual required baseline checks.",
                "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION"}
    output = campaign / "changes-alternatives/GOALREF-v1"
    record = changes.run_change_job(baseline, output, "B", "M04", patch, checks, lambda source, record: [], audit)
    additional_manifest = {"alternative_job_id": "CH-DP41-B-M04-GOALREF-v1", "original_job_id": "CH-DP41-B-M04",
        "original_result_ref": str(campaign / "changes/CH-DP41-B-M04/result.json"), "alternative_result_ref": record["result_ref"],
        "helper_generation": str(helper), "helper_source_digest": helper_manifest["source_digest"],
        "tool_source_digest": tool_manifest["source_digest"], "additional_required_regressions": extras,
        "actual_model_semantic_remeasurement": False, "api_calls": 0,
        "selection": "ROOT_EXPLICIT_SELECTION_MANIFEST_REQUIRED", "status": record["status"],
        "b_binder_integrity": patch_metadata.get("b_binder_sha256"), "responsibility_reviewers_required": 2}
    save(output / "alternative-manifest.json", additional_manifest)

    token_probe = load_module("m05_published_token_restart", helper / "m05-restart-v1/m05_restart_probe.py")
    supplements = []
    for variant in ["A", "B"]:
        if (campaign / "invalidated.json").exists(): raise ValueError("campaign invalidated before supplemental")
        original_ref = campaign / f"changes/CH-DP41-{variant}-M05/result.json"
        original_record = json.loads(original_ref.read_text())
        source = original_ref.parent / "source"
        if changes.snapshot_digest(changes.source_snapshot(source)) != original_record["after_source_hash"]:
            raise ValueError("original M05 change source hash changed")
        result = token_probe.run_supplement(source, variant,
            campaign / f"changes-supplemental/CH-DP41-{variant}-M05-PUBLISHED-TOKEN-RESTART-v1")
        supplements.append(result)
    result = {"alternative": additional_manifest, "m05_supplements": supplements,
              "api_calls": 0, "author_responsibility_review_applied": False}
    save(campaign / "change-control/additional-execution-summary.json", result)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--campaign", required=True)
    parser.add_argument("--helper", default=str(Path(__file__).resolve().parent))
    args = parser.parse_args()
    print(json.dumps(run(args.campaign, args.helper), ensure_ascii=False, default=str))
