# ASR-QA 평가 설계 안내

이 파일은 [03-02 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 발표용 연결 안내다. 정의/정답/분모/목표를 이곳에서 독립 개정하지 않는다.

| QA | 원본 | 평가 단위 |
| --- | --- | --- |
| ASR-QA-01 요청 처리 정확성 | 03-02 §3 | 18개 Voice 사례 × 5회 × 6개 조건 = 540개 |
| ASR-QA-02 상호작용 응답시간 | 03-02 §4 | 6개 사건 × 5회 = 30개 유효 반응시간 |
| ASR-QA-03 VIA 요청 처리시간 | 03-02 §4 | 6개 전체 사이클 × 5회 = 30개 귀속 시간 |
| ASR-QA-04 VIA 모델 사용 비용 | 03-02 §5 | 동일 8시간 활동의 청구 사용량 × 공개 단가 |
| ASR-QA-05 VIA 변경 용이성 | 03-02 §6 | 6개 변경 과제의 고유 수정 책임 수 평균 |
| ASR-QA-06 로컬 VIA 메모리 사용량 | 03-02 §7 | 8시간/3session 중 가장 큰 동시 commit량 |

정확성 18개 상세 명세는 [사례 투영 JSON](functional-coverage.json), 공통 규모/목표는 [평가 안내 JSON](evaluation-plan.json)에 원본 hash와 함께 수록했다. [검산 보고](../../architecture/12-decisions/decision-packages/03-02-poc-calculation-report.md)는 공개 단가/가정 계산이며 후보 성능 결과가 아니다.
