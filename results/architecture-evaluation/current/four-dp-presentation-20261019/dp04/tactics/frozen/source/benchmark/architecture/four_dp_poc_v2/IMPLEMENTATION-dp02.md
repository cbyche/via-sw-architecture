# DP02 review-v2 implementation and development corrections

Both basics consume identical actual utterances/sources and share interpretation, source/revision
validation, completion validation and one owner repair per semantic stage. No evaluator oracle or
expected Task/intent mapping is supplied to candidate code.

## Actual operational ownership

- `dp02.rs`: DP facade and dispatch.
- `dp02/state.rs`: captured/public conversation state, adopted Request revision, request-owned
  semantic data, pending relations, delivery and attempted revision bookkeeping.
- `dp02/interpretation.rs`: actual current-utterance multi-intent model contract/admission.
- `dp02/response.rs`: detailed response/fast path, actual supplied quote and shape checks/repair.
- `dp02/execution.rs`: A conversation step boundaries versus B request worker lifecycle, revision
  publication gate, capture schedule, Task answer dispatch/receipt and actual UI callbacks.

These are runtime responsibilities, not virtual modules counted from function names. QA04 future
patches start from the same frozen baseline and count actual modified modules, even when tied.

## A and B

A adopts later input at the current real model-call step boundary. It does not wait for an entire
multi-stage request merely because the first model stage completed. The previous conversation
step can be supplied explicitly as retained context. B continues the original request in its own
worker and may interpret a later input concurrently within the same total K. Public snapshots
contain adopted relation/scope/targets/handling/revisions and actual delivered content; unshown
`complete_answer` and `source_claims` drafts are not copied into another request's public context.
The current worker may read its own private draft; A's retained-step input is explicit.

Both keep the same captured ingress registry. A model interpreting event U2 sees capture text only
through U2, not the raw contents of subsequently captured U3. The full ingress/pending registry is
still retained to prevent publishing an obsolete result while a later correction is unresolved.

A single utterance can contain CONTINUE and NEW. CORRECTION updates its actually referenced
Request, preserving original raw request and all non-corrected conditions. ANSWER must name an
actually displayed question and its Task. IDs do not supply the expected semantic relation.

## First content and completion

Actual adopted handling is displayed as soon as that model decision is admitted. The existing
adoption stage first resolves objects/relations/handling. A naturally one- or two-sentence complete
answer may use a one-call fast path. A detailed multi-part comparison returns null for that optional
field and continues through the real detailed response stage. Candidate code never truncates a long
answer or invents later work just to create a QA01/QA03 gap.

Model caps: intent/adoption 2000 tokens, detailed response 2600; initial + at most one owner repair
per stage. Repairs receive the actual rejected proposal, source ID/quote and contiguous-substring
failure reason. Source metadata is not evidence. Exact quote validity does not prove the semantic
correctness of the displayed answer; evaluator-only finite output conditions are separate.

Every response attempt records its actual goal revision. A later revision can legitimately resume
an earlier completed/failed revision. Exhausting repair on the same revision is terminal and cannot
be silently retried by the post-join reopen path. Failed attempts and their actual times/cost remain.

## E4 goal scope

R1 runs from original capture to its final required revision 2. U2 correction is a QA01 input, not a
second QA03 goal. Q-MAIL-1 is a QA01 Task-question notice, never a new QA03 goal. U3 is explicitly
`ANSWER_DELIVERY_TO_EXISTING_TASK` for T-MAIL: interpret recipient, actually dispatch it to a fixed
acceptance shim, verify matching Task/question receipt and display. It is not the lifetime or domain
completion of the original email Task. The authored user wait precedes U3 and is not removed from
unrelated active R1 work.

## Fixed finite populations

Per one A or B sweep: ten QA01 stimuli and eight QA03 goals. Three repetitions of A/B yield QA01 60,
QA03 48. Evaluator manifests are frozen separately and still count missing inputs/goals after errors.
N-R is an exact per-utterance multiset of relation/target/question/Task associations, including
unexpected extra intents; it is not QA02 referent-set accuracy. Finite content checks use the final
required goal revision and actual GOAL_COMPLETE delivery, with numeric/word boundaries.

## Calibration-v1 preserved defects and v2 changes

The 12-row calibration-v1 is retained immutable. It exposed: a same-revision post-failure reopen
that made an extra completion attempt; U2 interpretation seeing later U3 text; omitted goal_id in
auxiliary EPISODE_RESULT.outputs despite correct actual callbacks; and invalid ellipsis/user-event
source citations. The next snapshot fixes those common A/B contracts and preserves every original
failed attempt/cost. The first-stage fast-path scope was clarified before formal execution. No
fixture-ID answer or expected source values were inserted. Source utterances/data are unchanged.
