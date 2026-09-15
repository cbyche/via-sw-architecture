import json
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "benchmark/contracts/dp-vnext/dp-catalog-v1.json"
SCHEMA_PATH = ROOT / "benchmark/schemas/dp-vnext-catalog-v1.schema.json"
CANDIDATE_PATHS = [
    ROOT / "benchmark/contracts/dp-vnext/dp00-r1-candidate-v1.json",
    ROOT / "benchmark/contracts/dp-vnext/dp00-r3-candidate-v1.json",
    ROOT / "benchmark/contracts/dp-vnext/dp00-r1-readonly-fastpath-tactic-v1.json",
]
ACTIVE_DOCS = [
    ROOT / "README.md",
    ROOT / "docs/architecture/system-overview.md",
    ROOT / "docs/architecture/qa-dp-traceability.md",
    ROOT / "docs/architecture/decision-points/catalog.md",
    ROOT / "docs/architecture/decision-points/vnext/README.md",
    ROOT / "docs/architecture/decision-points/vnext/DP-00-primary-reasoning-execution-boundary.md",
    ROOT / "docs/evaluation/README.md",
    ROOT / "docs/evaluation/dp00-vnext-evaluation-protocol-v1.md",
]
LEGACY_DP_DOCS = [
    ROOT / "docs/architecture/decision-points/DP-00-primary-execution-boundary.md",
    ROOT / "docs/architecture/decision-points/DP-00-executable-architecture-spec.md",
    ROOT / "docs/architecture/decision-points/DP-03-capability-placement-boundary.md",
    ROOT / "docs/architecture/decision-points/DP-13-execution-state-and-recovery-authority.md",
    ROOT / "docs/architecture/decision-points/DP-14-execution-lifecycle-control-boundary.md",
    ROOT / "docs/architecture/decision-points/DP-15-failure-containment-boundary.md",
    ROOT / "docs/architecture/decision-points/DP-16-local-execution-hosting-boundary.md",
]


def _load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def _catalog():
    return _load(CATALOG_PATH)


def _qa_ids():
    contract = _load(ROOT / "benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json")
    return {qa["qa_id"] for qa in contract["quality_attributes"]}


def _walk_keys(value):
    if isinstance(value, dict):
        for key, child in value.items():
            yield key
            yield from _walk_keys(child)
    elif isinstance(value, list):
        for child in value:
            yield from _walk_keys(child)


def test_catalog_and_schema_are_machine_readable_and_structurally_aligned():
    catalog = _catalog()
    schema = _load(SCHEMA_PATH)
    required = set(schema["$defs"]["decision_point"]["required"])
    assert catalog["catalog_id"] == schema["properties"]["catalog_id"]["const"]
    assert catalog["status"] == "ACTIVE"
    assert len(catalog["decision_points"]) == 10
    assert len({dp["dp_id"] for dp in catalog["decision_points"]}) == 10
    for dp in catalog["decision_points"]:
        assert required <= set(dp)
        assert dp["status"] == "active"
        assert len(dp["alternatives"]) >= 2
        assert len(dp["structural_discriminator"]) >= 20
        assert dp["excluded_mechanisms"]
        assert dp["legacy_mapping"]


def test_exactly_one_active_dp00_has_r1_r3_base_families_and_r1_at_tactic():
    matches = [dp for dp in _catalog()["decision_points"] if dp["dp_id"] == "DP-00"]
    assert len(matches) == 1
    alternatives = matches[0]["alternatives"]
    base_ids = {item["alternative_id"] for item in alternatives if item["classification"] == "base_family"}
    tactic = [item for item in alternatives if item["classification"] == "tactic_realization"]
    assert base_ids == {"R1", "R3"}
    assert tactic == [{"alternative_id": "R1+@", "name": "R1 with bounded deterministic read-only local tactic", "classification": "tactic_realization", "inherits": "R1"}]
    assert "D" not in base_ids


def test_dp00_candidates_do_not_preselect_dp01_turn_authority():
    invariant = "DP-00 does not select committed logical UserTurn authority; that authority remains unresolved until DP-01."
    r1 = _load(CANDIDATE_PATHS[0])
    r3 = _load(CANDIDATE_PATHS[1])
    assert invariant in r1["invariants"]
    assert invariant in r3["invariants"]
    assert "canonical input and turn handling" not in r1["via_owns"]
    active_dp00 = (ROOT / "docs/architecture/decision-points/vnext/DP-00-primary-reasoning-execution-boundary.md").read_text(encoding="utf-8")
    assert invariant in active_dp00


def test_all_active_dps_resolve_primary_qas_and_evaluate_all_twelve():
    catalog = _catalog()
    valid = _qa_ids()
    assert set(catalog["all_qa_ids"]) == valid
    assert len(valid) == 12
    for dp in catalog["decision_points"]:
        assert set(dp["primary_qas"]) <= valid
        assert dp["all_qas_evaluated"] is True


def test_no_active_document_preselects_r1_at_or_presents_abcd_as_current():
    policy = "No base architecture family is preselected. DP-00 selection will be driven first by measured QA-01 and QA-02 results under QA Evaluation Contract v1"
    joined = "\n".join(path.read_text(encoding="utf-8") for path in ACTIVE_DOCS)
    assert policy in joined
    assert not re.search(r"R1\+@.{0,80}(intended|default|preselected|preferred).{0,30}(winner|architecture)", joined, re.IGNORECASE)
    assert "DP-00 compares four Integrated Product execution topologies" not in joined
    assert "A/B/C/D are compared" not in joined


def test_legacy_abcd_documents_and_result_identities_remain_traceable():
    for path in LEGACY_DP_DOCS:
        assert path.exists()
        assert "LEGACY / HISTORICAL DECISION RECORD" in path.read_text(encoding="utf-8")[:1000]
    migration = (ROOT / "docs/architecture/decision-points/vnext/MIGRATION.md").read_text(encoding="utf-8")
    for identity in ("A — Thin VIA", "B — ARGO-centric", "C — Hybrid VIA Fast Path", "D — Adaptive Per-turn"):
        assert identity in migration
    registry = _load(ROOT / "benchmark/contracts/qa-v1/historical-evidence-registry-v1.json")
    assert registry["artifacts"]
    assert all((ROOT / artifact["path"]).exists() for artifact in registry["artifacts"])


def test_candidate_contracts_are_unmeasured_offline_specs_with_full_qa_binding():
    valid = _qa_ids()
    qa_schema = _load(ROOT / "benchmark/schemas/qa-evaluation-contract-v1.schema.json")
    binding_schema = qa_schema["$defs"]["decisionPointEvaluationBinding"]
    required_binding = set(binding_schema["required"])
    forbidden_result_keys = {"result", "results", "score", "scores", "raw_metric", "measured_value", "winner"}
    for path in CANDIDATE_PATHS:
        candidate = _load(path)
        assert candidate["evaluation_status"] == "NOT_EVALUATED"
        assert candidate["all_qas_evaluated"] is True
        assert set(candidate["qa_v1_binding"]) == required_binding
        assert binding_schema["additionalProperties"] is False
        assert candidate["qa_v1_binding"] == {
            "dp_id": "DP-00",
            "qa_contract_id": "qa-evaluation-contract-v1",
            "reference_environment_id": "via-reference-environment-v1",
            "corpus_manifest_id": "qa-v1-corpus-manifest-v1",
            "primary_qa_ids": ["QA-01", "QA-02", "QA-04", "QA-05"],
        }
        assert set(candidate["qa_v1_binding"]["primary_qa_ids"]) == {"QA-01", "QA-02", "QA-04", "QA-05"}
        assert set(candidate["qa_v1_binding"]["primary_qa_ids"]) <= valid
        assert not (set(_walk_keys(candidate)) & forbidden_result_keys)
        serialized = path.read_text(encoding="utf-8").lower()
        assert "api_key" not in serialized
        assert "https://" not in serialized
    r3 = _load(CANDIDATE_PATHS[1])
    assert all("ARGO" not in responsibility for responsibility in r3["primary_agent_runtime_owns"])
    tactic = _load(CANDIDATE_PATHS[2])
    assert tactic["contract_type"] == "tactic_realization"
    assert tactic["inherits_candidate"] == "R1"
    assert {"state-changing user or domain action", "arbitrary tool selection", "open-ended planning or planning loop", "durable workflow", "independent Agent execution state"} <= set(tactic["must_reject"])


def test_protocol_freezes_fair_comparison_and_decision_order_without_scores():
    text = (ROOT / "docs/evaluation/dp00-vnext-evaluation-protocol-v1.md").read_text(encoding="utf-8")
    for phrase in ("same frozen QA-v1 corpus", "same Reference Environment", "same semantic capability envelope", "same downstream Agent capability profiles", "same Voice and Text obligations", "same fault and event schedules", "same device and resource workload"):
        assert phrase in text
    assert "Do not collapse them into an arbitrary weighted sum" in text
    assert "Do not inject candidate-specific error probabilities" in text
    assert "NO CAMPAIGN RUN — NO CANDIDATE RESULTS" in text
    assert "do not invoke Cloud or model APIs" in text
