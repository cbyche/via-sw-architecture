# 공통 QA v9

2026-10-23 / COMMON-QA-MODEL-9.0 / QA07-ORDER-LINK-v4 적용 매핑 / 실제 실행 0회

**네 DP의 모든 방안은 [Excel](artifacts/outputs/01a15dfe-v9/VIA-QA-common-v9.xlsx)의 `전체비교` 또는 [HTML](artifacts/outputs/01a15dfe-v9/guide.html)에서 함께 본다.** 네 A행은 같은 BASE를 재사용하며 독립 실험 네 벌이 아니다.

새 QA07은 **업무 순서 및 연결 정확도**다. 사용자가 지정한 선후 관계와 NEW/EXISTING 및 기존 업무 ID의 최초 채택 결과를 모두 맞힌 입력의 비율 하나를 평가한다. 이름과 범위는 사용자 확정이다. **85% 목표, 28입력 × 5회 = 140회와 3%p 점수 구간은 작성자 제안**이며, 목표 성공 수는 119회다. 실제 실행과 관측이 없어 정확도와 점수는 미산출이다.

| 확인할 내용 | 열 자료 |
| --- | --- |
| 모든 DP/방안의 QA 대표값 | Excel `전체비교`, HTML 첫 표, [JSON 요약](artifacts/summary.json), [CSV](artifacts/comparison.csv) |
| QA07 반복별 판정과 근거 | Excel `QA07순서연결`, [980개 빈 관측 기록](artifacts/qa07-results.json) |
| 선정 28입력과 제외 56입력 | Excel `QA07선정입력`, [입력 매핑](02-qa07-cases.md), [manifest](qa07-case-manifest.json) |
| 순서 있음 2개/없음 26개 원값 | Excel `QA07선정입력` J:Q, HTML QA07 부록 |
| 목표와 판정 경계 및 DP 적용 한계 | [적용 계약](01-contract.md) |
| 독립 검토와 검증 범위 | [검토 기록](03-review-and-validation.md) |

QA07의 28입력은 기존 시간/비용 분석 U01~U24와 다르다. 그 24입력에 대한 시간·비용·RAM 분석 값은 그대로 보존하며 새 28입력의 값으로 복사하지 않는다. QA02는 관측 없음, QA04는 실제 8과제 patch 없음으로 미산출/미측정이다. QA06은 전체 제품 RAM 512 MB 예산이고 QA08은 추가 로컬 모델 1 GB 미만이며 합산하지 않는다.

새 평가표는 판정 근거를 입력하고 집계하는 자료다. 실행용 후보 입력/adapter, 모델 호출과 제품 계측을 구현하거나 실행한 것이 아니다. 평가자용 정답/해설 원장을 후보에게 제공하면 안 된다. 실제 실행 전에는 이전 대화/업무 상태를 입력 시점에 맞춰 복원하고 정답·미래 사건 누출을 제거한 fixture를 별도로 검증해야 한다.

[v8](../common-qa-v8/README.md)과 [v7](../common-qa-v7/README.md)은 당시 범위 그대로 보존한다. 새 파일 안의 `QA07v3*` 및 이전 QA07 시트도 보존 참고이며 현재 대표값에 연결하지 않는다. PPTX는 변경하지 않았다.
