"""Pin completed original judge ledgers and preserved retry origins without edits."""
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path

CAMPAIGN = Path(__file__).resolve().parents[1]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]

def run():
    target = CAMPAIGN / "evaluation/evaluation-origin-manifest.json"
    if target.exists():
        raise FileExistsError(target)
    records = rows(CAMPAIGN / "evaluation/adjudications.jsonl")
    keys = {(r["variant"], r["repeat"], r["case_id"]) for r in records}
    assert len(records) == len(keys) == 210
    calls = rows(CAMPAIGN / "evaluation/model-calls.jsonl")
    assert len(calls) == len({r["call_id"] for r in calls})
    result = {"status": "COMPLETE", "utc": datetime.now(timezone.utc).isoformat(),
        "script_sha256": sha(Path(__file__)), "original_adjudications": len(records),
        "variant_counts": dict(Counter(r["variant"] for r in records)),
        "methods": dict(Counter(r["method"] for r in records)),
        "original_records_not_rewritten": True, "candidate_remeasurement_for_evaluator_repair": False,
        "same_initial_prompt_model_oracle_thresholds": True,
        "schema_retry_feedback_evolved_with_all_outputs_preserved": ["V3 enclosing evidence references", "V4 actual publication and emitted-clause references"],
        "raw_ledgers": [], "evaluator_manifests": [],
        "preserved_retry_merge": {"source": "evaluation/a-preserved-retry",
            "new_adjudications_merged": sum(bool(r.get("evaluation_retry_origin")) for r in records),
            "copied_seed_adjudications_are_not_new_votes": True,
            "origin_fields": ["evaluation_retry_origin", "source_row_sha256"]}}
    for path in sorted((CAMPAIGN / "evaluation").glob("*.jsonl")):
        result["raw_ledgers"].append({"ref": str(path.relative_to(CAMPAIGN)), "sha256": sha(path)})
    for path in sorted((CAMPAIGN / "evaluation/a-preserved-retry").glob("*.jsonl")):
        result["raw_ledgers"].append({"ref": str(path.relative_to(CAMPAIGN)), "sha256": sha(path)})
    for name in ("evaluation-tool-manifest.json", "evaluation-tool-manifest-v2.json", "evaluation-tool-manifest-v3.json", "evaluation-tool-manifest-v4.json"):
        path = CAMPAIGN / name
        manifest = json.loads(path.read_text())
        result["evaluator_manifests"].append({"ref": name, "sha256": sha(path),
            "source_digest": manifest["source_digest"]})
    target.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "adjudications": len(records), "calls": len(calls)}))

if __name__ == "__main__":
    run()
