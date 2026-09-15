# DP-07 — Voice/Text Semantic Response Authority

**Status:** ACTIVE — NO ALTERNATIVE SELECTED

**Question:** Who owns the semantic meaning of user-facing Voice summary and Text detail?

- **A — Single semantic Response Author:** versioned `ResponsePlan` followed by Voice/Text renderers.
- **B — Channel-owned semantic authors:** independent Voice and Text authors plus fact/version reconciliation.

Both use the same validated `TaskResult`, facts, and version.

**Structural discriminator:** semantic publication authority, canonical response state ownership, reconciliation direction, and version lifecycle.

**Primary QAs:** QA-01, QA-02, QA-03, QA-05. **Dependencies:** DP-00, DP-01, TaskResult contract. All twelve QA-v1 attributes are evaluated.

**Excluded mechanisms:** sentence count, atomic versus incremental UI rendering, TTS provider, and voice style.
