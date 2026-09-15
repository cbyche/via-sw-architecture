# DP-00 Executable v3 Post-Campaign Validity Review

Campaign: `dp00-executable-reference-v3-qa-v1`  
Frozen source: `d56c09ab29254b2e35db64188a2dbc7dca1e2223`

## Disposition

**INVALIDATED — EVOLUTION ACCEPTANCE TESTS DID NOT PROVE THE REQUESTED CHANGES.**

This attempt is retained only as invalid historical evidence. QA-04/05/06 appended compile-only constants, generic regression tests did not assert the requested functionality, and the recorded extension seam was copied from evaluator-side allowed/approved fields rather than independently derived from the implemented change. Consequently, its scores and decision are not admissible DP-00 qualification evidence. No R1/R3 preference is asserted from this run. The implementation and instrument must be fixed, newly preregistered, and the complete campaign rerun.

## Critical questions

| Question | Finding | Evidence |
| --- | --- | --- |
| Were architecture-specific paths exercised? | Yes | 350 measured repetitions in each QA-01 stratum; R1 Specialist path was Driver→VIA→Specialist, R3 was Driver→Shell→Primary→Specialist. |
| Are ties explained by raw execution rather than simulator construction? | Yes | R3 p95 had 4 IPC operations/3 process hops and 7,594 serialized bytes at structural p95 versus R1 3/2 and 4,435 bytes. R3 QA-01 p95 was 106.207 ms versus R1 101.090 ms, but both score 5. |
| Did candidate-visible fixtures leak answers? | No | Candidate directories contained only RuntimeInput, SemanticReplay, AgentReplay, and state; no oracle path or expected/correct/required-result/graph field was present. |
| Were QA-04/05 observations actual, behaviorally accepted code changes? | **No** | 540 detached worktrees built/tested, but each edit was only a compile marker and its test command exercised generic regressions rather than a case-specific acceptance assertion. |
| Were failures real process faults? | Yes | General, Specialist, context Semantic service and R3 Primary topologies were killed/restarted; recovery times came from monotonic execution and persisted workflow state. |
| Were privilege results real boundary observations? | Yes | A separate Policy process granted 600/600 required scopes with 0 FP/FN and no hard-gate violation; R3 emitted Shell→Primary and Primary→Specialist crossings. |
| Did trace reconstruction use normal telemetry? | Yes | Reconstruction consumed the same interaction/semantic/policy/dispatch/execution/result/delivery spans emitted by ordinary executions. The missing-span code mutation failed reconstruction. |
| Is QA-01 wall-clock based? | Yes | Python monotonic request/response timing around actual release processes; 5 warmups, 7 repetitions, per-case median, 150-case nearest-rank p95. No summed hop constants. |
| Are scalars reproducible? | Yes | All 33 evaluated candidate/QA scalars were independently recomputed through the frozen shared evaluators and matched stored values exactly. |
| Did source change after preregistration? | No | `git diff d56c09ab… -- benchmark/dp_executable_v3 benchmark/fixtures/dp00-executable-v3 prototypes/dp00-executable-v3` is empty. |

## Invalidation trigger

The post-campaign review found a critical defect that the preflight mutation suite had not covered: compile success had been treated as functional completion. It also found evaluator-field leakage into the observation construction through `approved_extension_seams`. Per the frozen campaign protocol, this invalidates the full attempt even though runtime/topology measurements were otherwise structural.

## Measurement sensitivity observed before invalidation

All seven pre-registered real-defect probes were detected: bad result correlation, removed trace span, inserted proxy process, unauthorized policy grant, cross-zone dependency leak, Agent evolution requiring a Core edit, and 250 ms playback buffering. No boolean runtime-mutation switch was used.

## Coverage

- QA-01: bounded, General-Agent and Specialist strata each executed 50 cases × 7 repetitions per realization.
- QA-03: 400 chronological episodes; revision, clarification, approval, two-task concurrency, follow-up, cancellation, late result, resource acquire/release and interrupted delivery event families executed.
- QA-08: 200 faults per realization across VIA delivery/correlation, context dependency, General/Primary and Specialist ownership surfaces.
- QA-09: 300 cases per realization; all applicable trust boundaries emitted scope envelopes.
- QA-10: 200 normal execution traces per realization reconstructed after execution.

## Residual limitations

- QA-07 remains officially UNEVALUABLE: no approved physical minimum-mandatory-memory denominator exists. Local aggregate RSS was 8,592 KiB (R1), 8,800 KiB (R3), and 8,400 KiB (R1+@); these are non-scoring host diagnostics.
- Deterministic SemanticReplay and AgentReplay control semantic quality. The result qualifies architecture execution under QA-v1, not production model quality or target-device performance.
- Ceiling ties in QA-02–06 and QA-09/10/12 mean the frozen populations did not convert observed topology differences into score separation. They do not prove the architectures are structurally equivalent.
- No DP-00 ADR is warranted until QA-07 is calibrated and the full integrated campaign is rerun or explicitly accepted as non-discriminating by the architecture authority.
