# VIA-DP-06 evaluation v5

- Evidence: `MEASURED_REFERENCE_HARNESS`
- Campaign: `via-dp-06-evaluation-v5-20260927`
- Correctness: 24-case breadth, one deterministic trial per case
- Responsiveness: one representative normal case per QA, 2 warm-ups + 20 scored trials
- Decision: A=integrated authority, B=staged authorities, B′=B plus non-architectural deterministic fast-path tactic
- This is a target-Mac reference harness, not `PRODUCT_E2E`.

## 19-QA complete table

| QA | A | B | B_PRIME |
| --- | --- | --- | --- |
| QA-01 | 23714.1 ms p95 (mean 22076.9, n=20) | 42338.2 ms p95 (mean 40460.9, n=20) | 15320.7 ms p95 (mean 14770.3, n=20) |
| QA-02 | 22380.9 ms p95 (mean 21647.3, n=20) | 43863.1 ms p95 (mean 40234.7, n=20) | 15058.3 ms p95 (mean 14463.2, n=20) |
| QA-03 | N/A | N/A | N/A |
| QA-04 | N/A | N/A | N/A |
| QA-05 | 24026.2 ms p95 (mean 22587.1, n=20) | 15196.8 ms p95 (mean 14444.6, n=20) | 13811.0 ms p95 (mean 13144.1, n=20) |
| QA-11 | 33.3% strict (8/24) | 25.0% strict (6/24) | 25.0% strict (6/24) |
| QA-12 | 33.3% strict / 78.6% field | 25.0% strict / 66.5% field | 25.0% strict / 67.3% field |
| QA-13 | 70.8% strict (17/24) | 54.2% strict (13/24) | 58.3% strict (14/24) |
| QA-14 | N/A | N/A | N/A |
| QA-15 | BLOCKED | BLOCKED | BLOCKED |
| QA-21 | BLOCKED | BLOCKED | BLOCKED |
| QA-22 | BLOCKED | BLOCKED | BLOCKED |
| QA-23 | BLOCKED | BLOCKED | BLOCKED |
| QA-31 | BLOCKED | BLOCKED | BLOCKED |
| QA-32 | N/A | N/A | N/A |
| QA-41 | BLOCKED | BLOCKED | BLOCKED |
| QA-51 | N/A | N/A | N/A |
| QA-61 | 100.0% | 100.0% | 100.0% |
| QA-62 | 100.0% | 100.0% | 100.0% |

## Measurement disposition

`N/A` means the VIA-DP-06 structural axis does not participate. `BLOCKED` means it participates but the complete approved endpoint or pack has not run; no proxy value is substituted.

| QA | Disposition | Frozen reason |
| --- | --- | --- |
| QA-03 | N/A | `N/A_DP06_DOES_NOT_CHANGE_STATUS_RECONNECTION` |
| QA-04 | N/A | `N/A_DP06_DOES_NOT_CHANGE_BARGE_IN_CONTROL` |
| QA-14 | N/A | `N/A_NO_ASYNC_STATE_RACE_IN_THIS_DP` |
| QA-15 | BLOCKED | `BLOCKED_DP06_PARTICIPATES_BUT_NO_CHANNEL_OR_RECONNECT_TRANSITION_IN_FROZEN_CORPUS` |
| QA-21 | BLOCKED | `BLOCKED_FULL_A01_A09_CHANGE_PACK_NOT_EXECUTABLE` |
| QA-22 | BLOCKED | `BLOCKED_FULL_M01_M09_C01_C06_CHANGE_PACK_NOT_EXECUTABLE` |
| QA-23 | BLOCKED | `BLOCKED_FULL_E01_E05_CHANGE_PACK_NOT_EXECUTABLE` |
| QA-31 | BLOCKED | `BLOCKED_NO_FROZEN_SEMANTIC_RETRY_FAULT_REPETITION` |
| QA-32 | N/A | `N/A_NO_FAULT_CONTAINMENT_BOUNDARY_DIFFERENCE` |
| QA-41 | BLOCKED | `BLOCKED_ISOLATED_TARGET_DEVICE_PEAK_MEMORY_P95_NOT_IMPLEMENTED` |
| QA-51 | N/A | `N/A_NO_PROTECTED_INFORMATION_BOUNDARY_DIFFERENCE` |

## Model calls and repair locations

| Candidate | Calls | Mean calls/trial | Mean input tokens | Mean output tokens | Mean call time | Repair calls |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| A | 86 | 1.02 | 918.3 | 148.5 | 16529.9 ms | `{"integrated_repair": 2}` |
| B | 210 | 2.50 | 614.5 | 92.7 | 12015.5 ms | `{"association_repair": 1, "grounding_repair": 8}` |
| B_PRIME | 112 | 1.33 | 666.2 | 94.8 | 9312.8 ms | `{"association_repair": 1, "grounding_repair": 8}` |

## Evidence limits

- Generated Korean request WAV and macOS Yuna plus BlackHole are reference Voice endpoints, not the product S2S path.
- S2S is intentionally not invoked because VIA-DP-06 changes semantic authority after speech interpretation.
- The deterministic external Reference Agent exercises the process/identity contract, not third-party Agent domain quality.
- QA-21~23, QA-31 and QA-41 remain BLOCKED until their full frozen packs run in the integrated path.
