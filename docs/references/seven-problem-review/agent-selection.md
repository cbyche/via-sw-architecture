# Agent 선택 구조의 추가 조사 근거

2026-10-18 / 공식 문서 대조 / 관련 [통합 설계](../../architecture/12-decisions/decision-packages/seven-problem-review/06-agent-selection.md)

## 1. 조사 결과

선택을 현재 요청 판단의 위임 도구 호출에 결합하는 구조와, 별도 selector가 정리된 요청 및 참가자 정보를 읽는 구조는 모두 구현 근거가 있다. 이번 추가 조사는 채널의 고정 binding, worker의 handoff, 관리자의 업무 배분이 세 번째 기본안이 될 수 있는지도 확인했다. 이 세 경로를 VIA의 임의 업무에 대한 성능 기반 선택과 동일시하지 않는다.

| 자료와 확인 위치 | 구현 사실 | VIA에 적용할 범위와 제한 |
| --- | --- | --- |
| [Deep Agents Subagents](https://docs.langchain.com/oss/python/deepagents/subagents), Custom subagents와 SubAgent 필드 | 주 agent가 이름과 description을 보고 task 도구로 대상을 지정한다. isolated와 fork 문맥은 별도 설정이다 | 결합 선택 A의 근거. 후보 설명은 독립 품질 평가가 아니며 fork가 서로 다른 제품 사이의 내부 상태 이전을 보장하지 않음 |
| [OpenHands TaskTool 고정 코드](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/task/definition.py), `create` 및 factory 설명 | 등록 agent 유형을 도구에 노출하고 호출자가 유형과 입력을 지정하는 경로 | A의 위임 인터페이스. 현재 코드의 세부 근거는 아래 이전 원장의 SEL-02에 보존. 상용 Agent 사이의 성능 비교기 근거가 아님 |
| [CrewAI A2A v1.15.28](https://docs.crewai.com/v1.15.28/en/learn/a2a-agent-delegation), Multiple A2A Agents와 Error Handling | 여러 endpoint를 설정하고 모델이 업무 요구에 따라 위임 대상을 선택한다. 연결 실패를 문맥에 알리는 옵션이 있다 | 원격 대상에도 A 방식이 가능함. 모두 실패할 때 자체 수행하는 예시는 VIA의 업무 책임 경계와 달라 그대로 채택하지 않음 |
| [AutoGen SelectorGroupChat](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/selector-group-chat.html), How Does it Work와 Custom Selector/Candidate Function | 대화와 역할 설명을 별도 선택 모델에 주고 다음 참가자를 고른다. 사용자 함수와 후보 축소 경로가 있다 | B의 별도 선택 문맥 근거. 다음 발언자를 고르는 예시가 외부 업무의 최고 품질 Agent 선택을 검증한 것은 아님 |
| [AutoGen Magentic-One 고정 코드](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py), progress ledger와 next speaker | 진행 정보를 사용하여 다음 참가자와 지시를 결정한다 | 선택 실행 분리의 근거. 업무 내부 계획 전체를 VIA가 소유할 이유로 사용하지 않음 |
| [OpenClaw multi-agent routing](https://docs.openclaw.ai/concepts/multi-agent), Routing rules | 채널과 계정 및 상대 등 binding의 구체성으로 수신 메시지 담당을 결정한다 | 사용자 지정/계정 기본값의 입력 근거. 업무 목적에 적합한 후보를 판정하는 별도 품질 선택과 구별 |
| [AutoGen Swarm](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/swarm.html), How Does It Work | 현재 참가자가 HandoffMessage로 다음 참가자를 제안하고 공통 메시지를 이어 사용한다 | 현재 Agent의 추천을 입력으로 받는 보완에 적용 가능. VIA의 최종 실행 허가와 사용자 대화 소유권을 외부 Agent에게 넘기지 않음 |
| [CrewAI Processes v1.15.28](https://docs.crewai.com/v1.15.28/en/concepts/processes), Hierarchical process | 관리자가 참가자에게 업무를 배분하는 별도 관리 경로를 제공한다 | B의 관리자 패턴 보조 근거. VIA가 외부 업무를 내부 세부 작업으로 재계획하는 것은 범위 밖 |
| [A2A v1.0.0 specification](https://a2a-protocol.org/v1.0.0/specification/), AgentCard와 Cancel Task | Card는 인터페이스와 기능을 선언하고 취소는 현재 Task 상태에 따른 시도 및 결과를 반환한다 | capability 등록과 확인된 상태 경계. Card를 품질 정답으로 사용하거나 cancel 요청을 종료 사실로 간주하지 않음 |

고정 commit별 함수/줄 범위는 [기존 선택 원장 SEL-01~09](../five-problem-study/agent-selection.md)에 있다. 동적 공식 페이지는 2026-10-18 조회 시 설명이며 고정 코드의 동일 판본이라고 간주하지 않는다. OpenHands API 문서의 추측 경로는 접근에 실패했으므로 새 endpoint 검증 근거로 사용하지 않았다.

## 2. 조사에서 채택하지 않은 확대 해석

- 채널 binding을 선택 정확도가 높은 업무 router로 설명하지 않는다.
- 설명을 읽는 selector와 실제 능력의 독립 평가를 구분한다. 별도 실행은 모델을 더 똑똑하게 만드는 장치가 아니다.
- 제한 후보 prompt와 최종 실행 허가는 별도다. 고정 SelectorGroupChat 코드의 후보 축소를 강제 적격성 검사로 설명하지 않는다.
- Agent가 자기 다음 담당자를 추천해도 사용자 지정, 자료 사용권과 실행 중복 조건은 VIA가 확인한다.
- 후보들에게 실제 업무를 병렬 실행한 뒤 승자를 고르는 방식은 공짜 선택 절차가 아니다. 무부작용 평가 API와 별도 허가 없이 기본안으로 넣지 않는다.

이 자료가 보장하는 것은 두 기본 선택 경로의 구현 가능성이다. VIA에서의 정확도, 비용 우열과 실제 연결할 상용 서비스의 지원 범위는 별개의 근거가 필요하다.
