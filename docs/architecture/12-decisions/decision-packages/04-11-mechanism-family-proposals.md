# Stage 4 재발굴 — 두 해결 메커니즘의 실행과 상태

> 상태: **대안 구체화와 반론 검토 / 사용자 검토용 / baseline 변경 없음** / 2026-10-02
> 문제 선정은 [04-10](./04-10-mechanism-family-discovery.md), 판정과 전체 품질 검토는 [04-12](./04-12-mechanism-family-review.md).

## 1. 읽는 방법과 공통 조건

T는 실제 [target](../target-architecture/architecture.md)이다. C-대화와 C-독립은 이번에 작성한 설계 가설이다. 두 C를 합친 안이나 구현된 제품을 비교하지 않는다. 기존 T/A의 기능을 함께 쓰는 가상 확장도 실제 T와 구분한다.

사용자는 PC에서 Voice 또는 Text로 대화한다. VIA는 입력, 요청 관계, Agent 위임과 결과 전달을 맡고 외부 Downstream Agent는 업무의 세부 계획과 실행을 맡는다. 원래 화면 관측, 접근 가능한 source, Agent capability와 외부 지연 조건은 같다. Omni 가중치는 하나이며 역할과 session을 분리한다. 아래 어느 안도 모델 능력을 무료로 추가하거나 외부 업무를 VIA 안에서 실행하지 않는다.

**공통 준비:** Interaction Manager가 사용자 입력과 당시 화면 근거를 기록하고 새 발화 때 로컬 음성을 멈춘다. 독립 Streaming ASR이 입력 근거를 생산하며 Model Access가 공유 Omni의 입력과 추론 자원을 관리한다. Text 입력도 같은 입력 revision 경로에 들어온다. 좁은 자체 지식 S2S 응답 경로는 이 비교에서 유지한다. 아래는 Context와 업무 관계가 필요한 입력의 처리다.

사용자 입력의 의미, 코드가 검증한 실행 권한, 실제 외부 실행 결과는 서로 다르다. 모델의 제안이나 로컬 전송 기록을 Agent 업무 완료로 바꾸지 않는다. 모델 KV는 재사용 가능한 계산 상태이며 업무 사실의 원본이 아니다.

이 문서의 주요 용어는 다음 뜻으로 쓴다. **Request Graph**는 요청과 선행 결과의 의존 관계를 기록한 실행 자료다. **transaction**은 여러 상태 변경을 함께 성공하거나 함께 취소하는 저장 단위이며, **commit/확정**은 그 변경을 내구 기록으로 인정하는 시점이다. **hold**는 아직 전송하지 않은 요청의 일시 보류다. **revision/epoch**는 오래된 판단이나 권한을 구별하기 위한 버전이다. **outbox/inbox**는 보내거나 받은 사건을 재시작 뒤에도 다시 처리할 수 있도록 저장한 원장이다. **barrier**는 관련 처리 주체가 변경을 적용했다고 확인할 때까지 그 변경에 의존하는 진행을 막는 절차다.

## 2. C-대화 — 요청 관계를 코드로 실행할까, 사건마다 다음 처리를 판단할까

### 2.1 풀 문제와 품질 충돌

사용자가 “이 자료를 설명하고, 여기서 말한 보고서도 찾아줘”라고 한 뒤 “아니, 설명은 짧게 하고 보고서는 작년 것으로”라고 바꾼다. Agent의 기존 질문과 결과도 같은 대화에 들어온다. 정확한 관계를 유지하면서 불필요한 질문과 VIA 재처리를 줄여야 한다(P-01/04/05/06, V-01~05).

T는 구조화된 해석으로 관계를 명시하고 코드가 검증, 대기와 후속 해제를 수행한다. 이는 확인 가능한 진행과 재개에 유리하다. 반면 해석과 후속 처리의 고정된 계약에 담지 못한 의미 관계는 계약 확장이나 재해석이 필요하다. **실제 target에서 그런 누락이 얼마나 발생하는지는 아직 입증되지 않았다.** 이것이 C-대화의 이익을 검토할 가설이지 기존 target의 확인된 결함은 아니다.

T도 사용자 답변과 정정 뒤 재해석하고, 선행 결과를 받은 `WAIT_DEPENDENCY`에서 재검증 후 `RESOLVING/READY`로 진행한다([제어 §4](../target-architecture/control-and-lifecycle.md#4-request질문task의-상태-전이)). “후속 사건을 모델로 판단할 수 있음” 자체는 C만의 기능이 아니다. 차이는 다음 처리의 주된 구동 원본을 명시적 graph 전이에 두는지, 사건별 모델 판단에 두는지이며, 그 차이가 필요한지는 별도로 입증해야 한다.

C-대화는 입력뿐 아니라 필요한 후속 사건도 모델의 동일한 대화 처리 루프로 보내, 현재 근거에 맞는 다음 처리를 선택한다. 의미상 다음 행동을 매번 모델에 의존하므로 오류, 반복 추론, 긴 문맥과 복구 비용을 부담한다. 유연성이 정확성을 자동 보장하지 않는다.

### 2.2 실제 구조와 상태의 차이

**새 Component인 Conversation Agent**는 VIA의 대화 처리 순서를 결정하는 책임을 가진다. Downstream Agent와 구별한다. **새 Component인 Interaction Guard**는 모델 제안의 identity, revision, 현재 권한과 전송/게시 조건을 검증한다. 이름을 바꾼다는 사실은 구조 차이의 근거가 아니다. 아래 책임 이동과 재개 원리가 차이다.

| 책임 | T | C-대화 |
| --- | --- | --- |
| 모델의 역할 | Request Interpreter가 입력에 대한 SemanticProposal을 반환. Request Controller가 관계를 확정하고 다음 단계를 실행 | Conversation Agent가 입력, 답변, 관련 결과를 받고 다음 VIA 처리를 반복 제안. 모델 호출은 계속 Model Access의 SEMANTIC 역할 사용 |
| 추가 정보 획득 | Request Controller가 초기 package와 한 묶음의 추가 읽기를 조정 | 기본 자료는 같은 값싼 초기 조회로 준비. 이후 Conversation Agent가 모델 판단으로 필요한 읽기를 순서대로 제안. Interaction Guard가 scope와 예산 검증 후 Context Manager에 요청. 요청별 유한 호출/byte/deadline 예산 필수 |
| 진행 상태의 원본 | Request Controller의 Request Graph, 질문과 현재 Request 상태 | Conversation Agent의 내구 사건 기록과 Interaction Guard의 처리 의도/receipt 원장. 아직 하지 않은 후속 동작을 실행 가능한 Request Graph로 미리 확정하지 않음 |
| 다음 단계 실행 | Request Controller가 확정된 의존 관계와 결과 version으로 후속을 해제 | Conversation Agent가 현재 기록과 결과를 읽어 다음 동작을 제안. Interaction Guard는 허가된 동작만 확정하지만 의미상 빠진 후속을 스스로 만들어 주지 않음 |
| 질문/승인 | Request Controller가 질문 상태와 답변 결합을 소유 | Interaction Guard가 질문 ID, 실제 제시 범위, expiry와 승인 digest를 소유. Conversation Agent는 답변 대상 후보와 질문 내용을 제안. 모호하면 다시 질문 |
| 결과 설명 | Response Manager가 확정 사실로 구성하고 전달 | Conversation Agent가 문맥상 설명/질문 내용 제안. Response Manager는 확인된 사실과 범위 검증, Text 게시, Voice 음성화 및 receipt 책임 유지. source 없는 완료 주장은 거부 |
| 업무 사실과 외부 실행 | Task Manager와 Agent Gateway | 같은 Component가 계속 소유. 모델은 Task/Execution 상태를 임의 덮어쓰거나 외부 도구를 직접 호출할 수 없음 |
| 권한과 확정 | Policy Manager와 여러 owner의 State Store transaction | 동일한 현재 권한 검사와 공유 transaction 원리는 유지. 이 후보에 독립 저장이나 메시지 분산을 덧붙이지 않음 |
| 재시작 | owner 상태와 graph를 복구하여 남은 전이를 진행 | 사건 기록, 완료/미확정 처리 receipt, 현재 Task와 정책을 복구하고 모델이 다음 의미상 처리를 재판단. 미확정 전송은 먼저 외부 조회로 조정 |

`처리 의도`는 모델의 내부 사고 기록이 아니라, 사용자에게 영향을 주는 제안된 동작의 종류, 대상, 입력/정책 revision과 근거 참조다. `receipt`는 그 동작을 누가 실제로 접수하거나 수행했는지 확인한 기록이다. 모든 원음과 화면을 무기한 transcript에 넣지 않는다. Context Manager의 보관과 삭제 규칙을 따르는 참조를 저장하고 삭제된 내용과 그 파생 문맥을 재사용하지 않는다.

Conversation Agent가 요청할 수 있는 범위는 근거 조회, VIA 설명/질문, 독립 사용자 목표의 Agent 위임, 기존 Task의 제어 요청, 기억 변경 제안이다. Agent 업무 내부의 조사 계획이나 파일 조작 순서를 만드는 것은 계속 범위 밖이다. 한 업무를 동사마다 여러 Task로 쪼개지 않는다.

### 2.3 같은 사건으로 보는 정상 처리

공통 사건은 “현재 문단을 설명해줘. 그리고 진행 중인 보고서 작업이 끝나면 그 결과를 짧게 설명해줘”다. 진행 중인 Task의 결과를 기다리는 동안 현재 문단의 설명은 진행할 수 있다. 외부 Agent 업무 실행 시간은 두 안에서 같은 외부 조건이다.

| 순서 | T | C-대화 |
| --- | --- | --- |
| 1 | Interaction Manager → Request Controller: 확정 입력 revision과 시점별 근거 참조. Request Controller → Context Manager: 현재 선택과 Task snapshot 요청 | Interaction Manager → Interaction Guard: 같은 입력과 근거 참조. Interaction Guard가 이전 제안을 hold하고 입력을 내구 기록한 뒤 Conversation Agent에 사건 전달 |
| 2 | Context Manager → Request Controller → Request Interpreter: 본문, Task 상태, 출처 및 조회 한계. Request Interpreter → Model Access: 해석 요청 | Conversation Agent → Interaction Guard: 정해진 기본 준비로 현재 문단과 Task 조회 요청. Interaction Guard → Context Manager: 허용된 조회. Context Manager → Interaction Guard → Conversation Agent: 같은 본문과 상태, 출처 및 조회 한계. 초기 조회 선택에 숨은 모델 호출은 없음 |
| 3 | Model Access → Request Interpreter → Request Controller: 직접 설명과 결과 의존 관계의 해석 후보. Request Controller가 검증 후 Request Graph와 현재 revision 확정 | Conversation Agent → Model Access: 현재 사건과 receipt에 근거한 다음 처리 요청. Model Access → Conversation Agent: 현재 문단 설명 제안과 보고서 결과를 기다릴 의도. Conversation Agent → Interaction Guard: 해당 제안 |
| 4 | Request Controller → Response Manager: 현재 문단 설명의 승인된 내용 또는 구성 근거. Response Manager → Interaction Manager: Text와 허가된 Voice. Interaction Manager → Response Manager: 실제 전달 receipt | Interaction Guard가 입력/권한/근거를 검사하고 설명 의도를 확정 → Response Manager: 설명 제안과 검증 근거. 이후 Text/Voice 전달과 receipt는 T와 동일. 보고서 Task의 사건 구독과 미완료 사용자 목적은 내구 기록 |
| 5 | Agent Gateway → Task Manager: 보고서 terminal 사건. Task Manager가 source revision과 상태를 확정하고 Request Controller에 결과 사건 전달. Request Controller가 graph의 의존 조건을 검증해 후속 설명 해제. 필요한 의미 재검증은 Request Interpreter로 보내며 READY를 무조건 강제하지 않음 | Agent Gateway → Task Manager: 같은 사건과 확정. Task Manager → Conversation Agent: 결과 참조. Conversation Agent → Model Access: 미완료 목적, 현재 정책과 전달 receipt에 근거한 다음 처리 요청. Model Access → Conversation Agent: 후속 설명 제안 |
| 6 | Request Controller → Response Manager: 현재 결과의 설명 요청. Response Manager가 구성과 게시를 수행하고 실제 전달을 기록 | Conversation Agent → Interaction Guard → Response Manager: 결과 설명 제안, 근거와 현재 revision. 검증, 게시와 실제 전달 기록. 모델이 후속을 누락하면 완료로 표시할 수 없으며 미완료 목적이 남음 |

사건 구독은 “이 Task 결과를 나중에 확인해야 한다”는 주의 대상을 기록한다. 그 결과가 왔을 때 무엇을 어떻게 실행할지는 모델이 정한다. 이것을 실행 가능한 graph로 확장해 코드가 같은 후속을 모두 정하게 되면 T와의 차이가 줄어든다. 반대로 단순 로그만 남겨 후속 요청을 영원히 잊을 수 있는 안도 충분한 구조가 아니다. 유한 대기/처리 예산이 끝나면 미완료를 명시하고 사용자에게 재시도 선택을 제공해야 한다.

위 사례는 같은 사용자 결과를 서로 어떻게 처리하는지 확인한 것이며 C의 우위 사례는 아니다. T도 이 요청을 지원한다. C에서만 더 적은 재설명이나 더 정확한 후속을 얻는 구체적 상황은 아직 제시되지 않았으므로 기능 이익을 확정하지 않는다.

### 2.4 정정, 철회와 실패

| 같은 사건 | T | C-대화의 처리와 추가 위험 |
| --- | --- | --- |
| 설명 도중 “아니, 옆 표만” | Interaction Manager가 로컬 출력 중단. Request Controller가 입력 hold 후 정정 해석, 관련 graph와 admission revision 변경 | Interaction Guard가 즉시 hold와 revision 변경. 이전 Conversation Agent 제안 거부. Conversation Agent가 새 근거와 전달 receipt로 재판단. 모델이 이전 의도를 되살리면 검증만으로 모든 의미 오류를 잡을 수는 없음 |
| 대기 질문 둘에 “응” | 실제 제시된 질문 focus와 유일성 확인, 애매하면 재질문 | Interaction Guard가 유효 질문 목록과 제시 범위를 제공. Conversation Agent가 대상 제안. 질문 identity/승인 digest/유일성을 코드로 검사. 의미상 선택이 모호하면 전송 보류 |
| Agent terminal과 승인 답변 교차 | Task Manager와 Request Controller가 같은 transaction에서 Execution/질문 revision 확인 | Task Manager와 Interaction Guard가 같은 transaction으로 질문을 만료하고 늦은 승인 거부. 이 동작을 모델의 다음 호출까지 미루지 않음 |
| 권한 철회 또는 기억 삭제 | Policy Manager와 Context Manager의 epoch/tombstone으로 새 사용 차단, 파생 view와 모델 문맥 무효화 | 같은 검사 유지. Conversation Agent의 기록 참조, 구성 중 prompt와 KV도 삭제 의존성에 포함. 복원 때 삭제 전 내용을 대화 요약에서 다시 지시로 만들지 않음 |
| 모델 timeout/반복 | Request Controller의 attempt 예산 후 질문 또는 실패. 확인된 상태는 template로 전달 | Interaction Guard가 유한 누적 호출/시간/읽기 예산을 강제. 루프 중지 후 이미 수행된 처리는 receipt로 표시하고 남은 목적은 미완료. T의 2회보다 좋은 종료 가능성은 미확인 |
| 외부 start 후 ACK 유실 | Agent Gateway가 UNKNOWN. command key 조회, capability 없으면 자동 재전송 금지 | 같은 동작. Conversation Agent는 UNKNOWN인 목표를 새 key로 다시 위임할 수 없음. 기존 intent와 목표 연결 검사 필요. 표현만 달라진 중복 목표를 완벽히 판별한다는 보장은 없음 |
| Core 재시작, 저장소와 Agent 조회 정상 | owner 복구, Task 조회, 질문과 graph 후속 재개 | Interaction Guard가 전송 receipt와 Task를 먼저 조정. Conversation Agent가 현재 사건 기록과 미완료 목적에서 이어갈 제안 생성. 동일 이력에서 다른 제안을 낼 수 있으며, 이 경우의 정확한 복구는 검증 필요. 매번 UNKNOWN으로 끝내는 것은 정상 복구로 세지 않음 |

### 2.5 가장 싼 전환을 실제로 구성해 본 결과

**T → C의 가장 유리한 시도:** Request Interpreter에 tool loop를 넣고 Request Controller가 추가 조회와 이벤트를 더 자주 전달한다. Request Graph와 기존 질문, 전송, 복구 계약은 그대로 쓴다. 조회 횟수나 prompt만 달라지는 경우에는 이것으로 충분하며 주요 구조 차이가 아니다.

C-대화의 전체 동작까지 옮기려면 해석 완료 후에도 관련 Agent 결과와 질문 답변을 다음 판단 입력으로 보내야 한다. 코드가 실행하던 graph 후속 해제를 중지하고 미완료 목적, 사건 구독과 처리 receipt로 바꾸며, 재시작은 다음 코드 전이가 아니라 다음 의미상 판단을 복원해야 한다. Request Controller와 Request Interpreter의 계약, Response Manager의 내용 생산 입력, durable 상태의 재개 의미가 함께 바뀐다. Task Manager, Agent Gateway, Model Access 전체를 다시 작성할 필요는 없다.

**중요한 반론:** 기존 Request Graph에 “모델 재판단” 노드를 추가하고 현재 상태로 C의 모든 결정을 표현할 수도 있다. 그 호환 구현이 같은 품질상의 이익을 주고 기존 복구와 검증을 유지한다면, 별도 사건 원장을 원본으로 만드는 것은 불필요한 선택이다. 현 문서만으로 이 반론이 틀렸다고 입증하지 못했다. C-대화는 그래서 **다른 계열 가능성 검토 중**이다.

**C → T의 가장 유리한 시도:** 외부 Task는 그대로 두고 새 요청부터 T의 처리기를 사용한다. 기존 요청을 모두 끝내면 사건 원장의 진행 중 상태 이전비는 크게 줄어든다. 그래도 장래 요청에 대한 모델의 다음 동작 결정을 해석 schema, 관계/상태 전이와 재개 규칙으로 표현하고 검증하는 설계 작업은 필요하다. 이미 C 안에 같은 graph 규칙을 유지했다면 이 전환도 더 쉬워지며 다른 계열 주장은 약해진다.

**선택 조건:** T는 명시적 관계와 제한된 호출로 필요한 대화 변형을 충분히 수용할 때 유리하다. C-대화는 실제 중요한 대화 관계가 기존 고정 계약에 자주 걸리고, 사건별 판단이 재설명과 누락을 줄이며, 추가 추론과 복구의 의미 변동을 감수할 수 있을 때 검토할 이유가 있다. 이 조건의 빈도와 모델 능력은 미확인이다. 기존 구조의 작은 확장으로 같은 효과를 내면 C의 주요 후보 주장은 반증된다.

## 3. C-독립 — 한 번에 상태를 확정할까, 각각 확정한 뒤 조정할까

### 3.1 풀 문제와 목적의 한계

입력 정정, Task 결과와 정책 변경이 동시에 도착할 때 오전송을 막고, 일부 내부 처리기가 실패해도 무관한 상태 수신과 작업을 유지하려는 방향이다(P-06/08/11/13). 정확한 제어 V-01과 제어 지연 V-04, 실패 중 VIA 대기 V-05가 상위 연결이다. 주된 기대 이익은 장애 및 변경의 국소화이며, 한 사용자 PC에서 이것이 추가 비용보다 중요한지는 확인되지 않았다.

T도 비동기 inbox/outbox, 짧은 transaction, Conversation/Task별 직렬화, 별도 connector process를 사용한다. 모델과 네트워크를 기다리는 동안 전체 Core를 잠그지 않는다. 따라서 “T는 모든 처리가 동기라 느리다” 또는 “Agent 하나의 장애가 항상 전체를 중단시킨다”를 대안의 근거로 쓰지 않는다.

### 3.2 실제 경계

C-독립은 기존 Component 이름과 의미 책임을 가능한 한 유지하지만 실행 및 저장 권위를 다음처럼 바꾼다. 새 이름은 Component가 아니라 실제 process와 저장의 묶음을 나타내는 설명이다.

| 실행/저장 경계 | C-독립의 내용 | T와의 차이 |
| --- | --- | --- |
| 대화 처리 process와 전용 원장 | Request Controller, Request Interpreter, Response Manager. 입력, 질문, 요청 관계와 publication 상태를 소유 | Task/전송, 정책 원장과 하나의 transaction으로 묶이지 않음 |
| 업무 추적 process와 전용 원장 | Task Manager, Agent Gateway의 내구 command/inbox 처리. source-confirmed 상태와 전송을 소유. 외부 connector 격리는 유지 | 대화 입력 hold, 질문 종료나 권한 변경을 메시지로 수신해 로컬 상태에 적용 |
| 정보 사용 process와 전용 원장 | Policy Manager, Context Manager. 권한, 기억, 삭제와 source 조회 상태 소유 | 권한 철회와 파생 자료 무효화를 다른 원장에 직접 원자 반영할 수 없음 |
| State Store | 각 process의 local transactional 저장 기능으로 사용. 원장별 단일 writer, durable outbox/inbox, 중복 제거와 version 검사 필요 | 전체 owner를 묶는 Unit of Work가 없음. 하나의 DB에 테이블만 나눈 안은 이 C가 아님 |
| 공유 입력/추론 | Interaction Manager의 로컬 제어, Model Access의 단일 weights 및 bounded scheduler | T와 공유. process 수가 늘어도 물리 CPU/메모리/전력과 OS 장애는 공유 |

대화, 업무와 정책을 모두 독립 배포해야 한다는 새 요구를 만들지 않는다. 독립 실행을 선택했을 때 발생하는 비용과 조건을 검토하는 것이다.

### 3.3 같은 사건의 정상 처리와 경합

공통 사건은 사용자의 업무 요청을 전송하기 직전 새 발화가 시작되고, 이어 “방금 요청은 취소해”라고 정정하는 경우다. 두 안 모두 Interaction Manager의 로컬 음성 중단은 즉시 수행하며 Task 취소와 구분한다.

입력 해석과 자료 확보의 Component 및 모델 역할은 T와 같다. Request Controller가 Context Manager에서 근거를 받고 Request Interpreter가 Model Access를 통해 해석 후보를 반환하는 데까지 유지한다. 아래에서 갈라지는 지점은 그 의미 확정과 업무 command를 같은 저장 단위로 묶을 수 있는지다.

| 순서/사건 | T | C-독립의 비동기 조정 형태 |
| --- | --- | --- |
| 1. 요청 확정 | Request Controller가 현재 입력과 해석을 검증. Task Manager의 command 의미, Agent Gateway의 PENDING 기록을 State Store에 내구 확정 | Request Controller가 대화 원장에 요청과 위임 의도 확정 → outbox로 Task Manager에 전달. Task Manager가 중복 검사 후 업무 원장에 Task와 command 확정 → receipt 반환 |
| 2. 전송 직전 | Agent Gateway가 동일 transaction에서 현재 admission/hold, command epoch와 정책을 검사해 DISPATCHING 확정. 네트워크 호출은 밖에서 수행 | Agent Gateway가 로컬에 반영된 대화와 정책 revision으로 전송 판단. 조회 응답 뒤 다른 owner가 바뀔 수도 있으므로 이 검사만으로 전역 최신 상태를 보장할 수 없음 |
| 3. 새 발화 | Request Controller가 hold를 먼저 commit하면 PENDING 전송 금지. 전송이 먼저 DISPATCHING이면 조회/정정 경로 | Request Controller가 대화 원장에 hold 확정 → Task Manager/Agent Gateway에 메시지. 적용되기 전 전송될 수 있음. **같은 대화 hold의 효력이 target과 같지 않음** |
| 4. 취소 해석 | 미전송은 WITHDRAWN, 이미 나갔으면 capability에 따라 cancel/조회하고 결과 안내 | 메시지 수신 후 미전송은 WITHDRAWN. 이미 나갔다면 cancel/조회. 외부 변경이 일어났다면 “미전송으로 막았다”고 할 수 없음. 취소 불가/불명도 보존 |
| 5. 결과 | Agent Gateway → Task Manager: 사건 확정 → Request Controller: 관계 갱신 → Response Manager: 실제 상태 게시 | 같은 의미 책임이나 원장 사이마다 durable 메시지와 receipt. 일부 process 중단 중에도 다른 원장의 접수는 가능하지만 전체 사용자 완료는 지연될 수 있음 |

단순 메시지형 C는 B-06의 보류 의미를 완전히 지원하지 못한다. 이 손실을 기능 완전성 및 정확성에 기록하며 낮은 전송 지연의 성공 사례로 세지 않는다.

### 3.4 철회, 질문 종료, 실패와 재시작

| 사건 | T | C-독립의 동작과 한계 |
| --- | --- | --- |
| 권한 철회 | Policy Manager의 현재 revision을 실제 읽기/제공/게시 port가 검사. 공유 확정으로 미전송 차단 | Policy Manager가 철회 기록 후 관련 owner의 사용 차단 ACK를 모아야 전역 차단을 확인할 수 있음. ACK 전에는 `철회 적용 중`이며 신규 사용 가능 구간이 남음. 현재 즉시 사용 차단과 동등하지 않음 |
| terminal 뒤 늦은 승인 | Task Manager와 Request Controller가 같은 transaction에서 질문/Execution revision 확인 | 업무 원장이 terminal을 먼저 받아도 대화 원장에 열린 질문이 남을 수 있음. 최종 승인 전송은 업무 원장의 현재 terminal로 거절 가능하나 UI 질문 종료는 늦을 수 있음. 정책 철회와의 경합은 별도 해결 필요 |
| 업무 추적 process 장애 | target에서는 해당 Core owner 실패가 Core 재시작으로 이어지는 유형이면 다른 Core 처리도 잠시 중단. 외부 connector 단독 장애는 이미 격리 | 대화와 정보 사용 process는 자체 원장 처리를 계속할 수 있음. 업무 상태는 마지막 확인 범위와 미확인을 표시. 새 업무 접수 receipt를 Agent 접수로 오인하지 않음 |
| 대화 process 장애 | Core 복원 후 질문/publication/관계 재개 | 업무 원장은 Agent 사건을 계속 내구 수신 가능. 대화가 복귀하면 사건을 재전달하고 현재 질문/결과를 조정. 즉시 사용자에게 정상 설명했다는 뜻은 아님 |
| 원장 일부 쓰기 실패 | shared State Store 쓰기 실패는 새로운 내구 확정을 막음 | 정상 원장 접수와 실패 원장 적용 사이에 미완료 관계가 남음. 관련 새 전송/게시를 막고 무한 outbox 증가를 제한. 부분 진행을 전체 성공으로 표시하지 않음 |
| 재시작, 원장과 Agent 조회 정상 | 일관된 owner 기록을 복구하고 외부 상태 조회 | 원장별 inbox/outbox와 applied revision을 대조, 미적용 메시지 재전달, UNKNOWN command 조회, 질문/정책 barrier 조정 후 관련 admission 개방. 서로 다른 시점의 snapshot을 완전한 상태로 합치지 않음 |
| 기억 삭제 | 먼저 사용 차단, 파생 view 무효화와 purge 진행 | 삭제 owner만 성공해도 다른 process의 prompt/캐시 사용이 남을 수 있음. 관련 epoch 차단 ACK와 문맥 폐기 확인 필요. 물리 삭제와 전역 사용 차단을 구별 |

### 3.5 엄격한 기능을 되찾는 가장 강한 보완과 그 비용

첫 보완은 짧은 유효기간의 권한 증표다. 그러나 증표가 남은 동안의 전송을 즉시 철회할 수 없으므로 그 자체로 해결되지 않는다. 전송 직전 동기 조회도 조회와 전송 확정 사이의 경쟁을 없애지 못한다.

엄격한 동작을 되찾으려면 입력 hold, 정책 철회와 전송 확정 사이에 **공통 순서를 결정하는 권위**가 필요하다. 하나의 전송 승인 owner로 순서를 모으거나, 여러 owner가 준비/확정을 조정하고 결정이 불명인 동안 관련 처리를 차단하는 방법을 검토할 수 있다. 후자는 중간 상태, 중복 준비, coordinator 장애와 복구 규칙까지 필요하다. 보상 취소는 이미 일어난 외부 행동을 소급 방지하지 못한다.

이 보완은 아직 완성된 세 번째 설계가 아니다. 공통 권위에 맡기는 쪽은 T의 중앙 확정 원리로 가까워지고, 분산 조정은 관련 owner가 실패할 때의 독립 진행을 제한한다. 따라서 비동기 C의 독립성 이익과 엄격한 보완의 기능 보장을 합쳐 한 방안의 장점으로 기록하지 않는다.

### 3.6 양방향 전환과 선택 조건

**T → C:** Component를 별도 process로 옮기고 같은 DB의 Unit of Work를 유지하는 가장 싼 안은 가능하다. 그러나 독립 저장 실패나 별도 상태 확정이라는 C의 성질은 얻지 못한다. 이를 얻으려면 Request Controller의 admission, Task Manager의 질문/Execution, Agent Gateway의 전송, Policy Manager의 철회와 Response Manager의 게시가 공유 확정에 기대던 부분을 메시지, applied revision과 미완료 상태로 바꿔야 한다. 이름이나 adapter 교체만으로 끝나지 않는다.

**C → T:** 기존 작업을 끝내고 원장을 옮기면 실시간 migration의 비용은 줄어든다. 그래도 별도 owner가 이미 확정한 사실을 나중에 조정하던 규칙을 공동 transaction의 검증/변경 집합으로 바꾸고, 각종 중간 상태와 재시도 의미를 정리해야 한다. Task의 외부 identity와 provider adapter까지 다시 만들 필요는 없다. 원장 사이 조정을 한 범용 계층에 잘 가뒀다면 전환비는 낮아질 수 있다.

T는 짧은 local transaction으로 필요한 정확성과 응답성을 확보할 수 있을 때 유리하다. C는 내부 owner의 독립 실패와 변경이 실제로 중요하고, 그 상황의 무관한 처리를 계속할 가치가 추가 조정과 자원 비용보다 클 때 검토할 이유가 있다. 단순 connector 실패는 그 근거가 되지 않는다. 현재 VIA 조건에서 후자의 필요성과 엄격한 기능 보완의 실익이 미확인이고, 단순 형태에는 명시적 기능 손실이 있어 **보류**한다.
