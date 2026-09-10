# Observation Port Implementation Contract

## Status

**Prototype/Harness Specification Checkpoint — AUT observation vs benchmark provenance boundary**

This contract defines how an Architecture-under-Test (AUT) emits benchmark-relevant semantic observations without receiving or controlling evaluator-only provenance.

It complements:

- `benchmark/schemas/canonical-event-schema.md`
- `benchmark/schemas/run-event-schema.md`
- `docs/evaluation/prototype-benchmark-harness-spec.md`

The stored Canonical Event Envelope contains benchmark identifiers and authoritative timestamps, but those envelope fields are **not** the AUT-facing observation API.

---

# 1. Core rule

The architecture reports **what happened architecturally**. The benchmark records **which run/scenario/alternative the observation belongs to and when it was observed**.

```text
AUT semantic observation
        ↓
Benchmark-owned Observation Adapter
        + hidden run/episode/scenario context
        + monotonic timestamp
        + sequence/provenance
        ↓
Canonical Event Buffer
```

Reviewer-facing rule:

> **AUT는 semantic event를 보고하지만 benchmark provenance와 evaluator truth를 선택하지 않는다.**

---

# 2. AUT-visible `ObservationPort`

Conceptual interface:

```text
ObservationPort.emit(
  Observation {
    event_kind,
    architecture_payload,
    product_correlation,
  }
)
```

Possible product-native correlation includes values the architecture legitimately owns, such as:

```text
task_id
execution_id
dispatch_id
result_id
clarification_id
```

The AUT-facing API must not require internal benchmark identifiers to produce an event.

---

# 3. Fields the AUT must not control

The AUT must not supply/override:

```text
run_id
scenario_id
scenario_version
alternative_id
benchmark_version
canonical schema version
sequence_number
authoritative benchmark timestamp
emitter provenance classification
expected result / route
constraint conformance
QA eligibility
QA score
success predicate result
```

`episode_id` may exist as an outer runtime correlation token owned by the harness wrapper, but it must not be exposed as benchmark answer metadata to architecture decision code. The Observation Adapter binds the active episode out-of-band where practical.

---

# 4. Benchmark-owned enrichment context

The Observation Adapter maintains hidden execution context such as:

```text
ObservationContext
  run_id
  episode_id
  scenario_id
  scenario_version
  alternative_id
  benchmark_version
  canonical_event_schema_version
  source_git_commit
  current monotonic clock
  next sequence_number
```

On `emit`, the adapter creates the persisted envelope defined by `canonical-event-schema.md`.

---

# 5. Authoritative producer rules remain intact

Some scoring timestamps are not AUT observations at all.

## Acoustic EOS

```text
interaction.acoustic_eos
```

Authoritative source:

```text
BENCHMARK / INTERACTION_FIXTURE
```

The AUT does not choose the timestamp.

## Useful Outcome

```text
useful_outcome.observed
```

Authoritative source when feasible:

```text
OUTCOME_PROBE
```

An AUT self-report can remain diagnostic but cannot replace configured probe evidence for QA-01.

## Model generation

Model generation events and ModelCall linkage are emitted/validated by the benchmark ModelPort/Replay Adapter rather than trusted as arbitrary AUT labels.

## Episode completed/failed

Termination outcome is determined by the benchmark evaluator under the scenario termination contract.

---

# 6. Architecture-owned observations

Examples of semantic events that the AUT may initiate through `ObservationPort`:

```text
interaction.processing_started
clarification.requested
clarification.resolved
task.created
task.reused
execution.route_candidate_observed
execution.route_committed
consent.requested
consent.resolved
result.bound
cancel.propagated
```

The benchmark may validate these observations against fixture evidence, subsequent events, constraints, and causation.

`execution.route_committed` is especially important: an AUT reports that its initial route responsibility has reached the canonical boundary, while the evaluator verifies the route is coherent with dispatch/accept/execution evidence and has not been committed prematurely.

---

# 7. Internal debug telemetry vs canonical observation

Alternative-specific debug events are allowed, for example:

```text
A.AgentRouter candidate list
B.ARGOPrimary internal state transition
D.ExecutionPathSelector ranking detail
```

They must not become required Canonical Event types or correctness oracles.

Canonical observation remains topology-neutral.

---

# 8. Timing implementation rule

For timed qualification, `ObservationPort.emit` should perform only lightweight operations:

```text
read monotonic timestamp where benchmark-owned timing is needed
construct compact typed event
append to in-memory buffer
```

Do not synchronously:

- JSON serialize;
- write/flush files;
- call Python;
- generate report text;
- perform network logging.

The event buffer is serialized after the timed episode window.

---

# 9. Provenance spoofing guard

Contract tests must fail if an AUT-facing observation type permits architecture code to set benchmark provenance or evaluator outcomes.

Examples of prohibited API fields:

```text
Observation.scenario_id
Observation.alternative_id
Observation.sequence_number
Observation.expected_route
Observation.constraint_passed
Observation.qa_score
```

The stored `CanonicalEvent` may contain those provenance fields because the benchmark-owned adapter adds them after the AUT call.

This distinction resolves the apparent tension between a rich persisted event envelope and a narrow AUT-visible API.

---

# 10. Causation/correlation

Architecture-owned product identifiers may be included where needed to reconstruct causal chains.

The adapter may additionally attach:

```text
causation_event_id
correlation_id
```

from benchmark-owned runtime context or a neutral correlation token returned by benchmark ports.

The AUT must not use evaluator-only correlation identities as decision hints.

---

# 11. Negative contract tests

At minimum:

1. alternative crate cannot construct a persisted Canonical Event Envelope directly for scoring;
2. AUT observation API has no `scenario_id`/expected-result/constraint fields;
3. AUT cannot select canonical sequence number;
4. authoritative Outcome Probe timestamp cannot be replaced by AUT self-report;
5. Acoustic EOS timestamp cannot be emitted as AUT-authoritative evidence;
6. route commit without coherent subsequent route evidence is rejected/flagged;
7. duplicate initial route commits are rejected unless an explicitly versioned recovery transition permits them.

---

# 12. Implementation handoff

The Rust prototype should expose a narrow `ObservationPort` to alternatives and keep the persisted canonical envelope constructor inside evaluation-support crates.

Dependency direction should make it difficult for alternative crates to import evaluator/provenance internals.

This contract preserves:

```text
Architecture-visible
  semantic observation

Benchmark-hidden
  run/scenario identity
  canonical provenance
  authoritative timing source
  evaluator truth
```

and is required before S1~S5 smoke qualification.

## QA-02 actual semantic observation extension

The AUT-visible payload may contain product semantics the architecture has actually committed:

```text
ReferentBound(referent_role, resolved_referent_id)
TaskAssociated(task_relation) + product task_id correlation
RouteCommitted(ExecutionRoute)
ClarificationRequested(reason) / ClarificationResolved(request_turn_id, response_turn_id)
ResultBound + product task_id/execution_id/result_id correlation
```

Product turn/task/execution/result/dispatch/clarification identifiers are allowed when naturally owned by product state. They are not benchmark operation keys.

The collector still owns `run_id`, `scenario_id`, `alternative_id`, canonical sequence number, authoritative monotonic timestamp, emitter provenance, and all oracle/constraint expected values.

External effects are emitted by `OUTCOME_PROBE`, `TOOL_FIXTURE`, or `AGENT_FIXTURE`, not through the AUT `ObservationPort`. Terminal failure classification is emitted by the canonical runtime lifecycle. AUT decisions, fixture observations, and evaluator truth therefore remain distinct evidence authorities.
