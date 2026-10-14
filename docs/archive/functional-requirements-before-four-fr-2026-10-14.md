> 과거 기록: 2026-10-14 사용자 정리 이전의 6개 FR이다. 현재 요구사항은 [현재 원문](../presentations_files/functional-requirements-and-constraints.md)을 따른다. 아래 내용은 당시 정의와 발표 연결을 보존한다.

# VIA 기능 요구사항 및 제약사항

현재 발표의 04-41, 04-44, 04-45, 04-42와 직접 연결되는 요구사항을 정리한다. 기능은 고정 UC를 유지하며 모델 배치는 [공통 실행 계약 04-40](../architecture/12-decisions/decision-packages/04-40-common-execution-contract.md)을 따른다.

## 기능 요구사항

| ID | 기능 요구사항명 | 설명 | 관련 DP |
|---|---|---|---|
| FR1 | 지속 대화 처리 | VIA가 요청을 처리하거나 응답하는 동안에도 사용자의 음성 또는 텍스트 입력을 받아 대화를 이어갈 수 있어야 한다. | 04-44 |
| FR2 | 맥락 기반 요청 해석 | 화면, 자료와 대화 맥락을 이용해 사용자 입력을 목표, 대상, 조건과 관련 업무가 식별된 요청으로 해석할 수 있어야 한다. | 04-41 |
| FR3 | 복합 요청 처리 | 한 입력에 포함된 여러 요청을 사용자가 명시한 순서, 조건과 결과 의존 관계에 따라 처리할 수 있어야 한다. | 04-41, 04-44 |
| FR4 | 다중 업무 상호작용 | 여러 업무의 질문, 진행 상황, 결과와 사용자 후속 지시를 각각 해당 업무에 연결할 수 있어야 한다. | 04-42, 04-44 |
| FR5 | 업무 연속성 유지 | 음성 연결 종료, 대화 전환 또는 VIA 재시작 이후에도 기존 업무를 중복 실행 없이 확인 가능한 상태로 이어갈 수 있어야 한다. | 04-42 |
| FR6 | 과거 맥락 활용 | 이전 대화의 정정 내용, 업무 결과와 허용된 사용자 기억을 현재 요청에 필요한 맥락으로 활용할 수 있어야 한다. | 04-45 |

## 제약사항

| ID | 제약사항명 | 설명 | 관련 DP |
|---|---|---|---|
| C1 | 음성과 의미 모델 분리 | VIA와 VAD는 사용자 PC에서 실행하고, 음성과 의미 해석은 별도 클라우드 모델을 Model Access를 통해 사용해야 한다. | 04-41, 04-44, 04-45 |
| C2 | 업무 실행 위임 | 실제 업무 수행에 필요한 도메인 추론, 계획, 도구 선택과 실행은 Downstream Agent에 위임해야 한다. | 04-41, 04-42, 04-44 |

관련 DP는 직접적인 설계 영향의 연결이며 전체 준수 범위를 뜻하지 않는다. 로컬 VAD는 Interaction Manager의 Turn-Taking Control에 속하며 음성 모델과 의미 LLM은 별도 역할이다. Request, Task와 실제 전달의 권위는 로컬 VIA에 남는다. 공급자와 checkpoint는 기능 제약으로 고정하지 않으며, PoC 비용 reference는 품질 문서에서 별도로 지정한다. 이전 공유 Omni 참조 Architecture는 당시 검토 이력으로 보존한다.

## 한 장 발표자료

기능 요구사항 6개와 제약사항 2개를 관련 DP 열까지 포함한 한 장의 편집 가능한 표로 제공한다.

- [편집 가능한 PowerPoint](../presentations_files/functional-requirements/VIA-functional-requirements-and-constraints.pptx)
- [PNG 이미지](../presentations_files/functional-requirements/functional-requirements-and-constraints.png)
- [발표자료 미리보기](../presentations_files/functional-requirements/index.html)

## 선택 사용 가능한 발표용 그림

각 그림은 기능과 제약의 개념을 설명하는 선택 자료이며, 특정 설계 대안의 구조나 구현 결과를 나타내지 않는다. 그림의 문자 없이도 발표에서 축소해 사용할 수 있도록 제작했다.

| ID | 그림 | 시각적 의미 |
|---|---|---|
| FR1 | ![지속 대화 처리](../presentations_files/requirements-assets/fr1-continuous-dialogue.png) | 응답 중에도 열린 음성 및 텍스트 입력 |
| FR2 | ![맥락 기반 요청 해석](../presentations_files/requirements-assets/fr2-contextual-interpretation.png) | 화면의 지칭 대상과 사용자 요청의 연결 |
| FR3 | ![복합 요청 처리](../presentations_files/requirements-assets/fr3-compound-request.png) | 복수 요청의 순서, 조건과 결과 의존 |
| FR4 | ![다중 업무 상호작용](../presentations_files/requirements-assets/fr4-multiple-task-interaction.png) | 서로 섞이지 않는 업무별 질문과 결과 |
| FR5 | ![업무 연속성 유지](../presentations_files/requirements-assets/fr5-task-continuity.png) | 연결 또는 대화가 바뀌어도 이어지는 같은 업무 |
| FR6 | ![과거 맥락 활용](../presentations_files/requirements-assets/fr6-past-context.png) | 과거 정정, 결과와 사용자 기억의 현재 활용 |
| C2 | ![업무 실행 위임](../presentations_files/requirements-assets/c2-downstream-work-delegation.png) | 사용자 요청 연결과 실제 업무 실행의 책임 경계 |

[그림 모아 보기](../presentations_files/requirements-assets/index.html) / [이미지 생성 프롬프트](../presentations_files/requirements-assets/prompts.md)

현재 C1의 모델 배치는 [설계 Overview](../presentations_files/design-overview/design-overview-DP41-45.png)를 사용한다. 이전 C1 공유 Omni 일러스트는 이력 파일로만 보존한다.

## 관련 설계 문서

- [04-41 요청 의미 해결](../architecture/12-decisions/decision-packages/04-41-request-resolution-control.md)
- [04-42 대화와 업무의 수명](../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md)
- [04-44 지속 대화 실행](../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md)
- [04-45 기억과 Context](../architecture/12-decisions/decision-packages/04-45-memory-and-context.md)
