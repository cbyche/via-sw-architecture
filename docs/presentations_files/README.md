# VIA 발표자료

발표용 문서, 편집 가능한 PPTX, 이미지와 참고 자료는 `docs/presentations_files/`에 모은다. 기존 `docs/presentation_files/`의 자료도 이 폴더로 이동했다.

| 자료 | 설명 및 원문 | 편집 가능한 PPTX | 이미지 미리보기 |
| --- | --- | --- | --- |
| 과제 소개: 대표 시나리오 | [문안 및 QA·DP 연결](./representative-scenario/README.md) | [1장 PPTX](./VIA-representative-scenario.pptx) | [PNG 미리보기](./representative-scenario/representative-scenario.png) |
| 과제 배경과 필요성 | [필요성 페이지 생성 스크립트](../../scripts/presentations/rebuild_project_necessity.mjs) | [3장 PPTX (v7)](./VIA_과제배경_필요성_검토반영_v7.pptx) | [필요성 페이지](./project-necessity/necessity-v7.png) |
| 기능 요구사항 및 제약사항 | [원문](./functional-requirements-and-constraints.md) / [파일 안내](./functional-requirements/README.md) | [1장 PPTX](./functional-requirements/VIA-functional-requirements-and-constraints.pptx) | [미리보기](./functional-requirements/index.html) |
| 품질 요구사항 | [원문](./quality-attributes.md) / [파일 안내](./quality-attributes/README.md) | [2장 PPTX](./quality-attributes/VIA-quality-attributes.pptx) | [미리보기](./quality-attributes/index.html) |
| QA 목표 근거와 7단계 채점표 | [원문](./quality-metrics.md) / [파일 안내](./quality-metrics/README.md) | [10장 PPTX](./quality-metrics/VIA-quality-metrics.pptx) | [미리보기](./quality-metrics/index.html) |
| 설계 Overview와 DP 매핑 | [범위와 편집 안내](./design-overview/README.md) | [1장 PPTX](./design-overview/VIA-design-overview-DP41-45.pptx) | [PNG 미리보기](./design-overview/design-overview-DP41-45.png) |
| DP 41~45 배경 | [파일 안내](./dp-background/README.md) | [5장 PPTX](./dp-background/VIA-DP-background-41-45.pptx) | [미리보기](./dp-background/index.html) |
| DP 41~45 설계 비교 | [파일 안내](./dp-comparison/README.md) | [5장 PPTX](./dp-comparison/VIA-DP-comparison-41-45.pptx) | [미리보기](./dp-comparison/index.html) |
| 사용자 경험 도입과 설계 비교 발표 | [대본·발표 흐름](./dp-background/README.md) / [편집 방법](./DP-PPTX.md) | [11장 PPTX](./VIA-DP-background-and-comparison-41-45.pptx) | [도입 미리보기](./dp-background/dp-background-overview.png) 및 위의 배경·비교 |

통합 11장은 본 발표 9장(도입 → 41 → 44 → 45 → 42)과 43 부록 2장이다. 배경·비교 전용 5장 자료는 번호순 참조용으로 유지한다. 설명 순서이며 대안 선택이나 최종 DP 선정을 뜻하지 않는다.

기능 요구사항용 아이콘은 [requirements-assets](./requirements-assets/index.html)에 있으며, 제공받은 레퍼런스 이미지는 [presentation_example1](./presentation_example1/)과 [presentation_example2](./presentation_example2/)에 보관한다. 과제 배경 PPTX는 이 폴더 최상위에 있다. 최신본 v7은 3페이지인 “과제 소개 | 필요성”만 다시 구성했다. 레퍼런스의 세로 영역 세 개와 남색 제목 상자, 굵은 핵심 문구와 연두색 강조를 따른다. 정정 전후의 내용과 조건을 비교하는 편집 가능한 표, 대화와 업무가 함께 이어지는 시간축, VIA 내부 기술의 변화와 새로운 Downstream Agent 연동을 나타내는 도식을 넣었다. 페이지에는 QA 이름과 ID를 먼저 표시하지 않으며, 사용 특성에서 필요성을 설명해 뒤의 품질 요구사항과 DP 04-41~45로 이어진다. 다른 두 페이지와 시연 GIF, 상단 제목 및 진행 표시는 원본 그대로 유지한다. [v3](./VIA_과제배경_필요성_검토반영_v3.pptx), [v4](./VIA_과제배경_필요성_검토반영_v4.pptx), [v5](./VIA_과제배경_필요성_검토반영_v5.pptx), [v6](./VIA_과제배경_필요성_검토반영_v6.pptx)는 이전 버전으로 보관한다.

발표자료의 Architecture 원문은 [docs/architecture](../architecture/README.md)다. 생성 스크립트와 재생성 방법은 각 자료의 안내 문서를 따른다.
