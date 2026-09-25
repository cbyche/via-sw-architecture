# VIA System Understanding Review

> **Status: REVIEW NOTE / NON-NORMATIVE / NO DECISION**
>
> Reviewed baseline: `origin/main` commit `0447a90afae178e2ebbfd327837303ee26a1322a`
>
> Purpose: 중요한 Architecture 의사결정 후보를 처음부터 도출하기 전에, VIA 시스템에 대한 공통 이해를 확인하고 보존한다. 이 문서는 새로운 요구사항, DP, QA, ASR 또는 Architecture Decision을 추가하지 않는다. 충돌 시 [active Architecture baseline](../README.md)과 승인된 ADR이 우선한다.

## 1. Review basis

이 기록은 다음 active 문서를 순서대로 검토한 결과다.

- [System Mission & Boundary](../01-system-mission-and-boundary.md)
- [Terms](../02-terms.md)
- [Fixed Architecture Scope](../03-fixed-architecture-scope.md)
- [Canonical Interaction Flow](../04-canonical-interaction-flow.md)
- [Representative Use Cases](../05-representative-use-cases.md)
- [Fixed Assumptions](../06-fixed-assumptions.md)
- [Intentional Variables](../07-intentional-variables.md)
- [Quality Model](../08-quality-attributes/quality-model.md)
- [Voice Responsiveness](../08-quality-attributes/voice-responsiveness.md)
- [Measurement Event & Boundary Contract](../11-measurement/event-boundary-contract.md)
- [Measurement Guide](../11-measurement/README.md)

기존 DP와 ADR은 현재 상태를 확인하기 위한 참고자료일 뿐, 후속 의사결정 후보 도출의 정답이나 출발점으로 사용하지 않는다.

## 2. 사용자가 VIA를 통해 수행하는 핵심 시나리오

### 2.1 자연스러운 연속 대화

사용자는 Voice 또는 Text로 질문하고 S2S 또는 VIA Core의 직접 응답을 받는다. 지속 업무를 나타내는 VIA Task가 생기지 않아도 User Turn, VIA Request, 실제 응답, 확정된 대상과 필요한 맥락은 Conversation에 남아 후속 질문에 사용된다.

### 2.2 화면과 주변 정보를 지칭한 요청

사용자는 선택한 문단, 가리킨 그래프, 다른 탭, 과거 대화의 자료, 특정 파일·메일·일정 등을 자연어로 지칭한다. VIA는 마지막 좌표 하나가 아니라 사용자가 대상을 지정한 시점의 화면, 선택, 포인터, 창, 문서와 시간 관계를 이용해 Referent를 찾아야 한다. 범위가 정해진 read-only 조회·설명은 VIA가 직접 수행할 수 있다.

### 2.3 대화에서 실제 업무로 전환

사용자는 “방금 설명한 내용으로 발표자료를 만들어줘”처럼 Direct Response의 내용을 실제 업무로 이어간다. VIA는 앞 대화와 대상을 찾고 사용자 목표, 제약과 결과물 요구를 정리하여 VIA Task를 만든 뒤 적절한 Downstream Agent에 위임한다.

### 2.4 여러 장기 업무의 비동기 관리

사용자는 한 업무가 실행되는 동안 다른 질문이나 업무를 시작할 수 있다. Agent의 progress, clarification, approval request, partial result, completion과 failure는 새 사용자 입력이 없어도 올바른 VIA Task와 Conversation으로 돌아와야 한다. 사용자가 Agent의 thread나 run ID를 직접 관리하도록 요구하지 않는다.

### 2.5 중단·정정·채널 전환·재시작 이후의 연속성

사용자는 VIA의 음성을 끊고 요청을 정정하거나 Voice와 Text를 전환하고 Voice Connection을 다시 만들 수 있다. 특정 Task만 취소할 수 있어야 하며 음성 중단은 업무 취소가 아니다. VIA 재시작 뒤에는 저장된 관계와 Agent가 실제 제공하는 상태를 이용해 업무를 재연결하고, 확인되지 않은 외부 변경 업무를 중복 실행하지 않아야 한다.

## 3. VIA가 반드시 책임지는 것

VIA는 사용자 Interaction과 Orchestration의 기준 시스템이다.

- Voice/Text 입력과 Voice/Text 응답
- S2S 기반 Voice Runtime, Voice Connection과 interruption 관리
- User Turn을 하나 이상의 VIA Request로 정리하고 복합 요청의 독립·순차·데이터 의존·조건 관계 보존
- Conversation, VIA Request와 VIA Task의 identity 및 상태 관리
- 현재 Request의 Task Relation과 Request Handling 판단
- Interaction, Conversation, Task, Personal Information, Public Information, User Memory Context 사용
- Referent Resolution과 Interaction Grounding
- S2S Direct Response, VIA Core Direct Response와 Agent 위임 연결
- 적절한 Agent 선택과 필요한 요청·Context 구성
- VIA Task와 Agent Execution의 correlation
- progress, clarification, approval, correction, cancellation, result와 failure를 올바른 업무에 연결
- Context 접근 및 Model/Agent 제공 범위, consent와 사용자-facing provenance 통제
- 허용된 User Memory의 등록·조회·수정·삭제
- 확인된 상태, 추정과 미확인 상태를 구분한 사용자 안내
- 모든 사용자-facing 응답의 Text 기록과 활성 Voice에서의 짧은 핵심 전달

책임 경계는 다음과 같다.

> **Bounded Read / Search / Understand는 VIA에서 수행할 수 있다. Open-ended Research / 업무 Reasoning / Plan / Act는 Downstream Agent가 담당한다.**

VIA가 직접 수행할 수 있다는 것은 모든 해당 요청을 VIA Core에서 처리해야 한다는 뜻이 아니다.

## 4. Downstream Agent와 Model Runtime의 책임

### 4.1 Downstream Agent

Downstream Agent는 실제 업무 수행 주체다.

- open-ended research와 Source 탐색·선별·종합
- domain-specific reasoning과 업무 수행 계획
- Tool 선택과 실행
- 문서 작성·수정, 이메일 발송, 일정 변경, Application 조작과 Web transaction 등 외부 업무 상태 변경
- Agent 내부 workflow, sub-task, retry와 planning loop
- Agent 내부 Model과 실행 방식

VIA는 Agent 내부 계획이나 개별 Tool Call을 다시 구현하지 않는다. Agent 업무 Action의 실제 실행과 권한 강제도 Agent 책임이다. VIA는 자신의 Context 접근·외부 제공을 통제하고 사용자 질문·동의·승인을 해당 업무에 연결한다.

### 4.2 AI Model Runtime

S2S Model과 VIA semantic model은 VIA가 사용하는 inference dependency이며 VIA의 최종 권한이나 authoritative state를 대신하지 않는다.

- S2S Model은 실시간 음성 이해·생성과 허용된 직접 응답에 사용된다.
- VIA semantic inference는 요청 정리, 지칭 해소, Task 연결, 처리 경로, Agent 선택과 응답 구성에 사용될 수 있다.
- Model Runtime은 local 또는 remote에 배치될 수 있다.
- 하나의 물리 Model이 여러 역할을 담당하거나 deterministic logic과 조합될 수 있다.

VIA Architecture는 Model invocation interface, streaming/event contract, lifecycle, deployment binding, provider 교체 구조와 VIA state 연결을 책임진다. Model 내부 구조와 학습은 범위 밖이다.

## 5. 주요 상태와 데이터 흐름

핵심 lifecycle은 서로 분리된다.

| 상태 단위 | 의미 |
| --- | --- |
| Voice Connection | 현재 실시간 음성 입출력이 연결되어 있는가 |
| Conversation | 사용자와 VIA가 지금까지 무엇을 주고받았는가 |
| User Turn | 사용자가 한 번 말하거나 입력한 것 |
| VIA Request | VIA가 처리하는 하나의 논리적 요청 |
| VIA Task | 여러 Request에 걸쳐 추적할 사용자 업무 목표 |
| Agent Execution | Agent가 업무를 실제로 수행하는 한 번의 실행 |

```text
Voice/Text + 화면 interaction
  → User Turn
  → 하나 이상의 VIA Request와 request relation
  → Context·Referent·제약 확정
  → Task Relation 판단
  → Request Handling 결정
      ├─ S2S Direct Response
      ├─ VIA Core Direct Response
      └─ VIA Task ↔ Agent Execution 위임
  → Text 기록 + 필요한 짧은 Voice
```

Task Relation과 Request Handling은 서로 다른 판단이다. Direct Response에도 Conversation은 남으며, Agent 업무를 새로 시작할 때는 VIA Task와 Agent Execution의 관계를 관리한다.

Context는 Interaction, Conversation, Task, Personal Information, Public Information, User Memory의 여섯 범주다. 접근 권한, consent, 외부 제공 범위와 Agent trust 같은 Policy State는 의미 Context와 구분한다.

## 6. 비동기 interaction에서 보존해야 할 의미

Agent의 progress, question, approval request, result와 failure는 source에서 비동기로 도착한다. VIA는 Task ID, Agent Execution ID, revision과 관련 Request를 사용해 이를 다시 올바른 Conversation과 사용자 응답으로 연결해야 한다.

Architecture는 다음 교차 상황을 처리해야 한다.

- 결과 도착 순서 역전
- 같은 Agent의 복수 실행과 여러 Agent의 동시 실행
- 여러 Task에서 동시에 대기하는 clarification 또는 approval
- 완료 event와 취소 요청의 교차
- stale 또는 중복 status event
- 음성 출력 중 새 User Turn이나 다른 Task 결과 도착
- Voice 연결 종료 후 Text 또는 새 Voice 연결로 계속 진행
- VIA 재시작 동안 Agent 실행이 계속되거나 이미 끝난 경우
- 외부 실행 상태를 확인할 수 없는 경우의 중복 Action 방지

접수, 실행 중, 질문 대기, 취소 요청, 실제 취소 완료, 부분 완료, 실패와 확인 불가를 서로 다른 상태로 사용자에게 전달해야 한다.

## 7. 구조적으로 어려운 문제

아래는 아직 DP로 정제하지 않은 problem space다.

1. **서로 다른 시간축의 의미 결합** — 음성, 전사 수정, 포인터·선택과 화면 변화를 지칭 당시 기준으로 결합해야 한다.
2. **빠른 음성 경로와 단일한 사용자 의미의 양립** — S2S가 직접 답하면서도 정확히 기록되어야 하고 같은 Request가 Core에서 중복 실행되면 안 된다.
3. **사용자 Task와 외부 실행 identity의 분리** — VIA Task는 Agent thread/run ID에 종속될 수 없으며 한 Task와 여러 실행의 관계를 추적해야 한다.
4. **비동기 상태의 권위와 진실성** — 사용자에게 보이는 상태를 어떤 근거로 누가 확정하는지와 stale·중복 event 처리가 필요하다.
5. **재시작 복구와 중복 Action 방지** — 로컬 기록과 외부 실행 상태가 어긋난 경우에도 안전하게 재연결해야 한다.
6. **충분한 Context와 최소 노출의 긴장** — 정확한 지칭과 위임에 필요한 Context를 확보하면서 외부에는 최소 범위만 제공해야 한다.
7. **복합 요청 관계 보존과 책임 침범 방지** — 사용자가 명시한 관계는 VIA가 보존하되 업무 내부 planning을 가져오지 않아야 한다.
8. **실시간 Voice와 비동기 결과의 중재** — 사용자 발화, direct response와 Agent event가 겹칠 때 audio lane과 사용자 주의를 관리해야 한다.
9. **외부 생태계 변화의 파급 격리** — Model, Agent, Context provider, 저장 계약과 배치가 바뀌어도 사용자-facing 의미를 유지해야 한다.
10. **측정 가능한 Architecture 인과관계** — 관측 차이를 책임, 계약, 상태 소유권, dependency 방향, call graph 또는 process boundary의 차이로 설명할 수 있어야 한다.

현재 QA 초안은 이 problem space를 responsiveness(QA-01~05), correctness/continuity(QA-11~15), 변화 국소성(QA-21~23), recovery와 fault containment(QA-31/32), target-device resource(QA-41), 보호정보 최소 노출(QA-51), 실행 trace와 evidence 재현성(QA-61/62)으로 관찰하려 한다. QA catalog와 ASR은 아직 확정되지 않았다.

## 8. 후속 의사결정 후보 도출 원칙

후속 작업에서는 기존 DP 이름이나 현재 ADR의 결론을 출발점으로 삼지 않는다. 먼저 이 문서의 system understanding과 active baseline에서 독립적으로 구조적 질문을 도출한 다음, 기존 DP와의 중복·누락을 나중에 대조한다.

Architecture 의사결정 후보는 최소한 다음을 만족해야 한다.

- 책임, 권한, 상태 소유권, 계약, process/deployment 또는 dependency 방향을 결정한다.
- 정상적으로 구현 가능한 대안이 둘 이상 존재한다.
- 선택에 따라 여러 Component와 QA가 구조적으로 달라진다.
- 나중에 바꾸기 어렵거나 변경 비용이 크다.
- 특정 구현 세부사항이나 단순 성능 tuning이 아니다.
- 측정 결과에 맞추어 사후 구성한 선택이 아니다.

## 현행 inventory와 모델 제약 보충 · 2026-09-25

이해 기록의 당시 QA/DP 제안은 최종 선정이 아니다. 현행 후보는 [VIA-DP-01~18](./README.md), 이전 계열 관계는 [매핑](./legacy-dp-mapping.md)을 따른다. VIA는 S2S 1개와 semantic LLM 1개만 사용하며 Component/Task별 추가 모델을 적재하지 않는다. 역할별 prompt·세션은 공유 모델에 대한 호출 방식이다. 외부 Agent Runtime과 VIA 내부 Client는 구분한다. QA-41은 메모리 예산·의미 있는 차이 근거 없이 핵심 ASR로 끌어올리지 않는다.
