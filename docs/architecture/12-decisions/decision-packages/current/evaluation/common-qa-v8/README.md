# 공통 QA v8

2026-10-23 / COMMON-QA-MODEL-8.0 / QA07 v3 작성 완료 제안 적용 / 미측정

**전체 비교는 [Excel](artifacts/outputs/01a15dfe-v8/VIA-QA-common-v8.xlsx)의 `전체비교` 또는 [HTML](artifacts/outputs/01a15dfe-v8/guide.html)에서 확인한다.** DP2·DP3·DP5·DP44의 같은 7구성을 비교하며 네 A행은 동일 BASE다.

QA07은 **32사례 × 5반복 = 구성당 계획 160회**, 전체 충족률 **90% 이상**을 목표로 정한 작성자 제안이다. 실제 실행은 0회이고 정확도와 점수는 미산출이다. 최초 해석부터 응답 구성과 실제 전달까지 모두 맞아야 한 사례 반복이 성공이다. 새 32사례는 이전 24묶음과 별도 모집단이다.

| 볼 내용 | 자료 |
| --- | --- |
| QA별 대표값과 WAIT 보조값 | Excel `전체비교`, HTML 첫 표, [요약 JSON](artifacts/summary.json), [CSV](artifacts/comparison.csv) |
| QA07 목표·기한·판정과 제안 상태 | [적용 계약](01-contract.md), [정의 정본](../../qa/qa07-request-meaning-accuracy.md) |
| QA07 32사례의 입력/정답/금지 사항 | [사례표](02-qa07-cases.md), Excel `QA07v3명세`, [manifest](qa07-case-manifest.json) |
| QA07 반복별 관측과 두 평가자 판정 | Excel `QA07v3사례`/`QA07v3의무`, [원장](artifacts/qa07-results.json) |
| QA04 8개 실제 변경 과제 | Excel `QA04변경`, [v7 변경 과제와 증거](../common-qa-v7/02-qa04-changes.md). 56기록 모두 미측정 |
| 나머지 QA 세부 값 | Excel 기존 상세 시트, [v7 부록 안내](../common-qa-v7/03-appendix.md). QA07 v2 행은 보존 참고이며 새 대표값에 미사용 |
| 검토와 확인 범위 | [검증 기록](03-review-and-validation.md) |

시간·비용·RAM은 v7의 분석 추정 그대로이며 제품 실측이 아니다. QA01 NORMAL 평균 2초와 WAIT 분리, QA03 전체 목표 평균 5초를 유지한다. QA02는 관측이 없어 미산출, QA04는 실제 patch가 없어 미측정이다. QA06은 전체 제품 512 MB 예산, QA08은 추가 모델 1 GB 미만이며 합산하지 않는다.

[v7](../common-qa-v7/README.md)과 v6 및 이전 세대는 바이트 보존한다. [정의 snapshot](definition-sources.json)에 QA07 v3의 당시 정본을 고정했다. 모델 실행, 새 성능 수치, 대안 채택과 PPTX 수정은 수행하지 않았다.
