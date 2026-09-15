# DP-00 Executable Reference Architecture Qualification v3 Results

## Evidence status

VALID EXECUTABLE QA-v1 EVIDENCE; official QA-07 remains unevaluable.

## QA-v1 results

| QA | R1 raw / score | R3 raw / score | R1+@ raw / score | Target |
| --- | ---: | ---: | ---: | --- |
| QA-01 | 0.101122 seconds / 5 | 0.106199 seconds / 5 | 0.101047 seconds / 5 | p95 <= 3.0 seconds |
| QA-02 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=95% |
| QA-03 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=99% |
| QA-04 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=95% |
| QA-05 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=90% |
| QA-06 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=90% |
| QA-07 | UNEVALUABLE | UNEVALUABLE | UNEVALUABLE | <=1.5x |
| QA-08 | 0.00701942 seconds / 5 | 0.0135833 seconds / 5 | 0.00803221 seconds / 5 | p95 <=10 seconds |
| QA-09 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=98% |
| QA-10 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=98% |
| QA-11 | 9.8166e-05 seconds / 5 | 0.000139459 seconds / 5 | 9.1792e-05 seconds / 5 | p95 <=1.5 seconds |
| QA-12 | 10 milliseconds / 5 | 10 milliseconds / 5 | 10 milliseconds / 5 | p95 <=200 ms |

## Decision

**FULL QUALIFICATION BLOCKED ONLY BY QA-07**

R1 and R3 are not selected while QA-07 lacks the approved physical-memory denominator. R1+@ is evaluated as an R1 tactic; any latency improvement is reported without treating it as a third base architecture.

## Provenance and execution validity

- Branch: `exp/dp00-executable-reference-v3`
- Frozen preregistration: `1e4b868842dc5188eb9be41042364341efa44155`
- Imported snapshot: `cbyche/via-internal-rust-reference`, snapshot repository commit `37b69d81162357d4291ee9d5655a57ed99cc3c53`, original source commit `b6032a4c472e18ec216b3e8592346ab269495342`
- Reference manifest: 650 files; `SHA256SUMS` SHA-256 `118b62c1bb6fdf50b8ad2360118dea66e830cd80f486fbc9a57bcee0a8cd5319`
- Candidate-visible artifacts: RuntimeInput, SemanticReplay, AgentReplay, plus candidate-owned state only; evaluator oracle files were absent
- R1 topology: Scenario Driver → R1 VIA Control Plane → selected General or Specialist process
- R3 topology: Scenario Driver → R3 VIA Shell → Primary Runtime → optional Specialist process
- R1+@ topology: separate R1 executable with a deterministic local read-only, non-durable bounded tactic
- Label swap: changing only the external label while launching the same executable produced identical normalized behavior

## QA-01 diagnostics

| Realization | bounded/read p50 / p95 | General p50 / p95 | Specialist p50 / p95 | p95 IPC / hops / bytes | architecture overhead p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| R1 | 99.267 / 101.163 ms | 99.295 / 101.023 ms | 100.448 / 101.107 ms | 3 / 2 / 4,435 | 21.122 ms |
| R3 | 104.915 / 106.139 ms | 105.198 / 106.061 ms | 105.229 / 106.252 ms | 4 / 3 / 7,594 | 26.199 ms |
| R1+@ | 93.269 / 95.392 ms | 99.464 / 101.052 ms | 99.806 / 101.053 ms | 3 / 2 / 4,435 | 21.047 ms |

Each stratum contains 50 cases × 7 measured repetitions per realization. R1+@ improves bounded-read p95 by 5.770 ms relative to R1, but changes the official 150-case p95 by only 0.075 ms; it is not a material improvement to the frozen primary scalar.

## Behavioral and evolution evidence

- QA-02: 600/600 goals per realization; each of the 60 semantic family IDs passed 10/10, including an actually executed opposite-modality consistency probe.
- QA-03: 400/400 chronological episodes per realization; revision, clarification, approval, concurrent tasks, compound work, follow-up, cancellation, late result, and resource events were exercised.
- QA-04: 180/180 worktrees compiled and passed one named case-specific acceptance plus five common regressions. R1/R1+@ changed `src/ownership/r1_agent_adapter.rs`; R3 changed `src/ownership/r3_primary_extensions.rs`; all mapped to Z4 and their distinct frozen integration owners.
- QA-05: 180/180 worktrees passed. Each realization covered 8 Z1, 8 Z2, 8 Z3, 8 Z4, 7 Z5, 7 Z6, 7 Z7, and 7 Z8 cases. The Z4 source differed by architecture; observed dependency leakage was zero.
- QA-06: 180/180 Mobile/TV/Robot worktrees changed only `src/ownership/device_adapters.rs` (Z8); derived Core semantic changes were empty and all behavioral acceptance tests passed.

## Failure, privilege, trace, feedback, and playback evidence

- QA-08: 200/200 safe continuations per realization. One hundred cases per realization changed a real component PID. R1 owners were General Agent, Specialist, context dependency, and VIA delivery/correlation; R3 replaced General with Primary Runtime. Safety was derived from PID replacement, chronological events, Task identity, result version, one-result semantics, cancellation binding, and late-result rejection.
- QA-09: 600 true-positive scopes, zero false positives, zero false negatives, and zero hard violations per realization. R1 emitted 300 VIA→Agent crossings; R3 emitted 300 Shell→Primary and 60 Primary→Specialist crossings.
- QA-10: 200/200 normal telemetry graphs reconstructed per realization. The R1 Specialist chain is direct from VIA; the R3 chain includes Primary interpretation/planning and Specialist delegation.
- QA-11 event-class p95 ranges: R1 0.0938–0.1009 ms; R3 0.1350–0.1427 ms; R1+@ 0.0879–0.0942 ms across accepted, progress, approval-needed, completion, and failure.
- QA-12: 200/200 cases per realization stopped at one actual 10 ms frame after onset through the common local ring-buffer clear path.
- QA-07: officially UNEVALUABLE. Non-scoring local RSS was R1 9,680 KiB, R3 10,064 KiB, and R1+@ 9,776 KiB, five processes each. No approved physical denominator, PSS/private/shared split, or target calibration was invented.

## Validation totals

- Rust tests: 5
- Python/evaluator tests: 248
- Official observations: 7,290 (2,430 per realization; QA-07 excluded)
- Runtime E2E observations: 6,750; QA-01 measured executions: 3,150; QA-01 paired modality executions: 900
- Evolution: 540 clean worktrees, 540 named acceptance tests, 2,700 common regression executions
- Real mutation/validity probes: 8, all detected
- Cloud/API calls: 0
- Independent scalar recomputation: 33/33 evaluated candidate/QA scalars exact match
- Preregistration source diff after campaign: empty

Attempts preregistered at `d56c09ab…` and `8ce72bf0…` were preserved and invalidated during autonomous review. Neither contributes qualification evidence.
