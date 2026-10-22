# VIA 발표자료

> **2026-10-22 현재 편집 위치:** 사용자 확인 후 이 자료를 [네 DP 통합 패키지의 현재 기준본](../architecture/12-decisions/decision-packages/current/presentation/README.md)으로 이관했다. 후속 수정은 새 위치에서 한다. 아래 내용과 기존 파일은 해당 판본의 출처 및 재현을 위해 보존하며 새 편집본과 동시에 갱신하지 않는다.

> **2026-10-14 기능 요구사항 갱신:** [현재 원문](./functional-requirements-and-constraints.md)은 대화 지속성 제공, 음성-화면 지칭 연결, 과거 이력 맥락 제공, 대화-업무 관계 연결의 네 항목이다. 각각 DP-44, DP-47, DP-45, DP-42에 연결한다. 원문에 새 FR별 투명 배경 PNG 네 장을 첨부했다. 기존 발표 파일과 이전 FR 그림은 새 정의를 아직 반영하지 않았다.

현재 통합 발표자료는 **[VIA-architecture-presentation_v13.pptx, 37장](./VIA-architecture-presentation_v13.pptx)**. v11의 필요성 1장을 그림 구성 5안으로 교체해 **2~6장에 모두 수록**. 최신 QA 내용과 나머지 슬라이드는 보존. [현재 순서와 부분 편집 안내](./full-presentation/README.md), [v13 슬라이드 목록](./full-presentation/manifest-v13.json) 참조.

**필요성 시각 대안 검토:** [5장 비교 PPTX v12](./VIA-necessity-options_v12.pptx), [확대 미리보기](./necessity-options-v12/index.html), [안별 설명](./necessity-options-v12/README.md)도 별도로 제공. 사용자 요청에 따라 다섯 안 모두 v13 통합본에 반영. 이전 소개 편집의 [두 장 전용 v10](./VIA-project-introduction_v10.pptx)과 [근거 자료](./project-introduction-v10/README.md)는 이력으로 보존.

v1~v11 통합본은 이전 버전으로 보존. 아래 표는 기존 일곱 파일을 합친 **최초 통합본 v1의 32장 구성**이며 현재 순서는 위 안내 기준.

| 통합본 위치 | 발표 내용 |
| --- | --- |
| 1~3 | 과제 배경, VIA의 역할과 필요성 |
| 4 | 대표 시나리오 |
| 5 | 기능 요구사항 및 제약사항 |
| 6~11 | 품질 요구사항과 공통 평가 활동 |
| 12~17 | QA 목표 근거와 등급 기준 |
| 18 | 설계 Overview |
| 19~20 | 사용자 경험과 DP 도입 |
| 21~23 | DP41 배경 / 비교 / QA 상세 |
| 24~26 | DP44 배경 / 비교 / QA 상세 |
| 27~29 | DP45 배경 / 비교 / QA 상세 |
| 30~32 | DP42 배경 / 비교 / QA 상세 |

전체 16:9 크기를 맞추고 편집 가능한 텍스트, 표와 도형과 기존 이미지/GIF 및 발표자 노트를 보존했다. PowerPoint Section으로 구역을 구분했다. 대표 시나리오는 4페이지로 표시한다. 43과 DP46은 현재 발표에서 제외한다. 별도 배경본과 비교 전용본은 이전 참고본이다.

## 통합 전 구성파일 — 최초 이관 참고본

아래 일곱 PPTX와 PNG는 2026-10-11 최초 통합에 사용한 자료다. 파일은 보존하며 이후 발표 수정은 최신 통합본의 새 버전에 적용한다. 아래의 개별 생성·갱신 설명은 해당 판본의 작성 이력이다. Architecture Markdown/JSON과 그림의 원본은 계속 근거 자료로 사용하되, 일곱 PPTX를 재병합하여 현재 통합본을 덮어쓰지 않는다.

| 자료 | 설명 및 원문 | 편집 가능한 PPTX | 이미지 미리보기 |
| --- | --- | --- | --- |
| 과제 배경과 필요성 | [필요성 페이지 생성 스크립트](../../scripts/presentations/rebuild_project_necessity.mjs) | [3장 PPTX (v7)](./VIA_과제배경_필요성_검토반영_v7.pptx) | [필요성 페이지](./project-necessity/necessity-v7.png) |
| 과제 소개: 대표 시나리오 | [문안 및 QA와 DP 연결](./representative-scenario/README.md) | [1장 PPTX](./VIA-representative-scenario.pptx) | [PNG 미리보기](./representative-scenario/representative-scenario.png) |
| 기능 요구사항 및 제약사항 | [원문](./functional-requirements-and-constraints.md) / [파일 안내](./functional-requirements/README.md) | [1장 PPTX](./functional-requirements/VIA-functional-requirements-and-constraints.pptx) | [미리보기](./functional-requirements/index.html) |
| 품질 요구사항: ASR-QA-01~06 | [원문](./quality-attributes.md) / [파일 안내](./quality-attributes/README.md) | [6장 PPTX](./quality-attributes/VIA-quality-attributes.pptx) | [미리보기](./quality-attributes/index.html) |
| QA 목표 근거와 7단계 채점표 | [원문](./quality-metrics.md) / [파일 안내](./quality-metrics/README.md) | [6장 PPTX](./quality-metrics/VIA-quality-metrics.pptx) | [미리보기](./quality-metrics/index.html) |
| 설계 Overview와 네 DP 매핑 | [범위와 편집 안내](./design-overview/README.md) | [1장 PPTX](./design-overview/VIA-design-overview-DP41-45.pptx) | [PNG 미리보기](./design-overview/design-overview-DP41-45.png) |
| 사용자 경험, DP 배경과 설계 비교 | [도입과 배경 대본](./dp-background-v2/README.md) / [편집 방법](./DP-PPTX.md) | [통합 v5, 14장](./VIA-DP-background-and-comparison-41-45_v5.pptx) | 도입과 배경은 [기존 PNG](./dp-background-v2/README.md), [비교 및 QA 상세 v5](./dp-comparison/v5/README.md) |

통합 직전의 DP v5는 도입 2장과 41 → 44 → 45 → 42의 배경, 비교, QA 상세 각 3장이다. 이 14장은 전체 발표의 19~32장에 포함됐다. 43은 현재 논의에서 제외되어 부록도 제거했다. 비교 전용 v4는 통합 v4의 부분집합이며 비교 전용 v5는 만들지 않았다. 별도 배경본, DP v5와 이전 버전은 이력으로 보존한다. 네 DP의 A/B 대안 선택, 구현과 실측은 아직 없다.

**최초 통합 시점의 발표 정합성:** 기능 요구사항의 43 연결과 공유 Omni 제약을 제거하고 대표 시나리오의 문서/노트를 ASR-QA-v6와 현재 네 DP에 연결했다. 과제 배경 v7의 발표자 노트도 현재 모델 배치와 네 DP에 맞췄다. 통합 v5의 네 QA 요약/상세는 P21의 모집단, 여섯 목표, 보호 조건과 최신 모델 reference를 따른다. 이전 오류 수와 전체 평균/점수는 새 P21에 환산하지 않고 평가 전으로 표시한다. 계산 근거가 있는 지역 call-plan, 자원 예산과 P21 비용 소계 $0.35184/잔여 $0.14816만 가정으로 제시한다. 후보 실측/변경 patch가 없어 전체 A/B 값과 등급은 아직 없다. P21 일반 반응 표본 수 오기는 105개 중 100개로 바로잡고 품질 두 발표도 동기화했다. [현재 QA 데이터와 미리보기](./dp-comparison/v5/README.md)를 따른다.

기능 요구사항용 아이콘은 [requirements-assets](./requirements-assets/index.html)에 있으며, 제공받은 레퍼런스 이미지는 [presentation_kjh](./reference_presentaion/presentation_kjh/)과 [presentation_jjh](./reference_presentaion/presentation_jjh/)에 보관한다. 과제 배경 PPTX는 이 폴더 최상위에 있다. 최신본 v7은 3페이지인 “과제 소개 | 필요성”만 다시 구성했다. 레퍼런스의 세로 영역 세 개와 남색 제목 상자, 굵은 핵심 문구와 연두색 강조를 따른다. 정정 전후의 내용과 조건을 비교하는 편집 가능한 표, 대화와 업무가 함께 이어지는 시간축, VIA 내부 기술의 변화와 새로운 Downstream Agent 연동을 나타내는 도식을 넣었다. 페이지에는 QA 이름과 ID를 먼저 표시하지 않으며, 사용 특성에서 필요성을 설명해 뒤의 품질 요구사항과 DP 04-41, 04-44, 04-45, 04-42로 이어진다. 다른 두 페이지의 화면과 시연 GIF, 상단 제목 및 진행 표시는 원본 그대로 유지한다. 2/3페이지 발표자 노트는 현재 모델 배치와 DP 범위에 맞췄다. [v3](./VIA_과제배경_필요성_검토반영_v3.pptx), [v4](./VIA_과제배경_필요성_검토반영_v4.pptx), [v5](./VIA_과제배경_필요성_검토반영_v5.pptx), [v6](./VIA_과제배경_필요성_검토반영_v6.pptx)는 이전 버전으로 보관한다.

발표자료의 Architecture 원문은 [docs/architecture](../architecture/README.md)다. 부분 편집은 통합본의 안내를 따르고, 개별 자료의 생성 스크립트와 재생성 방법은 최초 구성의 재현 이력으로 읽는다.

과제 배경 v7의 노트만 갱신할 때는 [노트 갱신기](../../scripts/presentations/refresh_project_necessity_notes.mjs)를 새 private build에 복사하고 bundled `node_modules`와 환경 경로를 설정해 `<bundled-node> <build>/refresh_project_necessity_notes.mjs <repo> <build>`로 실행한다. 모든 슬라이드와 media part는 보존한다. 필요성 화면을 재구성하는 기존 생성기를 실행할 때도 이 노트 갱신기를 같은 build에 함께 복사한다. [노트 검수 기록](./project-necessity/notes-verification.json)에 확인 범위를 남겼다.
