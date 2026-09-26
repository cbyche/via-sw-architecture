# VIA-DP-06 evaluation v4

- Evidence: `MEASURED_REFERENCE_HARNESS`
- Campaign: `dp06-evaluation-v4-20260927`
- Decision: A=integrated authority, B=staged authorities, B′=B plus deterministic fast-path tactic
- Important: this is a structural reference-harness result, not `PRODUCT_E2E` and not a product S2S latency claim.

## 19-QA complete table

| QA | A | B | B_PRIME |
| --- | --- | --- | --- |
| QA-01 | 17778.4 ms (mean 13398.1) | 27718.4 ms (mean 22412.0) | 24335.6 ms (mean 15270.2) |
| QA-02 | 19702.4 ms (mean 10511.9) | 25534.8 ms (mean 17603.4) | 15867.3 ms (mean 11505.7) |
| QA-03 | N/A | N/A | N/A |
| QA-04 | N/A | N/A | N/A |
| QA-05 | 38411.0 ms (mean 16147.5) | 27499.0 ms (mean 16015.1) | 24289.6 ms (mean 14559.7) |
| QA-11 | 33.3% strict | 25.0% strict | 25.0% strict |
| QA-12 | 33.3% strict / 78.6% field | 25.0% strict / 66.5% field | 25.0% strict / 67.3% field |
| QA-13 | 70.8% strict | 54.2% strict | 58.3% strict |
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

## N/A and blocked audit

`N/A` means the DP-06 A/B structural difference does not physically participate in the frozen path. `BLOCKED` means it does participate, but this campaign lacks the approved endpoint, complete change pack, repetition, or isolation needed for a valid value.

| QA | Disposition | Frozen reason |
| --- | --- | --- |
| QA-03 | N/A | `N/A_DP06_DOES_NOT_CHANGE_STATUS_RECONNECTION` |
| QA-04 | N/A | `N/A_DP06_DOES_NOT_CHANGE_BARGE_IN_CONTROL` |
| QA-14 | N/A | `N/A_NO_ASYNC_STATE_RACE_IN_THIS_DP` |
| QA-15 | BLOCKED | `BLOCKED_DP06_PARTICIPATES_IN_FOLLOW_UP_SEMANTICS_BUT_THE_FROZEN_24_CASE_CORPUS_HAS_NO_CHANNEL_OR_RECONNECT_TRANSITION` |
| QA-21 | BLOCKED | `BLOCKED_DP06_PARTICIPATES_IN_CAPABILITY_SEMANTICS_BUT_THE_FULL_A01_A09_CHANGE_PACK_IS_NOT_EXECUTABLE` |
| QA-22 | BLOCKED | `BLOCKED_ONLY_TWO_DIAGNOSTIC_EXERCISES_EXIST_NOT_THE_REQUIRED_M01_M09_C01_C06_CLOSED_PACK` |
| QA-23 | BLOCKED | `BLOCKED_ONLY_TWO_DIAGNOSTIC_EXERCISES_EXIST_NOT_THE_REQUIRED_E01_E05_CLOSED_PACK` |
| QA-31 | BLOCKED | `BLOCKED_SEMANTIC_CALL_GRAPH_PARTICIPATES_IN_RETRY_RECOVERY_BUT_NO_FROZEN_FAULT_REPETITION_EXISTS` |
| QA-32 | N/A | `N/A_NO_FAULT_CONTAINMENT_BOUNDARY_DIFFERENCE` |
| QA-41 | BLOCKED | `BLOCKED_CALL_GRAPH_AND_BUFFERS_PARTICIPATE_BUT_ISOLATED_TARGET_DEVICE_PEAK_MEMORY_P95_IS_NOT_IMPLEMENTED` |
| QA-51 | N/A | `N/A_NO_PROTECTED_INFORMATION_BOUNDARY_DIFFERENCE` |

## Latency interpretation

Correctness failure does not replace an observed response time. A timeout appears only when the Architecture failed to produce the physical input/meaningful-audio endpoint. For an expected delegated case that incorrectly took no Agent path, both Agent interval events are absent; the actual input-to-audible-response wall time is retained while correctness remains failed. With one scored trial per case, the primary value is the maximum observed frozen-case latency; the mean is auxiliary.

| Candidate | QA | Applicable | Physical endpoint failures | Correct responses | Full wall-clock mean | Full wall-clock max |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| A | QA-01 | 10 | 0 | 1 | 13402.9 ms | 17820.5 ms |
| A | QA-02 | 8 | 0 | 3 | 10511.9 ms | 19702.4 ms |
| A | QA-05 | 6 | 0 | 4 | 16147.5 ms | 38411.0 ms |
| B | QA-01 | 10 | 0 | 0 | 22419.6 ms | 27718.7 ms |
| B | QA-02 | 8 | 0 | 2 | 17603.4 ms | 25534.8 ms |
| B | QA-05 | 6 | 0 | 4 | 16015.1 ms | 27499.0 ms |
| B_PRIME | QA-01 | 10 | 0 | 0 | 15278.0 ms | 24372.3 ms |
| B_PRIME | QA-02 | 8 | 0 | 2 | 11505.7 ms | 15867.3 ms |
| B_PRIME | QA-05 | 6 | 0 | 4 | 14559.7 ms | 24289.6 ms |

Wrong-route QA-01 cases with an observed audible response use actual wall time rather than a synthetic timeout. Counts are preserved in `summary.json` as `wrong_route_actual_time_count`.

## Correctness interpretation

QA-11~13 primary values are strict whole-case success. QA-12 additionally reports field-level correctness, and every field observation remains in `raw/trials.jsonl`.

## Model-call evidence

| Candidate | Calls | Mean calls/trial | Mean input tokens/call | Mean output tokens/call | Mean call time | Repair locations |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| A | 26 | 1.08 | 943.7 | 155.0 | 8965.6 ms | `{"integrated_repair": 2}` |
| B | 70 | 2.92 | 653.9 | 98.3 | 5429.3 ms | `{"association_repair": 1, "grounding_repair": 8}` |
| B_PRIME | 52 | 2.17 | 673.0 | 100.0 | 4832.3 ms | `{"association_repair": 1, "grounding_repair": 8}` |

## Evidence limitations

- One scored trial per frozen case: primary latency is maximum observed case latency, not production p95.
- Request WAV text annotations stand in for a common upstream speech interpretation; this campaign does not measure S2S recognition.
- macOS Yuna plus BlackHole is a common reference Voice renderer, not the product S2S output path.
- The external deterministic Reference Agent exercises the boundary and identity contract, not third-party Agent domain quality.

## Raw evidence

- `raw/trials.jsonl` SHA-256: `3fe50edc9cc1e8e91d28d861186859850a140881939c51b762e3b9fb2d4edbf3`
- `raw/change-exercises.json`: executed change observations
- `replay-receipt.json`: independent analyzer-process digest check
- `artifacts/`: per-trial input/output loopback reports and WAVs
