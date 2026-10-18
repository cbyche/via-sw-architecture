# Four-DP review-v2 numerical PoC

This generation addresses the user's review in `13-poc-user-review-and-remeasurement.md` and
implements the new execution contract in `14-corrected-poc-contract.md`. The earlier source and
`four-dp-qwen37-20261018` evidence remain unchanged.

## Scope

- Ten Rust basics: 01 A/B/C, 02 A/B, 04 A/B, 05 A/B/C, four finite authored scenarios, three repeats.
- Actual Qwen `qwen3.7-plus-2026-05-26`, thinking false; no audio models or local Qwen weights.
- Concrete adopted-handling callback and final detailed-result callback are separate when actual
  execution has separate stages. Grounding/direct complete answers can legitimately coincide.
- Goal links are semantic outputs. Evaluator-only `goal_manifests` preserve response/completion
  coverage even on crashes or missing adoption. Corrections retain original goal origin/revision.
- Null/shape/source/geometry failures get at most one owner repair per stage. Every attempt stays
  in raw evidence and cost. Missing completion means INCOMPLETE, never a fabricated timeout value.
- Fresh Python+Rust trial, registered PID/role/PPID, simultaneous RSS samples and logical object
  byte lifetimes. This is a macOS backend RSS proxy, not whole-VIA Windows/512MB qualification.
- One shared campaign-root USD4 budget ledger across all phase directories; receiver and unused
  preparation calls are included. Secrets are loaded only in process and never logged.

## Files and owners

`CONTRACT.md` describes API/events. `run.py`, `bridge.py`, `memory.py`, `timing.py` and `analyze.py`
are common. Rust owns alternative state and execution; Python provides the actual model bridge,
measurement and analysis. `offline_dpNN.py` are deterministic contract doubles only.

`prototypes/candidates/four_dp_poc_v2/src/dp02/` separates adopted state, semantic interpretation,
detailed response checks and execution because these are actual different state/prompt/lifecycle
responsibilities. It is not a virtual module-count estimate. QA04 uses an independently frozen
module registry and actual scratch changes, not line count or invented components.

## Before provider calls

1. Read the repository AGENTS contract and current docs 13/14.
2. Run focused Rust and Python checks and deterministic pipeline probes.
3. Create a new immutable phase directory with `--freeze-only`. It snapshots source/harness/
   fixtures/oracles and the built Rust binary. Existing freeze overwrite is rejected.
4. Parent authorization creates `MODEL-CALLS-AUTHORIZED.json` for that exact freeze, phase and job
   list and identifies the shared campaign_root ledger. No marker means no actual model call.
5. Every provider trial verifies frozen source hashes and pre/post executable hashes. `run.py`
   launches a fresh Python process for every trial when more than one job is requested.

Example deterministic check (zero provider calls):

```sh
cargo test --offline --manifest-path prototypes/candidates/four_dp_poc_v2/Cargo.toml
python3 benchmark/architecture/four_dp_poc_v2/check_analysis.py
python3 benchmark/architecture/four_dp_poc_v2/run.py --offline --dp 02 --reps 1 --offline-probe null-once --results results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/offline-dp02-check
```

`ps` access and actual Keychain/network access may require the execution permission used by the
coding agent. Deterministic output/timing is explicitly OFFLINE and is never provider performance.

## Measurement interpretation

QA01 uses the first substantive DISPLAY_PROXY per registered input, including meaningful wrong
content; empty/non-response is NOT_OBSERVED. QA03 uses registered user goals and latest required
revision; Agent notices are QA01 events without automatically becoming new QA03 goals. An Agent
question answer-delivery subgoal is labelled as that subgoal, not the original whole email Task.

The model-backed receiver is a logical bounded shim. Its remote model dependency is excluded only
within that explicitly scoped receiver-shim profile; local dispatch/receipt/source-serving remains
included. Full elapsed and excluded intervals accompany the API-boundary proxy. Pure remote compute
versus network is not observable and no invented split is reported. Without external work the
actual goal elapsed is used; sparse model parents are not claimed as a reconstructed complete DAG.

All costs are provider token usage times the frozen comparison prices ($0.40/M input, $1.60/M
output), not a billing invoice. Report all repetitions, errors, conditional means and coverage.
QA02 remains exact first referent set/version for 01/04. N-R, N-H and bounded content diagnostics
are separate from QA02. No course-wide QA coverage or population proof follows from these fixtures.
