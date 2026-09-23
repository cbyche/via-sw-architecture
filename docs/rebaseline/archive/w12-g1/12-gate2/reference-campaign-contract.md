# Gate 2 Reference Campaign Contract

> 상태: PRE-RESULT FROZEN REFERENCE CONTRACT

> **Historical notice (W12-G2 draft):** 이 contract의 W-01~W-03 정의와 수식은 [`11e-voice-responsiveness-measurement-redefinition.md`](../11e-voice-responsiveness-measurement-redefinition.md)에 의해 superseded되었다. 기존 값은 삭제하지 않지만 새 Voice W-01~W-03 결과나 DP별 A/B 판단에 사용하지 않는다. 측정 코드는 아직 변경되지 않았다.

이 campaign은 제품 실측이 아니라 Architecture 비교용 deterministic reference harness다. Qwen3-Omni 공개 first-packet reference 234ms, 생성 WAV, instrumented text/audio sink, Agent P/Q acceptance stub, frozen continuity/privacy/safety fixture를 사용한다. 결과는 `SIMULATED_REFERENCE` 또는 `MEASURED_*_FIXTURE`이며 production UI, live S2S, target Windows absolute latency 주장을 금지한다.

## Candidate mapping

Configuration ID는 `IR / TASK / AGENT / EXEC` 순서다. 다음 16개 `2^4` complete configuration을 모두 사용한다.

```text
AAAA AAAB AABA AABB ABAA ABAB ABBA ABBB
BAAA BAAB BABA BABB BBAA BBAB BBBA BBBB
```

기존 `AAAA/BAAA/ABAA/AABA/AAAB` one-factor 결과는 역사적 OFAT evidence로 보존하되, 최종 결합 판단에는 16개 campaign의 context별 A/B contrast를 사용한다.

## Frozen integrated execution order

각 configuration은 단순 행 복제가 아니라 동일한 reference request가 다음 결합 trace를 통과한다.

```text
Task view/revision read
→ IR semantic decision
→ TASK submit-pending/outbox commit
→ EXEC integration dispatch
→ AGENT acceptance/normalization
→ TASK feedback + durable ExecutionLink commit
→ reference delivery
```

모든 trace는 revision `7 → 8 → 9`, 정확히 한 Task writer, 선택된 Agent contract owner와 integration host를 기록한다. 이 trace는 구조적 결합과 선후관계를 검증하지만 production stack wall-clock 실행이라고 주장하지 않는다.

## Frozen logical timing model

- W-01: 234ms S2S reference + six frozen case offsets `[22,28,35,31,26,40]` + IR A/B `18/42ms` + EXEC A/B `4/9ms` + delivery `8ms` + frozen p95 jitter envelope `9ms`.
- W-02: acceptance stub `40ms` + IR A/B `10/25ms` + TASK A/B `6/9ms` + AGENT A/B `5/8ms` + p95 envelope `9ms`. 종료점은 acceptance와 durable link 모두다.
- W-03: reference render `8ms` + AGENT A/B event normalization `12/15ms` + p95 envelope `9ms`.
- W-04: same-candidate 4-to-1 ratio `1.04 + TASK-A 0.01 + EXEC-A 0.01`.

결과 확인 전에 다음 결합 비용을 고정한다.

| Metric | 교차항 | 값 | 구조적 이유 |
|---|---|---:|---|
| W-01 | IR-B × EXEC-B | +6ms | staged semantic envelope가 isolated bridge frame을 통과하는 reference 비용 |
| W-02 | IR-B × TASK-B | +4ms | AssociatedRequest revision을 activation command로 넘기는 비용 |
| W-02 | TASK-B × AGENT-B | +3ms | typed lifecycle result를 supervisor command로 바꾸는 비용 |
| W-02 | AGENT-B × EXEC-B | +4ms | worker에서 온 typed variant를 Core handler가 해석하는 추가 boundary |
| W-03 | AGENT-B × EXEC-B | +4ms | typed feedback payload가 IPC 뒤 Core normalization을 거치는 비용 |
| W-04 | TASK-B × EXEC-B | -0.01 ratio | per-Task queue와 isolated integration queue의 공유 간섭 제거 |

위 값은 상호작용이 0이라고 가정해 OFAT 결과를 복제하지 않기 위한 **사전 동결 reference-model 항**이다. 실측 성능 수치가 아니다. 그 외 pair/higher-order interaction은 이 reference model에서 0으로 고정한다.

이 상수는 실제 component 성능을 주장하는 값이 아니라 사전 고정된 mockup 비용 모델이다. Architecture sensitivity와 scoring-path 실행을 확인하며, 실제 제품 acceptance에는 사용할 수 없다.

## Frozen functional model

- W-06: continuity TC 30개 × 5회, 동일 fixture가 모든 obligation을 보존하는지 기록한다.
- W-11: 8 workload의 기능을 유지하며 20 protected unit 중 P01~P05가 remote endpoint로 전달되는 공통 profile을 사용한다.
- W-12: 6 family × allow/deny/stale/wrong-scope 24개를 모두 실행한다. allow 6개는 통과, block 18개는 차단하는 공통 policy fixture를 사용한다.

후보를 가르기 위해 사후 상수, 노출 unit, policy outcome을 바꾸지 않는다.

## Factorial analysis

각 metric에 대해 B-minus-A marginal main effect와 모든 applicable 2-way difference-of-differences를 산출한다. 또한 한 DP를 바꾸는 8개 matched context의 raw preference와 score-band split 수를 보존한다. Weighted total과 global winner는 만들지 않으며 W-05 누락을 대입하지 않는다.

## W-07~W-10 DP applicability

| Metric | Applicable DP | 다른 DP가 N/A인 이유 |
|---|---|---|
| W-07 | AGENT-DP01 | Agent substitution/change boundary를 바꾸는 DP이기 때문 |
| W-08 | IR-DP01, TASK-DP01, AGENT-DP01 | 현재 change catalog는 semantic/state/Agent contract 변화이며 EXEC process placement는 같은 code change set을 공유 |
| W-09 | TASK-DP01, EXEC-DP01 | durable state owner와 fault/restart boundary가 recovery critical path를 바꿈 |
| W-10 | EXEC-DP01 | 후보 사이 OS process blast radius를 바꾸는 유일한 DP |

다른 DP를 적용하려면 단순 데이터가 아니라 해당 DP에 자연 인과가 있는 별도 fault/change scenario를 결과 전에 정의하고 새 freeze해야 한다. 현재 결과를 본 뒤 임의 scenario를 추가하지 않는다.
