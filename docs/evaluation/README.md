# VIA Evaluation Status

## ACTIVE NORMATIVE QA CONTRACT

→ `docs/evaluation/qa-contracts/v1/README.md`

QA Evaluation Contract v1 is the only active QA definition for future architecture comparisons. Executable values are in `benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json`; its reference environment and population plan are beside it.

Historical reuse audit: `docs/evaluation/qa-contracts/v1/HISTORICAL-EVIDENCE-AUDIT.md`

Corpus gaps: `docs/evaluation/qa-contracts/v1/CORPUS-GAP-REPORT.md`

Shared evaluator: `benchmark/analysis/qa_v1/`

## LEGACY QA DEFINITIONS

→ `docs/archive/legacy-evaluation/pre-qa-contract-v1/`

The stable files under `docs/evaluation/quality-attributes/` are supersession redirects, not active definitions.

## HISTORICAL EXPERIMENTS

DP-00 protocols, contracts, analysis, and `results/` remain preserved for provenance and reproducibility. They retain the identifiers, populations, measurements, and scores of the legacy contract under which they were produced. They are not current scoring authority and must not be relabeled as QA-v1 evidence without a new evaluation.

## Dependency direction

```text
Product Mission / Scope
        ↓
QA Evaluation Contract v1
        ↓
Reference Environment + Frozen Evaluation Populations
        ↓
Decision Point Alternatives
        ↓
Benchmark / Analysis
        ↓
Architecture Decision
```

Reusable methodology in this directory remains active only where it is consistent with the v1 contract. A Decision Point selects which contract QAs are primary; it never creates local QA semantics.
