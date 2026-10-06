# VIA DP 설계 비교 발표 틀 — 41~45

[전체 미리보기](./index.html) / [5페이지 draw.io 편집 원본](./VIA-DP-comparison-41-45.drawio)

사용자가 제공한 DP01~DP04 방안 PNG의 **좌우 설계안, 구조, 장점, 단점, QA Trade-off** 행 구성을 따른 1920×1080 발표 틀이다. 기존 배경과 제목 및 서체를 맞추고 설계안 행은 레퍼런스의 연녹색을 사용했다. 각 DP의 실제 구조를 축약하여 넣었으며, 43은 기존 A/C 표기를 유지한다. 구조 요약은 실행 코드나 전체 호출 sequence가 아니다. 모든 도형, 이름, 선, 수치와 원형 점수는 draw.io에서 편집 가능하다.

| DP | 비교 | QA 세 항목 | 편집 원본 | SVG | PNG |
| --- | --- | --- | --- | --- | --- |
| 41 | 모델 중심 ReAct / 모델 틀과 코드 완성 | V-03 기능 완전성, V-05 요청 완료 신속성, V-08 변경 용이성 | [draw.io](./dp41-comparison.drawio) | [SVG](./dp41-comparison.svg) | [PNG](./dp41-comparison.png) |
| 42 | 통합 Core / 독립 대화 및 업무 서비스 | V-05 요청 완료 신속성, V-08 변경 용이성, V-06 메모리 효율성 | [draw.io](./dp42-comparison.drawio) | [SVG](./dp42-comparison.svg) | [PNG](./dp42-comparison.png) |
| 43 | 통합 의미 생산 / 기능별 의미 생산과 코드 조정 | V-01 기능 정확성, V-05 요청 완료 신속성, V-08 변경 용이성 | [draw.io](./dp43-comparison.drawio) | [SVG](./dp43-comparison.svg) | [PNG](./dp43-comparison.png) |
| 44 | 중앙 비동기 조정 / 반응형 Dataflow | V-04 응답 신속성, V-08 변경 용이성, V-06 메모리 효율성 | [draw.io](./dp44-comparison.drawio) | [SVG](./dp44-comparison.svg) | [PNG](./dp44-comparison.png) |
| 45 | 원본 서비스 조합 / 공통 파생 기억 생산과 조회 | V-01 기능 정확성, V-05 요청 완료 신속성, V-06 메모리 효율성 | [draw.io](./dp45-comparison.drawio) | [SVG](./dp45-comparison.svg) | [PNG](./dp45-comparison.png) |

## 수치와 원형 점수

사용자 지시에 따라 예상 수치와 점수를 채웠다. **형식을 검토하기 위한 작성자 가정이며 실측, 통계 추정 또는 선정 결과가 아니다.** 각 장에 예상 예시와 미측정을 표시했다. 장점은 해당 예상값에서 이기는 항목, 단점은 지는 항목을 원인 및 수치와 함께 한 줄로 설명하고 마지막에 QA 이름과 품질의 ↑/↓를 붙였다.

- QA 이름과 대표 지표는 2026-10-06의 [품질속성 발표자료](../../presentations_files/quality-attributes.md)를 사용했다. V ID의 원래 범위는 [03-00 품질 시나리오](../../architecture/12-decisions/decision-packages/03-00-quality-scenarios.md)를 따른다. 발표 지표 및 목표를 새 Architecture 계약으로 채택한 것이 아니다.
- Trade-off의 ↑/↓는 **수치의 좋은 방향**이다. 장단점 괄호의 ↑/↓는 **품질의 상대적인 향상과 저하**다. 예를 들어 시간이 짧으면 요청 완료 신속성 ↑다.
- ●●●, ●●○, ●○○는 작성자의 상대적인 3단계 예시다. 같은 행에서 유리한 수치에 더 많은 점을 배정했다. 공식 score band, 목표 달성 판정, 가중 총점은 아니다.
- V-01은 정확 처리율, V-03은 기능 지원 달성률, V-04는 p95 반응 지연, V-05는 p95 VIA 완료시간, V-08은 평균 변경 요소 수, V-06은 최대 전체 메모리다. V-05에서 외부 Agent 작업시간은 제외한다.
- 41의 지원율은 94개 변형 중 A 94개, B 82개라는 형식 가정이다. 실제 어떤 변형이 미지원인지 판정한 결과는 아니다. 시간 비교는 양안이 지원하는 동일 Request의 가정이며 전체 기능 모집단의 QA 통과를 뜻하지 않는다. 지원 손실은 별도 V-03 행에서 표시한다.
- 42의 변경 예시는 업무 저장 및 실행 수명 계약의 독립 변경이다. 일반 Agent adapter 변경도 B만 국소화할 수 있다고 주장하지 않는다.
- 43의 수치는 교차 의미 정정과 특정 지칭 기능 변경을 가정한다. C의 분해 자체가 정확성 개선을 보장하지 않는다.
- 44의 반응 지연은 집중 도착 사건의 유효 응답까지의 가정이다. 두 안의 로컬 음성 stop는 동일하며 B의 이익으로 계산하지 않는다. 같은 schema 안에서 단계 조합을 변경하는 경우를 가정한다.
- 45는 게시된 관계의 반복 재사용이 유효한 경우다. A의 index, cache 및 요약을 허용하고 B의 관계 추출 오류와 미게시 범위는 기존 본문에 남긴다. 항상 B가 정확하거나 빠르다는 뜻은 아니다.
- 메모리 예상값은 동일한 공유 Omni 1벌, ASR, 역할별 KV와 공통 기능을 포함한 전체값이다. 추가 서비스나 관계 생산 및 buffer 비용만 조건에 따라 다르다. 실제 장치 메모리 계산이나 profiling 결과는 아니다.

## 편집과 재생성

통합 draw.io에서 페이지를 골라 수정하거나 개별 파일을 복제해 같은 틀을 재사용한다. 구조 영역 약 46%, 하단 장단점 및 QA 영역 약 27%의 구성이다. 레퍼런스의 진행 단계 ribbon과 FR 번호는 VIA 자료에 해당 값이 없어 복사하지 않았다.

생성 원본은 [generate_dp_comparison_slides.py](../../../scripts/architecture/generate_dp_comparison_slides.py)다. 상단 `DATA`가 예시, 장단점, QA와 점수를, `Comparison`이 공통 행 틀을, `graph41`~`graph45`가 구조 요약을 정의한다.

```sh
python3 scripts/architecture/generate_dp_comparison_slides.py
node scripts/architecture/render_dp_comparison_slides.cjs
python3 scripts/architecture/generate_dp_comparison_slides.py --check
```

렌더러는 Playwright와 Chromium을 사용한다. 필요한 경우 `VIA_PLAYWRIGHT_MODULE`에 Playwright 모듈 경로, `VIA_CHROMIUM_PATH`에 브라우저 실행 파일을 지정한다. PNG 수정 없이 SVG만 바꾸면 해시 검사가 실패한다. draw.io 직접 편집 후에는 같은 PNG/SVG도 내보내고 생성 scene에도 변경을 반영한다.

[검수 기록](./REVIEW.md) / [예상값 데이터](./comparison-data.json) / [렌더 해시](./render-manifest.json)
