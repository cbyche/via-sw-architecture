# 의미 판단 실행, 단계 연결, 상태 확정 경계의 공식 근거

2026-10-18 / 공식 고정 코드 재확인 및 문서 조사 / 실행과 성능 측정 없음

대상은 [의미 판단 실행](../../architecture/12-decisions/decision-packages/seven-problem-review/02-interpretation-execution.md), [단계 실행 연결](../../architecture/12-decisions/decision-packages/seven-problem-review/03-execution-coordination.md), [상태 확정과 서비스 경계](../../architecture/12-decisions/decision-packages/seven-problem-review/07-lifecycle-boundaries.md)다. 제품 하나를 VIA 대안 하나와 동일시하지 않는다.

## 1. 조사 범위와 판본

아래 고정 URL에서 소스를 내려받아 표시한 함수 본문을 직접 읽었다. 기존 조사 원장의 설명만 재인용한 것은 아니다. Pipecat은 공식 저장소의 해당 파일 commit 조회로 판본을 식별한 뒤 그 commit의 소스를 읽었다. 다른 프로젝트는 이전 조사와 같은 commit을 다시 읽어 판본 혼합을 피했다. HTTP 문서는 조회일의 별도 보조 근거이며 같은 commit과 일치한다고 가정하지 않는다.

| 프로젝트 | 고정 commit | 직접 읽은 범위 |
| --- | --- | --- |
| OpenClaw | `e7a155a8ae860d71b1ab5eed45fa4af338011db9` | tool batch steering checkpoint |
| OpenHands SDK / Agent Server | `fd387b58dc7b707b73c159845c0396a17000be24` | 메시지 수신, LLM I/O lock, pause/interrupt, run permit, remote event 재동기화와 close |
| Deep Agents | `7764f1f8d21a594f0cbd31d273a7cad36597d6ba` | async task 생성과 갱신, 도구 없는 별도 질문 |
| AutoGen | `027ecf0a379bcc1d09956d46d12d44a3ad9cee14` | 중앙 runtime queue의 비동기 dispatch, Magentic-One의 다음 참여자 지정 |
| CrewAI | `3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8` | Flow runtime의 router와 listener 활성화, 후속 listener 연결 |
| Pipecat | `893a67a326f9b3f76bc32d3a2c4ed8f295fc5a6a` | 인접 processor 연결, 입력 / 처리 queue, system frame과 interruption |
| Cline | `39ff2359f7e08231281539696e48a166ce49270c` | SDK 공식 Architecture의 Local / Hub runtime 흐름 |

이 조사는 위 경로의 구현 계약을 확인한다. 배포판 전체, Windows 적합성, provider의 실제 취소 지연, queue 포화 성능, 장애 복구 성공률은 확인하지 않았다. 테스트 코드와 예제는 실행하지 않았다. `/tmp`에 받은 파일은 읽기용 사본이며 새로운 VIA 구현이 아니다.

## 2. E01: OpenClaw의 같은 실행에 입력을 반영하는 경계

- [agent-loop.ts L580~618](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/packages/agent-core/src/agent-loop.ts#L580): sequential tool batch가 이미 시작된 뒤 steering checkpoint를 읽고 미시작 tail을 처리하는 분기가 있다.
- [같은 파일 L750~768](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/packages/agent-core/src/agent-loop.ts#L750): tool batch가 끝난 뒤 다시 steering을 확인한다.
- [공식 queue 문서](https://docs.openclaw.ai/concepts/queue), 2026-10-18 확인: 같은 session의 실행 직렬화와 여러 session의 수용량을 구분한다. steer는 실행 중 도구의 즉시 중단이 아니며, 적용 불가 시 followup으로 기다린다. pending question의 답변은 별도 처리될 수 있다.

**VIA 대응:** 문제 2의 A처럼 수신과 의미 반영 시점을 분리할 수 있다. 기다리는 단위는 현재 모델 / 도구 판단 경계이며 외부 업무 전체가 아니다. queue mode별 정책을 각각 독립 Architecture로 세지 않는다. VIA의 자동 정정 범위 판단, 실제 질문 제시 기록과 원래 목표 보존은 추가 설계다.

## 3. E02: OpenHands의 수신, 판단, 취소와 실행 수용량

- [LocalConversation L1823~1885](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py#L1823): `send_message`가 MessageEvent를 상태에 추가한다.
- [L1892~1915](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py#L1892): LLM 네트워크 대기 중 상태 lock을 해제해 수신과 snapshot을 허용한다.
- [L2735~2790](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py#L2735): pause는 다음 step에서 효력이 생기며, interrupt는 추적하는 async task를 취소한다. sync run이면 pause로 돌아간다.
- [EventService L1422~1508](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-agent-server/openhands/agent_server/event_service.py#L1422): 중복 run과 permit을 확인하고 background 실행을 시작한다. sync worker는 asyncio waiter 취소만으로 종료되지 않으므로 실제 종료까지 slot을 유지한다.
- [L1970~1992](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-agent-server/openhands/agent_server/event_service.py#L1970): interrupt 뒤 cleanup 대기가 시간 초과돼도 살아 있는 run 참조를 성급하게 지우지 않는다.
- [공식 실행 중 메시지 예제](https://docs.openhands.dev/sdk/guides/convo-send-message-while-running), 2026-10-18 확인: 메시지 추가와 이후 Agent step의 소비를 구분한다.

**VIA 대응:** 입력 수신의 비동기성은 두 의미 판단의 독립성을 증명하지 않는다. 중단은 수신, 취소 요청, 실제 종료와 자원 회수로 나눠야 한다. 이 코드는 문제 2 A의 현실성과 호출 중단 보완의 비용을 뒷받침하며, DP-44 중앙 Dispatcher를 긴 동기 실행으로 약화하는 근거가 아니다.

## 4. E03: Deep Agents의 별도 실행과 상태 연결

- [async_subagents.py L430~511](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/async_subagents.py#L430): thread와 run을 생성하고 작업 ID와 실행 metadata를 `async_tasks`에 반환한다. child 완료를 기다리지 않는다.
- [L710~813](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/async_subagents.py#L710): update는 같은 thread에 `interrupt` 전략의 새 run을 만든다. 작업 identity와 실행 identity가 다르다.
- [공식 async subagents 설명](https://docs.langchain.com/oss/python/deepagents/async-subagents), 2026-10-18 확인: ASGI로 같은 서버에 배치해도 thread 상태가 분리된다. fork는 시작 시 snapshot이며 이후 부모 메시지를 자동 전달하지 않는다. worker pool 부족 시 launch가 대기하고 즉시 반복 polling은 foreground를 막을 수 있다.

**VIA 대응:** 문제 2 B의 별도 문맥과 완료 재연결에 직접 관련된다. 프로세스 분리는 필수가 아니며 서버 worker slot은 VIA의 cloud 모델 호출 slot과 다른 자원이다. VIA의 동일 대화 안 질문 답변 연결, 최신 조건에 맞는 결과 채택, provider 취소 확인과 durable recovery는 별도 계약이다. source의 업무 subagent 역할을 VIA 내부 domain planner로 가져오지 않는다.

## 5. E04: 제한된 별도 질문은 보완 구조다

[Deep Agents Btw L474~613](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/code/deepagents_code/btw.py#L474)는 snapshot에서 모델 설정을 복원하고 tools를 제거한다. `answer`는 읽기 전용 문맥과 별도 질문 이력을 사용하며 주 checkpoint를 갱신하지 않는다. 문맥 예산을 넘으면 오래된 내용을 줄이거나 overflow를 반환한다.

**VIA 대응:** 문제 2 A1의 한정된 side 답변 근거다. 사용자 자연어에서 독립 질문을 자동 분류하거나 새 자료를 읽는 기능을 이미 제공한다고 하지 않는다. VIA는 실제 표시한 side 답변을 자기 대화 원본에 남겨야 한다. 장기 조회와 독립 정정 수명까지 더하면 문제 2 B에 가까워지는 구조 변경이다.

## 6. E05: AutoGen의 중앙 운반과 중앙 의미 판단은 다른 층이다

- [SingleThreadedAgentRuntime L671~795](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-core/src/autogen_core/_single_threaded_agent_runtime.py#L671)는 공통 queue에서 envelope를 꺼내 send / publish / response 처리를 각각 `asyncio.create_task`로 시작한다. 중앙 queue가 handler 전체 완료를 기다리는 구조라는 해석은 틀리다.
- [Magentic-One L383~439](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L383)는 공유 progress ledger로 완료와 정체를 판단하고 지정한 다음 참여자에게 실행을 요청한다.

**VIA 대응:** 첫 코드는 중앙 비동기 운반의 구현 근거이며 VIA의 전체 continuation 계약을 구현한 것은 아니다. 두 번째는 의미 조정의 참고로, 코드 Dispatcher와 동일하지 않다. Magentic-One의 domain plan을 VIA에 포함하거나 topic 수로 독립 사용자 요청 처리 능력을 주장하지 않는다. AutoGen의 queue를 VIA에 필요한 bounded admission과 내구 inbox까지 갖춘 것으로 읽지 않는다.

## 7. E06: CrewAI Flow의 연결에 따른 후속 활성화

- [공개 Flow 표면](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/flow/flow.py)은 구현이 `runtime`으로 분리됐음을 명시한다.
- [runtime/__init__.py L3149~3285](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/flow/runtime/__init__.py#L3149): 완료한 method의 trigger로 router를 먼저 처리하고, 조건이 맞는 일반 listener들을 `gather`로 실행한다.
- [L3329~3425](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/flow/runtime/__init__.py#L3329): listener 완료가 다음 listener의 활성화로 이어진다. 재개 시 완료 추적과 순환 실행 제한도 관리한다.
- [공식 Flows 설명](https://docs.crewai.com/v1.15.28/en/concepts/flows), 2026-10-18 확인: `start`, `listen`, `and_` 등으로 결과 의존을 선언한다.

**VIA 대응:** DP-44 B의 생산자 결과와 소비자 실행 연결에 대한 부분 근거다. 공통 runtime 함수가 있다는 이유로 모두 A인 것은 아니다. 비교 기준은 애플리케이션의 다음 작업을 중앙 continuation이 지시하는지, 선언된 소비 조건이 활성화하는지다. CrewAI의 이 구현만으로 bounded backpressure, VIA revision join 또는 여러 동시 사용자 turn의 안전성이 증명되지는 않는다.

## 8. E07: Pipecat의 인접 처리자 연결과 별도 제어 경로

- [FrameProcessor L670~731](https://github.com/pipecat-ai/pipecat/blob/893a67a326f9b3f76bc32d3a2c4ed8f295fc5a6a/src/pipecat/processors/frame_processor.py#L670): `link`가 이전 / 다음 처리자 참조를 설정하고 `queue_frame`이 입력 queue에 전달한다. direct mode라는 다른 실행 정책도 있다.
- [L1160~1195](https://github.com/pipecat-ai/pipecat/blob/893a67a326f9b3f76bc32d3a2c4ed8f295fc5a6a/src/pipecat/processors/frame_processor.py#L1160): 다음 또는 이전 처리자의 `queue_frame`으로 직접 전달한다.
- [L1286~1339](https://github.com/pipecat-ai/pipecat/blob/893a67a326f9b3f76bc32d3a2c4ed8f295fc5a6a/src/pipecat/processors/frame_processor.py#L1286): system frame과 일반 처리 frame의 소비 task가 분리된다.
- [L835~850](https://github.com/pipecat-ai/pipecat/blob/893a67a326f9b3f76bc32d3a2c4ed8f295fc5a6a/src/pipecat/processors/frame_processor.py#L835): interruption과 cancel을 구별해 처리한다.

**VIA 대응:** DP-44 B의 인접 단계 전달을 실재하는 음성 SW에서 확인했다. 이 코드의 frame queue가 VIA의 전체 RAM 상한, durable 중요 사건 보존이나 현재 요청 효력을 자동 보장하지는 않는다. 음성 frame 처리량의 장점을 비교 범위 밖 capture 전체에 적용해 VIA 의미 단계의 지연 이익으로 세지 않는다.

## 9. E08: OpenHands의 Local / Remote 수명과 재동기화

- [공식 Conversation 설명](https://docs.openhands.dev/sdk/arch/conversation), 2026-10-18 확인: local은 process 내부 호출, remote는 agent-server에 HTTP / WebSocket을 사용한다. 동일 API 아래서 상태 보관과 실행 위치를 구별한다.
- [RemoteConversation L369~420](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/remote_conversation.py#L369): server event를 읽어 event ID 기준으로 cache와 합친다. fetch 실패 시 부분 결과가 반환될 수 있으므로 빈 cache를 완전한 최신 사실로 취급할 근거가 아니다.
- [L722~758](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/remote_conversation.py#L722), [L1846~1898](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/remote_conversation.py#L1846): 기본 `delete_on_close=False`이며 close에서 연결 정리와 server 삭제 요청을 구분한다.

**VIA 대응:** DP-42 B의 연결 종료와 상태 authority 수명 구별, 재동기화 비용의 부분 근거다. OpenHands의 remote conversation 하나가 VIA의 대화 원본과 업무 원본 두 개를 가진다는 증거는 아니다. workspace / 서버 종료에도 실행이 항상 지속된다는 보장도 아니다. local persistence와 background 실행을 지원하는 A를 약화할 수 없다.

## 10. E09: Cline의 local authority와 분리된 viewer

[SDK Architecture L153~204](https://github.com/cline/cline/blob/39ff2359f7e08231281539696e48a166ce49270c/sdk/ARCHITECTURE.md#L153)는 LocalRuntimeHost의 실행 및 저장과 Hub runtime에 대한 attach / detach를 설명한다. viewer의 연결을 끊어도 authority runtime을 멈추는 것과 같지 않다. hub는 client가 제공하는 도구 capability도 중개하므로 어떤 client가 없어져도 모든 기능이 계속된다고 해석하지 않는다.

**VIA 대응:** UI를 분리한 background 통합 Core만으로도 연결 수명 분리의 상당 부분을 얻을 수 있다는 DP-42 A의 반례 근거다. VIA의 두 상태 authority와 독립 commit까지 제공하는 완성 B의 근거로 확대하지 않는다.

## 11. E10: 실행 연결과 자원 한도의 인접 SW 근거

- [LangGraph Pregel runtime](https://docs.langchain.com/oss/python/langgraph/pregel), 2026-10-18 확인: channel을 갱신한 다음 step에서 구독 actor를 고른다. 같은 step의 actor가 모두 끝나는 경계가 있으므로 모든 graph가 즉시 downstream으로 흐르는 Pipecat식 연결인 것은 아니다.
- [Reactive Streams](https://www.reactive-streams.org/), 2026-10-18 확인: 비동기 경계의 비차단 backpressure 계약을 다룬다. 애플리케이션의 의미와 영속성을 규정하지 않는다.
- [Tokio bounded mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html), 2026-10-18 확인: bounded channel은 수용량과 송신 대기를 제공한다. 이 기법은 DP-44 A mailbox와 B stage channel 양쪽에 적용할 수 있다.
- [Microsoft Choreography](https://learn.microsoft.com/en-us/azure/architecture/patterns/choreography), 2026-10-18 확인: 다음 처리를 각 참여자가 결정하는 스타일과 중앙 조정의 차이를 설명한다. 그 문서의 독립 서비스 배치를 DP-44 B에 강제하지 않는다.

## 12. E11: 상태 확정과 저장 경합의 인접 SW 근거

- [SQLite WAL §2.2 / §2.3](https://www.sqlite.org/wal.html), 2026-10-18 확인: reader와 writer가 함께 진행할 수 있어도 같은 WAL의 writer는 한 번에 하나다. checkpoint도 추가 비용이다. 별도 DB 파일을 둔 B가 업무 쓰기와 대화 쓰기의 해당 lock 경합을 분리할 가능성은 있지만 같은 디스크의 I/O 경합은 남는다. 단일 파일 SQLite는 검토용 구체 배치이며 통합 Core의 필수 저장 기술은 아니다.
- [AWS Transactional Outbox](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html), 2026-10-18 확인: 상태와 발행할 event를 같은 local transaction에 기록해 dual write 공백을 줄인다. 소비자의 중복 처리와 순서, 서비스 간 확정 문제는 따로 남는다.

**VIA 대응:** A의 공동 확정과 B의 각 서비스 접수 / outbox / 결과 반영 차이를 실체화한다. outbox는 외부 Agent에서 실제로 한 번만 실행됐다는 증명이 아니며 단일 Core의 외부 전송에도 필요할 수 있다. B의 추가 비용에는 서비스 간 handoff용 기록만 차등으로 계산한다.

## 13. 이번 추가 조사에서 바뀐 판단

1. 문제 2의 별도 실행은 같은 프로세스 안에서도 성립한다. DP-42의 독립 서비스와 결합을 강제할 이유가 없다.
2. 중앙 queue도 비동기일 수 있으며 graph도 step barrier를 가질 수 있다. 제품 명칭만으로 DP-44의 지연 우열을 분류하지 않는다.
3. DP-44 B의 실재성은 확인됐지만 중요한 지연 / 모델 비용 차이가 증명된 것은 아니다. A의 긴 전체 요청 대기를 가정하면 다른 문제의 이익을 가져오게 된다.
4. DP-42 B에는 독립 업무 관측과 특정 저장 경합의 분리라는 구체 이익이 있다. UI 재접속과 외부 업무 지속만으로는 B를 정당화할 수 없고 일반 Agent protocol 변경도 A의 adapter에 국소화될 수 있다.
5. 세 주제의 추천과 최종 주요 DP 포함 판단은 위 사실과 VIA 사용 조건에 대한 설계 판단이다. 실행된 성능 비교, 제품 기능 보장이나 사용자의 최종 채택으로 표시하지 않는다.
