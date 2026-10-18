# 근거 공급과 업무 인계의 공식 구현 근거 원장

2026-10-18 / 고정 source 정적 확인 / 제품 실행, 모델 호출, 성능 측정 없음

대상은 [근거 공급 통합 분석](../../architecture/12-decisions/decision-packages/seven-problem-review/01-evidence-supply.md)과 [업무 인계 분석](../../architecture/12-decisions/decision-packages/seven-problem-review/05-agent-handoff.md)이다. VIA 요구를 먼저 정하고 작업 문맥 수명, 과거 근거 생산, 인계 자료 선택 책임에 해당하는 구현을 대조했다. 다섯 harness를 하나씩 A/B/C 제품으로 분류하지 않았다.

## 1. 확인 범위와 사실의 등급

| 저장소 | 고정 commit | 2026-10-18 직접 읽은 주요 범위 |
| --- | --- | --- |
| OpenHands/software-agent-sdk | `fd387b58dc7b707b73c159845c0396a17000be24` | RollingCondenser base/LLM 선언, delegate의 기존 child 사용 |
| langchain-ai/deepagents | `7764f1f8d21a594f0cbd31d273a7cad36597d6ba` | isolated/fork state 구성, child backend 전달, 실제 범위 읽기 |
| openclaw/openclaw | `e7a155a8ae860d71b1ab5eed45fa4af338011db9` | consolidation의 이전 항목/lineage 검증, 종료 transcript 접근 lease |
| crewAIInc/crewAI | `3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8` | task context 조합, memory recall의 write barrier |
| microsoft/autogen | `027ecf0a379bcc1d09956d46d12d44a3ad9cee14` | Magentic-One facts/plan 생성, 상태 저장/복원, reset/ledger 배포, progress 호출 |
| openclaw/acpx | `1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75` | 0.19.4의 실제 resume/load 선택, 파일 읽기와 반환 범위 |

추가 ACP 명세의 고정 commit은 `agentclientprotocol/agent-client-protocol@c8775197952214bf3d2cabdd83c66c3de358af57`이다. EH13~EH14의 일부 확인은 [2026-10-17 상세 조사](../five-problem-study/handoff-capability-check-20261017.md)를 재사용하며 그 범위를 각 항목에 표시했다. 과거 source 확인을 새 실행 검증이라고 표현하지 않는다. 이 조사는 pinned code의 메커니즘 확인이며 현재 배포된 모든 Agent의 지원표가 아니다.

## 2. 근거 공급

### EH01. OpenHands: 유지되는 view와 조건부 condensation

**직접 확인:** [RollingCondenser base.py](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/context/condenser/base.py#L90)는 event들로 view를 만들고 soft/hard 조건에서 condensation을 처리한다. soft에서 생성할 수 없는 경우 uncondensed view를 반환할 수 있지만 hard에서는 hard reset 경로를 시도하고 실패를 전파할 수 있다. [LLMSummarizingCondenser](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/context/condenser/llm_summarizing_condenser.py#L54)는 잊는 event를 요약하는 별도 LLM 설정과 보존 범위, hard reset 재시도 제약을 갖는다.

**설계에 쓰는 범위:** 같은 대화에서 근거 view를 유지하고 예산에 맞춰 압축하는 실제 경로다. 유지 입력과 압축 호출 비용을 모두 고려해야 한다. VIA의 실제 전달 구간, 현재 권한, source revision, 명시적 누락과 원본 보완은 VIA가 추가한다. “압축 실패하면 언제나 전체 원문으로 정상 처리”라는 주장은 source가 뒷받침하지 않는다.

### EH02. Deep Agents: 저장된 자료의 범위 읽기

**직접 확인:** [filesystem.py](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/filesystem.py#L2241)는 path와 읽기 권한을 검사하고 backend `read/aread`에 offset/limit를 전달한다. [이전 조사에서 확인한 summarization offload](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/summarization.py#L1526)는 원본을 별도 저장하고 요약을 생산하는 근거다. 후자는 이번에 해당 함수 전체를 다시 읽은 것으로 기록하지 않는다.

**설계에 쓰는 범위:** 큰 원본을 계속 prompt에 유지하지 않고 필요한 범위를 읽는 방법은 실제 구현이다. Deep Agents도 요약과 기억을 사용하므로 제품 전체가 요청별 무상태라는 주장은 틀리다. 범위 읽기 인터페이스만으로 모든 backend의 I/O/RAM이 범위에 비례한다고 단정하지 않는다.

### EH03. OpenClaw: 생산된 기억의 갱신 검증

**직접 확인:** [applyMemoryConsolidationPlan](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/memory-core/src/dreaming-consolidation.ts#L276)은 기존 항목의 손실 비율, promotion marker/중복, lineage의 기존 항목 일치 등을 검사하고 맞지 않으면 계획을 거절한다. 이 코드는 임의로 덮어쓰는 단순 요약과 구별된다.

**이전 동일 commit 확인:** [candidate 수집](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/memory-core/src/dreaming-consolidation-candidates.ts#L4)의 신뢰 입력 제한, [실행 부분](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/memory-core/src/dreaming-consolidation.ts#L387)의 모델 생산/실패 처리, [memory provenance](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/memory-provenance.md#L17)의 삭제 범위 한계를 기존 조사에서 확인했다.

**설계에 쓰는 범위:** 별도 기억 생산과 lineage 검사가 가능한 구현 근거다. VIA의 관계/coverage/cursor/job/outbox를 한 revision으로 게시하는 계약, 삭제 epoch와 사용등록의 직렬화, 의미 completeness는 이 코드가 자동 제공하는 것이 아니다. 해당 검사를 의미 정확성의 증명으로 사용하지 않는다.

### EH04. CrewAI: 선행 쓰기의 비용이 recall 경로에 남는다

**직접 확인:** [unified_memory.py recall](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L686)은 recall 전에 `drain_writes()`를 호출한다. shallow와 deep recall을 구별하고 deep에서는 질의 정제/여러 탐색을 사용한다고 설명한다.

**설계에 쓰는 범위:** 생산과 소비를 나누어도 미완료 생산이 읽기 대기로 다시 나타날 수 있다. C의 선생산을 무조건 요청 밖의 무료 작업으로 세면 안 된다. CrewAI의 검색 score나 depth를 VIA의 source completeness 또는 인계 정확도와 동일시하지 않는다.

### EH05. Magentic-One: 유지 상태와 재배포에는 실제 생성 비용이 있다

**직접 확인:** [초기 facts/plan 생성](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L130)은 모델로 초기 사실과 계획을 만든다. [save/load](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L225)는 message thread, task, facts, plan과 진행 카운터를 보존한다. [outer-loop 재진입](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L264)은 참가자를 reset하고 ledger를 thread에 넣고 배포한다. [progress ledger](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py#L307)는 모델 호출과 응답 검증/재시도를 갖는다.

**설계에 쓰는 범위:** 원본 메시지 외에 공통 작업 설명을 유지하고 다시 전달하는 구조가 실재하며 생성/갱신 비용이 있다. VIA는 그 domain planning을 맡지 않는다. facts/plan이 source-backed actual-delivery 계약이라는 근거도 아니므로, B의 기록 수명/재배포 근거로만 사용한다.

## 3. 인계와 수신자 읽기

### EH06. Deep Agents: isolated 입력과 fork는 서로 다른 인계다

**직접 확인:** [subagents.py state 준비](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py#L795)는 isolated child에 description 기반 메시지를 만들고, fork에서는 허용 상태와 부모 메시지를 구성한다. 선언형 fork와 opaque compiled subagent의 상태 전달 범위에도 제약이 있다.

**설계에 쓰는 범위:** 목적별 제한 입력과 지원되는 내부 상태 상속은 실제 대안이다. fork를 모든 외부 Agent의 private memory나 실행 상태 이전으로 확대하지 않는다. VIA portable capsule은 공개 자료와 명시 계약에 근거한다.

### EH07. Deep Agents: child에 실제 backend를 연결한다

**직접 확인:** [graph.py](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/graph.py#L722)는 child FilesystemMiddleware에 backend와 child 권한을 전달한다. EH02의 read 구현으로 이어진다. [StoreBackend](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/backends/store.py#L90)의 namespace/BaseStore 경로는 2026-10-17에 확인했다.

**설계에 쓰는 범위:** 수신자가 자료 경로를 받고 필요한 범위를 직접 읽는 C가 실제 가능한 구성이다. 같은 backend가 보인다는 것만으로 namespace, 권한, 원본 판본과 철회 의미가 모두 일치하는 것은 아니다. 이종 Agent의 VIA 읽기 API는 추가 adapter 계약이다.

### EH08. CrewAI: 목적별 task context 조합

**직접 확인:** [crew.py _get_context](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/crew.py#L1879)는 설정에 따라 전체 앞선 task outputs 또는 명시한 Task 출력들을 context로 조합한다.

**설계에 쓰는 범위:** 필요한 결과를 다음 작업에 명시적으로 제공하는 A의 근거다. 현재 사용자 조건/접근권한/실제 전달 범위의 완전성은 이 함수만으로 보장되지 않는다. 출력 문자열을 합치는 것과 VIA 인계 계약 검증은 구별한다.

### EH09. OpenHands: 같은 child 대화에 후속 작업 전달

**직접 확인:** [delegate 구현](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/delegate/impl.py#L287)은 등록된 child를 확인하고 그 LocalConversation에 task를 보내 실행한 뒤 최종 공개 응답을 받는다. child별 thread를 만들고 join하는 경로다.

**설계에 쓰는 범위:** 같은 harness의 기존 child 대화에 후속 요청을 보내는 native 재사용 근거다. 이 구현의 blocking join을 VIA 지속 입력의 설계로 복사하지 않는다. 해당 부분은 arbitrary Agent 간 교체나 프로세스 재시작 뒤 재연결 보장을 입증하지 않는다.

### EH10. OpenClaw: 공개 transcript도 무조건 얻을 수 없다

**직접 확인:** [session-end-transcript.ts](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/plugins/session-end-transcript.ts#L1)는 권한 없음, cutoff 없음, incognito 삭제, archive unavailable, unsupported source를 구별한다. readTail은 maxMessages/maxBytes와 truncated 결과를 갖는다. [lease 읽기](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/plugins/session-end-transcript.ts#L66)는 읽기 전후 유효성을 확인하며 [projection](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/plugins/session-end-transcript.ts#L107)은 명시 대화 접근 권한을 요구한다.

**설계에 쓰는 범위:** VIA가 자신이 교환한 공개 자료를 보존하는 것과 외부 전체 transcript를 모을 수 있다는 것은 다르다. 접근 가능한 실제 범위와 누락을 기록해야 한다. 이 hook이 Agent 내부 추론 export라는 주장은 하지 않는다.

### EH11. acpx 0.19.4: 실제 resume/load 선택

**직접 확인:** [createOrLoadRuntimeSession](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/runtime/engine/manager.ts#L272)은 resumeSessionId가 있으면 resume capability를 우선 사용하고, 없으면 load capability를 확인한다. 둘 다 없으면 오류다. ID가 없으면 새 session을 만든다.

**2026-10-17 대조:** [OpenClaw acpx 의존 버전](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/acpx/package.json#L10) 0.19.4와 이 acpx commit의 [package.json](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/package.json#L3)을 맞췄다. [실제 protocol 요청](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/acp/client.ts#L1230)도 확인했다. 배포 npm 바이너리 동일성이나 특정 설치 제품의 응답은 실행하지 않았다.

**설계에 쓰는 범위:** 같은 Agent의 지원되는 session을 재개하는 tactic 근거다. 다른 Agent로 ID만 보내 기억이 이전된다고 해석하지 않는다.

### EH12. acpx 파일 읽기: 작은 응답이 작은 RAM은 아니다

**직접 확인:** [filesystem.ts readTextFile](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/filesystem.ts#L91)은 호출 권위와 root 내 경로, permission mode를 확인한다. `workspace.readText`로 내용을 읽은 다음 권위를 다시 확인하고 요청 줄 범위를 자른다.

**설계에 쓰는 범위:** C를 위한 수신자 read 경로가 실재한다. 반환 줄 수만으로 전체 읽기 I/O/최대 RAM 절약을 입증하지 않는다. session 권한 검사가 VIA의 결과 revision과 현재 업무 목적까지 자동 검증하는 것도 아니다.

### EH13. 같은 ACP라도 연결 종류와 버전이 다르다

**2026-10-17 공식 source 확인을 유지:** [ACP v1 file-system](https://github.com/agentclientprotocol/agent-client-protocol/blob/c8775197952214bf3d2cabdd83c66c3de358af57/docs/protocol/v1/file-system.mdx#L8)은 client가 지원을 알린 경우의 선택 기능이다. [acpx handler 등록](https://github.com/openclaw/acpx/blob/1f1cc452fd3c6e9dfb2b32ce12cdf728d022aa75/src/acp/client.ts#L1032)도 capability 조건을 갖는다. [OpenClaw ACP bridge](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/cli/acp.md#L13)의 client filesystem/session MCP 제한은 [외부 harness acpx adapter](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/acpx/src/runtime.ts#L450)의 구성 MCP 전달 경로와 구별한다.

[v2 migration](https://github.com/agentclientprotocol/agent-client-protocol/blob/c8775197952214bf3d2cabdd83c66c3de358af57/docs/protocol/v2/migration.mdx#L781)은 filesystem 메서드를 제거하고 MCP로 제공하는 방향이며 [문서 상태](https://github.com/agentclientprotocol/agent-client-protocol/blob/c8775197952214bf3d2cabdd83c66c3de358af57/docs/protocol/v2/migration.mdx#L18)는 draft다. 현재 모든 Agent가 v2를 지원한다고 일반화하지 않는다.

### EH14. OpenClaw: 재개 요청과 동일 문맥 복원을 구별

**2026-10-17 공식 source 확인을 유지:** [resume authorization](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/acp/runtime/session-meta-resume-authorization.ts#L71)은 요청자와 생성/자식 연결을 검사한다. [후보 판정](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/acp/runtime/session-meta-resume.ts#L10)은 backend/harness와 저장 기록을 대조한다. [runtime 복원](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/src/acp/control-plane/manager.runtime-handle-ensure.ts#L228)은 특정 저장 ID 초기화 실패에서 ID 없이 새로 초기화하는 경로를 갖는다. 모든 오류가 이 경로를 거치는 것은 아니다.

**VIA 추가 설계:** fresh session이면 portable 계약으로 복귀한다. 동일 session 복원 확인과 필수 조건/자료 유효성 확인은 별개다. 외부 수행 여부 불명인 동일 Action은 새 session으로도 재실행시키지 않는다.

## 4. 확인 결과와 남은 한계

- 유지 view, 원본 범위 읽기, 별도 기억 생산, 목적별 context 전달, 공통 ledger 재배포, 지원되는 수신자 읽기와 native session 재개는 공식 구현에서 각각 확인된다.
- 다섯 harness 중 어느 것도 이 조사만으로 VIA actual-delivery/source revision/철회/인계 oracle 전체를 구현한다고 할 수 없다. 해당 계약과 복구는 VIA 설계다.
- 원본 접근이 없는 자료, 비공개 memory, 자유로운 실행 상태 이동을 전제로 대안을 만들지 않았다. 특정 상용 음성 앱의 전체 기능/성능을 검증하지 않았다.
- reference의 존재는 대안의 작동 가능성 근거다. VIA에서 A/B/C 중 하나의 정확도, 지연, 비용, RAM 우위의 측정 증거가 아니다.
- source code의 static 경로와 문서 상태를 확인했으며 라이브 API 호환, 모델 의미 정확도, concurrency race의 실행 재현, Windows 자원 측정은 범위 밖이다. 이전 리뷰를 이번 통합 문서의 독립 검증으로 전용하지 않는다.
