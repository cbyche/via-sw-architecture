# DP-02 — VIA Semantic Decision Responsibility Topology

**Status:** ACTIVE — NO ALTERNATIVE SELECTED; primarily applies within R1.

**Question:** Should grounded user-goal construction and initial Agent routing be separate authoritative semantic responsibilities or one unified VIA decision aggregate?

- **A — Separate authorities:** `GroundedRequest` authority plus `AgentDecision` authority.
- **B — Unified authority:** `RequestDecision` produces the grounded goal and routing proposal.

Both are agent-neutral. Neither may perform general-purpose domain planning, tools, or workflow.

**Structural discriminator:** one versus two authoritative semantic artifacts, their ownership, dependency direction, and independent version/lifecycle boundaries. It is not one LLM call versus two; batching and prompt fusion are tactics.

**Primary QAs:** QA-01, QA-02, QA-04, QA-05. **Dependency:** DP-00/R1. All twelve QA-v1 attributes are evaluated.

**Excluded mechanisms:** call count, prompt topology, model size/provider, classifier versus LLM, and confidence threshold.
