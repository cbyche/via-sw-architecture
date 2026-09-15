# DP-04 — Agent Request Context Materialization Boundary

**Status:** ACTIVE — NO ALTERNATIVE SELECTED

**Question:** Must task context be fully materialized before Agent admission, or may execution retain scoped context-resolution capability?

- **A — Pre-admission self-contained request package.**
- **B — Grounded request plus versioned/scoped context handles with lifecycle Context Broker resolution.**

Small mandatory identity and goal facts may be inline in both alternatives.

**Structural discriminator:** whether context resolution ends at admission or remains a lifecycle dependency with scoped/versioned authorization and failure semantics.

**Primary QAs:** QA-01, QA-02, QA-04, QA-05. **Dependencies:** DP-00, DP-03. All twelve QA-v1 attributes are evaluated.

**Excluded mechanisms:** package size, cache policy, serialization format, and prefetch threshold.
