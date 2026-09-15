//! Session-mode dispatch — which layers a session actually mounts.
//!
//! `docs/architecture.md` §2. The mode is fixed for the session's lifetime and
//! chosen by the client on the `connect` frame, which is why
//! [`via_protocol::SessionMode`] is a protocol type rather than a runtime flag.
//!
//! [`ModePlan`] turns the mode plus two facts — *is a harness configured?* and
//! *is a Context Engine mounted?* — into the four decisions the voice layer
//! makes on every turn:
//!
//! | | `dictation` | `direct` | `agent` | `interface` |
//! | --- | --- | --- | --- | --- |
//! | runs a model turn | no | yes | yes | yes |
//! | declares tools | none | control only | all | control only |
//! | delegates | no | no | yes | no |
//! | speaks | no | yes | yes | yes |
//!
//! # `interface` is real, and says so when it is not
//!
//! `via-context` landed in phase 7, so `interface` no longer degrades: a
//! session in that mode runs the model with the control tools and the Context
//! Engine resolving referents and ordered deixis. What can still be absent is
//! the *engine itself* — an embedder that builds `via-voice` without
//! `via-context` — and that caller says so through
//! [`ModePlan::with_context_engine`], which is the only way
//! [`Degradation::ContextEngineUnavailable`] is now produced.
//!
//! [`ModePlan::new`] answers as though the engine is mounted, because in a
//! Gateway built from this workspace it is. Whether that engine has a *screen*
//! to look at is a different question, and one the engine answers itself
//! (`via_context::SurfaceStatus`) rather than one the mode plan pretends to
//! know.

use via_protocol::SessionMode;

use crate::tools::catalog;

/// What a session mode resolves to, given the environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModePlan {
    requested: SessionMode,
    effective: SessionMode,
    harness_configured: bool,
    context_engine: bool,
}

/// Why the effective mode differs from the requested one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Degradation {
    /// `agent` with no harness configured runs as `direct`.
    ///
    /// This mirrors upstream's `AGENT_PROTOCOL=none` frontend-only mode and is
    /// what makes a fresh install useful before any backend is installed.
    NoHarness,
    /// `interface` with no Context Engine mounted runs as `direct`.
    ///
    /// Produced only by [`ModePlan::with_context_engine`] with
    /// `context_engine: false` — an embedder that builds this crate without
    /// `via-context`. A Gateway built from this workspace has the engine, so
    /// [`ModePlan::new`] never reports it.
    ContextEngineUnavailable,
}

impl Degradation {
    /// A stable code for `/api/health`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoHarness => "no_harness",
            Self::ContextEngineUnavailable => "context_engine_unavailable",
        }
    }
}

impl ModePlan {
    /// Resolve `requested` against whether a harness is configured.
    ///
    /// The Context Engine is taken to be mounted; see
    /// [`with_context_engine`](Self::with_context_engine) for the caller that
    /// has none.
    #[must_use]
    pub fn new(requested: SessionMode, harness_configured: bool) -> Self {
        Self::with_context_engine(requested, harness_configured, true)
    }

    /// Resolve `requested` against both facts.
    ///
    /// `context_engine` is *"is a Context Engine mounted for this session?"*.
    /// Only `interface` reads it, because it is the only mode that mounts one
    /// (`docs/architecture.md` §2). An `interface` session without an engine
    /// runs as `direct` and reports
    /// [`Degradation::ContextEngineUnavailable`] — the same shape as `agent`
    /// degrading without a harness, and reported the same way.
    #[must_use]
    pub fn with_context_engine(
        requested: SessionMode,
        harness_configured: bool,
        context_engine: bool,
    ) -> Self {
        let effective = match requested {
            SessionMode::Interface if !context_engine => SessionMode::Direct,
            other if harness_configured => other,
            other => other.degraded_without_harness(),
        };
        Self {
            requested,
            effective,
            harness_configured,
            context_engine,
        }
    }

    /// What the client asked for.
    #[must_use]
    pub const fn requested(&self) -> SessionMode {
        self.requested
    }

    /// What the session actually runs as.
    #[must_use]
    pub const fn effective(&self) -> SessionMode {
        self.effective
    }

    /// Whether a harness is configured.
    #[must_use]
    pub const fn harness_configured(&self) -> bool {
        self.harness_configured
    }

    /// Whether a Context Engine is mounted for this session.
    #[must_use]
    pub const fn context_engine(&self) -> bool {
        self.context_engine
    }

    /// Why the two differ, when they do.
    ///
    /// Reported on `/api/health`, never silent (`docs/architecture.md` §2).
    #[must_use]
    pub const fn degradation(&self) -> Option<Degradation> {
        match self.requested {
            SessionMode::Interface if !self.context_engine => {
                Some(Degradation::ContextEngineUnavailable)
            }
            SessionMode::Agent if !self.harness_configured => Some(Degradation::NoHarness),
            _ => None,
        }
    }

    /// Whether the realtime model takes a turn at all.
    ///
    /// False only for `dictation`, which mounts no model: the provider may be a
    /// plain streaming ASR, and asking it for a model turn is a protocol error
    /// rather than a slow answer.
    #[must_use]
    pub const fn runs_model_turn(&self) -> bool {
        self.effective.mounts_layer2()
    }

    /// Whether this session may emit synthesized speech.
    #[must_use]
    pub const fn speaks(&self) -> bool {
        self.effective.speaks()
    }

    /// Whether this session may create delegated Work.
    #[must_use]
    pub const fn delegates(&self) -> bool {
        self.effective.mounts_work_queue()
    }

    /// Whether the Injection Gate can ever have anything to deliver.
    ///
    /// A session that never delegates never produces a Work result, so the
    /// announcement path is inert rather than merely idle.
    #[must_use]
    pub const fn announces(&self) -> bool {
        self.delegates() && self.speaks()
    }

    /// Whether `tool_name` is declared to the model in this mode.
    ///
    /// `dictation` declares nothing. Every other mode declares the Layer-2
    /// **control** tools; only a mode that mounts the Work queue declares
    /// [`catalog::SPAWN_THINKING`], because a delegation tool that always
    /// answers "no backend" teaches the model to stop offering to do things.
    #[must_use]
    pub fn declares_tool(&self, tool_name: &str) -> bool {
        if !self.runs_model_turn() {
            return false;
        }
        if tool_name == catalog::SPAWN_THINKING {
            return self.delegates();
        }
        catalog::ALL_TOOL_NAMES.contains(&tool_name)
    }
}

impl Default for ModePlan {
    /// `agent` with a harness — the whole stack.
    fn default() -> Self {
        Self::new(SessionMode::default(), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn agent_degrades_to_direct_without_a_harness_and_says_so() {
        let plan = ModePlan::new(SessionMode::Agent, false);
        assert_eq!(plan.requested(), SessionMode::Agent);
        assert_eq!(plan.effective(), SessionMode::Direct);
        assert_eq!(plan.degradation(), Some(Degradation::NoHarness));
        assert!(!plan.delegates());
        assert!(!plan.declares_tool(catalog::SPAWN_THINKING));
        assert!(plan.declares_tool(catalog::MEMORY));
    }

    #[test]
    fn agent_with_a_harness_is_the_whole_stack() {
        let plan = ModePlan::new(SessionMode::Agent, true);
        assert_eq!(plan.effective(), SessionMode::Agent);
        assert_eq!(plan.degradation(), None);
        assert!(plan.delegates());
        assert!(plan.announces());
        assert!(plan.declares_tool(catalog::SPAWN_THINKING));
    }

    #[test]
    fn dictation_mounts_no_model_no_tools_and_no_speech() {
        for harness in [false, true] {
            let plan = ModePlan::new(SessionMode::Dictation, harness);
            assert_eq!(plan.effective(), SessionMode::Dictation);
            assert_eq!(plan.degradation(), None, "dictation never degrades");
            assert!(!plan.runs_model_turn());
            assert!(!plan.speaks());
            assert!(!plan.delegates());
            assert!(!plan.announces());
            for tool in catalog::ALL_TOOL_NAMES {
                assert!(!plan.declares_tool(tool), "{tool}");
            }
        }
    }

    #[test]
    fn direct_runs_the_model_with_control_tools_and_never_delegates() {
        for harness in [false, true] {
            let plan = ModePlan::new(SessionMode::Direct, harness);
            assert_eq!(plan.effective(), SessionMode::Direct);
            assert_eq!(plan.degradation(), None);
            assert!(plan.runs_model_turn());
            assert!(plan.speaks());
            assert!(!plan.delegates());
            assert!(!plan.declares_tool(catalog::SPAWN_THINKING));
            for tool in [
                catalog::SCHEDULE_REMINDER,
                catalog::CANCEL_AGENT_TASK,
                catalog::GET_AGENT_TASK_STATUS,
                catalog::GET_CURRENT_TIME,
                catalog::MEMORY,
                catalog::NOTES,
                catalog::RESPOND_AGENT_PERMISSION,
                catalog::ENTER_SLEEP,
            ] {
                assert!(plan.declares_tool(tool), "{tool}");
            }
        }
    }

    #[test]
    fn interface_runs_as_itself_now_that_the_context_engine_exists() {
        for harness in [false, true] {
            let plan = ModePlan::new(SessionMode::Interface, harness);
            assert_eq!(plan.requested(), SessionMode::Interface);
            assert_eq!(plan.effective(), SessionMode::Interface);
            assert_eq!(plan.degradation(), None, "harness_configured={harness}");
            assert!(plan.context_engine());
            assert!(plan.runs_model_turn());
            assert!(plan.speaks());
            // `interface` drives the host's UI; it never delegates, so a
            // missing harness is not a constraint on it.
            assert!(!plan.delegates());
            assert!(!plan.announces());
            assert!(!plan.declares_tool(catalog::SPAWN_THINKING));
            assert!(plan.declares_tool(catalog::MEMORY));
        }
    }

    #[test]
    fn interface_without_a_context_engine_degrades_to_direct_and_says_why() {
        // The caller that has no engine is an embedder building this crate
        // without `via-context`; it is the only way the degradation is now
        // reported.
        let plan = ModePlan::with_context_engine(SessionMode::Interface, true, false);
        assert_eq!(plan.requested(), SessionMode::Interface);
        assert_eq!(plan.effective(), SessionMode::Direct);
        assert!(!plan.context_engine());
        assert_eq!(
            plan.degradation(),
            Some(Degradation::ContextEngineUnavailable),
        );
        assert_eq!(
            plan.degradation().map(Degradation::as_str),
            Some("context_engine_unavailable"),
        );
        assert!(plan.runs_model_turn());
        assert!(!plan.delegates());
    }

    #[test]
    fn a_missing_context_engine_constrains_no_other_mode() {
        // Only `interface` mounts one, so the fact must not leak sideways.
        for mode in [
            SessionMode::Dictation,
            SessionMode::Direct,
            SessionMode::Agent,
        ] {
            let with = ModePlan::with_context_engine(mode, true, true);
            let without = ModePlan::with_context_engine(mode, true, false);
            assert_eq!(with.effective(), without.effective(), "{mode:?}");
            assert_eq!(with.degradation(), without.degradation(), "{mode:?}");
        }
    }

    #[test]
    fn new_is_with_context_engine_mounted() {
        for mode in SessionMode::ALL {
            assert_eq!(
                ModePlan::new(*mode, true),
                ModePlan::with_context_engine(*mode, true, true),
            );
            assert_eq!(
                ModePlan::new(*mode, false),
                ModePlan::with_context_engine(*mode, false, true),
            );
        }
    }

    #[test]
    fn the_default_plan_is_the_whole_stack() {
        assert_eq!(ModePlan::default().effective(), SessionMode::Agent);
        assert_eq!(ModePlan::default().degradation(), None);
    }

    #[test]
    fn an_unknown_tool_is_never_declared() {
        let plan = ModePlan::new(SessionMode::Agent, true);
        assert!(!plan.declares_tool("rm_rf"));
        assert!(!plan.declares_tool(""));
    }

    #[test]
    fn the_degradation_codes_are_stable() {
        assert_eq!(Degradation::NoHarness.as_str(), "no_harness");
        assert_eq!(
            Degradation::ContextEngineUnavailable.as_str(),
            "context_engine_unavailable",
        );
    }
}
