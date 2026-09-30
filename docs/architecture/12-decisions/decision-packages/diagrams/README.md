# Decision Reconstruction 그림

4개 후보 × 배경·설계 비교 각 1장 = **8장**, 각 장에 `.svg`와 편집 가능한 `.drawio`를 함께 둔다. [전체 브라우저 보기](./review.html), 설명과 비교표는 [상위 README](../README.md)를 따른다.

- 검정: 양안 공통 Component·책임. 파랑: 양안의 추가·제거·대체 대상. 2안 우수 표시 아님.
- 큰 실선: 논리 Component/서브시스템. 큰 점선: process. 원통: 저장소. 내부 상자는 구현 책임의 확대도이며 모두 독립 target Component를 뜻하지 않음.
- 검색은 생산·소비 계층, workflow는 실행체와 활동, 음성은 병렬 runtime 경계, 복구는 정상 commit과 재구축 체계로 각각 배치한다. 동일 개수의 격자에 억지로 맞추지 않는다.
- 2560×1440의 16:9 원본이다. SVG의 확대 보기와 draw.io 원본으로 발표자료를 편집할 수 있다. Markdown의 표는 별도 원고이며 SVG에는 핵심 이득·대가만 넣었다.
- 현재 본문·그림은 구조 설명이다. 구현 실행·성능 측정·native model capability 검증 결과가 아니다.

생성: `.venv/bin/python scripts/architecture/generate_decision_package_diagrams.py` (저장소 root). 검사: 같은 명령의 `--check`. 단일 scene에서 양 파일을 만들며 draw.io의 부모·연결 endpoint를 보존한다. generator 수정 시 양 파일을 재생성하고 전 SVG를 렌더링한다. 브라우저 glyph·시각 검토와 구조 독립 검토는 자동 geometry 검사와 별개다. draw.io 편집기 자체의 화면 렌더 호환은 별도 확인 대상이다.
