# VIA 기능 요구사항 및 제약사항

2026-10-14 사용자가 현재 다루는 DP-44, DP-47, DP-45, DP-42를 바탕으로 정리한 네 가지 기능 요구사항이다. 기능은 고정 UC를 유지하며 모델 배치는 [공통 실행 계약 04-40](../architecture/12-decisions/decision-packages/04-40-common-execution-contract.md)을 따른다.

## 기능 요구사항

| ID | 기능 요구사항명 | 설명 | 관련 DP |
|---|---|---|---|
| FR1 | 대화 지속성 제공 | 사용자의 끼어들기와 정정 발화가 Downstream Agent의 질문이나 결과 응답과 겹쳐도 대화를 계속 진행해야 한다. | [DP-44](../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md) |
| FR2 | 음성-화면 지칭 연결 | 사용자의 발화 속 지칭 표현(이 표를, 저 그래프를, 열려 있는 문서를 등)을 그 당시의 화면 조작 정보(마우스 위치/클릭/선택/범위, 활성화 창 등)에 따라 의도한 대상에 연결해야 한다. | [DP-47](../architecture/12-decisions/decision-packages/04-47-screen-voice-grounding.md) |
| FR3 | 과거 이력 맥락 제공 | 과거 대화와 업무 처리 결과 기록에서 현재 요청을 이해하는 데 필요한 맥락을 제공해야 한다. | [DP-45](../architecture/12-decisions/decision-packages/04-45-memory-and-context.md) |
| FR4 | 대화-업무 관계 연결 | 대화, 요청, 질문, 업무, Downstream Agent 실행의 관계를 유지해야 한다. | [DP-42](../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md) |

이 네 항목은 현재 DP 논의를 위한 기능 요구사항 정리다. 기존 고정 UC의 기능 범위를 삭제하거나 QA 정의와 대안 선택을 변경하지 않는다. 이전 FR 번호와 의미는 [6개 FR 이력](../archive/functional-requirements-before-four-fr-2026-10-14.md)에 보존하며 현재 번호로 해석하지 않는다.

## 기존 제약사항 기록

| ID | 제약사항명 | 설명 | 관련 DP |
|---|---|---|---|
| C1 | 음성과 의미 모델 분리 | VIA와 VAD는 사용자 PC에서 실행하고, 음성과 의미 해석은 별도 클라우드 모델을 Model Access를 통해 사용해야 한다. | 04-41, 04-44, 04-45 |
| C2 | 업무 실행 위임 | 실제 업무 수행에 필요한 도메인 추론, 계획, 도구 선택과 실행은 Downstream Agent에 위임해야 한다. | 04-41, 04-42, 04-44 |

관련 DP는 직접적인 설계 영향의 연결이며 전체 준수 범위를 뜻하지 않는다. 로컬 VAD는 Interaction Manager의 Turn-Taking Control에 속하며 음성 모델과 의미 LLM은 별도 역할이다. Request, Task와 실제 전달의 권위는 로컬 VIA에 남는다. 공급자와 checkpoint는 기능 제약으로 고정하지 않으며, PoC 비용 reference는 품질 문서에서 별도로 지정한다. 이전 공유 Omni 참조 Architecture는 당시 검토 이력으로 보존한다.

## 기존 발표자료와 동기화 상태

현재 Markdown은 위 네 가지 FR을 기준으로 한다. 아래 PPTX, PNG와 미리보기는 이전 기능 요구사항 6개와 제약사항 2개를 담은 참고본이며, 이번 원문 갱신은 발표 파일에 아직 반영하지 않았다.

- [편집 가능한 PowerPoint](./functional-requirements/VIA-functional-requirements-and-constraints.pptx)
- [PNG 이미지](./functional-requirements/functional-requirements-and-constraints.png)
- [발표자료 미리보기](./functional-requirements/index.html)

## 이전 FR 기준 발표용 그림

아래 그림과 FR 번호는 이전 6개 정의의 이력이다. 현재 네 가지 FR과 번호가 일치하지 않으므로 현재 요구사항 그림으로 그대로 사용하지 않는다. 각 그림은 개념 설명용이며 특정 설계 대안의 구현 결과를 나타내지 않는다. 그림의 문자 없이도 발표에서 축소해 사용할 수 있도록 제작했다.

| ID | 그림 | 시각적 의미 |
|---|---|---|
| FR1 | ![지속 대화 처리](./requirements-assets/fr1-continuous-dialogue.png) | 응답 중에도 열린 음성 및 텍스트 입력 |
| FR2 | ![맥락 기반 요청 해석](./requirements-assets/fr2-contextual-interpretation.png) | 화면의 지칭 대상과 사용자 요청의 연결 |
| FR3 | ![복합 요청 처리](./requirements-assets/fr3-compound-request.png) | 복수 요청의 순서, 조건과 결과 의존 |
| FR4 | ![다중 업무 상호작용](./requirements-assets/fr4-multiple-task-interaction.png) | 서로 섞이지 않는 업무별 질문과 결과 |
| FR5 | ![업무 연속성 유지](./requirements-assets/fr5-task-continuity.png) | 연결 또는 대화가 바뀌어도 이어지는 같은 업무 |
| FR6 | ![과거 맥락 활용](./requirements-assets/fr6-past-context.png) | 과거 정정, 결과와 사용자 기억의 현재 활용 |
| C2 | ![업무 실행 위임](./requirements-assets/c2-downstream-work-delegation.png) | 사용자 요청 연결과 실제 업무 실행의 책임 경계 |

[그림 모아 보기](./requirements-assets/index.html) / [이미지 생성 프롬프트](./requirements-assets/prompts.md)

현재 C1의 모델 배치는 [설계 Overview](./design-overview/design-overview-DP41-45.png)를 사용한다. 이전 C1 공유 Omni 일러스트는 이력 파일로만 보존한다.

## 관련 설계 문서

- [04-47 음성-화면 지칭 연결](../architecture/12-decisions/decision-packages/04-47-screen-voice-grounding.md)
- [04-42 대화와 업무의 수명](../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md)
- [04-44 지속 대화 실행](../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md)
- [04-45 기억과 Context](../architecture/12-decisions/decision-packages/04-45-memory-and-context.md)
