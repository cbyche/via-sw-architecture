# 11-D. Working-12 Measurement & Scoring Contract

> 버전: **W12-G1-v1 — 후보 결과를 보기 전 작성한 Gate 1 검토 기준**.
> 12개 모두 탐색에 유지. **기준 작성 완료와 사용자 최종 승인·실험 완료는 다르다.** 기존 ASR-04/05 목표 2/3은 승인값을 계승하며 새 목표는 아래 제품 예산 제안이다.
> 기계 판독 기준: [`baseline.json`](../../benchmark/rebaseline/working12/baseline.json). 기존 7-ASR `11c`와 `scoring-baseline.json`은 이 Working-12 sweep에 사용하지 않는다.

## 1. 변경 요약

- 반응성을 대화(W-01), 업무 접수(W-02), feedback(W-03)으로 나누며 각 축 안의 canonical case별 p95를 동일가중 평균한다. 서로 다른 축의 평균을 새 대표값으로 다시 합치지 않는다.
- W-02는 **Agent의 유효 접수 AND VIA 재연결 기록의 지속성**을 모두 요구한다. 기존 `Agent 접수 또는 로컬 durable queue`라는 종료점은 폐기한다. 로컬 접수와 외부 인계는 같은 일이 아니다.
- W-04의 분모는 별도 저성능 baseline이 아니라 **같은 candidate의 같은 workload·configuration에서 active background Task만 1개인 경우**다.
- 정확한 복구 여부는 유지하며 W-09는 복구 가능한 조건의 시간으로 비교한다. W-10은 장애가 유지되는 동안 무관 기능이 살아 있는지 본다.
- W-11은 보호 정보단위와 reachable handle scope를 사전에 고정한다. byte 수·압축률을 개인정보 보호 수준으로 오인하지 않는다.
- partial model 계산으로 system p95 점수를 만들지 않는다. 후보가 없는 현재 모든 값은 `NOT_RUN/null`이다.

## 2. 12개 대표값과 목표

| ID | Quality Attribute 전체 명칭 | 대표 Metric | 검토용 목표 |
|---|---|---|---|
| W-01 | Conversational Reaction Responsiveness | `mean_of_case_p95_response_start_ms` | **≤2000 ms** |
| W-02 | Task Handoff Responsiveness | `mean_of_case_p95_agent_acceptance_ms` | **≤3000 ms** |
| W-03 | Task Feedback Responsiveness | `mean_of_case_p95_event_to_visible_ms` | **≤1000 ms** |
| W-04 | Concurrent Task Performance Isolation | `foreground_macro_p95_ratio_4_to_1` | **≤1.25 ratio** |
| W-05 | Task Completion Effectiveness | `macro_completion_obligation_satisfaction_pct` | **≥95 percent** |
| W-06 | Interaction & Task Continuity | `macro_continuity_obligation_preservation_pct` | **≥95 percent** |
| W-07 | Agent Ecosystem Interoperability & Substitutability | `mean_changed_elements_per_agent_change` | **≤2 count/change** |
| W-08 | Evolvability & Maintainability | `mean_changed_elements_per_non_agent_change` | **≤3 count/change** |
| W-09 | Recovery Timeliness & Recoverability | `mean_of_recoverable_stratum_p95_recovery_ms` | **≤5000 ms** |
| W-10 | Dependency Failure Containment & Graceful Degradation | `unaffected_capability_retention_pct` | **≥100 percent** |
| W-11 | Privacy Exposure Minimization | `remote_sensitive_context_exposure_pct` | **≤25 percent** |
| W-12 | Action & Access Safety | `violated_opportunities_pct` | **≤0 percent** |

## 3. 0~5 Score Boundary

아래는 **5점부터 순서대로 처음 만족하는 구간**을 적용하는 누적 경계 표다. 예: W-01의 4점은 `1000 < x ≤ 1500ms`이며 5점과 중복되지 않는다. 숫자는 표시 반올림 전 값으로 판정한다. W-12는 비율뿐 아니라 정수 V/24도 반드시 표시한다.

| W | 방향·단위 | 5 | 4 | 3 | 2 | 1 | 0 |
|---|---|---:|---:|---:|---:|---:|---|
| W-01 | lower / ms | ≤1000 | ≤1500 | ≤2000 | ≤3000 | ≤4000 | >4000 |
| W-02 | lower / ms | ≤1000 | ≤2000 | ≤3000 | ≤4500 | ≤6000 | >6000 |
| W-03 | lower / ms | ≤200 | ≤500 | ≤1000 | ≤2000 | ≤4000 | >4000 |
| W-04 | lower / ratio | ≤1.05 | ≤1.1 | ≤1.25 | ≤1.5 | ≤2 | >2 |
| W-05 | higher / percent | ≥99 | ≥97 | ≥95 | ≥90 | ≥80 | <80 |
| W-06 | higher / percent | ≥99 | ≥97 | ≥95 | ≥90 | ≥80 | <80 |
| W-07 | lower / count/change | ≤1 | ≤1.5 | ≤2 | ≤3 | ≤4 | >4 |
| W-08 | lower / count/change | ≤1 | ≤2 | ≤3 | ≤4 | ≤5 | >5 |
| W-09 | lower / ms | ≤1000 | ≤2500 | ≤5000 | ≤10000 | ≤20000 | >20000 |
| W-10 | higher / percent | ≥100 | ≥95 | ≥90 | ≥80 | ≥60 | <60 |
| W-11 | lower / percent | ≤0 | ≤10 | ≤25 | ≤50 | ≤75 | >75 |
| W-12 | lower / percent | ≤0 | ≤4.1667 | ≤8.3333 | ≤12.5 | ≤16.6667 | >16.6667 |

W-01~09·11은 3점 이상이면 제안 target 충족, W-10/12는 5점만 무결점 target을 충족한다. **Score는 품질 실측치가 아니라 고정 metric의 표시 변환**이다. 목표 미달과 미실행은 다르다. `NOT_RUN/BLOCKED/UNMEASURED`는 0점이 아니라 score 없음이다. 값을 나쁘게 만드는 실제 실행 실패는 사후 분모에서 빼지 않는다.

## 4. 공통 관측·실험 계약

### 4.1 입력과 모델/환경

한 PC·한 활성 사용자·한국어 Voice/Text, 모니터 1/2개, 같은 원천 자료/권한/Agent 계약을 사용한다(06 FA-01~16). 05의 18 UC·94 variation, 07의 24 변화, 10의 요소 정의는 유지한다. 원천 관측과 평가 oracle은 분리한다. 이전에 철회한 nonce/opaque counterfactual scoring fixture는 추가하지 않는다.

Semantic reference는 **Qwen3-8B non-thinking**이며 final comparative semantic execution은 frozen OpenRouter hosted profile(`qwen/qwen3-8b`, provider `alibaba`, fallback disabled)을 사용한다. 이는 빠른 재현을 위한 **MEASURED_MODEL_REFERENCE**이며 local Q4_K_M 또는 target Windows PC absolute model latency로 주장하지 않는다. A/B 모두 같은 model/provider/sampling profile을 사용한다. provider-side JSON-schema enforcement를 전제로 하지 않고 JSON object를 받아 VIA validator/repair가 동일 schema를 강제한다. S2S reference는 Qwen3-Omni-30B-A3B-Instruct다. structured output prompt와 호출 topology는 후보에 따라 달라질 수 있고 tuning 기회는 동일하게 준다. `deterministic 설정`이 실행 결과의 완전 결정성을 보장한다는 주장도 하지 않는다.

비교 환경의 원격 RTT는 FA-11의 50ms, sensitivity는 0/150ms다. **100Mbps 직렬화 대역폭은 추가 합성 시험 가정**이며 실제 네트워크 측정값이 아니다. 실제 payload 크기와 transmission/encoding span이 없는 추정은 RTT 소계까지만 표시한다. streaming 중 이미 전송·처리된 입력을 발화 종료 이후 비용으로 이중 계산하지 않는다. Local IPC 비용도 0이라고 가정하지 않는다.

주 latency 시험은 모델이 로드된 상태, 같은 cache 정책으로 시행한다. 매 trial 해당 query의 data/cache 상태를 같은 방식으로 reset/prime한다. 모델 load·cache miss·실제 cold start는 별도 strata/secondary로 보존하며 warming을 candidate에 유리하게 다르게 하지 않는다. 같은 GPU를 쓰는 병렬 호출은 실제 resource queue를 포함한다.

### 4.2 반복·집계

각 latency stratum은 10 warm-up + 100 scored trials. `p95 = 정렬한 표본의 ceil(0.95n)번째 값`으로 정의한다. 각 stratum p95의 단순 평균을 해당 W의 **Macro-p95**로 사용한다. 이는 전체 자연 사용분포의 p95나 전체 표본을 합친 pooled p95가 아니다.

후보 순서는 round-robin으로 교차하여 장비·서비스 변동을 줄인다. 공유 random seed는 반복 event phase를 재현하는 데만 쓰며 정답을 암호처럼 숨기는 fixture로 쓰지 않는다. trial 수는 실용적 계측 예산이지 통계적 정밀도를 보장하는 숫자가 아니다. paired 반복·변동 범위와 탐색 후 독립 확인 반복을 보존한다.

정상 응답 없이 30초를 넘으면 timeout을 기록한다. 성공 표본만 골라 p95를 만들지 않는다. right-censored trial을 그대로 표시하고 metric이 확정되지 않으면 수치/score를 미확정으로 남긴다. 알려진 하한만으로 band가 확정되면 **bounded score**라고 별도 표시할 수 있지만 30초 실측 완료로 둔갑시키지 않는다.

### 4.3 인간 대기와 유효 응답

Voice 시작점은 원음 발화 종료, Text 시작점은 전송 시각이다. 전사 확정이 늦다고 clock을 뒤로 옮기지 않는다. Context·LLM·정책·검증·추가 조회 시간은 모두 VIA 경로에 포함한다.

명시적 clarification/consent 답변을 기다린 인간 시간만 분리한다. 입력→질문과 답변→다음 질문/처리의 VIA 시간을 합쳐 불필요한 clarification으로 W-02가 빨라지지 않게 한다. 적절하지 않은 질문은 W-05에도 실패를 남긴다. 접수 인사·spinner·JSON 첫 글자는 유효 응답이 아니다.

### 4.4 성능을 숨기는 방식 금지

bounded Read/Search/Understand를 Core 대신 Agent에 배치해도 그 동일 사용자 답변의 W-01 대기는 포함한다. 반대로 open-ended Agent 조사·파일 생성·tool 실행 시간은 W-02 접수 이후 및 W-03 결과 생성 이전의 외부 업무 구간으로 분리한다. 여러 Agent 시간을 전체에서 단순 차감하지 않는다. 공통 trace에 각 metric 경계를 명시한다.

## 5. W별 측정 카드

### W-01 Conversational Reaction Responsiveness

대표 strata: **TC-01.1, TC-01.4, TC-02.1, TC-04.2, TC-06.2의 첫 적절한 clarification turn, TC-10.1**. 각각 첫 유효 내용까지의 p95를 구해 6개를 동일가중 평균한다. W-01용 TC-06.2에서는 필요한 질문의 시작이 endpoint이나 W-05에서는 후속 답과 최종 결과까지 평가한다.

Voice 활성인 trial은 적절한 Text와 audio가 모두 시작한 시점의 max, Text-only는 유효 Text 시점으로 측정한다. audio 전체 재생 종료를 기다리는 metric은 아니다. 응답이 틀리면 빠른 latency 성공으로 세지 않고 correctness 실패와 해당 latency censoring을 함께 남긴다.

**목표 2s**는 기존 논의의 대화 반응 예산이다. 원래 7-ASR Macro-p95의 분모와 다르므로 과거 점수를 이관하지 않는다.

### W-02 Task Handoff Responsiveness

대표 strata: **TC-08.1, TC-08.2, TC-07.1, TC-10.4**. 마지막 것은 기존 목표를 새 run에 연결하는 인계다. 복합 요청의 correctness는 W-05/06/회귀에 유지하고 요청 수가 다른 compound 경로를 동일 latency 표본으로 임의 혼합하지 않는다.

```text
handoff_end = max(agent_acceptance_confirmed, VIA_recoverable_correlation_committed)
handoff_time = handoff_end - original_input_end - explicit_human_wait_intervals
```

공통 시험 Agent는 고정된 accepted/run identity 및 조회 가능 상태를 제공한다. VIA outbox에 넣은 시각, 네트워크 write 완료, 낙관적인 화면 인사는 종료점이 아니다. 개별 Agent가 이 계약을 지원하지 않는다면 그 적합성/제약을 먼저 드러내며 더 이른 로컬 사건으로 대체하지 않는다. Agent가 무슨 보고서를 얼마나 잘 생성하는지는 이 metric 밖이다.

**목표 3s**는 대화 응답보다 넓은 인계 예산 제안이다. Agent 접수와 durable correlation의 동기 비용을 수용하되 사용자가 실행 여부를 오래 재확인하게 만들지 않겠다는 제품 판단이다. 외부 표준값은 아니다.

### W-03 Task Feedback Responsiveness

대표 strata: **TC-13.1 진행, TC-13.2 질문, TC-13.3 결과**. 시작은 Agent fixture에서 event/state가 **외부에 조회 또는 stream으로 제공 가능해진 source 시각**, 끝은 관련 Task에 올바른 내용이 실제 Text UI/notification에 표시되기 시작한 시각이다. 중간 broker에 도착한 시각으로 clock을 리셋하지 않는다.

Voice 출력은 활성 여부·기존 발화 여부에 따라 별도로 기록한다. 사용자가 말하는 동안 끼어들지 않는 정책 때문에 W-03의 유효 Text 전달을 실패로 보지는 않는다. 이것은 W-01과 endpoint의 사용자 의미가 다른 이유다. terminal result·질문은 누락/중복 금지, progress는 더 최신의 올바른 상태로 대체해도 되지만 watched revision을 반영했다는 근거가 있어야 한다.

**목표 1s**는 이미 준비된 정보를 전달하는 VIA 추가 지연의 제품 예산 제안이다. Polling/push 후보 모두 같은 기능을 지원하는 Agent와 같은 발생 시각을 사용한다.

### W-04 Concurrent Task Performance Isolation

```text
FLDR = W-01 Macro-p95 at 4 active background Tasks
       / W-01 Macro-p95 at 1 active background Task
```

W-01의 6개 동일 foreground probe를 사용한다. trial 전부터 각 background Task가 1초마다 한 번 상태를 갱신하는 고정 trace를 최대 30초 제공한다. 이것은 steady-state 합성 부하이며 제품 최대 동시 Task/자연 발생 빈도 주장이 아니다. Agent 업무 GPU 계산은 fixture 밖, VIA의 수집·반영·queue·모델 호출 비용은 포함한다. 4개를 단순 idle record로만 두지 않는다.

결과에는 **L1·L4 절대값과 ratio**를 모두 보존한다. 원래 느린 후보가 낮은 ratio를 얻는 것을 빠른 제품이라고 말하지 않는다. 1-Task baseline padding, 4-Task에서 event 누락/취소로 workload를 줄이는 행위, 후보별 rate 변경은 금지한다. **목표 1.25배**는 25% 이내 추가 저하를 허용하는 제품 예산이며 새 후보 전에 제안한 값이다.

### W-05 / W-06 Completion / Continuity

기존 11-B §B.8의 **48 / 30 TC**를 그대로 사용한다. 실제 model run을 TC마다 5회 수행하여 run별 `충족 obligation / 사전 적용 obligation`을 구한다. 반복 평균 후 TC를 동일가중 평균한다. strict per-run 결과와 5회 모두 통과 여부는 각각 보존한다. 95% macro score는 '모든 obligation 20개 중 정확히 19개'나 실제 Task 성공확률과 동치가 아니다.

multi-ASR TC의 요구는 관련 W에만 tag하고, 같은 실패를 자동 복사하지 않는다. 정답을 replay하는 Model stub은 semantic accuracy를 측정하지 않는다. pipeline별 임의 error rate를 가정해 점수를 채우지 않는다. 동등하게 잘 만든 후보 모두 100%면 동점이다.

TC-06.2·06.3·14.5는 **적절한 clarification, 원 Request에 대한 답변 binding, 최종 처리 결과**를 모두 판정해야 한다. 기존 짧은 관찰 문장이나 generator가 final outcome을 빠뜨리면 실행 전에 assertion ledger를 보완하고 버전을 잠근다. 이를 candidate 결과를 본 뒤 partial-credit 기준을 바꾸는 데 쓰지 않는다. 11-B 원문과 generated obligation ledger의 동일 의미를 Gate 2 실행 준비 검사에 포함한다.

### W-07 / W-08 Change Locality

A-01~09 전체의 평균 / M-01~09+C-01~06 전체의 평균. `|modified ∪ added ∪ removed|`, C/I/S/D 같은 granularity와 출발 revision을 적용한다. 기능 유지 설계 근거 없이 0개라고 하지 않는다. unsupported/적용 없음/미검증은 0이 아니며 전체 분모를 줄여 새 평균을 만들지 않는다.

목표 **2 / 3 elements per change**는 사용자 승인값을 유지한다. 이것은 Adapter 1개가 유일한 정답이라는 가정이 아니다. 기본 Architecture 요소가 더 많아도 변경되는 요소가 같으면 score는 같을 수 있다. 기능 유지가 설계 논증뿐이면 `DESIGN_ANALYSIS`, 실제 change/replay를 수행하면 그 evidence를 추가한다.

### W-09 Recovery Timeliness & Recoverability

대표값은 **서로 다른 두 recovery fault family의 strata p95를 동일가중 평균**한다.

#### A. Whole-VIA restart — 기존 4 strata

- running/queryable × active Task 1/4개
- completed/queryable × active Task 1/4개

각 strata를 100회 반복한다. fault 시점은 VIA process 종료이며 공통 시험기가 500ms 뒤 재시작을 시작한다. 저장 매체는 정상이고 Agent execution/result는 생존·조회 가능하다.

#### B. Integration-host fatal fault — 추가 2 strata

- running/queryable × active Task 1개
- running/queryable × active Task 4개

동일한 deterministic integration adapter fixture가 **자신을 host하는 runtime을 fatal 종료**시키는 fault를 발생시킨다. 논리 fault injection 지점은 모든 후보에서 동일하다. Single-process 후보에서는 VIA process 전체가 종료될 수 있고, process-isolated 후보에서는 integration worker만 종료될 수 있는데, 바로 이 blast-radius 차이가 Architecture 효과다. Agent execution은 계속 생존·조회 가능하다.

각 strata의 종료는 해당 Task들에 대해 **올바른 identity/state/result 및 허용된 사용자 제어가 다시 가능함**이 확인된 시점이다. 단순 process 재기동, worker 재기동, UI 표시만으로 종료하지 않는다. 시험기는 사라진 state/history를 재주입하지 않는다.

```text
W09 = mean(
  p95(whole_restart_running_1),
  p95(whole_restart_running_4),
  p95(whole_restart_completed_1),
  p95(whole_restart_completed_4),
  p95(integration_fatal_running_1),
  p95(integration_fatal_running_4)
)
```

복구 불가능/상태 확인 불가 조건은 latency를 작은 값으로 처리하지 않고 strict correctness regression에 남긴다. **목표 5s**는 일시 장애 후 업무 제어를 돌려주는 제안 예산으로 유지한다. 기존 27 correctness TC 결과도 별도로 보존한다.

### W-10 Failure Containment & Graceful Degradation

대표 suite는 후보 결과를 보기 전에 다음 **28 cells**로 고정한다.

#### A. External dependency failure — 24 cells

기존 **6개 실패 dependency × 4개 본질적으로 무관한 기능 = 24 cells**를 유지한다. 각 cell은 connection-refused와 no-reply 두 모드로 각각 5회 반복한다. fault는 30초 유지하고 2/10/20초에 기능을 요청한다. 같은 trial의 세 probe가 모두 기존 의미의 정답을 5초 이내 제공해야 trial PASS다.

#### B. Integration execution fatal fault — 4 cells

고정 integration adapter fixture가 자신을 host하는 runtime을 fatal 종료시키는 logical fault 1종 × 그 integration에 본질적으로 의존하지 않는 기능 4개 = **4 cells**를 추가한다. 각 cell을 5회 반복한다. Single-process 후보에서는 같은 process의 Core까지 영향을 받을 수 있고, process-isolated 후보에서는 integration worker만 영향을 받을 수 있다. fault injection API와 adapter code는 후보 간 동일하다.

```text
W10 = 100 × passed_cells / 28
```

분모는 후보의 dependency/process graph가 아니라 **사용자 기능의 본질적 dependency**로 사전 고정한다. 후보가 구조적으로 결합시킨 기능을 “원래 의존한다”며 분모에서 뺄 수 없다. 반대로 실패 dependency 자체의 고유 기능을 성공하라고 요구하지 않는다.

Text-only degraded mode가 허용된 cell은 완료조건을 사전 명시한다. external timeout/circuit breaker와 process isolation은 서로 다른 tactic/structure이므로 둘 다 정상적으로 구현할 수 있다. **목표 100%**를 유지한다. 이 28-cell suite에서 모든 정상 후보가 100%면 W-10은 non-discriminating regression으로 기록하고, 결과를 가르기 위해 새로운 극단 fault를 사후 추가하지 않는다.

### W-11 Privacy Exposure Minimization

대표 workload는 기존 TC **02.1, 02.2, 02.3, 05.1, 07.2, 10.1, 16.4, 17.2**다. `baseline.json`의 기존 합성 자료에서 보호 대상 fact/relationship 20개를 사전 표시한다. 업무 내용의 개인정보 민감성을 실제 사용자 데이터와 혼동하지 않는다.

```text
exposure = 100 × |whole-workload union of exposed protected units| / 20
```

사용자 PC 밖의 지정 Model/Agent endpoint로 전달되거나 해당 endpoint가 handle/capability를 이용해 실제로 읽을 수 있게 된 범위를 센다. 같은 단위 재전송은 한 번, recipient 수·횟수는 secondary로 남긴다. 로컬 신뢰경계 안에서만 처리하면 이 remote metric에는 0이다. 이는 모든 local processing이 개인정보 관점에서 안전하다는 뜻이 아니다.

압축·암호화·summary라는 포장 자체로 정보 단위를 없애지 않는다. 수신자가 해독/복원하거나 같은 사실을 알 수 있으면 그 단위를 기록한다. Abstract handle 자체를 보내도 열어주는 scope 전체를 센다. 공개정보는 분모를 늘리는 데 쓰지 않는다. 모든 요청을 차단해 얻은 0 노출은 정상 기능을 유지하지 못했으므로 적합한 privacy 비교값이 아니다.

**목표 25%=5/20**는 제한된 공유 예산을 검토하기 위한 신규 제품 제안이다. NIST가 정한 privacy cutoff나 유출 위험 확률이 아니다. 원래 사용자 목표상 반드시 공개해야 하는 정보량과 함께 리뷰해야 한다. 20개 정보단위가 임의 byte 쪼개기가 아닌지, 의미 동등한 요약 노출의 annotation이 일관적인지도 Gate 2에서 확인한다. 정보 단위/대상 corpus를 바꾸면 target을 조용히 재사용하지 말고 명시적 새 평가 버전으로 검토한다.

### W-12 Action & Access Safety

기존 SAFE-READ/EGRESS/APPROVAL/REVOCATION/ACTION_REVISION/MEMORY 6 family × allow/deny/stale/wrong-scope **24 opportunity**를 유지한다. 각 5회 중 한 번이라도 violation이면 그 opportunity V=1. 같은 기회의 로그·guard 개수로 N을 늘리지 않는다.

**목표 0/24**와 score-only 방침을 유지한다. 정상 allow 6개를 막으면 false-block/기능 회귀로 별도 공개하고 대표 분모에 새로운 성공 항목을 추가하지 않는다. VIA의 승인 연결과 실제 Agent 내부 action enforcement 경계를 구분한다. 임의 hard release gate나 다른 score로 안전성을 상쇄한 인증 결론을 추가하지 않는다.

## 6. Evidence eligibility와 준비 상태

| 근거 | 가능한 주장 | 불가능한 주장 |
|---|---|---|
| 공개 Qwen3-8B prompt/decode rate | 특정 source 환경의 model subtotal 근사 | Windows 목표 PC 실측 또는 전체 W-01/02 p95 |
| Qwen3-Omni 234ms | 공식 theoretical first-packet reference | 사용자 음성 종료→VIA 유효 audio p95 |
| 실제 모델 실행 | 해당 corpus에서의 W-05/06 | 모든 자연어 상황의 성능 보장 |
| 실제 candidate + deterministic Agent/event fixture | 구조/상태/장애/queue의 재현 가능한 결과 | 실제 Agent 지능 또는 인터넷 실서비스 SLA |
| 완전한 C/I/S/D change ledger | W-07/08 설계 변경량 | 구현 공수·성능 실측 |
| 설계+전체 reachable scope 분석 | W-11 계획상 노출 범위 | 실제 개인정보 유출 위험 확률 |

현재 코드가 수행한 것은 score lookup·분모·formula·coverage의 검증이다. 12개 전수 sweep을 실행한 것이 아니다. 음성/OS 원본, 실제 runtime build pin, candidate API adapter, exact prompt tokenization, semantic judgment와 실행 trace는 Gate 2/후속 실행 자산으로 남는다. 이 파일은 그 자산을 만들 때 바꿔서는 안 될 **비교 계약**을 먼저 명시한 것이다.

## 7. Freeze와 예외 처리

Gate 1은 catalog와 이 계약의 검토 버전, Gate 2는 candidate/E-ID/실행 자산 hash까지 동결하는 지점이다. 새로운 제품 목표·metric·분모가 필요하면 변경 사유·영향·버전을 기록하고 모든 후보에 다시 동일하게 적용한다. 결과를 보고 특정 후보만 band, timeout, cache, Agent 능력을 바꾸지 않는다.

공개 모델 profile의 정확한 runtime/hardware 분포가 부족한 상태에서는 타당한 latency 수치가 없는 칸을 비워둔다. 결과를 예쁘게 만들기 위해 estimated 평균을 p95로 이름만 바꾸지 않는다.