# Quantitative adapter stage 03: bounded static closure

2026-10-19. Independent reviewer: GPT-6 Astra Ultra, `/root/quant_adapter_astra`.

**PASS_SOURCE_REPAIR_REVIEW_ONLY. S2-01–S2-05 are closed in the reviewed source scope. No new blocking finding. This is not actual final-report acceptance or permission to execute against the campaign.** Final quantitative execution still requires campaign quiescence and root GO.

## Exact accepted source scope

| Artifact | SHA256 |
| --- | --- |
| Stage-03 checkpoint | `2cfa6d971c2242b98c9b62e31c58e4de8ce856da10b63f42e3821550d1e355b9` |
| Copied `report_adapter.py` | `c0e369bab675d96cd2cbeb15c79523d89841cdc0a3e91bc2b12a477b94fe77a0` |
| Copied `test_report_adapter.py` | `1c01510f3d0e1a29f1b82dc7619ff46146371c31846ad8393435792b8758d3d4` |
| Copied output schema | `27dbad35efada5bfa7f5a4c93649d1d34b91ccb470e6495ab5664e0b5ac32628` |
| Copied input schema | `91f2353d321c58f2d5b2a3b0821ce6e842e9c5bc89a599c6af2a3ecbf4476546` |
| Copied adapter README | `608c347d378dbe00e5e5d7874736f234676247d644fe28942597f4abfa854e92` |
| Preserved stage02-to-stage03 source diff | `73e15a9bb5084c94a9246ef557cd076b55e1718f2fca37cd5550ed13c0eecce8` |
| Prior stage-02 review | `f9971c0869038b51d3e3b0699e40c66f1269c53b05a12f698ee36af32a849702` |
| Governing adapter specification | `b4bb8735e67ec84dbef1cb193ac3a36f6ea70411094515a7f5d616b0ee82a09e` |

The listed stage-03 hashes were checked. Review covered the repair diff, changed functions and their integration with episode/obligation certification, RSS summaries and pairs, independent artifact loading, bundle generation, UUID reconciliation and output invariants. The new/changed synthetic checks, saved results, schema changes and README were read. Relevant frozen controller and producer ABI was retained from stage02 review; the frozen RSS producer and analyzer CLI were additionally inspected in bounded text reads.

No test, adapter, analyzer, evaluator, model, parser, candidate, compiler, network request, full-corpus scan or process-control action ran during this review. The only new output is this review file. In particular, the reviewer did not inspect current formal traces or a live ledger.

## Finding closure

| Finding | Closure evidence and limits |
| --- | --- |
| **S2-01 / P2 lifecycle certification** | `progress_lifecycle` now requires exact unique ordered STARTED→DISPATCHED→TERMINATED before result certification. Unsupported, duplicated and out-of-order states conflict. Declared complete progress inconsistent with a saved result conflicts; ordered incomplete progress stays unverified. Episode state propagates into obligation partitions and certified pairs, with original values/cost retained. The small checks cover missing progress, incomplete/unsupported sequences and a valid sequence. This is controller-record reconciliation, not independent process attestation. |
| **S2-02 / P2 unavailable evidence** | `read_record_artifact` reads result, trace and association independently. Missing/unreadable evidence is retained in `unavailable_evidence`; malformed refs, stale hashes and identity/path contradictions remain conflicts. Supported terminal progress plus missing trace/result reaches TERMINATED_TRACE_MISSING, including a placeholder without run ID/refs. Missing association alone remains CLOSED_RESULT_UNVERIFIED. Available result/cost/calls survive a different unavailable artifact. The saved loader test moves temporary files aside and checks each distinct case, rather than only calling a state helper. |
| **S2-03 / P2 RSS completeness** | `recorded_peak_consistency` verifies the saved peak, same-sample concurrent sum and finite process contributions. Complete summaries and comparable RSS pairs require that consistency. Bad OBSERVED payloads receive an explicit input contradiction, raw values/reasons remain, and no substitute sample is selected. Frozen sample classification is not rerun. Tied samples are preserved. |
| **S2-04 / P2 repetition tables** | All56 fixed repetition bundles now include QA01/QA02/QA03 fixed partitions; QA01/QA03 use original integer sums and observed counts. Counts are43/15/23 for common9 and12/6/8 for supplement6. Output validation checks the56 unique bundles and denominator partitions. The unequal-observation synthetic case expects45ns/11, avoiding a mean of case means. |
| **S2-05 / P3 recorded discard** | `discarded_after_candidate_end` is projected explicitly. Per-source disposition occurrences preserve absence versus false/true. Disposition-only differences are excluded from financial UUID equality; conflicting annotations remain visible separately, while amount/usage/other financial-copy conflicts still invalidate reconciliation. One later trace annotation and an earlier global-log copy remain one cost. |

**RSS zero clarification:** the prior review suggested a true-zero RSS test alongside the generic nonnegative check. The exact frozen `memory_sample_state` in `common9/evaluate.py:84–85` requires a positive concurrent total even for STARTUP_ZERO_RSS; an individual starting process may have RSS0 while the sample total remains positive. Stage03's positive-total requirement therefore matches the supported frozen ABI and is accepted. It does not alter timing0 or individual startup-zero semantics. No frozen measurement policy was changed.

The input schema remains unchanged. Output schema additions describe retained invalid peak records and repetition bundles. Existing fixed populations, eight direct comparison groups, QA04 11/6 reuse, semantic annotations without QA01 filtering,24 UNREVIEWED cells and manifest-last immutable outputs remain in scope. No automatic topology, mechanism, semantic-success or architecture-winner inference was introduced.

## Saved checks and preserved failure

Six saved final batches report24 passing synthetic checks, maximum0.22293116594664752 seconds. Each record names the exact accepted source/test hashes above. These records were read and hashed, not rerun:

| Batch | Checks | Saved-record SHA256 |
| --- | ---: | --- |
| core |7| `967914e46222482eebfd1831335afd72c11153f5a6fd8659280a4b99c8837a0b` |
| review-fixes |3| `ce987559c83af3b00cd891ef388bccc299166ab752c9b41e2196d521bb7e4a4e` |
| projection |6| `d0777e6d589923b8f9ab0560b0eea4d0edfcb9c8f54d59a2ad614dd57762a3f7` |
| output |2| `65e552b38b6020aff6cfa111d9b5da128992d1978eed5991ff8287d6f9840e91` |
| loader |1| `aec1f649a6eb9dcf21369f40b38dd44c04bbeb386f723ddd302a0e2312498639` |
| stage02-fixes |5| `6f9897028981eaa54d9feb6445d17ac70f50443b3fa8074b49cbfdef3f2b76db` |

The earlier three-check record with one failed assertion remains preserved at `adapter_preparation/development/stage03-review-fixes-v1.json`, SHA256 `6163317c2923fdf094b4da04df551280e9480bd91ca3d77508b6d5954363579c`. It expected INVALID_REPORT_INPUT for unstarted TERMINATED while passing an explicitly incomplete progress context. The updated check declares complete progress when expecting that contradiction. The repaired source retains the distinction; failure evidence was not removed or represented as a pass.

These tiny checks establish their synthetic cases only. Actual441 ingestion, large-file read budgets, final result associations, budget reconciliation, optional semantic joins and completed report rendering over campaign data remain unexecuted and unaccepted.

## Minimal remaining final-input integration

One separate reporting tool is needed: **`scripts/architecture/common9_postanalysis/record_joint_report_inputs.py`** (proposed name; not implemented by this review). Keep it outside the frozen experiment and separate from the accepted adapter. Its narrow job is to record the authorized frozen analysis invocation and assemble the adapter's existing input ABI. It must not re-evaluate results in another implementation, launch episodes/models/parsers, alter fixtures or produce semantic judgments.

1. **Receive established quiescence.** Consume explicit root-provided evidence and immutable snapshots for campaign progress, status, global budget ledger and optionally the formal root's global call log. Verify their exact source associations and hashes. Record STOPPED/COMPLETED and `progress_complete` honestly; retain UNKNOWN/RESERVED or incomplete termination. Do not infer quiescence from elapsed time or a missing PID, and do not stop processes or settle calls itself.

2. **Record one actual frozen analysis operation after root GO.** Use the unchanged, hash-bound `frozen/source/benchmark/architecture/common_supplement6_20261019/analyze_joint.py` CLI with the exact formal `--root` and a fresh `--output`; no `--offline`. A small recording wrapper should capture the command as actually invoked, source/oracle/freeze identities, start/end, exit result, stderr/stdout references and final output hash. Preserve failed analysis evidence. The current analyzer accepts those flags and exclusively refuses an existing output file. It returns separate `primary_common9` and `supplement6` result arrays and `trace_provenance`. Do not invent an execution record afterwards from file hashes alone or promote old identity-only light evaluations.

3. **Read that joint output once and package bounded fragments.** Preserve the original output unchanged. For every observed result create an unchanged data-object copy in a small per-run file, with a manifest mapping its new hash to the exact original analysis path/hash/JSON pointer. Preserve both populations, all failures, result identity, original QA values and selected IDs. Match each result's run ID to `trace_provenance`, the exact planned job and explicit preserved trace path/hash. Refuse duplicate or conflicting associations rather than choosing one. Generate `JOINT441_RESULT_ASSOCIATION_v1` from the actual recorded analysis operation, including the correct population oracle, source112 mapping, binary/freeze, original and fragment provenance. Each `result_index` entry then references the small result and association files. Missing planned jobs remain controlled by frozen plan/progress; do not synthesize a numerical result for them.

4. **Package existing summaries and references without new algorithms.** Extract small unchanged per-configuration summary objects into `summary_index`; extract the two existing top-level financial totals with their original analysis selector; package supplied supplement mechanism rows with exact source selectors. Link the five already accepted QA04 approval roles and population oracles. Keep `raw_pairs` absent when the frozen joint output has none. Include only separately bound available semantic-review/preregistration inputs; missing items remain UNREVIEWED. Do not create model-based judgments or recompute endpoint/DAG/RSS/semantic metrics in the assembler.

5. **Emit the existing manifest and stop before reporting unless root directs it.** Write `JOINT441_QUANTITATIVE_INPUT_v1`, full fragment/association provenance and a final hash manifest exclusively in a new postanalysis directory. A later explicit invocation of the accepted adapter consumes this input and writes a separate new report directory. Runtime corpus execution and final report acceptance are distinct steps.

The fragment packaging is necessary integration work, not a new analysis method. `ArtifactReader.read` rereads and charges the whole referenced file for each JSON pointer;441 pointers into one large joint JSON can exceed its cumulative read limit. Small result/summary fragments avoid that multiplier without changing the accepted adapter or raising its caps. Original-analysis copies must carry exact source selectors and hash linkage, so fragmentation never loses the result-to-trace/oracle/analysis relationship. The assembler may parse the final analysis once under an explicit bounded size limit after quiescence; oversize input must fail visibly and preserve evidence. This review establishes no actual final size and authorizes no present corpus parse.

Sol can prepare this single glue tool and small temporary synthetic ABI cases before quiescence if root directs it. Needed cases are simply split/provenance preservation, duplicate/mismatched identity rejection and a preserved failed analysis record. No new audit campaign, score revision, topology reconstruction, measurement rerun or broad redesign is needed for this integration.

## Acceptance boundary

The five reported source defects are closed for the exact stage03 hashes. The running experiment, frozen contracts/sources/oracles, accepted QA04 generation and existing semantic extractors remain outside the mutation scope. No actual quantitative result, budget total, model outcome, completed semantic population or DP judgment is accepted by this closure. The root must still authorize final analysis after quiescence and review the resulting immutable associations and reporting output.
