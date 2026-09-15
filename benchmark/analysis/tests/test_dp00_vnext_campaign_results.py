import hashlib
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[3]
CAMPAIGN_ID = "dp00-vnext-qa-v1-campaign-v1"
RAW = ROOT / "results/raw/dp00-vnext-qa-v1" / CAMPAIGN_ID
DERIVED = ROOT / "results/derived/dp00-vnext-qa-v1" / CAMPAIGN_ID
REPORT = ROOT / "results/reports/dp00-vnext-qa-v1" / CAMPAIGN_ID
CANDIDATES = ("R1", "R3", "R1+@")
EXPECTED_COUNTS = {"QA-01": 150, "QA-02": 600, "QA-03": 400, "QA-04": 60, "QA-05": 60, "QA-06": 60, "QA-08": 200, "QA-09": 300, "QA-10": 200, "QA-11": 200, "QA-12": 200}


def _load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def _jsonl(path):
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]


def _sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_campaign_results_cover_every_registered_candidate_and_qa():
    summary = _load(DERIVED / "comparative-summary-data.json")
    assert summary["status"] == "COMPLETE_PRELIMINARY_EVIDENCE_FINAL_SELECTION_BLOCKED"
    assert set(summary["results"]) == set(CANDIDATES)
    assert summary["decision"]["outcome"] == "NO_QUALIFIED_WINNER"
    assert summary["decision"]["adr_proposed"] is False
    for candidate_id in CANDIDATES:
        assert set(summary["results"][candidate_id]) == {f"QA-{number:02d}" for number in range(1, 13)}


def test_raw_canonical_observations_have_exact_frozen_coverage():
    for candidate_id in CANDIDATES:
        for qa_id, count in EXPECTED_COUNTS.items():
            observations = _jsonl(RAW / candidate_id / qa_id / "canonical-observations.jsonl")
            executions = _jsonl(RAW / candidate_id / qa_id / "candidate-executions.jsonl")
            assert len(observations) == len(executions) == count
            identities = [item["observation_id"] for item in observations]
            assert len(set(identities)) == count
            assert all(item["qa_id"] == qa_id and item["evidence_mode"] == "semantic_replay" for item in observations)


def test_every_valid_result_envelope_uses_shared_provenance_and_frozen_schema_shape():
    required = set(_load(ROOT / "benchmark/schemas/qa-v1-result-envelope.schema.json")["required"])
    provenance = _load(RAW / "campaign-provenance.json")
    assert provenance["candidate_ids"] == list(CANDIDATES)
    assert provenance["network_or_cloud_calls"] == 0
    assert provenance["official_profile"] == "QWEN3_MEDIUM_REFERENCE"
    for candidate_id in CANDIDATES:
        for number in range(1, 13):
            qa_id = f"QA-{number:02d}"
            result = _load(DERIVED / candidate_id / f"{qa_id}-result-envelope.json")
            assert set(result) == required
            assert result["provenance"]["architecture_commit"] == provenance["architecture_commit"]
            assert result["provenance"]["alternative_id"] == candidate_id
            if qa_id != "QA-07":
                assert result["eligibility_status"] == "ELIGIBLE"
                assert result["raw_metric"] is not None
                assert result["score"] is not None


def test_input_artifact_hashes_match_and_candidates_equal_preregistration_commit():
    provenance = _load(RAW / "campaign-provenance.json")
    commit = provenance["registration_commit"]
    assert commit == provenance["architecture_commit"] == "9bb81ef3ff3831342f39437c52e039f9015a79a1"
    for relative, digest in provenance["artifact_sha256"].items():
        assert _sha256(ROOT / relative) == digest, relative
    registration = _load(ROOT / "benchmark/contracts/dp-vnext/evaluation/dp00-campaign-registration-v1.json")
    for candidate in registration["candidates"]:
        current = (ROOT / candidate["contract"]).read_bytes()
        frozen = subprocess.run(["git", "show", f"{commit}:{candidate['contract']}"], cwd=ROOT, check=True, capture_output=True).stdout
        assert current == frozen


def test_candidate_execution_streams_contain_no_hidden_evaluator_labels():
    forbidden = ("goal_oracle", "memory_oracle", "ground_truth", "expected_graph", "injected_fault_label")
    for path in RAW.glob("*/QA-*/candidate-executions.jsonl"):
        text = path.read_text(encoding="utf-8").lower()
        assert all(fragment not in text for fragment in forbidden), path


def test_comparable_qa01_dependencies_are_identical_and_tactic_is_not_posthoc():
    by_candidate = {
        candidate_id: {item["observation_id"]: item["measurement"] for item in _jsonl(RAW / candidate_id / "QA-01" / "canonical-observations.jsonl")}
        for candidate_id in CANDIDATES
    }
    for identity, r1 in by_candidate["R1"].items():
        r3 = by_candidate["R3"][identity]
        assert {key: value for key, value in r1.items() if key != "execution_path"} == {key: value for key, value in r3.items() if key != "execution_path"}
    for identity, r1_at in by_candidate["R1+@"].items():
        if r1_at["execution_path"] != "bounded-readonly-local":
            r1 = by_candidate["R1"][identity]
            assert {key: value for key, value in r1_at.items() if key != "execution_path"} == {key: value for key, value in r1.items() if key != "execution_path"}
        else:
            assert r1_at["voice_facts_audible_seconds"] < by_candidate["R1"][identity]["voice_facts_audible_seconds"]
    assert any(value["execution_path"] == "bounded-readonly-local" for value in by_candidate["R1+@"].values())


def test_qa07_has_no_fabricated_denominator_and_blocks_final_selection():
    summary = _load(DERIVED / "comparative-summary-data.json")
    for candidate_id in CANDIDATES:
        result = summary["results"][candidate_id]["QA-07"]
        assert result["raw_metric"] is None and result["score"] is None
        assert result["eligibility_status"] == "INCOMPLETE_POPULATION"
        assert summary["hard_gates"][candidate_id]["QA07-EVIDENCE-COMPLETENESS"] == "FAIL"
        gap = _load(RAW / candidate_id / "QA-07" / "evidence-gap.json")
        serialized = json.dumps(gap)
        assert "minimum_mandatory_bytes" not in serialized
        assert "candidate_resident_bytes" not in serialized
    assert summary["decision"]["blocker"] == "QA-07 lacks an evidence-anchored frozen minimum-mandatory-memory calibration."


def test_report_discloses_claim_limit_and_no_adr_is_created():
    report = (REPORT / "comparative-report.md").read_text(encoding="utf-8")
    assert "not production-device or Cloud-model performance" in report
    assert "NO QUALIFIED WINNER / MORE ARCHITECTURE WORK REQUIRED" in report
    assert "no ADR is proposed" in report
    assert not list((ROOT / "docs/adr").glob("*DP-00*vNext*"))
