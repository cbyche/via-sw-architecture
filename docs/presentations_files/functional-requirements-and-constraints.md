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

## 발표용 아이콘 그림

현재 네 가지 FR을 위해 각각 만든 그림이다. 모두 **1254 × 1254 투명 배경 PNG**이며 발표자료에서 제목과 설명을 별도로 붙일 수 있도록 그림 안에 문자를 넣지 않았다. 이미지를 클릭하면 원본 PNG를 열 수 있다.

| FR1. 대화 지속성 제공 / DP-44 | FR2. 음성-화면 지칭 연결 / DP-47 |
|---|---|
| [![사용자의 정정 발화와 Agent 질문 또는 결과가 겹쳐도 이어지는 대화](./requirements-assets/four-fr/fr1-dialogue-continuity.png)](./requirements-assets/four-fr/fr1-dialogue-continuity.png) | [![음성 지칭을 마우스로 가리킨 화면의 그래프에 연결](./requirements-assets/four-fr/fr2-voice-screen-grounding.png)](./requirements-assets/four-fr/fr2-voice-screen-grounding.png) |
| 사용자 발화와 정정, Agent 질문과 결과가 겹쳐도 다음 대화로 이어진다. | 음성 말풍선이 포인터와 선택 범위로 지정한 그래프에 연결된다. |

| FR3. 과거 이력 맥락 제공 / DP-45 | FR4. 대화-업무 관계 연결 / DP-42 |
|---|---|
| [![과거 대화와 완료 기록에서 필요한 내용을 현재 요청의 맥락으로 제공](./requirements-assets/four-fr/fr3-history-context.png)](./requirements-assets/four-fr/fr3-history-context.png) | [![대화, 요청, 질문, 업무와 Agent 실행을 하나의 관계로 연결](./requirements-assets/four-fr/fr4-dialogue-work-linkage.png)](./requirements-assets/four-fr/fr4-dialogue-work-linkage.png) |
| 과거 대화와 업무 결과 중 필요한 내용을 현재 요청의 맥락으로 가져온다. | 대화 말풍선, 요청 문서, 확인 질문, 업무 목록과 Agent 실행의 연결을 유지한다. |

내장 ImageGen으로 생성했으며 [최종 프롬프트](./requirements-assets/four-fr/prompts.md)를 함께 보관한다. 그림은 기능을 설명하는 비유이며 특정 Component 구조나 실행 순서를 규정하지 않는다.

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

이전 6개 FR 그림과 파일은 [이전 그림 모아 보기](./requirements-assets/index.html), [이전 프롬프트](./requirements-assets/prompts.md), [6개 FR 이력](../archive/functional-requirements-before-four-fr-2026-10-14.md)에 보존한다. 현재 네 가지 FR과 번호가 일치하지 않으므로 현재 요구사항에는 위의 새 그림을 사용한다.

현재 C1의 모델 배치는 [설계 Overview](./design-overview/design-overview-DP41-45.png)를 사용한다. 이전 C1 공유 Omni 일러스트는 이력 파일로만 보존한다.

## 관련 설계 문서

- [04-47 음성-화면 지칭 연결](../architecture/12-decisions/decision-packages/04-47-screen-voice-grounding.md)
- [04-42 대화와 업무의 수명](../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md)
- [04-44 지속 대화 실행](../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md)
- [04-45 기억과 Context](../architecture/12-decisions/decision-packages/04-45-memory-and-context.md)
