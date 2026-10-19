# Quantitative adapter stage-01: bounded source review

2026-10-19. Independent reviewer: GPT-6 Astra Ultra.

## Scope and decision

Read only immutable stage-01 copies and the relevant frozen output definitions. No test, adapter, evaluator, analyzer, corpus extraction, model, parser, candidate, build or API was run. The checkpoint and source hashes match:

- Checkpoint: `326ec0ac349b09dd8411bfe962836fbe7b769890361b8534e1b28fd1f2edfc1d`.
- `report_adapter.py`: `c443262805ca6dc8fd58862c0a52a986a8cd92d018eed1f5d13ef8d9ab47e702`.
- `test_report_adapter.py`: `731edd2fb36f630631d8bf7a805b7d44b27c8621938821fa2f3c0dc76b8ee16d`.
- Saved synthetic result: `ca45dd88d4eb5e38ad5e268981118f8d57637367c4fba77dd986f4a4a06dec1b`, reporting seven passing checks in 0.0016997919883579016 seconds. Read, not rerun.

**PARTIAL REVIEW: the reviewed core needs the bounded repairs below. No actual-report acceptance.** Missing financial, pairing, loader and rendering integration is explicitly disclosed by the checkpoint and is not treated as an unexpected defect. `build_report` must not execute at this stage. No finding concerns the running measurement or requires modifying a frozen artifact.

## Findings

### A01 — Result-body identity and obligation declarations are not fully tied to the associated record

In the copied `report_adapter.py:398–408`, association validation checks wrapper identity, file references, oracle and source hashes. `:249–252` checks the result's configuration/case/repetition/phase/version/mode but not its actual `run_id` or a present `population`. Therefore a body naming run B can be referenced through wrapper/trace A and still reach certified state in this core. File hashes prove bytes, not that the body's identity agrees with the association.

Likewise `:269–287` joins obligation IDs but does not compare the copied frozen declaration fields. The frozen evaluators copy each original declaration into their result rows: response event/required content, binding expression/expected set, and goal start/member events/completion. A row with the right ID and a contradictory event or expected set must not be reported under the correct declaration merely because its ID matches.

Minimal repair: require actual result `run_id` and all present population/identity fields to agree with the proven wrapper/trace/frozen job before certification. Require copied declaration fields to equal the exact frozen declaration for that obligation, with required fields present. Validate integer identity fields without accepting booleans. Preserve contradictory rows as raw evidence and mark input conflict; do not recompute the metric. The final loader must establish the same trace-to-result relationship before setting `provenance_valid`.

Suggested tiny cases: result run-ID mismatch, supplemental population mismatch, a correct response ID with wrong event ID, binding ID with altered expected set, goal ID with wrong member list, and bool repetition.

### A02 — Known progress identity conflicts do not reach the per-episode certification predicate

`:237–239` records duplicate STARTED/TERMINATED or unstarted TERMINATED as a report issue. The loop at `:244` then calls `episode_state` without that conflict, and `:245–246` only propagates duplicate result run IDs. A conflicting progress identity can remain CLOSED_CERTIFIED and contribute to certified timing/paired tables even though the final report status becomes INVALID_REPORT_INPUT.

Minimal repair: retain a set of conflicting progress identities and mark their episode rows IDENTITY_OR_SOURCE_CONFLICT before obligation partitioning. Preserve associated raw result numbers, progress records and actual costs, but do not call those numeric rows certified. Check the supported STARTED/DISPATCHED/TERMINATED lifecycle and contradictions with complete quiescent progress explicitly during final loader integration. This does not redefine the frozen evaluator's metric; it limits which saved result has a valid report association.

Suggested tiny case: a provenance-valid ELIGIBLE result plus duplicate STARTED/TERMINATED records remains an input conflict, with no certified timing or pair contribution and its known cost retained.

### A03 — JSON Pointer array indexing accepts noncanonical negative/leading-zero indexes

`get_pointer`, `:352–359`, converts any array token with `int(part)`. Tokens `-1` and `01` can select a real row while not being valid JSON Pointer array indexes. An exact artifact hash does not make that association pointer correct.

Minimal repair: when the current value is an array, permit only `0` or a nonzero digit followed by digits, require an in-range index, and reject negative/sign/leading-zero/append tokens. Keep dictionary string keys and existing `~0`/`~1` escaping semantics. Root remains the empty pointer. Report malformed pointers as input errors without choosing a different row.

### A04 — Unknown unresolved statuses are accepted as if emitted by this exact frozen ABI

`:141` accepts any status starting with UNRESOLVED. The inspected frozen QA01 producer emits `UNRESOLVED_EARLIEST_BOUNDARY`; frozen QA03 emits `UNRESOLVED`. The saved synthetic timing case uses invented `UNRESOLVED_EARLIEST`, so it does not demonstrate exact frozen-ABI validation.

Minimal repair: use exact allowed statuses per QA for this bound evaluator generation. Unknown status text remains an ABI contradiction with original data retained. This is not a change to failure semantics or a new metric filter.

## Positive scoped observations

The core explicitly represents both fixed populations and all configurations, preserves true observed zero separately from null, computes timing means from summed integer nanoseconds/count, partitions non-adoption separately from adopted incorrect bindings, and retains technical-unknown and unexecuted rows. It does not apply semantic correctness as a QA01 filter. Artifact references require explicit paths, hashes and sizes, and the association requires the exact population oracle and source mapping. These are appropriate directions, subject to actual loader and integration acceptance later.

The current seven small checks establish only their named synthetic cases. They do not establish actual-result compatibility, full441 ingestion, final ledger/RSS reconciliation, QA04 binding, paired completeness, semantic join correctness or report rendering. Preserve stage-01 and provide a new immutable checkpoint for repaired and completed source.
