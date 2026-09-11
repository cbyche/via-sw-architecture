# DP-00 Runtime Pilot v0 Offline Analysis

This package performs deterministic post-run analysis of immutable Rust raw evidence. Python is not an AUT dependency, never enters the Rust timed execution path, makes no architecture decision, and never rewrites `results/raw/**`. Derived files belong under `results/derived/**`.

`dp00-analysis-v9` validates the current `canonical-event-v3`, `model-call-v1`, `dp00-pilot-provenance-v4`, `dp00-calibration-manifest-v1`, and `dp00-pilot-campaign-provenance-v1` contracts while retaining read support for immutable v2/v3 provenance/event evidence. Required fields, unknown fields, invalid enums, absent/unsupported versions, and asset reference/version mismatches are errors; no implicit defaults or migrations are applied. v9 retains the v8 acceptance formulas and paired semantics unchanged, adds strict R1–R4 profile-id compatibility, and reports configured Model/Agent/Tool budgets plus the observed framework/timer residual for QA-01.

QA-01 derives exact integer nanosecond FTOL as authoritative Outcome Probe useful-outcome time minus Interaction Fixture acoustic EOS time, only for manifest-eligible successful episodes. Human-facing milliseconds are derived afterward. Both explicit nearest-rank and `(n-1)` linear-interpolated p50/p95/p99 candidates and small-N sensitivity are emitted. The dependency decomposition reports semantic event counts, configured Model/Agent/Tool budgets, their total, and FTOL minus that total; the residual includes framework/runtime and timer/scheduler effects and is not mislabeled as pure framework time.

QA-02 reconstructs Actual semantic facts from canonical/model/fixture raw streams before evaluator-only Oracle loading. It then evaluates Required/Allowed/Forbidden exact conformance, preserves numerator/denominator/fraction/percentage, and reports correctness separately from evidence completeness. The historical v0.1 36×4 map remains preserved. Corpus v0.2 selects `pilot-v0.2-constraint-alternative-evidence-map.json`, covering 47×4 cells. Both contain strategies and authorities, never Actual values or an Oracle fallback.

`dp00-analysis-v1` permits topology-neutral equivalent representations such as an explicit referent binding or an authoritative document-outcome subject. An explicit terminal failure makes an unobserved positive requirement deterministically FAIL. A Forbidden predicate may pass by absence only through `CLOSED_WORLD_ABSENCE`: the terminal must be unique and final, event count must match the captured stream, capture mode/schema must be recognized, the relevant boundary must be declared, and the episode must have no integrity error.

QA-04 independently derives ORCHESTRATION/MIXED logical-call inclusion from call timing/class/responsibility, decision owner, and route-commit evidence. For a scalar request the boundary is its single root commit. For a compound request, every manifest-declared required subgoal must have one committed route and the boundary is the latest required commit timestamp; partial coverage remains non-numeric `ROUTE_REQUIRED_NOT_COMMITTED`. Post-boundary calls are excluded, retries remain distinct logical generations, zero-call successes are zero, and no-route-commit episodes stay non-numeric diagnostics. `qa04_contract_diagnostics` implements `qa04-route-contract-policy-v1`: REQUIRED comparisons need route-contract satisfaction and QA-02 conformance, OPTIONAL is a separate stratum, and FORBIDDEN is excluded from Primary. Overall versus macro and the OPTIONAL comparison remain deferred.

CAPTURE and MINIMAL both persist the authoritative Measurement Spine and logical model-call stream. CAPTURE additionally retains model-generation diagnostic canonical events and fixture detail. `paired_calibration` groups only explicit `pair_id` values, rejects missing/duplicate modes and provenance or semantic mismatches, and never infers pairing from filenames, directory order, or execution order. QA-01 continues to use authoritative Acoustic EOS and useful-outcome timestamps in both modes.

For `dp00-calibration-protocol-v1`, the analyzer requires a manifest-declared two-cycle full pre-warm with explicit 96-path coverage per cycle, exactly 16 measured cycles, 1,536 measured executions, 768 adjacent pairs, within-cycle 24/24 mode-order balance, per-alternative 6/6 and per-scenario 2/2 balance, next-cycle inversion, per-pair full-run 8/8 balance, and the independent four-position alternative rotation. Pre-warm evidence is excluded from raw measured populations; malformed schedules are errors rather than repaired or inferred.

Pilot diagnostics are not Final Evaluation scores or architecture results. Official values are produced only when the input is a valid Official Z+C campaign with common commit/corpus/runtime identity and distinct profile aggregation.

Run from repository root:

```text
PYTHONPATH=benchmark/analysis .venv/bin/python -m dp00_analysis validate <raw-root>
PYTHONPATH=benchmark/analysis .venv/bin/python -m dp00_analysis derive <raw-root> --output <derived-root>
PYTHONPATH=benchmark/analysis .venv/bin/python -m dp00_analysis summary <derived-root>
.venv/bin/python -m pytest benchmark/analysis
```

`analysis-summary.json` contains `analysis_version`, validation errors/warnings, QA-01 counts/estimators/per-scenario diagnostics, QA-02 AECR/constraint/dimension/coverage diagnostics, QA-04 per-episode/overall/macro/class/no-route diagnostics, paired-calibration completeness/invariance diagnostics, counterbalanced-schedule diagnostics, and analysis provenance. Identical raw evidence, analysis version, and method configuration produce byte-deterministic sorted JSON output apart from the explicitly recorded interpreter/environment provenance.

In v8, `paired_calibration` exposes `incomplete_pairs`,
`duplicate_mode_pairs`, `semantic_mismatch_pairs`, `qa02_mismatch_pairs`,
`qa04_mismatch_pairs`, and `provenance_mismatch_pairs` independently. It also
provides a sorted `rejected_pairs` union and `rejection_reasons_by_pair`; the
semantic field contains only actual normalized-semantic signature mismatches.

For a complete v1 calibration manifest, `calibration_acceptance` uses only complete,
semantically/provenance-matched, QA-01 exact-correctness-qualified pairs. Its frozen
baseline `M` is the pooled MINIMAL linear-interpolated p50 and every gate uses the
absolute `0.05 × M` margin. CAPTURE and MINIMAL independently require both the
cycles 8–15 minus cycles 0–7 p50 shift and the Theil–Sen cycle-0-to-15 predicted
shift to be within that margin. Mode-order interaction is the absolute difference
between median paired deltas for CAPTURE-first and MINIMAL-first populations.
Bootstrap intervals use 10,000 pair-preserving, half/first-mode-stratified draws
with seed `20260911`; they are diagnostic only and never affect a gate. The
machine-readable source is `benchmark/contracts/dp00-calibration-acceptance-v1.json`.
