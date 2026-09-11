# DP-15 — Failure Containment and Resilience Boundary

## Status

**OPEN — NO ALTERNATIVE SELECTED**

## Decision question

> Which component detects and contains execution/dependency failure, owns
> retry/degrade coordination and authorization of recovery routing, and chooses
> controlled failure so unrelated tasks remain isolated?

## Why this is architectural

Legacy v1.1 QA-04, FR-17/30/40/41, and mandatory qualification constraints
require failure detection, bounded propagation, truthful state, and unaffected
task continuity. The alternatives change supervisor, dependency, retry,
state-transition, and user-feedback responsibilities across VIA, ARGO,
Specialist Agents, and local executors. This is broader than one process choice.

## Scope

In scope:

- failure-domain and blast-radius boundaries;
- detection and classification authority;
- ownership of retry/degrade coordination and the decision whether failure may
  trigger recovery routing or controlled failure;
- isolation between concurrent tasks and dependencies;
- failure events delivered to state/lifecycle and user projection.

Out of scope:

- retry counts, backoff durations, or route ranking values;
- replacement-route selection and commit, which belong to DP-11 after DP-15
  authorizes recovery routing;
- local-executor process hosting, which is DP-03 input;
- cancellation command/race ownership, which is DP-14;
- persistence/reconciliation mechanism, which is DP-13;
- reimplementing an Agent's internal tool security or recovery in VIA.

## Parent constraints and applicability

All DP-00 families must satisfy the same containment obligations. A/C mediate
Agent failures at the VIA boundary; B must preserve ARGO domain authority while
preventing primary-runtime failure from corrupting unrelated VIA tasks; D must
contain failures per committed route and make any alternate-route transition
explicit. C/D additionally consume DP-03 local-host isolation outcomes.

## Structural alternatives to investigate

Examples, not selections:

- central VIA failure coordinator consuming normalized executor events;
- executor-owned recovery with VIA containment/projection adapters;
- layered local supervisor plus cross-executor task isolation contract.

## QA and dependencies

Direct: Top QA-02/03 and Legacy v1.1 QA-04. Qualification: failure containment,
task-state integrity, recovery, trust/privacy, and audit. DP-15 consumes DP-03
hosting and DP-04 event contracts, informs DP-13 state, requests lifecycle
transitions through DP-14, and delegates replacement-route selection/commit to
DP-11 when recovery routing is allowed.

## Decision

TBD.
