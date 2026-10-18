# DP04 authored screen observation inputs

These documents were authored for the numerical PoC. `build_fixtures.py` creates editable SVG documents and rasterizes them to PNG with bundled sharp. They are not screenshots of user documents, a recorded user study, or a native-app/Windows capture validation. A/B consume the identical PNG bytes and the same authored transcript/pointer/source-event ledger. Neither candidate reads the SVG authoring source or evaluator oracle.

| Episode | Pixels and observation contract | Expressions/revisions |
| --- | --- | --- |
| Dense cards | Twelve similar cards with visible natural item labels. Two pointer locations at the two referring spans. | 2 at r1 |
| Free graph region | Eight labeled scatter samples, unsupported native selection. A closed pointer path encloses a nonrectangular intended sample group. | 1 at r1 |
| Page navigation | Original page A and later page B retained. Pointing and expression precede scrollbar drag/page change. | 1 at r1 |
| Partial correction | Three charts, then a transcript that preserves the first target and replaces the second. | 2 at r1 and 2 at r2 |

All object IDs used for exact-set scoring are visibly printed on the document. They identify source extents and are not a supplied answer set. Coordinates use 1000×650 frame pixels. The oracle is a separate evaluator-only JSON file; its eight expression/revision judgments are fixed before the campaign. No candidate receives the expected sets.

`late_result_replay` replays an **actual earlier-revision returned proposal** after registering the newer revision, before the next model work. It emits `REPLAYED_CONTROL_EVENT`. This checks the revision gate using an authored control event. It does not manufacture a model delay, emulate real network timing, or establish an A/B latency advantage.

## Executed scope of B

The PoC uses the same fixed Qwen model in an explicit screen-parser/producer role and a separate speech producer. The binding resolver receives stored producer candidate revisions and their coverage, competing/unknown regions and original source references. Complete unique typed relations may bind in code; remaining supported semantics use the model. Missing extents go back to the screen producer, which rereads original PNGs. The binder never reads pixels and is prevented from adopting IDs absent from the screen candidates. This is a **Qwen parser variant** of producer-owned exchange. No separate trained OmniParser service or its published efficacy is evaluated.

The default B producers run sequentially. B4 is a separately labeled parallel producer tactic. A4 supplies the same screen parser as auxiliary input to joint grounding; neither is silently installed in the basic runs. A rechecks the entire current binding set on each completed revision. B also regenerates both producers across completed revisions; cross-revision partial recomputation remains a later tactic. Reuse of the prior accepted binding and immutable source identity is essential correction behavior.

## Boundaries

Source reads, actual model calls, candidate publication, binding, first adoption, stale rejection, display proxy and goal completion are emitted separately. Actual text display remains the common UI's responsibility; the Rust callback is explicitly `DISPLAY_PROXY`. Voice/ASR/TTS and Downstream Agent task execution are outside this fixture.
