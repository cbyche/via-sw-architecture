# Quality Model and Core ASR Catalog

> 상태: **ACTIVE CORE-ASR BASELINE / QA-09·19·29·39 CONFIRMED_ASR / measurement freeze pending**
>
> 2026-09-28 사용자 결정으로 네 통합 QA를 핵심 ASR로 확정했다. 기존 상세 QA는 새 core QA의 measurement stratum·field family·change pack·fault diagnostic으로 유지하며 독립 ASR 점수로 중복 가중하지 않는다. target, score band, DP별 fixture와 반복은 별도 Measurement Freeze 전까지 미확정이다.

## 1. 분류 체계

**모든 QA의 공통 전제:** VIA는 S2S 모델 1개와 semantic LLM 1개만 사용한다. Component·Task마다 별도 모델을 올리지 않는다. 역할별 프롬프트·호출 수·세션은 달라질 수 있으므로 지연에는 실제 공유 모델 대기와 호출 graph를, 메모리에는 실제 cache·buffer를 포함한다. 모델 복제 유무로 DP의 trade-off를 만들지 않는다. 정의는 [Model Boundary](../01-system-mission-and-boundary.md#13-ai-model-boundary), 비교 조건은 [FA-10](../06-fixed-assumptions.md#65-모델-비교-규칙)을 따른다.

QA-41은 진단 QA로 유지한다. 현재 메모리 상한 요구가 없고 공유 local semantic model이 절대값을 지배하며, 현재 DP-11 reference path의 후보 차이는 약 3.7 MiB에 불과했다. 실측 없이 ‘모델 여러 개 절약’을 주장하지 않는다.

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

현재 확정된 핵심 ASR은 **QA-09, QA-19, QA-29, QA-39**다. 정의·집계·실패 처리·DP별 초기 적용 원장은 [Core ASR Contract](./core-asr-contract.md)가 기준이다. 확정은 이 네 품질 질문을 모든 DP A/B에서 평가한다는 의미이며, target 승인·harness 구현·실행 완료나 특정 후보의 승리를 뜻하지 않는다.

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
| QA-09 | Average VIA-attributable Interaction Responsiveness | delegation·direct response·Agent progress·Task control에서 VIA가 책임지는 처리시간을 사용자 관점의 하나의 평균으로 본다. | applicable scored trial 전체의 평균 VIA 책임시간 | **CONFIRMED_ASR**; target pending |
| QA-11 | VIA Request Handling Correctness | VIA가 사용자 요청을 이해하고 처리하여 응답이나 위임과 상태 연결까지 전체적으로 올바르게 완료하는지를 본다. | 올바르게 처리된 전체 요청 run 비율 | previous QA-05 metric preserved; integrated outcome; target draft |
| QA-12 | Request Semantic Resolution Correctness | VIA가 사용자의 목표와 대상, 조건 및 요청 사이의 관계를 올바르게 이해하는지를 본다. | 의미를 올바르게 해석한 run 비율 | driver draft; target pending |
| QA-13 | Task & Interaction Binding Correctness | VIA가 요청과 응답, 제어 및 외부 소식을 의도한 대화와 작업에 정확히 연결하는지를 본다. | 올바른 Task·Interaction에 연결한 run 비율 | driver draft; target pending |
| QA-14 | Async Task State Convergence Correctness | 외부 소식이 늦거나 중복되거나 순서가 바뀌어도 VIA가 최종적으로 올바른 작업 상태를 유지하는지를 본다. | 비동기 event 뒤 올바른 상태로 수렴한 run 비율 | driver draft; target pending |
| QA-15 | Interaction & Task Continuity Correctness | 음성과 텍스트, 대화와 작업 사이를 오가더라도 필요한 맥락과 작업 관계가 유지되는지를 본다. | 전환 뒤 필요한 대화·Task 관계가 유지된 scenario 비율 | driver draft; target pending |
| QA-19 | VIA Request Handling Field Accuracy | QA-11의 요청 처리 결과와 QA-12의 의미 해석을 구성하는 applicable field가 얼마나 정확한지 본다. | 전체 applicable QA-11/12 oracle field 중 correct field 비율 | **CONFIRMED_ASR**; target pending |
| QA-21 | Agent Change Locality | Agent를 추가하거나 바꿀 때 그 변화가 VIA Architecture의 여러 부분으로 얼마나 적게 퍼지는지를 본다. | Agent 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack current; target draft |
| QA-22 | Model, Context & State Change Locality | Model과 Context 또는 상태 계약을 바꿀 때 그 변화가 VIA Architecture의 여러 부분으로 얼마나 적게 퍼지는지를 본다. | Model·Context·State 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack current; target draft |
| QA-23 | Experiment & Logging Change Locality | 새로운 실험이나 로그를 추가할 때 그 변화가 VIA Architecture의 여러 부분으로 얼마나 적게 퍼지는지를 본다. | 실험·로그 변화 한 건당 바뀌는 Architecture Element 평균 수 | change pack draft; target pending |
| QA-29 | Average Architecture Change Locality | 해당 DP에 적용 가능한 Agent·Model·Context·State·Experiment·Logging 변화를 수용할 때 평균 몇 개의 Architecture Element가 바뀌는지 본다. | applicable change당 평균 changed Architecture Element 수 | **CONFIRMED_ASR**; target pending |
| QA-31 | Correct Task Recovery Time | 장애가 발생한 뒤 VIA가 영향받은 작업을 올바른 상태와 결과로 되돌리는 데 걸리는 시간을 본다. | 가장 느린 fault 종류의 p95 전체 Task 복구시간 | recovery draft; target draft |
| QA-32 | Fault Blast Radius | 한 부분의 장애가 그 부분에 의존하지 않는 사용자 기능이나 작업까지 불필요하게 멈추게 하는 범위를 본다. | 한 fault가 불필요하게 함께 중단시킨 사용자 기능·Task의 최대 수 | semantic draft; target pending |
| QA-39 | Fault Containment & Recovery Success Rate | 장애가 필요한 범위 안에 머물고 모든 영향 Task가 제한 시간 안에 올바르게 복구된 비율을 본다. | 전체 applicable fault trial 중 containment·recovery PASS 비율 | **CONFIRMED_ASR**; target pending |
| QA-41 | Target Device Memory Footprint | VIA가 실제 설치될 PC에서 업무를 처리하는 동안 사용하는 최대 메모리 크기를 본다. | 가장 무거운 workload의 p95 최대 메모리 사용량 | semantic draft; target pending |
| QA-51 | Protected Data Exposure Minimization | VIA가 업무에 꼭 필요한 범위를 넘어 보호정보를 외부에 드러내지 않는지를 본다. | 전체 workload에서 필요 이상 노출된 보호정보 단위 수 | exposure draft; target draft |
| QA-61 | Execution Trace Completeness | 저장된 로그만으로 한 요청이 어떤 경로와 상태를 거쳐 어떤 결과가 되었는지 재구성할 수 있는지를 본다. | 로그로 전체 실행을 재구성할 수 있는 run 비율 | semantic draft; target pending |
| QA-62 | Evidence Reproducibility | 저장된 측정 근거만으로 이전에 보고한 평가 결과를 정확히 다시 만들 수 있는지를 본다. | 저장된 근거로 같은 평가 결과를 정확히 다시 만든 비율 | semantic draft; target pending |

## 4. Core ASR과 상세 측정의 관계

기존 상세 QA는 새 core ASR의 원시 입력과 원인 분석에 사용한다.

```text
QA-09 average VIA time
  ← QA-01 delegation / QA-02 direct / QA-03 status / QA-05 control

QA-19 applicable field accuracy
  ← QA-11/12의 중복 없는 machine field registry
  ← QA-13~15는 별도 회귀 진단이며 자동 포함하지 않음

QA-29 average changed elements
  ← QA-21 Agent / QA-22 Model·Context·State / QA-23 Experiment·Logging pack

QA-39 fault success rate
  ← QA-31 recovery time·correctness / QA-32 excess blast radius
```

상세 값을 새 core QA와 별도 ASR 점수로 합산하지 않는다. QA-04는 Voice interruption regression, QA-41은 memory diagnostic, QA-51과 action/access rule은 safety qualification, QA-61/62는 evidence qualification이다.

## 5. Quality Concern coverage

| QC | Quality Concern | 현재 처리 |
| --- | --- | --- |
| QC-01 | User-Experienced Responsiveness | **QA-09**; QA-01/02/03/05 상세 stratum, QA-04 regression |
| QC-02 | VIA Request Handling Correctness | **QA-19**; QA-11/12 상세 field source |
| QC-03 | Interaction & Task Continuity | **QA-39**; QA-13~15 상세 회귀 진단, QA-31/32 fault 진단 |
| QC-04 | Agent Ecosystem Interoperability & Substitutability | **QA-29**; QA-21 change source |
| QC-05 | Evolvability & Maintainability | **QA-29**; QA-22/23 change source |
| QC-06 | Resource & Deployment Efficiency | QA-41. 다른 resource 값은 diagnostic으로만 보존하고 합성 QA를 추가하지 않음 |
| QC-07 | Concurrency & Capacity | 독립 QA 없음. 관련 QA의 workload condition |
| QC-08 | Reliability & Recoverability | **QA-39**; QA-14/31/32 상세 진단 |
| QC-09 | Privacy, Security & Action Safety | QA-51 + mandatory zero-violation gate |
| QC-10 | Observability & Evidence Integrity | QA-23, QA-61/62 + 모든 QA trace의 공통 evidence requirement |

## 6. 단일 metric 원칙

각 QA는 심사위원에게 한 문장으로 설명할 수 있는 대표 metric 하나만 가진다. component trace, case별 값, C/I/S/D breakdown, maximum, CPU/GPU sample과 failure reason은 대표 metric을 해석하는 supporting evidence이지 같은 QA의 추가 대표 metric이 아니다.

QA-09는 서로 다른 event를 동일한 `VIA-attributable milliseconds`로 정규화한 뒤 단순 평균하고, QA-39는 서로 다른 단위의 recovery time과 blast radius를 더하지 않고 사전 정의한 binary trial PASS로 통합한다. 세부 시작·종료 사건과 raw diagnostic은 유지한다.

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

2026-09-28 core-ASR generation은 빈 category 끝 번호를 새 통합 QA에 사용했다. 기존 상세 QA ID를 재번호화하지 않는다.

| 새 core QA | 통합하는 current detailed measurement |
| --- | --- |
| QA-09 | QA-01/02/03/05; QA-04 제외 |
| QA-19 | QA-11/12의 중복 없는 oracle field; QA-13~15 제외 |
| QA-29 | QA-21/22/23의 applicable change item |
| QA-39 | QA-31/32의 fault evidence로 만든 binary PASS |

아래 표는 그보다 앞선 category-range migration 이력이다. 당시의 old QA-09는 recovery time이었고 현재 새 QA-09와 의미가 다르다. old 정의는 historical provenance일 뿐 current QA-09로 읽지 않는다.

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
| `CANDIDATE` | 없음 |
| `UNASSESSED` | 없음 |
| `CONFIRMED_ASR` | **QA-09, QA-19, QA-29, QA-39** |
| `NOT_ASR` | QA-01~05, QA-11~15, QA-21~23, QA-31/32, QA-41, QA-51, QA-61/62 — supporting measurement·diagnostic·qualification으로 유지 |

`NOT_ASR`은 삭제나 미측정을 뜻하지 않는다. 상세 endpoint·field·change·fault evidence는 네 core ASR을 계산하고 설명하는 데 사용하며 QA-41·51·61·62와 action/access gate는 후보 자격을 확인한다.
