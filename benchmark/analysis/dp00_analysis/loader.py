"""Read-only strict loading for campaign and development raw layouts."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Callable

from .episodes import build_episode
from .models import EvidenceSet, ValidationIssue, ValidationReport
from .validation import (
    StrictValidationError,
    validate_campaign_provenance,
    validate_calibration_manifest,
    validate_canonical_event,
    validate_fixture_event,
    validate_model_call,
    validate_run_provenance,
    validate_runtime_scenario,
    validate_behavior_plan,
)


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def _json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as stream:
        return json.load(stream)


def _jsonl(path: Path, validator: Callable[[Any], dict[str, Any]]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, 1):
            if not line.strip():
                raise StrictValidationError(f"{path}:{line_number}: blank JSONL record")
            try:
                value = json.loads(line)
            except json.JSONDecodeError as error:
                raise StrictValidationError(f"{path}:{line_number}: {error}") from error
            rows.append(validator(value))
    return rows


def _catalog(root: Path, include_oracles: bool) -> tuple[dict[str, dict[str, Any]], dict[str, dict[str, Any]], dict[str, Any]]:
    index = _json(root / "benchmark/scenarios/pilot-v0/index.json")
    scenarios = {entry["scenario_id"]: validate_runtime_scenario(_json(root / entry["path"])) for entry in index["scenarios"]}
    plans_registry = _json(root / index["behavior_plan_registry_path"])
    plans = {p["behavior_plan_id"]: validate_behavior_plan(p) for p in plans_registry["plans"]}
    if include_oracles:
        oracles_registry = _json(root / index["oracle_registry_path"])
        oracles = {o["scenario_id"]: o for o in oracles_registry["oracles"]}
    else:
        oracles = {}
    coverage = {
        "v0.1": _json(
            root
            / "benchmark/contracts/pilot-v0-constraint-alternative-evidence-map.json"
        ),
        "v0.2": _json(
            root
            / "benchmark/contracts/pilot-v0.2-constraint-alternative-evidence-map.json"
        ),
    }
    return scenarios, plans, {"oracles": oracles, "coverage": coverage}


def load_evidence(raw_root: str | Path, *, include_oracles: bool = True) -> EvidenceSet:
    raw_root = Path(raw_root).resolve()
    repo = repository_root()
    scenarios, plans, evaluator = _catalog(repo, include_oracles)
    errors: list[ValidationIssue] = []
    warnings: list[ValidationIssue] = []
    campaign: dict[str, Any] | None = None
    calibration_manifest: dict[str, Any] | None = None
    campaign_path = raw_root / "campaign-provenance.json"
    calibration_path = raw_root / "calibration-manifest.json"
    try:
        if calibration_path.is_file():
            calibration_manifest = validate_calibration_manifest(_json(calibration_path))
        if campaign_path.is_file():
            campaign = validate_campaign_provenance(_json(campaign_path))
            run_dirs = sorted(p for profile in (raw_root / "profile-z", raw_root / "profile-c") if profile.is_dir() for p in profile.iterdir() if p.is_dir())
        elif (raw_root / "provenance.json").is_file():
            run_dirs = [raw_root]
        else:
            run_dirs = sorted(p.parent for p in raw_root.rglob("provenance.json"))
            if not run_dirs:
                raise StrictValidationError(f"no raw runs found under {raw_root}")
    except (OSError, json.JSONDecodeError, StrictValidationError) as error:
        return EvidenceSet(
            raw_root, campaign, (),
            ValidationReport((ValidationIssue("LOAD_ERROR", str(error), str(raw_root)),), ()),
            evaluator["coverage"]["v0.1"],
            calibration_manifest=calibration_manifest,
        )

    episodes = []
    for directory in run_dirs:
        try:
            provenance = validate_run_provenance(_json(directory / "provenance.json"))
            scenario = scenarios[provenance["scenario_id"]]
            plan = plans[provenance["semantic_behavior_plan_id"]]
            if provenance["scenario_version"] != scenario["scenario_version"]:
                raise StrictValidationError("run/scenario asset version mismatch")
            if provenance["semantic_behavior_plan_version"] != plan["behavior_plan_version"] or plan["scenario_id"] != provenance["scenario_id"]:
                raise StrictValidationError("run/behavior-plan asset version/ref mismatch")
            if scenario["provenance"]["canonical_event_schema_version"] != provenance["canonical_event_schema_version"]:
                if not (
                    provenance["pilot_corpus_version"] == "v0.1"
                    and provenance["canonical_event_schema_version"] == "canonical-event-v2"
                ):
                    raise StrictValidationError("scenario/run canonical schema mismatch")
            events = _jsonl(directory / "canonical-events.jsonl", validate_canonical_event)
            calls = _jsonl(directory / "model-calls.jsonl", validate_model_call)
            fixture_path = directory / "fixture-events.jsonl"
            fixtures = _jsonl(fixture_path, validate_fixture_event) if fixture_path.is_file() else []
            if provenance["event_count"] != len(events):
                raise StrictValidationError("provenance event_count mismatch")
            if campaign is not None:
                profile_id = directory.parent.name.removeprefix("profile-").upper()
                if provenance["campaign_id"] != campaign["campaign_id"]:
                    raise StrictValidationError("campaign/run campaign_id mismatch")
                if provenance["source_git_commit"] != campaign["source_git_commit"]:
                    raise StrictValidationError("campaign/run source commit mismatch")
                if (provenance["pilot_corpus_id"], provenance["pilot_corpus_version"]) != (campaign["pilot_corpus_id"], campaign["pilot_corpus_version"]):
                    raise StrictValidationError("campaign/run corpus mismatch")
                if provenance["latency_profile"]["profile_id"] != profile_id:
                    raise StrictValidationError("campaign directory/profile identity mismatch")
                campaign_profile = campaign["profile_sequence"][provenance["campaign_profile_sequence_index"]]
                if provenance["latency_profile"] != campaign_profile:
                    raise StrictValidationError("campaign/run latency profile mismatch")
                if not provenance["official"]:
                    raise StrictValidationError("campaign contains development run")
            elif provenance["official"]:
                raise StrictValidationError("official run must be loaded through its campaign root")
            oracle = evaluator["oracles"].get(provenance["scenario_id"]) if include_oracles else None
            episode = build_episode(directory, provenance, scenario, events, calls, fixtures, oracle)
            episodes.append(episode)
            errors.extend(episode.integrity_issues)
        except (OSError, KeyError, IndexError, json.JSONDecodeError, StrictValidationError) as error:
            errors.append(ValidationIssue("RUN_VALIDATION_ERROR", str(error), str(directory)))
    if campaign is not None and episodes:
        profiles_present = {e.provenance["latency_profile"]["profile_id"] for e in episodes}
        if profiles_present != {"Z", "C"}:
            errors.append(ValidationIssue("CAMPAIGN_PROFILE_COVERAGE", f"campaign profiles present: {sorted(profiles_present)}"))
        runtime_fields = ("rust_toolchain", "rustc_version", "cargo_version", "target", "build_profile", "tokio_resolved_version", "runtime_worker_policy", "cargo_lock_identity")
        identities = {tuple(e.provenance[field] for field in runtime_fields) for e in episodes}
        if len(identities) != 1:
            errors.append(ValidationIssue("CAMPAIGN_RUNTIME_IDENTITY_MISMATCH", "official Z/C runtime identity differs"))
    corpus_versions = {
        episode.provenance["pilot_corpus_version"] for episode in episodes
    }
    coverage_version = (
        next(iter(corpus_versions))
        if len(corpus_versions) == 1
        else (campaign or {}).get("pilot_corpus_version", "v0.1")
    )
    coverage = evaluator["coverage"].get(coverage_version)
    if coverage is None:
        errors.append(
            ValidationIssue(
                "COVERAGE_MANIFEST_VERSION_MISSING",
                f"no path-aware manifest for corpus {coverage_version}",
            )
        )
        coverage = evaluator["coverage"]["v0.1"]
    from .calibration import validate_calibration_schedule

    errors.extend(validate_calibration_schedule(calibration_manifest, episodes))
    return EvidenceSet(
        raw_root, campaign, tuple(episodes),
        ValidationReport(tuple(errors), tuple(warnings)), coverage,
        calibration_manifest=calibration_manifest,
    )
