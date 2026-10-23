# 공통 QA v7

2026-10-23 / COMMON-QA-MODEL-7.0 / 분석 추정과 미측정 평가표

**네 DP의 모든 방안을 함께 보려면 [Excel](artifacts/outputs/01a15dfe-v7/VIA-QA-common-v7.xlsx)의 `전체비교` 또는 [HTML 비교표](artifacts/outputs/01a15dfe-v7/guide.html)를 연다.** 같은 24입력과 7구성을 유지한다. 네 DP의 A는 동일 BASE를 반복 표시한 것이며 독립 실행 네 건이 아니다.

| 확인할 내용 | 자료 |
| --- | --- |
| 대표 숫자, 단위와 근거 상태 | Excel `전체비교`, HTML 첫 표, [요약 JSON](artifacts/summary.json), [CSV](artifacts/comparison.csv) |
| QA04 새 8개 변경 과제와 미측정 이유 | [변경 전후와 완료 조건](02-qa04-changes.md), Excel `QA04변경`, [증거 원장](qa04-change-manifest.json) |
| QA별 유형과 상세 계산 | [부록 안내](03-appendix.md), Excel `QA상세부록`, HTML 세부 유형 |
| 계산과 보존 범위 | [계약](01-contract-and-scope.md), [전체 원장](artifacts/ledger.json) |
| 독립 검토와 검증 한계 | [검토 기록](04-review-and-validation.md) |

QA04는 기존 6과제의 concept 추정값을 새 평균으로 사용하지 않는다. 새 8과제 × 7구성의 56기록에는 실제 patch가 없어 모두 **NOT_MEASURED**다. 시간과 비용, 메모리의 분석 원값은 v6에서 유지했다. QA08 대표 표시만 MB에서 GB로 바꾸었다. QA06의 전체 제품 예산 512 MB와 QA08의 추가 모델 경로 1 GB 미만은 더하지 않는다.

QA02와 QA07은 실제 후보 출력이 없어 전체 정확도 미산출이다. 이 판본의 QA07은 v2와 24묶음을 보존한다. 새 32사례·5회 반복과 목표 제안은 [v8](../common-qa-v8/README.md)에서 별도로 적용한다. 실제 제품 실행, 모델 호출, 새로운 성능 수치와 PPTX 개정은 이번 작업에 포함하지 않는다.

[v6와 이전 세대](../common-qa-v6/README.md)는 원본 그대로 보존한다. 정의의 사용 판본은 [스냅샷 원장](definition-sources.json), v6 보존 검사는 [해시 목록](preserved-v6-sha256.json)으로 확인한다.
