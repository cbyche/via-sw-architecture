import json

from conftest import REPO_ROOT
from dp00_analysis.validation import StrictValidationError, validate_behavior_plan, validate_canonical_event, validate_oracle, validate_runtime_scenario


def test_asset_golden_expectations_are_not_duplicated():
    root = REPO_ROOT / "benchmark/fixtures/pilot-v0/golden"
    manifest = json.loads((root / "validation-expectations.json").read_text())
    observed = []
    for case in manifest["cases"]:
        value = json.loads((root / case["fixture_path"]).read_text())
        validator = {
            "RUNTIME_SCENARIO": validate_runtime_scenario,
            "BEHAVIOR_PLAN": validate_behavior_plan,
            "ORACLE": validate_oracle,
        }[case["asset_kind"]]
        try:
            validator(value)
            result = "ACCEPT"
        except StrictValidationError:
            result = "REJECT"
        observed.append(result)
        assert result == case["expected"]
    assert observed == [
        "ACCEPT", "ACCEPT", "REJECT", "REJECT", "REJECT", "ACCEPT", "REJECT"
    ]


def test_canonical_event_golden_expectations_match_rust_manifest():
    root = REPO_ROOT / "benchmark/fixtures/pilot-v0/golden/raw-events"
    manifest = json.loads((root / "validation-expectations.json").read_text())
    for case in manifest["cases"]:
        try:
            validate_canonical_event(json.loads((root / case["file"]).read_text()))
            valid = True
        except StrictValidationError:
            valid = False
        assert valid is case["valid"], case["file"]
    assert len(manifest["cases"]) == 11
