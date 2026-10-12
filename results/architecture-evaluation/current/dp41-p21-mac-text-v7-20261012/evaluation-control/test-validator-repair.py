"""API-free validation repair regression on actual unmodified mixed deliveries."""
import copy
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

CAMPAIGN = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(CAMPAIGN / "evaluator-tools-v2"))

def load_judge(name, root):
    spec = importlib.util.spec_from_file_location(name, root / "benchmark/architecture/dp41_45_poc/judge.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

old = load_judge("original_judge", CAMPAIGN / "evaluator-tools")
new = load_judge("repaired_judge", CAMPAIGN / "evaluator-tools-v2")
assert old.SYSTEM == new.SYSTEM
assert old.calibration_cases() == new.calibration_cases()
checks = []
for case in ("S4-02", "S5-04"):
    trial_path = CAMPAIGN / "workers/B1/trials-unjudged.jsonl"
    trial = next(json.loads(line) for line in trial_path.read_text().splitlines()
                 if json.loads(line)["case_id"] == case)
    delivered = next(d for d in trial["deliveries"]
                     if d.get("publication_id") and d.get("text_presented_s") is not None
                     and (not d.get("voice_on") or d.get("voice_onset_s") is not None)
                     and (d.get("voice_text") if d.get("voice_on") else d.get("text")))
    quote = delivered["voice_text"] if delivered.get("voice_on") else delivered["text"]
    result = {"conditions": {f"C{i}": {"passed": False, "evidence": [], "reason": "Structural validator test; not a semantic grade."} for i in range(1, 7)},
              "reactions": [{"id": r["id"], "first_actual_publication_id": None,
                  "valid_publication_id": None, "first_actual_core_quote": None,
                  "valid_core_quote": None, "wrong_first": False, "evidence": [],
                  "reason": "Schema-only test."} for r in trial["reaction_blueprint"]],
              "cycle": {"valid_publication_id": delivered["publication_id"],
                  "valid_core_quote": quote, "evidence": [delivered["publication_id"]],
                  "reason": "Quote validation only, not a claimed valid cycle."},
              "critical_violations": [], "uncertain": False}
    try:
        old.validate_judgement(copy.deepcopy(result), trial)
    except KeyError as error:
        assert error.args == ("publication_id",)
    else:
        raise AssertionError("Original bug was not reproduced")
    new.validate_judgement(copy.deepcopy(result), trial)
    checks.append({"case_id": case, "check": "original_KeyError_and_repaired_exact_quote", "passed": True})
    for label, mutate in (
        ("fabricated_publication_rejected", lambda r: r["cycle"].update(valid_publication_id="fabricated-publication")),
        ("non_emitted_quote_rejected", lambda r: r["cycle"].update(valid_core_quote="NOT_AN_ACTUALLY_EMITTED_CLAUSE")),
        ("local_stop_publication_rejected", lambda r: next(x for x in r["reactions"] if next(b for b in trial["reaction_blueprint"] if b["id"] == x["id"])["endpoint_kind"] == "LOCAL_OUTPUT_STOP").update(valid_publication_id=delivered["publication_id"], valid_core_quote=quote)),
    ):
        bad = copy.deepcopy(result)
        mutate(bad)
        try:
            new.validate_judgement(bad, trial)
        except ValueError:
            checks.append({"case_id": case, "check": label, "passed": True})
        else:
            raise AssertionError(label)
before = json.loads((CAMPAIGN / "evaluation-tool-manifest.json").read_text())
after = json.loads((CAMPAIGN / "evaluation-tool-manifest-v2.json").read_text())
changes = []
for relative, original_hash in before["file_hashes"].items():
    assert hashlib.sha256((CAMPAIGN / "evaluator-tools" / relative).read_bytes()).hexdigest() == original_hash
    assert hashlib.sha256((CAMPAIGN / "frozen-source" / relative).read_bytes()).hexdigest() == original_hash
    repaired_hash = hashlib.sha256((CAMPAIGN / "evaluator-tools-v2" / relative).read_bytes()).hexdigest()
    assert repaired_hash == after["file_hashes"][relative]
    if original_hash != repaired_hash:
        changes.append(relative)
assert changes == ["benchmark/architecture/dp41_45_poc/judge.py"]
report = {"status": "PASS", "utc": datetime.now(timezone.utc).isoformat(),
          "API_calls": 0, "semantic_grades_created": 0, "checks": checks,
          "original_frozen_files_unchanged": len(before["file_hashes"]),
          "evaluator_only_changed_files": changes, "judge_prompt_unchanged": True,
          "oracle_threshold_candidate_unchanged": True,
          "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(CAMPAIGN / "evaluation-control/validator-repair-tests.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
print(json.dumps({"status": report["status"], "checks": len(checks), "API_calls": 0}))
