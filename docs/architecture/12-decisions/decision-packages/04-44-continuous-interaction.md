# 04-44. 계속 듣고 응답하는 대화의 실행 구조

> 상태: **STAGE_4_REFINED_CANDIDATE_REVIEW / A·B 미선정 / 구현·측정 없음** / 2026-10-06
> [탐색 지도 영역 6](./04-50-agent-architecture-decision-map.md#area-06) · [비교 기준](./04-30-comparison-guide.md#4-sw-구조적으로-다른-대안의-정확한-의미) · [독립 리뷰 기록](./04-44-interaction-review.md)

**사용자의 말을 계속 들으면서 현재 요청의 답변과 이전 업무의 질문·결과를 빠르고 올바르게 같은 대화로 이어가는 SW 구조**를 비교한다. A는 중앙 조정자가 사건마다 다음 실행을 명령하고 반환을 회수하는 **비동기 Orchestration/Mediator**다. B는 타입이 정해진 사건과 결과를 연결한 소비자가 조건 충족 시 활성화되는 **반응형 Dataflow/Pipes-and-Filters**다. 둘 다 입력과 작업을 병행한다. 차이는 비동기 API의 유무가 아니라 실행을 이어가는 책임과 중간 상태의 조직이다.

## 1. 발표용 메인 비교

![44 — 중앙 비동기 조정과 반응형 실행망](./diagrams/choice44-structure.svg)

[편집용 draw.io](./diagrams/choice44-structure.drawio) · [발표용 PNG](./diagrams/choice44-structure.png) · [확대 보기](./diagrams/choice44-review.html)

**왼쪽 블록은 A만, 오른쪽 블록은 B만 사용한다.** 각 칸 안에서 같은 입력·Task/Agent·모델부터 다른 실행/상태 구조와 실제 전달까지 완결한다. 공통 기능도 각각 반복 표시하고 칸 사이에는 연결을 두지 않는다. 반복한 Component/Omni는 서로 배타적인 대안의 같은 의존성이며 한 구현에서 모델을 두 벌 사용한다는 뜻이 아니다. 메인 한 장으로 목적·공통 입력·다른 제어/상태·실제 전달을 설명한다. 파랑은 A의 조정 책임, 초록은 B의 실행망, 검정은 공통 기능이다. 상자는 구현할 Component/Module 이름이며 OS process나 모델 개수를 뜻하지 않는다. 원통은 상태다. 화살표의 번호와 자료 ID는 실행 관계이며 길이는 시간 수치가 아니다.

### 1.1 그림을 짚는 발표 대본

1. **각 칸의 같은 두 생산자:** 사용자는 보고서 설명 P1을 듣다가 u2 “잠깐, 표부터 설명해줘”라고 말한다. 별도로 외부 Agent의 질문 Q1/결과 r1이 도착한다. Task Manager가 외부 사실을 확인해 제공한다. 어느 Task인지 정답을 미리 입력에 붙이지 않는다.
2. **각 칸 위쪽의 입력 제어:** Interaction Manager는 계속 수신/인식하고 Input Control Gate는 새 입력 세대로 이전 음성 출력을 즉시 차단하고 미전송 효과를 보류한다. 이 경로는 해석이나 응답 준비 완료를 기다리지 않는다. 발화 시작은 업무 취소 명령이 아니다.
3. **A의 중앙 왕복:** Interaction Orchestrator가 u2와 Q1/r1을 자기 진행 상태에 연결한다. Request Controller에 해석/처리를 요청하고, Response Manager에 확정 근거로 답변/질문 준비를 요청한다. 반환을 회수해 현재성과 후속 조건을 검사하고 Response Manager에 게시를 요청한다. Response Manager가 허용한 출력은 아래 Interaction Manager에서 실제 표시·재생한다. 오래 걸리는 호출 동안 중앙 상태 전이를 잠그지 않는다.
4. **B의 분기와 결합:** Input Resolution Stage는 확정 입력으로 Request Controller를 활성화한다. Task Notice Stage는 확인된 Q1/r1을 별도 경로로 공급한다. Response Manager는 준비 가능한 확정 근거를 소비하고 승인된 Direct 후보는 재생성 없이 통과시킨다. Publication Join은 후보 응답과 종류별 admission·자료/질문 revision·기존 실제 전달 snapshot을 결합한다. 내용 응답은 입력 해소를 기다리지만 현재 입력을 해결할 확인 질문은 제한적으로 먼저 나갈 수 있다. 결과가 중앙 명령자로 되돌아와 재배정되는 경로는 없다. 점선의 credit/cancel은 소비자가 수용 가능한 양과 폐기 조건을 상류에 전달하는 흐름이다. 출력 소비자의 credit은 Interaction Manager의 Channel I/O가 제공하고 Playback State는 과거 실제 전달 receipt snapshot만 제공한다. 게시 상태와 질문 focus의 원본은 각각 Response Manager와 Request Controller에 남는다.
5. **같은 마지막 경계:** Response Manager의 Publication Control은 Request Controller/Task Manager에 현재 입력·질문·원본·권한을 확인하고 게시 차례와 중복을 관리한다. 아래 Interaction Manager 안의 Turn-Taking Control은 현재 output epoch·발화 여부·release를 재생 직전에 검사하고, Channel I/O는 Text 표시·음성 재생과 실제 전달 receipt를 생산한다. receipt로 Response Manager의 Publication Outbox와 Request Controller의 실제 제시 질문 focus를 갱신한다. 생성된 Q1은 아직 질문 focus가 아니다. 같은 단일 Omni를 공유하므로 실행망의 병행이 모델 계산량 증가나 무료 병렬 추론을 뜻하지 않는다.

품질을 설명할 때 A의 집중된 상태/왕복과 B의 직접 분기/결합/역방향 흐름을 짚는다. A의 짧은 제어 전이와 B의 여러 buffer·join 비용도 함께 읽는다. B가 항상 빠르거나 정확하다는 결론은 이 그림에서 도출하지 않는다.

## 2. 사용자 문제와 설계 범위

사용자는 VIA가 생각하거나 말하거나 업무를 기다리는 동안에도 계속 말한다. 다른 업무의 질문과 결과는 사용자 입력과 다른 시점에 도착한다. 다음 네 결과가 필요하다.

| 공통 기능 | 필요한 결과 |
| --- | --- |
| 계속 듣기와 끼어들기 | capture·Streaming ASR을 유지하고 음성은 로컬에서 즉시 멈춘다. 입력 누락이나 인식 backlog를 정상으로 숨기지 않는다. |
| 현재 요청 처리 | 전체 원문과 관련 근거로 의도·대상·Task 연결을 판단하고 직접 응답/확인/위임으로 연결한다. |
| 비동기 질문/결과 전달 | 확인된 업무 이름·질문·부분 실패·결과를 올바른 대화에서 전달한다. Agent 접수와 업무 완료는 구별한다. |
| 정정과 대화 이어가기 | 정정 전 미전송 효과를 보류하고 실제 들은 범위를 기준으로 답변 재개/폐기/수정과 질문 답변 연결을 수행한다. |

핵심 근거는 [고정 범위](../../03-fixed-architecture-scope.md), [UC-01/06/10~15](../../05-representative-use-cases.md), [P-04/06/08/09/10/14](./01-00-problem-coverage.md)다. 비교의 대표 장면은 정상 대화에서 입력·질문·결과가 겹치는 상황이다. 드문 crash를 B의 대표 이익으로 삼지 않는다.

| 인접 주제 | 44에서 고정하거나 구별할 내용 |
| --- | --- |
| 34: 발화 중 잠정 준비 | **기존 A/B를 승격하지 않는다.** 두 안 모두 같은 확정 입력과 준비 정책을 사용한다. 부분 발화의 조기 해석은 별도 최적화이며 B의 성립 조건이 아니다. |
| 41: 의미 해결 제어 | 같은 의미 해결 방식을 양안에 적용한다. 실행망이 자연어 관계를 코드로 추측하지 않는다. |
| 43: 의미 생산 책임 | 같은 Request Interpreter/F1~F6 생산 방식을 사용한다. Stage는 업무별 semantic actor가 아니며 이전 33의 제외된 B를 복원하지 않는다. |
| 42: 서비스 수명/원본 | 같은 Core 배치, Conversation/Task 원본, 로컬 내구 저장과 외부 실행 수명을 사용한다. B에 별도 process나 독립 업무 서비스를 추가하지 않는다. |
| 공유 모델 | 같은 on-device Omni 1벌, 역할별 Context/KV, 독립 Streaming ASR, bounded scheduler를 유지한다. 새 학습 helper는 없다. |

## 3. 실제 설계: 누가 다음 처리를 진행시키는가

### 3.1 공통 소유와 허용 경계

아래 공통 Component와 Interaction Manager의 **Channel I/O**, **Turn-Taking Control** 이름은 [참조 구조 §4](../target-architecture/architecture.md#4-component와-상태-소유권)를 따른다. 입력 측과 출력 측에 그린 Interaction Manager는 **한 Component의 두 접점**이며 별도 실행체가 아니다. 출력 쪽은 이 Component 안에 재생 제어·입출력·Playback State를 넣어 그렸다. 별도의 출력 중재 Component를 추가하지 않는다.

**게시 차례와 실제 재생은 책임이 다르다.** Response Manager 안의 Response Composer와 Publication Control은 각각 기존 응답 준비와 게시 제어 기능을 펼친 Module 이름이며 새 Component나 두 안의 차이가 아니다. Publication Control은 publication ID·출력 차례/lease·현재성 확인·중복 방지·내구 Publication Outbox를 관리하고, 현재성/권한의 판단 원본은 Request Controller/Task Manager에 조회한다. Interaction Manager는 허용된 release만 현재 장치/출력 epoch에 맞춰 표시·재생하고 로컬 stop와 실제 전달 receipt를 책임진다. Playback State는 장치 재생 상태와 확인된 receipt의 제한된 전달 buffer다. publication의 내구 원본과 질문 focus를 이 buffer로 옮기지 않는다.

Input Control Gate는 이 비교에서 비차단 입력 제어와 Request Controller의 admission hold를 잇는 논리 Module이다. 그림의 m/t/v/p는 공통 모델/업무/현재성/실제 제시 서비스 연결 포트이며 별도 Component 사본을 뜻하지 않는다. 그림의 p 포트는 Response Manager의 확인된 실제 제시 기록을 Request Controller에 반영하는 같은 연결이다. `Presented(question ID, publication ID, actual-range, revision)`은 이름만 같은 임시 snapshot이 아니라 내구 전달 기록과 일치해야 한다. Request Controller가 PendingUserInteraction/질문 focus를 소유하고 답변을 연결한다. Interaction Manager가 “응”의 의미나 어느 질문에 대한 답인지 결정하지 않는다.

| 사실/권한 | 공통 최종 소유자 | 두 안의 소비 조건 |
| --- | --- | --- |
| 원음·전사·현재 input epoch·발화 중 여부 | Interaction Manager / Speech Input Worker | 확정 전사는 무오류 선언이 아니다. 늦은 정정은 새 revision으로 전달한다. |
| 의미 채택·Request·확인 질문·미전송 admission | Request Controller | Request Interpreter가 제안한 의미를 현재 근거/질문/권한으로 검사해 채택한다. 새 발화의 의미가 미해결이면 미전송 효과 hold를 해제하지 않는다. |
| Task·Execution·Agent 질문·외부 접수/진행/결과 | Task Manager / Agent Gateway | Input Control Gate의 hold epoch를 command admission/dispatch에서 검사하고 acknowledgment를 남긴다. capture와 hold 적용 사이 이미 외부로 나간 명령은 실제 외부 상태를 확인한다. 작업/질문 revision과 확인 상태가 필요하다. 기존 실행을 token만으로 취소 완료라고 하지 않는다. |
| Response 내용·publication/출력 차례·내구 전달 상태 | Response Manager | source와 의미 참조를 보존하고 원본 owner에 현재성/권한을 확인한다. Text와 음성의 대상·상태·결론·실패가 일치해야 한다. receipt로 Publication Outbox의 실제 전달 범위를 갱신한다. |
| 실제 표시·재생·로컬 중단·device receipt | Interaction Manager | Turn-Taking Control이 재생 직전 발화/출력 epoch/release를 검사하고 Channel I/O가 실제 전달을 수행한다. Playback State는 재생 상태/receipt buffer이며 내구 publication 원본이 아니다. |
| 실제 제시 질문 focus·질문 답변 연결 | Request Controller | 확인된 실제 전달 범위를 PendingUserInteraction에 반영한다. 생성 완료를 제시로 세지 않고 “응”은 실제 focus와 현재 질문 원본으로 연결한다. |

외부 업무는 **Request Controller → Task Manager → Agent Gateway → Downstream Agent**의 동일한 검증/전달 계약을 따른다. **Request Controller의 admission/hold 변경, Task Manager의 command/epoch 변경, Agent Gateway의 transmission CAS를 같은 local State Store transaction으로 조합한다.** 각 owner의 검증한 변경 집합을 확정하며 다른 owner의 필드를 직접 쓰지 않는다. hold/권한 철회가 먼저 확정되면 PENDING 전송을 막고, DISPATCHING이 먼저 확정되면 이미 시작된 전송과 UNKNOWN 가능성을 보존해 실제 외부 상태를 조회·정정·취소한다. 네트워크/모델 호출은 transaction 밖이다. 이는 [참조 제어 §5](../target-architecture/control-and-lifecycle.md#5-전송과-정정의-원자적-경계)의 같은 경합 계약이며 B의 stream event 도착 순서로 대체하지 않는다. 음성 stop의 즉시성과 외부 Agent의 실제 cancel 효력은 다른 시간·사실이다. S2S 직접 후보는 Voice Runtime에서 생산하고 Request Controller가 동일한 좁은 direct admission을 검사한다. 승인된 `AdmittedDirectCandidate(u, Text/audio, source refs)`는 A에서 Dispatcher의 후보 반환으로, B에서 Response Manager stream adapter의 passthrough로 이어져 같은 Response Manager의 게시 경계와 Interaction Manager의 실제 출력에 도달한다. 둘 다 이미 만든 후보를 Core에서 다시 생성하지 않는다. admission 거절 시 같은 Request ID로 Core 처리에 인계하며 직접 후보를 게시하지 않는다.

### 3.2 A — 중앙 비동기 Orchestration/Mediator

**Interaction Orchestrator**가 입력/업무/반환/실제 전달 사건을 받아 다음 실행을 정한다. 내부 **Dialogue Dispatcher**는 코드다. 상태 원본 owner를 조회하고 필요한 Request Controller/Response Manager 호출을 발행한다. 의미는 새로 만들지 않는다.

**Dialogue Progress State**에는 `(conversation, input revision, continuation, waiting IDs, source refs, candidate response, suspended delivery refs)`를 연결한다. 확정 Request/Task/질문 원본을 복제하지 않는 실행 상태다. 반환은 `job ID + 시작 revision`으로 중앙에 회수된다. Dispatcher는 유효하면 다음 처리를 명령하고, 오래됐으면 폐기/재시작한다. 원본을 바꾸는 확정은 §3.1 owner에게 요청한다.

A는 전체 대화 처리를 하나의 긴 lock/모델 호출로 묶지 않는다. 입력별 짧은 상태 전이, 우선순위 mailbox, 비동기 worker, cache·사전 준비·부분 수정, 후보 병행 준비, Task별 검색, handler 모듈을 모두 허용한다. 개별 job에 수용량 상한과 취소 token을 적용할 수 있다. 그림의 hub는 CPU 한 개/모델 호출 한 개가 아니라 **후속 실행을 명령하는 소유자**다.

Orchestrator는 질문 Q1의 실제 제시 receipt를 진행 상태에 연결한다. Task Manager는 확정 질문 원본을 유지하며 Request Controller가 확인된 실제 제시 사실로 focus를 갱신한다. 사용자의 “응”도 전체 입력과 실제 focus로 해석한다. 중앙에 관련 상태가 모여 원인을 추적하기 쉽지만, 여러 생산자/정정/차례 조건을 연결하는 전이와 dispatch 규칙이 조정 책임에 모인다.

### 3.3 B — 반응형 Dataflow/Pipes-and-Filters

**Reactive Interaction Runtime**은 명시적인 typed channels와 Stage의 구독, scheduling, bounded queue/credit, cancellation을 실행한다. 대화 전체를 판단하거나 다음 job을 임의로 선택하는 중앙 semantic/업무 조정자가 아니다. 코드로 작성한 그래프와 조건이 실행된다.

| Stage | 입력과 산출물 | Stage가 소유하는 실행 상태 |
| --- | --- | --- |
| **Input Resolution Stage** | 확정 `Input(u)`·근거/실제 제시 참조 → Request Controller의 `AdoptedMeaning(u)` 또는 `Unresolved(u)` | Input Window: 실행 중 input/job ID, 대체 revision, 근거 read set. Request/의미 원본은 Request Controller 소유 |
| **Task Notice Stage** | Task Manager의 확인된 `Question/Result(T,r)` → 응답 준비 가능한 `Notice(T,r)` | Notice Window: 원본 참조, 미전달 중요 항목, 준비 job, progress 최신값. 외부 질문/결과 원본은 Task Manager 소유 |
| **Response Manager의 stream adapter** | 채택 의미, 허용된 확인 질문/실패·제어 안내 또는 유효 Notice → Text/audio candidate와 source refs. direct admission이 승인한 기존 S2S Text/audio candidate는 재생성 없이 passthrough | Response Composer의 준비 상태와 Publication Control/Publication Outbox의 게시 상태. 준비 실패는 명시적 error envelope |
| **Publication Join** | 후보 + 후보 종류별 admission + 해당 source validity + 확정된 과거 delivery/focus snapshot → eligible candidate / hold / discard | Publication Window: 제한된 후보와 필요한 참조, dependency watermark, 중복 키. publication/내구 실제 전달 원본은 Response Manager, 질문 focus 원본은 Request Controller 소유. Playback State는 receipt snapshot 제공 |

InputSettled는 **새 입력이 없거나, 해당 입력의 의미·정정·제어 관계를 Request Controller가 해소했다는 확인**이다. 단순 ASR final이나 timeout으로 생산하지 않는다. Unresolved/WAIT_USER/WAIT_CONTEXT이면 기존 내용 응답과 관련 업무 Notice를 보류한다. **현재 u2를 해결하기 위한 확인 질문과 확인된 실패/제어 안내는 별도 `InteractionNotice(u, question ID, admission class)`로 반환한다.** 이 후보는 InputSettled를 요구하지 않고 현재 u2·유효 질문/사실·권한·발화 종료를 확인해 제한적으로 게시한다. 외부 효과 hold는 유지하며 질문 제시가 업무 실행 허가가 되지 않는다. 무관함이 채택 의미로 확인된 업무 결과는 기존 공통 정책대로 먼저 전달할 수 있다. Stage가 스스로 “무관하다”를 추측하지 않는다.

B의 Request Controller 반환은 `AdoptedMeaning/InputSettled` 채널에 게시되고 Response Manager/Publication Join이 구독한다. Task Notice 결과도 연결된 소비자로 전달된다. **모든 반환을 중앙에 회수해 다음 worker 호출을 결정하는 Dialogue Dispatcher와 Dialogue Progress State는 없다.** 각 Stage는 자기 입력의 충족 여부로 활성화된다. **Response Manager의 준비 입력과 게시 입력은 별도 typed port다.** Response Composer의 후보가 Publication Join으로 가고, Join의 eligible 후보는 Publication Control로 돌아간다. 이미 준비/승인된 Text/audio를 재생성하지 않으며 Publication Control은 다른 Stage의 다음 준비 job을 지시하지 않는다. 같은 프로그램·thread pool에서 구현할 수 있으며 process 분리의 이익을 주장하지 않는다.

Stage가 적용하는 최신성/중복/순서/credit 규칙은 타입 계약에 포함한다. 입력 u2에 대한 느린 u1의 의미/응답 반환은 downstream eligibility를 얻지 못한다. backend가 취소 불가이면 계산은 계속 비용으로 남되 결과를 신규 게시에 사용하지 않는다. Publication Join의 delivery 입력은 **새 후보가 게시되기 전의 확정 snapshot**이다. 첫 대화의 `known-empty`도 유효한 초기 값이고, 연결/복구 후 실제 상태가 UNKNOWN이면 이를 명시하며 질문 답변 연결을 보류한다. 새 후보 자신의 receipt는 게시의 선행조건이 아니다. 게시 뒤 Interaction Manager가 receipt를 생산하면 Response Manager가 동일 publication ID/revision의 actual-range를 내구 기록하고 Request Controller가 실제 질문 focus를 갱신한다. 다음 join은 이 확정 사실과 Playback State의 receipt snapshot을 참조한다. 불명 상태를 buffer의 빈 값으로 덮지 않는다.

단순 Reactive Streams 라이브러리만으로 VIA의 의미 정확성, source validity, 내구 저장과 실제 전달 연결이 자동 제공되지는 않는다.

### 3.4 flow-control, 순서와 확정

| 계약 | A | B |
| --- | --- | --- |
| 새 발화 우선 제어 | Input Control Gate가 즉시 output epoch 무효화·효과 hold. Dispatcher는 후속 사건 처리 | 같은 Gate가 먼저 차단. u2/control은 일반 후보 credit에 막히지 않는 별도 control channel로 Runtime/windows와 sink 및 command admission에 전달 |
| 유량과 상한 | Dispatcher가 job/worker/후보 queue admission을 제한 | 소비자 credit이 Stage/pipe admission으로 전파. 모델 admission은 공통 Model Access 제한도 확인 |
| 줄일 수 있는 자료 | Task progress 최신값, 폐기된 generation을 중앙에서 합침 | 동일 종류를 Stage에서 합침. input, 질문, 중요 실패/완료, receipt는 임의 drop 금지 |
| 외부 source를 늦출 수 없음 | 먼저 durable inbox 저장, 준비 admission 제한 | 동일 inbox 보존 후 Notice 준비를 늦춤. 외부 Agent에 credit 지원을 가정하지 않음 |
| join과 순서 | 진행 상태의 waiting ID/revision으로 결합 | conversation/input/source ID와 revision으로 결합. 전역 도착 순서나 latest 값 하나로 여러 업무를 합치지 않음. 후보 종류(content / clarification / control / failure)별 admission을 구별 |
| 내구 경계 | owner가 확정 Request/명령/publication을 저장 | 같은 owner가 저장. Stage 출력 자체는 명령 접수/실제 게시가 아님 |
| 최종 현재성 | Response Manager의 원본 owner 현재성 확인 + Interaction Manager의 재생 직전 epoch/release 검사 | 동일 검사. 늦게 도착한 validity event만으로 최신 상태를 믿지 않음 |

B의 credit/cancel은 실행망의 제어 feedback이며 Task 취소 의도가 아니다. cyclic feedback은 receipt/credit/revision 갱신으로 한정하고 이미 처리한 envelope ID를 반복 재활성화하지 않는다. delivery/focus는 known-empty로 초기화할 수 있지만 필수 source/admission 초기 값이 없는 join은 `not ready`이며 무한 대기 대신 Request/준비 deadline에 따라 명시적으로 보류/실패를 반환한다. 중요 항목이 상한을 넘으면 내구 원본을 유지하고 추가 준비 admission을 막으며 UI에 backlog/처리 불가를 드러낸다. 양안에서 메모리 무한 증가나 중요한 결과 유실을 정상 동작으로 허용하지 않는다.

## 4. 같은 정상·정정·교차 사건

![44 — 동일 사건과 현재성 연결](./diagrams/choice44-event.svg)

[사건도 draw.io](./diagrams/choice44-event.drawio). 보충 그림도 왼쪽 A/오른쪽 B로 분리하며 각 칸의 참여자와 사건 경로만 사용한다. 검토 확대용이며 메인 설명을 대신하지 않는다.

| 사건 | A의 경로 | B의 경로 | 공통 사용자 결과 |
| --- | --- | --- | --- |
| E1 보고서 설명 P1 재생 | Dispatcher가 P1 전달 진행/receipt 참조를 연결 | eligible P1이 sink로 진행, receipt가 Publication Join으로 돌아옴 | 실제 재생한 범위만 후속 근거 |
| E2 별도 Agent 질문 Q1/결과 r1 도착 | Task Manager의 확인 사실을 중앙에 연결하고 Response Manager 준비 호출 | Notice Stage가 원본 참조를 소비해 Response Manager로 공급 | 현재 음성을 끊지 않으며 Text/중요 항목은 보존 |
| E3 u2 “잠깐, 표부터 설명해줘” | 공통 stop/hold 후 중앙에서 u2 해석 호출; 기존 후보/continuation 현재성 갱신 | 같은 stop/hold 후 Input Stage 활성화; u2 barrier로 관련 후보 eligibility 차단 | 계속 듣고 P1 중단. 업무 자체 취소는 아님 |
| E4 u1/P1 준비 결과가 늦게 반환 | 시작 revision과 현재 u2를 비교해 폐기 | 관련 stream/window에서 u1을 superseded 처리, sink에서도 재검사 | 오래된 음성이 되살아나지 않음 |
| E5 u2 의미·대상이 채택 | 중앙이 새 설명 준비/무관 Notice의 후속 처리를 명령 | 채택 의미와 InputSettled가 관련 Stage/join을 활성화 | 모호하면 질문. 기존 유효 조건/Task는 보존 |
| E6 설명 후 유효 Q1을 제시 | 중앙에서 후보를 전달하고 실제 receipt를 Q1에 연결 | Join에서 유효 Q1 후보 → sink → receipt feedback | 실제 제시된 Q1만 focus 후보 |
| E7 사용자 “응” | 같은 Request Interpreter가 실제 제시/경쟁 질문을 해석; owner가 답변 채택 | Input Stage가 같은 해석을 요청; 결과가 공통 owner를 통해 채택 | 유일한 유효 대상일 때만 연결. 경쟁 후보면 확인 |

Text 입력도 같은 revision/hold/채택 경로를 사용한다. 음성을 껐으면 Text-only 전달을 기록하고 옛 음성을 재연결 뒤 자동 재생하지 않는다. S2S 자체 지식 직접 후보는 좁은 admission을 만족할 때만 같은 sink로 들어온다. Core로 인계하면 동일 Request ID를 사용해 이중 응답을 막는다.

## 5. 변경·철회·실패·재시작

| 사건 | 설계 동작과 두 구조의 차이 |
| --- | --- |
| 발화 중 반복 정정 | 원본 revision을 보존한다. A는 중앙 pending/jobs를 갱신, B는 관련 window/pipe를 invalidation. 한쪽만 부분 재사용을 허용하지 않는다. |
| source 변경·동의 철회 | 원본 owner의 permission/source revision이 admission과 sink를 차단. A 후보/jobs와 B pipes/windows 및 모델 Context/KV의 신규 사용을 폐기한다. 이미 전달한 정보의 소급 회수는 보장하지 않는다. |
| 준비/모델 실패 | A job error 또는 B error envelope로 해당 후보를 종료하고 확인된 원인을 표시. 다른 유효 경로는 진행하되 같은 모델/process의 공통 장애는 공유한다. |
| input gap·인식 실패 | 어떤 안도 의미를 추측해 효과 hold를 해제하지 않는다. Request Controller가 재입력/확인을 요청한다. |
| Task 질문 만료·외부 취소 실패 | 질문/명령 원본 owner가 유효성을 거절하고 확인된 상태를 안내한다. stage 완료와 외부 업무 완료를 구별한다. |
| Runtime/Core 재시작 | 확정 Request/Task/질문/publication/receipt를 같은 내구 owner에서 복원한다. A 진행 상태 또는 B 임시 windows를 재구성한다. 미확인 외부 명령을 새 ID로 재전송하지 않는다. |
| 재생 중 crash | 마지막 확인 밖의 음성 범위는 UNKNOWN. 자동 반복 없이 Text 기록과 현재 사실로 재개한다. |
| 그래프/handler 교체 | 신규 admission drain/fence 뒤 schema/contract 호환을 검사. B는 subscription/pipe/window schema와 in-flight envelope version도 관리해야 한다. A도 dispatcher/handler 진행 상태 호환이 필요하다. |

## 6. V-01~13의 같은 기준으로 본 손익

[03-00 §2](./03-00-quality-scenarios.md#2-어떤-품질을-보고-있는가)의 정의와 우선순위를 유지한다. 아래는 **구조에서 예상하는 조건부 인과**이며 수치·별점·측정 우위가 아니다. 기능 정확성/적절성/완전성을 먼저, 반응성/완료 시간을 다음으로 검토한다. 두 안의 외부 Agent 실행 시간과 모델 조건은 같다.

| 관점 | A 중앙 비동기 조정 | B 반응형 실행망 |
| --- | --- | --- |
| V-01 정확성 | 중앙에 입력/반환/질문 연결과 현재성 규칙을 모아 검토. 중앙 전이 누락/경합이 여러 경로에 영향 | typed revisions·join·최종 sink를 모든 경로에 적용할 기회. join key/초기 값/늦은 validity/취소 전파가 틀리면 stale 게시. 의미 정확성의 자동 이익 없음 |
| V-02 적절성 | 집중된 대화 제어로 질문·결과를 묶고 같은 설명 반복을 피할 수 있음 | 동일 사용자 결과 가능. 부분 준비 실패/불명 join을 불필요한 질문으로 넘기면 사용자 수고 증가 |
| V-03 완전성 | Voice/Text, S2S/Core 직접 응답, 위임, 교차 질문, 정정/재개를 공통 계약으로 지원하도록 설계 | 같은 범위. actual receipt, priority control, durable 중요 항목과 owner checks 없는 범용 stream 조합은 완전 지원으로 세지 않음 |
| V-04 반응성 | local stop는 공통. 중앙 전이를 짧게 하고 priority/async로 대기 제한 가능. 많은 반환/교차 규칙의 dispatcher 부하가 조건부 비용 | local stop는 공통. source→consumer 직접 활성화·단계별 credit으로 준비 부하를 제한할 기회. 추가 join/buffer/공유 모델 대기로 오히려 늦어질 수 있음 |
| V-05 VIA 완료 시간 | 중앙 명령/반환/후속 dispatch의 비용. 짧은 요청은 적은 경유와 상태로 유리할 수 있음 | 독립 Notice 준비를 연결해 불필요한 중앙 재배정을 줄일 기회. fan-in 조건 대기·폐기 작업·재검증 비용. 빠른 접수 멘트로 완료를 대신하지 않음 |
| V-06 자원/수용량 | 중앙 mailbox + 제한된 jobs, worker buffers. 같은 상한/포화 정책 구현 가능 | 다수 windows/pipes/subscriptions·credit bookkeeping·envelope 참조 비용. immutable 원문 참조를 공유하고 불필요한 모델 호출은 하지 않음. B만 자원 효율적이라는 주장 없음 |
| V-07 결함/복구 | 중앙 실행 상태와 owner 기록의 복원이 비교적 집중 | 임시 실행망의 재구성과 재구독/중복 제어가 추가. 같은 process/model/storage에서 자동 장애 격리 없음 |
| V-08 변경/모듈성 | handler별 국소 수정 가능. 여러 생산자 사이 activation/hold 규칙 변경은 중앙 dispatch 전이에 모임 | 같은 stream schema 안의 producer/consumer 조합 변경은 국소화 후보. schema·join·cancel 의미가 바뀌면 그래프 전체 영향. Stage 수만으로 변경성이 좋아지지 않음 |
| V-09 분석/시험 | 중앙 transition/job trace로 인과 추적. worker 순서 역전도 시험 필요 | graph/envelope/credit/cancel trace와 가상 시간 시험 경계. 분산 window·order 역전·중복·초기 join 조합의 분석 부담 |
| V-10 기밀성 | 중앙 후보/worker/model Context의 참조와 철회 정리 | 여러 pipe/window에 남은 참조/Context까지 철회 전파. envelope에 보호 원문을 매번 복제하지 않음. 입력 policy는 동일 |
| V-11 연동/공존 | 외부 protocol 변화는 공통 Gateway/owner에서 흡수. 중앙 burst와 workers의 PC 자원 경합 | 외부 credit 지원을 가정하지 않고 inbox 경계 유지. 내부 stream/error 계약 연동과 여러 buffer가 공존 비용. Omni scheduler 보장은 동일 |
| V-12 조작/오류 방지 | 같은 실제 제시/focus와 stop/cancel 표시. 잠정 준비를 완료로 표시하지 않음 | 동일 정책. Stage complete/stream end를 업무 완료/청취 완료로 오인하지 않도록 publication 상태 분리 |
| V-13 설치 | 동일 Omni/ASR/배치. coordinator/handler 계약 갱신 | 동일 inventory. Reactive Runtime/adapter/graph 버전 호환 추가. 특정 framework 설치를 선택하지 않았으며 runtime 교체 비용은 남음 |

## 7. 가장 싼 변경과 혼합에 대한 판단

| 반론 | 작성자의 판단 |
| --- | --- |
| A에 priority queue·async worker·stop fast path를 추가하면 충분하지 않은가? | **충분할 수 있다.** 모두 강한 A에 포함했다. 이 정도로 중요한 부하에서 반응성과 정확성을 만족하면 B의 도입 이유는 줄어든다. |
| A에 Event Bus/Rx를 붙이면 B인가? | 중앙이 모든 다음 실행을 명령하고 반환을 회수하면 A다. Bus/library는 운반 수단이다. B는 activation·return 연결·revision join·cancel/credit 책임을 실제 생산/소비 계약으로 옮긴다. |
| B도 Response Manager와 Interaction Manager로 모이니 중앙 구조 아닌가? | 게시 차례와 장치 출력이라는 마지막 경계는 공통이다. 준비 입력→Composer→Join→게시 입력은 정해진 자료 연결이며 중앙 후속 job 지시가 아니다. 이 경계가 모든 Request/Notice 준비 순서를 정해 worker를 호출하면 A+B 혼합이다. |
| A의 handler를 각각 독립 구독자로 만들면? | 단순 handler 등록은 A. handler가 자신의 stream readiness와 반환 소비/유량을 소유하고 Dispatcher를 대체한다면 실제 B로의 재설계다. |
| 일부 결과 경로만 stream으로 만들면? | 현실적인 혼합. 도입 범위의 activation/window/취소와 중앙 연결 비용을 공개한다. 작은 혼합으로 주요 이익을 얻으면 전체 B를 주요 필수 선택으로 주장하지 않는다. |

| 비용 | A → B | B → A |
| --- | --- | --- |
| 유지할 수 있는 것 | 입력/ASR, 의미 생산, Context, Task/Agent 원본·권한, 실제 전달, 모델 API | 동일 |
| 기능 개발 | stream envelopes, Stage adapters, joins, credit/error/cancel 계약 | coordinator handlers, 명령/반환 correlations, 중앙 continuation |
| 임시 상태 이행 | 진행 준비 drain 후 전환하면 window 이행 대부분 불필요. 내구 owner 기록 재사용 | 동일. drain으로 migration 비용은 줄지만 제어 재설계는 남음 |
| 핵심 재설계 | 중앙 return→다음 명령 체계를 producer→consumer 활성화와 국소 windows로 대체 | Stage readiness/feedback 상태를 중앙 전이/작업 관리로 모으고 소비 계약 변경 |

**판정:** 이 비교는 중앙 Orchestration과 반응형 Dataflow라는 실제 실행 조직의 차이다. 34의 준비 시점 정책이나 43의 의미 생산 분해로 환원하지 않는다. 다만 style 차이만으로 B의 선택 이유가 충분해지지는 않는다. 강한 A의 짧은 중앙 전이와 작은 혼합을 정면 비교하며, 중요한 정상 대화의 반응/현재성 처리와 변경 부담이 전환 비용에 비해 의미 있을 때만 B의 선택 근거가 성립한다.

## 8. 선택 조건과 반증 조건

**A를 선택할 이유:** 입력/Notice 경로가 비교적 적고 중앙 전이·우선순위·상한으로 반응성을 유지할 수 있으며, 교차 대화의 제어와 원인 추적을 집중시킬 이익이 큰 경우다. 내부 모듈화와 병행을 포기할 필요가 없다.

**B를 선택할 이유:** 서로 다른 입력/질문/결과 생산 경로가 계속 조합되고, 중앙 continuation이 여러 경로의 준비·취소·유량·현재성 연결 부담을 반복적으로 떠안는 경우다. 그 책임을 typed flow의 Stage/pipe 계약으로 실제 이전할 때 조합과 유량 제어의 이익을 기대할 수 있다. 지연 개선은 추가 join과 buffer 및 단일 모델 비용을 포함해 확인해야 한다.

| 주장 | 주장을 반박하는 조건 |
| --- | --- |
| B가 중요한 정상 부하에서 반응/완료 경로를 개선한다 | 강한 A의 짧은 dispatch/priority로 같거나 더 좋은 결과를 얻음. B의 추가 joins/buffers 또는 모델 경합이 이익을 상쇄 |
| B의 현재성/취소 계약이 정정에 유리하다 | u2 후 u1 게시, 미제시 Q1에 “응” 연결, 중요 event drop, 철회 후 신규 사용이 발생. A도 같은 공통 검사로 동등하게 처리 |
| B가 변경을 국소화한다 | producer 변경에 여러 stream/join/adapter 수정이 필요하거나 A handler/작은 혼합으로 같은 국소화를 얻음 |
| A가 더 단순하게 같은 목적을 만족한다 | 중요한 교차 입력/Notice에서 중앙 전이 조합·trace·취소 누락의 부담이 커지고 priority/handler 보완으로 해소되지 않음 |

현재는 문서/설계 검토다. 실제 V 결과, 장비 숫자, harness/freeze와 A/B 승자를 확정하지 않는다. 후속 검증이 승인되면 같은 입력·외부 사건·모델 조건에서 u2 이후 stale output, 실제 focus binding, 최종 전달, control/준비 backlog와 자원 비용을 실패까지 보존해 비교해야 한다.

## 9. 근거와 확인 범위

참조 설계의 [요청/대화 계약](../target-architecture/control-and-lifecycle.md), [공유 Omni](../target-architecture/shared-omni-runtime.md), [기억/Context 수명](../target-architecture/memory-and-context-lifecycle.md), [기능 완결성](../target-architecture/design-completeness.md)을 공통 기능과 소유 경계 확인에 사용했다. 참조 구조 자체가 A/B 승자나 B의 기반 구현은 아니다.

외부 근거는 스타일과 구성 계약의 출처이며 VIA 성능 근거가 아니다. 2026-10-06 공개 문서를 확인했다.

- [Microsoft — Choreography pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/choreography): 중앙 orchestration과 사건으로 이어지는 협력의 구별. VIA B는 독립 microservices를 추가하지 않고 같은 실행체 안의 dataflow로 구체화했다.
- [Microsoft — Pipes and Filters](https://learn.microsoft.com/en-us/azure/architecture/patterns/pipes-and-filters): 입력/출력 계약으로 연결한 처리, 중복/상태/실패 비용. 44는 단순 직렬 stateless pipeline 대신 분기·상태 있는 join·제어 feedback을 명시한 반응형 변형이다.
- [Reactive Streams](https://www.reactive-streams.org/): 비동기 경계의 수용량/비차단 backpressure 계약. input epoch, 의미 채택, 철회, durable inbox와 실제 전달은 이 문서에서 별도 설계한 VIA 계약이다.

그림 생성은 [44 전용 생성기](../../../../scripts/architecture/generate_continuous_interaction_diagrams.py) 한 scene에서 SVG/draw.io를 함께 만든다. `--check`는 source parity/형식/배치 보조 검사다. architecture 의미·우열은 자동 검사로 판정하지 않는다. 독립 리뷰·보완·재리뷰의 실제 발견과 범위는 [리뷰 기록](./04-44-interaction-review.md)에 남긴다.
