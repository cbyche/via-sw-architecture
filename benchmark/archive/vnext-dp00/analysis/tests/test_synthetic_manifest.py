import json

from conftest import ANALYSIS_ROOT


def test_t01_through_t14_are_explicitly_registered_non_official_fixtures():
    manifest = json.loads((ANALYSIS_ROOT / "tests/fixtures/synthetic-cases.json").read_text())
    assert manifest["official_result"] is False
    assert [case["id"] for case in manifest["cases"]] == [f"T{i:02}" for i in range(1, 15)]
