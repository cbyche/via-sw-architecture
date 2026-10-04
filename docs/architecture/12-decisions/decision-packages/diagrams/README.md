# Decision Reconstruction 그림

## 33 발표용 A/B/C 판단 책임 비교

[메인 SVG](./choice33-structure.svg) / [draw.io](./choice33-structure.drawio) / [PNG](./choice33-structure.png) / [본문과 발표 대본](../04-33-dialogue-coordination.md). 2560×1440의 한 장에 같은 입력/정정, A의 통합 판단, B의 공통 대화와 Task별 담당, C의 기능별 부분 판단, 상태/교환/공통 채택 및 조건부 손익을 표시한다. 세 열은 동시에 실행하는 시스템이 아닌 대안이다.

[보충 사건도 SVG](./choice33-event.svg) / [draw.io](./choice33-event.drawio)는 교차 사건과 예외를 확인하는 자료다. 메인 그림을 이해하기 위한 선행 자료가 아니다. 생성 scene은 `scripts/architecture/dialogue_abc_presentation.py`, 검사 entry point는 `generate_six_comparison_diagrams.py --check`다. 분해는 process/Omni 가중치 복제를 뜻하지 않으며 품질 우위는 미측정이다.

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
