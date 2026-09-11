# DP-13 — Execution State and Recovery Authority

## Status

**OPEN — NO ALTERNATIVE SELECTED**

## Decision question

> How are VIA's user-facing task projection and executor-authoritative domain
> execution state reconciled and recovered without creating two competing
> authorities?

## Why this is architectural

“Capability state” is not one state. The architecture must distinguish:

1. capability catalog/health facts (DP-12);
2. VIA's user-facing task projection and correlation state;
3. executor-authoritative plan/thread/tool/domain execution state;
4. persistence, checkpoint, replay, reconciliation, and deduplication mechanism.

FR-23/25/26/30/40, AP-06/07, Top QA-02/03, and mandatory task-state integrity
and recovery constraints require explicit authority across disconnects, process
failure, retry, and ownership transfer. Changing this split affects Task
Manager, executor adapters, event identity, durable storage, recovery workers,
result binding, and audit.

## Scope

In scope:

- authoritative versus projected fields and state transitions;
- execution/task correlation identifiers and event ordering;
- persistence/checkpoint ownership and recovery handshake;
- reconciliation after disconnect/restart;
- idempotency and completed-side-effect deduplication responsibility;
- ownership-transfer state needed by adaptive routes.

Out of scope:

- capability registration/health authority (DP-12);
- storage product/schema tuning;
- lifecycle command ownership and cancel races (DP-14);
- process hosting (DP-03);
- user-facing task-association decision (DP-09).

## Parent constraints and applicability

- **A/C:** VIA owns task lifecycle projection while delegated Agents own domain
  execution; C also correlates local results.
- **B:** the frozen DP-00 specification explicitly makes ARGO execution
  plan/thread/state authoritative and VIA a user-facing projection/correlation.
- **D:** VIA records committed routes and must reconcile local, ARGO-primary,
  and Specialist execution without unrestricted authority bouncing.

Those parent facts constrain alternatives but do not select persistence,
reconciliation, or recovery structure.

## Structural alternatives to investigate

Examples, not selections:

- event-fed VIA projection with executor-owned checkpoints and recovery query;
- shared durable execution journal with explicit single-writer domains;
- coordinator-owned lifecycle journal plus opaque executor recovery token.

## QA and dependencies

Direct: Top QA-02/03. Qualification: required task-state integrity, recovery,
privacy/trust, failure containment, and audit. DP-13 consumes DP-00 authority,
coordinates with DP-03/04/12/14, and supplies the state index used by DP-09.

## Decision

TBD.
