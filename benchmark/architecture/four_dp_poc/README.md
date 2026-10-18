# Four-DP numerical PoC

User-authorized 2026-10-18 numerical implementation. Operational alternatives are Rust, the cloud bridge/evaluator/replay UI are Python. Actual Qwen3.7-Plus snapshot calls, thinking off; no voice model. No Git publication.

## Run

From repository root:

```sh
cargo test --offline --manifest-path prototypes/candidates/four_dp_poc/Cargo.toml
python3 benchmark/architecture/four_dp_poc/analyze.py results/architecture-evaluation/current/four-dp-qwen37-20261018
python3 benchmark/architecture/four_dp_poc/demo.py
```

Actual calls need the existing macOS Keychain services `via-dashscope-api-key` and `via-dashscope-workspace-id`; no values are printed or passed in process arguments. Sandbox access/network may require escalated execution. All preparation, basic, tactic and failed requests share the single `budget.json` $5 cap. Unknown usage retains a conservative reservation. No other model or judge is used.

## Campaign provenance and derived outputs

The delivered BASIC campaign uses the original frozen crate for DP01/02/04 (84 trials), then the separately frozen DP05 receiver v2 for 36 trials. DP05 v2 was authorized before any measured DP05 trial: the common receiver can complete from actual valid supplied evidence or request original reads. Its source, reason, guards, binary and hashes are preserved under the campaign `basic-dp05-v2/` directory and `prototypes/candidates/four_dp_poc/variants/`. The original source and binary remain unchanged.

Each trial has a fresh Python bridge and Rust process; `campaign.py` and `dp05_v2_campaign.py` serialize episodes while retaining structural model concurrency. The existing campaign is evidence and must not be overwritten or resumed with a generic all-options command. A new authorized campaign needs a new result directory and freeze. The global ledger includes access and smoke calls.

After all 120 BASIC rows, `tactics.py` reads an explicit selected plan, runs only its frozen affected cases, and records B4 under DIAGNOSTIC. `compare_tactics.py` compares each with its own three BASIC repetitions. No tactic is compared to a whole-DP basic average.

```sh
python3 benchmark/architecture/four_dp_poc/case_report.py results/architecture-evaluation/current/four-dp-qwen37-20261018
python3 benchmark/architecture/four_dp_poc/preparation_report.py results/architecture-evaluation/current/four-dp-qwen37-20261018
python3 benchmark/architecture/four_dp_poc/failure_report.py results/architecture-evaluation/current/four-dp-qwen37-20261018
python3 benchmark/architecture/four_dp_poc/compare_tactics.py results/architecture-evaluation/current/four-dp-qwen37-20261018
python3 benchmark/architecture/four_dp_poc/export_demo.py results/architecture-evaluation/current/four-dp-qwen37-20261018
```

The exported `evidence-browser.html` is a standalone recorded replay with actual input PNGs, event payloads and model call timing/usage. It does not need a server. Local `demo.py` serves the same read-only replay.

## Frozen execution contract

- All 10 basics: 01ABC, 02AB, 04AB, 05ABC. Four authored episodes per DP, three repetitions = 120 basic episode runs. AB/BA/AB or ABC/BCA/CAB within each paired episode. One episode at a time; concurrency only inside its structure under K.
- Candidate fixtures, raw sources, screenshots, and event ledgers are author-created test inputs, not user recordings. Oracles are in separate evaluator-only files and never passed to Rust/model.
- `STIMULUS` marks each eligible input, `DISPLAY_PROXY` is the actual local output callback event received by the Python observer, `GOAL_COMPLETE` marks detailed response readiness. No physical GUI timing claim. Replay UI displays the actual recorded text and traces.
- QA01 = observer receipt of first nonempty substantive DISPLAY_PROXY minus STIMULUS. Wrong answer times remain; wrong referents/relations/contracts are separate. Missing expected stimuli/outputs are counted even when an episode fails early, without fabricating timeout values.
- QA02 = exact target-set at first adoption per expression + revision, with required source versions when specified. Internal candidates do not count. One expression is one denominator item even when its set contains multiple targets. DP01/04 only.
- QA03 = eligible stimulus to detailed GOAL_COMPLETE elapsed, subtracting only unioned actual external receiver-model and user-wait intervals intersecting that causal window. A user-wait interval occupied by VIA queued/active processing is not excluded. VIA model, queue, read and serving remain. Voice OFF; no artificial ASR/TTS constants. Goal-by-goal samples retained. For these fixtures all included intermediate handling is in the same request chain; no generic cross-DP timing inference.
- QA04 = actual operational Rust modules changed for independent C01 provider wire/usage and C03 source coordinate/time/revision changes, compile/probe evidence and diffs in `qa04/`. Scope is this small PoC; no estimated module fragmentation or product score.
- QA05 = provider-reported actual input/output usage at fixed Singapore <=256K standard price $0.40/$1.60 per million. All production/unused/preparation/failed calls retained. VIA and RECEIVER separately. Access/smoke overhead is campaign overhead, not silently assigned to an option. This is calculated standard-price cost, not a reconciled invoice or COMMON-28 extrapolation.
- QA06 = maximum simultaneous sampled Rust+Python RSS sum in decimal MB over actual execution, 100ms nominal period. Raw per-PID samples retained. Shared-page deduplication is unverified, so sum is an estimate. Browser UI and cloud weights excluded. No whole-VIA or Windows 512MB qualification.
- N-R input relation/request/question association and N-H actual handoff contract consistency are diagnostics; neither becomes QA02.
- Authored DP02 input arrival120ms and question-reply30ms are test user-event timing, not simulated model/service latency. Actual prior call readiness, unresolved-input publication gates and stale-result rejection are recorded. A reuses retained conversation draft/evidence at a step boundary; B retains request-owned draft independently from common state.
- All raw requests/responses, source/fixture hashes, event and usage records, failures, per-run CSV/JSON and replay remain preserved. Findings are finite-fixture observations, not statistical population proof or ADR adoption.
