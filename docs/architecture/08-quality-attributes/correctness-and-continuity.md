# Correctness & Continuity Measurement Contract

> 상태: **PRE-IMPLEMENTATION CONTRACT DRAFT / corpus·target·score 미확정 / 결과 NOT_RUN**
>
> 목적: QA-11~QA-15를 서로 다른 단일 metric으로 정의하고, integrated outcome과 driver QA의 중복 가중을 방지한다.

## 1. 공통 원칙

- Downstream Agent의 조사·계획·Tool 실행·문서 품질은 이 QA들의 채점 대상이 아니다.
- 사람이나 LLM이 실행 중 자유문장을 주관적으로 채점하지 않는다.
- 각 case의 evaluator-only predicate와 허용 해석은 결과 전에 machine-readable 형식으로 승인한다.
- timeout, invalid output, exhausted repair와 missing required trace는 실패다.
- 같은 실행을 여러 QA에서 관찰할 수 있지만 QA-11과 QA-12~15를 weighted total에 독립 표처럼 합산하지 않는다.
- wrong-target Action, duplicate external Action, revoked/mismatched approval과 unauthorized access는 별도 mandatory zero-violation gate다.

각 QA의 기본 판정은 다음 형식을 따른다.

```text
strict_run_pass = 해당 QA의 모든 applicable predicate가 PASS
QA pass rate = 100 × strict PASS 수 / 전체 scored run 수
```

case별 반복과 case aggregation은 결과 전에 고정한다. 쉬운 case 수를 늘려 어려운 lifecycle case를 희석하지 않는다.

## 2. QA-11 — VIA Request Handling Correctness

### 질문

사용자의 요청이 VIA 책임 범위 전체에서 올바르게 처리되어 요구한 응답·위임·상태 연결까지 완료됐는가?

### 단일 metric

```text
correct_machine_oracle_runs_pct
= 100 × mean(case별 PASS runs / 해당 case의 전체 scored runs)
```

이 이름, 범위와 macro case aggregation은 previous-generation QA-05에서 유지한다. 한 run은 해당 case에 사전 등록된 모든 applicable assertion이 맞아야 통과한다. 올바른 clarification을 요청했지만 고정 대화의 완료 조건까지 도달하지 못했다면 원래 업무 성공과 적절한 보류를 구분한다.

QA-11은 product-level integrated outcome이다. QA-12~15는 QA-11 실패의 구조 원인을 분리하며, QA-11 점수를 QA-12~15와 합산하지 않는다.

## 3. QA-12 — Request Semantic Resolution Correctness

### 질문

VIA가 사용자 입력과 허용된 Context에서 요청의 의미를 올바르게 결정했는가?

### 단일 metric

```text
strict_semantic_resolution_pass_rate_pct
```

### Predicate 범위

- goal, constraint와 결과물 요구
- referent, 집합, 범위와 역할
- compound Request decomposition과 독립·순차·데이터 의존·조건 관계
- clarification 필요 여부와 질문 의미
- No Tracked / New / Existing Task Relation
- Direct/Agent handling과 Agent/capability selection에 필요한 의미

Task ID, run ID, question ID에 실제 event를 연결하는 행위는 QA-13이다. event 순서·revision 뒤 최종 state는 QA-14다.

## 4. QA-13 — Task & Interaction Binding Correctness

### 질문

VIA가 사용자 control과 외부 event를 의도한 Conversation, Request, Task, Agent Execution과 pending interaction에 연결했는가?

### 단일 metric

```text
strict_interaction_binding_pass_rate_pct
```

### Predicate 범위

- progress/result/failure와 Task/run binding
- clarification·approval 질문과 사용자 답변 binding
- follow-up·correction·cancel과 대상 Task/run binding
- result/artifact와 Conversation/Task binding
- 같은 Agent의 복수 실행과 여러 Agent 실행 구분
- dispatch, control delivery와 user-facing response의 required cardinality

한 run에서 wrong target, missing binding 또는 duplicate control delivery가 하나라도 있으면 실패다. 실제 외부 Action 위반은 QA 점수와 별개로 mandatory gate도 실패한다.

## 5. QA-14 — Async Task State Convergence Correctness

### 질문

지연·중복·순서 역전·동시 event 이후 VIA의 authoritative Task state가 oracle state로 수렴하는가?

### 단일 metric

```text
strict_async_state_convergence_pass_rate_pct
```

### Trace 조건

- 새 progress 뒤 오래된 progress 도착
- 동일 revision 또는 terminal event 중복
- cancel 요청과 completion 교차
- clarification 대기 중 failure 또는 completion 도착
- push event와 query reconciliation 충돌
- partial result와 terminal result 순서 변화

최종 Task status, terminality, result reference, pending interaction, accepted revision과 허용 control이 oracle과 모두 일치해야 trace가 통과한다. crash/restart가 포함된 fault recovery 시간은 QA-31에서 측정한다.

## 6. QA-15 — Interaction & Task Continuity Correctness

### 질문

정상적인 채널·대화·처리 경로·실행 전환 뒤에도 필요한 Conversation, Referent와 Task identity가 유지되는가?

### 단일 metric

```text
strict_continuity_scenario_pass_rate_pct
```

### Scenario 범위

- S2S Direct Response 뒤 follow-up
- Direct Response에서 New Task로 전환
- Voice → Text → Voice 전환
- Voice Connection 종료와 재연결
- 다른 대화가 끼어든 뒤 과거 대상·Task 재참조
- 완료된 Task의 동일 목표·결과물 수정
- 이전 결과물을 자료로 사용하는 별도 New Task
- 여러 active Task 사이의 전환

Voice Connection ID, Model provider conversation ID 또는 Agent thread/run ID를 VIA Conversation/Task identity로 대신해서는 안 된다. 프로세스 장애 뒤의 복원은 QA-31이며 정상 continuity 분모에 섞지 않는다.

## 7. Oracle types

각 QA는 필요한 predicate만 사용한다.

| Oracle type | 예 |
| --- | --- |
| `EXACT` | `task_relation=EXISTING`, `task_id=T-PPT` |
| `ONE_OF` | 둘 이상의 의미상 허용된 handling |
| `SET_EQUAL` | referent 집합이 정확히 `{chart-A, chart-B}` |
| `ORDERED_RELATION` | `summarize → send` dependency |
| `REQUIRED_PROPOSITION` | constraint에 recipient 포함 |
| `FORBIDDEN_PROPOSITION` | 철회한 target 또는 다른 Task 포함 금지 |
| `CLARIFICATION_REQUIRED` | 정보 부족 시 확인 필요 |
| `BINDING` | event/control/result의 Conversation·Task·run·question identity |
| `CARDINALITY` | dispatch 또는 control 적용이 정확히 한 번 |
| `FINAL_STATE_EQUAL` | 전체 event trace 뒤 authoritative state 일치 |
| `CONTINUITY_RELATION` | 전환 전후 identity·referent 관계 유지 |

자연어 문체와 표현 차이는 점수에 넣지 않는다. 필요한 proposition과 structured provenance만 판정한다.

## 8. Freeze 전에 남은 작업

- QA별 canonical case와 isolation fixture membership
- structured trace와 predicate JSON schema
- actual VIA model profile, temperature/seed와 반복 수
- case aggregation과 failure treatment
- target과 0~5 score band
- DP별 applicability와 QA-11/driver QA 보고 형식
