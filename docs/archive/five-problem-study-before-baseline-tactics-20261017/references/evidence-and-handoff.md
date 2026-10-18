# 근거 공급과 Agent 간 업무 인계의 구현 근거

2026-10-16 / 문제 1과4의 심화 조사 / 공식 구현의 고정 판본

[공동 조사 기준](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/00-research-contract.md) / [문제 1](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/01-evidence-supply.md) / [문제 4](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/04-agent-independent-handoff.md)

## 1. 조사 범위와 읽는 방법

이 기록은 API 이름만 수집한 목록이 아니다. 문맥을 만드는 시점, 저장하는 원본과 파생물, 위임 입력과 반환값, 후속 실행의 재사용, 잘못된 기억의 갱신 및 삭제 경로를 확인했다. 실행하거나 품질을 측정하지 않았다. 아래 판본 이후의 변경은 포함하지 않는다.

| 저장소 | 고정 commit | 이번 조사에서 사용한 범위 |
| --- | --- | --- |
| `openclaw/openclaw` | `e7a155a8ae860d71b1ab5eed45fa4af338011db9` | 기존 active recall 근거 재사용. consolidation 구현, provenance, 제한된 transcript reader, ACP 재사용 경로 추가 확인 |
| `OpenHands/software-agent-sdk` | `fd387b58dc7b707b73c159845c0396a17000be24` | 기존 rolling view 근거 재사용. 이름별 child conversation과 반복 위임 구현 추가 확인 |
| `langchain-ai/deepagents` | `7764f1f8d21a594f0cbd31d273a7cad36597d6ba` | 기존 offload / 범위 읽기 근거 재사용. isolated / fork 입력과 반환, StoreBackend, CompositeBackend, MemoryMiddleware 추가 확인 |
| `crewAIInc/crewAI` | `3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8` | 기존 recall 근거 재사용. 명시 Task 결과 조합, pending write, 갱신 및 삭제 구현 추가 확인 |
| `microsoft/autogen` | `027ecf0a379bcc1d09956d46d12d44a3ad9cee14` | 이전에 확인한 Magentic-One ledger / message thread와 화면 관측 경로를 적용 범위 검토에 사용 |

기존에 직접 확인한 범위는 [1차 원장](../agent-harness-20261015.md)과 [첫 근거 공급 조사](../qa-first-evidence-supply-20261016.md)에 남아 있다. GitHub 검색의 이동하는 HEAD는 파일 경로 발견에만 썼다. 이 문서의 새로운 구현 주장은 표의 commit에서 파일을 읽었다. OpenHands Canvas와 SDK의 배포 버전 대응 관계를 확인한 것은 아니다.

## 2. 현재 요청에 근거를 공급하는 메커니즘

### R01. 원본 사건과 대화별 모델 입력 view의 분리

OpenHands [RollingCondenser의 view 구성](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/context/condenser/base.py#L107)은 기록한 사건에서 모델 입력 view를 만들고 condensation을 사건으로 남긴다. [LLMSummarizingCondenser](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/context/condenser/llm_summarizing_condenser.py#L54)는 사건 수와 token 등의 조건에 따라 별도 요약 호출을 수행한다. [Agent.step](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/agent/agent.py#L694)은 준비된 view를 통해 모델 입력을 구성한다.

- 정상 경로: 같은 conversation에서 최근 사건과 축약된 과거를 이어 사용한다.
- 비용과 실패 경계: condensation은 모델 호출이며 실행 경로에서 발생하면 대기다. 원본 보존만으로 축약에서 빠진 의미가 자동 복구되지는 않는다.
- VIA 적용: 현재 해석기의 대화 문맥 유지에 참고한다. 실제 음성 전달 구간, 화면 시점과 사용자 정정의 출처 관계는 VIA가 추가해야 한다.

### R02. 큰 자료를 원본 저장소에 두고 필요한 범위 재조회

Deep Agents의 [read_file 실행](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/filesystem.py#L2241)은 허용된 backend에서 offset과 limit 범위를 읽는다. [SummarizationMiddleware](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/summarization.py#L1526)는 이력 offload와 요약 및 제한된 재시도 경로를 갖는다.

- 정상 경로: 큰 결과와 이력을 모든 모델 호출에 반복 입력하지 않고 참조를 통해 다시 읽는다.
- 강한 보완: 제품에는 요약과 retrieval이 함께 있다. 이 근거를 ‘매 요청마다 빈 문맥으로 시작하는 제품’이라고 해석하면 안 된다.
- VIA 적용: 요청별 제한된 근거 구성을 지원하는 메커니즘이다. 검색어 선택이 잘못되면 원본이 있어도 못 찾을 수 있고, 원본 재조회가 필요한 경우 모델 → 도구 → 모델 의존이 생긴다.

### R03. 먼저 생산한 기억과 조건부 별도 recall

OpenClaw [consolidation 후보 필터](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/memory-core/src/dreaming-consolidation-candidates.ts#L4)는 provenance와 session 종류로 승격 후보를 제한한다. [consolidateMemory](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/memory-core/src/dreaming-consolidation.ts#L387)는 후보를 묶어 모델에 계획을 요청하고 검증한다. [계획 적용](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/memory-core/src/dreaming-consolidation.ts#L276)은 기존 항목 손실, 중복, lineage, 문자 예산을 검사한다. 실패하면 null을 반환하고 append-only fallback을 쓰도록 보고한다. 이 함수만으로 모든 파일 쓰기의 원자성을 증명하지는 않는다.

[Active Memory](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/active-memory/index.ts#L517)는 일부 입력에서 query를 구성하고 별도 recall을 기다린 뒤 주 prompt에 넣는다. 모든 turn에서 실행되는 helper가 아니다.

- 정상 이익: 이전 사건에서 재사용할 정보를 가공해 놓고 여러 요청에서 회상할 수 있다.
- 비용: 아직 쓰지 않을 기억 생산, 원본과 파생물 연결, 재호출, 수정 및 삭제 전파가 추가된다.
- VIA 적용: 대화 view와 별개로 유지하는 출처 있는 업무 / 전달 기억에 참고한다. 일반 대화의 임시 지시를 허가 없이 User Memory로 승격하는 근거로 사용할 수 없다.

### R04. 기억 삭제는 원본과 파생물마다 다른 범위

OpenClaw [provenance 설명](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/memory-provenance.md#L68)은 chunk provenance, entry origins, curated-write 기록을 구분한다. merge / supersede 후에도 부모의 출처를 유지하며, 혼합 출처인 항목을 제거해야 하는 경우와 lineage가 없는 수동 편집의 한계를 설명한다. [삭제 경계](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/memory-provenance.md#L17)는 원본 transcript나 외부 복사본까지 모두 지우는 기능이 아님을 명시한다.

이 문서와 consolidation의 lineage 검사는 확인했다. 실제 purge 구현 전체와 모든 writer의 동기화는 이번에 추적하지 않았다. VIA는 삭제된 자료가 뒤늦은 기억 생산으로 다시 게시되지 않도록 source revision / admission generation을 검사해야 하며, 그 계약은 VIA 설계 추가분이다.

### R05. 명시 기억 파일의 선행 주입도 다른 실용적 선택

Deep Agents [MemoryMiddleware.before_agent](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/memory.py#L279)는 구성한 파일을 내려받아 state에 저장하고 이미 있으면 다시 읽지 않는다. 없는 파일은 건너뛰고 다른 읽기 오류는 실패로 보고한다. [modify_request](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/memory.py#L347)는 이 내용을 모델 system message에 주입하며 선택적 prompt cache 처리를 한다.

이는 안정된 명시 지침을 준비해 두는 근거다. VIA에서 Agent 결과를 이 파일에 복사하면 자동으로 신뢰 가능한 사용자 지시가 되는 것은 아니다. 외부 업무 결과는 출처가 표시된 자료로 취급한다. 같은 state에 남은 내용의 수정 / 철회에는 명시적 무효화가 필요하다는 점도 드러난다.

## 3. Agent가 바뀌는 업무의 문맥 전달

### R06. 계속 재사용할 수 있는 child conversation

OpenHands [DelegateExecutor](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/delegate/impl.py#L45)는 이름에서 LocalConversation으로 가는 map을 보유한다. [생성 경로](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/delegate/impl.py#L207)는 부모 persistence 설정에 따라 자식 기록을 저장한다. [반복 위임 경로](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/delegate/impl.py#L287)는 해당 conversation에 새 task를 send_message하고 끝날 때까지 기다려 마지막 응답을 모은다. 여러 worker thread를 실행하지만 join하는 blocking 호출이다. 누적 비용을 반복 위임마다 더하지 않고 교체하는 코드도 있다.

- 실제 차이: 매번 새로운 작업자 문맥을 만들 필요 없이 같은 작업자의 대화를 이어간다.
- 제한: 이 map과 child persistence만으로 외부 다른 제품이 해당 session을 읽거나 실행 상태를 옮겨 받을 수 있음을 보장하지 않는다. 부모 UI의 계속 응답 가능성도 이 함수만으로 주장할 수 없다.
- VIA 적용: 같은 Agent에 후속 수정 시 native session 재사용을 허용하는 근거다. 모든 portable 인계안에 조합할 수 있다.

### R07. isolated 인계와 부모 문맥 fork

Deep Agents [상태 준비](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py#L804)는 isolated의 메시지를 위임 설명 하나로 만들고, fork는 부모의 유효 메시지와 허용 상태를 가져온다. fork의 제외 항목은 [이전 session과 structured result 등](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py#L52)이다. 선언형과 opaque compiled child의 전달 범위도 구별한다. fork는 이 판본에서 experimental이다.

[결과 반환](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py#L747)은 structured response 또는 마지막 비어 있지 않은 AI 응답을 ToolMessage로 돌려주고, 제외되지 않은 변경 state도 전달할 수 있다.

- 실용적 선택: 좁게 구성한 설명으로 시작하거나 풍부한 부모 문맥을 상속한다. 단순한 prompt 문장 변경에 그치지 않고 메시지와 private state 전달 범위가 바뀐다.
- VIA 경계: 같은 framework의 state fork는 이종 Agent 간의 보편적인 session migration이 아니다. VIA의 서로 다른 Agent에는 전달 가능한 공개 기록과 자료로 다시 구성해야 한다. 광범위 history replay는 구현 가능한 보완이지만 개인정보 범위와 입력 예산을 적용한다.

### R08. source / artifact를 공유하는 저장 경계

Deep Agents [StoreBackend](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/backends/store.py#L90)는 호출자가 제공한 namespace에서 thread를 넘어 파일을 읽고 쓴다. 실제 영속성은 공급한 BaseStore에 달려 있다. [CompositeBackend](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/backends/composite.py#L1)는 경로별로 다른 backend를 연결해 임시 상태와 지속 파일을 구분한다. 읽기 / 검색에서 오류와 잘린 결과를 보존하는 경로도 확인했다.

이와 isolated child 및 범위 읽기를 결합하면 결과를 모두 부모 메시지에 복사하지 않고 자료의 주소를 주고 작업자가 필요한 원본을 읽을 수 있다. 이는 실제 framework 내의 공유 저장 메커니즘이다. VIA에서 이종 외부 Agent가 같은 API 또는 connector를 사용할 수 있는지, read lease와 삭제를 지키는지는 별도 integration dependency다. 모든 상용 음성 Agent가 이 기능을 지원한다고 주장하지 않는다.

### R09. 명시 Task 출력 조합과 추출 기억

CrewAI [_get_context](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/crew.py#L1879)는 Task 설정에 따라 빈 문맥, 이전 출력, 지정한 Task 출력의 조합을 만든다. 사용자 수정과 승인까지 자동 정규화하는 코드라는 뜻은 아니다.

[Memory.remember_many](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L528)는 batch 기억 저장을 제공한다. [recall](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L686)은 pending write를 먼저 기다린다. [drain_writes](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L355)는 저장 실패를 이벤트로 알리고 원래 업무를 실패시키지 않는 경계를 설명한다. 따라서 업무 성공과 기억 생산 성공을 구별해야 한다.

[forget / update](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L823)는 scope / category / metadata / ID별 삭제 및 내용 수정 시 재embedding을 지원한다. [reset](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L1023)은 pending write를 기다린다. 이 API들이 VIA의 모든 파생 자료와 외부 Agent 복사본까지 삭제한다는 근거는 아니다.

### R10. 외부 harness의 session을 재사용하고 출력 경계를 번역

OpenClaw [ACP 전달 문서](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/tools/acp-agents/delivery.md#L41)는 외부 harness에 prompt와 지원되는 attachment를 보내고 내부 completion envelope는 외부용 plain prompt로 변환하는 경계를 설명한다. [runtime 초기화 코드](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/agents/subagents/spawn/acp-spawn-runtime.ts#L142)는 agent, backend, resumeSessionId와 검사를 session manager에 전달한다.

[resume 문서](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/tools/acp-agents/delivery.md#L89)의 upstream session ID는 해당 harness와 host의 식별자다. 코드에서 초기화 인자 전달까지 확인했고, 각 외부 harness의 session/load 구현은 실행하거나 추적하지 않았다. VIA Task identity와 이 session identity를 분리해야 한다.

### R11. 제한된 이력 수집 capability

OpenClaw [session-end transcript 타입과 lease](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/plugins/session-end-transcript.ts#L1)는 maxMessages / maxBytes, truncated, 읽기 불가 이유를 표현한다. [projectSessionEndTranscriptContext](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/plugins/session-end-transcript.ts#L107)는 명시된 conversation access가 있어야 reader를 제공하고 수명이 끝나면 revoke한다.

이 코드는 내부 plugin reader의 경계다. 모든 외부 Agent transcript를 수집할 수 있다는 증거는 아니다. VIA의 수집 계약에서도 ‘없음’, ‘미지원’, ‘일부만 수집’, ‘이미 지워짐’을 정상적인 관측 결과로 보존해야 함을 보여 준다.

### R12. Magentic-One의 공유 조정 기록

이전에 확인한 [MagenticOneOrchestrator](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L300)는 message thread와 task / progress ledger로 작업자를 조정한다. 다만 이 ledger에는 업무 내부 계획이 포함된다. VIA의 업무 기록을 설명하는 유사 사례로만 쓰며 그 계획 책임을 VIA로 가져오지 않는다. 이 경로만으로 이종 서비스 간 기억 이동이나 실제 사용자 청취 범위 추적을 증명할 수 없다.

## 4. 이 근거로 확정할 수 없는 것

- 이종 상용 Agent가 공유 파일 읽기, 구조화 접수, session resume, 범위 삭제를 모두 지원하는지 미확인이다.
- 공개 결과와 사용자 지시에서 portable 업무 기록을 구성할 수는 있지만, 실행 중인 domain plan과 숨겨진 tool 상태의 이전은 별도 문제다.
- 파생 기억에서 필요한 의미가 빠질 확률, 반복 인계에서의 비용 절감과 실제 RAM 최대치는 측정하지 않았다.
- 기억 생산과 인계의 성공을 업무 결과의 사실 정확성이나 Agent의 성능으로 대체할 수 없다.
- 소스의 검증 / fallback은 실용적 설계를 구성할 근거다. VIA에서 QA 우위를 확인한 실험 증거가 아니다.
