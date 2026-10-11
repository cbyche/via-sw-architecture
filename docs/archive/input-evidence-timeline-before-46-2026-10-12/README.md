# 46 이전 관측 타임라인 초안 보존

> 상태: **HISTORICAL_EXPLORATION / 현재 설계나 주요 DP 아님** / 보관일: 2026-10-12

2026-10-09 작성한 발화/화면/사용자 행동의 관측 타임라인 초안이다. 중앙 공통 원본 기록과 생산자별 원본 기록의 조합을 비교했지만, 사용자가 다른 DP와 비교축이 유사하다고 지적해 별도 주요 DP로 진행하지 않았다. 후속 확장 검토는 [46 본문](../../architecture/12-decisions/decision-packages/04-46-input-and-context-evidence.md)과 [46 검수](../../architecture/12-decisions/decision-packages/04-46-evidence-review.md)에 있다.

이 초안은 46 게시 당시 제외되어 활성 문서/도구 경로에 untracked로 남아 있었다. 2026-10-12 사용자가 다른 세션의 혼동을 막기 위해 archive 이동과 commit/push를 요청했다. 문서 2개, SVG/draw.io/PNG/render 기록 4개, 전용 생성/렌더 도구 2개를 원래 저장소 경로 구조로 옮겼다. [manifest](./manifest.json)에 원래 경로, 크기와 SHA-256을 기록했고 여덟 파일의 원본 bytes를 그대로 보존했다.

- [초안 본문](./docs/architecture/12-decisions/decision-packages/input-evidence-timeline-review.md)
- [당시 검수 기록](./docs/architecture/12-decisions/decision-packages/input-evidence-timeline-review-notes.md)
- [SVG](./docs/architecture/12-decisions/decision-packages/diagrams/input-evidence-timeline-structure.svg) / [draw.io](./docs/architecture/12-decisions/decision-packages/diagrams/input-evidence-timeline-structure.drawio) / [PNG](./docs/architecture/12-decisions/decision-packages/diagrams/input-evidence-timeline-structure.png)
- [당시 생성기](./scripts/architecture/generate_input_evidence_timeline_diagram.py) / [당시 렌더 도구](./scripts/architecture/render_input_evidence_timeline_diagram.cjs)

보존본 안의 상대 링크와 실행 명령은 작성 당시 경로를 따른다. 현재 경로에서 실행할 도구가 아니며 일부 외부 참조는 보존본 내부에 없다. 현재 46 문서/그림/생성기는 이 초안에 의존하지 않는다. 이번 정리는 현재 Architecture, 46의 설계/발표, QA, 구현이나 측정 상태를 변경하지 않는다.
