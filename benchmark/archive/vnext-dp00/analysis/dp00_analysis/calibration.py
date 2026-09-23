"""Strict validation of the counterbalanced DP-00 calibration schedule."""

from __future__ import annotations

from collections import Counter, defaultdict
from typing import Any, Iterable

from .models import EpisodeEvidence, ValidationIssue
from .validation import CALIBRATION_PROTOCOL_VERSION, RUN_PROVENANCE_VERSION

FULL_PREWARM_CYCLES = 2
MEASURED_CYCLES = 16
PAIRS_PER_CYCLE = 48
EXECUTIONS_PER_CYCLE = 96
EXPECTED_PAIRS = 768
EXPECTED_EXECUTIONS = 1_536
EXPECTED_QA01_PAIRS = 192
MODE_ORDER_POLICY = "SCENARIO_ALTERNATIVE_PARITY_INVERT_V1"
ALTERNATIVE_ROTATION_POLICY = "DETERMINISTIC_CYCLIC_V1"
REPETITION_POLICY = "FIXED_16_MEASURED_CYCLES_V1"
ALTERNATIVES = ("A", "B", "C", "D")
SCENARIOS = tuple(f"P{number:02}" for number in range(1, 13))


def _issue(code: str, message: str) -> ValidationIssue:
    return ValidationIssue(code, message)


def validate_calibration_schedule(
    manifest: dict[str, Any] | None,
    episodes: Iterable[EpisodeEvidence],
) -> list[ValidationIssue]:
    rows = [episode.provenance for episode in episodes]
    protocol_rows = [
        row for row in rows
        if row["provenance_schema_version"] == RUN_PROVENANCE_VERSION
    ]
    if manifest is None:
        return (
            [_issue("CALIBRATION_MANIFEST_MISSING", "v4 calibration runs require a manifest")]
            if protocol_rows else []
        )
    issues: list[ValidationIssue] = []

    expected_manifest = {
        "calibration_protocol_version": CALIBRATION_PROTOCOL_VERSION,
        "full_prewarm_cycle_count": FULL_PREWARM_CYCLES,
        "completed_full_prewarm_cycle_count": FULL_PREWARM_CYCLES,
        "invocation_warmup_count": 1,
        "measured_cycle_count": MEASURED_CYCLES,
        "mode_order_policy": MODE_ORDER_POLICY,
        "alternative_rotation_policy": ALTERNATIVE_ROTATION_POLICY,
        "repetition_policy": REPETITION_POLICY,
        "adaptive_stopping": False,
        "expected_measured_execution_count": EXPECTED_EXECUTIONS,
        "completed_measured_execution_count": EXPECTED_EXECUTIONS,
        "expected_pair_count": EXPECTED_PAIRS,
        "completed_pair_count": EXPECTED_PAIRS,
        "expected_qa01_pair_count": EXPECTED_QA01_PAIRS,
    }
    for key, expected in expected_manifest.items():
        if manifest.get(key) != expected:
            issues.append(_issue(
                "CALIBRATION_PROTOCOL_MISMATCH",
                f"manifest {key}={manifest.get(key)!r}, expected {expected!r}",
            ))

    prewarm = manifest.get("prewarm_cycles", [])
    if [cycle.get("cycle_id") for cycle in prewarm] != ["W0", "W1"]:
        issues.append(_issue("CALIBRATION_PREWARM_INVALID", "prewarm cycle ids must be W0,W1"))
    for cycle in prewarm:
        covered_paths = cycle.get("covered_paths", [])
        observed_paths = {
            (path.get("scenario_id"), path.get("alternative"), path.get("instrumentation_mode"))
            for path in covered_paths
        }
        expected_paths = {
            (scenario, alternative, mode)
            for scenario in SCENARIOS
            for alternative in ALTERNATIVES
            for mode in ("CAPTURE", "MINIMAL")
        }
        if (
            cycle.get("completed") is not True
            or cycle.get("expected_path_count") != EXECUTIONS_PER_CYCLE
            or cycle.get("completed_path_count") != EXECUTIONS_PER_CYCLE
            or len(covered_paths) != EXECUTIONS_PER_CYCLE
            or observed_paths != expected_paths
        ):
            issues.append(_issue(
                "CALIBRATION_PREWARM_INVALID",
                f"incomplete or invalid full prewarm coverage for {cycle.get('cycle_id')!r}",
            ))

    if len(protocol_rows) != EXPECTED_EXECUTIONS:
        issues.append(_issue(
            "CALIBRATION_EXECUTION_COUNT",
            f"measured executions {len(protocol_rows)}, expected {EXPECTED_EXECUTIONS}",
        ))
        return issues
    if len(rows) != len(protocol_rows):
        issues.append(_issue(
            "CALIBRATION_PREWARM_LEAK",
            "calibration raw root contains non-v4 or non-measured observations",
        ))
    if any(not row["measurement_population"] for row in protocol_rows):
        issues.append(_issue(
            "CALIBRATION_PREWARM_LEAK",
            "prewarm observation entered the measured population",
        ))

    stable = {
        (
            row["calibration_id"], row["source_sha"],
            row["calibration_protocol_version"], row["warmup_count"],
            row["measured_repetition_count"], row["order_policy"],
        )
        for row in protocol_rows
    }
    expected_stable = {
        (
            manifest["calibration_id"], manifest["source_sha"],
            CALIBRATION_PROTOCOL_VERSION, 1, MEASURED_CYCLES,
            ALTERNATIVE_ROTATION_POLICY,
        )
    }
    if stable != expected_stable:
        issues.append(_issue(
            "CALIBRATION_PROVENANCE_MISMATCH",
            "run identity/protocol/source/repetition provenance is inconsistent",
        ))

    ordinals = [row["calibration_execution_ordinal"] for row in protocol_rows]
    if sorted(ordinals) != list(range(EXPECTED_EXECUTIONS)):
        issues.append(_issue(
            "CALIBRATION_EXECUTION_ORDINAL",
            "measured execution ordinals must be unique contiguous 0..1535",
        ))
        return issues
    ordered = sorted(protocol_rows, key=lambda row: row["calibration_execution_ordinal"])
    if manifest["measured_run_ids"] != [row["run_id"] for row in ordered]:
        issues.append(_issue(
            "CALIBRATION_MANIFEST_ORDER_MISMATCH",
            "manifest measured_run_ids does not match explicit execution ordinal order",
        ))

    pair_rows = list(zip(ordered[::2], ordered[1::2], strict=True))
    if len(pair_rows) != EXPECTED_PAIRS:
        issues.append(_issue("CALIBRATION_PAIR_COUNT", "expected 768 adjacent pairs"))
    for left, right in pair_rows:
        if (
            left["pair_id"] != right["pair_id"]
            or [left["mode_order_slot"], right["mode_order_slot"]] != [0, 1]
            or left["instrumentation_mode"] == right["instrumentation_mode"]
            or right["calibration_execution_ordinal"]
            != left["calibration_execution_ordinal"] + 1
        ):
            issues.append(_issue(
                "CALIBRATION_PAIR_ADJACENCY",
                f"invalid adjacent pair at ordinal {left['calibration_execution_ordinal']}",
            ))
            break

    first_members = [left for left, _ in pair_rows]
    by_cycle: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for row in first_members:
        by_cycle[row["order_cycle"]].append(row)
    if set(by_cycle) != set(range(MEASURED_CYCLES)):
        issues.append(_issue("CALIBRATION_CYCLE_COVERAGE", "expected measured cycles 0..15"))
        return issues

    pair_first_history: dict[tuple[str, str], list[str]] = defaultdict(list)
    position_history: dict[str, Counter[int]] = defaultdict(Counter)
    for cycle in range(MEASURED_CYCLES):
        cycle_rows = by_cycle[cycle]
        if len(cycle_rows) != PAIRS_PER_CYCLE:
            issues.append(_issue(
                "CALIBRATION_CYCLE_ACCOUNTING",
                f"cycle {cycle} has {len(cycle_rows)} pairs, expected 48",
            ))
            continue
        mode_counts = Counter(row["instrumentation_mode"] for row in cycle_rows)
        if mode_counts != Counter({"CAPTURE": 24, "MINIMAL": 24}):
            issues.append(_issue(
                "CALIBRATION_MODE_BALANCE",
                f"cycle {cycle} mode-order balance {dict(mode_counts)}",
            ))
        for alternative in ALTERNATIVES:
            counts = Counter(
                row["instrumentation_mode"] for row in cycle_rows
                if row["alternative"] == alternative
            )
            if counts != Counter({"CAPTURE": 6, "MINIMAL": 6}):
                issues.append(_issue(
                    "CALIBRATION_ALTERNATIVE_MODE_BALANCE",
                    f"cycle {cycle} alternative {alternative} balance {dict(counts)}",
                ))
        for scenario in SCENARIOS:
            counts = Counter(
                row["instrumentation_mode"] for row in cycle_rows
                if row["scenario_id"] == scenario
            )
            if counts != Counter({"CAPTURE": 2, "MINIMAL": 2}):
                issues.append(_issue(
                    "CALIBRATION_SCENARIO_MODE_BALANCE",
                    f"cycle {cycle} scenario {scenario} balance {dict(counts)}",
                ))
        for position in range(4):
            counts = Counter(
                row["instrumentation_mode"] for row in cycle_rows
                if row["order_slot"] == position
            )
            if counts != Counter({"CAPTURE": 6, "MINIMAL": 6}):
                issues.append(_issue(
                    "CALIBRATION_MODE_POSITION_COUPLING",
                    f"cycle {cycle} position {position} balance {dict(counts)}",
                ))
        for row in cycle_rows:
            scenario_ordinal = int(row["scenario_id"][1:]) - 1
            alternative_ordinal = ALTERNATIVES.index(row["alternative"])
            expected_first = (
                "CAPTURE"
                if (scenario_ordinal + alternative_ordinal + cycle) % 2 == 0
                else "MINIMAL"
            )
            expected_position = (alternative_ordinal - cycle) % 4
            expected_pair_id = (
                f"{manifest['calibration_id']}:cycle-{cycle}:repetition-{cycle}:"
                f"{row['scenario_id']}:{row['alternative']}"
            )
            if row["instrumentation_mode"] != expected_first:
                issues.append(_issue(
                    "CALIBRATION_MODE_ASSIGNMENT",
                    f"unexpected first mode for {expected_pair_id}",
                ))
            if row["order_slot"] != expected_position:
                issues.append(_issue(
                    "CALIBRATION_POSITION_ROTATION",
                    f"unexpected position for {expected_pair_id}",
                ))
            if (
                row["cycle_id"] != f"cycle-{cycle}"
                or row["repetition_id"] != f"repetition-{cycle}"
                or row["repetition_index"] != cycle
                or row["pair_id"] != expected_pair_id
            ):
                issues.append(_issue(
                    "CALIBRATION_EXPLICIT_IDENTITY",
                    f"inconsistent cycle/pair/repetition identity for {row['run_id']}",
                ))
            pair_first_history[(row["scenario_id"], row["alternative"])].append(
                row["instrumentation_mode"]
            )
            position_history[row["alternative"]][row["order_slot"]] += (
                1 if row["scenario_id"] == "P01" else 0
            )

    for identity, modes in pair_first_history.items():
        if len(modes) != 16 or any(left == right for left, right in zip(modes, modes[1:])):
            issues.append(_issue(
                "CALIBRATION_CYCLE_INVERSION",
                f"mode order does not invert for {identity}",
            ))
        if Counter(modes) != Counter({"CAPTURE": 8, "MINIMAL": 8}):
            issues.append(_issue(
                "CALIBRATION_FULL_RUN_MODE_BALANCE",
                f"full-run mode balance invalid for {identity}",
            ))
    for alternative in ALTERNATIVES:
        if position_history[alternative] != Counter({0: 4, 1: 4, 2: 4, 3: 4}):
            issues.append(_issue(
                "CALIBRATION_POSITION_ROTATION",
                f"position balance invalid for alternative {alternative}",
            ))
    return issues


def calibration_schedule_diagnostics(
    manifest: dict[str, Any] | None,
    episodes: Iterable[EpisodeEvidence],
) -> dict[str, Any]:
    rows = [episode.provenance for episode in episodes]
    if manifest is None:
        return {"present": False}
    first = [row for row in rows if row.get("mode_order_slot") == 0]
    return {
        "present": True,
        "protocol_version": manifest["calibration_protocol_version"],
        "full_prewarm_cycles": manifest["completed_full_prewarm_cycle_count"],
        "measured_cycles": len({row["cycle_id"] for row in rows}),
        "measured_executions": len(rows),
        "measured_pairs": len(first),
        "qa01_pairs": sum(row["scenario_id"] in {"P01", "P02", "P03"} for row in first),
        "adaptive_stopping": manifest["adaptive_stopping"],
        "mode_order_policy": manifest["mode_order_policy"],
        "alternative_rotation_policy": manifest["alternative_rotation_policy"],
        "repetition_policy": manifest["repetition_policy"],
    }
