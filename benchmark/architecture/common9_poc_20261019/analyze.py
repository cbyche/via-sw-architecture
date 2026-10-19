"""Report COMMON9 evaluations without pooling phases or choosing good repeats."""
import argparse
import json
import statistics
from pathlib import Path
from evaluate import CONFIGURATIONS, evaluate, summarize


def paired_diagnostics(results):
    pairs = []
    for phase in sorted({r.get("phase") for r in results}, key=str):
        for configuration in CONFIGURATIONS[1:]:
            baseline = [r for r in results if r.get("phase") == phase and r.get("configuration") == "AAAA"]
            candidate = [r for r in results if r.get("phase") == phase and r.get("configuration") == configuration]
            index = {}
            for label, rows in (("baseline", baseline), ("candidate", candidate)):
                for row in rows:
                    index.setdefault((row["episode"], row["rep"]), {}).setdefault(label, []).append(row)
            comparison = {"phase": phase, "configuration": configuration, "baseline": "AAAA", "episodes": []}
            for key, sides in sorted(index.items()):
                item = {"episode": key[0], "rep": key[1]}
                if len(sides.get("baseline", [])) != 1 or len(sides.get("candidate", [])) != 1:
                    item["status"] = "MISSING_OR_DUPLICATE_SIDE"
                    comparison["episodes"].append(item)
                    continue
                a, b = sides["baseline"][0], sides["candidate"][0]
                item["status"] = "PAIRED"
                for metric, identifier, value in (("qa01", "response_id", "latency_ns"), ("qa03", "id", "via_ns")):
                    ai = {x[identifier]: x for x in a[metric]}
                    bi = {x[identifier]: x for x in b[metric]}
                    paired = []
                    for rid in sorted(set(ai) | set(bi)):
                        av, bv = ai.get(rid, {}).get(value), bi.get(rid, {}).get(value)
                        paired.append({"id": rid, "baseline_ns": av, "candidate_ns": bv,
                            "candidate_minus_baseline_ns": bv-av if av is not None and bv is not None else None,
                            "baseline_status": ai.get(rid, {}).get("status"),
                            "candidate_status": bi.get(rid, {}).get("status")})
                    item[metric] = paired
                item["qa02"] = {"baseline_correct": sum(x["correct"] for x in a["qa02"]),
                    "candidate_correct": sum(x["correct"] for x in b["qa02"]), "planned": len(a["qa02"])}
                item["qa05"] = {"baseline": a["qa05"], "candidate": b["qa05"]}
                comparison["episodes"].append(item)
            for metric in ("qa01", "qa03"):
                values = [v["candidate_minus_baseline_ns"] for e in comparison["episodes"] for v in e.get(metric, [])
                          if v["candidate_minus_baseline_ns"] is not None]
                comparison[metric + "_intersection_only"] = {"paired_observations": len(values),
                    "mean_candidate_minus_baseline_ms": statistics.mean(values)/1e6 if values else None,
                    "scope": "auxiliary observed intersection; does not replace fixed planned denominators"}
            pairs.append(comparison)
    return pairs


def report_markdown(summary):
    lines = ["# COMMON9 evaluation summary", "",
        "Latency means cover observed endpoints only; planned denominators and missing observations remain visible. "
        "OFFLINE/doubles and scaled timing are preparation diagnostics. Semantic body/Task/handoff review is separate from QA values.", "",
        "| Phase/config | Episodes attempted/planned | QA01 observed/planned; observed mean ms | QA02 correct/planned; judged | QA03 observed/planned; observed mean ms | QA05 status; USD | QA06 status; peak MB |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
    def number(value):
        return f"{value:.6f}" if value is not None else "NOT_MEASURED"
    for key, row in summary.items():
        q1, q2, q3, q5, q6 = [row[k] for k in ("qa01", "qa02", "qa03", "qa05", "qa06")]
        memory = q6["maximum_same_sample_rss_bytes"]
        cost = (f"{q5['status']}; bundle {number(q5['mean_nine_episode_bundle_usd'])}; "
            f"known total {number(q5['known_total_usd'])}; unknown {q5['unknown_calls']}; "
            f"reserved {number(q5['unknown_reservation_usd'])}")
        memory_status = q6["status"]
        lower_bound = "≥" if memory is not None and memory_status != "COMPLETE_POPULATION" else ""
        memory_text = (f"{memory_status}; {lower_bound}{number(memory/1e6 if memory is not None else None)}; "
            f"invalid samples {q6['invalid_samples']}; startup zeros {q6['startup_zero_rss_samples']}; exit boundaries {q6['confirmed_exit_boundary_samples']}; sampler failures {q6['sampler_failures']}")
        lines.append(f"| {key} | {row['episodes_observed']}/{row['episodes_planned']} | "
            f"{q1['observed']}/{q1['planned']}; {number(q1['observed_partial_mean_ms'])} | "
            f"{q2['correct']}/{q2['planned']}; {q2['judgments_observed']} | {q3['observed']}/{q3['planned']}; {number(q3['observed_partial_mean_ms'])} | "
            f"{cost} | {memory_text} |")
    lines += ["", "The ≥ memory marker is an observed lower bound because observations or the planned population are incomplete. "
        "Partial observed timing means are not whole-population success speeds. QA05 unknown charges remain reserved, not zero. "
        "Detailed usage, RSS incomplete samples and semantic review ledgers are retained in analysis.json. "
        "QA04 is supplied by its independent six-change exercise and is not inferred here. "
        "No score-band revision or aggregate winner is inferred from these values.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("runs", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--repetitions", type=int, default=5)
    args = parser.parse_args()
    traces = [json.loads(line) for line in args.runs.read_text().splitlines() if line.strip()]
    results = [evaluate(trace) for trace in traces]
    summary = summarize(results, args.repetitions)
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "analysis.json").write_text(json.dumps({"summary": summary,
        "paired_case_level_diagnostics": paired_diagnostics(results), "results": results},ensure_ascii=False,indent=2)+"\n")
    (args.output / "summary.md").write_text(report_markdown(summary))


if __name__ == "__main__":
    main()
