# Quality Model and Draft QA Catalog

> 상태: **USER REVIEW DRAFT / QA catalog 미확정 / ASR 미선정**
>
> 이 catalog의 QA 이름·ID·metric·target은 추가 검토에서 변경될 수 있다. 현재 결과나 최종 Architecture 결정을 뜻하지 않는다.

## 1. 분류 체계

```text
ISO/IEC 25010 기반 품질 관점
        ↓
QC — 제품 수준의 상위 품질 관심사
        ↓
QA — Architecture 대안을 비교하기 위한 측정 가능한 품질 속성
        ↓
ASR — Architecture 영향이 충분히 확인된 QA에 부여하는 분류
```

| 구분 | ID | 의미 |
| --- | --- | --- |
| Quality Concern | `QC-01`~`QC-10` | 제품 수준의 상위 품질 관심사. 자체 점수 없음 |
| Quality Attribute | 아래 active draft ID | stimulus·response·metric으로 A/B를 비교할 항목 |
| Architecture Significant Requirement | 별도 ID 없음 | Architecture에 중대한 QA에 부여하는 상태 |

현재 확정된 ASR은 없다. QA가 중요해 보여도 정상적인 Architecture 대안 사이에서 구조적 인과를 설명할 수 없으면 ASR로 확정하지 않는다.

## 2. Quality Concern catalog와 현재 처리

| ID | Quality Concern | 현재 처리 |
| --- | --- | --- |
| QC-01 | User-Experienced Responsiveness | QA-01~QA-03으로 측정. 동시 Task는 별도 QA가 아니라 workload condition |
| QC-02 | VIA Request Handling Correctness | QA-05로 측정 |
| QC-03 | Interaction & Task Continuity | 정상 연결은 QA-05, 장애 후 복원은 QA-09에서 측정 |
| QC-04 | Agent Ecosystem Interoperability & Substitutability | QA-07로 측정 |
| QC-05 | Evolvability & Maintainability | QA-08로 측정 |
| QC-06 | Cross-Device Portability & Adaptability | 현재 범위에서 독립 QA 없음 |
| QC-07 | Compute & Energy Efficiency | target-device constraint와 responsiveness workload 조건으로 관리 |
| QC-08 | Reliability & Recoverability | QA-09로 측정. 이전 failure-containment QA를 통합 |
| QC-09 | Privacy, Security & Action Safety | QA-11은 data minimization을 측정. Action/access safety는 필수 검증 조건으로 유지 |
| QC-10 | Diagnosability & Operability | 모든 QA trace의 공통 evidence 요구로 유지 |

## 3. Active draft QA catalog

ID는 이번 review가 끝날 때까지 재사용하거나 연속 번호로 당겨 쓰지 않는다. catalog 승인 후 필요하면 한 번만 재번호화한다.

| ID | Quality Attribute | 핵심 질문 | 상태 |
| --- | --- | --- | --- |
| QA-01 | Delegated Path VIA Responsiveness | Agent 실행을 제외한 VIA의 위임 전·결과 후 Voice 지연은 얼마인가? | semantic definition current; target draft |
| QA-02 | VIA Direct Voice Response Responsiveness | 직접 Voice 요청 뒤 유효 음성이 들리기까지 얼마인가? | semantic definition current; target draft |
| QA-03 | Agent Progress Voice Feedback Responsiveness | source status가 사용자에게 Voice로 전달되기까지 얼마인가? | semantic definition current; target draft |
| QA-05 | VIA Request Handling Correctness | VIA가 의도·대상·Task·처리 경로·Agent·결과 연결을 올바르게 결정했는가? | oracle contract draft |
| QA-07 | Agent Change Locality | Agent 변화 한 건이 평균 몇 Architecture Element를 바꾸는가? | change pack/element contract draft |
| QA-08 | Model & Context Change Locality | Model·Context·상태·배치 변화 한 건이 평균 몇 Architecture Element를 바꾸는가? | change pack/element contract draft |
| QA-09 | Correct Task Recovery Time | 장애 뒤 모든 영향 Task의 정확한 상태·결과·제어가 돌아오기까지 얼마인가? | merged recovery contract draft |
| QA-11 | Protected Data Exposure Minimization | 정상 업무에 필요한 최소 범위를 초과해 외부에 노출된 보호정보가 몇 개인가? | exposure oracle draft |

## 4. 독립 QA에서 제외·병합된 draft ID

| 이전 ID | 처리 | 이유 |
| --- | --- | --- |
| QA-04 Concurrent Task Performance Isolation | 독립 QA 제외 | 고정 1:4 ratio의 제품 근거가 없고 기존 값은 simulated reference였음. 동시성은 QA-01~QA-03·QA-05·QA-09의 workload condition으로 사용 |
| QA-06 Interaction & Task Continuity | QA-05·QA-09에 병합 | 정상 Task/session 연결은 request handling correctness, 장애 후 연결 복원은 recovery에 해당 |
| QA-10 Dependency Failure Containment | QA-09에 병합 | 장애 전파 여부보다 모든 영향 기능과 Task의 정확한 복구 결과·시간을 직접 측정 |
| QA-12 Action & Access Safety | 독립 QA 제외 | 현재 과제의 핵심 A/B trade-off로 채택하지 않음. 승인·접근 위반 0건은 필수 기능·회귀 조건으로 유지 |

제외는 요구 삭제가 아니다. 예를 들어 잘못된 Task에 Action을 연결하거나 승인 범위를 넘는 실행은 모든 후보에서 금지된다. 다만 그 위반 수를 독립적인 Architecture 비교 점수로 만들지 않는다.

## 5. QA 인정 기준

active QA는 다음 질문에 모두 답해야 한다.

1. 사용자의 제품 결과 또는 빠른 기술 변화에 왜 중요한가?
2. 정상적으로 구현된 합리적 Architecture A/B에서도 값이 달라질 수 있는가?
3. 차이를 책임 배치·계약·상태·배치·call graph로 설명할 수 있는가?
4. 사람이 매번 주관적으로 판정하지 않고 반복 측정할 수 있는가?
5. metric과 목표를 심사위원에게 한 문장으로 설명할 수 있는가?

단순 구현 버그 수, 드문 사건을 억지로 만든 실패율, 임의 복잡도 level은 active QA로 두지 않는다.

## 6. ASR 판정

QA별 상태는 `UNASSESSED`, `CANDIDATE`, `CONFIRMED_ASR`, `NOT_ASR` 중 하나로 관리한다. 현재 active draft QA는 모두 `UNASSESSED`다.

ASR은 점수가 갈렸다는 이유만으로 선정하지 않는다. 제품 중요성, 구조적 인과, 잘못된 선택의 결과와 추적 가능한 evidence를 함께 검토한다.

## 7. 이전 draft

직전 QA-01~QA-12 catalog와 scoring 정의는 [QA Catalog Draft v1 Archive](../../archive/qa-catalog-draft-v1/README.md)에 보존한다.
