# Closed-block extractor v3 r2: bounded source closure

2026-10-19. Reviewer: GPT-6 Astra Ultra, `/root/method_astra_review`.

**Decision: the three stage-01 helper findings are closed. Recommend LIMITED GO for root-scheduled preparation and extraction of the single next frozen block, COMMON_SUPPLEMENT6 / S80 / r1.** This is readiness for an UNREVIEWED evidence bundle. It does not certify model output, primary measurement eligibility, QA01 endpoints, semantic correctness or campaign completion.

## Exact reviewed generation

| Artifact | SHA256 |
| --- | --- |
| `source-checkpoint-v3-r2.json` | `c369d57b7f4c4890c260f9e11a9c2a97154422b3d6c6194924d1d3d149e8d953` |
| `v3_preparation/stage-02/manifest.json` | `46fece2b6b547a1212083548643f55d210836c768862a40f90098f986c2e0a3e` |
| `extract_closed_block_v3.py` | `b844729dd0235b07af9565bd9cdf56b062eaad5336ed6e3a6f27af3f0519b95c` |
| `prepare_block_index_v3.py` | `320e0e04b2cbcb7cbb759e40368a3e52a3f68979b759ded13397a7300976e228` |
| `schema_v3.py` | `ff7dffd96d0ee57ff30ddea7cb8cdb78494e8330a776eb3980688d47275b5b93` |
| `synthetic-final.json` | `892622ca14d8c327ce824f4eeb0a97fd4ed6c1d40e7126fb3722a56e0a0e3c4e` |

All 17 stage-manifest entries match their recorded hashes and sizes. The reviewer read the changed source, source differences, saved checks and preserved failed intermediate. The saved final result reports 34 passing synthetic checks in 0.3731387918815017 seconds. No checks were rerun.

The initial failed growth test and its exact source generation remain under `failed-test-generation/`; its result SHA256 is `d845233ca547590276270ff076a01967a83fb88ad29965bcdb588b8b689dc0c5`. The saved failure was an empty captured-read list caused by the temporary root alias. The final test normalizes that synthetic root and observes the intended two-byte bounded read followed by growth rejection. This is helper-test history, not an experimental episode.

The stage-01 review remains unchanged: `static-astra-review-v3-stage01.md`, SHA256 `7444ed31e3b64019d2f67275c2812368b5733693cbefe51b17e8a5970e6880b5`.

## Finding closures

| Finding | Exact repair and evidence | Judgment |
| --- | --- | --- |
| V3-01, null flags on nonzero controller termination | `schema_v3.py:128–130` requires both terminal flags to be present and boolean or null. Original line bytes retain null, return code and stderr. The added controller-shaped synthetic case preserves null and rejects a missing flag. | Closed. Null remains unknown and is not converted to false. Missing stdout run-ID evidence still cannot select an alternate run. |
| V3-02, ordinary cursor review association | `schema_v3.py:82–83` requires `ADVANCE_EXACT_PREVIOUS_V3_REVIEW`, exact freeze, next ordinal, previous block and the previous index/bundle/review hash map. The prior index/bundle association checks remain. The saved ordinary-cursor case first rejects bootstrap-only authority, then uses explicit advancement authority. | Closed. This binds root's exact advancement decision without requiring semantic PASS. The narrow S02 bootstrap remains separate. |
| V3-03, cap checked after read | `prepare_block_index_v3.py:109–116` checks the requested maximum against remaining budget before prefix/range reads. `extract_closed_block_v3.py:75–99` checks verified size before reading exactly that size, records attempted/returned bytes, and rejects growth or truncation. `schema_v3.py:147–179` applies bounded reads to checkpoint dependencies. | Closed for the fixed source generation and bounded input path. Saved near-cap and growth checks exercise rejection before over-reading. |

The extraction-body source difference is the exception-accounting wrapper and reviewed Reader repair, with the r2 checkpoint pointer. Callback projection, frozen expectation separation, typed context, missing-trace obligations, unknown-clock ambiguity and no-saved-evaluation behavior remain as reviewed in stage-01. Accepted v2 bytes remain preserved.

## Exact next-block recommendation

The original 143,974-byte formal freeze was read as bounded metadata. Its SHA256 remains `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd`; binary SHA256 remains `5aea60533c5b2a46c0e22cf8a7e4d86d51b4ba2c3860f6006341f0f073dd2e1d`.

- Next ordinal: `1`.
- Block ID: `SUPPLEMENT-r1-S80`.
- Population: `COMMON_SUPPLEMENT6`; episode: `S80`; repetition: `1`.
- Frozen job indexes: `[7, 8, 9, 10, 11, 12, 13]`.
- Frozen configuration order: `AAAA`, `BAAA`, `CAAA`, `ABAA`, `AABA`, `AAAB`, `AAAC`.
- Preceding block: ordinal `0`, `PRIMARY-r1-S02`, `COMMON9`, `S02`, repetition `1`, indexes `[0, 1, 2, 3, 4, 5, 6]`.

Root should supply and hash the explicit `ACCEPTED_PRE_V3_S02_BUNDLE` cursor and its narrow authorization before preparation. Its historical references are:

| Previous artifact under original joint-formal-v1 | SHA256 |
| --- | --- |
| `FIRST-BLOCK-S02-SUMMARY-v1.json` | `f63972c180de28326ce5b626e0c094822041e734688bac9c5c33d32a584d992f` |
| `postanalysis/blocks/COMMON9-S02-r1-v2/manifest.json` | `58fba33fba4df73dc659965967b8daf3dc76f18681cfd3243776d44f67a6c770` |
| `postanalysis/semantic-reviews/COMMON9-S02-r1-v1.json` | `4050441f651a25c72e127c77a8ea3b50f0c714e4129ff2e7b17430dae6138025` |

The previous review is the all-seven scoped review. Its unresolved dimensions and nonapplicable approval/handoff dimensions remain unchanged. No v3 S02 termination index is fabricated. `previous_index` remains `{ "status": "NOT_AVAILABLE_PRE_V3", "ref": null }` for this one bootstrap only.

Preparation may capture one prefix of at most 1,048,576 bytes from the exact original progress file and reread only the seven selected complete-line ranges. Only a successful new index with its exact resulting hash may feed the one-block extractor. A missing-prefix, malformed-identity, missing-run-ID or cap diagnostic stops this attempt without selecting another block or rerunning an episode. Missing indexed traces retain planned obligations as incomplete evidence.

Use fresh nonfrozen output directories, for example `postanalysis/block-indexes/COMMON_SUPPLEMENT6-S80-r1-v3-r2/` and `postanalysis/blocks/COMMON_SUPPLEMENT6-S80-r1-v3-r2/`, after root chooses and authorizes their exact paths. Keep the 16 MiB per-file and 128 MiB per-command read caps, no saved evaluator argument, and the existing initial 1-second execution policy. An exceeded bound does not authorize an automatic retry or cap increase.

## Limits and actions not performed

The reviewer did not read the active progress prefix or any new actual trace, create an actual index/cursor/bundle, run extraction, run tests, call an evaluator/model/parser, build a candidate or alter frozen measurement files. The current campaign continues unchanged.

The bounded prefix cannot establish absence of contradictions appended later. Final quiescent identity reconciliation remains required. Raw first matching display is still distinct from a frozen evaluator-selected endpoint. Semantic review of this new S80 bundle begins only after actual extraction and its manifest exist; every new judgment starts UNREVIEWED.
