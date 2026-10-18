# VIA 요청 근거 공급의 구현 근거

2026-10-16 / 공식 저장소 소스 확인 / 실행과 성능 측정 없음

[QA에서 시작한 문제 지도](../architecture/12-decisions/decision-packages/qa-first-exploration/README.md)에서 근거 공급 문제를 먼저 고른 뒤 관련 구현을 확인했다. [이전 다섯 harness 조사](./agent-harness-20261015.md)의 고정 commit을 사용했다. 이 문서는 제품 순위를 정하지 않고 해당 기능의 실행과 상태 경로를 기록한다.

## 1. OpenHands: 보존한 사건에서 유지 문맥을 만드는 경로

고정 판본: `OpenHands/software-agent-sdk@fd387b58dc7b707b73c159845c0396a17000be24`.

| 소스 | 확인한 구현 | VIA로 옮길 때 추가할 것 |
| --- | --- | --- |
| [RollingCondenser, 107행](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/context/condenser/base.py#L107) | 기록된 events로 `View`를 생성. 필요한 경우 `Condensation`을 사건 이력에 추가하고 축약된 모델 입력 view를 만듦 | 화면과 실제 전달 범위를 포함한 출처, 구간과 revision, 정정에 따른 view 무효화 |
| [LLMSummarizingCondenser, 54행](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/context/condenser/llm_summarizing_condenser.py#L54) | 별도 LLM으로 view를 요약. 사건 수와 token 등 조건으로 축약 필요성 판단 | VIA의 첫 응답 앞에서 요약이 발생하는 비용과, 요청 전에 수행하고 쓰지 않은 비용을 모두 기록 |
| [Agent.step, 694행](https://github.com/OpenHands/software-agent-sdk/blob/fd387b58dc7b707b73c159845c0396a17000be24/openhands-sdk/openhands/sdk/agent/agent.py#L694) | 현재 view에서 모델 메시지를 준비. condensation이 필요하면 그 사건을 남긴 뒤 이후 step에서 진행 | VIA의 bounded read와 업무 위임 경계, first content UI의 실제 표시 |

**근거가 지지하는 범위:** 원본 사건과 모델에 제공하는 유지 view를 분리하는 A의 실현 가능성. 모든 요약이 사용자 입력 전에 완료되거나 모든 오래된 지칭이 보존된다는 주장은 지지하지 않는다.

## 2. LangChain Deep Agents: 외부 보존과 필요한 범위의 재조회

고정 판본: `langchain-ai/deepagents@7764f1f8d21a594f0cbd31d273a7cad36597d6ba`.

| 소스 | 확인한 구현 | VIA로 옮길 때 추가할 것 |
| --- | --- | --- |
| [FilesystemMiddleware read_file, 2241행](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/filesystem.py#L2241) | 경로와 read 권한을 검사하고 backend의 지정 offset / limit을 읽어 ToolMessage로 반환 | 자유로운 파일 업무 실행 대신 허용된 화면과 대화 원본의 bounded read로 제한 |
| [SummarizationMiddleware, 1526행](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/summarization.py#L1526) | summary event를 적용해 입력을 재구성. 필요 시 backend offload와 요약. 원본 메시지를 지우는 방식과 구별 | 지칭 대상 후보의 원본 범위, 전달 여부, 허용 철회에 따른 사용 차단 |
| [SubAgentMiddleware, 747행](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py#L747) / [804행](https://github.com/langchain-ai/deepagents/blob/7764f1f8d21a594f0cbd31d273a7cad36597d6ba/libs/deepagents/deepagents/middleware/subagents.py#L804) | child 결과를 반환하며 isolated와 fork 입력 경로를 구분. 일반 task 호출은 child 완료를 기다림 | 별도 근거 처리자의 권한과 원본 반환 계약. 독립 문맥을 독립 OS 프로세스나 durable Task로 해석하지 않음 |

**근거가 지지하는 범위:** 모델이 현재 필요에 따라 원본의 일부를 읽고 입력을 확장하는 B의 핵심 메커니즘과 C의 문맥 분리. Deep Agents 자체도 rolling summary를 사용하므로 제품 전체를 순수 B라고 부를 수 없다. VIA의 요청별 view 수명은 이 도구 메커니즘을 이용해 구성한 설계 제안이다.

보조 공식 설명: [Context engineering](https://docs.langchain.com/oss/python/deepagents/context-engineering). 문서는 조회 시점의 설명이며 정확한 구현 판본은 위 commit을 기준으로 한다.

## 3. OpenClaw: 응답 전 별도 recall을 조건부로 기다리는 경로

고정 판본: `openclaw/openclaw@e7a155a8ae860d71b1ab5eed45fa4af338011db9`.

| 소스 | 확인한 구현 | VIA로 옮길 때 추가할 것 |
| --- | --- | --- |
| [Active Memory 설명](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/docs/concepts/active-memory/how-it-works.md#L9) | 강한 검색 결과의 제한된 주입과, 회상 의도가 있으나 근거가 약할 때 별도 worker를 통한 deep recall을 구별 | 기억 외의 화면과 실제 음성 전달 원장을 조회할 수 있는 제한된 API |
| [before_prompt_build 등록, 213행](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/active-memory/index.ts#L213) | 주 응답의 prompt 구성 전에 근거 공급 hook 실행 | 해석기 앞 선행 호출과 해석 중 추가 질의의 구분 |
| [escalation과 결과 주입, 517행](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/active-memory/index.ts#L517) | 일반 turn은 recall을 건너뛸 수 있음. 현재 입력과 최근 turn으로 query를 만들고 recall을 await한 뒤 summary를 prepend | 반환 summary와 원본 evidence를 구별. 확정 지칭과 Task 연결은 VIA의 Request Interpreter가 판단 |
| [publish 검사, 295행](https://github.com/openclaw/openclaw/blob/e7a155a8ae860d71b1ab5eed45fa4af338011db9/extensions/active-memory/index.ts#L295) | deadline, tool authority, hook 수명과 audience의 현재성을 확인해 주입 | VIA 요청 revision과 자료 범위의 철회 조건. 외부 hook 검사만으로 부분 정정의 의미를 판단했다고 볼 수 없음 |

**근거가 지지하는 범위:** C는 실제로 사용되는 응답 전 근거 서비스 구조다. 기본 escalation 경로가 모든 요청에 helper를 호출하지 않는다는 점까지 강한 대안에 반영했다. 이 구현이 VIA의 시간에 따른 화면 지칭을 완성한다는 뜻은 아니다.

## 4. CrewAI: 별도 회상 흐름의 탐색과 비용

고정 판본: `crewAIInc/crewAI@3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8`.

| 소스 | 확인한 구현 | VIA에서의 의미 |
| --- | --- | --- |
| [RecallFlow와 RecallState](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/recall_flow.py#L1) | 질의 분석, 탐색 범위, evidence gap, confidence와 exploration budget을 갖는 별도 흐름. 하위 질의와 병렬 검색 및 추가 탐색 | C가 한 번의 vector 검색 이상의 독립 실행 상태를 가질 수 있는 실제 근거 |
| [Memory.recall, 686행](https://github.com/crewAIInc/crewAI/blob/3da2d4e9932b5f2fa9aa73ea9245e0f8b5c154f8/lib/crewai/src/crewai/memory/unified_memory.py#L686) | pending write를 drain한 뒤 shallow 또는 deep recall 경로 사용 | 배경 준비와 별도 검색이 항상 무료이거나 첫 응답 경로에서 사라지는 것은 아님 |

**적용 한계:** 기억 검색 흐름이 제공하는 후보와 VIA의 최종 지칭 대상 집합은 다르다. recall 품질을 QA02 점수로 가져오지 않는다. 주 해석기 또한 같은 병렬 검색과 cache를 쓸 수 있으므로 C만의 성능 이익으로 계산하지 않는다.

## 5. AutoGen Magentic-One: 현재 관측을 묶어 모델에 제공

고정 판본: `microsoft/autogen@027ecf0a379bcc1d09956d46d12d44a3ad9cee14`.

[MultimodalWebSurfer, 490행](https://github.com/microsoft/autogen/blob/027ecf0a379bcc1d09956d46d12d44a3ad9cee14/python/packages/autogen-ext/src/autogen_ext/agents/web_surfer/_multimodal_web_surfer.py#L490)은 interactive rect와 viewport screenshot을 확보하고, 대상 목록과 vision 이미지로 모델 입력을 만든다. 화면에 보이는 구조 정보와 원본 시각 근거를 함께 공급하는 실제 예다.

이 경로는 A/B 공통의 현재 관측 묶음에 참고할 수 있다. 브라우저 조작과 domain plan은 VIA 밖이다. 발화 중 포인터 이동, 과거 실제 청취와 부분 정정은 별도로 설계해야 한다. Magentic-One의 제품 전체 조정 구조를 이번 근거 공급 A/B에 억지로 배정하지 않았다.

## 6. 소스 사실과 VIA 추론의 경계

| 구분 | 이번 조사에서 말할 수 있는 것 |
| --- | --- |
| 구현 사실 | 유지 view, 원본 offload와 bounded read, 별도 recall의 조건부 실행과 반환이 공식 코드에 존재 |
| VIA 설계 제안 | 해당 메커니즘을 최근 대화, 화면 관측, 실제 전달 원장과 Task 사실에 적용하고 동일 채택 경계로 연결 |
| 품질 가설 | 반복 조회를 줄이는 이익과 반복 입력 / 선행 유지 비용의 손익. 별도 recall의 root 문맥 축소와 왕복 비용의 손익 |
| 미확인 | VIA 장면에서 실제 지칭 정확도, 유의미한 지연과 총 비용 차이, Rust 변경 모듈 수, 512 MB 충족 |

다섯 프로젝트를 모두 조사했지만 각 제품을 한 대안에 독점 대응하지 않는다. 제품이 여러 메커니즘을 혼합한다는 사실 자체를 저비용 혼합안 검토에 반영한다.
