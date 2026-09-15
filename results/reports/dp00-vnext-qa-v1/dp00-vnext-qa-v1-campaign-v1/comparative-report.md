# DP-00 vNext QA-v1 Comparative Campaign

## Claim boundary

This is an offline, evidence-anchored deterministic semantic-replay campaign over the exact frozen QA-v1 populations. It is empirical evidence about these executable reference realizations, not production-device or Cloud-model performance.

All three candidates were pre-registered in one Git checkpoint before comparative execution. R1+@ remains an R1 tactic realization.

## Official results

| Candidate | QA-01 | QA-02 | QA-03 | QA-04 | QA-05 | QA-06 | QA-07 | QA-08 | QA-09 | QA-10 | QA-11 | QA-12 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R1 | 0.907544 seconds / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | UNEVALUABLE | 1.13 seconds / score 5 | 100 percent / score 5 | 100 percent / score 5 | 0.2 seconds / score 5 | 260 milliseconds / score 2 |
| R3 | 0.907544 seconds / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | UNEVALUABLE | 1.13 seconds / score 5 | 100 percent / score 5 | 100 percent / score 5 | 0.2 seconds / score 5 | 260 milliseconds / score 2 |
| R1+@ | 0.907544 seconds / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | 100 percent / score 5 | UNEVALUABLE | 1.13 seconds / score 5 | 100 percent / score 5 | 100 percent / score 5 | 0.2 seconds / score 5 | 260 milliseconds / score 2 |

The official scalar results do not discriminate R1 from R3. R1+@ executed exactly 50 visibly eligible bounded read-only cases locally; this lowered its QA-01 latency median from 0.748328 s to 0.705334 s, but the official p95 remained 0.907544 s because the corpus tail is outside the tactic-eligible subset. This median is diagnostic only and does not alter the QA-01 score.

All three candidates miss the QA-12 target (`p95 <= 200 ms`) at 260 ms. This shared regression is retained as a trade-off finding and is not converted into a candidate-specific failure probability.

## QA-01 sensitivity

| Candidate | Profile | p95 seconds | Score |
| --- | --- | ---: | ---: |
| R1 | QWEN3_MEDIUM_REFERENCE | 0.907544 | 5 |
| R1 | QWEN3_SMALL_REFERENCE | 0.422606 | 5 |
| R1 | QWEN3_30B_A3B_REFERENCE | 0.583110 | 5 |
| R3 | QWEN3_MEDIUM_REFERENCE | 0.907544 | 5 |
| R3 | QWEN3_SMALL_REFERENCE | 0.422606 | 5 |
| R3 | QWEN3_30B_A3B_REFERENCE | 0.583110 | 5 |
| R1+@ | QWEN3_MEDIUM_REFERENCE | 0.907544 | 5 |
| R1+@ | QWEN3_SMALL_REFERENCE | 0.422606 | 5 |
| R1+@ | QWEN3_30B_A3B_REFERENCE | 0.583110 | 5 |

## Qualification and decision

Functional obligations, candidate invariants, QA-08 safe-recovery gates, and QA-09 security gates passed for all three reference realizations.

QA-07 is not officially evaluable because no evidence-anchored frozen minimum-mandatory-memory calibration exists. No byte denominator was created or imputed. Consequently this campaign makes **no final DP-00 selection**, and no ADR is proposed.

Even apart from QA-07 evidence completeness, the current official scalar results do not distinguish the three realizations and all share the QA-12 target miss. A future decision must not break that tie by preference or by arithmetic weighting.

**Decision outcome: NO QUALIFIED WINNER / MORE ARCHITECTURE WORK REQUIRED.**

Exact blocker: complete the Reference Environment-governed QA-07 physical-memory calibration for the frozen P0–P5 workload, then run the complete integrated campaign again without changing candidates, QA semantics, or populations.
