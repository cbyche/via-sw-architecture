# 시나리오와 관측 모집단

> **현재 QA 정의와의 차이:** [QA01/QA03 개정 계약](../qa/timing-measurement-contract.md)은 2026-10-23 NORMAL/WAIT 분리와 목표/점수까지 합의됐다. 아래 기존 데이터는 당시 경계 그대로이며 새 음성 QA01이나 완료 경계의 결과가 아니다. 사용자가 담당 세션의 새 판본 갱신을 요청했으며 전달 및 진행 상태는 해당 계약을 따른다.

현재 네 DP의 가상 평가와 과거 목표 도출, 회귀 카탈로그를 이곳에서 구별해 연다. 이름에 모두 공통이 들어가더라도 같은 corpus로 합치지 않는다.

| corpus와 역할 | 사례와 관측 단위 | 문서와 원장 |
| --- | --- | --- |
| 현재 `COMMON-QA-MODEL-4.0` | 같은 24입력, 24목표, 33지칭, 6변경 과제, 7구성, 하나의 환경 | [v4 사례](../evaluation/common-qa-v4/02-configurations-and-scenarios.md), [명세](../evaluation/common-qa-v4/specification.json), [평가자 정답](../evaluation/common-qa-v4/oracles.json), [계약](../evaluation/common-qa-v4/01-common-contract.md) |
| 보존 `COMMON-QA-CORPUS-3.0-STRATIFIED` | 26개 사례군, Text, Voice/Screen, Delivery의 세 profile, 9구성 | [v3 구성과 시나리오](../evaluation/common-qa-v3/02-configurations-and-scenarios.md), [계약](../evaluation/common-qa-v3/01-common-contract.md), [통합 JSON](../evaluation/common-qa-v3/artifacts/normalized-data.json), [HTML 탐색기](../evaluation/common-qa-v3/artifacts/outputs/01a15dfe/guide.html) |
| 보존된 `COMMON-28-v1` | 대표 에피소드 28개와 지정 반복, 기능 회귀 F01~F18, 변경 C01~C06 | [명세](common-28-v1/common-scenarios.md), [JSON](common-28-v1/common-scenarios.json), [통합 경위](common-28-v1/common-evaluation-population-plan.md) |

v4는 같은 입력을 모든 구성에 적용하는 분석 모델이며 QA02/QA07의 실제 최초 채택 출력은 없다. 준비 활동을 정확도 분모에 더하지 않는다. 24개 사례는 실제 WAV/화면이 갖춰진 실행 fixture나 독립 통계 반복이 아니다. v3의 원값·분모·고정 정답 조건을 v4 결과로 옮기지 않는다.

## 보존 v3의 분모

| profile | 사례군 | 평가 입력/구성 | 지칭 표현/구성 | 목표/구성 | 별도 준비 활동 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Text 이력 | 12 | 240 | 250 | 180 | 관계 생산, 종료 후 갱신을 활동 원장에 포함 |
| Voice/Screen | 6 | 120 | 120 | 120 | 60 |
| Delivery | 8 | 80 | 120 | 90 | 80 |

QA01과 QA07은 입력, QA02는 표현, QA03은 목표를 분모로 사용한다. QA04는 각 구성에 동일 C01~C06 한 벌을 적용한다. QA05는 해당 profile의 준비, 갱신, 폐기까지 포함한 활동 묶음 비용이고, QA06은 동시 상태 최고값이다. 준비 활동을 입력 정확도의 분모에 더하지 않는다.

26개 사례군은 실제 모델 반복 횟수나 독립 통계 표본 수가 아니다. 다른 profile의 입력, 자원, 완료 조건은 고정된 서로 다른 가정이므로 수치와 DP 개선량을 합산하지 않는다. 현재 자료는 모두 `SYNTHETIC_AUTHORED_NOT_MEASURED`다.

COMMON-28의 목표, 점수, 비용 도출을 v3의 새 사례군에 자동 적용하지 않는다. 이관한 COMMON-28 JSON은 원본 바이트를 보존하며 평가자 정답을 포함하므로 후보 입력으로 통째로 제공하지 않는다. 문서 원전의 `qa_candidate_population.py` 등 기존 명령은 원래 경로를 대상으로 하므로 현재 편집 명령으로 사용하지 않는다. 후속 corpus 변경은 새 판본과 같은 조건의 재산출을 명시하며 [보존, 편집 규칙](../evaluation/common-qa-v3/migration-and-editing.md)을 따른다.
