# Agent harness 구현에서 확인한 VIA 구조 탐색 근거

2026-10-15 조사. OpenClaw, OpenHands, LangChain Deep Agents, AutoGen Magentic-One, CrewAI의 공식 저장소에서 확인한 구현 경로를 기록한다. 이 자료는 [VIA 설계 질문과 QA 검토](../architecture/12-decisions/decision-packages/harness-grounded-dp-review-20261015.md)의 근거다.

각 프로젝트는 여러 실행 방식을 지원한다. 아래 사실은 명시한 경로에 적용한다. 외부 구현의 존재는 VIA에서 같은 품질 차이가 발생한다는 실험 근거가 아니다.

## 조사 판본

| 프로젝트 | 저장소 | 고정 commit |
| --- | --- | --- |
| OpenClaw | `openclaw/openclaw` | `e7a155a8ae860d71b1ab5eed45fa4af338011db9` |
| OpenHands Agent Canvas | `OpenHands/OpenHands` | `5958302c1c5dbdc441cbb7d85dc650ffc4d606bd` |
| OpenHands SDK와 Agent Server | `OpenHands/software-agent-sdk` | `fd387b58dc7b707b73c159845c0396a17000be24` |
| LangChain Deep Agents | `langchain-ai/deepagents` | `7764f1f8d21a594f0cbd31d273a7cad36597d6ba` |
| AutoGen Magentic-One | `microsoft/autogen` | `027ecf0a379bcc1d09956d46d12d44a3ad9cee14` |
| CrewAI | `crewAIInc/crewAI` | `3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8` |

GitHub 검색은 경로 발견에 사용했고 본문 확인은 위 commit으로 고정했다. OpenHands의 두 저장소는 각각 확인한 snapshot이며, Canvas가 이 SDK commit을 배포 의존성으로 사용한다는 확인은 포함하지 않는다. AutoGen은 2026-04-06 commit의 구현을 분석했다. 유지보수 상태와 VIA에 참고할 구조의 존재를 구분한다.

## OpenClaw

### 세션 실행과 하위 작업

- [Queue 문서](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/queue.md#L18)는 같은 세션의 agent run을 직렬화한다. 입력을 저장하고 수신 확인하는 일과 실행을 시작하는 일을 구분한다. `steer`는 실행 중인 도구가 끝난 뒤 다음 모델 경계에서 입력을 반영할 수 있으며 실행 중 도구의 물리적 중단을 보장하지 않는다.
- [Subagents 문서](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/tools/subagents.md#L10)는 하위 작업에 별도 세션과 문맥을 두고 완료를 요청자에게 연결한다. 부모가 외부 업무가 끝날 때까지 모든 입력을 막아야 한다는 구조가 아니다.
- [부모 검토 후 완료 전달](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/tools/subagents/announce.md#L73)은 `completionTarget: "parent"`를 선택한 경로다. 부모의 비공개 검토 turn에서 결과를 받고 원 대화로 돌아간다. 이 선택을 모든 하위 작업의 기본 경로로 일반화하지 않는다.
- [세션과 writer 검사](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/config/sessions/session-transcript-turn-state.ts#L42)는 session ID, lifecycle revision, active writer run과 예상 상태를 대조한다. 이 검사가 보고서의 어느 문단이 정정 후에도 의미상 유효한지 판정하지는 않는다.
- [하위 작업 결과 주입](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/agents/agent-steering-queue.ts#L182)은 완료 결과 batch와 전달 lease를 사용한다. 여기서 prompt에 전달했다는 사실을 사용자에게 표시했거나 VIA Task 결과로 채택했다는 사실로 옮길 수 없다.

**VIA 적용:** 대화 실행, 독립 작업, 결과 전달을 서로 다른 상태로 관리하는 근거다. 자연어의 Task 연결, 부분 정정의 의미 범위, 실제 표시 및 청취 기록은 VIA가 추가해야 한다.

### 선행 기억 생산과 선택적 회상

- [Memory architecture](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/memory-architecture.md#L136)는 원본 기록과 파생 기억, 검색과 배경 consolidation을 구분한다.
- [기억의 두 회상 경로](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/memory-architecture.md#L222)는 미리 만든 metadata와 trigger에 따른 제한된 주입, 필요할 때의 별도 deep recall을 설명한다. 첫 경로에 질의 시 생성 모델 호출이 없다는 설명을 검색, embedding, I/O의 비용이나 지연이 0이라는 뜻으로 쓰지 않는다.
- [Active memory](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/active-memory/how-it-works.md#L9)는 약한 검색 결과와 회상 의도가 함께 있을 때 기억 전용 subagent를 실행한다. 이 경로는 주 응답 전에 기다리는 경로이며, 사용자에게 보이는 모든 세션과 내부 작업에 무조건 적용되지 않는다.
- [Dreaming](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/dreaming.md#L9)은 배경 consolidation과 provenance 및 승격 조건을 둔다. [memory-core hook](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/memory-core/index.ts#L366)에서도 prompt 구성 전 처리 경계를 확인했다.

**VIA 적용:** 요청 전에 의미 있는 기억을 생산하고 조회 시 활용하는 실제 설계 근거다. VIA의 실제 전달 구간, 원본 범위, 결과 버전 및 삭제 계약이 이 구현에 완성되어 있다는 뜻은 아니다.

## OpenHands

### 현재 제품 경계와 실행 상태

- [Agent Canvas architecture](https://github.com/OpenHands/OpenHands/blob/5958302c1c5dbdc441cbb7d85dc650ffc4d606bd/docs/architecture.md#L1)는 React와 TypeScript UI가 하나 이상의 Agent Server에 연결되는 구조다. UI, Agent 실행 및 workspace 격리, 선택적 automation backend의 책임을 구별한다.
- [LocalConversation의 입력 처리](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py#L1823)는 메시지 기록과 run을 구분한다. [Agent step](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/agent/agent.py#L694)은 하나의 conversation 안에서 모델 판단과 도구 동작을 이어 가는 경로다.
- [Agent Server의 메시지 처리](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-agent-server/openhands/agent_server/event_service.py#L842)는 입력을 먼저 저장하고 run permit, 현재 run 종료와 재실행, 명시적인 interrupt generation을 고려한다. 모든 Agent 종류가 같은 중단 방식을 갖는다는 의미는 아니다.

**1차 조사 정정:** 이전 EventStream과 Controller 중심 설명을 현재 `OpenHands/OpenHands`의 구조로 그대로 사용하면 안 된다. 현재 Canvas와 SDK/Server를 나누어 읽는다.

### 직접 도구 사용과 별도 문맥의 위임

- [Task tool](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/task/impl.py#L1)은 subagent 결과를 기다리는 호출이다. [도구 설명](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/task/definition.py#L120)은 간단한 조회에 위임 비용을 지불하지 않는 사용 방식을 설명한다.
- [DelegateExecutor](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/delegate/impl.py#L133)는 이름으로 연결하는 child conversation을 만들고 보유한다. 부모 persistence 설정에 따라 child 기록도 보존한다. 그러나 같은 파일의 `_delegate_tasks`는 여러 결과를 기다리는 blocking 경로다. child context가 있다는 이유만으로 부모의 모든 응답이 병행된다고 해석하지 않는다.
- [BrowserObservation](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/browser_use/definition.py#L61)은 도구 결과의 텍스트와 선택적 이미지를 모델 입력으로 구성한다. 직접 모델이 관측 근거를 읽는 구조의 실제 예다.

**VIA 적용:** 같은 요청 이해 작업을 주 대화에서 수행하거나 별도 작업 문맥으로 위임하는 두 경로의 근거다. OpenHands의 코드 변경과 브라우저 조작을 VIA 내부 업무로 가져오지는 않는다. VIA에 옮기는 도구는 허용된 bounded read와 상호작용 처리로 제한한다.

### 이력과 모델에 보여 주는 view

[RollingCondenser](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/context/condenser/base.py#L107)는 기록된 events로 모델 입력 view를 만든다. [LLMSummarizingCondenser](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/context/condenser/llm_summarizing_condenser.py#L54)는 그 view를 요약하는 실행을 제공한다. 대화 원본과 현재 모델 입력을 구별하는 근거이며, VIA의 장기 지칭 검색과 전달 구간 추적까지 증명하는 근거는 아니다.

## LangChain Deep Agents

### 격리와 문맥 상속

[SubAgentMiddleware](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py#L747)는 child의 structured response 또는 마지막 비어 있지 않은 응답을 ToolMessage로 부모에게 돌려준다. 설정과 제외 규칙에 따라 다른 state도 전달될 수 있으므로 모든 위임이 문자열 한 개만 반환한다고 설명하지 않는다.

같은 파일의 [isolated와 fork 입력](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py#L804)은 위임 설명에서 시작하는 문맥과 부모 대화를 상속하는 문맥을 구분한다. 이는 prompt 크기, 암묵적 맥락의 보존, 결과 반환 뒤 재판단 비용을 바꾸는 실제 실행 선택이다. isolated도 허용된 state와 backend를 공유할 수 있으며 OS process 격리와 동일하지 않다.

### 독립 실행과 별도 질문

- [AsyncSubAgentMiddleware](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/async_subagents.py#L430)는 remote thread와 run을 생성하고 부모의 `async_tasks`에 연결한다. thread identity와 현재 run identity를 구분하고 check, update, cancel 경로를 제공한다.
- 해당 middleware의 [waiting 상태](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/async_subagents.py#L132)는 human interrupt를 child thread에서 답하는 경로로 설명한다. VIA에서는 사용자가 원 대화에서 답할 수 있도록 별도의 질문 연결 계약이 필요하다.
- 일반 `task()`의 Python async 구현도 child 완료를 await한다. `async` 함수라는 사실과 독립적인 durable 실행을 구별한다. state에 ID가 남는 것과 crash 후 실행이 복구되는 것도 구분하며 후자는 checkpointer와 backend 조건을 확인해야 한다.
- [BtwOperation](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/code/deepagents_code/btw.py#L534)은 주 대화 snapshot으로 별도 질문에 답한다. 도구 사용과 원 대화 checkpoint 변경을 제한한다. 독립 질문 경로의 실제 근거지만 복잡한 정정과 업무 실행을 그대로 처리하는 완성 설계는 아니다.

### 이력 압축과 원본 재조회

[SummarizationMiddleware](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/summarization.py#L1)는 오래된 내용을 backend에 보관하고 현재 문맥을 요약하며 원본 경로를 남긴다. [공식 설명](https://www.langchain.com/blog/context-management-for-deepagents)은 도구 입출력 offloading과 요약을 구분한다. 요약에서 빠진 상세 내용을 필요할 때 다시 읽는 구조의 근거다. 이 기능만으로 모든 원본 지칭을 정확히 회복한다고 주장하지 않는다.

## AutoGen Magentic-One

[MagenticOneOrchestrator](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L300)는 공유 message thread와 task ledger를 사용해 진행을 검토하고 다음 담당자를 고른다. [다음 지시의 전달](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L408)은 그룹에 지시를 알리고 선정한 한 참가자에게 응답을 요청한다. 이벤트로 연결됐다는 이유만으로 분산된 의미 권한이나 모든 참가자의 병렬 실행을 가정하지 않는다.

[공식 구조 설명](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/magentic-one.html)은 task ledger와 progress ledger의 두 loop를 설명한다. 조사와 문서 제작 같은 domain plan은 VIA에서 Downstream Agent 책임에 해당한다. VIA가 참고할 부분은 조정 문맥, 전문 처리자와의 교환, 재판단 경로다.

[MultimodalWebSurfer](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-ext/src/autogen_ext/agents/web_surfer/_multimodal_web_surfer.py#L490)는 interactive rect, 화면 이미지, focus와 대상 목록을 결합해 모델에 보낸다. vision 지원 모델에는 표식을 넣은 이미지와 대상 텍스트를 함께 제공한다. native object 목록과 원본 시각 근거를 함께 쓰는 실제 예다. 이 구현은 브라우저 조작용이며 VIA의 발화 시간, click-free pointing, 원형 집합과 부분 정정의 처리 근거로 그대로 일반화할 수 없다.

## CrewAI

### Manager와 명시적 작업 문맥

[Crew의 hierarchical 경로](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/crew.py#L1522)는 manager를 만든 뒤 Task 목록을 실행한다. [위임 도구](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/tools/agent_tools/base_agent_tools.py#L110)는 선택한 agent에게 Task와 context를 전달하고 결과를 받는다. Magentic-One의 매 단계 next-speaker 선정과 같은 구현이라고 설명하지 않는다.

[Task context 구성](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/crew.py#L1879)은 지정된 이전 Task 결과 또는 기본 이전 출력들을 조합한다. 이 계약은 작업자에게 전달하는 문맥을 명시하는 근거다. CrewAI 전체가 고정 순차 처리만 지원하거나 persistence가 없다는 의미는 아니다.

### 기억 생산과 조회 비용

- [BaseAgentExecutor의 기억 저장](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/agents/agent_builder/base_agent_executor.py#L31)은 memory가 설정된 경우 Task 결과에서 모델로 기억을 추출하고 `remember_many`에 전달한다. [공식 memory 설명](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/docs/edge/en/concepts/memory.mdx#L74)은 결과 저장과 다음 Task 전 회상 경로를 설명한다.
- [Memory의 batch 저장](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L528)은 배경 encode를 지원하지만 [recall](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L686)은 pending write를 먼저 drain한다. 따라서 background 저장이 조회 지연을 항상 없앤다고 주장할 수 없다.
- 같은 recall에는 shallow vector search와 모델 기반 deep recall이 있다. B를 항상 무료 조회로, A만 여러 모델 호출을 하는 구조로 비교하면 이 실제 비용이 빠진다.

**VIA 적용:** 쓰기 시 의미 가공과 조회 시 탐색을 나누는 실용적 근거다. 추출된 사실만으로 VIA의 원본 범위, 실제 전달 내용, 정정 전후 버전을 대체할 수는 없다. 해당 식별 정보와 원본 재조회 경로를 함께 설계해야 한다.

## 근거에서 도출할 수 있는 범위

| 근거 | 확인한 것 | VIA 설계에서 추가할 것 |
| --- | --- | --- |
| 세션 lane과 child thread | 실행과 문맥을 나누는 실제 구현 | 대화와 업무 연결, 여러 업무에 걸친 정정의 권한 |
| isolated와 fork | 위임 시 정보량과 문맥 경계가 달라짐 | 지칭 근거 누락 검출과 원본 조회, 동일 예산의 비교 |
| event log와 condensed view | 원본 보존과 모델 입력 압축을 구분 가능 | 실제 전달 구간과 자료 버전을 이용한 지칭 |
| 결과 기반 memory와 background consolidation | 미래 조회 전에 의미 가공을 수행 | 원본 및 허용 범위 추적, 갱신과 삭제, 미사용 생산 비용 |
| writer와 run generation | 오래된 실행의 기술적 쓰기 차단 | 수정 후에도 남는 결과의 의미적 재사용 여부 |
| native target와 이미지 결합 | 구조화된 대상과 원본 시각 근거의 병용 | 발화와 사용자 행동의 시간 결합, 누락된 대상의 확장 |

선정한 경로는 소스와 문서로 확인했다. 이 조사에서는 해당 harness를 실행하거나 VIA 품질을 측정하지 않았다.
