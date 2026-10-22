# 공통 QA v4: 동일 사례와 환경의 구성 비교

2026-10-22 / `COMMON-QA-MODEL-4.0` / **같은 조건의 분석 모델. 실측 아님.**

**[Excel](artifacts/outputs/01a15dfe-v4/VIA-QA-common-v4.xlsx)의 `전체비교` 시트 하나를 먼저 본다.** DP2, DP3, DP5, DP44의 모든 방안과 QA01~07, 별도 첫 음성 진단이 한 표에 있다. 같은 사례에서 값이 달라진 이유는 [HTML 사례 비교](artifacts/outputs/01a15dfe-v4/guide.html)에서 확인한다.

| 확인할 것 | 자료 |
| --- | --- |
| 모든 DP/방안의 수치와 범위 | [전체 결과표](03-results-and-ranges.md), [Excel](artifacts/outputs/01a15dfe-v4/VIA-QA-common-v4.xlsx) |
| 같은 입력에 대한 A/B 처리와 비용 | [HTML](artifacts/outputs/01a15dfe-v4/guide.html)에서 사례와 구성 선택 |
| 무엇을 같게 했는가 | [공통 계약](01-common-contract.md), [24개 사례](02-configurations-and-scenarios.md) |
| 어떤 가정을 사용했는가 | [계산 명세](specification.json), Excel `공통가정` |
| 개별 수치의 출처 | [단계·비용·메모리 원장](artifacts/ledger.json), [요약 JSON](artifacts/summary.json), [CSV](artifacts/comparison.csv) |
| 정확도 정답과 남은 관측 | [평가자 전용 정답](oracles.json), Excel `지칭판정`/`의미판정` |
| 검토와 실제 검증 범위 | [독립 검토와 검증](04-review-and-validation.md) |
| 이전 판본 | [v3 보존본](../common-qa-v3/README.md) |

기준은 **DP2 A + DP3 A + DP5 A + DP44 A**다. 한 번에 DP 하나만 바꿔 DP2 B, DP3 B-R1/B-R2/B-R3, DP5 B, DP44 B의 여섯 구성을 더했다. 7구성에 동일한 24입력, 24목표, 33지칭, 6변경 과제를 적용했다. A 네 행은 같은 기준 원장이며 별도 네 번의 실행이 아니다. 기준 구성은 채택 권고가 아니다.

QA01은 모든 DP에서 **입력 종료부터 첫 구체적 내용의 화면 표시**다. 첫 음성은 별도 열이다. 같은 CPU·클라우드 속도와 단가, 같은 공통 RAM에서 각 구조의 실제로 모델링한 추가 활동을 계산하므로 v3처럼 profile마다 기준 scale이 달라지지 않는다.

시간·비용·RAM은 명시한 가정의 조건부 계산, QA04는 실제 Rust diff가 없는 모듈 영향 추정이다. QA02와 QA07은 **관측 출력 0개로 미산출**이다. 평가자 정답을 후보 출력으로 복사하거나 기존 오류율을 이식하지 않았다. 숫자들의 계산 기준은 통일했지만 실제 성능 검증은 아직 수행하지 않았다.

v3, COMMON-28, QA 정의와 점수, 기존 설계 및 발표 v10은 보존한다. 발표 v10에는 v4를 반영하지 않았다. 후속 실측은 실제 입력 자산과 후보 실행, 최초 채택 증거, 실패 처리와 실행 계약을 별도로 동결하고 새 판본으로 남겨야 한다.
