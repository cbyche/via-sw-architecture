from dp00_analysis.loader import load_evidence
import json

from conftest import model_call, provenance, success_events, write_run


def test_strict_loader_reads_correlated_run(raw_run):
    evidence = load_evidence(raw_run)
    assert evidence.validation.valid, evidence.validation.errors
    assert len(evidence.episodes) == 1
    assert evidence.episodes[0].actual.execution_routes[0]["identity"] == "EXECUTOR_DIRECT:ARGO"


def test_loader_rejects_unsupported_schema(raw_run):
    path = raw_run / "canonical-events.jsonl"
    path.write_text(path.read_text().replace("canonical-event-v3", "canonical-event-v1", 1))
    evidence = load_evidence(raw_run)
    assert not evidence.validation.valid
    assert "unsupported schema version" in evidence.validation.errors[0].message


def test_official_campaign_loads_z_and_c_without_mixing(tmp_path):
    root = tmp_path / "campaign"
    campaign = {
        "provenance_schema_version":"dp00-pilot-campaign-provenance-v1", "campaign_id":"campaign-1",
        "source_git_commit":"test-sha", "initial_working_tree_clean":True,
        "pilot_corpus_id":"DP00-PILOT-V0", "pilot_corpus_version":"v0.1",
        "profile_sequence":[
            {"profile_id":"Z","version":"pilot-z-v0","model_delay_micros":0,"agent_delay_micros":0,"tool_delay_micros":0,"calibration_status":"PILOT_TBD"},
            {"profile_id":"C","version":"pilot-c-v0","model_delay_micros":1,"agent_delay_micros":1,"tool_delay_micros":1,"calibration_status":"PILOT_TBD"}],
        "campaign_configuration":{"alternatives":["A"],"scenario_ids":["P01"],"warmup_count":0,"measured_repetition_count":1,"order_policy":"COUNTERBALANCED_ROTATION_V1","instrumentation_mode":"CAPTURE"},
        "runner_version":"dp00-pilot-runner-v0"}
    root.mkdir()
    (root / "campaign-provenance.json").write_text(json.dumps(campaign))
    for index, profile in enumerate(("Z", "C")):
        events = success_events()
        calls = [model_call()]
        run_id = f"run-{profile.lower()}"
        for row in events + calls:
            row["run_id"] = run_id
        prov = provenance(profile=profile, official=True, event_count=len(events))
        prov.update(run_id=run_id, campaign_profile_sequence_index=index)
        prov["latency_profile"] = campaign["profile_sequence"][index]
        write_run(root / f"profile-{profile.lower()}" / run_id, events=events, calls=calls, prov=prov)
    evidence = load_evidence(root)
    assert evidence.validation.valid, evidence.validation.errors
    assert {e.provenance["latency_profile"]["profile_id"] for e in evidence.episodes} == {"Z", "C"}
