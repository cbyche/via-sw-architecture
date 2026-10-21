# DP3 v8: 호출, 재허가와 확장 경로 수정

2026-10-22. 사용자가 지적한 DP3 10~12쪽을 다시 작성하고 설계 문서, SVG, draw.io와 PPTX를 대조했다.

[편집 가능한 PowerPoint v8](VIA-DP1-DP5-reference-style-29slides-v8-dp3-guide-fixed-windows.pptx). v7과 그 이전 파일은 보존한다. 전체 29쪽 가운데 변경한 페이지는 10, 11, 12쪽이다.

## 적용 기준과 수정

[INTENT §12](../../../INTENT.md), [그림 작성 기준](../../architecture/12-decisions/decision-packages/diagram-design-guide.md), [기존 발표 체크리스트](reference-checklist-and-review.md)의 R12~14와 DP1 p2, DP2 p6, DP4 p17의 실제 구성을 대조했다. 연녹색 대안 머리, 제목 칸이 있는 옅은 노랑 Component, 흰색 둥근 Module, 원통 상태를 사용한다. 검정은 공통, 파랑은 A 고유, 초록은 B 고유 처리다. 실제 process 경계는 회색 점선이다.

| 기존 지적 | v8의 실제 수정 |
| --- | --- |
| 호출선이 process 테두리에서 끝남 | host 실행 제어기와 worker IPC 수신기를 요청 및 반환선으로 직접 연결 |
| 재허가와 확장이 각주에만 있음 | A의 원본 보완, B의 후보 확장 및 새 원본 확보가 Request Controller와 Context Manager를 거쳐 되돌아오는 경로를 10쪽에 표현 |
| R1 내부가 설명 카드임 | Model Access의 IPC 수신기, CPU 시각 실행기, 현재 모델과 추적 상태를 연결하고 12쪽에서 adapter 및 runtime을 확대 |
| Module과 상태가 혼재함 | 능동 IPC 수신기와 수동 실행 대기열을 둥근 상자와 RAM 원통으로 분리 |
| worker 내부 반환이 불완전함 | runtime → adapter → IPC → host 반환을 표현. CloseEpisode의 추론 없는 adapter 완료도 같은 반환 경로 사용 |
| 상자에 동작 문장, 낯선 약칭이 남음 | 상자에는 구성요소 이름을 쓰고 동작과 조건은 연결선에 배치. 정식 Component 이름은 유지 |
| 모델과 영상 상태의 수명이 혼동됨 | EnsureModel/ReleaseModel과 RunStage/CloseEpisode를 별도 상태의 연결선으로 구분 |

[발표 구조 계약](../../architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/presentation-architecture-contract.md)에 그림의 번호와 Module 이름, 입력 및 반환, 조건부 실행을 맞췄다. B의 최초 후보 생산은 추가 의미 LLM 호출을 요구하지 않는다. UI 결과 또는 native seed를 확보한 뒤 필요한 SAM 작업을 발행하며, R1/R2/R3 모두 총 4 CPU thread와 한 번에 하나의 heavy stage라는 기존 조건을 유지한다.

## 독립 검토와 실제 렌더

독립 검토자가 새 세 페이지의 실제 PPTX 렌더와 설계 계약을 대조했다. 검토 중 발견한 다음 문제를 수정한 뒤 다시 확인했다.

- 10쪽 worker 완료 반환과 12쪽 adapter 완료 반환의 누락
- worker 제목과 상태 상자를 관통하는 선, 요청 및 반환 라벨의 겹침
- 10쪽 A의 4/5번, B의 2/3번 라벨 가림
- 페이지별 범례와 내부 Module 및 수동 대기열의 구분

최종 build04의 10~12쪽을 다시 연 독립 검토에서는 이 수정 범위의 추가 P1/P2를 발견하지 못했다. 주 작성자도 같은 렌더를 확인했다. SVG 세 장은 로컬 브라우저에서 직접 확인했고, 작은 상태 원통의 윗선이 글자와 겹치던 부분과 표의 글자 강조를 SVG 생성기에서 수정했다. PPTX와 SVG의 글꼴 렌더러가 달라 픽셀 동일성을 주장하지 않는다.

## 보존과 형식 검사

최종 PPTX SHA-256: `0b2a87104b740714a9ced05e35b36407ffca723e172e40c2afc5dc00118ace10`.

| 확인 항목 | 수행 결과와 범위 |
| --- | --- |
| 원래 26쪽 보존 | slide XML, notes, slide/notes relationships가 v7과 byte-identical. 전체 29쪽을 렌더하고 보존한 26쪽은 기존 preview와 pixel-identical 확인 |
| 같은 설계 원본 | PPTX, SVG와 draw.io를 같은 scene으로 생성. 83개 노드의 자체 라벨을 실제 PPTX XML에서 확인하고 SVG ID 및 draw.io 소유 관계와 대조 |
| 연결과 포함 | 80개 경로의 출발/도착 경계, Module 소유자와 포함 좌표, 직교 경로, 무관한 Module 및 상태 상자 관통 검사 통과. draw.io의 source/target 참조와 대조 |
| PPTX 형식 | package integrity, layout geometry, Calibri 서체 정책, native table, ArtifactTool 재import 통과 |
| Windows 서체 정책 | Calibri, 정수 pt, ko-KR 검사 위반 0 |
| 기존 합성 QA | QA01~07 페이지와 수치 보존. 12쪽 RAM 합계 592 / 1,954 / 2,374 / 2,662 MB 산술 재확인. 실측으로 재분류하지 않음 |
| Windows Open XML SDK | push 후 실제 v8 경로를 지정한 workflow_dispatch로 별도 확인. 이 절 아래에 수행 결과 기록 예정 |

[패키지 보존](dp3-v8-preservation.json), [렌더와 원본 대조](dp3-v8-render-preservation.json), [파일 검사](dp3-v8-validation.json), [연결 및 포함 검사](dp3-v8-scene-checks.json), [수정 세 페이지 preview](previews-v8-dp3-guide-fixed/).

검증 범위는 설계 표현, 문서 일치, 렌더와 파일 형식이다. 실제 모델 실행, 성능 측정이나 native Windows PowerPoint 실행 및 편집 동작은 검증하지 않았다. PPTX의 상자, 라벨, 표와 선은 편집 가능한 native 객체다. 선은 경로와 화살촉 도형으로 구성되어 상자를 이동할 때 자동으로 다시 연결되는 PowerPoint connector는 아니다. draw.io에는 실제 source/target 연결 참조가 있다.

## 편집 및 재생성 원본

[설계 문서와 현재 그림 목록](../../architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/README.md)에서 SVG, draw.io, PNG를 함께 연다. [공통 scene 모델](../../architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/diagrams/dp3-v8-scene-model.json)은 생성 결과이며, 아래 페이지 정의에서 구조를 수정한다.

- [10쪽 정의](../../../scripts/presentations/dp3_v8_grounding_page.mjs)
- [11~12쪽 정의](../../../scripts/presentations/dp3_v8_execution_pages.mjs)
- [scene 및 연결 검사](../../../scripts/presentations/dp3_v8_scene.mjs)
- [PPTX/SVG/draw.io 생성기](../../../scripts/presentations/build_dp3_cpu_v8.mjs)
- [26쪽 보존 패키징](../../../scripts/presentations/package_dp3_cpu_v8.py)

생성기는 repository와 새 private build directory를 인자로 받는다. build에 v7 파일을 `source-v7.pptx`로 복사하고 위 네 `.mjs` 파일을 같은 디렉터리에 둔다. bundled Node, Python, `@oai/artifact-tool`과 같은 경로의 `node_modules`를 사용하는 로컬 환경을 전제로 한다. `DP3_RENDER_ALL=1`은 전체 29쪽을 렌더하고, 기본값은 변경한 세 쪽을 렌더한다. `DP3_EXPORT_FIGURES_ONLY=1`은 같은 scene에서 SVG/draw.io만 다시 내보내며 PPTX를 변경하지 않는다. 구조를 바꾸면 전체 생성과 렌더 검토를 다시 수행한다.
