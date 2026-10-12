"""Apply explicit root decisions without editing an original judge record."""
import copy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sys

CAMPAIGN = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(CAMPAIGN / "evaluator-tools-v2"))
from benchmark.architecture.dp41_45_poc.judge import validate_judgement


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(",", ":")).encode()


def read_rows(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def get_path(value, path):
    for part in path:
        value = value[part]
    return value


def changed_leaves(before, after, path=()):
    if before == after:
        return []
    if isinstance(before, dict) and isinstance(after, dict) and set(before) == set(after):
        return [leaf for key in before for leaf in changed_leaves(before[key], after[key], (*path, key))]
    # Reaction arrays keep their declared identities; merge only changed leaves,
    # so two separately reviewed endpoints cannot erase one another.
    if (isinstance(before, list) and isinstance(after, list) and len(before) == len(after)
            and before and all(isinstance(row, dict) and "id" in row for row in before + after)
            and [row["id"] for row in before] == [row["id"] for row in after]):
        return [leaf for index in range(len(before))
                for leaf in changed_leaves(before[index], after[index], (*path, index))]
    return [(path, before, after)]


def run():
    decision_path = CAMPAIGN / "reviews/root-adjudication-decisions.json"
    decisions = json.loads(decision_path.read_text())
    originals = {(r["variant"], r["repeat"], r["case_id"]): r
                 for r in read_rows(CAMPAIGN / "evaluation/adjudications.jsonl")}
    trials = {(r["variant"], r["repeat"], r["case_id"]): r
              for path in sorted((CAMPAIGN / "workers").glob("*/trials-unjudged.jsonl"))
              for r in read_rows(path)}
    if len(originals) != 210 or set(originals) != set(trials):
        raise ValueError("All 210 original adjudications must exist before final review")
    proposals = {}
    proposal_sources = []
    for ref in decisions["proposal_files"]:
        path = CAMPAIGN / ref["path"]
        observed = hashlib.sha256(path.read_bytes()).hexdigest()
        if observed != ref["sha256"]:
            raise ValueError("Reviewed proposal ledger changed")
        proposal_sources.append(ref)
        for row in read_rows(path):
            if row["audit_id"] in proposals:
                raise ValueError("Duplicate audit proposal")
            proposals[row["audit_id"]] = row
    if set(decisions["dispositions"]) != set(proposals):
        raise ValueError("Every proposal needs an explicit root disposition")
    current, accepted = {}, []
    for audit_id, decision in decisions["dispositions"].items():
        if not decision["reason"] or decision["decision"] not in ("ACCEPT", "REJECT", "WITHDRAWN"):
            raise ValueError("Root disposition is incomplete")
        if decision["decision"] != "ACCEPT":
            continue
        proposal = proposals[audit_id]
        if proposal.get("status") != "PROPOSED_NOT_APPLIED":
            raise ValueError("A withdrawn proposal cannot be applied")
        key = proposal["variant"], proposal["repeat"], proposal["case_id"]
        original = originals[key]
        if hashlib.sha256(canonical(original)).hexdigest() != proposal["source_adjudication_sha256"]:
            raise ValueError("Original adjudication differs from reviewed source")
        if get_path(original, proposal["path"]) != proposal["old_value"]:
            raise ValueError("Original field differs from reviewed old value")
        record = current.setdefault(key, copy.deepcopy(original))
        for path, old, new in changed_leaves(proposal["old_value"], proposal["proposed_value"]):
            full_path = (*proposal["path"], *path)
            present = get_path(record, full_path)
            if present not in (old, new):
                raise ValueError(f"Conflicting reviewed changes: {audit_id}/{full_path}")
            parent = get_path(record, full_path[:-1])
            parent[full_path[-1]] = copy.deepcopy(new)
        record.setdefault("root_trace_audits", []).append({
            "audit_id": audit_id, "reviewer": proposal["reviewer"],
            "root_reason": decision["reason"], "actual_evidence": proposal["evidence"],
            "original_record_sha256": proposal["source_adjudication_sha256"],
            "API_calls_in_audit": 0, "is_human_review": False})
        accepted.append(audit_id)
    for key, record in current.items():
        validate_judgement(record["final_judgement"], trials[key])
        record["method"] += "_PLUS_EVIDENCE_LINKED_LLM_AGENT_AND_ROOT_TRACE_REVIEW"
        record["original_judge_records_preserved"] = True
    destination = CAMPAIGN / "evaluation/trace-adjudication-overrides.jsonl"
    if destination.exists():
        raise FileExistsError("Do not overwrite an earlier root review ledger")
    with destination.open("w") as handle:
        for key in sorted(current):
            handle.write(json.dumps(current[key], ensure_ascii=False) + "\n")
    report = {"status": "COMPLETE", "utc": datetime.now(timezone.utc).isoformat(),
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "decisions_sha256": hashlib.sha256(decision_path.read_bytes()).hexdigest(),
        "proposal_sources": proposal_sources, "proposals": len(proposals),
        "accepted_proposals": accepted, "overridden_trials": len(current),
        "override_ledger_sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
        "source_or_candidate_changes": 0, "API_calls": 0, "is_human_review": False}
    (CAMPAIGN / "reviews/root-adjudication-completion.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"proposals": len(proposals), "accepted": len(accepted),
                      "overridden_trials": len(current)}))


if __name__ == "__main__":
    run()
