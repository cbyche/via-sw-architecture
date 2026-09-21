# Architecture-Sensitive Probe Design — LLM과 구조 효과 분리

> 작성일: 2026-09-21
> 상태: 11-C/12 연결 규칙. 후보 Architecture 결과를 보기 전에 정의.

## 1. 문제

ASR-02/03/06/07은 기능 요구를 충실히 구현한 후보가 모두 만점을 받을 수 있다. 특히 LLM이 강해지면 부족한 semantic hint를 추론으로 보완하여 구조적으로 부족한 evidence/state 전달을 가릴 수 있다.

이 문제를 해결하기 위해 **Architecture-sensitive probe**를 사용한다. Probe의 목적은 후보를 일부러 실패시키는 것이 아니라, architecture가 보존·전달해야 하는 authoritative information이 없으면 intelligence만으로 정답을 확정할 수 없도록 만드는 것이다.

## 2. Probe qualification

Primary QA용 probe는 다음 조건을 모두 만족한다.

1. 후보의 structural mechanism이 다르다.
2. 정답이 그 mechanism이 보존/전달하는 state/evidence/authority에 실제로 의존한다.
3. 동일 Model/Agent fixture를 쓴다.
4. missing information을 test harness가 보충하지 않는다.
5. semantic shortcut이나 world knowledge로 authoritative identity를 추론할 수 없게 한다.
6. counterfactual pair에서 hidden mapping을 바꾸면 올바른 architecture는 함께 적응하지만 guess 기반 구조는 동시에 맞힐 수 없다.

## 3. ASR-02 Probe 예

### P02-GROUNDING-PAIR

- 화면의 두 target에 실행마다 random nonce/value를 부여한다.
- 자연어는 동일한 '여기와 여기 차이를 알려줘'를 사용한다.
- Pair A/B에서 pointer event의 target order만 뒤집는다.
- 정답은 timestamped pointer/source identity에만 존재한다.
- latest snapshot 또는 semantic guessing만으로는 A/B를 동시에 맞힐 수 없다.

검증 대상: interaction evidence ownership/materialization boundary.

### P02-TASK-BINDING-PAIR

- 동일 Agent가 비슷한 두 Task를 실행한다.
- Task/run/artifact ID는 opaque random ID를 사용한다.
- Pair A/B에서 user-visible follow-up이 연결되어야 하는 Task mapping만 바꾼다.
- Task title이나 Agent name만으로 정답을 추론할 수 없게 한다.

검증 대상: Request→Task association information과 execution correlation의 전달.

## 4. ASR-03 Probe 예

### P03-S2S-NONCE-CONTINUITY

- S2S Direct Response가 매 run random nonce를 한 개 생성해 실제 사용자 응답에 포함한다.
- 후속 요청은 '방금 말한 코드로 문서 만들어줘'처럼 nonce를 다시 직접 말하지 않는다.
- nonce는 world knowledge나 prompt prior로 추론할 수 없다.
- S2S 응답이 VIA Conversation에 실제 기록되어 다음 Core/Agent 경로에 제공될 때만 정확한 continuation이 가능하다.

검증 대상: S2S provider-local history vs VIA-owned Conversation continuity.

### P03-RUN-CORRELATION-PAIR

- 동일 Agent의 두 run이 random artifact A/B를 반환한다.
- follow-up은 prior interaction에서 선택된 한 run의 결과를 참조한다.
- Pair에서 run↔artifact mapping을 swap한다.

검증 대상: VIA Task ↔ Agent thread/run ↔ artifact correlation persistence.

## 5. ASR-06 Probe

LLM 추론과 무관한 fault schedule을 사용한다.

- persist 전/후 Agent handoff crash
- duplicate/out-of-order event
- cancel/result race
- restart 이후 external run alive
- 완료됐지만 local commit 전 crash
- same action replay 위험

정답은 durable state authority, idempotency key, atomic handoff/outbox, event correlation 같은 구조 mechanism에 의존한다.

두 후보가 서로 다른 tactic으로 모든 invariant를 만족하면 둘 다 100%가 정상이다. 이때 reliability는 해당 DP의 변별 QA가 아니거나 동점이며, tactic 비용/변경 ripple/latency는 ASR-01/05 등에서 비교한다.

## 6. ASR-07 Probe

정적 allow/deny만 보지 않고 **check와 use 사이에 state를 바꾸는 race**를 사용한다.

- t0: access/approval check
- t1: consent revoke 또는 Action revision 변경
- t2: 실제 egress/action commit

또한 pending Action ID, destination, policy revision을 opaque하게 한다.

정답은 LLM의 판단이 아니라 **실제 egress/action boundary에서 authoritative current policy와 revision을 enforcement하는 구조**에 달려 있다.

두 후보 모두 use-time enforcement와 exact approval binding을 구현하면 0/24가 정상이다. 이 경우 ASR-07은 constraint이며, 구조 비용은 다른 ASR에서 관찰한다.

## 7. Fixed-intelligence rule

Architecture 비교에서 semantic intelligence는 통제변수다.

- 동일 Qwen3-8B artifact/runtime/decoding
- 동일 Qwen3-Omni reference/dependency 조건
- 동일 prompt/schema contract where applicable
- 동일 Agent capability/result fixture

특정 후보만 더 강한 Model, 더 긴 hidden context, oracle metadata를 받아서는 안 된다.

## 8. DP 적용 규칙

각 DP에서 ASR-02/03/06/07을 Primary QA로 넣기 전에 해당 DP의 구조 차이를 자극하는 probe를 지정한다.

Probe에서 후보 결과가 같거나, 분석상 두 후보 모두 같은 authoritative information/mechanism을 제공한다면 해당 ASR을 Primary에서 내린다.

이 규칙은 후보 차이를 억지로 만드는 것을 금지하면서도, 실제 구조 sensitivity가 있는 QA만 trade-off 표에 남게 한다.
