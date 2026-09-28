# Core Architecture-Significant Requirements Contract

> **Status: active core-ASR definition / measurement freeze pending**
>
> **Confirmed core ASRs:** QA-09, QA-19, QA-29, QA-39
>
> 이 문서는 2026-09-28 사용자 결정에 따라 VIA-DP-01~18을 평가할 네 개의 핵심 Architecture Significant Requirement를 정의한다. ASR 분류와 대표 metric은 확정됐지만, DP별 fixture·반복 수·timeout·target·score band와 새 machine contract는 아직 동결·구현되지 않았다. 따라서 기존 reference 결과를 이 네 QA의 결과로 소급 변환하지 않는다.

## 1. 왜 네 개로 통합하는가

최종 Architecture 보고서는 다음 네 질문에 답해야 한다.

1. **빠른가?** — 사용자가 기다리는 VIA 책임시간
2. **정확한가?** — 요청 의미부터 Task·상태·결과까지의 field 정확도
3. **변화에 강한가?** — 변화 한 건을 수용할 때 바뀌는 Architecture Element 수
4. **장애에도 살아남는가?** — 장애를 필요한 범위에 가두고 올바르게 복구한 비율

이 네 질문은 기존 상세 QA의 원시 event·oracle·change ledger·fault evidence를 버리지 않는다. 기존 QA-01~05, QA-11~15, QA-21~23과 QA-31/32는 새 core QA의 **measurement input 또는 diagnostic breakdown**으로 유지한다. 다만 core 계산에 실제로 들어가는 범위는 아래 표처럼 제한하며, 나머지는 독립 ASR 점수로 중복 가중하지 않는다.

| Core ASR | 대표 metric | 상세 입력 |
| --- | --- | --- |
| QA-09 | applicable interaction trial의 평균 VIA 책임시간 | QA-01/02/03/05 event formula; QA-04 제외 |
| QA-19 | applicable oracle field 전체의 accuracy | QA-11/12 predicate를 중복 없이 평탄화; QA-13~15는 회귀 진단 |
| QA-29 | applicable change당 평균 changed Architecture Element 수 | QA-21/22/23 change pack의 applicable 합집합 |
| QA-39 | applicable fault trial의 containment·recovery 성공률 | QA-31 recovery evidence + QA-32 blast-radius evidence |

QA-41은 target-device memory 진단값, QA-51은 privacy/security qualification, QA-61/62는 evidence qualification으로 유지한다. 이들은 네 core ASR의 높은 값으로 상쇄할 수 없다.

## 2. 공통 A/B 원칙

- 평가 단위는 한 DP의 steelman A/B 직접 비교다. 다른 DP 조건은 고정한다.
- 같은 DP의 A/B에는 동일한 Test Case, oracle field, change item, fault item과 scored repetition을 사용한다.
- DP마다 applicable 모집단은 다를 수 있다. 따라서 서로 다른 DP의 QA-09/19/29/39 절대값을 순위화하거나 합산하지 않는다.
- applicability는 결과 전에 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED` 중 하나로 고정한다.
- `NOT_APPLICABLE`은 분모에서 제외하며 0으로 넣지 않는다. `UNRESOLVED`는 좋은 점수가 아니라 기능 유지 설계의 미완료다.
- 실패·timeout·missing evidence를 성공 표본에서 제거하지 않는다.
- 동일 fixture와 외부 dependency profile을 사용하고 후보 입력과 evaluator-only oracle을 분리한다.
- raw event, field verdict, changed element ID, fault verdict를 보존하여 대표값을 독립적으로 재계산할 수 있어야 한다.

### 2.1 VIA component 관점에서 무엇을 보는가

네 core ASR은 특정 Component의 내부 성능 점수가 아니라, 여러 Component가 연결된 **VIA 전체 Architecture의 결과**다. 아래 이름은 이해를 돕는 논리적 책임이며 한 Process에 같이 배치될 수도 있다.

| Component | 사용자에게 하는 일 | QA-09 | QA-19 | QA-29 | QA-39 |
| --- | --- | --- | --- | --- | --- |
| Voice Engine | 사용자의 음성을 받고 VIA 답·상태를 실제 소리로 재생한다. | input end와 first meaningful audible output을 관측하고, speech·playback·audio-buffer 시간을 포함한다. | Voice에서 확정된 요청 내용과 실제 제시된 response field의 근거를 낸다. | S2S provider·audio event 계약 변화가 applicable할 때 changed element를 센다. | Voice session 단절 뒤 재연결과 영향 경로 회복을 확인한다. |
| Context Engine | 화면·대화·기억·자료에서 이번 요청에 필요한 정보를 만든다. | 요청 처리에 실제 참여한 read/materialization 대기를 포함한다. | QA-11/12가 요구하는 referent·constraint·source field의 실제 근거를 제공한다. | Context source·schema·materialization 변화의 파급을 센다. | source loss나 재시작 뒤 같은 Context를 올바르게 다시 만들었는지 확인한다. |
| Intent Refiner | 사용자의 목표·대상·조건과 direct/delegate 의미를 구조화한다. | 의미 확정·clarification·route 판단의 VIA 시간을 포함한다. | QA-12 semantic field와 QA-11 handling field의 핵심 생산자다. | semantic model·prompt·schema·authority 변화의 파급을 센다. | semantic call 실패·timeout·repair 뒤 잘못된 요청을 실행하지 않는지 확인한다. |
| Task Manager | 장기 업무의 identity, state, revision, control과 결과 관계를 보존한다. | Task 생성·연결·제어 disposition에 걸린 VIA 시간을 포함한다. | QA-11의 Task relation과 required association field 근거를 낸다. QA-13~15는 별도 회귀다. | state schema·writer·persistence 변화의 파급을 센다. | crash 뒤 Task·result·allowed control을 정확히 복구했는지 확인한다. |
| Orchestrator | 직접 답할지, Task를 만들지, Agent에 맡길지와 전체 흐름을 조정한다. | direct/delegated/control 경로의 routing·coordination 시간을 포함한다. | handling·clarification·required outcome field가 oracle과 맞는지 확인한다. | orchestration·compound-request·logging 계약 변화의 파급을 센다. | 부분 실패를 필요한 범위에 가두고 안전한 재개·보류를 선택했는지 확인한다. |
| Agent Router | VIA와 외부 Agent 사이의 submit·status·question·result·cancel 계약을 번역한다. | Agent ingress 전과 source result/status 후의 VIA 구간을 포함한다. | Agent/capability 선택과 request-handling field의 실제 전달 근거를 낸다. | Agent 추가·교체·protocol 변화의 파급을 센다. | 연결 단절·worker crash 뒤 duplicate 없이 상태를 재확인하는지 본다. |
| Downstream Agent | 도메인 reasoning·계획·Tool 실행과 실제 업무 결과 생성을 책임진다. | 내부 queue·reasoning·Tool 실행시간은 제외하고 ingress와 result/status source 시각만 제공한다. | VIA가 보낸 요청과 돌려받은 source fact는 보되 Agent 자체 업무 품질을 VIA 점수로 채점하지 않는다. | Agent 자체 내부 파일 변경은 세지 않고 VIA가 Agent 변화에 대응해 바꾼 Architecture Element만 센다. | 외부 실행의 실제 상태를 확인할 수 있는 범위에서만 복구 판정하며 확인 불가를 성공으로 세지 않는다. |

S2S와 semantic LLM은 Component마다 따로 적재하는 모델이 아니다. VIA는 각각 한 모델을 공유하고 Component별 prompt·call·session·buffer만 달라질 수 있다. 따라서 QA 차이는 모델 개수를 임의로 늘리거나 줄여 만들지 않고, 책임·계약·상태·호출 graph·Process 경계의 차이로 설명한다.

## 3. QA-09 — Average VIA-attributable Interaction Responsiveness

### 3.1 품질 질문

사용자 요청 또는 외부 사건이 VIA 경계에서 처리 가능해진 뒤, VIA가 올바르고 의미 있는 사용자 결과를 전달하기까지 **VIA가 책임지는 처리 구간에 평균적으로 얼마나 많은 시간**을 소비하는가?

`end-to-end`라는 표현은 VIA 책임 구간의 양 끝을 실제 사건으로 관측한다는 뜻이다. Downstream Agent의 open-ended reasoning·planning·Tool 실행시간을 제외하므로 전체 사용자 wall-clock과 혼동하지 않는다. 전체 wall-clock은 secondary evidence로 보존한다.

### 3.2 포함 interaction class

| Class | 시작·종료와 trial 값 | 상세 계약 |
| --- | --- | --- |
| Delegation | `(agent ingress - user input end) + (audible result - result source available)` | 기존 QA-01 |
| Direct response | `first meaningful audible direct response - user input end` | 기존 QA-02 |
| Agent progress/status | `first meaningful audible status - valid status source available` | 기존 QA-03 |
| Task control | `correct disposition presented - control input end` | 기존 QA-05 |

QA-04 Voice interruption은 QA-09 모집단에서 제외한다. 실제 acoustic barge-in부터 audio stop까지의 기능은 Voice 회귀 qualification으로 유지하지만 core ASR 점수에는 넣지 않는다.

### 3.3 대표 metric

```text
effective_via_time_ms(trial)
= observed VIA-attributable interval sum
  if a correct meaningful terminal event is observed
= frozen_timeout_ms
  if the required terminal event is missing or timed out

QA-09
= arithmetic_mean(effective_via_time_ms over all applicable scored trials)
```

각 Test Case는 A/B에서 같은 scored repetition을 가져야 한다. 별도 사용자 승인으로 실제 제품 빈도 가중치를 고정하지 않는 한 canonical Test Case 사이의 반복 수를 같게 하여 쉬운 case의 반복 수로 평균을 바꾸지 않는다.

잘못된 내용·Task·상태를 빠르게 제시한 사건은 올바른 terminal event가 아니다. 실제 잘못된 출력의 시각은 raw trace에 보존하되 QA-09에는 frozen timeout을 적용하고 QA-19 field를 실패로 기록한다. timeout 값은 결과 전에 Test Case별로 동결한다.

### 3.4 보고 형식

대표값과 함께 다음을 반드시 공개한다.

- 총 합계 / scored trial 수 / 산술평균
- interaction class와 Test Case별 평균·표준편차·sample count
- A/B paired mean difference와 uncertainty interval
- timeout·incorrect endpoint 수
- delegation의 excluded Agent interval과 full user wall-clock
- 실제 audible/UI endpoint와 evidence label

p95는 대표 metric이나 score로 사용하지 않는다. 필요하면 tail diagnostic으로 raw evidence에서 계산할 수 있지만 QA-09 점수에 넣지 않는다. 반복 수는 “평균이므로 적게 한다”는 가정으로 정하지 않고 pilot variance와 요구 precision을 근거로 동결한다.

## 4. QA-19 — VIA Request Handling Field Accuracy

### 4.1 품질 질문

VIA가 사용자 요청의 의미를 이해하고 올바른 처리 경로와 요청 처리 결과를 결정하는 데 필요한 machine-readable field를 전체적으로 얼마나 정확하게 결정하는가?

QA-19는 사용자 요청대로 기존 QA-11 integrated request-handling outcome과 QA-12 semantic correctness의 **정확한 superset**이다. QA-11의 run-level 성공을 구성하던 판정 항목과 QA-12의 semantic field를 중복 없는 field registry로 풀어 전체 단순 accuracy를 계산한다.

QA-13~15의 Task binding·async state convergence·continuity predicate는 중요하지만 QA-19 분모에 자동 편입하지 않는다. 이들은 각 DP에서 같은 기능이 깨지지 않았는지 확인하는 회귀 진단이며, QA-11 또는 QA-12의 frozen field와 동일한 사실을 확인할 때만 그 기존 field evidence를 공유한다. 새 field를 QA-19에 추가하려면 결과를 보기 전에 QA-11/12 계약 자체를 개정하고 새 Measurement Freeze를 승인해야 한다.

### 4.2 중복 없는 field registry

각 field는 한 번만 등록한다.

| Field family | 예시 |
| --- | --- |
| Request meaning | goal, deliverable, constraint, referent, target set |
| Compound relation | independent, sequential, conditional, data dependency |
| Clarification | 질문 필요 여부, 질문 대상과 누락 정보 |
| Task relation | QA-11이 요구하는 no tracked/new/existing 처리와 intended Task identity |
| Handling | S2S direct/Core direct/Agent, Agent·capability 선택 |
| Required outcome | QA-11이 요구하는 response·dispatch·clarification·control disposition |
| Required association | QA-11 request-handling 성공에 필요한 Conversation·Request·Task association |
| Agent progress outcome | applicable status case의 phase, progress, blocked/answer-needed, terminal state, artifact version, staleness, notification disposition |

기존 QA-11과 QA-12가 같은 semantic fact를 검사했다면 새 registry에는 하나의 field ID만 둔다. field를 작게 쪼개거나 합쳐 결과를 바꾸지 못하도록 ID·source QA·oracle operator·applicability를 Measurement Freeze에서 고정한다.

Agent progress case의 `source_revision`은 status field의 freshness를 판정하는 provenance로 보존한다. 실제 Task/run/question binding과 비동기 event 수렴은 QA-13/14 회귀에서 독립적으로 검증하며, QA-11이 요구하는 outcome field와 동일한 사실을 판정할 때만 기존 QA-19 evidence를 재사용한다.

### 4.3 대표 metric

```text
field_verdict ∈ {CORRECT, INCORRECT, NOT_APPLICABLE}

QA-19
= 100 × count(CORRECT)
        / count(CORRECT or INCORRECT)
```

이는 applicable field 전체의 단순 micro-average다. case·field family별 macro-average나 가중 평균을 대표값으로 사용하지 않는다. 대신 결과표에 전체 분자·분모와 case/family breakdown을 함께 둔다.

missing output, invalid schema, exhausted repair, timeout과 required trace 부재는 해당 trial의 applicable field를 `INCORRECT`로 판정한다. evaluator가 판단할 근거를 후보 입력으로 제공하거나 사람이 자유문장을 주관적으로 채점하지 않는다.

### 4.4 qualification gate

다음 위반은 field 몇 개의 오류로 희석하지 않고 후보 qualification을 실패시킨다.

- wrong-target external Action
- retry/recovery 뒤 duplicate external Action
- revoked 또는 mismatched approval 사용
- unauthorized access 또는 disclosure
- 다른 Task/run/question에 control·answer·result를 적용

strict whole-run pass rate는 대표 metric이 아니라 diagnostic으로 보존한다. QA-19가 높아도 gate 위반이 있으면 후보는 선택할 수 없다.

## 5. QA-29 — Average Architecture Change Locality

### 5.1 품질 질문

해당 DP의 후보 Architecture가 적용 가능한 Agent·Model·Context·State·Experiment·Logging 변화를 수용할 때 변화 한 건당 평균 몇 개의 Architecture Element를 수정·추가·제거해야 하는가?

### 5.2 change superset

QA-29는 다음 세 상세 pack의 합집합에서 해당 DP에 `APPLICABLE`로 사전 판정한 change만 사용한다.

- A-01~A-09: Agent 생태계 변화
- M-01~M-09, C-01~C-06: Model·Context·State 변화
- E-01~E-05: Experiment·Logging 변화

가족별 macro-average를 만들지 않는다. DP마다 실제 구조 인과가 있는 change 집합을 결과 전에 고정하고 그 전체를 하나의 모집단으로 사용한다.

### 5.3 대표 metric

```text
N(change)
= |modified IDs ∪ added IDs ∪ removed IDs|

QA-29
= Σ N(change for applicable changes)
  / count(applicable changes)
```

Architecture Element는 Component·Interface·State·Runtime의 frozen ID이며 파일·함수·class·diagram box 수가 아니다. 설정값·rename·rebuild만 바뀌면 0개이고, 기능 축소나 미검증 상태는 작은 값이 아니다.

같은 DP의 A/B는 같은 applicable change ID를 사용한다. `REGRESSION_ONLY`와 `NOT_APPLICABLE`은 대표 분모에서 제외하되 기능 유지 결과는 남긴다. 한 change가 `UNRESOLVED`이면 성공 change만 평균 내 전체 QA-29를 만들지 않고 후보를 미완료로 표시한다.

### 5.4 보고 형식

- `sum changed elements / applicable change count = mean`
- change ID별 modified/added/removed element ID
- A/M/C/E family breakdown은 diagnostic으로만 표시
- C/I/S/D 유형별 breakdown
- 기능 유지 regression 결과와 미해결 change

DP마다 분모가 다를 수 있으므로 서로 다른 DP의 QA-29 절대값을 직접 순위화하지 않는다.

## 6. QA-39 — Fault Containment & Recovery Success Rate

### 6.1 품질 질문

주입된 장애가 필요 이상으로 확산되지 않고, 영향받은 모든 사용자 기능과 Task가 정해진 제한 시간 안에 올바른 상태로 복구된 fault trial의 비율은 얼마인가?

### 6.2 fault trial PASS

한 trial은 다음 조건을 모두 만족해야 `PASS`다.

1. 실제 unavailable/incorrect unit이 사전 승인한 necessary dependency closure 밖으로 퍼지지 않는다.
2. 영향받은 모든 Task의 identity·state·result reference·pending interaction·허용 control이 oracle과 일치한다.
3. frozen recovery deadline 안에 위 상태가 사용자에게 다시 사용 가능하다.
4. 중복 외부 실행, stale terminal state, result misbinding이 없다.
5. 확인할 수 없는 외부 상태를 정상 복구로 주장하지 않는다.
6. required recovery·containment evidence가 완전하다.

### 6.3 대표 metric

```text
fault_trial_success ∈ {0, 1}

QA-39
= 100 × Σ fault_trial_success
        / count(all applicable scored fault trials)
```

fault stratum별 macro-average를 사용하지 않는다. 별도 제품 빈도 가중치가 승인되지 않는 한 fault item별 scored repetition을 동일하게 하여 단순 평균의 가중을 통제한다. 같은 DP의 A/B에는 동일 fault item·주입 지점·반복·timeout을 사용한다.

기존 QA-31의 recovery time과 QA-32의 excess affected unit 수는 PASS 판정의 입력이자 diagnostic이다. 다음을 대표값과 함께 공개한다.

- 성공 수 / 전체 fault trial 수
- fault item별 성공 수와 실패 이유
- recovery time 분포
- maximum excess affected units
- duplicate execution·wrong binding·missing evidence 수

다른 DP는 다른 fault 모집단을 가질 수 있으므로 QA-39 절대값을 DP 사이에 직접 비교하지 않는다.

## 7. QA-41과 공통 qualification

QA-41은 core ASR이 아니다. 여기서 memory는 **VIA system을 실행하기 위해 target PC에 실제 commit된 전체 memory**다. VIA Core·Voice Runtime·Context·Task·Agent client 같은 software process의 heap/stack/queue/buffer, 후보가 추가한 helper process, local S2S/semantic model runtime과 model weights·KV cache를 모두 포함한다. Remote model 서버의 memory는 포함하지 않는다.

VIA는 후보마다 새 모델을 올리지 않고 S2S 한 개와 semantic LLM 한 개를 공유하므로, 큰 model weight baseline은 A/B에 공통이다. DP가 바꾸는 부분은 주로 process overhead, IPC buffer, session/KV cache 수명, duplicate serialization과 queue 크기다. 따라서 보고서는 `shared model baseline`, `candidate-specific VIA software`, `candidate-specific cache/buffer/helper process`를 분리하고 A/B peak delta도 함께 보여준다.

모든 candidate campaign에서 이 target-device peak committed memory를 diagnostic으로 기록하되 현재는 target·score·Architecture winner에 사용하지 않는다. 메모리 상한 요구가 없고 shared local model이 절대값을 지배하며, 현재 DP-11 reference path의 약 3.7 MiB 차이도 전체 후보에 일반화할 만큼 크거나 반복 검증된 차이가 아니기 때문이다. 향후 명시적 device budget 또는 구조적으로 반복되는 유의미한 A/B delta가 생기면 별도 사용자 승인으로 ASR 재분류를 검토한다.

다음은 네 core ASR과 별도로 반드시 통과하거나 근거를 보존한다.

| 항목 | 역할 |
| --- | --- |
| QA-04 detailed endpoint | Voice interruption 기능 회귀; QA-09에서 제외 |
| QA-41 | target-device memory diagnostic |
| QA-51 | protected-information exposure qualification |
| QA-61 | execution trace completeness qualification |
| QA-62 | evidence reproduction qualification |
| Action/access gate | wrong-target·duplicate·revoked approval·unauthorized use 0건 |

## 8. VIA-DP-01~18 적용 원장

모든 DP는 QA-09·19·29·39 네 값을 같은 A/B campaign package에 포함한다. 아래 `P`는 DP 구조 차이로 값이 달라질 Primary hypothesis, `R`은 같은 기능·계약을 지키는지 확인하는 Regression-only다. 최종 applicability와 case/change/fault ID는 각 Measurement Freeze에서 확정한다.

현재 보고서와 평가의 Core DP set은 VIA-DP-03·05·06·07·15·17의 6개로 확정했다. 아래 원장은 18개 전체 inventory의 QA 적용 가능성을 보존하는 표이므로 Core 밖 DP도 삭제하지 않는다. VIA-DP-11은 여섯 DP의 대체 후보가 아니며 실제 Process-fatal integration이 제품 범위에 새로 들어올 때 별도 scope decision으로 다시 검토한다.

| DP | QA-09 모집단 | QA-19 주요 field | QA-29 주요 change | QA-39 주요 fault | 초기 역할 |
| --- | --- | --- | --- | --- | --- |
| VIA-DP-01 | 같은 bounded goal의 direct/delegated VIA 구간 | handling·Task relation·response/result | M/C/E와 직접처리 계약 변화 | Agent 경로 장애·VIA 재시작 뒤 독립 direct 경로 | 09/19/29 P, 39 R |
| VIA-DP-02 | Conversation–Task 교차 commit이 참여하는 response/control/status | Task·Conversation binding, revision, final state | state schema·correlation·logging | 두 상태 확정 사이 crash와 reconciliation | 09/19/29/39 P |
| VIA-DP-03 | Voice 입력이 필요한 direct/delegated/control | input evidence, referent, correction, goal | M-01/M-07, C-03, E timing | Voice session 단절·재연결과 pending input evidence | 09/19/29 P, 39 R |
| VIA-DP-04 | S2S direct response publish | route authorization·response binding | S2S/session/approval/log contract | publish authority 회수·Core/Voice Runtime 재시작 | 09 P, 19/29/39 R |
| VIA-DP-05 | Plan-first 또는 demand-driven Context 획득이 필요한 direct/delegated/control | read-set generation, additional-read decision, referent, handling | C-01~06, M-09, Context tool·plan·E correlation | source loss·restart 뒤 plan 또는 tool-loop/read-set 복원 | 09/19/29/39 P |
| VIA-DP-06 | semantic 판단이 필요한 direct/delegated/control | 전체 semantic·handling·Task relation field | M/C/E semantic contract | semantic call/session 실패와 제한된 repair | 09/19/29 P, 39 R |
| VIA-DP-07 | node-level 또는 owner-affinity bundle의 compound delegation·status·control | dependency edge owner, node/bundle binding, artifact, partial/final state | Agent bundle/capability·routing·state·logging | node/bundle 실행 중 crash·부분 결과·cross-bundle 복원 | 09/19/29/39 P |
| VIA-DP-08 | durable write가 참여하는 정상 interaction | recovered Task·result·control field | persistent state·migration·recovery log | process 종료·checkpoint/journal/current-state 복원 | 39 P, 09/19/29 R 또는 조건부 P |
| VIA-DP-09 | Agent submit/status/control/result 경로 | lifecycle meaning·Task/run binding | A-01~09와 관련 E change | Agent 연결 단절·취소/완료 재확인 | 29 P, 09/19/39 R |
| VIA-DP-10 | model session이 참여하는 direct/delegated/control | Conversation·Request continuity와 output binding | M-01~09, session·logging | S2S/semantic session 단절·재연결 | 09/29/39 P, 19 R |
| VIA-DP-11 | Agent client process를 지나는 delegated/status/control | IPC 전후 binding·state·result | Agent/process/instrumentation change | VIA-owned Agent client/worker fatal crash | 09/29/39 P, 19 R |
| VIA-DP-12 | publish와 durable evidence가 참여하는 interaction | 실제 response·state와 기록 field | E-01~05와 evidence state | publish 직후 durable commit 전 crash | 09/29/39 P, 19 R |
| VIA-DP-13 | 고정 포화 workload의 direct/delegated/status/control | 올바른 admission·control disposition | scheduler/resource/log contract | 예약 lane·공유 lane 장애와 재기동 | 09 P, 19/29/39 R |
| VIA-DP-14 | Task command·status·control | revision, terminality, result, allowed control | state/supervisor/repository/log change | Task owner/Core crash와 fencing 복구 | 09/19/29/39 P |
| VIA-DP-15 | Agent progress notification·status query·result/control | phase, progress, blocked/answer-needed, terminal state, artifact version, staleness, notification disposition; source revision은 provenance | progress schema·cursor·projection·snapshot query·notification contract | event loss·역순·Agent 단절·VIA restart 뒤 projection/snapshot 복구 | 09/19/29/39 P |
| VIA-DP-16 | model history가 필요한 direct/delegated/control | semantic·Task relation·continuity field | M-09, C-04/C-06, cache/projector | working context 손실·cold rebuild | 09/19/29/39 P |
| VIA-DP-17 | canonical 또는 consumer-specific Context view가 필요한 direct/delegated/control | referent·Context value·provenance·handling field와 consumer 간 일관성 | C-01~05, canonical schema, consumer requirement, model consumer, logging | canonical materializer 또는 consumer view 실패와 Context 재구성 | 09/19/29/39 P |
| VIA-DP-18 | protected use가 참여하는 request/control | authorization scope·binding·handling | A-07/A-08, C/M policy, E audit | central authority/local gate 장애·revocation recovery | 09/19/29/39 P + safety gate |

`R`도 실행 package에서 누락하지 않는다. 다만 값이 같아야 할 경로에 인위적 지연·오류를 넣어 trade-off를 만들지 않는다. 한 DP의 source execution이 특정 core ASR에 물리적으로 참여하지 않으면 `source_execution_key`와 `REGRESSION_ONLY` 근거를 남기고 독립 표본처럼 복제하지 않는다.

## 9. 아직 구현·실행되지 않은 것

- 네 QA의 machine-readable schema와 common analyzer
- DP별 applicable Test Case·field·change·fault registry
- scored repetition과 run order
- QA-09 timeout과 precision requirement
- QA-19 field granularity ledger
- QA-39 recovery deadline
- 네 QA의 target과 0~5 score band
- 새 계약으로 실행한 A/B 결과

기존 QA-01~62 reference 결과는 상세 input의 예비 근거일 수 있지만 QA-09/19/29/39 결과가 아니다. 새 Measurement Freeze와 독립 실행 없이 네 core ASR의 숫자나 winner를 보고하지 않는다.
