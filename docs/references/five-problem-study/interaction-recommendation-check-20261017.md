# 문제 2 설계 권고를 위한 실행 경계 재확인

2026-10-17 / 공식 문서 web 재확인 / 코드 실행과 측정 없음

[기본 비교와 작성자 권고](../../architecture/12-decisions/decision-packages/qa-first-exploration/five-problem-study/02-continuous-interaction.md) / [기존 고정 코드 원장](./continuous-interaction.md)

## 1. 확인 목적과 판본

추천에 필요한 질문은 세 가지다. 같은 실행에 입력을 넣을 때 이미 진행 중인 판단을 기다리는가, 별도 실행을 시작하면 대화 조정이 실제로 반환되는가, 자원이 부족할 때 그 이익이 유지되는가. 공개 구현의 속도를 VIA의 수치로 옮기지 않는다.

아래 공식 문서는 2026-10-17에 web으로 읽은 현재 문서다. 다음 고정 commit에서 확인했던 코드 사실은 기존 원장을 재사용한다.

| 프로젝트 | 기존 코드 판본 | 재사용하는 확인 범위 |
| --- | --- | --- |
| OpenClaw | `e7a155a8ae860d71b1ab5eed45fa4af338011db9` | tool loop의 steering 경계, 입력 보존과 followup, 현재 writer 확인 |
| Deep Agents | `7764f1f8d21a594f0cbd31d273a7cad36597d6ba` | 별도 thread/run 시작, 상태 조회, 같은 thread의 update와 cancel, Btw |
| OpenHands SDK | `fd387b58dc7b707b73c159845c0396a17000be24` | 메시지 기록, LLM I/O 중 lock 해제, pause / interrupt, 서버의 실제 실행 종료와 permit |

이번 web 도구로 위 세 고정 판본의 raw 파일도 재조회했으나 cache miss로 본문을 받지 못했다. 새로 고정 코드를 읽었다고 기록하지 않는다. 현재 문서의 추가 설명은 아래에 별도로 표시하며, 이를 과거 commit의 기능으로 소급하지 않는다. 검색에 나타난 제3자 문서 복제본은 근거로 사용하지 않았다.

## 2. 공식 문서에서 재확인한 사실

### R-OC. 같은 실행 steering의 경계

[OpenClaw Command queue](https://docs.openclaw.ai/concepts/queue)의 How it works, Queue modes, Answering a pending question을 확인했다. 같은 session의 실행을 직렬화하며, steering은 실행 중 도구를 즉시 종료하지 않는다. 도구 묶음이 끝난 뒤 다음 모델 판단에 입력을 반영한다. 수용하지 못하면 후속 실행으로 기다린다. 현재 문서에는 대기 질문의 답을 일반 queue보다 먼저 연결하는 별도 경로도 있다.

**권고에 주는 근거:** A는 실제로 쓰이는 문맥 연속 구조다. 새 입력을 기록하는 것만으로 선행 모델 판단의 잔여 대기가 사라지지는 않는다. 반면 이미 식별된 질문에 대한 답이나 단순 상태 통지까지 매번 새 의미 호출을 요구할 이유는 없다. VIA의 자유로운 발화가 어느 질문에 대한 답인지 정확하게 판단하는 기능이 이미 입증됐다는 뜻은 아니다.

### R-DA. 실행을 분리한 뒤에도 남는 연결과 자원 비용

[Deep Agents Async subagents](https://docs.langchain.com/oss/python/deepagents/async-subagents)의 lifecycle, state management, fork mode, worker pool, troubleshooting을 확인했다. launch는 task ID를 반환해 조정자가 다음 대화를 이어갈 수 있게 한다. update는 같은 thread에 새 run을 만들며, task 기록은 대화 메시지와 별도 상태로 유지한다. 현재 fork 설명에서는 시작 시 대화 복사 후의 메시지가 자동 전달되지 않는다. worker pool이 부족하면 별도 run도 대기하고, 즉시 반복 polling하면 비동기 경로를 다시 막을 수 있다.

**권고에 주는 근거:** B에는 반환 후 대화 처리, 명시적 변경 전달과 완료 재연결이 모두 필요하다. 별도 ID만 만들거나 조정자가 완료를 계속 기다리면 목적을 달성하지 못한다. LangGraph의 worker slot과 VIA의 cloud 모델 동시 호출 한도는 다른 자원이다. 전자는 독립 실행에도 수용 한도가 있다는 구현 근거이며, 구체 수치를 후자에 그대로 적용하지 않는다.

### R-OH. 입력 수신과 pause의 차이

[OpenHands Send Message While Running](https://docs.openhands.dev/sdk/guides/convo-send-message-while-running)은 실행 중 메시지를 사건 목록에 넣고 이후 Agent step이 읽는 예를 제공한다. [Conversation API의 LocalConversation.pause](https://docs.openhands.dev/sdk/api-reference/openhands.sdk.conversation#pause)는 step 경계에서 멈추며 진행 중 LLM 호출의 완료까지 기다린다고 설명한다. 같은 API의 `ask_agent`는 주 대화를 기록하거나 변경하지 않는 별도 질문 경로다.

**권고에 주는 근거:** 실행 중 입력 수신을 독립적인 의미 판단으로 계산하지 않는다. pause와 즉시 interrupt도 구별한다. 현재 API 문서에서 async interrupt 세부를 모두 확인한 것은 아니며, 그 차이는 기존 고정 코드 원장의 확인 범위를 유지한다. 별도 질문 경로의 존재는 A1의 실용성을 보강하지만 VIA의 실제 전달 이력과 후속 지칭 연결을 대신하지 않는다.

## 3. 사실에서 설계 권고로 넘어가는 범위

이 자료는 기본 A와 B가 구현 가능한 메커니즘임을 뒷받침한다. VIA에는 범위가 정해진 근거 읽기와 의미 판단을 수행하는 실행만 B의 요청별 실행으로 두고, 외부 업무 내부 계획은 그대로 Downstream Agent에 남기는 구성을 권고한다. 이는 자료에서 그대로 복사한 제품 구조가 아니라 VIA의 기능에 적용한 작성자 판단이다.

B를 추천하는 근거는 VIA의 진행 중 판단을 보존하면서 새 입력을 처리할 경로를 확보하는 것이다. 외부 Agent가 오래 작업한다는 사실만으로 B를 추천하지 않는다. 외부 진행 코드 기록과 음성 중단은 양안 공통이며, 전체 호출 한도와 반환 경로의 추가 비용을 비교에 포함한다.

공식 자료에서 VIA의 입력 교차 빈도, cloud 호출 지연, 관계 판단 정확도, 비용과 최대 RAM을 얻은 것은 아니다. 따라서 수치 우위나 자동 복구를 주장하지 않는다. 추천의 조건과 반증 가능한 PoC 범위는 본문의 2026-10-17 설계 권고에 기록한다.
