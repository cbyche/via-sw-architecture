# Core DP PoC Mock & Shared Evaluation Spine Contract

> **Status: ACTIVE DRAFT — all-Core-DP mock method and shared spine defined / campaign freeze pending**
>
> 이 문서는 VIA-DP-03·05·17·06·07·15의 Architecture PoC가 실제 Model·Agent 제품 개발을
> 기다리지 않고 같은 방법으로 QA-09·19·29·39를 산출하기 위한 공통 계약이다. 공식 Core-DP PoC는
> deterministic dependency mock과 실제 candidate Architecture 경로를 실행하고
> `MEASURED_MOCK_E2E` evidence를 만든다.

## 1. PoC가 답하는 질문

이번 PoC는 Model 자체를 선발하거나 제품 E2E 성능을 주장하는 시험이 아니다. 다음 조건부 질문에
답한다.

> S2S, semantic LLM, Context Source와 Downstream Agent가 동결된 event·timing·fault contract로
> 동작할 때, 같은 사용자 목표를 처리하는 DP의 A/B Architecture 중 어느 쪽이 더 빠르고 정확하며,
> 변경이 국소적이고, 장애를 더 잘 격리·복구하는가?

Mock 결과는 네 Core ASR의 **공식 PoC 수치와 별점**에 사용한다. 실제 Model이 실행되지 않았다는
사실은 evidence label과 manifest에 기록하지만, mock이라는 이유로 별도의 비공식 점수로 낮추거나
winner 판단에서 제외하지 않는다.

## 2. 공통 dependency mock boundary

모든 Core DP는 다음 논리 dependency를 한 세트만 사용한다.

| Dependency | PoC 구현 | 고정해야 할 행동 |
| --- | --- | --- |
| S2S Model 1개 | deterministic Voice/S2S event mock | transcript/response event, turn end, interruption, scheduled first-audio span |
| Semantic LLM 1개 | deterministic structured-output mock | frozen input digest에 대한 semantic field 또는 명시적 오류·timeout |
| Context Source | versioned Context event/query mock | screen/document/mail/calendar source state, revision, delay, denial·loss |
| Downstream Agent | deterministic Reference Agent | submit, progress, question, result, status, cancel과 fault schedule |

역할별 session·call·buffer는 달라도 dependency instance를 Component·Task별로 복제하지 않는다.
DP-03 A만 비교 축으로 timestamp-capable Streaming ASR mock 하나를 추가하고, DP-03 B는 별도 ASR
없이 S2S mock의 turn-final time-aligned text output을 사용한다.

Candidate 구현은 mock 내부의 정답을 읽을 수 없다. Mock은 dependency가 실제로 보낼 event만
candidate-visible input으로 내보내고, 최종 expected field·referent·Task state와 PASS 판정은 별도
evaluator-only oracle에 둔다.

## 3. 근거 있는 mock의 요건

각 mock profile은 결과 전에 다음을 동결한다.

- 실제 또는 개발 예정 dependency의 책임·입력·출력 contract와 그 근거
- event schema, ordering, source clock, revision/finality와 identity
- 정상·느림·누락·invalid·duplicate·late·disconnect/reconnect behavior
- scheduled delay의 근거와 profile ID
- deterministic seed, fixture digest와 implementation revision
- candidate-visible data와 evaluator-only oracle의 분리
- 한계가 아니라 공식 evidence 종류를 나타내는 `MEASURED_MOCK_E2E` label

상용 후보가 존재해야 mock을 만들 수 있는 것은 아니다. 공개 specification·원 논문·팀이 개발할
dependency contract·기존 interface 관행 중 하나 이상으로 event shape와 가능한 동작을 설명하면 된다.
불확실한 timing은 한 점을 사실처럼 고정하지 않고 `fast`, `nominal`, `slow` profile로 나눠 같은 A/B에
적용한다. Primary score에 사용할 profile은 결과 전에 하나로 고정하고 나머지는 sensitivity
diagnostic으로 보고한다.

## 4. Shared Evaluation Spine

Machine-readable registry 초안은
[`contracts/core-shared-spine.json`](./contracts/core-shared-spine.json)에 둔다.

Core DP마다 별도 세계를 만들지 않는다. 다음 공통 사용자 여정과 source data를 재사용한다.

| Spine ID | 공통 사용자 여정 | QA-09 class | 주요 공통 oracle |
| --- | --- | --- | --- |
| SP-01 | 화면의 두 위치를 차례로 가리키며 차이를 묻는 Voice direct request | Direct response | 두 지칭 순서·referent·goal·direct handling |
| SP-02 | 선택한 표·문단을 근거로 수정 결과물을 요청하는 Voice delegated request | Delegation | target/constraint·Context provenance·Agent request·result |
| SP-03 | 여러 Source와 순차·data-dependent 하위 요청이 있는 compound delegation | Delegation | node relation·owner·artifact dependency·partial/final outcome |
| SP-04 | 지칭·자료·요청 내용을 dispatch 전에 정정하는 request | Direct 또는 Delegation | 이전 값 철회·최종 goal/target/constraint·중복 dispatch 없음 |
| SP-05 | 실행 중 Task의 진행·질문·결과를 조회하거나 전달받음 | Agent progress/status | phase·staleness·question/result·notification disposition |
| SP-06 | 여러 Task 중 하나를 수정·취소하고 다른 Task는 계속함 | Task control | intended Task·control disposition·다른 Task 무영향 |
| SP-07 | Voice/Model/Context/Agent event 단절·재연결·역순·중복 | 해당 정상 class의 fault variant | containment·정확한 recovery·deadline·no duplicate·evidence |

각 spine fixture는 같은 Conversation/Request/Task 초기 상태, Voice/Text input, UI/Context timeline,
Agent script와 evaluator oracle을 가진다. Shared spine의 목적은 모든 DP에서 같은 사용자 목표와
별점 잣대를 유지하는 것이다.

## 5. DP event patch variant

각 DP는 Shared Spine을 복제해 별도 Test Case를 만들지 않고, 구조 차이를 활성화하는 최소 event/state
patch만 적용한다.

| Core DP | Shared Spine에 적용할 patch |
| --- | --- |
| VIA-DP-03 | A의 streaming partial/timestamp/revision 또는 B의 turn-final aligned-text event |
| VIA-DP-05 | plan-first closed read-set 또는 bounded demand-driven additional-read event |
| VIA-DP-17 | canonical ContextValue materialization 또는 consumer-specific Context view event |
| VIA-DP-06 | integrated final semantic decision 또는 staged version/correction event |
| VIA-DP-07 | VIA node-edge release 또는 owner-affinity bundle 내부 edge release event |
| VIA-DP-15 | continuous progress projection event 또는 hint/cursor+authoritative snapshot event |

Patch는 다음을 바꾸지 않는다.

- 사용자 goal과 completion condition
- candidate A/B에 제공되는 원천 정보의 의미와 시점
- final oracle과 user-visible endpoint
- 다른 DP의 fixed context
- scored repetition, timeout, failure treatment와 evaluator

해당 DP가 spine의 특정 경로에 물리적으로 참여하지 않으면 새 표본을 만들지 않는다. 원본
`source_execution_key`와 `NOT_APPLICABLE` 또는 `REGRESSION_ONLY` 근거를 기록한다.

## 6. 네 Core ASR의 공통 산출 방식

### QA-09

Mock이 schedule한 dependency span을 포함해 실제 candidate 경로를 실행하고 Voice input 또는 source
event부터 first meaningful audible/visible endpoint까지의 VIA-attributable interval을 측정한다.
Downstream Agent의 open-ended execution interval은 기존 계약대로 제외한다. Scheduled dependency,
candidate computation, IPC, validation, Context, response generation, playback queue와 device onset span을
trace에서 분리한다.

### QA-19

Mock이 낸 dependency event를 candidate가 처리한 최종 machine-readable field를 hidden oracle과
비교한다. Correct, ambiguous, corrected, missing, stale와 malformed dependency behavior를 모두 포함하며,
mock의 정답 event를 evaluator output으로 그대로 복사하지 않는다. 결과는 실제 Model accuracy가 아니라
동결된 dependency behavior 아래 VIA system field accuracy다.

### QA-29

Shared Spine과 기능 oracle을 유지하면서 DP별 applicable QA-21/22/23 change patch를 후보 설계에
적용한다. Model 내부 학습 code는 VIA Architecture Element로 세지 않지만 mock/실제 dependency의
output contract, adapter, state, deployment와 consumer 변경은 센다.

### QA-39

SP-07과 DP별 fault patch를 같은 injection schedule로 A/B에 적용한다. Containment, full correct
recovery, frozen deadline, no duplicate/wrong binding과 complete evidence를 모두 만족한 trial만 PASS다.

## 7. 공통 score와 DP별 해석

네 QA는 [Scoring Contract](./scoring-contract.md)의 동일한 시스템 목표와 0~5 band를 모든 DP에
사용한다. Machine-readable proposal은
[`contracts/core-asr-score-bands.json`](./contracts/core-asr-score-bands.json)에 둔다. 같은 DP의
A/B는 동일 spine membership과 patch를 사용한다.

별점은 다음 두 목적으로 사용한다.

1. 같은 DP 안에서 A/B를 직접 비교한다.
2. VIA system이 공통 Architecture 목표를 달성했는지 판정한다.

DP별 applicable field/change/fault가 다르므로 서로 다른 DP의 별점을 더해 총점을 만들거나 DP 순위를
정하지 않는다. 보고서는 공통 `Shared Spine` 결과와 구조 차이를 강화한 `DP Patch` breakdown을 함께
제시한다.

## 8. 공통 실행 기본값

Dependency mock profile은
[`contracts/core-dependency-mock-profile.schema.json`](./contracts/core-dependency-mock-profile.schema.json)에
맞춰 동결한다.

다른 값이 Measurement Freeze에서 승인되지 않으면 다음을 사용한다.

| 항목 | 기본값 |
| --- | --- |
| run order | fixture·profile별 seeded A/B interleave |
| warm-up | candidate/profile 조합당 5회 |
| QA-09/19 scored run | spine+patch 조합당 A/B 각 30회 |
| QA-39 scored run | fault item당 A/B 각 10회 |
| QA-29 | change item당 독립 baseline에서 1개 reviewed ledger |
| incorrect/missing terminal | QA-09에 5,000 ms timeout을 넣고 QA-19 applicable field는 INCORRECT |
| QA-39 recovery deadline | 기능 상실 시점부터 5,000 ms |

Deterministic correctness field를 30회 복제해 새로운 의미 표본처럼 주장하지 않는다. Repetition은
실행 안정성과 latency 분포를 얻기 위한 것이며, QA-19 보고서는 run-level 분자·분모와 unique
fixture-field coverage를 함께 공개한다.

## 9. 결과 package

각 DP campaign은 최소한 다음을 보존한다.

- shared spine version, DP patch registry와 digest
- A/B candidate source revision과 fixed-context manifest
- mock profile·schedule·seed·source 근거
- evaluator-only oracle digest와 candidate-input separation receipt
- raw trace와 QA-09/19/29/39 numerator·denominator
- score band version, qualification 결과와 independent replay receipt
- evidence label `MEASURED_MOCK_E2E`

실제 dependency를 나중에 연결하는 것은 별도 revalidation campaign이다. 그 결과는 동일 spine과
evaluator를 재사용할 수 있지만 이번 mock PoC result를 무효화하거나 Architecture Decision을 자동으로
뒤집지 않는다.
