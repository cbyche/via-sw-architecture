# DP-14 — Execution Lifecycle Control Boundary

## Status

**OPEN — NO ALTERNATIVE SELECTED**

## Decision question

> Which component owns execution lifecycle commands and resolves races between
> start/follow-up/pause/resume/cancel requests and executor terminal events?

## Why this is architectural

Registration lifecycle and execution lifecycle are different. Registering a
capability makes it discoverable (DP-12); starting and controlling one execution
creates task/executor state, command acknowledgement, cancellation propagation,
and terminal-event arbitration. FR-17/23/24/25/26/28/31/40 and mandatory
cancellation/failure constraints require an end-to-end owner. Changing that
owner affects Task Manager, executor interface, event schema, state machine,
timeouts, recovery, and user feedback.

## Scope

In scope:

- ownership of start, follow-up, pause, resume, cancel, and status commands;
- accepted/rejected/unknown command acknowledgement;
- cancellation propagation and cancel-versus-complete arbitration;
- application of a DP-15 failure decision as a lifecycle transition;
- terminal-event ordering and transition authority.

Out of scope:

- cancellation timeout values or retry counts;
- capability registration lifecycle (DP-12);
- durable state/recovery mechanism (DP-13);
- process crash containment and hosting (DP-16);
- failure detection/classification and retry/degrade/controlled-failure strategy
  (DP-15);
- internal Agent tool-step lifecycle prohibited by CON-04/AP-05.

## Parent constraints and applicability

All DP-00 families require a user-facing control path. A/C mediate delegated
Agent lifecycle through VIA; B must not make VIA authoritative for ARGO's domain
execution state; D must propagate commands to the committed route and prevent
owner bouncing. Local and Agent routes may share or adapt contracts according
to DP-04.

## Structural alternatives to investigate

Examples, not selections:

- VIA lifecycle coordinator with executor acknowledgements and terminal events;
- executor-owned lifecycle with a VIA command/projection gateway;
- split command authority with a formally defined terminal-event arbiter.

Failure containment and cancellation are related but not identical. Local
process blast radius and restart isolation are DP-16 inputs; cross-system
containment belongs to DP-15; command propagation and race resolution belong
here. All must satisfy the same mandatory constraints.

## QA and dependencies

Direct: Top QA-01/02/03. Qualification: cancellation semantics, failure
containment, task-state integrity, recovery, and audit. DP-14 depends on DP-04's
contract topology and DP-13's authoritative/projected state split, applies
DP-15's containment outcome without choosing the strategy, and supplies DP-05
lifecycle outcomes for resource release.

## Decision

TBD.
