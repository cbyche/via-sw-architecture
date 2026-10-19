# Closed-block evidence helper v2: bounded independent static review

Date: 2026-10-19. Reviewer: GPT-6 Astra Ultra, independent method reviewer.

## Decision and scope

**LIMITED SOURCE ACCEPTANCE** for one initial COMMON9 S02 repetition-1 block, all seven configurations, with no saved-evaluation argument. Root authorization remains required before extraction. This review did not execute the helper, tests, evaluator, analyzer, candidate, parser or API, and did not inspect any additional raw run. It does not certify an extracted bundle that does not yet exist or authorize other closure formats.

The running joint441 campaign, its frozen sources and inputs, QA semantics, denominators and existing evidence remain unchanged. Extraction produces review evidence with semantic judgments UNREVIEWED. Raw first callback is not a certified QA01 endpoint or a correctness judgment.

## Exact reviewed checkpoint

| Artifact | SHA256 |
| --- | --- |
| `scripts/architecture/common9_postanalysis/source-checkpoint-v2.json` | `5b7528dcd26c238705976b088b0ca87c7c02cb8e7cbdcebe088cee41c212fb3c` |
| `extract_closed_block.py` | `108f4df6e70469902a9f6096003206639e4e0bda94cb609f59d0509b73252a75` |
| `bundle-schema-v2.json` | `c42655a58adb9ff0bba0224be175cbe39e7111528c24b0fed2f8087ec7d8bd2c` |
| `README.md` | `a52d92b18bc9c6f20ac1d05cdeab29372791755b80d18a428108f379192d1179` |
| `test_extract_closed_block.py` | `1c1fd85e2622d6f5b8e7c7ed8571ad46eb163bce22913bcd8251e3822a5bde42` |
| `synthetic-check-v2.json` | `782cf0c4b6ee062c0b90ef75118b945abc5ca81a11b4fb55fb92bd6c222d5a64` |
| `preparation/source-v1/manifest.json` | `251340643634f015450bb7fb93cdf3b2292a64bdb4b71264957dfd90a3b06468` |

The checkpoint's files, byte counts and hashes were independently read and matched. The saved synthetic result reports 20 passing checks in 0.10980620793998241 seconds. Those checks were read, not rerun. V1's exact source and its subsequent NOT READY judgment are preserved.

## Six v1 findings

| Finding | V2 source decision |
| --- | --- |
| Unknown callback clock could falsely select a first callback | Closed. `extract_closed_block.py:296` requires every matching display clock to have exact integer type. Missing, boolean, non-integer clocks, earliest ties or duplicate earliest IDs yield ambiguity and no selection. |
| Substring CLOSED accepted unsupported closure | Closed for the explicitly supported initial S02 ABI. `:189` checks exact status, root-relative filename, population, episode, repetition and fixed count shape; planned configuration membership and row uniqueness are checked. Missing rows remain explicit missing evidence. |
| Saved evaluator identity was treated as sufficient provenance | Closed by bounded scope removal. `:181` rejects every supplied saved-evaluation index. Primary eligibility and selected QA01 callback remain unavailable, and no background-progress endpoint rule is recreated. |
| Root JSON Pointer was incorrect | Closed. `:49` produces the empty root pointer and escapes child tokens. |
| Readable output lacked a hash | Closed. `:345` hashes exact UTF-8 `review.md` bytes and size with the other outputs before writing the final manifest. Output creation is exclusive. |
| Failure/effect context was absent from row summaries | Closed. `:132` retains explicit `WORKFLOW_FAILED.data.event_id` and `STATE_EFFECT_COMMIT.data.data.stimulus_id` links. Typed run-level failure, effect, rejection, cancellation, publication and authority references remain visible. Unmatched events are context, not inferred causal associations. |

## Actual S02 closure and input evidence

The bounded read of `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/FIRST-BLOCK-S02-SUMMARY-v1.json` matched SHA256 `f63972c180de28326ce5b626e0c094822041e734688bac9c5c33d32a584d992f` (4,074 bytes). Its exact status is `FIRST_FIXED7_CONFIGURATION_BLOCK_CLOSED_NOT_CAMPAIGN_COMPLETE`, with COMMON9/S02/repetition 1, seven closed episodes, 441 planned joint episodes and these explicit run IDs:

- `AAAA-S02-r1-fea606`
- `BAAA-S02-r1-dd201f`
- `CAAA-S02-r1-dc0030`
- `ABAA-S02-r1-632b7d`
- `AABA-S02-r1-3f78b4`
- `AAAB-S02-r1-96c198`
- `AAAC-S02-r1-a7db79`

This summary is an explicit membership index, not cryptographic evaluator certification. At extraction, each available trace must independently satisfy the helper's run/configuration/episode/repetition/FORMAL/binary/path and closed-process checks and receive its own exact file hash. This static review did not pre-validate the other six raw traces.

`candidate_input_scope` explicitly identifies the candidate input file as **SINGLE_INITIAL_JSON_OBJECT_ONLY**. It is not evidence that later streamed source, Agent, cancellation or control input was present. Later evidence remains in trace events, provided-source records and recorded raw references. Actual input/source context and frozen oracle descriptions are separate. Recorded model requests and responses are file/hash references, not inferred from oracle strings or copied as a new prompt.

All trace event order and indices are retained. Compact projections expose omission markers and original pointers for large text, arrays and encoded images. A reviewer must follow the original reference when the compact view omits evidence needed for a judgment. Absence from a short row timeline cannot establish absence from the run. The later semantic reviewer must keep actual-request correctness separate from literal oracle-field match, including the documented RU02/RU05 requirement ambiguity.

## Remaining limits and next gate

The helper reads explicit files with a 16 MiB per-file and 128 MiB total cap. It does not perform corpus discovery, import an evaluator, decode base64, contact a provider or modify campaign artifacts. Output immutability and references preserve provenance; they do not establish the correctness of response content, authority or completion.

No source blocker remains for the stated initial extraction. Root may authorize that single block with saved evaluations omitted. After extraction, inspect the immutable manifest and compare the AAAA/S02 evidence against the existing independent manual-reference review before accepting the bundle as sufficient for semantic work. Any unsupported closure, size limit, identity conflict or missing evidence must stay explicit; do not weaken checks or silently substitute another run. Broader block support and joining final certified endpoints are separate future review scopes.
