# 12-04. Measurement Freeze Review — Draft

> 상태: **PRE-BENCHMARK DRAFT**. 이 문서의 사용자 승인 전 candidate별 p95, W-05 실제 모델 점수, 0~5 score, winner를 산출하지 않는다.

## 1. 리뷰 대상

| 항목 | Freeze 시 확인할 내용 | 현재 상태 |
|---|---|---|
| 4 DP 실제 코드 | IR / TASK / AGENT / EXEC가 Gate 2 C/I/S/D와 일치 | IN_PROGRESS |
| 공통 fixed rule | FP-INT01 S2S Direct Fast Path | DOCUMENTED |
| 공통 tactic | TASK-T01 Event-first + Query Reconciliation | DOCUMENTED |
| Agent fixture | P/Q가 동일 능력, 다른 lifecycle shape만 제공 | INITIAL_IMPLEMENTATION |
| Repository | TASK A/B 동일 SQLite/WAL/FULL와 command dedup | INITIAL_IMPLEMENTATION |
| EXEC | same-process vs child-process boundary | INITIAL_IMPLEMENTATION |
| S2S timing | 0ms 금지, paired frozen trace, VIA-added와 simulated E2E 분리 | CONTRACT_DEFINED |
| W-01/W-04 observation | 6개 frozen foreground strata, raw endpoint derivation, correctness censoring, headless score 금지 | ADAPTER_READY / ACTUAL_DELIVERY_SOURCE_PENDING |
| IR prompts/schema | Integrated + Stage1/2/3 frozen input contract | DRAFT_IMPLEMENTED |
| IR actual model | Qwen3-8B Q4_K_M 동일 artifact/runtime | NOT_RUN |
| Mac environment | hardware/OS/toolchain manifest | USER_RUN_REQUIRED |
| candidate order | 10 warmup + 100 scored, ABBA/BAAB balanced blocks | DRAFT_DEFINED |
| 12-ASR raw ledger | null/NOT_RUN 유지 | NOT_RUN |
| score/winner | 산출 금지 | NOT_RUN |

## 2. S2S 시간

Voice path에서는 `user_input_end → S2S route/direct event available` 시간을 별도 dependency trace로 재생한다. 동일 paired trial의 A/B는 동일 sample을 사용한다.

- `VIA_added_latency`: VIA prototype에서 실제 계측.
- `SIMULATED_E2E_latency`: frozen S2S replay + VIA actual spans.
- TEST_ONLY trace는 smoke 전용이며 0~5 score에 사용할 수 없다.
- 실제 measured S2S trace를 나중에 넣을 때 candidate code·trial order를 바꾸지 않는다.

## 3. Mac 측정과 thermal drift

Target product platform과 measurement platform을 분리한다. 사용자의 Mac은 동일 A/B 비교의 measurement host이며 Windows에서 측정했다고 표기하지 않는다.

MacBook Air는 fanless이므로 A를 몰아 실행한 뒤 B를 몰아 실행하지 않는다. 각 stratum에서 `ABBA`, `BAAB` block을 교대로 사용하고 pre/post `pmset -g therm` 및 전체 elapsed time을 manifest에 남긴다. 절대 온도 sensor가 없다는 이유로 임의 temperature threshold를 만들지 않는다.

## 4. Qwen3-8B 실행 조건

- official `Qwen/Qwen3-8B-GGUF` revision `6a569868d07d3bd59e8b97fb001bf8c0b254bb20`.
- `Qwen3-8B-Q4_K_M.gguf` SHA256 `d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785`.
- 16K context, parallel 1, non-thinking.
- Qwen 권고 non-thinking sampling을 사용하고 동일 seed schedule을 A/B에 적용한다.
- llama.cpp runtime binary/version/hash는 실제 실행 전에 capture하고 고정한다.
- structured JSON은 client-side parse/schema smoke까지 확인한다. HTTP 200 자체를 schema 준수 증거로 보지 않는다.

## 5. 다음 리뷰에서 반드시 보여줄 것

1. CI green commit과 macOS/Ubuntu smoke 결과.
2. 실제 C/I/S/D ↔ source file mapping.
3. Candidate fairness matrix.
4. fault/restart smoke 결과와 아직 미구현인 fault.
5. IR prompt/schema와 model runner dry-run output 구조.
6. Mac bootstrap/environment capture command.
7. S2S TEST_ONLY trace와 measured-trace schema.
8. trial-order generator output.
9. 모든 BLOCKED/UNVERIFIED 항목.
10. **아직 비어 있는 candidate metric/score ledger.**

이 리뷰 뒤 frozen Git revision에서만 comparative benchmark를 시작한다.
