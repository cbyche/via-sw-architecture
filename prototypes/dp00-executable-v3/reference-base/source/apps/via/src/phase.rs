//! The one way this binary says "not yet".
//!
//! `docs/architecture.md` §15 is a nine-phase delivery plan, and `apps/via`
//! ships in phase 1 with four of its six commands still bodyless. That is a
//! design constraint rather than an accident: the *argument surface* carries
//! external contracts and is worth having early, while the body it dispatches
//! to belongs to the crate that implements it.
//!
//! A stub therefore has exactly one correct behaviour, and this module is it:
//!
//! * **exit non-zero.** A stub that exits `0` is a lie a shell script cannot
//!   detect. `set -e` would sail straight past it and the failure would
//!   resurface somewhere with no connection to the cause.
//! * **name the phase.** "not implemented" tells the reader to file a bug;
//!   "lands in phase 5 (docs/architecture.md §15)" tells them to read the plan.
//! * **localize.** VIA ships three locales; a refusal a third of the users
//!   cannot read is a refusal that gets misreported.

use core::fmt;

use via_i18n::{Locale, format, keys};

/// The document every refusal points the reader at.
pub const PLAN_REFERENCE: &str = "docs/architecture.md §15";

/// A phase of `docs/architecture.md` §15's delivery plan.
///
/// Only the phases that own an unwritten `apps/via` command are represented.
/// Adding a variant for a phase nothing refuses against would be adding a
/// value with no meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    /// `via-acp` + `via-backends` + `via-process` + `via-downstream` —
    /// backend installation, readiness and authentication.
    BackendIntegration,
    /// `via-work` + `via-coordinator` + `via-mcp-tools` — the five
    /// coordination tools `via mcp-serve` exposes over stdio.
    WorkQueue,
    /// `via-realtime` + `via-voice` + `via-app` — the Gateway server itself,
    /// and the text client that talks to it.
    GatewayRuntime,
}

impl Phase {
    /// The phase's number in `docs/architecture.md` §15.
    #[must_use]
    pub const fn number(self) -> u8 {
        match self {
            Self::BackendIntegration => 2,
            Self::WorkQueue => 3,
            Self::GatewayRuntime => 5,
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.number())
    }
}

/// A command that parsed, validated, and has no body in this build.
///
/// Constructing one is not enough to refuse with it — it has to be returned as
/// a [`CliError::Unimplemented`](crate::CliError::Unimplemented), which is the
/// only route to a non-zero exit. There is deliberately no way to turn one
/// into a success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unimplemented {
    /// The command as the user spelled it, e.g. `gateway` or `backend auth`.
    command: String,
    /// The phase that implements it.
    phase: Phase,
}

impl Unimplemented {
    /// Refuse `command` until `phase`.
    #[must_use]
    pub fn new(command: impl Into<String>, phase: Phase) -> Self {
        Self {
            command: command.into(),
            phase,
        }
    }

    /// The command this refusal is about.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The phase that implements it.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// The refusal, rendered in `locale`.
    ///
    /// The catalog key is `cli.not_implemented_until_phase`; its three locales
    /// share the `{command}`, `{phase}` and `{reference}` placeholder set, so
    /// none of them can drop the phase number silently.
    #[must_use]
    pub fn message(&self, locale: Locale) -> String {
        format(
            locale,
            keys::CLI_NOT_IMPLEMENTED_UNTIL_PHASE,
            &[
                ("command", self.command.as_str()),
                ("phase", &self.phase.number().to_string()),
                ("reference", PLAN_REFERENCE),
            ],
        )
    }
}

impl fmt::Display for Unimplemented {
    /// The `en` rendering. Every caller that has a locale uses
    /// [`Self::message`]; this exists so the type is a well-behaved error
    /// source.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message(Locale::En))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_phase_numbers_are_the_delivery_plans() {
        assert_eq!(Phase::BackendIntegration.number(), 2);
        assert_eq!(Phase::WorkQueue.number(), 3);
        assert_eq!(Phase::GatewayRuntime.number(), 5);
    }

    #[test]
    fn every_locale_names_the_command_the_phase_and_the_plan() {
        let refusal = Unimplemented::new("backend auth", Phase::BackendIntegration);
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let message = refusal.message(locale);
            assert!(message.contains("backend auth"), "{locale}: {message}");
            assert!(message.contains('2'), "{locale}: {message}");
            assert!(message.contains(PLAN_REFERENCE), "{locale}: {message}");
            assert!(!message.contains('{'), "{locale} left a hole: {message}");
            assert!(
                !message.contains("<via-i18n:"),
                "{locale} rendered a diagnostic: {message}"
            );
        }
    }

    #[test]
    fn the_three_locales_differ_from_each_other() {
        let refusal = Unimplemented::new("chat", Phase::GatewayRuntime);
        let en = refusal.message(Locale::En);
        let zh = refusal.message(Locale::Zh);
        let ko = refusal.message(Locale::Ko);
        assert_ne!(en, zh);
        assert_ne!(zh, ko);
        assert_ne!(en, ko);
    }

    #[test]
    fn display_is_the_english_rendering() {
        let refusal = Unimplemented::new("mcp-serve", Phase::WorkQueue);
        assert_eq!(refusal.to_string(), refusal.message(Locale::En));
    }
}
