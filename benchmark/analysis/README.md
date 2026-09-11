# DP-00 Runtime Pilot v0 Offline Analysis

This package performs deterministic post-run analysis of immutable Rust raw evidence. Python is not an AUT dependency, never enters the Rust timed execution path, makes no architecture decision, and never rewrites `results/raw/**`. Derived files belong under `results/derived/**`.

`dp00-analysis-v1` validates the current `canonical-event-v2`, `model-call-v1`, `dp00-pilot-provenance-v2`, and `dp00-pilot-campaign-provenance-v1` contracts. Required fields, unknown fields, invalid enums, absent/unsupported versions, and asset reference/version mismatches are errors; no implicit defaults or migrations are applied. The existing Rust-owned golden expectation manifests are read directly by parity tests rather than copied into Python.

QA-01 derives exact integer nanosecond FTOL as authoritative Outcome Probe useful-outcome time minus Interaction Fixture acoustic EOS time, only for manifest-eligible successful episodes. Human-facing milliseconds are derived afterward. Both explicit nearest-rank and `(n-1)` linear-interpolated p50/p95/p99 candidates and small-N sensitivity are emitted; the final percentile estimator remains TBD.

QA-02 reconstructs Actual semantic facts from canonical/model/fixture raw streams before evaluator-only Oracle loading. It then evaluates Required/Allowed/Forbidden exact conformance, preserves numerator/denominator/fraction/percentage, and reports correctness separately from evidence completeness. The historical 36-row map remains preserved; `pilot-v0-constraint-alternative-evidence-map.json` adds the executable 36×4 readiness gate. It contains strategies and authorities, never Actual values or an Oracle fallback.

`dp00-analysis-v1` permits topology-neutral equivalent representations such as an explicit referent binding or an authoritative document-outcome subject. An explicit terminal failure makes an unobserved positive requirement deterministically FAIL. A Forbidden predicate may pass by absence only through `CLOSED_WORLD_ABSENCE`: the terminal must be unique and final, event count must match the captured stream, capture mode/schema must be recognized, the relevant boundary must be declared, and the episode must have no integrity error.

QA-04 independently derives ORCHESTRATION/MIXED logical-call inclusion from call timing/class/responsibility, decision owner, and route-commit evidence. Post-commit calls are excluded, retries remain distinct logical generations, zero-call successes are zero, and no-route-commit episodes stay non-numeric diagnostics. Overall episode mean and equal-weight scenario-class macro-average are both candidates; their final selection and the no-route policy remain unresolved.

Pilot diagnostics are not Final Evaluation scores or architecture results. Official values are produced only when the input is a valid Official Z+C campaign with common commit/corpus/runtime identity and distinct profile aggregation.

Run from repository root:

```text
PYTHONPATH=benchmark/analysis .venv/bin/python -m dp00_analysis validate <raw-root>
PYTHONPATH=benchmark/analysis .venv/bin/python -m dp00_analysis derive <raw-root> --output <derived-root>
PYTHONPATH=benchmark/analysis .venv/bin/python -m dp00_analysis summary <derived-root>
.venv/bin/python -m pytest benchmark/analysis
```

`analysis-summary.json` contains `analysis_version`, validation errors/warnings, QA-01 counts/estimators/per-scenario diagnostics, QA-02 AECR/constraint/dimension/coverage diagnostics, QA-04 per-episode/overall/macro/class/no-route diagnostics, and analysis provenance. Identical raw evidence, analysis version, and method configuration produce byte-deterministic sorted JSON output apart from the explicitly recorded interpreter/environment provenance.
