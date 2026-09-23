# VIA Structural Decision Catalog

## Status and authority

**Authoritative catalog for vNext structural decision identities, dispositions,
and open Decision Points.**

This catalog supersedes the preliminary `ADP-01`~`ADP-12` candidate table in
`docs/requirements/requirements-v1.1.md` for current architecture-work
navigation only. It does not edit or reinterpret that Approved Baseline, select
a DP-00 alternative, or change any frozen evaluation contract.

The catalog review and complete before/after mapping are recorded in
`docs/architecture/analysis/AA-027-structural-decision-catalog-review.md`.

## Decision admission rule

A standalone DP must be driven by a product requirement or material quality
risk and must change at least one of these when alternatives change:

- component responsibility or dependency direction;
- an externally meaningful interface or contract;
- authoritative state ownership and reconciliation;
- process/runtime or trust/failure boundary;
- deployment, recovery, or lifecycle boundary.

A numeric value, allow-list entry, default route, timeout, or ranking weight is
policy/configuration unless the open question is which component owns,
enforces, observes, and validates that policy. Process separation alone is also
insufficient without a requirement-driven change surface. Prototype structure
is evidence about feasibility, not an approved product architecture.

Each DP must also be distinct from its parent: alternatives must remain
structurally different under the same DP-00 premise. Composable axes are not
written as one mutually exclusive alternative list.

## Fixed parent boundary

DP-00 remains exactly the A/B/C/D Primary Execution Boundary defined by its
authoritative records. Its evidence and deferred-selection status are unchanged.

| DP-00 family | Parent constraint relevant to downstream DPs |
| --- | --- |
| A — Thin VIA | Substantive execution is delegated; no VIA-owned local execution runtime is required. |
| B — ARGO-centric | ARGO owns primary substantive execution state; VIA keeps a user-facing task projection/correlation. |
| C — Hybrid VIA Fast Path | A bounded VIA-owned local execution path exists in addition to Agent delegation. |
| D — Adaptive Per-turn | A selector commits a per-turn topology; VIA-local, ARGO-primary, and Specialist routes may exist with explicit ownership transfer. |

Logical VIA ownership, same-process hosting, common execution interfaces,
execution-state authority, registration ownership, request-time routing, and
cancellation are separate axes. Some parent families make an axis inapplicable;
they do not make the axes synonymous.

## Catalog disposition summary

| ID | Structural decision question | Disposition | Applies under DP-00 | Authoritative detail |
| --- | --- | --- | --- | --- |
| DP-00 | Where does primary substantive reasoning/execution authority sit? | KEEP; characterized, selection deferred | A/B/C/D | `DP-00-primary-execution-boundary.md` and executable spec |
| DP-01 | Which pre-commit input-processing stages may create speculative artifacts, and who validates, cancels, or commits them? | REFRAME | A/B/C/D | This catalog until a dedicated DP is needed |
| DP-02 | Which component owns the immutable interaction-evidence timeline and exposes time-correlated referents across execution boundaries? | REFRAME | A/B/C/D | This catalog until a dedicated DP is needed |
| DP-03 | Should substantive work be delegated or may VIA execute local-safe capabilities? | ABSORBED INTO DP-00 / INACTIVE | Historical A/B/C/D ownership input | `DP-03-capability-placement-boundary.md` |
| DP-04 | Across the executors applicable to a DP-00 family, what common execution-contract core and specialization boundary should exist? | REFRAME | C/D local-versus-Agent; A/B Agent-to-Agent/ARGO-to-Specialist | This catalog until a dedicated DP is needed |
| DP-05 | Which component owns cross-task admission and exclusive-resource arbitration? | KEEP | A/B/C/D where concurrent conflicting work exists | This catalog until a dedicated DP is needed |
| DP-06 | Is the voice runtime an integrated S2S boundary or a composition of replaceable media/ASR/reasoning/TTS stages? | SPLIT/REFRAME | A/B/C/D | This catalog until a dedicated DP is needed |
| DP-08 | At which boundary do Voice and Text become a canonical logical turn while retaining modality-specific anchors and media lifecycle? | REFRAME | A/B/C/D | This catalog until a dedicated DP is needed |
| DP-09 | Which component owns existing-task association and ambiguity escalation, and what state index does it consume? | REFRAME | A/B/C/D; especially D | This catalog until a dedicated DP is needed |
| DP-10 | How are normalization, grounding, clarification, validation, and canonical-request commitment divided across components? | KEEP/REFRAME | A/C/D; B must not recreate Thin-VIA authority | This catalog until a dedicated DP is needed |
| DP-11 | Within the DP-00-selected routing owner, how are eligibility filtering, semantic selection, and route commitment divided? | REFRAME | A/B/C/D with different owner | This catalog until a dedicated DP is needed |
| DP-12 | How are declaration, registration, health, compatibility, and version authorities divided and projected into a canonical capability catalog? | REFRAME | A/B/C/D | This catalog until a dedicated DP is needed |
| DP-13 | How are VIA's user-facing task projection and executor-authoritative execution state reconciled and recovered? | NEW | A/B/C/D; strongest discriminator for B/D | `DP-13-execution-state-and-recovery-authority.md` |
| DP-14 | Which component owns execution lifecycle commands and terminal-event arbitration across the execution boundary? | NEW | A/B/C/D | `DP-14-execution-lifecycle-control-boundary.md` |
| DP-15 | Which component detects/contains failures, coordinates retry/degrade, authorizes recovery routing, and chooses controlled failure? | NEW | A/B/C/D | `DP-15-failure-containment-boundary.md` |
| DP-16 | When a VIA-owned local executor exists, is it hosted in the VIA process or behind a separately supervised local runtime boundary? | NEW ID for previously drafted hosting question | C/D; conditional variants only where VIA owns execution | `DP-16-local-execution-hosting-boundary.md` |

DP-07 is not active as a standalone DP. Model choice, timeout, retry count/
backoff/profile values, cost weight, and default binding remain versioned policy/configuration under FR-44
and AP-02. A future proposal may reopen a structural DP only if it demonstrates
a material change to policy ownership/enforcement or the Model Gateway boundary.

## Catalog DP cards

### DP-01 — Speculative Input Processing Boundary

- **Decision question:** Which pre-commit input-processing stages may create
  speculative artifacts, and which component validates, invalidates, cancels,
  or commits them after final input?
- **Need / risk:** FR-01/03/04 and responsiveness motivate early work, while
  stale partial input can corrupt request identity, referent binding, or route
  commitment (Top QA-01/02/04).
- **QA / mandatory constraints:** Top QA-01 measures saved/added latency,
  QA-02 detects stale-artifact correctness, and QA-04 observes pre-commit model
  calls; turn identity and required realtime interruption remain mandatory.
- **In scope:** speculative transcript-derived intent, read-only prefetch,
  artifact identity, invalidation and commit interface.
- **Out of scope:** ASR vendor choice, timing thresholds, and final route policy.
  DP-01 invalidates or prevents commit of pre-route speculative artifacts; user
  task execution cancellation and cancel-versus-complete races belong to DP-14.
- **Parent constraint:** DP-00 fixes who may own semantic/execution authority;
  speculation cannot smuggle a second owner into A/B/C/D.
- **Open structure / change surface:** a final-turn-only pipeline versus a
  speculative stage and artifact store changes Voice Runtime, Context Engine,
  Intent Refiner, cancellation hooks, and request-commit contracts.
- **Dependencies / condition:** consumes DP-02 evidence; precedes DP-10
  optimization. Applicable to every family, with the semantic owner differing.
- **Status / authority:** Open; catalog-level definition is sufficient.

### DP-02 — Interaction Evidence Authority

- **Decision question:** Which component owns the immutable interaction-evidence
  timeline and exposes time-correlated referent candidates across boundaries?
- **Need / risk:** FR-03/04 already require timestamped immutable evidence;
  the old single-snapshot alternative is therefore not open. Incorrect temporal
  binding affects Top QA-02 and external-context exposure affects mandatory
  privacy constraints.
- **QA / mandatory constraints:** Top QA-01 covers capture/transfer latency,
  QA-02 covers temporal binding, and QA-03 covers adapter/storage propagation;
  privacy, provenance, consent, and audit obligations remain mandatory/supporting.
- **In scope:** capture/clock normalization, timeline authority, immutable
  identity/provenance, and the query/transfer contract.
- **Out of scope:** whether FR-03 exists, a particular vision model, and egress
  policy values.
- **Parent constraint:** DP-00 changes the consumer and transfer boundary, not
  the requirement for authoritative evidence.
- **Open structure / change surface:** centralized Context Engine authority
  versus adapter-owned capture with canonical ingestion changes input adapters,
  storage, referent binding, and context-transfer interfaces.
- **Dependencies / condition:** feeds DP-01, DP-08, and DP-10; all A/B/C/D.
- **Status / authority:** Open; catalog-level definition is sufficient.

### DP-03 — Capability Placement Boundary

- **Historical question:** Should substantive work be delegated to a
  Downstream Agent, or may VIA directly execute bounded or broader local-safe
  capabilities?
- **Disposition:** **ABSORBED INTO DP-00 / INACTIVE.** DP-00 owns the logical
  execution-placement alternatives. Historical DP-03 references permanently
  retain this meaning.
- **Redirect:** `DP-03-capability-placement-boundary.md`. Physical hosting and
  isolation of a VIA-owned local executor is the distinct DP-16.

### DP-04 — Execution Contract Unification Boundary

- **Decision question:** Across the executors applicable to a DP-00 family,
  what common execution-contract core and specialization boundary should exist?
- **Need / risk:** a common contract can reduce integration propagation (Top
  QA-03), while forcing local, streaming, or specialized semantics into one
  lowest-common-denominator port can impair correctness and responsiveness
  (Top QA-01/02).
- **QA / mandatory constraints:** Top QA-01 measures adapter/hop overhead,
  QA-02 contract conformance, and QA-03 change containment; required status,
  cancellation, task integrity, trusted-boundary, and audit semantics must survive.
- **In scope:** execute/progress/status/result/error/cancel contract topology,
  extension points, adapters, and version compatibility.
- **Out of scope:** same-process versus service hosting (DP-16), capability
  metadata registration (DP-12), route ranking (DP-11), and timeout values.
- **Parent constraint:** A exposes Agent-to-Agent port commonality, B exposes
  ARGO-to-Specialist integration, and C/D add the local-versus-Agent question.
  B cannot demote ARGO authority through a VIA facade.
- **Open structure / change surface:** one port, common core plus extensions, or
  distinct ports changes Orchestrator/Task Manager, local executor, Agent
  harness adapters, event schema, and compatibility testing.
- **Dependencies / condition:** DP-16 may first assume a minimal host seam and
  feed hosting constraints into coordinated DP-04 work; DP-04 also coordinates
  with DP-12 and DP-14. Open; catalog-level definition is sufficient until
  interface work.

### DP-05 — Cross-task Resource Arbitration Authority

- **Decision question:** Which component owns admission, ordering, and conflict
  resolution when independent executions contend for an exclusive resource?
- **Need / risk:** FR-40/41 require independently controlled tasks and conflict
  handling; poor ownership couples unrelated failures and harms Top QA-01/02.
- **QA / mandatory constraints:** Top QA-01/02 measure contention latency and
  correct isolation, while Top QA-03 exposes scheduler/adapter propagation;
  capacity/co-existence, independent cancellation, and failure isolation remain
  retained constraints.
- **In scope:** resource-declaration consumption, leases/queues, priority and
  admission responsibility, and task-facing decisions.
- **Out of scope:** scheduling an Agent's internal tool calls, queue sizes,
  numeric priorities, or user-confirmation wording.
- **Parent constraint:** arbitration must respect the selected execution owner
  and CON-04; VIA may coordinate execution/resource leases without becoming a
  central Tool Gateway.
- **Open structure / change surface:** central admission, per-resource
  coordinators, or executor-cooperative scheduling changes Task Manager,
  Registry/resource contract, executor adapters, state, and failure handling.
- **Dependencies / condition:** DP-12 resource declarations and DP-14 lifecycle;
  active only where concurrent conflicting work exists. Open; catalog-level.

### DP-06 — Voice Runtime Composition Boundary

- **Decision question:** Is the voice runtime an integrated S2S boundary or a
  composition of replaceable media, ASR, reasoning, and TTS stages?
- **Need / risk:** CON-02 and FR-44 require a voice-first, traceable runtime;
  stage replacement, event correctness, and failure isolation affect Top
  QA-01/02/03/04 and mandatory containment constraints.
- **QA / mandatory constraints:** Top QA-01 covers audio/turn latency, QA-02
  transcript/event correctness, QA-03 stage replaceability, and QA-04
  pre-route model stages; realtime interruption, failure containment, profile
  traceability, and audit remain mandatory/supporting.
- **In scope:** runtime/stage responsibility, provider adapter boundaries,
  streaming event contracts, and restart/fallback boundaries.
- **Out of scope:** independent transcript/delta speculation, which belongs to
  DP-01; interaction-timeline authority, which belongs to DP-02; provider name
  and model-selection policy.
- **Parent constraint:** voice composition cannot silently move DP-00 semantic
  or execution authority.
- **Open structure / change surface:** integrated S2S, S2S plus observational
  adapters, or decomposed stages change Voice Runtime, Model Gateway adapters,
  event contracts, observability, and process lifecycle.
- **Dependencies / condition:** coordinates with DP-01/02/08; all A/B/C/D.
  Open; catalog-level definition is sufficient.

### DP-08 — Modality Unification Boundary

- **Decision question:** At which boundary do Voice and Text become a canonical
  logical turn while retaining modality-specific anchors and media lifecycle?
- **Need / risk:** FR-37/39 and AP-06/07 require continuity without equating
  media connection, conversation, and task state; Top QA-02/03 are sensitive.
- **QA / mandatory constraints:** Top QA-01 covers convergence overhead, QA-02
  identity/continuity correctness, and QA-03 modality-adapter containment;
  conversation/task recovery and state-domain separation remain mandatory.
- **In scope:** canonical turn identity, adapter-to-turn contract, modality
  anchors, and lifecycle correlation.
- **Out of scope:** whether Voice/Text share downstream processing at all (fixed
  by FR-39), UI policy, and task-association algorithm (DP-09).
- **Parent constraint:** the canonical turn feeds the owner chosen by DP-00.
- **Open structure / change surface:** early canonicalization versus a thin
  shared turn manager with modality extensions changes Voice/Text adapters,
  Conversation State, Context Engine, and downstream request contracts.
- **Dependencies / condition:** uses DP-02; feeds DP-09/10; all A/B/C/D. Open;
  catalog-level definition is sufficient.

### DP-09 — Task Association Authority

- **Decision question:** Which component owns existing-task association and
  ambiguity escalation, and what authoritative state index does it consume?
- **Need / risk:** FR-27/37/43 require correct continuation, reuse, and reroute;
  wrong association directly harms Top QA-02 and can add Top QA-04 calls.
- **QA / mandatory constraints:** Top QA-02 covers follow-up/new-task
  correctness and QA-04 pre-commit calls; task identity, recovery, truthful
  status, cancellation targeting, and clarification obligations remain live.
- **In scope:** association owner, candidate-index boundary, explicit-anchor
  handling, ambiguity/clarification interface, and route reuse output.
- **Out of scope:** scoring thresholds, prompt text, canonical request
  decomposition (DP-10), and persistence mechanics (DP-13).
- **Parent constraint:** clear continuation reuses the established route; B must
  preserve ARGO execution authority and D must preserve explicit route commits.
- **Open structure / change surface:** model-only association versus indexed
  filtering plus semantic decision changes Conversation/Task State, Intent
  Refiner, clarification, and route contracts.
- **Dependencies / condition:** consumes DP-08 and DP-13; feeds DP-11; all
  families, highest lifecycle discrimination under D. Open; catalog-level.

### DP-10 — Intent Refinement Responsibility Decomposition

- **Decision question:** How are normalization, interaction grounding,
  clarification, schema/state validation, and canonical-request commitment
  divided across components?
- **Need / risk:** FR-06~10 require canonical, grounded, valid requests; extra
  semantic stages trade Top QA-01/04 against Top QA-02/03.
- **QA / mandatory constraints:** all Top QAs are causally sensitive to stage
  latency, validity, coupling, and calls; context minimization, consent,
  schema/state validation, and clarification remain mandatory/supporting.
- **In scope:** responsibility boundaries and contracts between refinement
  stages, validation authority, and clarification loop ownership.
- **Out of scope:** prompt/model choice, numeric confidence thresholds, task
  association (DP-09), and route selection (DP-11).
- **Parent constraint:** A/C/D may place refinement before route commitment; B
  must test an ARGO-owned form rather than reconstruct a duplicate Thin-VIA
  pipeline.
- **Open structure / change surface:** a combined semantic owner versus staged
  deterministic/GenAI components changes Context Engine, validator, state,
  clarification, and router inputs.
- **Dependencies / condition:** consumes DP-01/02/08; feeds DP-11; conditional
  shape per family. Open; catalog-level.

### DP-11 — Request-time Routing Responsibility

- **Decision question:** Within the routing owner fixed by DP-00, how are
  capability eligibility, semantic selection, and final route commitment
  divided across components?
- **Need / risk:** FR-11/43 require eligible, available, policy-compliant
  routing; the split affects Top QA-01/02/03/04.
- **QA / mandatory constraints:** all Top QAs observe route latency,
  conformance, change propagation, and route-decision calls; trust, permission,
  availability, context-egress, and route-commit integrity remain constraints.
- **In scope:** filtering and semantic-selection dependency direction,
  selector/router interface, route-commit authority, and reroute boundary.
- **Out of scope:** default Agent, ranking weights, capability registration
  (DP-12), and primary execution ownership (DP-00).
- **Parent constraint:** A/C use VIA Agent routing, B uses ARGO self-versus-
  specialist authority, and D's Execution Path Selector commits topology and
  executor without a second top-level router.
- **Open structure / change surface:** combined semantic routing versus
  deterministic eligibility plus reranker changes Registry reads, Intent
  output, policy checks, route records, and Agent dispatch.
- **Dependencies / condition:** after DP-10/12; all families with different
  owner. Open; catalog-level.

### DP-12 — Capability Catalog and Registration Authority

- **Decision question:** How are declaration, registration, dynamic health,
  compatibility, and version authorities divided and projected into a
  canonical capability catalog?
- **Need / risk:** FR-11/35/43 require capability, trust, permission, execution
  characteristic, resource, and availability facts; stale or split authority
  harms Top QA-02/03 and mandatory trust constraints.
- **QA / mandatory constraints:** Top QA-02 depends on eligible/current facts
  and Top QA-03 on stable versioned contracts; trusted registration,
  permission/context constraints, resource declarations, freshness, and audit
  remain mandatory/supporting.
- **In scope:** canonical metadata owner, producer/consumer interfaces,
  registration and withdrawal, health freshness, version compatibility, and
  resource-declaration ownership.
- **Out of scope:** request-time ranking (DP-11), execute lifecycle (DP-14),
  runtime incident/failure handling (DP-15), static-versus-dynamic refresh
  interval, and individual capability values.
- **Parent constraint:** catalog facts may be consumed by VIA or ARGO according
  to DP-00 without moving primary execution authority.
- **Open structure / change surface:** central Registry, federated executor
  registration with canonical projection, or packaged manifests plus health
  authority changes registry services, executor adapters, routing inputs,
  deployment, and compatibility checks.
- **Dependencies / condition:** precedes DP-04/05/11; all families. Open;
  catalog-level definition is sufficient until contract design.

### DP-13 — Execution State and Recovery Authority

- **Decision question:** How are VIA's user-facing task projection and
  executor-authoritative domain execution state reconciled and recovered?
- **Need / scope / dependencies:** authoritative detail is in
  `DP-13-execution-state-and-recovery-authority.md`.
- **Status / authority:** Open; dedicated DP required because this missing axis
  spans state ownership, persistence, recovery, deduplication, and B/D transfer.

### DP-14 — Execution Lifecycle Control Boundary

- **Decision question:** Which component owns start/follow-up/pause/resume/
  cancel commands and resolves command-versus-terminal-event races?
- **Need / scope / dependencies:** authoritative detail is in
  `DP-14-execution-lifecycle-control-boundary.md`.
- **Status / authority:** Open; dedicated DP required because command authority
  is not equivalent to capability registration, hosting, or recovery storage.

### DP-15 — Failure Containment and Resilience Boundary

- **Decision question:** Which component detects and contains execution or
  dependency failure, coordinates retry/degrade, authorizes recovery routing,
  and chooses controlled failure?
- **Need / scope / dependencies:** authoritative detail is in
  `DP-15-failure-containment-boundary.md`.
- **Status / authority:** Open; dedicated DP required because cross-system
  blast radius is broader than local hosting and is not equivalent to cancel.

### DP-16 — Local Execution Hosting and Isolation Boundary

- **Decision question:** When VIA logically owns a local executor, is it hosted
  in the VIA process or behind a separately supervised local runtime boundary?
- **Need / risk:** local execution can affect FTOL while process isolation,
  credential/trust boundaries, restart independence, and blast radius affect
  Top QA-01/03 and mandatory failure/recovery constraints.
- **In scope / out of scope / dependencies:** authoritative detail is in
  `DP-16-local-execution-hosting-boundary.md`. The capability allow-list and
  whether VIA owns local execution at all are out of scope because DP-00 already
  owns that question.
- **Status / authority:** Open and conditional for C/D; no alternative selected.

## Non-standalone policy/configuration register

| Concern | Structural owner | Why it is not currently a standalone DP |
| --- | --- | --- |
| Model/provider selection, timeout, retry count/backoff/profile values, token budget | FR-44 Model Invocation Profile and AP-02 adapter boundary | Values/selection rules do not yet demonstrate a distinct component or authority boundary. DP-15 separately asks who coordinates retry/degrade after failure. |
| Fast-capability allow-list contents | DP-00 parent plus governance inside the selected family | Which capabilities qualify is policy; whether VIA owns local execution is DP-00 and where it is hosted is DP-16. |
| Default/preferred Agent | DP-11 route policy | A ranking preference does not relocate routing or execution responsibility. |
| Queue length, priority number, lease duration | DP-05 arbitration policy | Values configure the selected arbitration structure. |
| Confidence/clarification thresholds | DP-09/10 policy | Values do not define association/refinement ownership. |

## Authoring rule

Create a dedicated DP document when the item is the next investigation, needs
alternatives/evidence, or has acquired a decision. Keep lower-priority open
questions in this catalog to avoid one-file-per-row document growth.
