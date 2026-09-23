# Draft QA Measurement & Scoring Contract

> 상태: **USER REVIEW DRAFT / 실행·결과 생성 금지 / target·score 일부 미확정**
>
> 이 문서는 category-range QA catalog의 단일 대표 metric과 현재 target proposal을 기록한다. fixture·oracle·반복 수·target이 승인되기 전에는 Measurement Freeze나 새 결과를 만들지 않는다.

## 1. 공통 규칙

- 하나의 QA는 하나의 대표 metric만 가진다.
- raw metric과 score를 함께 공개한다. score만으로 A/B를 설명하지 않는다.
- 각 DP의 A/B를 직접 비교하고 다른 DP 조건은 고정한다.
- 실제 경로에 참여하지 않는 DP는 N/A로 둔다.
- 잘못된 결과, timeout, recovery failure와 gate violation을 성공 표본에서 제거하지 않는다.
- 후보 결과를 보기 전에 fixture, oracle, 반복 수, aggregation과 score boundary를 동결한다.
- QA-11은 integrated outcome, QA-12~15는 driver다. weighted total에 독립 표처럼 중복 가중하지 않는다.
- target이 PENDING인 QA에는 임의의 0~5 score를 만들지 않는다.

## 2. Active draft QA metric과 목표

| ID | 대표 Metric | 3점 목표 | 설명 |
| --- | --- | ---: | --- |
| QA-01 | worst-case canonical-case p95 delegated VIA active time | ≤2,000ms | 위임 전 VIA 구간 + Agent 결과 준비 후 VIA 구간 |
| QA-02 | worst-case canonical-case p95 direct Voice response | ≤1,000ms | user input end → first meaningful audible response |
| QA-03 | worst-case canonical-case p95 status-to-Voice | ≤1,000ms | source status available → first meaningful audible status |
| QA-04 | worst-case canonical-case p95 barge-in-to-audio-stop | PENDING | 실제 barge-in onset → interrupted audio의 마지막 audible sample |
| QA-11 | correct machine-oracle runs | ≥95% | previous QA-05 metric을 유지한 integrated correctness |
| QA-12 | strict semantic-resolution pass rate | PENDING | goal·referent·관계·Task Relation·route 의미 |
| QA-13 | strict interaction-binding pass rate | PENDING | control/event/result의 Request·Task·run·question binding |
| QA-14 | strict async-state-convergence pass rate | PENDING | reorder·duplicate·race 뒤 authoritative state |
| QA-15 | strict continuity-scenario pass rate | PENDING | 정상 channel·conversation·path transition |
| QA-21 | mean changed Architecture Elements / Agent change | ≤2 | A-01~A-09 전체 |
| QA-22 | mean changed Architecture Elements / non-Agent change | ≤3 | M-01~M-09 + C-01~C-06 전체 |
| QA-31 | worst fault-stratum p95 full Task recovery | ≤5,000ms | 정확한 state·result·control 복귀 |
| QA-32 | worst fault excess affected user-visible units | PENDING | necessary dependency closure 밖의 불필요한 장애 영향 |
| QA-41 | worst workload-stratum p95 peak committed memory | PENDING | target PC의 전체 후보-local accounting boundary |
| QA-51 | whole-workload union excess exposed protected units | 0 | recipient/purpose 최소 필요정보 초과분 |

## 3. 현재 draft 0~5 score bands

아래는 이전에 근거를 검토하던 QA의 ID만 migration한 proposal이다. 승인된 product SLO가 아니다.

| QA | 5 | 4 | 3 | 2 | 1 | 0 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| QA-01 | ≤1,000ms | ≤1,500ms | ≤2,000ms | ≤3,000ms | ≤4,000ms | >4,000ms |
| QA-02 | ≤500ms | ≤750ms | ≤1,000ms | ≤1,500ms | ≤2,000ms | >2,000ms |
| QA-03 | ≤500ms | ≤750ms | ≤1,000ms | ≤1,500ms | ≤2,000ms | >2,000ms |
| QA-11 | ≥99% | ≥97% | ≥95% | ≥90% | ≥80% | <80% |
| QA-21 | ≤1 | ≤1.5 | ≤2 | ≤3 | ≤4 | >4 |
| QA-22 | ≤1 | ≤2 | ≤3 | ≤4 | ≤5 | >5 |
| QA-31 | ≤1,000ms | ≤2,500ms | ≤5,000ms | ≤10,000ms | ≤20,000ms | >20,000ms 또는 recovery failure |
| QA-51 | 0 | 1 | 2~3 | 4~7 | 8~15 | ≥16 또는 unbounded |

QA-04/12/13/14/15/32/41의 target과 score band는 PENDING이다. 기존 QA의 threshold를 복사하지 않는다. QA-51은 5점만 target을 충족하며 나머지 band는 degradation 설명용이지 개인정보 노출 허용 기준이 아니다.

## 4. QA-01~QA-04 Responsiveness

정확한 event와 포함 구간은 [Voice Responsiveness](../08-quality-attributes/voice-responsiveness.md)와 [Event Boundary Contract](./event-boundary-contract.md)를 따른다. 각 QA는 canonical case별 p95 중 가장 느린 값을 대표값으로 사용한다.

- QA-01: (Agent ingress - user input end) + (audible result - result source availability)
- QA-02: first meaningful audible direct response - user input end
- QA-03: first meaningful audible status - status source availability
- QA-04: interrupted response last audible sample - actual barge-in speech onset

네 QA는 서로 다른 terminal event를 가지므로 평균내거나 하나의 Voice responsiveness score로 합치지 않는다. Qwen3-Omni 234ms와 VIA LLM token-rate estimate는 dependency planning input일 뿐 목표 근거나 실제 product p95가 아니다.

## 5. QA-11~QA-15 Correctness & Continuity

정확한 predicate 경계는 [Correctness & Continuity Contract](../08-quality-attributes/correctness-and-continuity.md)를 따른다.

~~~text
strict pass rate
= 100 × all-applicable-predicates-pass runs / all scored runs
~~~

QA별 predicate ownership:

- QA-11: previous QA-05와 같은 end-to-end request handling machine oracle
- QA-12: semantic meaning
- QA-13: identity와 pending interaction binding
- QA-14: event trace 뒤 authoritative state convergence
- QA-15: 정상 lifecycle transition의 identity/context continuity

같은 run을 여러 QA가 관찰할 수 있지만 QA-11과 driver QA를 하나의 합산 점수에 넣지 않는다. QA-12~15 전용 isolation fixture는 시험기가 정답을 후보에 주입하지 않으면서 해당 책임의 구조 인과를 분리해야 한다.

## 6. QA-21·QA-22 Change Locality

Architecture Element의 normative definition과 count rule은 [Architecture Element Definition](../10-element-definition.md)을 따른다.

~~~text
N(change) = |modified ∪ added ∪ removed Architecture Element IDs|
QA-21 = mean N(A-01...A-09)
QA-22 = mean N(M-01...M-09, C-01...C-06)
~~~

평균 changed element count가 각 QA의 유일한 대표 metric이다. change별 raw count, C/I/S/D breakdown, maximum, migration과 coordinated deployment 필요 여부는 supporting evidence다.

UNRESOLVED, 기능 축소 또는 미검증 change를 평균에서 제외해 작은 값을 만들지 않는다. 전체 frozen change pack에 대한 기능 유지 근거가 있을 때만 대표 평균을 낸다.

## 7. QA-31·QA-32 Reliability & Availability

정확한 계약은 [Reliability & Resource Contract](../08-quality-attributes/reliability-and-resource.md)를 따른다.

QA-31:

~~~text
sample = full correct Task recovery - actual loss of functionality/control
representative = worst fault-stratum p95
~~~

QA-32:

~~~text
excess_affected(f)
= unavailable or incorrect user-visible units
  outside fault f's pre-approved necessary dependency closure

representative = max excess_affected(f)
~~~

두 metric은 같은 fault run에서 계산할 수 있지만 합산하지 않는다. QA-31은 복구 시간, QA-32는 불필요한 영향 범위를 답한다.

## 8. QA-41 Target Device Memory Footprint

~~~text
trial value = workload window peak committed bytes
case value = trial p95
QA-41 = maximum case value across frozen workload strata
~~~

모든 VIA-owned process, local helper와 후보 선택 때문에 target PC에서 실행되는 local Model Runtime을 포함한다. process별 memory, CPU/GPU/VRAM과 energy는 supporting evidence다. target PC와 accounting boundary를 승인하기 전에는 target이나 score를 만들지 않는다.

## 9. QA-51 Protected Data Exposure Minimization

~~~text
excess_exposure
= externally accessible protected units
 - minimally necessary protected units for that recipient and purpose

QA-51 = whole-workload union of excess exposed units
~~~

같은 단위의 재전송은 한 번만 센다. recipient가 같은 사실이나 더 넓은 scope를 복원할 수 있으면 암호화·요약·handle도 노출로 판정한다. 모든 요청을 차단해 0을 만든 후보는 기능 적합성에 실패한다.

## 10. Workload condition과 qualification gate

동시 Task는 QA-01~04, QA-11~15, QA-31/32와 QA-41의 workload stratum이다. 이전 catalog의 concurrency ratio를 자동 승계하지 않는다.

다음 violation은 다른 QA 점수로 상쇄하지 않는다.

- wrong-target external Action
- retry/recovery 뒤 duplicate external Action
- revoked/mismatched approval 사용
- unauthorized access 또는 disclosure

한 건이라도 발생하면 해당 후보는 qualification gate에 실패한다.

## 11. 다음 승인 전 금지사항

- 새 QA 결과 생성
- PENDING target이나 score를 임의 작성
- draft target을 승인된 product SLO로 표현
- archived 결과를 현재 QA evidence로 재사용
- human free-text judge나 LLM-as-judge를 유일한 correctness oracle로 사용
- 후보 결과를 본 뒤 element granularity, change pack, fault unit 또는 score band 변경
