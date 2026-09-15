//! The session mode — which layers a session mounts.
//!
//! **New to VIA.** Neither upstream has this concept: qwen-audio-agent always
//! mounts the whole stack and ARGO deleted its own "direct mode" in August
//! 2026. The design is `docs/architecture.md` §2.
//!
//! A session runs in exactly one mode. The mode is fixed for the session's
//! lifetime, chosen by the client on the
//! [`connect`](crate::GatewayClientEvent::Connect) frame, and it decides which
//! layers are even mounted — which is why it lives here as a protocol type
//! rather than in the Gateway as a runtime flag.
//!
//! The wire name is the lowercase variant. It appears in the `connect` client
//! event, in `/api/health`, and in every Work record, so a stored Work can be
//! read back without ambiguity.

use crate::macros::wire_enum;

wire_enum! {
    /// Which of VIA's three layers a session mounts.
    ///
    /// | Mode | Layer 1 | Layer 2 | Layer 3 | What it is |
    /// | --- | --- | --- | --- | --- |
    /// | `dictation` | capture + ASR | — | — | Transcribe and nothing else. |
    /// | `direct` | full duplex | control tools only | — | The realtime model answers itself. |
    /// | `agent` | full duplex | full Work queue | pluggable harness | The whole stack. |
    /// | `interface` | full duplex | Context Engine | — | Voice drives the host's UI. |
    ///
    /// From `docs/architecture.md` §2.
    pub enum SessionMode {
        /// `dictation` — transcribe and nothing else. No model turn, no tools,
        /// no speech out.
        ///
        /// The one mode that mounts no model at all: the realtime provider may
        /// be a plain streaming ASR rather than a speech-to-speech model, which
        /// is why the provider trait must not assume a model turn exists. On
        /// the local pipeline this is the cheapest path by a wide margin — VAD
        /// plus ASR, no LLM, no TTS.
        Dictation = "dictation",
        /// `direct` — full duplex Layer 1, control tools only from Layer 2, no
        /// harness.
        ///
        /// The realtime model converses and answers **itself**. No delegation.
        /// This is also what [`Agent`](Self::Agent) degrades to when no harness
        /// is configured; see [`SessionMode::degraded_without_harness`].
        Direct = "direct",
        /// `agent` — the whole stack: full duplex, the full Work queue, and a
        /// pluggable harness behind `trait DownstreamAgent`.
        ///
        /// Fast-path answers plus delegated work. The default.
        Agent = "agent",
        /// `interface` — full duplex Layer 1 plus the Context Engine, no
        /// harness.
        ///
        /// Voice drives the **host's UI** through registered affordances rather
        /// than conversing. This is the Context Engine's reason to exist: in
        /// the other three modes it assembles context, and in `interface` it
        /// also resolves referents and ordered deixis into concrete on-screen
        /// targets (`docs/architecture.md` §5).
        Interface = "interface",
    }
}

impl Default for SessionMode {
    /// [`Agent`](SessionMode::Agent) — the whole stack.
    ///
    /// A client that says nothing gets the mode the product is named for. A
    /// fresh install with no harness configured still works, because `agent`
    /// degrades rather than failing; see
    /// [`degraded_without_harness`](SessionMode::degraded_without_harness).
    fn default() -> Self {
        Self::Agent
    }
}

impl SessionMode {
    /// Whether this mode mounts Layer 2 — middleware and coordination.
    ///
    /// True for every mode except [`Dictation`](Self::Dictation), which mounts
    /// no model and therefore has nothing to coordinate. Note that "mounts
    /// Layer 2" is not "mounts the Work queue": [`Direct`](Self::Direct) gets
    /// the control tools only and [`Interface`](Self::Interface) gets the
    /// Context Engine, while only [`Agent`](Self::Agent) gets the full queue
    /// (see [`mounts_work_queue`](Self::mounts_work_queue)).
    pub const fn mounts_layer2(&self) -> bool {
        !matches!(self, Self::Dictation)
    }

    /// Whether this mode mounts Layer 3 — a backend agent harness.
    ///
    /// True only for [`Agent`](Self::Agent). A session in any other mode never
    /// opens a `HarnessSession`, so a missing or misconfigured harness cannot
    /// affect it.
    pub const fn mounts_harness(&self) -> bool {
        matches!(self, Self::Agent)
    }

    /// Whether this mode may emit synthesized speech to the client.
    ///
    /// False only for [`Dictation`](Self::Dictation), the one mode
    /// `docs/architecture.md` §2 marks "no speech out". The other three run
    /// Layer 1 full duplex.
    pub const fn speaks(&self) -> bool {
        !matches!(self, Self::Dictation)
    }

    /// Whether this mode mounts the full Work queue — delegation, the
    /// per-owner FIFO and the coordinator lane.
    ///
    /// True only for [`Agent`](Self::Agent). [`Direct`](Self::Direct) gets the
    /// Layer 2 *control* tools (status, cancel, time, memory, notes, reminder,
    /// permission-reply) but creates no delegated Work.
    pub const fn mounts_work_queue(&self) -> bool {
        matches!(self, Self::Agent)
    }

    /// The mode this session actually runs in when no harness is configured.
    ///
    /// [`Agent`](Self::Agent) degrades to [`Direct`](Self::Direct) rather than
    /// failing — this mirrors upstream's `AGENT_PROTOCOL=none` frontend-only
    /// mode and is what makes a fresh install useful before any backend is
    /// installed. Every other mode is unaffected, because none of them mounts a
    /// harness in the first place.
    ///
    /// The degradation is **reported on `/api/health`, never silent**
    /// (`docs/architecture.md` §2), so a caller that applies this must also
    /// surface it.
    ///
    /// ```
    /// use via_protocol::SessionMode;
    ///
    /// assert_eq!(SessionMode::Agent.degraded_without_harness(), SessionMode::Direct);
    /// assert_eq!(SessionMode::Direct.degraded_without_harness(), SessionMode::Direct);
    /// ```
    pub const fn degraded_without_harness(self) -> Self {
        match self {
            Self::Agent => Self::Direct,
            other => other,
        }
    }
}
