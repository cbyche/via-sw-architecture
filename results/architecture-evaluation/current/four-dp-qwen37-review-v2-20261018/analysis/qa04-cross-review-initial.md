# Independent QA04 scratch review

Read-only review of calibration-v1-based `qa04/dp01-C05`, `dp01-C06`, `dp05-C04`, and `dp05-C06`. No new provider calls or candidate edits. This is a bounded code/patch review, not an independent replay of every acceptance run. Guard-v2/formal rebase remains required.

The independent A/B/C patches' changed Rust paths agree with reported counts: DP01 C05 1/1/1; DP01 C06 3/3/4; DP05 C04 1/1/1; DP05 C06 2/3/2. In particular, DP01 producer changes appear only in C's C06 patch and DP05 work-record producer changes only in B's C06 patch. No absent producer module was charged to another alternative in these reviewed patches. This does not establish architecture-wide modifiability beyond these finite tasks and existing module layout.

C05 reads actual committed outputs rather than future or failed proposals; C04 maintains artifact/section/task identity, preserves unrelated sections and immutable delivery history, rejects stale/wrong-task updates, and registers the new section as a future readable source. No oracle-ID control flow was observed in the reviewed feature additions.

## Required C06 correction before final acceptance

The reviewed initial C06 patches filtered span-backed blocks/records but retained ungrounded ancillary text: DP05 B's conditions/unknowns/coverage and DP01 C's coverage.gaps. The full work record/publication is later sent to semantic calls. Text derived from the withdrawn range could therefore survive after its supporting block was removed. This is a concrete data path, not a model-quality claim.

Reported to owner and root. Owner independently found the same issue and is conservatively clearing those strings/replacing current source metadata, retaining only independently valid span-backed records, and extending acceptance doubles to place the withdrawal marker in ancillary fields. Required check: marker absent from **all actual future semantic payloads**, not merely filtered-state logs. Owner completed the correction and expanded acceptance. Independent read-only recheck of the latest A/B/C records for both DP01/DP05 C06 found the withdrawal marker absent from every later recorded actual semantic payload after denied-range metadata first appears (6/6 variant records, zero provider calls). `qa04-cross-review-payload-checks.json` records run IDs and payload counts. Revised patch paths/counts remain the same. This closes the provisional ancillary-text path finding; formal baseline rebase/replay remains pending.

DP01's separate planned rebase removes host reconstruction of answer text from fact.statement: affected actually delivered answers must be discarded as a whole, retaining independently unaffected answers and precisely segmented original passages. Its old provisional implementation must not be described as the formal guard-v2 result.
