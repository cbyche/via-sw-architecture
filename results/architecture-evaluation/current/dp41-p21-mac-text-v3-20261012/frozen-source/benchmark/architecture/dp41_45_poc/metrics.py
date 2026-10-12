"""Aggregate observations, never synthesize a candidate's successful evidence.

The speech profile may be simulated. Its times and cost are reported as such;
provider account expenditure and the Windows memory reference stay separate.
"""
from __future__ import annotations

from collections import defaultdict
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
from statistics import mean
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
QA_DIRECTORY = ROOT / "docs/architecture/12-decisions/decision-packages"
CONDITIONS = tuple(f"C{i}" for i in range(1, 7))
UNITS = tuple(f"U{i:02d}" for i in range(1, 19))


def _number(value: Any) -> float | None:
    if isinstance(value, bool) or value is None:
        return None
    try:
        result = float(value)
    except (ValueError, TypeError):
        return None
    return result if math.isfinite(result) else None


def _mean(values: list[float]) -> dict:
    return {"value": mean(values) if values else None, "numerator": sum(values), "n": len(values)}


def _duration(start: Any, end: Any) -> float | None:
    start, end = _number(start), _number(end)
    if start is None or end is None or end < start:
        return None
    return end - start


def _score(value: float | None, qa: str, bands: dict) -> int | None:
    if value is None:
        return None
    item = bands[qa]
    for score, bound in zip(range(6, 0, -1), item["bounds"]):
        if (value >= float(bound) if item["direction"] == "high" else value <= float(bound)):
            return score
    return 0


def _condition(value: Any) -> bool:
    # A PASS without the required trace remains a failure, not an exclusion.
    return isinstance(value, dict) and value.get("passed") is True and bool(value.get("evidence"))


def _contracts(manifest: dict) -> tuple[dict, dict, dict]:
    input_path = Path(manifest.get("qa_input_path", QA_DIRECTORY / "03-02-poc-inputs.json"))
    workload_path = Path(manifest.get("workload_path", QA_DIRECTORY / "03-02-common-workload.json"))
    inputs = json.loads(input_path.read_text())
    workload = json.loads(workload_path.read_text())
    for relative, expected_hash in manifest.get("canonical_sources", {}).items():
        source = ROOT / relative
        if not source.is_file() or hashlib.sha256(source.read_bytes()).hexdigest() != expected_hash:
            raise ValueError(f"frozen canonical source changed: {relative}")
    provenance = {
        "qa_contract_version": inputs["contract_version"],
        "qa_input_path": str(input_path),
        "qa_input_sha256": hashlib.sha256(input_path.read_bytes()).hexdigest(),
        "workload_path": str(workload_path),
        "workload_sha256": hashlib.sha256(workload_path.read_bytes()).hexdigest(),
        "canonical_sources": manifest.get("canonical_sources", {}),
        "campaign_contract": manifest.get("campaign_contract", "DP41-P21-MAC-TEXT-v1"),
        "speech_mode": manifest.get("speech_mode", "SIMULATED_PROFILE"),
        "memory_platform": "macOS",
        "windows_commit_conversion": False,
    }
    return {b["id"]: b for b in inputs["score_bands"]}, workload, provenance


def _reaction(row: dict) -> dict:
    actual = _duration(row.get("start_s"), row.get("first_actual_s"))
    wall_elapsed = _duration(row.get("start_s"), row.get("end_s"))
    valid_elapsed = wall_elapsed
    attributed = _number(row.get("attributed_seconds"))
    if attributed is not None and attributed >= 0:
        valid_elapsed = attributed
    actual_attributed = _number(row.get("first_actual_attributed_seconds"))
    if actual is not None and actual_attributed is not None and actual_attributed >= 0:
        actual = actual_attributed
    wrong = row.get("wrong_first") is True
    valid = row.get("valid") is True and wall_elapsed is not None and valid_elapsed is not None
    # Successful six-second responses remain six seconds: cap is not clipping.
    cap = 5.0 if wrong or not valid else valid_elapsed
    if actual is None and valid and not wrong:
        actual = valid_elapsed
    return {
        "id": row.get("id"), "cap_seconds": cap,
        "correct_seconds": valid_elapsed if valid and not wrong else None,
        "actual_seconds": actual, "wrong_first": wrong,
        "eventually_valid": valid, "missing": not valid,
        "timestamp_missing": (row.get("valid") is True and wall_elapsed is None),
        "wall_clock_seconds": wall_elapsed,
        "first_actual_wall_clock_seconds": _duration(row.get("start_s"), row.get("first_actual_s")),
        "correction_seconds": valid_elapsed if wrong and valid else None,
    }


def _cycle(row: dict) -> dict:
    wall_elapsed = _duration(row.get("start_s"), row.get("valid_end_s"))
    elapsed = _number(row.get("attributed_seconds"))
    if elapsed is None:
        elapsed = wall_elapsed
    valid = row.get("completed") is True and elapsed is not None and elapsed >= 0 and wall_elapsed is not None
    within = valid and elapsed <= 30.0
    return {
        "id": row.get("id"), "cap_seconds": elapsed if within else 30.0,
        "completed_seconds": elapsed if valid else None,
        "completed": valid, "late_completed": valid and not within,
        "never_completed": not valid,
        "timestamp_missing": row.get("completed") is True and not valid,
        "wall_clock_seconds": (_number(row.get("actual_seconds")) if _number(row.get("actual_seconds")) is not None
                               else wall_elapsed),
    }


def _memory(records: list[dict], repeats: list[Any], manifest: dict) -> dict:
    rounds: dict[tuple, list[dict]] = defaultdict(list)
    for r in records:
        if r.get("included_in_via_sum", True):
            rounds[(r.get("repeat"), r.get("sample_id", r.get("sample_round")))].append(r)
    samples = []
    for (repeat, sample_id), rows in rounds.items():
        processes = {}
        for row in rows:
            identity = (row.get("process_id", row.get("pid")), row.get("generation", row.get("process_generation")))
            if identity in processes:
                raise ValueError(f"duplicate memory process in sample {sample_id}: {identity}")
            processes[identity] = row
        expected = set()
        for row in rows:
            expected.update(str(p) for p in row.get("expected_process_ids", []))
        observed = {str(p[0]) for p in processes}
        sums = {}
        for key in ("phys_footprint_bytes", "resident_size_bytes"):
            values = [_number(r.get(key)) for r in rows]
            complete = bool(values) and all(v is not None and v >= 0 for v in values)
            complete = complete and (not expected or expected <= observed)
            complete = complete and all(r.get("collection_status", "OK") in ("OK", "COLLECTED", "SUCCESS")
                                        and r.get("success", True) is True for r in rows)
            sums[key] = sum(values) if complete else None
        samples.append({"repeat": repeat, "sample_id": sample_id, "process_ids": sorted(observed),
                        "missing_process_ids": sorted(expected - observed), **sums,
                        "phase": str(rows[0].get("phase", "")).lower(),
                        "scan_skew_seconds": max((_number(r.get("scan_skew_seconds")) or 0 for r in rows), default=0)})
    result = {"platform": "macOS", "accounting": "OS_PROCESS_SUM_NOT_UNIQUE_PHYSICAL_RAM",
              "samples": samples, "windows_commit_score": None,
              "reference_budget_bytes": 4096 * 1024 * 1024, "reference_is_windows_plan_not_metric_equivalence": True}
    for key, name in (("phys_footprint_bytes", "phys_footprint"), ("resident_size_bytes", "resident_size_rss")):
        peaks = []
        per_repeat = []
        for repeat in repeats:
            selected = [s for s in samples if s["repeat"] == repeat]
            values = [s[key] for s in selected if s[key] is not None]
            peak = max(values) if values else None
            if peak is not None:
                peaks.append(peak)
            idle = [s[key] for s in selected if s["phase"] == "idle" and s[key] is not None]
            per_repeat.append({"repeat": repeat, "peak_bytes": peak,
                               "peak_mib": peak / 1048576 if peak is not None else None,
                               "sample_count": len(selected), "incomplete_samples": sum(s[key] is None for s in selected),
                               "idle_increase_bytes": max(0, idle[-1] - idle[0]) if len(idle) >= 2 else None})
        peak = max(peaks) if peaks else None
        result[name] = {"per_repeat": per_repeat, "peak_bytes": peak,
                        "peak_mib": peak / 1048576 if peak is not None else None,
                        "mean_of_repeat_peaks_bytes": mean(peaks) if peaks else None,
                        "mean_of_repeat_peaks_mib": mean(peaks) / 1048576 if peaks else None,
                        "reference_budget_within": peak <= result["reference_budget_bytes"] if peak is not None else None,
                        "complete_repeats": len(peaks), "expected_repeats": len(repeats)}
    footprint_idle = [r["idle_increase_bytes"] for r in result["phys_footprint"]["per_repeat"]]
    result["mac_reference_guards"] = {
        "rss_peak_within_6gib_reference": result["resident_size_rss"]["peak_bytes"] <= 6144 * 1048576
            if result["resident_size_rss"]["peak_bytes"] is not None else None,
        "idle_footprint_increase_all_within_16mib_reference": all(v <= 16 * 1048576 for v in footprint_idle)
            if footprint_idle and all(v is not None for v in footprint_idle) else None,
        "incomplete_os_samples": sum(r["phys_footprint_bytes"] is None or r["resident_size_bytes"] is None for r in samples),
        "windows_working_set_or_commit_validation": False,
    }
    return result


def _call_cost(call: dict) -> float | None:
    billed = _number(call.get("billed_usd"))
    return billed if billed is not None else _number(call.get("cost_usd"))


def _mock_call(call: dict) -> bool:
    return bool(call.get("mock")) or str(call.get("mode", "")).upper() in ("MOCK", "SCRIPTED")


def _estimated_call(call: dict) -> bool:
    return bool(call.get("estimated")) or "ESTIMATE" in str(call.get("evidence_level", "")).upper()


def _billing(calls: list[dict], variant: Any, repeats: list[Any], manifest: dict) -> dict:
    actual = defaultdict(lambda: Decimal("0"))
    estimated = defaultdict(lambda: Decimal("0"))
    judge = Decimal("0")
    external = Decimal("0")
    missing = []
    mock_calls = []
    missing_judge, missing_external = [], []
    actual_call_ids, estimated_call_ids, invoices = [], [], []
    for c in calls:
        if c.get("variant") != variant:
            continue
        role = str(c.get("role", "")).upper()
        scope = str(c.get("scope", "VIA")).upper()
        cost = _call_cost(c)
        if _mock_call(c):
            mock_calls.append(c.get("call_id"))
            continue
        estimate = _estimated_call(c)
        if cost is None:
            (missing_judge if "JUDGE" in role or scope in ("EVALUATION", "JUDGE") else missing_external
             if scope in ("EXTERNAL_AGENT", "AGENT") else missing).append(c.get("call_id"))
            continue
        amount = Decimal(str(cost))
        if "JUDGE" in role or scope in ("EVALUATION", "JUDGE"):
            judge += amount
        elif scope in ("EXTERNAL_AGENT", "AGENT"):
            external += amount
        elif estimate:
            estimated[c.get("repeat")] += amount
            estimated_call_ids.append(c.get("call_id"))
        else:
            actual[c.get("repeat")] += amount
            actual_call_ids.append(c.get("call_id"))
            if _number(c.get("billed_usd")) is not None:
                invoices.append(Decimal(str(c["billed_usd"])))
    rows = [{"repeat": r, "actual_via_usd": float(actual.get(r, Decimal("0"))),
             "estimated_speech_or_other_usd": float(estimated.get(r, Decimal("0"))),
             "campaign_model_cost_usd": float(actual.get(r, Decimal("0")) + estimated.get(r, Decimal("0")))} for r in repeats]
    known_value = mean([r["campaign_model_cost_usd"] for r in rows]) if rows else None
    actual_value = mean([r["actual_via_usd"] for r in rows]) if rows else None
    hybrid_primary = manifest.get("qa04_primary_metric") == "ACTUAL_PLUS_SIMULATED_SPEECH_ESTIMATE"
    level = ("INCOMPLETE_MODEL_COST" if missing else "HYBRID_REFERENCE_ESTIMATE" if estimated_call_ids
             else "MEASURED_MODEL" if actual_call_ids else "MOCK_NO_PROVIDER_COST" if mock_calls else "NO_MODEL_CALLS")
    return {"per_repeat": rows, "actual_via_account_usd": float(sum(actual.values())),
            "actual_via_usage_msrp_usd": float(sum(actual.values())),
            "invoice_reconciled_via_usd": float(sum(invoices)) if len(invoices) == len(actual_call_ids) and actual_call_ids else None,
            "estimated_speech_or_other_usd": float(sum(estimated.values())),
            "mean_actual_via_usd_per_activity": actual_value,
            "actual_usage_cost_value": actual_value if not missing else None,
            "simulated_speech_inclusive_value": known_value if not missing else None,
            "value": (known_value if hybrid_primary else actual_value) if not missing else None,
            "primary_metric": "ACTUAL_PLUS_SIMULATED_SPEECH_ESTIMATE" if hybrid_primary else "ACTUAL_VIA_PROVIDER_USAGE_X_PRICE",
            "known_cost_mean_lower_bound": known_value,
            "evidence_level": level,
            "actual_call_ids": actual_call_ids, "estimated_call_ids": estimated_call_ids,
            "judge_account_usd": float(judge), "external_agent_account_usd": float(external),
            "missing_usage_or_cost_call_ids": missing, "mock_call_ids": mock_calls,
            "missing_judge_cost_call_ids": missing_judge, "missing_external_cost_call_ids": missing_external,
            "actual_account_total_usd": float(sum(actual.values()) + judge + external),
            "actual_account_total_basis": "PROVIDER_USAGE_X_STANDARD_PRICE_UNLESS_INVOICE_RECONCILED"}


def _changes(records: list[dict]) -> dict:
    tasks = {f"M{i:02d}": None for i in range(1, 7)}
    baselines = set()
    for row in records:
        task = str(row.get("task_id", row.get("change_id", ""))).replace("-", "")
        if task not in tasks:
            continue
        if tasks[task] is not None:
            raise ValueError(f"duplicate change experiment {task}")
        changed = set(row.get("changed_unit_set", row.get("changed_units", [])))
        if changed - set(UNITS):
            raise ValueError(f"unknown functional responsibility: {changed - set(UNITS)}")
        evidence = row.get("evidence_level") == "MEASURED_CHANGE_EXPERIMENT"
        valid = evidence and row.get("completion_passed") is True and row.get("regression_passed") is True
        valid = valid and bool(row.get("baseline_source_hash")) and row.get("patch_verified") is True
        valid = valid and row.get("responsibility_review_passed") is True
        valid = valid and row.get("original_baseline_unchanged") is True
        valid = valid and (bool(changed) or row.get("baseline_support_audit_passed") is True)
        if valid:
            baselines.add(row["baseline_source_hash"])
        tasks[task] = {"task_id": task, "job_id": row.get("job_id"), "completed": valid,
                       "changed_unit_set": sorted(changed), "count": len(changed) if valid else None,
                       "evidence_level": row.get("evidence_level"), "result_ref": row.get("result_ref")}
    ready = all(v is not None and v["completed"] for v in tasks.values()) and len(baselines) == 1
    values = [v["count"] for v in tasks.values() if v and v["completed"]]
    return {"tasks": tasks, "completed_tasks": len(values), "expected_tasks": 6,
            "value": mean(values) if ready else None, "maximum": max(values) if values else None,
            "all_within_five": all(v <= 5 for v in values) if ready else None,
            "ready": ready, "baseline_source_hashes": sorted(baselines),
            "evidence_level": "MEASURED_CHANGE_EXPERIMENT" if ready else "INCOMPLETE_CHANGE_EXPERIMENTS"}


def aggregate(trials: list[dict], memory: list[dict], changes: list[dict], calls: list[dict], manifest: dict) -> dict:
    """Return raw-linked QA values per variant and repeat.

    Set ``strict_coverage=True`` for a full campaign. Candidate failures still
    carry their scheduled condition/reaction/cycle IDs and remain in denominators.
    Missing scheduled trials/IDs cause strict finalization to fail.
    """
    bands, workload, contract = _contracts(manifest)
    case_ids = manifest.get("case_ids") or [c["case_id"] for c in workload["cases"]]
    expected_cases = {c["case_id"]: c for c in workload["cases"] if c["case_id"] in case_ids}
    repeats = manifest.get("repeat_ids") or list(range(1, int(manifest.get("independent_repeats", workload["independent_repeats"])) + 1))
    variants = manifest.get("variants") or sorted({t["variant"] for t in trials})
    if len(set(case_ids)) != len(case_ids) or set(case_ids) != set(expected_cases):
        raise ValueError("unknown or duplicate case IDs in campaign manifest")
    if len(set(repeats)) != len(repeats) or len(set(variants)) != len(variants):
        raise ValueError("duplicate variant/repeat IDs in campaign manifest")
    identities = set()
    for trial in trials:
        identity = trial["variant"], trial["repeat"], trial["case_id"]
        if trial["variant"] not in variants:
            raise ValueError(f"unexpected trial variant {trial['variant']}")
        if identity in identities:
            raise ValueError(f"duplicate trial {identity}")
        identities.add(identity)
    # Updates to one billing record are not independent model calls.
    unique_calls = {}
    for i, call in enumerate(calls):
        key = (call.get("provider"), call.get("provider_response_id") or call.get("provider_request_id") or
               (call.get("call_id", f"unidentified-{i}"), call.get("attempt_id")))
        unique_calls[key] = call
    calls = list(unique_calls.values())
    result = {"contract": contract, "variants": {}, "warnings": [], "paired": {}, "provider_spend_usd": {}}
    provider_costs = defaultdict(lambda: Decimal("0"))
    for call in calls:
        value = _call_cost(call)
        if value is not None and not _estimated_call(call) and not _mock_call(call):
            provider_costs[str(call.get("provider", "UNKNOWN"))] += Decimal(str(value))
    result["provider_spend_usd"] = {k: float(v) for k, v in provider_costs.items()}
    for variant in variants:
        selected = [t for t in trials if t["variant"] == variant]
        coverage_errors = [f"unexpected repeat {t['repeat']}" for t in selected if t["repeat"] not in repeats]
        observations = []
        cycles = []
        passed = 0
        by_condition = defaultdict(int)
        groups = defaultdict(lambda: {"passed": 0, "total": 0})
        group_scenarios = defaultdict(lambda: {"passed": 0, "total": 0})
        scenario_passes = 0
        violations = []
        per_repeat = []
        for repeat in repeats:
            rows = {t["case_id"]: t for t in selected if t["repeat"] == repeat}
            unexpected = set(rows) - set(expected_cases)
            if unexpected:
                coverage_errors.append(f"repeat {repeat}: unexpected cases {sorted(unexpected)}")
            rp, rr, rc = 0, [], []
            for case_id, definition in expected_cases.items():
                row = rows.get(case_id)
                if row is None:
                    coverage_errors.append(f"repeat {repeat}: missing trial {case_id}")
                    continue
                values = row.get("conditions", {})
                if set(values) != set(CONDITIONS):
                    coverage_errors.append(f"repeat {repeat}/{case_id}: incomplete condition IDs")
                correct = [_condition(values.get(c)) for c in CONDITIONS]
                count = sum(correct)
                passed += count
                rp += count
                scenario_passes += all(correct)
                for c, p in zip(CONDITIONS, correct):
                    by_condition[c] += p
                group = case_id.split("-")[0]
                groups[group]["passed"] += count
                groups[group]["total"] += 6
                group_scenarios[group]["passed"] += all(correct)
                group_scenarios[group]["total"] += 1
                violations.extend(row.get("mandatory_violations", []))
                raw_rx = {r["id"]: r for r in row.get("reactions", [])}
                if len(raw_rx) != len(row.get("reactions", [])):
                    coverage_errors.append(f"repeat {repeat}/{case_id}: duplicate reaction IDs")
                expected_rx = {r["id"] for r in definition["reactions"]}
                if set(raw_rx) != expected_rx:
                    coverage_errors.append(f"repeat {repeat}/{case_id}: reaction ID mismatch")
                for reaction in definition["reactions"]:
                    rx = _reaction(raw_rx.get(reaction["id"], {"id": reaction["id"], "valid": False}))
                    rx.update({"variant": variant, "repeat": repeat, "case_id": case_id,
                               "local_stop": "acoustic onset" in reaction["start"]})
                    observations.append(rx)
                    rr.append(rx)
                if definition.get("cycle"):
                    raw_cycle = row.get("cycle") or {}
                    if raw_cycle.get("id") != definition["cycle"]["id"]:
                        coverage_errors.append(f"repeat {repeat}/{case_id}: cycle ID mismatch")
                    cycle = _cycle(raw_cycle)
                    cycle.update({"variant": variant, "repeat": repeat, "case_id": case_id})
                    cycles.append(cycle)
                    rc.append(cycle)
                elif row.get("cycle") is not None:
                    coverage_errors.append(f"repeat {repeat}/{case_id}: non-applicable cycle supplied")
            per_repeat.append({"repeat": repeat, "qa01": 100 * rp / (len(expected_cases) * 6) if expected_cases else None,
                               "qa02": _mean([r["cap_seconds"] for r in rr])["value"],
                               "qa03": _mean([c["cap_seconds"] for c in rc])["value"]})
        if coverage_errors and manifest.get("strict_coverage"):
            raise ValueError(f"incomplete {variant} campaign coverage: " + "; ".join(coverage_errors))
        denominator = len(expected_cases) * len(repeats) * 6
        complete = not coverage_errors and bool(selected)
        full_population = set(expected_cases) == {c["case_id"] for c in workload["cases"]} and len(repeats) == workload["independent_repeats"]
        final_scoring_allowed = complete and full_population
        accuracy = {"value": 100 * passed / denominator if denominator else None,
                    "passed": passed, "failed": denominator - passed, "denominator": denominator,
                    "scenario_passes": scenario_passes, "scenario_denominator": denominator // 6,
                    "by_condition": dict(by_condition), "by_group": dict(groups),
                    "scenario_by_group": dict(group_scenarios), "mandatory_violations": violations}
        scenario_total = denominator // 6
        accuracy["guards"] = {
            "target_98_percent": accuracy["value"] is not None and accuracy["value"] >= float(bands["ASR-QA-01"]["bounds"][-1]),
            "scenarios_95_percent": scenario_passes >= math.ceil(.95 * scenario_total) if scenario_total else False,
            "every_condition_95_percent": all(by_condition[c] >= math.ceil(.95 * scenario_total) for c in CONDITIONS) if scenario_total else False,
            "every_group_95_percent": all(g["passed"] >= .95 * g["total"] for g in groups.values()) if groups else False,
            "mandatory_violation_zero": not violations,
        }
        reactions = {
            "cap_mean": _mean([r["cap_seconds"] for r in observations]),
            "correct_only_mean": _mean([r["correct_seconds"] for r in observations if r["correct_seconds"] is not None]),
            "actual_including_wrong_mean": _mean([r["actual_seconds"] for r in observations if r["actual_seconds"] is not None]),
            "initial_wrong_count": sum(r["wrong_first"] for r in observations),
            "corrected_count": sum(r["wrong_first"] and r["eventually_valid"] for r in observations),
            "missing_count": sum(r["missing"] for r in observations),
            "timestamp_missing_count": sum(r["timestamp_missing"] for r in observations),
            "missed_endpoint_count": sum(r["wrong_first"] or r["missing"] for r in observations),
            "observations": observations,
        }
        ordinary = [r for r in observations if r["case_id"] != "S5-04"]
        burst = [r for r in observations if r["case_id"] == "S5-04"]
        stops = [r for r in observations if r["local_stop"]]
        reactions["guards"] = {
            "ordinary_n": len(ordinary), "ordinary_within_2": sum(r["correct_seconds"] is not None and r["correct_seconds"] <= 2 for r in ordinary),
            "burst_n": len(burst), "burst_within_5": sum(r["correct_seconds"] is not None and r["correct_seconds"] <= 5 for r in burst),
            "local_stop_n": len(stops), "local_stop_within_point1": sum(r["correct_seconds"] is not None and r["correct_seconds"] <= .1 for r in stops),
            "missing": reactions["missing_count"],
        }
        reactions["guard_pass"] = {
            "target_mean_le_1": reactions["cap_mean"]["value"] is not None and reactions["cap_mean"]["value"] <= float(bands["ASR-QA-02"]["bounds"][-1]),
            "ordinary_95_percent_le_2": reactions["guards"]["ordinary_within_2"] >= math.ceil(.95 * len(ordinary)),
            "burst_all_le_5": reactions["guards"]["burst_within_5"] == len(burst),
            "local_stop_all_le_point1": reactions["guards"]["local_stop_within_point1"] == len(stops),
            "missed_endpoint_zero": reactions["missed_endpoint_count"] == 0,
        }
        processing = {"cap_mean": _mean([c["cap_seconds"] for c in cycles]),
                      "completed_only_mean": _mean([c["completed_seconds"] for c in cycles if c["completed_seconds"] is not None]),
                      "completed_count": sum(c["completed"] for c in cycles),
                      "late_completed_count": sum(c["late_completed"] for c in cycles),
                      "incomplete_count": sum(c["never_completed"] for c in cycles),
                      "within_15_count": sum(c["completed_seconds"] is not None and c["completed_seconds"] <= 15 for c in cycles),
                      "observations": cycles}
        processing["guards"] = {
            "target_mean_le_10": processing["cap_mean"]["value"] is not None and processing["cap_mean"]["value"] <= float(bands["ASR-QA-03"]["bounds"][-1]),
            "95_percent_le_15": processing["within_15_count"] >= math.ceil(.95 * len(cycles)),
            "missed_endpoint_zero": processing["incomplete_count"] == 0 and processing["late_completed_count"] == 0,
        }
        billing = _billing(calls, variant, repeats, manifest)
        changing = _changes([c for c in changes if c.get("variant") == variant])
        mem = _memory([m for m in memory if m.get("variant") == variant], repeats, manifest)
        for row in per_repeat:
            cost_key = "campaign_model_cost_usd" if billing["primary_metric"] == "ACTUAL_PLUS_SIMULATED_SPEECH_ESTIMATE" else "actual_via_usd"
            row["qa04"] = next(r[cost_key] for r in billing["per_repeat"] if r["repeat"] == row["repeat"])
            for metric in ("phys_footprint", "resident_size_rss"):
                row[f"qa06_{metric}_mib"] = next(r["peak_mib"] for r in mem[metric]["per_repeat"] if r["repeat"] == row["repeat"])
        output = {"coverage": {"complete": complete, "errors": coverage_errors,
                               "observed_trials": len(selected), "expected_trials": denominator // 6,
                               "conditions": denominator, "reactions": len(observations), "cycles": len(cycles),
                               "scope": "FULL_P21_FIVE_REPEATS" if full_population else "PILOT_SUBSET",
                               "final_scoring_allowed": final_scoring_allowed},
                  "qa01": accuracy, "qa02": reactions, "qa03": processing, "qa04": billing, "qa05": changing,
                  "qa06": mem, "per_repeat": per_repeat,
                  "evidence_level": manifest.get("evidence_level", "HYBRID_REFERENCE_ESTIMATE")}
        for qa, item, value in (("01", accuracy, accuracy["value"]), ("02", reactions, reactions["cap_mean"]["value"]),
                                ("03", processing, processing["cap_mean"]["value"]), ("04", billing, billing["value"]), ("05", changing, changing["value"])):
            item["score"] = _score(value, f"ASR-QA-{qa}", bands) if final_scoring_allowed else None
        if billing["evidence_level"] not in ("MEASURED_MODEL", "HYBRID_REFERENCE_ESTIMATE"):
            billing["score"] = None
        billing["actual_usage_cost_score"] = (_score(billing["actual_usage_cost_value"], "ASR-QA-04", bands)
                                               if final_scoring_allowed and billing["actual_call_ids"] else None)
        billing["simulated_speech_inclusive_score"] = (_score(billing["simulated_speech_inclusive_value"], "ASR-QA-04", bands)
                                                       if final_scoring_allowed and billing["estimated_call_ids"] else None)
        mem["value"] = mem["phys_footprint"]["peak_mib"]
        mem["primary_metric"] = "MAC_PHYS_FOOTPRINT_MAXIMUM_SIMULTANEOUS_SUM_ACROSS_FIVE_ACTIVITIES_MIB"
        mem["metric_basis"] = {"platform": "macOS", "metric": "phys_footprint",
                               "comparison_scale": "CAMPAIGN_POC_USING_EXISTING_MIB_BANDS",
                               "windows_commit_equivalence": False}
        memory_scoring = manifest.get("memory_scoring_metric") == "phys_footprint"
        mem["mac_poc_score"] = (_score(mem["value"], "ASR-QA-06", bands) if memory_scoring and final_scoring_allowed
                                 and mem["phys_footprint"]["complete_repeats"] == len(repeats) else None)
        mem["score"] = mem["mac_poc_score"]
        output["qualification"] = {
            "qa01": complete and all(accuracy["guards"].values()),
            "qa02": complete and all(reactions["guard_pass"].values()),
            "qa03": complete and all(processing["guards"].values()),
            "qa04": complete and billing["value"] is not None and billing["value"] <= float(bands["ASR-QA-04"]["bounds"][-1])
                    and accuracy["failed"] == 0 and accuracy["guards"]["mandatory_violation_zero"],
            "qa05": changing["ready"] and changing["value"] <= float(bands["ASR-QA-05"]["bounds"][-1]) and changing["all_within_five"],
            "qa06_mac_footprint_reference_budget": mem["phys_footprint"]["reference_budget_within"],
        }
        output["numeric_completion"] = {
            "trial_population_complete": complete,
            "cost_complete": billing["evidence_level"] in ("MEASURED_MODEL", "HYBRID_REFERENCE_ESTIMATE") and not billing["missing_usage_or_cost_call_ids"],
            "change_jobs_complete": changing["ready"],
            "memory_repeat_coverage_complete": all(mem[k]["complete_repeats"] == len(repeats) for k in ("phys_footprint", "resident_size_rss")),
        }
        result["variants"][variant] = output
    if len(variants) >= 2:
        baseline = manifest.get("baseline_variant", variants[0])
        for variant in variants:
            if variant == baseline:
                continue
            paired = {}
            for qa in ("qa01", "qa02", "qa03", "qa04", "qa06_phys_footprint_mib", "qa06_resident_size_rss_mib"):
                base_rows = {r["repeat"]: r[qa] for r in result["variants"][baseline]["per_repeat"]}
                values = [{"repeat": r["repeat"], "baseline": base_rows.get(r["repeat"]), "candidate": r[qa],
                           "candidate_minus_baseline": r[qa] - base_rows[r["repeat"]]}
                          for r in result["variants"][variant]["per_repeat"]
                          if r[qa] is not None and base_rows.get(r["repeat"]) is not None]
                differences = [r["candidate_minus_baseline"] for r in values]
                paired[qa] = {"rows": values, "mean_difference": mean(differences) if differences else None,
                              "minimum": min(differences) if differences else None, "maximum": max(differences) if differences else None,
                              "all_same_nonzero_direction": bool(differences) and (all(d > 0 for d in differences) or all(d < 0 for d in differences))}
                threshold = {"qa01": 1, "qa02": .10, "qa03": .50, "qa04": .03,
                             "qa06_phys_footprint_mib": 128, "qa06_resident_size_rss_mib": 128}[qa]
                paired[qa]["practical_difference_threshold"] = threshold
                paired[qa]["consistent_practical_difference"] = (len(differences) == 5 and paired[qa]["all_same_nonzero_direction"]
                                                                and abs(mean(differences)) >= threshold)
            a, b = result["variants"][baseline]["qa05"], result["variants"][variant]["qa05"]
            paired["qa05"] = {"baseline": a["value"], "candidate": b["value"],
                              "candidate_minus_baseline": b["value"] - a["value"] if a["ready"] and b["ready"] else None,
                              "sampling": "SIX_INDEPENDENT_CHANGE_TASKS_NOT_FIVE_PERFORMANCE_REPEATS",
                              "practical_difference_threshold": .5,
                              "practical_difference": abs(b["value"] - a["value"]) >= .5 if a["ready"] and b["ready"] else None}
            result["paired"][f"{baseline}:{variant}"] = paired
    return result
