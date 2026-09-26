# VIA-DP-11 evaluation v4

- A: VIA-owned Agent client in an isolated supervised worker process
- B: the same client in the VIA Core process
- Evidence: `MEASURED_REFERENCE_HARNESS`; reference harness, not product E2E

## 19-QA complete table

| QA | A isolated worker | B same process |
| --- | ---: | ---: |
| QA-01 | BLOCKED | BLOCKED |
| QA-02 | N/A | N/A |
| QA-03 | BLOCKED | BLOCKED |
| QA-04 | N/A | N/A |
| QA-05 | BLOCKED | BLOCKED |
| QA-11 | BLOCKED | BLOCKED |
| QA-12 | N/A | N/A |
| QA-13 | BLOCKED | BLOCKED |
| QA-14 | BLOCKED | BLOCKED |
| QA-15 | N/A | N/A |
| QA-21 | BLOCKED | BLOCKED |
| QA-22 | N/A | N/A |
| QA-23 | BLOCKED | BLOCKED |
| QA-31 | 11.5 ms | 11.3 ms |
| QA-32 | 0 units | 4 units |
| QA-41 | 7813.2 MiB | 7809.5 MiB |
| QA-51 | N/A | N/A |
| QA-61 | 100.0% | 100.0% |
| QA-62 | 100.0% | 100.0% |

## Interpretation

This campaign scores only endpoints that the executable reference path actually implements. The older renderer, correctness and design-ledger proxies remain in raw diagnostics but are not promoted to active QA values.

## N/A and blocked audit

| QA | Disposition | Frozen reason |
| --- | --- | --- |
| QA-01 | BLOCKED | `BLOCKED_RENDERER_PROXY_IS_NOT_THE_REQUIRED_MEANINGFUL_AUDIBLE_ENDPOINT` |
| QA-02 | N/A | `N/A_DIRECT_RESPONSE_PATH_DOES_NOT_USE_THE_AGENT_CLIENT_PROCESS_BOUNDARY` |
| QA-03 | BLOCKED | `BLOCKED_RENDERER_PROXY_IS_NOT_THE_REQUIRED_MEANINGFUL_AUDIBLE_STATUS_ENDPOINT` |
| QA-04 | N/A | `N/A_NORMAL_BARGE_IN_PATH_DOES_NOT_USE_THE_AGENT_CLIENT_PROCESS_BOUNDARY` |
| QA-05 | BLOCKED | `BLOCKED_CONTROL_PRESENTATION_PROXY_OMITS_THE_REQUIRED_AUDIBLE_ENDPOINT` |
| QA-11 | BLOCKED | `BLOCKED_EXISTING_RUNTIME_FIXTURE_DOES_NOT_IMPLEMENT_ALL_ACTIVE_INTEGRATED_PREDICATES` |
| QA-12 | N/A | `N/A_DP11_DOES_NOT_CHANGE_THE_SHARED_SEMANTIC_RESOLUTION_PATH` |
| QA-13 | BLOCKED | `BLOCKED_EXISTING_RUNTIME_FIXTURE_IMPLEMENTS_ONLY_A_SUBSET_OF_ACTIVE_BINDING_PREDICATES` |
| QA-14 | BLOCKED | `BLOCKED_EXISTING_RUNTIME_FIXTURE_IMPLEMENTS_ONLY_A_SUBSET_OF_ACTIVE_CONVERGENCE_PREDICATES` |
| QA-15 | N/A | `N/A_NORMAL_CHANNEL_AND_TASK_CONTINUITY_DOES_NOT_USE_THIS_PROCESS_PLACEMENT_AXIS` |
| QA-21 | BLOCKED | `BLOCKED_AGENT_CHANGE_PACK_IS_A_DESIGN_LEDGER_NOT_EXECUTED_SOURCE_CHANGES` |
| QA-22 | N/A | `N/A_MODEL_CONTEXT_AND_STATE_CHANGE_PACK_DOES_NOT_CHANGE_THIS_AGENT_CLIENT_PLACEMENT_AXIS` |
| QA-23 | BLOCKED | `BLOCKED_OBSERVABILITY_CHANGE_PACK_IS_A_DESIGN_LEDGER_NOT_EXECUTED_SOURCE_CHANGES` |
| QA-51 | N/A | `N/A_BOTH_CANDIDATES_REMAIN_INSIDE_THE_SAME_VIA_TRUST_BOUNDARY` |

## Limitations

- This campaign deliberately does not score proxy Voice endpoints as QA-01~05.
- The deterministic Reference Agent validates the VIA boundary, not third-party Agent reasoning quality.
- Semantic, normal-path correctness, design-ledger and exposure records are retained as diagnostics but are not promoted to QA values where the active contract is incomplete or structurally non-applicable.

## Raw evidence

- `raw/trials.jsonl` SHA-256: `cc0c873ef2dba7152258503cfc30643ec1911b997a55d040fd3a2f8b24a4d318`
- Source strata remain separately available under `raw/`.
- `replay-receipt.json` records independent analyzer reproduction.
