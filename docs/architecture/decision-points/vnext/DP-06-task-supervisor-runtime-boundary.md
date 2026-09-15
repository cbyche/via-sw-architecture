# DP-06 — Interaction Frontend ↔ Task Supervisor Runtime Boundary

**Status:** ACTIVE — NO ALTERNATIVE SELECTED

**Question:** Does authoritative `UserTask` supervision share the frontend runtime lifecycle, or live in an independently running durable Task Service?

- **A — Co-hosted interaction and task-supervision runtime.**
- **B — Detached durable Task Service with typed command/event boundary.**

Both must support durable logical Task state. Frontend disconnect must not be used to cripple A or destroy its tasks.

**Structural discriminator:** runtime, deployment, supervision, failure, IPC, and recovery boundaries around authoritative Task state.

**Primary QAs:** QA-03, QA-05, QA-08, QA-11. QA-01 and QA-07 are important regression measurements. **Dependencies:** DP-00, DP-05. All twelve QA-v1 attributes are evaluated.

**Excluded mechanisms:** process names, retry count, heartbeat interval, timeout, and queue priority.
