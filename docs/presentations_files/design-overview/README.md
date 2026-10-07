# VIA 설계 Overview와 DP 매핑

[편집 가능한 1장 PPTX](./VIA-design-overview-DP41-45.pptx) / [PNG 미리보기](./design-overview-DP41-45.png)

사용자가 제공한 VIA overall architecture의 Interaction, Orchestration, Shared와 External Components 영역을 유지하고, NOS 설계 Overview 레퍼런스처럼 품질 목표와 설계 결정을 색상 점선으로 연결한다. 박스, 저장소, 텍스트, 점선과 번호를 모두 PowerPoint 도형으로 편집할 수 있다.

색상 점선은 참조 구조 위의 **주요 비교 위치**다. 각 DP의 모든 의존성이나 변경 영향의 전수 목록, 선택된 대안 또는 process 경계를 뜻하지 않는다. 같은 번호가 반복되면 하나의 결정이 여러 영역을 함께 다룬다는 뜻이며, 겹치는 색은 서로 다른 결정 질문의 교차다. 왼쪽 품질 목표는 발표용 배경의 주된 설계 동기이며, 각 비교 문서의 전체 V-01~13 품질 검토를 제한하지 않는다.

| DP | 그림의 주요 위치 | 비교 질문 |
| --- | --- | --- |
| [04-41](../../architecture/12-decisions/decision-packages/04-41-request-resolution-control.md) | Request Controller와 Request Interpreter | 조회와 의미 완성의 진행을 모델이 제안할지, 모델의 요청 틀을 코드가 완성할지 |
| [04-42](../../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md) | Request Controller, Task Manager, Agent Gateway와 State Store | 대화와 업무의 서로 다른 상태 및 실행 수명을 통합 Core에서 관리할지, 독립 서비스가 협력할지 |
| [04-43](../../architecture/12-decisions/decision-packages/04-43-request-interpretation.md) | Request Interpreter 내부 | F1~F6 의미를 통합 생산할지, 기능별 생산자와 코드 조정자가 협력할지 |
| [04-44](../../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md) | Interaction Manager, Request Controller와 Response Manager | 새 입력과 업무 사건의 후속 실행을 중앙 비동기 조정 또는 반응형 실행망으로 이어갈지 |
| [04-45](../../architecture/12-decisions/decision-packages/04-45-memory-and-context.md) | Context Manager와 State Store | 원본 소유자의 근거를 요청에서 조합할지, 공통 파생 기억의 게시 및 조회 계약으로 제공할지 |

41의 Context/Task 조회, 42의 Response 전달 상태, 44의 Task/Agent 사건, 45의 원본 소유자 및 공통 Policy/Model 계약은 발표자 노트에 함께 설명한다. 새 대안의 내부 구조를 이미 선택된 Component로 추가하지 않는다. 기존 참조 Architecture, QA 및 ADR는 변경하지 않으며 대안 채택과 구현 및 측정 결과를 주장하지 않는다.

생성 소스: [generate_design_overview.mjs](../../../scripts/presentations/generate_design_overview.mjs). 번들 Node.js와 `@oai/artifact-tool`을 연결한 임시 build 디렉터리에서 스크립트 사본을 실행한다. 인자는 저장소 절대 경로, build 절대 경로와 새 revision 번호다. 환경 변수 `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`, `RUNTIME_NODE_MODULES`는 설치된 Presentations 스킬과 번들 runtime 경로를 지정한다. 이전 final 파일을 덮지 않도록 revision을 바꾸고, 새 PPTX와 렌더를 검토한 뒤 제공한다.
