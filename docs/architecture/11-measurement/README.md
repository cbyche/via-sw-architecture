# Measurement Guide

> **Current state:** QA-09·19·29·39는 confirmed core ASR이고 상세 QA는 input·diagnostic·qualification으로 유지한다. 새 Core-ASR generation의 active harness, candidate와 result는 아직 없으며 여섯 Core DP 결과는 모두 `NOT_RUN`이다.
>
> **Current focus:** Core DP는 VIA-DP-03·05·06·07·15·17의 6개로 확정했다. 현재 단계는 **Measurement Contract Definition**이며 DP-03부터 05→17→06→07→15 순으로 한 번에 하나씩 심층 정의한다. 기존 결과는 재사용 가능한 예비 증거일 뿐 새 Core ASR 결과나 최종 winner가 아니다.

여섯 Core DP의 공식 PoC는 실제 Model·Agent 제품 개발을 선행조건으로 두지 않는다. 공통 Shared
Evaluation Spine과 근거가 동결된 deterministic dependency mock을 사용해
`MEASURED_MOCK_E2E` QA-09/19/29/39 결과와 별점을 만든다. DP별 차이는 같은 spine에 적용하는 최소
event/state patch로 표현한다.

이 디렉터리는 Architecture 후보를 비교하기 전에 고정해야 할 시험 입력, oracle, timing endpoint, 반복·집계, evidence level을 관리한다. 목적은 결과를 보고 유리한 계약을 선택하는 일을 막고, 각 DP의 A/B 차이를 같은 조건에서 재현하는 것이다.

## Authoritative inputs

| Document | Role |
| --- | --- |
| [Current Architecture Focus](../12-decisions/dp-executive-summary.md) | Core QA 4개, Core DP 6개, 심층 검토 순서와 미완료 작업 |
| [Core ASR Contract](../08-quality-attributes/core-asr-contract.md) | QA-09/19/29/39 정의·집계·실패 처리와 VIA-DP-01~18 적용 원장 |
| [Event & Boundary Contract](./event-boundary-contract.md) | 실제 사용자/source 사건, software 인식 event와 component 포함 규칙 |
| [QA-01~15 Common Harness Contract](./qa01-15-harness-contract.md) | DP 공통 trace·oracle·audio endpoint·sentinel qualification gate |
| [Voice Responsiveness](../08-quality-attributes/voice-responsiveness.md) | QA-01~QA-04 semantic boundary and raw trace requirements |
| [Task Control Responsiveness](../08-quality-attributes/interaction-control-responsiveness.md) | QA-05 control input and truthful disposition boundary |
| [Correctness & Continuity](../08-quality-attributes/correctness-and-continuity.md) | QA-11~QA-15 predicate ownership and non-additive reporting |
| [Reliability & Resource](../08-quality-attributes/reliability-and-resource.md) | QA-31/32/41 boundary and accounting rules |
| [Observability](../08-quality-attributes/observability.md) | QA-61/62 trace completeness and evidence reproduction |
| [Test Case Catalog](./test-case-catalog.md) | approved Use Case별 stimulus, state, event, oracle, failure rule |
| [QA Measurement & Scoring Contract](./scoring-contract.md) | 활성 초안 QA의 metric·score band와 승인 전 상태 |
| [Core DP PoC Mock & Shared Spine Contract](./core-poc-mock-and-spine-contract.md) | 모든 Core DP의 deterministic mock, 공통 사용자 여정, DP event patch, 실행 기본값과 공식 `MEASURED_MOCK_E2E` 결과 계약 |
| [Core Shared Spine Registry](./contracts/core-shared-spine.json) | SP-01~07의 machine-readable source case·oracle family·DP patch eligibility |
| [Core ASR Score Bands](./contracts/core-asr-score-bands.json) | QA-09/19/29/39 공통 system target과 0~5 boundary proposal |
| [Dependency Mock Profile Schema](./contracts/core-dependency-mock-profile.schema.json) | 모든 DP mock의 timing·behavior·근거·seed machine contract |
| [VIA Core Evaluation Profile](./evaluation-profile.md) | target Mac, 공통 Model과 DP-03 candidate별 speech evidence dependency, Reference Agent, 네 core ASR의 최소 workload/fault |
| [VIA-DP-03 Capability & Freeze Draft](./via-dp-03-capability-and-freeze.md) | A의 Streaming ASR partial timestamp와 B의 S2S turn-final timestamp capability gate, candidate 계약과 pre-result 모집단 초안 |
| [VIA-DP-03 Candidate Contract](./via-dp-03-candidate-contract.md) | A/B 공통 speech event, candidate schema, clock·UI anchor·retention, 지칭 pattern과 revision lifecycle 초안 |
| [VIA-DP-03 Fixture, Oracle, Fault & Change Contract](./via-dp-03-fixture-and-oracle.md) | 동일 audio/UI 입력, hidden oracle, functional qualification pack, Core-ASR endpoint·fault·change pack과 사용자 승인 항목 |
| [VIA-DP-03 Spine Patch Registry](./contracts/via-dp-03-spine-patch.json) | SP-01/02/04/07에 적용할 A/B event sequence와 fault patch 초안 |
| [VIA-DP-03 A Mock Profile](./contracts/via-dp-03-a-streaming-asr-mock-profile.json) / [B Mock Profile](./contracts/via-dp-03-b-s2s-mock-profile.json) | A의 Streaming ASR와 B의 time-aligned S2S timing·behavior·근거·seed 초안 |
| [VIA-DP-03 Candidate-visible Fixtures](./contracts/via-dp-03-fixture-inputs.json) / [Evaluator-only Oracles](./contracts/via-dp-03-hidden-oracles.json) | DP03-L-01~07의 동일 Voice/UI input과 물리적으로 분리한 QA-19 정답·금지 결과 초안 |
| [VIA-DP-03 Change Ledger](./contracts/via-dp-03-change-ledger.json) / [Fault Registry](./contracts/via-dp-03-fault-registry.json) | QA-29 A/B Architecture Element baseline·독립 change patch와 QA-39 fault·necessary dependency closure 초안 |
| [`check_via_dp03_contract.py`](../../../scripts/architecture/check_via_dp03_contract.py) | fixture/oracle 분리, detector offset, A/B timing 대칭, QA-29 element reference와 QA-39 closure를 검사하는 pre-freeze static validator |
| [Major DP Evaluation Plan](./major-dp-evaluation-plan.md) | Core DP 6개 실행 순서, 완결 조건과 전체 inventory의 보존 자산 원장 |
| [Evaluation Method](../12-decisions/evaluation-method.md) | one-DP-at-a-time A/B comparison and differentiation criteria |

외부 model 근거는 [Quality Attribute evidence](../08-quality-attributes/evidence/)에 둔다. 그 자료는 입력 profile이나 estimate를 정당화할 수 있지만 실행하지 않은 모델을 measured evidence로 만들지는 않는다.

## Required sequence

측정 작업은 아래 순서를 지킨다.

1. 해당 QA와 DP A/B의 structural causality 및 applicability를 기록한다.
2. representative Use Case와 Voice/input fixture membership을 확정한다.
3. component path, event names, monotonic clock, included/excluded interval을 정의한다.
4. model call별 serialized prompt, tokenizer, input/output token ledger, rate profile을 동결한다.
5. mock/reference/actual dependency와 audio playback observation contract를 구분한다.
6. warm-up, scored repetition, timeout, failure handling과 core 단순평균 집계를 동결한다.
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

Core ASR 4개의 system target·score band proposal은 작성했지만 Measurement Freeze 전이며, 일부
detailed diagnostic QA의 standalone target/score도 여전히 `PENDING`이다. 모든 QA의 event 의미와 실제/software 경계는
[Event & Boundary Contract](./event-boundary-contract.md)의 공통 형식으로 작성한다. Archived
predecessor 결과가 다른 DP의 미구현 contract를 자동으로 확정하지 않으며, archived code를 그대로
실행하는 것은 8~10을 충족하지 않는다.

## Evidence classes

| Label | What ran | What it may claim |
| --- | --- | --- |
| `ESTIMATED_MODEL_ONLY` | frozen tokens ÷ documented rate | model-only planning estimate |
| `HYBRID_REFERENCE_ESTIMATE` | observed spans plus estimated/reference spans | reference scenario estimate |
| `MEASURED_MOCK_E2E` | frozen dependency mock과 실제 candidate path 및 endpoint를 실행 | 해당 mock profile에서의 공식 Architecture PoC QA-09/19/29/39 결과 |
| `MEASURED_REFERENCE_HARNESS` | instrumented reference path | reference implementation behavior |
| `MEASURED_MODEL` | named model actually executed | that model in the recorded environment |
| `PRODUCT_E2E` | product path and required physical endpoints observed | recorded product/environment only |

Mock/reference evidence must not use `LIVE_S2S`, `MEASURED_MODEL`, `PRODUCT_E2E`, or target-device absolute latency. An instrumented delivery sink is not a physical speaker; audible onset needs an appropriate playback/loopback observation.

## Core ASR implementation prerequisites

Before implementation, freeze at least:

- exact case membership and valid semantic/audio oracle
- generated or recorded Voice input digest and audio format
- `user_input_end`, Agent ingress, Agent result/status source availability, speech, playback, and meaningful audible-onset events
- QA-01 included VIA segments and excluded Agent interval
- QA-02 direct-only route rule
- QA-03 valid, correlated, non-stale status rule
- QA-04 actual acoustic barge-in onset and last audible interrupted-response sample
- QA-05 Voice/Text control input end와 correct Task control disposition presentation
- 현재 DP의 core-ASR applicability와 다른 DP를 고정한 paired context
- prompt/token ledger and sequential/parallel model-call graph
- mock S2S meaning, scheduled delay profile, and playback profile
- warm-up/scored count, run order, timeout, failed-trial handling, percentile algorithm
- raw trace schema, summary schema, evidence label, source/fixture fingerprints

위 상세 event에서 QA-09는 QA-01/02/03/05 applicable trial의 `VIA-attributable milliseconds`를 산술평균한다. QA-04는 Voice 회귀이며 평균에서 제외한다. QA-19는 QA-11/12의 중복 없는 applicable field를 micro-average하고 QA-13~15는 회귀 진단으로 남긴다. QA-29는 DP별 applicable QA-21~23 change를, QA-39는 DP별 applicable fault trial을 단순 pooled average로 집계한다.

Correctness, reliability와 resource QA는 추가로 다음을 동결한다.

- QA-11 integrated corpus와 QA-12~15 driver corpus/predicate ownership
- QA-11과 QA-12~15 non-additive reporting rule
- QA-31 fault/recovery strata와 QA-32 necessary dependency closure 및 user-visible unit registry
- QA-41 target PC, process/local-model accounting boundary와 memory sampler
- mandatory action/access zero-violation gate fixture

Modifiability와 observability QA는 추가로 다음을 동결한다.

- QA-23 E-01~E-05 experiment/logging change pack과 element ledger
- QA-61 complete execution trace의 required identity·event·version relation
- QA-62 immutable evidence package, freeze manifest와 clean evaluator
- missing/duplicate/out-of-order evidence 처리와 privacy-preserving field

Qwen3-Omni 234 ms may be used only as a clearly labeled scheduled reference span for a matching full-S2S first-packet dependency. It is not a metric start point, generic TTS cost, network cost, or actual model execution.

## Archived predecessor assets and their limits

- The catalog contains 18 approved UCs and 94 explicit variants, plus 24 change scenarios.
- 2026-09-27 detailed-QA runner와 unfinished DP-11 v5는 [benchmark archive](../../../benchmark/archive/pre-core-asr-reference-20260927/README.md), 당시 candidate는 [prototype archive](../../../prototypes/archive/pre-core-asr-reference-20260927/README.md)에 있다.
- 당시 qualification과 VIA-DP-02/05/06/09/11/12/13 result는 [evidence archive](../../../results/architecture-evaluation/archive/pre-core-asr-reference-20260927/README.md)에 있다.
- Actual human recordings와 현재 Core DP A/B Voice 결과는 없다. Local semantic LLM, Alibaba S2S synthetic-WAV smoke와 과거 BlackHole capture는 새 공식 QA run이 아니다.
- Archive assets may inform a new fixture review, but their old timing names, constants, target, score, paths, commands and results are not inherited.

Current implementation belongs in [benchmark/architecture](../../../benchmark/architecture/README.md). Valid current evidence belongs in [results/architecture-evaluation/current](../../../results/architecture-evaluation/current/README.md).

## Result reporting and storage convention

- 새 결과는 `results/architecture-evaluation/current/dpNN-<campaign>-vN-YYYYMMDD/`에 campaign별로 추가하며 기존 결과를 덮어쓰지 않는다.
- `raw/`, frozen contract·fixture·oracle과 digest, environment/source manifest, 재생성 가능한 `summary.json`, `report.md`를 함께 보존한다.
- QA-09는 applicable QA-01/02/03/05 scored trial 전체의 합계·sample count·산술평균을 대표값으로 보고한다. p95는 선택적 tail diagnostic일 뿐 core score가 아니다.
- QA-19는 QA-11/12의 중복 없는 applicable field에 대한 `correct / total`을 대표값으로 보고하고 case·field-family breakdown과 strict run pass를 진단으로 둔다. QA-13~15는 별도 회귀 진단이다.
- QA-29는 `sum changed elements / applicable changes`, QA-39는 `successful fault trials / all applicable fault trials`을 원 분자·분모와 함께 보고한다.
- B′ 같은 선택안+tactic 후보를 실행했다면 applicable QA의 complete table에 함께 표시한다. 별도 실행·ledger가 없으면 A/B 값을 복제하지 않고 N/A로 둔다.
