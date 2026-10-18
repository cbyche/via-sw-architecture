# Calibration-v1 source-backed display contract diagnosis

2026-10-18. Author: delegated GPT-6.1 Sol High implementation agent. No additional provider calls.
Original calibration source/binary/requests/responses/failures/costs remain immutable.

## Mechanism causing rejection

The original guard required each `facts.statement` to be a literal substring of displayed `text`,
in addition to the actual quote/current source/line checks. Prompts explicitly asked for this
duplication, but valid paraphrases did not consistently repeat their evidence metadata verbatim.
This is an unnecessary field-synchronization contract, not proof of a semantic answer failure.

- DP01 `calibration-dp01-e3-A-r1-d50640`: q1 described the adopted tax/maintenance criteria,
  tie-break, purchase uncertainty and actual v1 quote amount/maintenance/delivery/quantity.
  All six quoted source spans matched the actual supplied current lines. Five statements differed
  literally from the paraphrased displayed proposal. Initial and repair were rejected. q2 completed.
- DP05 `calibration-dp05-e3-A-r1-314e96`: h3-mail's final and repair carried nine actual current
  quotes but paraphrased their statements in the body. The nine literal coupling failures caused
  the terminal failure. Earlier deck/mail COMPLETE proposals also entered read recovery for this
  coupling, despite source-backed actual supplied capsules.

## Separate semantic concern

The initial rejected/unshown mail draft preferred longer-maintenance Aster despite different
prices. The actual adopted rule is a price tie-break; it does not grant that unconditional
preference. Final repair body omitted the preference, while `constraints_respected` metadata
still asserted Aster selection. Preserve body and metadata separately. Removing the coupling
can allow such a recommendation to be displayed; it must be assessed by the separately frozen
final-content rubric, not treated as fixed merely because host provenance passes.

## Parent-approved general correction

Across all ABC, keep bounded/nonempty/raw-echo rejection, exact actual quote-in-cited-line,
current revision/access and target evidence checks. Remove only statement-in-display literal
duplication. DP01 additionally rejects a response that merely repeats the referent expression.
The actual final text is independently assessed by pre-authored content conditions and manual
conditional-scope review. No expected price/product/answer is injected into candidate code.

The previous common-recent path also treated `facts.statement` as delivered text. After this
schema simplification, retain only actual displayed answer text plus separate original source
provenance, typed `ACTUALLY_DELIVERED_ANSWER`. Quotes from that derived evidence must occur
in the actual displayed text; unshown statements/original quotes are removed from recent input.
This is distinct from exact raw-source-line evidence and is documented as such. All ABC share it.

Receiver instructions also state the general rule to preserve conditional antecedents and not
turn a tie-break into an unconditional recommendation. This does not decide a product or add
an oracle/model-judge call. Actual output is still evaluated independently.

## Preflight evidence and limits

- `proposed.patch`: exact candidate diff against calibration-v1, five owned files.
- `dp01-tests.txt` / `dp05-tests.txt`: nine guards each, including accepted paraphrase, denied
  metadata-only/stale/revoked/unseen/out-of-range/echo evidence and no-unshown-recent regression.
- `runs.jsonl`: 18 deterministic OFFLINE E3 ABC trials covering normal, repair and exhaustion.
  No EPISODE_ERROR; deliberate GOAL_FAILED is retained. These do not test model semantics.
- `../contract-v2-proposed-replay/record.json`: the original DP01 A E3 responses replayed through
  the proposed scratch. Four saved responses produced two actual completion callbacks, zero
  new provider calls. This is guard/renderer evidence, not new inference or latency improvement.
- `../../analysis/dp05-semantic-review-rubric.md`: evaluator-only rubric and actual final text
  review requirements, separate from structural N-H and host completion coverage.

Active source remains untouched until the runtime finishes calibration-v1 and authorizes the
new snapshot transition. A separate calibration-v2 must verify the revised contract with real
model calls before formal BASIC. Existing calibration failures never become successful runs.

The authorized transition was subsequently applied only after the runtime reported all 12
calibration-v1 rows finished. See `applied.json` for before/after hashes. The parent additionally
approved replacing the arbitrary three-character quote minimum with nonempty exact quotes;
`quote-length-correction.json` records that change before calibration-v2 began. Valid short
quotes such as `8W` and empty-quote rejection have separate tests. Current owned guard count
is 11 DP01 plus 11 DP05. Quote matching still does not establish semantic entailment.
