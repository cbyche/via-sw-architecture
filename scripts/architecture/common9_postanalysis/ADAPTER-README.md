# Joint441 quantitative reporting adapter v1

Implementation outside the frozen runtime. **Not authorized for actual corpus execution until campaign quiescence and root GO.** No evaluator imports, subprocesses, network, candidate commands, model requests, image decoding or recursive discovery. Accepted extractor v2 and lead-owned v3 tools are separate.

Approved specification: `astra-quantitative-adapter-spec-v1.md`, SHA256 `b4bb8735e67ec84dbef1cb193ac3a36f6ea70411094515a7f5d616b0ee82a09e`. This adapter does not create semantic correctness, endpoint certification, a new QA01 filter or an architectural winner. All24 prediction cells initialize UNREVIEWED. Semantic annotations never replace or filter frozen numerical observations.

## Input ABI

`report_adapter.input.schema.json` defines `JOINT441_QUANTITATIVE_INPUT_v1`. Every file ref is an explicit absolute regular file path, lowercase SHA256, exact integer bytes and optional RFC6901 pointer. No symlink ancestors, traversal or silent restore relocation. A restore mapping is currently unsupported and must be explicitly implemented/reviewed before use. Reads are sequential and capped: default16MiB/file and8GiB total; hard maximum64MiB/file and16GiB total, recorded in provenance. File timestamps/size/hash are rechecked. Nonfinite JSON constants are rejected. No model request or response file bodies are opened.

Required artifacts: exact formal freeze, exact approval binding, independent quiescence record, immutable progress/status/ledger snapshots, explicit result index. Oracles are explicitly chosen per population and must match the frozen source path/hash. All112 frozen source entries are hashed explicitly; no directory scan or weight/binary read. The immutable manifest is inside the exact formal root. The fixed441 ordered jobs come from the freeze, not regenerated filesystem order.

The quiescence ABI is `JOINT441_QUIESCENT_SNAPSHOT_v1`: `controller_state=STOPPED|COMPLETED`, `no_active_episode_processes=true`, boolean `progress_complete`, exact `progress_snapshot_sha256`, `status_snapshot_sha256`, `ledger_snapshot_sha256`, and nonempty `evidence_refs`. Evidence is consumed, not independently inferred from a missing PID/time. Each snapshot ref has `snapshot_of` the exact controller file or `freeze.ledger_path`, and uses a different immutable path. RESERVED/UNKNOWN calls remain explicit incomplete accounting; they are never silently settled.

The result index is JSONL, one declared item per observed identity, containing `identity={population,configuration,episode,rep}`, `run_id`, `result_ref`, `trace_ref`, optional `association_ref`. Duplicate records remain conflicts, not a preferred-record selection. Repetition is a strict integer (bool is rejected). Missing association is unverified evidence. Trace path must be `<formal_root>/raw/runs/<run_id>/trace.json`; phase, run ID, configuration, episode, rep, binary and terminal fields must agree. All actual planned identities remain even if unexecuted/failed/missing.

An accepted independently recorded association has version `JOINT441_RESULT_ASSOCIATION_v1`, FORMAL phase, strict mode, COMMON9-EVALUATOR-v3, exact formal root/freeze SHA/binary SHA, identity/run ID, exact result/trace/population-oracle refs, complete `source_sha256` mapping of all frozen112 source paths, `recorded_analysis_argv`, `recorded_by`, `recorded_at`. These are historical facts supplied by a separate recorder. The adapter never synthesizes that association from hashes read now. Actual result body identity and copied frozen declaration fields must also agree. Identity-only old light evaluations and the incomplete early strict-envelope example remain unverified; no forced compatibility upgrade.

Optional artifacts:

- `summary_index`: JSONL `{population,configuration,ref}`; ref selects the exact frozen per-configuration summary object (e.g. `/FORMAL:AAAA`). Reproduced counts/means and supported QA02/QA06 fields are compared; decimal QA05 representation residuals remain explicit.
- `global_call_snapshot`: immutable JSONL global call log, not the live path. Exact frozen job mapping supplies a missing population only when unambiguous.
- `frozen_analysis_totals`: immutable object ref containing `total_actual_recorded_API_cost_usd` and `unknown_call_count` from the recorded frozen analysis, with a provenance file/pointer rather than unattached copied numbers.
- `raw_pairs`: ref to preserved original paired diagnostics. Never called certified.
- `mechanism_index`: JSONL `{identity,ref}`; diagnostic only. No activation inference is performed by this adapter.
- `semantic_review_index`: JSONL `{identity,qa,obligation_id,dimension_refs:{dimension:file_ref}}`. Each exact obligation must exist; each selected dimension object carries PASS/FAIL/UNREVIEWED/NOT_APPLICABLE. Actual-request correctness, literal oracle match and mismatch/applicability remain separate annotations. NOT_APPLICABLE is not PASS.
- `registrations`: ref to an array with unique `{dp,qa,preregistered_statement_ref,prior_direction,favorable_conditions,counterconditions}`. Missing ranking is `UNRESOLVED_PREREGISTRATION_REVIEW`, never invented `NO_PRIOR_RANKING`.

QA04 supplies five explicit approval role/ref pairs: report, classification, final_review, raw630, checkpoint. Their hashes are fixed by the accepted dataset. Each exact role/path/SHA/bytes must match the approval artifacts array. Final checkpoint and final review links are checked separately from preserved historical status labels. The one42-change/630-regression dataset is reused, not multiplied by population. Accepted vector is [1,1,3,3,1,2],11/6; offline RSS diagnostics are not formal QA06.

## Output and state rules

`report_adapter.schema.json` defines `JOINT441_QUANTITATIVE_REPORT_v1`; Python additionally validates uniqueness, all fixed populations/configurations/denominators,3381 obligation rows,4872 planned pair rows and24 UNREVIEWED predictions. Clarification from spec §8 illustrative schema: `pairs` is an object holding `rows_file`, planned count, certified summaries and separate raw diagnostics. Each population has its own `{file,population}` `case_repetition_refs` object; both references are retained.

QA01 allowed unresolved status is exactly `UNRESOLVED_EARLIEST_BOUNDARY`; QA03 uses exactly `UNRESOLVED`. Unknown statuses/invalid ns are ABI contradictions. True observed0 survives. Missing timing is null, rendered as an em dash. Ordinary functional failure does not erase technical certification. Progress conflicts, copied declaration mismatch and result identity mismatch remove that affected identity from certified timing/pairs while preserving cost/raw references.

Eight direct alternative pairs retain every planned row. Certified timing deltas require both sides CLOSED_CERTIFIED and the same obligation CERTIFIED_OBSERVED. B/C is computed directly. AAAA reuse creates no additional expenditure. QA05/QA06 episode comparisons are supplementary; representative QA05 is the fixed9/6-case bundle mean, QA06 is a same-sample fixed-population maximum. Incomplete comparisons stay null/lower-bound. RSS ties retain all same-sample process contributions and references. No sums of role maxima, remote model memory or local monetary compute estimates.

Calls are UUID-deduplicated with conflicting copies visible. Actual cloud, local, administrative/unsent, recorded discarded/retried work, full price identity and usage refs are separate. Decimal arithmetic preserves source representations/residuals. Scope-less ledger entries, unknown/reserved amounts, unmatched UUIDs, late settlements and differing prices remain explicit. Ledger budget accounting and per-population comparison are separate. Neither elapsed time nor API0 means zero local cost.

An entirely new output directory is required. Eight payload outputs include two population tables, separate coverage/failure information, all planned rows, pair/call rows and prediction worklist. Every payload including Markdown is hashed; `manifest.json` is exclusively created last. If input rejection occurs after a verified plan, all planned identities/obligations are retained with unavailable evidence; before a verified plan, `input-rejection.json` explicitly reports plan unavailable rather than inventing identities. A failed partial write directory is retained and cannot be reused.

Future authorized command (not executed against the corpus):

```
python3 -B scripts/architecture/common9_postanalysis/report_adapter.py --input-manifest /EXACT/FORMAL/ROOT/postanalysis/immutable-input.json --input-sha256 SHA256 --output /EXACT/FORMAL/ROOT/postanalysis/NEW-REPORT
```

Only temporary synthetic test batches are authorized during measurement. Test runner emits exclusive JSON records binding exact tested source hashes and elapsed duration. Stage01 originals/review and all later development records remain preserved under `adapter_preparation/`; final review uses an immutable copied source checkpoint, not moving files.


## Stage03 corrections to the preserved stage02 implementation

Stage02 source and independent review remain immutable. A closed strict result requires the exact supported per-identity controller sequence STARTED→DISPATCHED→TERMINATED before certification. Unsupported, duplicate or out-of-order states are contradictions. With a complete progress snapshot, absent/incomplete lifecycle contradicting a saved result is a source/identity conflict. With explicitly incomplete progress, an ordered unique partial sequence is recoverable unavailable evidence, never certified; a supported started prefix without a result remains STARTED_UNCLOSED. Exact terminal lifecycle with missing trace/result is TERMINATED_TRACE_MISSING. Partial progress does not invent unexecuted identities.

Missing/unreadable trace/result references (including declared placeholders with absent refs) are availability records, separate from stale hashes, unsafe paths or identity contradictions. Missing independent association alone is CLOSED_RESULT_UNVERIFIED. Each available artifact is read separately, so available result raw values/cost or actual trace call records survive a different artifact being unavailable. No replacement artifact is searched for.

RSS completeness and comparable episode pairs now require supported recorded peak arithmetic: a positive finite recorded peak, a same-sample object with exactly that concurrent sum, nonempty finite nonnegative process contributions summing to that recorded number. This validates only saved-result internal consistency; it never reruns sample classification, changes the frozen status or chooses a substitute peak. Inconsistent saved OBSERVED data is an explicit reporting-input contradiction; raw peak/sample/reason remain in unverified_peak_records and the original episode row, without COMPLETE labels or comparable differences.

Each fixed repetition bundle includes QA01/02/03 planned partitions and QA01/03 sums/counts/micro means: COMMON9 43/15/23, supplement12/6/8. Cases, repetitions and configurations stay fixed.

Recorded work disposition includes discarded_after_candidate_end. Since the trace may add this after the global log was written, disposition annotations are excluded from financial UUID identity equality. Both occurrences retain exact fields, including absence versus explicit false. Nonconflicting explicit fields are merged as a convenience; disagreements remain separate annotation records. Neither a later true flag nor its absence on an earlier copy creates another expenditure or a false financial-copy conflict. Actual financial/token/usage differences still conflict.
