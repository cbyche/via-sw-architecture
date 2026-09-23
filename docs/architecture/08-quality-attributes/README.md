# Quality Concerns and Quality Attributes

이 디렉터리는 VIA의 상위 품질 관심사(QC)를 정리하고, Architecture 후보를 비교할 측정 가능한 품질 속성(QA)을 정의한다. **QA category와 single-metric 방향은 합의됐지만 target, score band, workload, DP applicability와 ASR은 아직 사용자 검토 초안이다.**

## Current documents

| Document | Role |
| --- | --- |
| [Quality Model](./quality-model.md) | category range, active QA catalog, QA-11과 driver QA 관계, ASR 판정 계약 |
| [Machine-readable QA Registry](./qa-catalog-draft.json) | active ID, category, single metric, legacy ID migration |
| [QA Review Plan and Decision Log](./qa-review-plan.md) | 합의·미결정·다음 순서 |
| [Voice Responsiveness](./voice-responsiveness.md) | QA-01~QA-04 semantic contract |
| [Correctness & Continuity](./correctness-and-continuity.md) | QA-11~QA-15 semantic and oracle boundary |
| [Reliability & Resource](./reliability-and-resource.md) | QA-31, QA-32, QA-41 semantic contract |
| [Evidence](./evidence/) | 외부 specification과 planning reference의 출처·한계 |

## Current QA catalog

| Category | Active draft IDs |
| --- | --- |
| Responsiveness | QA-01/02/03/04 |
| Correctness & Continuity | QA-11/12/13/14/15 |
| Modifiability | QA-21/22 |
| Reliability & Availability | QA-31/32 |
| Resource Efficiency | QA-41 |
| Privacy & Security | QA-51 |

QA-61~69 Observability 범위는 예약했지만 현재 active QA는 없다. 현재 확정된 ASR도 없다.

## Interpretation rules

- 하나의 QA는 하나의 대표 metric만 가진다.
- QA-11은 통합 outcome이고 QA-12~15는 원인별 driver다. weighted total에 독립 표처럼 중복 가중하지 않는다.
- fast but invalid response는 성공이 아니다.
- Voice latency endpoint는 실제 audible onset 또는 QA-04의 실제 audio stop이다.
- QA-11~15는 사전 승인된 machine-readable predicate로 판정한다.
- QA-21/22는 frozen Architecture Element의 변경 수 평균을 대표 metric으로 유지한다.
- QA-31은 정확한 recovery time, QA-32는 excess blast radius를 별도로 측정한다.
- QA-41은 target device 전체 후보 경로의 peak committed memory를 측정한다.
- QA-51은 정상 기능에 필요한 최소 범위를 넘긴 보호정보 노출을 센다.
- mandatory action/access gate 위반은 다른 QA 점수로 상쇄하지 않는다.

## ID generation note

이번 category-range generation에서 기존 active QA-05/07/08/09/11은 각각 QA-11/21/22/31/51로 이동했다. 과거 archive의 번호와 문서는 변경하지 않는다. 현재 QA-04와 QA-12는 과거 동명의 번호를 재사용한 것이 아니라 새 generation에서 정의한 새로운 QA다.
