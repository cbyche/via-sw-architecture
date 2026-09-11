# QA Generations and Legacy v1.1 Traceability

## Status and purpose

This is the authoritative navigation for QA identifier generations. It moves
duplicated migration explanation out of the current DP catalog without changing
any requirement, metric, denominator, boundary, aggregation rule,
qualification, evaluator, threshold status, or historical record.

`docs/requirements/requirements-v1.1.md` remains the Approved Baseline.

## 1. Current scored Top QAs

| Current ID | Current name | Primary metric | Authoritative definition |
| --- | --- | --- | --- |
| QA-01 | Fast-task End-to-End Responsiveness | FTOL p95 | `docs/evaluation/quality-attributes/QA-01-fast-task-e2e-responsiveness.md` |
| QA-02 | VIA Interaction-Orchestration Correctness | AECR | `docs/evaluation/quality-attributes/QA-02-via-interaction-orchestration-correctness.md` |
| QA-03 | Flexibility | CCR | `docs/evaluation/quality-attributes/QA-03-change-flexibility.md` |
| QA-04 | Model Call Overhead | Average Model Calls to Commit Execution Route | `docs/evaluation/quality-attributes/QA-04-model-call-overhead.md` |

The linked documents alone define formulas, eligibility, boundaries,
denominators, inclusion/exclusion, aggregation, qualification, and evaluator
behavior. This migration record must not be used as a substitute definition.

## 2. Retained current obligations and supporting concerns

The word **Legacy** disambiguates identifiers; it does not mean obsolete.

| Approved v1.1 source | Current role | What remains live |
| --- | --- | --- |
| Legacy v1.1 QA-02 — concurrent-task capacity and PC co-existence (`requirements-v1.1.md:438-450`) | Detailed operational QA / supporting constraint | Capacity, independent task control, resource use, and foreground co-existence targets remain approved. |
| Legacy v1.1 QA-03 — conversation/task recovery (`requirements-v1.1.md:452-464`) | Mandatory recovery obligation | Approved recovery targets remain normative; only a future executable vNext gate encoding is TBD. |
| Legacy v1.1 QA-04 — dependency failure containment (`requirements-v1.1.md:466-478`) | Mandatory failure-containment obligation | Unrelated tasks must not fail from the same dependency cause and approved feedback targets remain normative; future gate encoding is TBD. |
| Legacy v1.1 QA-06 — external-context minimization (`requirements-v1.1.md:494-506`) | Privacy/security supporting QA and mandatory constraint evidence | Approved minimization/egress targets remain normative with CON-03 and FR-20~22/36. |
| Legacy v1.1 QA-08 — observability and audit overhead (`requirements-v1.1.md:522-534`) | Cross-cutting supporting QA | Approved audit availability and instrumentation-overhead targets remain normative with FR-34. |

The following Approved Baseline obligations must stay visible in current
architecture work regardless of Top-QA scoring:

- FR-02: immediate realtime audio-output interruption/barge-in; this is distinct
  from durable task/executor cancellation;
- FR-17, FR-23~30, and FR-40/41: truthful executor state, lifecycle control,
  recovery, side-effect deduplication, independent task cancellation/result,
  and resource-conflict handling;
- FR-18~22/34/36: identity, consent, context egress, and audit;
- CON-03/04 and AP-04/05: local-first context, trusted Agent execution
  boundary, and no VIA central reimplementation of Agent tool execution;
- AP-06/07: media connection, logical conversation, context, and durable task
  are distinct lifecycles/state domains.

Current architecture evaluation therefore keeps these non-compensable classes
separate from the four scored drivers:

```text
Security / privacy / trusted boundary
Required cancellation semantics
Required task-state integrity
Failure containment
Mandatory recovery behavior
```

Their requirement obligations are not TBD. Only the exact new executable gate
set or a not-yet-frozen vNext threshold is TBD; this review does not invent one.

## 3. Retained diagnostic and correctness-slice mapping

| Approved v1.1 source | Relationship to current Top QAs | Preservation rule |
| --- | --- | --- |
| Legacy v1.1 QA-01 — VIA software processing latency (`requirements-v1.1.md:424-436`) | Secondary FTOL decomposition under Top QA-01 | The approved VIA-overhead target is not replaced by FTOL p95. |
| Legacy v1.1 QA-05 — changeability and integration (`requirements-v1.1.md:480-492`) | Secondary/contract evidence under Top QA-03 | CPR/contract compatibility targets are not replaced by CCR. |
| Legacy v1.1 QA-07 — Model/Token cost efficiency (`requirements-v1.1.md:508-520`) | Secondary resource/cost evidence near Top QA-04 | Token/cost targets are not replaced by logical route-decision call count. |
| Legacy v1.1 QA-09 — screen/pointer referent binding (`requirements-v1.1.md:536-548`) | Correctness slice under Top QA-02 | Its approved suite/target remains separately traceable. |
| Legacy v1.1 QA-10 — request and task-association correctness (`requirements-v1.1.md:550-562`) | Correctness slice under Top QA-02 | Its approved suite/target remains separately traceable. |
| Legacy v1.1 QA-11 — Downstream Agent routing correctness (`requirements-v1.1.md:564-576`) | Correctness slice under Top QA-02 | Its approved suite/target remains separately traceable. |

Top-QA classification organizes current trade-space scoring; it does not erase
or renumber Approved Baseline targets.

## 4. Historical identifier rule

In current documents, use `QA-01`~`QA-04` for the current Top QAs. When both
generations appear, spell older identifiers as `Legacy v1.1 QA-xx`.

Do not mass-replace identifiers in requirements-v1.1, historical AA records,
experiment protocols, raw/derived results, or reports. Those identifiers retain
the meaning they had when the artifact was created. A new document cites this
migration record when a historical ID could otherwise be ambiguous.

The earlier duplicated Legacy-to-vNext explanation in current navigation is
superseded by this record. No approved QA or requirement is superseded merely
because its explanatory row moved here.

## Source locations

- Approved definitions and targets: `docs/requirements/requirements-v1.1.md`
  sections 5~8 and 11.
- Current Top-QA definitions: `docs/evaluation/quality-attributes/`.
- Score versus qualification separation:
  `docs/evaluation/evaluation-principles.md` and
  `docs/evaluation/architecture-experiment-methodology.md`.
- Rebaseline rationale/history:
  `docs/architecture/analysis/AA-001-qa-rebaseline-and-primary-execution-boundary.md`.
