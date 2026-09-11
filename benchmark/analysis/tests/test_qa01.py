from dataclasses import replace

from conftest import success_events
from dp00_analysis.loader import load_evidence
from dp00_analysis.qa01 import derive_qa01, linear_percentile, nearest_rank


def test_t01_known_ftol_and_t02_failed_exclusion(raw_run):
    episode = load_evidence(raw_run).episodes[0]
    failed_events = tuple(e for e in episode.canonical_events if e["event"] != "EpisodeCompleted" and "UsefulOutcomeObserved" not in (e["event"] if isinstance(e["event"], dict) else {}))
    failed_events += (dict(episode.canonical_events[-1], event={"EpisodeFailed": {"reason": "EXECUTION_FAILURE"}}),)
    failed = replace(episode, provenance=dict(episode.provenance, run_id="run-failed"), canonical_events=failed_events)
    result = derive_qa01([episode, failed])["profile-z:A"]
    assert result["counts"] == {"eligible_count": 2, "successful_count": 1, "failed_count": 1}
    assert result["statistics"]["min"] == 1_000_000
    assert result["statistics"]["milliseconds"]["mean"] == 1.0


def test_explicit_percentile_estimators_and_small_n_difference():
    samples = [1, 2, 3, 4]
    assert nearest_rank(samples, .95) == 4
    assert linear_percentile(samples, .95) == 3.8499999999999996
