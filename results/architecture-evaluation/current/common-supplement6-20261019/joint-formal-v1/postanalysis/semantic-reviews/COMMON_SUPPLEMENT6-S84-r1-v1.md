# S84 r1 bounded semantic review

Reviewer: GPT-6 Astra Ultra (`/root/method_astra_review`). Status: postlaunch diagnostic only. This review covers ordinal7, `SUPPLEMENT-r1-S84`, in the explicitly authorized seven-configuration bundle. It is not a metric reevaluation or a campaign approval.

Full immutable evidence and all14 response/all7 goal rows: [COMMON_SUPPLEMENT6-S84-r1-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON_SUPPLEMENT6-S84-r1-v1.json) (SHA256 `7d1ba48bea1b26b9864ca999ef8658ea26594c37aa6759a5d66a970712ba62a6`).

## Result and coverage

| Dimension | Scoped result |
| --- | --- |
| Actual inputs | 14 provided and captured |
| User-visible responses | 0; all14 delivery obligations `NOT_OBSERVED` |
| Displayed-body judgment | 0 PASS, 0 FAIL, 14 `UNREVIEWED` because no body exists |
| Required corrected goal | All7 unmet; final Request maps empty |
| Adopted goal relation | All7 `UNREVIEWED`; no adopted relation to grade |
| Post-adoption revision guard | All7 `NOT_EXERCISED` |
| Approval and Agent handoff | Not applicable; no PASS credited |
| Workflow failures | 12 bounded joint-grounding failures; 2 AABA observation cancellations |
| Actual final-response anchor | 0; all7 used the declared missing-final fallback |

The observed missing responses and unmet goals remain in their fixed populations. A raw proposal is neither a delivered answer nor an adopted current referent set. Goal-relation rows are not extra independent successes.

## Actual request, source and screen

U01: `표시한 두 버튼의 행 이름을 알려줘.`

U02: `직전 요청을 정정할게. 지금 표시한 두 버튼의 행 이름을 알려줘.`

The frozen image and actual raw circles designate B03/B04 (설계, 검토), then B03/B05 (설계, 구현). Each utterance supplies two `pointer_path` cues, each17 points, with `button=none`. Native selection is unavailable. The paths are not clicks or a single `pointer_move`. Actual request payloads retain these two cues and the same `sup-sparse-v1` frame metadata; all inspected joint requests contain one image URL and the actual utterance. Original image base64 was not decoded or copied into this review. The exact frozen PNG was visually inspected.

The source `SUP-SCREEN` stays at revision1. Its lines map B03 to 설계, B04 to 검토 and B05 to 구현. All7 initial source records match the frozen source and permit local reading while prohibiting external export. Actual `SOURCE_RANGE_READ` or `EVIDENCE_ORIGINAL_REUSED` records and model request passages establish access to lines1–12. Catalog presence alone was not treated as a read. There is no source revision transition here; the user changes the current target set.

The frozen functional expectations agree with these actual requests and circles. No oracle-only content requirement was added. The user asks for row names and does not request document opening, modification, export or Agent work. No such adopted action or Agent ingress is observed. This is an observation of the exercised path, not certification of unexercised action/approval enforcement. Internal AAAB handoff-record production is not an actual downstream handoff.

WIDE changes button size32×32 and row spacing96 together, versus DENSE size16×16 and spacing24. Actual relevant button bounds are B03 `[1400,408,1432,440]`, B04 `[1400,504,1432,536]`, B05 `[1400,600,1432,632]`. Authoring geometry is used only for reviewer cross-check and is not claimed to have been injected as a full scene map. Contract20 lines118–138 define this observation, and line171 limits the comparison to the size-plus-spacing condition.

## All response obligations

Event indexes below are zero-based raw `/events/N`; event IDs are retained. Every row has no observed display and body `UNREVIEWED`.

| Configuration | Obligation | Input capture | Failure | Classification |
| --- | --- | --- | --- | --- |
| AABA | S84-RU01 | `event-7` `/events/6` | `event-88` `/events/76` | Observation cancellation |
| AABA | S84-RU02 | `event-34` `/events/29` | `event-85` `/events/73` | Observation cancellation |
| AAAB | S84-RU01 | `event-21` `/events/17` | `event-56` `/events/48` | Joint relation/target proposal rejected |
| AAAB | S84-RU02 | `event-59` `/events/51` | `event-86` `/events/75` | Joint relation/target proposal rejected |
| AAAC | S84-RU01 | `event-7` `/events/6` | `event-36` `/events/31` | Joint relation/target proposal rejected |
| AAAC | S84-RU02 | `event-39` `/events/34` | `event-66` `/events/58` | Joint relation/target proposal rejected |
| AAAA | S84-RU01 | `event-7` `/events/6` | `event-36` `/events/31` | Joint relation/target proposal rejected |
| AAAA | S84-RU02 | `event-39` `/events/34` | `event-66` `/events/58` | Joint relation/target proposal rejected |
| BAAA | S84-RU01 | `event-7` `/events/6` | `event-34` `/events/29` | Joint relation/target proposal rejected |
| BAAA | S84-RU02 | `event-37` `/events/32` | `event-65` `/events/56` | Joint relation/target proposal rejected |
| CAAA | S84-RU01 | `event-19` `/events/15` | `event-53` `/events/45` | Joint relation/target proposal rejected |
| CAAA | S84-RU02 | `event-56` `/events/48` | `event-81` `/events/70` | Joint relation/target proposal rejected |
| ABAA | S84-RU01 | `event-7` `/events/6` | `event-36` `/events/31` | Joint relation/target proposal rejected |
| ABAA | S84-RU02 | `event-39` `/events/34` | `event-66` `/events/58` | Joint relation/target proposal rejected |

## Six joint routes: wrong proposals and contract rejection

AAAB, AAAC, AAAA, BAAA, CAAA and ABAA each make two attempts for U01 and two for U02:24 raw proposals. All select B01/B02, rather than the actual B03/B04 or corrected B03/B05. They all claim `point_inside` with both actual path cue IDs. This is a raw target/relation error, with no displayed wrong answer.

Every repair request includes the actual prior rejection `One claimed designation point required`; the final workflow failure is `Joint grounding contract failed after bounded reread: ["One claimed designation point required"]`. Frozen `grounding/wire.rs:196–205` requires exactly one actual `pointer_move` for that particular relation. `grounding/joint.rs:4` also permits relation `other`; therefore these observations do not establish an impossible interface in which all valid circle interpretations are rejected. No prompt or validator change is proposed.

## AABA: correct raw U02 IDs, invalid full binding and unfinished work

AABA actually performs local parser, screen-producer, speech-producer, binder and expansion work. Its U01 does not reach an observed final binding. For U02, raw speech output names the correction but declares `point_inside` with `S84-U02-path01` and `S84-U02-path02`. The binder response at `/calls/7/response_file` (`1d0a0b0b-81c1-48bd-b5d0-0f9cfe9c691d-response.json`) proposes `SUP-SCREEN:v1:B03` and `SUP-SCREEN:v1:B05`. These IDs match the corrected set.

That ID match is not a valid full binding. Event-73 `/events/63` records `Point needs unique cue` and expansion, consistent with frozen `grounding/binding.rs:142–151`. The recorded producer extents also do not cover the actual button x range1400–1432: B03 is `[1094,387,1354,441]`, B05 `[1094,567,1354,621]`. Thus correct names/IDs must not be promoted into correct complete grounding.

WORK_CANCELLATION_ACK is event-82 `/events/70`. U02 failure is event-85 `/events/73`; U01 failure is event-88 `/events/76`. Both are `OBSERVATION_DEADLINE_CANCELLED_NO_STATE_EFFECT`. Final event-89 `/events/77` contains empty requests, delivered history and ingress. Some already admitted raw producer output is recorded after the cancellation acknowledgment; that is not adopted current state or a delivered answer. The classification is mixed processing incompletion plus an invalid relation and inaccurate intermediate extents, with a correct unadopted U02 target-ID set. It is not solely a displayed wrong-answer event.

## Current relation, fallback and limits

| Configuration | Final Request state | Required corrected goal | Adopted relation |
| --- | --- | --- | --- |
| AABA | Empty | Unmet | UNREVIEWED |
| AAAB | Empty | Unmet | UNREVIEWED |
| AAAC | Empty | Unmet | UNREVIEWED |
| AAAA | Empty | Unmet | UNREVIEWED |
| BAAA | Empty | Unmet | UNREVIEWED |
| CAAA | Empty | Unmet | UNREVIEWED |
| ABAA | Empty | Unmet | UNREVIEWED |

All7 receive U02 through the frozen fallback: its anchor is exactly actual U01 provision+30s, its authored input starts+2s and source root follows6s speech. No actual RESPONSE_FINAL is present. The fallback is not a fabricated successful prior response or proof of the intended completed-response arrival condition. Both requests remain in the population.

The previously reviewed S83 r1 likewise had no adopted/displayed response. Its AABA U02 raw producer-bound targets referred to B03/B04; S84 AABA names B03/B05. This is a bounded intermediate difference between these two observed paths. It is not QA02 success, an architecture winner, a general screen result, or proof that spacing alone caused a difference. Button size and spacing both changed, and only one repetition is reviewed here.

No new technical contradiction was established by these scoped source/input/proposal checks. Strict technical eligibility and primary timing/set results were not recomputed or imported from saved evaluations. No tuning, retry, stop or metric filtering follows from this note. All raw failures and costs remain.

No raw trace was newly read, no corpus scan or executable evaluator/parser/model/build/test/network operation occurred, and no runtime/contract/fixture/frozen-source file changed. This is a new diagnostic artifact. Next ordinal8 requires separate root authorization; the cursor is not advanced.

## Immutable provenance

| Artifact | SHA256 |
| --- | --- |
| [Formal freeze](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/freeze.json) | `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` |
| [Accepted bundle](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S84-r1-v3-r2/manifest.json) | `b5d88886eb8f41e2aeaabd19eeec2f2edcd3ba193c5dbeb13c9b92689e6dbc29` |
| [Evidence](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S84-r1-v3-r2/evidence.json) | `78890c5890ee6838cfc0a79992d1c63548832f4f7e13ebc28326469d81d3b699` |
| [Worklist](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S84-r1-v3-r2/worklist.json) | `407c3b8f943f8202d73c0003842897ef20b8f08c354a5929cda843c5ebaf9088` |
| [Index](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/block-indexes/COMMON_SUPPLEMENT6-S84-r1-v3-r2/index.json) | `0a6eca55f45ef46026abdcbd405cd5320ff4be00419d6ee0c8ab4c59a0253f16` |
| [Execution receipt](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/review-cursors/COMMON_SUPPLEMENT6-S84-r1-v3-r2/EXECUTION-RECORD.json) | `1c33f354c97d24a9cba6826d927928d3a9ba9fb61f8ecdd139dd0e78fa2142a9` |
| [Prior S83 review](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON_SUPPLEMENT6-S83-r1-v1.md) | `eb52e66b1b1ecca7a80a1f36e1e2a2a229db7fd41d8114a222f37ef0fedcf1b8` |

All frozen fixture/oracle/source, consumed model request/response and raw event identities are retained in the JSON. The approved receipt records prepare0.16504s and extraction0.21680s; extraction issues/omissions are zero. Those saved execution facts are not a new evaluator run.
