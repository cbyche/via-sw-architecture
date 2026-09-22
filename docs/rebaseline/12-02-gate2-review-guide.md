# 12-02. Gate 2 — Core Architecture Candidate Review

> **G2-DESIGN-v1.1 / 2026-09-22 / 사용자 승인 완료 — 구현·측정 준비로 전환**
> Source snapshot: `e9646d4a074afb7a3cf4e85f5ca48c939a697fbf`.
> 목적은 **발표에서 억지 trade-off를 만드는 것이 아니라, 실제로 강한 SW Architecture decision만 남겨 “잘 만든 A vs 잘 만든 B”를 비교 가능하게 만드는 것**이다. 아직 점수·승자·최종 Architecture 없음.

## Working ASR 빠른 참조

| ID | ASR / QA 의미 |
|---|---|
| **W-01** | **Conversational Reaction Responsiveness** — 사용자가 말한 뒤 의미 있는 첫 응답이 얼마나 빨리 시작되는가 |
| **W-02** | **Task Handoff Responsiveness** — 사용자 요청이 실제 Agent 접수+복구 가능한 연결까지 얼마나 빨리 도달하는가 |
| **W-03** | **Task Feedback Responsiveness** — Agent 상태·질문·결과가 준비된 뒤 사용자에게 얼마나 빨리 보이는가 |
| **W-04** | **Concurrent Task Performance Isolation** — 여러 Task가 동시에 있을 때 foreground 반응성이 얼마나 덜 저하되는가 |
| **W-05** | **Task Completion Effectiveness** — referent·요청·Task·Agent 연결 의무를 얼마나 정확히 만족하는가 |
| **W-06** | **Interaction & Task Continuity** — modality/connection/Task 전환에도 맥락·identity를 얼마나 보존하는가 |
| **W-07** | **Agent Ecosystem Interoperability & Substitutability** — Agent 추가·교체 시 VIA 변경이 얼마나 국소적인가 |
| **W-08** | **Evolvability & Maintainability** — Model/Context/Storage 변화가 VIA 구조에 얼마나 적게 퍼지는가 |
| **W-09** | **Recovery Timeliness & Recoverability** — 장애·재시작 뒤 올바른 Task control을 얼마나 빨리 회복하는가 |
| **W-10** | **Dependency Failure Containment & Graceful Degradation** — 한 dependency 장애가 무관 기능까지 얼마나 덜 전파되는가 |
| **W-11** | **Privacy Exposure Minimization** — 민감 Context를 외부 dependency에 얼마나 최소 범위로 노출하는가 |
| **W-12** | **Action & Access Safety** — 잘못된 접근·반출·승인 연결을 정확히 차단하는가 |

## 1. 자체 리뷰 후 Gate 2 비교 대상 — 4개

처음 상세화한 6개 중 두 개를 다시 탈락시켰다. **INT-DP01은 S2S Direct Fast Path를 고정 원칙으로, TASK-DP02는 event+query 혼합 동기화 tactic으로 내린다.** 둘 다 유효한 설계 관심사지만 현재 요구에서는 강한 상호배타적 Architecture 대안이 아니다.

| Gate 2 DP | A | B | 중요하게 보는 ASR |
|---|---|---|---|
| [IR-DP01](./12-gate2/IR-DP01.md) | **Integrated Semantic Authority** | **Staged Semantic Authorities** | W-01 대화 반응성, W-02 인계 반응성, W-05 요청 처리 충실도, W-08 변경 용이성 |
| [TASK-DP01](./12-gate2/TASK-DP01.md) | **Shared Transactional Task Service** | **Durable Per-Task Supervisor** | W-02 인계, W-04 동시 Task 격리, W-08 변경 용이성, W-09 복구 |
| [AGENT-DP01](./12-gate2/AGENT-DP01.md) | **Edge-normalized Canonical Contract** | **Core-visible Typed Contracts** | W-02 인계, W-03 feedback, W-07 Agent 교체성, W-08 변경 용이성 |
| [EXEC-DP01](./12-gate2/EXEC-DP01.md) | **Single-process Partitioned Runtime** | **Process-isolated Integration Runtime** | W-01 대화 반응성, W-04 동시성, W-09 복구, W-10 장애 격리 |

이 네 개는 각각 **semantic authority topology / Task single-writer model / Agent contract boundary / OS process fault boundary**라는 서로 다른 Architecture 축을 결정한다. Supporting DP인 CTX-DP01/02, SEC-DP01은 Master Catalog에 유지하되 본 Gate 2의 우선 비교에서 제외한다.

## 2. DP에서 내린 두 주제

### FP-INT01 — S2S Direct Fast Path를 고정 원칙으로 둔다

Voice 입력은 Voice Runtime과 S2S Model을 거친다. S2S가 **자체 지식+Conversation만으로 직접 응답 가능하다고 유효하게 판단한 경우**, Core에 “허락”을 받으러 갔다 돌아오는 B 구조는 현재 요구에서 추가적인 제품 가치를 증명하지 못한다. 따라서 S2S Direct Response는 Voice Runtime이 release하고, Core는 필요한 요청만 escalation받으며 Conversation 기록·Task 연계는 공통 계약으로 보장한다.

상세 rationale은 [INT-DP01 note](./12-gate2/INT-DP01.md)에 보존한다. 이것은 W-01을 무조건 좋게 만들기 위한 선택이 아니라 **비교 가치가 약한 dominated candidate를 제거한 것**이다.

### TASK-T01 — Event-first + Query Reconciliation을 공통 tactic으로 둔다

실제 Agent protocol은 query와 event/stream을 함께 제공할 수 있다. A2A도 polling, streaming, push를 **complementary mechanisms**로 설명한다. 따라서 “query만 vs event만”을 Architecture family처럼 비교하지 않는다.

VIA의 기본 tactic은 **stream/event를 지원하면 low-latency update에 사용하고, query를 reconnect/gap/current-state reconciliation 및 지원 제한 Agent의 fallback으로 사용**한다. 지원 여부·polling cadence·cursor/reconnect는 Agent profile에 남긴다. 상세 rationale은 [TASK-DP02 note](./12-gate2/TASK-DP02.md)에 보존한다.

## 3. 네 DP에 대한 자체 리뷰 결과

### IR-DP01 — 유지, 단 W-05는 Model 의존적이다

같은 Qwen reference Model이라도 한 번에 joint decision을 내릴 때와 3개 stage로 나눌 때 실제 정확도는 달라질 수 있다. **그 방향은 Architecture만으로 예측할 수 없고 Model의 structured reasoning 능력에 상당 부분 의존한다.** 따라서 W-05를 “stage형이 더 정확하다/부정확하다”는 논리로 쓰지 않고 실제 동일 Model·동일 corpus 결과로만 평가한다.

Architecture 자체가 직접 바꾸는 것은 prompt/call critical path와 semantic contract의 변경 국소성이다. 따라서 W-01/W-02/W-08은 강한 구조 인과, W-05는 **model-dependent empirical discriminator**로 취급한다.

### TASK-DP01 — 유지, 단 ‘Agent harness의 두 표준 형태’라고 말하지 않는다

A는 일반적인 shared service+durable store에 가깝고, B는 workflow/actor 계열의 **per-execution durable owner**에 가깝다. LangGraph는 thread/checkpointer 기반 shared state를 제공하고, Temporal은 Workflow Execution별 durable Event History와 복구 가능한 실행 단위를 제공한다. 이는 두 구조 family가 실무적으로 존재한다는 참고이지, “모든 Agent harness가 A/B 중 하나”라는 표준 분류는 아니다.

VIA는 여러 async Task, cancel/race, restart reconnect를 직접 소유하므로 이 선택이 실제 Task supervision Architecture에 의미가 있다. 두 안 모두 같은 DB/persistence 조건으로 비교한다.

### AGENT-DP01 — 유지, protocol 선택과 별개다

질문은 “A2A냐 custom protocol이냐”가 아니다. **어떤 wire protocol을 쓰더라도 Agent별 capability/lifecycle 차이를 어디에서 semantic하게 흡수할 것인가**다. 같은 A2A에서도 polling/streaming/push capability와 Message/Task lifecycle 차이가 존재할 수 있다.

따라서 protocol binding은 외부 interface의 한 입력이고, 이 DP는 VIA Core가 provider variation을 보게 할지 integration edge에서 canonicalize할지를 결정한다. 모든 Agent가 동일한 엄격한 lifecycle profile을 보장한다면 이 DP의 trade-off는 작아질 수 있으며 그 경우 결과도 동점으로 인정한다.

### EXEC-DP01 — 유지, Rust/Tokio에서 실현 가능하다

Rust라고 단일 process여야 하는 것은 아니다. Tokio는 child process를 비동기 관리할 수 있고 Windows named pipe용 async API도 제공한다. 따라서 A는 **한 process 안의 Tokio task/queue 격리**, B는 **Core process + integration worker process를 Tokio로 supervise하고 named pipe 등 IPC로 연결**하는 현실적인 비교다.

Tokio가 process isolation 자체를 제공하는 것은 아니다. **격리는 Windows OS process boundary이고 Tokio는 child lifecycle과 async IPC를 관리하는 실행 도구**다. B의 IPC/serialization/restart 비용을 숨기지 않는다.

## 4. 공정한 비교 기준

공통 계약과 전체 C/I/S/D는 [common-contract](./12-gate2/common-contract.md)를 따른다. 한 DP 비교에서는 해당 축만 바꾸며 Model/Agent fixture, Context·Policy 조건, DB와 worker budget을 동일하게 유지한다.

- IR: 같은 Model/corpus. A에 작은 prompt, B에 일부러 많은 call을 강제하지 않음.
- TASK: A에 전역 mutex를 넣지 않고 B에 별도 DB를 주지 않음.
- AGENT: 두 안이 동일 required capability를 보존.
- EXEC: 같은 integration code/fault를 host 경계만 다르게 실행.

4 pair의 change ledger는 공통 reference + 각 DP의 B variant라는 one-factor contrast를 유지한다. Runtime/reference 평가는 **16개 `2^4` complete configuration**으로 조립해 IR×TASK, TASK×AGENT, TASK×EXEC 등 결합 효과를 함께 확인한다.

## 5. 결과 상태와 다음 단계

Gate 2 구조는 사용자 승인 완료했다. 다음 구현·측정 단계에서는 아래 네 구조를 그대로 동결해 실행 자산을 만든다.

1. IR의 1회 통합 판단 vs 단계별 판단이 실제 VIA semantic responsibility를 잘 나타내는가.
2. TASK의 shared writer vs per-Task durable writer가 지나치게 특수하거나 불공정하지 않은가.
3. AGENT의 normalization boundary가 protocol 선택과 명확히 분리되는가.
4. EXEC의 same-process vs separate-process가 Windows/Rust 구현에서 현실적인가.

이 네 구조의 C/I/S/D와 metric ledger를 Gate 2 승인본으로 freeze하고, 12-A sensitivity 측정 준비로 넘어간다.
