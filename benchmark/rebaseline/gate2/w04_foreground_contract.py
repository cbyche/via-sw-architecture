#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

EXPECTED_CASES = [
    "TC-01.1",
    "TC-01.4",
    "TC-02.1",
    "TC-04.2",
    "TC-06.2",
    "TC-10.1",
]


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def validate_contract(contract):
    if contract.get("version") != "W01-FG-G2-v1":
        raise ValueError("unexpected W-01 foreground contract version")

    w01 = contract.get("w01", {})
    if w01.get("representative_metric") != "mean_of_case_p95_response_start_ms":
        raise ValueError("W-01 representative metric changed")
    if (w01.get("endpoint_start"), w01.get("endpoint_end")) != (
        "user_input_end",
        "first_meaningful_output",
    ):
        raise ValueError("W-01 endpoint changed")
    if w01.get("warmup_trials_per_stratum") != 10:
        raise ValueError("W-01 warmup count changed")
    if w01.get("scored_trials_per_stratum") != 100:
        raise ValueError("W-01 scored trial count changed")
    if w01.get("p95") != "nearest-rank":
        raise ValueError("W-01 p95 rule changed")
    if w01.get("macro_aggregation") != "equal_weight_mean_of_six_case_p95":
        raise ValueError("W-01 macro aggregation changed")
    if w01.get("timeout_ms") != 30000:
        raise ValueError("W-01 timeout changed")

    cases = w01.get("cases", [])
    ids = [case.get("case_id") for case in cases]
    if ids != EXPECTED_CASES:
        raise ValueError(f"W-01 foreground case set/order changed: {ids}")
    if len(set(ids)) != 6:
        raise ValueError("W-01 foreground case ids must be unique")
    modalities = {case["case_id"]: case.get("modality") for case in cases}
    if modalities.get("TC-01.4") != "text":
        raise ValueError("TC-01.4 must remain the Text foreground stratum")
    if any(
        modalities[case_id] != "voice"
        for case_id in EXPECTED_CASES
        if case_id != "TC-01.4"
    ):
        raise ValueError("non-Text W-01 foreground strata must remain voice")
    clarification = next(case for case in cases if case["case_id"] == "TC-06.2")
    if "clarification" not in clarification.get("output_contract", "").lower():
        raise ValueError("TC-06.2 W-01 endpoint must remain the appropriate clarification turn")
    if "W-05" not in clarification.get("output_contract", ""):
        raise ValueError("TC-06.2 must distinguish W-01 clarification latency from W-05 terminal outcome")

    w04 = contract.get("w04", {})
    if w04.get("representative_metric") != "foreground_macro_p95_ratio_4_to_1":
        raise ValueError("W-04 representative metric changed")
    if w04.get("background_task_counts") != [1, 4]:
        raise ValueError("W-04 background Task counts changed")
    if w04.get("background_update_period_ms") != 1000:
        raise ValueError("W-04 cadence changed")
    if w04.get("background_window_max_ms") != 30000:
        raise ValueError("W-04 background window changed")
    if w04.get("metric_eligible_in_hosted_ci") is not False:
        raise ValueError("hosted CI must remain metric-ineligible for W-04")
    required = {"L1_macro_p95_ms", "L4_macro_p95_ms", "ratio"}
    if set(w04.get("absolute_values_required", [])) != required:
        raise ValueError("W-04 must preserve L1, L4, and ratio")
    pairing = w04.get("pairing_rule", "")
    for marker in ("same candidate", "same W-01 case", "only active background Task count changes"):
        if marker not in pairing:
            raise ValueError(f"W-04 pairing rule missing {marker!r}")

    prohibitions = contract.get("prohibitions", [])
    if not any("pool" in rule.lower() for rule in prohibitions):
        raise ValueError("W-01 pooled-p95 prohibition missing")
    if not any("Measurement Freeze" in rule for rule in prohibitions):
        raise ValueError("pre-freeze scoring prohibition missing")

    return {
        "status": "PASS_W01_W04_FOREGROUND_CONTRACT",
        "w01_cases": ids,
        "background_task_counts": [1, 4],
        "candidate_observation": "NOT_RUN",
        "representative_metrics": {
            "W-01": "NOT_RUN",
            "W-04": "NOT_RUN",
        },
        "score": None,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", required=True)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    result = validate_contract(load_json(args.contract))
    if not args.validate_only:
        raise SystemExit("measurement execution is intentionally not implemented by this pre-freeze validator")
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
