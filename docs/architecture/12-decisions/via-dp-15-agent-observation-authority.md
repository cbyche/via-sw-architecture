# VIA-DP-15 — Agent progress state의 유지 방식

> **핵심 DP 초안 v2 · 2026-09-28 · 사용자 선정 반영 / 상세 Measurement Freeze 전**
>
> 질문: Agent의 중간 progress·질문·결과를 VIA가 지속적으로 projection해 관리할 것인가, event는 변화 hint로만 쓰고 필요한 시점에 authoritative snapshot을 조회할 것인가?
>
> 현재 판단: **2026-09-28 핵심 DP shortlist에 포함**. Agent delegation 이후 사용자가 보지 않는 동안에도 VIA가 진행 상태를 어떻게 관리하는지까지 범위를 넓혔다. 단순 event 대 polling이나 event마다 query하는 약한 비교가 아니며 실제 QA 측정은 `NOT_RUN`이다.

## 1. 배경 — 완료 소식을 받으면 바로 완료로 알려도 될까?

Agent가 완료 event를 보냈지만 연결이 끊겼고 뒤늦게 오래된 progress가 도착한다. VIA는 빠르게 소식을 전하면서도 Task를 과거 상태로 되돌리지 않아야 한다. Event와 query를 같이 쓰는 것은 자연스럽다. 남는 선택은 유효한 event 자체가 상태 확정 근거인지, 별도 query 결과가 있어야 확정하는지다.

```mermaid
flowchart TB
 A["외부 Agent Runtime"] -.->|"완료 event 뒤 오래된 progress"| Q["검토 지점<br/>event로 확정할까?<br/>query 확인을 기다릴까?"]
 A -->|"현재 상태 query 응답"| Q
 Q -->|"유효한 관측만 반영"| T["VIA Task 상태"]
 T -->|"확정된 상태를 안내"| U["사용자의 Voice·화면"]
```

배경의 검토 지점은 추가 Component가 아니다. VIA는 Voice·Text·화면 interaction과 Task 연결을 소유한다. 실제 업무 계획·도구 실행은 외부 Downstream Agent의 책임이고 모델은 추론 dependency다.

## 2. 비교 범위와 공통 조건

대상은 Agent delegation 이후 progress·question·artifact·terminal state를 VIA가 시간에 따라 유지하는 방식이다. 사용자가 매 event를 통지받는지는 notification policy이고 이 DP와 분리한다. 통지하지 않더라도 A는 VIA 내부 projection을 갱신하고, B는 마지막 확인 snapshot과 dirty/cursor 상태를 유지한다.

Agent 의미 정규화는 VIA-DP-09, VIA Task의 실제 writer topology는 14다. 같은 Agent가 identity·revision/cursor·event와 권위 있는 snapshot query를 제공하는 profile에서 비교한다. 제공하지 않는 revision·snapshot 일관성은 VIA가 만들어내지 않는다. poll 간격이나 알림 빈도만 바꾸는 tuning은 범위 밖이다.

**모델 불변식:** S2S 모델 1개 + semantic LLM 1개. Component·Task별 모델을 별도 적재하지 않는다. 프롬프트·세션·호출을 나눠도 공유 모델이며 동시 처리·취소 지원을 임의 가정하지 않는다.

**그림의 구현 수준:** VIA Core Process는 A/B 공통 비교용 배치다. Process 격리는 VIA-DP-11의 별도 축이며 외부 Agent는 양쪽 모두 별도 Runtime이다. 실선은 라벨의 호출·반환·저장, 점선은 비동기 event다. 메모리 queue 수락과 디스크 commit을 구별하며 별도 message bus 제품은 가정하지 않는다. queue 용량·포화 정책은 추후 동일 조건으로 동결한다. 이 구조도는 후보 명세이며 구현 완료 증거가 아니다.

**출처와 한계:** [시스템 경계](../01-system-mission-and-boundary.md), [UC](../05-representative-use-cases.md), [공통 조건](../06-fixed-assumptions.md), [현행 QA](../08-quality-attributes/quality-model.md)가 요구의 기준이다. 아래 구조는 그 요구를 만족시키려는 후보 설계다. 실제 지연·오류 빈도·변경 요소 ledger는 미확인이다.

## 3. 대안 A — Continuous Progress Projection

정상 상태 event가 계약 검사를 통과하면 query를 기다리지 않고 durable VIA progress projection에 반영한다. projection은 phase·progress·blocked reason·pending question·artifact version·terminal state·source revision·last-confirmed time을 Task와 연결한다. 사용자가 통지를 받지 않더라도 이 상태는 계속 갱신된다. gap·재연결·불일치 때 query로 조정하는 hybrid다.

```mermaid
flowchart TB
 A["외부 Agent Runtime<br/>event와 query 동일 기능"] -.->|"비동기: run·revision·status"| Q
 subgraph V["VIA Core Process"]
 Q["공통 bounded event inbox"] --> E["[변경] Event Validator<br/>유효 event 확정 근거 인정"]
 E <-->|"source cursor·dedup"| C[("관측 checkpoint")]
 E -->|"유효 Observation"| T["[변경] Continuous Progress Projection<br/>Task·phase·question·artifact·revision"]
 E -->|"gap·충돌·reconnect"| R["공통 Query Reconciler"]
 R -->|"확인된 snapshot·순서 경계"| E
 T -->|"commit된 상태"| P["공통 Voice·Text Publisher"]
 end
 R <-->|"query current state"| A
```

**실제 호출·상태·실패 처리 순서**

1. Inbox가 event를 받고 Validator가 run identity·실제 source revision·terminal 규칙을 검사한다. 유효하면 바로 Task Owner에 Observation을 제출한다.
2. Task Owner가 commit하고 Publisher가 상태를 알린다. event 처리와 checkpoint 저장 사이의 crash에도 중복 적용되지 않게 command identity를 연결한다.
3. gap·재연결 때는 query로 현재 상태를 확인한다. query와 늦은 event의 순서는 source 계약으로 정하며 비교 불가능하면 보류·재확인한다.

## 4. 대안 B — On-demand Authoritative Snapshot

event를 상태 변경 hint로 사용하지만 전체 progress state는 query의 검증된 snapshot으로 materialize한다. VIA는 마지막 confirmed snapshot, source cursor, `dirty` 여부와 짧게 살아 있는 question/terminal hint를 보존한다. 사용자의 status 질문, notification rule 발동, terminal·approval 후보, event gap·reconnect·VIA restart 때 query한다. 같은 run의 event burst는 합치고 concurrent query는 하나로 제한한다.

따라서 event 하나마다 query 하나를 보내는 안이 아니다. 권위 있는 snapshot 경계를 택하는 대신 사용자가 묻거나 notification을 결정할 때 query 대기가 남고, query가 제공하지 않는 짧은 중간 사건을 놓칠 수 있다.

```mermaid
flowchart TB
 A["외부 Agent Runtime<br/>event와 query 동일 기능"] -.->|"비동기: 상태 변경 알림"| Q
 subgraph V["VIA Core Process"]
 Q["공통 bounded event inbox"] --> H["[변경] Hint Coalescer<br/>cursor·dirty·question/terminal hint"]
 H -->|"중복 요청 합치기"| R["공통 Query Reconciler"]
 R -->|"검증된 snapshot만"| E["[변경] Snapshot Validator"]
 E <-->|"query watermark·dedup"| C[("관측 checkpoint")]
 E -->|"유효 snapshot"| T["[변경] Last Confirmed Snapshot<br/>필요 시 materialize"]
 T -->|"commit된 상태"| P["공통 Voice·Text Publisher"]
 end
 R <-->|"query current state"| A
```

**실제 호출·상태·실패 처리 순서**

1. 같은 event를 받지만 Hint Coalescer가 cursor·dirty 상태와 즉시 확인이 필요한 question/terminal hint만 기록한다. payload의 progress 값을 곧바로 authoritative Task projection으로 확정하지 않는다.
2. 사용자가 상태를 묻거나 notification rule·terminal hint·gap·reconnect가 query를 요구하면 Reconciler가 snapshot을 요청한다. Snapshot Validator가 identity·신선도·terminal 의미를 검사해 last-confirmed state를 갱신한다.
3. 동시 query를 합치고 오래된 snapshot을 거절한다. ‘query는 언제나 최신’이라고 가정하지 않는다. 필요한 source 보장이 없으면 이 후보도 부적합이다.

## 5. 구조 차이·상호 배타성·Hybrid 검토

| 항목 | A | B |
| --- | --- | --- |
| 정상 event의 효력 | continuous progress projection의 상태 전이 입력 | cursor·dirty·query를 깨우는 hint |
| 사용자 status/notification 경로 | local projection에서 즉시 구성 가능 | 필요하면 authoritative snapshot query |
| 유지 상태 | event history/checkpoint·현재 projection·gap | last snapshot·cursor·dirty·hint·query freshness |

같은 유효 event로 continuous projection을 갱신하면 A, event는 dirty/hint만 남기고 사용자에게 필요한 현재 상태를 snapshot query로 materialize하면 B다. event-first와 gap query 복구의 hybrid를 A에 포함했다. B의 coalescing·terminal fast query도 허용한다. query/event의 단순 transport 선택이나 notification 빈도는 독립 DP로 만들지 않는다.

양쪽은 같은 기능·안전 조건·자원·외부 capability를 만족하는 **서로의 steelman**이어야 한다. 동일 결정 범위에서는 **mutually exclusive**해야 한다. cache·batch·공통 library·정확성 검사·로그를 한쪽에서 금지해 차이를 만들지 않는다. 같은 강한 설계로 수렴한다면 동점 또는 보조 결정으로 남긴다.

## 6. 같은 사건을 통과시키는 사고실험

<a id="t1"></a>

### T1. 정상 흐름과 critical path

같은 시각에 source progress 또는 완료가 제공된다. A는 event 검증→projection commit→notification/status Voice를 거친다. B는 hint→필요성 판단→snapshot query·검증→동일 출력을 거친다. 추가 query가 critical path에 남으면 QA-03과 사용자 status query의 QA-09는 A가 유리할 수 있다. 반대로 event가 매우 많고 사용자가 거의 묻지 않으면 B가 projection write·schema 부담을 피할 수 있다.

<a id="t2"></a>

### T2. 정정·실패·재연결

event 하나 누락 뒤 오래된 event가 도착한다. A는 gap 감지와 projection reconciliation, B는 dirty/cursor에서 snapshot을 다시 확인한다. 두 안 모두 stale terminal rollback을 금지한다. Agent가 불가용하면 A는 마지막 confirmed projection으로 사용자에게 시각과 staleness를 설명할 수 있고 B는 last snapshot 이후 dirty 상태를 명시해야 한다. Query가 최종 상태만 제공해 승인 질문이나 필요한 중간 milestone을 잃는 profile이면 B의 기능 적합성이 먼저다.

<a id="t3"></a>

### T3. 변경·연구 기록과 반례

A-04 상태 제공·A-05 실행 identity·A-08 질문 계약 변화에서 cursor/validator와 query/coalescer의 변경 요소를 비교한다. B도 typed adapter·공통 reader를 재사용할 수 있다. A가 event 구조를 이미 공통 경계에서 흡수하면 반대 방향 변경 이점이 사라진다. QA-61에는 source event와 실제 query·commit·audible 관계를 남기며 모델 사고과정은 수집하지 않는다.

## 7. Core ASR 적용과 상세 QA 사고실험

이 DP의 역할은 **QA-09/19/29/39 모두 `PRIMARY`**다. Agent progress/status 전달과 사용자 status query 시간, source-confirmed field, progress schema·notification/query 계약 변화와 event-stream loss 뒤 복구를 함께 측정한다. 최종 모집단은 [Core ASR Contract의 DP 원장](../08-quality-attributes/core-asr-contract.md#8-via-dp-0118-적용-원장)에서 freeze한다.

QA-19에는 개정한 QA-11 status outcome contract에 따라 `phase`, `progress`, `blocked_reason`, `answer_needed`, `terminal_state`, `artifact_version`, `staleness`, `notification_disposition` 중 case에 applicable한 field를 중복 없이 등록한다. `source_revision`은 freshness 판정 provenance로 보존한다. 실제 `task_id`, `agent_execution_id`, `pending_question_id` binding과 역순·중복 event 뒤 수렴은 QA-13/14 회귀로 독립 검증한다. 잘못된 Task의 progress나 오래된 terminal state 적용은 qualification gate 위반과 함께 기록한다. 아래 표는 상세 input·diagnostic이다.

**사고실험 예상 / 실제 측정 `NOT_RUN`.** 모든 판정은 §2의 동일 조건과 T1~T3의 가설에 한정한다. 조건부 방향은 전체 metric의 실측 우세가 아니다. 시간·변경 수·장애 단위·메모리·노출은 작을수록, 성공·완전성·재현 비율은 클수록 좋다. 일부 사례의 차이를 최악 p95·전체 change pack 평균으로 확대하지 않는다.

| QA · 단일 metric | 예상 방향·크기 | 확실성 | 구조적 이유·반례 | 역할 |
| --- | --- | --- | --- | --- |
| QA-01 위임 경로 VIA 처리시간 · 최악 case p95; Agent 실행 제외 | 조건부·크기 미정 | 낮음 | 결과 확인에 추가 query가 실제 참여할 때 A 가능 [T1](#t1) | 비교 후보 |
| QA-02 직접 Voice 응답시간 · 최악 case p95 | 비슷 | 중간 | Agent 비참여 direct 경로 [T1](#t1) | 회귀 |
| QA-03 Agent 상태 Voice 전달시간 · 최악 case p95 | 조건부 A 우세 가능·크기 미정 | 중간 | 정상 source 이후 query 대기가 남는 경우 [T1](#t1) | 비교 후보 |
| QA-04 음성 중단시간 · 최악 case p95 | 비슷 | 중간 | playback stop은 외부 상태 확인과 분리 [T1](#t1) | 회귀 |
| QA-05 Task 제어 응답시간 · 최악 control case p95 | 조건부 A 우세 가능 | 낮음 | 올바른 disposition에 추가 확인이 필요한 경우 [T2](#t2) | 비교 후보 |
| QA-11 전체 요청 처리 정확도 · case별 strict 성공률 평균 | 판단 근거 부족 | 낮음 | 동등 기능·실제 corpus 필요 [T2](#t2) | 필수 검증 |
| QA-12 의미 해석 정확도 · strict 성공 run 비율 | 비슷 | 중간 | 의미 모델·입력 해석 고정 [T1](#t1) | 회귀 |
| QA-13 Task·Interaction 연결 정확도 · strict 성공 run 비율 | 비슷 예상 | 중간 | run·question·Task mapping 공통 [T2](#t2) | 필수 검증 |
| QA-14 비동기 상태 수렴 · strict 성공 run 비율 | 비슷 예상 | 중간 | 양쪽 source freshness 검증 필수 [T2](#t2) | 필수 검증 |
| QA-15 대화·Task 연속성 · 성공 scenario 비율 | 비슷 예상 | 중간 | 같은 대화·Task identity [T2](#t2) | 회귀 |
| QA-21 Agent 변경 영향 · 9개 변화의 변경 요소 평균 | 조건부·방향 미정 | 낮음 | event 관리와 query 계약의 전체 9건 변경 [T3](#t3) | 비교 후보 |
| QA-22 Model·Context·State 변경 영향 · 15개 변화의 변경 요소 평균 | 비슷 예상·ledger 필요 | 낮음 | 관측 checkpoint의 C-06 영향만 별도 확인 [T3](#t3) | 회귀 |
| QA-23 실험·로그 변경 영향 · 5개 변화의 변경 요소 평균 | 판단 근거 부족 | 낮음 | source/query correlation 계측 변경 [T3](#t3) | ledger 확인 |
| QA-31 올바른 Task 복구시간 · 최악 fault p95 | 조건부·방향 미정 | 낮음 | 공통 재연결 query와 cursor 복구 [T2](#t2) | 비교 후보 |
| QA-32 불필요한 장애 영향 · 초과 중단 단위 최대 수 | 비슷 예상 | 중간 | 같은 Process·외부 dependency [T2](#t2) | 회귀 |
| QA-41 PC 메모리 · 최악 workload의 peak p95 | 판단 근거 부족 | 낮음 | inbox·query 동시 수명 차이 미측정 [T1](#t1) | 자원 확인 |
| QA-51 보호정보 초과 노출 · 초과 단위 수 | 비슷 | 중간 | 같은 최소 상태·수신 범위 [T3](#t3) | 필수 회귀 |
| QA-61 실행 trace 완전성 · 완전한 trace run 비율 | 비슷 예상 | 중간 | source부터 출력까지 양쪽 기록 가능 [T3](#t3) | 필수 회귀 |
| QA-62 평가 재현성 · 동일 결과 재계산 비율 | 비슷 | 중간 | 동일 evidence·분석기 보존 [T3](#t3) | 회귀 |

QA-11과 QA-12~15는 통합 결과와 원인 지표이므로 중복 합산하지 않는다. QA-41은 제품 메모리 상한·중요한 구조 차이가 미확인이라 자원 확인 대상으로 유지하며 핵심 ASR 우선 추천에서는 제외한다. 이 표의 관련성만으로 ASR을 확정하지 않는다.

## 8. 공정한 검증 계획 — 실행하지 않음

Agent event/query의 정보량·순서·snapshot 일관성을 먼저 명세한다. event 누락·중복·역순, query 지연·오래된 snapshot, 질문의 짧은 수명을 동일하게 주입한다. QA-03의 source 시점을 query 응답 시점으로 옮겨 B의 대기를 지우지 않는다.

동일 입력·외부 사건·모델 설정·총 자원을 사용하고 이 DP만 바꾼다. 실제 Voice audible endpoint, 실패·timeout 포함, 반복·집계·target·동점 기준을 결과 전에 동결한다. 잘못된 대상 Action·중복 Action·무효 승인·무단 접근은 점수로 상쇄하지 않는다. 모델 replay는 실제 의미 정확도 측정이 아니다. 근거 계약은 [공통 검토 절차](./dp-review-protocol.md), [event 경계](../11-measurement/event-boundary-contract.md), [QA catalog](../08-quality-attributes/quality-model.md)를 따른다.

## 9. 다른 DP·변경 비용·구현 근거

09의 의미 변환, 14의 writer, 08의 복구 저장, 11의 배치를 고정한다. 전환에는 cursor·query watermark·진행 질문·중복 관측을 이행한다. 기존 event-first 문서는 참조 tactic이며 이 A/B의 현행 계약과 측정 결과를 대신하지 않는다. 현재 제품용 후보와 QA 결과는 NOT_IMPLEMENTED / NOT_RUN이다.

## 10. 현재 판단과 재검토 조건

**Continuous projection 대 on-demand snapshot으로 핵심 shortlist에 유지한다.** 후속 심층 검토에서 필수 milestone·짧은 질문의 보존 범위, notification policy와 상태 authority의 분리, status query/Voice endpoint, progress field registry와 event-loss/restart fault pack을 동결한다. 모든 중간 event를 반드시 VIA에 영속해야 하는 제품 요구가 확정되면 B는 비교 후보가 아니라 기능 부적합이 된다.

## 11. 자체 검토에서 반영한 개선점

push 대 polling처럼 섞을 수 있는 기술 대비를 버리고 ‘query 없이 상태를 확정할 수 있는가’로 좁혔다. 단순 polling 주기·외부 기능 부족으로 우열을 만들지 않았다.

이 검토는 문서·사고실험이며 외부 심사나 후보 QA 검증 통과가 아니다. 옛 번호는 [요약의 추적성 부록](./dp-executive-summary.md#legacy-mapping)에만 연결하고, 현재 설명은 이 VIA-DP와 현행 QA 번호로 완결한다.
