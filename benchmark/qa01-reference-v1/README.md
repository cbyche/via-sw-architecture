# DP-00 QA-01 Design Reference v1

> **HISTORICAL DESIGN-REFERENCE EVIDENCE — LEGACY QA-01.** These scenarios and generated results are not `qa01-fast-v1` and must not be reported as QA Contract v1 scores. The Qwen profile evidence is reused only through Reference Environment v1.

This additive benchmark lineage evaluates user-visible Fast-task Outcome Latency without modifying `pilot-v0` or R1–R4.

Inputs:

- `scenarios.json` — 36 unique semantic Fast-task scenarios;
- `behavior-plans.json` — architecture-neutral F1/F2/F3 semantic plans and scenario bindings;
- `oracles.json` — topology-neutral useful-outcome rules;
- `traces.json` — P2 architecture mappings and defining-path gates;
- `../contracts/dp00-qa01-design-reference-latency-v1.json` — P1/P3 latency evidence, transformations, distributions, and seed.

Reproduce with the repository virtual environment:

```bash
.venv/bin/python benchmark/analysis/generate_qa01_reference_report.py
.venv/bin/python -m pytest benchmark/analysis/tests/test_qa01_reference.py
```

Outputs are written to `results/reports/qa01-reference-v1/`. They contain aggregate derived distributions, not fabricated measured traces. The profile performs no live API calls.

The generated report presents two derived architecture views without changing
the samples: Stage 1 maps existing A/B evidence to A0/B0, and Stage 2 maps
existing C/B/D evidence to C/B1/D1 with F1 as its primary population. The
pooled A/B/C/D view remains a secondary cross-stage diagnostic.
