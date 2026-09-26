# Quality Model and Draft QA Catalog

> 상태: **USER REVIEW DRAFT / category·single-metric 구조 합의 / ASR 후보 10개 선정·확정 ASR 없음**
>
> 이번 generation은 QA를 품질 계열별 번호 범위로 한 번 재번호화했다. 각 QA는 하나의 대표 metric만 가진다. 이름·metric은 current draft이며 target, score band, workload와 ASR은 추가 승인 전까지 확정값이 아니다.

## 1. 분류 체계

**모든 QA의 공통 전제:** VIA는 S2S 모델 1개와 semantic LLM 1개만 사용한다. Component·Task마다 별도 모델을 올리지 않는다. 역할별 프롬프트·호출 수·세션은 달라질 수 있으므로 지연에는 실제 공유 모델 대기와 호출 graph를, 메모리에는 실제 cache·buffer를 포함한다. 모델 복제 유무로 DP의 trade-off를 만들지 않는다. 정의는 [Model Boundary](../01-system-mission-and-boundary.md#13-ai-model-boundary), 비교 조건은 [FA-10](../06-fixed-assumptions.md#65-모델-비교-규칙)을 따른다.

QA-41은 카탈로그에서 유지하되 현재 메모리 상한 요구나 의미 있는 구조 차이의 근거가 없으므로 최종 보고서의 핵심 ASR로 우선 추천하지 않는다. 실측 없이 ‘수 MB 차이’나 ‘모델 여러 개 절약’을 주장하지 않는다. 현재 번호 체계만 새 보고서·표에 사용하며 이전 QA ID는 이력 매핑에만 남긴다.

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

현재 확정된 ASR은 없다. 다만 [VIA Core Evaluation Profile](../11-measurement/evaluation-profile.md)에서 QA-01, 02, 05, 11, 12, 21, 22, 31, 32, 61을 한 campaign에서 검증할 `CANDIDATE` working set으로 선정했다. QA가 중요해 보여도 정상적인 Architecture 대안 사이에서 구조적 인과와 실제 차이를 확인하지 못하면 `CONFIRMED_ASR`로 올리지 않는다.

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

| ID | Quality Attribute | 한국어 한 문장 설명 | 단일 대표 Metric | 상태 |
| --- | --- | --- | --- | --- |
| QA-01 | Delegated Path VIA Responsiveness | 사용자가 작업을 맡겼을 때 Agent의 실제 작업시간을 제외하고 VIA가 위임하고 결과를 얼마나 빨리 전달하는지를 본다. | 가장 느린 대표 case의 p95 VIA 처리시간(Agent 실행 제외) | semantic definition current; target draft |
| QA-02 | VIA Direct Voice Response Responsiveness | Agent 도움 없이 VIA가 직접 답하는 음성 요청에 얼마나 빨리 응답하는지를 본다. | 가장 느린 대표 case의 p95 직접 Voice 응답시간 | semantic definition current; target draft |
| QA-03 | Agent Progress Voice Feedback Responsiveness | Agent의 진행 상태가 나온 뒤 VIA가 그 소식을 사용자에게 음성으로 얼마나 빨리 알려주는지를 본다. | 가장 느린 대표 case의 p95 Agent 상태 Voice 전달시간 | semantic definition current; target draft |
| QA-04 | Voice Interruption Responsiveness | 사용자가 말로 끼어들었을 때 VIA가 재생 중인 음성을 얼마나 빨리 멈추는지를 본다. | 가장 느린 대표 case의 p95 barge-in 후 음성 정지시간 | semantic draft; target pending |
| QA-05 | Task Control Responsiveness | 사용자가 작업을 취소하거나 정정했을 때 VIA가 올바른 처리 상태를 얼마나 빨리 알려주는지를 본다. | 가장 느린 대표 control case의 p95 올바른 처리상태 응답시간 | semantic draft; target pending |
| QA-11 | VIA Request Handling Correctness | VIA가 사용자 요청을 이해하고 처리하여 응답이나 위임과 상태 연결까지 전체적으로 올바르게 완료하는지를 본다. | 올바르게 처리된 전체 요청 run 비율 | previous QA-05 metric preserved; integrated outcome; target draft |
| QA-12 | Request Semantic Resolution Correctness | VIA가 사용자의 목표와 대상, 조건 및 요청 사이의 관계를 올바르게 이해하는지를 본다. | 의미를 올바르게 해석한 run 비율 | driver draft; target pending |
| QA-13 | Task & Interaction Binding Correctness | VIA가 요청과 응답, 제어 및 외부 소식을 의도한 대화와 작업에 정확히 연결하는지를 본다. | 올바른 Task·Interaction에 연결한 run 비율 | driver draft; target pending |
| QA-14 | Async Task State Convergence Correctness | 외부 소식이 늦거나 중복되거나 순서가 바뀌어도 VIA가 최종적으로 올바른 작업 상태를 유지하는지를 본다. | 비동기 event 뒤 올바른 상태로 수렴한 run 비율 | driver draft; target pending |
| QA-15 | Interaction & Task Continuity Correctness | 음성과 텍스트, 대화와 작업 사이를 오가더라도 필요한 맥락과 작업 관계가 유지되는지를 본다. | 전환 뒤 필요한 대화·Task 관계가 유지된 scenario 비율 | driver draft; target pending |
| QA-21 | Agent Change Locality | Agent를 추가하거나 바꿀 때 그 변화가 VIA Architecture의 여러 부분으로 얼마나 적게 퍼지는지를 본다. | Agent 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack current; target draft |
| QA-22 | Model, Context & State Change Locality | Model과 Context 또는 상태 계약을 바꿀 때 그 변화가 VIA Architecture의 여러 부분으로 얼마나 적게 퍼지는지를 본다. | Model·Context·State 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack current; target draft |
| QA-23 | Experiment & Logging Change Locality | 새로운 실험이나 로그를 추가할 때 그 변화가 VIA Architecture의 여러 부분으로 얼마나 적게 퍼지는지를 본다. | 실험·로그 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack draft; target pending |
| QA-31 | Correct Task Recovery Time | 장애가 발생한 뒤 VIA가 영향받은 작업을 올바른 상태와 결과로 되돌리는 데 걸리는 시간을 본다. | 가장 느린 fault 종류의 p95 전체 Task 복구시간 | recovery draft; target draft |
| QA-32 | Fault Blast Radius | 한 부분의 장애가 그 부분에 의존하지 않는 사용자 기능이나 작업까지 불필요하게 멈추게 하는 범위를 본다. | 한 fault가 불필요하게 함께 중단시킨 사용자 기능·Task의 최대 수 | semantic draft; target pending |
| QA-41 | Target Device Memory Footprint | VIA가 실제 설치될 PC에서 업무를 처리하는 동안 사용하는 최대 메모리 크기를 본다. | 가장 무거운 workload의 p95 최대 메모리 사용량 | semantic draft; target pending |
| QA-51 | Protected Data Exposure Minimization | VIA가 업무에 꼭 필요한 범위를 넘어 보호정보를 외부에 드러내지 않는지를 본다. | 전체 workload에서 필요 이상 노출된 보호정보 단위 수 | exposure draft; target draft |
| QA-61 | Execution Trace Completeness | 저장된 로그만으로 한 요청이 어떤 경로와 상태를 거쳐 어떤 결과가 되었는지 재구성할 수 있는지를 본다. | 로그로 전체 실행을 재구성할 수 있는 run 비율 | semantic draft; target pending |
| QA-62 | Evidence Reproducibility | 저장된 측정 근거만으로 이전에 보고한 평가 결과를 정확히 다시 만들 수 있는지를 본다. | 저장된 근거로 같은 평가 결과를 정확히 다시 만든 비율 | semantic draft; target pending |

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

QA별 ASR 상태는 `UNASSESSED`, `CANDIDATE`, `CONFIRMED_ASR`, `NOT_ASR` 중 하나로 관리한다.

| 상태 | QA |
| --- | --- |
| `CANDIDATE` | QA-01, QA-02, QA-05, QA-11, QA-12, QA-21, QA-22, QA-31, QA-32, QA-61 |
| `UNASSESSED` | QA-03, QA-04, QA-13, QA-14, QA-15, QA-23, QA-41, QA-51, QA-62 |
| `CONFIRMED_ASR` | 없음 |
| `NOT_ASR` | 없음 |

`UNASSESSED`도 harness에서 제외한다는 뜻이 아니다. QA-03/04/13~15/62는 같은 trace에서 보조 QA로 계산하고, QA-41은 target-device 진단값으로 기록한다.
