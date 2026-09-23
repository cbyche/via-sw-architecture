# Draft QA Measurement & Scoring Contract

> 상태: **USER REVIEW DRAFT / 실행·결과 생성 금지 / QA catalog 미확정**
>
> 이 문서는 active draft QA의 측정 방향과 0~5 score proposal을 기록한다. fixture·oracle·반복 수·target이 승인되기 전에는 Measurement Freeze나 새 결과를 만들지 않는다.

## 1. 공통 규칙

- 모든 QA는 0~5의 6단계 점수를 사용한다.
- raw metric과 score를 함께 공개한다. score만으로 A/B를 설명하지 않는다.
- 각 DP의 A/B를 직접 비교하고 다른 DP 조건은 고정한다.
- 실제 경로에 참여하지 않는 DP는 `N/A`로 둔다.
- 잘못된 결과, timeout과 복구 실패를 성공 표본에서 제거하지 않는다.
- 후보 결과를 보기 전에 fixture, oracle, 반복 수, aggregation과 score boundary를 동결한다.
- 현재 모든 target과 band는 review proposal이며 승인값이 아니다.

## 2. Active draft QA metric과 목표

| ID | 대표 Metric | 3점 목표 | 설명 |
| --- | --- | ---: | --- |
| QA-01 | worst-case case p95 delegated VIA active time | ≤2,000ms | 위임 전 VIA 구간 + Agent 결과 준비 후 VIA 구간 |
| QA-02 | worst-case case p95 direct Voice response | ≤1,000ms | user input end → first meaningful audible response |
| QA-03 | worst-case case p95 status-to-Voice | ≤1,000ms | source status available → first meaningful audible status |
| QA-05 | correct VIA request-handling runs / all runs | ≥95% | Downstream Agent 업무 품질 제외 |
| QA-07 | mean changed Architecture Elements / Agent change | ≤2 | A-01~A-09 |
| QA-08 | mean changed Architecture Elements / non-Agent change | ≤3 | approved AI/System change pack |
| QA-09 | worst fault-stratum p95 full Task recovery | ≤5,000ms | 정확한 상태·결과·제어 복귀 |
| QA-11 | excess protected information units exposed externally | 0 | 최소 필요정보를 초과한 workload union |

## 3. Draft 0~5 score bands

| QA | 5 | 4 | 3 | 2 | 1 | 0 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| QA-01 | ≤1,000ms | ≤1,500ms | ≤2,000ms | ≤3,000ms | ≤4,000ms | >4,000ms |
| QA-02 | ≤500ms | ≤750ms | ≤1,000ms | ≤1,500ms | ≤2,000ms | >2,000ms |
| QA-03 | ≤500ms | ≤750ms | ≤1,000ms | ≤1,500ms | ≤2,000ms | >2,000ms |
| QA-05 | ≥99% | ≥97% | ≥95% | ≥90% | ≥80% | <80% |
| QA-07 | ≤1 | ≤1.5 | ≤2 | ≤3 | ≤4 | >4 |
| QA-08 | ≤1 | ≤2 | ≤3 | ≤4 | ≤5 | >5 |
| QA-09 | ≤1,000ms | ≤2,500ms | ≤5,000ms | ≤10,000ms | ≤20,000ms | >20,000ms 또는 복구 실패 |
| QA-11 | 0 | 1 | 2~3 | 4~7 | 8~15 | ≥16 또는 unbounded |

QA-11은 5점만 목표를 충족한다. 나머지는 A/B 상대 차이를 보여주기 위한 degradation band이며 개인정보 노출 허용 기준이 아니다.

## 4. QA-01~QA-03 Voice responsiveness

정확한 event와 포함 구간은 [Voice Responsiveness](../08-quality-attributes/voice-responsiveness.md)를 따른다. 여러 case p95를 평균내지 않고 가장 느린 canonical case p95를 대표값으로 사용한다.

1초 direct/status 목표는 자연스러운 대화의 짧은 turn gap을 그대로 달성한다는 주장이 아니다. 사람 대화의 수백 ms 전환과 실제 모델·검증·speech 생성 비용 사이에 둔 제품 반응 예산이다. QA-01은 서로 분리된 위임 전·결과 후 두 VIA 구간에 각각 약 1초 budget을 두어 합계 2초를 3점으로 제안한다.

Qwen3-Omni 234ms와 VIA LLM token-rate estimate는 dependency planning input일 뿐 목표 근거 또는 실제 제품 p95가 아니다.

## 5. 동시 Task workload condition

이전 QA-04 ratio는 사용하지 않는다. 동시 Task는 다음 QA의 별도 stratum이다.

- QA-01~QA-03: foreground Voice 지연
- QA-05: Task association·routing·result binding 정확성
- QA-09: 복구해야 할 active Task 집합

동시 Task 수는 제품이 지원할 대표 workload를 승인한 뒤 고정한다. 이전의 `1 versus 4`를 자동 승계하지 않는다. 부하 조건별 raw 결과는 보존하지만 독립 QA 점수로 평균내지 않는다.

## 6. QA-05 VIA Request Handling Correctness

Downstream Agent는 고정된 정상 결과를 반환한다. Agent의 조사·계획·도구 사용·문서 품질은 채점하지 않는다.

후보가 실제로 생성한 structured trace에서 다음을 machine-readable oracle과 비교한다.

- goal, referent와 constraint
- request decomposition과 dependency
- `No Tracked / New / Existing` Task 결정과 Task ID
- direct/delegation route
- Agent와 capability
- Agent request payload의 필수 의미
- result의 Conversation·Task·run binding
- duplicate/lost dispatch 또는 duplicate response 부재

자유문장 전체를 exact string으로 채점하지 않는다. 각 case의 정답은 exact value, 허용 집합, 필수 proposition, 금지 proposition과 clarification requirement로 미리 구조화한다. 애매한 입력은 하나의 임의 답 대신 허용 가능한 해석 집합이나 확인 질문을 oracle로 둔다. 상세 계약은 [QA-05 Oracle Contract](../08-quality-attributes/evidence/qa05-request-handling-oracle.md)를 따른다.

각 case를 동일 VIA model profile로 반복하고 run-level all-or-nothing correctness를 계산한 뒤 case를 동일가중한다. wrong Task, duplicate state-changing dispatch와 unauthorized Action은 평균에 묻히지 않게 별도 critical failure로 공개한다.

## 7. QA-07·QA-08 Change Locality

Architecture Element의 normative definition과 count rule은 [Architecture Element Definition](../10-element-definition.md)을 따른다.

```text
N(change) = |modified ∪ added ∪ removed Architecture Element IDs|
QA-07 = mean N(A-01...A-09)
QA-08 = mean N(approved non-Agent change pack)
```

설정값 변경, rename, rebuild와 같은 설계 의미 불변 작업은 0이다. 실제 책임·계약·상태·배치가 바뀌면 각각 C/I/S/D 요소를 한 번 센다. 파일·class·method·LOC 또는 M/M으로 환산하지 않는다. M/M은 사람·도구·prototype 완성도에 더 크게 좌우되므로 secondary 기록만 허용한다.

평균과 함께 change별 raw count, C/I/S/D breakdown과 maximum을 공개한다. Change pack은 [QA-07/QA-08 rationale](../08-quality-attributes/evidence/qa07-qa08-change-locality-rationale.md)에 고정한다.

## 8. QA-09 Correct Task Recovery Time

```text
t0 = fault가 VIA 기능 또는 Task control을 실제로 사용할 수 없게 만든 최초 시각
t1 = 모든 영향 Task의 올바른 identity·state·result와 허용 control이 다시 사용 가능한 시각
sample = t1 - t0
```

process·worker·UI가 다시 시작된 시점은 종료점이 아니다. Task 누락, duplicate execution, stale terminal state 또는 잘못된 result binding이 있으면 recovery failure다.

후보의 장애가 worker에 머물렀는지 전체 VIA로 전파됐는지는 secondary blast-radius trace로 남긴다. 장애가 전파됐어도 t1을 빠르고 정확하게 만족하면 높은 점수를 받을 수 있다. 대표값은 각 fixed fault stratum의 p95 중 가장 느린 값이다.

## 9. QA-11 Protected Data Exposure Minimization

고정 workload는 보호정보 단위와 각 요청을 정상 완료하는 데 필요한 최소 단위를 사전 표시한다.

```text
excess_exposure
= externally accessible protected units
 - minimally necessary protected units for that recipient and purpose

QA-11 = whole-workload union of excess exposed units
```

같은 단위의 재전송은 한 번만 센다. 암호화·요약·handle이라도 recipient가 같은 사실이나 더 넓은 scope를 복원할 수 있으면 노출이다. 모든 요청을 차단해 0을 만든 후보는 기능 유지에 실패한 것이다.

실제 unauthorized disclosure는 모든 후보에서 0이어야 하는 필수 조건이다. QA-11은 올바르게 동작하는 Architecture 사이에서도 local filtering, remote model placement와 Context packaging 때문에 달라지는 불필요한 exposure를 측정한다.

## 10. 제외된 독립 점수와 필수 회귀

- Task/session continuity의 정상 연결은 QA-05 assertion에 포함한다.
- 장애 후 continuity는 QA-09 recovery correctness에 포함한다.
- dependency failure containment은 QA-09의 secondary blast-radius trace다.
- Action/access safety는 독립 QA 점수에서 제외하지만 stale/wrong-target/revoked approval 위반 0건을 모든 후보의 필수 회귀로 유지한다.

## 11. 다음 승인 전 금지사항

- 새 QA 결과 생성
- draft target을 승인된 product SLO로 표현
- archived W12-G1 결과를 현재 QA evidence로 재사용
- human free-text judge나 LLM-as-judge를 유일한 QA-05 oracle로 사용
- 후보 결과를 본 뒤 element granularity, change pack 또는 score band 변경
