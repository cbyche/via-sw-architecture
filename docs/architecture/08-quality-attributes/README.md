# Quality Attributes and Working-12

이 디렉터리는 VIA의 Architecture-significant quality를 도출하고, 현재 평가 지표인 Working-12를 정의한다. 지표는 후보 구조의 차이를 관찰하기 위한 것이며, 아직 실행하지 않은 값을 암시하지 않는다.

## Current documents

| Document | Role |
| --- | --- |
| [Architecture Significant Requirements](./architecture-significant-requirements.md) | UC와 변화 시나리오에서 ASR을 도출한 근거 |
| [Voice Responsiveness](./voice-responsiveness.md) | W-01~W-03의 current semantic contract |
| [Evidence](./evidence/) | 외부 specification과 planning reference의 출처·한계 |

## ASR과 Working-12의 관계

7개 ASR은 제품이 지켜야 할 상위 품질 요구이고, Working-12는 그 품질을 Architecture 비교에서 관찰하기 위해 분해한 측정 관점이다. 따라서 `ASR 7개`와 `W 12개`는 서로 충돌하는 두 목록이 아니다. W는 새로운 요구사항 번호가 아니라 측정·평가 ID다.

과거 reserve QA 선정 과정은 [historical archive](../../archive/w12-g1/README.md)로 이동했다. 현재 평가에서는 아래 Working-12 map과 [Measurement Guide](../11-measurement/README.md)만 사용한다.

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
