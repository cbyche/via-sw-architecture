# Quality Attributes and Working-12

이 디렉터리는 VIA의 Architecture-significant quality를 도출하고, 현재 평가 지표인 Working-12를 정의한다. 지표는 후보 구조의 차이를 관찰하기 위한 것이며, 아직 실행하지 않은 값을 암시하지 않는다.

## Current documents

| Document | Role |
| --- | --- |
| [Architecture Significant Requirements](./architecture-significant-requirements.md) | UC와 변화 시나리오에서 ASR을 도출한 근거 |
| [Voice Responsiveness](./voice-responsiveness.md) | W-01~W-03의 current semantic contract |
| [Reserve Driver Candidates](./reserve-driver-candidates.md) | 보조·예비 driver 후보 |
| [Reserve Formal Reassessment](./reserve-formal-reassessment.md) | reserve 항목의 재평가 |
| [Evidence](./evidence/) | 외부 specification과 planning reference의 출처·한계 |

## Working-12 map

| ID | Metric focus | Current state |
| --- | --- | --- |
| W-01 | Delegated Task Result Responsiveness | Voice definition current; harness/target/result pending |
| W-02 | VIA Direct Voice Response Responsiveness | Voice definition current; harness/target/result pending |
| W-03 | Agent Progress Voice Feedback Responsiveness | Voice definition current; harness/target/result pending |
| W-04 | Concurrent Task Performance Isolation | scoring baseline review required after W-01 change |
| W-05 | Task Completion Effectiveness | active definition; execution evidence pending/reviewed per campaign |
| W-06 | Interaction & Task Continuity | active definition |
| W-07 | Agent Ecosystem Interoperability & Substitutability | active definition |
| W-08 | Evolvability & Maintainability | active definition |
| W-09 | Recovery Timeliness & Recoverability | active definition |
| W-10 | Dependency Failure Containment & Graceful Degradation | active definition |
| W-11 | Privacy Exposure Minimization | active definition |
| W-12 | Action & Access Safety | active definition |

W-01~W-03에는 [Voice Responsiveness](./voice-responsiveness.md)가 우선한다. 이전 W12-G1의 이름, endpoint, target, score, 결과는 [historical archive](../../archive/w12-g1/README.md)에만 있으며 새 측정값으로 재사용하지 않는다.

## Interpretation rules

- Quality definition, measurement contract, target, score, result는 서로 다른 상태를 가진다.
- 중요하다는 이유만으로 모든 W가 모든 DP에 applicable한 것은 아니다.
- DP별 applicability는 실제 call graph와 구조적 인과관계로 판단한다.
- fast but invalid response는 성공이 아니다.
- Voice latency의 user-visible endpoint는 first audible onset이다.
- 공개 model specification과 token-rate calculation은 planning evidence이지 VIA product measurement가 아니다.
