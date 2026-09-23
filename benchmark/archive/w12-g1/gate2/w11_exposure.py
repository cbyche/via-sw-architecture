#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

FROZEN_WORKLOADS = {
    "TC-02.1", "TC-02.2", "TC-02.3", "TC-05.1",
    "TC-07.2", "TC-10.1", "TC-16.4", "TC-17.2",
}


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def validate_oracle(oracle):
    if oracle.get("version") != "W11-G2-v1":
        raise ValueError("unexpected W-11 oracle version")
    metric = oracle.get("metric", {})
    if metric.get("denominator") != 20:
        raise ValueError("W-11 denominator must remain 20")
    if metric.get("formula") != "100 * whole_workload_union_exposed_units / 20":
        raise ValueError("W-11 formula changed")
    if metric.get("target_pct") != 25.0:
        raise ValueError("W-11 target must remain 25 percent")

    workloads = set(oracle.get("workloads", []))
    if workloads != FROZEN_WORKLOADS:
        raise ValueError(f"W-11 workload set changed: {sorted(workloads)}")

    units = oracle.get("protected_units", [])
    if len(units) != 20:
        raise ValueError("W-11 requires exactly 20 protected units")
    unit_ids = [unit.get("id") for unit in units]
    if len(set(unit_ids)) != 20 or any(not unit_id for unit_id in unit_ids):
        raise ValueError("W-11 protected unit ids must be unique and non-empty")

    covered = set()
    for unit in units:
        relevant = set(unit.get("relevant_workloads", []))
        if not relevant or not relevant <= FROZEN_WORKLOADS:
            raise ValueError(f"invalid workload mapping for {unit['id']}")
        if not unit.get("semantic_unit"):
            raise ValueError(f"missing semantic unit for {unit['id']}")
        if not unit.get("evidence_literals"):
            raise ValueError(f"missing generator evidence for {unit['id']}")
        covered |= relevant
    if covered != FROZEN_WORKLOADS:
        raise ValueError("every frozen W-11 workload must have protected units")
    return {unit["id"] for unit in units}


def _score(exposure_pct, bands):
    for band in bands:
        maximum = band.get("max_pct")
        if maximum is None or exposure_pct <= maximum:
            return band["score"]
    raise AssertionError("unreachable W-11 score band")


def evaluate_trace(oracle, trace):
    known_units = validate_oracle(oracle)
    if trace.get("measurement_status") != "RECORDED":
        raise ValueError("W-11 trace must be explicitly RECORDED")

    success = trace.get("functional_success_by_workload", {})
    missing = sorted(FROZEN_WORKLOADS - set(success))
    failed = sorted(workload for workload in FROZEN_WORKLOADS if not success.get(workload))
    if missing:
        raise ValueError(f"missing functional result for workloads: {missing}")
    if failed:
        raise ValueError(
            "W-11 comparison invalid because frozen workload functionality failed: "
            + ", ".join(failed)
        )

    exposed = set()
    remote_events = 0
    for event in trace.get("egress_events", []):
        if event.get("boundary") != "outside_user_pc":
            continue
        remote_events += 1
        ids = set(event.get("payload_unit_ids", []))
        ids |= set(event.get("reachable_handle_unit_ids", []))
        unknown = ids - known_units
        if unknown:
            raise ValueError(f"trace references unknown W-11 units: {sorted(unknown)}")
        exposed |= ids

    denominator = oracle["metric"]["denominator"]
    exposure_pct = 100.0 * len(exposed) / denominator
    return {
        "status": "MEASURED_FROM_REVIEWED_UNIT_ANNOTATIONS",
        "candidate_id": trace.get("candidate_id"),
        "exposed_unit_ids": sorted(exposed),
        "exposed_units": len(exposed),
        "denominator": denominator,
        "remote_sensitive_context_exposure_pct": exposure_pct,
        "target_pct": oracle["metric"]["target_pct"],
        "target_met": exposure_pct <= oracle["metric"]["target_pct"],
        "score": _score(exposure_pct, oracle["metric"]["score_bands"]),
        "remote_event_count_secondary": remote_events,
        "note": (
            "Unit ids must be produced by reviewed semantic annotation. "
            "This evaluator does not infer privacy exposure from bytes or guess semantic equivalence."
        ),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--oracle", required=True)
    parser.add_argument("--trace")
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()

    oracle = load_json(args.oracle)
    validate_oracle(oracle)
    if args.validate_only:
        print(json.dumps({
            "status": "PASS",
            "oracle_version": oracle["version"],
            "protected_units": len(oracle["protected_units"]),
            "workloads": len(oracle["workloads"]),
            "measurement": "NOT_RUN",
        }, ensure_ascii=False, indent=2))
        return
    if not args.trace:
        parser.error("--trace is required unless --validate-only is used")
    result = evaluate_trace(oracle, load_json(args.trace))
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
