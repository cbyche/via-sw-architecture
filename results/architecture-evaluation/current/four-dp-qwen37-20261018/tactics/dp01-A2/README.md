# DP01 A2 correction follow-up

The user-selected BASIC design is A. This separate tactic preserves unaffected valid working-view fragments when a source changes. BASIC A invalidates the whole view. The source revision, current access and actual-delivery guards are identical. E3 runs three times, after all 120 BASIC episodes. The same E3 baseline repetitions are the before comparison; this is not a full four-episode tactic result.

No baseline source, binary, fixture, oracle or result was modified. `candidate.diff` records the only production change. The persisted variant is `prototypes/candidates/four_dp_poc/variants/dp01-A2-fragment-invalidation.rs`. Compile it as `src/dp01.rs` in a separate copy of the candidate crate; the prepared binary path/hash and original source hashes are in `freeze.json`. Pass `variant=A` and `tactic=A2`; all other variants/default inputs retain the original invalidation behavior.

`review/` retains three deterministic source-dependency/access/delivery guard checks and actual output. They are not model measurements. The runtime agent executes the real-model repetitions under the existing single $5 ledger after BASIC completion.

In this small raw-passage implementation, the expected change is avoiding unaffected disk rereads. A2 may add semantic input tokens or increase cost/RAM. Ties and regressions remain in the final comparison without additional tuning.
