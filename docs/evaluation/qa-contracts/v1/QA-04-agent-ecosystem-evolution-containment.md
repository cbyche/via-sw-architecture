# QA-04 — Agent Ecosystem Evolution Containment

## Purpose / Product Concern

Measure whether expected Agent ecosystem evolution remains inside the architecture's predefined Agent Integration ownership area.

## Official Scalar Metric

**Agent Evolution Containment Rate**. Change counts and affected-area lists are non-scoring diagnostics.

## Unit / Direction

Percent; higher is better.

## Measurement Formula

`100 × contained passing Agent evolution cases / 60 frozen Agent evolution cases`.

## Measurement Boundary

From a frozen Agent change request through completed implementation and common regression evaluation. Allowed ownership is the Agent adapter, protocol mapping, capability manifest/extension, or an approved Agent-integration seam.

## Frozen Population Reference

`qa04-agent-evolution-v1`: 60 cases including onboarding, replacement, session semantic changes, progress/result/approval variations, cancel/reconnect variation, capability extension/removal, and protocol/version evolution.

## Failure / Missing-data Treatment

Incomplete functionality, common regression failure, or required change to unrelated Core semantic responsibilities means the case is not contained.

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

Agent protocols and capabilities should evolve without forcing unrelated interaction, task, response, or device semantics to change.

## What This QA Does Not Measure

It does not measure general product change containment, developer speed, LOC, file count, or runtime Agent quality.

## Typical Architecture Sensitivity

Adapter ownership, protocol normalization, capability manifests, session boundaries, and Core-to-Agent dependency direction affect containment.

## Evidence / Claim Limits

Passing means containment against predeclared ownership plus regression success; it does not prove all future Agent changes are contained.
