# VIA System Overview — Active vNext Working View

## Status

This view does not modify the approved v1.1 baseline and does not select an architecture winner. The active DP catalog is `decision-points/vnext/`; older A/B/C/D documents are historical records.

## Product boundary common to candidates

VIA is Voice-first and Text-capable, owns a stable user-facing Task identity, enforces context/permission boundary interaction, preserves correlation and provenance, and delivers validated results. Domain action remains subject to consent, least privilege, cancellation, recovery, and audit obligations.

## DP-00 responsibility alternatives

```text
R1 — Agent-neutral Control Plane
User → VIA interaction/turn/grounding/goal/initial delegation/task UX
     → selected Agent: domain reasoning/planning/arbitrary tools/workflow state

R3 — Primary General-purpose Agent Runtime
User → VIA interaction/task correlation/permission boundary/result delivery
     → primary Agent Runtime: substantive interpretation/planning/tools/workflow
       → specialist Agent delegation
```

R3 is vendor-neutral; ARGO is only a possible realization. R1 does not make VIA a general-purpose Agent runtime.

R1+@ preserves R1 ownership and adds only bounded deterministic read-only local execution. Speech-stop and provisional-work cancellation are interaction controls. Any state-changing action, arbitrary tool selection, planning loop, durable workflow, or independent Agent execution state requires architectural reclassification.

## Remaining active structural decisions

| DP | Boundary |
| --- | --- |
| DP-01 | committed canonical UserTurn authority |
| DP-02 | separate versus unified R1 grounded-goal/routing authority |
| DP-03 | temporal historical-evidence retention authority |
| DP-04 | pre-admission versus lifecycle context materialization |
| DP-05 | shared versus task-scoped native Agent session state |
| DP-06 | co-hosted versus detached durable Task supervision |
| DP-07 | single versus channel-owned response semantics |
| DP-08 | shared versus enforceably realtime-isolated inference resources |
| DP-09 | Core-neutral versus device-family-owned capability semantics |

The choices are related but independently structural. For example, R1/R3 does not decide native session sharing, Task Supervisor deployment, response publication authority, or inference resource isolation.

## Evaluation invariants

- No family is preferred before measurement.
- Every DP uses all twelve frozen QA-v1 definitions and populations.
- DP-00 compares QA-01 and QA-02 first without an arbitrary weighted sum.
- QA-09 remains a non-offsettable security hard gate.
- The same semantic capability, Agent profiles, Voice/Text obligations, events/faults, and device/resource workloads apply to integrated candidates.
- Historical A/B/C/D scores retain their identities and do not become QA-v1 results.
- No Cloud/model API is required for the prepared evaluation.

See [`decision-points/catalog.md`](decision-points/catalog.md), [`qa-dp-traceability.md`](qa-dp-traceability.md), and the [`DP-00 protocol`](../evaluation/dp00-vnext-evaluation-protocol-v1.md).
