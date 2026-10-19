# Generic closed-block evidence helper v3: bounded design

2026-10-19. GPT-6 Astra Ultra. **Plan only; no implementation or additional extraction authorized by this record.** Preserve the accepted v2 source/checkpoint and the immutable S02 bundle. Continue the fixed campaign unchanged.

## Scope

Add a separately versioned helper and explicit immutable block index so one already terminated seven-configuration block can be reviewed in frozen job order across COMMON9 and COMMON_SUPPLEMENT6. Preserve v2 evidence projection, explicit omissions, raw event IDs/indices, original-file references, actual-input versus oracle separation, unknown-clock ambiguity and all UNREVIEWED defaults. Saved evaluations remain unsupported. No evaluator, analyzer, parser, model, candidate, build, network call or image decoding is part of extraction.

The index is termination and membership evidence. It is not technical eligibility, semantic correctness, a selected QA01 endpoint or permission to execute jobs.

## Exact index ABI

Use `version: JOINT441_CLOSED_BLOCK_INDEX_v1` and the following required structure. Paths identify exact files; all immutable references include SHA256 and byte size.

```json
{
  "version": "JOINT441_CLOSED_BLOCK_INDEX_v1",
  "kind": "POSTLAUNCH_TERMINATION_INDEX_NOT_MEASUREMENT_CERTIFICATION",
  "formal_root": "explicit original joint-formal-v1 root",
  "freeze": {"path": "freeze.json", "sha256": "exact frozen hash", "bytes": 0},
  "binary_sha256": "exact frozen candidate hash",
  "block": {
    "block_id": "literal frozen job block_id",
    "population": "COMMON9 or COMMON_SUPPLEMENT6",
    "episode": "literal frozen case ID",
    "rep": 1,
    "ordinal_in_frozen_block_order": 0,
    "job_indexes": [0, 1, 2, 3, 4, 5, 6]
  },
  "source_progress": {
    "original_path": "campaign-progress.jsonl",
    "captured_at_utc": "actual capture time",
    "scope": "EXPLICIT_COMPLETE_LINE_RANGES_ONLY_FILE_REMAINS_APPENDABLE"
  },
  "rows": [
    {
      "job_index": 0,
      "configuration": "AAAA",
      "episode": "literal case ID",
      "rep": 1,
      "population": "literal population",
      "run_id": "explicit actual run ID",
      "termination_record": {
        "copied_path": "immutable raw line copy",
        "sha256": "hash of original exact line bytes including LF",
        "bytes": 0,
        "original_byte_offset": 0,
        "original_byte_length": 0,
        "state": "TERMINATED"
      },
      "run_id_origin": {
        "method": "RECORDED_STDOUT_LAST_JSON",
        "field": "run_id"
      },
      "trace": {"path": "raw/runs/explicit-run-id/trace.json", "sha256": "exact closed trace hash", "bytes": 0}
    }
  ],
  "construction": {
    "tool_or_reviewer": "explicit producer identity/version",
    "method": "explicit seven known terminated records and trace references",
    "runtime_modified": false,
    "jobs_executed": false
  }
}
```

This is a field-shape example, not an index: zero sizes, example IDs and example hashes are invalid real inputs. `rows` must contain exactly seven records, one for each frozen configuration, in the same order as the seven referenced jobs. Each row's termination copy must retain the full original JSONL record, including job fields, returncode, stdout, stderr, technical_failure, budget_stop and actual termination timestamp. Do not replace the record with selected success fields.

The frozen controller (`joint_campaign.py:181–191`) appends TERMINATED only after `process.communicate()`. It retains the actual stdout. Derive run ID only from the exact recorded final JSON summary when present, and compare it with the explicitly supplied ID and closed trace. If the record has no supported run-ID evidence, keep a separate unresolved-index diagnostic and do not invent or substitute a run. A future supported alternative run-ID source requires its own explicit ABI and review. Existing manual block summaries may be linked as secondary context, but do not replace these records.

## Index construction without an active-file scan

A coordinator supplies the seven known terminal-record byte ranges or seven exact captured terminal records from existing controller output. A new preparation tool may read only those explicit ranges, confirm complete newline-terminated JSON records, and copy their exact bytes to a fresh immutable index directory. Record each original offset/length/hash. If original byte ranges are available, reread only those same ranges to confirm stability. Concurrent appends outside them are permitted; do not require or claim an immutable hash for the entire active progress file. If using captured controller output instead, preserve its exact file/hash/range and label that source accurately.

Do not scan active `runs.jsonl`, the whole progress history, directories or status files to guess run IDs. Do not trust a substring such as CLOSED, newest filename, exit0 alone or a manually asserted seven-row count. The index producer never calls the campaign controller and never changes its files. The immutable line copies and index each receive hashes; an explicit root-approved index path and SHA are extractor arguments.

## Extractor validation

1. Verify the index hash, supported version, exact formal root/freeze/binary and safe root-relative paths. Reject symlink escapes, duplicate JSON identities and unsupported source-reference schemes. Hash all copied terminal records and compare the preserved bytes to their recorded metadata.
2. Select the seven jobs from the exact freeze by the supplied job indexes. Require one frozen block ID, contiguous positions, identical population/case/rep and all seven configurations once. Validate every row against those jobs; do not regenerate order from configuration names.
3. Parse the copied terminal records. Require exact `state == TERMINATED`, matching frozen job identity and block ID. Keep nonzero returncode, technical failure and budget stop as evidence, not exclusion criteria. A terminal record does not automatically make the associated model response successful or technically certified.
4. Require unique explicit run IDs and exact run-ID linkage from retained stdout, exact expected trace path, matching trace SHA/bytes, FORMAL phase, run/configuration/case/rep/binary, integer closed `end_ns` and returncode, and candidate-input parent association. Reject bool values where an integer is required. A missing trace or unsupported identity yields an immutable incomplete diagnostic with all planned obligations retained; no alternate trace is selected. It does not authorize another measurement run.
5. Load the appropriate frozen oracle and, for the supplement, separate functional-oracle record by the exact population and episode. Check source hashes against the freeze. Actual user/source inputs come from the trace and candidate-initial object, with that initial-only limitation explicit. No expectation is injected into a prompt or inferred to be a delivered input.
6. Keep every raw callback and all known failure/effect context. Saved primary eligibility and selected QA01 endpoint remain unavailable. No BG eligible-endpoint logic or semantic correctness gate is recreated. Apply the v2 unknown-clock, tie and duplicate-ID ambiguity rules unchanged.
7. Write evidence, worklist, readable output and manifest exclusively to a new directory. Hash every output and bind v3 source, any imported v2 dependency, schema, index and freeze. Preserve all original files and the previous helper generation.

A narrowly diffed v3 file may reuse audited v2 pure utilities. Do not alter v2 constants to make a generic block masquerade as S02, feed a synthetic S02 closure to v2, or import a moving unbound helper dependency. Review the actual new source diff/checkpoint before root permits its first extraction.

## Fixed review order and bounds

Use frozen block order for review admission. The first S02 block is already reviewed. The next index should be the next block in that same frozen order. Record an immutable review cursor containing the previous accepted index/bundle/review hashes and expected next block ordinal. The cursor controls postanalysis selection only. If the next block is not fully terminated or lacks valid evidence, report that fact and wait or retain an explicitly unresolved review entry; do not select a better-performing later block. Root may authorize an explicit exception with its reason preserved.

Keep one block per command, 16 MiB per file and 128 MiB total read cap. Count index, terminal copies, freeze, source references, traces and referenced raw files toward the cap. Preserve explicit omission/error diagnostics when a limit is exceeded. No silent cap increase, multi-block prefetch, full-corpus extraction or evaluator execution. Root schedules each initial use under the existing bounded wall-time policy while measurement continues.

## Small synthetic acceptance required before first actual v3 use

Implementation remains Sol's responsibility after root authorization. Required synthetic cases cover:

- Valid COMMON9 and supplement blocks with their own obligations and functional expectations.
- Wrong freeze/root/binary, mixed population/case/rep, wrong ordinal/job indexes, duplicate/missing configuration and duplicate run ID.
- NOT_TERMINATED/NOT_CLOSED labels, manual-only summaries, incomplete JSONL lines, bad range/hash and unsupported run-ID origin.
- Stdout run-ID mismatch, nonzero/technical terminal preservation, missing trace, open trace and bool timestamp rejection.
- A concurrent append outside selected terminal ranges succeeds; mutation inside a selected range fails.
- Exact next-block cursor enforcement and explicit unresolved handling without selecting a later successful block.
- V2 retained behavior: missing callback clock ambiguity, root JSON Pointer, source/actual-input separation, candidate initial-only scope, raw-first versus uncertified endpoint, unmatched failure/effect context, no saved-evaluation acceptance, all output hashes and immutable creation.
- Per-file/total limits, traversal/symlink escape rejection and absence of evaluator/model/parser/candidate/network execution.

These are prospective test requirements. No implementation, test or new extraction was performed for this plan.
