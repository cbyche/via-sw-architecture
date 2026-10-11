#!/usr/bin/env python3
"""Static consistency checks for the pre-freeze VIA-DP-03 contract package."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CONTRACTS = ROOT / "docs/architecture/11-measurement/contracts"
INPUTS = CONTRACTS / "via-dp-03-fixture-inputs.json"
ORACLES = CONTRACTS / "via-dp-03-hidden-oracles.json"
PATTERNS = CONTRACTS / "via-dp-03-deictic-patterns.json"
PROFILE_A = CONTRACTS / "via-dp-03-a-streaming-asr-mock-profile.json"
PROFILE_B = CONTRACTS / "via-dp-03-b-s2s-mock-profile.json"
CHANGE_LEDGER = CONTRACTS / "via-dp-03-change-ledger.json"
FAULT_REGISTRY = CONTRACTS / "via-dp-03-fault-registry.json"
CATALOG = ROOT / "docs/architecture/11-measurement/test-case-catalog.md"


class ContractError(RuntimeError):
    pass


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def strip_longest_suffix(value: str, suffixes: list[str]) -> tuple[str, int]:
    for suffix in sorted(suffixes, key=len, reverse=True):
        if value.endswith(suffix) and len(value) > len(suffix):
            return value[: -len(suffix)], len(suffix)
    return value, 0


def detect_deictic_ranges(text: str, registry: dict) -> list[tuple[int, int]]:
    tokens = [(m.group(), m.start(), m.end()) for m in re.finditer(r"\S+", text)]
    suffixes = registry["particle_suffixes"]
    plurals = registry["optional_plural_markers"]
    standalone = {
        form
        for pattern in registry["standalone_patterns"]
        for form in pattern["forms"]
    }
    determiners = set(registry["determiner_noun_patterns"]["determiners"])
    nouns = set(registry["determiner_noun_patterns"]["nouns"])
    ranges: list[tuple[int, int]] = []

    for index, (token, start, _) in enumerate(tokens):
        base, _ = strip_longest_suffix(token, suffixes)
        if base in standalone:
            ranges.append((start, start + len(base)))

        if base not in determiners or index + 1 >= len(tokens):
            continue
        noun_token, noun_start, _ = tokens[index + 1]
        noun_base, _ = strip_longest_suffix(noun_token, suffixes)
        noun_stem, _ = strip_longest_suffix(noun_base, plurals)
        if noun_stem in nouns:
            ranges.append((start, noun_start + len(noun_stem)))

    return sorted(set(ranges))


def final_event(case: dict, alternative: str) -> dict:
    finals = [event for event in case["speech_events"][alternative] if event["kind"] == "final"]
    require(len(finals) == 1, f"{case['fixture_id']} {alternative}: expected exactly one final event")
    return finals[0]


def ranges(event: dict) -> list[tuple[int, int]]:
    return [
        (unit["text_start_char"], unit["text_end_char"])
        for unit in event["alignment_units"]
    ]


def validate() -> None:
    fixture_doc = load(INPUTS)
    oracle_doc = load(ORACLES)
    registry = load(PATTERNS)
    profile_a = load(PROFILE_A)
    profile_b = load(PROFILE_B)
    change_ledger = load(CHANGE_LEDGER)
    fault_registry = load(FAULT_REGISTRY)
    catalog = CATALOG.read_text(encoding="utf-8")

    require(fixture_doc["visibility"] == "CANDIDATE_VISIBLE", "fixture visibility must be candidate-visible")
    require(fixture_doc["oracle_fields_present"] is False, "candidate fixture must declare no oracle fields")
    require(oracle_doc["visibility"] == "EVALUATOR_ONLY", "oracle visibility must be evaluator-only")
    require(oracle_doc["candidate_access"] == "PROHIBITED", "candidate access to oracle must be prohibited")

    forbidden_keys = {"expected_fields", "expected_final_spans", "forbidden"}
    fixture_text = INPUTS.read_text(encoding="utf-8")
    for key in forbidden_keys:
        require(f'"{key}"' not in fixture_text, f"candidate fixture leaks evaluator key: {key}")

    fixtures = {case["fixture_id"]: case for case in fixture_doc["cases"]}
    oracles = {case["fixture_id"]: case for case in oracle_doc["cases"]}
    require(len(fixtures) == len(fixture_doc["cases"]), "duplicate fixture_id")
    require(len(oracles) == len(oracle_doc["cases"]), "duplicate oracle fixture_id")
    require(fixtures.keys() == oracles.keys(), "fixture and oracle case sets differ")

    for fixture_id, case in fixtures.items():
        require(case["source_case"] in catalog, f"{fixture_id}: source case absent from active catalog")
        ui_times = [event["at_ms"] for event in case["ui_events"]]
        require(ui_times == sorted(ui_times), f"{fixture_id}: UI events are not time-ordered")

        a_final = final_event(case, "A")
        b_final = final_event(case, "B")
        require(a_final["text"] == b_final["text"], f"{fixture_id}: A/B final transcript differs")
        require(ranges(a_final) == ranges(b_final), f"{fixture_id}: A/B final alignment differs")
        require(
            a_final["observed_at_ms"] == case["utterance_end_ms"] + 300,
            f"{fixture_id}: A primary final is not utterance_end + 300 ms",
        )
        require(
            b_final["observed_at_ms"] == case["utterance_end_ms"] + 300,
            f"{fixture_id}: B primary final is not utterance_end + 300 ms",
        )

        text = a_final["text"]
        for start, end in ranges(a_final):
            require(0 <= start < end <= len(text), f"{fixture_id}: invalid text range {start}:{end}")
        require(
            detect_deictic_ranges(text, registry) == ranges(a_final),
            f"{fixture_id}: detector ranges {detect_deictic_ranges(text, registry)} != final alignment {ranges(a_final)}",
        )

        for event in case["speech_events"]["A"]:
            if event["kind"] != "partial":
                continue
            latest_end = max(unit["end_ms"] for unit in event["alignment_units"])
            require(
                event["observed_at_ms"] == latest_end + 160,
                f"{fixture_id} {event['id']}: A partial is not referenced interval end + 160 ms",
            )

        oracle = oracles[fixture_id]
        oracle_ranges = [
            (span["text_start_char"], span["text_end_char"])
            for span in oracle["expected_final_spans"]
        ]
        require(oracle_ranges == ranges(a_final), f"{fixture_id}: oracle span ranges differ from final event")
        require(
            oracle["expected_fields"]["input.final_text"] == text,
            f"{fixture_id}: oracle final text differs from source event",
        )

    a_primary = next(
        profile for profile in profile_a["timing_profiles"] if profile["timing_profile_id"] == profile_a["primary_timing_profile_id"]
    )
    b_primary = next(
        profile for profile in profile_b["timing_profiles"] if profile["timing_profile_id"] == profile_b["primary_timing_profile_id"]
    )
    a_final_delay = next(event["scheduled_offset_ms"] for event in a_primary["events"] if event["event_name"] == "final_alignment_observed")
    b_final_delay = next(event["scheduled_offset_ms"] for event in b_primary["events"] if event["event_name"] == "time_aligned_final_observed")
    require(a_final_delay == b_final_delay == 300, "A/B primary final-arrival budgets must both be 300 ms")

    expected_changes = {"M-01", "M-07", "C-03", "E-01", "E-02", "E-03"}
    for alternative, candidate in change_ledger["candidates"].items():
        elements = {element["id"]: element for element in candidate["baseline_elements"]}
        require(len(elements) == len(candidate["baseline_elements"]), f"{alternative}: duplicate Architecture Element ID")
        require(
            {element["type"] for element in elements.values()} <= {"C", "I", "S", "D"},
            f"{alternative}: invalid Architecture Element type",
        )
        changes = {change["change_id"]: change for change in candidate["changes"]}
        require(changes.keys() == expected_changes, f"{alternative}: QA-29 change set differs from frozen draft")
        for change_id, change in changes.items():
            groups = [change["modified"], change["added"], change["removed"]]
            touched = [element_id for group in groups for element_id in group]
            require(len(touched) == len(set(touched)), f"{alternative} {change_id}: duplicate element across change groups")
            require(
                set(change["modified"] + change["removed"]) <= elements.keys(),
                f"{alternative} {change_id}: modified or removed element absent from baseline",
            )
            require(
                not (set(change["added"]) & elements.keys()),
                f"{alternative} {change_id}: added element already exists in baseline",
            )
            require(change["function_regression"] == ["SP-01", "SP-02", "SP-04"], f"{alternative} {change_id}: regression spine differs")

    unit_ids = {unit["id"] for unit in fault_registry["user_visible_units"]}
    require(len(unit_ids) == len(fault_registry["user_visible_units"]), "duplicate user-visible fault unit")
    require(fault_registry["repetitions_per_fault_per_candidate"] == 10, "QA-39 repetition must be 10")
    require(fault_registry["recovery_deadline_ms"] == 5000, "QA-39 recovery deadline must be 5000 ms")
    fault_ids = {fault["fault_id"] for fault in fault_registry["faults"]}
    require(fault_ids == {f"DP03-F-0{index}" for index in range(1, 7)}, "DP-03 fault registry must contain F-01 through F-06")
    for fault in fault_registry["faults"]:
        for alternative in ("A", "B"):
            closure = fault["necessary_dependency_closure"][alternative]
            require(len(closure) == len(set(closure)), f"{fault['fault_id']} {alternative}: duplicate closure unit")
            require(set(closure) <= unit_ids, f"{fault['fault_id']} {alternative}: unknown closure unit")


def main() -> int:
    try:
        validate()
    except (ContractError, KeyError, TypeError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    print("PASS: VIA-DP-03 pre-freeze fixture, oracle, detector, and timing contracts are consistent")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
