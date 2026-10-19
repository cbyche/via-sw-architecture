# COMMON9 evaluator and review plan

This implements the common9 contract19. Candidate code never imports oracle.json or this evaluator. The independent runtime supplies authored inputs, actual source times, stdout-reader observations, bridge timing/usage and same-sample process RSS. No model outcomes were used to choose these rules. Prior DP-specific populations and campaigns are not pooled here.

## Commands and APIs

```sh
python3 benchmark/architecture/common9_poc_20261019/check_causal_timing.py
python3 benchmark/architecture/common9_poc_20261019/evaluate.py RUNS.jsonl --output EVALUATION.json
python3 benchmark/architecture/common9_poc_20261019/analyze.py RUNS.jsonl --output ANALYSIS_DIRECTORY
```

`evaluate(trace, episode_oracle=None)` returns the episode's raw judgments and diagnostics. `regression(trace, episode_oracle=None)` returns PASS/FAIL, checks, counts and the evaluation; the QA04 runner aggregates the9 episode counts43 responses,23 final goals,15 exact sets. This predicate proves an offline Rust/protocol regression with semantic doubles. It does not establish model accuracy, complete semantic correctness or performance.

## QA01 and content review

Every required response remains in the population. The first eligible actual DISPLAY_PROXY callback after its actual source time is measured. Empty output, generic acknowledgement or unspecified work promise, exact echo of the user's utterance, orphan references and unresolved deictic-only promises are excluded. An actually displayed resolved target card can make the handling concrete. Actual Agent facts may repeat the received source fact. The explicit background-progress exception may use a newer actual same-Task completion and keeps the earlier progress source time. Task identity must come from actual captured metadata.

`COMMON9-ENDPOINT-v1` is a deterministic textual proxy filter, not a universal semantic judge. Each rejected/accepted raw text and reason, first accepted raw text and required content are retained for independent review. Wrong concrete content retains its time. Unexpected generic paraphrases or uncertain substantive content require review against this same fixed rule for all configurations; any rule change requires a new evaluation version and applies to every preserved output, with both versions retained.

The `semantic_diagnostics` ledger contains three separate examinations:

- Body conditions: each required response's expected conditions, every actual display and actual adopted plan. A missing display is AUTO_VIOLATION. Present prose is NEEDS_REVIEW until independently checked; literal phrase matching is not semantic proof.
- N-R: expected original goal and member inputs, current/corrected Task/revision and actual relation/control decisions. NEEDS_REVIEW covers whether a new request was wrongly attached, a correction changed the wrong Task, or an approval answered the wrong question. This does not expand QA02.
- N-H: actual Agent/control ingress wire, the preceding adopted plan and fields preserved into the mandatory request. Literal preservation may be AUTO_CONFIRMED_CONTRACT_FIELDS_ONLY; a direct field mismatch is AUTO_VIOLATION. Purpose satisfaction, exclusions and permission semantics remain NEEDS_REVIEW with raw recipient input/result available in the trace. Unknown wire formats remain NEEDS_REVIEW rather than receiving a fabricated normal form.

The review statuses do not change QA01/03 numerators or create a new composite score. Automatic host/workflow/publication failures and exact QA02 failures remain separate. A report must publish unresolved review counts and cannot describe all timed outputs as correct.

## QA02

Each fixed oracle row is evaluated once at the first SEMANTIC_ADOPTION for its finalized input. The expression matches whitespace-normalized exact text or exactly one trailing Korean case particle. Plural scope and target versions are not repaired using the answer key. Multiple matching bindings are ambiguous and wrong. The adopted target set must equal the expected canonical source/revision/extent set; no missing adoption is removed. Duplicate targets remain a functional diagnostic. All configurations use the same15 judgments per repetition.

## QA03 causal graph

All vertices use the host monotonic domain. Rust event times are stamped at stdout line observation, bridge times at enqueue/send/terminal and source times at authored/actual source end. Raw Rust elapsed_ns remains provenance. IPC observer overhead is included in this common host proxy; no precise Rust/host offset is invented.

For each model call, the graph contains operation start→enqueue→serialization completion→actual slot acquisition→send→terminal→operation end. Older traces without detailed queue markers retain a combined dispatch/queue interval. VIA dispatch, actual slot queue, service and decode/IPC intervals remain counted. RECEIVER service is zero, but VIA's actual wait for an occupied shared slot is resource contention and remains. A resource predecessor joins from its service terminal at current acquisition; it is not serialized after its entire decode. A historical owner already released before the current caller began waiting is not a causal parent. For actual contention, the scheduling tail after both release and current wait-start remains counted. Queue and predecessor paths merge by MAX, so full queue time is not added a second time to predecessor service.

Local source reads and response publication remain counted VIA operations. A multi-parent join counts only the glue after the latest ACTUAL parent endpoint. USER_WAIT/speech and EXTERNAL sources are zero-cost endpoint joins, not serialized duration nodes. An external source can depend on ingress and an actual receiver receipt inside its external interval; both excluded service intervals stay zero while parallel VIA survives. Actual DISPLAY_PROXY/RESPONSE_FINAL/ingress/receipt points retain their publication/decoding prefix before a source anchor.

Final completion must match the oracle's original input and final required member source, occur after that source, and use the current adopted goal revision at that time. A premature completion cannot close a later correction. The original corrected goal keeps its original source-end root. A new goal clips prior work's counted intervals to its own source-end root. Only actual causal/resource ancestors contribute; wasted work is not made causal merely because it has the same goal ID. The runtime must explicitly retain a dependency when old work is consumed.

Every required goal-member source must actually have been provided. The original source vertex must be an actual ancestor of the accepted completion. Missing source identities or parents, missing call timestamps, reversed edges and cycles are UNRESOLVED rather than zero. The raw wall endpoint and critical path remain available. The required six synthetic examples, idle historical owners, resource-release/receipt overlap and missing-data cases are recorded in `check_causal_timing.py`.

## QA05/06 and population

QA05 includes every recorded call, including preparation, unused production, failures, repair and receiver calls. Role totals, actual usage and unknown reservations are retained. A local helper has zero API charge only; its measured initialization/inference and RSS remain, while monetized local compute is NOT_MEASURED. Unknown usage is never assigned zero cost.

QA06 takes the largest observed complete same-sample sum of registered Python+Rust+helper RSS. It never sums role/episode peaks. A ps failure, zero sum, missing registered active process or absent core Python/Rust registry invalidates a sample. The peak's actual process list, invalid/incomplete samples, sampler failures and maximum sampling gap are retained. An episode with some complete samples and any invalid samples/failures is INCOMPLETE; its observed peak is only a lower bound. All-invalid sampling is INVALID and has no peak. Missing helper observations cannot be hidden by calling the sample complete. Remote weights/KV are outside this local diagnostic.

The parent approved `COMMON9-RSS-v2-PREFORMAL` before formal measurement: a candidate alive at sample start but with exit0 confirmed by sample end is separately CONFIRMED_EXIT_BOUNDARY. Its raw rows/PIDs/lifecycle and exclusion count remain, but the sample does not enter the peak and is never treated as a normal zero-RSS sample. This requires positive RSS for every other registered process; a missing live helper, uncertain exit or ps failure remains INVALID. Complete positive observations of a sample crossing that confirmed boundary are likewise conservatively excluded. The remaining maximum is a **nominal50ms sampled observation**, not a guarantee of the true continuous-time peak. The actual sample brackets/gaps remain available. The same policy applies to every configuration.

The subsequent pre-formal `COMMON9-RSS-v3-PREFORMAL` retains that rule and handles an actually present PID observed with RSS0 during exec startup. STARTUP_ZERO_RSS requires successful ps, every registered PID present, actual Popen/registration timestamps for each zero-RSS process, and independently observed later positive RSS in the same episode. The evaluator derives first-positive observations from raw rows, not asserted candidate flags. The startup sample retains and includes its actual numeric0 in the same-sample sum; it is separately tagged/counted. A required PID never observed positive makes its affected samples invalid/incomplete, including an otherwise confirmed exit boundary. Missing PIDs, uncertain spawn/exit, ps failure and active0 after a prior positive observation remain invalid. Python's already-running process has no invented spawn record and cannot use the startup exception. PID registration and initialization are not delayed to avoid the zero observation.

Summaries retain215/115/75 planned denominators per configuration and5 repetitions. Missing episodes, late failures and partial means are explicit. Duplicate or unexpected episode/repetition identities invalidate scored means; a favorable duplicate is never selected. OFFLINE and FORMAL phases are separate. Paired case/repetition intersection timing is auxiliary and does not replace the full population. QA04 comes from its independent six-change evidence. No score-band revision or winner is inferred by the analyzer.
