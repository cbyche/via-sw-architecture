# VIA-DP-11 Pilot Result

> Evidence: `MEASURED_REFERENCE_HARNESS`
> Contract: `VIA-DP-11-PILOT-v1` / `fdd622282b2ab04d946217d520864ef792d364652bd519aedfc7e1d9098dab4c`
> This pilot does not select a winner.

## Result at a glance

| Observation | A: isolated worker | B: same process | Interpretation |
| --- | ---: | ---: | --- |
| Normal bridge lifecycle diagnostic p95 | 0.464 ms | 0.015 ms | Supporting subspan only; not QA-01/03/05 |
| QA-31 pilot: integration-client fatal worst-stratum p95 | 32.000 ms | 32.000 ms | One fault stratum, not final all-fault QA-31 |
| QA-32 excess affected user-visible units | 0 | 4 | Frozen independent-unit registry |
| QA-61 complete trace rate | 100.0% | 100.0% | Package-level rate across both candidates |

## What may be concluded

- The same Reference Agent contract and Task authority ran on both sides; the changed axis was the Agent-client process boundary.
- QA-32 is structurally sensitive to this DP if the frozen client-fatal fault is credible for the production dependency.
- QA-31 evidence is provisional because the final metric is the maximum across the complete frozen fault pack.
- Normal bridge timings show the direct cost of IPC, but cannot be substituted for user/acoustic responsiveness metrics.

## Evidence package

- `manifest.json`: target, Git state, binary hashes, exact commands
- `raw/normal-bridge.jsonl`: every scored normal-path diagnostic trial
- `raw/recovery.jsonl`: every recovery trial, including task-count stratum
- `raw/blast-radius.jsonl`: every fatal-fault containment observation
- `derived/summary.json`: deterministic aggregation of raw data

## Limitations

- This is not PRODUCT_E2E evidence.
- QA-31 covers one frozen integration-client-fatal fault stratum, not the final all-fault maximum.
- The normal bridge timings are supporting diagnostics and must not be reported as QA-01, QA-03, or QA-05.
- QA-32 applies only to the frozen independent-unit registry in this contract.
