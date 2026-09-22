# 12-04. Measurement Freeze Review — Readiness Gate

> 상태: **REVIEW DECISIONS RESOLVED / APPROVAL INPUT**. 이 문서의 승인 전 candidate별 representative metric, 0~5 score, ranking/winner를 산출하지 않는다.
> 기준 branch: `architecture-rebaseline-20260918`.

## 1. Freeze 판정 체계

| 상태 | 의미 |
|---|---|
| **READY** | 비교 계약/구현이 freeze 입력으로 충분함 |
| **READY_WITH_LIMITATION** | 비교는 가능하지만 주장 범위를 제한해야 함 |
| **REVIEW_DECISION_REQUIRED** | 실행 전에 범위/증거 사용 여부를 명시적으로 결정해야 함 |
| **POST_FREEZE_EXECUTION** | 구현/계약은 준비됐고 representative run만 freeze 뒤 수행 |
| **BLOCKED** | freeze 전에 반드시 닫아야 함 |

## 2. 현재 readiness

| 항목 | 현재 상태 | Freeze 판단 |
|---|---|---|
| 4 Core DP code-to-C/I/S/D mapping | `12-04a`에 실제 구현 mapping 존재 | **READY** |
| TASK-DP01 A/B | 동일 SQLite/WAL/FULL + shared service vs per-Task supervisor | **READY** |
| AGENT-DP01 A/B | 동일 P/Q capability + edge normalized vs Core-visible typed | **READY** |
| EXEC-DP01 A/B | same-process vs child-process boundary | **READY_WITH_LIMITATION** — prototype IPC의 절대 Windows 성능을 주장하지 않음 |
| IR-DP01 A/B prompt/schema/controller | integrated vs staged contract + client validation/repair | **READY** |
| Semantic reference execution | OpenRouter `qwen/qwen3-8b`, provider `alibaba`, fallback off | **READY_WITH_LIMITATION** — `MEASURED_MODEL_REFERENCE`; local/target-PC absolute latency가 아님 |
| Working-12 machine scoring contract | `benchmark/rebaseline/working12/baseline.json` | **READY** |
| W-07/W-08 24-change raw ledger | 192 DESIGN_ANALYSIS rows; aggregation pre-freeze 금지 | **READY / POST_FREEZE_EXECUTION** |
| W-09 recovery controller | 6-strata controller smoke complete | **POST_FREEZE_EXECUTION** |
| W-10 containment controller | 28-cell candidate endpoint controller smoke complete | **POST_FREEZE_EXECUTION** |
| W-11 protected-unit oracle | 20 unit corpus/evaluator | **READY** |
| W-12 safety oracle | 24 opportunity corpus/evaluator | **READY** |
| 94 TC executable-slice mapping / terminal outcome audit | CI guard 존재 | **READY** |
| S2S final evidence | TEST_ONLY/synthetic/replay trace는 보조 분석만 가능 | **READY_WITH_LIMITATION** — representative score 불가 |
| W-01/W-04 actual user-visible delivery source | raw adapter는 준비, 실제 source wiring은 미완료 | **READY_WITH_LIMITATION** — endpoint가 없으면 BLOCKED/NOT_RUN |
| representative metric/score ledger | null / NOT_RUN | **READY — 반드시 이 상태로 freeze review에 진입** |

## 3. Hosted Qwen reference의 정확한 의미

IR A/B는 동일한 hosted Qwen3-8B profile을 controlled dependency로 사용한다.

- model: `qwen/qwen3-8b`
- OpenRouter provider: `alibaba`
- provider fallback: disabled
- non-thinking: `/no_think` + frozen sampling
- structured response: JSON object + VIA client-side schema validation/repair
- API key: environment variable only; repository/evidence에 저장 금지

이 실행에서 얻은 W-05 semantic result와 model-call critical-path latency는 **동일 hosted reference condition의 Architecture A/B 비교 근거**로 사용할 수 있다. 그러나 이를 local Q4_K_M 또는 target Windows PC의 absolute model latency라고 기술하지 않는다.

## 4. Freeze 시 반드시 잠그는 것

1. exact Git source fingerprint
2. Working-12 metric/target/0~5 boundary
3. four DP A/B element inventory와 fairness rules
4. 94 TC / W-10 / W-11 / W-12 oracle
5. W-07/W-08 raw change rules
6. IR prompt/schema/validation/repair limit
7. hosted model id/provider/fallback/non-thinking/sampling contract
8. trial count/order 및 timeout/censoring rules
9. 아래 D-1/D-2의 S2S와 actual-delivery evidence scope decision

`freeze.py` fingerprint와 approval fingerprint가 달라지면 comparative execution/scoring을 거부한다.

## 5. Freeze Review 확정 결정

### D-1. S2S
실측 S2S trace가 없으면 synthetic/replay는 `SIMULATED_E2E`로만 유지한다. 구조 correctness, 계약 검증, sensitivity 보조 분석에는 사용할 수 있지만 **representative metric/0~5 score와 final absolute product latency 주장에는 사용하지 않는다.** Representative score에는 `LIVE_MEASURED_S2S`가 필요하다.

### D-2. W-01/W-04 delivery source
대표값은 `ACTUAL_USER_DELIVERY` sink의 실제 Text/audio/user-visible endpoint만 허용한다. 준비되지 않으면 해당 representative metric은 **`BLOCKED/NOT_RUN`**으로 남긴다. Headless smoke를 score로 승격하거나, 0점·추정값으로 대체하거나, 후보별로 다른 proxy를 적용하지 않는다. 가용 metric 기반 결론에는 evidence coverage를 함께 표시한다.

### D-3. Target-PC external validity
Hosted Qwen과 macOS/Linux process prototype 결과는 Architecture 비교 근거이며 target Windows PC absolute performance를 증명하지 않는다.

## 6. Freeze 승인 후 순서

```text
approved source fingerprint
  → representative raw execution
  → W-01..W-12 metric materialization
  → Working-12 fixed scoring
  → 12-A sensitivity sweep
  → Differentiation Gate
  → DP decision / weakness / tactic
  → ADR
```

W-07/W-08 평균과 모든 0~5 score는 이 승인 이후 frozen revision에서만 계산한다. W-01/W-04는 실제 delivery source가 추가되지 않는 한 승인 후에도 `BLOCKED/NOT_RUN`이며 전체 점수에 대입하지 않는다.
