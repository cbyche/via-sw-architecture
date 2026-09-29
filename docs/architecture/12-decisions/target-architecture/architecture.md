# VIA 목표 SW Architecture

> 상태: **PROPOSED / 사용자 검토 중 / 구현·측정 없음**
> 작성일: 2026-09-29
> [설계 개요](./README.md) · [합의 상태와 검토 기록](./review-log.md)

## 1. 한 문장 정의

**VIA는 시간에 맞춰 수집한 근거로 사용자의 목표·대상·Task를 먼저 정확히 확정하고, 그 확정 경계를 지키면서 직접 응답과 장기 Agent 업무를 빠르게 이어 주는 PC interaction·orchestration 소프트웨어다.**

세 부분은 동시에 움직이되 사용자에게 무엇을 답하고 어떤 업무를 시작할지는 검증된 요청과 확인된 상태를 기준으로 결정한다. 이하의 구조는 주 설계안이며, 성능상 이점은 아직 검증되지 않은 가설이다.

설계 우선순위는 **QA-19 semantic accuracy → QA-09 responsiveness → QA-29 modifiability → QA-39 reliability/recoverability**다. 낮은 순위도 선택 사항은 아니며, 상위 품질을 이유로 구조적 변경 파급이나 복구 실패를 숨기지 않는다.

이 Architecture는 다음을 invariant로 둔다.

1. 잘못된 대상·Task·승인·Agent command를 확정하거나 게시하지 않는다.
2. 필요한 근거가 없거나 후보가 충분히 탐색되지 않았으면 추가 조회, clarification 또는 확인 불가로 끝낸다.
3. 위 정확성 경계를 낮추지 않고 사전 준비·병렬 조회·speculative 생성·선택적 재검증으로 latency를 줄인다.
4. Model·Agent·Context source별 차이는 versioned canonical contract와 adapter 뒤에 두고, 변경이 무관한 책임·상태로 번지지 않게 한다.
5. 외부 장애와 process crash는 durable intent·inbox·projection·publication 기록으로 격리하고, 확인되지 않은 실행·상태·전달을 성공으로 복원하지 않는다.

## 2. 사용자 관점의 전체 흐름

사용자가 그래프를 가리키며 “이거 지난번 발표자료에 넣어줘. 보고서 만드는 건 계속하고”라고 말한다.

1. 말하는 동안 그래프를 지정한 시점과 화면 근거를 확보한다.
2. 그래프와 이전 발표자료를 식별하고, 유지할 보고서 Task를 구분한다.
3. 후보가 둘이면 구분에 필요한 질문을 한다. 대상을 확정할 수 있으면 목표·완료 조건·제약을 Agent에 전달한다.
4. Agent 실행 중에도 사용자는 다른 질문을 한다.
5. 진행·질문·결과는 원래 Task와 Conversation으로 돌아온다.
6. 사용자가 VIA 음성 중 끼어들면 재생은 즉시 중단하고, 업무 취소 여부는 새 발화의 의미를 확인해 결정한다.

| 처리 흐름 | 계속 처리하는 일 | 기다리지 않는 것 |
| --- | --- | --- |
| 실시간 interaction | 입력, 화면 증거, 음성 재생·중단 | semantic 해석, Agent 업무 실행 |
| 요청 이해와 확정 | 목표·대상·Task 관계·처리 방향 | 무관한 Task의 진행·완료 |
| 장기 업무 관리 | 위임, 진행, 질문, 제어, 결과·복구 | 사용자의 다음 발화 |

## 3. 전체 구조

![VIA 전체 목표 Architecture](./diagrams/01-system-overview.svg)

[draw.io 편집 원본](./diagrams/01-system-overview.drawio)

그림은 **VIA 소속, 논리적 책임, 외부 의존성**을 구분한다. 큰 VIA 경계 안의 실선 박스는 같은 수준의 논리 Component다. 위쪽은 interaction·orchestration, 아래쪽은 공통 저장·모델 연동 서비스다. 외부의 모델·Agent·Context source는 VIA가 내부 동작을 설계하지 않는 의존성이다. 외부 책임이라는 표시는 원격 배치를 뜻하지 않으며, 프로세스 배치는 §14에서 따로 다룬다.

핵심은 Request Controller가 입력과 근거를 결합하고 처리 권한을 확정하는 것이다. Interpreter는 의미를 제안하고, Context Manager는 근거를 제공하며, Policy Manager는 허용 범위를 판단한다. Controller가 그 결과를 적용한다. Policy Manager가 Response Manager를 직접 지휘하거나 Interpreter가 Agent를 호출하지 않는다.

| 그림의 연결 | 전달하는 계약과 책임 |
| --- | --- |
| ① Interaction Manager → Controller | 입력 시작·revision·확정·중단 이벤트와 시점별 evidence 참조; raw audio chunk 전달 경로가 아님 |
| ② Controller ↔ Context Manager | 허용된 기본 준비·추가 조회 요청과 후보·근거·조회 receipt |
| ③ Controller ↔ Interpreter | 확정 입력·근거를 사용한 해석 요청과 semantic proposal; 추가 읽기는 제안으로 반환 |
| ④ Controller ↔ Policy Manager | Context 접근·외부 제공·consent의 현재 허용 범위 확인; Controller가 읽기·처리 경로에 적용 |
| ⑤ Controller → Task Manager | 검증한 목표·대상·제약·Task 관계와 dispatch admission |
| ⑥ Task Manager → Agent Gateway | 생성·기록한 Agent Command; Gateway가 capability·revision 조건을 확인하고 Agent로 전송 |
| ⑦ Agent Gateway → Task Manager | Agent에서 수신·기록한 event; Task Manager가 순서·중복·correlation을 검증해 업무 상태 반영 |
| ⑧ Task Manager → Controller | 확인된 진행·결과·실패·질문; Controller가 원래 Conversation·Request와 결합 |
| ⑨ Controller → Response Manager | 게시를 허용한 payload·출처·출력 소유권; Response Manager는 실제 게시 사실을 반환 |
| ⑩ Response Manager ↔ Interaction Manager | 표시·재생·release/cancel 명령과 generation handle·실제 전달 receipt |

번호는 연결을 설명하기 위한 ID이며 항상 순서대로 실행하는 단계가 아니다. ②~④는 입력 준비·추가 근거 확보·확정 시점에 사용하고, 이미 실행 중인 Agent의 결과는 새 사용자 발화 없이 ⑦~⑩로 돌아온다. Controller의 결합·검증은 모든 Agent progress에 semantic LLM을 다시 호출한다는 뜻이 아니다.

**State Store는 VIA 내부의 공유 저장 서비스다.** Conversation·Task뿐 아니라 Context memory·policy·송수신·응답 기록을 함께 지원하므로 semantic Control Plane에만 속하게 두지 않는다. 저장 대기가 dispatch·publication의 latency 경로에 포함될 수 있다. 영속 데이터는 Core process 종료 후에도 남아야 하지만, 별도 DB process를 강제하는 것은 아니다. 각 상태 소유자가 repository·transaction port를 통해 자기 상태를 기록한다.

점선 `S`와 `M1~M3`는 위 Component들의 공통 서비스 접근 관계를 펼쳐 놓은 표기다. 추가 Component, 모델 복제본 또는 메시지 bus가 아니다. S는 명시된 상태 소유자들의 Store 접근이며, M1은 Interaction Manager의 S2S stream, M2는 Interpreter의 semantic 해석, M3는 Response Manager의 필요한 요약·음성 생성이다. 모두 하나의 Model Access를 통해 같은 S2S 1개·semantic LLM 1개를 사용한다.

박스 색은 책임, 선 색은 전달 계약의 영역을 뜻한다. 파랑은 interaction·publication, 보라는 의미 제안·확정, 청록은 evidence·Context, 노랑은 policy·consent, 초록은 Task 상태·결과, 주황은 Agent command, 회색은 저장·모델 연동이다. 초록 event도 수신 즉시 확정 사실이 되는 것은 아니다. `→`는 전달 방향, `↔`는 조회·반환처럼 양쪽 메시지가 있는 protocol, 점선은 공통 서비스 접근 표기다. 양방향은 공동 상태 소유를 뜻하지 않는다. 선 두께에는 추가적인 권한·우선순위 의미가 없다.

## 4. Component와 상태 소유권

| Component | 책임 | 소유 상태 |
| --- | --- | --- |
| Interaction Manager | Channel I/O, 화면 evidence capture, 입력·근거 timeline과 speculative generation handle 관리; Model Access의 S2S client | 사용자·device session, 입력 stream, 짧은 evidence buffer, clock mapping, playback·barge-in·generation handle 상태 |
| Request Controller | Turn 수신, Request Graph, 의미 확정, 정정, 모든 사용자 질문·승인의 결합, 게시·dispatch admission | Conversation, Turn, Request Graph, semantic revision·commit, Pending User Interaction |
| Context Manager | 후보 탐색, bounded read, 근거 package·coverage, cache, 허용된 User Memory | Context cache, query receipt, 관측한 source revision, 기억과 삭제 상태 |
| Request Interpreter | 목표·대상·복합 관계·Task 관계·처리 방향의 후보와 field별 근거 제안 | 호출 중 임시 상태; `READY`·업무 상태·게시 권한의 권위는 없음 |
| Task Manager | 지속 업무와 Agent Execution projection, 진행·제어·복구 | Task, Execution 연결, source-confirmed 업무 상태 |
| Agent Gateway | Agent별 계약 변환, capability 확인, command 전송·event 수신·조회 | 연결 상태, capability profile, outbox 전송 상태, inbox cursor |
| Response Manager | 하나의 확정 payload를 Text·Voice·알림으로 게시하고 실제 전달 범위를 기록 | publication outbox, 출력 소유권, 채널별 전달 상태 |
| Policy Manager | Context 접근·제공·기억 사용 통제 | 현재 권한, consent 범위·revision |
| State Store | transaction, 송수신 기록, 근거 참조, 복구 저장 | 영속 데이터; 상태의 의미는 각 소유 Component가 결정 |
| Model Access | S2S 1개와 semantic LLM 1개의 provider session·adapter, 호출 우선순위, timeout·취소·예산 | 모델별 queue, provider session·cache 관리 정보 |

Turn Workspace는 Controller 내부의 요청별 작업 데이터다. Decision Validator는 Controller의 확정 절차에 포함한다. 별도 서비스로 분리해 매 요청에 추가 왕복을 강제하지 않는다.

Interaction Manager는 기존 Interaction Runtime의 이름을 바꾼 것이다. 여러 내부 모듈로 구현하지만 전체 그림에서는 다른 Manager·Controller와 같은 논리 Component 수준으로 표시한다. Interpreter의 prompt·schema 처리나 Gateway의 provider adapter를 전체 그림에서 펼치지 않는 것과 같은 원칙이다.

| 내부 모듈 | 책임 | 금지되는 권한 |
| --- | --- | --- |
| Channel I/O | Voice·Text·UI 입출력, microphone·playback, barge-in, 실제 표시·재생 | 응답 내용·routing·Task 확정 |
| Evidence Capture | 화면·window·focus·pointer·selection revision 수집 | 지칭 대상의 의미 확정 |
| Timeline & Buffer | 입력 revision과 evidence 시각 결합, gap·clock mapping, S2S transcript·speculative generation handle 연결 | speculative 응답의 게시 허용 |

S2S provider 연결 자체는 Model Access가 소유한다. Interaction Manager는 Model Access client로 audio·control stream을 주고 transcript·generation handle을 받는다. 이 구분으로 device·OS adapter 변화와 model provider 변화가 같은 모듈에 섞이지 않게 한다. Model Access는 모델을 복제하거나 내부에 새 모델을 정의하는 Component가 아니라 VIA와 모델 runtime 사이의 adapter다.

State Store에 저장한다고 상태 소유권이 Store로 넘어가지 않는다. Aggregate별 단일 writer를 유지한다. Controller는 Task 변경을 Task Manager에 요청하고, Task Manager는 Gateway의 durable inbox event를 검증한 뒤 projection을 바꾼다. 여러 상태를 함께 확정해야 할 때는 owner가 State Store transaction을 요청한다.

Component 경계는 배포 단위가 아니라 변화와 상태 권한을 가두는 논리적 port다. S2S·semantic model provider 차이는 Model Access, Context source 차이는 Context Manager의 source adapter, Agent protocol 차이는 Agent Gateway adapter에서 canonical contract로 변환한다. Canonical schema와 persisted state는 version·migration 규칙을 가지며, provider 교체가 Request·Task·Response 의미 계약까지 직접 바꾸지 않게 한다. 공통 contract 자체가 바뀌면 영향받는 Architecture Element를 숨기지 않고 migration과 compatibility 범위를 명시한다.

## 5. Conversation·Turn·Request·Task·Execution

![Conversation, Request, Task와 Agent Execution의 수명 및 소유권](./diagrams/02-lifecycle-and-ownership.svg)

[draw.io 편집 원본](./diagrams/02-lifecycle-and-ownership.drawio)

복합 Turn은 Controller가 소유하는 durable `Request Graph`가 된다. 각 node는 독립 Request ID·handling·상태·입력과 결과 version을 가지며, edge는 `independent`, `sequential`, `data-dependent`, `conditional`과 source result version을 기록한다. Task Manager는 node가 Task에 연결된 뒤의 Task lifecycle만 소유한다.

- Conversation은 대화와 참조 관계를 유지한다. Voice 연결 종료나 새 대화 시작을 Task 종료로 취급하지 않는다.
- Turn은 한 번의 사용자 입력이며, 여러 Request를 포함할 수 있다. 대상이 여럿이라는 이유만으로 Request를 나누지는 않는다.
- Request는 현재 처리할 논리적 요청이다. 직접 응답에는 Task가 없어도 입력·답변·근거·대상은 남는다.
- Clarification 답변은 새 Turn이지만 원래 Request의 부족한 정보를 채운다. 이미 확정된 대상과 제약은 유지한다.
- 같은 목표·결과물의 수정은 기존 Task, 이전 결과를 참고한 별도 목표는 새 Task로 연결한다.
- Task는 사용자 업무 identity, Execution은 실제 Agent 실행이다. 동일 Task에 후속 Execution을 연결할 수 있다.

Task 상태는 업무 단계(준비·실행·입력 대기·완료·실패·취소), 제어 요청(수정·취소 요청 중), 확인 상태(확인됨·재조회 중·확인 불가)를 구분한다. 취소 접수와 취소 완료, 연결 단절과 업무 실패를 혼동하지 않는다.

VIA clarification, Agent 질문, Context consent와 Action approval은 모두 `Pending User Interaction`으로 등록한다. 이 기록은 interaction ID, Conversation·Turn·Request, 선택적 Task·Execution·question, 허용 답변, 생성 revision, policy revision과 만료 조건을 가진다. 짧은 답변을 어느 질문에 적용할지는 Controller 한 곳이 결정하며, 유일하게 결합할 수 없으면 다시 묻는다.

Request는 `RESOLVING`, `WAIT_CONTEXT`, `WAIT_USER`, `WAIT_DEPENDENCY`, `READY`, `HANDLING`, `COMPLETED / FAILED / CANCELLED / SUPERSEDED`를 구분한다. Direct Response와 내부 기억 변경은 Request만으로 처리하며 이를 위해 별도의 VIA-local 장기 Execution을 만들지 않는다. Agent 위임 시에는 지속 추적할 Task를 연결한다. Request의 접수·위임 완료와 사용자 업무의 Task 완료를 같은 상태로 표현하지 않는다.

새 Conversation을 열어도 기존 Task를 취소하거나 새 대화에 자동 이관하지 않는다. Agent 알림은 원래 Conversation과 Task view로 연결하고, 새 대화에서 기존 Task를 지칭하면 명시적인 참조 관계를 추가한다. Voice 재연결은 현재 Conversation을 다시 연결할 뿐 Request·Task를 재실행하지 않는다.

## 6. 요청 이해와 처리 확정

![Evidence 수집부터 Semantic Commit까지의 요청 확정 흐름](./diagrams/03-request-resolution.svg)

[draw.io 편집 원본](./diagrams/03-request-resolution.drawio)

입력 중에는 근거 수집과 값싼 Context 준비를 겹쳐 수행한다. 주 semantic 해석은 확정된 입력 revision을 사용한다. 모든 partial마다 모델을 호출하지 않는다. 발화 종료 시점의 화면 하나로 전체 발화의 지칭을 처리하지 않고, 표현별 시점의 증거를 사용한다.

### 음성 stream과 요청 이벤트의 경계

| 입력·이벤트 | 실제 경로와 동작 |
| --- | --- |
| Audio chunk | Interaction Manager가 Model Access를 통해 S2S에 연속 전달한다. Controller·Context Manager로 매 chunk를 중계하지 않는다. |
| InputStarted | Controller가 provisional Turn과 입력 revision을 만들고 현재 권한 안에서 값싼 기본 Context 준비를 요청한다. |
| 화면·선택·transcript revision | Interaction Manager가 timeline과 buffer에 기록한다. Controller에는 변경·gap·revision 참조를 전달하며 중간 알림은 묶을 수 있다. 원본 시점 근거는 합쳐 없애지 않는다. 관련 identity·선택·source가 바뀐 경우에만 기본 Context를 갱신한다. |
| InputFinal | 최종 입력 revision과 해당 구간 evidence 참조를 고정해 Controller에 전달한다. Interpreter는 이 revision으로 해석한다. 아직 partial transcript라면 확정 입력으로 취급하지 않는다. |
| 추가 근거 필요 | Interpreter가 부족한 field와 bounded read를 제안한다. Controller가 권한·예산을 확인해 Context Manager에 요청하고 제한된 재해석으로 돌아온다. |
| Barge-in / 새 입력 | Interaction Manager가 현재 재생을 즉시 멈추고 출력 세대를 무효화한다. Controller에는 새 입력·보류 신호, Response Manager에는 중단 receipt를 비동기로 전달한다. 재생 중단은 semantic 해석이나 게시 취소 승인을 기다리지 않는다. |

예를 들어 “이 그래프를… 아니, 저 표를 넣어줘”에서는 두 지칭 시점의 근거를 보존하지만 audio packet마다 자료를 검색하지 않는다. 최종 발화에서 정정한 대상을 해석하고 확정 전에는 위임하지 않는다. 수정 발화가 기존 command의 전송과 경쟁하면 §11의 dispatch 경계를 적용한다. S2S가 partial transcript를 제공하지 않는 경우에도 audio·화면 시점 근거를 보존하고 최종 transcript에 결합해야 하며, 그 기능 확보는 별도 검증 대상이다.

Interpreter는 목표·대상·Task 관계·처리 방향과 경쟁 후보를 함께 제안한다. 업무 수행 순서나 tool 계획을 새로 만들지 않는다. `READY` 여부는 모델이 선언하지 않고 Controller가 field별 상태와 다음 조건으로 계산한다.

1. 최신 입력·정정 revision인가?
2. 대상·Task가 실제 근거에 연결되고 필수 정보·명시된 제약이 보존됐는가?
3. 후보 탐색 범위·truncation·실패 source가 기록됐고, Action 대상의 coverage가 충분하며 동등 후보가 남지 않았는가?
4. Context 읽기·외부 제공 권한이 유효한가?
5. 해석이 의존한 Conversation·Task 후보·자료·policy revision이 바뀌었는가?
6. 요청한 Agent capability와 실행 전제조건을 사용할 수 있는가?

사용자 업무의 dispatch에는 `Semantic Commit`이 필요하다. Host 검증은 사용자의 진짜 의도를 증명하지 못하므로, 필수 field가 `RESOLVED`이고 admissible evidence가 있으며 충돌·미해결·coverage 불완전·stale dependency가 없을 때만 commit한다. 특히 외부 Action은 불완전한 후보 집합에서 기본값을 선택하지 않는다.

응답 게시는 Controller의 publication admission을 요구한다. 그 근거는 확정 해석, §8의 Direct Admission, 또는 Task Manager가 확인한 Agent 상태·질문일 수 있다. Clarification·근거 확보 실패 안내는 미확정 field를 그대로 보존해 게시하며, 질문하려고 업무 의도를 억지로 commit하지 않는다. 이미 위임한 업무의 progress를 알릴 때에도 새 Semantic Commit을 만들지 않는다.

## 7. Semantic LLM 계약과 호출 예산

공유 semantic LLM 1개가 목표·대상·Task 관계·routing을 통합 해석한다. 각 판단마다 필수 별도 모델 호출을 두지 않는다.

| 입력 | 내용 |
| --- | --- |
| 사용자 입력 | 원문, 확정 revision, 필요한 발화 시각 |
| 기본 근거 | 선택·화면·자료 식별 정보, 원문 또는 시각 근거, 출처 |
| 대화·Task | 관련 원문·확정 대상·대기 질문·업무 후보·확인된 상태 |
| 기능 범위 | 직접 처리 범위, Agent capability |
| 제한 | 허용 source, 조회·시간·출력 예산 |

| 출력 | 필수 의미 |
| --- | --- |
| Semantic Proposal | subrequest·관계, 목표·완료 조건·제약, referent·Task·handling 후보, 경쟁 후보와 배제 근거 |
| Field Resolution | field별 `RESOLVED / AMBIGUOUS / MISSING / N/A`, 값의 origin, evidence ID, freshness 요구, 충돌·미해결 사항 |
| Next Evidence Request | 부족한 field와 허용 source에서 읽어야 할 대상·범위; 실행 여부는 host가 결정 |
| Clarification Proposal | 이미 확정된 내용과 사용자만 구분할 수 있는 최소 차이; 실제 질문 등록은 host가 결정 |

**초기 운영 가설은 확정된 입력 revision당 semantic 호출 최대 2회**다. 첫 해석 후 필요한 bounded read를 한 묶음 수행하고 재해석한다. 두 번째에도 필수 field가 해결되지 않으면 clarification 또는 근거 확보 실패로 끝내며 추측해 commit하지 않는다. 이 횟수는 Architecture invariant가 아니라 accuracy·불필요한 clarification·latency를 함께 측정해 바꿀 수 있는 resource policy다. Constrained output 또는 형식 오류 처리는 semantic refinement와 구분하되 전체 deadline 안에서 제한한다.

직접 답변은 확정 전 speculative하게 생성할 수 있지만, decision envelope와 필수 proposition이 semantic commit에 일치한다고 확인한 뒤 게시한다. 긴 답변이 구조화 판단의 확정을 막거나 interactive 요청 queue를 점유하지 않도록 응답 생성 예산을 분리한다. Agent의 긴 결과 요약에는 별도 응답 생성 호출 1회를 허용하며 이를 해석 비용에 숨기지 않는다. 상태 template으로 충분한 progress·completion 안내에는 semantic LLM을 호출하지 않는다.

Core 직접 답변도 짧고 근거가 충분하면 첫 semantic 출력에 답변 초안을 함께 포함하고, 추가 구성·긴 요약이 필요하면 Response Manager가 **별도의 생성 호출 최대 1회**를 사용한다. 이는 해석 최대 2회와 별도 비용이며 같은 전체 deadline·공유 모델 queue를 사용한다. 형식 repair·stale 재검증을 새 호출 종류로 이름만 바꿔 무한 재시도하지 않는다. 재생 전 새 정정 revision이 연속 도착하면 이전 호출을 취소·폐기하고 최신 입력에 합친다.

LLM이 요청하는 도구는 bounded read-only Context 도구뿐이다. Host가 허용·실행하며 LLM은 권한, Task 상태, 외부 실행 도구를 직접 변경하지 않는다. 일반적인 자유 실행 ReAct loop로 확장하지 않는다.

## 8. S2S와 직접 응답

| 경로 | 대상 | Semantic LLM |
| --- | --- | --- |
| S2S 직접 응답 | 외부·개인·화면·과거 대화·Task·Action·최신성·복합 관계가 필요 없는 self-contained Voice 질문 | admission 판정 방식에 따라 생략 가능 |
| Core 처리 후 음성 전달 | 화면·자료·Task 판단, clarification, 위임, 진행·결과 | 필요한 해석·구성에 사용 |

S2S는 Model Access를 통해 speculative 응답 생성을 수행할 수 있지만 스스로 게시 권한을 갖지 않는다. Channel I/O가 audio stream을 Model Access에 보내면 Model Access는 transcript revision과 speculative generation handle을 Timeline & Buffer로 돌려준다. Handle은 입력 revision·session·출력 세대에 결합하며 audio는 Interaction Manager의 buffer에 보류한다. Interaction Manager는 이 handle을 Input + Evidence Record와 함께 Controller에 전달할 수 있지만 승인 전에는 재생하지 않는다.

Controller는 모든 입력에 Request identity를 만들고 직접 경로의 허용 여부를 `Direct Admission Record`로 기록한 뒤 한 경로에만 응답 소유권을 부여한다. 허용한 경우 generation handle과 확정 proposition을 Canonical Response Payload에 묶어 Response Manager에 보낸다. Response Manager가 handle·Request·input revision·출력 세대를 확인해 Interaction Manager에 release 또는 cancel을 명령하고, Interaction Manager는 실제 표시·재생·중단 receipt를 돌려준다. 따라서 직접 S2S 응답도 `Model Access ↔ Interaction Manager ↔ Response Manager` 전달 protocol과 `Controller → Response Manager` admission을 모두 지난다. 외부 근거·개인 자료·화면 지칭·과거 대화·Task 관계·Action/control·최신성·복합 관계의 가능성이 하나라도 남으면 Core로 보낸다. admission 전 audio는 재생하지 않고, 기각한 generation은 폐기한다.

허용된 S2S 응답도 Text·audio generation과 사용한 근거를 같은 Response Record에 남긴다. 동일 Request에 S2S와 Core가 중복 응답하지 않으며 기록을 위해 재실행하지 않는다. Text 일반 질문에는 S2S 경유를 강제하지 않는다. 이 gate가 정확도를 지키면서 실제 latency 이점을 남기는지는 아직 검증되지 않았다.

현재 기본 정책의 **제안**은 S2S 답변을 재생성하지 않되, 최초 semantic 호출에서 직접 처리 가능 여부·입력 의미·답변 scope를 확인하는 것이다. Host가 단순히 schema를 확인하거나 S2S의 자기 판정을 믿는 것만으로 routing 정확성을 보장하지 않는다. Semantic 확인을 생략하는 fast path는 별도의 admission 근거가 마련될 때 제한적으로 허용하는 방향이며, 이 기본 정책은 사용자에게 확인 중이다. 이 확인 호출은 §7의 해석 예산에 포함하고 latency에도 포함한다.

### 확보해야 하는 모델 기능

- S2S: 입력 기록, 지칭 시각 근거, host 출력 제어, barge-in, 지정한 응답 의미를 보존하는 음성 생성.
- 공유 semantic LLM: 구조화된 요청 해석. UI 구조 정보가 없는 이미지 화면까지 이해하려면 시각 근거 처리 기능.
- Core의 응답도 같은 S2S로 음성화한다. 독립 ASR·OCR·TTS/helper 모델을 몰래 추가하지 않는다.

이 기능이 실제 dependency에 확보되었다는 주장이 아니다. 시각 입력이나 시간 근거를 제공하지 않는 모델로 전 기능이 가능한 것처럼 설명하지 않는다. 제품 모델 선정·연결 전 capability 확인이 필요하다.

## 9. Context·cache·stale 처리

![발화와 화면 evidence를 연결하는 Interaction Evidence Timeline](./diagrams/04-interaction-evidence-timeline.svg)

[draw.io 편집 원본](./diagrams/04-interaction-evidence-timeline.drawio)

| 범위 | 기본 준비 | 필요 시 조회 |
| --- | --- | --- |
| 화면·선택 | foreground, 문서 identity, 선택, pointer·focus 시점 | 관련 영역, UI 구조, 원문 |
| 대화 | 최근 원문, 확정 대상, 대기 질문 | 더 오래된 관련 대화 |
| Task | active·최근·현재 표현과 관련된 Task 요약·결과물 참조 | 상세 상태, 과거 업무 검색, 최신 Agent 상태 |
| 파일·메일·일정 | 이미 연결된 자료 metadata | 한정된 검색·본문 |
| 기억 | 허용된 관련 선호 | 특정 기억 상세 |
| Agent | capability·연결 상태 | 실행별 상세 상태 |

기본 준비도 현재 권한 안에서 수행한다. Context는 자료 identity, 필요한 원문·이미지와 출처·revision을 묶는다. 요약 하나가 원문과 후보의 충돌을 모두 대체하지 않는다.

- cache key는 자료 identity·revision·접근 범위를 포함한다.
- 변경 알림으로 무효화하고 알림 없는 source에는 유효 기간·재조회 조건을 둔다.
- 후보 탐색은 query, source 집합, 범위, 전체·반환 후보 수, truncation, 실패 source, 제외 후보와 이유를 `Evidence Query Receipt`에 남긴다.
- Action 대상의 bounded completeness가 불명확하거나 필수 source가 실패했으면 `Semantic Commit`을 금지한다. read-only 답변도 누락·staleness가 결론에 영향을 주면 한계를 표시하거나 clarification한다.
- Interpreter가 실제 의존한 receipt ID의 `Context Read Set`만 dispatch 전에 재검증하여 무관한 source 변화로 전체 요청을 다시 해석하지 않는다.
- 권한 철회·기억 삭제 시 관련 cache와 모델 session 재사용을 중단한다. 지속 기억은 명시적 허용과 확인·수정·삭제를 지원한다.
- 짧은 화면 buffer를 유지하고 현재 요청이 참조한 구간을 필요한 수명 동안 보존한다. 전체 화면·음성을 영구 보관하지 않는다.
- 증거 유실·buffer 초과는 공백으로 표시한다. 현재 화면으로 과거를 복원한 것처럼 채우지 않는다.

발화 중 지칭은 `Interaction Evidence Timeline`으로 연결한다. 각 evidence event는 producer, monotonic sequence, clock domain, capture·receive time, clock mapping uncertainty와 gap을 가진다. Referential utterance span은 final transcript revision, acoustic interval, display·window·document·viewport identity, screen/UI revision, pointer·selection interval과 candidate referent set을 가리킨다. Partial transcript가 바뀌거나 event watermark 전후로 늦은 evidence가 들어오면 영향을 받은 span과 field만 무효화한다.

**당시 무엇을 가리켰는가**와 **지금 그 대상에 적용 가능한가**를 나눈다. Scroll만 바뀌면 과거 지칭을 유지하고, 대상의 내용·identity가 변경·삭제되면 관련 해석을 재검증한다. 무관한 화면 revision 변화로 전체 요청을 재해석하지 않는다.

### Context 사용 권한과 기억의 수명

![Context 읽기, 외부 제공, 권한 철회와 User Memory 삭제](./diagrams/10-context-policy-and-memory.svg)

[draw.io 편집 원본](./diagrams/10-context-policy-and-memory.drawio)

Context Manager는 Conversation·Task·capability를 임의로 소유하거나 변경하지 않는다. Controller·Task Manager·Gateway의 versioned read port에서 snapshot을 읽어 Evidence Package를 만든다. snapshot마다 source revision을 남기며 여러 owner를 읽은 결과를 하나의 동시 snapshot이라고 가정하지 않는다. 관련 revision의 일관성은 commit에서 확인한다. State Store에 접근할 수 있다는 이유로 다른 owner의 내부 schema에 직접 의존하지 않는다.

Policy Manager는 `source / recipient / purpose / scope / policy revision / expiry`로 허용 범위를 정의한다. Controller가 승인한 envelope를 Context Manager, Model Access, Agent Gateway, Response Manager가 **실제 읽기·제공·게시 직전**에 검사한다. 이는 각 Component에 별도 정책 엔진을 복제하는 것이 아니라 같은 versioned 정책 계약을 강제하는 것이다. 현재 revision을 확인할 수 없으면 보호정보를 새로 사용하지 않는다. 일반 질문마다 사용자 승인을 추가하지는 않는다.

권한 철회는 새로운 사용을 차단하고 관련 cache·prompt view·model session을 무효화하며 진행 중 생성의 게시를 막는다. 이미 외부에 제공한 정보나 수행된 Action이 소급 회수되지는 않는다. 외부 provider의 삭제 지원 여부와 실제 확인 범위를 구분해 안내한다.

User Memory 등록·수정·삭제는 Controller가 의미를 확정하고 Context Manager가 자기 aggregate를 변경한다. 삭제 tombstone·memory revision을 파생 view와 재시작 시에도 적용하여 과거 대화나 이전 model session에서 삭제된 선호를 자동 복원하지 않는다. 삭제 사실을 기록하는 audit에는 삭제한 원문을 복제하지 않는다. Memory 삭제와 원래 Conversation 삭제는 별도 요청이며 보관기간·백업 삭제·외부 삭제 보장은 아직 확정할 제품 정책이다.

## 10. 주요 데이터 계약

아래는 필수 의미를 정의한 논리 계약이다. Machine-readable schema는 아직 작성하지 않았다.

| 계약 | 핵심 내용 |
| --- | --- |
| Input Record | Conversation·Turn ID, 원문·modality, 입력 revision, producer sequence, 사건·수신 시각과 clock domain |
| Interaction Evidence Timeline | utterance span, transcript revision, acoustic interval·오차, display/window/document, screen/UI revision, pointer·selection interval, capture gap |
| Evidence Query Receipt | query와 source 범위, 관측 revision·유효 구간, 전체·반환 후보, truncation·실패, 제외 후보·이유, cache provenance |
| Resolution Record | subrequest graph, 목표·완료 조건, hard/soft 제약, referent·Task·handling 후보, field별 상태·origin·evidence·freshness·충돌·미해결 |
| Semantic Commit | immutable 확정 해석, 입력·Conversation·Task 후보·evidence·policy revision vector, dependency read set, supersession 관계 |
| Pending User Interaction | 질문·승인 유형, 결합할 Request·Task·Execution, unresolved field·후보, 허용 답변, revision·만료·응답 Turn |
| Agent Command | Request·Task·command ID와 type, 목표·완료 조건·제약·대상, Execution·artifact version, approval·policy revision, precondition, epoch·중복 방지 key |
| Agent Event | Agent·Execution·event ID, command correlation, source sequence/revision, emitted·received 시각, 상태·질문·artifact version·실패·확실성 |
| Canonical Response Payload | 게시할 proposition·질문·불확실성, Request·Task·result identity, source·staleness, admission 근거·revision, notification disposition, 선택적 검증된 generation handle |
| Response Record | payload·publication ID, Text 게시 내용, Voice generation과 실제 audible prefix/range, 표시·재생·중단·ack 상태 |
| Domain Event / Delivery Intent | event ID, owner aggregate·revision, 원래 Conversation·Request·Task, 원인 event, consumer 적용 상태와 publication ID |
| Use Envelope | source·recipient·purpose·scope, policy revision·expiry, 요청·generation 결합; 실제 사용 port에서 검증 |
| Model Call Envelope | call·Request·input revision, role·허용 Context view·출력 schema, deadline·cancel generation·budget |

외부 문서·Agent 내용은 데이터이며 VIA 정책을 바꾸는 지시가 아니다. 현재 권한은 과거 대화의 동의 문장이 아니라 Policy State에서 확인한다. Agent Action Approval은 VIA가 해당 실행에 중계하고 실제 Action의 권한 강제는 Agent가 담당한다.

```text
목표: 발표자료에 선택한 그래프 추가
완료 조건: 수정된 발표자료의 참조 제공
대상: 발표자료 ID와 확인한 version
입력 자료: 그래프 원본 또는 확인 가능한 참조
제약: 기존 내용 유지, 사용자 지정 위치 반영
관련 업무: Task A
```

위 계약은 실제 편집 단계·도구 선택을 지정하지 않는다. 대상 version이 중요하면 Agent가 실행 시점에도 확인해야 한다. VIA의 사전 검증만으로 외부 변경과의 경쟁까지 해결되지는 않는다.

## 11. 동시성·정정·취소·전송

Controller는 Conversation별, Task Manager는 Task별 짧은 상태 전이를 직렬 처리한다. 모델·네트워크 대기 중 lock을 잡지 않는다. 비동기 응답은 시작 당시 revision과 현재 revision을 비교해 적용한다. 늦게 도착한 이전 해석은 최신 요청을 덮어쓰지 않는다.

Voice 수신과 기본 Context 준비, 독립 source 조회, 여러 Agent event 수신, 장기 업무와 새 사용자 질문은 겹쳐 수행할 수 있다. Semantic LLM은 공유 자원이므로 병렬 제출을 무료 병렬 추론으로 간주하지 않는다.

Model Access는 현재 입력·clarification을 우선하고, 무효 호출을 취소하거나 결과를 버린다. 백그라운드 요약은 길이를 제한하고 오래 대기한 작업의 우선순위를 올린다. 실행 중 선점·동시 추론은 실제 모델 capability에 종속된다.

Queue는 모두 유한하다. 로컬 재생 중단은 모델·Store queue를 기다리지 않고, host의 명시적 제어 접수와 Agent terminal/question event는 일반 progress보다 먼저 처리한다. Progress는 Task별 최신 상태로 합칠 수 있지만 완료·실패·질문·정정 intent는 조용히 버리지 않는다. 내구 수신 여력이 없으면 지원되는 source에 backpressure를 걸고, 유실이 가능한 source는 gap을 기록해 재조회한다. 새 일반 요청을 수용할 수 없으면 바쁜 상태와 재시도 가능 여부를 명시하며 완료나 Agent 접수로 표시하지 않는다. 숫자 예산은 미정이어도 무제한 queue는 허용하지 않는다.

![Command dispatch, Agent event와 사용자 응답 게시의 내구 경계](./diagrams/05-dispatch-and-recovery.svg)

[draw.io 편집 원본](./diagrams/05-dispatch-and-recovery.drawio)

| 정정·취소 도착 시점 | 처리 |
| --- | --- |
| 전송 시작 전 | 관련 미전달 요청을 보류하고 최신 입력 반영 |
| 전송 시작 후·접수 불명 | 실행 여부 확인 후 수정·취소 연결 |
| 실행 중 | 지원되는 수정·취소 명령 전달 |
| 이미 완료 | 완료 사실과 가능한 후속 수정 안내 |

새 Voice 입력 시작은 현재 Conversation의 아직 보내지 않은 요청을 잠시 보류할 수 있다. 기존 모든 Task를 자동 중단하지 않는다. 선형화 지점 전에 들어온 정정·취소는 기존 command를 보내지 않고 새 semantic revision과 `supersedes_command_id`로 표현한다. 그 뒤에는 이미 막았다고 주장하지 않고 `UNKNOWN`, `CANCEL_REQUESTED`, `CORRECTION_PENDING` 중 실제 확인 상태를 기록한다.

선형화의 구체적인 주 설계는 State Store에서 `PENDING → DISPATCHING`을 변경하는 조건부 transaction이다. Gateway는 동일 transaction에서 Controller의 admission revision·Conversation hold와 Task Manager의 command epoch·현재 policy revision을 검사한다. 새 입력 보류·정정도 같은 조건을 변경하므로 먼저 commit한 전이가 순서를 결정한다. 네트워크 전송은 transaction 밖에서 수행하고 crash 시 `DISPATCHING`을 접수 불명으로 취급한다. 실제 acoustic 입력 시작과 host의 hold 기록 사이에는 인식·IPC 지연이 있으므로 물리적으로 먼저 말하기 시작했다는 사실만으로 이미 나간 command를 막았다고 주장하지 않는다.

새 입력이 기존 요청과 무관하면 hold를 해제한다. 입력이 끊기거나 전체 deadline을 넘기면 `WAIT_USER`로 남기고 안내하며, timeout만으로 잠재적으로 수정된 Action을 자동 전송하지 않는다. 보류는 해당 Conversation의 미전송 command 범위에 한정하고 실행 중인 무관한 Task에는 전파하지 않는다.

Outbox는 재시작 후 의도를 복구하지만 외부 exactly-once 실행을 단독 보장하지 않는다. Agent가 중복 방지 key·epoch precondition을 지원하면 같은 key로 재전송한다. 미지원이고 전송 결과가 불명이면 조회 없이 재실행하지 않는다. 취소 전송도 취소 완료나 외부 변경의 rollback을 뜻하지 않는다. 필요한 precondition이나 상태 조회가 없는 Agent에는 정정·취소 정확성이 필요한 Action을 맡기지 않거나 보장 수준을 사용자에게 낮춰 표시한다.

## 12. 장기 업무·복합 요청·Agent event

**Agent Gateway로 업무 요청을 보내는 주체는 Task Manager다.** Controller가 의미와 dispatch admission을 확정하면 Task Manager가 Task·Execution 관계와 immutable Agent Command를 만들고 outbox에 기록한다. Gateway의 전송 worker는 현재 command epoch·admission 유효성과 Agent capability를 확인한 뒤 전송한다. 정정·취소와 전송 시작의 경쟁은 §11의 선형화 경계를 공유한다. Gateway가 독자적으로 목표·대상·처리 경로를 선택하지 않는다.

Command의 불변 payload는 Task Manager, 전송 시도·접수 불명·재조회 기록은 Gateway가 각각 소유하고 command ID로 연결한다. 같은 레코드의 의미 field를 두 Component가 경쟁해서 변경하지 않는다.

Agent 선택에 필요한 capability profile은 Gateway가 소유한다. Interpreter는 요구 capability와 후보를 제안하고 Controller가 사용자 지정 Agent·기능·권한·제약·상태 조회·중복 방지·제어 지원 조건을 검증한다. 기능적으로 동등한 후보는 설정된 선호와 안정적인 우선순위로 선택한다. 비용·권한·완료 조건이 달라지는 대체는 사용자 확인 없이 조용히 적용하지 않는다. 적합한 후보가 없으면 미지원으로 안내하며 VIA가 업무 실행을 대신하지 않는다. 전송 후 접수 불명인 업무를 다른 Agent로 넘기기 전에는 기존 실행 상태부터 확인한다.

**Downstream Agent 결과를 직접 받는 곳은 Agent Gateway다.** Push event, stream 또는 polling 반환을 canonical Agent Event로 변환해 durable inbox에 먼저 기록한다. Task Manager가 correlation·중복·순서를 검증하고 `inbox 적용 상태 + source cursor + Task projection`을 한 transaction으로 반영한다. terminal event에 연결된 pending question이 있으면 Controller가 소유한 종료 전이도 같은 transaction에 참여시켜 오래된 질문을 닫는다. Task Manager가 Controller 상태를 임의로 쓰지 않으며, 뒤늦은 답변 admission도 현재 Task·question revision을 검사한다.

Task Manager는 확인된 변경을 Controller에 알린다. Controller는 원래 Conversation·Request·Task에 결합하고, 질문 등록·알림 시점·공개 범위를 결정해 Response Manager에 publication admission과 payload를 보낸다. 이 경로는 새 발화가 없어도 동작하며 일반 progress에는 semantic LLM 호출이 필요 없다. 긴 결과를 요약해야 할 때만 Response Manager가 공유 모델에 별도 생성 요청을 한다.

이 알림을 메모리 callback으로만 처리하지 않는다. Task projection과 **domain event outbox**를 같은 transaction에 기록하고 Controller가 event ID로 멱등 적용한다. Controller의 Request·질문 갱신과 publication intent도 함께 기록하고 Response Manager가 publication ID로 인계받는다. 따라서 Task 상태 저장 직후 또는 Response 인계 직전에 crash해도 사용자에게 전달할 결과·질문을 다시 찾을 수 있다. 관련 질문의 종료 등 여러 owner 전이가 필요한 경우 각 owner가 만든 변경을 하나의 Unit of Work로 commit하며, Store가 업무 의미를 판단하거나 owner 권한을 대체하지 않는다.

정상 progress마다 무조건 query하지 않고 다음 경우 재조회한다.

- event 순서 공백 또는 상태 모순
- 연결 복구, VIA 재시작
- 최신 상태를 요구하는 사용자 요청

중복 event를 제거하고 오래된 progress가 terminal state를 되돌리지 못하게 한다. Source 순서·revision을 제공하지 않는 Agent는 event를 변경 hint로 사용하고 조회로 확정한다. 조회도 불가능하면 확인 불가를 보존한다.

“자료를 요약한 다음 김대리에게 보내고, 발표자료는 계속 만들어”에서는 실제 요약 결과 version을 발송의 입력으로 연결한다. VIA는 사용자가 명시한 의존 관계의 후속 요청을 해제하고 독립 업무는 계속 진행한다. 업무 분석·발송의 계획과 도구는 Agent 책임이다. 앞 요청 실패 시 의존한 뒤 요청을 실행하지 않고 부분 완료를 구분한다.

![복합 요청의 데이터 의존, 독립 Task와 조건 분기](./diagrams/09-compound-and-task-routing.svg)

[draw.io 편집 원본](./diagrams/09-compound-and-task-routing.drawio)

Controller는 의존 node를 `WAIT_DEPENDENCY`로 두고, 실제 선행 결과 version·조건 사실·현재 권한이 확보되면 재검증해 해제한다. `graph revision + node ID + dependency result version`으로 해제 identity를 기록하여 같은 event 재처리가 후속 command를 중복 생성하지 않게 한다. 조건은 참·거짓·불명을 구분하며 업무 분석이 필요한 조건의 판단은 Agent에 맡긴다. 이미 해제한 결과가 나중에 바뀌면 이전 실행을 되돌린 것으로 취급하지 않고 정정 관계를 만든다.

범위가 명확한 자료 요약은 VIA 직접 처리일 수 있고, 조사·업무 분석·파일 생성은 Agent 책임이다. VIA의 조건 검증과 실제 외부 Action 사이에 source가 바뀔 수 있으므로 Agent에도 version precondition과 충돌 시 처리 조건을 전달한다.

Agent 질문·승인은 공통 Pending User Interaction에 Task + Execution + question ID + 요청 version으로 등록한다. 여러 질문 중 답변 대상을 특정하지 못하면 “응”을 임의 승인으로 사용하지 않는다.

## 13. 응답 전달

![S2S 및 Core 응답의 게시와 로컬 barge-in 경로](./diagrams/08-response-and-interruption.svg)

[draw.io 편집 원본](./diagrams/08-response-and-interruption.drawio)

Response Manager는 Controller가 admission한 Canonical Response Payload를 publication outbox에 기록하고 Text·Voice·알림 publication을 조정한다. 미리 생성된 S2S 응답이면 payload의 generation handle을 검증해 Interaction Manager에 release/cancel을 보낸다. 새로운 음성은 Model Access를 통해 같은 S2S로 생성한다. 긴 Agent 결과를 요약할 때는 같은 Model Access의 공유 semantic LLM을 사용하며, Controller가 허용한 source·목표·제약 범위를 벗어난 새 판단·실행을 만들지 않는다. 상태 template이면 semantic 호출을 생략한다.

생성 결과는 승인된 proposition·질문·불확실성과 Request·Task·result identity·staleness를 유지해야 한다. 내용이 이를 바꾸거나 근거가 부족하면 게시를 보류하고 Controller로 반환한다. Interaction Manager는 유효한 출력 세대와 release를 가진 응답만 표시·재생하고 delivery receipt를 반환한다. Barge-in은 이 승인 흐름과 독립적으로 즉시 재생을 중단하며, 오래된 release가 중단한 출력을 다시 살릴 수 없다.

Publication ID는 요청 전체와 별개이며 Text·Voice·알림에 공통으로 연결한다. Text는 같은 ID·내용 version으로 UI에 멱등 upsert한다. Voice는 generation·segment·output epoch로 중복과 오래된 packet을 거절한다. 전체 답변을 반드시 다 만든 뒤 재생하지는 않는다. source와 의미가 검증된 최소 문장 단위에서 Text·audio 대응과 내구 publication intent를 확보하면 순차 release할 수 있다. 이를 제공하지 않는 S2S는 완성 단위까지 buffer해야 하며 추가 지연을 숨기지 않는다. Audio를 먼저 재생하고 나중에 Text의 의미를 맞추는 경로는 허용하지 않는다.

실제 재생과 receipt의 내구 기록은 원자적이지 않다. Crash 전에 저장된 마지막 확인 범위를 넘어선 구간은 `DELIVERY_UNKNOWN`으로 복원하고 이미 들었다거나 전혀 못 들었다고 단정하지 않는다. 확인 불가 음성을 자동 재생하지 않고 결과 Text를 복원하며 사용자가 요청하면 다시 읽는다. 생성된 전체 Text, UI에 실제 표시한 Text, 확인된 audible prefix, 전달 불명 구간을 별도로 남긴다.

- 모든 사용자 응답은 Text와 Conversation에 남고, Voice 활성 시 핵심을 짧게 전달한다.
- 사용자가 말하는 동안 일반 progress 음성이 끼어들지 않는다. 여러 결과를 동시에 재생하지 않는다.
- 반복 progress는 묶되 실패·완료·입력 필요 event는 보존한다.
- 중단한 출력 세대의 늦은 audio packet을 버리고 자동으로 이어 재생하지 않는다.
- Text 표시와 실제 Voice 전달을 별도로 기록한다. 화면에 전체 Text가 있어도 음성을 전부 들려준 것으로 기록하지 않는다.
- Conversation은 실제 게시된 Text와 사용자가 들을 수 있었던 audible prefix를 참조한다. 중단된 뒤의 후속 지칭에 생성만 되고 전달되지 않은 내용을 사용하지 않는다.
- OS 알림은 해당 Task와 상세 결과로 연결한다. 사용자를 별도 Agent 대화창으로 보내지 않는다.

## 14. 프로세스 배치·fault boundary

![VIA runtime process와 fault boundary](./diagrams/06-runtime-and-fault-boundaries.svg)

[draw.io 편집 원본](./diagrams/06-runtime-and-fault-boundaries.drawio)

주 배치는 UI Process, Voice Process, Core Process를 분리하고, 위험한 native·blocking 연동을 Connector Worker에 격리하는 구조다. Interaction Manager의 audio·playback·local barge-in과 Model Access의 S2S adapter·session owner는 Voice Process에 둔다. Controller·Interpreter·Context·Task·Response·Policy와 Gateway의 상태 권위, Model Access의 semantic queue·adapter는 Core에 둔다. Response Manager의 음성 생성 요청도 bounded IPC로 Voice의 같은 S2S session owner를 사용한다. Model Access는 하나의 논리 계약이지만 모델별 adapter의 배치가 다르며 모델을 추가 적재하는 것은 아니다.

Interaction Manager의 Text·화면·pointer 수집은 UI process 쪽 OS adapter와 제한된 evidence buffer를 사용한다. 무거운 화면 읽기는 worker로 격리한다. Voice와 UI의 producer sequence·사건 시각·capture gap을 Core의 Interaction Manager timeline 모듈이 결합한 뒤 Controller에 넘긴다. Voice를 끈 상태에서도 Text·화면 경로는 유지된다. UI의 명시적 음성 stop은 Voice로 직접 전달하며 Task 제어는 Core를 거친다. 따라서 단일 논리 Component의 하위 모듈이 여러 process에 배치된다는 비용과 IPC 계약을 숨기지 않는다.

Connector Worker는 실제 연동의 장애·접근 경계에 따라 나누며 Task별로 만들지 않는다. Core 안에는 Store access module이 있지만 durable State Store의 데이터는 Core crash 후에도 복구 가능해야 한다. Embedded database와 별도 DB process 모두 이 논리 경계를 구현할 수 있다. Supervisor는 health·restart·backoff를 담당하며 Request·Task의 의미 상태를 직접 변경하지 않는다. 이 배치도 제안이지 기존 배치 결정의 자동 변경이 아니다.

Core가 끊겨도 Voice의 로컬 stop은 동작한다. 새 응답의 admission은 중단하고 기존 playback도 lease 만료 또는 연결 단절 감지로 정지한다. Voice는 Core incarnation과 output epoch가 바뀐 이전 release를 거절한다. UI는 마지막 확인 상태와 Core 연결 문제를 표시하며 Task 취소를 내구 접수한 것처럼 보고하지 않는다. 명시적 UI stop과 실제 Task 취소의 보장 범위는 다르다.

모델 local 배치 시 모델별 runtime 하나, remote 배치 시 연결 adapter를 사용한다. 배치별 지연·메모리·네트워크 비용은 다르며 실제 주 배치는 아직 정하지 않았다.

| 장애 | 동작 |
| --- | --- |
| Voice Process·S2S 단절 | 음성 복구; Text·Task 추적 유지 |
| 특정 Context source 실패 | 해당 근거가 필요한 요청만 보류·실패 |
| Semantic LLM·전체 deadline timeout | partial inference나 불완전 Context를 commit하지 않고 clarification·확인 불가·안전한 실패로 종료; 확인된 상태와 명시적 UI 제어 유지 |
| Agent 단절 | 실행 실패로 단정하지 않고 상태 재조회·재연결 |
| Core 재시작 | 영속 기록과 Agent 상태로 업무 연결 복원 |
| Store 쓰기 실패 | 복구 근거가 필요한 새 위임·수정·취소 전송 중단 |

모든 외부 호출에 deadline을 두고 요청 전체 deadline을 우선한다. 조회 재시도는 남은 예산 안에서 수행하며 반복 실패 connector의 새 호출을 잠시 제한한다. 수치형 deadline·보관·메모리·queue 예산은 아직 미확정이다. 무한 대기·무한 refinement·무조건 재전송은 허용하지 않는다.

재시작 시 command outbox, event inbox·cursor·Task projection, response publication outbox·delivery receipt를 각각 복원한다. 전송 대기·전송 불명·실행 중, event 반영 전·후, Text 게시·Voice 부분 전달을 구분해 상태를 확인한다. 결과·질문·허용 제어와 사용자가 실제 접한 응답이 다시 연결되어야 복구다. 프로세스가 재기동됐다는 이유만으로 복구 완료라고 하지 않는다.

복구 순서는 `새 incarnation·전송 fence 설정 → schema·owner state 복원 → inbox·domain event 재적용 → Agent 상태 조정 → 질문·publication 재결합 → 새 admission 개방`이다. 확인이 끝나지 않은 Task는 재조회 중·확인 불가로 남긴다. 한 Agent의 장기 장애가 무관한 대화의 개방을 무한히 막지 않게 Task별 recovery deadline을 적용한다. Store 손상·migration 실패에서는 외부 Action을 새로 전송하지 않으며, 확인되지 않은 rollback·성공 복원을 주장하지 않는다.

## 15. 주요 runtime 시나리오

| 상황 | 동작 |
| --- | --- |
| “이거 지난번 자료에 넣어줘” | 당시 지칭 근거와 과거 자료 후보를 연결하고 목적 자료·Task 관계 확정 후 위임 |
| 후보가 둘 이상 | 값싼 추가 근거로 구분하거나 사용자 선택이 필요한 차이를 질문 |
| 해석 중 화면 변경 | 당시 근거 유지; 대상 내용·identity 변경 시 관련 해석만 무효화 |
| Agent 장기 실행 | 새 대화 계속; 마지막 확인 상태·시각 유지; 진행률 추측 금지 |
| Agent 실패·부분 완료 | 완료·실패 부분 분리; 외부 변경 확인 없이 전체 재실행 금지 |
| Voice barge-in | 재생 중단·출력 세대 폐기 먼저, 새 요청 해석은 이후 |
| 실행 직전 정정·취소 | 미전달 요청 보류; 전송 시작 뒤면 외부 실행 상태 확인 |
| 여러 업무의 확인 질문 | 질문별 identity 유지; 모호한 답변을 임의 승인으로 사용하지 않음 |
| Voice/Text 전환 | 동일 Conversation·대상·Task 유지; 음성 연결 수명과 분리 |
| Memory 삭제·권한 철회 | 지속 상태와 관련 cache·session 사용 중단을 함께 반영 |

## 16. 네 ASR의 설계 우선순위와 critical path

이 Architecture는 QA-19, QA-09, QA-29, QA-39 순으로 우선해 설계한다. 지금은 이 네 관점에서 목표 구조가 완결됐는지 검토하며, 구조 선택과 steelman 비교는 전체 Architecture 합의 뒤에 시작한다.

![네 core ASR의 Architecture critical path](./diagrams/07-four-asr-critical-paths.svg)

[draw.io 편집 원본](./diagrams/07-four-asr-critical-paths.drawio)

Context, queue, Store, IPC, network, speech generation, playback buffer 비용도 경로에 포함된다. 입력 종료 전에 겹쳐 수행한 작업을 종료 후 비용에 다시 더하지 않는다. Agent 내부 업무 시간은 분리하되 전체 사용자 대기 시간은 유지한다. Filler·접수 인사를 유효 결과의 도착으로 취급하지 않는다. 이는 [event boundary 의미](../../11-measurement/event-boundary-contract.md)를 보존한 설계 설명이며 새 측정 계약은 아니다.

Accuracy 경로는 입력·시점 → 당시 근거 → 대상·목표 → Task·처리 방향 → 최신 요청 확정 → Agent 계약 → 진행·결과 연결이다.

Modifiability 경로는 Model·Agent·Context·policy 변화 → 해당 port와 adapter → canonical contract·state migration → 영향받는 Component·Interface·State·Runtime이다. Provider별 차이를 Core semantic state나 Task lifecycle로 누출하면 변경 파급이 커진다.

Reliability/recoverability 경로는 fault 발생 → process·queue·transaction boundary에서 격리 → durable intent/event/publication 복원 → source 상태 재확인 → 중복·오연결 없는 사용자 상태 수렴이다. 재시작 자체가 아니라 확인 가능한 Task·Execution·응답 관계의 복원이 끝점이다.

| 영향 지점 | QA-19 accuracy | QA-09 responsiveness | QA-29 modifiability | QA-39 reliability/recoverability |
| --- | --- | --- | --- | --- |
| Evidence·Context contract | 지칭·자료·시점·후보 coverage | 사전 준비·조회·prompt·재해석 비용 | source adapter와 canonical receipt가 변화 파급을 제한 | source fault·gap·stale evidence를 해당 요청에 격리 |
| Semantic Commit | 목표·대상·Task·handling 일관성 | 호출·검증·clarification 비용 | model/prompt/schema 변화가 Resolver·contract에 집중 | timeout·늦은 결과가 확정 상태를 덮지 못함 |
| Request·Task·Command state | 정정·승인·업무 binding 보존 | transaction·queue·dispatch 비용 | aggregate별 단일 writer와 versioned state migration | outbox·epoch·projection으로 crash와 중복 전송 복구 |
| Agent·Response ports | 정확한 capability·event·사용자 결과 연결 | 변환·재조회·게시·재생 지연 | Agent/provider 변화가 Gateway·Publisher adapter에 집중 | inbox·cursor·publication receipt로 event·전달 복원 |

위 표는 네 ASR을 목표 구조에 반영한 인과 가설이다. 실제 수치나 다른 구조보다 우수하다는 측정 결과는 없다.

## 17. 비용·약점·재검토 조건

1. **틀린 근거의 일관된 해석:** 지칭 시각 오류나 source 후보 누락은 schema 검사를 통과할 수 있다. 근거 연결은 의미 정확성의 충분조건이 아니다.
2. **refinement 예산 초과:** 일반 요청조차 두 번 안에 안정되지 않고 clarification이 반복되면 기본 Context와 해석 책임을 재검토해야 한다.
3. **공유 모델 병목:** 긴 요약이 새 사용자 요청을 막을 수 있다. 지원되지 않는 선점을 scheduling만으로 해결할 수 없다.
4. **필수 dependency 기능 미확보:** 시간 근거·시각 이해·음성 통제·Agent 조회/중복 방지가 없으면 adapter만으로 필수 행동을 충족하지 못할 수 있다.
5. **지속적으로 바뀌는 자료:** 관련 근거가 계속 바뀌면 재검증으로 진행이 멈출 수 있다. 과거 대상으로 할 수 있는 요청과 현재 version이 필요한 Action을 구분해야 한다.
6. **구현 비용:** 증거·revision·outbox·event 정합성·출력 전달 상태가 추가된다. IPC·저장·cache 비용을 성능 이점에서 빼놓지 않는다.

반증 가능한 약점은 지금 보존하되 강한 대안, Decision Package, 구체 실험은 전체 구조 합의 이후에 설계한다. 다음 검토 항목은 [검토 기록](./review-log.md)에 있다.

대표 UC의 누락 여부와 이번 그림 검토에서 보강한 경계, 남은 dependency·제품 정책 질문은 [설계 완결성 점검](./design-completeness.md)에 모았다. 문서상의 경로 완결성과 실제 모델 기능·성능 검증은 별개다.
