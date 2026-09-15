# DP-09 — Device Capability Abstraction Boundary

**Status:** ACTIVE — ADMITTED — NO ALTERNATIVE SELECTED

## Admission evidence

The approved baseline records Samsung-device expansion as a strategic stakeholder concern (`requirements-v1.1.md`, STK-01). QA Evaluation Contract v1 then freezes a causal QA-06 adaptation population spanning Mobile, TV, and Robot families. This makes the PC-first product scope non-exclusive.

The alternatives move rich device-semantic authority across the Core/runtime interface. They have meaningful opposing Core-churn versus duplication/divergence risks and cannot be combined without naming another authority boundary. The choice changes where QA-06 evolution cases require semantic code changes, so all four admission conditions are met.

## Decision question and alternatives

Where should device-specific interaction, context, and capability semantics be normalized?

- **A — Device-neutral VIA Core capability/context model plus thin device adapters/providers.**
- **B — Device-family interaction runtime owns rich device semantics and projects a smaller canonical interaction/task contract into shared VIA Core.**

**Structural discriminator:** authoritative ownership of rich device semantics, the direction and expressiveness of the Core/device contract, and which side absorbs device-family state and evolution.

**Primary QAs:** QA-06, QA-05, QA-02, QA-03. **Dependencies:** DP-00, DP-01, DP-03, DP-04. All twelve QA-v1 attributes are evaluated.

**Excluded mechanisms:** adapter language, device allow-list, payload format, feature flags, and per-device timeout values.
