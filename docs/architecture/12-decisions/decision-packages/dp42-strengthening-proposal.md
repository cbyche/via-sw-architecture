# DP-42 B 강화 검토: 자원 회수안과 사용자 리뷰

> 2026-10-13 / **USER_REVIEWED / 자원 회수안을 주요 강화 방향에서 제외**
> 42 유지. 기존 A/B, Reference Architecture, 공통 계약과 품질 정의를 변경하지 않는다. 구현, 모델 호출, 측정과 최종 대안 선정은 이 검토에 포함하지 않는다.
> [현재 42](./04-42-lifecycle-ownership.md), [42 검토 이력](./04-42-lifecycle-review.md), [공통 실행 계약](./04-40-common-execution-contract.md), [현재 품질 정의](./03-02-quality-attribute-definitions.md), [13개 품질 관점](./03-00-quality-scenarios.md#2-어떤-품질을-보고-있는가).

## 0. 사용자 리뷰 후 현재 판단

사용자는 핵심 요약을 검토한 뒤 **메모리가 그렇게 크게 중요한 QA가 아니므로 회수를 중심으로 한 방안은 좋은 설계 대안으로 보기 어렵다**고 판단했다. 이 피드백을 반영해 **정상 대화 공백의 process 회수를 DP-42 B의 주요 강화 방향에서 제외한다.** 아래 §1~11은 첫 제안 당시의 기제, 비용과 검토 이력으로 보존하며 현재 추천으로 읽지 않는다.

문제는 회수량의 측정만 남았다는 것이 아니다. 현재 우선순위는 정확성, 상호작용 응답시간과 VIA 요청 처리시간이 앞서며, 회수안은 이 축에 기동/재결합 지연과 추가 상태 경합을 넣고 낮은 우선순위의 idle 자원 절약을 주된 이익으로 삼았다. 강한 A의 cache/worker 회수 이후에도 이 선택을 정당화할 정상 사용상 이익이 제시되지 않았다. 따라서 이 안의 측정이나 회수량 확대를 다음 주된 작업으로 삼지 않는다.

**42 자체와 기존 A/B는 유지한다.** 이번 판단은 B의 패배나 서비스 분리 전반의 기각이 아니며, 메모리 QA 및 자원 한도를 삭제하거나 완화하는 것도 아니다. §8의 PRIMARY/REGRESSION_ONLY는 첫 회수안의 탐색 분류이며 앞으로 42를 메모리 중심으로 비교하라는 결정이 아니다.

다음 강화 검토는 정상적인 답변, 정정, 질문 철회, 결과 도착과 후속 수정에서 **관련 대화 상태와 업무 상태의 확정 권한 및 실행 의존을 어떻게 다르게 구성할지**에 초점을 둔다. 새 안은 다음 조건을 충족해야 한다.

- 정확성, 상호작용 응답시간 또는 VIA 요청 처리시간이 달라지는 구체적인 정상 사건과 인과 경로를 설명한다. 의미 해석 정확성과 업무 연결/접수/전달의 정확성을 구별한다.
- 실제 owner의 원본, 중간 상태와 commit 경계를 바꾸는 기제를 제시한다. process 분리나 독립 수명이라는 이름만으로 품질 이익을 부여하지 않는다.
- 강한 A의 비동기 처리, Task별 전이, 관련 상태의 짧은 transaction과 현재성 검사를 모두 허용한다. 기존 A도 수행하는 질문 유효성 검사나 업무 event 저장을 B만의 장점으로 쓰지 않는다.
- 추가 접수/동기화와 경합 비용을 포함한다. 44의 다음 실행/대기 조직이나 45의 Context 생산/조회로 주제를 이동해 42의 이익처럼 표시하지 않는다.

이 리뷰에서 새로운 대안을 선택하거나 B의 상위 QA 이익을 확인하지는 않았다. 독립 접수와 공동 확정의 차이가 있다는 사실만으로 B의 선택 가치가 충분해졌다고 결론내리지 않는다.

## 1. 최초로 제안한 구체적인 강화

**B의 독립 업무 권위를 이용해, 업무가 진행 중이어도 처리가 없는 대화 서비스의 process를 정상적으로 회수하고 필요한 사건에서 다시 활성화하는 방향을 제안한다.** 업무 서비스는 Task, 외부 Execution, 질문, 전송 상태와 미반영 알림을 계속 소유한다. 대화 서비스는 내구 기록을 유지하되 실행 중 작업집합과 process의 private allocation을 계속 붙잡지 않아도 되는 구조로 구체화한다.

VIA 사용자는 보고서를 맡긴 뒤 문서를 읽거나 다른 앱으로 이동하고, 잠시 후 진행을 묻고, Agent 질문에 답하고, 완료된 결과를 수정한다. **업무의 관측과 제어 원본을 유지해야 하는 시간은 길지만, 요청 이해와 응답 구성이 실행되어야 하는 시간은 간헐적일 수 있다.** 그 차이를 실제 실행 자원의 수명에 연결하는 것이 이번 강화의 출발점이다. 드문 업그레이드나 장애를 대표 이익으로 삼지 않는다.

교환하는 품질은 분명하다. B는 대화 공백의 상주 자원과 다른 PC 앱과의 경합을 줄일 가능성을 얻고, 새 입력과 질문, 결과에 대한 process 활성화 및 상태 재결합 지연을 지불한다. **의미 정확성이나 기본 모델 호출 비용의 개선을 주장하지 않는다.** 메모리의 최대값이 감소한다고도 아직 주장할 수 없다. 현재 우선순위가 높은 정확성, 반응시간과 처리시간에서 손실이 크면 자원 절약으로 상쇄하지 않는다.

이 제안은 B의 정상 사용상 선택 가치를 시험할 첫 구체안이다. 강한 A의 내부 회수로 같은 효과가 나오거나 회수 대상이 작으면 추가 서비스 비용을 정당화하지 못할 수 있다. 그 결과도 42를 제외하라는 결론으로 확대하지 않는다.

### 1.1 기존 B와 제안의 경계

| 구분 | 내용 |
| --- | --- |
| 양안 공통 요구 | Voice Connection, Conversation, 개별 Request, 질문, Task와 외부 Execution의 수명이 다름. 여러 업무와 직접 대화, 조회, 답변, 정정, 결과 전달을 지원 |
| 기존 B의 기제 | 대화와 업무가 독립 상태 권위 및 process 수명을 가짐. 대화 의도 저장 → 업무 로컬 접수 → 대화 반영. 업무 event 수신과 저장은 대화 반영을 기다리지 않을 수 있음 |
| 이번 새 제안 | 정상 대화 공백을 명시적인 process 회수 기회로 사용. 회수 가능 조건, 업무 gate의 보호 상태, 내구 wake 의무, 필요한 범위의 재활성화와 재결합을 설계 |
| 유지하는 42 경계 | 상태 원본, 확정 권한, 관리 process 수명. 44의 Dispatcher/Stage 선택과 45의 과거 관계 생산/조회 방식은 동일하게 고정 |

본문의 **B 제안**은 이 추가 기제를 적용한 검토안이다. 기존 04-42의 B를 승인 없이 대체하는 새 기준선 이름이 아니다. 기존 MAIN, 배경 그림과 PPTX는 이 강화안을 설명하지 않으며 이번에 수정하지 않는다.

## 2. 먼저 이해할 같은 사용자 장면

아래 순서는 설명용이며 P21 fixture, 실제 시간이나 새로운 필수 UC가 아니다. 기능 근거는 [UC-08, 10, 13, 14, 15](../../05-representative-use-cases.md)다.

| 사건 | 사용자 장면 | 끝나는 것과 남는 것 |
| --- | --- | --- |
| E1 | C1에서 “3분기 보고서 만들어줘. 회의 안내 메일 초안도 만들어줘.” | 별도 목표의 R1과 R메일을 T1/X1, T메일/X메일에 연결. 각 위임 Request는 해당 외부 접수 확인 후 처리 완료. 업무는 계속됨 |
| E2 | 사용자가 Voice를 끄고 다른 앱에서 자료를 읽음 | 음성 연결만 끝남. C1, 업무와 외부 실행은 남음. VIA의 필요한 Text 결과 전달도 계속 지원 |
| E3 | 다른 업무에서 진행이 도착하고 보고서 Agent가 Q1 “기간은 상반기로 할까요?”를 보냄 | 진행은 해당 업무 상태에 반영. Q1은 T1/X1에 연결되고 실제 제시 P1은 별도로 기록. 사용자 답 없이 질문 대기를 끝내지 않음 |
| E4 | 사용자가 C1에 돌아와 “보고서 어디까지 됐어?” | 새 R2는 상태 조회. T1과 X1을 새 작성 실행으로 바꾸지 않음. 마지막 확인 상태와 새 조회 사실을 구분 |
| E5 | 실제 제시된 Q1에 “응, 상반기로 해줘.”라고 답하고 “아니, 하반기로”라고 정정 | R3과 정정 입력을 Q1/T1/X1에 연결. 전송 전 철회와 전송 후 정정은 다름. 다른 업무는 유지 |
| E6 | 보고서 X1이 완료되고 나중에 “결론만 짧게 고쳐줘.” | X1은 종료. 같은 목표와 결과물의 R4를 T1의 새 revision 및 X2에 연결. 이전 결과와 완료 실행의 이력을 보존 |
| E7 | C2를 열어 일반 질문을 하거나 또 대화 공백이 생김 | C1 기록과 T1은 자동 이관, 취소 또는 삭제되지 않음. 필요한 C2 처리와 업무 관측을 함께 지원 |

이 장면에서 B만 제공하는 사용자 기능은 없다. **B 제안이 바꾸는 것은 E2와 E7 같은 공백에서 업무 권위를 유지한 채 회수할 수 있는 실행 자원과 E3~E6의 복귀 비용이다.** C1/C2를 오갈 때마다 process를 하나씩 만드는 설계도 아니다.

## 3. 현재 B의 약점과 강한 A의 반론

| 현재 설명에서 이익으로 쓰기 쉬운 항목 | 강한 A가 이미 할 수 있는 일 | 강화에 사용할 판단 |
| --- | --- | --- |
| 대화와 업무의 논리 수명 분리 | 별도 identity, 비동기 상태, 내구 기록, background Core와 UI 재연결 | 필수 공통 기능. 서비스 분리의 필요성을 증명하지 못함 |
| 여러 Task 진행과 질문/결과 수신 | Task별 상태 전이, 별도 queue와 worker. 모델/네트워크 대기 중 lock을 유지하지 않음 | 전역 lock 또는 하나의 대화=하나의 업무로 A를 약화하지 않음 |
| 대화 반영을 기다리지 않는 업무 event 저장 | 대화 상태에 영향을 주지 않는 업무 사실은 Task Manager가 독립 전이로 반영 가능. 필요한 관련 변경만 공동 확정 | B의 모든 event가 A보다 먼저 확정된다는 주장 기각 |
| Agent protocol 변경 국소화 | 내부 API와 Agent Gateway adapter로 동일 의미의 wire 변경을 흡수 | B의 독립 배포와 실제 변경 책임 수 감소를 구분 |
| idle 작업집합 해제 | Request Interpreter 임시 상태, Context cache, 끝난 job 및 사용하지 않는 대화 객체를 회수 | **가장 강한 저비용 반론. A의 회수 이후 남는 자원과 B의 추가 자원만 비교해야 함** |
| 해석 worker의 process 종료 | 생성/읽기용 worker를 회수하고 Task 원본은 Core에 유지 가능 | 무거운 할당이 worker에 있으면 B 없이 process 회수 이익을 상당 부분 재현 |

현재 B는 정상 ANSWER에도 의도 저장, IPC, 업무 접수, 대화 반영과 stale 검사 비용을 지불한다. 두 서비스가 항상 상주하면 독립 수명은 가능성으로만 남고 자원 비용이 더 커질 수 있다. 따라서 이번 제안은 **무엇을 실제로 쉬게 하며 그때도 어떤 권위와 기능이 유지되는지**를 닫는다.

## 4. 검토한 강화 방향과 이번 제안의 선택 이유

| 방향 | 실제 기제 | 정상 사용상 가치와 반론 | 이번 처리 |
| --- | --- | --- | --- |
| 업무 event 확정과 대화 표시를 더 느슨하게 연결 | Task Manager가 업무 projection 및 반환 outbox를 로컬 확정하고 대화 반영은 나중에 수행 | 필요한 기반이지만 기존 B에 이미 있음. A도 독립 업무 전이 및 내구 전달 intent 가능 | 공통/기존 기반으로 유지. 단독 강화 근거로 사용하지 않음 |
| **업무 권위를 유지하며 대화 process를 회수** | 내구 대화 상태와 실행 작업집합 분리. 업무 gate 보호와 wake 의무를 확정한 뒤 process 회수, 새 사건에서 활성화 | 일상 대화 공백에서 실제 상주 자원 교환을 만들 수 있음. A의 arena/cache/worker 회수 및 B의 cold 지연이 강한 반론 | **첫 강화 제안으로 구체화** |
| 확인된 후속 Request 관계까지 업무 서비스에 인수 | 대화가 없어도 후속 조건과 명령의 해제를 업무 authority가 담당 | 정상 후속 처리의 의존을 바꾸지만 Request Graph 권한 이동과 정정/철회 전파를 새로 설계해야 함. 44와 중첩 큼 | 이번 제안에 넣지 않음. 기존 Request Controller의 의존 권한 유지 |
| process별 CPU/메모리 예약과 부하 제어 | OS 자원 경계 또는 별도 executor quota | 같은 PC 자원은 공유. A의 worker와 bounded queue로도 재현 가능. 설정만으로 상태 권위의 이익이 생기지 않음 | 자동 이익 부여 금지. 필요하면 별도 tactic으로 검토 |

선택 이유는 B의 승리가 예상돼서가 아니다. 현재 주제의 **독립 상태 권위와 실행 수명**을 정상 사용에서 직접 작동시키고 비용까지 끝까지 설명할 수 있는 방향이기 때문이다. 별도 후속 실행 엔진이나 새 의미 생산자를 추가하지 않는다.

## 5. B 제안의 owner, 실행 배치와 중간 상태

### 5.1 회수 범위를 명시한 배치 제안

기존 B에서 아직 상세 배치가 열려 있던 공통 책임을 아래처럼 둔다. 이 배치는 **이번 제안의 검토용 구체화**다. 새로운 Component 명칭이나 권위를 만들지 않는다.

| 실행 영역 | 정식 Component와 권위 | 회수 시 남거나 사라지는 것 |
| --- | --- | --- |
| 공통 입력/장치 실행 영역 | Interaction Manager. 필요한 Model Access의 입력/출력 provider adapter | 마이크 capture, 로컬 VAD, Text 입력, 근거 buffer, local stop와 실제 receipt 재전달을 유지. B 때문에 Voice session을 끊어 비용을 줄이지 않음 |
| 상주 업무 서비스 | Task Manager, Agent Gateway. 현재 권한 port를 유지하도록 Policy Manager도 이 영역에 배치 | Task/Execution/질문/명령/gate, 내구 inbox/outbox와 상태 조회를 유지. 모델 판단이나 응답 게시 권한을 인수하지 않음 |
| 회수 가능한 대화 서비스 | Request Controller, Request Interpreter, Response Manager, Context Manager. 필요한 Model Access의 의미/응답 client | 대화/Request/질문 연결, 실제 전달 원장과 허용 Memory 원본은 내구 보존. 임시 해석, 응답 구성, 원본 읽기 cache와 process의 private 작업집합은 회수 대상 |
| State Store | 각 owner의 저장/transaction port. 대화와 업무의 독립 쓰기 scope | 대화 process가 없어도 DB의 데이터는 남음. State Store를 제3의 의미 owner나 필수 DB server로 만들지 않음. 실제 DB page/cache 자원은 계상 |
| process 실행 기반 | 기존 실행 host/supervisor 역할. 작은 activation 상태와 기동 핸들 | 의미, Task 전이, 게시 차례나 후속 Request를 결정하지 않음. 신규 사용자 입력/내구 알림의 기동 요청과 process incarnation만 연결 |

Model Access는 하나의 논리 Component이고 위 client 배치는 가중치 복제를 뜻하지 않는다. **현재 비교는 04-40의 로컬 VIA와 클라우드 음성/의미 모델 조건을 따른다.** Reference Architecture의 공유 on-device Omni와 별도 ASR 설계는 보존하며 일괄 변경하지 않는다. 이 제안에서 클라우드 weights/KV를 로컬 회수량에 포함하지 않는다.

Context Manager의 원본 User Memory와 보관/삭제 상태는 대화 저장 scope에 내구 보관한다. dormant 중 새로운 Context 조회/의미 생산은 하지 않고 필요한 입력이 오면 대화 서비스를 활성화한다. Policy Manager의 철회는 상주 원본과 invalidation outbox에 기록하며, 복귀한 Context Manager와 Model Access client는 현재 policy/deletion epoch를 확인하기 전 옛 cache나 job을 사용하지 않는다. 사용자 Memory 삭제 요청 자체는 대화 서비스를 활성화해 처리한다. **기존 45A/B 선택, 생산 방식과 보관/삭제 의미를 바꾸지 않는다.** 계속 실행해야 하는 45의 maintenance가 있으면 그 기간은 회수 불가이며 자원 절감에서 제외한다.

### 5.2 대화 실행의 세 상태

| 실행 상태 | 조건과 가능한 일 |
| --- | --- |
| ACTIVE | 입력 해석, 직접 응답, 업무 명령 인계, 질문/결과 제시 또는 미해결 job 처리 중. 사용자와 필요한 반응을 끝까지 수행 |
| QUIESCING | 회수 후보. 신규 입력/필수 알림은 우선 받아 ACTIVE로 돌아감. 미전송 gate 보호, 내구 저장, 실제 전달 receipt와 job 종료를 확인 |
| DORMANT | 대화 process가 없음. 대화 원본은 내구 상태. 업무 관측과 질문 유효성 원본은 업무 서비스가 유지. 새 입력/필수 알림으로 ACTIVE에 복귀 |

이 상태는 Conversation/Request/Task 상태와 다르다. DORMANT가 C1 종료, R1 취소, Task 완료 또는 질문 만료를 뜻하지 않는다. 복수 Conversation의 기록은 유지하지만 **회수 가능한 대화 process는 하나의 공유 실행체**이며 Conversation별 process나 Task별 모델 session을 추가하지 않는다.

### 5.3 정상 회수의 구체적 조건과 추가 handshake

1. **대화 작업이 실제로 쉬어도 되는지 검사한다.** 입력 시작/전사 확정/근거 확보 중, 미완료 직접 응답, 필요한 질문/결과 전달, 진행 중 모델/read job, 미저장 receipt가 있으면 회수하지 않는다. 단순 침묵, Voice OFF나 사용자의 앱 전환만으로 회수하지 않는다. ACTIVE 유지 시간과 재기동 억제 정책은 유한 설정으로 후속 검토하며 숫자를 동결하지 않는다.
2. **Request Controller가 자기 원본을 내구 보존한다.** Request 관계와 채택 revision, WAIT_USER/WAIT_DEPENDENCY, 명령 ID, 질문/P1 참조 및 hold 의도를 저장한다. WAIT_USER의 원음과 임시 모델 상태를 영구 유지하지 않으며 기존 최소 근거/보관 계약을 따른다. 해결되지 않은 Action이 있으면 보호 hold는 유지한다.
3. **업무 gate를 보호한 사실을 확인한다.** 기존 control_session/단조 control_sequence를 사용해 관련 대화의 미전송 명령을 hold하고 HoldAck를 받는다. QUIESCING 동안 새 인계를 중단한다. ACK가 없으면 차단됐다고 선언하지 않고 process를 유지한다. 이미 DISPATCHING/UNKNOWN인 명령은 별도 원본으로 남고 기존 외부 조회/정정 경로를 유지한다. 실행 중 Task를 취소하지 않는다. hold는 해당 대화에 연결된 미전송 효과를 보호하며, 기존 업무 event 수신/상태 조회나 무관한 Task를 전역 차단하지 않는다.
4. **Response Manager가 실제 전달 checkpoint를 저장한다.** 질문 P1의 실제 제시, Text/Voice의 별도 범위, 중단/불명 상태를 저장한다. Interaction Manager의 미반영 receipt가 있으면 반영하거나 명시적 UNKNOWN으로 처리한다. 전달 완료를 임의로 만든 뒤 회수하지 않는다.
5. **추가 제안인 회수 handshake를 완료한다.** Request Controller는 `PrepareDormancy(dormancy_id, activation generation, conversation refs, control session/sequence, expected gate revisions, durable checkpoint refs)`를 제출한다. 업무 서비스는 gate 보호, 미반영 중요 event와 wake 등록 상태를 검사해 `DormancyReady(dormancy_id, gate revisions, notice watermark, wake sequence)`를 로컬 기록/반환한다. 이미 미반영 중요 event가 있으면 Ready를 반환하지 않고 대화 활성화를 요청한다. 같은 ID와 payload의 재수신은 같은 결과를 조회하고, 다른 payload나 오래된 세대는 거절한다. 이는 업무 접수나 사용자 전달 ACK가 아니다. 회수 중 새 필수 event/입력이 생기면 회수를 취소하거나 다음 활성화를 지속 요청해야 한다.
6. **process 실행 기반이 회수와 새 wake의 경합을 판정한다.** host는 해당 activation generation과 checkpoint의 회수 요청만 받는다. 자기 원본의 wake sequence가 진행했거나 미처리 기동 요청이 있으면 회수를 취소한다. 기동/종료 조작은 incarnation별로 순서를 정해 오래된 종료가 새 process를 죽이지 않게 한다. 완료 응답 유실은 같은 dormancy_id로 조회하며 조용히 gate를 해제하지 않는다. 업무와 host 저장을 하나의 전역 transaction이라고 가정하지 않으며 그 사이 event는 §5.4의 내구 재전달로 처리한다.

기존 B의 “control 단절 감지 후 보호 hold”보다 **정상 회수는 hold ACK를 먼저 받는 명시적 경로**다. 감지 지연을 정상 회수의 안전성 근거로 쓰지 않는다. 비정상 중단에서는 기존 lease/감지/UNKNOWN 계약을 그대로 사용한다. 회수는 미전송 명령을 허가한 상태에서 대화 authority를 없애는 shortcut이 아니다.

### 5.4 dormant 중 업무와 필수 알림

Task Manager와 Agent Gateway는 기존 수신/조회, source revision, event ID 및 command identity로 업무 원본을 확정한다. 질문 철회/실행 종료는 업무 원본에서 먼저 적용하고 대화에 보낼 outbox를 함께 저장한다. 다른 Task의 진행이나 결과가 새 입력 해석을 기다려야 한다는 의존을 추가하지 않는다. A도 이런 독립 업무 전이는 가능하다.

**사용자에게 제시해야 할 질문, 결과, 실패, 확인된 제어 결과와 Request 의존 해제 사건은 대화 서비스의 활성화를 요청한다.** 중요 event의 wake 의무와 소비 cursor는 내구 상태이며 메모리 callback 하나로 대체하지 않는다. Task Manager는 source 사실과 재연결용 참조를 제공하고, 최종 공개 범위/질문 focus/게시 admission은 Request Controller가 결정한다. 의미나 공개 필요성을 판단할 수 없으면 활성화해 기존 owner가 판단한다. task event만으로 승인이나 실제 제시 P1을 만들지 않는다.

업무 서비스는 source fact, 반환 outbox와 wake 의무를 같은 업무 로컬 확정으로 저장한 뒤 host에 반복 전달한다. host는 기동 요청을 내구 등록한 ACK를 반환하며, 이 ACK는 업무의 소비 cursor나 사용자 전달 상태를 완료로 바꾸지 않는다. Request Controller가 해당 notice 또는 최신 snapshot을 자기 저장 scope에 반영하고 반환한 적용 ACK 뒤에만 업무의 wake 의무를 해소한다. 전달 원본과 실제 Text/Voice receipt는 그 이후에도 각 owner가 관리한다. host 등록 전 대화가 종료되는 경합에서는 업무 의무가 남아 재기동을 요청한다. host 등록 뒤 종료와 경합하면 host의 세대 검사로 회수를 취소하거나 새 incarnation을 기동한다. 재전달/기동에는 유한 대기와 실패 기록이 필요하며 무지연 wake나 정확히 한 번의 IPC를 보장하지 않는다.

대화 적용 ACK 뒤에도 미완료 게시와 Request 처리는 대화의 내구 pending 원본에 남는다. host는 정상 회수 handshake를 완료한 incarnation의 종료만 DORMANT로 인정한다. ACTIVE 중 예상하지 못한 종료는 복구 기동 대상으로 유지한다. 업무 wake가 해소됐다는 이유로 아직 전달하지 않은 결과를 완료 처리하거나 이 복구 의무를 지우지 않는다.

일반 progress의 합치기는 양안 공통 정책 안에서만 가능하다. 원본 질문/철회/terminal/제어 사실은 버리지 않는다. **알림을 늦추고 dormant 시간을 늘려 이익을 만드는 것은 금지한다.** Voice OFF라도 Text 결과와 필요한 알림을 전달해야 하므로 해당 event는 즉시 활성화 경로로 간다. 대화 서비스가 깨어나지 못하면 업무 fact와 미전달 상태를 보존하고 전달 실패를 남긴다.

기동 host는 이미 있는 공통 입력 실행 영역의 작은 실행 기반으로 제안한다. 제3의 Dispatcher나 항상 켜진 Request Controller 복제본을 숨기지 않는다. wake 자료는 ID, cursor, activation generation과 이유 정도이며 전체 대화나 Task payload를 또 보관하지 않는다. host/IPC/감시 queue의 실제 비용도 B에 포함한다.

### 5.5 재활성화와 현재 질문 답변

1. 새 Voice/Text 입력 또는 내구 알림이 wake를 요청한다. Interaction Manager는 그 동안에도 capture와 로컬 stop를 수행하고 유한 buffer/producer watermark 또는 gap으로 입력을 보존한다. 대화 기동 지연을 입력 성공으로 감추지 않는다.
2. Request Controller와 Response Manager가 **필요한 Conversation, pending Request/질문 및 실제 전달 원장**을 읽는다. 2,000turn나 모든 완료 Task를 RAM/모델 입력에 복원하도록 강제하지 않는다. Context Manager의 재조회 범위는 고정한 45 선택을 따른다.
3. 업무 서비스의 현재 snapshot, command 접수/전송 상태, 질문 유효성, event watermark 이후 변경과 현재 권한을 대조한다. snapshot과 이후 event의 gap은 재조회한다. 대화 원본과 업무 원본이 전역 동시 snapshot이라고 주장하지 않는다.
4. 업무 authority가 발급한 새 control_session에서 gate는 보호 상태로 시작한다. Request Controller가 현재 입력/의존 관계를 해소한 뒤 기존 ReleaseAndReconcile로 관련 명령만 유지, 철회, 교체하거나 해제한다. dormant 이전 release와 늦은 ACK로 새 hold를 풀지 않는다. C2의 새 입력을 해소했다고 C1의 WAIT_USER/WAIT_DEPENDENCY 또는 hold를 함께 해제하지 않는다.
5. “응, 상반기로”이면 Request Interpreter가 P1과 현재 Q1 후보로 의미를 제안하고 Request Controller가 답변 관계를 채택한다. 의미 후보의 Task 정답을 wake host나 업무 service가 미리 채우지 않는다. Q1이 이미 철회/만료됐다면 유효 답변처럼 접수하지 않는다.
6. Request Controller가 R3의 CommandIntent K1을 대화 scope에 저장하고 업무 service가 Q1/T1/X1/gate/권한 조건을 검사한다. Q1=ANSWERED와 K1, AcceptanceReceipt를 업무 로컬 확정한다. Request Controller가 receipt를 별도로 반영한다.
7. Agent Gateway는 현재 조건을 검사해 PENDING→DISPATCHING CAS 후 외부 전송한다. 외부 실제 접수는 별도 receipt/조회로 확인한다. 답변 직후 “아니, 하반기로”가 오면 기존 hold 먼저/dispatch 먼저의 두 분기를 적용하고 새 K2를 이전 command/Execution에 연결한다.
8. 확인된 외부 사실만 Response Manager에 admission한다. Text 표시, Voice 생성/차례/release와 Interaction Manager의 실제 receipt를 거쳐 전달 원장을 저장한다. 대화 process가 기동됐다는 사실이나 내부 ACCEPTED는 이번 요청의 완료 endpoint가 아니다.

기존 WAIT_DEPENDENCY는 Request Controller가 소유한다. 결과 도착은 활성화 이유이며 업무 서비스가 임의로 후속 Request를 해제하지 않는다. 대화 기동/재결합 시간은 VIA 귀속 처리 지연에 남긴다.

## 6. 같은 정상 사건의 A와 B 제안을 끝까지 비교

| 사건 | 강한 A의 입력, 확정 및 실행 의존 | B 제안의 입력, 확정 및 실행 의존 | 사용자에게 보이는 결과와 구조 비용 |
| --- | --- | --- | --- |
| E1 두 업무 위임 | Request Interpreter의 의미 → Request Controller/Task Manager의 관련 변경 → State Store의 공동 확정. 외부 전송/실제 접수는 transaction 밖 | 같은 의미 → 대화 의도 저장 → 각 업무 명령/Task의 로컬 접수 → 대화 반영. 외부 전송/실제 접수 별도 | 같은 목표/제약으로 별도 업무를 맡김. B의 추가 저장/IPC/중간 상태. 두 안 모두 접수와 완료 구별 |
| E2 공백 | background Core에서 업무 추적. 대화 workspace/cache/worker를 회수할 수 있음. 업무 원본 owner와 공동 확정 기반은 상주 | §5.3의 보호/내구 handshake 뒤 대화 process 회수. Task Manager/Agent Gateway는 상주 | 업무 지속 기능 동일. A의 실제 회수 후 남는 private 자원이 있을 때만 B의 추가 회수 여지 |
| E3 진행/질문 | 해당 업무 projection과 필요한 outbox 저장. 질문 제시와 실제 P1 기록. 대화 처리와 무관한 전이는 비동기로 진행 가능 | 업무 snapshot/질문/outbox 확정 → wake → 대화의 현재성 확인과 admission → P1 실제 전달/저장 | Q1의 동일한 Text/필요 Voice. B의 기동 비용이 source availability부터 포함. 질문은 답변 대기 유지 |
| E4 상태 조회 | 현재 T1/X1 확인으로 R2 직접 응답 가능. 필요하면 외부 상태 조회. 작성 start 없음 | 대화가 dormant면 기동/재결합 뒤 업무 read. 이미 ACTIVE면 일반 IPC read | 같은 상태와 확인 시점 안내. B가 새 작성 start나 자동 전체 모델 재해석을 추가하지 않음 |
| E5 유효 답변/정정 | 실제 P1 및 현재 Q1/T1/X1 검사. R3 연결 + 질문/명령을 공동 확정. hold/CAS 선후로 철회/정정 분기 | §5.5의 재결합과 독립 접수/반영. 업무 gate가 hold/전송 CAS 선후를 확정 | 확인된 전달/정정 상태만 안내. B의 stale view, ACK 유실, UNKNOWN 재조회 비용을 포함 |
| E6 완료 후 결론 수정 | Agent source의 완료/결과를 T1/X1에 반영해 전달. 새 R4의 수정 의미를 채택해 T1 새 revision/X2 연결 | source 완료를 업무 로컬 확정 → 필요하면 대화 wake/전달. R4 의도와 업무의 새 revision/X2 접수를 별도 확정 | 같은 결과물의 새 실행, 이전 X1은 종료 이력. B의 대화 회수로 후속 Task identity가 사라지지 않음 |
| E7 C2 또는 새 공백 | C2의 workspace만 구성 가능. C1 이력을 유지하고 Task별 업무는 계속 | 공유 대화 service의 활성 범위를 C2로 전환. C1 관련 필수 알림도 기존 정책대로 처리. 다시 회수 가능한 조건에서만 dormant | 서로 다른 대화의 “응”을 Q1에 자동 적용하지 않음. B의 process 수가 Conversation 수만큼 증가하지 않음 |

**지속적인 대화와 여러 사건이 겹치는 S5-04에서는 대화 서비스가 ACTIVE여야 한다.** 그 구간에서 B 제안은 특별한 회수 이익을 갖지 않는다. A의 비동기/priority/worker와 B의 독립 업무 전이는 모두 유지한다. process 경계가 같은 CPU, 디스크와 모델 quota 경합을 없애지 않으며 어느 안에서도 별도 Task들을 전역 lock으로 막지 않는다.

## 7. 공통 계약을 보존하거나 추가하는 곳

| 계약 | 이번 제안의 처리와 비용 |
| --- | --- |
| hold 의도, 적용 HoldAck, release와 transmission CAS | 기존 의미 보존. 정상 회수 전에 적용 ACK를 요구하는 추가 handshake. 물리 발화 시작부터 무지연 차단을 주장하지 않음 |
| control_session/sequence/gate revision | 복귀 시 새 세대와 보호 gate. 과거 release 거절. 재결합/새 세대 협상 비용 추가 |
| command identity와 payload | 같은 command ID의 같은 payload는 같은 내부 접수 결과 조회. 다른 payload는 새 ID와 supersession 관계. 내부 중복 제거와 외부 idempotency 구별 |
| 내부 접수와 외부 실제 접수 | 기존 구별 유지. 회수 handshake가 새 ACCEPTED나 외부 접수 사실이 되지 않음 |
| Q1의 현재 유효성과 실제 제시 P1 | 업무 원본과 Response Manager의 전달 원장을 복귀 때 재결합. 철회/미제시/다른 대화의 질문은 답변 근거로 사용하지 않음 |
| Response Manager의 실제 전달 원장 | owner 유지. process 회수 전에 내구 checkpoint. 실제 장치 출력과 내구 기록의 원자성은 새로 보장하지 않음 |
| Request 의존 관계와 의미 채택 | Request Controller 유지. 대화 service가 없어도 자동 후속 실행한다는 기능을 추가하지 않음. 결과 도착 시 활성화 비용 발생 |
| event/outbox/cursor와 snapshot | 기존 B의 수렴 계약 유지. 추가 wake 의무/activation generation과 회수 경합 확인. 영속 event log를 최종 원본으로 강제하지 않음 |
| policy/삭제 및 외부 제공 | 현재 원본 검사와 epoch fence 유지. dormant cache는 없더라도 내구 파생물/외부 제공 이력은 삭제 완료로 간주하지 않음 |
| sleep, OS lock, shared disk/model 장애 | 기존 제한 유지. 입력/공개 통제와 resume 재조회. 업무 서비스도 OS sleep 중 계속 동작한다고 주장하지 않음 |

응답 유실, wake 유실, process 기동 실패와 partial attach가 늘어난다. handshake나 기동 실패 시 업무의 확인 가능한 사실과 미전달 상태를 유지하며, 확인 없는 새 start와 자동 gate 해제를 하지 않는다. B의 회수 가능성은 외부 Agent의 cancel/correct/status/idempotency 기능을 늘려주지 않는다.

## 8. 현재 여섯 QA에서 실제로 주장할 수 있는 것

현재 비교의 ID는 **ASR-QA-01~06**이고 [03-02의 P21](./03-02-quality-attribute-definitions.md)이 원본이다. Reference Architecture와 기존 Core QA-09/19/29/39, QA-41은 별도 세대의 정의/지위를 보존한다. 이번 제안은 목표, 점수, 분모, 장치 또는 비교 우선순위를 개정하지 않는다.

이번 강화의 직접 교환 축인 02/03/06은 `PRIMARY`, 정확성 및 비용/변경 계약을 보존할 01/04/05는 `REGRESSION_ONLY`로 기록한다. 이 분류는 현재 우선순위를 바꾸거나 평가 축을 제외하지 않는다. 승인된 비교를 수행한다면 여섯 적용 축을 모두 같은 계약에서 관측해야 한다.

| 현재 QA | 구조의 물리적 영향과 첫 가설 | 한계와 관찰할 것 |
| --- | --- | --- |
| **ASR-QA-01 요청 처리 정확성** | 독립 접수/복귀 중 command/질문/receipt 재결합이 C3/C5/C6에 직접 참여. 정상 공백에도 같은 원본 유지 | 의미 정확도 우세 없음. missing wake, 오래된 질문, 잘못된 외부 접수/전달 주장과 새 실행 중복은 추가 회귀 위험. 더 작은 working set이 정확성을 자동 개선하지 않음 |
| **ASR-QA-02 상호작용 응답시간** | ACTIVE의 추가 handoff와 DORMANT의 기동/원본 read/재결합이 경로에 들어감 | 낮은 부하에서 A 유리 가능. B의 짧은 기동을 가정하지 않음. S5-03/04, S6-04의 실제 endpoint와 local stop 분리. 중요한 event wake를 늦춘 절약 금지 |
| **ASR-QA-03 VIA 요청 처리시간** | 추가 확정, 현재 검사, pending receipt 조정 및 Request 의존을 가진 복귀가 cycle에 참여 | 외부 Agent 일과 사용자 대기는 고정. 그 동안 수행한 VIA 재결합과 뒤따른 queue 비용은 남김. 업무 source 확정과 사용자 전달 완료 구별 |
| **ASR-QA-04 모델 사용 비용** | 회수/기동/receipt 조정은 코드. 기본 입력/의미/응답 호출 조건은 동일 | 기본 감소 가설 없음. context 재구성/폐기/재해석이 실제 모델 호출이나 청구 cache miss를 만들면 비용 증가. 무음/idle 청구가 자동 0이라고 하지 않음 |
| **ASR-QA-05 변경 용이성** | U13 인계, U14 저장/복귀 및 관련 U04/U06/U07 책임의 wake/checkpoint 계약. M-05 질문 계약 변경에도 회귀 필요 | 일반 provider/Agent/source wire 변경은 A도 adapter에서 끝남. 새 supervisor만 추가해 변경 책임 수가 감소했다고 계산하지 않음. 실제 최소 patch 전 수치 미정 |
| **ASR-QA-06 로컬 VIA 메모리** | dormant의 대화 process private allocation 회수와 active의 추가 업무 runtime/IPC/queue/copy가 교환됨 | **idle 감소는 현재 peak 지표 개선과 다름.** 동시 ACTIVE와 기동 중 이중 할당에서 최대값이 같거나 더 클 수 있음. 전체 동시 commit 관측 전 우세 미정 |

### 8.1 자원 이익의 성립 조건

회수 효과는 다음 세 가지를 구별해야 한다.

1. **필수 데이터:** Task/Execution/질문/전달/대화 원본은 양안 모두 보존한다. B가 덜 보관하거나 A가 모든 과거를 RAM에 복원한다는 차이를 만들지 않는다.
2. **회수 가능한 작업집합:** 양안 모두 끝난 job, cache와 대화 workspace를 해제할 수 있다. B의 잠재적 잔여 이익은 A의 최선 회수 뒤에도 남는 대화 관련 process/runtime private commit을 대화 process 전체 회수로 줄일 수 있는 부분이다. OS paging이나 단순 object free를 동일한 회수량으로 단정하지 않으며 실제 allocation과 commit을 확인해야 한다.
3. **추가 B 비용:** 별도 업무 process/runtime, 독립 저장의 page/cache, command/receipt 중간 기록, IPC 복사, 내구 알림, wake host 및 반복 복귀의 일시 할당. 이 비용을 뺀 이후에 순절약이 남아야 한다.

상주 시간이나 에너지의 차이는 진단 가설이다. 현재 ASR-QA-06의 대표값은 전체 활동의 **최대 동시 commit**이므로 평균/idle 값으로 바꿔 B를 유리하게 만들지 않는다. idle 차이가 있더라도 peak 차이와 상위 QA 손실을 별도로 보고해야 한다. current P21의 idle 10초가 회수 경로를 활성화하기에 충분한지 미확인이며 fixture를 이 제안에 맞춰 늘리지 않는다.

외부 API 가격, 초 단위 기동, MiB 절약량이나 호출 감소 수치를 이번에 산출하지 않았다. 모두 문서 수준의 인과 가설이며 DP41의 이전 실측치를 42 또는 이 기제의 근거로 재사용하지 않는다.

## 9. 13개 품질 관점으로 누락과 비용 확인

| 관점 | B 제안의 조건부 효과와 강한 A 대비 한계 |
| --- | --- |
| V-01 기능 정확성 | 현재 업무 원본 유지. 복귀 중 stale 질문/receipt/command와 wake 경합이 새 오류 경로. A도 같은 검사를 수행하며 의미 판단 우세 없음 |
| V-02 기능 적절성 | 공백 뒤 같은 업무/질문을 재설명하지 않아야 함. A도 지원. 복귀 실패를 사용자의 수동 thread 연결로 떠넘기면 손실 |
| V-03 기능 완전성 | 공통 UC 유지가 목표. dormant 때문에 필요한 Text/Voice 알림, 답변 또는 의존 해제를 빠뜨리면 기능 손실. 미지원/실패를 절약 성공으로 세지 않음 |
| V-04 상호작용 반응성 | 기동과 재결합의 추가 지연. 메모리 압박이 줄어 실제 처리 지연도 줄 수 있는 경로는 있으나 A의 회수/worker와 비교 전 미확인 |
| V-05 VIA 귀속 요청 완료 시간 | Request 의존 권위는 대화에 남음. 해당 사건에 기동 비용 추가. 기존 업무 관측만 계속되는 것은 최종 전달 완료의 개선이 아님 |
| V-06 자원 활용성과 수용량 | 대화 공백의 private 자원 회수 가능성과 active 추가 자원 교환. 에너지/수용량 자동 증가 없음. 적은 process 수 또는 짧은 idle만으로 결론 금지 |
| V-07 결함 허용성과 복구성 | 업무 owner의 지속성과 추가 wake/attach 실패. 정상 회수와 crash 복구 구분. shared OS/디스크/모델 문제의 우세 없음 |
| V-08 변경 용이성과 모듈성 | 실행 수명/checkpoint/handoff를 명시해 경계를 관리. 계약 의미가 바뀌면 producer/consumer와 이행이 함께 변경. A의 내부 API 우위 재현 인정 |
| V-09 분석 및 시험 용이성 | activation generation, gate ACK, event cursor와 실제 receipt의 인과 추적 필요. 단독 서비스 검토와 복잡한 전체 경합 분석의 교환 |
| V-10 기밀성 | dormant의 임시 복사 감소 가능. 내구 원본/IPC 복사와 철회 재확인은 계속 필요. 동일 OS 권한 process 분리를 보안 containment로 주장하지 않음 |
| V-11 상호운용성과 공존성 | Agent 지원 capability 동일. 다른 Office/browser 앱과의 자원 공존 개선 가능성은 실제 순회수량과 cold 부하에 달림 |
| V-12 조작 용이성과 사용자 오류 방지 | 사용자는 process 상태를 조작할 필요 없이 같은 Task/질문을 제어. 대화 준비 중을 외부 접수/완료로 안내하면 실패 |
| V-13 설치 용이성 | 추가 실행체/감시/기동/버전 호환 비용. 같은 설치 package 가능성은 설치 및 제거 실패 처리의 근거를 대신하지 않음 |

## 10. 저비용 A 보완과 반증 조건

### 10.1 A에 허용할 최소 보완

첫 비교에서 A에 다음을 모두 허용해야 한다.

- 대화별 workspace의 lazy load 및 idle eviction. 실제 전달/Request/Task 원본은 내구 보존.
- Request Interpreter 및 무거운 read/response 준비의 worker 격리와 종료. Task 원본과 공동 확정 권위는 Core에 유지.
- 요청/대화 allocation arena, cache 상한, DB cache 조정, 모델 client의 임시 payload 해제. 필요한 protected input과 receipt는 보존.
- Task별 event queue, progress coalescing, async 처리와 짧은 관련 transaction. 모델/네트워크 대기 및 무관한 Task의 전역 lock은 없음.

이 보완으로 A가 동일한 자원 회수와 상위 품질을 얻으면 B의 정상 회수 이익은 작아진다. 회수 가능한 계산 worker만 분리하는 A의 혼합은 유력한 비교 대상이다. 공동 확정과 Task 상태 권위를 유지하므로 자동으로 순수 B가 되지는 않는다. 반면 Request Controller/Response Manager의 내구 권위와 실행 수명까지 다른 process로 옮기고 command/receipt 재결합으로 바꾸면 실제 42 선택이 이동한 것이므로 단순 설정 변경이라고 부르지 않는다.

### 10.2 주장을 반박하는 조건

| 반증 조건 | 받아들일 결론 |
| --- | --- |
| A의 cache/arena/worker 회수 뒤 남는 대화 private commit이 B의 추가 상주 비용보다 작음 | B 제안의 순자원 절감 근거 없음 |
| 실제 대화/필수 Agent 알림이 자주 와 dormant가 거의 없거나 process가 반복 기동됨 | 정상 사용상 이익이 작고 기동/청구/동기화 비용이 지배 가능 |
| idle 자원은 감소하지만 전체 동시 peak는 동일/증가 | ASR-QA-06 개선 주장 기각. idle/공존 가설만 진단으로 남김 |
| 새 입력/질문/결과의 기동 지연 또는 재결합이 상위 시간 품질을 악화 | 자원 절약만으로 B를 선택할 근거 부족. warm 유지/회수 축소의 비용을 다시 포함 |
| wake/gap/실제 전달 재결합 오류로 질문, 제어 또는 결과가 누락/오연결 | 기능 손실. 낮은 자원 값으로 상쇄하지 않음 |
| 추가 검증이 모델 재해석/Context 재생산을 늘림 | 기본 모델 비용 동등 가설 수정. 실제 폐기/재시도 비용 포함 |
| 동일한 owner 계약을 갖춘 A worker 혼합으로 같은 회수와 기능을 더 작게 달성 | 해당 혼합의 강점을 인정. B만의 이익으로 발표하지 않음 |
| 회수와 복귀를 검토하는 데 44 후속 실행 또는 45 과거 생산을 함께 바꿔야만 함 | 순수 42 강화라는 범위 주장을 재검토. 변경을 숨긴 조합 비교 금지 |

**B 제안을 더 살펴볼 조건**은 긴 업무 추적 중 대화 공백이 반복되고, 회수 대상이 실제로 크며, A의 저비용 회수 뒤에도 의미 있는 잔여 자원이 남는 경우다. 기동/재결합 뒤에도 같은 질문/명령/전달 계약과 상위 시간 품질을 유지해야 한다. 이 조건은 사용자 기능이나 제품 빈도가 확인됐다는 사실이 아니라 선택 가치의 가설이다.

## 11. 첫 제안 당시의 검토 결론과 남은 판단

기존 B의 독립 운영을 **정상 대화 공백의 자원 회수 → 필요한 사건의 활성화 → 현재 업무/질문/전달 원장의 재결합**으로 구체화했다. 새 박스 수나 서로 다른 논리 수명만으로 설명하던 것보다 실제 관리 process, 저장 확정, 실행 의존과 비용이 드러난다.

그러나 현재 중요한 품질에서 B의 우세를 확보한 것은 아니다. 오히려 이 제안은 **자원을 쉬게 하는 이익과 대화/결과 복귀 지연의 교환**을 분명히 하며, 강한 A와 현재 peak 메모리 지표 아래서는 이익이 작을 가능성을 열어 둔다. 42는 유지하고 이 기제를 사용자 검토 대상으로 제시한다. 원본 B 채택/변경, 다른 강화 방향 선정, Stage 5/6, 구현/측정과 figure/PPTX 동기화는 별도 결정이다.

사용자 검토의 핵심은 이 정상 사용상 교환이 42에서 발전시킬 가치가 있는지, 그리고 A의 cheap conversion 이후에도 어떤 계약/자원 차이가 남아야 설득력이 있는지다. 다른 강화 방향을 택하더라도 현재 A/B와 이번 검토 기록을 이력으로 보존한다.

## 12. 작성 및 검증 범위

작업 시작 branch는 main이고 시작 HEAD는 `a73d3b3d8`이다. 기존 04-42 배경 본문, 공용 workplan/README/AGENTS.md, 40/41/44/45/46/04-50, 배경 그림/공용 생성기와 work 디렉터리에 병행 변경이 있었다. 이 작업의 write 범위는 **이 새 Markdown 한 파일**이다. 기존 본문, MAIN/배경 그림, 발표자료, 공용 생성기, navigation과 실험 자료를 수정하지 않는다.

설계의 자체 대조 범위는 E1~E7, 정상 답변 및 두 정정 경합, dormant 전/후 질문 철회, event/ACK 유실, 새 입력과 회수 경합, WAIT_DEPENDENCY, 실제 receipt/권한/삭제의 복귀다. 이 대조는 실행 protocol proof, 독립 심사 또는 측정이 아니다.

| 자체 대조에서 발견한 모호성 | 반영한 수정과 남는 한계 |
| --- | --- |
| 회수 응답 유실 뒤 동일 handshake를 무엇으로 조회하는가 | dormancy_id와 동일 payload 재조회, activation generation 및 gate revision 조건 명시. 실제 schema/구현은 없음 |
| 업무의 새 event가 host 등록 전에 오거나 process 종료와 경합하면 wake가 사라지는가 | 업무 fact/outbox/wake 로컬 확정, host 내구 등록 및 incarnation별 기동/종료, 대화 내구 반영 ACK 전 wake 의무 유지. 재전달 지연/실패 가능성은 남음 |
| 새 C2의 복귀가 C1의 미해결 hold를 함께 풀거나 무관한 업무를 멈추는가 | 대화별 보호 및 선택적 reconciliation 명시. 기존 업무 추적과 무관한 Task는 계속 진행 |
| idle 자원 감소를 현재 메모리 QA 개선으로 부를 수 있는가 | 전체 동시 peak와 분리. 현재 지표 개선은 미확인이고 A의 저비용 회수 이후 순이익만 남김 |

2026-10-13 첫 작성의 문서 검사에서 `.venv/bin/python scripts/architecture/check_active_markdown_links.py`가 활성 Markdown 391개의 로컬 링크에 PASS를 반환했고, `.venv/bin/python scripts/architecture/check_active_terminology.py`가 활성 용어와 정식 target Component 명칭에 PASS를 반환했다. 새 파일의 diff와 공백, 중점 문장부호 및 활성 참조도 확인했다. 사용자 리뷰 후에는 이 파일의 현재 상태와 §0만 추가하고 첫 제안의 본문을 보존했다. 이 검사는 문서 일관성을 확인하며 B의 안전성, 성능 또는 선택 가치를 검증하지 않는다.

사용자 리뷰 반영 뒤 용어 검사, `git diff --check` 및 이 파일의 공백/문장부호 검사는 통과했다. 같은 링크 검사기의 규칙을 이 파일에 적용한 로컬 링크 검사도 통과했다. 커밋 전 전체 링크 검사에서는 병행 작업 중인 04-44-continuous-interaction.md가 참조하는 `dp44-common-path.png`가 없어 실패했다. 해당 참조는 이 파일에 없으며 이번 write 범위 밖이므로 수정하지 않았다. 첫 작성의 PASS와 이번 전체 검사 실패를 구별한다.
