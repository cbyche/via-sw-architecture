# QA-v1 Frozen Corpus Coverage

| QA | Population | Frozen material | Families | Primary variation |
|---|---|---:|---:|---|
| QA-01 | `qa01-fast-v1` | 150 goals | 30 | modality, context version, capability, result obligations |
| QA-02 | `qa02-goals-v1` | 600 goals | 60 | six goal strata and meaningful fixture/task variations |
| QA-03 | `qa03-continuity-v1` | 400 episodes | 20 | 20 causal event interleavings |
| QA-04 | `qa04-agent-evolution-v1` | 60 changes | 10 | integration contract profiles |
| QA-05 | `qa05-product-evolution-v1` | 60 changes | 8 zones | predeclared ownership and seams |
| QA-06 | `qa06-device-adaptation-v1` | 60 cells | 3 devices | Mobile/TV/Robot domain requirements |
| QA-07 | `qa07-resource-v1` | P0–P5 fixed timeline | 1 | concurrent residency phases |
| QA-08 | `qa08-recovery-v1` | 200 episodes | 10 | fault type and lifecycle timing |
| QA-09 | `qa09-sensitive-scope-v1` | 300 scenarios | 15 | principal, scope, purpose, expiry, gate probes |
| QA-10 | `qa10-trace-v1` | 200 traces | 20 | 100 success and 100 fault/edge |
| QA-11 | `qa11-feedback-v1` | 200 obligations | 5 | exactly 40 per event class |
| QA-12 | `qa12-barge-in-v1` | 200 episodes | 10 | buffer, generation, contention, task state |

E2 legacy concepts are explicitly labeled in family manifests. They are
semantic seeds, not observations. Synthetic inputs are frozen and deterministic;
semantic model behavior is replayed, Agent behavior is emulated, and timing
inputs are evidence-anchored simulations. No live model or hardware performance
claim follows from corpus completeness.

The active corpus is complete as an evaluation instrument. Its empirical
evaluation status remains `NOT_EVALUATED` until a candidate supplies a complete,
provenance-bound observation set.

## Coverage against VIA UC-01–UC-16

Every frozen goal or episode carries validated `use_case_tags`. The executable
matrix is `benchmark/contracts/qa-v1/use-case-coverage-v1.json`; the detailed
review is `USE-CASE-COVERAGE.md`.

All UC-01–UC-16 have explicit multi-instance coverage. UC-02 has ten temporal
pointing goal families; UC-05 spans eight action domains; UC-11 distinguishes
four dependency forms; UC-12 distinguishes resolvable omissions from required
clarification; UC-14 covers the complete memory lifecycle; UC-15 covers four
Voice/Text transitions; and UC-16 includes explicit exclusive-resource
contention with an unblocked read-only task.

Use-case tags are descriptive product traceability only. They never select a
candidate route, component, Agent product, or architecture alternative.
