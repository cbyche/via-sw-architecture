"""Record the root's source-and-trace review; never alter an experiment."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sys

CAMPAIGN = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(CAMPAIGN / "evaluator-tools"))
from benchmark.architecture.dp41_45_poc.change_experiments import (
    RESPONSIBILITIES, TASKS, snapshot_digest, source_snapshot,
)

UNITS = {
    "M01": ["U15"], "M02": ["U16"], "M03": ["U09"],
    "M04": ["U01", "U03", "U07", "U08"],
    "M05": ["U04", "U07"], "M06": [],
}
REASONS = {
    "M01": "Actual request/text/usage wire changed; the normalized interpretation port and cancellation disposition are preserved. One provider-adapter responsibility, regardless of method or file count.",
    "M02": "Actual routing/action/input/result wire changed, preserving command identity, original payload and receipt uncertainty. One Agent-adapter responsibility; no new Task authority or semantic responsibility.",
    "M03": "Calendar source fields and UTC/timezone envelope changed; normalization retains source/version/access and busy/free facts. One source-adapter responsibility; no new permission decision.",
    "M04": "New finite terminal-state semantics require a model contract, validation contract, Task failure/currentness authority and actual producer continuation. Request revision and Task/Execution lineage belong to the continuation responsibility. Existing publication, storage, policy and downstream transport contracts are reused. The typed B binder is byte-identical, so U02 is not added merely because its representation differs.",
    "M05": "The actual Task question owner issues and validates a scope token, and the dialogue adoption reflects the actually presented token. Fresh-process restoration reuses unchanged generic serialization; issuing a token for pre-existing owner questions belongs to U07, with no separate persistent migration U14.",
    "M06": "The measured baseline already supports the full use fence and actual original-owner preference registration. Both independent unchanged sources pass the exact S1-03 preservation oracle and revocation regressions. A prior baseline implementation correction is not a change-experiment delta. There is no patch and no new responsibility.",
}

def file_proof(path):
    path = path.resolve()
    return {"path": str(path.relative_to(CAMPAIGN)),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}

def run():
    manifest = json.loads((CAMPAIGN / "manifest.json").read_text())
    baseline = CAMPAIGN / "frozen-source"
    baseline_hash = snapshot_digest(source_snapshot(baseline))
    reviews = []
    for variant in "AB":
        for task in TASKS:
            if task == "M04":
                generation = "ORIGINAL-A-LINEAGE-v1" if variant == "A" else "TYPED-GOALREF-v2"
                directory = CAMPAIGN / "changes-alternatives" / generation / f"CH-DP41-{variant}-{task}"
            else:
                directory = CAMPAIGN / "changes" / f"CH-DP41-{variant}-{task}"
            record = json.loads((directory / "result.json").read_text())
            assert record["status"] == "AWAITING_RESPONSIBILITY_REVIEW"
            assert all(record[flag] for flag in ("completion_passed", "regression_passed", "patch_verified", "original_baseline_unchanged"))
            assert record["baseline_source_hash"] == baseline_hash
            assert snapshot_digest(source_snapshot(directory / "source")) == record["after_source_hash"]
            assert sorted(record["responsibility_claims"]) == UNITS[task]
            assert hashlib.sha256((directory / "patch.diff").read_bytes()).hexdigest() == record["patch_sha256"]
            if task == "M06":
                assert record["baseline_support_audit_passed"] and not record["changed_paths"]
            evidence = [file_proof(directory / name) for name in (
                "result.json", "patch.diff", "baseline-manifest.json", "after-source-manifest.json", "regression.json")]
            if task == "M04":
                evidence.append(file_proof(CAMPAIGN / "changes-semantic" / f"M04-{variant}-attempt1" / "change-semantic-validation.jsonl"))
            if task == "M05":
                evidence.append(file_proof(CAMPAIGN / "changes-supplemental" / f"CH-DP41-{variant}-M05-PUBLISHED-TOKEN-RESTART-v1" / "result.json"))
            if task == "M06":
                for phase in ("before", "after"):
                    evidence.append(file_proof(CAMPAIGN / "changes-supplemental/M06-EXACT-S103-v3" / f"{variant}-{phase}" / "result.json"))
            reviews.append({
                "reviewer_id": "ROOT_SOURCE_AND_TRACE_REVIEW_20261012",
                "review_type": "LLM_AGENT_CODE_AND_RECORDED_FUNCTIONAL_TRACE_REVIEW",
                "is_human_review": False, "variant": variant, "task_id": task,
                "result_ref": str(directory / "result.json"), "accepted": True,
                "changed_unit_set": UNITS[task], "reason": REASONS[task], "evidence": evidence,
                "all_18_assessment": [{"unit": unit, "responsibility": label,
                    "changed": unit in UNITS[task],
                    "reason": record["responsibility_claims"][unit]["reason"] if unit in UNITS[task]
                        else "No separate before/after responsibility delta in the inspected patch and executed ports; reused generic behavior is not counted again."}
                    for unit, label in RESPONSIBILITIES.items()],
                "actual_model_extension_validation": "FAIL_PRESERVED_SEPARATELY" if task == "M04" else "NOT_AN_ACTUAL_MODEL_ACCURACY_EXPERIMENT",
            })
    output = Path(__file__).with_name("root-responsibility-review.json")
    if output.exists():
        raise FileExistsError(output)
    output.write_text(json.dumps({"reviewed_utc": datetime.now(timezone.utc).isoformat(),
        "frozen_source_digest": manifest["frozen_source_digest"], "source_snapshot_digest": baseline_hash,
        "reviewer_script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "jobs": reviews, "selected_jobs": 12, "source_and_raw_modified": False}, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"path": str(output), "jobs": len(reviews), "units_per_variant": UNITS}, ensure_ascii=False))

if __name__ == "__main__":
    run()
