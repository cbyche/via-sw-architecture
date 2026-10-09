# 04-46 발표자료: 시간·과거 근거의 요청별 결합과 공통 생산

[편집 가능한 PowerPoint](./VIA-DP46-evidence.pptx) / [원본 문서](../../architecture/12-decisions/decision-packages/04-46-input-and-context-evidence.md) / [그림 검토 페이지](../../architecture/12-decisions/decision-packages/diagrams/choice46-review.html)

순서는 배경 → 전체 구조 → 조건별 QA → 정정 사건 → 상태 수명이다. 모든 slide의 도형·선·텍스트는 편집 가능하다. 구조와 예시는 같은 source scene에서 SVG/draw.io/PPTX로 생성한다. 41~45 통합 발표의 기존 slides를 바꾸지 않는다.

| 페이지 | GitHub 미리보기 / 편집 원본 |
| --- | --- |
| 배경 | [SVG](../../architecture/12-decisions/decision-packages/diagrams/dp46-background.svg) / [draw.io](../../architecture/12-decisions/decision-packages/diagrams/dp46-background.drawio) |
| MAIN 구조 | [SVG](../../architecture/12-decisions/decision-packages/diagrams/choice46-structure.svg) / [draw.io](../../architecture/12-decisions/decision-packages/diagrams/choice46-structure.drawio) |
| 조건별 QA | [SVG](../../architecture/12-decisions/decision-packages/diagrams/choice46-quality.svg) / [draw.io](../../architecture/12-decisions/decision-packages/diagrams/choice46-quality.drawio) |
| 정정 사건 | [SVG](../../architecture/12-decisions/decision-packages/diagrams/choice46-timeline.svg) / [draw.io](../../architecture/12-decisions/decision-packages/diagrams/choice46-timeline.drawio) |
| 상태 수명 | [SVG](../../architecture/12-decisions/decision-packages/diagrams/choice46-lifecycle.svg) / [draw.io](../../architecture/12-decisions/decision-packages/diagrams/choice46-lifecycle.drawio) |

MAIN은 양안 같은 Component·원본·입출력 위치에서 실제 결합 주체와 상태 수명, 생산→게시→조회 여부의 차이를 보여준다. 큰 경계는 흰색, 차이 Module/상태는 살구색이며 공통 범례를 사용한다. 시간 후보는 현재 지칭/Task 정답이 아니다. 46 전체 A는 시간 A+45A, 전체 B는 시간 B+45B이고 교차 조합은 별도 계약이다.

QA 페이지는 정확성·응답속도·모델 사용 비용·변경 용이성·로컬 메모리의 **조건부 가설**이다. 숫자 우열·선택 승자·구현/실측 결과를 뜻하지 않는다. 시간 관계는 코드이며 모델 요금 이익은 실제 과거 의미 재사용 경로가 있을 때만 기대한다. 원본 보존·현재성·권한 확인과 A cache/부분 갱신은 유지한다.

생성기는 `scripts/presentations/generate_dp46_editable_pptx.mjs`이며 bundled Node의 node_modules에 연결한 별도 build 디렉터리에서 실행한다. 환경 변수 `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`, `RUNTIME_NODE_MODULES`에 설치된 절대 경로를 지정한다. 같은 source의 SVG/draw.io는 `python3 scripts/architecture/generate_evidence_context_diagrams.py --check`, 시간 예시는 `python3 scripts/architecture/check_dp46_temporal_examples.py`, PPTX source/hash/text/비평면화는 `python3 scripts/presentations/check_dp46_presentation.py`로 확인한다. 브라우저 PNG는 `render_evidence_context_diagrams.cjs`의 hash/geometry 기록을 따른다.

실제 브라우저 SVG와 native PPTX 재import 렌더를 시각 확인했다. source 일치·도형/텍스트·font/geometry/integrity 검사는 구현, provider word timing 지원, 의미 정확성이나 QA 우열의 증명이 아니다. draw.io native 편집기 이동/저장의 round-trip은 확인하지 않았다.
