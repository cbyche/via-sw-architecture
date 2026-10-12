"""API-free recalculation from immutable V7 raw traces and approved judgements.

Does not import candidate/evaluator code or alter any input. The final normalizer
is checked by reconstructing clocks and causal attribution independently.
"""
from __future__ import annotations

import argparse
import ast
from collections import Counter, defaultdict
import csv
from datetime import datetime, timezone
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
from statistics import mean


EXCLUDED = {"EXTERNAL_AGENT_WORK", "EXTERNAL_USER_WAIT", "EXTERNAL_USER_SPEECH"}
MIB = 1048576


def rows(path):
    if not path.exists():
        return
    with path.open() as handle:
        for line in handle:
            if line.strip():
                yield json.loads(line)


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def identity(row):
    return row["variant"], row["repeat"], row["case_id"]


def main(campaign):
    final = campaign / "final"
    if not (final / "completion.json").exists():
        raise SystemExit("Final completion is not available; no final audit was generated")
    checks = Counter()
    failures = []
    input_hashes = {}

    def check(name, observed, expected=True, detail=None):
        checks["total"] += 1
        good = observed == expected
        checks["passed" if good else "failed"] += 1
        if not good:
            failures.append({"check": name, "observed": observed, "expected": expected, "detail": detail})

    def close(name, observed, expected, detail=None):
        good = observed is not None and expected is not None and math.isclose(
            float(observed), float(expected), abs_tol=1e-8, rel_tol=1e-9)
        check(name, good, detail={"observed": observed, "expected": expected, "context": detail})

    def load(relative):
        path = campaign / relative
        input_hashes[relative] = sha(path)
        return json.loads(path.read_text())

    manifest = load("manifest.json")
    completion = load("final/completion.json")
    official = load("final/qa-results.json")
    for field, expected in {"observed_trials": 210, "conditions": 1260,
                            "reactions": 270, "cycles": 190, "change_jobs": 12}.items():
        check("completion." + field, completion[field], expected)
    check("completion.status", completion["status"], "COMPLETE")
    actual_hashes = {name: sha(campaign / "frozen-source" / name)
                     for name in manifest["frozen_source_hashes"]}
    check("frozen_files", len(actual_hashes), 53)
    for name, actual in actual_hashes.items():
        check("frozen_file", actual, manifest["frozen_source_hashes"][name], name)
    flat_digest = hashlib.sha256(json.dumps(actual_hashes, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    check("frozen_digest", flat_digest, manifest["frozen_source_digest"])
    evaluator_generations = []
    baseline_judge = campaign / "frozen-source/benchmark/architecture/dp41_45_poc/judge.py"
    baseline_ast = ast.parse(baseline_judge.read_text())
    original_system = next(n.value.value for n in baseline_ast.body
                           if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "SYSTEM" for t in n.targets))
    original_view = ast.dump(next(n for n in baseline_ast.body if isinstance(n, ast.FunctionDef) and n.name == "evaluation_view"))
    for relative in ("evaluation-tool-manifest.json", "evaluation-tool-manifest-v2.json", "evaluation-tool-manifest-v3.json", "evaluation-tool-manifest-v4.json"):
        path = campaign / relative
        if not path.exists():
            continue
        generation = load(relative)
        directory = Path(generation.get("tool_directory", generation.get("root", str(campaign / "evaluator-tools"))))
        hashes = generation["file_hashes"]
        check("evaluator_generation_file_count", len(hashes), 53, relative)
        for name, expected in hashes.items():
            check("evaluator_generation_file", sha(directory / name), expected, [relative, name])
            if name != "benchmark/architecture/dp41_45_poc/judge.py":
                check("evaluator_only_judge_change", expected, actual_hashes[name], [relative, name])
        digest = hashlib.sha256(json.dumps(hashes, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        check("evaluator_generation_digest", digest, generation["source_digest"], relative)
        judge_ast = ast.parse((directory / "benchmark/architecture/dp41_45_poc/judge.py").read_text())
        system = next(n.value.value for n in judge_ast.body
                      if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == "SYSTEM" for t in n.targets))
        view = ast.dump(next(n for n in judge_ast.body if isinstance(n, ast.FunctionDef) and n.name == "evaluation_view"))
        validator = ast.dump(next(n for n in judge_ast.body if isinstance(n, ast.FunctionDef) and n.name == "validate_judgement"))
        original_validator = ast.dump(next(n for n in baseline_ast.body if isinstance(n, ast.FunctionDef) and n.name == "validate_judgement"))
        check("evaluator_SYSTEM_unchanged", system, original_system, relative)
        check("evaluator_first_pass_view_unchanged", view, original_view, relative)
        if relative == "evaluation-tool-manifest-v4.json":
            v3_ast = ast.parse((campaign / "evaluator-tools-v3/benchmark/architecture/dp41_45_poc/judge.py").read_text())
            v3_validator = ast.dump(next(n for n in v3_ast.body if isinstance(n, ast.FunctionDef) and n.name == "validate_judgement"))
            check("V4_validator_unchanged_from_repaired_V3", validator, v3_validator, relative)
        elif relative == "evaluation-tool-manifest.json":
            check("original_validator_same_as_frozen", validator, original_validator, relative)
        evaluator_generations.append({"manifest": relative, "manifest_sha256": sha(path), "source_digest": digest,
                                      "changes": generation.get("changes", [])})
    score_inputs = load("frozen-source/docs/architecture/12-decisions/decision-packages/03-02-poc-inputs.json")
    bands = {b["id"]: b for b in score_inputs["score_bands"]}
    workload = load("frozen-source/docs/architecture/12-decisions/decision-packages/03-02-common-workload.json")
    definitions = {d["case_id"]: d for d in workload["cases"]}
    expected_keys = {(v, r, case) for v in ("A", "B") for r in range(1, 6) for case in definitions}

    # Selection is already independently reviewed; verify the exact pinned final
    # files again instead of silently finding every preserved alternative.
    selection = load("qa05-selection-manifest.json")
    change_counts = defaultdict(dict)
    for chosen in selection["selected"]:
        path = campaign / chosen["reviewed_result_ref"]
        record = load(chosen["reviewed_result_ref"])
        check("selected_result_hash", sha(path), chosen["reviewed_result_sha256"], str(path))
        key = chosen["variant"], chosen["task_id"]
        check("selected_identity", (record["variant"], record["task_id"]), key)
        check("selected_complete", record["status"], "COMPLETED")
        for flag in ("completion_passed", "regression_passed", "patch_verified",
                     "responsibility_review_passed", "original_baseline_unchanged"):
            check("selected_" + flag, record[flag], detail=key)
        units = set(record["changed_unit_set"])
        check("selected_count", record["count"], len(units), key)
        check("selection_count", chosen["count"], len(units), key)
        check("selected_patch_hash", sha(Path(record["patch_ref"])), record["patch_sha256"], key)
        if not units:
            check("zero_delta_baseline_support", record["baseline_support_audit_passed"], detail=key)
        check("unique_change_task", chosen["task_id"] not in change_counts[chosen["variant"]], detail=key)
        change_counts[chosen["variant"]][chosen["task_id"]] = len(units)
    check("selected_count12", sum(map(len, change_counts.values())), 12)
    natural = selection["M04_actual_natural_language_probes"]
    # Keep actual semantic FAIL separate from designated-port functional PASS.
    semantic_statuses = []
    for variant, probe in natural.items():
        path = campaign / probe["ref"]
        input_hashes[probe["ref"]] = sha(path)
        actual = list(rows(path))[-1]
        check("M04_actual_FAIL_manifest_preserved", probe["status"], "FAIL", variant)
        check("M04_actual_FAIL_raw_preserved", actual["status"], "FAIL", variant)
        check("M04_extra_not_P21", probe["included_in_P21"], False, variant)
        check("M04_extra_scope", probe["scope"], "EVALUATION_CHANGE", variant)
        semantic_statuses.append({"variant": variant, "path": str(path), "sha256": sha(path),
                                  "status": actual["status"], "actual_record": actual})

    adjudication_path = campaign / "evaluation/adjudications.jsonl"
    input_hashes["evaluation/adjudications.jsonl"] = sha(adjudication_path)
    original = {}
    for row in rows(adjudication_path):
        key = identity(row)
        check("original_adjudication_unique", key not in original, detail=key)
        original[key] = row
    override_path = campaign / "evaluation/trace-adjudication-overrides.jsonl"
    overrides = {}
    if override_path.exists():
        input_hashes["evaluation/trace-adjudication-overrides.jsonl"] = sha(override_path)
        for row in rows(override_path):
            key = identity(row)
            check("override_unique", key not in overrides, detail=key)
            check("override_has_original", key in original, detail=key)
            overrides[key] = row
    judged = {**original, **overrides}
    check("final_judgement_population", set(judged), expected_keys)
    for key, row in judged.items():
        check("uncertainty_resolved", row["final_judgement"].get("uncertain") is False, detail=key)
        check("condition_ids", set(row["final_judgement"]["conditions"]), {f"C{i}" for i in range(1, 7)}, key)

    normalized_path = final / "normalized-trials.jsonl"
    input_hashes["final/normalized-trials.jsonl"] = sha(normalized_path)
    normalized = {identity(r): r for r in rows(normalized_path)}
    check("normalized_population", set(normalized), expected_keys)
    for filename, count in {"conditions.csv": 1260, "reactions.csv": 270,
                            "cycles.csv": 190, "changes.csv": 12, "repeat-results.csv": 10}.items():
        path = final / filename
        input_hashes["final/" + filename] = sha(path)
        with path.open(newline="") as handle:
            check("CSV_population", sum(1 for _ in csv.DictReader(handle)), count, filename)
    stats = {v: {"passed": 0, "scenario_passes": 0, "by_condition": Counter(),
                 "groups": defaultdict(lambda: {"passed": 0, "total": 0}),
                 "violations": [], "reactions": [], "cycles": [],
                 "repeat": {r: {"passed": 0, "reactions": [], "cycles": []} for r in range(1, 6)}}
             for v in ("A", "B")}
    raw_keys = set()
    attribution_endpoints = 0

    def start_time(trial, blueprint):
        kind = blueprint.get("start_kind", "SPEECH_END")
        if kind == "SPEECH_ONSET":
            return next(e["scene_seconds"] for e in trial["events"] if e["kind"] == "INPUT_STARTED")
        if kind in ("SPEECH_END", "FOLLOWUP_SPEECH_END"):
            return trial["observed_inputs"][0]["speech_end_s"]
        if kind == "SOURCE_EVENT_AVAILABLE":
            found = [e for e in trial["observed_source_availability"]
                     if e["event"]["event_id"] == blueprint["source_event_id"]]
            check("unique_actual_source_boundary", len(found), 1, identity(trial))
            return found[0]["actual_source_offset_seconds"]
        raise ValueError("unknown actual boundary " + kind)

    def endpoint(trial, publication_id, quote):
        if publication_id is None:
            return None, None
        matches = [d for d in trial["deliveries"] if d.get("publication_id") == publication_id]
        check("unique_publication", len(matches), 1, identity(trial))
        delivery = matches[0]
        if delivery.get("text_presented_s") is None or (delivery["voice_on"] and delivery.get("voice_onset_s") is None):
            return None, None
        if not delivery["voice_on"]:
            return delivery["text_presented_s"], delivery["text_node_id"]
        check("exact_actual_emitted_quote", bool(quote) and quote in delivery.get("voice_text", ""), detail=identity(trial))
        index = delivery["voice_text"].index(quote)
        emitted = next(c for c in delivery["voice_char_events"] if c["char_index"] == index)
        return max(delivery["text_presented_s"], emitted["onset_s"]), emitted["causal_node_id"]

    def attributable(trial, scene_start, node_id):
        nonlocal attribution_endpoints
        attribution_endpoints += 1
        start = trial["scene_origin_evidence_seconds"] + scene_start
        graph = {n["node_id"]: n for n in trial["causal_timeline"]["nodes"]}
        memo, active = {}, set()

        def solve(key):
            if key in memo:
                return memo[key]
            if key in active:
                raise ValueError("cycle in raw DAG")
            active.add(key)
            node = graph[key]
            parents = [solve(p) for p in node["parents"]]
            parent_end = max([start] + [graph[p]["end_s"] for p in node["parents"]])
            check("DAG_temporal_parent", parent_end <= max(start, node["start_s"]) + 1e-6, detail=key)
            gap = max(0.0, node["start_s"] - parent_end)
            service = max(0.0, max(start, node["end_s"]) - max(start, node["start_s"]))
            wall = max((p[0] for p in parents), default=0.0) + gap + service
            via = max((p[1] for p in parents), default=0.0) + gap + (0.0 if node["kind"] in EXCLUDED else service)
            active.remove(key)
            memo[key] = wall, via
            return wall, via

        wall, via = solve(node_id)
        close("DAG_wall_reconstruction", wall, graph[node_id]["end_s"] - start, identity(trial))
        check("DAG_nonnegative_attribution", -1e-8 <= via <= wall + 1e-8, detail=identity(trial))
        return via

    for worker in sorted((campaign / "workers").iterdir()):
        trial_path = worker / "trials-unjudged.jsonl"
        input_hashes[str(trial_path.relative_to(campaign))] = sha(trial_path)
        summary = load(str((worker / "worker-summary.json").relative_to(campaign))) if (worker / "worker-summary.json").exists() else None
        if summary is None:
            summaries = list(rows(worker / "worker-summary.jsonl"))
            check("worker_complete", summaries[-1]["status"], "COMPLETE", worker.name)
            check("worker_cases", summaries[-1]["case_count"], 21, worker.name)
        for trial in rows(trial_path):
            key = identity(trial)
            check("raw_trial_unique", key not in raw_keys, detail=key)
            raw_keys.add(key)
            judgement = judged[key]["final_judgement"]
            norm = normalized[key]
            check("normalized_conditions_from_judge", norm["conditions"], judgement["conditions"], key)
            check("normalized_violations_from_judge", norm["mandatory_violations"], judgement["critical_violations"], key)
            check("raw_trial_evidence_level", trial["evidence_level"], "MEASURED_MODEL_WITH_SIMULATED_SPEECH", key)
            current = stats[key[0]]
            repeat = current["repeat"][key[1]]
            passed = [judgement["conditions"][f"C{i}"].get("passed") is True and bool(judgement["conditions"][f"C{i}"].get("evidence"))
                      for i in range(1, 7)]
            current["passed"] += sum(passed)
            current["scenario_passes"] += all(passed)
            repeat["passed"] += sum(passed)
            current["violations"].extend(judgement["critical_violations"])
            for i, value in enumerate(passed, 1):
                current["by_condition"][f"C{i}"] += value
            group = current["groups"][key[2].split("-")[0]]
            group["passed"] += sum(passed)
            group["total"] += 6
            check("reaction_ids", {r["id"] for r in norm["reactions"]}, {r["id"] for r in definitions[key[2]]["reactions"]}, key)
            for bp in trial["reaction_blueprint"]:
                j = next(r for r in judgement["reactions"] if r["id"] == bp["id"])
                n = next(r for r in norm["reactions"] if r["id"] == bp["id"])
                start = start_time(trial, bp)
                if bp["endpoint_kind"] == "LOCAL_OUTPUT_STOP":
                    stops = [d for d in trial["deliveries"] if d["kind"] == "LOCAL_STOP"]
                    actual, node = (stops[0]["end_s"], stops[0]["causal_node_id"]) if stops else (None, None)
                    end, end_node, wrong = actual, node, False
                else:
                    end, end_node = endpoint(trial, j["valid_publication_id"], j.get("valid_core_quote"))
                    actual, node = endpoint(trial, j["first_actual_publication_id"], j.get("first_actual_core_quote"))
                    wrong = j["wrong_first"]
                close("reaction_start", n["start_s"], start, key)
                for field, value in {"end_s": end, "first_actual_s": actual, "wrong_first": wrong,
                                     "valid_endpoint_node": end_node, "first_actual_endpoint_node": node}.items():
                    check("reaction_raw_endpoint_" + field, n.get(field), value, key)
                valid_via = attributable(trial, start, end_node) if end_node else None
                actual_via = attributable(trial, start, node) if node else None
                if valid_via is not None:
                    close("reaction_via", n["attributed_seconds"], valid_via, key)
                if actual_via is not None:
                    close("first_actual_via", n["first_actual_attributed_seconds"], actual_via, key)
                rx = {"case_id": key[2], "id": bp["id"], "cap": 5.0 if wrong or end is None else valid_via,
                      "correct": valid_via if not wrong and end is not None else None,
                      "actual": actual_via, "wrong": wrong, "missing": end is None,
                      "local_stop": bp["endpoint_kind"] == "LOCAL_OUTPUT_STOP"}
                current["reactions"].append(rx)
                repeat["reactions"].append(rx)
            if trial["cycle_blueprint"]:
                start = start_time(trial, {"start_kind": "SPEECH_END"})
                end, node = endpoint(trial, judgement["cycle"]["valid_publication_id"], judgement["cycle"].get("valid_core_quote"))
                n = norm["cycle"]
                close("cycle_start", n["start_s"], start, key)
                check("cycle_endpoint", n["valid_end_s"], end, key)
                via = attributable(trial, start, node) if node else None
                if via is not None:
                    close("cycle_via", n["attributed_seconds"], via, key)
                cy = {"cap": via if via is not None and via <= 30 else 30.0,
                      "completed": via, "late": via is not None and via > 30,
                      "missing": via is None}
                current["cycles"].append(cy)
                repeat["cycles"].append(cy)
            else:
                check("nonapplicable_cycle", norm["cycle"], None, key)
    check("raw_population", raw_keys, expected_keys)

    def score(value, qa):
        band = bands[f"ASR-QA-{qa:02d}"]
        thresholds = list(map(float, band["bounds"]))
        eligible = [6 - i for i, bound in enumerate(thresholds)
                    if (value >= bound if band["direction"] == "high" else value <= bound)]
        return max(eligible, default=0)

    computed = {}
    for variant, current in stats.items():
        off = official["variants"][variant]
        rx, cy = current["reactions"], current["cycles"]
        check("condition_denominator", off["qa01"]["denominator"], 630, variant)
        check("reaction_population", len(rx), 135, variant)
        check("cycle_population", len(cy), 95, variant)
        close("QA01_value", off["qa01"]["value"], current["passed"] / 630 * 100, variant)
        check("QA01_passed", off["qa01"]["passed"], current["passed"], variant)
        check("QA01_failed", off["qa01"]["failed"], 630 - current["passed"], variant)
        check("QA01_scenario_passes", off["qa01"]["scenario_passes"], current["scenario_passes"], variant)
        check("QA01_by_condition", off["qa01"]["by_condition"], dict(current["by_condition"]), variant)
        check("QA01_by_group", off["qa01"]["by_group"], dict(current["groups"]), variant)
        qa01_guards = {"target_98_percent": current["passed"] / 630 * 100 >= 98,
                       "scenarios_95_percent": current["scenario_passes"] >= math.ceil(.95 * 105),
                       "every_condition_95_percent": all(n >= math.ceil(.95 * 105) for n in current["by_condition"].values()),
                       "every_group_95_percent": all(g["passed"] >= .95 * g["total"] for g in current["groups"].values()),
                       "mandatory_violation_zero": not current["violations"]}
        check("QA01_guards", off["qa01"]["guards"], qa01_guards, variant)
        values = {1: current["passed"] / 630 * 100, 2: mean(r["cap"] for r in rx), 3: mean(c["cap"] for c in cy)}
        for field, values_rx in {"cap_mean": [r["cap"] for r in rx],
                                 "correct_only_mean": [r["correct"] for r in rx if r["correct"] is not None],
                                 "actual_including_wrong_mean": [r["actual"] for r in rx if r["actual"] is not None]}.items():
            item = off["qa02"][field]
            check("QA02_mean_n", item["n"], len(values_rx), [variant, field])
            close("QA02_mean_sum", item["numerator"], sum(values_rx), [variant, field])
            if values_rx:
                close("QA02_mean", item["value"], mean(values_rx), [variant, field])
            else:
                check("QA02_empty_mean", item["value"], None, [variant, field])
        ordinary = [r for r in rx if r["case_id"] != "S5-04"]
        burst = [r for r in rx if r["case_id"] == "S5-04"]
        stops = [r for r in rx if r["local_stop"]]
        q2_guards = {"target_mean_le_1": values[2] <= 1,
                     "ordinary_95_percent_le_2": sum(r["correct"] is not None and r["correct"] <= 2 for r in ordinary) >= math.ceil(.95 * len(ordinary)),
                     "burst_all_le_5": all(r["correct"] is not None and r["correct"] <= 5 for r in burst),
                     "local_stop_all_le_point1": all(r["correct"] is not None and r["correct"] <= .1 for r in stops),
                     "missed_endpoint_zero": not any(r["wrong"] or r["missing"] for r in rx)}
        check("QA02_guards", off["qa02"]["guard_pass"], q2_guards, variant)
        for field, values_cy in {"cap_mean": [c["cap"] for c in cy],
                                 "completed_only_mean": [c["completed"] for c in cy if c["completed"] is not None]}.items():
            item = off["qa03"][field]
            check("QA03_mean_n", item["n"], len(values_cy), [variant, field])
            close("QA03_mean_sum", item["numerator"], sum(values_cy), [variant, field])
            if values_cy:
                close("QA03_mean", item["value"], mean(values_cy), [variant, field])
            else:
                check("QA03_empty_mean", item["value"], None, [variant, field])
        q3_guards = {"target_mean_le_10": values[3] <= 10,
                     "95_percent_le_15": sum(c["completed"] is not None and c["completed"] <= 15 for c in cy) >= math.ceil(.95 * 95),
                     "missed_endpoint_zero": not any(c["missing"] or c["late"] for c in cy)}
        check("QA03_guards", off["qa03"]["guards"], q3_guards, variant)
        costs, call_count = {}, 0
        footprint_peaks, rss_peaks, idle_increases, memory_count, failed_samples = {}, {}, {}, 0, 0
        for repeat in range(1, 6):
            worker = campaign / "workers" / f"{variant}{repeat}"
            call_path = worker / "model-calls.jsonl"
            input_hashes[str(call_path.relative_to(campaign))] = sha(call_path)
            total, ids = Decimal(0), set()
            for call in rows(call_path):
                check("unique_VIA_attempt", call["call_id"] not in ids, detail=call["call_id"])
                ids.add(call["call_id"])
                check("VIA_role", call["role"] in ("CINTERPRET", "CRESPONSE"), detail=call["call_id"])
                usage, price = call["usage"], call["price"]
                cached = usage.get("input_tokens_details", {}).get("cached_tokens", 0)
                write = usage.get("input_tokens_details", {}).get("cache_write_tokens", 0)
                amount = ((Decimal(usage["input_tokens"] - cached - write) * Decimal(str(price["input"]))
                           + Decimal(usage["output_tokens"]) * Decimal(str(price["output"]))
                           + Decimal(cached) * Decimal(str(price.get("cached", price["input"])))
                           + Decimal(write) * Decimal(str(price.get("cache_write", price["input"])))) / Decimal(1000000))
                close("actual_usage_price", call["cost_usd"], float(amount), call["call_id"])
                total += amount
                call_count += 1
            costs[repeat] = float(total)
            close("QA04_repeat", off["per_repeat"][repeat - 1]["qa04"], float(total), [variant, repeat])
            memory_path = worker / "memory.jsonl"
            input_hashes[str(memory_path.relative_to(campaign))] = sha(memory_path)
            samples = defaultdict(dict)
            phases = {}
            for m in rows(memory_path):
                if not m.get("included_in_via_sum", True):
                    continue
                key = m["sample_id"]
                pid = m["process_id"], m["generation"]
                check("unique_process_sample", pid not in samples[key], detail=[variant, repeat, key])
                samples[key][pid] = m
                phases[key] = m["phase"]
            fp, rss, idle = [], [], []
            for sid, processes in samples.items():
                good = all(m.get("success") is True and m.get("phys_footprint_bytes") is not None and m.get("resident_size_bytes") is not None
                           for m in processes.values())
                memory_count += 1
                failed_samples += not good
                if not good:
                    continue
                f = sum(m["phys_footprint_bytes"] for m in processes.values())
                r = sum(m["resident_size_bytes"] for m in processes.values())
                fp.append(f)
                rss.append(r)
                if phases[sid].lower() == "idle":
                    idle.append(f)
            footprint_peaks[repeat], rss_peaks[repeat] = max(fp), max(rss)
            idle_increases[repeat] = max(0, idle[-1] - idle[0]) if len(idle) >= 2 else None
            for metric, peak in (("phys_footprint", max(fp)), ("resident_size_rss", max(rss))):
                item = off["qa06"][metric]["per_repeat"][repeat - 1]
                close("QA06_repeat_peak", item["peak_bytes"], peak, [variant, repeat, metric])
                check("QA06_repeat_sample_count", item["sample_count"], len(samples), [variant, repeat, metric])
                close("QA06_per_repeat", off["per_repeat"][repeat - 1][f"qa06_{metric}_mib"], peak / MIB, [variant, repeat, metric])
            rp = current["repeat"][repeat]
            close("QA01_per_repeat", off["per_repeat"][repeat - 1]["qa01"], rp["passed"] / 126 * 100, [variant, repeat])
            close("QA02_per_repeat", off["per_repeat"][repeat - 1]["qa02"], mean(r["cap"] for r in rp["reactions"]), [variant, repeat])
            close("QA03_per_repeat", off["per_repeat"][repeat - 1]["qa03"], mean(c["cap"] for c in rp["cycles"]), [variant, repeat])
        values[4] = mean(costs.values())
        close("QA04_value", off["qa04"]["value"], values[4], variant)
        close("QA04_campaign_cost", off["qa04"]["actual_via_usage_msrp_usd"], sum(costs.values()), variant)
        check("QA04_all_attempts", len(off["qa04"]["actual_call_ids"]), call_count, variant)
        values[5] = mean(change_counts[variant].values())
        close("QA05_value", off["qa05"]["value"], values[5], variant)
        check("QA05_task_count", off["qa05"]["completed_tasks"], 6, variant)
        for task, count in change_counts[variant].items():
            check("QA05_task_units", off["qa05"]["tasks"][task]["count"], count, [variant, task])
        values[6] = max(footprint_peaks.values()) / MIB
        close("QA06_value", off["qa06"]["value"], values[6], variant)
        for metric, peaks in (("phys_footprint", footprint_peaks), ("resident_size_rss", rss_peaks)):
            close("QA06_global_peak", off["qa06"][metric]["peak_bytes"], max(peaks.values()), [variant, metric])
            close("QA06_aux_mean_peak", off["qa06"][metric]["mean_of_repeat_peaks_bytes"], mean(peaks.values()), [variant, metric])
        check("QA06_incomplete_samples", off["qa06"]["mac_reference_guards"]["incomplete_os_samples"], failed_samples, variant)
        q6_guards = {"rss_peak_within_6gib_reference": max(rss_peaks.values()) <= 6144 * MIB,
                     "idle_footprint_increase_all_within_16mib_reference": all(v is not None and v <= 16 * MIB for v in idle_increases.values()),
                     "incomplete_os_samples": failed_samples, "windows_working_set_or_commit_validation": False}
        check("QA06_guards", off["qa06"]["mac_reference_guards"], q6_guards, variant)
        check("QA06_Windows_score_absent", off["qa06"]["windows_commit_score"], None, variant)
        qualification = {"qa01": all(qa01_guards.values()), "qa02": all(q2_guards.values()),
                         "qa03": all(q3_guards.values()),
                         "qa04": values[4] <= .5 and current["passed"] == 630 and not current["violations"],
                         "qa05": values[5] <= 3 and max(change_counts[variant].values()) <= 5,
                         "qa06_mac_footprint_reference_budget": values[6] <= 4096}
        check("all_qualification", off["qualification"], qualification, variant)
        for qa, value in values.items():
            check("QA_score", off[f"qa{qa:02d}"]["score"], score(value, qa), [variant, qa])
        computed[variant] = {"values": {f"qa{k:02d}": v for k, v in values.items()},
                             "scores": {f"qa{k:02d}": score(v, k) for k, v in values.items()},
                             "condition_passes": current["passed"], "condition_denominator": 630,
                             "reaction_denominator": len(rx), "cycle_denominator": len(cy),
                             "QA02_correct_n": sum(r["correct"] is not None for r in rx),
                             "QA02_actual_n": sum(r["actual"] is not None for r in rx),
                             "QA03_completed_n": sum(c["completed"] is not None for c in cy),
                             "QA05_changed_units": change_counts[variant], "VIA_call_count": call_count,
                             "VIA_cost_by_repeat": costs, "phys_footprint_peak_bytes": max(footprint_peaks.values()),
                             "RSS_peak_bytes": max(rss_peaks.values()), "memory_samples": memory_count,
                             "qualification": qualification}
    for qa, paired in official["paired"]["A:B"].items():
        if qa == "qa05":
            close("paired_QA05", paired["candidate_minus_baseline"], computed["B"]["values"][qa] - computed["A"]["values"][qa])
            continue
        deltas = []
        for row in paired["rows"]:
            repeat = row["repeat"]
            a = official["variants"]["A"]["per_repeat"][repeat - 1][qa]
            b = official["variants"]["B"]["per_repeat"][repeat - 1][qa]
            close("paired_row", row["candidate_minus_baseline"], b - a, [qa, repeat])
            deltas.append(b - a)
        check("paired_repeat_count", len(deltas), 5, qa)
        close("paired_mean", paired["mean_difference"], mean(deltas), qa)
        check("paired_consistent_practical", paired["consistent_practical_difference"],
              (all(d > 0 for d in deltas) or all(d < 0 for d in deltas)) and abs(mean(deltas)) >= paired["practical_difference_threshold"], qa)
    origin_manifest = campaign / "evaluation/evaluation-origin-manifest.json"
    check("merged_origin_manifest_exists", origin_manifest.exists())
    origin_review = None
    if origin_manifest.exists():
        origin_review = load(str(origin_manifest.relative_to(campaign)))
        check("origin_complete", origin_review["status"], "COMPLETE")
        check("origin_trial_count", origin_review["original_adjudications"], 210)
        check("origin_variant_counts", origin_review["variant_counts"], {"A": 105, "B": 105})
        check("origin_methods", origin_review["methods"], dict(Counter(r["method"] for r in original.values())))
        check("origin_no_candidate_remeasurement", origin_review["candidate_remeasurement_for_evaluator_repair"], False)
        check("origin_initial_prompt_stable", origin_review["same_initial_prompt_model_oracle_thresholds"], True)
        check("origin_manifest_generations", {r["ref"] for r in origin_review["evaluator_manifests"]},
              {r["manifest"] for r in evaluator_generations})
        generation_by_ref = {r["manifest"]: r for r in evaluator_generations}
        for entry in origin_review["evaluator_manifests"]:
            generation = generation_by_ref[entry["ref"]]
            check("origin_manifest_sha256", entry["sha256"], generation["manifest_sha256"], entry["ref"])
            check("origin_manifest_source_digest", entry["source_digest"], generation["source_digest"], entry["ref"])
        for ledger in origin_review["raw_ledgers"]:
            path = campaign / ledger["ref"]
            actual = sha(path)
            check("origin_raw_ledger_sha256", actual, ledger["sha256"], ledger["ref"])
            input_hashes[ledger["ref"]] = actual
        merged_count = 0
        preserved_sources = {}
        for key, row in original.items():
            reference = row.get("evaluation_retry_origin")
            if not reference:
                continue
            merged_count += 1
            if reference not in preserved_sources:
                preserved_sources[reference] = {identity(item): item for item in rows(campaign / reference)}
            source = preserved_sources[reference].get(key)
            check("merged_row_source_exists", source is not None, detail=key)
            if source is None:
                continue
            source_sha = hashlib.sha256(canonical(source)).hexdigest()
            check("merged_row_source_canonical_sha256", row.get("source_row_sha256"), source_sha, key)
            copied = {k: v for k, v in row.items() if k not in {"evaluation_retry_origin", "source_row_sha256"}}
            check("merged_row_content_preserved", copied, source, key)
        check("origin_merged_row_count", origin_review["preserved_retry_merge"]["new_adjudications_merged"], merged_count)
        call_rows = list(rows(campaign / "evaluation/model-calls.jsonl"))
        check("judge_call_ids_unique_after_merge", len(call_rows), len({r["call_id"] for r in call_rows}))
    source_unchanged = {name: sha(campaign / "frozen-source" / name) for name in actual_hashes}
    check("frozen_unchanged_after_audit", source_unchanged, actual_hashes)
    for relative, initial_hash in list(input_hashes.items()):
        check("audit_input_unchanged", sha(campaign / relative), initial_hash, relative)
    report = {"schema": "DP41-FINAL-INDEPENDENT-REVIEW-v1", "campaign": str(campaign),
              "generated_utc": datetime.now(timezone.utc).isoformat(), "author": "/root/poc_qa_readiness",
              "api_calls": 0, "input_mutations": 0, "candidate_or_frozen_modifications": 0,
              "status": "PASS_WITH_DECLARED_EVIDENCE_SCOPE" if not failures else "FINDINGS",
              "method": "STDLIB_RAW_TRACE_RECONSTRUCTION_NO_CANDIDATE_OR_METRICS_IMPORT",
              "script_sha256": sha(Path(__file__)), "frozen_source_digest": flat_digest,
              "evaluator_generations": evaluator_generations,
              "population": completion, "check_summary": dict(checks), "failed_checks": failures,
              "computed": computed, "attribution_endpoints_recomputed": attribution_endpoints,
              "original_adjudications": len(original), "approved_overrides": len(overrides),
              "evaluation_origin_review": origin_review,
              "M04_natural_language_manifest": natural, "M04_actual_FAIL_records": semantic_statuses,
              "input_file_sha256": input_hashes,
              "scope_limits": ["Model semantic truth relies on trace-adjudicated fixed oracles; this audit verifies arithmetic and provenance, not an independent human judgement.",
                               "QA05 patch author verifies selected-result integrity and approved counts; this is not an additional independent responsibility-count vote.",
                               "Mac one-worker OS process accounting and simulated speech do not validate Windows commit or physical voice/ASR quality."]}
    output = campaign / "raw-audits/final-independent-review.json"
    if output.exists():
        preserved = output.with_name("final-independent-review-preserved-" + sha(output)[:12] + ".json")
        if not preserved.exists():
            preserved.write_bytes(output.read_bytes())
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2, default=lambda x: sorted(x) if isinstance(x, set) else str(x)) + "\n")
    print(json.dumps({"status": report["status"], "checks": dict(checks), "computed": computed,
                      "report": str(output), "sha256": sha(output)}, ensure_ascii=False))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--campaign", type=Path, required=True)
    main(parser.parse_args().campaign.resolve())
