# Gate 2 Reference Campaign Contract

> 상태: PRE-RESULT FROZEN REFERENCE CONTRACT

이 campaign은 제품 실측이 아니라 Architecture 비교용 deterministic reference harness다. Qwen3-Omni 공개 first-packet reference 234ms, 생성 WAV, instrumented text/audio sink, Agent P/Q acceptance stub, frozen continuity/privacy/safety fixture를 사용한다. 결과는 `SIMULATED_REFERENCE` 또는 `MEASURED_*_FIXTURE`이며 production UI, live S2S, target Windows absolute latency 주장을 금지한다.

## Candidate mapping

`AAAA`를 기준으로 IR/TASK/AGENT/EXEC 한 축만 B로 바꾼 `BAAA/ABAA/AABA/AAAB` 5개 configuration을 사용한다.

## Frozen logical timing model

- W-01: 234ms S2S reference + six frozen case offsets `[22,28,35,31,26,40]` + IR A/B `18/42ms` + EXEC A/B `4/9ms` + delivery `8ms` + frozen p95 jitter envelope `9ms`.
- W-02: acceptance stub `40ms` + IR A/B `10/25ms` + TASK A/B `6/9ms` + AGENT A/B `5/8ms` + p95 envelope `9ms`. 종료점은 acceptance와 durable link 모두다.
- W-03: reference render `8ms` + AGENT A/B event normalization `12/15ms` + p95 envelope `9ms`.
- W-04: same-candidate 4-to-1 ratio `1.04 + TASK-A 0.01 + EXEC-A 0.01`.

이 상수는 실제 component 성능을 주장하는 값이 아니라 사전 고정된 mockup 비용 모델이다. Architecture sensitivity와 scoring-path 실행을 확인하며, 실제 제품 acceptance에는 사용할 수 없다.

## Frozen functional model

- W-06: continuity TC 30개 × 5회, 동일 fixture가 모든 obligation을 보존하는지 기록한다.
- W-11: 8 workload의 기능을 유지하며 20 protected unit 중 P01~P05가 remote endpoint로 전달되는 공통 profile을 사용한다.
- W-12: 6 family × allow/deny/stale/wrong-scope 24개를 모두 실행한다. allow 6개는 통과, block 18개는 차단하는 공통 policy fixture를 사용한다.

후보를 가르기 위해 사후 상수, 노출 unit, policy outcome을 바꾸지 않는다.

## W-07~W-10 DP applicability

| Metric | Applicable DP | 다른 DP가 N/A인 이유 |
|---|---|---|
| W-07 | AGENT-DP01 | Agent substitution/change boundary를 바꾸는 DP이기 때문 |
| W-08 | IR-DP01, TASK-DP01, AGENT-DP01 | 현재 change catalog는 semantic/state/Agent contract 변화이며 EXEC process placement는 같은 code change set을 공유 |
| W-09 | TASK-DP01, EXEC-DP01 | durable state owner와 fault/restart boundary가 recovery critical path를 바꿈 |
| W-10 | EXEC-DP01 | 후보 사이 OS process blast radius를 바꾸는 유일한 DP |

다른 DP를 적용하려면 단순 데이터가 아니라 해당 DP에 자연 인과가 있는 별도 fault/change scenario를 결과 전에 정의하고 새 freeze해야 한다. 현재 결과를 본 뒤 임의 scenario를 추가하지 않는다.
