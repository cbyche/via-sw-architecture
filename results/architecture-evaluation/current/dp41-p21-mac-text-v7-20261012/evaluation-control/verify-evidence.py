"""Read-only completion and raw-artifact integrity checks; never changes grades."""
import datetime
import gzip
import hashlib
import json
from pathlib import Path

CAMPAIGN = Path(__file__).resolve().parents[1]


def rows(path):
    with path.open() as handle:
        for line in handle:
            if line.strip():
                yield json.loads(line)


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def run():
    manifest = json.loads((CAMPAIGN / "manifest.json").read_text())
    errors, workers, calls, artifact_count = [], [], set(), 0
    for relative, expected in manifest["frozen_source_hashes"].items():
        path = CAMPAIGN / "frozen-source" / relative
        if not path.exists() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            errors.append(f"frozen source differs: {relative}")
    directories = sorted((CAMPAIGN / "workers").iterdir())
    if {p.name for p in directories} != {f"{v}{r}" for v in "AB" for r in range(1, 6)}:
        errors.append("worker population differs")
    for directory in directories:
        trials = list(rows(directory / "trials-unjudged.jsonl"))
        if [t["case_id"] for t in trials] != manifest["sequence"]:
            errors.append(f"trial sequence differs: {directory.name}")
        summaries = list(rows(directory / "worker-summary.jsonl"))
        if not summaries or summaries[-1]["status"] != "COMPLETE":
            errors.append(f"worker unfinished: {directory.name}")
        allocation = next(r for r in rows(directory / "allocations.jsonl")
                          if r["kind"] == "ACTIVITY_ORIGINALS_CREATED")
        with gzip.open(directory / "activity-baseline.json.gz", "rb") as handle:
            source_hash = hashlib.sha256(handle.read()).hexdigest()
        if source_hash != allocation["buffer_sha256"]:
            errors.append(f"original source differs: {directory.name}")
        memory_count, memory_failed = 0, 0
        for sample in rows(directory / "memory.jsonl"):
            memory_count += 1
            memory_failed += not sample["success"]
        workers.append({"worker": directory.name, "trials": len(trials),
                        "original_source_sha256": source_hash,
                        "memory_samples": memory_count, "failed_memory_samples": memory_failed})
        for trial in trials:
            nodes = {n["node_id"]: n for n in trial["causal_timeline"]["nodes"]}
            for node in nodes.values():
                if any(parent not in nodes for parent in node.get("parents", [])):
                    errors.append(f"causal parent absent: {directory.name}/{trial['case_id']}/{node['node_id']}")
    for directory in [*directories, CAMPAIGN / "evaluation"]:
        for call in rows(directory / "model-calls.jsonl"):
            if call["call_id"] in calls:
                errors.append(f"duplicate call id: {call['call_id']}")
            calls.add(call["call_id"])
            if call["provider"] == "qwen" and (call.get("usage") is None or call.get("cost_usd") is None):
                errors.append(f"unknown VIA usage: {call['call_id']}")
            for key in ("context", "output"):
                ref = call.get(key)
                if not ref:
                    continue
                artifact = directory / ref["path"]
                observed = hashlib.sha256(canonical(json.loads(artifact.read_text()))).hexdigest()
                artifact_count += 1
                if observed != ref["sha256"]:
                    errors.append(f"artifact differs: {artifact}")
    completion = json.loads((CAMPAIGN / "final/completion.json").read_text())
    for key, expected in {"observed_trials": 210, "conditions": 1260, "reactions": 270,
                          "cycles": 190, "change_jobs": 12}.items():
        if completion[key] != expected:
            errors.append(f"completion differs: {key}")
    report = {"status": "PASS" if not errors else "FAIL",
              "checked_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "verifier_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "frozen_source_digest": manifest["frozen_source_digest"],
              "frozen_files": len(manifest["frozen_source_hashes"]), "workers": workers,
              "model_calls": len(calls), "verified_model_artifacts": artifact_count,
              "completion": completion, "errors": errors}
    output = CAMPAIGN / "final/evidence-integrity.json"
    if output.exists():
        raise FileExistsError("Do not overwrite the completed integrity audit")
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "model_calls": len(calls),
                      "model_artifacts": artifact_count, "errors": errors}, ensure_ascii=False))
    if errors:
        raise SystemExit(1)


if __name__ == "__main__":
    run()
