# VIA-DP-03 — 음성 입력 근거의 최종 기준

> **검토 초안 v1 · 2026-09-24 · 사용자 검토 전**
>
> 질문: 음성·정정·시각 근거의 의미를 VIA가 정규화해 소유할 것인가, S2S 제공자의 이벤트 계약을 기준으로 삼을 것인가?
>
> 현재 판단: **선행 기능 적합성 결정 — 기존 보조 모델 묶음을 재정의, 핵심 점수 비교는 보류** 대안 선택·구현·QA 측정은 하지 않았다. 실제 결과는 모두 `NOT_RUN`이다.

## 1. 배경 — 늦게 도착한 ‘여기’는 어느 화면을 가리키는가?

사용자는 첫 그래프를 가리키며 “여기”, 두 번째 그래프를 가리키며 “여기와 비교해줘”라고 말한다. 전사 결과가 늦거나 수정되어 들어와도 VIA는 당시 화면과 각각의 표현을 연결해야 한다. S2S가 필요한 정보를 항상 같은 형식으로 제공한다고 가정할 수 없다. 이 문제는 ASR 제품 선택이 아니라 어떤 입력 근거를 최종 기준으로 유지하고, 누가 그 의미를 보존하는가의 문제다.

```mermaid
flowchart TB
 A["음성·화면 원천 시각"] -->|동시 관측| Q["[검토 지점] 입력 근거와 revision 기준"]
 S["S2S의 지연·수정 event"] -->|전사·시각| Q
 Q -->|지칭 시점의 근거| C["VIA 요청 이해"]

classDef common fill:#F3F4F6,stroke:#64748B,color:#111827;
classDef change fill:#FFF7ED,stroke:#C2410C,stroke-width:3px,color:#7C2D12;
class A,S,C common;
class Q change;
```

배경 그림의 주황색 검토 지점은 설계 질문이지 추가 Component가 아니다. VIA는 사용자 PC의 Voice·Text interaction과 업무 연결을 책임진다. Downstream Agent는 실제 업무 계획·Tool 실행을, Model Runtime은 추론 실행을 담당한다. 이 보고서는 그 내부 알고리즘이나 학습을 고르지 않는다.

## 2. 비교 범위와 공통 조건

**용어:** S2S는 음성을 입력받아 음성으로 응답하는 모델 의존성이다. native 계약은 그 제공자가 원래 내보내는 정보의 의미와 형식이다. revision은 입력 해석의 정정 버전이며, 같은 원음·화면의 시각을 연결해야 한다. 보조 처리는 빠진 근거를 확보하는 VIA 책임이지 원천에 없던 정답을 시험기가 제공한다는 뜻이 아니다.

대상은 Core에 전달하는 음성 입력·시각·정정 근거의 의미 계약이다. S2S 사용은 양쪽에 고정한다. 음성 출력용 TTS, 물리 재생 정지, 응답 게시 권한을 하나의 ‘보조 모델 사용’ 선택으로 묶지 않는다. 필요한 입력 근거가 없는 후보는 느리거나 부정확한 점수가 아니라 기능 부적합이다.

**기준선에서 확인한 사실:** UC-03·04는 사전 선택과 발화 중 지칭·정정을 모두 요구하며, 06 FA-09·10은 시험기가 잃어버린 이력이나 S2S에 없는 기능을 무료로 보충하지 못하게 한다. 요구의 출처는 [System Mission](../01-system-mission-and-boundary.md), [Fixed Scope](../03-fixed-architecture-scope.md), [Use Cases](../05-representative-use-cases.md)다.

**이번 비교의 설계 가정:** 같은 음성·pointer·screen source, source 시각과 도착 시각, S2S 기능 profile, 허용 Context와 정답 corpus를 사용한다. 실제 acoustic onset·playback stop은 양쪽에서 별도 관측한다. 역할이 추가되는 보조 모델의 자원과 지연은 숨기지 않는다.

**미확인 사항:** 현재 선택된 S2S 제공자가 없으므로 B가 전체 지칭·정정 UC를 만족하는 데 충분한 native evidence를 제공하는지 미확인이다. A의 정규화·보조 처리 비용과 모델 적합성도 미측정이다. 사실·후보 설계·미확인 가정을 서로 바꿔 쓰지 않는다. Conversation은 이어지는 대화, Request는 논리적 요청, Task는 여러 요청에 걸쳐 추적하는 업무다. Oracle은 실행 전에 정한 정답·허용 상태 조건이고, fixture는 고정 입력·외부 사건이다.

## 3. 대안 A — VIA 입력 계약 + 필요한 경우의 보조 처리

VIA의 입력 근거 관리자가 S2S event와 원음·화면 시각을 정규화해 Request version에 연결한다. S2S가 충분한 근거를 제공하면 그대로 활용하고, 계약상 필요한 정보가 부족하면 보조 인식·정렬을 사용한다. 고정된 ASR·VAD·TTS 전체 묶음을 항상 실행하지 않는 hybrid다.

최종 입력 version과 원천 근거의 관계를 VIA가 소유한다. S2S와 보조 결과가 충돌하면 조용히 덮어쓰지 않고 정정·보류 기준으로 처리한다. 강점은 제공자 변화에 대한 VIA 계약의 안정성이고, 약점은 충돌 해소와 보조 처리의 독립 수명·자원 책임이다.

```mermaid
flowchart TB
 S["공통 S2S Runtime"] -->|입력 event| N
 R["공통 원음·화면 시각"] -->|관측 가능한 근거| N
 subgraph V["VIA 논리 경계 / A"]
 direction TB
 H["[변경] 필요 시 보조 인식·정렬"] -->|보완 근거| N["[변경] VIA 입력 근거 관리자"]
 N -->|VIA version·출처 계약| C["공통 요청 이해"]
 C -->|VIA revision에 연결| E[("입력 근거 기록")]
 end

classDef common fill:#F3F4F6,stroke:#64748B,color:#111827;
classDef change fill:#FFF7ED,stroke:#C2410C,stroke-width:3px,color:#7C2D12;
class S,R,C,E common;
class H,N change;
```

보조 처리기는 항상 호출되는 고정 단계가 아니다. 어떤 결과가 현재 입력을 나타내는지 결정하는 계약은 VIA가 소유한다.

## 4. 대안 B — S2S 입력 계약을 기준으로 사용

S2S가 제공하는 transcript·segment·revision·time evidence를 최종 음성 입력 근거로 삼는다. VIA adapter는 표현·clock 연결을 변환하고 필요한 원천을 보존하지만, 독립된 보조 의미 입력으로 제공자 결과를 대체하지 않는다. 제공자별 typed event를 숨기지 않는 대신 Core와 grounding 경계에서 명시적으로 해석한다.

충분한 native 기능을 가진 제공자에서는 이중 입력 source의 충돌과 보조 Runtime을 피할 수 있다. 빠른 source 처리와 사전 원음·화면 보존, 재연결·중복 제거는 허용한다. 그러나 native 계약으로 필요한 의미·정정 근거를 만들 수 없으면 이 profile에서 B는 성립하지 않는다.

```mermaid
flowchart TB
 S["공통 S2S Runtime"] -->|기준 입력·revision| N
 R["공통 원음·화면 시각"] -->|clock·화면 근거| C
 subgraph V["VIA 논리 경계 / B"]
 direction TB
 N["[변경] S2S 계약 adapter"] -->|제공자 의미를 보존한 event| C["공통 요청 이해"]
 C -->|provider revision에 연결| E[("입력 근거 기록")]
 end

classDef common fill:#F3F4F6,stroke:#64748B,color:#111827;
classDef change fill:#FFF7ED,stroke:#C2410C,stroke-width:3px,color:#7C2D12;
class S,R,C,E common;
class N change;
```

adapter가 있다는 이유만으로 A가 되지 않는다. 독립 입력 source로 제공자 의미를 대체·중재하는 권한을 두지 않는 것이 B의 경계다.

두 구조도는 같은 확대 영역을 그린다. 회색은 공통 책임, 주황색과 `[변경]` 표기는 바뀌는 책임이다. 실선은 이름을 붙인 기능 흐름, 점선은 명시된 비동기 전달이다. **별도 Process라고 적힌 경우 외에는 논리 경계**다. 상자 수는 변경 요소 수나 메모리 크기가 아니다.

## 5. 구조 차이·상호 배타성·Hybrid 검토

| 차이 | A | B |
| --- | --- | --- |
| 음성 입력의 최종 의미 계약 | VIA-owned input revision | S2S-native revision 의미 |
| 부족한 native 정보 | 보조 근거로 보완 가능 | 주어진 native profile로 요구 충족 여부 심사 |
| 변경 경계 | 정규화·보조 처리 책임 | native 계약 소비·adapter 책임 |
| 공통 | 원천 시각·출처, S2S, 화면 기록, 실제 음성 정지 | 동일 |

같은 입력 충돌에서 A는 VIA 계약에 따라 근거를 중재하고 B는 S2S 계약을 최종 의미 기준으로 삼는다. B가 보조 인식 결과를 최종 대체 입력으로 인정하면 A로 이동한다. 단순 format 변환·clock 변환·원음 보존은 양쪽에 허용된다.

S2S만으로 충분하면 직접 사용하고 부족할 때 helper를 켜는 구조를 A로 포함했다. 초기 후보의 VAD·ASR·TTS·정렬 묶음은 서로 독립인 책임을 섞으므로 폐기했다. TTS는 출력 구현 조건, 물리 중단은 공통 필수 기능, response authority는 DP-04로 남긴다. 동등 기능의 B가 없는 profile에서는 억지로 A/B를 만들지 않는다.

A/B 모두 같은 기능·권한·실패 의미와 합리적인 보완책을 허용하는 **steelman**이다. 같은 결정 범위의 최종 기준은 **mutually exclusive**해야 한다. 속도 차이를 만들기 위해 한쪽의 검증·기록·cache를 빼지 않는다.

### 같은 사건에서 확인하는 순서 차이

다음 그림은 A/B의 차이가 드러나는 동일 사건을 비교한다. 외부 완료와 VIA 내부 확정을 구별하며, 아래 사고실험에서 지연·실패 조건까지 확인한다.

```mermaid
sequenceDiagram
 participant S as S2S 원천
 participant V as VIA 입력 경계
 participant H as 선택적 보조 처리
 participant C as 의미 처리
 S-->>V: 시각이 붙은 입력 revision
 alt A VIA 근거 중재
 V->>H: 필요할 때만 원음 근거 보완
 H-->>V: 보조 근거와 출처
 V->>V: VIA 기준 revision 확정
 else B Native 의미 기준
 V->>V: 원천 revision 의미 유지·형식 변환
 end
 V->>C: 기준 revision과 화면 시점
 S-->>V: 늦은 정정
 V->>C: 같은 기준에 따라 이전 판단 무효화
 Note over S,C: B에 필요한 원천 정보가 없으면 기능 적합성 실패
```

## 6. 같은 사건을 통과시키는 사고실험

<a id="t1"></a>

### T1. 두 지칭과 늦은 정정

같은 source timeline에서 첫 ‘여기’의 수정 event가 두 번째 지칭 뒤 도착한다. A는 raw evidence → 정규화 revision → 필요한 보조 정렬 → 최종 referent 확정 순서로 처리한다. B는 native segment/revision → provider 의미 해석 → 같은 화면 근거 연결을 수행한다. 양쪽 모두 최신 화면 좌표로 과거 지칭을 대체하지 않는다.

A가 부가 정보를 얻어 더 잘 맞춘다는 주장은 실제 추가 근거의 생성 가능성과 정확도가 필요하다. Native evidence가 충분하면 B도 같은 대상을 얻는다. 반대로 어느 안도 원음·시각에서 복원할 근거가 없으면 clarification이 정답이며 oracle가 지칭을 무료로 알려주지 않는다. QA-12/11 방향은 실제 corpus 없이 확정할 수 없다.

<a id="t2"></a>

### T2. 지연·중단과 보조 호출

A의 추가 처리가 critical path에 남는 경우에만 B의 QA-01/02/05 이점이 생긴다. 입력 중 병렬로 끝나거나 A가 native 결과를 그대로 쓰면 차이는 사라진다. A가 더 빠른 turn evidence를 제공하면 VIA 인식 지연을 줄일 가능성도 있지만, source 기능·정확성 검증 없이 그 시간을 만들지 않는다.

QA-04는 양쪽의 acoustic barge-in부터 실제 재생 정지까지다. ‘B는 S2S 서버가 인식할 때까지 기다린다’는 약한 안을 만들지 않는다. 공통 local playback 제어가 같다면 비슷하다. 추가 helper가 PC 상주 자원을 요구하면 A의 QA-41 부담이 커질 수 있으나 provider가 충분한 case에서는 추가 상주를 강제하지 않는다.

<a id="t3"></a>

### T3. 제공자 변경·재연결·연구 기록

M-01/07은 이번 경계에 직접 관련된다. M-07의 단어 시각→구간 시각 변경에도 원음은 제공되지만, B가 native 의미만으로 UC-04를 만족할 수 있는지는 별도 적합성 확인이다. 실패를 낮은 change count나 낮은 정확도 점수로 포장하지 않는다. A도 보조 정렬·입력 계약이 함께 바뀌면 여러 요소를 수정할 수 있다. M-02~06/08/09는 실제 역할·Runtime 변화가 닿는 경계를 각각 확인하고 C-01~06은 공통 Context·상태 계약 중심의 회귀다.

Agent A-01~09는 동일 경계를 유지한다. E-01~05는 입력 source·revision의 계측과 schema reader까지 추적하되 더 많은 event가 자동으로 더 완전한 trace를 뜻하지 않는다. 재연결 후 provider epoch를 새 VIA Conversation으로 착각하지 않고, 과거 근거와 이어 주어야 한다. QA-31은 실제 Task가 영향을 받은 fault에 한정하며 Voice-only 재연결을 Task 복구로 대체하지 않는다. 양쪽 모두 원음 무제한 보존이나 과다 외부 제공은 허용하지 않는다.

## 7. 전체 19개 QA 비교

**사고실험 예상 / 실제 측정 `NOT_RUN`.** §2의 조건과 위 사고실험을 적용한다. 시간·변경 수·중단 수·메모리·노출은 작을수록, 정확성·연속성·완전성·재현 비율은 클수록 좋다. “비슷”은 명시한 조건에서 차이가 작다는 예상이며, “판단 근거 부족”은 방향·크기를 모른다는 뜻이다. 확실성은 실측 신뢰구간이 아니다. **부분 사례의 차이를 최악 case p95·전체 corpus·전체 change pack의 대표값 차이로 확대하지 않는다.**

| QA · 단일 metric | 예상 방향·크기 | 확실성 | 구조적 이유·반례와 근거 | 역할 |
| --- | --- | --- | --- | --- |
| QA-01 위임 경로 VIA 처리시간 · 최악 case p95; Agent 실행 제외 | 조건부; 크기 미정 | 낮음 | 보조 처리의 비중첩 구간이면 B, 더 이른 유효 입력이면 A 가능 [T2](#t2) | 주 비교 가능성 |
| QA-02 직접 Voice 응답시간 · 최악 case p95 | 조건부; 크기 미정 | 낮음 | native fast path가 같으면 비슷; 호출 수만으로 판단 금지 [T2](#t2) | 주 비교 가능성 |
| QA-03 Agent 상태 Voice 전달시간 · 최악 case p95 | 비슷 | 중간 | source status의 공통 음성 전달은 입력 근거 결정과 별개 [T2](#t2) | 회귀 |
| QA-04 음성 중단시간 · 최악 case p95 | 비슷 | 중간 | local playback stop은 두 안 모두 허용 [T2](#t2) | 필수 회귀 |
| QA-05 Task 제어 응답시간 · 최악 control case p95 | 조건부; 크기 미정 | 낮음 | Voice control 입력 해석의 추가 근거 처리에 한정 [T2](#t2) | 주 비교 가능성 |
| QA-11 전체 요청 처리 정확도 · case별 strict 성공률의 평균 | 판단 근거 부족 | 낮음 | native capability와 전체 corpus 성공 조건 미확인 [T1](#t1) | 기능 적합성 |
| QA-12 의미 해석 정확도 · strict 성공 run 비율 | 판단 근거 부족 | 낮음 | 추가 evidence의 실제 유효성과 native 정보량 확인 필요 [T1](#t1) | 기능 적합성 |
| QA-13 Task·Interaction 연결 정확도 · strict 성공 run 비율 | 비슷 | 중간 | 최종 Request·Task identity 연결은 공통 [T3](#t3) | 회귀 |
| QA-14 비동기 Task 상태 수렴 · strict 성공 run 비율 | 비슷 | 중간 | Task source event 수렴 규칙은 같은 조건 [T3](#t3) | 회귀 |
| QA-15 대화·Task 연속성 · 성공 scenario 비율 | 비슷 | 중간 | provider epoch가 Conversation identity를 대체하지 않음 [T3](#t3) | 회귀 |
| QA-21 Agent 변화 영향 범위 · 9개 변화의 변경 요소 평균 | 비슷 | 중간 | Agent 변경 집합과 경계는 공통 [T3](#t3) | 회귀 |
| QA-22 Model·Context·State 변화 영향 · 15개 변화의 변경 요소 평균 | 조건부; 방향·크기 미정 | 낮음 | 정규화의 흡수 효과 대 helper 계약 유지, M-07 적합성 우선 [T3](#t3) | 주 비교 가능성 |
| QA-23 실험·로그 변화 영향 · 5개 변화의 변경 요소 평균 | 판단 근거 부족 | 낮음 | 추가 source producer의 E 변경 ledger 필요 [T3](#t3) | 회귀·ledger |
| QA-31 올바른 Task 복구시간 · 최악 fault p95 | 판단 근거 부족 | 낮음 | Task가 영향받은 재연결·복원 경로만 비교 [T3](#t3) | 회귀 |
| QA-32 불필요한 장애 영향 범위 · 초과 중단 단위 최대 수 | 비슷; 격리 효과 미확정 | 중간 | 논리 입력 경계가 Process containment를 만들지는 않음 [T3](#t3) | 회귀 |
| QA-41 PC 메모리 · 최악 workload의 peak p95 | 조건부: helper 상주 시 B 우세 가능 | 중간 | 필수 helper의 실제 local 메모리가 있을 때, 크기는 미정 [T2](#t2) | 주 비교 가능성 |
| QA-51 불필요한 보호정보 노출 · 초과 노출 단위 수 | 비슷 | 중간 | source별 허용 목적·범위를 동일하게 유지 [T3](#t3) | 필수 회귀 |
| QA-61 실행 trace 완전성 · 완전한 trace run 비율 | 비슷 | 중간 | 필요한 source revision을 모두 기록하면 native도 complete 가능 [T3](#t3) | 필수 회귀 |
| QA-62 평가 재현성 · 동일 평가 재계산 비율 | 비슷 | 중간 | evidence와 evaluator 보존은 두 안 모두 가능 [T3](#t3) | 필수 회귀 |

A는 provider 변화를 견디는 입력 계약을, B는 충분한 native 기능을 단순하게 사용하는 구조를 선택할 이유가 있다. 현재 그 provider 적합성이 확인되지 않아 두 정상 후보가 실제로 성립한다고 확정하지 않는다.

## 8. 공정한 검증 계획 — 실행하지 않음

UC-03·04의 모든 하위 유형에 대해 native 제공 정보, 보존 원음·화면, 필요한 변환, 추가 helper 여부를 적은 capability matrix부터 작성한다. 한쪽이 정보적으로 불가능하면 그 profile에서 비교를 멈춘다. 실제 모델과 고정 응답 재생의 의미 정확도 evidence를 구별한다.

측정에 앞서 동일한 목표·fixture·외부 기능·자원 조건, case별 실제 참여 경로, 실패·timeout 처리, 반복·집계·target·동점 기준을 동결한다. 최종 점수나 승리 개수는 지금 만들지 않는다. QA-11과 QA-12~15의 성공을 중복 합산하지 않는다. 잘못된 대상·중복 Action, 무효 승인, 무단 접근은 점수로 상쇄할 수 없는 필수 위반 조건이다.

근거 계약: [Voice responsiveness](../08-quality-attributes/voice-responsiveness.md), [Task 제어](../08-quality-attributes/interaction-control-responsiveness.md), [실제 event 경계](../11-measurement/event-boundary-contract.md), [정확성·연속성](../08-quality-attributes/correctness-and-continuity.md), [복구·장애·메모리](../08-quality-attributes/reliability-and-resource.md), [19개 QA catalog](../08-quality-attributes/quality-model.md), [변경 전체 집합](../07-intentional-variables.md), [변경 요소와 실험 change pack](../08-quality-attributes/evidence/change-locality-rationale.md), [관측·재현](../08-quality-attributes/observability.md), [Privacy·집계](../11-measurement/scoring-contract.md). 문서 사고실험은 실행 evidence label이나 기존 archive 결과로 대신하지 않는다.

## 9. 다른 DP·변경 비용

DP-05는 확보한 Context의 소비 계약, DP-06은 그 근거로 판단하는 권한이다. DP-04의 응답 승인과 DP-13의 물리 제어 자원은 별개다. A→B는 VIA 입력 version과 provider revision 이행, B→A는 정규화 기록·충돌 해소와 helper 수명 이행이 필요하다. 기존 기능이 없는데 새 helper를 측정기에서 무료 제공할 수 없다.

## 10. 현재 판단과 재검토 조건

**선행 기능 적합성 결정으로 유지하고 핵심 점수 비교는 보류한다.** ‘보조 모델을 몇 개 쓰는가’는 하나의 강한 DP가 아니었다. 재정의된 입력 authority도 동일 기능의 native B가 확인되어야 평가 후보로 승격한다. 임의 S2S 제품을 선정하거나 부족한 기능을 실제 제공 사실처럼 쓰지 않았다.

## 11. 자체 검토에서 반영한 개선점

음성 입력·음성 출력·물리 중단을 한꺼번에 바꾸던 결합을 해체했다. Native 기능 부재를 Architecture 점수 열세로 계산하는 오류를 제거했다. 추가 모델은 제품 선택이 아니라 계약을 충족하기 위한 미확인 dependency로 명시했다.

검토 범위는 문서·사고실험이다. 외부 심사나 후보 성능 검증을 완료했다는 뜻이 아니다. 렌더링·정합성 검사와 전체 후보의 최종 분류는 [전체 검토 종합](./dp-review-synthesis.md)에 기록한다.
