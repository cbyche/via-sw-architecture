# 일곱 문제 분석의 공식 구현 근거

2026-10-18 / [설계 결과 진입점](../../architecture/12-decisions/decision-packages/seven-problem-review/README.md)

## 읽기 순서

| 원장 | 확인한 문제 | 주요 source |
| --- | --- | --- |
| [근거 공급과 업무 인계](evidence-handoff.md) | 01, 05 | OpenHands view와 child session, Deep Agents 범위 읽기/격리/공유 backend, OpenClaw 기억과 session, CrewAI memory/task context, Magentic-One ledger, acpx |
| [판단 실행, 단계 연결과 서비스 경계](execution-lifecycle.md) | 02, 03, 07 | OpenClaw steering, OpenHands 수신/step/remote lifecycle, Deep Agents run, AutoGen runtime, CrewAI Flow, Pipecat, SQLite |
| [화면과 발화 지칭](screen-grounding.md) | 04 | Magentic-One WebSurfer, QuickSet, finite-state multimodal parsing, ReALM, OmniParser, Windows capture/UIA, Psi/InproTKS |
| [Agent 선택](agent-selection.md) | 06 | Deep Agents task, OpenHands TaskTool, CrewAI A2A/manager, AutoGen selector/swarm/Magentic-One, OpenClaw bindings, A2A |

각 원장은 확인한 판본/파일 또는 공식 문서, 실제 메커니즘, VIA에 추가한 계약, 적용 한계를 구별한다. 이전 고정 source 조사를 재사용한 항목과 이번에 직접 다시 읽은 항목, 조회 실패를 구별했다. repository 전체를 실행하거나 모든 기능의 존재/부재를 확인했다는 뜻은 아니다.

## 요청한 다섯 harness가 분석에 들어간 위치

| reference | 실제로 활용한 설계 요소 | 전체 제품을 단일 대안으로 분류하지 않는 이유 |
| --- | --- | --- |
| **OpenClaw** | 동일 run의 steering, memory 생산/provenance, binding routing, ACP session 지원 확인 | 같은 제품에 유지 문맥과 조회/기억, 결정적 routing이 공존. routing은 목적별 성능 최적화와 다름 |
| **OpenHands** | rolling view/condensation, I/O 수신과 step의 분리, child 대화와 remote 서버 | 대화별 view를 사용한다고 모든 요청/원본이 그 view에만 있는 것은 아님. 서버 분리가 VIA의 두 권위 서비스와 같지는 않음 |
| **LangChain Deep Agents** | offload와 read 범위, isolated/fork, async thread/run, 제한된 side 질문 | 필요한 원본 조회와 유지 memory가 공존. 별도 child에 VIA의 업무 계획을 이식하지 않음 |
| **AutoGen Magentic-One 및 AutoGen runtime** | 공통 facts/plan ledger 재배포, WebSurfer 공동 입력, selector/swarm, async queue runtime | Magentic-One 모델 Orchestrator와 코드 Dispatcher를 구별. domain plan은 VIA 소유 기능이 아님 |
| **CrewAI** | task context, unified memory, listener/Flow, A2A와 hierarchical manager | Flow 연결과 독립 의미 문맥은 다른 축. 모델이 후보를 선택한다고 적격성/정확도가 자동 보장되지는 않음 |

다섯 harness에서 직접 충분하지 않은 temporal speech/screen 지칭은 실제 multimodal 시스템과 원 논문으로 넓혔다. “다섯 제품 중 하나에 이름이 있다”는 이유로 구조 전체가 검증됐다고 말하지 않는다.

## 근거의 세 수준

1. **실제 존재하는 메커니즘:** 소스/원 논문에서 확인한 view 유지, 범위 읽기, 부분 의미 결합, 비동기 실행, 자료 인계 등.
2. **VIA에 맞춘 설계:** 실제 전달 범위, Request/Task 연결, source revision/권한, 오래된 결과의 채택 차단, 인계 계약과 producer revision 등.
3. **품질 차이의 예상:** 동일 목표에서 대기/입력/생산/상태가 어떻게 달라지는지 분석. 외부 실험이 있는 화면 parser도 VIA QA02 수치로 이전하지 않음.

이번 결과의 대안은 1과 2를 근거로 구현 가능한 설계다. 3의 실제 크기와 API 지원/비용을 제품 환경에서 확인한 것은 아니다. 상업 서비스 가용성이나 cloud parser 가격을 확인하지 않은 곳은 자료에 명시했다.
