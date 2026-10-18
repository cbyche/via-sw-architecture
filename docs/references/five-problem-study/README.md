# 다섯 VIA 문제를 위한 reference 조사

2026-10-17 / 고정 소스 정적 조사와 공식 문서 추가 확인 / 실행과 성능 측정 없음

2026-10-17의 첫 개정에서는 기존 소스 사실을 유지하면서 기본안 A/B/C와 선택 후 보완안 A1/B1의 대응을 나누었다. 이후 설계 권고 작업에서는 아래 추가 확인 네 문서를 작성했다. 각 문서가 고정 commit과 조회일의 현재 공식 문서를 구별한다. 실행 검증은 수행하지 않았다. cache, 취소와 부분 갱신 메커니즘이 모두 기본안에 포함되는 것은 아니며 [현재 비교 규칙](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/07-basic-designs-and-tactics.md)을 따른다.

[설계 종합](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/README.md)

## 읽는 순서

| 기록 | 답하는 문제 | 추가 확인한 주요 구현 |
| --- | --- | --- |
| [근거와 인계](evidence-and-handoff.md) | 현재 판단에 근거를 주는 구조, Agent가 바뀔 때 업무 문맥을 넘기는 구조 | rolling view, offload, consolidation / lineage, fork / isolated, child conversation 재사용, shared backend |
| [후속 발화와 외부 Agent 통지](continuous-interaction.md) | 새 입력이 실행 중 판단에 들어가는 경계 | steering checkpoint, ACP supersede, thread / run update와 cancel, read-only side 질문 |
| [화면 지칭](screen-speech-grounding.md) | 당시 관측의 보존, 후보 생산과 결합, 증분 가설 | screenshot + native 입력, snapshot identity, 부분 의미 결합, Fuse, IU add / revoke / commit |
| [Agent 선택](agent-selection.md) | 후보 정보가 어느 판단에 들어가고 선택을 어떻게 채택하는가 | task catalog, A2A card 주입, 별도 selector, progress ledger, handoff 제안 |
| [인계 기능 추가 확인](handoff-capability-check-20261017.md) | 실제 수신자의 읽기와 native session 재사용 범위 | child backend 전달, OpenClaw / acpx resume 및 load, fresh session fallback, 파일 읽기 범위 |
| [후속 입력 권고 추가 확인](interaction-recommendation-check-20261017.md) | 독립 실행이 가능한 경계와 제한 | OpenClaw queue, Deep Agents async task와 capacity, OpenHands message / ask |
| [화면 지칭 권고 추가 확인](grounding-recommendation-check-20261017.md) | 원본 공동 입력과 후보 생산 / 결합의 실제 근거 | WebSurfer, Live Console, QuickSet, OmniParser, InproTKS |
| [Agent 선택 권고 추가 확인](selection-recommendation-check-20261017.md) | 별도 selector의 의미와 독립 선택 정답의 조건 | AutoGen SelectorGroupChat, CrewAI A2A, Deep Agents subagents |

## 다섯 harness의 조사 역할

| Reference | 이번 탐색에서 실질적으로 제공한 근거 | 확대 해석하지 않은 부분 |
| --- | --- | --- |
| OpenClaw | 같은 run 입력 합류, 원본과 파생 기억의 lineage, 외부 harness 연결, writer / snapshot 현재성 | writer 권한이 사용자 정정의 의미 효력을 판정하지 않음. channel routing이 최적 Agent 선택이 아님 |
| OpenHands | 기록과 rolling view 분리, 실제 수신 / 실행 / 재시작 경계, child 문맥 재사용, browser 관측 입력 | SDK와 Canvas 구현을 혼합하지 않음. background 실행을 자동적인 내구 복구로 취급하지 않음 |
| LangChain Deep Agents | 원본 offload와 범위 읽기, isolated / fork, 독립 thread / run, Btw, backend / namespace | 내부 state fork를 임의 상용 Agent memory 이전으로 확대하지 않음. Btw는 자동 자연어 관계 분류를 완성하지 않음 |
| AutoGen Magentic-One 및 AgentChat | 공동 관측 입력, task / progress ledger, selector 문맥, handoff 선택 | domain 계획을 VIA로 가져오지 않음. 다중 Agent라는 이유로 사용자 동시 개입 지원을 주장하지 않음 |
| CrewAI | 결과 의존 context, 기억 생산 / recall, conversational turn, A2A card와 위임 | async 이름을 자유로운 동시 실행으로 해석하지 않음. card의 자기소개를 업무 성능 검증으로 취급하지 않음 |

각 원장에 파일 / 함수 / 고정 commit과 사실, VIA용 추가 계약, 미확인 사항을 분리했다. 현재 배포 버전의 호환성이나 Agent의 모든 기능을 검증한 목록은 아니다.

화면과 음성의 시간 지칭은 다섯 harness의 확인한 경로만으로 충분하지 않아 Google Live Console, QuickSet, OmniParser, Microsoft Psi, InproTKS와 ReTiCo를 추가 조사했다. Agent discovery / 상태 / 취소의 범위를 위해 A2A 1.0.0 공식 명세를 확인했다. 외부 실험 수치를 VIA의 예상값으로 옮기지 않았다.

## 이전 조사와의 관계

[1차 harness 원장](../agent-harness-20261015.md)과 [첫 근거 공급 조사](../qa-first-evidence-supply-20261016.md)를 보존했다. 이번에는 같은 파일을 재인용하는 데서 끝내지 않고 실제 입력 반영 경계, 기억 게시 / 무효화, 업무 인계와 선택 경로를 추가로 읽었다. 관련 기능을 확인하지 못한 경우는 조사 범위의 한계로 표시하며, 저장소 전체에 기능이 없다고 단정하지 않는다.
