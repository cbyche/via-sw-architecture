# 문제 3: 발화 중 바뀌는 화면과 지칭을 어떻게 보존하고 결합할 것인가

2026-10-16 / 다섯 문제 공동 조사 / 복수 구조 탐색 / 구현과 측정 없음

[공동 조사 기준](./00-research-contract.md) / [코드와 논문 근거](../../../../../references/five-problem-study/screen-speech-grounding.md) / [기존 DP-47](../../04-47-screen-voice-grounding.md)

## 1. 작성자 결론

**이 문제는 VIA의 핵심 DP 후보로 계속 발전시킬 가치가 있다.** 사용자는 화면을 멈추거나 정확한 객체 이름을 말하지 않고도 업무를 이어야 한다. 발화가 끝난 뒤 현재 화면을 잘 읽는 것만으로는 당시 사용자가 가리킨 대상을 복구할 수 없다. VIA가 직접 당시 근거와 수정 관계를 소유해야 다른 Agent에 올바른 자료를 넘길 수 있다.

조사 후 남긴 주요 대안은 다음 두 개다. 기존 47의 A/B와 같은 의미 생산 축이며, 새 번호나 별개의 DP로 중복 계산하지 않는다.

| 대안 | 현재 지칭을 생산하는 방식 | 정상 사용에서 선택할 이유 | 치르는 비용 |
| --- | --- | --- | --- |
| **A 원본 공동 해석** | 하나의 지칭 해석 책임이 필요한 원음 / 전사, 당시 화면과 동작을 함께 읽고 직접 대상 연결을 생산 | 말이 있어야 화면 범위나 동작의 의미가 정해지는 장면에서 중간 후보의 재생산 왕복을 줄일 수 있음 | 공동 입력의 범위가 크면 입력 비용과 재판단 부담 증가 |
| **B 생산자 소유 후보의 교환과 결합** | 발화의 부분 의미와 화면 대상 후보를 별도 생산자가 관리하고, 결합자가 후보를 연결. 부족하면 원 생산자가 원본에서 확장 | 같은 화면을 여러 지칭과 정정에서 재사용하고, 생산 / 결합의 자료 계약을 따로 관리할 수 있음 | 후보 누락, 호환 revision 관리와 producer → 결합 → 재생산의 의존 비용 |

추가 조사한 **증분 가설 갱신**은 의미 있는 실행 선택이다. 다만 A와 B 모두 사용할 수 있으므로 세 번째 경쟁 구조로 세우지 않았다. 원본 보존, 시각 정렬과 유한 대기도 두 안의 공통 기반이다. 별도 helper, cache, crop이나 early start를 B만의 이익으로 삼지 않는다.

**현재 강한 차이 가설은 QA01 / QA03의 판단 의존 경로와 QA05의 전체 모델 처리량이다. QA02는 직접 관련되지만 우위가 미확인이다. QA04와 QA06은 비교할 영향 경로가 있으나 자동 차별화 축으로 세지 않는다.** 기능의 중요성만으로 현재 대안이 과정의 3~4개 QA 차별화 요건을 충족했다고 판정하지 않는다.

## 2. VIA의 기능 문제와 같은 완료 조건

주 관련 UC는 [UC-03, UC-04, UC-05, UC-06, UC-09, UC-10, UC-11](../../../../05-representative-use-cases.md)이다. 입력 채널 재연결, 접근 철회와 실패는 UC-15, UC-16, UC-18과 연결한다.

대표 사용 장면은 “이 표는 그대로 두고, 이 두 그래프의 차이를 설명해줘”라고 말하며 표를 가리키고 다음 페이지로 이동해 두 그래프를 둘러싼 뒤, “아니 아래 것만”이라고 고치는 경우다. 필요한 결과는 현재 화면의 object 목록이 아니라 **각 표현이 당시 어느 대상과 범위를 뜻했고, 무엇을 유지하고 바꾸었는지**다. 문서 수정이라는 외부 Action이 필요하면 이 해석 뒤 Downstream Agent에 위임한다.

### 2.1 대안과 무관한 완료 조건

1. 발화 시작 이후 실제 취득한 원음 / 전사, 화면, pointer / selection과 source별 관측 범위를 사용한다. 발화 전에 만든 selection이 첫 조회 시 여전히 유효하면 해당 초기 상태를 포함한다. 사전에 허용해 보존한 pre-roll이 실제 있으면 양안에 같은 범위를 제공한다. 상시 수집을 새로 요구하거나 관측하지 못한 과거 화면을 생성하지 않는다.
2. 발화 표현마다 단일 대상, 집합 또는 범위를 연결한다. 채택 대상은 native 참조가 실제 있으면 그 참조와 revision, 없으면 당시 이미지의 시각적 범위로 표현한다.
3. 원형 동작, page 이동, 우연한 pointer 통과를 구별할 근거를 남긴다. 시간상 가깝거나 pointer가 지나갔다는 이유만으로 지칭이라고 단정하지 않는다.
4. 부분 정정은 유지할 대상과 바꿀 대상을 구별한다. 수정 범위 자체가 불명확하면 넓혀 판단하고, 근거가 부족하면 필요한 부분을 확인한다.
5. 현재 input revision과 사용권에 유효한 제안만 채택한다. 옛 제안은 진단 기록으로 보존할 수 있지만 현 요청의 정답으로 되살리지 않는다.
6. 모든 안은 동일 모델 capability와 원본, capture 한도, cache, crop, native 조회, 병렬 읽기와 부분 patch를 사용할 수 있다. 쉬운 사례에서 불필요한 모델 단계는 강제하지 않는다.

정확한 단어 시각, 모든 앱의 stable object ID와 native selection은 공통으로 보장하지 않는다. 공급된 시간 구간과 오차를 보존한다. cloud가 원음과 image의 공동 입력을 지원하지 않으면 양쪽 모두 전사와 알려진 구간을 사용하며 해당 지원 한계를 기록한다. 이 조건을 해결하기 위해 숨은 local 모델을 추가하지 않는다.

### 2.2 다른 탐색 문제와의 경계

| 문제 | 이 비교와 연결되는 부분 | 별도로 남기는 결정 |
| --- | --- | --- |
| 1 현재 요청의 근거 공급 | 필요한 과거 원본과 채택된 대상의 조회 | 대화 전체 문맥 유지 / 요청별 retrieval 전략 |
| 2 겹치는 입력과 사건 | 정정 도착, 늦은 모델 결과와 input revision | 여러 Request / Agent 사건의 스케줄링과 전역 실행 조정 |
| 4 업무 기억과 인계 | 채택 target, 당시 근거와 정정 결과를 인계 자료로 보존 | 여러 Agent 결과의 장기 기억과 인계 문맥 생산 |
| 5 Agent 선택과 전환 | 해석한 대상과 필요한 자료 접근 조건을 선택 입력으로 제공 | Agent 적합성 판단과 실행 전환 |

이 문서의 원본 buffer는 현재 발화와 후속 정정의 근거다. 모든 화면을 계속 해석해 전역 기억을 게시하는 구조로 넓히지 않는다. 기존 제외된 독립 recorder / ownership 비교를 복원하지 않는다.

## 3. reference를 읽고 남긴 것

[별도 source 원장](../../../../../references/five-problem-study/screen-speech-grounding.md)에 고정 commit, 실제 파일과 읽은 범위, VIA 적용과 한계를 기록했다.

| 확인 자료 | 구조 판단에 사용한 사실 | 이번 탐색에서의 역할 |
| --- | --- | --- |
| Magentic-One WebSurfer, OpenHands BrowserObservation | native target 또는 text와 screenshot을 함께 모델 입력으로 구성. image는 선택적으로 포함 / 저장 가능 | A의 실용적 입력 형태와 양안의 native + visual 공통 보완 |
| OpenClaw browser snapshot | ref 게시 중 document identity와 frame 현재성을 검사 | 과거 대상 근거와 현재 Action selector를 구별하는 공통 계약 |
| Google Live Console | audio와 screenshot frame이 같은 live client에 별도 수집 경로로 들어감 | A가 한 번의 전체 batch에 한정되지 않는 실제 선례 |
| QuickSet 원 논문, OmniParser server | 복수 부분 의미를 결합하는 실제 시스템, image → 요소 목록의 실제 API | B의 후보 생산과 결합 구조. 둘을 동일 구현이라고 주장하지 않음 |
| Psi Fuse | primary 원시 시각에 맞춰 secondary를 읽고 자료 부족 시 대기 | 자료 도착과 실제 시각의 분리, 유한 대기의 설계 근거 |
| InproTKS 원 논문, ReTiCo core | 수정 가능한 incremental unit과 add / revoke / commit, 원본 의존과 update queue | A/B 공통으로 선택할 수 있는 발화 중 잠정 판단과 철회 계약 |

Deep Agents와 CrewAI는 공동 조사에 포함되지만 문제 3의 시간 지칭 구현을 직접 입증하는 새 근거로 사용하지 않았다. 단순히 여러 agent를 지원한다는 이유로 B의 multimodal binding 근거로 세지 않는다.

### 3.1 후보 탐색에서 제외하거나 별도로 남긴 방향

| 방향 | 판정과 이유 |
| --- | --- |
| 전사 도착 때 마지막 screenshot만 읽음 | EX08에서 사라진 근거를 잃는다. 기능 제한이 있는 단순 구현이며 강한 주 대안으로 남기지 않음 |
| native 객체를 우선 쓰고 부족하면 image 판단 | 양안의 값싼 공통 보완. visual 대상과 집합까지 없애지 않는 한 별도 주요 구조가 아님 |
| screen 1위 후보를 확정한 뒤 음성을 붙임 | 언어가 화면 범위를 바꿔야 하는 장면에 약함. B는 복수 후보와 원본 재생산까지 포함 |
| 실제 선택 또는 overlay 확인을 항상 요구 | 추가 사용자 동작을 요구하는 기능 절충. 모호한 경우에는 양안의 확인 수단이며 주요 경쟁 구조로 승격하지 않음 |
| source별 worker / 별도 process | 배치만으로 지칭의 의미 생산이나 정확도가 바뀌지 않음. 장애 / 자원 문제를 별도로 입증해야 함 |
| live model 세션만 믿고 local 원본을 버림 | 이후 정정, 원본 provenance와 사용권 통제에 필요한 자료가 사라짐. 현재 공통 완료 조건을 충족할 근거 미확보 |
| 모든 순간의 의미를 미리 생산 | 정상 사용 비용과 상시 수집 범위를 새로 요구. 현재 발화 중 필요한 증분 판단으로 제한하면 A/B 실행 축 |

## 4. 공통 데이터와 수명

다음 이름은 이번 비교의 설명용 데이터 계약이다. target Component 이름은 그대로 쓰며, 내부 Module과 별도 process를 동일시하지 않는다.

| 자료 | 작성 / 보존 책임 | 수명과 실제 의미 |
| --- | --- | --- |
| `ObservationWindow` | Interaction Manager가 capture하고 bounded buffer에 보존 | 발화 구간의 audio / frame / pointer, 첫 조회에서 유효한 실제 selection과 허용된 기존 pre-roll의 참조, source 시각과 도착 시각, 오차와 gap. 단어 의미나 정답 target을 미리 넣지 않음 |
| `GroundingInput` | Request Controller가 Request Interpreter에 전달 | input revision, 원본 조회 권한, 이전 채택 binding과 예산. 원본 참조를 가지며 모든 media의 복사본이 아님 |
| `GroundingProposal` | Request Interpreter의 A 또는 B가 생산 | 표현별 target 집합, extent / source revision, 근거, 불확실성과 유지 / 교체 제안. 사용자 표시나 Task 실행 허가가 아님 |
| 채택 binding | Request Controller가 검증하고 상태 계약에 따라 보존 | 최종 채택 revision과 근거를 후속 대화 / 인계에 연결. 허용된 당시 snapshot은 최신 문서를 클릭할 stable selector와 다름 |
| 원본 사용권과 만료 | Policy Manager와 Context Manager의 공통 계약 | 철회 / 만료 시 원본뿐 아니라 파생 후보와 전달 사본의 재사용도 막음. 미보존 원본을 현재 화면으로 대체하지 않음 |

원본 buffer를 RAM에 오래 둔다고 모든 frame을 모델에 전송하는 것은 아니다. 디스크에 내리는 것도 무한 보존이나 512 MB 준수의 증명이 아니다. local staging, queue, decoded image, candidate와 serialization 사본을 전체 RAM에 포함한다.

## 5. A: 원본 공동 해석

Request Interpreter 안의 **Joint Grounding Interpreter**가 현재 지칭을 직접 생산한다. 후보와 source helper를 사용할 수 있지만, 별도 생산자가 만든 target만 최종 결과에 사용할 의무는 없다.

### 5.1 정상 실행과 반환

1. Request Controller → Request Interpreter: 현재 `GroundingInput`을 전달한다. Task association은 아직 정답으로 주어지지 않는다.
2. Joint Grounding Interpreter → 공통 evidence read: 필요한 발화 / frame / 동작 범위를 요청한다. 반환은 실제 관측 범위와 gap이다. native selection과 pointer 근처 crop을 먼저 볼 수 있고 필요하면 주변 원본으로 넓힌다.
3. Joint Grounding Interpreter → Model Access → 공통 cloud 의미 모델: 필요한 원본, 시간 불확실성, 이전 binding과 정정 원문을 함께 제공한다. 반환은 새 target 연결, 수정 제안 또는 추가 근거 요구다.
4. Joint Grounding Interpreter는 요청별 `Joint Grounding State`에 원본 dependency, 잠정 binding과 revision을 유지한다. 원본 추가 조회 / 부분 patch를 수행할 수 있다.
5. Request Interpreter → Request Controller: `GroundingProposal`을 반환한다. Controller가 revision / 권한을 검사한 뒤 전체 요청 해석과 응답 / 위임에 사용한다.

### 5.2 정정과 실패

“표는 유지하고 아래 그래프만”이라는 정정이면 이전 binding 중 유지와 교체 대상을 함께 검토하고 영향받은 부분만 다시 읽을 수 있다. 정정 범위가 이전 판단 전체에 영향을 주면 원본 범위를 넓힌다. source gap, cloud 실패 또는 철회가 있으면 해당 binding의 미해결 이유를 반환한다. 취소 후 늦게 온 판단은 현재 state에 조건부 게시하지 않는다.

**A가 유리한 정상 조건:** 언어와 화면이 강하게 상호 의존하고 화면 대상의 경계가 매 요청마다 달라질 때, 중간 후보를 생산했다가 다시 확장하는 경로가 짧아질 수 있다. 명시적 native 선택이 충분한 쉬운 경우에도 작은 입력으로 끝낼 수 있다.

**A의 비용:** 필요한 원본 범위가 넓거나 같은 visual 관계를 반복 판단하면 media 입력과 재판단이 커질 수 있다. 다만 A가 cache와 targeted patch를 잘 사용하면 B의 기대 이익이 사라질 수 있다. 원본의 공동 사용 자체가 정확도를 보장하지 않는다.

## 6. B: 생산자 소유 후보의 교환과 결합

Request Interpreter 안에서 **Speech Reference Producer**는 표현과 수정 관계 후보를, **Screen Target Producer**는 화면 대상과 동작 후보를 생산한다. **Multimodal Binding Resolver**는 이를 결합한다. **Grounding Coordinator**는 실행과 version을 관리하며 후보의 의미를 대신 채우지 않는다.

### 6.1 정상 실행과 반환

1. Request Controller → Grounding Coordinator: A와 같은 `GroundingInput`을 전달한다.
2. Coordinator → 두 producer: 원음 / 전사와 화면 / 동작의 원본 참조를 각각 준다. 병행할 수 있으나 같은 모델 / worker 예산을 따른다. native 자료만으로 표현 가능한 후보에 모델 호출을 강제하지 않는다.
3. 각 producer → 자기 `Candidate Revisions`: 원본과 coverage를 가진 부분 후보를 게시한다. speech는 “두 개”, “아래 것만”의 수 / 관계 / 수정 범위 후보를, screen은 native 또는 visual 범위와 navigation / designation의 경쟁 해석을 남긴다. 작성한 revision을 Coordinator에 알린다.
4. Coordinator → Resolver: 같은 input과 policy에 속하는 후보 read-set, 관련 excerpt, 이전 binding을 전달한다. Resolver가 Model Access를 통해 필요한 결합을 판단한다.
5. Resolver → Coordinator: 제안 또는 `RejudgeRequest`를 반환한다. “한 figure가 아니라 내부 두 graph가 필요”하면 원 Screen Target Producer가 원본을 다시 읽어 새 범위를 생산한다. Resolver가 source에서 없는 target을 직접 만들어 채택하지 않는다.
6. Resolver → Request Controller: producer version에 의존하는 `GroundingProposal`을 반환한다. 공통 최신성과 사용권 검사 뒤 채택한다.

### 6.2 상태, 정정과 실패

| 상태 | owner와 수정 책임 | 수명 / 무효화 |
| --- | --- | --- |
| 표현 후보 | Speech Reference Producer | 현재 요청과 정정 중 재사용. transcript / 의미 revision이 바뀌면 영향 범위 재판단 |
| 대상 후보 | Screen Target Producer | source extent / revision과 coverage를 참조. 화면이 바뀌어도 당시 원본의 후보로 남을 수 있으나 현재 객체라고 재사용하지 않음 |
| 후보 결합 | Multimodal Binding Resolver | 사용한 producer read-set을 명시. 관련 revision 교체 시 재결합 |
| job / 예산 / 반환 상태 | Grounding Coordinator | 요청별 bounded 상태. 영속 Task 상태와 구별. 추가 왕복 한도, 취소와 늦은 완료 처리 |

EX09에서 표 집합만 바뀌면 관련 표현 / target과 결합을 재판단하고 문단 P2의 유효 근거는 재사용할 수 있다. dependency가 의미상 완전하다는 보장은 없으므로 수정 범위가 불명확하면 함께 판단한다. 후보 생산 실패를 “화면에 대상 없음”으로 바꾸지 않는다. 원본을 확대해도 후보가 없거나 서로 모순되면 확인 질문을 제안한다.

**B가 유리한 정상 조건:** 같은 화면에서 여러 지칭이 생기고 변화가 적은 target을 반복 수정하는 경우, target 생산 비용을 여러 결합에 나눠 쓸 수 있다. 구조화된 native 범위가 충분한 경우에는 작은 후보를 결합하는 경로도 실용적이다.

**B의 비용:** producer 출력과 Resolver 입력, startup과 추가 왕복을 모두 쓴다. 후보 누락과 잘못된 범위는 원본 재생산을 거쳐야 한다. 작은 최종 후보를 받는다는 이유로 producer의 image 판단 비용이나 원본 RAM을 제외하지 않는다. A의 cache / 부분 patch가 같은 재사용을 달성하면 B의 비용 이익은 작아진다.

## 7. 조합 가능한 추가 축: 완성 입력 중심과 증분 가설 중심

InproTKS / ReTiCo 조사에서 의미 있는 추가 선택은 **입력이 누적되는 동안 어떤 잠정 의미 상태를 유지할 것인가**다. A/B의 생산 권한과 별개로 다음 두 실행 방식이 가능하다.

| 실행 방식 | 실제 상태와 의존 | 이익과 비용 |
| --- | --- | --- |
| 완성 입력 중심 | 원본은 계속 수집하고 저비용 read / cache를 준비. 완성 입력의 revision을 기준으로 주 판단을 실행 | 짧은 발화와 잦은 ASR 변경에서 폐기 판단을 줄일 수 있음. 주 판단이 뒤에 남으면 첫 응답 대기가 길어짐 |
| 증분 가설 중심 | 의미 있는 입력 변화마다 잠정 binding 또는 후보를 갱신하고 input dependency에 따라 revoke / replace. 채택은 현재 revision에서 별도 수행 | 긴 안정 발화에서 작업 일부를 먼저 끝낼 수 있음. 반복 모델 호출, 폐기 결과와 오래 살아 있는 상태가 증가 |

실행 횟수 제한과 coalescing을 양쪽에 허용한다. “ASR partial 하나당 모델 호출”을 필수로 하지 않는다. 증분 값이 안정됐다는 provider 표시는 사용자의 최종 의도 확정과 같지 않다. Inpro의 commit은 해당 처리자의 가설 확정 표시다. VIA의 영속 채택이나 이후 사용자 정정 금지를 뜻하지 않는다.

실제 비교는 `A + 완성 입력`, `A + 증분`, `B + 완성 입력`, `B + 증분`의 조합이 가능하다. 먼저 의미 생산 A/B를 같은 실행 조건에서 비교하고, 증분 실행의 손익은 같은 A 또는 같은 B에서 따로 검토해야 원인을 분리할 수 있다. 기존 A/B 모두 early start를 허용했으므로 이것을 새 C의 독점적 기능으로 포장하지 않는다.

시각 자료를 더 기다릴지의 선택도 이와 구별한다. Psi의 자료 부족 대기는 결합할 수 있는 관측이 준비됐는지의 조건이다. timestamp로 이미 사용자의 뜻을 알았다는 조건이 아니다. 어느 안도 모든 source의 무제한 완료를 기다리지 않으며, deadline과 source gap을 함께 반환한다.

## 8. 같은 여러 장면을 따라간 비교

아래는 [공동 EX 목록](./00-research-contract.md#3-공동-사용-장면의-출발-목록)을 구체화한 사고실험이다. 기존 COMMON-28 / DP-RELATED 모집단을 바꾸거나 측정 PASS를 선언하지 않는다. `EX03-G1` 등은 이 문서의 변형명이다.

| 장면과 같은 정답 조건 | 강한 A의 처리 | 강한 B의 처리 | 남는 손익과 검사 |
| --- | --- | --- | --- |
| **EX02 명시 선택:** native 문단 범위가 노출되고 “이 부분을 요약해줘” | 실제 selection과 짧은 발화로 binding을 판단. 불필요한 전체 image 읽기 생략 | native target 후보를 저비용 구성하고 speech와 결합. visual producer 모델을 생략 가능 | 동점 가능. B의 항상 3회 호출, A의 항상 전체 frame 가정을 금지 |
| **EX08 이동 지칭:** 차트 A를 가리킨 뒤 B, 끝에는 빈 곳. 화면 전환 후 final 전사 도착 | 당시 두 frame / 경로와 발화를 함께 읽어 두 역할을 연결 | 당시 target 후보와 시간 범위 / 순서 후보를 교환해 두 역할을 결합 | 양안 원본 수집이 우선. 마지막 화면만 보는 실패는 A/B 차이의 근거가 아님 |
| **EX08-G1 원형 집합과 navigation:** 두 graph 주위를 click 없이 돌고 page drag, 다시 “이 둘” | circle, button 상태와 page 변화를 말과 공동 판단 | screen이 navigation / designation과 one-of / set 후보를 남기고 “둘” 정보로 확장 | B의 최초 후보 부족 시 추가 왕복. A도 circle 오해 가능. B 정확도 손해를 기정사실화하지 않음 |
| **EX08-G2 pixels만 있는 figure:** UIA에는 PDF container만. “오른쪽 두 그래프” | 원본 visual extent를 직접 target으로 생산 | screen 생산자가 두 visual target을 생산하고 Resolver가 참조. figure 하나뿐이면 분할 재생산 | 후보 vocabulary 밖 대상을 복구하는 실질 비용. 양쪽 같은 crop / 전체 원본 권한 |
| **EX09 부분 정정:** T1/T2 보존과 P2 수정 뒤 “표는 첫째와 이 표로 바꿔. 문단은 유지”, T3 지칭 | 이전 binding을 받아 T2 → T3 patch, P2 유지 검토 | speech의 수정 관계와 T3 target을 갱신하고 표 binding 재결합. P2 관련 후보 재사용 | 재사용은 양안 가능. B가 실제로 덜 읽고 덜 호출해야 비용 이익이 성립 |
| **EX09-G1 반복 / cache hit:** 같은 source revision에서 바로 “그 그래프의 제목도” | 보존 target과 crop / 모델 cache를 활용 | 보존 target 후보와 새 speech를 결합 | A의 저비용 부분 판단으로 B의 이익이 사라질 수 있음. 반복 입력을 제외하지 않음 |
| **EX10-G1 모호함:** 기존 selection P1과 pointer P2가 충돌하고 말은 “이 부분” | 두 근거를 보존해 대상 확인 제안 | competing target과 부족한 언어 제약을 결합해 확인 제안 | 모호한 지정 입력은 QA02 분모에 넣지 않는 기존 규칙. 질문은 기능 동작이지 정확도 점수의 공짜 성공이 아님 |
| **EX10-G2 관측 공백:** page 전환 중 필요한 frame 누락, 단어 시각도 없음 | 허용된 추가 원본이 있으면 읽고, 없으면 미해결 / 재지정 | 후보 coverage를 공백으로 남기고 추가 생산. 원본 없으면 같은 미해결 | 정확한 타임스탬프나 대상을 만들어내지 않음. 확인 가능한 대조 장면에서는 불필요한 질문을 실패로 남김 |
| **EX03-G1 실제 전달 후 지칭:** text A/B/C, audio A만 전달 후 “들려준 부분” | 전달 이력을 Context Manager에서 받아 A 연결 | 전달 이력으로 표현 / target 후보를 만들고 A 연결 | 화면 grounding 자체보다 문제 1의 자료 공급. 같은 효과를 두 DP의 독립 차이로 세지 않음 |
| **EX16-G1 철회:** 같은 화면 crop 사용을 다음 판단 전에 철회 | 사용권 gate에서 미전송 / 재사용 차단, 관련 binding 재검토 | 원본과 파생 후보, in-flight read-set까지 차단 | B metadata가 늘 수 있지만 A도 사본과 cache를 관리. 안전 기능을 어느 한 안에서 생략하지 않음 |
| **EX17-G1 재접속:** 임시 상태를 잃었지만 허용 원본과 채택 binding은 남음 | 원본에서 해당 grounding 상태 재구성 | 원본에서 필요한 producer 후보 재생산 후 결합 | B의 추가 재생산 비용 가능. 두 안의 process 배치와 원본 내구성을 동일하게 둠 |

### 8.1 EX08의 인과 순서를 더 구체적으로 보면

1. t0에 발화와 관측이 시작된다. chart A 시점, chart B 시점, 마지막 빈 곳의 관측을 각각 보존한다. 전사는 늦게 도착할 수 있다.
2. A는 필요한 관측 범위와 전사 / 원음을 읽어 표현별 직접 binding을 생산한다. B는 표현 / 대상 후보를 생산한 다음 같은 input의 후보를 결합한다.
3. B의 대상 후보가 둘을 한 figure로만 만들었다면 Resolver가 원 Screen Target Producer에 두 범위 생산을 요청하고 새 revision으로 재결합한다. A가 같은 오류를 범하면 공동 해석 책임에서 원본을 다시 읽고 수정한다. 어느 쪽도 한 번에 항상 맞는다고 가정하지 않는다.
4. Request Controller는 현재 제안만 채택한다. 채택한 두 대상을 구체적으로 밝히는 첫 내용이 표시되면 QA01이 끝날 수 있다. 이후 실제 비교 내용을 위한 상세 읽기는 QA03에 남는다.
5. 배경 Agent 완료가 겹쳐도 grounding 원본을 그 Task로 바꾸지 않는다. 처리 우선순위는 문제 2의 결정이고 이 문서에서 B의 정확도 장점으로 세지 않는다.

## 9. QA별 인과와 뒤집히는 조건

기준은 [공동 계약](./00-research-contract.md#2-공통-기능-및-품질-계약)과 [현재 QA 후보](../../qa-candidate-package/README.md)다. 예상 수치, 점수와 승자를 만들지 않는다.

| QA | 구조 차이 → 실제 처리 / 상태 → 품질 가설 | 방향을 바꾸거나 차이를 없애는 조건 | 현재 판정 |
| --- | --- | --- | --- |
| **QA01 첫 응답** | A 직접 binding과 B producer → 결합 / 재생산의 의존이 첫 내용 전에 필요하면 A의 왕복 감소 가능. B 후보가 이미 준비되고 입력이 작으면 반대로 감소 가능 | 명시 선택의 저비용 경로, 같은 cache hit, 양안 early start, 상세 비교 전 처리 의도 표시가 가능하면 차이 축소 | 차이 가설 있음. 어느 안의 우위도 미확인 |
| **QA02 대상 집합 정확도** | A는 원본의 공동 단서를 직접 쓰고, B는 경쟁 후보 / unknown과 원본 확장 계약을 통해 판단. 표현별 채택 target 집합이 달라질 수 있음 | B의 강한 확장과 A의 부분 재판단 후 같아지면 정확도 차이가 없음. 후보 개수나 시간 정렬 자체는 정답 증거가 아님 | 직접 관련. 중요한 미확인 축 |
| **QA03 VIA 처리시간** | 후보 생산, 재결합과 공동 media 판단의 실제 critical path 차이. 상세 내용과 음성 요약 준비까지 포함 | 분리 호출의 overhead보다 작은 입력 효과가 크거나, 동시 호출 제한이 있으면 방향 변화. 외부 문서 수정 시간 제외 | 차이 가설 있음. QA01과 합산하지 않음 |
| **QA04 실제 Rust 모듈 변경 범위** | C03 source 좌표 / 시각 / version 변경, C05 근거 표시, C06 철회가 data / producer / binding에 전파될 수 있음 | 공통 read adapter로 wire 변경이 흡수되면 동점. B의 여러 계약이 오히려 변경을 늘릴 수도 있음 | 영향 경로만 확보. 상자 수로 변경 모듈 수 예측 금지 |
| **QA05 전체 VIA 모델 비용** | A의 공동 입력 / 재판단과 B의 producer 출력 / 결합 입력 / 재생산 비용. 증분 처리의 미사용 생산과 폐기까지 포함 | source 반복이 많고 후보가 안정적이면 B 재사용 이익 가능. A cache / patch 또는 빈번한 후보 확장에서는 동점이나 역전 | 차이 가설 있음. COMMON-28 전체 집계 단위 보존 |
| **QA06 최대 RAM** | 같은 raw buffer 위에 A 공동 입력 staging, B 후보 / 교환 상태와 동시 worker 사본이 추가 | compact reference, 순차 staging, image decode 수를 제한하면 차이 축소. B도 raw를 버릴 수 없음 | 512 MB 공통 요구. 준수 여부와 의미 있는 차이는 미확인 |

이 문제에는 별도 신규 핵심 QA가 우선 필요하지 않다. 지칭의 정답은 이미 QA02가 다룬다. source coverage, 후보 miss, 재생산 횟수와 잠정 가설 철회량은 원인 분석용 관측으로 남기고 핵심 QA를 늘리는 데 쓰지 않는다. Downstream Agent 선택 정확도는 문제 5의 평가이며, 이 문서의 원본 지칭 오류를 routing 성능으로 바꾸어 중복 집계하지 않는다.

기능 완전성 / 적합성, 자원, 사용 편의, 복구, 접근 통제, privacy, 설명 가능성과 증거 보존은 공통 완료 조건과 EX10/16/17에 남긴다. 기존 13개 품질 관점의 [47 검토](../../04-47-screen-voice-grounding.md)도 보존한다. QA07~09의 우선도를 낮춘 것이 입력 수신, 실제 전달과 관련 회귀 기능을 삭제하는 뜻은 아니다.

## 10. 가장 싼 보완과 혼합으로 반박

| 시도 | 허용하는 강한 보완 | 차이가 남거나 사라지는 판단 |
| --- | --- | --- |
| A에 screen helper와 cache 추가 | native / image 후보를 보조 자료로 받고 직접 원본 판단도 유지 | B의 대부분 이익을 얻으면 B의 별도 owner와 교환 계약을 주요 DP로 둘 근거 약화 |
| A에 dependency와 부분 patch 추가 | 이미 주안에 포함. 수정 범위를 좁히고 원본 / media cache 재사용 | B만 정정과 재사용이 가능하다는 주장은 기각 |
| B가 native와 안정 후보를 빠르게 처리 | 모델 없는 native 자료 준비, 호출 batching / 병행과 동일 budget | 후보 소유와 재생산 관문은 남지만 실제 호출 overhead가 줄어 A의 지연 이익이 없어질 수 있음 |
| B의 Resolver가 원본에서 직접 target 생성 | 구현 가능하나 A형 직접 경로를 추가한 **명시적 hybrid** | 우회 성공과 비용을 pure B의 값으로 세지 않음. 우회가 주 경로가 되면 A+helper에 가깝게 재분류 |
| 요청 종류마다 A/B 선택 | 공통 proposal와 원본 read를 유지한 선택 가능 | routing 비용과 두 엔진 유지 비용을 기록. 독립적으로 우수한 안을 두 개 가진다는 사실만으로 추가 주요 DP를 만들지 않음 |
| 요청 종료 후 엔진 교체 | 임시 candidate migration을 피하고 주변 Component / 영속 binding 유지 | 제품 전체 재설계 비용을 주장하지 않음. 핵심 변경은 후보 identity / coverage / producer revision과 재생산 협력 |

**개발 비용**은 B의 producer 계약과 재판단 경로, A의 공동 입력 / 부분 patch에 각각 남는다. **실행 중 migration 비용**은 요청 종료 시 교체하면 낮아질 수 있다. **근본적 차이**는 최종 지칭자가 원본에서 새 target을 직접 생산할 권한과 별도 producer를 반드시 거칠 의존이다. 이 차이가 코드 몇 개와 helper만으로 값싸게 오갈 수 있고 QA 차이도 작으면 주요 DP 대신 상세 설계로 내려야 한다.

## 11. DP 후보 권고와 반증 조건

**권고:** 문제 3을 기존 DP-47의 심화 후보로 유지한다. A/B를 강하게 보존하고 증분 처리와 시간 대기를 조합 축으로 분리한다. 현재 조사로 세 번째 독립 의미 생산 구조를 추가하지 않는다. reference를 충분히 읽었을 때 두 개만 남는 것도 탐색 결과다.

우선 반증해야 할 주장은 다음과 같다.

1. A의 bounded read / cache / patch가 동일한 정확도와 재사용을 더 적은 처리로 달성하면 B의 정상 사용 이익은 약해진다.
2. B가 원본 확장 없이 후보를 충분히 만들고 같은 화면을 반복 재사용하여 A보다 실제 입력량과 처리 경로를 줄이면 A의 적은 단계만으로 지연 / 비용 우위를 주장할 수 없다.
3. 양안이 같은 API 한계 때문에 중요한 EX08 / EX10을 해결하지 못하면 구조의 우열로 숨기지 않고 공통 capability 제약을 먼저 드러낸다.
4. 차이가 QA01/03의 같은 지연 현상과 미확인 정확도 주장뿐이고 QA05도 미미하면 요구한 핵심 DP 구성이 충족됐다고 말하지 않는다. QA를 새로 만들어 빈칸을 채우지 않는다.
5. 증분 실행에서 버린 판단과 queue / RAM이 늘어도 첫 내용 표시가 당겨지지 않으면 해당 실행 방식의 선택 이유가 약해진다. partial 입력을 받는다는 기능만으로 이익을 선언하지 않는다.

후속 검증이 허용되면 같은 원본을 사용한 명시 선택, 이동 지칭, visual 후보 누락, 반복 / 부분 정정, 모호함 / gap을 함께 비교해야 한다. 이 문서는 시나리오와 관측 이유만 제안하며 모델 실행, PoC 코드, frozen 모집단이나 새로운 점수를 만들지 않았다.

## 12. 이번에 확인한 범위

- 직접 읽은 공개 코드: Magentic-One, OpenHands, OpenClaw, Google Live Console, OmniParser, Psi와 ReTiCo의 특정 고정 commit 경로.
- 직접 읽은 원 논문: QuickSet §3과 InproTKS §2–5. 실제 시스템의 보고와 현재 실행 가능성을 구별했다.
- 보존한 기존 설계: 47의 세 설계 포인트, 강한 A/B, 기존 그림과 16개 사고실험. 이 문서가 기존 47의 본문이나 승인 상태를 바꾸지 않는다.
- 미확인: 실제 Windows 앱의 source 노출, cloud media / timestamp capability, 한국어 지칭과 집합의 성공, 호출 / queue / local RAM과 모델 비용. 공개 source가 이 결과를 대신하지 않는다.
