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
