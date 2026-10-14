# DP-45 구현 구조 한 페이지 비교

2026-10-15 추가한 **편집 가능한 1장**이다. 기존 비교 페이지와 A/B별 상세 그림, 통합 발표자료는 보존한다.

[PowerPoint 1장](./VIA-DP45-implementation-comparison.pptx) / [PPTX 렌더 미리보기](./pptx-preview.png) / [본문에서 SVG와 전체 계약 읽기](../../architecture/12-decisions/decision-packages/04-45-memory-and-context.md#dp45-fig-implementation-comparison)

같은 원본과 같은 현재 Query에서 A의 읽기 계획/원본 I/O/조합, B의 생산 job/후보/검증/게시/현재 조회를 비교한다. 저장 원통은 현재 read RAM, 재사용 cache, 지속 생산 job과 게시 자료를 구별한다. 번호는 본문 §3.10.7에 대응하며 S/M/U는 각각 입력 이력 read, 모델 연동, 현재 사용 검사 port다. 설계 제안이며 구현이나 품질 측정 결과는 아니다.

PPTX의 모든 코드/상태 상자, 글자와 선은 native 도형이다. Component와 Module 이름은 해당 도형의 text body에 들어 있다. Calibri, 정수 pt, 한국어 문서/텍스트 기본 설정을 적용한다. [기존 통합 v13](../VIA-architecture-presentation_v13.pptx)은 이 작업에서 수정하지 않았다.

## 재생성

[scene 생성기](../../../scripts/architecture/generate_dp45_implementation_comparison.py)가 SVG/draw.io를 같은 장면으로 생성한다. 저장소 root에서 `.venv/bin/python scripts/architecture/generate_dp45_implementation_comparison.py --json <private-build>/scene.json`을 실행한다. [Chrome 렌더 도구](../../../scripts/architecture/render_dp45_implementation_comparison.cjs)는 PNG와 해시를 갱신한다. 생성기의 `--check`는 도형 소유와 경로, 원본 일치, 본문 삽입 및 렌더 해시를 검사한다.

[PowerPoint 작성기](../../../scripts/presentations/build_dp45_implementation_comparison.mjs)를 새 private build로 복사하고 bundled `node_modules`를 연결한다. `RUNTIME_NODE_MODULES`, `VIA_RUNTIME_PYTHON`, `VIA_PRESENTATION_SKILL_DIR`에 bundled dependency와 Presentations skill의 절대 경로를 지정한 후 bundled Node로 `<작성기> <repo> <private-build>`를 실행한다. finalizer를 통과한 `final/VIA-DP45-implementation-comparison.pptx`를 바이트 변경 없이 이 폴더에 복사한다. 기존 산출물을 덮어쓰는 재생성은 별도 판본으로 수행한다.

SVG의 글자/선 geometry 검사, PPTX package/layout/font와 재불러오기, 실제 렌더 확인은 문서/그림 검수다. 제품 구현, 모델 호출, QA 실측 또는 PowerPoint 앱 자체의 렌더 검증을 뜻하지 않는다. 게시 이후 Windows Open XML SDK 및 CI 상태는 해당 commit과 최종 응답에서 확인한다.
