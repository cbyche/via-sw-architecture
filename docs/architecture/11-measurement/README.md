# Measurement Guide

> **Current state:** QA-01~QA-03 semantic definitions exist; machine contract, target, score bands, harness, and current results do not.
>
> **Current phase: Measurement Contract Definition.** 현재는 QA-01~QA-12 각각에 대해 무엇을 어떤 경계와 조건으로 측정할지 확정하는 단계다. 후보 구현이나 A/B 실행 단계가 아니다.

이 디렉터리는 Architecture 후보를 비교하기 전에 고정해야 할 시험 입력, oracle, timing endpoint, 반복·집계, evidence level을 관리한다. 목적은 결과를 보고 유리한 계약을 선택하는 일을 막고, 각 DP의 A/B 차이를 같은 조건에서 재현하는 것이다.

## Authoritative inputs

| Document | Role |
| --- | --- |
| [Event & Boundary Contract](./event-boundary-contract.md) | 실제 사용자/source 사건, software 인식 event와 component 포함 규칙 |
| [Voice Responsiveness](../08-quality-attributes/voice-responsiveness.md) | QA-01~QA-03 semantic boundary and raw trace requirements |
| [Test Case Catalog](./test-case-catalog.md) | approved Use Case별 stimulus, state, event, oracle, failure rule |
| [QA Measurement & Scoring Contract](./scoring-contract.md) | QA-04~QA-12와 재동결 전 상태 |
| [Evaluation Method](../12-decisions/evaluation-method.md) | one-DP-at-a-time A/B comparison and differentiation criteria |

외부 model 근거는 [Quality Attribute evidence](../08-quality-attributes/evidence/)에 둔다. 그 자료는 입력 profile이나 estimate를 정당화할 수 있지만 실행하지 않은 모델을 measured evidence로 만들지는 않는다.

## Required sequence

측정 작업은 아래 순서를 지킨다.

1. 해당 QA와 DP A/B의 structural causality 및 applicability를 기록한다.
2. representative Use Case와 Voice/input fixture membership을 확정한다.
3. component path, event names, monotonic clock, included/excluded interval을 정의한다.
4. model call별 serialized prompt, tokenizer, input/output token ledger, rate profile을 동결한다.
5. mock/reference/actual dependency와 audio playback observation contract를 구분한다.
6. warm-up, scored repetition, timeout, failure handling, percentile와 macro aggregation을 동결한다.
7. target, 0~5 score boundary와 evidence label을 결과 전에 승인한다.
8. machine-readable fixture와 validation test를 구현한다.
9. source revision과 fixture digest를 Measurement Freeze로 고정한다.
10. A/B paired run을 수행하고 raw trace에서 summary를 재생성한다.

## Lifecycle names

1. **Measurement Contract Definition** — QA별 의미, fixture, endpoint, 반복·집계, target과 evidence 범위를 확정한다.
2. **Candidate Implementation** — 확정된 계약을 만족하는 DP별 A/B 후보와 harness를 구현한다.
3. **A/B Measurement & Evaluation** — 다른 DP 조건을 고정한 paired run으로 raw evidence를 생성·평가한다.
4. **Architecture Decision** — 측정 결과와 구조적 원인을 ADR에 반영한다.

`Gate 1`과 `Gate 2`는 현재 lifecycle 용어로 사용하지 않는다. archive 경로의 과거 campaign 식별자만 그대로 보존한다.

현재 QA-01~QA-03은 위 `Required sequence`의 1~3에 해당하는 semantic/event-boundary draft까지만 진행됐다. 모든 QA의 event 의미와 실제/software 경계는 [Event & Boundary Contract](./event-boundary-contract.md)의 공통 형식으로 작성한다. archived predecessor code를 그대로 실행하는 것은 8~10을 충족하지 않는다.

## Evidence classes

| Label | What ran | What it may claim |
| --- | --- | --- |
| `ESTIMATED_MODEL_ONLY` | frozen tokens ÷ documented rate | model-only planning estimate |
| `HYBRID_REFERENCE_ESTIMATE` | observed spans plus estimated/reference spans | reference scenario estimate |
| `MEASURED_MOCK_E2E` | executable mock path with observed endpoints | that mock harness under its frozen profile |
| `MEASURED_REFERENCE_HARNESS` | instrumented reference path | reference implementation behavior |
| `MEASURED_MODEL` | named model actually executed | that model in the recorded environment |
| `PRODUCT_E2E` | product path and required physical endpoints observed | recorded product/environment only |

Mock/reference evidence must not use `LIVE_S2S`, `MEASURED_MODEL`, `PRODUCT_E2E`, or `TARGET_WINDOWS_LATENCY`. An instrumented delivery sink is not a physical speaker; audible onset needs an appropriate playback/loopback observation.

## QA-01~QA-03 implementation prerequisites

Before implementation, freeze at least:

- exact case membership and valid semantic/audio oracle
- generated or recorded Voice input digest and audio format
- `user_input_end`, Agent ingress, Agent result/status source availability, speech, playback, and meaningful audible-onset events
- QA-01 included VIA segments and excluded Agent interval
- QA-02 direct-only route rule
- QA-03 valid, correlated, non-stale status rule
- IR/TASK/AGENT/EXEC applicability and fixed paired context
- prompt/token ledger and sequential/parallel model-call graph
- mock S2S meaning, scheduled delay profile, and playback profile
- warm-up/scored count, run order, timeout, failed-trial handling, percentile algorithm
- raw trace schema, summary schema, evidence label, source/fixture fingerprints

Qwen3-Omni 234 ms may be used only as a clearly labeled scheduled reference span for a matching full-S2S first-packet dependency. It is not a metric start point, generic TTS cost, network cost, or actual model execution.

## Existing assets and their limits

- The catalog contains 18 approved UCs and 94 explicit variants, plus 24 change scenarios.
- Synthetic fixtures and evaluator oracles exist in the [W12-G1 archive](../../../benchmark/archive/w12-g1/README.md).
- Actual human recordings, Windows playback capture, current S2S/VIA LLM runs, and current DP A/B Voice results do not exist.
- Archive assets may inform a new fixture review, but their old timing names, constants, target, score, and results are not inherited.

Current implementation belongs in [benchmark/architecture](../../../benchmark/architecture/README.md). Valid current evidence belongs in [results/architecture-evaluation/current](../../../results/architecture-evaluation/current/README.md).
