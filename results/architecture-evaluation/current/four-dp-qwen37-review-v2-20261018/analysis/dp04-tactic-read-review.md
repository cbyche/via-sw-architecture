# A4/B4 final-baseline read review — no execution authorization

Read-only review of the existing paths under `basic-v1/frozen/source/prototypes/candidates/four_dp_poc_v2/`. No source changes, candidate execution, compile or API calls. Proposed scope is subject to root approval after BASIC; this document does not schedule runs.

## Verified path invariants

- `src/dp04.rs:44` A4 calls the **same** `producers::screen()` as B and then `joint::ground_with_parser()`. Final binding writer remains A joint; it receives actual original+overlay PNGs, raw events, prior and fallible auxiliary screen state. It never calls B's `binding::resolve()` in this branch. The helper can increase total calls/output cost before first adoption; it does not replace them with a free precomputed fixture.
- `src/dp04.rs:48` B4 starts speech in an actual Rust thread and screen in the current context from the same prior/source parent. It joins both actual terminal nodes before the **same** `binding::resolve()` used by serial B. Only initial independent production is overlapped. Binder/owner expansion remains dependent on actual results; no synthetic delay is introduced.
- `joint.rs:11`, `producers.rs:14,44`, `binding.rs:130` all append the same final `geometry::RELATION_CONTRACT` once per model-call instruction. Role schemas and caps remain joint2600,screen3000,speech1400,binder2000. Both A/A4 use joint owner reread; B/B4 use current candidate/speech final checking and bounded screen expansion. A4's same parser may itself use the existing output-parse repair.
- All four variants share original observations, narrow raw count checking, coordinate/cue geometry, source/current input revision admission, progress/completion callbacks and replayed stale-publication rejection. BASIC disables A4/B4; no intrinsic reuse/cache tactics were silently added.
- B4 producer calls share the bridge's actual `case.k` semaphore (the same default2), not a new per-worker slot budget. Causal join records both branches. Parallelism does not imply reduced token/call cost; provider timing variation and actual call failures remain observable.

## Important runner detail

Actual DP04 dispatch uses **input.variant == A4/B4**. In the existing runner, `--tactic` only stores metadata and `case.tactic`; it does not switch A/B into the A4/B4 branch. An approved future schedule must therefore use `--variant A4 --tactic A4` or `--variant B4 --tactic B4`, retaining A/B as explicit comparison base metadata in the separate plan. Running `--variant A --tactic A4` would execute ordinary A. No runner edit is needed if the approved schedule uses the correct existing flag.

## Proposed before/after evidence if approved

Compare the same4cases×3repetitions against their own12 BASIC rows, not against calibration or a dense-only subset. Each A4 or B4 phase then has QA01 **15 stimuli**, QA03 **12 goals**, QA02 **24 first expression/revision sets**. Never remove an unresolved/failure from those opportunities.

Report overall and each-case values side by side: correct/expected first sets; response/completion observed/missing; actual input-to-first/goal-to-final elapsed with failed-attempt latency; every stage's call count, input/output tokens, billed/unknown cost and owner repair; actual same-sample concurrent RSS and incomplete observation; logical buffers as a separate scoped diagnostic. Preserve actual raw request/response UUIDs and freeze/binary hashes. A4 and B4 have separate phase/freeze and never change BASIC120 values.

For A4, a gain diagnoses that **the same screen producer can help A's writer**. It does not alone prove producer-owned B binding superior. For B4, compare actual producer overlap and joined path, and avoid attributing different model target/bbox outputs to parallelism alone. On the correction case, prior accepted state may itself differ after stochastic outputs; record that difference rather than claiming every model request was byte-identical.

## Failure diagnosis to preserve

Use the unchanged QA02 actual first-adoption definition plus parallel labels: (a) proposed target set wrong, (b) proposed set correct but bbox/raw geometry contradicts and is not adopted, (c) count/type/required metadata mismatch, (d) provider/JSON failure. One revision can have b→c recovery history (as calibration-v3 A did); keep per-attempt causes and a final terminal cause rather than adding overlapping counts as disjoint failures. Source-authoring bboxes are evaluator-only. Model self-consistency does not prove the true visual extent. B's final adoption advantage must not be relabeled as pure object-recognition superiority.
