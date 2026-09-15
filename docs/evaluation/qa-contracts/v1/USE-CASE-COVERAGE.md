# VIA UC-01–UC-16 Coverage in the QA-v1 Corpus

## Interpretation policy

UC-01–UC-16 are product-scenario coverage inputs, not architecture answers.
Their user goals, observable outcomes, consent and security constraints, task
relationships, and delivery obligations are reusable. Historical component
names and route placement are not QA-v1 oracle requirements.

The executable mapping is
`benchmark/contracts/qa-v1/use-case-coverage-v1.json`. Counts below are
population assignments: one semantic case may contribute to multiple use cases
and QA populations. They are not production-frequency estimates.

| UC | Semantic families | Frozen assignments | QA populations | Coverage strength | Known limitation |
|---|---:|---:|---|---|---|
| UC-01 | 13 | 165 | QA-01, QA-02, QA-05, QA-07 | bounded Voice/Text outcomes and contention | no production speech distribution |
| UC-02 | 11 | 108 | QA-02, QA-05 | multi-pointer, group, movement, focus/window, revision, update, ambiguity, correction | synthetic interaction timelines |
| UC-03 | 33 | 563 | QA-01, QA-02, QA-05, QA-09 | authorized file/mail/calendar/browser/memory use plus wrong-scope exposure | frozen personal fixtures only |
| UC-04 | 32 | 326 | QA-01, QA-02, QA-04, QA-05 | heterogeneous capability/session/protocol contracts | anonymous Agent profiles |
| UC-05 | 38 | 430 | QA-01, QA-02, QA-04, QA-05, QA-06 | navigation, search, file, communication, media, transaction, transformation, creative UI | downstream behavior is emulated |
| UC-06 | 23 | 435 | QA-01, QA-02, QA-03, QA-09, QA-11 | purpose/scope/principal/approval and hard-gate probes | no live account authorization |
| UC-07 | 19 | 294 | QA-02, QA-05, QA-07, QA-08, QA-11 | long-task lifecycle, waiting, progress, recovery, resource overlap | deterministic Agent state |
| UC-08 | 20 | 338 | QA-02, QA-03, QA-05, QA-08, QA-12 | semantic correction/cancel separated from physical audible stop | emulated audio timeline |
| UC-09 | 19 | 307 | QA-02, QA-03, QA-05, QA-08, QA-10 | reconnect, restart, replay, projection rebuild, duplicate/late events | no production fault frequency |
| UC-10 | 100 | 1,404 | QA-01, QA-02, QA-03, QA-05–08, QA-10, QA-11 | concise Voice/detailed Text plus event-specific card/notification delivery | channel availability is fixture-defined |
| UC-11 | 5 | 48 | QA-02, QA-05 | independent, sequential, data-dependent, conditional decomposition with task/result bindings | four canonical dependency forms |
| UC-12 | 15 | 161 | QA-01, QA-02, QA-05 | omitted source/target/parameter/referent and sufficient-vs-insufficient context | synthetic ambiguity fixtures |
| UC-13 | 35 | 567 | QA-02, QA-03, QA-05, QA-09, QA-10 | same/new-task binding, follow-up, cancel, result and approval interleavings | no production conversation mix |
| UC-14 | 11 | 175 | QA-01, QA-02, QA-03, QA-09, QA-10 | store, retrieve/use, modify, delete, expiry, conflict, provenance, consent, principal/task isolation | memory implementation is deliberately unspecified |
| UC-15 | 15 | 202 | QA-02, QA-03, QA-05, QA-06, QA-10 | Voice→Text correction, Text→Voice follow-up, Voice result→Text follow-up, interruption→Text continuation | modality transitions are replayed |
| UC-16 | 13 | 164 | QA-02, QA-03, QA-05, QA-07, QA-09, QA-10 | explicit T1/T2 mouse-keyboard exclusion, unblocked read-only T3, cancel/binding invariants | resource scheduling is simulated |

All use cases satisfy the minimum one-family gate. Strengthened use cases
UC-02, UC-05, UC-11, UC-12, UC-14, UC-15, and UC-16 have multi-family,
multi-instance coverage rather than token examples.

## Strengthened coverage

### UC-02 — temporal pointing

Ten QA-02 families cover source→destination pointing, pointed groups, pointer
movement, selection/focus change, active-window change, transcript revision,
source-content revision, ambiguity, explicit correction, and multiple ordered
referents. Oracles retain temporal evidence and correct referent obligations;
they do not require a Context Engine, Intent Refiner, or Temporal Referent
Binder component.

### UC-05 — action diversity

The goal catalog identifies multiple action domains: navigation/open,
search/candidate selection, local file/content work, communication/send,
media/system control, constrained external transactions, document
transformation, and creative UI automation. Execution may use any eligible
contract-compliant downstream capability. No named Agent or Router is required.

### UC-11 — compound utterance

Independent, sequential, data-dependent, and conditional families declare
atomic tasks, dependency relations, and task/result bindings. Merely observing
that all actions eventually occurred is insufficient.

### UC-12 — contextual resolution

Frozen variants distinguish omitted application/source, target, parameter, and
referent. Sufficient context must be resolved without unnecessary
clarification; insufficient or genuinely ambiguous context requires
clarification.

### UC-14 — personalization and memory

Seven QA-02 memory families cover store, retrieve-and-use, modify, delete,
expiry, conflict resolution, provenance, and consent. Variants add wrong-user
and wrong-task probes. QA-03 preserves memory version continuity, QA-09 covers
scope/isolation, and QA-10 covers causal reconstruction. No database, memory
service, or component placement is assumed.

### UC-15 — mixed modality

Four goal families and continuity episodes cover Voice request→Text correction,
Text request→Voice follow-up, Voice result→Text follow-up, and Voice
interruption→Text continuation while retaining one logical task/conversation.

### UC-16 — concurrent resource conflict

QA-02 and QA-03 explicitly define T1 and T2 as exclusive mouse/keyboard users
and T3 as an informational read-only task requiring neither. T1/T2 must not
corrupt each other; T3 must not be needlessly blocked; cancel, progress, result,
and approval identities remain exact. The oracle explicitly rejects the
interpretation that VIA schedules an Agent's internal tool calls.

## Historical wording reconciliations

- UC-01's historical “local fast path” wording is not an oracle. R1, R3,
  R1+@, or another compliant topology may satisfy the same result, delivery,
  deadline, correctness, and security obligations.
- UC-02's Context Engine / Intent Refiner decomposition is removed. The corpus
  fixes temporal evidence and referent outcomes only. Its historical reference
  to legacy QA-09 referent accuracy does not override current QA-09
  least-privilege semantics.
- UC-05's historical Agent Router/component language is removed. External task
  reasoning and tool execution remain downstream-owned where required, but the
  corpus does not select a named Agent or routing component.
- UC-08 is split: audible stopping produces QA-12 observations; semantic
  correction/cancel produces QA-02 and QA-03 observations.
- UC-10 is refined by the active output contract: Voice carries concise key
  facts, Text carries detail, and progress channels are frozen per scenario.
  Not every progress event is spoken.
