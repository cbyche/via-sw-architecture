# 12-04A. Gate 2 Implementation Mapping & Candidate Fairness

> 상태: **PRE-MEASUREMENT REVIEW INPUT** / comparative benchmark, score, winner는 아직 NOT_RUN.
> 이 문서는 Gate 2의 4개 Core DP가 실제 executable comparison slice에 어떻게 대응되는지와 A/B 공정성 조건을 추적한다.

## 1. 이 구현이 의미하는 것

현재 prototype/gate2는 **완성형 VIA 제품 구현이 아니라 Architecture trade-off를 검증하기 위한 최소 executable slice**다. scored DP에 필요한 책임, 상태, integration/fault boundary는 실제 코드로 구현하지만 Voice UI, 실제 OS Context capture, production Policy/Memory, 전체 Presentation shell은 fixture/harness로 대체하거나 아직 구현하지 않는다.

상태 표기는 다음 의미다.

- **IMPLEMENTED:** 후보 차이를 실제 실행 코드가 만든다.
- **HARNESS:** 실제 모델, TC, 계측을 연결하는 평가 코드이며 product runtime이라고 주장하지 않는다.
- **FIXTURE:** 후보 간 동일한 외부 dependency behavior를 제공한다.
- **PARTIAL/BLOCKED:** full 12-A 대표값을 만들기에는 추가 driver/환경이 필요하다.

## 2. IR-DP01 — Semantic Decision Ownership

| Gate 2 Element | 실제 구현 | 상태 | 설명 |
|---|---|---|---|
| G2-C-JOINT | benchmark/rebaseline/readiness/ir_contract.py prepare(integrated), gate2/ir_model_runner.py run_integrated | HARNESS_READY | 한 structured semantic decision을 동일 Model에 요청 |
| G2-C-GROUNDREFINE | prepare(grounding), run_staged Stage 1 | HARNESS_READY | referent/request/constraint/관계만 판단 |
| G2-C-ASSOCIATE | prepare(association), run_staged Stage 2 | HARNESS_READY | Task/Pending binding만 판단 |
| G2-C-SELECT | prepare(handling), run_staged Stage 3 | HARNESS_READY | handling/capability 판단 |
| G2-C-SEMCHECK | validate_result, call_stage, correction/context controller | HARNESS_READY | schema/provenance/Task revision/capability 검증 |
| G2-I-GROUNDED | grounding JSON schema + validator | HARNESS_READY | Stage 1 output contract |
| G2-I-ASSOCIATED | association JSON schema + validator | HARNESS_READY | Stage 2 output contract |
| G2-S-SEMSTAGES | runner의 prior-stage/correction state | HARNESS_READY | Request 수명 내 상태. crash 시 unfinished semantic decision은 재수행 |

### IR fairness

**같게 고정:** Qwen3-8B Q4_K_M artifact, llama.cpp build, tokenizer/template, non-thinking, sampling contract, 같은 run seed, canonical TC, ContextProvider source corpus, scripted clarification, validator repair limit 2, Context acquisition limit 2.

**다른 것:** A는 joint decision 1개 topology, B는 Grounding→Association→Handling의 독립 contract topology다. B의 Stage 2/3에 raw source 전체를 반복 전달하지 않고 이전 stage의 provenance-preserving output과 해당 stage가 소유한 evidence만 전달한다.

B의 cross-stage correction과 추가 model call은 B의 구조 비용이다. 반대로 A의 큰 joint prompt/schema와 joint repair도 실제 비용으로 남긴다.

**아직 없음:** actual Qwen execution, exact token ledger, W-05 5-run/case 결과. 실제 모델 실행은 Measurement Freeze 승인 fingerprint가 없으면 runner가 거부한다.

## 3. TASK-DP01 — Task State Authority & Supervision

| Gate 2 Element | 실제 구현 | 상태 | 설명 |
|---|---|---|---|
| G2-C-TASKSERVICE | runtime/task.rs SharedTaskService | IMPLEMENTED | stateless handler가 Repository conditional transition 수행 |
| G2-I-TXNTRANSITION | Repository apply/apply_with_epoch | IMPLEMENTED | expected revision, command payload dedup, atomic commit |
| G2-C-TASKACTOR | runtime/task.rs PerTaskSupervisors | IMPLEMENTED | Task별 mailbox + single-writer processing |
| G2-C-ACTIVATION | sender_for/create_sender, Repository activate | IMPLEMENTED | activation discovery/epoch fencing |
| G2-I-MAILBOX | Tokio mpsc + oneshot command/commit completion | IMPLEMENTED | channel delivery를 durable completion으로 오인하지 않음 |
| G2-S-ACTIVATION | SQLite task_owners | IMPLEMENTED | Task owner epoch 영속 |
| 공통 G2-S-TASK | SQLite tasks | IMPLEMENTED | A/B 동일 schema |
| 공통 G2-S-LINK | SQLite execution_links | IMPLEMENTED | Task↔Agent run/context/submission 재연결 |
| 공통 G2-S-DELIVERY | handoff_outbox, commands, command_payloads | IMPLEMENTED | 외부 접수 전후 crash 조정 |

runtime/handoff.rs HandoffCoordinator는 두 후보 공통으로 **PrepareHandoff(outbox) → Agent acceptance → ConfirmHandoff(ExecutionLink commit)**을 수행한다. Agent acceptance 뒤 link commit 전에 VIA memory loss가 발생한 경우 동일 submission key로 pending outbox를 재조정하는 smoke도 있다.

### TASK fairness

**같게 고정:** SQLite WAL/FULL, 같은 DB schema/file, same deterministic Agent, same HandoffCoordinator/outbox/link contract, same Event-first+Query reconciliation tactic, same process placement during one-factor comparison.

**다른 것:** Task transition writer/scheduling authority만 A=SharedTaskService, B=PerTaskSupervisor+activation epoch이다.

A에 global mutex/전역 serial queue를 강제하지 않는다. B에 별도 DB/shard를 주지 않는다. scored steady-state trial 전에 B activation을 완료해 one-time startup을 foreground steady-state latency로 섞지 않고, recovery trial에서는 activation 비용을 포함한다.

## 4. AGENT-DP01 — Agent Integration Contract Boundary

| Gate 2 Element | 실제 구현 | 상태 | 설명 |
|---|---|---|---|
| G2-C-EDGESEM | runtime/agent.rs EdgeNormalized + normalize functions | IMPLEMENTED | native lifecycle variation을 Edge에서 canonicalize |
| G2-I-CANONAGENT | AgentBoundary, CanonicalAccepted/Snapshot, AgentObservation, CancelOutcome | IMPLEMENTED | Core 소비 공통 의미 |
| G2-C-TYPEDHANDLER | CoreVisibleTyped + Typed lifecycle enums | IMPLEMENTED | typed variation을 Core handler가 해석 |
| G2-I-TYPEDAGENT | typed lifecycle enums + Native P/Q contract | IMPLEMENTED | variation을 Core까지 보존 |
| 공통 native fixture | fixture DeterministicAgent(P/Q) | FIXTURE_FULL_LIFECYCLE | 동일 기능, 다른 native lifecycle shape |
| 공통 state update | runtime/sync.rs EventFirstQuerySync | IMPLEMENTED_TACTIC | TASK-T01 적용 |

P/Q는 동일하게 submit/query/event/follow-up/cancel을 지원하지만 native 표현은 다르다.

- **P:** single run identity, same-run follow-up, immediate cancel confirmation, P-style event.
- **Q:** context+run identity, continuation creates a new run, cancel request와 실제 confirmation 분리, Q-style event.

이 차이는 업무 능력 차이가 아니라 **integration lifecycle 표현 차이**다.

### AGENT fairness

**같게 고정:** capability set, underlying deterministic workload/result, event semantic content/revision, Task writer, sync policy, EXEC placement during one-factor comparison.

**다른 것:** native variation을 A가 edge에서 canonicalize할지, B가 typed contract로 Core까지 보존한 뒤 handler가 해석할지만 다르다.

Unsupported native 기능을 한 후보에만 emulation으로 공짜 제공하지 않는다. P/Q full-capability fixture는 두 후보 모두 동일하게 사용한다.

## 5. EXEC-DP01 — Runtime Fault-Isolation Boundary

| Gate 2 Element | 실제 구현 | 상태 | 설명 |
|---|---|---|---|
| G2-C-LOCALBRIDGE | runtime/exec.rs LocalBridge | IMPLEMENTED | Agent integration code in-process |
| G2-C-REMOTEBRIDGE | ProcessBridge | IMPLEMENTED | child worker와 IPC |
| G2-C-WORKERLIFE | gate2-host isolated mode + worker respawn | IMPLEMENTED | worker lifecycle/restart |
| G2-I-BRIDGE | IntegrationBridge | IMPLEMENTED | submit/query/follow-up/cancel/events 공통 logical operation |
| G2-I-IPC | WorkerRequest/WorkerResponse newline JSON over OS pipe | IMPLEMENTED_PROTOTYPE | macOS/Linux portable prototype IPC |
| G2-S-IPC | ProcessSession in-flight connection state | PARTIAL | volatile connection state. production operation generation/backpressure는 full runner 전 보강 필요 |
| G2-D-VIA | gate2-host shared/isolated | IMPLEMENTED_FIXTURE_HOST | 비교용 Core host |
| G2-D-INTEGRATION | gate2-worker | IMPLEMENTED | B의 separate process fault domain |

동일 AbortHost command를 외부 controller가 주입한다. Shared mode에서는 integration과 Core host가 함께 종료되고, isolated mode에서는 worker만 종료·재시작되어 Core host가 살아 있음을 smoke로 검증한다.

### EXEC fairness

**같게 고정:** deterministic Agent code, logical bridge operation, Task/DB semantics, fault command, candidate input, worker/resource budget 정책.

**다른 것:** integration code의 OS process boundary와 그에 따른 IPC/lifecycle만 다르다.

현재 IPC는 macOS/Linux CI에서 stdin/stdout OS pipe를 쓴다. 이것은 process isolation correctness를 실제 검증하지만 **Windows Named Pipe absolute latency를 측정한 것으로 주장하지 않는다.** Mac 사용자 실측도 해당 prototype transport의 절대값으로만 보고한다.

## 6. Common critical path implementation

| Common concern | 실제 구현 | 상태 |
|---|---|---|
| Durable Task state / CAS / dedup | runtime/repository.rs | IMPLEMENTED |
| Durable handoff / ExecutionLink | runtime/handoff.rs, SQLite outbox/link | IMPLEMENTED |
| Agent P/Q source | fixture/src/lib.rs | FIXTURE |
| Event-first + Query Reconciliation | runtime/sync.rs | IMPLEMENTED_TACTIC |
| Trace primitive | runtime/trace.rs + Python timing contract | PARTIAL |
| S2S elapsed-time replay | S2sDelayTrace, timing contract | CONTRACT_READY / measured trace missing |
| Model semantic path | Python IR harness + frozen prompts/schema | HARNESS_READY / actual model missing |
| Voice/UI actual delivery | 없음 | NOT_IMPLEMENTED |
| OS Context capture / production Context Broker | canonical fixture | FIXTURE_ONLY |
| Policy/Consent/Memory production service | safety/context fixture | FIXTURE_ONLY |
| full Compound runtime | canonical relationship assets, no full runtime | PARTIAL |

## 7. 현재 smoke/invariant evidence가 증명하는 것

현재 source revision은 다음을 CI에서 검증해야 Measurement Freeze 입력으로 인정한다.

- Task command dedup/revision/race/terminal invariants.
- stale/wrong-run Agent event rejection.
- Per-Task activation fencing.
- Agent acceptance 후 local link commit 전 crash의 same-submission recovery.
- Edge/Typed Agent boundary의 동일 canonical semantics.
- same-process/process bridge의 full lifecycle transport.
- isolated worker fatal 후 Core host 생존과 worker restart.
- 동일 fatal fault에서 shared host 종료 vs isolated Core host 생존.
- S2S TEST_ONLY replay가 scoring evidence로 승격되지 않음.
- 4 DP catalog, 5 unique configuration, raw metric/change ledger가 모두 미측정 상태.

이것들은 **product p95, W-05 accuracy, W-09 6-strata 대표값, W-10 28-cell 대표값을 증명하지 않는다.**

## 8. Measurement Freeze 전에 남은 blocker

| Blocker | 왜 필요한가 | 상태/담당 |
|---|---|---|
| 최신 full-lifecycle 코드의 macOS+Ubuntu CI green | 현재 source revision 검증 | 자동 CI |
| Mac 실제 environment manifest | 실제 measurement host 식별 | 사용자 Mac에서 1회 실행 |
| Qwen artifact/runtime/tokenizer hash | IR A/B 동일 dependency 증명 | 사용자 Mac setup 후 자동 capture |
| measured S2S delay trace 또는 synthetic 사용 범위 승인 | simulated E2E provenance | 리뷰 시 결정. TEST_ONLY trace는 score 금지 |
| full W-04 workload driver | 1 vs 4 active Task steady-state ratio | 구현 필요 |
| full W-09 6-strata controller | recovery representative metric | 구현 필요 |
| full W-10 28-cell controller | containment representative metric | 구현 필요 |
| 94 TC→prototype end-to-end observation adapter | W-05/06와 회귀 전수 | 구현 필요 |
| W-07/08 24 change raw analysis | change locality | freeze 후 동일 source revision에서 실행 |
| actual Qwen run | W-05/IR latency | freeze 승인 후 사용자 Mac |

따라서 현재 상태는 **Architecture candidate implementation이 상당 부분 executable해진 상태**지만, 아직 MEASUREMENT_READY라고 선언하지 않는다. blocker를 닫거나 리뷰에서 명시적 범위를 승인한 뒤 measurement revision을 freeze한다.
