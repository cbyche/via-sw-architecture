# VIA-DP-11 Pilot Result

> Evidence: `MEASURED_REFERENCE_HARNESS`
> Contract: `VIA-DP-11-PILOT-v2` / `76e19385228f25a78a866e4fd51ad6b793eb2ac66210222273d6af37b6ec0f42`
> This pilot does not select a winner.

## Result at a glance

| Observation | A: isolated worker | B: same process | Interpretation |
| --- | ---: | ---: | --- |
| Normal bridge lifecycle diagnostic p95 | 0.482 ms | 0.015 ms | Supporting subspan only; not QA-01/03/05 |
| QA-31 pilot: integration-client fatal worst-stratum p95 | 32.686 ms | 33.778 ms | One fault stratum, not final all-fault QA-31 |
| QA-32 excess affected user-visible units | 0 | 4 | Frozen independent-unit registry |
| QA-61 complete trace rate | 100.0% | 100.0% | Package-level rate across both candidates |

## What may be concluded

- The same Reference Agent contract and Task authority ran on both sides; the changed axis was the Agent-client process boundary.
- The normal bridge p95 difference is 0.468 ms. It is real at this subspan but far below the pre-existing 100 ms responsiveness interpretation threshold and is not a user-endpoint QA result.
- The QA-31 pilot difference is 1.092 ms, so this pilot treats the candidates as practically similar for this stratum rather than declaring a recovery winner.
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
- The QA-61 pilot result validates the external evidence envelope; it does not yet validate preservation of worker-local log records immediately before a fatal abort.
