# DP-05 — UserTask ↔ Native Agent Session Binding

**Status:** ACTIVE — NO ALTERNATIVE SELECTED

**Question:** Do independent `UserTask`s share a long-lived native Agent conversational/execution session, or receive isolated native execution state?

- **A — Shared long-lived Agent session.**
- **B — Task-scoped native Agent session.** Follow-ups to the same Task reuse it.

Transport, model weights, and caches may be shared in both.

**Structural discriminator:** native conversational/execution state ownership, isolation, correlation, persistence, and cleanup boundary across UserTasks.

**Primary QAs:** QA-01, QA-02, QA-03, QA-04. **Dependencies:** DP-00, DP-04. All twelve QA-v1 attributes are evaluated.

**Excluded mechanisms:** transport pooling, cache size, session timeout, and model instance count.
