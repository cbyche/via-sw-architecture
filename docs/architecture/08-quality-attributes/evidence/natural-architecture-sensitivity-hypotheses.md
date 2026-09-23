# Natural Architecture Sensitivity Hypotheses

> 작성일: 2026-09-23
> 상태: **USER REVIEW DRAFT** — 최종 DP, 결과 또는 ASR 판정이 아니다.

## 1. 판정 원칙

제품적으로 중요한 관심사, 특정 DP의 A/B를 가르는 QA, 시스템 수준 ASR은 서로 다르다. 정상 기능을 충족한 합리적 대안 사이에서 같은 현실적 조건 아래 metric이 달라질 구조 인과가 있어야 해당 DP의 Primary QA가 된다.

현재 active catalog는 QA-01~05, QA-11~15, QA-21~23, QA-31/32, QA-41, QA-51, QA-61/62다. 모든 QA는 하나의 대표 metric을 가진다. QA-11은 integrated outcome이고 QA-12~15는 non-additive driver다.

## 2. 예상되는 구조 민감도

| QA | 값이 달라질 수 있는 구조 원인 |
| --- | --- |
| QA-01~03 | call graph, process/IPC boundary, state lookup, validation, buffering, playback path |
| QA-04 | input arbitration authority, cancellation propagation, playback queue/renderer ownership |
| QA-05 | control interpretation, Task binding, durable command commit, source confirmation과 response composition path |
| QA-11 | 전체 semantic→binding→execution→result 연결이 보존되는 정도 |
| QA-12 | semantic authority topology, Context materialization, model input/output contract와 validation 위치 |
| QA-13 | Conversation/Interaction/Task/external-run identity owner와 binding contract |
| QA-14 | canonical state authority, event revision/idempotency, reconciliation과 stale-event policy |
| QA-15 | channel/session lifetime과 Conversation/Task lifetime의 분리, durable relationship ownership |
| QA-21 | Agent-specific 차이를 adapter/contract boundary에 국소화하는 정도 |
| QA-22 | Model·Context·state·deployment 변화의 dependency direction과 ripple effect |
| QA-23 | instrumentation contract, correlation ownership, trace schema와 evidence exporter dependency direction |
| QA-31 | persistence, reconciliation, restart boundary와 recovery call graph |
| QA-32 | process/fault boundary, bulkhead, queue와 shared-resource topology |
| QA-41 | process topology, duplicated runtime state, queue/buffer ownership, local-model placement |
| QA-51 | local/remote placement, Context packaging과 recipient별 filtering boundary |
| QA-61 | event envelope, identity propagation, timestamp provenance, collector/spool과 process boundary |
| QA-62 | immutable raw evidence, freeze manifest, analyzer version과 result-generation dependency direction |

동시성은 독립 QA가 아니라 applicable QA의 workload stratum이다. Action/access safety는 모든 후보의 0-violation 필수 회귀다. Observability는 각 metric을 믿을 수 있게 하는 evidence condition이며, 별도 metric이 승인되기 전에는 점수 축으로 만들지 않는다.

## 3. Correctness와 continuity를 해석하는 법

QA-11은 사용자의 전체 요청 처리가 올바르게 끝났는지를 보여준다. QA-12~15는 각각 의미, binding, 비동기 상태 수렴, 정상 continuity를 분리한다. 한 실행이 여러 predicate의 evidence를 제공할 수 있지만 다섯 점수를 합산해 같은 성공을 다섯 번 보상하지 않는다.

예를 들어 integrated semantic path와 staged path는 information loss, joint schema 복잡도, validation 위치가 달라 QA-12를 바꿀 수 있다. Task owner와 external run 관계가 달라지면 QA-13/14가, Voice 재연결과 Text 전환에서 수명을 어디에 귀속하는지가 달라지면 QA-15가 갈릴 수 있다. 이러한 구조 인과가 없고 단지 다른 모델이나 시험기 정답 주입 때문에 생긴 차이는 Architecture 증거가 아니다.

QA-14는 정상 비동기 상호작용이 정답 상태로 수렴하는지 본다. 장애 주입 뒤 올바른 상태로 돌아오는 데 걸린 시간은 QA-31이다. QA-15는 정상적인 채널·세션·직접/위임 전환에서 관계가 유지되는지 보며, 장애 복구 시간을 대신하지 않는다.

## 4. Reliability, resource, privacy의 분리

QA-31은 **모든 영향 Task의 올바른 identity·state·result·control이 돌아온 시각**까지의 시간이다. QA-32는 같은 fault에서 허용 범위를 넘어 영향을 받은 user-visible unit 수다. 격리 구조도 느리게 복구할 수 있고, 넓게 재시작한 구조도 빠르게 복구할 수 있으므로 두 metric을 합치지 않는다.

QA-41은 고정한 target-device workload stratum별 peak committed memory의 worst p95 하나만 대표값으로 사용한다. CPU, disk, network, energy는 raw diagnostic으로 보존할 수 있지만 이번 QA-41 점수에 섞지 않는다.

QA-51은 unauthorized disclosure 허용량이 아니다. 그 값은 항상 0이어야 한다. 정상 기능을 유지하면서 recipient와 목적에 필요한 최소 단위를 넘겨 외부에 접근 가능하게 만든 보호정보 단위 수를 센다. 모든 요청을 차단해 0을 만드는 후보는 기능 유지에 실패한다.

## 5. 사용 규칙

1. 합리적인 후보와 공통 tactic을 먼저 정의한다.
2. QA별 구조 인과와 applicable DP를 결과 전에 기록한다.
3. 실제 인과가 있는 QA만 그 DP의 Primary driver로 둔다.
4. QA-11과 QA-12~15를 weighted total에 중복 가중하지 않는다.
5. 동점이나 non-discriminating QA를 숨기지 않는다.
6. ASR 여부는 점수 차이만으로 정하지 않고 제품 중요성·구조 위험·evidence를 별도 검토한다.
