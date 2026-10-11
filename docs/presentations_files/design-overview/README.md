# VIA 설계 Overview와 현재 네 DP 매핑

> **현재 발표 편집본:** [전체 통합 PPTX](../VIA-architecture-presentation.pptx)의 18장이다. 아래 개별 PPTX/PNG는 최초 통합 시점의 참고본이다. 발표 수정은 [부분 편집 안내](../full-presentation/README.md)를 따른다. 아래 생성 절차는 이전 구성의 재현 이력이다.

[편집 가능한 1장 PPTX](./VIA-design-overview-DP41-45.pptx) / [PNG 미리보기](./design-overview-DP41-45.png)

2026-10-11 사용자 지시에 따라 43을 제외하고 41 → 44 → 45 → 42의 발표 흐름을 반영했다. 기존 Interaction, Orchestration, Shared와 External Components 영역, 서체와 색상 점선을 유지한다. 박스, 저장소, 텍스트, 점선과 번호를 모두 PowerPoint 도형으로 편집할 수 있다.

색상 점선은 공통 책임도 위의 **주요 비교 위치**다. 각 DP의 모든 의존성이나 변경 영향의 전수 목록, 선택된 대안 또는 process 경계를 뜻하지 않는다. 같은 번호가 반복되면 하나의 질문이 여러 영역을 함께 다룬다는 뜻이다. 왼쪽 연결선도 주된 설계 동기의 일부이며 네 DP는 최신 여섯 ASR-QA와 보호 조건을 모두 검토한다. 기존 V 관점과 참조 Architecture의 검토 이력은 보존한다.

## 최신 품질 목표와 모델 배치

[03-02 품질 정의](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 ASR-QA-v6를 [발표 투영 JSON](../quality-attributes/quality-attributes.json)에서 읽는다. 번호순 우선순위는 요청 처리 정확성, 상호작용 응답시간, VIA 요청 처리시간, 모델 사용 비용, 변경 용이성, 로컬 VIA 메모리 사용량이다. 목표는 각각 98% 이상, 평균 1초 이하와 local stop 0.1초 이하, 평균 10초 이하, $0.50/P21 이하, 평균 3개와 과제별 5개 이하, 전체 commit 4,096MiB 이하다. 숫자는 작성자 평가 기준이며 후보 실측 결과가 아니다.

모델 배치는 [04-40 공통 실행 계약](../../architecture/12-decisions/decision-packages/04-40-common-execution-contract.md)을 따른다. VIA와 권위 상태는 사용자 PC의 로컬 owner가 관리한다. Interaction Manager 내부 로컬 VAD가 발화 경계와 즉시 재생 중단을 담당한다. Model Access는 **클라우드 음성 모델과 별도 의미 LLM**에 연결한다. 기존 LLM Omni와 공통 필수 Streaming ASR 박스는 제거했다. 전사 의존성은 공급자별 연동과 청구 항목으로 확인하며 모델 provider/checkpoint는 이 그림에서 선택하지 않는다.

| DP | 그림의 주요 위치 | 비교 질문 |
| --- | --- | --- |
| [04-41](../../architecture/12-decisions/decision-packages/04-41-request-resolution-control.md) | Request Controller와 Request Interpreter | 조회와 의미 완성의 진행을 모델이 제안할지, 모델의 요청 틀을 코드가 완성할지 |
| [04-44](../../architecture/12-decisions/decision-packages/04-44-continuous-interaction.md) | Interaction Manager, Request Controller와 Response Manager | 새 입력과 업무 사건의 후속 실행을 중앙 비동기 조정 또는 반응형 실행망으로 이어갈지 |
| [04-45](../../architecture/12-decisions/decision-packages/04-45-memory-and-context.md) | Context Manager와 State Store | 원본 소유자의 근거를 요청에서 조합할지, 공통 파생 기억의 게시 및 조회 계약으로 제공할지 |
| [04-42](../../architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md) | Request Controller, Task Manager, Agent Gateway와 State Store | 대화와 업무의 서로 다른 상태 및 실행 수명을 통합 Core에서 관리할지, 독립 서비스가 협력할지 |

41의 Context/Task 조회, 42의 Response 전달 상태, 44의 Task/Agent 사건, 45의 원본 소유자 및 공통 Policy/Model 계약은 발표자 노트에 함께 설명한다. 새 대안의 내부 구조를 이미 선택된 Component로 추가하지 않는다. 기존 참조 Architecture, QA 및 ADR는 변경하지 않으며 대안 채택과 구현 및 측정 결과를 주장하지 않는다.

생성 소스: [generate_design_overview.mjs](../../../scripts/presentations/generate_design_overview.mjs). 번들 Node.js와 `@oai/artifact-tool`을 연결한 임시 build 디렉터리에서 스크립트 사본을 실행한다. 인자는 저장소 절대 경로, build 절대 경로와 새 revision 번호다. 환경 변수 `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`, `RUNTIME_NODE_MODULES`는 설치된 Presentations 스킬과 번들 runtime 경로를 지정한다. 이전 final 파일을 덮지 않도록 revision을 바꾸고, 새 PPTX와 렌더를 검토한 뒤 제공한다.
