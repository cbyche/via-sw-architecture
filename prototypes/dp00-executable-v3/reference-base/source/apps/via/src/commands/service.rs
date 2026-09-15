//! `via service <install|start|stop>` — the launchd / systemd unit.
//!
//! Ported in surface only from `cli/src/gateway-service.mjs` and its call site
//! (`cli/src/launcher.mjs:300-379`). Installing a unit that starts a Gateway
//! is meaningless until there *is* a Gateway, so the body is phase 5.
//!
//! # Why an unknown action is a variant rather than a `clap` error
//!
//! *cli-commands/GATEWAY_ACTIONS* lists seven verbs; §10 keeps three. The four
//! that are gone — `run` (now `via gateway`), `restart`, `status`,
//! `uninstall` — are exactly what a user carrying muscle memory from upstream
//! will type. `clap`'s "unrecognized subcommand" would tell them the verb does
//! not exist; the catalogued *"unsupported Gateway service action: `<x>`"*
//! tells them the same thing in the words the rest of the product uses, and in
//! their own language.

use via_i18n::keys;

use crate::cli::{ServiceAction, ServiceArgs};
use crate::error::{CODE_INVALID_ARGUMENT, CliError};
use crate::host::Host;
use crate::phase::{Phase, Unimplemented};

/// The service actions `docs/architecture.md` §10 defines.
///
/// **Partial contract** — the other four of upstream's seven are recorded as
/// owed in `docs/deviations/phase-1.md`.
pub const SERVICE_ACTIONS: [&str; 3] = ["install", "start", "stop"];

/// Run `via service`.
///
/// # Errors
///
/// [`CliError::Refused`] for an action outside [`SERVICE_ACTIONS`], otherwise
/// [`CliError::Unimplemented`] naming phase 5.
pub fn run(args: &ServiceArgs, host: &Host) -> Result<(), CliError> {
    match &args.action {
        ServiceAction::Install | ServiceAction::Start | ServiceAction::Stop => {
            Err(CliError::Unimplemented(Unimplemented::new(
                std::format!("service {}", args.action.name()),
                Phase::GatewayRuntime,
            )))
        }
        ServiceAction::Unknown(words) => Err(CliError::refused_with(
            CODE_INVALID_ARGUMENT,
            host.locale(),
            keys::CLI_SERVICE_UNSUPPORTED_ACTION,
            &[("action", words.first().map_or("", String::as_str))],
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command, parse_from};
    use crate::error::CODE_NOT_IMPLEMENTED;
    use rstest::rstest;
    use via_i18n::Locale;

    fn host(locale: Locale) -> Host {
        Host::new(
            [("VIA_LOCALE", locale.as_str())].into_iter().collect(),
            "/home/via".into(),
            "/srv/via".into(),
        )
    }

    fn run_action(action: &str) -> Result<(), CliError> {
        run_action_in(Locale::En, action)
    }

    fn run_action_in(locale: Locale, action: &str) -> Result<(), CliError> {
        let cli: Cli = parse_from(["service", action]).expect("parses");
        let Command::Service(args) = cli.command else {
            panic!("not a service invocation");
        };
        run(&args, &host(locale))
    }

    #[rstest]
    #[case("install")]
    #[case("start")]
    #[case("stop")]
    fn a_section_ten_action_refuses_naming_phase_five(#[case] action: &str) {
        let error = run_action(action).expect_err("the body lands in phase 5");
        assert_eq!(error.code(), CODE_NOT_IMPLEMENTED);
        let message = error.message(Locale::En);
        assert!(
            message.contains(&std::format!("service {action}")),
            "{message}"
        );
        assert!(message.contains('5'), "{message}");
    }

    #[rstest]
    // The four upstream verbs §10 does not keep.
    #[case("restart")]
    #[case("status")]
    #[case("uninstall")]
    #[case("run")]
    #[case("bogus")]
    fn an_action_outside_section_ten_is_named_in_the_catalogued_refusal(#[case] action: &str) {
        let chinese = run_action_in(Locale::Zh, action).expect_err("not a VIA service action");
        assert_eq!(chinese.code(), CODE_INVALID_ARGUMENT);
        assert_eq!(
            chinese.message(Locale::Zh),
            std::format!("不支持的 Gateway 服务操作：{action}")
        );
        let english = run_action_in(Locale::En, action).expect_err("not a VIA service action");
        assert_eq!(
            english.message(Locale::En),
            std::format!("unsupported Gateway service action: {action}")
        );
    }

    #[test]
    fn the_declared_actions_match_what_the_parser_accepts() {
        for action in SERVICE_ACTIONS {
            let cli: Cli = parse_from(["service", action]).expect("parses");
            let Command::Service(args) = cli.command else {
                panic!("not a service invocation");
            };
            assert!(
                !matches!(args.action, ServiceAction::Unknown(_)),
                "{action} must be a declared action, not an external one"
            );
            assert_eq!(args.action.name(), action);
        }
    }
}
