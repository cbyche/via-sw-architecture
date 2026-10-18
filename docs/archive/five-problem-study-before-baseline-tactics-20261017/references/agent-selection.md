# Agent 선택과 전환의 구현 근거

2026-10-16 / 공식 소스 정적 조사 / 실행 검증 없음

관련 설계: [문제 5](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/05-agent-selection-and-transition.md).

## 1. 조사 범위와 읽은 경로

검색 결과는 경로 발견에 사용하고 아래 commit의 본문을 읽었다. 제품 전체의 기능 유무를 판정한 목록은 아니다. ‘선택 가능’과 ‘해당 업무에 가장 적합함’은 별개다.

| ID | 고정 소스와 읽은 위치 | 확인한 사실 | VIA에 옮길 때의 제한 |
| --- | --- | --- | --- |
| SEL-01 | [Deep Agents subagents.py](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py), `SubAgent` 84–145, task 도구 구성 660–747 및 실행부 | 이름과 설명으로 main agent에게 위임 대상을 노출한다. 별도 subagent의 도구와 모델, isolated / fork 문맥을 구성한다 | 선택은 현재 판단의 도구 호출에 결합된다. 설명이 실제 업무 품질의 검증 자료라는 뜻은 아니다. VIA가 내부 업무를 분해할 권한까지 가져오는 근거도 아니다 |
| SEL-02 | [OpenHands TaskTool](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-tools/openhands/tools/task/definition.py), 120–143, `create`와 factory 정보 구성 | 등록된 agent 유형과 도구 설명을 task tool에 넣고 현재 agent가 유형과 prompt를 지정한다. 별도 위임 자체의 overhead를 명시한다 | 작업별 성능 학습이나 서로 다른 상용 Agent의 우열 판정은 확인하지 않았다 |
| SEL-03 | [CrewAI A2A wrapper](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/a2a/wrapper.py), `_execute_task_with_a2a` 302–390, `_augment_prompt_with_a2a` 639–735, `_prepare_delegation_context` 1014–1063, `_delegate_to_a2a` 1231–1295 | 실행 전 card를 가져오고 description / URL / skills와 접속 실패를 task prompt에 공급한다. 원래 agent의 모델 응답이 직접 처리 또는 위임과 대상을 정한다. endpoint의 등록 여부를 검사하고 context / task ID를 전달한다 | 기본 첫 endpoint fallback, 모두 연결 실패 시 자체 처리 등은 VIA에 그대로 채택하지 않는다. VIA의 권한, 업무 범위, 보류 정책을 별도로 적용해야 한다. card 조회는 실제 실행 예약이나 품질 검증이 아니다 |
| SEL-04 | [CrewAI BaseAgentTool](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/tools/agent_tools/base_agent_tools.py), 46–125 | role 문자열을 정규화하여 등록 후보를 찾아 Task를 만들어 실행한다 | 이름 일치가 capability나 업무 적합성 확인을 대체하지 않는다 |
| SEL-05 | [AutoGen SelectorGroupChat](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_selector_group_chat.py), `select_speaker` 152–218, `_select_speaker` 232–310 | selector 전용 model context에서 역할, 후보, history로 다음 speaker를 선택한다. candidate 함수, 단일 후보 fast path, 선택 함수 override가 있다. 잘못된 출력의 재시도와 fallback도 있다 | candidate 함수는 prompt 후보를 축소하지만 모델 반환은 전체 participant 이름 기준으로 확인한다. 반환값의 candidate subset membership를 강제하는 hard admission으로 인용하지 않는다. VIA는 별도 code로 최신 적격성과 request revision을 검사해야 한다. 이전 / 첫 speaker fallback도 VIA default로 복사하지 않는다 |
| SEL-06 | [Magentic-One orchestrator](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_magentic_one/_magentic_one_orchestrator.py), team description 102–105, progress ledger 311–444 | task와 team description, 실행 history를 사용하여 진행 여부, 다음 speaker와 instruction을 고르고 검증한다. plan 갱신과 재시도는 별도 상태를 사용한다 | VIA의 선택 전용 판단을 별도 문맥에 두는 근거다. 업무 내부 plan과 tool 선택까지 VIA에 넣는 근거로 사용하지 않는다 |
| SEL-07 | [OpenClaw multi-agent 문서](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/multi-agent.md), 팀 / 허용 대상 112–180, routing rules 절 | coordinator와 specialist를 구성하고 위임 허용 대상을 제한한다. `delegationMode: prefer`는 prompt guidance이다. 채널 binding은 결정적인 routing 규칙이다 | 채널 / identity binding을 현재 요청에 가장 적합한 Agent를 찾는 알고리즘으로 취급하지 않는다. 해당 문서는 scheduler 구현의 근거가 아니다 |
| SEL-08 | [AutoGen Swarm](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-agentchat/src/autogen_agentchat/teams/_group_chat/_swarm_group_chat.py), 45–110 | 마지막 HandoffMessage의 target으로 다음 speaker를 바꾸고 현재 speaker / message thread를 저장한다. target이 등록된 participant인지 확인한다 | worker가 다음 대상 제안을 할 수 있다는 근거다. VIA의 대화나 외부 Action 채택 권한을 worker에 넘겨야 한다는 뜻은 아니다 |
| SEL-09 | [A2A 1.0.0 specification](https://a2a-protocol.org/v1.0.0/specification/), Agent Card, capability validation, context / Task, cancel 절 | 기능과 인터페이스 선언, Task 상태 조회와 취소 요청의 공통 표현을 제공한다 | 등록 기능은 업무 품질 증명이 아니다. Agent 간 내부 상태 이식, cancellation 성공 보장, 선택용 무부작용 trial은 이 문서를 근거로 가정할 수 없다 |

CrewAI의 [A2A 설명](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/docs/edge/en/learn/a2a-agent-delegation.mdx) 17–31, 221–291도 코드와 대조했다. 동적으로 가져온 card에 기반한 선택과 연결 실패 처리를 확인했지만, 여러 후보의 실제 업무 성공률을 독립적으로 검증하여 최고 성능 Agent를 고르는 기능은 위 경로에서 확인하지 않았다.

## 2. 대안으로 남기는 메커니즘

- **현재 요청 판단에서 선택:** SEL-01~04. 요청을 해석하는 모델이 후보 설명을 읽고 위임 선택까지 제안한다.
- **독립된 선택 문맥에서 선택:** SEL-05~06. 정해진 요청과 역할 / 진행 근거를 선택 전용 실행에 공급하고, 그 출력과 현재 요청을 다시 연결한다.
- **공통 보완:** SEL-07의 후보 허용 제한, SEL-09의 capability / protocol 확인은 양쪽 모두 필요하다. SEL-05의 prompt 후보 축소만으로 실행 적격성이 강제된다고 주장하지 않는다. 최종 code admission은 VIA용 공통 설계다.
- **조합 가능한 전환 정책:** SEL-08의 worker handoff 제안은 어느 선택 구조에도 입력으로 붙일 수 있다. 이것만으로 경쟁안 C를 만들지 않는다.

## 3. 확보하지 못한 근거와 설계 확장

이 조사에서 ‘모든 후보에게 같은 업무를 시험시켜 품질과 가격을 비교한 뒤 실행 권한을 부여’하는 구조를 위 다섯 harness의 확인된 일반 동작으로 제시할 근거는 확보하지 못했다. 그런 방식은 실제 외부 Action과 비용이 발생할 수 있으므로, card 조회를 가상 trial로 바꾸어 설명하지 않는다.

VIA의 registry revision, 요청 revision과 선택 결과의 연결, 동일 Task의 execution attempt 교체, 실행 여부 불명 상태의 보류, 독립 품질 근거의 유효 범위는 **VIA용 설계 제안**이다. 위 프로젝트가 모두 동일하게 구현했다는 주장이 아니다.
