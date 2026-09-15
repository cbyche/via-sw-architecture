# DP-00 QA-01 Design-Time Reference Report v1

**Status: design-time reference evidence; not production measurement or SLO certification.**

## 1. Purpose and decision framing

FTOL remains **Acoustic EOS → earliest Useful Outcome**. This report reuses the completed 36-scenario, 432,000-realization evidence in two architecture views. It does not create a new simulation campaign, reweight the workload, or change any latency primitive.

The primary narrative is no longer one peer comparison of A/B/C/D. Stage 1 isolates the primary execution boundary; Stage 2 asks how bounded latency-sensitive capability is accommodated.

## 2. Two-stage DP-00 model

### 2.1 Stage 1 — Primary Execution Boundary

**A0 vs B0.** A0 keeps intent interpretation and top-level Agent selection in VIA; ARGO remains a peer Agent. B0 moves primary interpretation and initial execution/delegation authority into ARGO while VIA retains interaction and user-facing task projection/correlation.

Stage-1 question: **Should top-level semantic orchestration and Agent selection remain in VIA, or should primary semantic/execution authority move into ARGO?**

### 2.2 Why D0 is not retained

Removing `VIA_FAST` from D removes the execution-topology class that materially distinguishes its selector from A's Agent Router. Under the current specification, both remaining structures have VIA select ARGO or a Specialist, keep route/task state in VIA, reuse the route for clear follow-up, and place domain state in the selected executor. D0 is therefore **structurally equivalent to A0 for this trade-space purpose**, not a mathematical identity or an independent Stage-1 candidate. No new D0 responsibilities are invented.

### 2.3 Stage 2 — Bounded Fast-capability Accommodation

**C vs B1 vs D1.** C is A0 plus deterministic VIA Fast eligibility and local execution. B1 is B0 evaluated with bounded capability execution remaining inside the ARGO-owned execution boundary; it is not a VIA Fast Path or a new primary architecture. D1 exposes VIA Fast, ARGO Primary, and Specialist Direct inside one semantic topology-selection responsibility.

Stage-2 question: **How does each architecture accommodate bounded latency-sensitive capability while preserving its defining ownership boundary?**

## 3. Design-time latency methodology

The corpus remains 12 unique F1, 12 F2, and 12 F3 scenarios. Each scenario has 1,000 synthetic latency realizations per existing candidate and model profile. The 432,000 primary realizations are statistical samples of 36 semantic tasks, not 432,000 tasks.

The unchanged model is `V + architecture-derived M/H/X composition`: V is fixed at 500 ms; Qwen3-1.7B, Qwen3-8B, and Qwen3-30B-A3B provide three reference profiles; H = 5 ms and X1/X2/X3 = 20/50/100 ms are explicit assumptions; H = 1/5/20 ms and X = 0.5x/1x/2x form the sensitivity grid. Every candidate uses the same keyed primitive samples and model profile.

All results are evidence-anchored synthetic design-time distributions. They are not production traces, a production workload mix, or SLO evidence.

## 4. Stage 1 results — A0 vs B0

Current A evidence is reused exactly as A0; current B evidence is reused exactly as B0. F1 here means how each base boundary handles a local-capable request without a VIA Fast bypass.

### QWEN3_SMALL_REFERENCE

| Candidate | Overall p50 | Overall p95 | F1 p95 | F2 p95 | F3 p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| A0 | 982.337 | 1080.596 | 1026.639 | 1047.033 | 1110.412 |
| B0 | 815.939 | 1085.690 | 844.278 | 865.137 | 1120.124 |

### QWEN3_MEDIUM_REFERENCE

| Candidate | Overall p50 | Overall p95 | F1 p95 | F2 p95 | F3 p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| A0 | 1642.131 | 1817.879 | 1773.577 | 1797.682 | 1853.757 |
| B0 | 1258.024 | 1841.564 | 1344.232 | 1367.859 | 1917.442 |

### QWEN3_30B_A3B_REFERENCE

| Candidate | Overall p50 | Overall p95 | F1 p95 | F2 p95 | F3 p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| A0 | 1233.540 | 1358.001 | 1312.177 | 1334.321 | 1389.222 |
| B0 | 980.651 | 1368.459 | 1031.374 | 1053.897 | 1416.188 |

### Stage-1 path/model decomposition

| Category | A0 model generations / H | B0 model generations / H |
| --- | --- | --- |
| F1 | 3 / 1 | 1 / 1 |
| F2 | 3 / 1 | 1 / 1 |
| F3 | 3 / 1 | 2 / 2 |

A0 uses separate interpretation, route-decision, and short executor generations. B0 uses one combined ARGO generation for F1/F2; its F3 path adds Specialist reasoning and a second handoff. This explains both B0's shorter direct path and its Specialist-tail exposure.

### Stage-1 H/X crossover analysis

- QWEN3_SMALL_REFERENCE: A0 < B0 (7/9 cells); B0 < A0 (2/9 cells).
- QWEN3_MEDIUM_REFERENCE: A0 < B0 (9/9 cells).
- QWEN3_30B_A3B_REFERENCE: A0 < B0 (7/9 cells); B0 < A0 (2/9 cells).

Exact Stage-1 cells:

#### QWEN3_SMALL_REFERENCE

| H ms | X multiplier | A0 p95 | B0 p95 | Relation |
| ---: | ---: | ---: | ---: | --- |
| 1 | 0.5x | 1026.867 | 1020.218 | B0 < A0 |
| 1 | 1.0x | 1076.596 | 1077.690 | A0 < B0 |
| 1 | 2.0x | 1194.410 | 1198.408 | A0 < B0 |
| 5 | 0.5x | 1030.867 | 1028.218 | B0 < A0 |
| 5 | 1.0x | 1080.596 | 1085.690 | A0 < B0 |
| 5 | 2.0x | 1198.410 | 1206.408 | A0 < B0 |
| 20 | 0.5x | 1045.867 | 1058.218 | A0 < B0 |
| 20 | 1.0x | 1095.596 | 1115.690 | A0 < B0 |
| 20 | 2.0x | 1213.410 | 1236.408 | A0 < B0 |

#### QWEN3_MEDIUM_REFERENCE

| H ms | X multiplier | A0 p95 | B0 p95 | Relation |
| ---: | ---: | ---: | ---: | --- |
| 1 | 0.5x | 1774.202 | 1779.716 | A0 < B0 |
| 1 | 1.0x | 1813.879 | 1833.564 | A0 < B0 |
| 1 | 2.0x | 1909.247 | 1946.429 | A0 < B0 |
| 5 | 0.5x | 1778.202 | 1787.716 | A0 < B0 |
| 5 | 1.0x | 1817.879 | 1841.564 | A0 < B0 |
| 5 | 2.0x | 1913.247 | 1954.429 | A0 < B0 |
| 20 | 0.5x | 1793.202 | 1817.716 | A0 < B0 |
| 20 | 1.0x | 1832.879 | 1871.564 | A0 < B0 |
| 20 | 2.0x | 1928.247 | 1984.429 | A0 < B0 |

#### QWEN3_30B_A3B_REFERENCE

| H ms | X multiplier | A0 p95 | B0 p95 | Relation |
| ---: | ---: | ---: | ---: | --- |
| 1 | 0.5x | 1309.633 | 1304.820 | B0 < A0 |
| 1 | 1.0x | 1354.001 | 1360.459 | A0 < B0 |
| 1 | 2.0x | 1460.524 | 1477.026 | A0 < B0 |
| 5 | 0.5x | 1313.633 | 1312.820 | B0 < A0 |
| 5 | 1.0x | 1358.001 | 1368.459 | A0 < B0 |
| 5 | 2.0x | 1464.524 | 1485.026 | A0 < B0 |
| 20 | 0.5x | 1328.633 | 1342.820 | A0 < B0 |
| 20 | 1.0x | 1373.001 | 1398.459 | A0 < B0 |
| 20 | 2.0x | 1479.524 | 1515.026 | A0 < B0 |

The Stage-1 relation is reference-regime dependent: A0/B0 crossovers occur in the small and 30B-A3B H/X grids, while A0 has the lower pooled p95 in all nine medium-profile cells. This does not establish a universal winner.

## 5. Stage 2 results — C vs B1 vs D1

F1 is the primary Stage-2 evidence because it directly exercises the bounded capability accommodation. F2/F3 are retained as secondary non-local diagnostics.

### QWEN3_SMALL_REFERENCE

| Candidate | F1 p50 | F1 p95 | F1 path | Model generations | H |
| --- | ---: | ---: | --- | ---: | ---: |
| C | 665.341 | 717.859 | VIA_FAST | 1 | 0 |
| B1 | 776.996 | 844.278 | ARGO_PRIMARY | 1 | 1 |
| D1 | 770.205 | 832.320 | VIA_FAST | 2 | 0 |

### QWEN3_MEDIUM_REFERENCE

| Candidate | F1 p50 | F1 p95 | F1 path | Model generations | H |
| --- | ---: | ---: | --- | ---: | ---: |
| C | 883.838 | 983.700 | VIA_FAST | 1 | 0 |
| B1 | 1184.029 | 1344.232 | ARGO_PRIMARY | 1 | 1 |
| D1 | 1145.720 | 1270.289 | VIA_FAST | 2 | 0 |

### QWEN3_30B_A3B_REFERENCE

| Candidate | F1 p50 | F1 p95 | F1 path | Model generations | H |
| --- | ---: | ---: | --- | ---: | ---: |
| C | 748.544 | 817.580 | VIA_FAST | 1 | 0 |
| B1 | 929.645 | 1031.374 | ARGO_PRIMARY | 1 | 1 |
| D1 | 913.594 | 1000.098 | VIA_FAST | 2 | 0 |

C performs semantic interpretation followed by deterministic Fast eligibility, then VIA-local execution. B1 performs one combined generation after crossing into ARGO and executes the bounded capability inside ARGO's existing authority. D1 performs interpretation plus a semantic Execution Path Selector generation before VIA Fast execution.

Across all three reference profiles, the F1 relation is **C < D1 < B1** at H = 5 ms and X = 1x. C's advantage over D1 is causally consistent with responsibility decomposition: both share V/X and have no H, but D1 adds `EXECUTION_ROUTE_DECISION`.

### Stage-2 H/X sensitivity

- C and D1 have zero F1 handoffs; B1 has one. At X = 1x, moving H from 5 ms to 1/20 ms shifts B1 F1 p95 by exactly -4/+15 ms and leaves C/D1 unchanged. This range does not close the observed D1-to-B1 reference gaps.
- The same keyed X sample and multiplier are applied to C, B1, and D1. X therefore creates no candidate-specific discount and cancels from paired per-realization differences.
- C versus D1 has no H/X crossover under the trace contract: D1 always contains the additional semantic route-decision generation. This is a statement about the frozen design-time composition, not production performance.

## 6. Stage 2 secondary diagnostics

| Model profile | Candidate | F2 p95 | F3 p95 | Pooled p95 |
| --- | --- | ---: | ---: | ---: |
| QWEN3_SMALL_REFERENCE | C | 1047.033 | 1110.412 | 1079.661 |
| QWEN3_SMALL_REFERENCE | B1 | 865.137 | 1120.124 | 1085.690 |
| QWEN3_SMALL_REFERENCE | D1 | 1047.033 | 1110.412 | 1079.661 |
| QWEN3_MEDIUM_REFERENCE | C | 1797.682 | 1853.757 | 1808.912 |
| QWEN3_MEDIUM_REFERENCE | B1 | 1367.859 | 1917.442 | 1841.564 |
| QWEN3_MEDIUM_REFERENCE | D1 | 1797.682 | 1853.757 | 1808.912 |
| QWEN3_30B_A3B_REFERENCE | C | 1334.321 | 1389.222 | 1353.564 |
| QWEN3_30B_A3B_REFERENCE | B1 | 1053.897 | 1416.188 | 1368.459 |
| QWEN3_30B_A3B_REFERENCE | D1 | 1334.321 | 1389.222 | 1353.564 |

C and D1 tie on pooled p95 because their common F2/F3 paths occupy the tail. That tie is not evidence that their bounded-capability designs have equivalent latency: their F1 p95 differs materially in every model profile. Pooled Fast-task p95 can mask a defining-path difference, so Stage 2 requires F1 and path-level reporting.

## 7. Cross-stage aggregate diagnostic

The original balanced A/B/C/D view is retained below only as a secondary diagnostic. It mixes the Stage-1 authority boundary with the Stage-2 bounded-capability question and must not be used as the primary architecture-selection basis.

### QWEN3_SMALL_REFERENCE

| Existing candidate | p50 | p95 | p99 | F1 p95 | F2 p95 | F3 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A | 982.337 | 1080.596 | 1124.676 | 1026.639 | 1047.033 | 1110.412 |
| B | 815.939 | 1085.690 | 1133.530 | 844.278 | 865.137 | 1120.124 |
| C | 966.942 | 1079.661 | 1124.150 | 717.859 | 1047.033 | 1110.412 |
| D | 966.942 | 1079.661 | 1124.150 | 832.320 | 1047.033 | 1110.412 |

### QWEN3_MEDIUM_REFERENCE

| Existing candidate | p50 | p95 | p99 | F1 p95 | F2 p95 | F3 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A | 1642.131 | 1817.879 | 1898.204 | 1773.577 | 1797.682 | 1853.757 |
| B | 1258.024 | 1841.564 | 1944.493 | 1344.232 | 1367.859 | 1917.442 |
| C | 1595.091 | 1808.912 | 1892.094 | 983.700 | 1797.682 | 1853.757 |
| D | 1595.091 | 1808.912 | 1892.094 | 1270.289 | 1797.682 | 1853.757 |

### QWEN3_30B_A3B_REFERENCE

| Existing candidate | p50 | p95 | p99 | F1 p95 | F2 p95 | F3 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A | 1233.540 | 1358.001 | 1416.887 | 1312.177 | 1334.321 | 1389.222 |
| B | 980.651 | 1368.459 | 1437.265 | 1031.374 | 1053.897 | 1416.188 |
| C | 1207.176 | 1353.564 | 1414.158 | 817.580 | 1334.321 | 1389.222 |
| D | 1207.176 | 1353.564 | 1414.158 | 1000.098 | 1334.321 | 1389.222 |

## Defining-path coverage

| Existing candidate:path | Semantic scenarios | Gate |
| --- | ---: | --- |
| A:GENERAL_AGENT | 24 | >= 4 — PASS |
| A:SPECIALIST_DIRECT | 12 | >= 4 — PASS |
| B:ARGO_PRIMARY | 24 | >= 4 — PASS |
| B:ARGO_TO_SPECIALIST | 12 | >= 4 — PASS |
| C:GENERAL_AGENT | 12 | >= 4 — PASS |
| C:SPECIALIST_DIRECT | 12 | >= 4 — PASS |
| C:VIA_FAST | 12 | >= 8 — PASS |
| D:ARGO_PRIMARY | 12 | >= 4 — PASS |
| D:SPECIALIST_DIRECT | 12 | >= 4 — PASS |
| D:VIA_FAST | 12 | >= 8 — PASS |

## 8. Architectural interpretation

A0 and B0 are meaningfully different primary execution-boundary architectures. Their latency relation depends on the reference model and H/X regime. C, B1, and D1 are three structurally different accommodations of bounded capability: new deterministic VIA-local authority, capability retained inside ARGO authority, and unified semantic topology selection, respectively.

Latency evidence does not decide Agent neutrality, coupling, correctness, flexibility, lifecycle complexity, or product priority. Those remain separate dimensions; QA-02/03/04 may raise follow-up structural questions but are not modified here.

## 9. Provenance and limitations

- **P1 Published reference:** Qwen official SGLang BF16 batch-1 speeds (input length 1, 2,048 generated tokens) are 227.80 tok/s for Qwen3-1.7B, 81.73 tok/s for Qwen3-8B, and 137.18 tok/s for Qwen3-30B-A3B. Qwen defines this as aggregate prompt-plus-generation throughput, so the simulator uses its reciprocal only as a first-order TPOT approximation.
- **P1 Public API default:** V = 500 ms is the OpenAI Realtime `server_vad` default `silence_duration_ms`, used as an Acoustic-EOS/end-of-speech detection reference—not as measured OpenAI latency.
- **P2 Repository structure:** model-operation and handoff counts come from the frozen DP-00 executable architecture responsibilities and the committed trace contract.
- **P3 Engineering assumptions:** TTFT p50/p95, 25% TPOT p95 spread, output-token budgets (24/16/32/48), H = 5 ms, and X = 20/50/100 ms with their stated distributions.
- Model latency is `TTFT + (output_tokens - 1) × TPOT`; log-normal parameters are fit from stated synthetic p50/p95 values. Seed: 20260914.

This report makes conditional design-time comparisons only. It does not measure deployed VIA, real API latency, a production workload mix, production p95, or SLO compliance. The three Qwen profiles are plausible family reference profiles, not a monotonic size/latency ladder; the 30B-A3B model is MoE. QA-02/03/04 populations, oracles, qualification, and route-commit semantics are unchanged. Historical Profile Z and R1–R4 evidence remains valid for its original fixed-delay experiment and has not been rewritten.

No global A/B/C/D, Stage-1, or Stage-2 winner is selected. No weighted score is introduced.
