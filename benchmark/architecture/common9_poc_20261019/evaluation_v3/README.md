# COMMON9 evaluator correction v3

Status: implementation draft;75finite checks pass (14approved addendum cases plus61additional cases). Final corrected-candidate trace integration and independent review remain pending. No paid calls are made by this package. The earlier49-check source is preserved separately under `development/evaluator-v3-draft-04/source/`; the73-check source is preserved under `development/evaluator-v3-draft-05/source/`.

This separate package preserves `../evaluate.py`, `../causal_timing.py` and `../analyze.py`. It imports the original evaluator for unchanged exact-set matching, DAG construction, all-call cost and complete-trace RSS. Every result records the imported evaluator SHA. No source text, model output, oracle, timestamp or raw trace is changed.

The first-endpoint correction implements the root-approved `common9-newer-completion-endpoint-addendum-20261019.md` (SHA `fce69c1b86cd8a7002a77b07265b6b7d83660b2135ff42d46f5dcc8af27d950f`). The three frozen BG obligations use actual captured RESULT provenance and earliest-union selection. Ambiguous earlier candidates retain possible/certain bounds and null exact latency.

Contract19 §5 observation correction treats `observed_ns <= observation_deadline_ns` as the screen/goal endpoint boundary. State effects instead need actual commit `linearization_ns` strictly before their applicable deadline. A unique actual semantic-adoption ledger joins QA02 by actual stimulus ID; missing or ambiguous proof cannot become a successful adoption. Full DAG ancestry stays intact; only late terminal eligibility is excluded. No wall-duration subtraction is introduced.

Source `source_ns` remains the original QA latency root. Actual provision/send brackets independently prove availability and the final observation deadline. Final cutoff must follow actual last provision by60seconds, followed by120seconds of settlement. An unarmed earlier effect needs explicit control-order proof and must still precede final cutoff. This ABI audit is being finalized with the runtime lead.

Run from the parent common9 directory:

```sh
python3 -m evaluation_v3.checks
python3 -m evaluation_v3.run_checks --output /absolute/new-check-evidence
python3 -m evaluation_v3.audit_traces /absolute/runs.jsonl --output /absolute/new-audit
python3 -m evaluation_v3.analyze /absolute/runs.jsonl --output /absolute/new-output --repetitions 5
```

`--mode legacy_diagnostic` can show endpoint-only differences on preserved old raw data. It never certifies a primary episode. Old formal-v2 did not execute the required cancellation; its counterfactual cost, RSS and state effects cannot be reconstructed. Keep its nine executed rows and306unexecuted identities distinct from a fresh corrected campaign. Original and corrected endpoint/goal/QA02 selections are reported together; planned215/75/115 denominators do not change.

The JSON retains raw publication counts, callback rejection reasons, unresolved boundaries, state-effect checks, late endpoints, actual all-call usage and RSS validity. A numeric partial mean is not a whole-population speed or a semantic success claim. QA04 remains an independent exact-baseline exercise.

Primary QA01/03 aggregates admit only technically certified observations while preserving the215/115planned denominators. QA02 retains75planned judgments and explicitly separates technical-unjudgeable from semantic-incorrect; a fully certified accuracy is null when judgments/population are incomplete. Ordinary model failures under valid instrumentation remain actual primary outcomes. All QA05 spend/unknown reservations remain in the full ledger; uncertified execution has no comparable primary bundle mean. Incomplete QA06 observations remain raw sampled lower-bound diagnostics. `raw_observation_diagnostics` retains the former unfiltered numbers rather than silently deleting them.

Pending integration: final mutation-point source review belongs to the independently assigned Astra reviewer; corrected-candidate full9/63trajectories, clock/control boundary regressions, frozen source/binary mapping and fresh QA04 remain separate. The intermediate `AAAA-S02-r1-665b61` sample proves ABI usability and asserted-ledger consistency, not the final corrected mutation implementation or performance.

The receipt ABI uses unique `publication_id`/display event ID, explicit `PUBLICATION_DISPATCH_ADMITTED`, host receipt/send brackets, candidate `PUBLICATION_HOST_RECEIPT`, and subsequent `PUBLICATION_DELIVERY_STATE`. The latter explicitly distinguishes `historical_display_recorded:true` from `complete:true`. QA03 requires receipt-linked current completion plus `GOAL_COMPLETE.goal_complete:true`; historical display alone cannot prove completion or authorize a new delivered source. Raw display observed before cutoff remains its QA01 fact even when ACK is missing, cancelled or late. Receipt wait stays a measured VIA DAG operation. Duplicate/unknown accepted receipts are not silently certified; explicit safe rejection is retained as a diagnostic.

The existing `ACTUAL_AGENT_FACT` branch has a distinct factual Task-state effect: actual display phase, uniquely captured Agent event class/Task, and its specific receipt prove that state update. It does not append user-goal delivery history or complete a user goal. Its effect neither requires nor invents `historical_display_recorded`, and cannot prove GOAL completion or a completion-required delivered-source registration. This preserves that branch's existing meaning. Missing initial clock/ACK proof may leave legitimate earlier work unarmed; before a recorded technical abort it remains technical-unknown, not an asserted late state effect. The episode is still NOT_CERTIFIED.

Technical abort retains contract/proposed/last-accepted deadline aliases, actual cancellation, pending earlier cutoffs, effective admission stop and settlement anchor. State/dispatch must precede effective stop; settlement is anchored there+120seconds without a later reset. Such an episode requires campaign STOP and remains NOT_CERTIFIED, with all raw observations, spend and reservations preserved. The observation contract deadline itself is not overwritten by abort.

This draft is not ready to freeze until the lead's stable actual63 trajectories match the finalized ABI, the independent Astra review closes, and the exact source/binary/QA04 mappings are recorded. Intermediate39a5a6 demonstrates receipt-ID usability but predates the final history/completion and explicit abort-alias fields; it is not the final certified integration proof.
