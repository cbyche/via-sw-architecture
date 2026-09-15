# VIA Evaluation Status

## ACTIVE NORMATIVE QA CONTRACT

→ `docs/evaluation/qa-contracts/v1/README.md`

QA Evaluation Contract v1 is the only active QA definition for future architecture comparisons. Executable values are in `benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json`; its reference environment and population plan are beside it.

Historical reuse audit: `docs/evaluation/qa-contracts/v1/HISTORICAL-EVIDENCE-AUDIT.md`

Corpus gaps: `docs/evaluation/qa-contracts/v1/CORPUS-GAP-REPORT.md`

Shared evaluator: `benchmark/analysis/qa_v1/`

Active vNext Decision Point catalog: `../architecture/decision-points/catalog.md`

DP-00 R1/R3 candidate contracts: `../../benchmark/contracts/dp-vnext/`

DP-00 frozen protocol: `dp00-vnext-evaluation-protocol-v1.md`

DP-00 pre-registration and controls: `../../benchmark/contracts/dp-vnext/evaluation/`

First offline QA-v1 campaign report: `../../results/reports/dp00-vnext-qa-v1/dp00-vnext-qa-v1-campaign-v1/comparative-report.md`

The campaign produced complete preliminary results for QA-01 through QA-06 and QA-08 through QA-12. QA-07 remains unscored because the active Reference Environment has no evidence-anchored frozen memory denominator, so no final DP-00 winner or ADR is claimed.

## LEGACY QA DEFINITIONS

→ `docs/archive/legacy-evaluation/pre-qa-contract-v1/`

The stable files under `docs/evaluation/quality-attributes/` are supersession redirects, not active definitions.

## HISTORICAL EXPERIMENTS

Historical A/B/C/D DP-00 protocols, contracts, analysis, and `results/` remain preserved for provenance and reproducibility. They retain the identifiers, populations, measurements, and scores of the legacy contract under which they were produced. They are not R1/R3 QA-v1 scores and must not be relabeled as current evidence without a new evaluation. The active base families are R1 and R3; R1+@ is an R1 tactic realization.

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
