# QA-02 — Task Completion Effectiveness

## Purpose / Product Concern

Measure whether the integrated product completes the user's goal with every required semantic, consent, binding, result, and cross-channel condition satisfied.

## Official Scalar Metric

**Constraint-Conformant Goal Completion Rate**. Condition-level rates are non-scoring diagnostics.

## Unit / Direction

Percent; higher is better.

## Measurement Formula

`100 × successful unique goal instances / all 600 frozen goal instances`. Success requires correct goal and referent, explicit constraints, required consent, correct task/result binding, required result facts, Voice/Text factual consistency, and the fixed scenario deadline.

## Measurement Boundary

Start is the scenario-defined user-goal stimulus. End is the fixed deadline after all required conditions are evaluated.

## Frozen Population Reference

`qa02-goals-v1`: 600 unique instances across at least 60 semantic families. Six equal strata cover Fast/bounded, short general-Agent, specialist-Agent, temporal grounding/referent/correction, multi-turn/concurrent/approval, and compound/longer Agent tasks.

## Failure / Missing-data Treatment

Any failed or missing required condition makes the entire goal instance unsuccessful in the denominator.

## Product Target

`>=95%`.

## 0–5 Score Mapping

| Score | Value x (%) |
| --- | --- |
| 5 | x = 100 |
| 4 | 98 ≤ x < 100 |
| 3 | 95 ≤ x < 98 |
| 2 | 85 ≤ x < 95 |
| 1 | 70 ≤ x < 85 |
| 0 | x < 70 |

## Rationale

Exact goal completion prevents strong partial behavior from hiding a failed safety, binding, or user-constraint condition.

## What This QA Does Not Measure

It does not claim a general production success probability or independently score latency, continuity, or privilege minimization.

## Typical Architecture Sensitivity

Grounding authority, semantic decisions, task identity, approval flow, Agent routing, result binding, and multimodal response assembly affect success.

## Evidence / Claim Limits

The 600-case corpus is an architecture qualification corpus, not a claim of production success probability.
