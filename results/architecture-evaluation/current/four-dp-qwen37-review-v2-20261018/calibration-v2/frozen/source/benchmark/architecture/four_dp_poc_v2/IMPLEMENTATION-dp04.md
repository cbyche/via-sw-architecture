# DP04 review-v2 implementation and pre-call contract

## Scope and common observations

The four original episodes, raw source documents, timestamped transcript/pointer events and expected referent sets are retained. The v2 oracle JSON is exactly equal to v1. Native selection remains unsupported; no native object bounding boxes are injected.

`assets/screen/build_fixtures.py` additionally renders raw event overlays from the same source SVG plus only recorded pointer coordinates/path/event IDs. The actual original and overlay PNGs, transform and hashes remain available. Both A and B receive the same original+overlay images. Overlay labels name events, never inferred targets. Page frames without events retain their original image. Candidate code does not read SVG authoring source or oracle files. An offline external-service double reads SVG as a TEST-ONLY vision stand-in; that is not a provider/model capability result.

## Basic structures

- A: actual original+overlay pixels, transcript, events and prior accepted binding enter the joint writer. It produces direct extents and final bindings. One bounded owner reread handles output/geometry contradictions.
- B: screen producer publishes candidate extents/cue relations; speech producer publishes expressions/count/intent. Basics dispatch screen then speech. Complete/unique non-navigation relations can bind in typed code; otherwise the actual model binder consumes the candidates. Final count and geometric checks apply to both paths. Needed new/repaired extents go back to the screen owner once. No final raw-image bypass by the binder.
- B1 cross-input cache, B2 partial regeneration and B4 initial parallelism are disabled in BASIC. A1 crop/cache, A2 partial patch and A3/B3 incremental interpretation are absent. A4/B4 code paths are explicitly flagged and excluded unless later authorized as separate phases.
- All actual stages, rejected proposals, repair calls, producer snapshots, outputs and failures remain recorded. Rejudgment can increase calls/cost and can end unresolved; it does not silently make the answer correct.

## Mechanical checks and limits

`geometry.rs` checks unique ready IDs, matching expression IDs, finite frame-bounded model-produced boxes, explicit semantic count, claimed point-in-box and loop sample-center-in-polygon relations using the ORIGINAL raw coordinates. Relation and cue types must agree; a region path must mechanically close within 2 pixels and contain at least 4 points, otherwise owner repair or unresolved is required. It does not know expected IDs/coordinates and cannot prove that a plausible model bbox really corresponds to its visual label. If the model's own extents and raw events disagree, the owner can reread once; exhausted inconsistency is unresolved, with GOAL_FAILED and no successful progress/completion callback.

For B final count, cue IDs and relation come from the validated speech producer and remain in the published adoption, not the binder's reinterpretation. For A it comes from its joint semantic output. In both paths a narrow common raw-expression recognizer also enforces explicit numeric+item/point units and one/two/three/four or Korean 한/두/세/네+개/점/항목. This is declared limited lexical validation, not general semantic parsing; ambiguous/unsupported wording stays model-dependent. Both may still semantically misread speech, falsely claim a consistent location or choose the wrong visible ID; evaluator-only exact-set checks preserve those failures. Source/revision gates and first-adoption scoring remain common.

## Output capacity before actual calibration

| Stage | Output token cap | Compact fields |
| --- | ---: | --- |
| A / A4 joint and bounded reread | 2600 | intent<=120chars, evidence<=100chars, visible header<=60chars, no reasoning chain |
| B/A4 screen and source repair | 3000 | <=30 candidates, short headers, explicit bounds/coverage, no reasoning chain |
| B speech and output repair | 1400 | intent<=120chars, meaning<=100chars, all expressions/counts |
| B binder | 2000 | selected candidate IDs only, evidence<=100chars, no new extents |

These are role-specific capacity limits, not equal-cost calls. All stages and failed/retried usage count toward the shared budget. Actual Qwen protocol/capacity calibration remains required before formal freeze. The local mock proves shape/flow, not that Qwen will obey these limits or be accurate.

## Callbacks and causal goal linkage

Input capture occurs before source/model work. Model calls are scoped to the grounding goal and record actual dependencies. A successful binding produces the actual IDs plus visible labels where available, source/frame revisions, extents and semantic handling intent. Then progress and completion callbacks run. Grounding/adoption/display is DP04's complete VIA scope; no fabricated business task is inserted to separate QA01/03.

Correction updates the same goal. GOAL_START is emitted after actual semantic adoption but refers to the original first input capture even when revision1 failed and revision2 later succeeds. Failed gate/recovery paths explicitly fail; missing callbacks are not successes. Actual late result test still replays a prior real proposal with REPLAYED_CONTROL_EVENT; no fake provider delay is introduced.

## Offline evidence (no API calls)

- `cargo test --offline --manifest-path prototypes/candidates/four_dp_poc_v2/Cargo.toml dp04`: 11 unit checks passed after the calibration-v2 correction.
- `python3 benchmark/architecture/four_dp_poc_v2/check_dp04_contract.py`: 18 actual Rust acceptance/recovery/late-adoption-origin checks passed. Normal A/B on all4 cases, injected bad initial geometry with actual owner repair on all4, and initial revision failure followed by correction success for both A/B.
- Evidence: `results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/offline-dp04-contract-checks/checks.json`; raw OFFLINE traces are retained separately, excluded from model measurement populations.

## Calibration-v2 correction provenance

Calibration-v1 source/raw remains unchanged. Five DP04 submodules were updated from the independently checked scratch patch in `analysis/dp04-calibration-fix-v2/`: relation/cue type enforcement, retention of B validated count/cues/relation, and user action/topic/scope intent prompts. Role output caps are unchanged. Eleven Rust checks and eighteen full offline probes passed in the scratch copy; actual provider recalibration remains required.

## Proposed QA04 feature evolution acceptance, independent scratch copies

Baseline must freeze before patches; C01/C03 remain format controls. No B benefit is assigned in advance.

### C05 delivered referent explanation source query

After a successful displayed adoption, query `(expression_id, input_revision, adoption_id)` returns only the actually delivered target's source ID/revision, frame, original extent, cue IDs and source snapshot. Rejected, unresolved, superseded or never-displayed proposals are not explanation records. Querying two successive correction revisions returns their separate delivered records; a fake/unknown adoption returns NOT_FOUND. Same acceptance applies to A/B; their actual producer/binding provenance must participate. Candidate records are not read from the evaluator oracle.

### C06 source-region withdrawal

An external authorization event revokes one actual rectangular source region after a valid two-region adoption. It must invalidate intersecting raw-derived target/candidate/binding use, retain the disjoint allowed region, reject a held earlier publication under the current policy generation, and disallow rereads/model transmission of revoked pixels. Any model rejudgment receives an actually redacted/permitted source view. The authorization rectangle comes from the external policy event, not an expected target or oracle. Whole-source revision/permission gates remain intact. Scratch acceptance must demonstrate overlapping region rejected, disjoint region retained, old proposal rejected, no revoked pixel/read admission, and idempotent repeated withdrawal. Actual changed operating modules/diff and regressions are counted after implementation; the count may tie.

C05/C06 are new functionality, not merely renamed bbox envelopes. They are not silently included in BASIC performance.
