# Core Decision Point Discovery

> 상태: **USER REVIEW DRAFT / 후보 도출만 수행 / 기존 ADR 변경 없음 / 구현·측정 NOT_RUN**
>
> 목적: 기존 DP 목록을 출발점으로 삼지 않고 VIA의 고정 책임, 구조적 난제와 active QA에서 전체 Core DP 후보를 도출한다. 이 문서의 A/B 방향은 trade-off hypothesis이며 승자 예측이나 ASR 판정이 아니다.

## 1. Core DP 승격 기준

설계 관심사를 Core DP 후보로 남기려면 다음 조건을 모두 만족해야 한다.

1. 책임, authority, state ownership, contract, dependency direction 또는 process/deployment boundary를 바꾼다.
2. 고정 Scope와 같은 사용자 기능을 만족하는 정상적인 A/B가 존재한다.
3. A/B에 따라 둘 이상의 Architecture Element 종류(`C/I/S/D`)가 달라진다.
4. 둘 이상의 QA에 직접적인 구조 인과가 있다.
5. A와 B 각각에 유리할 수 있는 QA pressure가 있어 한쪽이 정의상 지배하지 않는다.
6. 나중에 반대안으로 이동하면 여러 Component·Interface·State·Deployment를 함께 바꿔야 한다.
7. Model·Agent·DB 제품 선택, timeout, buffer 크기나 단순 tuning이 아니다.

여기서 “A/B에 trade-off가 있다”는 것은 결과를 미리 정한다는 뜻이 아니다. 결과가 동점이거나 예상과 반대일 수 있다. 필요한 것은 **각 대안의 장점이 생기는 구조 경로를 결과 전에 설명할 수 있는가**이다.

## 2. 백지 도출 결과

시스템 설명을 들은 Architecture 심사위원이 자연스럽게 물을 질문을 책임 축으로 분리하면 다음 12개가 Core DP 후보가 된다.

| Code | Decision Point | A | B | 바뀌는 구조 | 상태 |
| --- | --- | --- | --- | --- | --- |
| DH | VIA Direct-Handling Boundary | Rich VIA Core | Thin Interaction Core | bounded 처리 책임, dependency, 처리 경로 | Core candidate |
| LA | Lifecycle State Authority Topology | Unified Lifecycle Authority | Federated Lifecycle Authorities | Conversation/Request/Task/Pending/ExecutionLink writer | Core candidate |
| VC | Voice Processing Composition | S2S-native Voice Runtime | S2S-centered Hybrid Voice Runtime | speech dependency, streaming contract, call graph | Core candidate |
| TA | Voice–Core Turn Authority | Core-gated Turn Commit | Voice-owned Direct Turn Commit | response release authority, route lease, reconciliation | Core candidate |
| CX | Context Materialization & Release | Immutable Context Package | Scoped Context Broker | capture/materialization, release contract, provenance state | Core candidate |
| SA | Semantic Authority Topology | Integrated Semantic Authority | Staged Semantic Authorities | semantic responsibility, intermediate contracts, call graph | Core candidate |
| OG | Compound Delegation Granularity | VIA-coordinated Request Graph | Agent-owned Composite Delegation | request-edge scheduling, execution cardinality, result binding | Core candidate; common-capability fixture required |
| PR | Durable State & Recovery Record | Current State + Outbox/Inbox | Event Journal + Projections | durable truth, migration, replay, recovery | Core candidate |
| AI | Agent Integration Semantics Boundary | Edge-normalized Canonical Contract | Core-visible Typed Contracts | Agent variation 해석 책임과 계약 | Core candidate |
| MI | Model Runtime Integration Boundary | Component-owned Model Integration | Shared Model Gateway/Runtime Manager | model invocation, scheduling, placement binding, failure boundary | Core candidate |
| FI | Runtime Fault-Isolation Boundary | Single-process Partitioned Runtime | Supervised Process-isolated Runtime | OS fault boundary, IPC, supervision, deployment | Core candidate |
| EV | Observability & Evidence Ownership | Component-owned Trace + Offline Join | Shared Evidence Plane | correlation, trace schema, collector, immutable evidence | Core candidate |

12라는 숫자가 목표는 아니다. A/B를 구체화하는 과정에서 독립성이 사라지거나 한 대안이 정상 구현으로 성립하지 않으면 합치거나 제외한다.

## 3. 후보별 A/B와 QA trade-off pressure

아래의 “A pressure”와 “B pressure”는 해당 대안이 유리해질 수 있는 구조적 이유다. 실제 우열은 동결된 A/B 측정 전에는 확정하지 않는다.

### DH — VIA Direct-Handling Boundary

- **A — Rich VIA Core:** bounded read/search/understand와 해당 Direct Response capability를 VIA가 소유한다.
- **B — Thin Interaction Core:** VIA는 interaction·semantic routing·Task orchestration을 소유하고, bounded 정보 처리도 원칙적으로 Agent execution으로 위임한다. S2S Direct Response와 VIA가 이미 보유한 Task 상태 응답은 유지한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| Agent round trip이 없는 QA-02, Agent 변화가 direct path에 덜 퍼지는 QA-21, local 처리로 외부 노출을 줄일 수 있는 QA-51 | VIA-local dependency와 memory를 줄이는 QA-41, bounded 처리 구현·Model/Context 변화를 Core 밖에 둘 수 있는 QA-22 |

이 DP에서 업무를 Agent 시간으로 이동시킨 효과를 VIA-only latency 개선으로 주장하지 않는다. 전체 사용자 wall-clock과 QA-11을 함께 보존한다.

### LA — Lifecycle State Authority Topology

- **A — Unified Lifecycle Authority:** 하나의 transactional authority가 Conversation, Request, Task, Pending Interaction과 ExecutionLink 관계를 소유한다.
- **B — Federated Lifecycle Authorities:** Conversation/Request interaction owner와 Task owner가 각각 single writer이며 versioned link/event 계약으로 관계를 연결한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| 관계를 한 transaction에서 검증하는 QA-13/14/15, 전체 실행 연결을 한 authority에서 추적하는 QA-61 | 상태 종류별 변경 국소화 QA-22, owner/fault 분리 QA-32, 필요 owner만 활성화하는 QA-41 |

기존 TASK-DP01의 shared Task service와 per-Task supervisor는 이보다 좁은 Task-writer 질문이다. LA 후보를 승인할 때 TASK-DP01을 독립 DP로 유지할지 LA의 alternative refinement로 흡수할지 결정한다.

### VC — Voice Processing Composition

- **A — S2S-native Voice Runtime:** S2S가 speech understanding, generation과 direct response의 주 dependency이고 deterministic audio I/O 외 별도 speech model을 최소화한다.
- **B — S2S-centered Hybrid Voice Runtime:** S2S를 기본으로 유지하되 VAD/ASR/time-alignment/TTS 또는 helper model의 명시적 계약을 조합한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| 적은 model call과 boundary의 QA-02, 적은 local runtime의 QA-41, speech dependency 변경 요소가 적을 수 있는 QA-22 | 명시적 speech timing/control의 QA-03/04, 안정된 transcript·revision의 QA-12, 세부 event provenance의 QA-61 |

특정 helper 제품 선택은 이 DP가 아니다. A/B는 같은 Voice 기능과 같은 S2S 기본 dependency를 유지한다.

### TA — Voice–Core Turn Authority

- **A — Core-gated Turn Commit:** S2S가 stream을 만들 수 있지만 Core가 direct/escalated route와 policy를 확정한 뒤 meaningful response를 release한다.
- **B — Voice-owned Direct Turn Commit:** 허용된 direct turn은 Voice Runtime이 response를 commit하고 Core에는 중복 방지·기록·escalation 계약으로 연결한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| 동일 semantic/policy authority를 통과하는 QA-11/12/15, release 전 Context filtering의 QA-51 | Core round trip을 피하는 QA-02, Voice lane이 직접 interruption을 소유하는 QA-04, 적은 foreground state hop의 QA-41 |

VC는 어떤 speech dependency를 조합하는지, TA는 누가 turn과 response를 최종 commit하는지의 결정이므로 서로 독립이다.

### CX — Context Materialization & Release

- **A — Immutable Context Package:** Request revision마다 필요한 Context를 수집·필터링하여 불변 package와 provenance로 확정한다.
- **B — Scoped Context Broker:** Request에는 scoped handle/capability를 연결하고 소비자가 필요한 시점에 정책이 허용한 항목을 조회한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| 반복 조회가 적은 QA-01/02, 동일 evidence snapshot을 쓰는 QA-12/61/62 | 최신 원천을 읽을 수 있는 QA-12, 필요한 항목만 materialize하는 QA-41/51, Source 변화가 broker에 국소화될 수 있는 QA-22 |

Policy rule은 양쪽에서 동일하게 고정한다. enforcement 위치 자체가 Context 방식과 독립적인 A/B를 만들면 후속 DP로 분리한다.
사용자가 대상을 지정한 시점이 중요한 Interaction Context는 B에서도 해당 source version/time에 고정된 handle을 사용한다. “on demand”를 현재 화면으로 임의 치환하지 않는다.

### SA — Semantic Authority Topology

- **A — Integrated Semantic Authority:** referent, Request graph, Task relation, handling과 capability requirement를 하나의 authority가 함께 산출한다.
- **B — Staged Semantic Authorities:** grounding/refinement, Task association, handling/selection이 독립 계약과 correction boundary를 가진다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| 적은 call/boundary의 QA-01/02와 QA-41, joint evidence가 필요한 QA-12 | 단계별 변경 국소화 QA-22, stage provenance QA-61, 집중된 schema가 Model에 적합한 경우 QA-12 |

QA-12 방향은 Model-dependent empirical question이다. B가 항상 정확하고 A가 항상 빠르다고 가정하지 않는다.

### OG — Compound Delegation Granularity

- **A — VIA-coordinated Request Graph:** VIA가 사용자가 명시한 independent/sequential/data-dependent/conditional edge를 실행 readiness로 관리하고 request node별 Agent execution을 연결한다.
- **B — Agent-owned Composite Delegation:** VIA는 사용자 graph를 보존하지만 연결된 subgraph를 이를 처리할 capability가 있는 Agent에 하나의 composite execution으로 위임한다. Agent 내부 domain plan은 계속 Agent 책임이다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| node별 control/binding의 QA-05/13, 부분 상태와 edge 수렴의 QA-14/15, 실행 관계가 보이는 QA-61 | VIA handoff·상태 수를 줄일 수 있는 QA-01/41, graph orchestration 변경을 Agent 내부에 둘 수 있는 QA-22 |

B가 정상 대안이 되려면 같은 composite 기능을 실제로 지원하는 Agent fixture가 있어야 한다. 한쪽만 기능이 부족한 비교는 만들지 않는다.
B의 composite execution도 UC가 요구하는 request-node correlation, 부분 결과와 대상별 control을 제공해야 하며, VIA가 내부 domain plan을 대신 만들지는 않는다.

### PR — Durable State & Recovery Record

- **A — Current State + Transactional Outbox/Inbox:** authoritative current state와 effect identity를 transactionally 저장하고 외부 query로 조정한다.
- **B — Event Journal + Projections:** append-only lifecycle event가 durable truth이며 current view와 evaluator input을 projection으로 만든다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| 직접 current read와 적은 replay의 QA-05/31, 적은 runtime state의 QA-41, 단순 current-schema 변경이면 QA-22 | event 순서·correction을 재생하는 QA-14/15, 원시 history가 남는 QA-61/62 |

LA의 writer topology와 PR의 durable representation은 독립적으로 비교한다. 특정 DB 제품은 고정 조건이다.

### AI — Agent Integration Semantics Boundary

- **A — Edge-normalized Canonical Contract:** provider lifecycle 차이를 integration edge가 versioned canonical operation/observation으로 정규화한다.
- **B — Core-visible Typed Contracts:** provider-neutral typed variation을 Core capability handler가 해석한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| Agent 추가·교체를 edge에 국소화하는 QA-21/22, Core binding 규칙의 일관성 QA-13 | native capability 의미를 명시적으로 보존하는 QA-05/11/14, provider mode가 trace에 직접 드러나는 QA-61 |

A가 capability를 소실하지 않고 B가 native payload를 Core 전체에 누출하지 않는 잘 만든 후보를 비교한다. B의 이점이 단지 “더 많은 분기”뿐이라면 이 DP는 재승격하지 않는다.

### MI — Model Runtime Integration Boundary

- **A — Component-owned Model Integration:** Voice/semantic component가 각 역할에 필요한 Model adapter, stream과 lifecycle을 직접 소유한다.
- **B — Shared Model Gateway/Runtime Manager:** 공통 gateway가 invocation, stream normalization, scheduling, placement binding, health와 resource sharing을 소유한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| 추가 hop 없는 QA-01/02/04, 한 gateway fault가 여러 기능으로 퍼지지 않는 QA-32 | provider/deployment 변경 국소화 QA-22, model instance/resource 공유 QA-41, 공통 health/filter/trace의 QA-31/51/61 |

local/cloud 위치나 특정 Model 선택은 변화 시나리오와 배치 조건이며 이 DP의 A/B가 아니다.

### FI — Runtime Fault-Isolation Boundary

- **A — Single-process Partitioned Runtime:** Core와 외부 Agent/Context integration code를 한 OS process에 두고 queue·task·timeout으로 논리 격리한다.
- **B — Supervised Process-isolated Runtime:** 동일한 외부 Agent/Context integration code를 별도 worker process에 두고 versioned IPC와 lifecycle supervision을 사용한다. Core state와 policy authority는 Core process에 유지한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| IPC/copy가 없는 QA-01~05, process/runtime 중복이 적은 QA-41 | fatal fault 격리 QA-32, 부분 재시작 QA-31, process별 trace와 provenance를 분리할 수 있는 QA-61 |

IPC 종류나 worker 수는 DP가 아니라 후보 구현 시 고정할 조건이다.

### EV — Observability & Evidence Ownership

- **A — Component-owned Trace + Offline Join:** 각 Component가 versioned local trace를 소유하고 evaluator/exporter가 correlation identity로 사후 결합한다.
- **B — Shared Evidence Plane:** 공통 event envelope, collector/spool과 immutable evidence store가 end-to-end trace를 구성한다.

| A에 유리한 pressure | B에 유리한 pressure |
| --- | --- |
| collector dependency와 공통 buffering이 적은 QA-41, evidence plane fault의 blast radius를 줄일 수 있는 QA-32, 중앙 전송을 줄일 수 있는 QA-51 | instrumentation 변경을 공통 plane에 국소화하는 QA-23, 실행 trace 완전성 QA-61, evidence 재현성 QA-62 |

양 후보 모두 모든 QA가 요구하는 최소 trace를 제공해야 한다. 로그가 부족한 A를 만들어 B가 이기게 해서는 안 된다.

## 4. Structural strength check

| DP | 변경 element 종류 | 여러 QA 직접 인과 | A/B opposing pressure | 변경 비용 | Gate |
| --- | --- | --- | --- | --- | --- |
| DH | C/I/D | QA-02/11/21/22/41/51 | 있음 | Core capability와 dependency 재배치 | PASS |
| LA | C/I/S | QA-05/13/14/15/22/31/32/41/61 | 있음 | lifecycle writer와 관계 contract 이행 | PASS |
| VC | C/I/D | QA-02/03/04/12/22/41/61 | 있음 | audio/model call graph와 runtime 교체 | PASS |
| TA | C/I/S | QA-02/04/11/12/15/41/51/61 | 있음 | response commit protocol과 route state 이행 | PASS |
| CX | C/I/S/D | QA-01/02/12/22/41/51/61/62 | 있음 | Context contract·provenance·consumer 변경 | PASS |
| SA | C/I/S | QA-01/02/11/12/22/41/61 | 있음 | semantic contract와 prompt/call graph 이행 | PASS |
| OG | C/I/S | QA-01/05/13/14/15/22/41/61 | 있음 | Task/execution cardinality와 edge state 이행 | PASS* |
| PR | C/I/S/D | QA-05/14/15/22/31/41/61/62 | 있음 | durable data와 recovery migration | PASS |
| AI | C/I/S | QA-05/11/13/14/21/22/61 | 있음 | Agent contract와 소비자 책임 이동 | PASS* |
| MI | C/I/S/D | QA-01/02/04/22/31/32/41/51/61 | 있음 | 모든 model consumer와 deployment binding 변경 | PASS |
| FI | C/I/S/D | QA-01~05/31/32/41/61 | 있음 | IPC, supervision, packaging과 recovery 변경 | PASS |
| EV | C/I/S/D | QA-23/32/41/51/61/62 | 있음 | 모든 producer와 evidence pipeline schema 변경 | PASS |

`PASS*`는 정상적인 B 이점을 candidate contract에서 증명해야 한다는 조건이다. OG는 같은 composite capability, AI는 capability 비손실 조건을 먼저 고정한다.

## 5. 구조적 난제 coverage sweep

System Understanding Review의 10개 구조적 난제가 후보 집합에서 빠지지 않았는지 확인한다.

| 구조적 난제 | 주로 다루는 DP |
| --- | --- |
| 음성·전사·pointer·화면의 서로 다른 시간축 결합 | VC, CX, SA |
| 빠른 Voice path와 하나의 사용자 의미 양립 | VC, TA, SA, LA |
| VIA Task와 Agent Execution identity 분리 | LA, OG, AI, PR |
| 비동기 상태의 권위와 진실성 | LA, OG, PR, AI |
| 재시작 복구와 중복 Action 방지 | PR, FI, AI |
| 충분한 Context와 최소 노출 | DH, CX, MI |
| Compound Request 관계와 Agent planning 경계 | DH, SA, OG |
| 실시간 Voice와 비동기 결과 중재 | TA, LA, FI |
| Model·Agent·Context·evidence 생태계 변화 격리 | CX, AI, MI, EV |
| 측정 가능한 Architecture 인과관계 | EV, PR, LA |

현재 후보 단계에서 coverage 공백은 없다. 그러나 하나의 난제에 여러 DP가 연결된다는 이유로 모두 Primary QA를 받는 것은 아니다.

## 6. QA coverage sweep

표의 DP code는 해당 QA가 Primary가 될 가능성이 있는 hypothesis다. 실제 `Primary / Regression / N/A`는 A/B element와 physical path를 확정한 뒤 결과 전에 결정한다.

| QA | DP hypothesis |
| --- | --- |
| QA-01 | DH, CX, SA, OG, AI, MI, FI |
| QA-02 | DH, VC, TA, CX, SA, MI, FI |
| QA-03 | LA, VC, AI, MI, FI |
| QA-04 | VC, TA, MI, FI |
| QA-05 | LA, OG, PR, AI, FI |
| QA-11 | DH, TA, CX, SA, OG, AI |
| QA-12 | VC, TA, CX, SA, MI |
| QA-13 | LA, OG, PR, AI |
| QA-14 | LA, OG, PR, AI |
| QA-15 | LA, TA, CX, OG, PR |
| QA-21 | DH, OG, AI |
| QA-22 | DH, LA, VC, TA, CX, SA, OG, PR, AI, MI |
| QA-23 | EV |
| QA-31 | LA, PR, AI, MI, FI |
| QA-32 | LA, MI, FI, EV |
| QA-41 | DH, LA, VC, TA, CX, SA, OG, PR, MI, FI, EV |
| QA-51 | DH, TA, CX, MI, EV |
| QA-61 | LA, VC, TA, CX, SA, OG, PR, AI, MI, FI, EV |
| QA-62 | CX, PR, EV |

이 표는 QA마다 DP를 억지로 붙이는 용도가 아니다. 한 DP의 A/B가 해당 metric의 physical path나 element를 실제로 바꾸지 않으면 그 QA는 regression 또는 `N/A`로 내려간다.

## 7. Independence rules and required cross-checks

| DP | 비교할 때 반드시 고정할 다른 축 | 강한 interaction이 있어 제한 교차 확인할 축 |
| --- | --- | --- |
| DH | 같은 user goal, source와 Agent capability | CX, SA, OG |
| LA | 같은 durable representation과 process boundary | PR, OG, FI |
| VC | 같은 turn authority와 S2S profile | TA, MI |
| TA | 같은 Voice composition과 semantic topology | VC, SA |
| CX | 같은 source, policy rule과 semantic responsibility | DH, SA, MI |
| SA | 같은 Model profile, Context contract와 handling scope | TA, CX, MI |
| OG | 같은 Request graph, Agent capability와 Task semantics | LA, AI |
| PR | 같은 lifecycle writer topology와 DB conditions | LA, FI, EV |
| AI | 같은 Agent fixture, Task owner와 process placement | OG, FI |
| MI | 같은 semantic/Voice responsibility와 Model profile | VC, SA, FI |
| FI | 같은 code, worker budget, persistence와 logical fault | LA, PR, AI, MI, EV |
| EV | 같은 required event semantics와 privacy rule | PR, FI |

강한 interaction은 두 DP를 하나로 합쳐야 한다는 뜻이 아니다. 한 축만 바꿀 수 있지만 선택의 방향이 다른 축에서 뒤집힐 가능성이 있을 때만 작은 교차 확인을 한다. 전체 조합을 primary winner selection으로 사용하지 않는다.

## 8. Core DP에서 제외하거나 흡수한 항목

| 항목 | 처리 | 이유 |
| --- | --- | --- |
| 특정 Model/Agent/DB/IPC 제품 | 제외 | dependency 또는 구현 선택이며 구조 축이 아님 |
| polling vs event | 공통 tactic | low-latency event와 gap/reconnect query는 상호 보완적 |
| timeout/retry/buffer/worker 수 | tuning/freeze 조건 | 책임·상태 소유권을 바꾸지 않음 |
| local vs cloud Model 위치 자체 | MI의 deployment scenario | Architecture는 위치 변화를 흡수해야 하며 한 provider 위치를 영구 정답으로 두지 않음 |
| Policy enforcement boundary | CX에 우선 흡수 | Context release와 분리된 정상 A/B가 확인되면 별도 승격 |
| response/audio scheduling | TA/VC tactic | 별도 authority·state owner가 필요한지 후보 상세화에서 재검토 |
| Agent state event vs query | AI/LA 공통 tactic | delivery mechanism 선택만으로는 강한 상호배타 DP가 아님 |

## 9. 기존 DP와의 관계

기존 DP는 이 도출의 출발점이 아니며 다음처럼 사후 대조한다.

| 기존 항목 | 백지 도출 후보와의 관계 | 후속 처리 제안 |
| --- | --- | --- |
| IR-DP01 | SA와 거의 동일 | SA 후보 상세화 때 기존 정의를 참고하되 새 QA로 trade-off 재검토 |
| TASK-DP01 | LA의 Task-writer 부분집합 | LA 범위를 먼저 확정하고 독립 DP 유지 또는 alternative refinement 여부 결정 |
| AGENT-DP01 | AI와 거의 동일 | capability 비손실 상태에서 B의 독립 장점을 명확히 한 뒤 유지 |
| EXEC-DP01 | FI의 integration-worker 구체화 | fault boundary 범위가 충분한지 확인하고 기존 A/B 참고 |
| FP-INT01 S2S Direct Fast Path | TA의 B를 고정한 원칙 | TA의 정상 A/B를 비교하기 전에는 고정 원칙으로 전제하지 않음 |
| TASK-T01 Event-first + Query | 공통 tactic과 일치 | DP로 승격하지 않음 |
| CTX supporting choices | CX의 부분 후보 | active candidate contract로 다시 정의하기 전에는 결정으로 간주하지 않음 |

Accepted ADR은 이 review draft만으로 변경되지 않는다. Core DP 집합과 candidate contract가 승인된 뒤에만 기존 ADR의 유지·재검증·대체 필요성을 별도로 판단한다.

## 10. 권장 논의 순서

다음 순서는 숫자를 맞추기 위한 것이 아니라 앞 결정이 뒤 결정의 후보 범위를 정의하는 정도를 따른다.

1. **DH** — VIA가 직접 소유할 processing capability의 범위
2. **LA** — 전체 lifecycle truth와 writer topology
3. **VC** — Voice processing dependency composition
4. **TA** — Voice와 Core 사이의 turn/response commit authority
5. **CX** — Context의 materialization, release와 provenance
6. **SA** — semantic authority의 통합/단계 topology
7. **OG** — compound Request와 Agent execution의 coordination authority
8. **PR** — durable truth와 recovery representation
9. **AI** — Agent variation을 해석하는 boundary
10. **MI** — Model Runtime integration과 resource/failure ownership
11. **FI** — OS process fault boundary
12. **EV** — trace/evidence ownership과 dependency direction

EV의 상세 대안은 뒤에서 논의하지만, 어떤 후보도 측정 불가능해지지 않도록 최소 event/correlation 계약은 모든 candidate definition 작성 전에 공통으로 둔다.

## 11. 다음 승인 단위

이 문서를 승인한다고 A/B 구현이나 측정을 시작하지 않는다. 다음에는 DP별로 아래 한 장짜리 candidate card를 순서대로 검토한다.

1. 한 문장 Architecture question
2. 고정 invariant와 명시적 scope exclusion
3. 구현 가능한 A/B 책임·계약·상태·배치
4. A와 B 각각의 장점이 생기는 causal path
5. Primary QA hypothesis와 Regression/N/A 후보
6. 다른 DP와의 독립성 및 필요한 제한 교차 확인
7. 한쪽이 dominated이면 DP를 폐기하는 반증 조건

각 card가 승인된 뒤에만 전체 Core DP inventory를 확정한다.
