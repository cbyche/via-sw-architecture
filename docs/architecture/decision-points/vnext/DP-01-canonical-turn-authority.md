# DP-01 — Canonical User-Turn Authority

**Status:** ACTIVE — NO ALTERNATIVE SELECTED

**Question:** When Voice/S2S, streaming ASR, Text, and interaction events coexist, which boundary owns the committed logical `UserTurn`?

- **A — Native/S2S turn authority with VIA identity/state mapping.**
- **B — VIA canonical-turn authority using Voice/ASR/Text/UI evidence.**

Both may use S2S and ASR. The discriminator is commit/revision authority when streams disagree or revise, not S2S versus ASR technology.

**Structural discriminator:** authoritative UserTurn identity, commit state, revision/reconciliation state, and the interface direction between native media turns and VIA task state.

**Primary QAs:** QA-01, QA-02, QA-03, QA-05. **Dependency:** DP-00. All twelve QA-v1 attributes are evaluated.

**Excluded mechanisms:** ASR/S2S provider, transcript confidence threshold, buffer size, prompt layout, and speculation on/off.
