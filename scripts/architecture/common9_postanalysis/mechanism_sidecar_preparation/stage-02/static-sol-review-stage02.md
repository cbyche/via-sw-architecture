# DP1/DP4 sidecar stage 02: independent static closure review

**Verdict: REQUEST_CHANGES.** Five stage-01 findings are closed by static inspection; MS1-03 is partially closed with one remaining P2. No additional distinct P1/P2 was identified within the stated scope. Reviewer: GPT-6.1 Sol High, `/root/sol_high_continuation/sidecar_closure_sol`, 2026-10-20. This is a new immutable review of stage 02; stage 01 is preserved.

## Bound source and inspected artifacts

The complete 853-line stage-02 source copy, complete saved `test_fake.py`, both saved synthetic result JSON files, README, CHECKPOINT, complete stage-01 six-finding review, relevant repository instructions, and accepted `mechanism-evidence-readiness-v1.md` were read. The active source and copied source have the same SHA-256, so the complete copied-source inspection applies to both. Line references below name the stage-02 source copy.

| Artifact | SHA-256 |
| --- | --- |
| Active and copied `extract_dp1_dp4_mechanism_evidence.py` | `9f1390ec343716328bfc19bc8ddabed4ad32da0c3cbeac3237bcf3c97c88bf4e` |
| `test_fake.py` | `a176954aa97a2c62e3d2df2fe909ebfa74fd9b15123d2494de857b9e34b8215a` |
| `CHECKPOINT.json` | `63e6f9c6353796ae4f7bca579ddb1dae95d1d5d403e4e9a6e77fc94036d33c33` |
| `README.md` | `88119193892ac016947ffcd741ab11f826b2a864c83f1ecd0eec39e10f77a1a1` |
| `fake-results-attempt-01.json` | `099401946a447779da186b193e73dca52221439fa8a607b6999e22d29f676e87` |
| `fake-results-attempt-02.json` | `4ee6f3e78a7c832397867353f2c28a01fb7d0014ed021074885292e0b1378057` |
| Accepted readiness specification | `071f5872010214aca45f168c585ac1e323f7c303187693f69c45d8e0796e85ea` |

The saved first attempt reports 22 cases against source `2b53fd70c8a265948e1b99efcb0b1e2be8cae52179d4eb4496516f7c5d34f6af`. The saved final attempt reports 26 unique cases against the reviewed source, no failures/errors, 0.030923250131309032 seconds and 52,263 named fake-file bytes. Those are saved preparation claims, not checks rerun by this reviewer. The two attempts total 48 executions, not 48 unique cases. Saved virtual writer evidence preserves 441 rows for late-boundary rejection and malformed-index processing; it does not exercise the remaining malformed-event case below.

## Six-finding closure

| Prior finding | Static closure | Evidence and limits |
| --- | --- | --- |
| MS1-01 | CLOSED | One retained Reader exists before source-self/preflight reads and survives rejection. `capture` gates reads against both file and aggregate allowance, includes source-self and failed-byte captures, and retains attempts. Input manifest is read once; report metadata uses the same Reader. Receipt finalization keeps inventory/bytes on late failure. Saved cases 01–04 address these paths. Live quiescence and actual file behavior remain untested here. |
| MS1-02 | CLOSED | Occurrence tuples independently carry `association_pointer`; actual ingress uses `/events/i/data/request`, work-record input `/mandatory`, receiver input `/contract/mandatory`. Both ordinary and decoded projectors use that pointer, with exact outer user-text pointers retained. Cases 05–07 cover ingress, array content and work-record envelopes. |
| MS1-03 | PARTIALLY CLOSED | Identity scalar validation, malformed-index conflicts, row serialization containment and retention of accepted planned identities are present. Collection processing state and per-collection `coverage_issue` close the cited null-parent/null-support cases. The invalid-event branch still permits false COMPLETE collection coverage; see MS2-01. |
| MS1-04 | CLOSED | UUID signatures and operation-node/UUID maps separate raw occurrences, recorded UUIDs and established operation identities. Conflicting/missing identities receive no established operation key; ingress IDs have independent conflict handling. Exposure summaries retain unresolved coverage and raw counts. Cases 12–16 and 23–25 cover the specified contradictions, missing identities and duplicate copies. A failed dispatched call remains supply without a success requirement. |
| MS1-05 | CLOSED | Acquisition matches require `n < consumer_index`, matching acquisition node and source-span containment. Reuse and record-support joins pass their own recorded consumer index. Unknown/ambiguous acquisitions remain unresolved. Unique exact committed-payload matching deliberately does not require universal commit-before-consumption ordering. Cases 17–19 and 26 reflect these separate rules. |
| MS1-06 | CLOSED | `INPUT_FIELDS` includes exact `observed_evidence` and `actual_delivered`; existing decoded projection retains source identities and hashed prose. Source-shaped case 20 checks both keys and the decoded original-text pointer. No extra stage discovery or prose interpretation was added. |

## MS2-01: P2, malformed events lose collection failure coverage

**Locations:** `finish_collection` lines 252–256; invalid-event branch lines 381–385; finalizers at 431–432, 465, 474, 550, 573 and 642. This is the remaining part of MS1-03, not a seventh independent mechanism problem.

**Input condition:** a terminal-shaped trace (`end_ns` and `returncode` integers, `calls: []`) has an event that is not an object or whose `data` is not an object. For example, `/events/0` can be `{"kind":"SOURCE_RANGE_READ","data":null}`. No campaign trace was inspected or executed to construct this example; it follows directly from the branch conditions.

The branch calls `issue` without a collection, sets every collection's `status` to `UNSUPPORTED`, and continues. It does not set `coverage_issue` on those collections or change their `PENDING` processing state. The local terminal-derived `status` remains `COMPLETE_WITHIN_DECLARED_SCOPE`. Each later `finish_collection` consequently overwrites the collection status to COMPLETE. The malformed source-read event can therefore disappear from an empty, COMPLETE `source_reads` collection; the same reset affects the other extraction collections. `join_issues` remains UNSUPPORTED because `issue` sets its own coverage flag, and the row's extraction status remains UNSUPPORTED. Those row-level signals preserve the error but do not satisfy the accepted per-collection coverage contract.

**Impact:** a reader using the explicit collection coverage field can mistake omitted or uninspectable events for complete extraction coverage. The `trace_complete_for_absence_claims=false` safeguard remains useful, but does not make the collection coverage accurate. No measured result, campaign stop or source inactivity is inferred from this defect.

**Required repair:** make an event/data-shape failure persist on every affected collection through finalization, for example by setting its coverage flag as well as status, or by routing that state through a common failure helper. Preserve already projected facts and all planned rows. Add a separately authorized tiny synthetic terminal-trace case containing invalid event/data shapes and inspect every affected collection's final status, the row status and retained issue. Tests were not added or run in this review.

## Execution boundary and handoff

Actual extraction executions by this reviewer: **0**. Tests/analyzers/extractors/candidates/domain parsers/models/builds/network operations: **0**. Campaign trace/result/model-input/response reads: **0**. Only named source/specification/preparation files and the four separately authorized final semantic-review Markdown files for the independent prose audit were read. No paid441 controller operation or state probe occurred. No source, checkpoint, saved test result, frozen artifact, accepted adapter/assembler, metric, presentation or previous review was edited.

The root-approved required `--formal-root` is compatible with this review scope. Source acceptance still requires closing MS2-01 in a preserved fresh stage and independent review. Actual extraction additionally requires independently established controller quiescence and an exact separate root GO. This review establishes neither and does not authorize either. Semantic use, saved original work, causal savings, product completion and a winner remain outside this static review.
