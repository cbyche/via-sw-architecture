# 시나리오와 관측 모집단

> **2026-10-23 QA07 v4 반영:** 현재 QA07은 **업무 순서 및 연결 정확도**다. 사용자 지정 선후 관계와 NEW/EXISTING 및 기존 업무 ID의 최초 판단을 평가한다. 이름과 범위는 사용자 확정, **85%·28입력 × 5회(140회)·3%p 점수는 작성자 제안**이다. 공통 v9에 별도 매핑과 관측 전 평가표를 두었다. 실제 실행 0회, 결과 미산출이며 기존 v7/v8은 보존한다. v8의 응답 구성·전달 범위와 90%·160회는 이전 제안이다.

> **2026-10-23 QA 구체화:** [대표값/부록 계약](../qa/measurement-appendix.md)과 [QA04의 8개 변경 과제](../qa/change-scenarios.md)를 적용한다. QA01은 P95 없이 평균 2초와 NORMAL/WAIT 분리, QA02/QA07은 각각 단일 비율로 별도 유지한다. QA06은 전체 제품 예산이다. QA04-CHANGE-v4는 Rust/Python 등 실제 운영 모듈을 세며 v6에 보존된 기존 6과제의 추정값과 호환되지 않는다. 새 QA04 값은 미산출이며 기존 평가 파일과 PPTX는 보존한다. 후속 담당 작업은 [구체화 전달 계약](../qa/refinement-handoff.md)을 따른다.

> **이전 범위 QA07 판본 구분:** [보존 QA07 v3](../qa/history/qa07-before-order-connection-20261023/qa07-request-meaning-accuracy.md.txt)는 32사례 × 5회와 목표/기한/점수의 작성 완료 제안이다. [v8 사례표](../evaluation/common-qa-v8/02-qa07-cases.md)에 계획 기록을 두며 실제 실행은 없다. 기존 24입력/24묶음과 다른 모집단이다.

> **보존 v6:** [QA07 사례 매핑](../evaluation/common-qa-v6/02-qa07-case-mapping.md)과 [의무 manifest](../evaluation/common-qa-v6/qa07-case-manifest.json)를 추가했다. 24입력을 사례 검토안에 연결했으며 새 확정 분모로 쓰지 않는다.

> **보존 v5:** 입력별 VIA/Agent 책임, 음성42예정 사건의 NORMAL/WAIT 분류와24목표 완료 의무를 명시했다. [현재 계약](../qa/timing-measurement-contract.md)을 적용하며 이전 v4/v3는 보존한다.

현재 네 DP의 가상 평가와 과거 목표 도출, 회귀 카탈로그를 이곳에서 구별해 연다. 이름에 모두 공통이 들어가더라도 같은 corpus로 합치지 않는다.

| corpus와 역할 | 사례와 관측 단위 | 문서와 원장 |
| --- | --- | --- |
| 현재 `COMMON-QA-MODEL-9.0` | QA07의 원전 84사용자 입력 중 28선정/56제외, 5반복=140회. 순서 있음2/없음26. 원전 음성/Text는 출처로 보존, 대표 시험은 모두 Text. 실제0 | [28입력과 제외 원장](../evaluation/common-qa-v9/02-qa07-cases.md), [계약](../evaluation/common-qa-v9/01-contract.md) |
| 이전 QA07 범위 `COMMON-QA-MODEL-8.0` | QA07 v3의 32사례 × 5회/구성, 계획 1,120기록 모두 UNKNOWN. 다른 QA는 v7 유지 | [32사례](../evaluation/common-qa-v8/02-qa07-cases.md), [계약](../evaluation/common-qa-v8/01-contract.md) |
| 보존 `COMMON-QA-MODEL-7.0` | 시간/비용/자원은 기존 24입력·7구성 유지. QA04 새 8과제의 56기록 모두 미측정 | [계약](../evaluation/common-qa-v7/01-contract-and-scope.md), [8과제](../evaluation/common-qa-v7/02-qa04-changes.md), [부록](../evaluation/common-qa-v7/03-appendix.md) |
| 보존 `COMMON-QA-MODEL-6.0` | v5 원값과 QA07 24묶음 검토안, 7구성의168 작성 슬롯, 실제 실행0 | [사례/의무](../evaluation/common-qa-v6/02-qa07-case-mapping.md), [판정 계약](../evaluation/common-qa-v6/01-qa07-contract.md) |
| 보존 `COMMON-QA-MODEL-5.0` | 24입력/24목표/33지칭/6변경 과제, 7구성. 음성42예정: NORMAL36, WAIT6 중5출력·1철회 | [입력과 역할](../evaluation/common-qa-v5/02-configurations-and-scenarios.md), [명세](../evaluation/common-qa-v5/specification.json), [계약](../evaluation/common-qa-v5/01-common-contract.md) |
| 보존 `COMMON-QA-MODEL-4.0` | 같은 24입력, 24목표, 33지칭, 6변경 과제, 7구성, 하나의 환경 | [v4 사례](../evaluation/common-qa-v4/02-configurations-and-scenarios.md), [명세](../evaluation/common-qa-v4/specification.json), [평가자 정답](../evaluation/common-qa-v4/oracles.json), [계약](../evaluation/common-qa-v4/01-common-contract.md) |
| 보존 `COMMON-QA-CORPUS-3.0-STRATIFIED` | 26개 사례군, Text, Voice/Screen, Delivery의 세 profile, 9구성 | [v3 구성과 시나리오](../evaluation/common-qa-v3/02-configurations-and-scenarios.md), [계약](../evaluation/common-qa-v3/01-common-contract.md), [통합 JSON](../evaluation/common-qa-v3/artifacts/normalized-data.json), [HTML 탐색기](../evaluation/common-qa-v3/artifacts/outputs/01a15dfe/guide.html) |
| 보존된 `COMMON-28-v1` | 대표 에피소드 28개와 지정 반복, 기능 회귀 F01~F18, 변경 C01~C06 | [명세](common-28-v1/common-scenarios.md), [JSON](common-28-v1/common-scenarios.json), [통합 경위](common-28-v1/common-evaluation-population-plan.md) |

v4는 같은 입력을 모든 구성에 적용하는 분석 모델이며 QA02/QA07의 실제 최초 채택 출력은 없다. 준비 활동을 정확도 분모에 더하지 않는다. 24개 사례는 실제 WAV/화면이 갖춰진 실행 fixture나 독립 통계 반복이 아니다. v3의 원값·분모·고정 정답 조건을 v4 결과로 옮기지 않는다.

## 보존 v3의 분모

| profile | 사례군 | 평가 입력/구성 | 지칭 표현/구성 | 목표/구성 | 별도 준비 활동 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Text 이력 | 12 | 240 | 250 | 180 | 관계 생산, 종료 후 갱신을 활동 원장에 포함 |
| Voice/Screen | 6 | 120 | 120 | 120 | 60 |
| Delivery | 8 | 80 | 120 | 90 | 80 |

이전 v3 계약에서는 QA01과 QA07은 입력, QA02는 표현, QA03은 목표를 분모로 사용했다. 현재 QA07 v4는 업무 요청 입력×반복을 단위로 하며 별도 28입력 계획을 따른다. QA04는 각 구성에 동일 C01~C06 한 벌을 적용한다. QA05는 해당 profile의 준비, 갱신, 폐기까지 포함한 활동 묶음 비용이고, QA06은 동시 상태 최고값이다. 준비 활동을 입력 정확도의 분모에 더하지 않는다.

26개 사례군은 실제 모델 반복 횟수나 독립 통계 표본 수가 아니다. 다른 profile의 입력, 자원, 완료 조건은 고정된 서로 다른 가정이므로 수치와 DP 개선량을 합산하지 않는다. 현재 자료는 모두 `SYNTHETIC_AUTHORED_NOT_MEASURED`다.

COMMON-28의 목표, 점수, 비용 도출을 v3의 새 사례군에 자동 적용하지 않는다. 이관한 COMMON-28 JSON은 원본 바이트를 보존하며 평가자 정답을 포함하므로 후보 입력으로 통째로 제공하지 않는다. 문서 원전의 `qa_candidate_population.py` 등 기존 명령은 원래 경로를 대상으로 하므로 현재 편집 명령으로 사용하지 않는다. 후속 corpus 변경은 새 판본과 같은 조건의 재산출을 명시하며 [보존, 편집 규칙](../evaluation/common-qa-v3/migration-and-editing.md)을 따른다.
