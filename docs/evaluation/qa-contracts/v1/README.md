# VIA QA Evaluation Contract v1

## Status and authority

**ACTIVE NORMATIVE QA CONTRACT — VERSION 1**

> **QA Contract v1 is immutable for all architecture comparisons performed under this evaluation campaign. A Decision Point may nominate Primary QAs, but it may not redefine a QA's metric, population, denominator, target, measurement boundary, failure treatment, or scoring function. Such a change requires a new QA Contract version and re-evaluation of affected alternatives.**

This directory is the normative human-readable entry point. The executable source of truth is `benchmark/contracts/qa-v1/qa-evaluation-contract-v1.json`, validated by `benchmark/schemas/qa-evaluation-contract-v1.schema.json`. `reference-environment-v1.json` and `corpus-manifest-v1.json` freeze the evaluation context and population identities separately.

If prose and executable data disagree, the discrepancy is a contract defect: stop evaluation, correct it through an explicit contract-version process, and do not choose the convenient interpretation.

## Contract-wide rules

- One official scalar metric determines each QA's 0–5 score. Diagnostic breakdowns never alter that score.
- Every eligible case remains in its stated denominator unless that QA explicitly defines a latency penalty or hard eligibility gate.
- Missing/failure treatment, measurement start/end, population, target, and score bands are part of the QA definition.
- QA-08 unsafe/incorrect recovery and QA-09 security violations are non-offsettable qualification failures.
- All twelve populations are now frozen as an evaluation instrument. This does
  not make them empirical evidence: QA-v1 remains `NOT_EVALUATED` until a
  candidate completes a provenance-bound campaign.
- Results must record the ID, version, and SHA-256 of the QA contract, reference environment, and corpus manifest.

## Reference environment

Architecture scoring uses `via-reference-environment-v1`. Semantic output may come from frozen replay or optional Cloud API generation, but Cloud wall-clock/provider latency never enters the architecture score. Cloud generation provenance must retain model/version, input/output, prompt/schema hash, and token counts.

The official timing profile is the existing `QWEN3_MEDIUM_REFERENCE`. `QWEN3_SMALL_REFERENCE` and `QWEN3_30B_A3B_REFERENCE` are sensitivity analyses only. Real target hardware requires a new Reference Environment version; hardware calibration alone does not change QA targets, populations, or scoring.

## QA index

| ID | Official scalar metric | Target | Frozen population |
| --- | --- | --- | --- |
| QA-01 | Fast Useful Outcome Latency p95 | p95 ≤ 3.0 s | `qa01-fast-v1` |
| QA-02 | Constraint-Conformant Goal Completion Rate | ≥ 95% | `qa02-goals-v1` |
| QA-03 | Continuity Episode Exact Success Rate | ≥ 99% | `qa03-continuity-v1` |
| QA-04 | Agent Evolution Containment Rate | ≥ 95% | `qa04-agent-evolution-v1` |
| QA-05 | Change Containment Rate | ≥ 90% | `qa05-product-evolution-v1` |
| QA-06 | Core-Preserving Device Adaptation Rate | ≥ 90% | `qa06-device-adaptation-v1` |
| QA-07 | Peak Memory Amplification Ratio | ≤ 1.5x | `qa07-resource-v1` |
| QA-08 | Safe Task Recovery Time p95 | p95 ≤ 10 s | `qa08-recovery-v1` |
| QA-09 | Least-Privilege Scope F1 | ≥ 98% + hard gate | `qa09-sensitive-scope-v1` |
| QA-10 | End-to-End Decision Trace Reconstruction Rate | ≥ 98% | `qa10-trace-v1` |
| QA-11 | Task Event-to-Useful-Feedback Latency p95 | p95 ≤ 1.5 s | `qa11-feedback-v1` |
| QA-12 | Barge-in Audible Stop Latency p95 | p95 ≤ 200 ms | `qa12-barge-in-v1` |

## Decision Point binding

An active DP evaluation binding contains only its DP ID, the three versioned contract identities, and valid `primary_qa_ids`. The schema deliberately rejects DP-local `metric`, `target`, `score_bands`, or `population` fields. DP alternatives remain downstream inputs; this contract does not redesign them.

## Migration and history

See `MIGRATION.md`. Historical DP-00 evidence retains its original legacy semantics. It may inform rationale and reproducibility but must never be silently rescored or relabeled as QA-v1 evidence.

Evidence reuse is governed by `EVIDENCE-REUSE-POLICY.md`; the completed
repository audit is `HISTORICAL-EVIDENCE-AUDIT.md`. `CORPUS-DESIGN.md` and
`CORPUS-COVERAGE.md` describe the frozen instrument; `USE-CASE-COVERAGE.md`
traces it to VIA UC-01–UC-16 without importing historical architecture choices.
`CORPUS-GAP-REPORT.md` is retained as the pre-materialization planning
snapshot. The machine-readable registry is
`benchmark/contracts/qa-v1/historical-evidence-registry-v1.json`.

## Shared evaluation engine

All future DPs use `benchmark/analysis/qa_v1/` for frozen-population loading,
provenance, evaluator interfaces, and contract-driven scoring. An incomplete
observation set produces no scalar and no score. Result envelopes must validate
against `benchmark/schemas/qa-v1-result-envelope.schema.json`.
