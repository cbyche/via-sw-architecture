# VIA Architecture Requirements Baseline v1.1

> Status: **Approved Baseline**
> Source: `VIA_Architecture_Requirements_Baseline_v1.1.docx`

VIA SW Architecture Requirements Baseline

Version 1.1 — Approved Baseline

Samsung PC Voice Interaction Agent

| 문서의 목적<br>이 문서는 SW Architecture 전문지식이 없는 심사위원도 VIA의 범위, 기능, 품질 목표와 그 근거를 읽는 순서대로 이해할 수 있도록 작성한다. 기술 용어는 필요한 경우 사용하되, 처음 등장할 때 뜻과 측정 범위를 명확히 설명한다. v1.0 Approved Baseline은 유지하며 이 문서는 v1.1 승인 전 검토본이다. |
| --- |


# 1. 문서 범위와 용어

## 1.1 설계 범위

본 과제의 설계 대상은 Samsung PC에 설치되어 사용자의 음성 중심 interaction을 받아들이고, 필요한 context를 수집·해석한 뒤 적절한 Downstream Agent에 작업을 위임하고, task와 권한 및 결과 전달을 관리하는 VIA(Voice Interaction Agent) SW이다.

| 설계 범위에 포함<br>Voice/Text 입력 adapter, Voice Engine, Context Engine, Intent Refiner, Agent Router, Session & Conversation State, Task/Workflow Manager, Agent Harness Port와 adapter, Policy/Consent/Identity, Model Gateway, Agent Registry, Memory, Observability/Audit, Notification, concurrent-task 및 resource-conflict coordination. |
| --- |

| 기본 설계 범위에서 제외<br>OpenCode, Claude Code, Qoder, Kimi, Qwen Code 등과 같은 범용 Downstream Agent runtime 자체의 내부 설계와 해당 Downstream Agent가 소유한 tool selection/tool execution 구조. VIA는 이들과 연결하기 위한 공통 Agent Harness Port/Contract를 설계하지만, Downstream Agent 내부의 tool runtime을 다시 구현하거나 중앙 통제하지 않는다. 다만 자체 Downstream Agent에만 가능한 특화 integration이 Product E2E 품질을 유의미하게 개선한다면 공통성 대 특화성의 trade-off를 Architecture Decision Point로 비교한다. |
| --- |

## 1.2 문서에서 사용하는 시스템 경계

| 용어 | 정의 |
| --- | --- |
| VIA 시스템 | 본 과제의 직접 설계 대상. Voice/Text interaction, context, intent, Downstream Agent routing/delegation, conversation/task lifecycle, context sharing/consent, model gateway, registry, memory, observability 등의 VIA core와 adapter를 포함한다. |
| Downstream Agent | VIA가 사용자 요청의 실제 업무 판단·계획을 위임하는 실행 주체. 내부 Samsung Agent 또는 3rd-party Agent일 수 있으며, 해당 Agent의 내부 reasoning/runtime 설계는 원칙적으로 VIA 설계 범위 밖이다. |
| Model / Voice Runtime | OpenAI Realtime, Qwen3-Omni 계열, GPT-5.6 Terra 등 VIA가 port를 통해 사용하는 AI 실행 dependency. Model의 내부 구조는 VIA가 소유하지 않는다. VIA는 Model Gateway에서 provider adapter와 함께 versioned Model Invocation Profile(prompt/instruction, model configuration, output schema, token budget, timeout/retry)을 관리한다. |
| Downstream Agent Tooling | Downstream Agent가 자체적으로 소유하는 tool selection, tool invocation, computer-use, external API integration 및 domain workflow execution. VIA 설계 범위 밖이며, VIA는 개별 tool call을 승인하거나 다시 실행하지 않는다. |
| 통합 제품(Integrated Product) | 사용자가 실제로 경험하는 전체 경로를 측정할 때 사용하는 범위. VIA 시스템 + 선택된 Model/Voice Runtime + Downstream Agent + Downstream Agent가 사용하는 Tool/External Service를 포함한다. |

| 중요한 책임 구분<br>VIA는 사용자의 voice/text 요청을 이해하고, 필요한 context를 연결하며, 적절한 Downstream Agent를 선택해 요청을 위임하고, conversation/task lifecycle과 progress·cancel·follow-up·result interaction을 관리한다. Downstream Agent는 실제 업무 reasoning과 planning을 수행하고 자체 tool을 선택·실행하여 OS/App/Web/External Service에서 작업을 완료한다. VIA는 Downstream Agent의 개별 tool call을 다시 승인하거나 실행하지 않는다. 다만 VIA가 외부 Model/Downstream Agent에 어떤 사용자 context를 제공할지는 VIA의 privacy/consent policy가 통제한다. Downstream Agent가 사용자 승인이나 추가 입력을 요청하면 VIA는 그 interaction을 사용자에게 전달하고 응답을 다시 해당 Downstream Agent execution에 전달한다. |
| --- |

# 2. 이해관계자(Stakeholders)

| ID | 이해관계자 | 주요 관심사항 |
| --- | --- | --- |
| STK-01 | CTO | Voice interaction이 향후 Samsung device의 핵심 interface가 될 수 있다는 전략적 관점, 제품 확장성, 장기 기술 방향 |
| STK-02 | 사용자 | 빠른 반응, 자연스러운 대화, 화면 맥락 이해, 안전한 실행, 여러 작업의 상태 관리, 개인정보와 비용 |
| STK-03 | 과제 책임자 | 제품 범위, 사용자 가치, Product E2E 목표, 일정과 복잡도 trade-off |
| STK-04 | VIA 개발자 | component 책임 경계, 변경 용이성, provider 교체, concurrency/state/failure 설계 |
| STK-05 | Downstream Agent 개발자 | Agent Harness Port, capability/permission descriptor, stream/cancel/status/follow-up contract |
| STK-06 | Model 개발자 | Voice/model latency·semantic quality 요구, cloud/private/local/on-device deployment 및 교체 |
| STK-07 | PC 플랫폼 연동 개발자 | Windows accessibility, pointer/window/focus capture, local context connector, resource co-existence 및 VIA 자체 local capability가 선택될 경우의 OS adapter |
| STK-08 | 보안 담당자 | authorization, consent, context egress, identity, audit, trust boundary |
| STK-09 | 검증 담당자 | request suite, latency/quality/resource benchmark, failure injection, architecture alternative comparison |
| STK-10 | 외부 Downstream Agent/서비스 제공자 | 표준 Agent Harness integration contract, authentication, timeout/cancel/status/follow-up, 제한된 context sharing, approval/clarification interaction |

# 3. 사용자/시스템 Use Case

| ID | Use Case | Goal / 설명 |
| --- | --- | --- |
| UC-01 | 실시간 음성 대화 및 로컬 즉답(Real-time Voice Interaction & Local Response) | VIA는 음성을 실시간으로 수신하고, 외부 data/action이 필요 없는 허용된 대화·일반 지식·interaction control 요청은 Downstream Agent 없이 local fast path에서 즉시 답할 수 있다. |
| UC-02 | 화면을 가리키며 요청하기(Screen-pointing Context Interaction) | 사용자가 pointer, selection, focus 등으로 화면 대상을 가리키면서 말한 내용을 해당 시점의 PC context와 연결한다. 한 발화 안에서 여러 위치를 순서대로 가리키는 경우도 포함한다. |
| UC-03 | 개인 정보 맥락 활용(Personal Context-aware Request) | 파일, Gmail, calendar, browser 등 허용된 personal context를 현재 요청에 필요한 범위에서 활용한다. |
| UC-04 | 적절한 Downstream Agent로 요청 위임(Downstream Agent Delegation) | VIA가 직접 완료하지 않는 요청을 capability·health·trust·policy에 따라 적절한 Downstream Agent에 위임한다. Downstream Agent는 trusted execution boundary를 가지며 자체 tool을 실행한다. Model endpoint 선택과는 별개의 문제이다. |
| UC-05 | PC·앱·웹·외부 서비스 작업 요청(PC/Application/Service Action Request) | 사용자의 PC·app·web·external service 작업 목표를 적절한 Downstream Agent에 위임하고, VIA는 task 상태·progress·cancel·follow-up·result interaction을 관리한다. 실제 업무 reasoning과 tool execution은 Downstream Agent가 소유한다. |
| UC-06 | 민감 정보 및 작업 승인(Sensitive Data/Action Approval) | VIA가 사용자 context를 외부 Model/Downstream Agent에 제공해야 하거나, Downstream Agent가 사용자 승인/추가 입력을 요구하는 경우 VIA가 해당 내용을 사용자에게 설명하고 interaction을 중계한다. 실제 action authorization/enforcement는 trusted Downstream Agent의 security boundary가 소유한다. |
| UC-07 | 장시간·다단계 작업 수행(Stateful / Long-running Task) | 즉시 끝나지 않는 작업을 durable task로 생성하고 waiting, checkpoint, resume, completion을 관리한다. |
| UC-08 | 현재 상호작용 즉시 중단·수정(Immediate Interrupt / Cancel / Correction) | 현재 VIA 음성 출력 또는 현재 진행 중인 interaction을 사용자가 즉시 끊거나 바로잡는다. 실시간 control이 핵심이다. |
| UC-09 | 세션 및 작업 복구(Session / Task Recovery) | connection 또는 process 문제가 발생해도 복구 가능한 conversation/task state를 다시 연결한다. |
| UC-10 | 진행 상황 및 결과 전달(Result / Progress Delivery) | task 상태에 따라 voice audio, text/chat surface, on-screen card/UI, Windows notification 등 적절한 channel로 진행·승인·완료·실패를 전달한다. |
| UC-11 | 한 발화의 복수 요청 처리(Compound Utterance Handling) | 한 발화에 섞여 있는 여러 요청을 나누고 independent/sequential/data-dependent/conditional 관계를 유지한다. |
| UC-12 | 생략되거나 모호한 요청 보완(Underspecified / Contextual Request Resolution) | app/source/target/parameter가 빠진 자연스러운 발화를 conversation, interaction context, personal context로 보완하고 필요한 경우에만 clarification한다. |
| UC-13 | 기존 작업 이어가기 및 후속 지시(Task Continuation & Follow-up) | 새로운 사용자 turn이 기존 task의 연장인지 새로운 task인지 판단한다. 기존 task의 연장이라면 관련 task와 기존 Downstream Agent execution/thread를 찾아 상태 조회·추가 지시·수정·취소를 이어가고, 새로운 요청이라면 별도 task로 시작한다. |
| UC-14 | 개인화 및 장기 기억 관리(Personalization & Memory Control) | 허용된 preference, routine, stable fact를 저장·조회·수정·삭제하고 lifecycle을 관리한다. |
| UC-15 | 음성·텍스트 혼합 대화(Mixed-Modality Conversation) | 하나의 logical conversation에서 voice와 text turn을 섞어도 conversation/context/task state를 이어간다. |
| UC-16 | 여러 작업 동시 처리(Concurrent Task Handling) | 동일 사용자가 여러 active task를 동시에 보유·조회·제어한다. 두 Downstream Agent execution이 같은 mouse/keyboard 같은 exclusive PC resource를 동시에 필요로 하는 경우, Agent가 선언한 resource requirement를 기준으로 VIA가 execution scheduling을 조정할 수 있다. VIA가 개별 tool call을 관장한다는 의미는 아니다. |

## 3.1 UC-02: 한 발화 안에서 여러 위치를 가리키는 경우

기존 v1.0의 ‘speech_started 시점에 단일 snapshot을 freeze한다’는 방식만으로는 아래와 같은 요청을 충분히 처리할 수 없다.

| 대표 요청<br>“(source 폴더를 가리키며) 여기에 있는 파일을… (pointer를 이동해 destination 폴더를 가리키며) 여기로 옮겨줘. 그리고 그림판에서 (여러 객체 주변을 pointer로 원을 그리며) 얘네들 위치가 어색하니까 (다른 위치를 가리키며) 이쪽으로 옮겨줘.” |
| --- |

이 경우 ‘여기’, ‘여기’, ‘얘네들’, ‘이쪽’이 모두 서로 다른 시간의 pointer 위치/gesture와 연결되어야 한다. 따라서 단일 snapshot이 아니라 user turn 전체의 interaction evidence를 시간순으로 보존해야 한다.

| 위치 | 구성요소 후보 | 책임 |
| --- | --- | --- |
| Context Engine 확장 | Interaction Timeline Recorder | 발화 시작부터 종료까지 pointer position/trajectory, hover, active window, focus, selection, accessibility hit-test 결과를 timestamp와 함께 기록하고 turn 종료 시 immutable timeline으로 고정한다. |
| Context Engine 선택 기능 | Pointer Gesture Analyzer | pointer dwell, 빠른 이동, 원형 trajectory 등에서 point/region/group candidate를 만든다. 최종 의미를 결정하지 않고 evidence만 제공한다. |
| Intent Refiner 확장 | Temporal Referent Binder | transcript의 ‘여기/얘네들/이쪽’ 같은 표현과 Interaction Timeline의 시간 구간을 맞춰 source/destination/group 등 여러 referent를 순서대로 binding한다. |

새로운 top-level Engine을 추가할 필요는 없다고 판단한다. Live Interaction Context Service를 ‘단일 snapshot + turn-level timeline’ 구조로 확장하고, Intent Refiner의 Referent Binder를 시간 정렬이 가능한 형태로 확장하는 것이 현재 책임 경계와 가장 일관된다. Sampling 방식, event 방식, transcript timestamp 의존 여부는 Architectural Decision Point에서 비교한다.

이 시나리오는 기능 존재 여부뿐 아니라 품질도 중요하다. 각 “여기/얘네들/이쪽” 표현의 발화 시점과 pointer/gesture evidence를 시간축에서 정확히 맞춰야 하며, QA-09에서 referent 연결 정확도와 Interaction Timeline capture 지연을 별도로 측정한다.

## 3.2 UC-05 대표 작업 패턴

| 구분 | 작업 패턴 | 대표 예 |
| --- | --- | --- |
| A | 앱/페이지 열기 | “네이버 증권 메인 화면 열어줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| B | 검색 후 후보 제시/열기 | “유튜브에서 최근 손흥민 경기 하이라이트 찾아줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| C | 로컬 파일 탐색/조작 | “문서 폴더에서 최근 PPT 찾아서 열어줘.” / “이 파일을 저 폴더로 옮겨줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| D | 화면 대상을 가리켜 조작 | “이 버튼 눌러줘.” / “이 항목을 아래로 옮겨줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| E | 한 발화의 다중 위치 지정 | “여기 파일을 여기로 옮기고, 얘네들은 이쪽으로 옮겨줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| F | 개인 정보 조회·요약 | “Gmail에서 최근 김대리 메일 열어서 요약해줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| G | 문서/콘텐츠 변환 | “이 문서 핵심만 정리해서 새 파일로 만들어줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| H | 메시지/메일 전달 | “방금 요약한 내용을 김대리에게 메일로 보내줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| I | 미디어 제어 | “음악 폴더에서 음악 하나 틀어줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| J | 시스템 설정 | “Windows 소리 설정 열어줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| K | 외부 transaction | “최신 갤럭시 버즈를 장바구니까지만 담아줘. 결제는 하지 마.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |
| L | 다단계 UI/creative automation | “그림판 열고 로켓을 간단히 그려줘.” (실제 작업 수행은 선택된 Downstream Agent가 담당) |

# 4. Architecture Evolution Scenarios

| ID | Scenario | 설명 |
| --- | --- | --- |
| AES-01 | Voice Runtime 교체 | 현재 구현 후보는 OpenAI Realtime 또는 Qwen3-Omni 계열이다. 이후 local S2S 또는 VAD+ASR+LLM+TTS stack으로 바꾸더라도 상위 VIA contract가 유지되어야 한다. |
| AES-02 | Model Endpoint 교체 | VIA 내부에서 text/reasoning model이 필요할 경우 GPT-5.6 Terra를 initial reference로 사용하되 Gemini Flash, private-cloud, local/on-device endpoint 등으로 교체 가능해야 한다. |
| AES-03 | Downstream Agent 추가 | CLI, ACP, internal Samsung agent, 3rd-party agent 등 새로운 Downstream Agent를 표준 Port/Contract로 추가한다. |
| AES-04 | Context Source / Interaction 연동 추가 | 새 file/mail/browser/context connector 또는 새로운 user interaction input adapter를 VIA에 추가한다. Downstream Agent가 자체적으로 사용하는 Context Source/Input Adapter 추가는 해당 Agent의 내부 evolution이며 VIA Architecture Evolution Scenario로 보지 않는다. |

| 별도의 중요한 Decision Point<br>기본 구조에서는 실제 업무 reasoning과 tool execution을 Downstream Agent가 소유한다. 다만 “소리 설정 열기”, “음악 멈춰”처럼 매우 단순하고 latency-critical하며 local-safe한 일부 capability까지 항상 Downstream Agent에 위임할지, 제한된 local fast path capability를 VIA가 직접 소유할지는 별도의 Capability Placement Boundary Decision Point로 비교한다. 이는 Tool Gateway를 VIA에 두는 결정과는 다르며, 기본안은 Thin VIA(Downstream Agent 위임)이다. |
| --- |

# 5. Functional Requirements

이 장에서 ‘시스템’이라는 표현은 사용하지 않고 모두 ‘VIA 시스템’이라고 명시한다. Downstream Agent 내부가 수행해야 하는 reasoning/task-planning은 VIA Functional Requirement에 포함하지 않는다. VIA 요구사항은 Downstream Agent 선택·위임·follow-up·task lifecycle·approval/clarification interaction·progress/result handling을 규정한다.

## 5.1 입력·대화·로컬 응답

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-01 | VIA 시스템은 사용자의 음성 입력을 수신하고 발화의 시작과 종료를 식별하여 user turn으로 관리해야 한다. | UC-01 | P0 |
| FR-02 | VIA 시스템은 음성 응답 중 새로운 사용자 발화가 발생하면 현재 audio output을 중단하고 새로운 turn을 처리할 수 있어야 한다. | UC-01,08 | P0 |
| FR-37 | VIA 시스템은 input modality와 관계없이 새로운 user turn이 기존 conversation/pending interaction/task의 연장인지 새로운 interaction인지 판단하고 적절한 conversation state와 연결할 수 있어야 한다. | UC-09,12,13,15 | P0 |
| FR-39 | VIA 시스템은 사용자의 text input을 user turn으로 수신하고 voice input과 공통 downstream request-processing flow에서 처리할 수 있어야 한다. | UC-15 | P0 |
| FR-42 | VIA 시스템은 외부 data/action 또는 Downstream Agent가 필요하지 않고 policy상 local response가 허용된 요청을 local fast path에서 처리할 수 있어야 한다. | UC-01 | P0 |

## 5.2 화면·개인 Context 수집과 지시 대상 연결

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-03 | VIA 시스템은 user turn 동안 pointer position/trajectory, hover, active window, focus, selection, accessibility hit-test 등 interaction evidence를 timestamp와 함께 수집하고 turn 종료 후 immutable interaction timeline으로 보존해야 한다. | UC-02 | P0 |
| FR-04 | VIA 시스템은 한 발화 안의 하나 또는 여러 지시 표현을 Interaction Timeline의 해당 시간 구간과 결합하여 여러 referent/source/destination/region candidate를 구분할 수 있어야 한다. | UC-02 | P0 |
| FR-05 | VIA 시스템은 허용된 file, Gmail, calendar, browser 등 personal context source에서 현재 요청과 관련된 정보를 검색하여 요청 처리에 사용할 수 있어야 한다. | UC-03 | P0 |
| FR-09 | VIA 시스템은 target/source/parameter가 생략된 경우 interaction context, conversation state 및 허용된 personal context를 이용해 누락된 의미를 보완할 수 있어야 한다. | UC-12 | P0 |
| FR-10 | VIA 시스템은 요청을 충분히 명확히 해석할 수 없는 경우 clarification을 요청하고 이미 확정된 context와 요청 정보를 후속 turn에 유지해야 한다. | UC-02,12 | P0 |

## 5.3 요청 구조화 및 복수 요청 처리

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-06 | VIA 시스템은 user turn을 goal, action, target, parameter 및 required capability를 포함하는 canonical request로 구조화해야 한다. | UC-01,05,12 | P0 |
| FR-07 | VIA 시스템은 한 user turn에 복수 실행 요청이 포함된 경우 이를 독립 처리 가능한 atomic request로 분해해야 한다. | UC-11 | P0 |
| FR-08 | VIA 시스템은 atomic request 간 independent, sequential, data-dependent, conditional 및 shared-context 관계를 표현하고 유지해야 한다. | UC-11 | P0 |

## 5.4 Downstream Agent 선택·위임·후속 요청

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-11 | VIA 시스템은 등록된 Downstream Agent의 capability, permission requirement, health 및 policy 조건을 기준으로 요청을 처리할 적절한 Downstream Agent를 선택하고 작업을 위임해야 한다. | UC-04 | P0 |
| FR-12 | VIA 시스템은 하나의 사용자 goal을 처리하기 위해 여러 Downstream Agent/capability가 필요한 경우 dependency에 맞게 실행 관계를 구성할 수 있어야 한다. | UC-04,07,11 | P1 |
| FR-35 | VIA 시스템은 Downstream Agent의 identity, capability, trust profile, permission/interaction requirement, execution characteristic, resource requirement 및 availability를 Agent Registry에 등록·조회할 수 있어야 한다. | UC-04,05 | P0 |
| FR-43 | VIA 시스템은 사용자의 후속 요청이 기존 task의 연장이고 동일 Downstream Agent의 conversation/execution context를 유지해야 하는 경우 해당 task와 Downstream Agent execution/thread를 식별하여 후속 요청을 전달해야 한다. 해당 Agent가 unavailable하거나 정책상 재사용할 수 없는 경우 Router가 재선택해야 한다. | UC-13 | P0 |

## 5.5 Task·Workflow·동시 실행 관리

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-23 | VIA 시스템은 여러 단계 또는 장시간 요청을 독립 식별 가능한 task로 생성하고 lifecycle state를 관리해야 한다. | UC-07 | P0 |
| FR-24 | VIA 시스템은 진행 중 task를 pause, resume, cancel 또는 failed state로 전환할 수 있어야 한다. | UC-07,08,13 | P1 |
| FR-25 | VIA 시스템은 external response, approval 또는 다른 task를 기다리는 동안 task state와 중간 결과를 유지하고 이후 실행을 계속할 수 있어야 한다. | UC-06,07 | P0 |
| FR-26 | VIA 시스템은 현재 interaction session이 종료된 이후에도 지속이 허용된 task를 계속 수행할 수 있어야 한다. | UC-07,15 | P0 |
| FR-27 | VIA 시스템은 사용자의 자연어 표현을 기존 또는 진행 중 task와 연결하고 관련 task를 조회할 수 있도록 해야 한다. | UC-13 | P1 |
| FR-28 | VIA 시스템은 기존 task의 현재 상태, 진행 단계, 완료 및 실패 여부를 사용자에게 제공해야 한다. | UC-13 | P0 |
| FR-29 | VIA 시스템은 후속 요청에 따라 기존 task를 취소, 재개하거나 허용 범위 내에서 변경할 수 있어야 한다. | UC-13 | P1 |
| FR-30 | VIA 시스템은 connection/session 중단 후 복구 가능한 conversation/task state를 다시 연결하고, 이미 완료된 side-effect action을 임의로 중복 실행하지 않아야 한다. | UC-09 | P0 |
| FR-40 | VIA 시스템은 동일 사용자에 대해 복수 task가 동시에 active 상태일 수 있도록 하고 각 task의 state, execution, cancellation 및 result를 독립적으로 관리해야 한다. | UC-16 | P0 |
| FR-41 | VIA 시스템은 복수 task가 동일 exclusive resource 또는 충돌하는 system state를 사용·변경하려는 경우 이를 식별하고 serialize, defer, reject 또는 사용자 확인 등의 방식으로 조정해야 한다. | UC-16 | P0 |

## 5.6 PC/서비스 작업 위임과 Downstream Agent 실행 연동

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-13 | VIA 시스템은 application 실행, web navigation, search, file/content access, media control, device setting 및 external service action과 같은 사용자 작업 목표를 적절한 Downstream Agent에 위임하고 해당 Agent execution을 task와 연결해 관리해야 한다. | UC-05 | P0 |
| FR-14 | VIA 시스템은 하나의 사용자 요청이 Downstream Agent 내부에서 여러 tool/action 단계로 수행되더라도 VIA가 그 내부 단계를 직접 orchestration하지 않고, Agent가 제공하는 progress/status/final event를 canonical AgentEvent로 받아 task 상태와 사용자 interaction에 반영할 수 있어야 한다. | UC-05,07 | P1 |
| FR-15 | VIA 시스템은 Context Engine에서 준비한 context, 이전 conversation/task 결과 또는 사용자의 후속 입력 중 Downstream Agent 실행에 필요한 정보를 Agent Harness contract를 통해 전달할 수 있어야 한다. | UC-05,07,11 | P1 |
| FR-16 | VIA 시스템은 Downstream Agent가 approval_required 또는 clarification_required event를 반환하면 해당 요청의 목적·대상·선택지를 사용자에게 전달하고, 사용자의 응답을 동일 Downstream Agent execution/thread에 다시 전달할 수 있어야 한다. | UC-02,05,06 | P0 |
| FR-17 | VIA 시스템은 Downstream Agent가 반환한 progress, completed, failed, cancelled, waiting 상태를 해당 task와 연결하고, Agent가 실패 또는 결과 불확실 상태를 반환한 경우 이를 성공으로 표시하지 않아야 한다. | UC-05,07 | P0 |

## 5.7 Identity·Consent·Context Egress

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-18 | VIA 시스템은 현재 사용자와 device identity를 식별하고 보호 resource 접근 전에 필요한 connected-account authorization 상태를 확인해야 한다. | UC-03,06 | P0 |
| FR-19 | VIA 시스템은 사용자가 external service account를 연결하고 access scope를 관리·철회할 수 있도록 해야 하며 invalid authorization을 후속 요청에 사용하지 않아야 한다. | UC-03,06 | P1 |
| FR-20 | VIA 시스템은 external Model/Downstream Agent로의 personal context egress 또는 VIA가 직접 소유하는 policy 대상 interaction에 사용자 consent가 필요한지 식별해야 한다. Downstream Agent 내부 tool/action에 대한 security policy는 해당 trusted Agent가 소유한다. | UC-03,05,06 | P0 |
| FR-21 | VIA 시스템은 context sharing에 consent가 필요하거나 Downstream Agent가 approval_required event를 반환한 경우 purpose, target, data/action summary를 사용자에게 제시하고 사용자 응답을 수집해야 한다. | UC-06 | P0 |
| FR-22 | VIA 시스템은 external Model/Downstream Agent로 전달하는 sensitive context를 사용자가 승인한 목적과 범위 안으로 제한해야 한다. Downstream Agent action의 authorization/enforcement는 해당 trusted Agent의 security boundary가 담당한다. | UC-06 | P0 |
| FR-36 | VIA 시스템은 context를 Downstream Agent 또는 Model에 제공하기 전에 purpose, destination identity, trust level 및 allowed context scope를 검증하고 허용된 context만 제공해야 한다. | UC-03,04,06 | P0 |

## 5.8 Result Delivery·Memory·Audit

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-31 | VIA 시스템은 task progress, approval 필요, completion, failure 또는 cancellation 결과를 현재 interaction 상태와 task 특성에 따라 voice audio, text/chat surface, on-screen card/UI 또는 Windows notification으로 전달할 수 있어야 한다. | UC-10,15 | P0 |
| FR-32 | VIA 시스템은 승인된 long-term preference, routine, stable fact 및 constraint를 저장하고 관련 요청에 활용할 수 있어야 한다. | UC-14 | P1 |
| FR-33 | VIA 시스템은 사용자가 저장된 personal memory를 조회, 수정 또는 삭제하도록 할 수 있어야 하며 변경된 상태를 이후 요청에 사용해야 한다. | UC-14 | P1 |
| FR-34 | VIA 시스템은 sensitive context access/egress, user consent/approval interaction, Downstream Agent execution lifecycle 및 VIA가 관찰 가능한 주요 AgentEvent에 대해 추적 가능한 audit event를 생성해야 한다. Downstream Agent 내부의 모든 tool call을 VIA audit의 필수 대상으로 요구하지 않는다. | UC-03~07 | P0 |
| FR-38 | VIA 시스템은 long-term memory candidate의 approval, provenance, expiration 및 conflict 상태를 관리하고 invalid memory가 active retrieval에 사용되지 않도록 해야 한다. | UC-14 | P1 |

## 5.9 Model 호출 Profile 및 Prompt 관리

| ID | 요구사항 | Source | Pri. |
| --- | --- | --- | --- |
| FR-44 | VIA 시스템은 Voice Runtime과 VIA 내부 Model 호출에 사용하는 system instruction/prompt template, model ID, generation parameter, output schema, token budget 및 timeout/retry 설정을 versioned Model Invocation Profile로 관리하고, 각 invocation에 사용된 profile/model/schema version을 trace할 수 있어야 한다. | UC-01,02,04,11,12 | P0 |

Model Invocation Profile은 prompt 문자열만 저장하는 기능이 아니다. 어떤 Model을 어떤 설정·schema·token budget으로 호출했는지를 하나의 versioned configuration으로 묶어 재현성과 평가 가능성을 확보한다. Provider-specific request 변환은 Model Gateway adapter가 담당한다.

# 6. Constraints

Constraint는 VIA가 반드시 따라야 하는 외부적/제품적 경계만 최소한으로 유지한다. Context sharing, consent, approval interaction처럼 VIA가 수행해야 하는 동작은 Functional Requirement로 옮겼다.

| ID | Constraint |
| --- | --- |
| CON-01 | VIA는 Samsung PC 환경에 local application 형태로 설치되어 동작해야 한다. |
| CON-02 | VIA는 Voice-first product여야 한다. Text input을 지원하더라도 음성이 primary interaction modality라는 제품 방향은 유지한다. |
| CON-03 | screen, pointer, UI state 및 personal context는 local-first 원칙으로 처리하고, external trust boundary로의 전송은 FR-20~22/36의 policy·consent 절차를 통과해야 한다. |
| CON-04 | VIA는 등록·검증된 trusted Downstream Agent만 execution 대상으로 사용한다. 각 Downstream Agent는 자체 tool/action security boundary와 credential/permission enforcement를 소유하는 것으로 가정하며, VIA가 이를 중앙 Tool Gateway로 재구현하지 않는다. |

# 7. 설계 원칙(Architecture Principles)

아래 항목은 요구사항 검토에서 이미 고정된 방향이며, 이후 대안 비교에서 임의로 뒤집지 않는 원칙이다. 반대로 아직 trade-off가 필요한 항목(예: single-agent 우선, partial processing 방식, capability placement)은 이 장에 두지 않고 Decision Point로 이동한다.

| ID | 원칙 |
| --- | --- |
| AP-01 | VIA 내부 Model의 출력은 VIA의 확정 상태로 바로 사용하지 않는다. Model은 intent, referent, parameter 또는 routing 후보를 제안할 수 있고, VIA는 canonical schema, 현재 conversation/task state, 실제 존재하는 context reference 및 허용 범위와 대조해 유효성을 확인한 뒤 VIA 상태에 반영한다. 이 원칙은 VIA 내부 상태와 context sharing에 적용되며, trusted Downstream Agent 내부의 reasoning이나 tool execution을 VIA가 다시 검증한다는 의미는 아니다. |
| AP-02 | 특정 OpenAI/Qwen/LLM SDK의 event·session·type이 VIA의 공통 interface 밖으로 새어나가지 않도록 provider-specific 구현을 adapter boundary 안에 격리한다. |
| AP-03 | 화면 지시 대상은 처리 완료 시점의 현재 pointer가 아니라 사용자가 요청을 말하는 동안 수집한 timestamped interaction evidence에 근거하며, turn 종료 후 그 evidence는 변경되지 않는다. |
| AP-04 | Downstream Agent와 Model에 제공하는 사용자 context는 목적 수행에 필요한 최소 범위로 제한하며 purpose-bound, time-bound, least-privilege 원칙을 적용한다. VIA는 Downstream Agent 내부 tool permission을 별도로 관리하지 않는다. |
| AP-05 | Downstream Agent는 domain reasoning, planning, tool selection 및 tool execution을 소유한다. VIA는 개별 tool call을 다시 승인하거나 실행하지 않고 Agent Harness Port를 통해 execution, progress, approval/clarification, cancel, status, follow-up, result를 교환한다. |
| AP-06 | Logical conversation, task state 및 context state는 voice/text modality와 WebRTC 같은 transport connection에서 독립적으로 관리한다. |
| AP-07 | Media connection, logical conversation/session, durable task는 서로 다른 lifecycle을 가질 수 있으므로 하나의 동일 state object로 취급하지 않는다. |

# 8. 품질 요구사항 체계(Quality Requirement Framework)

품질 요구사항은 사용자 E2E 경험과 VIA SW Architecture의 책임을 분리하여 정의한다. Product E2E Experience Objective는 통합 제품 전체에서 사용자가 느끼는 결과이고, Model Requirement는 Model 개발자에게 요구할 observable contract이며, VIA SW Quality Attribute는 VIA 구조가 직접 통제할 수 있는 품질이다.

| 품질 요구사항의 흐름<br>1) Product End-to-End Experience Objective(제품 E2E 경험 목표)를 먼저 정한다.<br>2) 그 목표를 만족하기 위해 Model/Voice Runtime에 필요한 latency·quality 요구를 배정한다.<br>3) 남은 budget과 구조적 책임을 VIA SW Quality Attribute로 정의한다.<br>4) Downstream Agent 내부 성능은 VIA QA로 책임지지 않지만, 동일 Downstream Agent를 직접 호출한 경우와 VIA를 경유한 경우를 비교하여 VIA가 추가한 overhead/quality degradation을 측정한다. |
| --- |

## 8.1 ISO/IEC 25010:2023 용어 사용

ISO/IEC 25010:2023은 ICT/software product quality를 9개 characteristic으로 분류하고 요구사항 명세·측정·평가에 사용할 수 있는 reference model을 제공한다. [REF-01]

| ISO/IEC 25010:2023 | 관련 sub-characteristic | VIA에서의 의미 |
| --- | --- | --- |
| Performance Efficiency | Time Behaviour, Resource Utilization, Capacity | 응답 시간, VIA가 추가하는 처리 지연, CPU/memory/model-token resource, concurrent task capacity |
| Compatibility | Co-existence, Interoperability | 다른 Windows app과의 resource 공존성, Downstream Agent/Tool 연동 |
| Interaction Capability | Operability 등 | 사용자가 VIA를 제어하고 상태를 이해할 수 있는 정도. 단, time 목표는 Performance Efficiency로 대표 분류 |
| Reliability | Faultlessness, Availability, Fault Tolerance, Recoverability | conversation/task recovery, dependency failure isolation |
| Security | Confidentiality, Integrity, Accountability 등 | context confidentiality, consent/context-sharing policy, audit |
| Maintainability | Modularity, Analysability, Modifiability, Testability | 변경 영향 범위, provider/model/Downstream Agent/Context Connector 연동 구조 |
| Flexibility | Adaptability, Scalability, Installability, Replaceability | Voice/Model runtime replacement와 workload 변화 대응 |

2023판에서는 Interaction Capability와 Flexibility가 공식 characteristic으로 사용되며, Compatibility의 co-existence는 같은 환경과 resource를 공유하면서 다른 product에 해로운 영향을 주지 않는 능력으로 정의된다. [REF-02]

## 8.2 p50/p95를 사용하는 이유

Latency는 평균만 보면 일부 사용자가 반복적으로 겪는 느린 tail을 숨길 수 있다. Google SRE는 latency처럼 분포가 한쪽으로 치우칠 수 있는 metric을 percentile로 보는 방식을 설명한다. 본 과제의 prototype에서는 p50을 typical experience, p95를 primary acceptance metric, p99를 diagnostic metric으로 사용한다. 200회 반복 시 p95 tail에는 약 10개 sample이 남지만 p99 tail은 약 2개뿐이어서 architecture prototype 단계에서는 p95가 더 안정적으로 비교 가능하다. [REF-05]

## 8.3 목표값의 근거 표기 방식

| 근거 유형 | 의미 |
| --- | --- |
| 외부 사용자 경험 기준 | Microsoft Windows interaction class 등 외부 자료가 제공하는 사용자 체감 기준 |
| 제품 경험 목표 | 외부 기준을 참고하여 VIA product가 선택한 E2E SLO |
| 상위 E2E 목표에서 역산 | Product 목표에서 Model/Downstream/Media 시간을 제외해 VIA budget을 산출 |
| Reference 구현 실측 | OpenAI Realtime, hosted Qwen, GPT-5.6 Terra, Reference Development Machine benchmark 결과 |
| 프로젝트 비교 기준 | 산업 표준이 없는 변경성·cost 최적화에서 architecture 대안의 의미 있는 개선 여부를 판단하기 위해 프로젝트가 명시적으로 선택한 상대 기준 |

# 9. Product End-to-End Experience Objectives (제품 E2E 경험 목표)

Microsoft Windows performance guidance는 AI model의 SLA가 아니다. VIA도 Windows에서 사용자가 직접 상호작용하는 application이므로, 사용자가 어느 정도의 delay를 ‘빠름/반응함/기다림’으로 인지하는지 정하는 UX anchor로 사용한다. Microsoft는 Fast 100–200ms, Typical/Interactive 300–500ms, Responsive 약 500ms–1s, Launch/Wait 1–3s 등의 class를 제시한다. [REF-03] [REF-04]

| ID | 목표 | 측정 범위 | 목표값 | 근거 | 논리 |
| --- | --- | --- | --- | --- | --- |
| PEO-01 | 첫 음성 반응 시간 | 사용자가 실제로 발화를 끝낸 시점 → VIA의 첫 audible response 시작 | p95 ≤ 1.0s | 외부 사용자 경험 기준 + 제품 경험 목표 | Windows Responsive class의 상한 1초를 voice-first product의 첫 반응 목표로 선택한다. 최종 답이 아니라 local answer 시작 또는 “확인해볼게요” 같은 acknowledgement도 가능하다. |
| PEO-02 | Barge-in 반응 시간 | 사용자가 assistant 발화 중 다시 말하기 시작한 실제 시점 → assistant audio가 멈춘 시점 | p95 ≤ 200ms | 외부 사용자 경험 기준 | Windows Fast interaction의 최대 200ms를 interruption UX의 upper bound로 사용한다. |
| PEO-03 | 위임 작업의 진행·결과 전달 | Downstream Agent가 즉시 끝나지 않는 작업을 수행하는 경우 | acknowledgement ≤1s; 첫 meaningful status/progress ≤3s; Downstream AgentResult 수신 → user-visible delivery 시작 p95 ≤500ms | 외부 사용자 경험 기준 + 제품 목표 | Final completion 자체는 Downstream Agent/Tool에 좌우되므로 universal seconds target을 두지 않는다. 대신 VIA가 상태를 늦게 전달하지 않도록 1s/3s/500ms 경계를 둔다. |
| PEO-04 | 작업 제어 응답 | 사용자가 기존 task cancel/pause/resume을 요청 → VIA가 control 요청을 접수/거절했음을 사용자에게 알림 | p95 ≤1.0s | 제품 경험 목표 | 실제 Downstream Agent/Tool stop completion은 해당 dependency capability에 따라 별도이다. |
| PEO-05 | 대화·작업 이어가기 | connection이 다시 연결되거나 voice↔text로 바뀐 후 VIA가 저장된 conversation/task state를 다시 사용할 준비가 되는 시간 | p95 ≤3.0s | 외부 사용자 경험 기준 | Windows Launch class 최대 3초를 interaction-ready upper bound로 사용한다. 자연어 ‘아까 그거’ 해석 정확도는 Model 평가로 분리한다. |
| PEO-06 | VIA 경유 시 작업 성공률 유지 | 동일 Downstream Agent와 동일 task set을 직접 adapter로 실행한 baseline과 VIA를 통해 실행한 결과 비교 | VIA-integrated success rate ≥ direct baseline - 3 percentage points | Reference 구현 실측 + 프로젝트 비교 기준 | 이 지표는 Downstream Agent 자체가 좋은지 평가하지 않는다. VIA의 intent/context 변환·routing·task 전달이 동일 Agent의 성공률을 크게 떨어뜨리지 않는지 확인한다. |
| PEO-07 | PC 사용성 유지 | VIA background/concurrent task가 실행되는 동안 대표 Windows foreground interaction | 해당 interaction에 지정한 Windows interaction class의 maximum을 넘지 않음 | 외부 사용자 경험 기준 + Reference Development Machine 실측 | target PC가 없어도 같은 PC에서 VIA OFF/ON을 비교하고 사용자-facing class boundary로 pass/fail을 정한다. |
| PEO-08 | 성공 task당 cloud inference 비용 | naive cloud-heavy baseline과 architecture 대안 비교 | median cloud inference cost/successful task ≤70% of naive baseline; PEO-01~07 위반 금지 | 프로젝트 비교 기준 | 30% 절감은 산업 표준이 아니라 model placement/context minimization/caching 등 구조 복잡성을 추가할 만한 ‘유의미한 효과’ 기준으로 명시한다. |

# 10. Model 및 Downstream Agent 요구사항

이 장의 Model은 OpenAI Realtime/Qwen3-Omni 계열 Voice Runtime과 VIA가 필요 시 사용하는 GPT-5.6 Terra 같은 LLM을 의미한다. ‘Agent’라는 표현은 모호하므로, 업무 실행 주체는 항상 Downstream Agent라고 쓴다.

## 10.1 Voice/Model 요구사항: 누가·언제·어디서·무엇을·왜 측정하는가

| ID | 요구사항 | 핵심 목표 |
| --- | --- | --- |
| MR-01 | Voice Runtime 첫 음성 packet 시간 | p95 ≤ 800ms |
| MR-02 | 사용자 발화 시작 감지 시간 | p95 ≤ 100ms |
| MR-03 | Streaming 음성 생성 속도 | Qualification: p95 < 1.0 / Engineering target: p95 ≤ 0.8 |
| MR-04 | 의미 해석·구조화 품질 | Overall ≥ 95% of reference / Critical slice ≥ reference - 3pp |
| MR-05 | Auxiliary LLM latency·quality | p95 TTFT ≤ 1.2 × Terra measured p95, MR-04 품질 조건 유지 |

### MR-01 — Voice Runtime 첫 음성 packet 시간

| 항목 | 내용 |
| --- | --- |
| 누가 책임지는가 | Voice Runtime 개발/제공 조직 |
| 언제/어디서 측정하는가 | 사용자가 한 발화를 끝낸 순간부터 Voice Runtime이 첫 audio response packet을 만들어 VIA에 전달할 때까지 |
| 무엇을 측정하는가 | acoustic end-of-speech → first generated audio packet |
| 정량 목표 | p95 ≤ 800ms |
| 논리의 흐름 | PEO-01의 ‘첫 audible response 1초 이내’ 목표를 만족하려면 VIA SW와 local media/output에 약 200ms 수준의 budget을 남겨야 한다. 따라서 Voice Runtime에는 endpointing과 첫 audio generation을 합쳐 800ms 이내라는 planning budget을 배정한다. 이 800ms는 OpenAI의 SLA가 아니라 VIA의 E2E 목표에서 역산한 값이다. OpenAI Realtime은 realtime audio와 VAD 기능을 제공하고, Qwen3-Omni technical report의 theoretical first-packet 234ms는 이 수준의 목표가 기술적으로 불가능하지 않음을 보여주는 feasibility reference이다. |
| 외부 근거 | REF-06, REF-07, REF-09 |

### MR-02 — 사용자 발화 시작 감지 시간

| 항목 | 내용 |
| --- | --- |
| 누가 책임지는가 | Voice Runtime/VAD 개발 조직 |
| 언제/어디서 측정하는가 | assistant가 말하는 중 사용자가 다시 말하기 시작했을 때, 실제 speech onset을 감지해 VIA에 speech_started event를 전달할 때까지 |
| 무엇을 측정하는가 | actual speech onset → internal speech_started event |
| 정량 목표 | p95 ≤ 100ms |
| 논리의 흐름 | PEO-02의 전체 barge-in 목표가 200ms이므로 detector/event path에 100ms, VIA output-cancel path에 100ms를 나누어 배정한다. 이 경로에서는 semantic reasoning 결과를 기다리지 않는다. OpenAI Realtime API도 VAD start event와 response interruption을 별도로 제공한다. |
| 외부 근거 | REF-04, REF-07 |

### MR-03 — Streaming 음성 생성 속도

| 항목 | 내용 |
| --- | --- |
| 누가 책임지는가 | Voice generation Model 개발 조직 |
| 언제/어디서 측정하는가 | VIA가 audio response를 streaming playback하는 동안 |
| 무엇을 측정하는가 | RTF(Real Time Factor)=generation time / generated audio duration |
| 정량 목표 | Qualification: p95 < 1.0 / Engineering target: p95 ≤ 0.8 |
| 논리의 흐름 | RTF가 1 이상이면 평균적으로 1초 분량 audio를 만드는 데 1초 이상이 필요하므로 지속적인 realtime playback에 여유가 없다. 0.8은 network/jitter/scheduling을 위한 20% headroom을 둔 project engineering target이다. Qwen3-Omni technical report는 theoretical RTF 약 0.47을 보고한다. |
| 외부 근거 | REF-09 |

### MR-04 — 의미 해석·구조화 품질

| 항목 | 내용 |
| --- | --- |
| 누가 책임지는가 | Model 개발 조직 |
| 언제/어디서 측정하는가 | Intent, referent, compound request, structured output을 VIA가 사용할 때 |
| 무엇을 측정하는가 | VIA-specific Model Evaluation Set에서 reference stack 대비 score |
| 정량 목표 | Overall ≥ 95% of reference / Critical slice ≥ reference - 3pp |
| 논리의 흐름 | VIA는 Model 자체를 개발하지 않으므로 ‘모든 Model은 accuracy 95% 이상’ 같은 임의의 절대 기준을 두지 않는다. 먼저 OpenAI/Qwen/GPT reference stack을 동일한 test set에서 측정한 뒤, 교체 candidate가 reference 대비 얼마나 품질을 떨어뜨리는지를 제한한다. 95% relative와 -3 percentage points는 provider 교체를 허용하면서도 체감되는 degradation을 작게 유지하기 위한 project acceptance rule이다. |
| 외부 근거 | REF-06, REF-08, REF-09 |

### MR-05 — Auxiliary LLM latency·quality

| 항목 | 내용 |
| --- | --- |
| 누가 책임지는가 | Text/Reasoning Model 개발·운영 조직 |
| 언제/어디서 측정하는가 | VIA 설계 대안이 first-reaction 또는 execution-control critical path에 별도 LLM call을 추가하려는 경우 |
| 무엇을 측정하는가 | Candidate p95 TTFT와 task-specific quality/cost를 GPT-5.6 Terra reference와 비교 |
| 정량 목표 | p95 TTFT ≤ 1.2 × Terra measured p95, MR-04 품질 조건 유지 |
| 논리의 흐름 | GPT-5.6 Terra는 initial reference이고 fixed constraint가 아니다. Gemini Flash/local model 등 대안은 동일 network와 prompt에서 Terra baseline 대비 TTFT가 20% 이상 악화되지 않도록 비교한다. Absolute TTFT는 API/network condition에 크게 좌우되므로 benchmark로 실측한다. |
| 외부 근거 | REF-08 |

## 10.2 Model Invocation Profile 관리

VIA 내부 Model 호출은 prompt 문자열을 코드 곳곳에 흩어두지 않는다. Model Gateway는 versioned Model Invocation Profile을 통해 system instruction/prompt template, model/provider ID, generation parameter, structured-output schema, token budget, timeout/retry 설정을 함께 관리한다. Observability에는 invocation마다 profile_version, model_id, provider, schema_version을 남겨 prompt/config 변경과 QA-09~11 정확도 변화를 추적한다.

## 10.3 Downstream Agent 요구사항과 성능 경계

Downstream Agent 자체의 reasoning latency와 task success는 VIA SW Architecture QA의 직접 책임이 아니다. VIA는 같은 Downstream Agent를 직접 사용할 때와 VIA를 경유할 때를 비교하여 ‘VIA가 추가한 overhead와 품질 degradation’을 측정한다.

| 항목 | 요구/원칙 |
| --- | --- |
| Execution contract | execute/stream/status/cancel/follow-up 식별자를 공통 Port로 제공하거나 adapter가 이를 canonical event로 변환할 수 있어야 한다. |
| Capability declaration | streaming, cancel, status, checkpoint/thread continuation, user-approval interaction, required context class, latency/cost class, exclusive resource requirement 등을 Agent Registry에 선언해야 한다. 내부 tool 목록의 공개는 VIA contract의 필수 조건이 아니다. |
| Follow-up continuity | 해당 Downstream Agent가 thread/session continuation을 제공하면 VIA는 기존 task와 Downstream Agent execution/thread를 저장하고 후속 요청에 재사용할 수 있어야 한다. |
| Qualification profile | Downstream Agent onboarding 시 reference task set으로 observed latency, task success, streaming/cancel/status support를 기록한다. 이것은 VIA Critical QA가 아니라 routing/selection input이다. |
| Internal specialized path | 자체 Downstream Agent에만 가능한 context streaming, state sharing, low-latency fast path가 Product E2E를 개선할 수 있다면 generic Agent Harness Port 대비 specialization trade-off를 Decision Point에서 평가한다. 특화 경로도 Agent 내부 tool execution을 VIA로 이전한다는 뜻은 아니다. |

# 11. VIA SW Quality Attributes

## 11.1 중요도·난이도 점수 기준

| 구간 | 채점 기준 |
| --- | --- |
| 중요도 1–4 (Low) | 일부 기능에만 영향을 주고 실패 시 우회가 쉬우며 VIA의 핵심 사용자 가치/신뢰를 직접 좌우하지 않음 |
| 중요도 5–7 (Mid) | 주요 flow 또는 운영 품질에 영향을 주지만 사용자 영향이 제한적이거나 대체 수단이 존재 |
| 중요도 8–10 (High) | Voice-first 제품의 핵심 가치, 사용자 신뢰, 비용 또는 CTO-level 전략 요구에 직접 연결되고 실패 영향이 큼 |
| 난이도 1–4 (Low) | 한두 component의 국소 변경으로 해결 가능하고 알려진 pattern으로 trade-off가 적음 |
| 난이도 5–7 (Mid) | 여러 interface/component가 연관되지만 책임 경계가 명확하고 일반적인 pattern으로 해결 가능 |
| 난이도 8–10 (High) | cross-cutting state/concurrency/security/provider boundary를 건드리며 서로 다른 QA 간 trade-off가 크고 나중에 구조 변경 비용이 큼 |

## 11.2 독립 QA 완전성 점검 결과

ISO/IEC 25010:2023 전체 characteristic과 NIST AI RMF의 accuracy/evaluation guidance를 기준으로 독립적으로 다시 검토했다. 기존 QA에는 performance, concurrency/co-existence, reliability, changeability, privacy, cost, observability는 있었지만 VIA의 핵심 가치인 Functional Correctness(사용자의 의도·화면 지시·routing 결과가 정확한가)가 빠져 있었다. 이에 QA-09~QA-11을 추가한다. NIST는 AI accuracy를 realistic test set과 false-positive/false-negative 같은 measure로 평가하고 segment별 결과와 test methodology를 문서화할 것을 권고한다. 별도 Safety/Availability/Interaction Capability top-level QA는 현재 FR/Constraint/PEO 및 QA-03/04/06/08과 중복되므로 추가하지 않는다.

| 검토 관점 | 판정 |
| --- | --- |
| Functional Suitability / Correctness | 누락 → QA-09~11 추가 |
| Performance Efficiency | QA-01/02/07에서 다룸 |
| Compatibility / Interaction Capability | QA-02/05 및 Product E2E/FR에서 다룸 |
| Reliability / Availability | QA-03/04와 task/session lifecycle에서 다룸; 별도 service availability SLO는 운영조건 부재로 보류 |
| Security / Safety | Context confidentiality는 QA-06, audit는 QA-08, trusted Downstream Agent security boundary는 Constraint로 다룸 |
| Maintainability / Flexibility | QA-05로 통합 |

## 11.3 QA 상세

| ID | QA 이름 | ISO/IEC 25010:2023 | 중요도 | 난이도 | Class |
| --- | --- | --- | --- | --- | --- |
| QA-01 | VIA 소프트웨어 처리 지연 | Performance Efficiency / Time Behaviour | 10 (모든 interaction의 체감 latency에 직접 영향.) | 9 (여러 core component와 병렬화/capability placement를 동시에 건드림.) | H,H |
| QA-02 | 동시 작업 처리와 PC 공존성 | Performance Efficiency / Capacity & Resource Utilization; Compatibility / Co-existence | 9 (Concurrent task는 핵심 Use Case이며 PC foreground 경험에 직접 영향.) | 9 (Scheduling/state isolation/resource arbitration/backpressure를 함께 설계.) | H,H |
| QA-03 | 대화·작업 상태 복구 | Reliability / Recoverability | 10 (State loss는 장시간 task 제품 신뢰를 직접 훼손.) | 9 (Durable state/checkpoint/reconcile/multi-process lifecycle 복잡.) | H,H |
| QA-04 | 의존성 장애 격리 | Reliability / Fault Tolerance | 7 (빈도는 낮지만 장애 시 신뢰 영향 큼.) | 8 (Fallback이 privacy/cost/predictability와 trade-off.) | M,H |
| QA-05 | 변경 및 연동 용이성 | Maintainability / Modifiability + Flexibility / Replaceability + Compatibility / Interoperability | 10 (교체 가능성은 과제의 명시적 전략 목표.) | 9 (Stable contract와 specialization trade-off가 전체 architecture에 영향.) | H,H |
| QA-06 | 외부 Context 노출 최소화 | Security / Confidentiality | 9 (Privacy는 사용자 신뢰에 직접 영향.) | 8 (Minimization과 task quality/context richness trade-off 큼.) | H,H |
| QA-07 | Model/Token 비용 효율 | Performance Efficiency / Resource Utilization | 9 (사업/사용자 비용과 architecture placement에 직접 연결.) | 8 (여러 component의 latency/quality/cost trade-off를 동시 고려.) | H,H |
| QA-08 | 관측성과 감사의 부하 | Maintainability / Analysability + Security / Accountability | 7 (운영/보안에는 중요하지만 직접 사용자 핵심 Driver는 아님.) | 6 (Known async/sampling patterns로 비교적 국소 해결 가능.) | M,M |
| QA-09 | 화면 지시 대상 연결 정확도 | Functional Suitability / Functional Correctness | 10 (VIA 핵심 차별점인 화면 지시 이해에 직접 영향하며 잘못된 source/destination은 task 자체를 바꿈.) | 9 (Clock normalization, event capture, timeline storage, gesture candidate, ASR timing, referent binding이 함께 영향.) | H,H |
| QA-10 | 사용자 요청 및 기존/신규 Task 연계 해석 정확도 | Functional Suitability / Functional Correctness + Interaction Capability / Operability | 10 (사용자 목표를 Downstream Agent가 실행 가능한 정확한 요청으로 만드는 것이 VIA의 핵심 존재 목적.) | 9 (Conversation state, context retrieval, staged refinement, validator, clarification policy, prompt/profile이 함께 영향.) | H,H |
| QA-11 | Downstream Agent 선택 정확도 | Functional Suitability / Functional Correctness | 9 (잘못된 routing은 이후 Agent 성능과 무관하게 task 실패/지연을 발생.) | 8 (Capability contract, candidate filter, semantic scoring, health/state, routing cache 구조가 영향.) | H,H |

### QA-01 — VIA 소프트웨어 처리 지연

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Performance Efficiency / Time Behaviour |
| Stimulus / 무엇이 발생하는가 | 사용자 request, Voice Runtime event, Downstream Agent event가 VIA에 들어왔을 때 VIA-owned context/routing/policy/task/state/IPC/media control이 추가하는 latency. |
| Environment / 어떤 조건에서 보는가 | 동일 Voice Runtime과 deterministic Downstream Agent stub을 고정한 정상 benchmark. |
| Scope / 어디서부터 어디까지 측정하는가 | Model/Downstream Agent 내부 시간은 제외한다. 병렬 단계에서는 전체 응답 시점을 결정하는 가장 긴 의존 단계의 연속 구간(critical path)에 실제로 포함된 VIA 시간만 계산한다. |
| 정량 목표 | First-reaction VIA+local-media p95 ≤200ms; speech_started→audio stop commit p95 ≤100ms; canonical request ready→Agent dispatch p95 ≤200ms; Agent event→user delivery 시작 p95 ≤500ms. |
| 목표값의 근거 | PEO-01/02/03에서 역산. |
| 중요도 | 10점 — 모든 interaction의 체감 latency에 직접 영향. |
| 난이도 | 9점 — 여러 core component와 병렬화/capability placement를 동시에 건드림. |
| 우선순위 Class | H,H |

### QA-02 — 동시 작업 처리와 PC 공존성

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Performance Efficiency / Capacity & Resource Utilization; Compatibility / Co-existence |
| Stimulus / 무엇이 발생하는가 | 여러 background task가 있어도 새 voice/text interaction과 일반 Windows foreground interaction이 느려지지 않도록 한다. |
| Environment / 어떤 조건에서 보는가 | RDM-1, deterministic stub, concurrency 1/2/4/8. |
| Scope / 어디서부터 어디까지 측정하는가 | 실제 Downstream Agent compute는 VIA QA에서 제외하고 VIA scheduling/state/resource overhead만 측정한다. |
| 정량 목표 | Concurrency 4에서 PEO-01 유지; QA-01 first-reaction overhead p95 single-task 대비 악화 <20%; foreground operation은 지정 Windows interaction class maximum 이내. |
| 목표값의 근거 | Product capacity point + RDM 실측. |
| 중요도 | 9점 — Concurrent task는 핵심 Use Case이며 PC foreground 경험에 직접 영향. |
| 난이도 | 9점 — Scheduling/state isolation/resource arbitration/backpressure를 함께 설계. |
| 우선순위 Class | H,H |

### QA-03 — 대화·작업 상태 복구

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Reliability / Recoverability |
| Stimulus / 무엇이 발생하는가 | Connection/process interruption 뒤 conversation과 durable task state를 복구한다. |
| Environment / 어떤 조건에서 보는가 | Explicit conversation/task ID와 persisted state가 있는 failure-injection 환경. |
| Scope / 어디서부터 어디까지 측정하는가 | RTO(Recovery Time Objective)는 장애 후 다시 해당 conversation/task를 사용할 수 있기까지 허용하는 최대 시간이다. |
| 정량 목표 | Conversation/session restore p95 ≤3s; interactive task RTO p95 ≤3s; background task RTO p95 ≤10s. |
| 목표값의 근거 | Windows interaction boundary + project recovery budget. |
| 중요도 | 10점 — State loss는 장시간 task 제품 신뢰를 직접 훼손. |
| 난이도 | 9점 — Durable state/checkpoint/reconcile/multi-process lifecycle 복잡. |
| 우선순위 Class | H,H |

### QA-04 — 의존성 장애 격리

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Reliability / Fault Tolerance |
| Stimulus / 무엇이 발생하는가 | Model, Downstream Agent, Context Connector 등 한 dependency 장애가 무관한 interaction/task로 확산되지 않도록 한다. |
| Environment / 어떤 조건에서 보는가 | Failure Injection Suite에서 하나의 dependency만 실패. |
| Scope / 어디서부터 어디까지 측정하는가 | Failure detection, 영향 범위, retry/defer/alternate/user-confirm/controlled-failure choice와 feedback을 측정. |
| 정량 목표 | 사용자 상태 p95 ≤3s; 무관한 task가 같은 failure cause로 함께 종료되면 correctness failure. |
| 목표값의 근거 | Windows Wait/Launch upper bound. |
| 중요도 | 7점 — 빈도는 낮지만 장애 시 신뢰 영향 큼. |
| 난이도 | 8점 — Fallback이 privacy/cost/predictability와 trade-off. |
| 우선순위 Class | M,H |

### QA-05 — 변경 및 연동 용이성

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Maintainability / Modifiability + Flexibility / Replaceability + Compatibility / Interoperability |
| Stimulus / 무엇이 발생하는가 | Voice Runtime, Model endpoint, Downstream Agent, Context/Input Adapter 변경이 VIA core 전체로 확산되지 않도록 한다. |
| Environment / 어떤 조건에서 보는가 | E01~E04 evolution exercise와 direct-coupled reference 비교. |
| Scope / 어디서부터 어디까지 측정하는가 | CPR(Change Propagation Ratio)=VIA architecture에서 수정한 기존 production component 수 / direct-coupled reference에서 수정한 기존 component 수. |
| 정량 목표 | 4개 exercise median CPR ≤0.5; standard-compatible change는 versioned canonical contract breaking change 없이 완료. |
| 목표값의 근거 | Project comparison threshold. |
| 중요도 | 10점 — 교체 가능성은 과제의 명시적 전략 목표. |
| 난이도 | 9점 — Stable contract와 specialization trade-off가 전체 architecture에 영향. |
| 우선순위 Class | H,H |

### QA-06 — 외부 Context 노출 최소화

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Security / Confidentiality |
| Stimulus / 무엇이 발생하는가 | External Model/Downstream Agent에 context sharing이 허용된 경우에도 필요한 것보다 많은 screen/file/mail data를 전달하지 않는다. |
| Environment / 어떤 조건에서 보는가 | 동일 request/Agent, naive full-context baseline 비교. |
| Scope / 어디서부터 어디까지 측정하는가 | 성공 task당 external context tokens/bytes와 task-success degradation을 함께 측정. |
| 정량 목표 | Median external context volume ≤50% of naive baseline; PEO-06 success 추가 degradation ≤3pp. |
| 목표값의 근거 | Project significance threshold. |
| 중요도 | 9점 — Privacy는 사용자 신뢰에 직접 영향. |
| 난이도 | 8점 — Minimization과 task quality/context richness trade-off 큼. |
| 우선순위 Class | H,H |

### QA-07 — Model/Token 비용 효율

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Performance Efficiency / Resource Utilization |
| Stimulus / 무엇이 발생하는가 | Model placement, local/cloud selection, context size, caching/call count가 cloud inference cost에 미치는 영향을 줄인다. |
| Environment / 어떤 조건에서 보는가 | Naive Baseline B0 vs architecture alternatives. |
| Scope / 어디서부터 어디까지 측정하는가 | 성공 task당 text/audio tokens, model call count, cached token, paid API cost. |
| 정량 목표 | Median cloud inference cost/successful task ≤70% of B0; PEO-01~07 및 MR-04 조건 위반 금지. |
| 목표값의 근거 | Project significance threshold. |
| 중요도 | 9점 — 사업/사용자 비용과 architecture placement에 직접 연결. |
| 난이도 | 8점 — 여러 component의 latency/quality/cost trade-off를 동시 고려. |
| 우선순위 Class | H,H |

### QA-08 — 관측성과 감사의 부하

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Maintainability / Analysability + Security / Accountability |
| Stimulus / 무엇이 발생하는가 | Trace/audit가 필요한 chain을 추적하면서 latency/privacy/storage를 과도하게 악화시키지 않는다. |
| Environment / 어떤 조건에서 보는가 | Single-task 및 concurrency 4에서 telemetry OFF/ON 비교. |
| Scope / 어디서부터 어디까지 측정하는가 | Critical chain trace availability와 QA-01 latency increase를 측정. |
| 정량 목표 | Critical event searchable p95 ≤10s; telemetry로 QA-01 primary latency metric 증가 <5%. |
| 목표값의 근거 | Project operations overhead budget. |
| 중요도 | 7점 — 운영/보안에는 중요하지만 직접 사용자 핵심 Driver는 아님. |
| 난이도 | 6점 — Known async/sampling patterns로 비교적 국소 해결 가능. |
| 우선순위 Class | M,M |

### QA-09 — 화면 지시 대상 연결 정확도

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Functional Suitability / Functional Correctness |
| Stimulus / 무엇이 발생하는가 | 한 발화 안의 “여기/얘네들/이쪽” 같은 지시 표현을 해당 시점의 pointer/gesture/UI evidence와 정확히 연결한다. |
| Environment / 어떤 조건에서 보는가 | 고정된 reference Voice/ASR/semantic Model을 사용하고 Context architecture만 바꾸는 60-episode Interaction Grounding Suite. |
| Scope / 어디서부터 어디까지 측정하는가 | 각 speech cue의 timestamp와 ground-truth target/region/order를 사람이 annotation한다. Timeline event timestamp 정렬, referent binding 결과와 capture latency를 측정한다. |
| 정량 목표 | Multi-Referent Exact Match ≥ reference best-known pipeline의 95%; single-snapshot baseline 대비 +10pp 이상 개선. OS interaction event→timeline append p95 ≤50ms. |
| 목표값의 근거 | Reference-relative acceptance + architecture significance threshold. 50ms는 100–200ms Fast interaction budget의 절반을 local evidence capture에 배정한 project budget. |
| 중요도 | 10점 — VIA 핵심 차별점인 화면 지시 이해에 직접 영향하며 잘못된 source/destination은 task 자체를 바꿈. |
| 난이도 | 9점 — Clock normalization, event capture, timeline storage, gesture candidate, ASR timing, referent binding이 함께 영향. |
| 우선순위 Class | H,H |

### QA-10 — 사용자 요청 및 기존/신규 Task 연계 해석 정확도

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Functional Suitability / Functional Correctness + Interaction Capability / Operability |
| Stimulus / 무엇이 발생하는가 | 자연스럽고 모호한 발화를 올바른 canonical intent로 정제하고, 기존 task의 follow-up인지 신규 task인지 정확히 구분한다. |
| Environment / 어떤 조건에서 보는가 | 고정 reference Model을 사용한 120+ natural utterance suite. Explicit/underspecified/compound/self-correct/follow-up-vs-new slice를 분리한다. |
| Scope / 어디서부터 어디까지 측정하는가 | Intent/action/required-parameter exact match, unnecessary clarification, follow-up association을 측정한다. False Continuation(신규 task를 기존에 잘못 연결)과 False Split(기존 follow-up을 신규로 분리)을 별도 기록한다. |
| 정량 목표 | Overall score ≥ reference best-known pipeline의 95%; ambiguous/contextual slice는 transcript-only single-model baseline 대비 +5pp 이상 개선; False Continuation은 reference pipeline보다 악화되지 않아야 한다. |
| 목표값의 근거 | Reference-relative acceptance + architecture significance threshold. NIST 권고에 따라 realistic conditions와 segment별 결과를 사용한다. |
| 중요도 | 10점 — 사용자 목표를 Downstream Agent가 실행 가능한 정확한 요청으로 만드는 것이 VIA의 핵심 존재 목적. |
| 난이도 | 9점 — Conversation state, context retrieval, staged refinement, validator, clarification policy, prompt/profile이 함께 영향. |
| 우선순위 Class | H,H |

### QA-11 — Downstream Agent 선택 정확도

| 항목 | 내용 |
| --- | --- |
| ISO/IEC 25010:2023 분류 | Functional Suitability / Functional Correctness |
| Stimulus / 무엇이 발생하는가 | 정제된 요청을 실제로 처리할 수 있는 적절한 Downstream Agent에 전달한다. |
| Environment / 어떤 조건에서 보는가 | Agent Catalog를 고정한 80-case Routing Suite. 각 case에 acceptable agent set, ineligible agent set, capability overlap을 annotation한다. |
| Scope / 어디서부터 어디까지 측정하는가 | Acceptable Route Rate와 unnecessary reroute/handoff를 측정한다. Deterministic policy-ineligible route는 FR correctness 위반으로 별도 처리한다. |
| 정량 목표 | Acceptable Route Rate ≥ reference best-known router의 95%; LLM-only routing baseline 대비 unnecessary reroute/handoff ≥20% 감소. |
| 목표값의 근거 | Reference-relative acceptance + architecture significance threshold. 정답 Agent가 하나가 아닐 수 있으므로 acceptable set으로 평가한다. |
| 중요도 | 9점 — 잘못된 routing은 이후 Agent 성능과 무관하게 task 실패/지연을 발생. |
| 난이도 | 8점 — Capability contract, candidate filter, semantic scoring, health/state, routing cache 구조가 영향. |
| 우선순위 Class | H,H |

## 11.4 Critical QA 후보와 예상 Trade-off

| Critical QA | 주요 Trade-off |
| --- | --- |
| QA-01 VIA 소프트웨어 처리 지연 | Latency ↔ context/policy/observability; latency ↔ generic delegation. Partial processing/local fast path로 개선 가능하지만 complexity/coupling 증가. |
| QA-02 동시 작업 처리와 PC 공존성 | Concurrency ↔ foreground latency/resource; fairness ↔ priority; aggressive parallelism ↔ exclusive-resource conflict. |
| QA-03 대화·작업 상태 복구 | 짧은 RTO ↔ persistence/checkpoint overhead; 정확한 recovery ↔ storage/complexity. |
| QA-05 변경 및 연동 용이성 | Generic contract ↔ specialized fast path/performance; abstraction stability ↔ provider-specific capability 활용. |
| QA-06 외부 Context 노출 최소화 | Privacy/cost ↓ ↔ Downstream Agent context richness/task success; secure indirection ↔ latency. |
| QA-07 Model/Token 비용 효율 | Cost ↓ ↔ Model quality/latency; local model ↔ PC resource; caching ↔ memory/privacy/freshness. |
| QA-09 화면 지시 대상 연결 정확도 | Accuracy ↑ ↔ event capture/gesture/model complexity와 latency/resource ↑; richer timeline ↔ privacy/storage ↑. |
| QA-10 요청·Task 연계 해석 정확도 | Accuracy ↑ ↔ additional context/model pass/clarification으로 QA-01 latency와 QA-07 cost 악화 가능. |
| QA-11 Downstream Agent 선택 정확도 | Routing accuracy ↑ ↔ richer capability metadata/semantic reranking으로 latency·changeability 비용 증가 가능. |

# 12. Measurement / Evaluation Plan

## 12.1 Reference Development Machine (RDM-1)

실제 target Samsung PC를 확보할 수 없으므로 현재 보유 PC 한 대를 RDM-1(Reference Development Machine)로 고정한다. CPU%나 RAM MB의 절대값을 제품 SLA처럼 주장하지 않고, 동일 장비·동일 조건에서 VIA OFF/ON 및 concurrency 단계의 상대 변화를 측정한다.

| 항목 | 방법 |
| --- | --- |
| 기록 | CPU model/cores, RAM, GPU/NPU, storage, Windows build, power mode, network path |
| 통제 | 동일 power mode와 network, background update 최소화, cold/warm run 구분 |
| Latency run | 최소 200회 반복하여 p50/p95 보고 |
| Resource | Windows Performance Recorder/Analyzer + VIA trace로 CPU time, working set, GPU/NPU, network, energy 가능 시 기록 |
| Pass/Fail | 대표 foreground Windows operation이 지정 interaction class maximum을 넘는지와 VIA QA metric 유지 여부로 판단 |

## 12.2 Request Suite 카테고리와 충분성 기준

| 카테고리 | Core Scenario 수 | 포함 내용 | Coverage |
| --- | --- | --- | --- |
| A. 로컬 음성 대화·즉답 | 4 | Downstream Agent 없이 처리하는 음성 대화, 일반 지식, 반복/속도조절, barge-in | UC-01/08, QA-01 |
| B. 화면 Context·Pointer 지시 | 6 | single pointer, source→destination, group/circle, self-correction, selected text, multi-point timestamp-alignment | UC-02, FR-03/04, QA-09 |
| C. Downstream Agent 위임·PC/서비스 작업 | 6 | navigation, local retrieval, Gmail+reasoning, external search, sandbox constraint/approval, routing-overlap case | UC-04/05, QA-11 |
| D. 대화·Task 연속성 | 6 | task status, same-agent follow-up, mixed voice/text, reconnect, ambiguous follow-up, clearly-new task after prior context | UC-09/13/15, QA-10 |
| E. 동시 Task·복구·충돌 | 3 | independent concurrent tasks, exclusive UI conflict, process/dependency failure | UC-07/16, QA-02/03/04 |
| F. Natural Long / Compound Speech | 3 | 6–15초 filler/hesitation/self-correction, multi-intent, speculative-processing candidate | UC-11/12, ADP partial processing |

Architecture Request Suite는 총 28개 core scenario로 구성한다. 이 수는 model generalization을 통계적으로 증명하기 위한 dataset 크기가 아니라, 서로 다른 architecture path와 failure/state/concurrency boundary를 빠짐없이 실행하기 위한 구조 검증용이다. 각 Critical FR과 H,H QA는 최소 2개의 scenario(정상 + boundary/negative 또는 stress)에 연결되도록 coverage matrix를 확인한다. Model semantic quality(MR-04)는 별도의 120개 이상 utterance dataset으로 평가하며, 6개 카테고리당 최소 20개 natural utterance를 구성한다.

## 12.3 28개 Architecture Request Suite

| ID | Scenario | 대표 발화/Setup | 검증 목적 |
| --- | --- | --- | --- |
| A01 | 로컬 즉답 | “아, 방금 말한 거 다시 한번만 조금 천천히 말해줘.” | local response |
| A02 | 로컬 일반 지식 응답 | “그러니까… 태양이 지구보다 대충 얼마나 큰 거야? 최신 정보 말고 그냥 일반적으로.” | S2S/model self-knowledge local response |
| A03 | 대화 출력 제어 | “잠깐만, 그만 말해.” | stop output |
| A04 | 사용자 끼어들기 | assistant 5초 발화 중 2초 시점에 새 사용자 발화 | barge-in latency |
| B01 | 한 지점 가리키기 | pointer가 버튼 위에 있을 때 “이거 열어줘.” | single referent |
| B02 | 출발 위치→도착 위치 가리키기 | source folder를 가리키며 “여기 파일을”, destination으로 옮기며 “여기로 옮겨줘.” | two timed referents |
| B03 | 여러 대상 그룹→도착 위치 가리키기 | 여러 UI object 주변을 원으로 가리키며 “얘네들을”, 다른 곳을 가리키며 “이쪽으로 옮겨줘.” | gesture/group/region |
| B04 | Pointer 지시 중 자기 수정 | 첫 대상을 가리키다가 “아 이거 말고”라고 하고 다른 대상을 가리켜 “이걸 열어줘.” | temporal correction |
| B05 | 선택 영역 Context | 문서 text를 선택하고 “이 부분만 요약해줘.” | selection grounding |
| C01 | 빠른 앱/페이지 열기 | “그… 네이버 증권 있잖아. 그냥 거기 메인 화면 좀 열어줘.” | downstream quick action |
| C02 | 최근 PPT 찾기 | “문서 폴더에 내가 어제였나 오늘 아침이었나 마지막으로 만진 PPT 하나 있을 텐데 그거 찾아서 열어줘.” | retrieval/ambiguity |
| C03 | Gmail 조회·요약 | “Gmail에서 최근 메일 중에 김대리가 보낸 거 있으면 열어보고, 길면 핵심만 먼저 요약해줘.” | personal context + reasoning |
| C04 | 외부 검색 | “유튜브에서 손흥민 경기 영상 좀 찾아줘. 풀경기 말고 최근 경기 하이라이트로.” | external search |
| C05 | Sandbox transaction | “최신 갤럭시 버즈 맞는지 확인하고 장바구니까지만 담아줘. 결제는 하지 마.” | 사용자 constraint/approval interaction이 Downstream Agent에 정확히 전달되는지 확인 |
| D01 | 작업 상태 조회 | long task 중 “아까 시킨 분석 그거 지금 어디까지 됐어?” | task reference/status |
| D02 | 동일 Downstream Agent 후속 요청 | 기존 coding/downstream task 후 “거기에 테스트도 추가해줘.” | same agent/thread continuation |
| D03 | 음성·텍스트 혼합 | Voice “최근 PPT 몇 개 찾아줘.” → Text “두 번째 거” → Voice “응, 그거 열어줘.” | modality-independent continuity |
| D04 | 재연결 | connection drop 후 재접속해 이전 task status 확인 | state restore |
| E01 | 서로 독립적인 동시 작업 | 분석 task 실행 중 음악 재생 + Gmail 요약 요청 | task isolation |
| E02 | 독점 UI resource 충돌 | 그림판 automation 중 browser UI automation 시작 | resource arbitration |
| E03 | 장애·복구 | long task 중 Task Manager restart 또는 downstream timeout | recovery/fault isolation |
| F01 | 복수 요청 발화 | “내일 첫 회의 자료 좀 찾아주고, 아 아니 그 전에 최근 김대리 메일부터 열어줘. 그리고 음악도 하나 틀어줘.” | decomposition/order |
| F02 | 긴 자연 발화 | 10–15초 filler/hesitation/source omission을 포함하고 후반에 target/action 확정 | natural spoken processing |
| F03 | 사전 검색 후보 발화 | 초반에 ‘내일 첫 회의’가 나오고 후반에 ‘관련 PPT 찾아줘’가 확정되는 8–15초 발화 | partial ASR prefetch evaluation |
| B06 | 다중 위치 timestamp 정렬 | 12–15초 발화 중 3개의 “여기/이쪽” cue와 pointer 위치를 ground truth timestamp로 annotation | QA-09 temporal alignment/capture latency |
| C06 | Agent capability 중첩 routing | 동일 request를 수행 가능한 Agent 2개와 수행 불가능하지만 semantic similarity가 높은 Agent 1개를 Catalog에 함께 등록 | QA-11 acceptable route / reroute |
| D05 | 모호한 기존 Task 연계 | 이전 분석 task가 여러 개 존재하는 상태에서 “그거 지난달 것도 포함해줘” | QA-10 follow-up association / clarification |
| D06 | 이전 맥락 이후 명확한 신규 Task | 기존 coding task 대화 직후 “음악 좀 틀어줘” | QA-10 false continuation 방지 |

## 12.4 Accuracy Evaluation Sets

Accuracy QA는 Model 자체 benchmark와 VIA architecture 효과를 분리하기 위해 reference Model/Voice stack을 고정하고 VIA 구조만 바꾸어 비교한다. NIST AI RMF는 accuracy를 realistic test set에서 평가하고 false-positive/false-negative 및 segment별 결과를 문서화할 것을 권고한다.

| Suite | 규모 | 구성 | 연결 QA |
| --- | --- | --- | --- |
| Interaction Grounding Suite | 60 episodes | single pointer 20 / source-destination 20 / group·self-correction·multi-point 20 | QA-09 |
| Intent & Task Association Suite | 120+ utterances | explicit 30 / underspecified 30 / compound·self-correction 30 / follow-up-vs-new 30+ | QA-10 |
| Downstream Agent Routing Suite | 80 cases | single-capability, overlapping capability, unavailable/degraded agent, semantic-confuser cases | QA-11 |

## 12.5 Downstream Agent Stub과 실제 Agent의 분리

| Stub | 동작 | 용도 |
| --- | --- | --- |
| D0 | 200ms 후 result | VIA dispatch/relay overhead 측정 |
| D1 | 2s 후 result | short delegated task |
| D2 | 10s 동안 2/5/8s progress 후 result | progress relay / long interactive |
| D3 | 30s 동안 5s마다 progress | background task / task status |
| D-Cancel | cancel 전까지 실행 | control propagation |
| D-Fail | 지정 시점 timeout/error | fault containment |

QA-01/02/03/04처럼 VIA 자체를 평가할 때는 deterministic stub을 사용하여 Downstream Agent의 processing power와 latency를 제거한다. Product E2E와 PEO-06은 실제 reference Downstream Agent를 고정하고 direct baseline과 VIA-mediated path를 비교한다.

## 12.6 Voice/Model Benchmark

| Reference | 대상 | 측정 |
| --- | --- | --- |
| OpenAI Realtime | GPT-Realtime-2.1, WebRTC 우선 | acoustic EOS, speech_started/stopped, first audio event, local playback timestamp를 수집하여 p50/p95 측정 |
| Hosted Qwen | Alibaba Cloud qwen3-omni-flash-realtime | 동일한 acoustic EOS/first audio/playback metric으로 managed Qwen-family 실측 |
| Qwen3-Omni-30B-A3B | Technical report; GPU 가능 시 self-host 추가 | 234ms theoretical first-packet 및 RTF reference. Hosted Flash 결과를 30B-A3B 실측이라고 표기하지 않음 |
| Auxiliary LLM | GPT-5.6 Terra initial reference | VIA-specific prompt에서 TTFT, tokens, task quality, cost 측정; Gemini Flash/local 대안과 동일 조건 비교 |

## 12.7 Failure / Evolution / Cost Suites

| Suite | 내용 |
| --- | --- |
| Failure Injection | voice disconnect; Task Manager restart; Downstream Agent timeout; Model unavailable; Downstream Agent가 uncertain/partial result를 반환; duplicate event; cancel/completion race; Context connector failure; exclusive UI conflict |
| Evolution Exercise | Voice Runtime 교체; auxiliary Model endpoint 교체; CLI→ACP Downstream Agent adapter 추가; 신규 Context Source/Input Adapter 추가 |
| Cost Baseline B0 | 모든 non-local reasoning=GPT-5.6 Terra, full retrieved context, caching/model routing/call dedup 없음 |
| Cost Alternative | context minimization, caching, call dedup, local/cloud model placement, capability hoisting 등 대안을 scenario별 비교 |

GPT-5.6 Terra의 현재 official API page는 input $2/1M tokens, cached input $0.20/1M, output $12/1M을 제시한다. 가격은 실험 실행 시점에 다시 확인하여 cost calculation에 사용한다. [REF-08]

# 13. Architectural Decision Point 후보 — 아직 결정하지 않음

| ID | Decision Point | 왜 중요한가 | 대안별 SW 구조 차이 | 영향 QA |
| --- | --- | --- | --- | --- |
| ADP-01 | Partial/Streaming Input Processing | 발화 완료 전 정보를 활용해 첫 반응 latency를 줄일 것인가? | A Final-turn only / B partial ASR local read-only prefetch / C incremental intent preparation. B/C는 partial state·cancel path와 speculative cache가 추가된다. | QA-01, QA-07, QA-10 |
| ADP-02 | 화면 지시 Context 수집 구조 | 한 발화에서 여러 위치를 가리킬 때 speech cue와 pointer/gesture를 어떻게 정확히 맞출 것인가? | A speech-start single snapshot / B timestamped turn-level Interaction Timeline / C Timeline + Gesture Analyzer + optional vision. 구조가 갈수록 event volume·state·alignment component가 증가한다. | QA-09, QA-01, QA-06 |
| ADP-03 | Capability Placement Boundary | 모든 업무를 Downstream Agent에 둘지 일부 local-safe capability를 VIA가 직접 처리할지? | A Thin VIA: 모두 delegation / B Hybrid: allow-listed local fast path / C Rich VIA: broader local capabilities. Local capability가 늘수록 latency/cost는 좋아질 수 있으나 coupling/change surface 증가. | QA-01, QA-05, QA-07 |
| ADP-04 | Generic vs Specialized Downstream Agent Integration | 공통 Agent Port와 자체 Agent 특화를 어느 수준까지 허용할 것인가? | A generic CLI/ACP adapter only / B common port + internal extension / C dedicated internal fast path. specialization이 증가할수록 commonality는 감소하고 streaming/context/state 최적화 여지는 증가. | QA-01, QA-05, QA-06, QA-07 |
| ADP-05 | Concurrent Task Resource Arbitration | 여러 Agent execution이 같은 interactive desktop 같은 exclusive resource를 요구할 때 어떻게 조정할 것인가? | A global lease / B per-resource lock & queue / C cooperative scheduler+priority / D user-mediated conflict. VIA는 Agent 내부 tool을 schedule하지 않고 Agent execution/resource lease만 조정한다. | QA-02, QA-03, QA-04 |
| ADP-06 | Voice Runtime Composition & ASR Side-channel | S2S 중심 구조에서 transcript/delta를 얼마나 독립된 side-channel로 확보할 것인가? | A S2S + final transcript / B S2S + streaming ASR/delta side-channel / C decomposed VAD+ASR+LLM+TTS. B/C는 temporal grounding·prefetch·observability가 좋아지지만 complexity/resource 증가. | QA-01, QA-09, QA-10, QA-05, QA-07 |
| ADP-07 | Model Gateway Selection Policy | VIA 내부 Model을 고정 바인딩할지 runtime policy로 선택할지? | A component-fixed binding / B pre-qualified model 중 capability·privacy·cost 기반 policy selection / C telemetry 기반 fully dynamic optimizer. 동적일수록 cost/adaptability potential과 비결정성·검증 복잡성이 함께 증가. | QA-01, QA-05, QA-07, QA-10 |
| ADP-08 | Voice/Text 공통 대화 경계(Voice/Text Interaction Unification Boundary) | Voice와 Text를 어느 계층에서 하나의 conversation turn으로 합칠 것인가? | A modality별 pipeline 후 late merge / B early canonical UserTurn / C common Conversation Turn Manager + modality-specific InteractionAnchor. C는 continuity와 context anchor를 명시하지만 common layer가 커진다. | QA-01, QA-03, QA-10, QA-05 |
| ADP-09 | 기존 Task 연계 판단 구조 | 새 turn이 기존 task follow-up인지 신규 task인지 어떻게 판정할 것인가? | A Model-only / B deterministic candidate filtering(active/pending/recency/explicit anchor)+Model classification / C explicit-anchor-first + ambiguity scorer + clarification. B/C는 state index와 candidate layer가 추가된다. | QA-10, QA-01, QA-03 |
| ADP-10 | Intent Refinement 구조 | 모호한 자연 발화를 어떤 단계로 정제할 것인가? | A single-shot Model / B Normalizer+Temporal Referent Binder+Model+Schema/State Validator / C iterative context retrieval+refinement loop. 단계가 많을수록 accuracy potential과 latency/cost가 함께 증가. | QA-10, QA-09, QA-01, QA-07 |
| ADP-11 | Downstream Agent Routing 구조 | 정제된 request를 어떤 방식으로 적절한 Agent에 매핑할 것인가? | A LLM-only routing / B capability eligibility filter + semantic reranker / C capability ontology + deterministic policy scoring + reranker. Deterministic filtering이 늘수록 correctness/설명가능성은 좋아지나 catalog 관리 복잡도 증가. | QA-11, QA-01, QA-05 |
| ADP-12 | Downstream Agent Capability Contract | Router가 Agent capability와 현재 availability를 어떻게 알 것인가? | A static manifest / B runtime capability discovery / C stable manifest + dynamic health/resource status hybrid. Static은 단순·예측 가능, dynamic은 최신성, hybrid는 둘을 결합하지만 Registry/refresh 구조가 필요. | QA-11, QA-05, QA-04 |

# 14. Architectural Driver 선정에 사용할 입력

| Critical FR 후보<br>FR-03/04 Interaction Timeline 및 multiple referent, FR-06~10 intent refinement/compound/clarification, FR-11/35/43 Downstream Agent capability·routing·follow-up, FR-23/25/26/30 durable task, FR-37/39 mixed modality, FR-40/41 concurrency/conflict, FR-42 local fast path, FR-44 Model Invocation Profile 관리. |
| --- |

| Hard Constraints<br>CON-01 local PC deployment, CON-02 Voice-first, CON-03 local-first context, CON-04 trusted Downstream Agent model(Agent 자체 tool/action security boundary를 신뢰하며 VIA 중앙 Tool Gateway를 두지 않음). |
| --- |

| H,H Critical QA 후보<br>QA-01 VIA 소프트웨어 처리 지연, QA-02 동시 작업 처리와 PC 공존성, QA-03 대화·작업 상태 복구, QA-05 변경 및 연동 용이성, QA-06 외부 Context 노출 최소화, QA-07 Model/Token 비용 효율, QA-09 화면 지시 대상 연결 정확도, QA-10 사용자 요청 및 Task 연계 해석 정확도, QA-11 Downstream Agent 선택 정확도. |
| --- |

최종 Architectural Driver는 Critical FR + Hard Constraint + H,H QA를 함께 놓고 선정하며, 각 Driver에 대해 대안과 QA trade-off를 비교한다.

# 15. 외부 근거 자료

아래 자료는 수치나 ISO 분류를 방어하기 위한 근거이다. 문서에서는 외부 자료가 직접 말하는 내용과 본 프로젝트가 선택한 planning target을 구분하여 기술한다.

[REF-01] ISO/IEC 25010:2023 — Product quality model (ISO official)
https://committee.iso.org/standard/78176.html

[REF-02] ISO/IEC 25010:2023 sample/preview — characteristic definitions
https://cdn.standards.iteh.ai/samples/78176/13ff8ea97048443f99318920757df124/ISO-IEC-25010-2023.pdf

[REF-03] Microsoft Learn — Plan and measure app performance
https://learn.microsoft.com/en-us/windows/apps/develop/performance/planning-measuring-performance

[REF-04] Microsoft Learn — Responsive interactions and Windows Application Performance
https://learn.microsoft.com/en-us/windows/apps/develop/performance/responsive

[REF-05] Google SRE Book — Service Level Objectives
https://sre.google/sre-book/service-level-objectives/

[REF-06] OpenAI API — GPT-Realtime-2.1 model
https://developers.openai.com/api/docs/models/gpt-realtime-2.1

[REF-07] OpenAI API — Realtime API VAD and interruption
https://platform.openai.com/docs/api-reference/realtime

[REF-08] OpenAI API — GPT-5.6 Terra
https://developers.openai.com/api/docs/models/gpt-5.6-terra

[REF-09] Qwen3-Omni Technical Report
https://arxiv.org/abs/2509.17765

[REF-10] Alibaba Cloud Model Studio — Qwen Omni Realtime models
https://docs.modelstudio.console.alibabacloud.com/en/model-studio/realtime

[REF-11] Alibaba Cloud Model Studio — Omni-modal model list
https://www.alibabacloud.com/help/en/model-studio/omni

[REF-12] NIST AI RMF — AI Risks and Trustworthiness. Accuracy measures should use realistic test sets, false-positive/false-negative rates where appropriate, and disaggregated results by data segment. https://airc.nist.gov/airmf-resources/airmf/3-sec-characteristics/

# 16. Baseline 상태

| 항목 | 현재 상태 |
| --- | --- |
| 문서 상태 | v1.1 — Approved Baseline |
| 이해관계자 | 10 |
| Use Case | 16 |
| Functional Requirements | 44 |
| Constraints | 4 |
| Architecture Principles | 7 |
| Product E2E Objectives | 8 |
| Model Requirements | 5 + Downstream Agent integration/qualification rules |
| VIA SW Quality Attributes | 11 |
| Architecture Request Suite | 28 core scenarios + 3 accuracy evaluation suites |
| Architectural Decision Point 후보 | 12 |
| Architectural Driver | 아직 선정하지 않음 |

| Baseline 승인<br>본 v1.1 문서는 요구사항·제약·설계 원칙·Product E2E 목표·Model 요구사항·VIA SW Quality Attribute·Measurement Suite·Architectural Decision Point 후보를 포함하는 승인된 요구사항 Baseline이다.<br><br>이후 Architectural Driver와 Decision Point를 구체화하는 과정에서 새 사실이나 trade-off가 발견되면, 기존 Baseline을 임의로 덮어쓰지 않고 변경 항목을 명시하여 v1.1.x 또는 후속 version으로 관리한다. |
| --- |
