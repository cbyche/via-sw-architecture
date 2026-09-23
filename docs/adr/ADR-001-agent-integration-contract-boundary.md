# ADR-001 — Normalize Agent Lifecycle Semantics at the Integration Edge

- Status: Accepted
- Date: 2026-09-22
- Related DP: AGENT-DP01

> Current measurement-contract impact: 이전 QA-02/QA-03 responsiveness 수치는 superseded되어 현재 근거로 사용하지 않는다. 결정은 previous-generation QA-07 change-locality band split과 change ledger에 근거해 유지한다. 현재 QA-21은 같은 concern을 새 번호로 계승하지만 새 catalog 결과는 아직 없다.

## Context

VIA must support Agents whose submit, follow-up, cancel, question, observation, and artifact lifecycles differ. The earlier candidate evaluation compared edge-normalized canonical semantics with Core-visible typed variations under the same native fixtures and Task-state authority. The later `2^4` campaign checked this choice in all eight contexts formed by IR, TASK, and EXEC alternatives.

## Decision

Adopt candidate A, **Edge-normalized Canonical Contract**. Native clients and edge semantic adapters translate provider lifecycle differences into versioned canonical operations and observations before Core Task handling.

Capabilities and unsupported operations remain explicit. The adapter must not report an unsupported native behavior as successful, synthesize source revisions, or write canonical Task state directly.

## Alternatives considered

Candidate B exposed provider-neutral typed lifecycle variations to Core capability handlers. It preserves functionality but propagates three reviewed Agent changes into an additional Core handler/interface element.

## Quality-attribute rationale

| QA | Selected alternative impact |
|---|---|
| previous-generation QA-07 (current concern: QA-21) | 1.444 changed elements/change, score 4; B was 1.778, score 3 |
| previous-generation QA-08 (current concern: QA-22) | 1.933, score 4 for both; no general maintainability advantage claimed |
| QA-02/QA-03 responsiveness (historical) | Superseded by the current Voice definitions; not current decision evidence |

Previous-generation QA-07 passed the then-frozen differentiation criteria with a one-band split and direct change-ledger traceability. The full-factorial projection preserved that split in all 8/8 matched contexts: A remained score 4 and B score 3. This is historical evidence, not a measured current QA-21 result.

## Evidence

- Prototype: `prototypes/candidates/runtime/src/agent.rs`
- Benchmark: `benchmark/archive/w12-g1/gate2/change_analysis.py`
- Results: `results/gate2/archive/w12-g1/rebaseline/gate2-freeze-b6ff0b07/w07-w08-scores.json`
- Full-factorial results: `results/gate2/archive/w12-g1/rebaseline/gate2-factorial-c8869c88/full-factorial.json`

## Consequences

### Positive

- Core consumes one lifecycle contract independent of provider protocol.
- Agent additions and lifecycle changes are localized at the integration boundary more often.
- Provider identity and capability remain observable without infecting Task state authority.

### Negative

- Edge adapters carry substantial semantic responsibility.
- A new lifecycle concept may require extending the canonical contract.
- Adapter implementations can drift while still compiling.

### Tactics and risks

Use a versioned canonical contract, shared lifecycle conformance fixtures, explicit capability declarations, and provenance-rich traces. Reject or safely hold unsupported behavior. The remaining risk is semantic drift across adapters; conformance tests and production trace review are mandatory controls.

## Requirement changes

None.
