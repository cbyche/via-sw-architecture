# QA04 actual Rust change exercises

C01 changes one operating Rust module, C03 two. Mean **1.5 modules**, tied across all ten alternatives. Independent scratch copies preserve the live candidate. Actual unified diffs, baseline snapshot, compile/test logs and twenty end-to-end contract traces are retained.

C01 changes the provider request/response and usage envelope. A negative baseline probe fails, the migrated adapter passes JSON/usage/error checks. C03 changes source version metadata and frame coordinate/time envelopes. Shared source adapter plus main input admission restore exact canonical fixture objects; all ten alternatives execute against the migrated contract with a deterministic contract responder. No semantic accuracy or model timing is inferred from these responders. Frame coordinates/time apply to DP04; source revision compatibility applies to all four DPs. Test harness and probe modules are excluded from operational module counts.

This is a finite PoC compatibility exercise, not the full original product change-task population. No live baseline edits, model calls, or additional spending occur.

Supplement: six DP05 E2 delegation probes add actual wire, authorized source reads and receiver traversal for A/B/C under both migrations. Original twenty traces remain intact. All 26 contract checks pass.
