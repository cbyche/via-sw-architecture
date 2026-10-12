"""Pin twelve independently reviewed change jobs without replacing raw results."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sys

CAMPAIGN = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(CAMPAIGN / "evaluator-tools"))
from benchmark.architecture.dp41_45_poc.change_experiments import finalize_responsibility_review
from benchmark.architecture.dp41_45_poc.finalize import selected_change_records

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def run():
    selection_path = CAMPAIGN / "qa05-selection-manifest.json"
    if selection_path.exists():
        raise FileExistsError(selection_path)
    root_path = Path(__file__).with_name("root-responsibility-review.json")
    contract_path = CAMPAIGN / "reviews/qa05-contract-independent-review.json"
    root = json.loads(root_path.read_text())
    contract = json.loads(contract_path.read_text())
    manifest = json.loads((CAMPAIGN / "manifest.json").read_text())
    assert root["frozen_source_digest"] == contract["frozen_source_digest"] == manifest["frozen_source_digest"]
    selected = []
    for primary in root["jobs"]:
        result_path = Path(primary["result_ref"])
        record = json.loads(result_path.read_text())
        secondary = contract["jobs"][record["job_id"]]
        assert primary["accepted"] and secondary["accepted"]
        assert primary["changed_unit_set"] == secondary["changed_unit_set"]
        updated = finalize_responsibility_review(result_path, [primary, secondary])
        assert updated["status"] == "COMPLETED"
        reviewed = Path(updated["result_ref"])
        selected.append({"variant": updated["variant"], "task_id": updated["task_id"],
            "reviewed_result_ref": str(reviewed.relative_to(CAMPAIGN)),
            "reviewed_result_sha256": sha(reviewed), "source_snapshot_digest": updated["after_source_hash"],
            "count": updated["count"], "changed_unit_set": updated["changed_unit_set"]})
    selection_path.write_text(json.dumps({
        "status": "COMPLETE", "created_utc": datetime.now(timezone.utc).isoformat(),
        "frozen_source_digest": manifest["frozen_source_digest"], "selected": selected,
        "independent_reviews": [{"path": str(p.relative_to(CAMPAIGN)), "sha256": sha(p)} for p in (root_path, contract_path)],
        "review_type": "TWO_DISTINCT_LLM_AGENT_SOURCE_AND_FUNCTIONAL_TRACE_REVIEWS", "is_human_review": False,
        "M04_selection_reason": "Select corrected A and typed B from fresh measured-baseline copies after required actual producer lineage probes. Original M04 records and their supplemental failures remain preserved, and are not counted twice.",
        "M04_actual_natural_language_probes": {
            variant: {"status": "FAIL", "ref": f"changes-semantic/M04-{variant}-attempt1/change-semantic-validation.jsonl",
                "scope": "EVALUATION_CHANGE", "included_in_P21": False} for variant in "AB"},
        "M06_zero_reason": "Full preserved preference registration and revocation oracle passes on measured baseline and independent unchanged job sources. A common pre-freeze baseline repair is not a feature delta.",
        "original_results_overwritten": False,
    }, ensure_ascii=False, indent=2) + "\n")
    verified = selected_change_records(CAMPAIGN)
    assert len(verified) == 12
    print(json.dumps({"status": "COMPLETE", "selected": len(verified),
        "counts": {v: [r["count"] for r in verified if r["variant"] == v] for v in "AB"},
        "selection_sha256": sha(selection_path)}, ensure_ascii=False))

if __name__ == "__main__":
    run()
