"""Deterministic QA-03 change-containment classification.

The evaluator consumes frozen scenario, role-map, and execution-manifest JSON.
It deliberately does not infer architecture semantics from line counts.
Architecture resistance to a valid request is NOT_CONTAINED; only experiment
contract failure is INVALID_EXPERIMENT. Comparative CCR uses one scenario-wide
denominator for A/B/C/D.
"""

from __future__ import annotations

import argparse
import json
from fnmatch import fnmatchcase
from pathlib import Path
from typing import Any


FINAL_CLASSES = {"CONTAINED", "NOT_CONTAINED", "INVALID_EXPERIMENT", "INCONCLUSIVE"}
ALTERNATIVES = ("A", "B", "C", "D")
VALID_PRIMARY_CLASSES = {"CONTAINED", "NOT_CONTAINED"}
NON_SCORING_CHANGE_CLASSES = {
    "TEST",
    "DOCUMENTATION",
    "EVIDENCE",
    "FIXTURE_ONLY",
    "FORMATTING_ONLY",
    "GENERATED_OUTPUT",
}
EXPECTED_CONTRACT_HASHES = {
    "catalog_git_blob": "d4de658fbcf096d0a9aa3269f751bdf4796ae316",
    "role_map_git_blob": "ecc77066c3540191d74c707bdc7e6bb5dc3f4e9f",
    "baseline_regression_git_blob": "ac8dabf096ca14326061f10e003fe1dfdcc3946d",
}


class ContractError(ValueError):
    """Raised when a frozen QA-03 contract is internally inconsistent."""


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def validate_contracts(catalog: dict[str, Any], role_map: dict[str, Any]) -> None:
    """Validate prospective catalog and A/B/C/D mapping completeness."""
    _require(catalog.get("protocol_version") == "dp00-qa03-flexibility-protocol-v1", "protocol version")
    _require(catalog.get("catalog_version") == "dp00-qa03-evolution-catalog-v1", "catalog version")
    _require(role_map.get("role_map_version") == "dp00-qa03-role-map-v1", "role-map version")
    vocabulary = set(role_map.get("role_vocabulary", []))
    _require(bool(vocabulary), "empty role vocabulary")
    alternatives = role_map.get("alternatives", {})
    _require(set(alternatives) == {"A", "B", "C", "D"}, "A/B/C/D mappings required")
    shared = role_map.get("shared", [])
    for entry in shared:
        _require(bool(entry.get("patterns")), "shared mapping without pattern")
        _require(set(entry.get("roles", [])) <= vocabulary, "unknown shared role")
    for alternative, entries in alternatives.items():
        _require(bool(entries), f"empty mapping for {alternative}")
        for entry in entries:
            _require(bool(entry.get("patterns")), f"mapping without pattern for {alternative}")
            _require(set(entry.get("roles", [])) <= vocabulary, f"unknown role in {alternative}")
        mapped = {role for entry in shared + entries for role in entry.get("roles", [])}
        absent = set(role_map.get("absent_or_external_roles", {}).get(alternative, {}))
        _require(mapped | absent == vocabulary, f"unresolved role vocabulary for {alternative}")

    scenarios = catalog.get("scenarios", [])
    _require(5 <= len(scenarios) <= 10, "scenario count must be 5..10")
    ids = [scenario.get("id") for scenario in scenarios]
    _require(len(ids) == len(set(ids)), "duplicate scenario id")
    _require({scenario.get("category") for scenario in scenarios} == {"E1", "E2", "E3", "E4", "E5"}, "E1..E5 coverage")
    for scenario in scenarios:
        expected = set(scenario.get("expected_change_roles", []))
        forbidden = set(scenario.get("forbidden_change_roles", []))
        _require(bool(expected), f"{scenario.get('id')}: expected roles required")
        _require(expected <= vocabulary, f"{scenario.get('id')}: unknown expected role")
        _require(forbidden <= vocabulary, f"{scenario.get('id')}: unknown forbidden role")
        _require(not expected & forbidden, f"{scenario.get('id')}: expected/forbidden overlap")
        _require(bool(scenario.get("acceptance_tests")), f"{scenario.get('id')}: acceptance tests required")
        _require(bool(scenario.get("regression_tests")), f"{scenario.get('id')}: regression tests required")


def roles_for_path(path: str, alternative: str, role_map: dict[str, Any]) -> set[str]:
    """Resolve all candidate roles for a path using only the frozen mapping."""
    entries = role_map.get("shared", []) + role_map["alternatives"][alternative]
    roles: set[str] = set()
    for entry in entries:
        if any(fnmatchcase(path, pattern) for pattern in entry["patterns"]):
            roles.update(entry["roles"])
    return roles


def new_regressions(baseline: dict[str, Any], observed: dict[str, Any]) -> list[str]:
    """Return new/worsened required failures relative to a frozen baseline signature."""
    baseline_failures = set(baseline.get("failed_required_ids", []))
    observed_failures = set(observed.get("failed_required_ids", []))
    introduced = observed_failures - baseline_failures
    worsened = set(observed.get("worsened_required_ids", []))
    previously_passing_failed = set(observed.get("previously_passing_failed_ids", []))
    return sorted(introduced | worsened | previously_passing_failed)


def classify_run(
    manifest: dict[str, Any],
    scenario: dict[str, Any],
    role_map: dict[str, Any],
) -> dict[str, Any]:
    """Classify one Alternative x Scenario attempt under the binary CCR contract."""
    invalid_reasons: list[str] = []
    for key, expected in (
        ("protocol_version", "dp00-qa03-flexibility-protocol-v1"),
        ("catalog_version", "dp00-qa03-evolution-catalog-v1"),
        ("role_map_version", "dp00-qa03-role-map-v1"),
        ("baseline_git_commit", "d4059eccd2883b3029b1e8c94fc86d092a5041f8"),
    ):
        if manifest.get(key) != expected:
            invalid_reasons.append(f"{key}_mismatch")
    if manifest.get("scenario_id") != scenario.get("id"):
        invalid_reasons.append("scenario_id_mismatch")
    if manifest.get("alternative") not in {"A", "B", "C", "D"}:
        invalid_reasons.append("invalid_alternative")
    if not manifest.get("isolated_from_baseline", False):
        invalid_reasons.append("isolation_not_proven")
    if not manifest.get("frozen_before_implementation", False):
        invalid_reasons.append("prospective_freeze_not_proven")
    if not manifest.get("contract_artifact_hashes_match", False):
        invalid_reasons.append("frozen_contract_drift")
    if manifest.get("contract_hashes") != EXPECTED_CONTRACT_HASHES:
        invalid_reasons.append("frozen_contract_hash_mismatch")
    if not manifest.get("equal_scenario_semantics", False):
        invalid_reasons.append("unequal_scenario_semantics")
    if not manifest.get("alternative_definition_preserved", False):
        invalid_reasons.append("alternative_definition_changed")
    if invalid_reasons:
        return _result("INVALID_EXPERIMENT", invalid_reasons, [], [], [])

    if not manifest.get("evidence_complete", False):
        return _result("INCONCLUSIVE", ["incomplete_evidence"], [], [], [])

    alternative = manifest["alternative"]
    expected_roles = set(scenario["expected_change_roles"])
    changed_roles: set[str] = set()
    unexpected: set[str] = set()
    ambiguous: list[str] = []
    for change in manifest.get("changes", []):
        change_class = change.get("change_class")
        if change_class in NON_SCORING_CHANGE_CLASSES:
            continue
        if change_class not in {"PRODUCTION_SOURCE", "CONTRACT_SCHEMA", "CONFIGURATION", "REGISTRATION", "GENERATOR_SOURCE"}:
            ambiguous.append(change.get("path", "<missing>"))
            continue
        candidate_roles = roles_for_path(change["path"], alternative, role_map)
        asserted_roles = set(change.get("asserted_roles", []))
        if asserted_roles and not asserted_roles <= candidate_roles:
            ambiguous.append(change.get("path", "<missing>"))
            continue
        if len(candidate_roles) > 1 and not asserted_roles:
            ambiguous.append(change.get("path", "<missing>"))
            continue
        roles = asserted_roles or candidate_roles
        if not candidate_roles:
            unexpected.add("UNMAPPED_ARCHITECTURE_CHANGE")
        changed_roles.update(roles)
        unexpected.update(roles - expected_roles)

    if ambiguous:
        return _result("INCONCLUSIVE", ["ambiguous_change_class"], sorted(changed_roles), sorted(unexpected), ambiguous)

    regression_failures = new_regressions(
        manifest.get("baseline_regression", {}), manifest.get("observed_regression", {})
    )
    reasons: list[str] = []
    if not manifest.get("acceptance", {}).get("pass", False):
        reasons.append("acceptance_failed")
    if regression_failures:
        reasons.append("new_or_worsened_regression")
    if unexpected:
        reasons.append("out_of_area_propagation")
    classification = "NOT_CONTAINED" if reasons else "CONTAINED"
    return _result(classification, reasons, sorted(changed_roles), sorted(unexpected), regression_failures)


def calculate_ccr(results: list[dict[str, Any]]) -> dict[str, Any]:
    """Calculate a single-alternative diagnostic CCR.

    Primary comparative reporting must use ``calculate_comparative_ccr`` so
    A/B/C/D cannot acquire asymmetric denominators.
    """
    valid = [r for r in results if r["classification"] in {"CONTAINED", "NOT_CONTAINED"}]
    contained = sum(r["classification"] == "CONTAINED" for r in valid)
    return {
        "contained": contained,
        "valid_evaluated": len(valid),
        "excluded": len(results) - len(valid),
        "ccr_percent": None if not valid else contained / len(valid) * 100.0,
    }


def calculate_comparative_ccr(
    cells: list[dict[str, Any]], catalog: dict[str, Any]
) -> dict[str, Any]:
    """Calculate A/B/C/D CCR using one scenario-wide primary denominator.

    A scenario enters S_primary only when exactly one complete, valid result is
    present for every alternative. One INVALID_EXPERIMENT, INCONCLUSIVE,
    missing, or duplicate cell excludes that scenario symmetrically.
    """
    scenarios = {item["id"]: item for item in catalog["scenarios"]}
    grouped: dict[str, dict[str, list[dict[str, Any]]]] = {
        scenario_id: {alternative: [] for alternative in ALTERNATIVES}
        for scenario_id in scenarios
    }
    unknown_cells: list[dict[str, Any]] = []
    for cell in cells:
        scenario_id = cell.get("scenario_id")
        alternative = cell.get("alternative")
        if scenario_id not in grouped or alternative not in ALTERNATIVES:
            unknown_cells.append(cell)
            continue
        grouped[scenario_id][alternative].append(cell)

    primary: list[str] = []
    exclusions: list[dict[str, Any]] = []
    for scenario_id, by_alternative in grouped.items():
        invalid_cells: list[dict[str, str]] = []
        for alternative in ALTERNATIVES:
            attempts = by_alternative[alternative]
            if len(attempts) != 1:
                invalid_cells.append({
                    "alternative": alternative,
                    "classification": "MISSING" if not attempts else "DUPLICATE",
                })
                continue
            classification = attempts[0].get("classification")
            if classification not in VALID_PRIMARY_CLASSES:
                invalid_cells.append({
                    "alternative": alternative,
                    "classification": str(classification),
                })
        if invalid_cells:
            exclusions.append({
                "scenario_id": scenario_id,
                "excluded_for_all_alternatives": True,
                "cells": invalid_cells,
            })
        else:
            primary.append(scenario_id)

    denominator = len(primary)
    by_alternative_result: dict[str, dict[str, Any]] = {}
    for alternative in ALTERNATIVES:
        contained = sum(
            grouped[scenario_id][alternative][0]["classification"] == "CONTAINED"
            for scenario_id in primary
        )
        by_alternative_result[alternative] = {
            "contained": contained,
            "denominator": denominator,
            "ccr_percent": None if denominator == 0 else contained / denominator * 100.0,
        }

    covered_categories = sorted({scenarios[scenario_id]["category"] for scenario_id in primary})
    required_categories = ["E1", "E2", "E3", "E4", "E5"]
    taxonomy_complete = set(covered_categories) == set(required_categories)
    full_matrix = denominator == len(scenarios) == 5 and len(cells) == 20 and not unknown_cells
    if full_matrix and taxonomy_complete and not exclusions:
        campaign_status = "QA-03 EVALUATION COMPLETE"
    elif not taxonomy_complete:
        campaign_status = "QA-03 COMPARATIVE CAMPAIGN INCOMPLETE — TAXONOMY COVERAGE LOST"
    else:
        campaign_status = "QA-03 COMPARATIVE CAMPAIGN INCOMPLETE"

    return {
        "s_primary": primary,
        "common_denominator": denominator,
        "alternatives": by_alternative_result,
        "excluded_scenarios": exclusions,
        "covered_categories": covered_categories,
        "taxonomy_complete": taxonomy_complete,
        "unknown_cells": unknown_cells,
        "campaign_status": campaign_status,
    }


def _result(
    classification: str,
    reasons: list[str],
    changed_roles: list[str],
    unexpected_roles: list[str],
    details: list[str],
) -> dict[str, Any]:
    _require(classification in FINAL_CLASSES, "invalid classification")
    return {
        "classification": classification,
        "reasons": reasons,
        "changed_architecture_roles": changed_roles,
        "unexpected_changed_roles": unexpected_roles,
        "details": details,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Classify one frozen DP-00 QA-03 run")
    parser.add_argument("--catalog", type=Path, required=True)
    parser.add_argument("--role-map", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args(argv)
    catalog = json.loads(args.catalog.read_text())
    role_map = json.loads(args.role_map.read_text())
    manifest = json.loads(args.manifest.read_text())
    validate_contracts(catalog, role_map)
    matches = [item for item in catalog["scenarios"] if item["id"] == manifest.get("scenario_id")]
    if len(matches) != 1:
        raise ContractError("manifest scenario_id does not resolve exactly once")
    print(json.dumps(classify_run(manifest, matches[0], role_map), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
