# Independent completed-tactic review

Read-only comparison of the three BASIC and three tactic runs for the same fixed affected case; no new model calls, candidate edits, oracle edits or primary-score changes.

## DP01 A2 / E3

A2 preserved unaffected source-backed working-view fragments after the vendor-quote revision changed. That mechanism is present. Its measured outcome is not a read-reduction result:

- BASIC: two actual `EVIDENCE_READ` events per episode, six across three repeats (criteria v1 and quote v1).
- A2: three per episode, nine total; the extra read is quote v2. No `ROLLING_EVIDENCE_REUSE` event occurs, although the retained criteria passage is actually supplied directly in the next model context.
- Four actual model calls per episode in both configurations. VIA costs rise from $0.0060360 to $0.0061700. QA01 means fall from 8385.31 to 7800.45 ms, but this does not establish that fewer reads caused the difference.
- The changed-input q2 mean falls from 8561.99 to 7578.92 ms; even unchanged q1 varies from 8208.63 to 8021.99 ms. Service/output variation remains in these observations.

In BASIC q2 the selector requested `ANSWER` with no reads based on catalog identity metadata. The host rejected direct completion for missing actual provenance, then invoked final interpretation without a newly requested body read. A2 instead retained actual criteria evidence and read quote v2. Both identify the required source IDs/versions, hence existing QA02 remains 6/6. Their displayed q2 text repeats the referring phrase and does not supply the required frozen summary facts. The separate `semantic-conditions` diagnostic records this omitted content/closure check; it is not erased by QA02 success or nonempty `GOAL_COMPLETE`.

## DP02 A1 / E2

All three side calls decided suitability from actual utterance/context and returned concise TCP/UDP explanations. Existing R1 input/lifetime, shared total K=2 and two actual calls per episode are retained. No event ID or evaluator answer determined the branch.

|Endpoint|BASIC mean ms|A1 mean ms|
|---|---:|---:|
|Independent U2 response|11673.40|4081.31|
|Original R1 response|11803.80|11025.40|

Individual R1 results were 9599.58→6335.79, 17072.00→12227.26 and **8739.83→14513.17 ms**. The third repeat regresses by 5773.34 ms. R1 ready-to-display hold shrinks from roughly 2835 ms to 0.35 ms, which supports removal of the held publication dependency; original model duration/output length still varies. Do not claim guaranteed original-goal acceleration or hide the third repeat behind the mean.

VIA model cost is $0.0104500→$0.0078872. Same-case N-R is 3/6→6/6, but E2 combines “continue that work” with an independent question under one baseline relation label. A1 explicitly handles the side question. This finite contract result is not a general relation-accuracy improvement and does not revise the frozen BASIC N-R score.

A4, C2 and B4 remain separate populations for their final raw/result review. A4 keeps final binding in joint A; B4 is a scheduling diagnostic, not a selected A improvement. Full evidence audit and offline replay checks follow the completion notice.
