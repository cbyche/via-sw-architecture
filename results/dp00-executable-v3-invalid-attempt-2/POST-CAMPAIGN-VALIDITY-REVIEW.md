# DP-00 Executable v3 Attempt 2 — Invalidation Review

Frozen source: `8ce72bf0e18430bfea7d34b16c3ae7b8eccacc25`

## Disposition

**INVALIDATED — QA-01 CORRECTNESS EVIDENCE WAS INCOMPLETE.**

The corrected evolution instrument completed its full 540-worktree campaign, with real request-specific Rust handlers, exactly one behavioral acceptance test per case, common regressions, and source-derived seams/dependency evidence. However, critical scalar inspection found that the shared goal correctness function required a derived Voice/Text consistency observation while the QA-01 timed path had not run its paired modality probe. Every QA-01 case therefore received the contract's 30-second incorrect-outcome penalty. The aggregate campaign-validity gate checked structural coverage but did not reject the failed QA qualification.

This defect invalidates the whole campaign under the frozen protocol. The output is retained only for audit. The fix must add an untimed paired-modality execution for every QA-01 case/realization, propagate the derived result into each timed record, and require every evaluated QA except officially unevaluable QA-07 to meet its target and qualification gates before the campaign can be valid.

Decision: **EVALUATION STILL INVALID**.
