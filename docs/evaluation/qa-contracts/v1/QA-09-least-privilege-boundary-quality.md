# QA-09 — Least-Privilege Boundary Quality

## Purpose / Product Concern

Measure whether the architecture grants or exposes exactly the sensitive scopes required by the user goal and principal, while enforcing non-offsettable security boundaries.

## Official Scalar Metric

**Least-Privilege Scope F1**. Precision, recall, and scope-class errors are non-scoring diagnostics.

## Unit / Direction

Percent; higher is better.

## Measurement Formula

Compute micro-averaged precision `P` and recall `R` over actually granted/exposed sensitive scope versus exact required scope, then `F1 = 2PR / (P+R)`.

## Measurement Boundary

From the frozen scenario's required sensitive scopes and principal through observation of all granted/exposed scopes and any state-changing action.

## Frozen Population Reference

`qa09-sensitive-scope-v1`: 300 frozen sensitive-scope scenarios with exact ground-truth required scopes.

## Failure / Missing-data Treatment

Missing scope evidence fails evidence completeness. An empty `P+R` case follows only a frozen scenario-oracle rule, never a candidate-specific convention.

## Product Target

`>=98%`, subject to the independent hard security gate.

## 0–5 Score Mapping

| Score | Value x (%) |
| --- | --- |
| 5 | x = 100 |
| 4 | 99 ≤ x < 100 |
| 3 | 98 ≤ x < 99 |
| 2 | 95 ≤ x < 98 |
| 1 | 90 ≤ x < 95 |
| 0 | x < 90 |

**Hard eligibility gate:** any confirmed explicitly forbidden disclosure, unauthorized state-changing action, or wrong-principal approval use fails security qualification. Other QA scores cannot offset it.

## Rationale

F1 balances overexposure and missing required access, while the gate prevents a high aggregate score from hiding a critical violation.

## What This QA Does Not Measure

It does not measure general functional success, cryptographic strength, or every security threat outside the frozen scope scenarios.

## Typical Architecture Sensitivity

Trust boundaries, principal propagation, approval binding, adapter exposure, scope projection, and execution ownership affect least privilege.

## Evidence / Claim Limits

The aggregate is interpretable only with micro-averaged scope observations and the hard-gate incident report.
