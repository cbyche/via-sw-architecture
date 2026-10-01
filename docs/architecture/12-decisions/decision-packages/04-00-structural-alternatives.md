# 4단계 — 중요한 품질 차이를 만드는 SW 구조 탐색

> 상태: **STAGE_4_REVIEW_READY / 독립 검토 및 수정 후 사용자 리뷰 대기** / 2026-10-01
> 입력: [02 요구와 수단](./02-00-requirements-and-choices.md), [03 품질 시나리오](./03-00-quality-scenarios.md). 절차: [작업 계획](./00-workplan.md).
> 검토 및 검사: [04-01 독립 검토 기록](./04-01-structural-review.md).
> Target은 REVIEWED_BASELINE으로 보존한다. 여기의 S 번호는 탐색 질문을 가리킨다. DP 번호, 선정 결과 또는 방안 2의 확정이 아니다.

## 1. 이번 단계에서 한 일과 읽는 순서

**품질 우선순위에 따라 탐색 노력을 다르게 배분하고, 같은 문제를 푸는 실제 실행 구조들을 펼쳐 보았다.** 먼저 §2에서 P별 탐색 깊이를, §3에서 구조 차이의 전체 모습을 볼 수 있다. §4는 집중 질문별 실제 target, 대안의 정상 흐름과 상태, 실패와 복구, 품질 인과다. §5는 연계 문제와 덜 탐색한 영역, §6은 조합 관계와 다음 검토 질문이다.

V-01 기능 정확성, V-02 기능 적절성, V-03 기능 완전성을 공동 1순위로, V-04 상호작용 반응성과 V-05 VIA 귀속 요청 완료 시간을 공동 2순위로 본다. 이어 V-08 변경 용이성과 모듈성, V-06 자원 활용성과 수용량, V-07 결함 허용성과 복구성 순이다. V-09~13은 공동 6순위다. 연결된 V 개수로 P의 중요도를 계산하지 않는다.

### 1.1 모든 스케치에 적용한 비교 규칙

- **T는 실제 target이다.** 대안 A/B의 신규 실행체 이름은 설명을 위한 제안이며 target의 Component 이름이 아니다. T의 기능을 빼거나 target에 없는 추가 처리를 넣어 대안을 유리하게 만들지 않는다.
- **지원 목표와 확인 상태를 나눈다.** 아래의 ‘지원 목표’는 설계 의도이며 구현 성공을 뜻하지 않는다. U는 [02 §5](./02-00-requirements-and-choices.md#5-지원이-필요하다는-것과-이미-확보했다는-것은-다르다)의 미검증 지원이다. target에도 같은 구별을 적용한다.
- **일부 기능을 포기한 안도 남긴다.** 승인된 02의 요구 목록은 비교 기준으로 보존한다. 이후 사용자 합의에 따라 완전 충족을 후보 진입 조건으로 쓰지 않는다. 기능 차이와 명시 경계의 차이는 V-03, 사용자 우회 부담은 V-02에 기록한다. 해당 기능의 축소나 target 변경을 이번에 승인한 것은 아니다.
- **시간은 VIA의 책임 구간이다.** 동일한 외부 Agent profile에서 VIA의 준비, 경합, 인계, 후속 해제와 결과 전달을 본다. Agent 내부 대기와 업무 실행, 사람의 답변 대기를 VIA 성능으로 합치지 않는다. 빠른 거절이나 미지원 안내를 요청의 성공 완료로 세지 않는다.
- **QA는 모든 DP와 모든 방안에서 같아야 한다.** 13개 관점의 의미는 [03 §2](./03-00-quality-scenarios.md#2-어떤-품질을-보고-있는가)를 따른다. 아래 인과 설명은 새 지표가 아니다. 정확한 정의, 지표, 모집단, 실패 처리와 측정 방법은 공통 계약으로 측정 전에 고정한다.
- **추가 기능에는 실제 제공자가 있어야 한다.** 모델 weights 복제, 숨은 cloud 호출, 무료 helper나 독립 검증을 가정하지 않는다. VIA가 Agent의 내부 업무 계획과 Tool 실행을 가져오지 않는다. 기능 제한을 허용한다는 이유로 권한 위반이나 불명 외부 실행의 재전송을 정상 동작으로 설계하지 않는다.

품질 표의 ‘유리할 수 있다’는 조건부 구조 가설이다. 측정한 우열이 아니며, 인과가 약하거나 지원이 확인되지 않았으면 그대로 표시한다. 정식 양안 선택과 조건별 (+)/(-)/(0) 비교는 05에서 한다. 낮은 순위의 관점도 모든 구체 대안에 대해 확인했다.

## 2. P별로 어디에 집중했는가

| P | 상위 품질과의 직접 연결 | 구조적으로 바꿀 수 있는 것 | 탐색 깊이 | 이유 및 연계 P |
| --- | --- | --- | --- | --- |
| P-01 요청 이해 | V-01 조건과 정정 확정, V-02 반복 질문, V-04/05 근거 보완 대기 | 해석 중간 상태와 읽기를 실행하는 주체 | 집중: S-01 | 입력 근거가 충분해도 실행 의존성 때문에 보완을 못 하는지 본다. P-03, 06, 14 연결 |
| P-02 화면 지칭 | V-01 당시 대상, V-03 자연스러운 복수 지칭, V-04/05 근거 도착 | 원시 화면 이력, 사전 객체 이력 또는 명시 선택 자료 생산 | 집중: S-02 | VIA의 화면과 발화 결합을 직접 바꾼다. P-01, 11, 14 연결 |
| P-03 자료 확보 | V-01 대상과 내용, V-02 재선택, V-05 자료 준비, V-08 source 변화 | 요청 중 획득과 사전 의미 색인 서브시스템 | 집중: S-01, 03 | ‘무엇을 읽나’와 ‘누가 읽기 흐름을 진행하나’를 구별한다. P-10~12, 15 연결 |
| P-04 직접 응답과 위임 | V-02 같은 대화의 후속 처리, V-03 S2S 지원, V-04/05 경로 길이 | speculative 직접 경로와 Core 공통 경로 | 집중: S-05 | 기능 축소와 경로 단순화의 교환을 구체적으로 볼 수 있다. P-09, 14 연결 |
| P-05 복합 요청 | V-01 후속 대상, V-02 수동 연결, V-03 의존 요청, V-05 후속 해제 | 내구 domain 상태, 지속 continuation, 세션 한정 실행 | 집중: S-04 | 실행과 대기 상태의 권위가 실제로 달라진다. P-06, 07, 13 연결 |
| P-06 업무와 질문 연결 | V-01 답변 binding, V-02 재설명, V-03 재개 | 질문을 소유하는 domain 상태 또는 workflow continuation | 집중: S-04 | P-05와 같은 상태 이동을 별도 대안처럼 중복 작성하지 않는다. P-08, 10, 11 연결 |
| P-07 Agent 연동 | V-01 실제 실행 상태, V-05 인계와 수신, V-08 계약 변화 | source 관측, 명령 전달, capability 변환 경계 | 연계: S-04와 §5 | 동일 Agent 지원 조건에서 대기 실행체와의 연결을 본다. 전송 protocol 자체의 별도 대안은 보류 |
| P-08 중단과 정정 | V-01 정정 효력, V-04 즉시 중단, V-05 취소 후 정리 | 입력 제어와 추론의 의존성, 전송 확정 경계 | 연계: S-01, 04~06 | 고순위지만 독립 주제 수를 늘리지 않는다. 각 대안에서 hold, 취소와 늦은 결과를 대조 |
| P-09 실제 전달 | V-01 전달 사실, V-04 첫 유효 응답, V-05 최종 결과 | 응답 생산과 게시, 실제 전달 권위 | 연계: S-05와 §5 | 응답 경로 변화는 구체화했다. 전달 원장을 client로 옮기는 별도 선택은 추가 근거가 필요 |
| P-10 연속성 | V-02 재설명, V-03 채널 전환, V-05 이력 재구성 | 장기 원본과 모델 작업 상태의 수명 | 연계: S-03~05 | 검색, 대기 상태와 게시 기록이 바뀔 때 실제 수명 차이를 확인 |
| P-11 정보와 승인 | V-01 승인 대상, V-03 사용 범위, V-08 새 제공 경로 | 권한 강제와 철회 전파 경계 | 연계: 모든 S | 새로운 read broker, index, continuation이 생기는 지점마다 현재 권한을 검사. 중요도가 낮다는 뜻이 아님 |
| P-12 기억 | V-01 현재 지시와 삭제 효과, V-03 기억 관리, V-08 파생 구조 변경 | 원본과 색인, 요약, KV의 의존성 | 연계: S-02~04 | 자료를 늘리는 대안의 삭제 및 복구 비용을 같이 본다. 기억 원본 자체의 대체는 보류 |
| P-13 복구 | V-01 중복과 관계, V-02 재입력, V-05 재개 지연 | current records, continuation 원본, 세션 상태 | 연계 심화: S-04와 §5 | 대기 상태 선택의 전제이므로 복구까지 상세화. 독립 event-sourcing 주제는 자동 승계하지 않음 |
| P-14 PC 자원 | V-03 연속 인식, V-04/05 공유 추론 경합 | 독립 입력 생산자 또는 Omni native 근거 | 집중: S-06 | 화면과 의미 처리의 전제이며 정상 처리와 입력이 함께 진행되어야 한다. 모든 S의 비용 연결 |
| P-15 변화 | V-03 교체 뒤 지원, V-08 변경 범위, V-05 이행 후 진행 | read, model, workflow의 계약 및 저장 version | 연계 심화: 모든 S와 §5 | 각 구조의 실제 변경 사례를 확인한다. ‘adapter가 있으니 수정이 적다’로 판정하지 않음 |
| P-16 근거 | V-01 오류 원인, V-08 관측 계약, V-09 분석 | 업무 원본과 진단 자료의 생성 및 보관 | 간략 점검: §5 | 현재 별도 관측 서브시스템이 상위 품질을 바꾼다는 근거는 부족. 각 S의 최소 추적 연결은 유지 |

집중은 7개 P에 두되, S-01과 S-03은 서로 다른 P-03 메커니즘을 다루고 S-04는 P-05/06을 함께 다룬다. 그래서 집중 질문은 6개다. 이 수를 최종 DP 수로 이어받지 않는다. 실제 사용 빈도나 성능 효과의 크기는 아직 모르므로 수치로 우선도를 꾸미지 않았다.

## 3. 구조 질문과 대안 전체 보기

| 질문 | 실제 target T | 탐색한 대안 | 실제로 달라지는 구조 | 대안이 설득력 있는 조건 |
| --- | --- | --- | --- | --- |
| [S-01 근거를 보완하는 해석 실행](#s-01) | Request Controller가 제안과 읽기를 번갈아 실행 | A: 요청별 Semantic Resolution Worker와 Capability Read Broker | Worker가 중단 및 재개 가능한 모델 읽기 continuation을 소유. host는 최종 확정 권위 유지 | 중간 제안 종료와 Context 재조립 및 재호출이 의미 있는 비용인 요청, 해당 runtime 지원 필요 |
| [S-02 당시 화면의 근거 생산](#s-02) | 원시 RAM timeline을 확보한 뒤 발화와 해석 시 결합 | A: Screen Grounding Service가 객체 이력을 미리 생산. B: 사용자가 명시적으로 만든 Capture Package만 사용 | 사전 의미 자료 생산과 보관이 생기거나, 연속 관측과 과거 복수 지칭 기능이 줄어듦 | A는 반복 지칭과 안정된 UI 구조, B는 명시 선택 위주의 제한된 사용 |
| [S-03 보이지 않는 자료 발견](#s-03) | owner read port, metadata/keyword index와 cache | A: Semantic Retrieval Service, Indexing Worker, Embedding Runtime, Vector Index | 의미 후보 생산, 색인 갱신과 삭제를 위한 별도 자료 수명 | 이름보다 주제로 찾고 동일 자료를 반복 참조하는 환경 |
| [S-04 목표와 질문의 대기 실행](#s-04) | Request Controller의 durable Request Graph와 Pending User Interaction | A: Interaction Workflow Runtime. B: 세션 한정 요청 coordinator | 내구 continuation이 실행 원본을 대체하거나 대기 그래프 내구 복구를 포기 | A는 복잡한 장기 대기, B는 짧고 단순한 대화와 제한된 복구 요구 |
| [S-05 직접 응답의 실행 경로](#s-05) | 좁은 S2S VoiceProposal 경로와 Core 경로 | A: Core-only 응답 pipeline | 직접 speculative 생성과 admission 경로 제거. 확정된 Text에서 Voice 생성 | 맥락 의존 요청이 많고 직접 S2S 기능 축소를 감수하는 경우 |
| [S-06 음성 입력 근거의 생산자](#s-06) | 독립 Speech Input Worker의 Streaming ASR와 공유 Omni | A: Omni native evidence와 Native Evidence Adapter | ASR 실행체와 weights 제거, 인식의 진행과 장애가 Omni에 의존 | 필요한 native 시간 근거와 동시 session이 실제 지원되고 자원이 맞는 경우 |

A/B는 서로 자동 결합되는 완제품 안이 아니다. 하나의 질문에서 선택을 달리하고 나머지 맥락을 고정한 스케치다. §6에서 결합 시 달라지는 조건을 별도로 다룬다. 모든 target 행은 설계상 지원을 기술하며 실측 완료를 뜻하지 않는다.

## 4. 집중 탐색

<a id="s-01"></a>
### S-01. 추가 근거가 필요할 때 누가 해석을 계속 진행하는가

**문제와 기준선.** P-01/03의 “아까 그 자료를 김 팀장에게 보내줘”에서 대상 자료와 수신자 단서가 서로 영향을 줄 수 있다. Target은 Request Controller → Context Manager의 기본 준비 → Request Interpreter의 제안 → 필요 시 Request Controller → Context Manager → Request Interpreter 순이다. Request Interpreter는 도구와 확정 권한이 없는 제안자다. 기본 자료 준비는 입력과 겹칠 수 있으며, target이 모든 조회를 매번 처음부터 직렬 실행하는 것은 아니다. 현재 정책은 입력 revision당 semantic 총 2회와 추가 읽기 한 묶음이다. [구조 §6~7](../target-architecture/architecture.md), [제어 §3](../target-architecture/control-and-lifecycle.md#3-core-해석읽기응답-예산).

**대안 A — 읽기에서 중단하고 재개하는 해석 실행체.** Semantic Resolution Worker가 하나의 요청에 대한 임시 field와 read set, 모델 실행의 읽기 continuation을 가진다. 이 스케치에서는 Model Access를 통해 runtime이 최종 semantic proposal을 끝내기 전에 `READ_REQUIRED`와 재개 handle을 반환하는 계약을 제안한다. 실행은 그 지점에서 연산 점유를 양보하고, Capability Read Broker가 현재 Policy Manager의 권한과 읽기 예산을 검사한 뒤 Context Manager의 source/owner read port에서 근거를 가져온다. Worker는 동일 input revision과 receipt에 묶인 근거만 handle에 넣어 실행을 재개하고, 마지막에 최종 proposal을 Request Controller에 반환한다. 대기 중 KV를 유한 범위로 보관하며 회수되거나 handle이 무효하면 재개 실패로 처리한다. Request Controller는 현재 입력 revision, 질문과 권한을 확인하고 최종 commit한다. Worker에는 Agent 업무 실행, 외부 변경 또는 자체 승인의 권한이 없다. 이 스케치는 local Core 안의 독립 실행 수명과 계약을 가진 작업으로 두며, 별도 process의 장애 격리 이익을 가정하지 않는다.

| 실행과 상태 | T와 다른 점 및 비용 |
| --- | --- |
| 정상 보완 | host가 해석 제안과 다음 호출을 번갈아 조정하는 대신 Worker가 근거 보완의 중간 실행을 지속한다. 중간 proposal 종료 → host의 다음 Context 조립 → 새 semantic 호출이라는 의존 경계를 runtime의 중단 및 근거 주입 후 재개로 대체한다. 읽기 권한 검사와 broker 호출은 남는다. |
| 책임의 교체 | Request Interpreter의 단발 제안 계약을 Worker와 Model Access의 양방향 중단 및 재개 계약으로 대체한다. Request Controller의 최종 확정과 입력 hold는 유지하되 상세 읽기 continuation은 소유하지 않는다. 같은 함수를 다른 파일로 옮기는 안이 아니다. |
| 정정과 철회 | 새 입력 revision이나 정책 epoch에서 Worker 결과를 폐기한다. Broker는 시작뿐 아니라 실제 읽기와 결과 사용 때 유효성을 확인한다. 이미 반환된 자료의 사용과 KV도 차단한다. 이전 session을 새 요청의 권한으로 재사용하지 않는다. |
| 실패와 복구 | 임시 continuation은 권위 업무 상태가 아니다. Worker crash 뒤 현재 Request에서 읽기를 재시도하거나 사용자에게 한계를 알린다. 읽기 실패와 부분 coverage를 남긴다. 동일 input revision의 누적 읽기 및 semantic 연산 예산과 deadline은 Request Controller와 Broker의 예산 기록에 남겨 Worker 재시도로 초기화하지 않는다. 유한 예산에서 종료한다. 최종 commit과 Action 전송은 host의 내구 경계를 따른다. |
| 지원 목표와 U | 조건 보완과 정정 기능은 유지 목표. 중단 및 재개 가능한 read callback, 근거 주입, 격리 session, 취소와 receipt 전달은 U-03/04/07/08로 미확인이다. 기능 없는 runtime을 wrapper 이름만으로 보완했다고 하지 않는다. |

이 안의 핵심은 **완료된 중간 제안과 새 호출 사이를 host가 잇는 구조를, 중단 및 재개 가능한 실행 계약으로 바꾸는 것**이다. session과 KV 격리는 target에도 있다. Target의 cache 및 prefix 재사용이 불가능하거나 A가 prefill을 반드시 줄인다고 가정하지 않는다. 동일한 호출, Context와 예산을 유지한 채 제어 코드만 Worker로 옮겼다면 별도 구조 효과의 근거가 없다. 제안한 runtime 계약을 확보하지 못하면 그 차이는 성립 미확인으로 남긴다. 05에서는 공통 읽기 및 연산 예산으로 구조와 예산 효과를 구별하며 API 호출 수가 줄었다고 연산 예산까지 줄었다고 세지 않는다. 같은 모델의 반복 검토는 독립 정답 검증이 아니다.

| 관점 | A에서 확인할 품질 경로와 조건 |
| --- | --- |
| V-01 기능 정확성 | 읽기 결과를 잇는 중간 상태가 조건 보완을 도울 수 있다. 반대로 오래된 임시 field나 부적절한 읽기 계획이 오류를 이어갈 수 있어 최종 read set 검증이 필요하다. |
| V-02 기능 적절성 | 필요한 추가 자료를 스스로 확보하면 사용자 재선택을 줄일 수 있다. 근거가 모호한 경우의 질문까지 없애지는 않는다. |
| V-03 기능 완전성 | 02 P-01/03 기능을 유지하는 설계 목표. callback 미지원과 접근할 수 없는 source는 실제 지원 미확인 또는 제한으로 남긴다. |
| V-04 상호작용 반응성 | 중간 제안 종료와 host 재제출 대기를 줄일 가능성이 있으나 실제 이득은 미확인이다. Worker의 긴 실행이나 점유로 질문과 정정 응답이 늦어질 수 있다. 입력 stop은 이 실행체 밖에 둔다. |
| V-05 VIA 귀속 요청 완료 시간 | 재개 계약이 줄이는 재조립 및 재호출 비용과 broker 읽기, KV 보관, handle 무효 뒤 재처리 비용을 함께 본다. Agent 업무 시간을 단축한다는 주장은 없다. |
| V-06 자원 활용성과 수용량 | 읽기 대기 중의 pinned KV, 동시 Worker와 broker 비용을 본다. Target에도 session/KV가 있으므로 전체 KV를 A의 순증가로 세지 않는다. weights는 공유하며 Worker마다 모델을 만들지 않는다. |
| V-07 결함 허용성과 복구성 | 임시 작업만 폐기할 수 있지만 긴 읽기 진행은 잃는다. 재시도 가능 읽기와 commit 이후 효과를 구별한다. |
| V-08 변경 용이성과 모듈성 | read callback 형식 변경은 Worker, Broker와 Model Access에 걸친다. source 형식은 기존 Context Manager port로 흡수할 수 있으나 의미 변화까지 자동 격리되지는 않는다. |
| V-09 분석 및 시험 용이성 | read plan, receipt, input revision과 최종 proposal 연결이 필요하다. 하나의 긴 모델 session 내부만 보이면 오히려 원인 분리가 어렵다. Broker의 지연과 잘못된 revision, 재개 handle 무효를 통제할 시험 경계가 필요하며 runtime 지원은 미확인이다. |
| V-10 기밀성 | 지속 session과 중간 읽기에 자료가 더 남을 수 있다. 현재 정책 검사, 철회와 KV 폐기까지 비용에 넣는다. |
| V-11 상호운용성과 공존성 | callback 계약 의존성이 늘고 긴 추론이 다른 앱과 자원을 공유한다. 지원되지 않는 모델에 동등 연동을 가정하지 않는다. |
| V-12 조작 용이성과 사용자 오류 방지 | 실제 제시된 질문에 답을 연결하는 host 규칙은 유지한다. 늦은 Worker 질문이 새 질문 focus를 덮지 않도록 한다. |
| V-13 설치 용이성 | Worker와 Broker의 배포 및 호환 확인이 생긴다. 같은 제품에 묶을 수 있지만 설치 부담의 실제 크기는 미확인이다. |

**05로 가져갈 판단:** 연쇄적인 근거 보완이 있는 조건과 기본 Context만으로 끝나는 조건을 나눈다. target도 같은 정도로 상태를 재사용하거나 Context 재조립 비용이 작고 대기 KV 유지 및 잘못된 읽기가 더 비싸면 이점 가설은 약해진다. 실제 runtime이 같은 실행의 중단과 재개를 지원하지 않으면 이 구조 스케치는 그대로 실현되지 않는다.

<a id="s-02"></a>
### S-02. 말한 당시의 화면을 나중에 해석할 것인가, 미리 구조화할 것인가

**문제와 기준선.** P-02의 “이것들은 남기고 저것만 바꿔”는 현재 screenshot 하나로 풀 수 없다. Target의 Interaction Manager는 OS/UI 사건, 유한 sampling, pre-roll, 발화 구간 pin을 RAM timeline에 남긴다. 늦은 전사와 시각 오차를 당시 관측에 연결하고 Context Manager와 Request Interpreter가 자료 및 의미 후보를 만든다. target은 partial transcript의 ‘이거’ token마다 capture를 시작하지 않으며 화면 구조 자료도 활용한다. [기억 §2, 4](../target-architecture/memory-and-context-lifecycle.md), [구조 §9](../target-architecture/architecture.md#9-contextcachestale-처리).

**대안 A — 사전 객체 이력.** Screen Grounding Service가 허용된 UI 사건과 제한된 crop을 받아 window/document, 객체, 영역, 선택과 version을 가진 파생 이력을 만든다. 구조 정보가 없으면 공유 Omni의 별도 vision session으로 객체 후보를 생산한다. 이 추가 추론과 자원은 공짜가 아니다. 전용 모델이 필요하다고 밝혀지면 helper 비용과 설치를 추가 검토하며 현재 안에 숨겨 두지 않는다. 발화 시점에는 원시 화면을 처음 해석하기보다 준비된 객체 이력에 결합하고 필요한 원문을 확인한다.

A는 발화 뒤 수행하던 지칭 준비의 일부를 별도 producer와 Derived Object Store로 옮긴다. 이 스케치에서 객체 이력은 유한 RAM에 두고, 요청에 채택한 최소 객체 근거와 source 참조만 허용 범위에서 내구 보관한다. 화면 전체의 영구 객체 DB를 추가한 안은 아니다. 객체 ID는 외부 앱의 영구 ID와 같다고 가정하지 않는다. source revision, 시간 오차, confidence, gap과 원시 근거 참조를 보존하고 제한된 RAM 원시 buffer를 fallback으로 유지한다. 재시작 후 오래된 객체를 현재 화면으로 재사용하지 않으며, 허용된 durable 자료가 없는 과거는 복원 불명이다. producer가 늦으면 최신 객체가 아니라 해당 시점의 가용 근거를 사용하거나 재지칭을 요청한다. 삭제와 권한 철회는 객체 이력과 생성 중 작업까지 전파한다.

**대안 B — 명시 Capture Package.** 사용자가 영역 또는 객체를 선택하고 ‘이 자료로 요청’하는 시점에 UI/문서 identity, revision, timestamp, crop을 한 묶음으로 봉인한다. 이후 발화는 그 패키지의 대상만 참조한다. 연속 화면 이력 producer와 자연스러운 과거 지칭 결합을 제거한다. 패키지는 허용 수명 안에서 요청 근거로 보관하고 source 변경 시 적용 가능성을 다시 확인한다. crash 뒤 남지 않은 선택은 사용자가 다시 만든다. 접근 거부나 source 불명은 패키지 성공으로 표시하지 않는다.

| 지원 범위 | T | A | B |
| --- | --- | --- | --- |
| 말하면서 여러 대상을 가리킨 당시 관계 | 지원 설계, U-01/02/03 필요 | 지원 목표. 객체 추적과 시간 정렬 능력 미확인 | 미지원. 명시 선택한 묶음만 지원 목표 |
| 구조 정보가 없는 화면 | 시각 근거 사용 설계 | vision producer와 fallback의 실제 지원 미확인 | 명시 crop은 가능 목표, crop 안의 의미 해석은 여전히 미확인 |
| 늦은 전사와 화면 변경 | pin된 근거 범위에서 연결 | 당시 객체 이력과 제한된 원시 근거 범위에서 연결 목표 | 지정 패키지 연결만 가능. 선택 전 이동과 역할 변화는 복원하지 않음 |
| 사용자 우회 | 필수 근거가 없으면 재지칭 | producer 실패나 모호함에서 재지칭 | 사용자가 매번 자료를 미리 선택. 자연스러운 연속 지칭 목표를 완전히 대체하지 못함 |

| 관점 | A: 사전 객체 이력 | B: 명시 Capture Package |
| --- | --- | --- |
| V-01 기능 정확성 | 반복 객체 지칭에 일관된 근거를 줄 수 있으나 추적 오류가 후속 요청에 퍼질 수 있음 | 명시 대상은 좁아지지만 사용자의 잘못된 선택과 crop 내 해석 오류는 남음 |
| V-02 기능 적절성 | 준비가 맞으면 재지칭을 줄일 가능성 | 선택, 봉인과 재선택을 사용자에게 요구 |
| V-03 기능 완전성 | 자연 지칭 유지 목표, 실제 객체 추적과 fallback 지원 미확인 | 시간에 따른 자연스러운 복수 지칭을 포기. 공통 요구 목록에서 제거하지 않음 |
| V-04 상호작용 반응성 | 이미 준비된 객체면 결합이 짧아질 수 있으나 producer backlog이면 대기 | 선택 뒤 경로는 짧아질 수 있으나 사전 UI 준비는 사용자 수고. 미지원 안내는 유효 처리 성공이 아님 |
| V-05 VIA 귀속 요청 완료 시간 | 준비 비용 이동과 요청 때 재확인, 추론 경합을 함께 봄 | 연속 관측 비용이 줄 수 있으나 패키지 검증과 재요청 비용을 포함. 사람의 선택 시간은 별도 |
| V-06 자원 활용성과 수용량 | 객체 저장, 갱신, background vision과 raw fallback 비용 추가 | 연속 buffer와 producer 제거 여지, 패키지와 이미지 해석 비용은 남음 |
| V-07 결함 허용성과 복구성 | producer 장애와 객체/raw 불일치 추가. 파생 상태를 재생산하는 비용 | 연속 관측 작업과 임시 상태는 줄지만 UI/Core process와 그 장애 경계는 남음. 보관되지 않은 선택은 잃고 자연 연속 지칭은 미지원 |
| V-08 변경 용이성과 모듈성 | UI schema 변경이 객체 변환과 저장 version까지 전파 | source의 selection/capture API 변화에 의존. 시간 추적 변환은 줄어듦 |
| V-09 분석 및 시험 용이성 | 객체 생성 근거와 발화 결합을 구분해 남기고 producer gap, 시각 오차와 늦은 객체 갱신을 통제할 시험 경계 필요 | 패키지 생성 뒤 source 변경과 권한 철회를 통제해 판정할 수 있어야 함. 원래 자연 지칭의 실패 자료가 사라지는 한계는 남음 |
| V-10 기밀성 | 객체 이력도 보호자료이며 raw와 함께 철회 및 삭제 필요 | 수집 시점과 범위가 좁지만 선택 패키지의 보관과 제공 권한 검사는 필요 |
| V-11 상호운용성과 공존성 | UI 구조 지원 차이, background vision의 다른 앱 경합 | 선택 API 지원과 수동 crop 대체 범위에 의존. 연속 부하는 줄어들 여지 |
| V-12 조작 용이성과 사용자 오류 방지 | 오래된 객체를 현재 선택처럼 보여주지 않아야 함 | 선택 확정이 눈에 보이나 잘못 봉인한 대상을 그대로 쓸 위험 |
| V-13 설치 용이성 | producer와 파생 저장 version 관리 추가. 현재 별도 helper 미포함 | 기존 UI 기능으로 구현 가능 목표. OS capture 권한 설정 부담은 남음 |

**05로 가져갈 판단:** A는 지칭 전에 자료가 준비될 수 있고 객체 변화가 추적 가능한 조건에서 강하다. 짧게 나타나는 화면, 빠른 변화, 자원 포화에서는 그 전제가 무너질 수 있다. B는 의도적인 기능 제한안이다. 선택 작업이 실제 사용 목표를 지나치게 방해하면 자원 이익이 있어도 V-02/03의 손실이 크다. 양안 모두 U-01/02/03/04/07/08을 확인해야 한다.

<a id="s-03"></a>
### S-03. 자료를 요청할 때 찾을 것인가, 의미 후보를 미리 준비할 것인가

**문제와 기준선.** P-03의 “전에 이야기한 고객 이탈 분석 자료”는 정확한 파일명이 아닐 수 있다. Target의 Context Manager는 owner read port, source adapter, metadata/keyword index, revision cache와 파생 summary를 사용한다. 기본 embedding 모델과 vector DB는 없다. source의 내용, 권한, 부분 coverage를 receipt로 남긴다. 따라서 이 비교는 ‘검색 없음 대 검색 있음’이 아니다. [기억 §1~3](../target-architecture/memory-and-context-lifecycle.md), [02 P-03](./02-00-requirements-and-choices.md#p-03).

**대안 A — 지속 의미 검색 서브시스템.** Indexing Worker가 지속 가공과 보관이 허용된 유한 collection의 자료와 revision을 읽고 Embedding Runtime이 의미 표현을 만든다. Vector Index와 source manifest가 파생 자료를 보관하며 Semantic Retrieval Service가 요청의 후보를 생산한다. Context Manager는 후보의 현재 source, 권한, identity와 필요한 본문을 다시 확인한다. 의미 검색 결과는 확정 대상이나 전체 coverage 증명이 아니다. 한정 조회 뒤 여러 후보가 남으면 질문한다.

추가 embedding helper는 역할, weights, runtime, CPU/accelerator, 갱신과 설치 비용을 모두 공개할 별도 dependency다. 이를 shared Omni의 무료 기능으로 계산하지 않는다. 색인이 없는 최초 요청과 갱신 중 요청에는 target의 metadata/keyword 및 source 조회 경로를 fallback으로 유지한다. 따라서 구성 단순화가 아니라 **발견 기능과 준비 비용의 교환**이다.

색인 commit은 source revision과 자료 사용 epoch에 묶는다. 늦은 갱신이 삭제 자료를 되살리지 못하게 tombstone과 purge 작업을 적용한다. index 손상은 원본 복구와 구별해 재구축하고 그동안 fallback의 지원 한계를 알린다. 외부 source가 revision이나 삭제 확인을 제공하지 않는 범위는 freshness와 coverage 미확인으로 남긴다. helper의 검색 품질, source connector와 삭제 전파의 실제 동작은 U-03/04/07/08로 미확인이다.

| 관점 | A에서 확인할 품질 경로와 조건 |
| --- | --- |
| V-01 기능 정확성 | 주제 지칭에서 후보 발견을 도울 수 있지만 유사한 다른 자료와 오래된 vector가 오확정을 늘릴 수 있다. 원문 및 identity 확인이 필요하다. |
| V-02 기능 적절성 | 파일명을 기억하거나 자료를 다시 고르는 일을 줄일 여지가 있다. 부정확한 후보 목록이 오히려 선택 부담을 늘릴 수 있다. |
| V-03 기능 완전성 | P-03의 자료 유형과 예외 지원 유지 목표. 색인 불가능 source는 fallback 범위를 밝히며 ‘검색 안 됨’을 ‘자료 없음’으로 바꾸지 않는다. |
| V-04 상호작용 반응성 | 준비된 index에서 후보 응답이 빨라질 수 있으나 cold start, 갱신과 원문 재조회에서는 다르다. |
| V-05 VIA 귀속 요청 완료 시간 | 후보 탐색 단축과 본문 확인, 잘못된 후보 재처리, background 자원 경합을 함께 본다. 결과 몇 건이 빨리 나왔다는 사실만으로 전체 완료 이익을 주장하지 않는다. |
| V-06 자원 활용성과 수용량 | helper weights, vector와 manifest, 초기 색인, 재색인, 삭제와 fallback 유지 비용이 추가된다. 요청 밖 비용도 포함한다. |
| V-07 결함 허용성과 복구성 | 원본과 index의 불일치, 부분 갱신과 손상을 처리해야 한다. fallback이 있더라도 의미 검색의 동일 기능 복구는 아니다. |
| V-08 변경 용이성과 모듈성 | 새 source, embedding 차원 또는 청크 표현 변경이 adapter뿐 아니라 재색인, manifest와 검증에 영향을 준다. 검색 계약으로 호출부를 가릴 수 있는 범위와 구별한다. |
| V-09 분석 및 시험 용이성 | 후보 목록, index/source version과 확정 근거가 연결되어야 발견 실패와 해석 실패를 구분할 수 있다. 전체 원문 로그는 필수가 아니다. Worker 갱신을 멈추거나 삭제 뒤 늦은 색인 commit을 전달해 결과 사용 차단을 확인할 시험 경계도 필요하다. |
| V-10 기밀성 | vector와 snippet도 파생 보호자료다. 읽기 허용과 지속 보관 허용을 구별하고 철회 이후 신규 사용 차단과 purge를 수행한다. |
| V-11 상호운용성과 공존성 | source 열거 및 revision 지원과 embedding runtime 호환에 의존한다. background 색인이 다른 앱의 자원을 사용할 수 있다. |
| V-12 조작 용이성과 사용자 오류 방지 | 의미 유사도를 유일 정답처럼 표시하지 않는다. 후보 출처와 구별 단서를 사용자에게 제시해야 한다. |
| V-13 설치 용이성 | helper와 index 초기화, 버전 호환, 제거 후 파생 자료 정리 단계가 늘어난다. |

**05로 가져갈 판단:** 반복되는 주제 조회와 안정된 자료에서는 준비 비용을 감수할 이유가 있다. exact ID 위주의 조회, 자주 바뀌는 자료, 지속 보관이 허용되지 않는 환경에서는 기존 index/cache보다 추가 효익이 작을 수 있다. [기존 의미 검색안](./semantic-retrieval-subsystem.md)은 이 질문의 입력이지만 이전 QA 표와 선정 판단을 그대로 승계하지 않는다.

<a id="s-04"></a>
### S-04. 목표와 질문의 대기를 어떤 실행 원본으로 이어갈 것인가

**문제와 기준선.** “보고서가 끝나면 그 결과로 발표자료를 만들고, 별도로 일정도 확인해줘”에서 VIA는 사용자 목표 사이 관계를 다룬다. 보고서 내부의 조사와 도구 순서는 Agent 책임이다. Target은 Request Controller의 durable Request Graph, Pending User Interaction과 domain 전이로 대기와 후속 admission을 소유한다. Task Manager는 Task/Execution, Agent Gateway는 전송과 source 관측을 소유한다. current records와 미완료 원장으로 복구하며 audit log를 실행 원본으로 replay하지 않는다. [구조 §5, 12, 14](../target-architecture/architecture.md), [제어 §4~5, 7](../target-architecture/control-and-lifecycle.md).

**대안 A — 지속 interaction workflow.** Interaction Workflow Runtime이 목표별 continuation, 의존 결과 version, 질문 대기와 timer를 내구 실행 원본으로 가진다. Signal Inbox는 source 사건과 사용자 답변을 중복 제거하며 Timer Service는 내구 deadline에 맞는 timeout signal을 만든다. Activity Dispatcher는 source 읽기, 질문 게시, 후속 admission 요청을 보낸다. Request Controller는 해석과 현재 권한의 최종 승인 경계를 유지하지만 같은 대기 그래프와 질문 상태를 별도 권위 원본으로 중복 보유하지 않는다. 조회용 view는 workflow 원본에서 만든다. Task Manager의 실제 업무 lifecycle과 Agent Gateway의 명령 전송은 유지한다.

Continuation Store는 State Store의 별도 논리 영역으로 두고 signal 수락, continuation 전이와 activity intent를 하나의 로컬 transaction에 기록하는 안으로 잡았다. Task Manager가 확정한 사건은 내구 outbox에서 전달하고, runtime은 중복 제거 후 수락한다. Activity Dispatcher가 보낸 admission 요청은 host에서 identity와 현재 epoch를 다시 검사한다. 이 연결을 단순 함수 호출로만 두면 crash 때 유실 또는 중복이 생긴다.

답변은 실제 제시된 interaction ID, 현재 workflow version, Execution 및 승인 digest에 맞는 continuation에만 들어간다. 답변 수락과 해당 대기 종료도 같은 runtime transaction에서 처리한다. 선행 결과 version이 바뀌면 미해제 continuation을 무효화한다. 이미 해제한 외부 실행은 되감지 않고 새 정정으로 처리한다. Workflow의 재개 기록만으로 외부 Agent의 exactly-once를 보장하지 않는다. Activity의 stable identity와 host admission, command outbox, source 조회가 여전히 필요하다.

**대안 B — 세션 한정 요청 coordinator.** 살아 있는 Core 세션에서만 목표 관계, 후속 대기와 질문 focus를 유지한다. durable Request Graph와 질문 continuation 저장 및 복구 loader를 제거한다. Conversation과 이미 접수한 Task/Execution, 전송 의도와 중복 방지 원장, 실제 게시 기록은 계속 보관한다. 사용 중에는 독립 요청과 질문을 연결하지만 Core가 재시작하면 미완료 목표 관계와 대기 질문 연결은 재구성하지 않는다. 살아 있는 세션의 Voice/Text 전환은 지원 목표다.

B의 재시작 뒤에는 확인 가능한 기존 Task와 결과를 보여주되 미제출 목표와 후속 관계는 사용자가 다시 지정한다. 옛 답변이나 승인을 새 요청에 재사용하지 않는다. 외부 접수가 불명인 요청은 자동 재전송하지 않는다. Agent가 질문을 재조회해 줄 수 있더라도 원래 VIA clarification과 전체 요청 그래프 복구를 대신하지 못한다. 이는 FA-14와 P-05/06/13의 일부 기능 포기이며, 단순 구현 미완료를 정상 복구로 포장한 안이 아니다.

| 상황 | T | A | B |
| --- | --- | --- | --- |
| 정상 결과 도착 | current graph의 version과 조건 확인 후 후속 해제 | Signal Inbox 반영 후 해당 continuation의 activity 실행 | 메모리 관계 확인 후 후속 요청 생성 |
| 질문 대기 중 새 목표 | 별도 Request와 질문 identity로 진행 | 별도 continuation 진행, 제시 focus를 결합 | 메모리 identity로 진행, 단일 ‘마지막 질문’만 두지 않음 |
| 정정과 전송 경쟁 | host hold/epoch와 조건부 dispatch | workflow 취소 신호와 별개로 동일 host dispatch 경계 필요 | 내구 command admission 경계 유지. 그래프가 메모리라는 이유로 전송 검사 생략 불가 |
| crash 뒤 대기 복원 | current records와 미완료 원장 및 source 확인 | 내구 continuation과 signal/activity 기록 복원 및 source 확인 | 해당 기능 미지원. 확인되는 Task 상태와 잃은 목표 관계를 구분 |
| 저장 또는 이행 실패 | 새 외부 전송 차단, 상태 보존 및 실패 표시 | continuation version과 activity 이행 실패에서도 동일한 제한. 같은 저장소여도 activity 전달과 외부 실행은 별도 경계이므로 중복 및 유실 조정 필요 | 남아 있는 Task/command 저장 실패에서도 새 전송 금지. 제거한 저장만큼 전체 persistence가 사라진 것은 아님 |

| 관점 | A: 지속 workflow | B: 세션 한정 coordinator |
| --- | --- | --- |
| V-01 기능 정확성 | 명시 signal과 continuation이 관계 보존을 도울 수 있으나 잘못 연결한 signal이나 replay가 반복 오류를 만들 수 있음 | 정상 세션의 binding은 유지 목표. 재시작 후 관계 상실과 오연결 위험을 드러내고 재확인 |
| V-02 기능 적절성 | 긴 대기 뒤 자동 재개 목표. 실패 복구가 실제로 간단해지는지는 미확인 | crash 후 목표 재설명과 수동 연결 부담 증가 |
| V-03 기능 완전성 | 02의 요청 및 질문 복구 유지 목표. runtime의 version, timer, 취소와 signal 지원 미확인 | 재시작 뒤 미완료 요청 관계 및 질문의 자동 복원 미지원. 지원하는 Task 복구와 구별 |
| V-04 상호작용 반응성 | signal 접수와 질문 게시에 내구 단계가 추가될 수 있음. local stop은 workflow 대기를 거치지 않음 | 그래프 저장 대기를 줄일 여지가 있으나 의미 확정과 command 저장은 남음 |
| V-05 VIA 귀속 요청 완료 시간 | 후속 release와 재개 비용, activity 전달 및 중복 제거 비용을 함께 봄 | 정상 흐름의 저장 비용 감소 가능. 잃은 후속 목표를 빠른 완료로 세지 않고 재처리와 실패 기록 유지 |
| V-06 자원 활용성과 수용량 | Runtime, timer, signal/continuation 기록과 조회 view 유지 비용 | 제거한 내구 그래프 비용 감소 가능. 열린 세션의 메모리와 Task 원장은 남음 |
| V-07 결함 허용성과 복구성 | 대기 재개를 runtime 계약에 맡길 수 있으나 activity 전달과 외부 실행의 불일치는 별도 처리 | 그래프와 질문 복구 능력이 명시적으로 낮아짐. 프로세스 재시작은 정상 기능 복구가 아님 |
| V-08 변경 용이성과 모듈성 | 새로운 대기 유형은 workflow로 표현 가능. 실행 중 continuation version과 activity 계약 이행 부담 추가 | 내구 그래프 migration은 없어지지만 변경 시 살아 있는 대기를 잃거나 drain해야 함 |
| V-09 분석 및 시험 용이성 | signal, activity와 domain 사건을 연결하고 답변, terminal, timeout의 순서를 통제할 시험 경계 필요. runtime 이력만으로 외부 효과 판정 불가 | 세션 종료 전 최소 기록이 없으면 사후 분석 범위 축소. 재시작을 주입해 잃은 그래프와 남은 Task를 구별해야 함. 임시 상태 전체 로그로 내구 그래프를 몰래 재도입하지 않음 |
| V-10 기밀성 | signal/continuation에 민감한 payload를 복제하지 않고 참조 사용. 삭제 및 철회 전파 대상 증가 | 임시 자료 수명은 짧아질 수 있지만 남는 Conversation/Task 자료의 권한 및 삭제 의무는 동일 |
| V-11 상호운용성과 공존성 | Agent event/질문을 signal로 바꾸는 계약 추가. 내부 timer가 Agent 기능을 대신하지 않음. timer 및 signal/activity 재개가 공유 CPU와 IO를 사용하므로 다른 PC 앱과의 공존성도 미확인 | 동일 Agent 지원 수준. 메모리 절약 가능성은 대기 부하와 잔존 runtime에 따라 다름 |
| V-12 조작 용이성과 사용자 오류 방지 | 실제 제시 focus와 유효 continuation을 표시해야 늦은 승인을 막음 | 재시작 뒤 남은 Task와 잃은 요청을 분명히 구별해야 중복 재요청을 줄임 |
| V-13 설치 용이성 | 내장 가능한 runtime이라도 version 및 상태 호환 관리가 추가됨 | 별도 workflow dependency는 없으나 target도 원래 그런 dependency는 없음. 그래프 제거가 자동 설치 이익은 아님 |

**05로 가져갈 판단:** A는 긴 대기와 다양한 재개 형태에서 유력하지만 VIA가 runtime에 넣을 domain 계약이 여전히 많다면 추상화의 이점이 작을 수 있다. B는 세션 중 처리만으로 충분한 제한된 환경에서 검토할 수 있다. target의 기본 복구 목표와 같다고 주장하지 않는다. U-05/06/07/08과 runtime 재개 기능은 미확인이다. [기존 지속 요청안](./durable-request-orchestration.md)은 참고 입력이다.

<a id="s-05"></a>
### S-05. 좁은 S2S 직접 경로를 별도로 유지할 것인가

**문제와 기준선.** Target은 자체 지식으로 답할 수 있는 current-Turn-only 질문에서 VoiceProposal을 먼저 만들고 Request Controller의 admission 뒤 보류한 generation을 release한다. 개인 자료, 화면, Task나 과거 대화가 필요하면 같은 Request를 Core로 인계한다. Core 응답의 Text와 Voice는 Response Manager가 공통 사실에 연결하고 Interaction Manager가 실제 전달 receipt를 보낸다. target도 검증된 문장 단위 출력을 허용하므로 ‘항상 전체 답을 기다린다’는 비교는 잘못이다. [제어 §2, 6](../target-architecture/control-and-lifecycle.md), [구조 §8, 13](../target-architecture/architecture.md).

**대안 A — Core-only 응답 pipeline.** 모든 발화를 정규 입력 근거 → Request Interpreter의 의미 및 handling 제안 → Request Controller의 확정으로 보낸다. 직접 답할 내용은 같은 호출의 허용된 draft 또는 필요한 Response Manager 구성으로 만들고 승인된 Text에서 공유 Omni의 SpeechRender를 실행한다. 별도 VoiceProposal, speculative S2S 생성과 그 전용 buffer/admission 상태는 제거한다. Response Manager의 publication outbox, 채널별 전달 사실과 local stop의 output epoch는 유지한다.

자체 지식 질문도 답할 수 있고 후속 Agent 업무에 같은 Response를 참조할 수 있다. 그러나 **S2S 직접 기능(B-03)은 미지원**이다. Text 기반 의미 처리와 SpeechRender를 연결한 것을 동일 S2S 기능이라고 부르지 않는다. 자료 기반 직접 처리와 Agent 위임의 책임 경계는 유지한다. SpeechRender도 미지원이면 Text-only 제한이며 전체 음성 기능 성공이 아니다.

정정 때 Core 제안과 음성 generation을 현재 revision에 맞게 폐기한다. 먼저 생성된 Text가 이미 게시됐으면 게시 사실과 정정 기록을 보존한다. 늦은 Voice segment는 output epoch로 막는다. crash 뒤 미완료 publication을 복원하되 audible receipt가 없는 구간은 DELIVERY_UNKNOWN으로 남긴다. 경로를 하나로 만들었다고 실제 전달과 저장의 비원자성이 사라지지 않는다. U-01/03/06/07/08이 필요하다.

| 관점 | A에서 확인할 품질 경로와 조건 |
| --- | --- |
| V-01 기능 정확성 | 경로별 불일치와 handoff 오류를 줄일 여지가 있다. Target S2S도 정규 전사와 input_echo 대조를 사용하므로 ASR 의존성을 A만의 비용으로 세지 않는다. A는 VoiceProposal의 자체 분류와 원음 활용 경로를 정규 입력 기반 Request Interpreter의 semantic 처리로 바꾸므로 오류 유형과 음성 단서 보존의 차이를 확인해야 한다. 같은 모델 사용은 독립 검증이 아니다. |
| V-02 기능 적절성 | 직접 답변에서 후속 업무로 이어지는 사용자 흐름은 유지 목표. 경로가 단일하다는 이유만으로 사용자 단계가 줄었다고 주장하지 않는다. |
| V-03 기능 완전성 | 일반 질문 답변은 유지하되 B-03의 S2S 직접 응답은 포기한다. 음성의 풍부한 표현 보존 여부도 실제 SpeechRender 지원으로 확인한다. |
| V-04 상호작용 반응성 | 양안 모두 host 확정을 거친다. A는 단순 자체 지식 질문도 Request Interpreter의 semantic 호출 뒤 SpeechRender로 이어지므로 target의 speculative 직접 경로보다 첫 Voice 대기가 늘 수 있다. 맥락 요청에서 불필요한 speculative 작업은 줄일 여지가 있다. |
| V-05 VIA 귀속 요청 완료 시간 | handoff와 미사용 생성 제거, 모든 질문의 Core 처리 및 SpeechRender 의존성을 함께 본다. 첫 음성이 빠른 것과 최종 내용 전달은 구별한다. |
| V-06 자원 활용성과 수용량 | speculative generation/buffer가 줄지만 Core 부하가 늘 수 있다. 같은 weights를 쓰므로 Omni 모델 하나가 통째로 줄어드는 효과는 없다. |
| V-07 결함 허용성과 복구성 | direct/Core 전환 실패는 줄일 수 있다. Core process 장애는 target의 S2S admission과 publication도 막는 공통 한계다. Request Interpreter의 의미 제안만 실패하고 host admission 및 게시가 살아 있는 국소 장애에서는 target의 좁은 직접 경로가 남을 가능성과 A의 전 경로 의존성을 구별한다. 실제 격리 가능성은 미확인이다. |
| V-08 변경 용이성과 모듈성 | VoiceProposal와 direct admission 연동을 제거한다. SpeechRender의 Text/audio 대응 계약 변경은 여전히 Model Access, Response Manager와 전달 검증에 영향을 준다. |
| V-09 분석 및 시험 용이성 | 한 semantic commit에서 게시까지 추적할 수 있다. 전사, 해석, 생성 및 실제 전달 원인은 계속 구분해야 한다. semantic 실패, 늦은 generation과 receipt, 출력 중단을 각각 통제할 시험 경계가 필요하다. |
| V-10 기밀성 | target의 직접 경로는 current-Turn-only다. Core 경로에 필요 이상의 개인 Context를 넣으면 노출 범위가 늘 수 있으므로 최소 자료 준비가 필요하다. |
| V-11 상호운용성과 공존성 | VoiceProposal 없는 runtime의 연동 여지가 생기나 SpeechRender와 정규 근거 지원은 필요하다. Core 경합은 다른 앱 부하에 따라 다르다. |
| V-12 조작 용이성과 사용자 오류 방지 | 사용자에게 경로 선택을 요구하지 않는다. 실제 게시와 audible 범위, 중단과 Task 취소 구별은 같은 UI 계약으로 유지한다. |
| V-13 설치 용이성 | 별도 helper 추가는 없다. 동일 Omni runtime 안의 계약 일부를 줄이는 안이므로 설치 크기 감소는 미확인이다. |

**05로 가져갈 판단:** 단순 자체 지식 대화 비중이 높으면 S2S 포기의 손실이 크다. 맥락 기반 요청이 많고 native VoiceProposal 지원 비용이 높으면 경로 축소의 이점이 있을 수 있다. 실제 빈도는 아직 모른다. 문장 streaming 여부만 바꾸거나 gate를 건너뛰는 안은 별도 구조로 세지 않았다.

<a id="s-06"></a>
### S-06. 의미 추론과 별개인 인식 생산자를 둘 것인가

**문제와 기준선.** Target은 Speech Input Worker의 경량 Streaming ASR로 transcript revision, partial/final과 시각 근거를 생산하며 음성 입력은 Omni에도 전달한다. Model Access의 Shared Inference Service는 한 Omni weights를 역할별 session과 KV로 공유한다. capture와 local stop은 추론 밖이고 독립 ASR의 CPU 비용, 오류와 Omni 해석 불일치 처리도 존재한다. process 분리가 열과 메모리 대역폭까지 격리하는 것은 아니다. [공유 Omni 계약](../target-architecture/shared-omni-runtime.md).

**대안 A — Omni native evidence.** 독립 ASR weights와 Speech Input Worker를 제거하고, Omni 음성 session이 인식 근거까지 생산하게 한다. Native Evidence Adapter는 native transcript/event를 input revision, sample 시간, uncertainty와 gap으로 변환한다. 근거가 없는 시간을 adapter가 만들어내지는 않는다. capture와 local stop은 계속 독립 경로이며 음성/semantic 역할의 session, KV와 권한 격리는 유지한다.

native 인식은 semantic과 같은 inference runtime의 실행 기회에 의존한다. bounded scheduling, 동시 session과 긴 비선점 연산의 실제 한계를 확인해야 한다. API가 비동기라는 사실만으로 지속 인식이 입증되지 않는다. Omni crash 동안 capture가 계속되어도 recognition은 멈추며, 복구 후 유한 backlog와 gap을 구분한다. 계속 녹음했다는 이유로 연속 인식 성공이라 하지 않는다.

U-01/03/08의 native partial/final, revision, 실제 발화 시각과 취소 지원은 미확인이다. target의 독립 ASR 정확도와 지속 처리도 아직 미측정이다. A에서 시각 근거가 없으면 P-02의 해당 지칭 기능 제한, 동시 인식이 불가능하면 B-02의 지속 인식 제한으로 V-03에 남긴다. target과 동등 지원을 주장하지 않으며 숨은 보조 ASR를 넣어 비용을 제외하지 않는다.

| 관점 | A에서 확인할 품질 경로와 조건 |
| --- | --- |
| V-01 기능 정확성 | 독립 전사와 Omni 해석의 충돌은 줄 수 있지만 오류가 같은 모델에 결합된다. 일치율이 높아져도 정확성 개선의 증거가 아니다. |
| V-02 기능 적절성 | native 근거가 충분하면 재질문을 줄일 여지. 부하 때문에 늦거나 유실되면 사용자의 반복 발화가 늘어난다. |
| V-03 기능 완전성 | 지속 인식과 당시 화면 연결 유지 목표. native 시각 및 동시 처리 미지원은 해당 기능 제한이다. |
| V-04 상호작용 반응성 | 별도 ASR 전달 경로는 줄지만 인식이 semantic 연산을 기다릴 수 있다. local stop의 독립 실행과 인식 완료를 구별한다. |
| V-05 VIA 귀속 요청 완료 시간 | evidence 변환 비용 감소 가능성과 공유 inference 대기, gap 후 재처리 비용을 함께 본다. |
| V-06 자원 활용성과 수용량 | ASR weights와 CPU 작업은 제거되지만 Omni의 인식 session, KV 및 연산 부담이 늘 수 있다. 총 메모리와 지원 부하의 이익은 미확인이다. |
| V-07 결함 허용성과 복구성 | ASR 별도 고장은 없어지나 Omni 장애가 인식과 의미 처리를 함께 중단한다. backlog와 근거 gap을 복구해야 한다. |
| V-08 변경 용이성과 모듈성 | ASR 교체 계약은 없어지지만 모델 교체 시 native evidence의 시각, revision, 해석과 Voice 계약을 함께 맞춰야 한다. |
| V-09 분석 및 시험 용이성 | 인식과 의미 오류를 구분할 관측이 더 어려울 수 있다. native event와 sample 대응 자료가 실제 제공되는지 확인한다. Adapter 경계의 event 유실과 역순 시험만으로 실제 inference의 인식 진행을 입증할 수 없으므로 runtime 경합과 중단도 통제할 수 있는지 확인해야 한다. |
| V-10 기밀성 | target도 원음을 Omni에 보낸다. Omni의 새 원음 노출이라는 가짜 차이는 없다. 제거된 ASR buffer와 native session 보관 및 철회 차이를 본다. |
| V-11 상호운용성과 공존성 | native 계약 지원 모델로 선택 범위가 좁아질 수 있다. CPU 감소와 accelerator 경합 증가는 외부 앱 부하별로 다르다. |
| V-12 조작 용이성과 사용자 오류 방지 | 지연된 인식 중 입력이 접수됐는지와 실제 제어 효력을 구별해야 한다. 녹음 표시만으로 정정이 반영됐다고 알리지 않는다. |
| V-13 설치 용이성 | ASR model/runtime 배포를 줄일 수 있다. 대신 필요한 native build 지원과 업그레이드 호환 확인이 남는다. |

**05로 가져갈 판단:** native 계약이 충분하고 공유 runtime이 지원 부하를 처리할 수 있을 때 비용 절감의 설득력이 생긴다. 긴 semantic 처리나 runtime 장애가 입력을 막으면 그 전제는 성립하지 않는다. scheduler 우선순위 값만 달리하는 안은 이 구조와 별개의 DP 후보로 만들지 않는다. [기존 음성 근거안](./speech-evidence-source.md)은 재평가 입력이다.

## 5. 연계 문제와 추가 탐색을 남긴 이유

아래는 구체 대안의 누락을 감추기 위한 ‘공통 처리’ 칸이 아니다. 무엇을 보았고 어디까지 설계하지 않았는지 구별한다. **보류한 아이디어는 품질 동등이나 열등 판정을 받지 않았다.** 실제 대안으로 구체화하면 동일하게 13개 관점을 모두 검토해야 한다.

| 문제와 검토한 지점 | 현재 반영 | 별도 구조 탐색을 넓힐 조건 |
| --- | --- | --- |
| P-07: push event/inbox와 source query, Agent별 실행 capability | S-04에서 workflow signal과 Task 상태, command 전송의 권위를 구별했다. Agent 내부 실행 시간과 지원은 공통 외부 조건이다. | event 없이 source query만으로 상태를 구성하거나 제공자별 실행 package를 두는 것이 V-01/05/08을 크게 바꾼다는 구체 조건이 나오면 별도 구체화. polling 주기만 달리하면 부족 |
| P-08: local stop, 입력 hold, 최종 dispatch 확정 | S-01/04~06에서 취소 통지와 실제 결과 사용 차단, 외부 전송을 구별했다. 어떤 안도 녹음 시작부터 host hold까지 지연이 없다고 가정하지 않는다. | 추론 또는 durable 저장이 local control을 막는 의존성이 발견되거나 dispatch 권위를 다른 실행체로 옮길 실익이 확인되면 심화 |
| P-09: 게시 원장과 실제 UI/audio 전달 | S-05에서 생성 경로를 바꾸되 Response Manager의 publication과 Interaction Manager receipt를 보존했다. 전달 receipt 불명은 제거되지 않는다. | client 측 내구 Delivery Agent가 게시 및 재개 권위를 소유하는 구조는 별도 가능성. Core/Voice 단절에서도 출력 진행을 유지할 필요와 양쪽 상태 조정 비용을 확인한 뒤 구체화 |
| P-10: Conversation/Task 원본과 session/KV 수명 | S-03은 과거 자료의 파생 검색, S-04는 대기 수명, S-05는 Task 없는 답변의 원본을 대조했다. 검색 index나 모델 KV를 대화 원본으로 바꾸지 않았다. | 장기 대화의 독립 projection이 기존 cache/summary와 달리 복구 및 조회 의존성을 바꾼다는 사례가 있으면 확대 |
| P-11/12: 권한, 기억 삭제와 늦은 파생 결과 | S-01 Broker, S-02 객체 이력, S-03 index, S-04 continuation, S-05/06 session의 현재 정책과 삭제 비용을 반영했다. 읽기, 지속 보관, 외부 제공 허용은 서로 다르다. | 독립 capability 실행 경계나 별도 memory authority가 현재 전파 구조보다 중요한 차이를 만들면 별도 설계. 검사 생략이나 무조건 영구 보관은 강한 대안이 아님 |
| P-13: 복구 권위와 실제 외부 상태 | S-04에서 current records, continuation과 휘발 상태를 대조했다. [기존 event-sourced 복구안](./recovery-state-source.md)은 독립 선택으로 보존했다. 지금은 새 DP로 승계하지 않는다. | 여러 owner의 상태를 같은 사건에서 재구성해야 하는 변화가 핵심이면 journal/projection 원본 구조를 심화. 현재 target에도 durable inbox/outbox가 있으므로 ‘기록 없음 대 기록 있음’은 아님 |
| P-15: 제공자 의미와 저장 version 변화 | 모델의 read callback/native evidence, UI source schema, embedding 재색인, 진행 workflow version과 VoiceProposal 제거의 실제 변경 경로를 각 V-08에 적었다. | 제공자 전용 실행 package와 공통 canonical 계약의 차이가 위 질문들로 설명되지 않으면 독립 탐색. 파일 수가 아니라 영향을 받는 계약과 진행 상태로 판단 |
| P-16: 최소 추적과 평가 근거 | input revision → 근거/파생 version → 확정 → command/실행 → publication/receipt 연결이 필요하다. 확대 원문 기록은 동의와 수명을 전제로 한다. 신규 trace schema나 평가 runner는 만들지 않았다. | 원인 구분에 필요한 사건을 기존 실행 경계에서 얻을 수 없거나 관측이 V-04/05 경로를 막는 경우 별도 수집 구조를 심화 |

### 5.1 별도 구조로 세지 않은 변화

| 살펴본 변화 | 현재 판단 |
| --- | --- |
| semantic 2회를 3회로, 추가 읽기 개수나 scheduler 우선순위를 변경 | 정책 효과일 수 있으나 단독으로 실행체, 상태 권위나 의존 구조가 달라지지 않는다. S-01/06 구조와 설정 효과를 섞지 않는다. |
| Component 이름 교체, Request Controller와 Task Manager의 단순 합침 | 새 데이터 획득, 실행 권위나 복구 계약이 설명되지 않으면 독립 안으로 세지 않는다. |
| 화면 sampling 빈도, vector 검색 알고리즘만 변경 | 같은 producer 및 저장 구조 안의 조정이다. S-02/03의 실제 서브시스템 대체와 구별한다. |
| VIA가 모든 복합 요청을 Agent의 내부 도구 계획으로 분해 | B-01의 책임 범위 변경이다. 기능 일부 축소와 달리 VIA의 일반 업무 실행을 새로 추가하므로 현재 탐색 범위 밖이다. |
| role마다 Omni weights를 복제하거나 숨은 cloud로 인식 | B-02의 공통 배치 경계를 바꾼다. 현재 대안의 비용을 낮춘 것처럼 사용하지 않는다. |

## 6. 조합 관계와 다음 검토

### 6.1 독립적인 차이와 함께 움직이는 비용

| 연결 | 구별해야 할 것 |
| --- | --- |
| S-01 + S-03 | Worker가 읽기를 진행하는 것과 의미 후보를 미리 저장하는 것은 다른 선택이다. S-01에 vector index를 전제하지 않고, S-03을 target의 host 제어에서도 사용할 수 있다. 함께 쓰면 읽기 축소 효과를 두 번 세지 않는다. |
| S-02 A + S-06 | background vision과 native recognition이 한 Omni runtime을 공유한다. 객체가 미리 준비되고 인식도 항상 즉시 진행된다고 각각 독립 가정하면 모순이다. |
| S-04 + 복구 원본 | workflow continuation의 저장과 전체 시스템 event sourcing은 같은 선택이 아니다. workflow 도입만으로 Task 및 publication까지 event-sourced가 되지 않는다. |
| S-04 B + S-03 | 파생 index에 미완료 요청을 저장해 B의 잃은 그래프를 복원하면 사실상 다른 내구 구조를 다시 도입한 것이다. B의 단순화 이익으로 숨기지 않는다. |
| S-05 + S-06 | Core-only에서도 native 음성 근거 생산을 위해 Omni 음성 session은 필요할 수 있다. S2S 출력 경로 제거가 Omni 음성 연산 전체 제거는 아니다. |
| 모든 S + P-11/12/15/16 | 새 저장과 실행체마다 현재 권한, 삭제, version 이행, 최소 추적이 따라간다. 공통 기반을 여러 후보의 독립 이익으로 중복 계산하지 않는다. |

### 6.2 이번 리뷰에서 결정할 질문

1. 여섯 질문이 상위 품질에 직접 차이를 만드는 구조를 충분히 드러내는가? 특히 P-08, P-09와 P-15를 연계로 둔 판단에 빠진 큰 구조 차이가 있는가?
2. S-01은 실행과 중간 상태의 차이가 충분한가? 실제 runtime 지원을 확인하기 전에도 합리적인 강한 대안으로 구체화할 가치가 있는가?
3. S-02 B, S-04 B, S-05 A의 기능 손실과 사용자 부담이 분명한가? 모두 완전한 안으로 만들지 않고 비교에 남길 이유가 있는가?
4. 각 안에 유리한 조건뿐 아니라 비용과 가설이 무너지는 조건이 드러나는가? native runtime, helper, 객체 추적과 workflow를 무료 capability로 둔 곳은 없는가?

04의 결과는 **집중 구조 질문 6개와 target 밖 대안 8개, 그리고 명시한 추가 탐색 영역**이다. 6개 DP나 8개 방안 2를 선정한 것이 아니다. 05에서는 리뷰를 반영해 강한 대안을 구체화하고 실제 target과 조건별 품질 차이를 비교한다. 새 QA 정의와 측정 방법, 수치 목표, freeze, 구현 또는 모델 실행은 아직 시작하지 않았다.
