# Variant-specific changed-module attribution

Four distinct scratch copies now have actual patches and execution evidence: C05/A, C05/B, C06/A and C06/B. They start from the same calibration-v1 frozen source. The earlier common patch evidence remains as development provenance, not the final attribution proof.

## C05: 2 modules for each

- `dp04.rs`: actual displayed-adoption commit and external query operation dispatch; required by A and B.
- `dp04/explanation.rs`: query/store implementation over actual delivered source extents; required by both.
- A's separate module contains no B speech-state/read-set logic. B's separate module reads its actual speech producer and candidate read-set, with no A fallback. Both compile and execute successful queries, failed-proposal exclusion and unknown-adoption NOT_FOUND.

## C06: 5 modules for each

- `dp04.rs`: actual external authorization event, current allowed binding state, policy update and subsequent permitted source re-read. Active in both.
- `dp04/gate.rs`: rejects a held earlier-policy proposal. Actual rejection is asserted in both.
- `dp04/geometry.rs`: final target extent admission rejects overlap with withdrawn source region. The negative probe makes an otherwise shaped new-policy proposal inside the denied region; both reject it through actual owner recovery and GOAL_FAILED. Thus this module is independently needed by A, beyond B's candidate checks.
- `dp04/observations.rs`: builds actual permitted original/overlay PNG paths before any rejudgment input. Both produce actual redacted images and subsequent model messages with those paths.
- `dp04/region_policy.rs`: actual region state, partial-binding invalidation and Sharp pixel-redaction adapter; all are needed by both. A's variant-specific patch removes the B candidate eligibility function and facade call entirely. B keeps this additional operation inside the same already-needed cohesive policy module, so it does not add a sixth Rust module.

Actual new raw pixel tests verify inside is black and disjoint pixel equals original. Disjoint C2 survives and is re-grounded; old policy rejects held publication; duplicate withdrawal leaves epoch unchanged. These are offline functional checks with a mock semantic provider, not actual model correctness/performance measurements. Node/Sharp is an operational dependency invoked by Rust policy code and is disclosed; no extra JS source module is omitted from a Rust count.

Ties follow actual separate patches (C05=2, C06=5), not duplicating one common count. These are concrete patches, not a mathematical proof that no alternate implementation could change fewer modules. Formal baseline evolution may require replay/rebase; the anchor is explicitly calibration-v1.
