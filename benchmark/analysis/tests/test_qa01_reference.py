import json
from pathlib import Path

from generate_qa01_reference_report import model_latency_ms, p95_ranking, realize_trace, simulate, validate_contracts, validate_supporting_contracts


ROOT = Path(__file__).resolve().parents[3]
CORPUS = json.loads((ROOT / "benchmark/qa01-reference-v1/scenarios.json").read_text())
TRACES = json.loads((ROOT / "benchmark/qa01-reference-v1/traces.json").read_text())
PROFILE = json.loads((ROOT / "benchmark/contracts/dp00-qa01-design-reference-latency-v1.json").read_text())
PLANS = json.loads((ROOT / "benchmark/qa01-reference-v1/behavior-plans.json").read_text())
ORACLES = json.loads((ROOT / "benchmark/qa01-reference-v1/oracles.json").read_text())


def test_corpus_and_defining_path_coverage():
    validation = validate_contracts(CORPUS, TRACES, PROFILE)
    assert validation["categories"] == {"F1": 12, "F2": 12, "F3": 12}
    assert validation["path_observations"]["C:VIA_FAST"] == 12
    assert validation["path_observations"]["D:VIA_FAST"] == 12
    for key, required in TRACES["coverage_gates"].items():
        assert validation["path_observations"][key] >= required
    validate_supporting_contracts(CORPUS, PLANS, ORACLES)


def test_same_effect_sample_is_candidate_neutral():
    scenario = CORPUS["scenarios"][0]
    model = PROFILE["models"][0]
    c_trace = TRACES["category_traces"]["F1"]["C"]["trace"]
    c_without_x = [item for item in c_trace if item != "X"]
    d_trace = TRACES["category_traces"]["F1"]["D"]["trace"]
    d_without_x = [item for item in d_trace if item != "X"]
    c_effect = realize_trace(scenario, c_trace, model, PROFILE, 17, 5, 1) - realize_trace(scenario, c_without_x, model, PROFILE, 17, 5, 1)
    d_effect = realize_trace(scenario, d_trace, model, PROFILE, 17, 5, 1) - realize_trace(scenario, d_without_x, model, PROFILE, 17, 5, 1)
    assert c_effect == d_effect


def test_fixed_seed_is_reproducible_with_small_realization_count():
    contract = dict(PROFILE, latency_realizations_per_scenario_alternative_profile=2)
    assert simulate(CORPUS, TRACES, contract) == simulate(CORPUS, TRACES, contract)


def test_model_profile_is_applied_consistently():
    for category in ("F1", "F2", "F3"):
        for alternative in "ABCD":
            assert not any(item.startswith("MODEL:") for item in TRACES["category_traces"][category][alternative]["trace"])
    assert PROFILE["model_fairness"].startswith("Within one profile every semantic generation")


def test_model_sample_has_no_candidate_specific_input():
    model = PROFILE["models"][1]
    args = (model, "EXECUTION_ROUTE_DECISION", 16, PROFILE["random_seed"], "QF2-01", 9, 0)
    assert model_latency_ms(*args) == model_latency_ms(*args)


def test_reported_p95_ties_are_not_artificially_ordered():
    values = {key: {"p95_ms": value} for key, value in {"A": 2.0, "B": 3.0, "C": 1.0, "D": 1.0}.items()}
    assert p95_ranking(values) == "C = D < A < B"


def test_two_stage_views_are_exact_aliases_without_reweighting():
    contract = dict(PROFILE, latency_realizations_per_scenario_alternative_profile=2)
    report = simulate(CORPUS, TRACES, contract, PLANS, ORACLES)
    views = report["architecture_views"]
    assert views["stage1"]["candidates"] == ["A0", "B0"]
    assert "D0" not in views["stage1"]["candidates"]
    assert views["stage2"]["candidates"] == ["C", "B1", "D1"]
    assert views["stage2"]["primary_population"] == "F1"
    for profile_id, source in report["primary"].items():
        assert views["stage1"]["profiles"][profile_id]["A0"]["overall"] == source["A"]["statistics"]
        assert views["stage1"]["profiles"][profile_id]["B0"]["overall"] == source["B"]["statistics"]
        assert views["stage2"]["profiles"][profile_id]["C"]["f1_primary"] == source["C"]["category_statistics"]["F1"]
        assert views["stage2"]["profiles"][profile_id]["B1"]["f1_primary"] == source["B"]["category_statistics"]["F1"]
        assert views["stage2"]["profiles"][profile_id]["D1"]["f1_primary"] == source["D"]["category_statistics"]["F1"]
        assert views["stage2"]["profiles"][profile_id]["B1"]["pooled_secondary"] == source["B"]["statistics"]
