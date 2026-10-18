# 동시 입력과 Agent 사건 처리의 고정 소스 근거

2026-10-16 / [VIA 문제 2 설계 비교](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/02-continuous-interaction.md)의 근거 원장

## 조사 방법과 판본

공식 저장소의 경로 검색 후 아래 commit으로 파일 본문을 다시 읽었다. 검색 결과의 최신 HEAD는 경로 발견에만 사용했다. 이전 [harness 조사](../agent-harness-20261015.md)의 관련 경로를 재확인하고 실제 run과 입력 반영 코드까지 범위를 늘렸다. 어떤 harness도 실행하지 않았으며 아래 내용은 해당 snapshot의 확인한 경로에 한정한다.

| 프로젝트 | 고정 commit | 이번 본문 확인 범위 |
| --- | --- | --- |
| OpenClaw | `e7a155a8ae860d71b1ab5eed45fa4af338011db9` | queue와 steering 문서, tool loop, active reply adoption, subagent 완료 lease, transcript writer 확인 |
| OpenHands SDK와 Agent Server | `fd387b58dc7b707b73c159845c0396a17000be24` | LocalConversation 메시지와 lock/pause/interrupt, EventService 메시지와 run/rearm/interrupt |
| Deep Agents | `7764f1f8d21a594f0cbd31d273a7cad36597d6ba` | async subagents의 start/check/update/cancel와 Btw 별도 질문 |
| AutoGen Magentic-One | `027ecf0a379bcc1d09956d46d12d44a3ad9cee14` | orchestrator의 task/progress ledger와 다음 참여자 선택, save/load |
| CrewAI | `3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8` | Crew kickoff_async와 native async Task join, conversational turn 초기화와 이력 경로 |

OpenHands Canvas의 과거 Controller/EventStream을 현재 SDK의 실제 경로로 대신 설명하지 않는다. 최신 배포 조합이나 호환성을 실행 검증한 기록이 아니다. 각 source의 L 번호는 고정 commit의 줄 번호다.

## 1. OpenClaw: 입력 합류와 기술적 쓰기 권한

### 실제 입력 반영 경계

- [queue.md L18~58](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/queue.md#L18): session lane 직렬화와 global capacity를 구별한다. 같은 turn steering과 후속 turn, 호환 입력 collect, interrupt를 지원한다. 일반 입력의 저장과 실행 admission도 구별한다.
- [queue-steering.md L16~50](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/queue-steering.md#L16): 이미 실행 중인 도구는 steering 때문에 즉시 중단되지 않는다. sequential tail의 미시작 호출을 건너뛰고 paired 결과를 남길 수 있다. parallel batch는 시작을 막지 않고 batch 뒤 입력을 읽는다. tool 결과 뒤 새 입력을 넣고 다음 모델 판단으로 간다.
- [agent-loop.ts L580~618](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/packages/agent-core/src/agent-loop.ts#L580), [L750~768](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/packages/agent-core/src/agent-loop.ts#L750): 문서와 맞는 실제 steering checkpoint와 skipped 결과 구성이 있다. `executionStarted`, sequential 여부, abort 여부와 drain 결과에 따라 경로가 달라진다.

**품질 해석:** 같은 실행에서 근거와 잠정 판단을 활용할 수 있는 대신, 입력이 다음 의미 판단에 들어가는 경계를 가진다. 이는 VIA 첫 응답 지연의 구조적 후보이지 실제 지연 수치는 아니다. 순수 tool wait를 비동기화하면 해당 대기는 줄일 수 있으므로 A를 동기 I/O로 약화하지 않는다.

### 입력 custody와 fallback

- [agent-runner-steer-adoption.ts L90~178](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/auto-reply/reply/agent-runner-steer-adoption.ts#L90): steer 후보를 먼저 queue에 park하고 lifecycle generation과 authority를 확인한다. fallback은 입력을 후속 실행으로 보존한다.
- [같은 파일 L189~260](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/auto-reply/reply/agent-runner-steer-adoption.ts#L189): 현재 active operation과 injection target, 권한과 모델 fallback의 현재성을 재확인한다. 준비 도중 backend가 바뀌었다고 다른 실행에 입력을 자동 주입하지 않는다.
- [queue.md의 Input durability](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/queue.md#scope-and-guarantees): chat.send 입력 저장과 queue의 process memory 수명을 구별한다. Gateway가 종료되기 전에 transcript에 들어가지 않은 입력의 보존이 queue 자동 replay를 뜻하지 않으며 명시적 재전송이 필요할 수 있다.

**VIA에 추가할 것:** 입력 저장, 의미 반영, 관계 채택, 실제 표시를 분리한다. 입력 queue의 자동 overflow summary 정책을 사용자 정정이나 취소의 무손실 전달로 간주하지 않는다.

### 완료와 오래된 writer

- [agent-steering-queue.ts L143~178](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/agents/agent-steering-queue.ts#L143): child 결과들을 prompt 크기 기준으로 묶으며 source의 현재성을 확인한다. oversized 첫 결과를 전달하는 soft cap의 예외가 있어 hard memory cap으로 인용하지 않는다.
- [같은 파일 L182~267](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/agents/agent-steering-queue.ts#L182): 결과 I/O 뒤 run owner와 source store를 재확인하고 delivery 상태에 lease를 둔다. 동일 lease와 현재 run을 확인한 후 settlement한다.
- [session-transcript-turn-state.ts L42~100](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/config/sessions/session-transcript-turn-state.ts#L42): session ID, lifecycle revision, active writer run과 recovery 기대 상태를 비교한다.

**한계:** writer fencing은 기술적인 쓰기 권한 검사다. 사용자가 보고서의 표만 정정한 뒤 어느 문단 결과를 유지할지는 판정하지 않는다. child 결과가 model input에 주입됐다는 사실을 VIA가 사용자에게 표시하거나 결과로 채택한 사실로 옮길 수 없다.

## 2. OpenHands: 수신, 실행, 재실행과 실제 정리

### LocalConversation

- [local_conversation.py L1823~1885](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py#L1823): `send_message`는 사용자 MessageEvent를 기록한다. ACP 초기화는 가능한 경우 run까지 미뤄 수신 경로와 분리한다.
- [L1892~1915](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py#L1892): await LLM I/O 동안 state lock을 풀어 send와 snapshot을 가능하게 한다. 수신이 가능하다는 사실과 두 의미 실행이 병렬이라는 주장은 다르다.
- [L2735~2790](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py#L2735): pause는 다음 step 경계, interrupt는 tracked async arun을 cancel하고 token으로 도구에 알린다. async task가 없는 sync run에서는 pause로 fallback한다.

### Agent Server

- [event_service.py L842~918](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-agent-server/openhands/agent_server/event_service.py#L842): 입력을 저장하고 run intent를 구별한다. ACP prompt supersede가 필요할 때 permit을 pin한 뒤 내부 interrupt와 재실행을 시도한다. explicit interrupt generation이 바뀌면 이전 재실행 의도를 적용하지 않는다.
- [L920~952](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-agent-server/openhands/agent_server/event_service.py#L920): **ACPAgent에만** supersede를 적용한다. 현재 prompt가 이미 최신 사용자 메시지를 포함했다면 다시 중단하지 않는다.
- [L1422~1508](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-agent-server/openhands/agent_server/event_service.py#L1422): run lock, 현재 task와 capacity permit을 확인한다. `arun`이 없는 경우 sync thread로 실행하며 asyncio waiter 취소가 sync thread의 종료를 뜻하지 않으므로 실제 worker 완료 전 slot을 반환하지 않는다.
- [L1563~1644](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-agent-server/openhands/agent_server/event_service.py#L1563): pending event publish 뒤 최종 상태를 내보낸다. wrap-up 중 들어와 stranded된 입력에 명시적 rerun intent가 있으면 상태와 generation을 확인해 다시 실행한다. 사용자 pause/interrupt가 이전 자동 재시작보다 우선한다.
- [L1957~1992](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-agent-server/openhands/agent_server/event_service.py#L1957): interrupt cleanup은 shield된 대기이며 timeout이 나도 여전히 실행 중인 `_run_task`를 조기에 지우지 않는다.

**VIA 적용:** ‘새 입력 때문에 실행을 바꾼다’에는 저장, 중단, 자원 보존, cleanup, 재실행이라는 실제 비용이 있다. run을 background task로 만들었다는 사실만으로 독립 요청의 지속 실행과 복구를 주장하지 않는다. 중단 API는 이미 수행한 외부 Action을 되돌리는 계약이 아니다.

## 3. Deep Agents: 별도 실행과 제한된 side 질문

### AsyncSubAgentMiddleware

- [async_subagents.py L430~511](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/async_subagents.py#L430): 새 thread와 run을 생성하고 launch 결과와 `async_tasks` 기록을 반환한다. 완료를 기다리는 일반 task tool과 다르다. task ID는 thread ID이며 현재 run ID와 분리돼 있다.
- [L524~635](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/async_subagents.py#L524): run success만 보지 않고 thread interrupted를 waiting으로 해석한다. 다른 client의 재개가 있으면 최신 run을 조회할 수 있다. paused task를 취소한 뒤 이전 pause가 상태를 되살리지 않도록 보완한다.
- [L710~813](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/async_subagents.py#L710): `update_async_task`는 같은 thread에서 `multitask_strategy="interrupt"`인 새 run을 만든다. task identity는 유지하고 run identity와 상태를 갱신한다.
- [L816~885](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/async_subagents.py#L816): waiting이면 active run이 없으므로 로컬 task 취소 표시를 사용하고, 실행 중이면 run cancel을 호출한다.

**한계:** `_WAITING_NOTE`는 사용자가 child에서 직접 답하며 parent가 승인하지 말라고 설명한다. VIA의 원 대화에서 답을 연결하는 방식은 추가 설계다. check API의 상태 이름을 외부 업무의 물리적 종료 증명으로 삼지 않는다. ID tracking의 persistence와 remote run의 내구성은 checkpointer와 서버 계약에 의존한다. 해당 실행 환경을 이번 조사에서 검증하지 않았다. 일반 Python async await와 durable remote run을 혼동하지 않는다.

### BtwOperation

- [btw.py L474~532](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/code/deepagents_code/btw.py#L474): live snapshot이나 checkpoint에서 모델과 지시를 복원하되 tools를 제거한다.
- [L534~613](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/code/deepagents_code/btw.py#L534): read-only state와 side 대화 history로 별도 답을 생성한다. tools와 checkpoint writes가 없다. context budget을 넘으면 오래된 내용을 제거하거나 overflow를 보고한다.

**VIA 적용:** 긴 의미 실행을 유지한 채 간단한 별도 질문을 처리하는 C의 직접 근거다. VIA에서는 실제 표시한 side 응답을 Conversation 원본에 저장해야 이후 지칭을 지원한다. 자동으로 ‘독립 질문’을 분류하는 기능, source read와 정정 처리는 이 코드의 확인 범위를 넘는다.

## 4. Magentic-One: 중앙 의미 재판단과 순차 참여

- [orchestrator L135~215](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L135): 초기 facts와 plan을 만들고, 참가자 응답을 message thread에 반영한 뒤 다음 조정 단계로 간다.
- [L225~245](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L225): message thread와 task, facts, plan, 회차 등을 save/load한다. save 메서드만으로 실행 중인 도구의 exactly-once 복구를 주장할 수 없다.
- [L300~405](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L300): 모델로 progress ledger를 만들고 만족 여부와 loop/stall을 판단한다. 정체 시 facts와 plan을 갱신하고 다시 시작한다.
- [L408~439](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L408): 지시를 broadcast하고 선택한 한 참가자에게 publish를 요청한다.

**한계와 제외:** 여러 agent, topic과 async event를 쓴다고 사용자 입력을 독립적으로 처리하는 구조로 분류하지 않는다. VIA가 domain facts와 plan을 생성하는 것은 외부 업무 책임을 침범할 수 있다. 참고 대상은 공유 의미 문맥에서 결과를 확인하고 다음 처리를 정하는 의존 관계다. 이 코드에서 일반적인 중간 사용자 질문, VIA의 정정 효력과 실제 청취 focus는 확인되지 않았다.

## 5. CrewAI: async 실행과 대화 turn의 수명 구별

- [crew.py L1135~1187](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/crew.py#L1135): `kickoff_async`의 non-stream 경로는 `asyncio.to_thread(self.kickoff, ...)`를 await한다. 호출 형식의 async를 독립적인 foreground 대화 구조로 해석하지 않는다.
- [L1354~1425](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/crew.py#L1354): native async 실행은 async Task들을 생성하고 일반 Task 전에 모아 기다린 뒤 결과를 context로 만든다. 실행 병렬성과 join 의존을 동시에 보여준다.
- [conversational_mixin.py L318~389](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/flow/conversational_mixin.py#L318): 현재 turn의 route를 고르며 stale last_intent를 무조건 재사용하지 않는다. 기본 converse는 canonical conversation history로 답한다.
- [L429~502](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/flow/conversational_mixin.py#L429): 사용자 메시지를 pending으로 보존하고 persistence 복원 뒤 적용한다. 새 turn은 이전 graph 실행 추적을 지우고 kickoff하며 public 결과를 다시 기록한다.
- [L1145~1182](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/flow/conversational_mixin.py#L1145): completed methods, outputs, pending events와 turn classified intent 등 일시 실행 추적을 초기화한다. 지속 대화 원본과 일시 실행 상태가 같은 수명이 아님을 확인한다.

**한계:** 같은 Flow 인스턴스에 여러 동시 handle_turn이 안전하게 들어온다는 보장을 확인한 것은 아니다. persistence 복원과 실행 초기화는 입력 중단 후 arbitrary 외부 업무의 자동 복구와 다르다. 여기서는 새 Architecture 대안을 만들 만큼 다른 사용자 개입 메커니즘을 추가로 확보하지 못했으므로 보조 근거로 남긴다.

## 조사에서 얻은 설계 판단

1. 같은 run에 입력을 넣는 경로, 독립 thread/run, 기존 실행을 바꾸지 않는 제한 side 답변은 서로 다른 실제 메커니즘이다.
2. steer, interrupt와 followup은 위 구조들 내부에서도 선택할 수 있는 정책이다. 네 queue mode를 네 주요 DP 대안으로 세는 것은 부적절하다.
3. 별도 실행의 효용을 주장하려면 질문 수신, relation 판단, 실제 worker 진행, 완료의 현재성 확인과 게시를 모두 연결해야 한다. `async` 키워드나 event bus만으로 이 계약이 생기지 않는다.
4. 모든 source는 자체 workload와 권한 범위를 가진다. VIA의 요청 관계와 부분 정정, 실제 표시 및 음성 전달, 외부 Task 취소와 완료를 이 소스들이 이미 해결했다고 주장하지 않는다.
5. 직접 관련 메커니즘 세 계열과 저비용 혼합 근거를 확보했다. 이번 범위에서 부족한 것은 또 다른 queue 제품 목록보다 VIA의 자동 관계 판단과 같은 예산 아래 정상 사용 손익이다. 이는 이후 후보 상세화와 승인된 평가에서 확인할 사항이다.
