# Quality Concerns and Quality Attributes

이 디렉터리는 ISO/IEC 25010을 바탕으로 VIA의 상위 품질 관심사(QC)를 정리하고, Architecture 후보를 비교할 측정 가능한 품질 속성(QA)을 정의한다. **QA catalog는 아직 사용자 검토 초안이며 ASR은 선정하지 않았다.**

## Current documents

| Document | Role |
| --- | --- |
| [Quality Model, QA Catalog, and ASR Selection](./quality-model.md) | QC·QA·ASR의 관계, 현재 QA catalog와 ASR 판정 계약 |
| [Machine-readable QA Registry](./qa-catalog-draft.json) | 활성/퇴역 ID, QC mapping, draft metric·target |
| [QA Review Plan and Decision Log](./qa-review-plan.md) | 이번 검토의 합의·미결정·다음 순서 |
| [Voice Responsiveness](./voice-responsiveness.md) | QA-01~QA-03의 current semantic contract |
| [Evidence](./evidence/) | 외부 specification과 planning reference의 출처·한계 |

## Current QA catalog

| ID | Quality Attribute | Current state |
| --- | --- | --- |
| QA-01 | Delegated Path VIA Responsiveness | user-review draft; harness/result pending |
| QA-02 | VIA Direct Voice Response Responsiveness | user-review draft; harness/result pending |
| QA-03 | Agent Progress Voice Feedback Responsiveness | user-review draft; harness/result pending |
| QA-05 | VIA Request Handling Correctness | user-review draft; structured oracle pending |
| QA-07 | Agent Change Locality | user-review draft; element ledger pending |
| QA-08 | Model & Context Change Locality | user-review draft; element ledger pending |
| QA-09 | Correct Task Recovery Time | user-review draft; fault strata pending |
| QA-11 | Protected Data Exposure Minimization | user-review draft; workload/oracle pending |

QA-04/06/10/12의 직전 정의는 현재 catalog가 아니다. 병합·제외 근거와 현재 위치는 [Quality Model](./quality-model.md)에 있고, 원문은 [QA Catalog Draft v1 Archive](../../archive/qa-catalog-draft-v1/README.md)에 보존한다. Catalog가 승인될 때까지 번호를 재사용하거나 당겨 붙이지 않는다. 과거 W-series와 numbered ASR baseline은 [historical archive](../../archive/w12-g1/README.md)에만 보존한다.

## Interpretation rules

- QC는 상위 품질 관심사이고 QA는 측정 가능한 비교 단위다.
- ASR은 별도 번호가 아니라 Architecture 영향이 확인된 QA의 분류다.
- 중요하다는 이유만으로 모든 QA가 모든 DP에 applicable한 것은 아니다.
- DP별 applicability는 실제 call graph와 구조적 인과관계로 판단한다.
- fast but invalid response는 성공이 아니다.
- Voice latency의 user-visible endpoint는 first audible onset이다.
- 공개 model specification과 token-rate calculation은 planning evidence이지 VIA product measurement가 아니다.
- QA-05는 실행 중 사람의 주관적 채점이 아니라 사전 승인된 machine-readable predicate로 판정한다.
- QA-07/08의 Architecture Element는 파일·클래스 수가 아니라 책임·계약·상태·배치 단위다.
