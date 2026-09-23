# Quality Concerns and Quality Attributes

이 디렉터리는 ISO/IEC 25010을 바탕으로 VIA의 상위 품질 관심사(QC)를 정리하고, Architecture 후보를 비교할 측정 가능한 품질 속성(QA)을 정의한다. 현재 ASR은 아직 선정하지 않았다.

## Current documents

| Document | Role |
| --- | --- |
| [Quality Model, QA Catalog, and ASR Selection](./quality-model.md) | QC·QA·ASR의 관계, 현재 QA catalog와 ASR 판정 계약 |
| [Voice Responsiveness](./voice-responsiveness.md) | QA-01~QA-03의 current semantic contract |
| [Evidence](./evidence/) | 외부 specification과 planning reference의 출처·한계 |

## Current QA catalog

| ID | Quality Attribute | Current state |
| --- | --- | --- |
| QA-01 | Delegated Task Result Responsiveness | Voice definition current; harness/target/result pending |
| QA-02 | VIA Direct Voice Response Responsiveness | Voice definition current; harness/target/result pending |
| QA-03 | Agent Progress Voice Feedback Responsiveness | Voice definition current; harness/target/result pending |
| QA-04 | Concurrent Task Performance Isolation | contract review required after QA-01 change |
| QA-05 | Task Completion Effectiveness | active definition; execution evidence pending |
| QA-06 | Interaction & Task Continuity | active definition |
| QA-07 | Agent Ecosystem Interoperability & Substitutability | active definition |
| QA-08 | Evolvability & Maintainability | active definition |
| QA-09 | Recovery Timeliness & Recoverability | active definition |
| QA-10 | Dependency Failure Containment & Graceful Degradation | active definition |
| QA-11 | Privacy Exposure Minimization | active definition |
| QA-12 | Action & Access Safety | active definition |

과거 W-series와 numbered ASR baseline은 [historical archive](../../archive/w12-g1/README.md)에만 보존한다.

## Interpretation rules

- QC는 상위 품질 관심사이고 QA는 측정 가능한 비교 단위다.
- ASR은 별도 번호가 아니라 Architecture 영향이 확인된 QA의 분류다.
- 중요하다는 이유만으로 모든 QA가 모든 DP에 applicable한 것은 아니다.
- DP별 applicability는 실제 call graph와 구조적 인과관계로 판단한다.
- fast but invalid response는 성공이 아니다.
- Voice latency의 user-visible endpoint는 first audible onset이다.
- 공개 model specification과 token-rate calculation은 planning evidence이지 VIA product measurement가 아니다.
