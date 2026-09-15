# DP-08 — On-device Inference Resource Topology

**Status:** ACTIVE — NO ALTERNATIVE SELECTED

**Question:** Do Voice-critical and background semantic workloads share one inference execution domain, or is Voice-critical capacity independently protected?

- **A — Work-conserving shared inference pool.**
- **B — Realtime-isolated inference domain with enforceable worker, context, capacity, and admission boundaries.**

Process names alone are not isolation. Shared weights may be reused where physically supported.

**Structural discriminator:** enforceable resource admission, capacity, failure, scheduling, and deployment boundaries for realtime workloads.

**Primary QAs:** QA-01, QA-07, QA-11, QA-12. **Dependencies:** DP-00, DP-06. All twelve QA-v1 attributes are evaluated.

**Excluded mechanisms:** queue priority number, worker count, model size, provider, cache size, and process naming.
