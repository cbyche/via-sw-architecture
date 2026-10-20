"""Audit saved COMMON9/COMMON_SUPPLEMENT6 comparisons without model calls.

Keep every planned endpoint and adopted-referent judgment. Time differences
use identical observed identities and remain diagnostics, not whole-population
speed or semantic-success scores. No deadline penalties or imputed times.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import statistics


ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "results/architecture-evaluation/current/common-supplement6-20261019"
CONFIGS = ("AAAA", "BAAA", "CAAA", "ABAA", "AABA", "AAAB", "AAAC")
PAIRS = (
    ("DP1", "AAAA", "BAAA"), ("DP1", "AAAA", "CAAA"),
    ("DP1", "BAAA", "CAAA"), ("DP2", "AAAA", "ABAA"),
    ("DP3", "AAAA", "AABA"), ("DP4", "AAAA", "AAAB"),
    ("DP4", "AAAA", "AAAC"), ("DP4", "AAAB", "AAAC"),
)
POPULATIONS = {"COMMON9": ("primary_common9", 45, 215, 115, 75, 5),
               "COMMON_SUPPLEMENT6": ("supplement6", 18, 36, 24, 18, 3)}
# Authored screen-input identities, not a subset chosen by successful output.
SCREEN_INPUTS = {"S08-U01", "S08-U02", "S15-U01"} | {
    f"S{case}-U{turn:02}" for case in (83, 84, 85) for turn in (1, 2)
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text())


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def mean(values):
    return statistics.mean(values) if values else None


def endpoint_value(row, qa):
    value = row.get("latency_ns" if qa == "QA01" else "via_ns")
    observed = row["status"] == "OBSERVED"
    assert observed == (value is not None), (qa, row)
    assert value is None or value >= 0
    return value / 1e9 if observed else None


def phase(row):
    return next((x["phase"] for x in row.get("callback_audit", [])
                 if x["event_id"] == row.get("accepted_callback_id")), None)


def endpoint_summary(rows):
    values = [r["seconds"] for r in rows if r["seconds"] is not None]
    return {"planned": len(rows), "observed": len(values),
            "not_observed": len(rows) - len(values),
            "observed_percent": 100 * len(values) / len(rows),
            "observed_mean_seconds": mean(values),
            "status_counts": dict(Counter(r["status"] for r in rows))}


def compare(dp, population, qa, left, right, lm, rm, ledger):
    assert lm.keys() == rm.keys(), (population, qa, left, right)
    counts = Counter()
    common, reps = [], defaultdict(list)
    for key in sorted(lm):
        l, r = lm[key], rm[key]
        lv, rv = l["seconds"], r["seconds"]
        category = ("both_observed" if lv is not None and rv is not None else
                    "left_only" if lv is not None else
                    "right_only" if rv is not None else "neither_observed")
        counts[category] += 1
        if category == "both_observed":
            common.append((lv, rv))
            reps[key[1]].append(rv - lv)
        if qa == "QA03":
            assert l["start_event"] == r["start_event"]
            assert l["member_events"] == r["member_events"]
        ledger.append({"population": population, "dp": dp, "qa": qa,
                       "left_configuration": left, "right_configuration": right,
                       "episode": key[0], "rep": key[1], "endpoint_id": key[2],
                       "category": category, "left": l, "right": r})
    assert sum(counts.values()) == len(lm)
    raw_l = mean([r["seconds"] for r in lm.values() if r["seconds"] is not None])
    raw_r = mean([r["seconds"] for r in rm.values() if r["seconds"] is not None])
    delta = mean([r - l for l, r in common])
    raw_delta = raw_r - raw_l if raw_l is not None and raw_r is not None else None
    return {"population": population, "dp": dp, "qa": qa, "left": left,
            "right": right, "planned": len(lm),
            **{c: counts[c] for c in ("both_observed", "left_only", "right_only", "neither_observed")},
            "paired_left_mean_seconds": mean([l for l, _ in common]),
            "paired_right_mean_seconds": mean([r for _, r in common]),
            "paired_right_minus_left_seconds": delta,
            "paired_difference_by_repeat_seconds": {str(k): mean(v) for k, v in sorted(reps.items())},
            "unpaired_right_minus_left_seconds": raw_delta,
            "unpaired_order_reverses_on_pairing": raw_delta * delta < 0 if delta is not None and raw_delta is not None else None,
            "whole_population_speed_winner": None,
            "semantic_correctness_required_by_endpoint_filter": False}


def render_tables(summary, output):
    labels = {"AAAA": "공통 A", "BAAA": "DP1 B", "CAAA": "DP1 C",
              "ABAA": "DP2 B", "AABA": "DP3 B", "AAAB": "DP4 B", "AAAC": "DP4 C"}
    names = {"COMMON9": "대표 9개", "COMMON_SUPPLEMENT6": "추가 조건 6개"}
    lines = ["# 동일 신원의 응답과 목표 비교", "",
             "기존 441개 실행을 재분석했다. 추가 모델 호출은 0회다. 두 모집단을 합산하지 않는다.", "",
             "관측은 callback 또는 host 완료 endpoint의 존재다. 의미상 정답이나 실제 업무 성공을 보증하지 않는다. 시간은 초이며, 미관측에 가짜 시간을 넣지 않았다.", ""]
    for pop, cfgs in summary["populations"].items():
        lines += [f"## {names[pop]} 전체 예정 대상", "",
                  "| 구성 | 첫 텍스트 관측 | 미응답 | 완료 endpoint 관측 | 미완료 | 지칭 정답 | API USD/묶음 | 로컬 RSS MB |",
                  "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
        for c, v in cfgs.items():
            a, b, q = v["qa01"], v["qa03"], v["qa02"]
            mark = "*" if v["qa06"]["incomplete_runs"] else ""
            lines.append(f"| {labels[c]} | {a['observed']}/{a['planned']} | {a['not_observed']} | {b['observed']}/{b['planned']} | {b['not_observed']} | {q['correct']}/{q['planned']} | {v['qa05_api_usd_per_bundle']:.6f} | {v['qa06']['observed_peak_mb']:.3f}{mark} |")
        lines += ["", "`*` 불완전 RSS 표본이 포함된 구성의 관측 하한이다.", ""]
        for qa in ("QA01", "QA03"):
            lines += [f"### {qa}: 같은 신원의 짝 비교", "",
                      "양쪽 관측, 왼쪽만, 오른쪽만, 양쪽 미관측의 합이 전체 예정 분모다. 마지막 두 시간은 양쪽 관측한 **동일 신원**만 비교하는 진단이며 전체 속도 순위가 아니다.", "",
                      "| DP와 왼쪽/오른쪽 | 예정 | 양쪽 관측 | 왼쪽만 | 오른쪽만 | 양쪽 미관측 | 같은 건 왼쪽 초 | 같은 건 오른쪽 초 |",
                      "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
            for p in summary["endpoint_pairs"]:
                if p["population"] != pop or p["qa"] != qa:
                    continue
                index = int(p["dp"][-1]) - 1
                pair_label = f"{p['dp']} {p['left'][index]}/{p['right'][index]}"
                lines.append(f"| {pair_label} | {p['planned']} | {p['both_observed']} | {p['left_only']} | {p['right_only']} | {p['neither_observed']} | {p['paired_left_mean_seconds']:.3f} | {p['paired_right_mean_seconds']:.3f} |")
            lines.append("")
        lines += ["### QA02: 같은 지칭 판정의 성공과 실패", "",
                  "| DP와 왼쪽/오른쪽 | 예정 | 양쪽 정답 | 왼쪽만 정답 | 오른쪽만 정답 | 양쪽 실패 |",
                  "| --- | ---: | ---: | ---: | ---: | ---: |"]
        for p in summary["qa02_pairs"]:
            if p["population"] == pop:
                index = int(p["dp"][-1]) - 1
                pair_label = f"{p['dp']} {p['left'][index]}/{p['right'][index]}"
                lines.append(f"| {pair_label} | {p['planned']} | {p.get('both_correct', 0)} | {p.get('left_only_correct', 0)} | {p.get('right_only_correct', 0)} | {p.get('both_failed', 0)} |")
        lines += ["", "### 모든 사례별 결과", "",
                  "사례별 행은 주 점수를 교체하지 않는 원인 진단이다. 첫 텍스트와 완료의 시간은 각각 그 행의 관측된 건에만 존재한다.", "",
                  "| 구성 | 사례 | 첫 텍스트 관측 | 관측 평균 초 | 완료 관측 | 관측 평균 초 | 지칭 정답 |",
                  "| --- | --- | ---: | ---: | ---: | ---: | ---: |"]
        for c, v in cfgs.items():
            for case, q1 in v["qa01"]["by_scenario"].items():
                q3 = v["qa03"]["by_scenario"][case]
                ref = v["qa02"]["by_scenario"].get(case)
                t1 = f"{q1['observed_mean_seconds']:.3f}" if q1["observed"] else "시간 없음"
                t3 = f"{q3['observed_mean_seconds']:.3f}" if q3["observed"] else "시간 없음"
                q2 = f"{ref['correct']}/{ref['planned']}" if ref else "비적용"
                lines.append(f"| {labels[c]} | {case} | {q1['observed']}/{q1['planned']} | {t1} | {q3['observed']}/{q3['planned']} | {t3} | {q2} |")
        lines.append("")
    (output / "tables.md").write_text("\n".join(lines).rstrip() + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=BASE / "comparison-audit-v1")
    args = parser.parse_args()
    analysis_path = BASE / "joint-recovery184-v1/postanalysis/recovery-pool-v1/analysis.json"
    decisions_path = BASE / "qa02-regrade-v1/decisions.json"
    qa04_path = ROOT / "results/architecture-evaluation/current/common9-poc-20261019/qa04-method-v3-count-classification/CLASSIFICATION-RESULTS.json"
    analysis, decisions, qa04 = load(analysis_path), load(decisions_path), load(qa04_path)
    parent_path = BASE / "joint-formal-v1/postanalysis/stopped258-input-assembly-v1/joint-analysis.json"
    assert digest(parent_path) == analysis["parent_analysis_sha256"]
    parent = load(parent_path)
    trace_hashes = {r["run_id"]: r["sha256"] for r in
                    parent["trace_provenance"] + analysis["recovery_trace_provenance"]}
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    summary = {"version": "COMMON-COMPARISON-AUDIT-v1", "additional_model_calls": 0,
               "new_measurement_campaign": False, "original_metrics_modified": False,
               "new_scores_or_imputed_times": False, "populations": {},
               "endpoint_pairs": [], "qa02_pairs": [], "qa04": {},
               "all_first_correct_responses_scored": False,
               "all_semantically_correct_goal_completions_scored": False}
    provenance = {"sources": [{"path": str(p.relative_to(ROOT)), "sha256": digest(p)}
                               for p in (analysis_path, parent_path, decisions_path, qa04_path)],
                  "script_sha256": digest(Path(__file__)), "traces": []}
    all_endpoints, endpoint_ledger, referent_ledger = {}, [], []
    unique_runs = set()
    selected = []
    for population, (section, episodes, n_response, n_goal, n_ref, repeats) in POPULATIONS.items():
        runs = analysis[section]["results"]
        assert len(runs) == episodes * len(CONFIGS)
        assert len({(r["configuration"], r["episode"], r["rep"]) for r in runs}) == len(runs)
        result = {}
        for cfg in CONFIGS:
            rs = sorted((r for r in runs if r["configuration"] == cfg), key=lambda r: (r["episode"], r["rep"]))
            assert len(rs) == episodes
            stages = defaultdict(lambda: {"calls": 0, "cost_usd": 0.0, "input_tokens": 0, "output_tokens": 0})
            errors, direct_screen_inputs = Counter(), set()
            for r in rs:
                assert r["run_id"] not in unique_runs
                unique_runs.add(r["run_id"])
                matches = list(BASE.glob(f"*/raw/runs/{r['run_id']}/trace.json"))
                assert len(matches) == 1, r["run_id"]
                path = matches[0]
                t = load(path)
                assert (t["configuration"], t["episode"], t["rep"]) == (cfg, r["episode"], r["rep"])
                # The immutable capture can retain its pre-analysis PENDING label.
                # Certification is the finalized evaluator's episode eligibility.
                assert r["primary_measurement_eligibility"] == "ELIGIBLE"
                sha = digest(path)
                assert sha == trace_hashes[r["run_id"]], r["run_id"]
                provenance["traces"].append({"run_id": r["run_id"], "path": str(path.relative_to(ROOT)), "sha256": sha})
                total = 0.0
                for call in t["calls"]:
                    stage = stages[call["stage"]]
                    stage["calls"] += 1
                    stage["cost_usd"] += call.get("cost_usd") or 0
                    stage["input_tokens"] += call.get("input_tokens") or 0
                    stage["output_tokens"] += call.get("output_tokens") or 0
                    total += call.get("cost_usd") or 0
                    if call["stage"].startswith("grounding."):
                        direct_screen_inputs.update(g[6:] for g in call.get("causal_trace", {}).get("goal_ids", []) if g.startswith("input:"))
                assert abs(total - r["qa05"]["known_api_cost_usd"]) < 1e-8
                for error in r["functional_failures"]:
                    errors[error.get("data", {}).get("reason", "UNCLASSIFIED")] += 1
            v = {"episodes": episodes, "repeats": repeats, "qa01": {}, "qa03": {},
                 "qa05_api_usd_per_bundle": sum(r["qa05"]["known_api_cost_usd"] for r in rs) / repeats,
                 "qa05_all_calls_by_stage": dict(sorted(stages.items())),
                 "workflow_failure_reason_counts": dict(errors),
                 "actual_screen_input_ids": sorted(direct_screen_inputs),
                 "semantic_body_review_status": dict(Counter(x["status"] for r in rs for x in r["semantic_diagnostics"]["body_conditions"]))}
            for qa, field, identity, planned in (("QA01", "qa01", "event_id", n_response), ("QA03", "qa03", "id", n_goal)):
                mapping = {}
                for r in rs:
                    for q in r[field]:
                        key = (r["episode"], r["rep"], q[identity])
                        assert key not in mapping
                        entry = {"run_id": r["run_id"], "status": q["status"], "seconds": endpoint_value(q, qa)}
                        if qa == "QA01":
                            entry.update(input_class="user" if q[identity].split("-")[1].startswith("U") else "agent",
                                         first_phase=phase(q), accepted_callback_id=q.get("accepted_callback_id"),
                                         expected_content=q["required_content"], actual_first_text=q.get("first_accepted_raw_text"))
                        else:
                            entry.update(start_event=q["start_event"], member_events=q["member_events"],
                                         accepted_terminal_event_id=q.get("accepted_terminal_event_id"))
                        mapping[key] = entry
                assert len(mapping) == planned
                all_endpoints[population, cfg, qa] = mapping
                v[field] = endpoint_summary(list(mapping.values()))
                v[field]["by_scenario"] = {case: endpoint_summary([e for k, e in mapping.items() if k[0] == case]) for case in sorted({k[0] for k in mapping})}
                v[field]["by_endpoint_id"] = {eid: endpoint_summary([e for k, e in mapping.items() if k[2] == eid]) for eid in sorted({k[2] for k in mapping})}
                if qa == "QA01":
                    v[field]["by_input_class"] = {cl: endpoint_summary([e for e in mapping.values() if e["input_class"] == cl]) for cl in ("user", "agent") if any(e["input_class"] == cl for e in mapping.values())}
                    v[field]["first_phase_counts"] = dict(Counter(e["first_phase"] for e in mapping.values() if e["seconds"] is not None))
                    v[field]["authored_screen_inputs"] = endpoint_summary([e for k, e in mapping.items() if k[2] in SCREEN_INPUTS])
            ds = [d for d in decisions if d["population"] == population and d["configuration"] == cfg]
            assert len(ds) == n_ref
            assert len({(d["episode"], d["rep"], d["binding_id"]) for d in ds}) == n_ref
            v["qa02"] = {"planned": n_ref, "correct": sum(d["correct"] for d in ds),
                         "accuracy_percent": 100 * sum(d["correct"] for d in ds) / n_ref,
                         "reason_counts": dict(Counter(d["reason"] for d in ds)),
                         "by_input_id": {eid: {"planned": len(xs := [d for d in ds if d["input_id"] == eid]), "correct": sum(d["correct"] for d in xs)} for eid in sorted({d["input_id"] for d in ds})},
                         "by_scenario": {case: {"planned": len(xs := [d for d in ds if d["episode"] == case]), "correct": sum(d["correct"] for d in xs)} for case in sorted({d["episode"] for d in ds})}}
            peak = max(rs, key=lambda r: r["qa06"]["peak_concurrent_sum_bytes"])
            mem = peak["qa06"]
            v["qa06"] = {"observed_peak_mb": mem["peak_concurrent_sum_bytes"] / 1e6,
                         "peak_run_id": peak["run_id"], "peak_episode": peak["episode"],
                         "roles_at_peak_bytes": mem["peak_same_sample"]["rss_bytes_by_role"],
                         "incomplete_runs": [r["run_id"] for r in rs if r["qa06"].get("peak_is_observed_lower_bound_if_incomplete") or r["qa06"]["status"] != "OBSERVED"],
                         "cloud_model_memory_included": False, "target_windows_commit_memory": False}
            result[cfg] = v
            selected.extend(rs)
        summary["populations"][population] = result
        for dp, left, right in PAIRS:
            for qa in ("QA01", "QA03"):
                summary["endpoint_pairs"].append(compare(dp, population, qa, left, right, all_endpoints[population, left, qa], all_endpoints[population, right, qa], endpoint_ledger))
            maps = [{(d["episode"], d["rep"], d["binding_id"]): d for d in decisions if d["population"] == population and d["configuration"] == c} for c in (left, right)]
            assert maps[0].keys() == maps[1].keys()
            counts = Counter()
            for key in sorted(maps[0]):
                l, r = (m[key] for m in maps)
                category = "both_correct" if l["correct"] and r["correct"] else "left_only_correct" if l["correct"] else "right_only_correct" if r["correct"] else "both_failed"
                counts[category] += 1
                referent_ledger.append({"population": population, "dp": dp, "left_configuration": left,
                                       "right_configuration": right, "episode": key[0], "rep": key[1], "binding_id": key[2],
                                       "category": category, "left_decision_key": l["key"], "right_decision_key": r["key"],
                                       "left_reason": l["reason"], "right_reason": r["reason"]})
            summary["qa02_pairs"].append({"population": population, "dp": dp, "left": left, "right": right,
                                         "planned": len(maps[0]), **dict(counts)})
    for cfg in CONFIGS:
        counts = qa04["all_counts_by_configuration"][cfg]
        assert counts == [1, 1, 3, 3, 1, 2]
        summary["qa04"][cfg] = {"tasks": 6, "changed_elements_by_task": counts,
                               "mean_changed_elements": sum(counts) / 6,
                               "producer_specific_change_coverage_established": False}
    assert len(unique_runs) == 441
    assert len(decisions) == 651
    assert len(endpoint_ledger) == 3120
    assert len(referent_ledger) == 744
    assert len(summary["endpoint_pairs"]) == 32
    selected_cost = sum(r["qa05"]["known_api_cost_usd"] for r in selected)
    assert abs(selected_cost - 11.9452128) < 1e-8
    summary["verification"] = {"selected_certified_runs": len(unique_runs), "qa02_rows": len(decisions),
                               "endpoint_pair_rows": len(endpoint_ledger), "qa02_pair_rows": len(referent_ledger),
                               "selected_run_api_cost_usd": selected_cost, "all_identity_and_count_assertions_passed": True}
    for name, rows in (("endpoint-pairs.jsonl", endpoint_ledger), ("qa02-pairs.jsonl", referent_ledger)):
        (output / name).write_text("".join(json.dumps(r, ensure_ascii=False, separators=(",", ":")) + "\n" for r in rows))
    save(output / "summary.json", summary)
    save(output / "provenance.json", provenance)
    render_tables(summary, output)
    print(json.dumps(summary["verification"], ensure_ascii=False))


if __name__ == "__main__":
    main()
