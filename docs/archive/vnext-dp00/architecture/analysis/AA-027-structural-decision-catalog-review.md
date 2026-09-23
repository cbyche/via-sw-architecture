# AA-027 — Structural Decision Catalog Review

## Status

**COMPLETE — CATALOG REFRAMED; NO ARCHITECTURE ALTERNATIVE SELECTED**

## Review identity and execution record

| Item | Value |
| --- | --- |
| Repository | `cbyche/via-sw-architecture` |
| Fetched `origin/main` | `5f00cea12fadeaf5dc5508691c0d4bfbe080ceaa` |
| Review base SHA | `5f00cea12fadeaf5dc5508691c0d4bfbe080ceaa` |
| Work branch | `arch/decision-catalog-review` |
| Dedicated worktree | `/private/tmp/via-sw-architecture-dp-catalog-review` |
| Review date | 2026-09-11 |
| Identifier-governance correction | 2026-09-12; DP-03 preserved, hosting/isolation assigned DP-16 |

The base matched the user-provided checkpoint. Existing DP-00 experiment
worktrees were not reused or modified.

The governance correction changes identifiers and traceability only. It does
not change any structural question, applicability, dependency, QA relationship,
or recommendation established by the review.

Working checklist:

- [x] inspect source-of-truth and full-text DP references;
- [x] review DP-00~12 against structural admission criteria;
- [x] review six candidate questions and mixed axes;
- [x] preserve Top QA and Approved Baseline meanings;
- [x] publish authoritative catalog, redirects, and conditional roadmap;
- [x] run link/ID/protected-file/diff checks;
- [x] peer read-only reviews for DP overlap, candidate coverage, and QA meaning.

## Sources reviewed

The review included:

- `qa-dp-traceability.md`, `system-overview.md`, all files under
  `docs/architecture/decision-points/`, and all DP references found in full text;
- DP-00 primary/executable records and AA-001/005/006/026;
- `requirements-v1.1.md` and `requirements-vNext.md`;
- the four current QA source documents and evaluation principles/methodology;
- relevant reference-design topology, behavioral, traceability, and assumption
  records; ADR inventory; benchmark/evidence navigation.

Only DP-00 had an independent authoritative DP record before this review.
DP-01~12 were current central rows derived from preliminary `ADP-01`~`ADP-12`
questions in the Approved Baseline. Reference prototype mappings were treated
as implementation observations, never as selected VIA architecture.

## Architecture Decision admission test

A standalone DP passes only when:

1. a product requirement or material quality risk makes the choice necessary;
2. alternatives change responsibility, dependency, interface, authoritative
   state, runtime/process/trust, or lifecycle/recovery boundaries;
3. a later change propagates across identifiable subsystems/contracts;
4. the choice is not already fixed by its parent DP or baseline constraint;
5. structurally distinct alternatives remain under the same parent premise;
6. it does not duplicate another DP;
7. conditional applicability is explicit; and
8. its causal link to current Top QAs and mandatory constraints is stated.

Policy wording does not disqualify a question. Policy ownership/enforcement can
be structural; a value such as timeout, default Agent, allow-list contents, or
ranking weight normally is not. A separate process is also not automatically a
DP unless failure, deployment, trust, recovery, or change impact is material.

## Core findings

1. The catalog mixed parent and child scope. Old DP-03 repeated the DP-00
   Thin/Hybrid/Rich ownership question.
2. It mixed orthogonal axes. DP-06 combined voice composition with ASR
   side-channel use; old DP-03 language blurred logical ownership and hosting;
   DP-04 blurred common interface with specialization.
3. Several alternatives were no longer open under approved requirements.
   FR-03/04 already require an immutable timestamped interaction timeline,
   FR-39 requires common downstream Voice/Text processing, AP-01 requires VIA
   state validation, and AP-02/FR-44 fix provider isolation/profile management.
4. Several rows described algorithms or policy values rather than a durable
   responsibility boundary, especially DP-07 and portions of DP-09/11/12.
5. State and lifecycle were under-specified. The existing material recognizes
   VIA task projection versus executor-authoritative state but had no DP for
   persistence/reconciliation/recovery or lifecycle/cancellation authority.
6. Legacy QA explanations were duplicated and used “candidate/likely” wording
   that could make approved recovery/privacy/containment obligations look
   optional. The obligations remain normative; only future gate encoding is TBD.

## Before / after mapping

| Before ID and meaning | Disposition | After / redirect and reason |
| --- | --- | --- |
| DP-00 Primary Execution Boundary | KEEP | Unchanged authoritative DP; A/B/C/D and deferred selection preserved. |
| DP-01 partial/streaming processing | REFRAME | DP-01 Speculative Input Processing Boundary; policy/tactic values excluded. |
| DP-02 context representation | REFRAME | DP-02 Interaction Evidence Authority; snapshot-vs-timeline choice closed by FR-03/04. |
| DP-03 Thin/Hybrid/Rich capability placement | MERGE / ABSORBED INTO DP-00; ID INACTIVE | Ownership question redirects to DP-00. DP-03 permanently retains its historical Capability Placement meaning. |
| DP-04 generic vs specialized Agent integration | REFRAME | DP-04 Execution Contract Unification Boundary, including local-versus-Agent ports where applicable. |
| DP-05 concurrent resource arbitration | KEEP | Structural when it decides resource/admission authority; numeric policy excluded. |
| DP-06 voice composition and ASR side-channel | SPLIT | DP-06 retains voice runtime composition; partial/delta use moves to DP-01 and evidence authority to DP-02. |
| DP-07 Model Gateway selection policy | POLICY_OR_CONFIGURATION_NOT_STANDALONE_DP | FR-44/AP-02 own the fixed structural constraints; reopen only with evidence of a new policy-authority boundary. |
| DP-08 Voice/Text unification | REFRAME | DP-08 locates canonical turn/anchor authority without reopening FR-39. |
| DP-09 existing/new task decision | REFRAME | DP-09 task-association authority/state-index boundary; classifier/scorer values are mechanisms. |
| DP-10 intent refinement | KEEP / REFRAME | Component responsibility decomposition under the semantic owner fixed by DP-00. |
| DP-11 Agent routing | REFRAME | Eligibility/semantic selection/route-commit split inside the DP-00-fixed owner; does not reopen owner placement. |
| DP-12 capability contract | REFRAME | Capability catalog and registration/health/version authority; execution lifecycle split out. |
| No explicit owner for task projection/executor state reconciliation | ADD DP-13 | Execution State and Recovery Authority. |
| No explicit owner for lifecycle control/cancel races | ADD DP-14 | Execution Lifecycle Control Boundary. |
| No explicit cross-system dependency-failure boundary | ADD DP-15 | Failure Containment and Resilience Boundary. |
| No explicit VIA-local hosting/isolation boundary | ADD DP-16 | Local Execution Hosting and Isolation Boundary, conditional for C/D. |

DP-03 was briefly reused for the hosting question in commit `7eb83eae`. The
follow-up governance correction assigns that question to DP-16. DP-03 is
inactive and permanently retains Capability Placement identity; historical
references are not rewritten.

## Six-question coverage matrix

| Candidate question | Ambiguity / DP-00 scope | Remaining structural decision | Placement | DP-00 applicability / relationships |
| --- | --- | --- | --- | --- |
| Q1. Put an execution runtime “inside VIA”? | “Inside” mixes logical ownership, product deployment, and process hosting. DP-00 already fixes logical primary/local execution ownership. | None at ownership level; hosting remains. | Ownership ABSORBED by DP-00; hosting redirects to DP-16. DP-03 remains the inactive historical Capability Placement ID. | A: no VIA local executor; B: ARGO authority is not automatically VIA; C/D: VIA-local path exists. |
| Q2. Local capability in VIA process or separate runtime/service? | “Local” must mean VIA-owned/on-device, and runtime/service differ in supervision/deployment. | IPC, isolation, privilege, crash/restart, upgrade boundary. | DP-16. | C/D direct; A/B N/A. A local ARGO hosting question would require separate admission because ARGO is not VIA-owned. Orthogonal to DP-04. |
| Q3. Same execution abstraction for VIA-local and Agent execution? | Common outer lifecycle contract is not identical implementation or shared runtime type. | One port, common core plus extensions, or distinct ports/adapters. | DP-04. | C/D strongest; A Agent commonality; B ARGO-to-specialist. Hosting remains DP-16. |
| Q4. Should VIA or executor own capability/execution state? | Conflates catalog/health, VIA task projection, domain execution truth, and persistence/recovery. DP-00 already constrains projection/domain authority by family. | Persistence, checkpoint, reconciliation, idempotency, and recovery authority. | Catalog state DP-12; task/execution reconciliation DP-13. | All; B/D most discriminating. DP-13 supplies DP-09 state. |
| Q5. Who owns capability registration and execution lifecycle? | These are two lifecycles. DP-00 fixes route/owner families, not publisher/validator or command authority. | Registration/withdrawal/health/version; separately start/follow-up/pause/resume/cancel/terminal arbitration. | Registration DP-12; execution lifecycle DP-14. | Registration and lifecycle apply to all; local publication is C/D conditional. |
| Q6. Where is the failure/cancellation boundary? | Conflates process blast radius, dependency failure propagation, domain cancellation, and realtime audio interruption. | Hosting isolation; cross-system detection/isolation/degrade/retry; task cancel acknowledgement and terminal race. | Local hosting input DP-16; cancellation DP-14; cross-system containment DP-15; voice barge-in remains DP-01/06/08 concern. | All; C/D local isolation, B/D cross-owner control are strongest. |

### Evidence anchors for the matrix

| Question | Existing authoritative evidence location |
| --- | --- |
| Q1 | `docs/architecture/decision-points/DP-00-primary-execution-boundary.md:28-82` (question and ownership independent variable); `docs/architecture/decision-points/DP-00-executable-architecture-spec.md:561-583` (family responsibility matrix). |
| Q2 | `docs/architecture/decision-points/DP-00-executable-architecture-spec.md:561-583` (responsibility, explicitly not process count); `docs/evaluation/dp00-experimental-boundary.md:134-151` (IPC/serialization may be architecture-owned); `docs/requirements/requirements-v1.1.md:452-464` (recovery/multi-process risk). |
| Q3 | `docs/requirements/requirements-v1.1.md:370-380` (Agent execution contract); `docs/architecture/decision-points/DP-00-executable-architecture-spec.md:198-217,749-784` (canonical observations/interfaces are not mandatory shared runtime components/types). |
| Q4 | `docs/architecture/decision-points/DP-00-executable-architecture-spec.md:46-71,309-350,587-610` (common task obligations, B authority split, state matrix); `docs/requirements/requirements-v1.1.md:166-179` (task/recovery requirements). |
| Q5 | `docs/requirements/requirements-v1.1.md:161-164,370-380` (Registry versus execution contract); `docs/architecture/decision-points/DP-00-executable-architecture-spec.md:264-275,339-348,421-430,522-531` (family-specific lookup/lifecycle responsibilities). |
| Q6 | `docs/requirements/requirements-v1.1.md:170-189,466-478,684-690` (lifecycle/cancel, failure containment, injection suite); `docs/architecture/decision-points/DP-00-executable-architecture-spec.md:46-63` (common cancellation obligation). |

AA-026 sections 16~18 and 22 preserve the original unresolved list,
conditional roadmaps, old DP-03 recommendation, and DP-00 revisit triggers.

## Dependency and conditional roadmap

Arrows in this relationship view mean **consumes or informs**, not strict
one-pass execution order:

```text
DP-00 — logical execution ownership/topology family (selection deferred)
  ├─ DP-02 interaction evidence authority ─┬─ DP-01 speculation
  │                                       ├─ DP-08 modality unification
  │                                       └─ DP-10 intent refinement
  ├─ DP-12 capability catalog/registration ┬─ DP-11 route decision
  │                                        └─ DP-05 resource arbitration
  ├─ DP-13 state/recovery authority ────────┬─ DP-09 task association
  │                                        └─ DP-14 lifecycle/cancellation
  ├─ DP-15 failure containment ─────────────┬─ DP-13 recovery facts
  │                                        ├─ DP-14 lifecycle transition
  │                                        └─ DP-11 replacement-route commit
  └─ [C/D] DP-16 local hosting/isolation ───┬─ DP-04 execution contract
                                           └─ DP-15 local blast radius

DP-06 voice composition → DP-01 partial artifacts + DP-02 evidence events
```

Sequencing is conditional, not a mandate to evaluate every combination:

- all-family control sequence: establish DP-13 authority/recovery model, then
  DP-14 lifecycle/cancellation semantics, then DP-15 containment strategy;
  reconcile feedback explicitly rather than treating the graph as acyclic;
- capability foundation: DP-12 facts before DP-11/05 consumers;
- C/D local-execution branch: DP-16 assumes a minimal host seam, then feeds
  hosting constraints into coordinated DP-04 and DP-15 work;
- semantic chain: DP-02/01/08 → DP-10 → DP-09 → DP-11;
- concurrency: DP-12 resource declaration + DP-13/14 lifecycle → DP-05;
- revisit DP-00 only when AA-026 triggers become decision-relevant.

## Relationship to AA-026 and the next investigation

AA-026 is unchanged historical synthesis. Its recommendation to investigate
“DP-03 Capability placement” was correct under the old catalog framing and
remains evidence that local placement was the E4 discriminator. This review
found that old question already duplicates the DP-00 ownership families.

The next structural investigation is therefore **DP-13 — Execution State and
Recovery Authority**:

> For each VIA task and executor execution, who owns durable task/execution
> truth, and which correlation/checkpoint/reconciliation contract preserves
> recovery and cancellation correctness?

It applies to A/B/C/D, tests B's VIA-projection/ARGO-authority split and D's
ownership transfer directly, and connects mandatory recovery/task-integrity
requirements before another experiment is designed. DP-16 is the next
conditional C/D investigation; this review does not run either investigation.

## QA preservation result

Top QA-01~04 definitions and evaluation meaning are unchanged. The current
four-bucket presentation is:

1. current scored Top QA-01~04 and authoritative metric links;
2. retained current mandatory/supporting obligations;
3. retained Legacy diagnostic/correctness-slice mappings and approved targets;
4. identifier migration/history and superseded duplicate explanation.

`docs/architecture/qa-legacy-migration.md` is authoritative for this navigation.
No baseline requirement was declared obsolete. Recovery, cancellation,
task-state integrity, failure containment, privacy/trust, resource coexistence,
and audit remain visible current obligations. Only the duplicated working
explanation is superseded.

## Peer-review record

Three read-only reviews were used:

- parent/lower-DP overlap and mixed-axis audit;
- six-question coverage and missing-decision audit;
- Top-QA/Legacy-QA semantics and protected-artifact audit.

The reviewers agreed on the principal overlap and missing state distinction;
their agreement is not the evidence basis. The file/requirement/contract anchors
above and the per-item admission test are the basis.

## Non-decisions

This review does not:

- select DP-00 A/B/C/D or any downstream alternative;
- change a requirement baseline, QA metric, gate, threshold, evaluator, or raw
  result;
- authorize a prototype, benchmark campaign, or production technology;
- treat reference implementation structure as approved architecture.
