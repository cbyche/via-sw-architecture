"""API-free checks of the copied V4 schema-retry feedback and fixed validator."""
import ast
import copy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
campaign = Path(__file__).resolve().parent.parent
source = campaign / "evaluator-tools-v4"
sys.path.insert(0, str(source))
from benchmark.architecture.dp41_45_poc.judge import evidence_repair_feedback, validate_judgement
import benchmark.architecture.dp41_45_poc.judge as actual_module


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


checks = []


def check(name, passed, detail=None):
    checks.append({"name": name, "passed": bool(passed), "detail": detail})


check("copied_V4_module_actually_imported", Path(actual_module.__file__).resolve().is_relative_to(source.resolve()), actual_module.__file__)
old_path = campaign / "evaluator-tools-v3/benchmark/architecture/dp41_45_poc/judge.py"
new_path = source / "benchmark/architecture/dp41_45_poc/judge.py"
old_ast, new_ast = ast.parse(old_path.read_text()), ast.parse(new_path.read_text())
for name in ("evaluation_view", "validate_judgement", "grade", "calibration_cases", "calibrate_judge"):
    before = next((n for n in old_ast.body if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name == name), None)
    after = next((n for n in new_ast.body if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name == name), None)
    check("unchanged_" + name, ast.dump(before) == ast.dump(after) if before is not None and after is not None else before is after)
for name in ("SYSTEM",):
    before = next(n for n in old_ast.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == name for t in n.targets))
    after = next(n for n in new_ast.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == name for t in n.targets))
    check("unchanged_" + name, ast.dump(before) == ast.dump(after))

trial = {"events": [{"kind": "READ_RETURNED", "evidence_id": "E1", "source_id": "D1", "causal_node_id": "TL:N1"}],
         "effects": [], "meanings": [], "initial": {}, "observed_inputs": [], "final_tasks": {}, "final_questions": {},
         "cycle_blueprint": {"id": "cycle"}, "reaction_blueprint": [{"id": "r1", "endpoint_kind": "MEANINGFUL_RESPONSE"}],
         "deliveries": [{"kind": "LOCAL_STOP", "text": "old"},
                        {"kind": "RESPONSE", "publication_id": "P_REAL", "request_id": "R_CURRENT", "generation": 1,
                         "text": "기한은 5일입니다. 아직 발화하지 않은 문장입니다.", "voice_text": "기한은 5일입니다.",
                         "voice_on": True, "text_presented_s": 1.0, "voice_onset_s": 1.1},
                        {"kind": "RESPONSE", "publication_id": "P_TEXT", "request_id": "R_CURRENT", "generation": 1,
                         "text": "텍스트 표시 원문입니다.", "voice_text": "", "voice_on": False,
                         "text_presented_s": 2.0, "voice_onset_s": None}]}
result = {"conditions": {f"C{i}": {"passed": i == 1, "evidence": ["E1"], "reason": "fixed semantics"} for i in range(1, 7)},
          "reactions": [{"id": "r1", "first_actual_publication_id": "P_REAL", "valid_publication_id": "P_REAL",
                         "first_actual_core_quote": "기한은 5일입니다.", "valid_core_quote": "기한은 5일입니다.",
                         "wrong_first": False, "evidence": ["P_REAL"], "reason": "fixed semantics"}],
          "cycle": {"valid_publication_id": "P_REAL", "valid_core_quote": "기한은 5일입니다.", "evidence": ["P_REAL"]},
          "critical_violations": [], "uncertain": False}


def feedback_for(label, value):
    before, trial_before = copy.deepcopy(value), copy.deepcopy(trial)
    feedback = evidence_repair_feedback(value, trial, ValueError(label))
    check(label + "_no_result_or_trial_mutation", value == before and trial == trial_before)
    check(label + "_all_semantic_fields_unchanged", value["conditions"] == before["conditions"] and
          value["uncertain"] == before["uncertain"] and value["reactions"][0]["wrong_first"] == before["reactions"][0]["wrong_first"])
    return json.loads(feedback.split("\n", 1)[1]) if "\n" in feedback else None


def reject(label, value):
    try:
        validate_judgement(value, trial)
    except ValueError:
        check(label + "_still_rejected", True)
    else:
        check(label + "_still_rejected", False)


check("valid_original_still_validates", validate_judgement(copy.deepcopy(result), trial) == result)
wrong = copy.deepcopy(result)
wrong["cycle"]["valid_publication_id"] = "P_INVENTED"
body = feedback_for("fabricated_publication", wrong)
reject("fabricated_publication", wrong)
check("unknown_publication_field_located", any(i["field"] == "cycle.valid_publication_id" and i["issue"] == "UNKNOWN_PUBLICATION_ID"
                                              for i in body["invalid_endpoint_reference_or_quote_fields"]))
check("only_actual_publication_ids_listed", {p["publication_id"] for p in body["actual_publications_for_schema_reference"]} == {"P_REAL", "P_TEXT"})
check("LOCAL_STOP_not_invented_as_publication", all(p["publication_id"] is not None for p in body["actual_publications_for_schema_reference"]))
check("voice_sink_prefix_not_generated_tail", next(p for p in body["actual_publications_for_schema_reference"] if p["publication_id"] == "P_REAL")["actual_emitted_text"] == "기한은 5일입니다.")
wrong = copy.deepcopy(result)
wrong["reactions"][0]["first_actual_core_quote"] = "기한은 6일입니다."
body = feedback_for("quote_typo", wrong)
reject("quote_typo", wrong)
check("typo_exact_field_and_actual_text", any(i["field"] == "reactions[0].first_actual_core_quote" and i["actual_emitted_text"] == "기한은 5일입니다."
                                            for i in body["invalid_endpoint_reference_or_quote_fields"]))
wrong = copy.deepcopy(result)
wrong["cycle"]["valid_core_quote"] = "아직 발화하지 않은 문장입니다."
body = feedback_for("unemitted_generated_tail", wrong)
reject("unemitted_generated_tail", wrong)
check("unemitted_tail_not_supplied_as_spoken", body["invalid_endpoint_reference_or_quote_fields"][0]["actual_emitted_text"] == "기한은 5일입니다.")
wrong = copy.deepcopy(result)
wrong["cycle"]["valid_publication_id"] = None
body = feedback_for("null_publication_quote", wrong)
reject("null_publication_quote", wrong)
check("null_id_quote_issue_reported", body["invalid_endpoint_reference_or_quote_fields"][0]["issue"] == "NULL_PUBLICATION_HAS_QUOTE")
text_value = copy.deepcopy(result)
text_value["cycle"].update(valid_publication_id="P_TEXT", valid_core_quote="텍스트 표시 원문입니다.")
check("Voice_OFF_displayed_quote_still_validates", validate_judgement(text_value, trial) == text_value)
wrong = copy.deepcopy(text_value)
wrong["cycle"]["valid_core_quote"] = "텍스트 표시 오타입니다."
body = feedback_for("Voice_OFF_quote_typo", wrong)
reject("Voice_OFF_quote_typo", wrong)
check("Voice_OFF_actual_displayed_text", body["invalid_endpoint_reference_or_quote_fields"][0]["actual_emitted_text"] == "텍스트 표시 원문입니다.")
wrong = copy.deepcopy(result)
wrong["conditions"]["C1"]["evidence"] = ["D1", "TL:N1", "D_INVENTED"]
body = feedback_for("evidence_reference_regression", wrong)
check("existing_actual_enclosing_evidence_preserved", body["actual_supplied_records_containing_those_values"]["D1"] == [{"evidence_id": "E1"}])
check("invented_evidence_not_mapped", body["actual_supplied_records_containing_those_values"]["D_INVENTED"] == [])
check("valid_feedback_no_automatic_endpoint_rewrite", feedback_for("already_valid", copy.deepcopy(result)) is None)

original_manifest = json.loads((campaign / "manifest.json").read_text())
for name, expected in original_manifest["frozen_source_hashes"].items():
    check("frozen_original_unchanged", sha(campaign / "frozen-source" / name) == expected, name)
manifest = json.loads((campaign / "evaluation-tool-manifest-v4.json").read_text())
previous_manifest = json.loads((campaign / "evaluation-tool-manifest-v3.json").read_text())
for name, expected in manifest["file_hashes"].items():
    check("V4_copied_hash", sha(source / name) == expected, name)
    if name != "benchmark/architecture/dp41_45_poc/judge.py":
        check("V4_only_judge_changed", expected == previous_manifest["file_hashes"][name], name)
    check("V3_preserved", sha(campaign / "evaluator-tools-v3" / name) == previous_manifest["file_hashes"][name], name)
report = {"status": "PASS" if all(c["passed"] for c in checks) else "FINDINGS",
          "utc": datetime.now(timezone.utc).isoformat(), "API_calls": 0, "candidate_or_oracle_changes": 0,
          "SYSTEM_first_pass_view_validation_and_semantic_flags_unchanged": True,
          "actual_emitted_text_only": True, "automatic_semantic_or_endpoint_correction": False,
          "V4_source_digest": manifest["source_digest"], "check_count": len(checks),
          "failed_checks": [c for c in checks if not c["passed"]], "checks": checks,
          "script_sha256": sha(Path(__file__)), "imported_module": actual_module.__file__}
output = campaign / "evaluation-control/exact-emission-feedback-repair-tests.json"
output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
print(json.dumps({k: v for k, v in report.items() if k != "checks"}, ensure_ascii=False))
sys.exit(0 if report["status"] == "PASS" else 1)
