# Quality Model and Draft QA Catalog

> 상태: **USER REVIEW DRAFT / category·single-metric 구조 합의 / target·score·ASR 미확정**
>
> 이번 generation은 QA를 품질 계열별 번호 범위로 한 번 재번호화했다. 각 QA는 하나의 대표 metric만 가진다. 이름·metric은 current draft이며 target, score band, workload와 ASR은 추가 승인 전까지 확정값이 아니다.

## 1. 분류 체계

```text
ISO/IEC 25010 기반 품질 관점
        ↓
QC — 제품 수준의 상위 품질 관심사
        ↓
QA category — 품질 계열과 ID 범위
        ↓
QA — 하나의 stimulus/response와 하나의 대표 metric
        ↓
ASR — Architecture 영향이 확인된 QA에 부여하는 분류
```

현재 확정된 ASR은 없다. QA가 중요해 보여도 정상적인 Architecture 대안 사이에서 구조적 인과를 설명할 수 없으면 ASR로 확정하지 않는다.

## 2. QA category와 번호 범위

| 번호 범위 | Category | 해석 |
| --- | --- | --- |
| QA-01~09 | Responsiveness | 사용자 입력·외부 event부터 의미 있는 응답 또는 제어 효과까지의 시간 |
| QA-11~19 | Correctness & Continuity | 의미, binding, state와 lifecycle 관계의 정확성 |
| QA-21~29 | Modifiability | 외부·제품 계약 변화가 Architecture에 퍼지는 범위 |
| QA-31~39 | Reliability & Availability | 장애의 영향 범위와 정확한 복구 |
| QA-41~49 | Resource Efficiency | target device의 실행 가능성과 자원 부담 |
| QA-51~59 | Privacy & Security | 보호정보 노출과 보안 관심사 |
| QA-61~69 | Observability | 실행 기록의 완전성과 평가 근거의 재현성 |

번호 범위 안의 공백은 의도적이다. QA가 추가·제거되어도 기존 ID를 당겨 붙이지 않는다.

## 3. Active draft QA catalog

| ID | Quality Attribute | 단일 대표 Metric | 상태 |
| --- | --- | --- | --- |
| QA-01 | Delegated Path VIA Responsiveness | 가장 느린 대표 case의 p95 VIA 처리시간(Agent 실행 제외) | semantic definition current; target draft |
| QA-02 | VIA Direct Voice Response Responsiveness | 가장 느린 대표 case의 p95 직접 Voice 응답시간 | semantic definition current; target draft |
| QA-03 | Agent Progress Voice Feedback Responsiveness | 가장 느린 대표 case의 p95 Agent 상태 Voice 전달시간 | semantic definition current; target draft |
| QA-04 | Voice Interruption Responsiveness | 가장 느린 대표 case의 p95 barge-in 후 음성 정지시간 | semantic draft; target pending |
| QA-05 | Task Control Responsiveness | 가장 느린 대표 control case의 p95 올바른 처리상태 응답시간 | semantic draft; target pending |
| QA-11 | VIA Request Handling Correctness | 올바르게 처리된 전체 요청 run 비율 | previous QA-05 metric preserved; integrated outcome; target draft |
| QA-12 | Request Semantic Resolution Correctness | 의미를 올바르게 해석한 run 비율 | driver draft; target pending |
| QA-13 | Task & Interaction Binding Correctness | 올바른 Task·Interaction에 연결한 run 비율 | driver draft; target pending |
| QA-14 | Async Task State Convergence Correctness | 비동기 event 뒤 올바른 상태로 수렴한 run 비율 | driver draft; target pending |
| QA-15 | Interaction & Task Continuity Correctness | 전환 뒤 필요한 대화·Task 관계가 유지된 scenario 비율 | driver draft; target pending |
| QA-21 | Agent Change Locality | Agent 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack current; target draft |
| QA-22 | Model, Context & State Change Locality | Model·Context·State 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack current; target draft |
| QA-23 | Experiment & Logging Change Locality | 실험·로그 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack draft; target pending |
| QA-31 | Correct Task Recovery Time | 가장 느린 fault 종류의 p95 전체 Task 복구시간 | recovery draft; target draft |
| QA-32 | Fault Blast Radius | 한 fault가 불필요하게 함께 중단시킨 사용자 기능·Task의 최대 수 | semantic draft; target pending |
| QA-41 | Target Device Memory Footprint | 가장 무거운 workload의 p95 최대 메모리 사용량 | semantic draft; target pending |
| QA-51 | Protected Data Exposure Minimization | 전체 workload에서 필요 이상 노출된 보호정보 단위 수 | exposure draft; target draft |
| QA-61 | Execution Trace Completeness | 로그로 전체 실행을 재구성할 수 있는 run 비율 | semantic draft; target pending |
| QA-62 | Evidence Reproducibility | 저장된 근거로 같은 평가 결과를 정확히 다시 만든 비율 | semantic draft; target pending |

## 4. QA-11과 QA-12~15의 관계

QA-11은 previous-generation QA-05의 이름·범위·metric을 계승한 **integrated outcome**이다. QA-12~15는 같은 실행 또는 전용 isolation fixture에서 구조적 실패 원인을 분리하는 **driver QA**다.

```text
QA-11 VIA Request Handling Correctness
  ├─ QA-12 semantic meaning
  ├─ QA-13 identity and interaction binding
  ├─ QA-14 asynchronous state convergence
  └─ QA-15 lifecycle continuity
```

각 QA는 하나의 독립 metric을 가진다. 다만 QA-11과 QA-12~15를 weighted total에 서로 독립된 다섯 표처럼 합산하지 않는다. QA-11은 통합 결과, QA-12~15는 원인·trade-off 설명에 사용한다. 한 DP에서 실제 구조 인과가 없는 driver QA는 regression 또는 `N/A`로 둔다.

## 5. Quality Concern coverage

| QC | Quality Concern | 현재 처리 |
| --- | --- | --- |
| QC-01 | User-Experienced Responsiveness | QA-01~05. 동시 Task는 workload condition |
| QC-02 | VIA Request Handling Correctness | QA-11~13 |
| QC-03 | Interaction & Task Continuity | 정상 조건 QA-13~15, 장애 후 QA-31 |
| QC-04 | Agent Ecosystem Interoperability & Substitutability | QA-21 |
| QC-05 | Evolvability & Maintainability | QA-22/23 |
| QC-06 | Resource & Deployment Efficiency | QA-41. 다른 resource 값은 diagnostic으로만 보존하고 합성 QA를 추가하지 않음 |
| QC-07 | Concurrency & Capacity | 독립 QA 없음. 관련 QA의 workload condition |
| QC-08 | Reliability & Recoverability | QA-14, QA-31, QA-32 |
| QC-09 | Privacy, Security & Action Safety | QA-51 + mandatory zero-violation gate |
| QC-10 | Observability & Evidence Integrity | QA-23, QA-61/62 + 모든 QA trace의 공통 evidence requirement |

## 6. 단일 metric 원칙

각 QA는 심사위원에게 한 문장으로 설명할 수 있는 대표 metric 하나만 가진다. component trace, case별 값, C/I/S/D breakdown, maximum, CPU/GPU sample과 failure reason은 대표 metric을 해석하는 supporting evidence이지 같은 QA의 추가 대표 metric이 아니다.

서로 다른 terminal event나 단위를 한 scalar로 임의 합성하지 않는다. Voice interruption은 QA-04, Task 취소·정정 처리 상태는 QA-05로 분리한다. 실행 trace 완전성은 QA-61, 그 trace에서 평가 결과를 다시 만드는 능력은 QA-62로 분리한다.

## 7. Mandatory qualification gates

다음 위반은 다른 QA의 높은 점수로 상쇄하지 않는다.

```text
wrong-target external action = 0
duplicate external action after retry/recovery = 0
revoked or mismatched approval use = 0
unauthorized access or disclosure = 0
```

모든 요청을 차단해 gate를 통과한 후보는 기능 적합성에 실패한다.

## 8. ID migration

| 이전 active ID | 현재 ID | 의미 |
| --- | --- | --- |
| QA-05 | QA-11 | VIA request handling integrated outcome |
| QA-07 | QA-21 | Agent Change Locality |
| QA-08 | QA-22 | Model, Context & State Change Locality |
| QA-09 | QA-31 | Correct Task Recovery Time |
| QA-11 | QA-51 | Protected Data Exposure Minimization |

과거 QA-04/06/10/12 정의는 [QA Catalog Draft v1 Archive](../../archive/qa-catalog-draft-v1/README.md)에만 남는다. Previous-generation QA-05의 의미는 QA-11로 이동했다. 현재 QA-04, QA-05와 QA-12는 category-range generation에서 새로 정의된 다른 품질 속성이다. Archive의 ID와 문서는 변경하지 않는다.

## 9. QA 인정과 ASR 판정

active QA는 다음 질문에 모두 답해야 한다.

1. 사용자의 제품 결과 또는 빠른 기술 변화에 왜 중요한가?
2. 정상적으로 구현된 합리적 Architecture A/B에서도 값이 달라질 수 있는가?
3. 차이를 책임 배치·계약·상태·배치·call graph로 설명할 수 있는가?
4. 사람이 매번 주관적으로 판정하지 않고 반복 측정할 수 있는가?
5. metric과 목표를 한 문장으로 설명할 수 있는가?

QA별 ASR 상태는 `UNASSESSED`, `CANDIDATE`, `CONFIRMED_ASR`, `NOT_ASR` 중 하나로 관리한다. 현재 active draft QA는 모두 `UNASSESSED`다.
