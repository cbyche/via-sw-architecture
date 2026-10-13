# Decision Reconstruction 그림

## 47 화면과 음성의 지칭 결합 구조

[MAIN SVG](./choice47-structure.svg) / [draw.io](./choice47-structure.drawio) / [PNG](./choice47-structure.png) / [설계](../04-47-screen-voice-grounding.md). 같은 음성, Windows 관측에서 A의 공동 원본 지칭 해석과 B의 생산자별 후보, Coordinator, 의미 결합, 재판단을 비교한다. 원본과 부분 정정은 양안 공통이며 process 수, 모델 수나 승자 비교가 아니다.

모든 DP-47 그림은 [47 본문](../04-47-screen-voice-grounding.md#0-이-문서에서-모든-dp-47-그림을-확인한다)에 직접 표시한다. 새 그림도 본문 목록과 해당 절에 함께 등록한다.

생성기는 `scripts/architecture/generate_screen_voice_grounding_diagram.py`, 동기화 및 본문 삽입 검사는 동일 명령의 `--check`, renderer는 `render_screen_voice_grounding_diagram.cjs`다. 동일 scene에서 editable SVG/draw.io를 생성하며 2560×1440 PNG를 Chrome에서 검사한다. 자세한 확인 범위는 [작성자 리뷰](../dp47-design-review.md)를 따른다. 기존 PPTX나 다른 DP 그림을 교체하지 않는다.

## 45 과거 이력 정의와 같은 기록의 재사용 설명

[45 본문 §0](../04-45-memory-and-context.md#0-먼저-정의할-것-과거-이력에서-현재-context까지)에서 설명을 시작한다.

- [과거 이력 정의 SVG](./dp45-history-definition.svg) / [draw.io](./dp45-history-definition.drawio) / [PNG](./dp45-history-definition.png): 다섯 원본 기록 종류, 파생 자료, 현재 Context와 별도 User Memory를 구별한다. 자료의 분류이며 새로운 Component나 물리 DB 세 개가 아니다.
- [같은 기록의 A/B 흐름 SVG](./dp45-history-walkthrough.svg) / [draw.io](./dp45-history-walkthrough.drawio) / [PNG](./dp45-history-walkthrough.png): 같은 실제 설명 구간과 두 결과 참조에서 A의 현재 조합과 B의 별도 관계 생산/게시/조회를 보여준다. 현재 의미와 채택은 양안 공통이며 생산 대기/미지원도 표시한다.

생성: `.venv/bin/python scripts/architecture/generate_dp45_history_diagrams.py`. 검사: 같은 명령의 `--check`. 두 그림은 2560×1440이며 같은 scene에서 SVG/draw.io를 생성한다. PNG는 Chrome의 실제 렌더 preview다. 기존 MAIN과 수명, 통합 PPTX는 이 보충 설명 작업으로 교체하지 않는다. 작성자 검수는 렌더와 문서 일치 범위이며 모델 의미, 품질 우위나 최종 선택의 검증은 아니다.

## 43 발표용 A/C 판단 책임 비교

[메인 SVG](./choice43-structure.svg) / [draw.io](./choice43-structure.drawio) / [PNG](./choice43-structure.png) / [본문과 대본](../04-43-request-interpretation.md). 2560×1440의 한 장에서 F1 의도, F2 대상, F3 연결, F4 관계, F5 정정, F6 처리 방향을 A는 통합 Module, C는 기능별 Module로 생산한다. 실제 SW 설계 이름과 임시 상태/교환/공통 채택을 표시한다. 기존 Task별 B는 제외하며 예시 업무명을 Module 이름으로 사용하지 않는다.

[보충 정정 사건도 SVG](./choice43-event.svg) / [draw.io](./choice43-event.drawio). 생성 scene은 `scripts/architecture/generate_request_interpretation_diagrams.py`, 검사는 `generate_request_interpretation_diagrams.py --check`다. 공유 모델 조건과 미측정 상태를 유지하며 여섯 기능은 고정 호출 횟수나 독립 process 개수가 아니다.

33 도식과 `dialogue_ac_presentation.py`는 이관 시점의 이력으로 유지한다. 최신 개선은 43에만 적용한다. [33 보존 문서](../04-33-dialogue-coordination.md).

## 42 발표용 수명 구조 비교

[전체 보기](./choice42-review.html) / [설계 문서](../04-42-lifecycle-ownership.md). 2560×1440의 16:9 다섯 장이며 각 SVG에 편집 가능한 draw.io를 짝으로 둔다.

- [메인 구조 비교](./choice42-structure.svg) / [draw.io](./choice42-structure.drawio): Ⅰ 공통 사례의 서로 다른 수명, Ⅱ 상태 소유·process 경계, Ⅲ 공동 저장/독립 접수의 협력 방식. 같은 C1/R1/Q1/T1/X1/X2를 수명 축과 소유 영역에 표시.
- [수명 시간축](./choice42-lifetimes.svg) / [draw.io](./choice42-lifetimes.drawio): 같은 E1~E6 사례에서 기록별 수명.
- [A 사건도](./choice42-event-a.svg) / [draw.io](./choice42-event-a.drawio), [B 사건도](./choice42-event-b.svg) / [draw.io](./choice42-event-b.drawio): 같은 E4 답변의 현재 검사·확정·외부 접수·실제 전달.
- [변경 영향 비교](./choice42-change.svg) / [draw.io](./choice42-change.drawio): 일반 protocol 변경의 공통 국소화와 독립 운영의 조건부 차이.

생성기는 `scripts/architecture/generate_lifecycle_ownership_diagrams.py`다. 아래는 기존 도식 세대의 기록이며 42의 수량/의미와 합치지 않는다.

## 기존 비교 도식

4개 후보 × 배경·설계 비교 각 1장 = **8장**, 각 장에 `.svg`와 편집 가능한 `.drawio`를 함께 둔다. [전체 브라우저 보기](./review.html), 설명과 비교표는 [상위 README](../README.md)를 따른다.

- 검정: 양안 공통 Component·책임. 파랑: 양안의 추가·제거·대체 대상. 2안 우수 표시 아님.
- 큰 실선: 논리 Component/서브시스템. 큰 점선: process. 원통: 저장소. 내부 상자는 구현 책임의 확대도이며 모두 독립 target Component를 뜻하지 않음.
- 검색은 생산·소비 계층, workflow는 실행체와 활동, 음성은 병렬 runtime 경계, 복구는 정상 commit과 재구축 체계로 각각 배치한다. 동일 개수의 격자에 억지로 맞추지 않는다.
- 2560×1440의 16:9 원본이다. SVG의 확대 보기와 draw.io 원본으로 발표자료를 편집할 수 있다. Markdown의 표는 별도 원고이며 SVG에는 핵심 이득·대가만 넣었다.
- 현재 본문·그림은 구조 설명이다. 구현 실행·성능 측정·native model capability 검증 결과가 아니다.

생성: `.venv/bin/python scripts/architecture/generate_decision_package_diagrams.py` (저장소 root). 검사: 같은 명령의 `--check`. 단일 scene에서 양 파일을 만들며 draw.io의 부모·연결 endpoint를 보존한다. generator 수정 시 양 파일을 재생성하고 전 SVG를 렌더링한다. 브라우저 glyph·시각 검토와 구조 독립 검토는 자동 geometry 검사와 별개다. draw.io 편집기 자체의 화면 렌더 호환은 별도 확인 대상이다.

## 45 방안별 구현 수준의 설계도

[45 본문 §3.10](../04-45-memory-and-context.md#310-구현-수준의-sw-architecture-두-안을-각각-따라가기)에 두 그림을 직접 표시하고 번호별 호출/완료와 상태/복원 계약을 설명한다.

- [A SVG](./dp45-implementation-a.svg) / [draw.io](./dp45-implementation-a.drawio) / [PNG](./dp45-implementation-a.png): 요청별 원본 조회/완료, 유효 cache, 근거 집합과 사용 검사.
- [B SVG](./dp45-implementation-b.svg) / [draw.io](./dp45-implementation-b.drawio) / [PNG](./dp45-implementation-b.png): 독립 생산 job/후보/게시 상태, Coverage 조회와 미생산 후 재조회.

두 장은 3840×2160이며 `generate_dp45_implementation_diagrams.py`가 같은 scene으로 SVG/draw.io를 생성한다. `render_dp45_implementation_diagrams.cjs`는 Chrome 글자/경로 배치 검사와 PNG, `dp45-implementation-render.json`을 만든다. 생성기의 `--check`는 생성물, 소유/경로, 본문 preview와 렌더 해시를 함께 검사한다. 제품 실행이나 모델 검증 자료가 아니다.
