# Common9 method v3 independent judgment review

2026-10-19 / **CORE SOURCE REVIEW CLOSED FOR BOUNDED DEVELOPMENT PROBE; PAID GO AND FORMAL FREEZE REMAIN SEPARATE**

Reviewer role: user-requested GPT-6 Astra Ultra, independent method and source judgment. Implementation and test execution remain with GPT-6.1 Sol High. This reviewer has read source and existing records, and has not run implementation tests, a parser, an evaluator or a paid model call. Findings below are static findings unless an actual acceptance record is explicitly linked.

## Scope and preserved evidence

The current contract is [19 common evaluation contract](../19-common9-evaluation-contract.md), with its current interpretation in [17 sections 9 through 11](../17-presentation-results.md#9-예측의-성립-조건과-반대-조건-재측정). The approved primary campaign remains nine fixed cases, seven configurations and five repetitions, for 315 episodes under the total $40 cap. The user subsequently approved a separate six-condition, seven-configuration, three-repetition supplement of 126 episodes, within that same cap; its preregistration is [20 supplemental evaluation contract](../20-common-supplement-evaluation-contract.md). The actual executed v1 and v2 generations remain separate from a fresh primary campaign. Their failures, 315-row plans and raw evidence must not be pooled into the new result.

The [formal-v2 closure](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/formal-v2/METHOD-HOLD-CLOSURE.json) records nine executed and 306 unexecuted episodes, 189 API calls and $0.5198596 for that generation. Total prior spend is $0.6635124 with no unknown or in-flight reservations at closure. The closure's old `section5.3` reference is a historical citation error; the governing observation rule is contract19 **section 5**. This review does not rewrite the preserved closure.

The [deadline correction proposal](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/deadline-correction-design-proposal.md) separates actual response completion, missing-anchor fallback, actual source provision, observation cutoff and settlement. The [newer completion endpoint addendum](common9-newer-completion-endpoint-addendum-20261019.md) is an independent source-provenance correction. The [prediction coverage review](common9-prediction-coverage-review-20261019.md) limits what the fixed nine cases can establish.

## Accepted method interpretation

- A 30-second missing-anchor fallback releases later authored input and retains failure. It is not a response, goal completion or actual Agent source.
- Once all mandatory source obligations are actually provided or recorded missing, the observation cutoff is the last successful actual source pipe-flush end plus 60 seconds. Authored source due times remain the separate QA latency roots.
- A late actual mandatory Agent source can extend the window only before the old cutoff. The host checks the old gate before and after its actual flush. A crossing or post-cutoff source cannot revive an expired window.
- Observe the full 60-second window. Cancellation starts settlement, whose deadline remains cutoff plus 120 seconds. No later join, ACK or fallback resets it.
- A bounded in-memory state batch has a logical linearization point under its owning lock and the common effect gate. The check immediately before that point must not perform logging, acquire another lock or do IO before mutation.
- Actual host callback observation is a different boundary. A valid state commit before cutoff does not make a callback observed after cutoff an eligible QA endpoint.
- No new cloud or local inference dispatch is admitted after cutoff. Already admitted work retains its late raw returns, cost or unknown reservation, and RSS evidence.
- Every configuration uses the same clock proof, gates, K=2 limit and cancellation rules. No whole-network-call gate lock may serialize the two slots.

## Findings and closure requirements

| ID | Static finding | Required correction and finite evidence | Current status |
| --- | --- | --- | --- |
| M3-01, P2 | `Context::commit_effect` checks the clock and then synchronously writes `STATE_EFFECT_COMMIT` through the stdout mutex and flush before the caller mutates state. Stdout contention or backpressure can move the actual mutation past cutoff. | Pure immediate clock check, bounded mutation under existing owning locks, then post-mutation audit. Exercise an actual blocked stdout path and confirm no post-cutoff mutation. Also inspect all callers rather than relying on the audit label. | Owner confirmed the finding and is replacing the API. No final source or acceptance proof reviewed yet. |
| M3-02, P2 | `publication` writes `DISPLAY_PROXY` before later delivered-history, counter and completion-status mutations. A single pre-output check does not protect those later commits after blocking output. | Separate publication admission, actual host receipt and later state commits. Do not call an unobserved pending write actual delivery. See the publication decision below. | Root approved the actual host receipt ACK design. Implementation and acceptance remain pending. |
| M3-03, P2 | Local parser admission is checked before `GroundingAdapter.call`, but that call can then wait for its own lock, read/hash files or cold-initialize before starting a process or sending fresh frame inference without another gate check. | Pass the same gate into the adapter. Recheck at actual spawn and new parse admission after owning lock, IO and initialization; do not hold the gate across model work. Keep already admitted work for settlement. Exercise queued admission and cold-ready-after-cutoff cases. | Owner notified. No corrected source or acceptance proof reviewed yet. |
| M3-04, P2 | The draft evaluator's dispatch audit checks admission before cutoff but does not independently prove admission occurred after the verified host-clock handshake. Local model dispatch also needs explicit proof. | Require the actual clock-proof observation to precede each cloud/local dispatch. Retain spawn and parse admission evidence from the adapter. Test missing and late proof. | Evaluator owner notified. |
| M3-05, P2 | The draft evaluator can match any earlier `PUBLICATION` for the same stimulus to a later display. Earlier handling and later detailed output share a stimulus, so a missing final-publication proof can be hidden. | Correlate the specific publication with phase, goal and revision or a unique publication/commit identity. Test an earlier valid handling commit followed by an unproved final callback. | Evaluator owner notified. |
| M3-06, P2 | The host can propose a window extension before the old cutoff while the candidate receives it after that cutoff and rejects revival. The inspected harness only logged a rejected deadline ACK and continued with its extended host deadline. | Explicit technical fail-stop, bounded ACK wait and separate proposed, accepted and actual abort clocks. Never use an uncertified extension for normal dispatch or primary certification. | Root approved the exceptional abort policy below. Implementation and finite acceptance pending. |

Earlier adoption, raw Agent fact, input capture and source-registration checks were placed before their owned locks. The implementation owner reports those paths have been moved after the owning locks. That report is not final acceptance, and M3-01 explains why the shared logging helper still prevented closure at this review point.

## Root-approved publication design, implementation and acceptance pending

The reviewer recommended an actual host receipt ACK for the strict final baseline, and root explicitly approved that design for the implementation and evaluator owners. A post-flush local check prevents a late state mutation, but flush completion alone does not prove the host observed the display before cutoff. The common ACK carries only technical publication identity and actual host observation time, with no semantic judgment or expected answer.

Use a unique publication identity. The host stdout reader records its actual observation time and sends the receipt. Candidate wait must release state, source, effect and stdout locks. Before actual delivered-history, source registration or goal completion commits, the candidate checks the receipt and current gate, authority and revision again. A late receipt leaves the actual raw display in evidence but permits no new post-cutoff authority commit. An actual historical display fact and current-revision completion eligibility are distinct.

This adds a common measured technical path. Include its delay in every configuration instead of subtracting it. Finite tests must cover late, missing and duplicate ACK, cancellation while waiting, changed revision or source authority, and the absence of lock deadlock. Root's design approval authorizes this repair, not a paid probe or the formal campaign. Both paid GO decisions remain pending.

The receipt wait is VIA technical work in the QA03 causal graph. It must remain in the relevant operation and parent path; it is not external Agent execution with zero cost.

## Root-approved exceptional technical abort policy

Normal observation still uses last actual source provision plus 60 seconds and settlement at that cutoff plus 120 seconds. On a clock or deadline-ACK technical failure, preserve the contract deadline, proposed deadline and last accepted deadline as separate raw facts. Record `effective_admission_stop_ns` as the earlier of explicit technical abort time and an already effective earlier cutoff. Exceptional settlement ends at that effective admission stop plus 120 seconds.

Do not replace the contract deadline with an earlier abort time. Such an episode is technically incomplete and cannot become a primary certified observation. During the first deadline-ACK wait, conservatively enforce the proposed bound even though there is no previous deadline. A missing ACK must not leave admission unbounded. An ACK or late update received after abort cannot resume work. Root explicitly approved this policy and directed finite checks before any paid GO.

## Primary reporting when technical proof is incomplete

Root and reviewer agree that technically uncertified traces retain their raw observations but cannot silently contribute certified timing successes. Apply the same predeclared technical predicate to all seven configurations.

- QA01 and QA03 use certified exact endpoint observations in their timing sums and observed counts. Keep the original 215 and 115 planned denominators, and list unobserved, unresolved and technically invalid obligations separately.
- QA02 keeps all 75 planned judgments. Separate technical-unjudgeable rows from ordinary semantic incorrect judgments. A known-correct/75 lower bound is a disclosed observation, not a fully certified accuracy when technical gaps remain.
- Ordinary model failures under valid instrumentation remain primary outcomes. Poor answers are not evidence of an instrument defect by themselves.
- QA05 retains all actual spend, late usage and unknown reservations. A broken observation or settlement window makes the representative cost comparison incomplete; it does not erase spent money.
- QA06 retains incomplete observed maxima as lower-bound diagnostics. Do not present them as a certified complete maximum or sum independently occurring role peaks.

## Fixed compatibility probe sequencing

Root proposes code and negative checks, the unchanged 63-case offline regression and evaluator checks first, then a probe-specific exact freeze and explicit root GO for exactly AAAA/S02 and AAAA/S18 once each. Fresh QA04 42-copy acceptance can follow those development probes on the final stable baseline. Requiring QA04 before a development compatibility probe would not improve the probe's validity and could create repeated QA04 work if a common protocol defect is found.

The plan must freeze call and dollar caps, source and binary hashes, expected protocol classifications and stop rules before either paid attempt. Probe pass criteria concern the ability to express valid common schema, evidence about any repeated common format misunderstanding, and correct validator acceptance or rejection. Full user-goal success, all QA correctness and success-until retrying are not compatibility pass criteria. Preserve ordinary format-valid semantic failures. Additional changes require a documented cause and separate freeze decision. Probe results are excluded from the primary QA population, and their costs remain in the total $40 ledger.

The reviewer subsequently read the [fixed probe plan](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/compatibility-probe-v3-PLAN.md): AAAA/S02 then AAAA/S18 once each, at most 100 cloud calls per episode and 200 in total, with an additional $1 cap including reservations inside the global $40. Its bounded sequence and outcome separation are acceptable in principle. Two clarifications were requested before GO: inspect every actual target/binding string for membership in the actual catalog even if an earlier validation error prevented a syntax rejection, and state that format FAIL/NOT_OBSERVED does not automatically demand retuning or repetition. Root must distinguish a common interface defect from ordinary model noncompliance. Exact freeze, reservation projection and implementation acceptance remain pending.

The final 315-run GO still requires corrected source, actual finite acceptance, all 63 cases, fresh QA04 42-copy acceptance, independent evidence review and a single final frozen baseline. No paid GO is granted by this review.

## Fitness of the nine-case experiment

The common nine-case design can satisfy the user's requirement for the same QA and population across every DP alternative. It cannot, by itself, validate every stated mechanism condition. In particular it lacks independent semantic goals arriving during a long semantic execution, and small dense screen targets. The [coverage review](common9-prediction-coverage-review-20261019.md) established those omissions before the new outcomes.

Disclosing the omissions is necessary but does not perform the missing test. Before paid formal GO, describe the campaign as a fair comparison on these nine fixed cases, with DP2 independent-goal concurrency and DP3 small/dense-target benefit still unverified. A tie or reversal on these inputs is not falsification of a hypothesis whose necessary condition is absent. Complete QA tables do not establish complete hypothesis coverage or the course's trade-off coverage.

If additional condition testing is later authorized, preregister a separate common supplemental population and give every configuration the same cases and QA rules. Keep it separate from the approved nine-case representative mean, and preserve all current and old outcomes. This review does not authorize changing the nine cases, creating a new paid population or tuning cases for an expected winner. The prior contract18 partial diagnostic generation is historical evidence and cannot certify the missing mechanisms in the new connected baseline.

### Historical optional supplemental proposal, superseded by the separate 126 approval

The earlier broad request for necessary condition tests motivated this proposal. At this proposal checkpoint the later explicit nine-case choice fixed the approved paid population and the addition was unapproved. The user has since approved 126 supplemental episodes. Preserve the proposal below as its rationale; the exact current design and remaining freeze requirements belong to [contract20](../20-common-supplement-evaluation-contract.md), not to the earlier optional alternatives below.

Two reviewable choices remain:

1. Finish the approved 315 episodes; retain the two named unverified hypotheses.
2. Separately authorize the six-condition diagnostic below for every configuration: **6 × 7 × 3 = 126 additional episodes**, or **210** if five repetitions are chosen. Three is a small descriptive repeat design, with no statistical sufficiency or minimum-sample claim. Choose the count before results; do not stop on success or repeat failures selectively.

These are new supplemental variants, not unchanged COMMON-28 cases. Keep their results separate from common9. They address the two omissions, not the full 24-cell prediction matrix.

| Proposed condition | Common authored input and controlled difference | Required interpretation |
| --- | --- | --- |
| SUP-D2-OVERLAP | Relate constraints in a permitted multi-section document; finalize an independent request about a different document 2 seconds after first-request capture. | Require actual semantic overlap and useful K=2 capacity before attributing a concurrency benefit. Observe both goals and queue occupancy. |
| SUP-D2-SEQUENTIAL | Identical requests/sources, but the second follows the first actual final callback plus 2 seconds. | Matched absence-of-overlap control. A missing callback remains failure; fallback release cannot prove this control succeeded. |
| SUP-D2-CORRECTION | Same initial request and 2-second arrival; the later utterance replaces one constraint of that goal. | Dependency countercondition: retain revision, reuse/discard, repair and completion evidence. Two revisions are not independent goals. |
| SUP-D3-DENSE | 12 readable row items with repeated small action icons and compact spacing. Click-free gesture plus speech identifies two items; a later correction changes one. No native selection. | Observe first adopted exact set, candidate coverage/geometry, expansion and failure. Candidate omissions remain outcomes. |
| SUP-D3-SPARSE | Identical items, labels, speech, sets and correction; enlarge targets and spacing, transforming gesture coordinates on the same viewport. | Matched combined small/dense countercondition; does not separately identify effects of size and spacing. |
| SUP-D3-NATIVE | DENSE content and intended sets, with selected-object metadata from an authored app selection instead of click-free pointing. Both alternatives receive it. | Native-sufficient countercondition. Verify actual parser skip; metadata must describe the selection action, not leak an evaluator answer. |

Before implementation, fix exact source bytes, geometry, PNGs, gestures, native events, timing, permissions and evaluator-only oracles. The proposed 12 items and 2-second timing are review choices, not parameters to search after outcomes. Check readability/geometry without selecting fixtures from favorable model trials.

Retain the common candidate, models, K=2, repair and lifecycle rules. Use actual inference without artificial response stalls or B-specific triggers. Absent overlap/capacity means `MECHANISM_NOT_ACTIVATED`, with no automatic rerun. Saturated-slot behavior, arbitrary regions, multiple pages and helper placement remain outside this proposal; B still has its original-image producer input.

All seven configurations use all six QA meanings, fixed obligation denominators, full costs and local helper RSS. Report the whole supplemental bundle plus every condition. QA04 retains the same six change tasks: exact unchanged-baseline evidence may be referenced as such, not counted as new measurements. Supplemental patched-copy acceptance requires actual regressions; candidate changes cannot inherit old acceptance.

Before paid GO, freeze interleaved order, paired repetitions, call caps and a separate reservation inside the global $40, including prior spend and unknown usage. No dollar projection or affordability is asserted here. If the whole plan cannot fit, revise it before execution. Preserve technical failures, ordinary model errors and all planned rows; report raw pairs, ranges and failures without population-significance claims.

## Draft source fingerprints, not final freeze

Read-only hashes captured at 2026-10-19 07:29:36 UTC. Source is being edited concurrently; these identify this review checkpoint only.

| Path relative to repository | SHA256 |
| --- | --- |
| `prototypes/candidates/common9_poc_20261019/src/context.rs` | `11713c0092bc17304bc6eee080aa9fbdb01b6f8161700ab37709dd8093a1a1e9` |
| `prototypes/candidates/common9_poc_20261019/src/runtime.rs` | `d2ad8719b4e422ea134d2ddeea7e56e1cee1e2c2449240beaa3c48715c998349` |
| `prototypes/candidates/common9_poc_20261019/src/lifecycle.rs` | `4a52cb7fa79ae98a6527ee7bff9b0484bad05925310d6462469aa406c0f371f1` |
| `benchmark/architecture/common9_poc_20261019/harness.py` | `6f6606096d97f7d50cc98c71aff5261d2b0e4c2d5b138ef82ab6b67b9d7872ec` |
| `benchmark/architecture/common9_poc_20261019/grounding_adapter.py` | `b5999bca0b60ffdeb55faafa2e760ae1d029456c14d03249e8acd8e44d736c74` |
| `benchmark/architecture/common9_poc_20261019/evaluation_v3/deadlines.py` | `8c9c53d7086a0c5b350fe226f25a1640efcb9e2482bf590d89de936e20d78227` |
| `benchmark/architecture/common9_poc_20261019/evaluation_v3/endpoints.py` | `3cc3984c725bd5d60838dae8586d9760d3ec90a2f1efa3a67ee8a33d63a49b86` |
| `benchmark/architecture/common9_poc_20261019/evaluation_v3/evaluate.py` | `0ef85e5258b496ff278fa1f367f09296bd52949183f74e3144efee9b711074a3` |

Final hashes, actual acceptance paths, finding closures and a narrowly scoped GO or HOLD decision will be appended after the implementation owner supplies a stable review checkpoint. The intermediate S02 offline success and previous QA04 generations do not prove the current repaired source.

## Subsequent evaluator checkpoint

The reviewer read the revised dispatch and publication matching logic and the [draft-04 finite-check manifest](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/evaluator-v3-draft-04/manifest.json). The manifest reports 49 passing checks: the 14 approved endpoint cases and 35 additional cases, with zero API calls. It explicitly includes cloud/local dispatch before clock proof, handling proof incorrectly covering a direct publication, wrong revision, ordinary model failure and technical-unjudgeable reporting cases. The reviewer did not rerun those checks.

This addresses the initial code forms of M3-04 and M3-05. Their final closure remains pending the implementation's publication ID and actual host receipt ABI, local admission records and fresh actual traces. Phase/goal/revision matching is an improvement, but cannot replace the unique publication proof once that ABI is adopted. Final freeze remains **HOLD**.

## Subsequent source-only caller audit

The draft replaced the pre-mutation logging helper with a pure gate check and a post-scope audit record. Inspection then found caller-level gaps: evidence and handoff filters could acquire SourceRegistry locks after the final check; producer version-map construction could do the same; and an evidence eviction loop could emit synchronous stdout events between successive state changes. These are static findings on draft source, not failures observed in a new model run.

The owner reports moving those filters, source validations, version maps and eviction lists before the final check, followed by bounded assignments and only then audit events. The owner also reports a successful compile. This reviewer has not yet accepted those repairs against a stable source checkpoint or actual blocked-lock/blocked-output regressions. M3-01 remains open until that evidence is reviewed. A post-mutation audit label is not independent proof that every operation between the final check and the actual state batch was nonblocking.

The publication draft must also distinguish actual display history from current-revision completion. A receipt for content actually observed before cutoff is a historical fact even if a later input has changed the current revision. It does not authorize completing the new revision or reusing revoked source material. Root approved preserving that distinction through the existing shared authority contracts; a post-cutoff receipt still cannot create new history state. Host raw evidence is retained in either case. A timestamp assigned while constructing a display event is a preparation time, not the stdout admission time or the host observation endpoint.

## QA04 preparation inspection

The reviewer read `qa04_method_v3/PLAN.md`, `patches.py` and `driver.py`. Their preparation-only status is accurate. The copied driver still imports the preserved old `evaluate.regression`; its direct feature Session does not yet implement the final clock, deadline and actual-display receipt protocol. The C03 textual patch anchor also targets the earlier Context formatting. These are pending rebase requirements, not failed new QA04 trials, because this generation has not been executed.

After the candidate and evaluator ABI is stable, the implementation owner must adapt these tools to the exact frozen source, retain clock/cancel/receipt controls through C01/C02/C03 transformations, and use the strict new evaluator for each common9 regression. The feature exercises must also preserve the baseline effect gates when C04/C06 mutate state. Their actual operating module sets and all 42 acceptance results remain unknown. The previous 42-copy generations cannot close this work.

### C04 section-edit transaction judgment

At root's request, the reviewer read the revised preparation plan, copied C04 patch and feature driver, and current baseline publication method. This is a method judgment, not patch acceptance; no tests were run. C04 must preserve the same distinction between an actual historical display and an authoritative current result.

Root approved the transaction semantics below. Implementation, finite acceptance and the final QA04 freeze remain pending.

1. Check the actual target, expected section revision and source authority at common publication admission. Display the proposed revised content with a unique publication identity. Release all input/effect/source/state/stdout locks while awaiting actual host receipt. Publication alone is not successful edit completion.
2. After receipt, acquire the necessary owning locks and recheck the current target section, Task result identity, source revision/authority and cutoff. Prepare all fallible work, IO and replacement values before the final pure gate check. A valid edit then commits the new source revision, specified Task result and canonical displayed section together as a bounded batch, followed by the applied audit. Preserve all other sections, Requests and Tasks.
3. If the target or authority has changed, reject the authoritative replacement. A valid actual receipt may still produce one historical display fact through a fresh pre-cutoff gate, recording publication identity, actual observation time, target and proposed revision, with `applied=false` and the rejection reason. Preserve the original result and current canonical state. The failed proposal is not the revised source or a successful edit. Future use of its historical text must still obey the existing authority checks.
4. If cutoff/cancellation prevents that fresh history commit, retain raw display/receipt evidence only. No post-cutoff history, source or Task commit is allowed. Missing or mismatched receipt cannot be fabricated into delivery. A receipt retry or edit replay cannot append a duplicate fact or apply a second edit.

Rebuild the target-specific update against live state after the wait. Replacing whole Task/history collections with a pre-ACK snapshot could erase unrelated concurrent work even when the target revision still matches. Source registration must not leave a partial authoritative source commit if the corresponding Task/history batch fails; preparing a file is distinct from publishing a source revision.

Finite acceptance must cover normal edit plus actual subsequent read; target section change during ACK wait; source revision change and authority withdrawal; preservation of unrelated concurrent Task/section changes; duplicate or mismatched ACK and edit replay; cutoff/cancellation during ACK or owning-lock wait; and source-preparation failure without partial canonical replacement. These checks apply to the C04 copies and preserve the approved feature meaning. They authorize no paid execution or claim about the pending 42 results.

## Stable core checkpoint review: 1971903306748208

The reviewer read the fixed [core checkpoint](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/method-v3-core-checkpoint-1971903306748208/checkpoint.json) and its snapshot sources, not moving QA04 tools. The manifest identifies 61 source files and binary SHA256 `0c41ac25ecdec102b0b2f2a2f5a49d260e238b7e1c69b9d580adab3e41dfa605`. It explicitly excludes the separately owned evaluator, harness boundary checker and QA04 tools; it is not a complete paid freeze.

| Reviewed snapshot | Manifest SHA256 |
| --- | --- |
| Candidate `context.rs` | `e6ad3d6a2d0cf62e2d9994820cfd906748623f1cab7189f13b7748a60af60de2` |
| Candidate `runtime.rs` | `7efd0963ab50b5e911d2de8b303052330e007f1f8bfea8e0ccaf6aa2b0e7e3d8` |
| Candidate `sources.rs` | `45cbd94a2cc1b8b89c9513ec1e3311fcbddcb42ca7c95ceb0c780e0de74943fb` |
| Harness | `1ba329e43d7a6c4bef8eff11d46c934a3c6baea90d43db9ad6a78910d633f799` |
| Cloud bridge | `27f4200ff02d8c2f0dfca4e7b09c7ebd9ce076b4d1d2c570ebb712c6afb6ce64` |
| Local parser adapter | `2da67f1b92e687a84be02cca61915efe22f37ba5b442a6ac76b4b1b72f8aab1d` |

### Source findings and narrowly scoped repair

**M3-01 is substantially corrected but not closed.** The Context check now performs no audit IO before mutation; source registration checks after its IO and registry lock; permission updates are prepared before the check; evidence/handoff filters and eviction planning precede bounded assignments. Two remaining ingress call sites construct their record with `ctx.last_node()` after the final check (`runtime.rs` lines 475–476 and 520–521). That function acquires the Context parent mutex. Prepare the record and parent identity before the final check. This is a small caller correction, not a scheduler redesign.

**M3-02 remains open for post-publication state races.** The user-goal publication batch correctly preserves actual pre-cutoff history while denying stale completion. Later separate batches still need their own current-state checks: WAIT/CLARIFY writes the question at lines 440–443; Agent-result Task completion writes at lines 599–611. A new revision may be adopted after the earlier publication releases its gate. Recheck revision, Task association and relevant authority under the current owning locks immediately before these later commits. Preserve the actual displayed historical fact when current-state adoption is rejected. Separately registered delivered sources at lines 549–557 and 613–625 must not let a superseded result move the current source pointer backward. Preserve historical versions through their actual version identities; current selection requires fresh eligibility.

The snapshot also exposed a real evaluator/implementation ABI mismatch for `ACTUAL_AGENT_FACT`: runtime lines 629–645 update factual Task state after receipt but do not append user-goal delivered history. Draft-05 `publications.py` line 63 requires `historical_display_recorded=true` for every `PUBLICATION_DELIVERY_STATE`. The reviewer and implementation/evaluator owners agreed to preserve the existing factual Task path and verify it separately using its actual phase, captured Task/event class and matched publication/receipt. Do not invent a history append by adding a true marker, or use this fact effect to prove user-goal completion or a completion-required source registration. The user-goal history/completion distinction remains unchanged. The first suggested append was superseded by this narrower, truthful ABI distinction.

**M3-03 source correction is accepted within the inspected path.** The parser receives the common gate; checks after its own queued lock and file preparation; separately checks actual process spawn, parse dispatch and cache mutation; and retains already admitted raw returns. No inference is newly admitted after cancellation. This is source acceptance with the finite mocked admission evidence below, not a fresh actual heavyweight parser performance result.

**M3-04/05 evaluator logic is improved but final ABI acceptance remains pending.** The [draft-05 manifest and source](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/evaluator-v3-draft-05/manifest.json) report 73 finite Python checks, including 14 approved endpoint cases and 59 additional cases. The reviewer read the snapshot clock/dispatch, publication, completion and primary-summary code. Unique publication/admission/actual-host-receipt matching and separate history-only versus completion proof address the original findings. The factual Task ABI mismatch above must be resolved against actual traces. This reviewer did not rerun the 73 checks.

**M3-06 requires an honest effective-stop clock.** `harness.py` lines 175–183 include an optional `effective_cutoff` in the minimum; rejected ACK paths pass the earlier host detection time. The reader has not cancelled the admission gate at that detection timestamp. The actual `gate.cancel()` occurs later, so the earlier timestamp is not independently proven to be an effective stop. Preserve detection/requested-conservative-bound timestamps as raw diagnostics; calculate effective stop from actual gate cancellation and already effective earlier deadlines. Do not synthesize equal timestamps or widen evaluator tolerance. The resulting technical abort remains incomplete and never primary eligible.

The inspected cloud bridge enforces the $40 global ledger and a shared probe-root $1 subcap under the same ledger file lock, including unknown reservations. Each of the two fixed probe jobs has a 100-call cap, with no automatic re-execution of a started identity. K=2 is retained and the gate is not held over the network call. This source review does not replace the required pre-probe cost projection, exact full freeze or root GO.

### Acceptance evidence read, without rerunning

- Checkpoint command output: 30 Rust checks passed, including actual owning-lock waits and blocked stdout/cancellation checks. These are finite implementation checks, not model performance observations.
- Checkpoint lifecycle output: seven passing mocked guard checks, including K=2, queued/encoding cancellation, unknown reservation retention, local queued/cold-ready/cache cutoff and probe subcap. Zero paid API calls.
- [Actual runtime boundary summary](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/deadline-runtime-1972076614229583/summary.json): 20 passing cases on the checkpoint binary, including normal/cancelled return across seven configurations and missing, late, rejected, duplicate/unknown receipt and authority change cases.
- [Protocol summary](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/protocol-v2-1972082218830000/summary.json): 98 passing cases with zero API calls. The reviewer read its summary and representative rows; this is not an independent replay of all artifacts.
- [Harness boundary summary](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/harness-lifecycle-v3-1972070567144250/summary.json): nine passing actual-binary cases with deterministic model doubles and time scale 0.05. They cover failed-model full-window behavior, clock proof failure, rejected/missing initial and extension ACK, and actual late Agent source boundaries. The effective-stop alias discrepancy above remains a review finding despite the checker's PASS.

All these paths remain evidence for this exact intermediate generation. The running common63 cannot close later source changes by inheritance. **HOLD remains:** apply the narrow fixes, preserve this checkpoint, then review the new exact source and corresponding actual acceptance. No paid probe or formal GO follows from this checkpoint review.

## Corrected stable core: 1972493055557208

This section supersedes the preceding source HOLD for the inspected method repairs. The reviewer compared the fixed runtime, SourceRegistry and harness sources with the preceding checkpoint and reviewed the unchanged Context, bridge and parser gate paths. The [new checkpoint](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/method-v3-core-checkpoint-1972493055557208/checkpoint.json) has manifest SHA256 `4144c1f4d31a216d47f8a0171ab1c7cd815969ff40ca0ea12fdba052b79b21d8` and binary SHA256 `5aea60533c5b2a46c0e22cf8a7e4d86d51b4ba2c3860f6006341f0f073dd2e1d`.

| Finding | Source closure at this checkpoint |
| --- | --- |
| M3-01 | Both ingress records and their Context parent identity are prepared before the final check. Source/evidence/handoff preparation, nested locks and IO precede the final pure check; inspected bounded state assignments precede the audit. Runtime SHA256 is `d96f61406119d870bf65bb9045a8fe2bb8eb1aa68ae8319a65a09845a4437402`; SourceRegistry SHA256 is `346272b30e59705e6ddf7c1fde71b7e6892597336517bc06e1a7192c19758342`. |
| M3-02 | WAIT/CLARIFY rechecks current revision, pending input and support authority under the owning locks. Post-publication Task changes recheck request revision, actual Task/ingress association, Task existence and authority. Guarded delivered/artifact source registration rechecks those conditions and rejects a backward current-source pointer. Actual displayed history remains distinct from current completion. The factual Agent path reports `task_fact_state_recorded`, phase, Task and event class truthfully; it does not claim user-goal history or completion. |
| M3-03 | The previously accepted shared parser admission checks remain present after queued locks/preparation and at actual spawn, parse and cache mutation. Already admitted work remains raw settlement evidence. |
| M3-04/05 | Evaluator draft07 matches actual clock proof before dispatch and the unique publication, host receipt and eligible state effect. Factual Task updates, historical display and user-goal completion are separate proofs. A factual update cannot certify a user goal. |
| M3-06 | Harness SHA256 `d30750622e97978f769da8366a8c12510e91cbd482ead32d1111ed9cc7e079d9` retains ACK detection and requested-bound timestamps as diagnostics. Effective stop uses actual gate cancellation and already effective old/proposed cutoffs. It does not backdate cancellation to detection. Rejected/missing control ACK remains a technical stop; no late ACK resumes it. |

The Task check is intentionally the current request revision plus actual Task/ingress association and authority. No cross-goal Task generation policy existed in this comparison, and one is not required to close the identified post-publication stale-request defect. This acceptance does not establish an additional cross-goal conflict policy.

### Corresponding finite and actual offline evidence

The reviewer read these records without rerunning their commands. The actual tests were run by the implementation and independent evaluator owners.

- The fixed checkpoint retains 32 passing Rust tests, a successful build and seven mocked lifecycle guard checks, including the shared probe subcap and unknown reservations.
- [Actual deadline/runtime boundaries](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/deadline-runtime-1972531067901333/summary.json): 20 PASS on this binary.
- [Actual protocol cases](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/protocol-v2-1972531067886916/summary.json): 98 PASS; [runtime guards](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/runtime-guards-1972533238715291/summary.json): seven PASS.
- [Actual harness boundaries](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/harness-lifecycle-v3-1972558515401875/summary.json), SHA256 `46bbe40b7d05f2e0c210f7fe29c4eff578432f879a201ad534ca0f7997b6dffe`: nine PASS with deterministic model doubles and scale 0.05. They cover the full window despite ordinary model failure, missing/mismatched clock proof, rejected/missing first and extension deadline ACK, and actual late Agent provision before/after cutoff. The owner reports three normal traces consistent and six injected technical traces unresolved, with no strict violation; unresolved technical cases remain uncertified.
- [Evaluator draft07](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/evaluator-v3-draft-07/manifest.json), SHA256 `5a6e77ce4bb5aa2c8f0fb9e59cf0d744ee26e0b32bee9e9b3a43f9ef196e03d9`: 75 finite PASS with exact source snapshots. Previous drafts remain preserved.
- [Fresh final63 independent audit](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/method-v3-final63-independent-audit/qualified-manifest.json), SHA256 `00a5c55ed313dda1894e19852b4ed7aacdd3ceacdc57f133765a36edd6b25651`: `PASS_OFFLINE_PREPARATION_ONLY`. The independent auditor matched 61 core live/snapshot hashes, ten evaluator sources, three unchanged legacy dependencies, the binary, actual commands and source-send brackets. All 63 distinct episodes have consistent strict proof: 301/301 callbacks, 161/161 goals and 105/105 sets; each configuration has 43/23/15. The 63 factual Task effects prove no invented history or goal. RSS is observed for all 63, with 4,699 samples, zero invalid samples and eight confirmed exit boundaries. API calls are zero.

These are finite regressions using deterministic model doubles. They establish the inspected controls and known failure boundaries, not exhaustive thread-interleaving correctness, model accuracy, real parser accuracy, measured winners or QA04 change scope. The evidence is sufficient to close the six identified method/source findings for a bounded development compatibility probe. It does not make the fresh315 or supplemental126 ready to execute.

### Narrow fixed-probe readiness judgment

The reviewer read the prepared two-job freeze and readiness record: exactly AAAA/S02 then AAAA/S18, repetition one, 13 responses, nine goals and two binding obligations; binary `5aea605…`; K=2; 100 cloud calls per episode, 200 total; a $1 additional reservation-aware subcap inside the global $40, including the prior $0.6635124. The assessor independently examines actual proposed target literals against the catalog carried by each actual HTTP request, preserving other protocol/semantic errors. No all-goal-success criterion, automatic replacement case or success-until retry is permitted.

One bounded assessor edge was identified before GO: a present non-array target field must not be omitted merely because another target array is valid. Record that field as one unchanged literal, mark its container-schema failure separately and apply exact catalog membership. A catalog-valid scalar may pass the narrow literal-format check while still failing the proposal schema; scalar `BAD` must fail membership. No splitting, coercion or normalization is allowed. The implementation owner is applying that assessor-only change and focused finite checks. This does not alter the core or require another63, but it requires an updated assessor/plan hash and exact freeze.

**Recommendation:** after the corrected assessor evidence and refreshed exact freeze are recorded, root may authorize only the two fixed development probes. Fresh QA04 acceptance is not a prerequisite for these development calls. Actual prelaunch load and clock qualification remain mandatory. This review grants no paid authorization itself. Root should save an immutable approval artifact referencing the final freeze and this judgment snapshot; the continuing review document must not be used as a moving frozen source.

**Formal HOLD remains:** fresh QA04 42-copy acceptance and independent review, final candidate/evaluator/fixture mapping, supplemental contract/fixture acceptance, a joint reservation-aware projection and complete formal freeze are still pending. The two development outcomes cannot be pooled into primary315 or supplement126.

### Refreshed probe-r2 acceptance

The bounded assessor correction is now source-reviewed. [Eight finite cases](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/development/probe-assessment-finite-1973363451404041/summary.json) pass, including a scalar invalid reference beside a valid array, a catalog-valid scalar with an explicit schema error, a non-string target and a malformed bindings container that is not reinterpreted as a reference. Assessor SHA256 is `af0c11ff1f15b2c1d8ae27968858449ff7b84e0d1f2a9b4a0eaf591e4354d0b2`; finite-summary SHA256 is `b67f61df8f15019c83cfe095820a801674a6f74f5af05c71a9862a8cf217d2e7`.

The reviewer read [probe-r2 readiness](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/compatibility-probe-v3-r2/READINESS.json) and the relevant exact [freeze](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/compatibility-probe-v3-r2/freeze.json) fields, then independently hashed that freeze as `16ca2b14e91fef09ec3c9499913d2e4e7cfd88ea7c4b50586bd2ae83d9960af7`. It has 84 sources and the same accepted core binary, evaluator and final63 mapping. The previous prepared freeze remains preserved with zero dispatches.

**ACCEPTED FOR ROOT'S LIMITED TWO-PROBE GO DECISION.** No source or assessor blocker remains for exactly AAAA/S02 then AAAA/S18 once each, the existing $1 additional/200-call bounds and global $40, actual prelaunch load and per-episode clock checks, with no automatic retry. This is a readiness recommendation, not the authorization record or successful probe evidence. Formal315, supplement126 and fresh QA04 remain pending as described above.

The completed [immutable probe-r2 judgment](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/preflight/astra-probe-r2-readiness-20261019.md) has SHA256 `599f77085bbdef57ea768cbd714cad72e9ce99f8067c99ce9a2d415510729426`. Root subsequently granted the limited two-probe GO for that exact freeze; this does not change the formal HOLD or establish either probe's result.

## Fixed two-probe result judgment

The initial launch was rejected by automatic approval review because the external destination and transmitted data classes lacked explicit human consent. No process or API call ran on that attempt. Root obtained the user's explicit consent for the authored synthetic material and instructions sent to DashScope Singapore within the same $40 scope, and the same normal launch was approved. That approval history is distinct from method readiness; no bypass was used.

The reviewer subsequently read both retained episode summaries, the exact-literal assessment, controller status, and the S18 raw proposals/failure events. [Reference-format assessment](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/compatibility-probe-v3-r2/assessment/reference-format.json) reports PASS for the fixed pair: S02 has12 and S18 has4 actual target-literal occurrences, with no missing catalog proof or canonical-reference rejection. This is the predefined narrow compatibility outcome, not complete semantic/schema success.

| Actual run | API calls / known cost | Technical proof / RSS | Diagnostic QA observations |
| --- | --- | --- | --- |
| AAAA-S02-r1-650ce1 | 12 / $0.0384256 | CONSISTENT / OBSERVED | QA01 6/6; QA03 6/6; no QA02 obligations |
| AAAA-S18-r1-e322ec | 6 / $0.0131236 | CONSISTENT / OBSERVED | QA01 5/7; QA03 1/3; QA02 0/2 |

S18 U02 and U03 actually proposed `DELEGATE` with `completion_condition:null`, despite the instruction's declared string field. The existing handoff guard rejected both with `HANDOFF_ADOPTION_completion_condition_MISSING`; no successful ingress or Agent A01/A02 result was fabricated. The actual missing-anchor records and incomplete user goals remain in the raw evidence. The early meaning validator allowed the proposal to reach that later guard, so zero `MEANING_PROPOSAL_REJECTED` must not be summarized as all protocol/schema success. Wrong/missing referent bindings and Task semantics remain separate ordinary outcome failures.

**Judgment:** these observations are ordinary model output contract noncompliance and semantic failures under consistent instrumentation. They do not demonstrate a broken source-reference catalog, host clock or publication receipt interface. No automatic prompt/source retuning or repeat of the fixed two probes is justified. Preserve the failed outputs and proceed with final stable QA04 acceptance and the separately preregistered supplemental implementation under root's existing scope. A newly demonstrated technical defect would still require a documented separate decision.

The pair spent **$0.0515492** over18 calls; the controller closed with global known spend **$0.7150616**, unknown/in-flight reservations0 and no active paid job. Both probe rows remain development-only and are excluded from the fresh315 and126 populations. Their internal evaluator `ELIGIBLE` flag describes trace certification, not permission to pool them into a formal result.

## Approved supplemental preregistration

The user-approved six-condition, three-repeat design is now specified in [contract20](../20-common-supplement-evaluation-contract.md): S80/S81/S82 independent overlap, matched sequential arrival and same-goal correction; S83/S84/S85 small dense, larger sparse and actual authored native selection. It fixes source text, requests, timing, screen geometry and raw cues,12/8/6 obligations per bundle, separate36/24/18 per-configuration denominators, and a predeclared block order interleaved with the unchanged315 relative order. Three repetitions carry no statistical sufficiency claim.

The implementation owner's read-only feasibility check found no need to change the existing candidate/evaluator ABI. A separate actual-provided trigger runner and population-specific aggregation adapter are necessary. Actual fixture/asset/font hashes, finite and connected acceptance, the additional six QA04 regressions per real patched copy, joint budget projection and formal paid GO remain pending. The supplemental inputs were specified independently of the development probe's success/failure and must not be adjusted to manufacture predicted wins.

## Joint441 review snapshot closure

The later [supplement source and formal-method review](common-supplement6-astra-review-20261019.md#10-공동441-review-snapshot의-최종-방법-검토) closes the supplemental implementation work described as pending above. It records the atomic final/fallback scheduler repair, final42 offline acceptance, separate12/8/6 aggregation, bounded activation interpretation, controller/order/budget review and existing human egress consent. QA0442-copy/630-regression acceptance remains separately owned and is not claimed complete here.

The exact review-only joint freeze SHA256 is `779f811570da28e0503d4398c0d459379199f94124a8c8d2b394e370fd7816fd`. An independent read-only digest audit found all112 live/frozen source pairs, the binary and copied projection consistent. All77 sources shared with the accepted probe are unchanged; all34 final supplemental checkpoint sources are included unchanged. The441 identities and63 complete seven-configuration blocks preserve primary315 order and the preregistered supplemental rotations. No authorization file exists in this review root. No test, evaluator or API was executed by this reviewer.

The user consent expressly covers authored synthetic inputs/screens/instructions for2+315+126 through DashScope Singapore within $40. Current egress proof SHA256 is `5a6334d229e133ebe684024168b8aa50762dec0d97be7ef4f24878dc50da7964`. The inspected payload paths contain authored data, current synthetic state and model-derived results, with no acquisition of actual personal files, repository source text or secrets into model inputs. Existing consent applies; it does not itself authorize the future formal launch.

One final provenance requirement remains for the new formal root. Bind the still-operative newer-completion endpoint addendum (`fce69c1b86cd8a7002a77b07265b6b7d83660b2135ff42d46f5dcc8af27d950f`) and deadline correction proposal (`238f7472c9d2c452680f977483d4c997870aa65f6ea13071b86c3fb5a4450369`), both absent from the112-source review snapshot, together with immutable copies of the accepted method/supplement closures, final QA04 acceptance/review, egress proof/consent and the exact new formal freeze/projection. The proposal retains its original design status; accepted subsequent receipt and technical-abort refinements are documented in the closure. No runtime edit, additional regression or new destination permission follows from this packaging correction.

**Conditional source/method acceptance:** no remaining inspected source blocker. Preserve this review snapshot, finish actual QA04 acceptance and the specified artifact binding, then obtain root's exact formal paid GO. Ordinary planning $30.3569 fits the existing441/$40 scope with the fixed block stop, while MAX3 $47.7324 discloses incomplete-campaign risk; no policy or population reduction is authorized. Formal execution remains unstarted in this review, and no primary/supplement winner or prediction success is claimed.

### New formal root contract staging

The [staging review](common-supplement6-astra-review-20261019.md#11-새-formal-root의-계약-staging-검토) independently matched the new formal freeze `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` and contract-staging manifest `761a0cb5b2963fd64345e705ebbff3c0219e9279662589f96e536ab251005af8`. All18 copied contracts/evidence files match their recorded hashes and sizes. All112 source pairs, binary,441 jobs, block amounts, projection and22 dependency records retain the accepted review generation. The two operative-contract omissions are now closed by exact immutable copies. No authorization/final-binding file or formal raw trace existed at this review; the actual ledger remained $0.7150616 known, zero unknown/inflight and a $40 cap.

This staging is accepted with actual QA0442/630 and its corrected final report, independent raw audit and final QA04 judgment still pending. The staged count-v2 document preserves its pending-at-creation status; its later scoped acceptance belongs in the new final binding with the exact QA04 review hash. Preserve incomplete RSS diagnostics separately from the unchanged QA04 functional regression predicate; do not relabel or rerun teardown samples to create OBSERVED memory results.

The final immutable approval binding must connect the freeze, staging and copied artifacts to completed QA04 evidence and independent closures. Root's GO/authorization must identify both freeze and binding hashes. The unchanged controller does not execute the descriptive `final_approval_binding_requirement`; root's recorded prelaunch step must actually verify the binding and copied artifact bytes. This is a provenance/approval step with no new runtime change, test campaign or human destination permission required. Full dependency validation, current ledger and host isolation checks remain required at actual launch. No paid GO is granted by this review.
