from dataclasses import replace

from conftest import model_call
from dp00_analysis.loader import load_evidence
from dp00_analysis.qa04 import derive_qa04, episode_calls_to_commit


def test_t08_two_precommit_calls_and_t09_postcommit_exclusion(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    calls = (model_call(1, start=1_150_000), model_call(2, start=1_250_000), model_call(3, start=1_350_000, included=False, before=True))
    item = episode_calls_to_commit(replace(episode, model_calls=calls))
    assert item["calls_to_route_commit"] == 2
    assert item["included_model_call_ids"] == ["M1", "M2"]


def test_t10_no_route_commit_has_diagnostics_not_numeric(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    actual = replace(episode.actual, execution_routes=(), failure_outcomes=("MODEL_TIMEOUT",))
    item = episode_calls_to_commit(replace(episode, actual=actual))
    assert item["no_route_commit"] is True
    assert item["calls_to_route_commit"] is None
    assert item["model_calls_before_terminal_failure"] == 1
    assert item["route_contract_status"] == "ROUTE_REQUIRED_NOT_COMMITTED"


def test_t11_retry_counts_two_logical_generations_and_t12_zero_call_success(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    retry = replace(episode, model_calls=(model_call(1), model_call(2, start=1_250_000)))
    assert episode_calls_to_commit(retry)["calls_to_route_commit"] == 2
    assert episode_calls_to_commit(replace(episode, model_calls=()))["calls_to_route_commit"] == 0


def test_overall_and_macro_candidates_with_class_contribution(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    r1 = replace(episode, model_calls=(model_call(1),), provenance=dict(episode.provenance, run_id="r1"))
    r2 = replace(episode, model_calls=(model_call(1), model_call(2)), provenance=dict(episode.provenance, run_id="r2"), scenario=dict(episode.scenario, scenario_class="R2"))
    result = derive_qa04([r1, r2])
    aggregate = result["aggregates"]["profile-z:A"]
    assert aggregate["overall_mean"] == aggregate["macro_average"] == 1.5
    assert set(aggregate["per_class"]) == {"R1", "R2"}


def test_contract_candidate_requires_route_and_qa02_independently(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    committed = replace(
        episode,
        provenance=dict(episode.provenance, run_id="committed"),
    )
    no_route = replace(
        episode,
        provenance=dict(episode.provenance, run_id="no-route"),
        actual=replace(episode.actual, execution_routes=()),
    )
    qa02 = {
        "scenario_diagnostics": [
            {"run_id": "committed", "exact_conformant": True},
            {"run_id": "no-route", "exact_conformant": True},
        ]
    }
    result = derive_qa04([committed, no_route], qa02)
    diagnostic = result["qa04_contract_diagnostics"]["profile-z:A"]
    assert diagnostic["required_population"] == 2
    assert diagnostic["qualified_primary_observations"] == 1
    assert diagnostic["primary_comparability_qualification"] == "FAIL"
    assert diagnostic["qualified_primary_mean"] is None


def test_forbidden_and_optional_no_route_statuses_are_distinct(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    no_route_actual = replace(episode.actual, execution_routes=())
    forbidden = replace(
        episode,
        actual=no_route_actual,
        scenario={
            **episode.scenario,
            "qa_eligibility": {
                **episode.scenario["qa_eligibility"],
                "qa04": {
                    "eligible": True,
                    "route_commit_expectation": "FORBIDDEN",
                    "exclusion_reason": "NOT_APPLICABLE",
                },
            },
        },
    )
    optional = replace(
        forbidden,
        scenario={
            **forbidden.scenario,
            "qa_eligibility": {
                **forbidden.scenario["qa_eligibility"],
                "qa04": {
                    "eligible": True,
                    "route_commit_expectation": "OPTIONAL",
                    "exclusion_reason": "NOT_APPLICABLE",
                },
            },
        },
    )
    assert episode_calls_to_commit(forbidden)["route_contract_status"] == "ROUTE_FORBIDDEN_NOT_COMMITTED"
    assert episode_calls_to_commit(optional)["route_contract_status"] == "ROUTE_OPTIONAL_NOT_COMMITTED"
