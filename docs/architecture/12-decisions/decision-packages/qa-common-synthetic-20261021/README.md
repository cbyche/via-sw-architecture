# DP2와 DP5 공통 QA 비교

> 후속 통합: [DP2, DP3, DP5의 공통 QA 층별 평가 v2](../qa-common-synthetic-v2-20261021/README.md). 이 문서와 v1 원장은 원래 Text 조건의 이전 세대로 보존한다.

2026-10-21 / `COMMON-QA-SYN-1.0`

DP별로 달랐던 QA01~07의 의미, 모집단과 계산 경계를 맞춘 **가상 시나리오 기반 설계 평가**다. 같은 120개 에피소드와 240개 입력을 공통 기준 BASE, DP2만 바꾼 EXEC, DP5만 바꾼 REL에 적용했다. DP2 A와 DP5 A는 같은 BASE를 참조한다.

먼저 [결과와 사례 해설](04-results-and-walkthrough.md)을 읽거나 [대화형 설명서](../../../../presentations_files/qa-common-synthetic-20261021/outputs/01a15d51/VIA-QA-common-guide.html)를 연다. [13개 시트의 Excel 원장](../../../../presentations_files/qa-common-synthetic-20261021/outputs/01a15d51/VIA-QA-common-synthetic.xlsx)에서 입력, 최초 채택, 처리 순서와 계산 근거를 확인할 수 있다.

| 문서 | 역할 |
| --- | --- |
| [01 공통 계약](01-common-contract.md) | QA 정의와 관측 단위, 공통 환경, BASE 공유 규칙 |
| [02 변경과 자원 근거](02-change-and-resource-rationale.md) | 같은 변경 과제, 예상 모듈, 비용과 메모리 가정 |
| [03 이관과 확장](03-migration-and-extension.md) | 이전 자료의 보존, 다른 DP 추가와 판본 관리 |
| [04 결과와 사례](04-results-and-walkthrough.md) | 전체 비교, 유형별 결과, 입력을 따라가는 설명 |
| [05 검토 기록](05-review-and-validation.md) | 독립 감사의 발견과 수정 및 검증 범위 |
| [06 다음 세션 지침](06-session-handoff.md) | 새 DP 준비 후의 재개 요청, 읽기 순서, 확장과 검증 절차 |
| [산출물과 재생성](../../../../presentations_files/qa-common-synthetic-20261021/README.md) | JSON, manifest, workbook 및 생성 명령 |

지칭은 250개 표현, 의미는 240개 입력, 시간은 180개 목표, 변경 범위는 6개 과제로 각각 계산한다. 수치는 작성 가정에서 산출했으며 실제 시험 기록이 아니다. 기존 QA 목표와 점수, accepted ADR 및 이전 결과는 유지한다.

후속 DP는 같은 QA 경계와 BASE 조건에 연결한다. 새 기능 때문에 공통 사례를 바꾸면 corpus 판본을 올리고 기존 DP2와 DP5도 같은 사례로 다시 산출한다. 다른 DP의 개별 개선량을 더해 조합안 결과를 만들지 않는다.
