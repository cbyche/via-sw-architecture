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
| QA-61~69 | Observability | 예약 범위. 현재 active QA 없음 |

번호 범위 안의 공백은 의도적이다. QA가 추가·제거되어도 기존 ID를 당겨 붙이지 않는다.

## 3. Active draft QA catalog

| ID | Quality Attribute | 단일 대표 Metric | 상태 |
| --- | --- | --- | --- |
| QA-01 | Delegated Path VIA Responsiveness | worst canonical-case p95 delegated VIA active time | semantic definition current; target draft |
| QA-02 | VIA Direct Voice Response Responsiveness | worst canonical-case p95 direct Voice response time | semantic definition current; target draft |
| QA-03 | Agent Progress Voice Feedback Responsiveness | worst canonical-case p95 status-to-Voice time | semantic definition current; target draft |
| QA-04 | Voice Interruption Responsiveness | worst canonical-case p95 barge-in-to-audio-stop time | semantic draft; target pending |
| QA-11 | VIA Request Handling Correctness | correct machine-oracle runs | previous QA-05 metric preserved; integrated outcome; target draft |
| QA-12 | Request Semantic Resolution Correctness | strict semantic-resolution pass rate | driver draft; target pending |
| QA-13 | Task & Interaction Binding Correctness | strict interaction-binding pass rate | driver draft; target pending |
| QA-14 | Async Task State Convergence Correctness | strict async-state-convergence pass rate | driver draft; target pending |
| QA-15 | Interaction & Task Continuity Correctness | strict continuity-scenario pass rate | driver draft; target pending |
| QA-21 | Agent Change Locality | mean changed Architecture Elements per Agent change | change pack current; target draft |
| QA-22 | Model & Context Change Locality | mean changed Architecture Elements per non-Agent change | change pack current; target draft |
| QA-31 | Correct Task Recovery Time | worst fault-stratum p95 full Task recovery time | recovery draft; target draft |
| QA-32 | Fault Blast Radius | worst fault excess affected user-visible units | semantic draft; target pending |
| QA-41 | Target Device Memory Footprint | worst workload-stratum p95 peak committed memory | semantic draft; target pending |
| QA-51 | Protected Data Exposure Minimization | whole-workload union of excess exposed protected units | exposure draft; target draft |

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
| QC-01 | User-Experienced Responsiveness | QA-01~04. 동시 Task는 workload condition |
| QC-02 | VIA Request Handling Correctness | QA-11~13 |
| QC-03 | Interaction & Task Continuity | 정상 조건 QA-13~15, 장애 후 QA-31 |
| QC-04 | Agent Ecosystem Interoperability & Substitutability | QA-21 |
| QC-05 | Evolvability & Maintainability | QA-22 |
| QC-06 | Resource & Deployment Efficiency | QA-41. Energy는 future candidate |
| QC-07 | Concurrency & Capacity | 독립 QA 없음. 관련 QA의 workload condition |
| QC-08 | Reliability & Recoverability | QA-14, QA-31, QA-32 |
| QC-09 | Privacy, Security & Action Safety | QA-51 + mandatory zero-violation gate |
| QC-10 | Observability & Evidence Integrity | 모든 QA trace의 공통 evidence requirement |

## 6. 단일 metric 원칙

각 QA는 심사위원에게 한 문장으로 설명할 수 있는 대표 metric 하나만 가진다. component trace, case별 값, C/I/S/D breakdown, maximum, CPU/GPU sample과 failure reason은 대표 metric을 해석하는 supporting evidence이지 같은 QA의 추가 대표 metric이 아니다.

서로 다른 terminal event나 단위를 한 scalar로 임의 합성하지 않는다. 예를 들어 Voice interruption과 Agent cancel delivery는 모두 interaction control이지만 endpoint가 다르므로 현재 QA-04에는 Voice interruption만 포함한다.

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
| QA-08 | QA-22 | Model & Context Change Locality |
| QA-09 | QA-31 | Correct Task Recovery Time |
| QA-11 | QA-51 | Protected Data Exposure Minimization |

과거 QA-04/06/10/12 정의는 [QA Catalog Draft v1 Archive](../../archive/qa-catalog-draft-v1/README.md)에만 남는다. 현재 QA-04와 QA-12는 category-range generation에서 새로 정의된 다른 품질 속성이다. Archive의 ID와 문서는 변경하지 않는다.

## 9. QA 인정과 ASR 판정

active QA는 다음 질문에 모두 답해야 한다.

1. 사용자의 제품 결과 또는 빠른 기술 변화에 왜 중요한가?
2. 정상적으로 구현된 합리적 Architecture A/B에서도 값이 달라질 수 있는가?
3. 차이를 책임 배치·계약·상태·배치·call graph로 설명할 수 있는가?
4. 사람이 매번 주관적으로 판정하지 않고 반복 측정할 수 있는가?
5. metric과 목표를 한 문장으로 설명할 수 있는가?

QA별 ASR 상태는 `UNASSESSED`, `CANDIDATE`, `CONFIRMED_ASR`, `NOT_ASR` 중 하나로 관리한다. 현재 active draft QA는 모두 `UNASSESSED`다.
