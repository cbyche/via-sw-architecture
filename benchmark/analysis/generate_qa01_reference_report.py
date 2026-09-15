#!/usr/bin/env python3
"""Generate DP-00 QA-01 design-time reference latency evidence.

This simulator consumes committed semantic scenarios, architecture traces, and
latency provenance. It never calls a model or external service and does not
produce or claim measured production traces.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import random
from collections import Counter, defaultdict
from pathlib import Path
from typing import Iterable


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CORPUS = REPO_ROOT / "benchmark/qa01-reference-v1/scenarios.json"
DEFAULT_TRACES = REPO_ROOT / "benchmark/qa01-reference-v1/traces.json"
DEFAULT_PLANS = REPO_ROOT / "benchmark/qa01-reference-v1/behavior-plans.json"
DEFAULT_ORACLES = REPO_ROOT / "benchmark/qa01-reference-v1/oracles.json"
DEFAULT_PROFILE = REPO_ROOT / "benchmark/contracts/dp00-qa01-design-reference-latency-v1.json"
DEFAULT_JSON = REPO_ROOT / "results/reports/qa01-reference-v1/campaign-report-data.json"
DEFAULT_MD = REPO_ROOT / "results/reports/qa01-reference-v1/campaign-report.md"
Z95 = 1.6448536269514722


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def nearest_rank(values: Iterable[float], percentile: float) -> float:
    ordered = sorted(values)
    if not ordered:
        raise ValueError("percentile requires samples")
    return ordered[max(1, math.ceil(percentile * len(ordered))) - 1]


def stats(values: list[float]) -> dict:
    return {
        "realization_count": len(values),
        "p50_ms": round(nearest_rank(values, 0.50), 3),
        "p95_ms": round(nearest_rank(values, 0.95), 3),
        "p99_ms": round(nearest_rank(values, 0.99), 3),
    }


def p95_ranking(alternatives: dict) -> str:
    """Render a ranking without converting exact reported ties into an order."""
    ordered = sorted(alternatives, key=lambda alt: (alternatives[alt]["p95_ms"], alt))
    groups: list[list[str]] = []
    for alternative in ordered:
        if not groups or alternatives[groups[-1][0]]["p95_ms"] != alternatives[alternative]["p95_ms"]:
            groups.append([alternative])
        else:
            groups[-1].append(alternative)
    return " < ".join(" = ".join(group) for group in groups)


def relation(left: str, left_value: float, right: str, right_value: float) -> str:
    if left_value < right_value:
        return f"{left} < {right}"
    if right_value < left_value:
        return f"{right} < {left}"
    return f"{left} = {right}"


def trace_decomposition(mapping: dict) -> dict:
    model_operations = [item.split(":", 1)[1] for item in mapping["trace"] if item.startswith("M:")]
    return {
        "path": mapping["path"],
        "model_generation_count": len(model_operations),
        "model_operations": model_operations,
        "handoff_count": mapping["trace"].count("H"),
        "effect_count": mapping["trace"].count("X"),
    }


def derive_architecture_views(primary: dict, sensitivity: dict, traces: dict) -> dict:
    stage1_profiles = {}
    stage1_sensitivity = {}
    stage2_profiles = {}
    for profile_id, alternatives in primary.items():
        stage1_profiles[profile_id] = {}
        for alias, source in (("A0", "A"), ("B0", "B")):
            source_data = alternatives[source]
            stage1_profiles[profile_id][alias] = {
                "source_candidate": source,
                "overall": source_data["statistics"],
                "categories": source_data["category_statistics"],
            }

        stage2_profiles[profile_id] = {}
        for alias, source in (("C", "C"), ("B1", "B"), ("D1", "D")):
            source_data = alternatives[source]
            stage2_profiles[profile_id][alias] = {
                "source_candidate": source,
                "f1_primary": source_data["category_statistics"]["F1"],
                "f2_secondary": source_data["category_statistics"]["F2"],
                "f3_secondary": source_data["category_statistics"]["F3"],
                "pooled_secondary": source_data["statistics"],
            }

    for profile_id, cells in sensitivity.items():
        stage1_sensitivity[profile_id] = []
        for cell in cells:
            a = cell["alternatives"]["A"]["p95_ms"]
            b = cell["alternatives"]["B"]["p95_ms"]
            stage1_sensitivity[profile_id].append({
                "handoff_ms": cell["handoff_ms"],
                "effect_multiplier": cell["effect_multiplier"],
                "A0_p95_ms": a,
                "B0_p95_ms": b,
                "relation": relation("A0", a, "B0", b),
                "source": "existing pooled A/B sensitivity cell",
            })

    f1_traces = traces["category_traces"]["F1"]
    stage2_decomposition = {
        "C": trace_decomposition(f1_traces["C"]),
        "B1": trace_decomposition(f1_traces["B"]),
        "D1": trace_decomposition(f1_traces["D"]),
    }
    stage1_decomposition = {
        category: {
            "A0": trace_decomposition(traces["category_traces"][category]["A"]),
            "B0": trace_decomposition(traces["category_traces"][category]["B"]),
        }
        for category in ("F1", "F2", "F3")
    }

    return {
        "stage1": {
            "question": "primary_execution_boundary",
            "candidates": ["A0", "B0"],
            "candidate_result_mapping": {"A0": "primary.<profile>.A", "B0": "primary.<profile>.B"},
            "interpretation": "A0 keeps top-level semantic routing in VIA; B0 moves primary interpretation and initial execution/delegation authority into ARGO.",
            "d0_disposition": "Not retained. Removing VIA_FAST collapses D0's architecture-level distinction from A0 under the current responsibility, state, follow-up, and route-commit specification.",
            "profiles": stage1_profiles,
            "trace_decomposition": stage1_decomposition,
            "handoff_effect_sensitivity": stage1_sensitivity,
        },
        "stage2": {
            "question": "bounded_fast_capability_accommodation",
            "candidates": ["C", "B1", "D1"],
            "candidate_result_mapping": {"C": "primary.<profile>.C", "B1": "primary.<profile>.B", "D1": "primary.<profile>.D"},
            "interpretation": "C uses deterministic VIA-local eligibility, B1 keeps bounded capability execution inside ARGO's authority, and D1 uses one semantic selector across VIA Fast, ARGO, and Specialist topologies.",
            "primary_population": "F1",
            "secondary_populations": ["F2", "F3", "pooled F1/F2/F3"],
            "profiles": stage2_profiles,
            "f1_trace_decomposition": stage2_decomposition,
            "sensitivity_interpretation": {
                "handoff": "C and D1 F1 have zero H charges; B1 F1 has one H charge. At X=1, B1 p95 shifts exactly by H-reference delta while C and D1 do not.",
                "effect": "The identical keyed X realization and multiplier apply to C, B1, and D1; X creates no candidate-specific latency discount and cancels from paired per-realization differences.",
                "c_vs_d1": "C and D1 have the same V and X primitives and no H. D1 adds EXECUTION_ROUTE_DECISION, so the C-versus-D1 F1 difference is insensitive to H and X under this contract.",
            },
        },
        "cross_stage_aggregate": {
            "status": "SECONDARY_DIAGNOSTIC",
            "source": "primary and sensitivity",
            "warning": "The pooled A/B/C/D view mixes primary execution-boundary placement with bounded fast-capability accommodation and is not a primary architecture-selection basis.",
        },
    }


def keyed_rng(seed: int, *parts: object) -> random.Random:
    material = "|".join([str(seed), *(str(part) for part in parts)]).encode()
    derived = int.from_bytes(hashlib.sha256(material).digest()[:8], "big")
    return random.Random(derived)


def lognormal_from_percentiles(rng: random.Random, p50: float, p95: float) -> float:
    if p50 <= 0 or p95 < p50:
        raise ValueError("invalid log-normal percentile parameters")
    mu = math.log(p50)
    sigma = math.log(p95 / p50) / Z95
    return rng.lognormvariate(mu, sigma)


def validate_contracts(corpus: dict, traces: dict, profile: dict) -> dict:
    scenarios = corpus["scenarios"]
    ids = [item["id"] for item in scenarios]
    if len(ids) != len(set(ids)):
        raise ValueError("scenario ids must be unique")
    categories = Counter(item["category"] for item in scenarios)
    if categories != Counter({"F1": 12, "F2": 12, "F3": 12}):
        raise ValueError(f"expected 12 scenarios per category, got {categories}")
    if len(scenarios) != corpus["semantic_scenario_count"]:
        raise ValueError("semantic scenario count mismatch")
    if {item["effect_class"] for item in scenarios} - {"X1", "X2", "X3"}:
        raise ValueError("unknown effect class")
    if any("specialist" not in item for item in scenarios if item["category"] == "F3"):
        raise ValueError("F3 scenarios require specialist capability facts")

    allowed = {"V", "H", "X", *(f"M:{name}" for name in profile["operations"])}
    coverage = Counter()
    for category, alternatives in traces["category_traces"].items():
        if set(alternatives) != {"A", "B", "C", "D"}:
            raise ValueError(f"{category} lacks A/B/C/D traces")
        scenario_count = categories[category]
        for alternative, mapping in alternatives.items():
            unknown = set(mapping["trace"]) - allowed
            if unknown:
                raise ValueError(f"unknown primitives: {unknown}")
            if mapping["trace"].count("V") != 1 or mapping["trace"].count("X") != 1:
                raise ValueError("every trace requires exactly one V and X")
            coverage[f"{alternative}:{mapping['path']}"] += scenario_count
    failures = {
        key: {"required": required, "observed": coverage[key]}
        for key, required in traces["coverage_gates"].items()
        if coverage[key] < required
    }
    if failures:
        raise ValueError(f"defining-path coverage failed: {failures}")

    models = profile["models"]
    if len(models) != 3 or len({item["profile_id"] for item in models}) != 3:
        raise ValueError("exactly three unique model profiles are required")
    if profile["common_primitives"]["V"]["reference_ms"] != 500:
        raise ValueError("V must remain fixed at 500 ms")
    return {"categories": dict(categories), "path_observations": dict(sorted(coverage.items()))}


def validate_supporting_contracts(corpus: dict, plans: dict, oracles: dict) -> None:
    expected = defaultdict(set)
    for scenario in corpus["scenarios"]:
        expected[scenario["category"]].add(scenario["id"])
    actual = {key: set(value) for key, value in plans["scenario_bindings"].items()}
    if dict(expected) != actual:
        raise ValueError("behavior-plan scenario bindings do not match corpus")
    if oracles["applies_to"] != corpus["corpus_id"]:
        raise ValueError("oracle does not apply to corpus")
    if "topology" not in oracles["architecture_neutrality"].lower():
        raise ValueError("oracle must state topology neutrality")


def model_latency_ms(
    model: dict, operation: str, token_count: int, seed: int, scenario_id: str,
    realization: int, occurrence: int,
) -> float:
    ttft = lognormal_from_percentiles(
        keyed_rng(seed, model["profile_id"], scenario_id, realization, "TTFT", operation, occurrence),
        model["ttft_p50_ms"], model["ttft_p95_ms"],
    )
    tpot = lognormal_from_percentiles(
        keyed_rng(seed, model["profile_id"], scenario_id, realization, "TPOT", operation, occurrence),
        model["tpot_p50_ms"], model["tpot_p95_ms"],
    )
    return ttft + (token_count - 1) * tpot


def realize_trace(
    scenario: dict, trace: list[str], model: dict, contract: dict,
    realization: int, handoff_ms: float, effect_multiplier: float,
) -> float:
    seed = contract["random_seed"]
    total = 0.0
    occurrences: Counter[str] = Counter()
    for primitive in trace:
        occurrence = occurrences[primitive]
        occurrences[primitive] += 1
        if primitive == "V":
            total += contract["common_primitives"]["V"]["reference_ms"]
        elif primitive == "H":
            total += handoff_ms
        elif primitive == "X":
            effect = contract["common_primitives"][scenario["effect_class"]]
            total += effect_multiplier * lognormal_from_percentiles(
                keyed_rng(seed, scenario["id"], realization, scenario["effect_class"]),
                effect["p50_ms"], effect["p95_ms"],
            )
        elif primitive.startswith("M:"):
            operation = primitive.split(":", 1)[1]
            total += model_latency_ms(
                model, operation, contract["operations"][operation]["output_tokens"],
                seed, scenario["id"], realization, occurrence,
            )
        else:
            raise ValueError(f"unsupported primitive {primitive}")
    return total


def simulate(corpus: dict, traces: dict, contract: dict, plans: dict | None = None, oracles: dict | None = None) -> dict:
    validation = validate_contracts(corpus, traces, contract)
    if plans is not None and oracles is not None:
        validate_supporting_contracts(corpus, plans, oracles)
        validation["supporting_contracts"] = "PASS"
    n = contract["latency_realizations_per_scenario_alternative_profile"]
    handoff_ref = contract["common_primitives"]["H"]["reference_ms"]
    primary = {}
    sensitivity = {}

    for model in contract["models"]:
        profile_id = model["profile_id"]
        primary[profile_id] = {}
        for alternative in "ABCD":
            overall: list[float] = []
            by_category: dict[str, list[float]] = defaultdict(list)
            by_path: dict[str, list[float]] = defaultdict(list)
            for scenario in corpus["scenarios"]:
                mapping = traces["category_traces"][scenario["category"]][alternative]
                for realization in range(n):
                    latency = realize_trace(
                        scenario, mapping["trace"], model, contract, realization,
                        handoff_ref, 1.0,
                    )
                    overall.append(latency)
                    by_category[scenario["category"]].append(latency)
                    by_path[mapping["path"]].append(latency)
            primary[profile_id][alternative] = {
                "semantic_scenario_count": len(corpus["scenarios"]),
                "statistics": stats(overall),
                "category_statistics": {key: stats(value) for key, value in sorted(by_category.items())},
                "path_statistics": {key: stats(value) for key, value in sorted(by_path.items())},
            }

        sensitivity[profile_id] = []
        for handoff_ms in contract["common_primitives"]["H"]["sensitivity_ms"]:
            for effect_multiplier in contract["effect_sensitivity_multipliers"]:
                cell = {"handoff_ms": handoff_ms, "effect_multiplier": effect_multiplier, "alternatives": {}}
                for alternative in "ABCD":
                    values = []
                    for scenario in corpus["scenarios"]:
                        mapping = traces["category_traces"][scenario["category"]][alternative]
                        values.extend(
                            realize_trace(
                                scenario, mapping["trace"], model, contract, realization,
                                handoff_ms, effect_multiplier,
                            )
                            for realization in range(n)
                        )
                    cell["alternatives"][alternative] = stats(values)
                cell["p95_order"] = sorted(
                    "ABCD", key=lambda alt: cell["alternatives"][alt]["p95_ms"]
                )
                cell["p95_ranking"] = p95_ranking(cell["alternatives"])
                sensitivity[profile_id].append(cell)

    robustness = {}
    for profile_id, cells in sensitivity.items():
        orders = Counter(cell["p95_ranking"] for cell in cells)
        robustness[profile_id] = {
            "distinct_p95_orders": len(orders),
            "orders": dict(sorted(orders.items())),
            "stable_across_handoff_and_effect_range": len(orders) == 1,
        }

    return {
        "report_id": "dp00-qa01-design-reference-v1",
        "status": "DESIGN_TIME_REFERENCE_NOT_PRODUCTION_MEASUREMENT",
        "baseline_commit": "5f00cea12fadeaf5dc5508691c0d4bfbe080ceaa",
        "method": {
            "ftol_boundary": "Acoustic EOS to earliest Useful Outcome",
            "design_reference_mix": corpus["design_reference_mix"],
            "semantic_scenario_count": len(corpus["scenarios"]),
            "latency_realizations_per_scenario_alternative_profile": n,
            "primary_total_realizations": len(corpus["scenarios"]) * 4 * len(contract["models"]) * n,
            "percentile_estimator": "nearest-rank",
            "synthetic_data_warning": "Latency realizations are evidence-anchored synthetic reference samples, not measured traces or different tasks.",
        },
        "validation": validation,
        "architecture_views": derive_architecture_views(primary, sensitivity, traces),
        "primary": primary,
        "sensitivity": sensitivity,
        "robustness": robustness,
        "provenance": {
            "latency_contract": "benchmark/contracts/dp00-qa01-design-reference-latency-v1.json",
            "scenario_contract": "benchmark/qa01-reference-v1/scenarios.json",
            "behavior_plan_contract": "benchmark/qa01-reference-v1/behavior-plans.json",
            "oracle_contract": "benchmark/qa01-reference-v1/oracles.json",
            "trace_contract": "benchmark/qa01-reference-v1/traces.json",
            "random_seed": contract["random_seed"],
            "levels": contract["provenance_levels"],
        },
        "limitations": [
            "No production system, production workload telemetry, or live model calls were used.",
            "Official Qwen aggregate generation speed is transformed into a first-order TPOT approximation; it is not a published TTFT/TPOT trace.",
            "TTFT percentiles, TPOT variability, handoff, effect distributions, and output-token budgets are explicit engineering assumptions.",
            "The balanced F1/F2/F3 Design Reference Mix v1 is not a production workload mix.",
            "QA-01 alone does not select a DP-00 winner and does not certify an SLO."
        ]
    }


def markdown_report(report: dict, coverage_gates: dict | None = None) -> str:
    views = report["architecture_views"]
    lines = [
        "# DP-00 QA-01 Design-Time Reference Report v1", "",
        "**Status: design-time reference evidence; not production measurement or SLO certification.**", "",
        "## 1. Purpose and decision framing", "",
        "FTOL remains **Acoustic EOS → earliest Useful Outcome**. This report reuses the completed 36-scenario, 432,000-realization evidence in two architecture views. It does not create a new simulation campaign, reweight the workload, or change any latency primitive.", "",
        "The primary narrative is no longer one peer comparison of A/B/C/D. Stage 1 isolates the primary execution boundary; Stage 2 asks how bounded latency-sensitive capability is accommodated.", "",
        "## 2. Two-stage DP-00 model", "",
        "### 2.1 Stage 1 — Primary Execution Boundary", "",
        "**A0 vs B0.** A0 keeps intent interpretation and top-level Agent selection in VIA; ARGO remains a peer Agent. B0 moves primary interpretation and initial execution/delegation authority into ARGO while VIA retains interaction and user-facing task projection/correlation.", "",
        "Stage-1 question: **Should top-level semantic orchestration and Agent selection remain in VIA, or should primary semantic/execution authority move into ARGO?**", "",
        "### 2.2 Why D0 is not retained", "",
        "Removing `VIA_FAST` from D removes the execution-topology class that materially distinguishes its selector from A's Agent Router. Under the current specification, both remaining structures have VIA select ARGO or a Specialist, keep route/task state in VIA, reuse the route for clear follow-up, and place domain state in the selected executor. D0 is therefore **structurally equivalent to A0 for this trade-space purpose**, not a mathematical identity or an independent Stage-1 candidate. No new D0 responsibilities are invented.", "",
        "### 2.3 Stage 2 — Bounded Fast-capability Accommodation", "",
        "**C vs B1 vs D1.** C is A0 plus deterministic VIA Fast eligibility and local execution. B1 is B0 evaluated with bounded capability execution remaining inside the ARGO-owned execution boundary; it is not a VIA Fast Path or a new primary architecture. D1 exposes VIA Fast, ARGO Primary, and Specialist Direct inside one semantic topology-selection responsibility.", "",
        "Stage-2 question: **How does each architecture accommodate bounded latency-sensitive capability while preserving its defining ownership boundary?**", "",
        "## 3. Design-time latency methodology", "",
        "The corpus remains 12 unique F1, 12 F2, and 12 F3 scenarios. Each scenario has 1,000 synthetic latency realizations per existing candidate and model profile. The 432,000 primary realizations are statistical samples of 36 semantic tasks, not 432,000 tasks.", "",
        "The unchanged model is `V + architecture-derived M/H/X composition`: V is fixed at 500 ms; Qwen3-1.7B, Qwen3-8B, and Qwen3-30B-A3B provide three reference profiles; H = 5 ms and X1/X2/X3 = 20/50/100 ms are explicit assumptions; H = 1/5/20 ms and X = 0.5x/1x/2x form the sensitivity grid. Every candidate uses the same keyed primitive samples and model profile.", "",
        "All results are evidence-anchored synthetic design-time distributions. They are not production traces, a production workload mix, or SLO evidence.", "",
        "## 4. Stage 1 results — A0 vs B0", "",
        "Current A evidence is reused exactly as A0; current B evidence is reused exactly as B0. F1 here means how each base boundary handles a local-capable request without a VIA Fast bypass.", "",
    ]
    for profile_id, candidates in views["stage1"]["profiles"].items():
        lines += [f"### {profile_id}", "", "| Candidate | Overall p50 | Overall p95 | F1 p95 | F2 p95 | F3 p95 |", "| --- | ---: | ---: | ---: | ---: | ---: |"]
        for candidate, data in candidates.items():
            overall = data["overall"]
            category = data["categories"]
            lines.append(
                f"| {candidate} | {overall['p50_ms']:.3f} | {overall['p95_ms']:.3f} | "
                f"{category['F1']['p95_ms']:.3f} | {category['F2']['p95_ms']:.3f} | {category['F3']['p95_ms']:.3f} |"
            )
        lines.append("")

    lines += ["### Stage-1 path/model decomposition", "", "| Category | A0 model generations / H | B0 model generations / H |", "| --- | --- | --- |"]
    for category, candidates in views["stage1"]["trace_decomposition"].items():
        a, b = candidates["A0"], candidates["B0"]
        lines.append(f"| {category} | {a['model_generation_count']} / {a['handoff_count']} | {b['model_generation_count']} / {b['handoff_count']} |")
    lines += ["", "A0 uses separate interpretation, route-decision, and short executor generations. B0 uses one combined ARGO generation for F1/F2; its F3 path adds Specialist reasoning and a second handoff. This explains both B0's shorter direct path and its Specialist-tail exposure.", "", "### Stage-1 H/X crossover analysis", ""]
    for profile_id, cells in views["stage1"]["handoff_effect_sensitivity"].items():
        relations = Counter(cell["relation"] for cell in cells)
        summary = "; ".join(f"{key} ({count}/9 cells)" for key, count in sorted(relations.items()))
        lines += [f"- {profile_id}: {summary}."]
    lines += ["", "Exact Stage-1 cells:", ""]
    for profile_id, cells in views["stage1"]["handoff_effect_sensitivity"].items():
        lines += [f"#### {profile_id}", "", "| H ms | X multiplier | A0 p95 | B0 p95 | Relation |", "| ---: | ---: | ---: | ---: | --- |"]
        for cell in cells:
            lines.append(f"| {cell['handoff_ms']} | {cell['effect_multiplier']:.1f}x | {cell['A0_p95_ms']:.3f} | {cell['B0_p95_ms']:.3f} | {cell['relation']} |")
        lines.append("")

    lines += [
        "The Stage-1 relation is reference-regime dependent: A0/B0 crossovers occur in the small and 30B-A3B H/X grids, while A0 has the lower pooled p95 in all nine medium-profile cells. This does not establish a universal winner.", "",
        "## 5. Stage 2 results — C vs B1 vs D1", "",
        "F1 is the primary Stage-2 evidence because it directly exercises the bounded capability accommodation. F2/F3 are retained as secondary non-local diagnostics.", "",
    ]
    decomposition = views["stage2"]["f1_trace_decomposition"]
    for profile_id, candidates in views["stage2"]["profiles"].items():
        lines += [f"### {profile_id}", "", "| Candidate | F1 p50 | F1 p95 | F1 path | Model generations | H |", "| --- | ---: | ---: | --- | ---: | ---: |"]
        for candidate in ("C", "B1", "D1"):
            data, trace = candidates[candidate], decomposition[candidate]
            lines.append(f"| {candidate} | {data['f1_primary']['p50_ms']:.3f} | {data['f1_primary']['p95_ms']:.3f} | {trace['path']} | {trace['model_generation_count']} | {trace['handoff_count']} |")
        lines.append("")

    lines += [
        "C performs semantic interpretation followed by deterministic Fast eligibility, then VIA-local execution. B1 performs one combined generation after crossing into ARGO and executes the bounded capability inside ARGO's existing authority. D1 performs interpretation plus a semantic Execution Path Selector generation before VIA Fast execution.", "",
        "Across all three reference profiles, the F1 relation is **C < D1 < B1** at H = 5 ms and X = 1x. C's advantage over D1 is causally consistent with responsibility decomposition: both share V/X and have no H, but D1 adds `EXECUTION_ROUTE_DECISION`.", "",
        "### Stage-2 H/X sensitivity", "",
        "- C and D1 have zero F1 handoffs; B1 has one. At X = 1x, moving H from 5 ms to 1/20 ms shifts B1 F1 p95 by exactly -4/+15 ms and leaves C/D1 unchanged. This range does not close the observed D1-to-B1 reference gaps.",
        "- The same keyed X sample and multiplier are applied to C, B1, and D1. X therefore creates no candidate-specific discount and cancels from paired per-realization differences.",
        "- C versus D1 has no H/X crossover under the trace contract: D1 always contains the additional semantic route-decision generation. This is a statement about the frozen design-time composition, not production performance.", "",
        "## 6. Stage 2 secondary diagnostics", "",
        "| Model profile | Candidate | F2 p95 | F3 p95 | Pooled p95 |", "| --- | --- | ---: | ---: | ---: |",
    ]
    for profile_id, candidates in views["stage2"]["profiles"].items():
        for candidate in ("C", "B1", "D1"):
            data = candidates[candidate]
            lines.append(f"| {profile_id} | {candidate} | {data['f2_secondary']['p95_ms']:.3f} | {data['f3_secondary']['p95_ms']:.3f} | {data['pooled_secondary']['p95_ms']:.3f} |")
    lines += [
        "", "C and D1 tie on pooled p95 because their common F2/F3 paths occupy the tail. That tie is not evidence that their bounded-capability designs have equivalent latency: their F1 p95 differs materially in every model profile. Pooled Fast-task p95 can mask a defining-path difference, so Stage 2 requires F1 and path-level reporting.", "",
        "## 7. Cross-stage aggregate diagnostic", "",
        "The original balanced A/B/C/D view is retained below only as a secondary diagnostic. It mixes the Stage-1 authority boundary with the Stage-2 bounded-capability question and must not be used as the primary architecture-selection basis.", "",
    ]
    for profile_id, alternatives in report["primary"].items():
        lines += [f"### {profile_id}", "", "| Existing candidate | p50 | p95 | p99 | F1 p95 | F2 p95 | F3 p95 |", "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
        for alternative, data in alternatives.items():
            overall, category = data["statistics"], data["category_statistics"]
            lines.append(f"| {alternative} | {overall['p50_ms']:.3f} | {overall['p95_ms']:.3f} | {overall['p99_ms']:.3f} | {category['F1']['p95_ms']:.3f} | {category['F2']['p95_ms']:.3f} | {category['F3']['p95_ms']:.3f} |")
        lines.append("")

    lines += ["## Defining-path coverage", "", "| Existing candidate:path | Semantic scenarios | Gate |", "| --- | ---: | --- |"]
    gates = coverage_gates if coverage_gates is not None else load_json(DEFAULT_TRACES)["coverage_gates"]
    for key, observed in report["validation"]["path_observations"].items():
        required = gates.get(key)
        gate = "PASS" if required is None or observed >= required else "FAIL"
        requirement = "diagnostic" if required is None else f">= {required} — {gate}"
        lines.append(f"| {key} | {observed} | {requirement} |")

    lines += [
        "", "## 8. Architectural interpretation", "",
        "A0 and B0 are meaningfully different primary execution-boundary architectures. Their latency relation depends on the reference model and H/X regime. C, B1, and D1 are three structurally different accommodations of bounded capability: new deterministic VIA-local authority, capability retained inside ARGO authority, and unified semantic topology selection, respectively.", "",
        "Latency evidence does not decide Agent neutrality, coupling, correctness, flexibility, lifecycle complexity, or product priority. Those remain separate dimensions; QA-02/03/04 may raise follow-up structural questions but are not modified here.", "",
        "## 9. Provenance and limitations", "",
        "- **P1 Published reference:** Qwen official SGLang BF16 batch-1 speeds (input length 1, 2,048 generated tokens) are 227.80 tok/s for Qwen3-1.7B, 81.73 tok/s for Qwen3-8B, and 137.18 tok/s for Qwen3-30B-A3B. Qwen defines this as aggregate prompt-plus-generation throughput, so the simulator uses its reciprocal only as a first-order TPOT approximation.",
        "- **P1 Public API default:** V = 500 ms is the OpenAI Realtime `server_vad` default `silence_duration_ms`, used as an Acoustic-EOS/end-of-speech detection reference—not as measured OpenAI latency.",
        "- **P2 Repository structure:** model-operation and handoff counts come from the frozen DP-00 executable architecture responsibilities and the committed trace contract.",
        "- **P3 Engineering assumptions:** TTFT p50/p95, 25% TPOT p95 spread, output-token budgets (24/16/32/48), H = 5 ms, and X = 20/50/100 ms with their stated distributions.",
        "- Model latency is `TTFT + (output_tokens - 1) × TPOT`; log-normal parameters are fit from stated synthetic p50/p95 values. Seed: 20260914.", "",
        "This report makes conditional design-time comparisons only. It does not measure deployed VIA, real API latency, a production workload mix, production p95, or SLO compliance. The three Qwen profiles are plausible family reference profiles, not a monotonic size/latency ladder; the 30B-A3B model is MoE. QA-02/03/04 populations, oracles, qualification, and route-commit semantics are unchanged. Historical Profile Z and R1–R4 evidence remains valid for its original fixed-delay experiment and has not been rewritten.", "",
        "No global A/B/C/D, Stage-1, or Stage-2 winner is selected. No weighted score is introduced.", "",
    ]
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", type=Path, default=DEFAULT_CORPUS)
    parser.add_argument("--traces", type=Path, default=DEFAULT_TRACES)
    parser.add_argument("--plans", type=Path, default=DEFAULT_PLANS)
    parser.add_argument("--oracles", type=Path, default=DEFAULT_ORACLES)
    parser.add_argument("--profile", type=Path, default=DEFAULT_PROFILE)
    parser.add_argument("--json-output", type=Path, default=DEFAULT_JSON)
    parser.add_argument("--markdown-output", type=Path, default=DEFAULT_MD)
    args = parser.parse_args()
    report = simulate(
        load_json(args.corpus), load_json(args.traces), load_json(args.profile),
        load_json(args.plans), load_json(args.oracles),
    )
    args.json_output.parent.mkdir(parents=True, exist_ok=True)
    args.json_output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    args.markdown_output.write_text(markdown_report(report, load_json(args.traces)["coverage_gates"]), encoding="utf-8")


if __name__ == "__main__":
    main()
