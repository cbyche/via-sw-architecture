import json

from conftest import REPO_ROOT


def load_policy():
    return json.loads(
        (
            REPO_ROOT / "benchmark/contracts/qa04-route-contract-policy-v1.json"
        ).read_text()
    )


def test_policy_is_contract_aware_and_contains_no_numeric_penalty():
    policy = load_policy()
    assert policy["policy_version"] == "qa04-route-contract-policy-v1"
    assert set(policy["route_contract_classes"]) == {
        "REQUIRED",
        "OPTIONAL",
        "FORBIDDEN",
    }
    assert policy["primary_observation_rule"]["contract"] == "REQUIRED"
    assert policy["primary_observation_rule"]["no_route_numeric_value"] is None
    assert policy["primary_comparability_qualification"][
        "qa02_pass_alone_is_sufficient"
    ] is False
    assert policy["arbitrary_numeric_penalty"]["allowed"] is False
    assert policy["arbitrary_numeric_penalty"]["status"] == "REJECT"


def test_policy_declares_all_machine_readable_route_statuses():
    statuses = {rule["status"] for rule in load_policy()["derived_status_rules"]}
    assert statuses == {
        "ROUTE_COMMITTED",
        "ROUTE_REQUIRED_NOT_COMMITTED",
        "ROUTE_OPTIONAL_NOT_COMMITTED",
        "ROUTE_FORBIDDEN_NOT_COMMITTED",
        "ROUTE_FORBIDDEN_BUT_COMMITTED",
    }
