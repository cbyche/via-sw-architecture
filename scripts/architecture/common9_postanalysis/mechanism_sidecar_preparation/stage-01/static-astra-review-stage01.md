# DP1/DP4 sidecar stage 01 — independent static review

**Verdict: REQUEST_CHANGES before source acceptance or any actual extraction GO.** Six bounded findings below concern this new descriptive sidecar. They do not invalidate the paid campaign, require a controller action, revise a metric, or change the accepted quantitative adapter. Reviewer: GPT-6 Astra Ultra (`/root/quant_adapter_astra`), 2026-10-20.

## Scope and provenance

The complete 659-line copied implementation, complete 233-line synthetic check source, README, checkpoint and final saved synthetic result were read. The first two saved result hashes and their checkpoint/README summaries were checked; their test processes were not rerun. Bounded named frozen source excerpts and the accepted adapter's input/output manifest code were compared with the new source. No campaign trace, raw model input/response, actual result corpus, evaluator, analyzer, candidate, parser, build, test, API, network or controller operation was accessed/executed. This review writes only this new Markdown.

| Input | SHA-256 |
| --- | --- |
| `CHECKPOINT.json` | `71c013cb4025846edba992f74c67779d1df41287e902a9e4a90d2ad54f9714d5` |
| Copied and active `extract_dp1_dp4_mechanism_evidence.py` | `7b4499dfeb5d5160c501245817f9135d9c136dfcd5909abbdee2844a67a2a477` |
| `test_fake.py` | `3acab706f40ce4ab6d5e546e73b9cb3386c8907aeb677c321401b470188c5bf0` |
| `README.md` | `0da6edd99640aee1471b97d33a01211b808ceabb5839ef545b247613ed37256a` |
| `fake-results.json` | `b98cbe8556761a69faad1f61f3a0dc3617d2584858eeac3d39848f5d540a8bcc` |
| `fake-results-attempt-02.json` | `cc53b43f490fae183c388e4b5449b308e3a72861129d93a0cc8ca4c9793b210d` |
| `fake-results-attempt-03.json` | `7bff8ff20047c30d0cad74f7d928a79c8b6e00ca9fefa6e3c7532681bb95a7d6` |
| `J/postanalysis/planning/mechanism-evidence-readiness-v1.md` | `071f5872010214aca45f168c585ac1e323f7c303187693f69c45d8e0796e85ea` |
| Accepted `adapter_preparation/stage-03/report_adapter.py` | `c0e369bab675d96cd2cbeb15c79523d89841cdc0a3e91bc2b12a477b94fe77a0` |
| Frozen candidate `src/evidence.rs` | `7a2a4f53e45c65d57efe2d08ab1bd18b4be27ae5b5f08dc38a6d19c671c16ab9` |
| Frozen candidate `src/handoff.rs` | `22290f705503ccb85c7ace2ae5a41f55f162abd054e6fa917748c1d57e7e2b1d` |

Unprefixed preparation files are beside this review. `J` is `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1`; frozen candidate files are under `J/frozen/source/prototypes/candidates/common9_poc_20261019/`. The active tool and accepted adapter are under `scripts/architecture/common9_postanalysis/`. Implementation line references below refer to the immutable stage-01 copy.

Saved evidence reports **20 unique final fake cases**, all passing in the final 0.016552082961425185 seconds with 65,865 bytes of named fake input. Three saved attempts contain 60 check executions, not 60 unique cases or actual campaign runs. These results do not establish the missing cases identified below.

## Findings

### MS1-01 — P2: account and cap every input read; preserve failed-boundary read receipts

**Locations:** 145–179, 477–511, 535–552, 561–577, 631–635.

The preliminary pinned input-manifest read at 564 is repeated at 508 and only the second is charged. The accepted report is read by `pinned_file` before its bytes are added at 552, so it bypasses the aggregate precheck. A file growing after `stat` can also consume more than the aggregate remainder because the read is bounded by the per-file cap rather than the remaining aggregate allowance. Rejection after the read does not restore a hard read bound.

Additionally, `load_boundary` owns its Reader locally. If a later boundary validation raises after bounded files have been read, the assignment at 577 never completes; `execute.reader` remains `None`, so the saved rejection receipt omits that attempt's input inventory and byte count. This is a separate consequence of the same accounting ownership problem, not evidence that no files were read.

**Repair:** share a retained read context from preflight through failure; avoid the duplicate read or charge it explicitly; route report metadata through the same aggregate gate; limit reads against the remaining budget and preserve attempted/failed read accounting. Bound or explicitly account for source-self metadata too. Keep rejected-attempt output immutable. Add tiny fake cases with the aggregate budget near exhaustion during metadata loading and a late boundary rejection; inspect the saved rejection receipt rather than only the successful helper result.

### MS1-02 — P2: occurrence association pointers name the wrong JSON location

**Locations:** 308–316, 398–406, 428–454.

The occurrence tuple carries a capsule/record-list pointer `p`, but its `association` is a different object. Line 441 projects the association at `p`, and line 454 repeats that for decoded inputs. For an actual ingress, the association is the whole request at `/events/i/data/request`, while `p` is `/events/i/data/request/contract/capsule`. For a receiver request it is `/contract/mandatory`, while `p` is `/contract/capsule`. For `work_records` input it is `/mandatory`, while `p` is `/work_records`. Any hashed prose in these associations therefore receives a nonexistent or wrong raw/decoded pointer. The correct outer user-text pointer does not repair the wrong inner pointer.

**Repair:** carry the association's own exact pointer in the occurrence tuple and use it in both projectors. Keep record pointers unchanged. Check a purpose/constraint string in each envelope against the exact source location, including an array-form user message. This needs no semantic rejudgment.

### MS1-03 — P2: partial processing can leave COMPLETE collections or abandon planned rows

**Locations:** 182–184, 251–255, 335–350, 357–368, 582–615, 616–632.

All collections are set COMPLETE before extraction. Some malformed-but-parseable evidence escapes inner handling: for example, `causal_trace.parent_ids: null` raises at 368, or a committed `supporting_passages: null` raises at 348. The outer per-run catch updates the row status but leaves unprocessed collections COMPLETE. In particular, occurrences can remain empty/COMPLETE although their final linking pass was never reached.

Also, `identity` validates only `rep`; an index identity with a list-valued population/configuration produces an unhashable tuple at 585 and exits through the outer rejection path before the 441 rows are written. A serialization failure at 616 likewise escapes the per-run handler. Once the exact plan has been accepted, malformed individual evidence should not remove all later planned identities from the sidecar.

**Repair:** validate identity scalar types before key use, classify malformed index entries as explicit conflicts, and track collection progress. On an interrupted run, mark all affected/unprocessed collections unsupported/partial rather than complete. Contain per-row serialization/schema failures and emit a bounded failure row so all remaining planned identities survive. Preserve already extracted facts without asserting empty complete collections. Add tiny malformed-call, malformed-record and malformed-index cases, checking row/collection statuses and the full planned-row count.

### MS1-04 — P2: consumer keys can inflate repeated-generation exposure

**Locations:** 353–369, 398–406, 455–473; `test_fake.py:123–140`.

Receiver consumer keys combine `call_uuid` with the entire projected `causal_trace`. Identical copies deduplicate, as the saved fake case checks. But two copies with the same UUID and conflicting causal metadata become two distinct keys and increase `distinct_receiver_operations`. Two UUIDs for the same explicit operation node also become two operations. Conversely, absent identities can collapse to the same null-valued key. None of these cases is flagged as identity conflict or unestablished operation identity. Similar unvalidated ingress event-ID reuse can hide conflicting occurrences.

This integer is the proposed evidence for repeated consumption; it must not count inconsistent copies or a call/operation naming ambiguity as additional consumers. A recorded dispatch remains useful raw evidence even when its unique consumer identity is unresolved.

**Repair:** distinguish raw call occurrences, unique dispatch UUIDs and unique receiver operation nodes. Use explicit canonical operation identity when established; preserve UUID/node disagreements as conflicts and exclude unresolved identities from any certified distinct-operation count, with their coverage separately reported. Check ingress ID consistency similarly. Keep failed dispatched operations, unsent inputs and built-only capsules in their existing distinct categories. Add contradictory same-UUID, same-node/different-UUID and missing-identity fake cases; do not require any successful receiver outcome.

### MS1-05 — P2: reuse provenance may join a later original acquisition

**Locations:** 275–278, 318–350.

`acquisition_links` searches every source read in the run without a consumer-event bound. A reuse event followed by the sole matching acquisition node/span will therefore receive a COMPLETE acquisition link and an attributed owner. A record's supporting-acquisition links have the same issue. The extractor has recorded event indexes available, but discards them for this join. The frozen path reads and emits `SOURCE_RANGE_READ` before returning that passage for reuse or producer input; a subsequent acquisition cannot establish the earlier passage's origin.

**Repair:** supply the reuse/producer consumer position and restrict acquisition candidates to an established prior acquisition. Preserve unknown or ambiguous links instead of attributing ownership from a future match. Retain explicit operation/parent references and do not use timestamp proximity as a substitute. A producer commit-event order can race with consumption publication, so do not generalize this read-before-use rule into an unsupported universal commit-order claim. Add a future-only acquisition fake case and retain the existing ambiguous/prior-read cases.

### MS1-06 — P2: DP1 producer input projection omits its actual originals and delivered history

**Locations:** 54–56, 389–397; frozen `evidence.rs:317`.

The actual frozen `evidence_publish_relations` input contains `observed_evidence` and `actual_delivered`. Neither is in `INPUT_FIELDS`. This stage is explicitly selected, yet its original-source and actual delivered-history input is only listed among unprojected fields. The model request remains linked, so the omission is disclosed; nevertheless it leaves the central planned DP1 input-evidence gap unfilled while the call's extraction can be complete. The frozen DP4 producer instead uses `actual_evidence` and `actual_public_state`, which the implementation already supports.

**Repair:** include the exact two DP1 producer keys with the existing source/span/prose-digest treatment. Do not add generic semantic interpretation or new model-stage discovery. Add a source-shaped fake producer request using these actual keys, so the tests exercise the frozen ABI rather than only invented `work_records` examples.

## Sound boundaries retained

- Exact 315+126 plan membership, integer repetitions, pinned freeze/approval, formal-root/symlink checks and declared quiescence checks precede actual trace reads. Execution authority remains root's separate external GO; this review does not establish live quiescence.
- The accepted adapter's `JOINT441_QUANTITATIVE_INPUT_v1` / output-manifest shapes and relative `calls.jsonl` role are supported. The cost link is explicitly reference-only: validating recorded hash/size metadata is not independent verification of the calls file or ledger. No ledger recalculation is needed for these repairs.
- Records are matched by full producer-backed payload to distinct commit/record references, not `record_id` alone; duplicate generations remain ambiguous. Built capsules, actual ingress and dispatched receiver-input exposure have separate phases. Branch-plan IDs do not create consumers, and unsent requests do not become receiver supply.
- Full trace/request caps, one-run processing, no runtime imports, no image decoding, exclusive new output directory and `xb` output writes are useful safeguards. The accounting/failure fixes above are still required.
- No mechanism inactivity is certified from an empty event set (`trace_complete_for_absence_claims` remains false). No semantic use, causal savings, new QA endpoint, changed eligibility, product completion or winner is calculated. Keep those limits.

## Acceptance scope and handoff

Preserve this stage and all three saved attempts. Repair only the new sidecar/tests/README in a fresh preparation stage, with a new source/checkpoint hash and targeted tiny synthetic checks if root authorizes them. The accepted adapter/assembler, frozen sources, campaign, presentation and existing evidence remain unchanged. A fresh independent static closure is needed before source acceptance. **Actual all-441 extraction remains forbidden until controller quiescence and a separate exact root GO.** Honest final reporting may still leave mechanism judgments unresolved if this optional sidecar cannot establish a required link.
