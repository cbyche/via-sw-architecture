# VIA SW Architecture — Gate 1 Final

> **W12-G2 notice:** 아래 W-01~W-03 DP 연결은 이전 정의의 historical hypothesis다. 새 Voice endpoint와 명칭은 [11-E](./11e-voice-responsiveness-measurement-redefinition.md)를 따르며, 새 측정 전 DP×W mapping을 다시 동결한다.
> **Post-Gate-1 note (2026-09-22):** Gate 2 상세 리뷰에서 INT-DP01은 `FP-INT01 S2S Direct Fast Path` 고정 원칙으로, TASK-DP02는 `TASK-T01 Event-first + Query Reconciliation` tactic으로 내리는 안을 제안했다. 후보 실행·점수 산출 전 보정이며, 현재 Gate 2 리뷰본은 [12-02](./12-02-gate2-review-guide.md)를 우선한다.
> **리뷰 목적: 비교할 구조 질문과 그 상호배타적 대안의 방향을 승인하는 것. 후보 점수나 승자를 고르는 단계가 아닙니다.**
> **상태: Gate 1 최종 기준선 — 사용자 피드백 반영 완료.**
> W12-G1-FINAL / Working ASR 12개 전수 유지 / UC18·variation94·Change24 유지.

## 0. DP 선정 원칙

좋은 DP는 단순히 ASR 점수를 잘 가르는 주제가 아니라 다음을 순서대로 만족해야 한다.

1. **Structural significance** — Component/Interface/State/Runtime의 책임·권한·경계가 실제로 달라지는가.
2. **Mutual exclusivity** — 두 대안이 같은 authoritative owner / canonical contract / primary state path / fault boundary를 서로 다르게 결정해, 단순히 A+B를 동시에 적용하면 decision이 사라지는가.
3. **Intuitive alternatives** — 심사위원이 “아, 이 구조와 저 구조를 비교하는구나”를 바로 이해할 수 있는가.
4. **ASR leverage** — 위 조건을 통과한 DP 중 우선순위가 높은 ASR을 여러 개 자연스럽게 건드리는가.

두 대안에 공통 tactic을 넣는 것은 허용한다. 그러나 “A의 좋은 점과 B의 좋은 점을 그냥 같이 사용”할 수 있다면 두 안은 architecture alternative가 아니라 tactic/feature 조합으로 보고 DP를 재정의하거나 제외한다. Hybrid는 별도 authority split과 독립 비용이 있을 때만 별도 architecture family로 승격한다.

## 1. 제안하는 DP 목록 — 9개

| DP | 구조적으로 결정하는 것 | 강한 대안 2개 | 관찰 우선 W-ASR |
|---|---|---|---|
| **INT-DP01 Interaction Routing & Fast-Path Ownership** | User Turn의 direct/semantic/task 경로를 누가 authoritative하게 결정하는가 | **Voice-owned Turn Router** / **Core-owned Turn Router** | W-01, W-02, W-08 |
| **CTX-DP01 Context Materialization Ownership** | Source reference를 소비 가능한 Context로 만드는 canonical 책임이 어디에 있는가 | **Central Materialization Authority** / **Consumer-owned Resolution** | W-01, W-05, W-08, W-11 |
| **IR-DP01 Semantic Decision Ownership** | Referent·Request·Task·Handling·Agent 판단을 하나의 semantic authority가 결정하는가, 단계별 authority가 결정하는가 | **Integrated Semantic Authority** / **Staged Semantic Authorities** | W-01, W-02, W-05, W-08 |
| **CTX-DP02 Model-facing Context State Architecture** | 매 inference의 Model-visible history를 매번 재구성하는가, 지속 working set으로 유지하는가 | **Request-reconstructed Context** / **Incremental Working Context** | W-01, W-06, W-08 |
| **TASK-DP01 Task State Authority & Supervision** | VIA Task 상태 전이의 authoritative owner가 하나의 공유 service인가, Task별 supervisor인가 | **Central Task Authority** / **Per-Task Authority** | W-02, W-04, W-08, W-09 |
| **AGENT-DP01 Agent Integration Contract Boundary** | Agent별 차이를 integration edge에서 숨길지, Core 계약에 typed variation으로 노출할지 | **Edge-normalized Canonical Contract** / **Core-visible Typed Contracts** | W-02, W-03, W-07, W-08 |
| **TASK-DP02 Agent State Synchronization Architecture** | Agent 실행 상태를 query가 authoritative하게 갱신하는가, revisioned event가 authoritative하게 갱신하는가 | **Pull-authoritative Reconciliation** / **Event-authoritative Streaming** | W-03, W-04, W-07, W-09 |
| **SEC-DP01 Policy Enforcement Hot-path Architecture** | 민감 Context/Action 사용 때 중앙 online authorization을 매번 거칠지, 사전 발급한 revocable capability를 local 검증할지 | **Online Reference Monitor** / **Revocable Scoped Capability** | W-02, W-08 *(W-11/12 regression)* |
| **EXEC-DP01 Runtime Fault-Isolation Boundary** | integration workload를 Core와 같은 process fault domain에 둘지, 별도 process fault domain에 둘지 | **Single-process Partitioned Runtime** / **Process-isolated Integration Runtime** | W-01, W-04, W-09, W-10 |

## 2. 이번 정제에서 제외한 것

- **MODEL-DP01은 추가하지 않는다.** 현재 과제는 on-device를 기본 배치로 진행하고 local/remote/hybrid Model placement를 독립 DP로 넓히지 않는다.
- **STATE-DP01은 추가하지 않는다.** journal/snapshot/outbox는 현재 우선 ASR leverage가 낮고 TASK-DP01의 후보를 구현하는 persistence tactic으로 우선 유지한다.
- 기존 **ORCH-DP01 Bounded Request Execution Placement는 독립 DP에서 제외한다.** bounded request를 일부 Core, 일부 Agent로 처리하는 혼합이 제품상 자연스럽기 때문에 Core vs Agent를 전역 양자택일로 두면 상호배타적 Architecture decision이 아니다. handling 선택은 IR-DP01의 semantic decision과 AGENT-DP01의 integration contract 안에서 정책으로 다룬다.

## 3. 경계가 헷갈리기 쉬운 묶음

- **CTX-DP01 vs CTX-DP02**: Source→Context materialization 책임 / 이미 보존한 Conversation·Task→Model-visible working context 책임.
- **TASK-DP01 vs TASK-DP02 vs EXEC-DP01**: Task state authority / Agent state synchronization authority / process fault-domain 배치.
- **INT-DP01 vs IR-DP01**: 어느 subsystem이 turn의 routing authority를 갖는가 / semantic 판단 자체를 하나로 할지 단계로 할지.

## 4. Gate 1 최종 분류

**Core DP 6개** — 구조 질문이 크고 상호배타성이 명확하며 responsiveness 계열과 다른 중요 품질의 trade-off를 동시에 만들 가능성이 높다.

- INT-DP01
- IR-DP01
- TASK-DP01
- AGENT-DP01
- TASK-DP02
- EXEC-DP01

**Supporting DP 3개** — Architecture significance는 있으나 hybrid 가능성, measurement sensitivity 또는 우선 ASR leverage가 상대적으로 약해 Master Catalog에는 유지하되 발표 본문 우선순위는 낮다.

- CTX-DP01
- CTX-DP02
- SEC-DP01

Core/Supporting은 중요도 점수가 아니라 **Gate 2에서 먼저 상세화·측정할 우선순위**다. Supporting DP도 결과가 강하면 발표 본문에 올라갈 수 있고, Core DP도 결과가 평평하면 appendix로 내려갈 수 있다.

## 5. 발표 관점의 1차 기대

현재 후보 중 responsiveness 계열과 함께 다른 구조 품질을 동시에 건드릴 가능성이 높은 축은 **INT-DP01, IR-DP01, TASK-DP01, AGENT-DP01, TASK-DP02, EXEC-DP01**이다. 이는 발표 채택 확정이 아니라 Gate 2/3에서 실제 수치 sensitivity를 우선 확인할 묶음이다.

CTX-DP01/02와 SEC-DP01도 master catalog에는 유지하되, 실제 결과가 평평하거나 우선 ASR leverage가 약하면 발표 본문 대신 appendix decision으로 남길 수 있다.

## 6. 결과 상태

**후보 선택 없음. 후보 성능값 없음.** Gate 1에서 9개 구조 질문과 상호배타적 대안 family를 최종 확정한다. Gate 2에서 각 대안을 C/I/S/D, 공통 tactic, runtime/API까지 채워 “잘 만든 A vs 잘 만든 B”로 동결한 뒤 점수 산출을 시작한다.

상세: [Master Catalog](./12-01-dp-master-catalog.md) · [Coverage](./12-01a-scope-and-coverage-ledger.md) · [12개 평가 계약](./11d-working-12-measurement-and-scoring.md).
