# 평가 계약, 수치와 근거의 현재 진입점

> **QA07 판본 구분:** 현재 [QA07-REQUEST-RESPONSE-v2](../qa/qa07-request-meaning-accuracy.md)는 요청 해석과 응답 구성/실제 전달까지 확장됐다. 아래 기존 판본의 입력 단위 QA07 값/정답은 당시 계약으로 보존하며 새 사례별 전체 충족률로 재명명하지 않는다. 새 분모와 관측 종료 및 목표/점수는 검토 중이다.

> **현재 공통 v6:** [QA07 사례/판정](common-qa-v6/README.md)을 추가했다. 기존 v5 시간/자원 값은 유지하며 24개 QA07 묶음은 미동결 검토안이다. 실제 정확도는 미산출이다.

> **보존 공통 v5:** [QA01/QA03 개정 계약](../qa/timing-measurement-contract.md)의 NORMAL/WAIT, 전체 완료 경계와 합의한 목표·점수 및 QA08 귀속 메모리를 반영했다. 기존 v4/v3는 당시 정의의 보존본이다.

현재 DP2, DP3, DP5, DP44의 같은 입력·환경 비교는 [공통 QA v6](common-qa-v6/README.md)에서 읽는다. [Excel 전체비교](common-qa-v6/artifacts/outputs/01a15dfe-v6/VIA-QA-common-v6.xlsx), [HTML 사례 비교](common-qa-v6/artifacts/outputs/01a15dfe-v6/guide.html), [모든 수치](common-qa-v5/03-results-and-ranges.md)를 제공한다. 정의, 목표, 점수의 편집 기준은 [QA](../qa/README.md), corpus 구분은 [시나리오](../scenarios/README.md)다.

| 근거 | 현재 읽을 자료 | 말할 수 있는 범위 |
| --- | --- | --- |
| 현재 QA07 확장 | [공통 QA v6](common-qa-v6/README.md), [판정과 검증](common-qa-v6/03-review-and-validation.md) | 24개 사례 묶음 제안, 단계별 의무와 전체 판정. 실제 실행0, 분모와 목표/점수 미정 |
| 보존 공통 분석 모델 v5 | [공통 QA v5](common-qa-v5/README.md), [검토](common-qa-v5/04-review-and-validation.md) | 24개 동일 입력, 7구성. 시간·비용·RAM 계산과 모듈 영향 추정. QA02/07 후보 출력 없음. 실측·제품 성능·선택 아님 |
| 이전 공통 분석 | [공통 QA v4](common-qa-v4/README.md) | 이전 화면 QA01, 완료 경계와 수치를 보존 |
| 가상 작성 평가 | [공통 QA v3](common-qa-v3/README.md), [수치와 판단](common-qa-v3/03-results-and-decisions.md) | 작성한 구조, 시간, 자원, 오류 가정 안의 상대 비교. 실제 성능, 제품 적합성, 결합 성능을 입증하지 않음 |
| 출처와 계산 연결 검증 | [원장 및 산출물](common-qa-v3/artifacts/README.md), [이관 및 검토](common-qa-v3/04-migration-and-review.md) | 원장 산술, 같은 파일 세대, Excel, HTML 표시 검증과 실제 확인 한계 |
| 이전 실제 모델 캠페인 | [이전 실제 모델 캠페인의 보고서](../../seven-problem-review/17-presentation-results.md)의 **§14 현재 결론**, §13 복구, 재채점, §12 이전 부분 중단 기록 | 원래 설계와 계약, 모집단, proxy 관측 범위의 증거. 현재 가상 평가의 실측 근거로 소급하지 않음 |

실측, proxy, 합성, 예상 변경 범위를 한 값이나 하나의 검증 PASS로 합치지 않는다. 보고서 17의 후속 완주와 §14 정정을 루트의 이전 부분 중단 요약보다 먼저 읽는다. 당시 callback 시간과 host 완료 endpoint의 관측 한계를 보존하며 철회한 속도 승자를 되살리지 않는다.

v3의 산출물과 부모 사본은 바이트 그대로 이관했다. 문서만 새 상대경로와 역할 안내를 적용했다. [편집, 보존 규칙](common-qa-v3/migration-and-editing.md)과 [이관 SHA256 원장](../qa-migration.json)을 함께 확인한다. 이 정리는 새로운 모델 실행, 측정, 점수 변경, 대안 채택이 아니다.
