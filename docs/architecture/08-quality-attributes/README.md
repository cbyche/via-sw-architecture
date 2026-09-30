# Quality Concerns and Quality Attributes

이 디렉터리는 VIA의 상위 품질 관심사(QC)를 정리하고, Architecture 후보를 비교할 측정 가능한 품질
속성(QA)을 정의한다. **QA-09·19·29·39는 핵심 `CONFIRMED_ASR`이며 공통 system target·score band
proposal을 작성했다. 이 숫자와 DP별 Shared Spine patch·mock profile·fixture digest·applicable
population은 Measurement Freeze 전까지 미확정이다.**

## Current documents

| Document | Role |
| --- | --- |
| [ISO/IEC 25010 품질 대응](./iso-25010-quality-basis.md) | 2023판 특성·부특성에 기존 QA와 DP별 추가 품질 질문을 연결한 근거 |
| [Quality Model](./quality-model.md) | category range, active QA catalog, QA-11과 driver QA 관계, ASR 판정 계약 |
| [Machine-readable QA Registry](./qa-catalog-draft.json) | active ID, category, single metric, legacy ID migration |
| [Core ASR Contract](./core-asr-contract.md) | QA-09/19/29/39 정의·집계·실패 처리와 VIA-DP-01~18 적용 원장 |
| [QA Review Plan and Decision Log](./qa-review-plan.md) | 합의·미결정·다음 순서 |
| [Voice Responsiveness](./voice-responsiveness.md) | QA-01~QA-04 semantic contract |
| [Task Control Responsiveness](./interaction-control-responsiveness.md) | QA-05 semantic contract |
| [Correctness & Continuity](./correctness-and-continuity.md) | QA-11~QA-15 semantic and oracle boundary |
| [Reliability & Resource](./reliability-and-resource.md) | QA-31, QA-32, QA-41 semantic contract |
| [Observability](./observability.md) | QA-61/62 trace completeness and result reproducibility |
| [Evidence](./evidence/) | 외부 specification과 planning reference의 출처·한계 |

## Current QA catalog

| Category | Active draft IDs |
| --- | --- |
| Responsiveness | **QA-09 core ASR**; QA-01/02/03/05 input, QA-04 regression |
| Correctness & Continuity | **QA-19 core ASR**; QA-11/12 field source, QA-13~15 회귀 진단 |
| Modifiability | **QA-29 core ASR**; QA-21~23 change source |
| Reliability & Availability | **QA-39 core ASR**; QA-31/32 fault diagnostic |
| Resource Efficiency | QA-41 |
| Privacy & Security | QA-51 |
| Observability | QA-61/62 |

현재 확정된 핵심 ASR은 **QA-09, QA-19, QA-29, QA-39**다. 모든 VIA-DP-01~18은 같은 A/B package에서 네 QA의 역할을 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED` 중 하나로 명시한다. 구조 인과가 없거나 물리적으로 참여하지 않는 QA를 억지로 측정값으로 만들지 않는다.

## Interpretation rules

- 하나의 QA는 하나의 대표 metric만 가진다.
- 상세 QA는 네 core ASR의 input·diagnostic이며 독립 ASR 점수로 중복 가중하지 않는다.
- fast but invalid response는 성공이 아니다.
- Voice latency endpoint는 실제 audible onset 또는 QA-04의 실제 audio stop이다.
- QA-05는 사용자 제어 입력부터 올바른 Task 처리 상태가 보이거나 들릴 때까지 측정한다.
- QA-11~15는 사전 승인된 machine-readable predicate로 판정한다.
- QA-29는 DP별 applicable QA-21~23 change item을 하나의 단순 평균으로 집계한다.
- QA-39는 QA-31 recovery와 QA-32 containment 조건을 모두 만족한 fault trial의 단순 성공률이다.
- QA-41은 target device 전체 후보 경로의 peak committed memory를 측정한다.
- QA-51은 정상 기능에 필요한 최소 범위를 넘긴 보호정보 노출을 센다.
- QA-61은 완전하게 재구성 가능한 실행 trace 비율, QA-62는 정확히 다시 생성한 평가 결과 비율을 센다.
- mandatory action/access gate 위반은 다른 QA 점수로 상쇄하지 않는다.

## ID generation note

2026-09-28 generation은 기존 ID를 옮기지 않고 비어 있던 category 끝 번호 QA-09/19/29/39를 통합 core ASR에 사용했다. 과거 archive의 old QA-09는 recovery time이었고 현재 QA-09와 의미가 다르다. 과거 번호와 문서는 변경하지 않는다.
