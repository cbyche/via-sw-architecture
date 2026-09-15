# DP-00 — Primary Reasoning & Execution Boundary

## Status

**ACTIVE — ALTERNATIVES DEFINED — NO WINNER PRESELECTED — NOT YET EVALUATED**

## Decision question

Where does primary substantive reasoning, planning, execution workflow, and specialist-delegation authority reside: an agent-neutral VIA Control Plane or a primary general-purpose Agent Runtime?

## Base alternatives

### R1 — Agent-neutral Control Plane

VIA owns user interaction; canonical input/turn handling; context and temporal grounding; grounded user-goal representation; agent-neutral initial routing/delegation; user-facing task lifecycle; consent interaction; and result/response binding.

The selected downstream Agent owns domain reasoning, planning, arbitrary tool selection/execution, and domain execution state/workflow. R1 does not make VIA a general-purpose Agent runtime and does not prescribe a number of model calls.

### R3 — Primary General-purpose Agent Runtime

One vendor-neutral general-purpose Agent Runtime is structurally primary for substantive semantic interpretation, planning, tool/runtime execution, execution state/workflow, and downstream specialist delegation. VIA retains Voice/Text interaction, user-facing Task correlation, context/permission boundary interaction, and result delivery. ARGO can realize R3 but is not the architecture definition.

## Evaluated tactic realization

**R1+@ = R1 plus bounded deterministic read-only local execution.** The tactic has no state-changing user/domain action, open-ended planning, durable workflow, arbitrary tool selection, or Agent execution state. Stopping VIA speech playback and cancelling provisional work are interaction control and do not violate the read-only restriction. Growth beyond this boundary requires a new architecture decision and reclassification.

## Structural discriminator

The discriminator is which side of the VIA-to-Agent boundary owns substantive semantic interpretation, planning, tool execution, workflow state, and specialist delegation. R1+@ does not move that authority.

## Evaluation binding

Primary QAs: **QA-01, QA-02, QA-04, QA-05**. All QA-01 through QA-12 are measured. The candidate contracts and next-task protocol are in `benchmark/contracts/dp-vnext/` and `docs/evaluation/dp00-vnext-evaluation-protocol-v1.md`.

## Decision policy

No base family is preferred. Qualify functional obligations, QA-09's hard gate, and invariants; compare QA-01 and QA-02 without collapsing them into a weighted sum; then examine the remaining causally relevant QAs. Any tactic remediation must be evaluated as a full integrated candidate. Final selection uses measured QA-v1 evidence only.

Historical A/B/C/D results remain historical and are mapped in `MIGRATION.md`.
