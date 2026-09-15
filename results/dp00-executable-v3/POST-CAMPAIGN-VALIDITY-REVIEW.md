# DP-00 Executable v3 Final Post-Campaign Validity Review

Campaign: `dp00-executable-reference-v3-qa-v1`  
Frozen source: `1e4b868842dc5188eb9be41042364341efa44155`  
Review time: `2026-09-15T09:16:58Z`

## Disposition

**ACCEPTED AS VALID EXECUTABLE QA-v1 EVIDENCE. FULL QUALIFICATION BLOCKED ONLY BY QA-07.**

No R1/R3 preference is asserted. Every evaluated QA target and hard gate passed; QA-07 remains formally unevaluable because its approved physical-memory denominator does not exist. R1+@ measurably improves the bounded-read diagnostic but not the frozen global primary scalar materially.

## Critical audit

| Question | Finding |
| --- | --- |
| Were architecture-specific paths exercised? | Yes. Each QA-01 stratum executed 350 measured repetitions per realization; Specialist was Driver→VIA→Specialist in R1 and Driver→Shell→Primary→Specialist in R3. |
| Are scalar ties explained by raw execution? | Yes. At structural p95, R3 emitted 4 IPC operations, 3 hops, and 7,594 bytes versus R1 3, 2, and 4,435; both remained score 5. |
| Did candidate-visible fixtures leak evaluator answers? | No. Runtime directories contained only RuntimeInput, SemanticReplay, AgentReplay, and state. Static gates reject expected/correct/required-result/graph fields and evaluator imports. |
| Were QA-04/05/06 real and behaviorally accepted? | Yes. All 540 clean worktrees contain a request-specific Rust handler and named acceptance test. The output proves that test and all five common regressions ran; seams and dependencies were parsed from the source patch. |
| Were failure measurements real? | Yes. 300 total process-fault cases changed an owning component PID across the three realizations; event-only failures exercised delivery/correlation, duplicate, late, and projection recovery paths. |
| Were privilege observations real? | Yes. An independent Policy process produced scoped grants and every applicable process boundary recorded the envelope. |
| Did trace reconstruction use normal telemetry? | Yes. It used ordinary component spans; the missing-span mutation made reconstruction fail. |
| Is QA-01 wall-clock based and correct? | Yes. Timed driver request/response uses monotonic wall clock; paired modality probes are real but excluded from latency. There are no summed architecture constants. |
| Are official scalars reproducible? | Yes. Re-running the frozen evaluator over raw JSONL reproduced all 33 evaluated scalars, scores, targets, qualification states, and hard gates exactly. |
| Did source change after preregistration? | No. The diff from `1e4b8688…` over candidate, fixture, protocol, and evaluator source is empty. |

## Autonomous invalidation history

- Attempt 1 (`d56c09ab…`) was invalidated because QA-04/05/06 used compile-only markers and copied approved seam data.
- Attempt 2 (`8ce72bf0…`) was invalidated because QA-01 omitted its modality-consistency probe and the campaign gate failed to reject penalty results.
- Attempt 3 (`1e4b8688…`) corrected both defects, added mutations/gates for them, reran the complete campaign, and passed this audit.

## Residual limits

The deterministic replay controls model and Agent quality; the evidence qualifies executable architecture behavior under QA-v1, not production model quality. The local memory diagnostic is not a QA-07 substitute. Ceiling scores in many QAs mean the frozen contract did not convert observed structural differences into preference evidence.

Decision: **FULL QUALIFICATION BLOCKED ONLY BY QA-07**.
