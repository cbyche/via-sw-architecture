# Closed-block extractor v3: stage-01 static review

2026-10-19. Reviewer: GPT-6 Astra Ultra, `/root/method_astra_review`.

**Decision: HOLD actual v3 index preparation and extraction pending three bounded helper repairs.** The running joint441 campaign and its frozen sources remain unchanged. This is postlaunch evidence-tool review, not a measurement approval or a judgment of model outcomes.

## Exact scope and evidence

- Source checkpoint: `source-checkpoint-v3.json`, SHA256 `7655fcca77814c896208340cdd04a1c0f86d2ab47016c856814db6349957573a`.
- Immutable stage manifest: `v3_preparation/stage-01/manifest.json`, SHA256 `9fc7c0c85480417ce6c70e86017f4d624a255785e32f0e4ff3bf8e84ded71d2d`.
- The eleven files named by that manifest were read or hash/size checked against their immutable stage copies. All eleven hashes and sizes matched.
- Extractor SHA256: `b5f7467617d3258624a71c24f74ed61ffafbf55a9b62496d756ac2cbc8c7d325`; preparation tool: `f30dca2d67b7ce75219cad5e09a994a1235359beed61c5e53d792d3c218f82ec`; schema utilities: `dbe83890e60b0f080a3fd1ab735908c5ffddd9994405a488de90dd4325378041`.
- Saved synthetic result: `synthetic-final.json`, SHA256 `949a5d42598118357fc24b590150f3d6ca131ee144a3a545fa44c5b7538842af`. It reports 31 passing checks in 0.31310187513008714 seconds. The reviewer read the result and test source and did not rerun them.
- Review requirements: `postanalysis/planning/astra-closed-block-v3-plan-v1.md`, SHA256 `1e8d7b36fe70f4b9dfb983f049cac2a16a9e6160696135b000c3951a7657cbcf`, and its prefix addendum, SHA256 `299f87a17afa436e4c1bb1e4e4424c9e4c4213a5f91154b0411618246bee9bfe`, under the original joint-formal-v1 root.
- Two bounded frozen ABI checks supplemented the helper read: joint controller lines 158–202 and the harness final-summary statements; the 3,097-byte frozen supplemental `functional-oracles.json` was inspected for its direct episode-key shape. Its SHA256 is `484d1f00c708bea7dc010c54e37d98831b77ee77c188d1c6be888da89484ffaa`. Direct S80–S85 episode lookup is correct and remains evaluator-only context.

No actual progress prefix, index or corpus trace was extracted. No evaluator, analyzer, test, model, parser, candidate, build or network operation ran. Only new review prose was written.

## Findings requiring repair

Line references below identify the immutable stage-01 copies, except the explicitly named frozen controller.

### V3-01: terminal flags must preserve the actual null state

`schema_v3.py:126–127` requires `technical_failure` and `budget_stop` to be booleans. The frozen `benchmark/architecture/common_supplement6_20261019/joint_campaign.py:184–185` uses an empty summary when the harness process return code is nonzero, then records both values through `summary.get(...)`. Such a real terminal record has null flags. The helper rejects it even if its recorded stdout provides a supported run ID and its trace is closed.

`check_v3.py:153–155` covers a nonzero terminal record whose flags are artificially set to true. That saved passing case does not cover the frozen controller's nonzero/null shape.

Required minimal repair: require both fields to be present and accept boolean or null. Preserve null as unknown, retain return code, stderr and the complete original terminal line, and do not normalize it to false. Add a synthetic controller-shaped nonzero/null record with supported stdout and a closed trace. Missing or unsupported run-ID evidence must still yield an incomplete diagnostic without selecting another run.

### V3-02: ordinary cursor authorization is not linked to the preceding review

`schema_v3.py:68–81` hashes the previous review and root authorization, but the nonbootstrap branch checks neither record against the current next ordinal or exact preceding index/bundle/review set. `check_v3.py:195–211` demonstrates the gap: its ordinal-2 cursor reuses the old S02 review and the bootstrap-only authorization while accepting a different previous v3 index and bundle.

This permits a cursor to claim advancement after the preceding reviewed block without evidence that the referenced review or explicit root decision concerns that block. A semantic PASS requirement is unnecessary; unresolved reviews are valid records.

Required minimal repair: retain the narrow S02 bootstrap exception, and require an ordinary authorization record with an explicit scope such as `ADVANCE_EXACT_PREVIOUS_V3_REVIEW`, exact freeze hash, next ordinal, previous block identity and previous index/bundle/review hash map. Test stale bootstrap authority, mismatched review hash and wrong ordinal rejection. The record can authorize advancement with unresolved review scope without converting it to PASS.

### V3-03: enforce the read cap before the read

`prepare_block_index_v3.py:50–54` reads up to the full prefix cap before checking the remaining aggregate budget. Lines 82–86 do the same for each selected terminal range. `extract_closed_block_v3.py:75–81` checks the pre-read file size but requests `limit + 1` bytes and checks file change afterward. A nearly exhausted aggregate allowance or a file growing between stat and read can consume more than the declared hard cap before rejection.

Required minimal repair: check or reserve the maximum requested bytes before prefix and range reads. Bound immutable-file reads to the verified size and remaining allowance, then reject changed or truncated content without reading beyond the cap. Preserve attempted-read accounting in available diagnostics. Tiny near-cap synthetic cases are sufficient; no actual corpus read is required.

## Accepted behavior within this stage

- The requested frozen block and cursor are checked before progress discovery. Ordinal times seven selects the exact contiguous frozen jobs; configuration order is preserved rather than regenerated.
- Discovery uses one prefix read from byte zero, capped at 1,048,576 bytes. It parses only complete LF-terminated bytes, retains a partial tail, selects only the requested block and rereads only its seven exact ranges. The saved tests cover append, selected-byte mutation, replacement, duplicate terminal and truncated-prefix cases.
- Prefix SHA is scoped to captured bytes. The artifact explicitly disclaims global duplicate absence after capture and requires final reconciliation.
- Supported stdout run IDs are explicit and checked against trace identity and expected path. Existing trace hashes, closed integer fields and candidate-initial parent association are checked. Missing-at-index traces remain missing evidence if a later file appears; the helper does not substitute it silently.
- Frozen obligations survive missing traces. Nonzero/technical evidence is intended to remain reviewable; V3-01 is the concrete ABI correction needed for that intent.
- Raw callbacks with unusable clocks, tied earliest clocks or duplicate selected event IDs remain ambiguous. They are not substituted for certified QA01 endpoints.
- Initial candidate context is explicitly initial-only. Later input and control evidence stays in recorded trace events. Frozen expectations, actual inputs and raw prompt-file references remain distinct.
- Supplemental functional expectations are separate metadata. No oracle material is introduced into candidate input.
- Saved evaluator inputs remain unsupported. All review dimensions start UNREVIEWED. Failure/effect context, original pointers, image/binary omissions and output hashes are preserved.
- Output creation is exclusive in a new nonfrozen directory. The accepted v2 extractor and its checkpoint are preserved and hash-bound.

## Closure boundary

Sol acknowledged V3-01 and V3-02 and selected the explicit ordinary-authorization repair. V3-03 was sent as the final finding for this batch. Preserve this stage and its saved checks, then provide a new immutable checkpoint and bounded synthetic results for review of the exact repairs.

Even after source closure, an actual index/extraction command still needs root scheduling with the exact cursor and output path. This review does not authorize another block, change the 1-second initial execution policy, certify semantic outcomes, certify primary metrics or alter the running experiment.
