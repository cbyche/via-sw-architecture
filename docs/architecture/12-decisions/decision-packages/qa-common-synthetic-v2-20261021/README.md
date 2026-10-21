# DP2, DP3, DP5의 공통 QA 층별 평가

2026-10-21 / `COMMON-QA-SYN-2.0` / **SYNTHETIC_AUTHORED_NOT_MEASURED**

**QA01~07의 정의와 원장 형식을 통일하고, Text 이력과 Voice/Screen의 서로 다른 평가 조건을 두 층으로 보존했다.** DP2와 DP5의 공통판 및 최신 DP3의 네 대안을 다시 집계해 총 7개 구성의 49개 QA 값을 확인했다. 이는 가상 수치의 정합성을 검토한 결과이며 실제 모델 실행이나 제품 시험 결과가 아니다.

동일한 조건 안에서 내리는 개별 권고는 **DP2 B, DP3 A, DP5 B**다. 이 셋을 동시에 적용한 제품 구성의 성능이나 승리를 뜻하지 않는다. DP2와 DP5는 같은 Text 기준 구성과 비교했고, DP3는 별도 Voice/Screen 기준과 비교했다. 새로운 결합안의 효과는 아직 작성하지 않았다.

| 문서 | 읽을 내용 |
| --- | --- |
| [01 공통 계약](01-common-contract.md) | 같은 QA 의미, 층별 관측 경계와 분모, 합산 제한 |
| [02 구성과 시나리오](02-configurations-and-scenarios.md) | 독립 선택 축, 18개 사례군, 구체적인 입력과 실패 설명 |
| [03 수치와 판단](03-results-and-decisions.md) | 재산출 비교표, 단일 축 차이, 준비 비용, RAM 및 조건부 선택 |
| [04 이관과 검토](04-migration-and-review.md) | 부모 자료의 보존, 수정한 경계, 독립 검토, 다음 DP 추가 규칙 |
| [산출물](../../../../presentations_files/qa-common-synthetic-v2-20261021/README.md) | 공통 원장, Excel, HTML, 재계산과 검증 |

원본은 [DP2와 DP5 v1](../qa-common-synthetic-20261021/README.md)과 [DP3 Voice/Screen 고정 원본 사본](../../../../presentations_files/qa-common-synthetic-v2-20261021/parents/voice.json)이다. DP3 사본의 출처는 `seven-problem-review/dp3-grounding-architecture-20261021/synthetic-comparison/`이며, 원본 폴더와 PPTX를 변경하지 않았다. 새 판본은 원본의 입력과 작성 가정을 SHA256으로 고정한 사본에서 계산하며, 원래 숫자를 새 실험의 결과로 바꾸지 않는다.

이 판본에서 완성한 것은 **공통 의미, 층별 시나리오 은행, 출처가 연결된 원장, 비교와 선택 근거**다. 모든 DP를 같은 단일 workload에서 바꿔 보는 결합 실험이나 전체 시스템의 통합 점수는 아니다. 표에서 Text와 Voice를 고르는 것은 비교 조건을 고르는 것이며 제품의 동시 실행 mode를 전환하는 시뮬레이션이 아니다.
