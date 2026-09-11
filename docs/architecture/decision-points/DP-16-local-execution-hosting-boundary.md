# DP-16 — Local Execution Hosting and Isolation Boundary

## Status

**OPEN — CONDITIONAL STRUCTURAL INVESTIGATION; NO ALTERNATIVE SELECTED**

## Decision question

> When VIA logically owns a local executor, is it hosted in the VIA process or
> behind a separately supervised local runtime boundary?

## Why this is architectural

DP-00 already decides whether a topology contains VIA-owned local execution.
It does not decide the process/runtime hosting boundary. Hosting changes IPC,
deployment, privilege/credential boundaries, fault blast radius, restart and
upgrade lifecycle, cancellation propagation, and latency. A later change
therefore touches VIA orchestration, local executor adapters, supervision,
state/recovery, packaging, observability, and security review.

This DP is driven by Top QA-01 and QA-03 plus mandatory failure containment,
recovery, trusted-boundary, cancellation, and task-state integrity constraints.

## Scope

In scope:

- in-process module versus separate local runtime/service hosting;
- IPC and supervision boundary;
- crash/restart and deployment/upgrade isolation;
- privilege, credential, and resource-isolation consequences;
- the minimum host-facing execution adapter.

Out of scope:

- whether VIA owns local execution at all (DP-00);
- which capabilities are allow-listed or their policy values;
- common versus distinct execution lifecycle interface (DP-04);
- authoritative execution-state persistence (DP-13);
- cancellation/terminal-event ownership (DP-14);
- selecting a production runtime technology.

## Parent constraints and applicability

| DP-00 family | Applicability |
| --- | --- |
| A | Not applicable unless a later DP-00 revision adds VIA-owned local execution. |
| B | Not applicable to an ARGO-owned executor merely because it runs locally; logical owner is not VIA. |
| C | Directly applicable to the bounded VIA Fast Path. |
| D | Applicable to the VIA Fast route; ARGO/Specialist hosting remains executor-owned. |

## Structural alternatives to investigate

Examples, not selections:

1. in-process local executor with explicit module and failure guards;
2. separately supervised local runtime with IPC and independent restart;
3. capability-class-dependent hosting only if a stable requirement forces both.

These alternatives must not be mixed with “one common execution interface” as
if they were mutually exclusive. Hosting and interface unification are
composable DP-16 and DP-04 axes.

## Dependencies and evidence plan

- DP-12 supplies capability identity/trust/resource facts.
- DP-16 assumes only a minimal host seam; its hosting constraints inform later
  coordinated DP-04 execution-contract work.
- DP-13 defines state/recovery authority.
- DP-14 defines lifecycle and cancellation authority.
- DP-15 defines cross-system failure containment and consumes this DP's local
  process-isolation outcome.

When this conditional investigation begins, it should first derive
hosting-sensitive scenarios from FR-42, FR-30, FR-40/41, CON-01/03/04, Legacy
v1.1 QA-03/04/06, and Top QA-01/03. It should compare responsibility and
failure models before any prototype or benchmark campaign is authorized.

## Catalog history

This question was initially drafted under a reused DP-03 identifier in commit
`7eb83eae`. The governance correction moved it to DP-16 before any alternative
was investigated or selected. DP-03 permanently retains its historical
Capability Placement meaning and is inactive because that question is absorbed
into DP-00.

## Decision

TBD. This review does not choose hosting.
