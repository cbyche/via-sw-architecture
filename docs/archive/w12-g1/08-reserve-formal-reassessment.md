# 08-B. Reserve QA Formal Reassessment

> 작성일: 2026-09-21
> 상태: **정식 재평가 결과 — 사용자 승인 전. 기존 ASR catalog는 아직 변경하지 않음.**
> 목적: 기존 ASR-02/03/06/07의 DP 변별력 우려를 보완할 Reserve QA를 08의 동일 H/H 방법으로 재평가한다.

## 1. 동일 선정 규칙

- Importance 1~10, Architecture Difficulty 1~10
- 1~4=L, 5~7=M, 8~10=H
- **Importance=H && Difficulty=H만 승격 후보**
- 구조를 가르기 쉽다는 이유만으로 Importance를 올리지 않음
- 기존 ASR과 실질적으로 같은 response measure이면 신규 ASR로 중복 추가하지 않음

ISO/IEC 25010:2023은 product quality model을 요구 정의·설계 목표·시험 및 acceptance criteria에 사용할 수 있는 reference model로 제공한다. SEI의 quality-attribute/tactic 관점은 architecture decision이 measurable QA response에 영향을 주어야 함을 강조한다.

외부 근거:
- https://www.iso.org/standard/78176.html
- https://www.sei.cmu.edu/library/illuminating-the-fundamental-contributors-to-software-architecture-quality/
- https://www.sei.cmu.edu/library/realizing-and-refining-architectural-tactics-availability/

## 2. Formal result

| 후보 | Importance | Difficulty | 판정 | 권고 |
| --- | ---: | ---: | --- | --- |
| **RQA-01 Concurrent Task Scalability & Performance Isolation** | **8 H** | **9 H** | **H/H** | 신규 ASR 승격 권고 |
| **RQA-02 Recovery Timeliness / Availability** | **9 H** | **9 H** | **H/H** | 신규 ASR보다 **ASR-06 대표 Metric 교체 권고** |
| **RQA-03 Privacy Exposure Minimization** | **9 H** | **8 H** | **H/H** | QA-09 분리 후 신규 ASR 승격 권고 |
| **RQA-04 Dependency Failure Containment & Graceful Degradation** | **9 H** | **9 H** | **H/H** | 신규 ASR 승격 권고 |
| QA-07 Compute & Resource Efficiency 재검토 | 4 L | 8 H | L/H | Reserve 유지 |
| QA-10 Diagnosability & Operability 재검토 | 6 M | 8 H | M/H | Reserve 유지 |
| Task State Freshness | 8 H | 8 H | H/H이나 중복 | 신규 ASR 대신 ASR-01 secondary metric 권고 |
| Deployment / Model Placement Flexibility | 7 M | 9 H | M/H + ASR-05 중복 | 승격하지 않음 |

## 3. RQA-01 Concurrent Task Scalability & Performance Isolation

### Importance = 8

UC-13/14와 FA-08에 multiple active Task가 이미 명시되어 있다. VIA는 background Agent event, foreground interaction, multiple pending interaction을 동시에 관리하는 제품이므로 한 Task의 workload가 foreground interaction을 크게 훼손하면 핵심 제품 경험이 무너진다.

### Difficulty = 9

Task supervision, queueing, shared Model/Context resources, state locking, event fan-in, scheduling/backpressure가 여러 runtime/component 경계를 가로지른다.

### Representative Metric

**Foreground Latency Degradation Ratio under 4 Active Tasks (FLDR)**

    FLDR = Macro-p95(4 active Tasks) / Macro-p95(1 active Task)

### Natural architecture sensitivity

centralized queue/supervisor, task-local supervision, shared resource arbitration, synchronization 방식이 달라도 모든 기능 요구를 충족할 수 있지만 foreground interference는 자연스럽게 달라질 수 있다.

SEI performance tactics는 scheduling, synchronization, concurrency, resource management, computational overhead가 response time에 영향을 주는 architectural tactic임을 설명한다.

## 4. RQA-02 Recovery Timeliness — ASR-06 metric replacement

### Importance = 9 / Difficulty = 9

UC-18과 FA-14가 restart/reconnection을 필수로 요구한다. 단순 '결국 복구함'보다 사용자가 언제 다시 올바른 Task 상태와 control을 사용할 수 있느냐가 실제 availability response다.

### 권고

현재 ASR-06 representative Metric:

    Reliability/recovery scenario pass rate

를 다음으로 교체한다.

**p95 Time to Correct User-Visible Recovery (TCR)**

    fault/restart occurrence
    → authoritative Task state가 복원/재확인되고
      사용자가 correct status/result/control을 다시 이용 가능한 시점

27개 strict invariant PASS/FAIL은 **회귀 constraint**로 유지한다.

### Natural architecture sensitivity

event-log replay, snapshot+reconciliation, Agent-query reconstruction, eager/lazy recovery, persistence boundary가 모두 기능을 정확히 구현하더라도 복구 시간은 자연스럽게 달라진다.

SEI availability tactics는 fault detection/recovery/reintroduction을 architecture tactic으로 다룬다.

## 5. RQA-03 Privacy Exposure Minimization

### Importance = 9

VIA는 personal context, memory, local file/mail/calendar, remote model, downstream Agent를 연결한다. 모든 access가 authorized여도 필요 이상의 private context를 remote dependency에 제공하는 Architecture는 제품 privacy risk가 더 크다.

NIST privacy engineering은 granular administration/selective disclosure(Manageability)와 operational requirement를 넘어 association을 최소화하는 Disassociability를 별도 privacy objective로 둔다.

### Difficulty = 8

local/remote model placement, context materialization, redaction/minimization, egress package, handle scope가 여러 component/deployment boundary에 걸친다.

### Representative Metric

**Remote Sensitive-Context Exposure Ratio (%)**

canonical workload의 pre-labeled sensitive context units 중 remote Model/Agent가 실제 접근 가능하게 된 unit의 비율.

- full payload 제공 → 포함 unit 전체 노출
- field/paragraph selective provision → 해당 unit만 노출
- dereference 가능한 handle → handle scope가 열어주는 unit 전체 노출
- local-only processing → remote exposure 0

lower is better.

### 권고

현재 QA-09 `Privacy, Security & Action Safety`에서 **Privacy Exposure Minimization을 분리**한다. 기존 ASR-07 violation metric은 Action/Access Safety constraint로 유지한다.

## 6. RQA-04 Dependency Failure Containment & Graceful Degradation

### Importance = 9

UC-18에는 Source 없음, Model unavailable, Agent unavailable, partial/unknown state가 이미 포함된다. VIA가 heterogeneous dependencies를 orchestration하는 제품이므로 한 dependency failure가 무관한 Direct Response/다른 Task까지 중단시키는 blast radius는 핵심 사용자 가치에 직접 영향을 준다.

### Difficulty = 9

dependency graph, circuit breaking, bulkhead/isolation, fallback/degraded mode, task-local vs shared supervision, cache/state ownership, routing이 여러 Architecture boundary에 걸친다.

### Representative Metric

**Essential Capability Retention Rate under Single-Dependency Failure (%)**

fixed failure set에서 실패한 dependency를 본질적으로 필요로 하지 않는 canonical capability 중 정상 또는 명시된 degraded mode로 계속 제공되는 비율.

예를 들어 Agent A outage 시:
- Agent A 자체 업무는 denominator에서 제외
- S2S Direct Response
- VIA Core bounded read가 다른 source를 사용
- Agent B의 unrelated Task status/follow-up
- Conversation/Text interaction

등의 unaffected capability가 계속 동작하는지를 본다.

### Natural architecture sensitivity

- shared central runtime/queue vs failure-isolated boundaries
- per-Agent adapter/process isolation
- hard dependency vs soft dependency/fallback
- shared state invalidation vs dependency-local state
- circuit breaker/backpressure scope

정상 후보 모두 실패한 dependency의 기능을 수행하지 못할 수 있지만, **실패의 blast radius**는 Architecture에 따라 자연스럽게 달라질 수 있다.

AWS Well-Architected는 graceful degradation을 dependency failure가 전체 system function을 최소한만 방해하도록 만드는 reliability practice로 설명한다. Microsoft Well-Architected의 Bulkhead pattern도 malfunction blast radius를 격리하는 architecture pattern으로 설명한다.

근거:
- https://docs.aws.amazon.com/wellarchitected/latest/reliability-pillar/rel_mitigate_interaction_failure_graceful_degradation.html
- https://learn.microsoft.com/en-us/azure/well-architected/reliability/design-patterns

## 7. 검토했지만 신규 ASR로 권고하지 않는 후보

### Task State Freshness

Agent state가 바뀐 뒤 user-visible state에 반영되기까지의 p95 staleness는 구조에 민감하다. 그러나 ASR-01의 Existing Task Interaction / Agent Result Delivery latency와 response 의미가 크게 겹친다. 새 ASR보다 ASR-01 secondary diagnostic으로 두는 것이 낫다.

### AI Runtime / Compute Efficiency

Model call 수, total Model busy time, token demand는 구조에 매우 민감하다. 그러나 현재 제품에 CPU/GPU/power/cloud-cost budget이 H 수준 요구로 고정되지 않았다. QA-07 Importance를 결과 변별성을 이유로 4→8 이상으로 올리는 것은 금지한다.

### Diagnosability / Fault Localization

multi-Agent/multi-Model architecture에 중요하지만 현재 OPEX/MTTR이 제품 핵심 success criterion으로 정의되지 않아 Importance는 M이다. QA-10 Reserve 유지.

### Deployment / Model Placement Flexibility

local↔remote/private-cloud 이동은 중요하지만 이미 ASR-05의 M-04~06 change ripple에서 직접 측정한다. 별도 QA로 추가하면 중복 가중 위험이 크다.

## 8. 권고되는 Working ASR 변화안

사용자 승인 전에는 적용하지 않는다.

| 현재 | 권고 |
| --- | --- |
| ASR-01 Responsiveness | 유지 |
| ASR-02 Completion | 유지 — DP sensitivity는 조건부 |
| ASR-03 Continuity | 유지 — DP sensitivity는 조건부 |
| ASR-04 Agent Interoperability | 유지 |
| ASR-05 Evolvability | 유지 |
| ASR-06 Reliability/Recoverability pass rate | **QA 유지, 대표 Metric을 Recovery Timeliness로 교체** |
| ASR-07 Privacy/Security/Action Safety | **Action/Access Safety constraint로 범위 축소** |
| 없음 | **Concurrent Task Scalability & Performance Isolation 추가 후보** |
| 없음 | **Privacy Exposure Minimization 추가 후보** |
| 없음 | **Dependency Failure Containment & Graceful Degradation 추가 후보** |

승인한다면 다음 작업은 08 QA catalog의 명시적 rebaseline → 09 mapping → 11-A/B/C metric/test/target 업데이트 순서다. 12 DP 설계는 그 이후에 시작한다.
