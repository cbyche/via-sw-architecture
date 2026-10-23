# 평가 계약, 수치와 근거의 현재 진입점

> **2026-10-23 QA07 v4 반영:** 현재 QA07은 **업무 순서 및 연결 정확도**다. 사용자 지정 선후 관계와 NEW/EXISTING 및 기존 업무 ID의 최초 판단을 평가한다. 이름과 범위는 사용자 확정, **85%·28입력 × 5회(140회)·3%p 점수는 작성자 제안**이다. 공통 v9에 별도 매핑과 관측 전 평가표를 두었다. 실제 실행 0회, 결과 미산출이며 기존 v7/v8은 보존한다. v8의 응답 구성·전달 범위와 90%·160회는 이전 제안이다.

> **2026-10-23 QA 구체화:** [대표값/부록 계약](../qa/measurement-appendix.md)과 [QA04의 8개 변경 과제](../qa/change-scenarios.md)를 적용한다. QA01은 P95 없이 평균 2초와 NORMAL/WAIT 분리, QA02/QA07은 각각 단일 비율로 별도 유지한다. QA06은 전체 제품 예산이다. QA04-CHANGE-v4는 Rust/Python 등 실제 운영 모듈을 세며 v6에 보존된 기존 6과제의 추정값과 호환되지 않는다. 새 QA04 값은 미산출이며 기존 평가 파일과 PPTX는 보존한다. 후속 담당 작업은 [구체화 전달 계약](../qa/refinement-handoff.md)을 따른다.

> **이전 범위 QA07 판본 구분:** 보존 [QA07-REQUEST-RESPONSE-v3](../qa/history/qa07-before-order-connection-20261023/qa07-request-meaning-accuracy.md.txt)는 32사례 × 5회와 90% 목표, 종료/점수의 작성 완료 제안이다. [공통 v8](common-qa-v8/README.md)에 새 원장을 두며 이전 24묶음은 v7/v6에 보존한다. 실제 실행은 없다.

> **보존 공통 v6:** [QA07 사례/판정](common-qa-v6/README.md)을 추가했다. 기존 v5 시간/자원 값은 유지하며 24개 QA07 묶음은 미동결 검토안이다. 실제 정확도는 미산출이다.

> **보존 공통 v5:** [QA01/QA03 개정 계약](../qa/timing-measurement-contract.md)의 NORMAL/WAIT, 전체 완료 경계와 합의한 목표·점수 및 QA08 귀속 메모리를 반영했다. 기존 v4/v3는 당시 정의의 보존본이다.

현재 DP2, DP3, DP5, DP44의 QA별 공통 조건 비교는 [공통 QA v9](common-qa-v9/README.md)에서 읽는다. [Excel 전체비교](common-qa-v9/artifacts/outputs/01a15dfe-v9/VIA-QA-common-v9.xlsx), [HTML 사례 비교](common-qa-v9/artifacts/outputs/01a15dfe-v9/guide.html), [현재 요약](common-qa-v9/artifacts/summary.json)을 제공한다. QA04는 새 8과제의 실제 증거가 없어 미측정이며 예전 6과제 수치를 사용하지 않는다. 정의 정본은 [QA](../qa/README.md), corpus 구분은 [시나리오](../scenarios/README.md)다.

| 근거 | 현재 읽을 자료 | 말할 수 있는 범위 |
| --- | --- | --- |
| 현재 QA07 v4 | [공통 QA v9](common-qa-v9/README.md), [검토](common-qa-v9/03-review-and-validation.md) | 28입력 × 5회 계획, 85% 목표 제안. 다른 QA의 기존 24입력과 별개. 실제0·980 UNKNOWN |
| 이전 범위 QA07 v3 | [공통 QA v8](common-qa-v8/README.md), [검토](common-qa-v8/03-review-and-validation.md) | 32사례 × 5회 계획과 90% 목표 제안. 실제 실행0, 모든 결과 UNKNOWN |
| 보존 대표값/부록 | [공통 QA v7](common-qa-v7/README.md), [검토](common-qa-v7/04-review-and-validation.md) | QA04 8과제와 실제 모듈 증거 원장, 유형별 부록, QA06 전체 제품 범위와 QA08 GB 표시. 새 실측 없음 |
| 보존 QA07 확장 | [공통 QA v6](common-qa-v6/README.md), [판정과 검증](common-qa-v6/03-review-and-validation.md) | 24개 사례 묶음 제안, 단계별 의무와 전체 판정. 실제 실행0, 분모와 목표/점수 미정 |
| 보존 공통 분석 모델 v5 | [공통 QA v5](common-qa-v5/README.md), [검토](common-qa-v5/04-review-and-validation.md) | 24개 동일 입력, 7구성. 시간·비용·RAM 계산과 모듈 영향 추정. QA02/07 후보 출력 없음. 실측·제품 성능·선택 아님 |
| 이전 공통 분석 | [공통 QA v4](common-qa-v4/README.md) | 이전 화면 QA01, 완료 경계와 수치를 보존 |
| 가상 작성 평가 | [공통 QA v3](common-qa-v3/README.md), [수치와 판단](common-qa-v3/03-results-and-decisions.md) | 작성한 구조, 시간, 자원, 오류 가정 안의 상대 비교. 실제 성능, 제품 적합성, 결합 성능을 입증하지 않음 |
| 출처와 계산 연결 검증 | [원장 및 산출물](common-qa-v3/artifacts/README.md), [이관 및 검토](common-qa-v3/04-migration-and-review.md) | 원장 산술, 같은 파일 세대, Excel, HTML 표시 검증과 실제 확인 한계 |
| 이전 실제 모델 캠페인 | [이전 실제 모델 캠페인의 보고서](../../seven-problem-review/17-presentation-results.md)의 **§14 현재 결론**, §13 복구, 재채점, §12 이전 부분 중단 기록 | 원래 설계와 계약, 모집단, proxy 관측 범위의 증거. 현재 가상 평가의 실측 근거로 소급하지 않음 |

실측, proxy, 합성, 예상 변경 범위를 한 값이나 하나의 검증 PASS로 합치지 않는다. 보고서 17의 후속 완주와 §14 정정을 루트의 이전 부분 중단 요약보다 먼저 읽는다. 당시 callback 시간과 host 완료 endpoint의 관측 한계를 보존하며 철회한 속도 승자를 되살리지 않는다.

v3의 산출물과 부모 사본은 바이트 그대로 이관했다. 문서만 새 상대경로와 역할 안내를 적용했다. [편집, 보존 규칙](common-qa-v3/migration-and-editing.md)과 [이관 SHA256 원장](../qa-migration.json)을 함께 확인한다. 이 정리는 새로운 모델 실행, 측정, 점수 변경, 대안 채택이 아니다.
