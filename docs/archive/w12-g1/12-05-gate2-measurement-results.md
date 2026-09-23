# 12-05. Gate 2 Measurement Results and Decision Closure

> **W12-G2 notice:** 이 문서의 W-01~W-03 상태와 판단은 이전 endpoint의 historical evidence다. 새 Voice 정의는 [11-E](./11e-voice-responsiveness-measurement-redefinition.md)를 따르며 아직 측정되지 않았다.
> 후속 16-configuration campaign과 갱신된 결정은 `12-07-full-factorial-results.md`를 따른다. 이 문서는 최초 structural campaign의 역사적 결과다.

> 상태: **MEASUREMENT COMPLETE / 12-A COVERAGE CLOSED / ONE DECISION ACCEPTED / THREE DEFERRED**
>
> 측정 source: `b6ff0b073185a5a08ac1dfe0c5c94ad9510b0ba1`
>
> freeze fingerprint: `9b8cf4bf8633e10b10c05b91f05bb5c31fc67bf29e4ae3bb07ae3d0b4ff7084d`

## 1. 결론

Measurement Freeze 승인 뒤 W-07/W-08 aggregate, W-09 representative recovery 1,200 trials, W-10 28-cell × 2 candidate × 10 trials를 실행했다. Working-12 전체는 측정 가능 여부까지 빠짐없이 기록했으며, 미실행 값을 0점으로 바꾸거나 weighted winner를 만들지 않았다.

- **AGENT-DP01: A — Edge-normalized Canonical Contract를 Accepted로 결정한다.** W-07이 A `1.444 / score 4`, B `1.778 / score 3`으로 1 score band 차이를 보여 Differentiation Gate를 통과했다.
- **IR-DP01, TASK-DP01, EXEC-DP01: Deferred.** 현재 측정값은 동점 band이거나 필수 endpoint/dependency가 없어 G3를 통과하지 못했다. Reference A를 유지하되 Architecture winner로 주장하지 않는다.
- EXEC B의 recovery raw metric 개선은 유효한 secondary evidence다. 그러나 A `550.833ms / score 5`, B `374.667ms / score 5`로 같은 band와 같은 target pass이므로 사전 고정한 Gate 규칙을 바꾸지 않는다.

Machine-readable source of truth:

- `results/rebaseline/gate2-freeze-b6ff0b07/measurement-coverage.json`
- `results/rebaseline/gate2-freeze-b6ff0b07/sensitivity-sweep.json`
- `results/rebaseline/gate2-freeze-b6ff0b07/representative-final/representative-summary.json`
- `results/rebaseline/gate2-freeze-b6ff0b07/w07-w08-scores.json`

## 2. Working-12 evidence coverage

| Working ASR | 상태 | 이번 결론에 사용한 근거 |
|---|---|---|
| W-01 | BLOCKED_NOT_RUN | 실제 user-visible delivery endpoint 부재. Headless timing은 승인 정책상 score 불가 |
| W-02 | NOT_RUN | representative Agent-acceptance trace 없음 |
| W-03 | NOT_RUN | representative event-to-visible trace 없음 |
| W-04 | BLOCKED_NOT_RUN | 실제 user-visible delivery endpoint 부재 |
| W-05 | BLOCKED_NOT_RUN | runner는 준비됐으나 실행 환경에 `OPENROUTER_API_KEY` 없음 |
| W-06 | NOT_RUN | representative continuity trace 없음 |
| W-07 | MEASURED | 9 Agent changes, DESIGN_ANALYSIS |
| W-08 | MEASURED | 15 non-Agent changes, DESIGN_ANALYSIS |
| W-09 | MEASURED | whole-process 4 strata + integration-fatal 2 strata |
| W-10 | MEASURED | shared/isolated 각각 frozen 28-cell matrix |
| W-11 | NOT_RUN | oracle 검증 완료, representative protected-unit trace 없음 |
| W-12 | NOT_RUN | oracle 검증 완료, representative action trace 없음 |

Synthetic/replay S2S는 구조·계약·simulated sensitivity 보조 근거로만 남긴다. Live measured S2S가 없으므로 representative score나 absolute product latency 주장을 하지 않는다.

## 3. 측정 결과

### W-07 / W-08

| DP candidate | W-07 metric / score | W-08 metric / score |
|---|---:|---:|
| AGENT A | **1.444 / 4** | 1.933 / 4 |
| AGENT B | **1.778 / 3** | 1.933 / 4 |
| IR A/B | N/A for DP hypothesis | 1.933 / 4 tie |
| TASK A/B | N/A for DP hypothesis | 1.933 / 4 tie |
| EXEC A/B | N/A for DP hypothesis | N/A for DP hypothesis |

W-07 차이는 A-03, A-08, A-09에서 Core-visible typed contract가 Core handler/interface까지 변경되는 구조로 추적된다. W-08은 모든 후보가 같은 non-Agent change set을 보여 non-discriminating regression이다.

### W-09 representative recovery

| 비교 | A | B | 판정 |
|---|---:|---:|---|
| TASK shared vs per-Task | 550.833ms / 5 | 551.167ms / 5 | noise 수준, non-discriminating |
| EXEC shared vs isolated | 550.833ms / 5 | 374.667ms / 5 | B raw 개선, 같은 band·target pass |

EXEC B의 차이는 integration-fatal strata에서 직접 발생했다. Shared는 `545/563ms`, isolated는 `18/33ms` p95였다. 반면 whole-process 4 strata는 두 EXEC 후보가 공유하므로 composite score band는 갈리지 않았다. 이 결과는 process boundary의 구조 효과를 보여주지만 사전 고정 G3를 충족하지 않는다.

### W-10 representative containment

| EXEC candidate | passed cells | metric | score |
|---|---:|---:|---:|
| A shared | 28 / 28 | 100% | 5 |
| B isolated | 28 / 28 | 100% | 5 |

두 후보 모두 고정된 external 24 + fatal 4 cell을 통과했다. 따라서 W-10은 중요한 regression constraint지만 이번 DP의 구분축은 아니다.

이 수치는 macOS arm64 prototype에서 얻은 `MEASURED_STRUCTURAL` 증거다. Target Windows absolute performance나 local Q4_K_M model latency로 해석하지 않는다.

## 4. Differentiation Gate

| DP | G3를 통과한 Primary | 상태 | 선택 |
|---|---|---|---|
| IR-DP01 | 없음 | Deferred | 없음; reference A 유지 |
| TASK-DP01 | 없음 | Deferred | 없음; reference A 유지 |
| AGENT-DP01 | W-07 | **Passed G1~G5** | **A Accepted** |
| EXEC-DP01 | 없음 | Deferred | 없음; reference A 유지 |

AGENT/W-07은 제품 관련성, 자연스러운 contract-boundary 인과성, 1 score band sensitivity, W-08과의 비중복성, change ledger traceability를 모두 만족한다. 나머지는 결과를 숨기지 않고 secondary/regression 또는 blocked evidence로 남긴다.

## 5. 선택안 weakness와 tactic

### AGENT-DP01 A — Accepted

Weakness는 provider lifecycle 의미가 integration edge에 집중되어 adapter가 작은 semantic core처럼 비대해지거나 provider별 해석이 drift할 수 있다는 점이다.

적용 tactic:

1. versioned canonical Agent operation/observation contract
2. 모든 adapter가 공유하는 conformance suite와 lifecycle fixtures
3. capability를 명시하고 미지원 기능은 `Unsupported` 또는 safe hold로 반환
4. native identity, source revision, pending action, artifact provenance를 trace에 보존

이 tactic은 authority를 Core-visible typed handler로 옮기지 않으므로 후보 A의 identity를 유지한다.

### Deferred DP의 interim tactics

- IR A reference: schema validator, bounded repair, prompt/schema versioning으로 joint authority의 결합 위험을 제한한다.
- TASK A reference: short transaction, CAS/revision check, outbox/inbox, 외부 호출 lock-outside로 contention과 중복 effect를 제한한다.
- EXEC A reference: bounded worker, timeout/backpressure, process-wide restart supervision으로 blast radius를 완화한다. B의 process isolation은 재평가 후보로 유지한다.

## 6. 재개 조건

- IR: frozen OpenRouter profile로 W-05와 model-call path를 실행할 credential 제공
- TASK: W-02 actual acceptance와 W-04 actual user-visible delivery trace 제공
- EXEC: W-01/W-04 actual delivery 측정, 또는 사전 승인된 더 민감한 score contract로 **새 freeze** 수행

현재 checkout에서 분석 코드/ADR을 추가했으므로 기존 fingerprint는 과거 측정의 provenance다. 추가 representative 측정은 새 source manifest와 새 approval을 생성해야 한다.
