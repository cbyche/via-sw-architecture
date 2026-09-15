//! `via backend <install|status|auth>` — the backend agent lifecycle.
//!
//! `docs/architecture.md` §10's `backend` verb covers three of upstream's
//! surfaces: `install NAME` becomes `backend install NAME`, `setup` becomes
//! `backend status`, and `auth` is the onboarding command the catalog already
//! declares per backend (`onboarding.command`). All three bodies belong to
//! phase 2, with `via-acp`, `via-backends` and `via-process`.
//!
//! The **target validation is not deferred**, because it is catalogued
//! (*error-code/install target validation*) and because it is the half a user
//! meets first: `via backend install opencde` should say which ids exist, not
//! spend a minute discovering that npm has never heard of it.

use via_catalog::{backend_definition, normalize_backend_protocol};
use via_i18n::{Locale, keys};

use crate::cli::{BackendAction, BackendArgs};
use crate::error::{CODE_INVALID_ARGUMENT, CliError};
use crate::host::Host;
use crate::phase::{Phase, Unimplemented};

/// The one backend VIA never installs.
///
/// **External contract** — `cli/src/arguments.mjs:198-200`: generic ACP names
/// no product, so there is nothing to fetch; the user installs their own agent
/// and points `ACP_COMMAND` at it. `via-catalog` says the same thing
/// structurally — `acp` is the only definition whose `lifecycle.installation`
/// is `None` — and this constant exists so the two cannot disagree silently.
pub const GENERIC_ACP_ID: &str = "acp";

/// Run `via backend`.
///
/// # Errors
///
/// [`CliError::Refused`] for a missing, unknown or uninstallable target;
/// [`CliError::Core`] for a `--backend` id the catalog does not have;
/// otherwise [`CliError::Unimplemented`] naming phase 2.
pub fn run(args: &BackendArgs, host: &Host) -> Result<(), CliError> {
    let locale = host.locale();
    match &args.action {
        BackendAction::Install(install) => {
            let target = install_target(install.name.as_deref(), locale)?;
            refuse(&std::format!("backend install {target}"))
        }
        BackendAction::Status(status) => {
            if let Some(requested) = status.backend.as_deref() {
                selected_backend(requested)?;
            }
            refuse("backend status")
        }
        BackendAction::Auth(auth) => {
            let target = install_target(auth.name.as_deref(), locale)?;
            refuse(&std::format!("backend auth {target}"))
        }
    }
}

/// Resolve and validate an installation target.
///
/// **External contract** — `cli/src/arguments.mjs:181-200`, in upstream's
/// order:
///
/// 1. absent, or `none` (which normalises to absent) → *"install is missing a
///    backend name (available: …)"*, with the ids in catalog order;
/// 2. present but not in the catalog → *"unsupported backend: `<x>`"*;
/// 3. generic `acp` → *"install an Agent reached over generic ACP yourself…"*.
///
/// The order is what makes the messages useful: `via backend install none`
/// gets the list rather than "unsupported backend: none", because `none` is
/// upstream's spelling for *no backend at all*.
fn install_target(name: Option<&str>, locale: Locale) -> Result<String, CliError> {
    let normalized = normalize_backend_protocol(name.unwrap_or_default());
    if normalized.is_empty() {
        return Err(CliError::refused_with(
            CODE_INVALID_ARGUMENT,
            locale,
            keys::CLI_INSTALL_NEEDS_BACKEND,
            &[(
                "names",
                &crate::commands::gateway::backend_name_list(locale),
            )],
        ));
    }
    let definition = backend_definition(&normalized).ok_or_else(|| {
        CliError::from(via_catalog::CatalogError::UnsupportedBackend {
            protocol: normalized.clone(),
        })
    })?;
    if definition.id == GENERIC_ACP_ID {
        return Err(CliError::refused(
            CODE_INVALID_ARGUMENT,
            locale,
            keys::CLI_INSTALL_GENERIC_ACP,
        ));
    }
    Ok(normalized)
}

/// Validate a `--backend` selection for `via backend status`.
///
/// Upstream's `setup --backend NAME` narrows the report to one backend
/// (`cli/src/launcher.mjs:148-151`). Unlike an install target, `none` is not
/// meaningful here — there is nothing to report on — so it is refused with the
/// same message an unknown id gets.
fn selected_backend(requested: &str) -> Result<&'static str, CliError> {
    let normalized = normalize_backend_protocol(requested);
    backend_definition(&normalized)
        .map(|definition| definition.id)
        .ok_or_else(|| {
            CliError::from(via_catalog::CatalogError::UnsupportedBackend {
                protocol: if normalized.is_empty() {
                    requested.trim().to_lowercase()
                } else {
                    normalized
                },
            })
        })
}

/// The phase-2 refusal.
fn refuse(command: &str) -> Result<(), CliError> {
    Err(CliError::Unimplemented(Unimplemented::new(
        command,
        Phase::BackendIntegration,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command, parse_from};
    use crate::error::CODE_NOT_IMPLEMENTED;
    use rstest::rstest;
    use std::path::PathBuf;

    fn host(locale: Locale) -> Host {
        Host::new(
            [("VIA_LOCALE", locale.as_str())].into_iter().collect(),
            PathBuf::from("/home/via"),
            PathBuf::from("/srv/via"),
        )
    }

    fn run_args(args: &[&str]) -> Result<(), CliError> {
        run_args_in(Locale::Zh, args)
    }

    fn run_args_in(locale: Locale, args: &[&str]) -> Result<(), CliError> {
        let cli: Cli = parse_from(args.iter().map(|s| (*s).to_owned())).expect("parses");
        let Command::Backend(backend) = cli.command else {
            panic!("{args:?} is not a backend invocation");
        };
        run(&backend, &host(locale))
    }

    #[rstest]
    #[case(&["backend", "install", "codex"], "backend install codex")]
    #[case(&["backend", "install", "OPENCODE"], "backend install opencode")]
    #[case(&["backend", "install", "kimi", "--yes"], "backend install kimi")]
    #[case(&["backend", "install", "kimi", "-y"], "backend install kimi")]
    #[case(&["backend", "status"], "backend status")]
    #[case(&["backend", "status", "--json"], "backend status")]
    #[case(&["backend", "status", "--backend", "codex"], "backend status")]
    #[case(&["backend", "auth", "claude"], "backend auth claude")]
    fn a_valid_invocation_refuses_naming_phase_two(#[case] args: &[&str], #[case] command: &str) {
        let error = run_args_in(Locale::En, args).expect_err("the body lands in phase 2");
        assert_eq!(error.code(), CODE_NOT_IMPLEMENTED);
        let message = error.message(Locale::En);
        assert!(message.contains(command), "{message}");
        assert!(message.contains('2'), "{message}");
    }

    #[rstest]
    #[case(&["backend", "install"])]
    #[case(&["backend", "install", "none"])]
    #[case(&["backend", "install", "NONE"])]
    #[case(&["backend", "install", "--yes"])]
    fn a_missing_install_target_lists_every_backend(#[case] args: &[&str]) {
        let error = run_args(args).expect_err("install needs a name");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
        let message = error.message(Locale::Zh);
        assert_eq!(
            message,
            std::format!(
                "install 缺少后台名称（可选：{}）",
                via_catalog::backend_names().join("、")
            )
        );
    }

    #[test]
    fn an_unknown_install_target_is_refused_by_the_catalog() {
        let error = run_args(&["backend", "install", "opencde"]).expect_err("typo");
        assert_eq!(error.code(), "VIA_BACKEND_UNSUPPORTED");
        assert_eq!(error.message(Locale::Zh), "不支持的后台 Agent：opencde");
    }

    #[test]
    fn generic_acp_is_never_installed() {
        let error = run_args(&["backend", "install", "acp"]).expect_err("nothing to fetch");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
        assert_eq!(
            error.message(Locale::Zh),
            "通用 ACP 接入的 Agent 请自行安装，并通过 ACP_COMMAND 配置"
        );
        let english =
            run_args_in(Locale::En, &["backend", "install", "acp"]).expect_err("nothing to fetch");
        assert!(english.message(Locale::En).contains("ACP_COMMAND"));
    }

    #[test]
    fn the_generic_acp_id_is_the_one_backend_the_catalog_never_installs() {
        let uninstallable: Vec<&str> = via_catalog::backend_definitions()
            .iter()
            .filter(|definition| definition.lifecycle.installation.is_none())
            .map(|definition| definition.id)
            .collect();
        assert_eq!(uninstallable, vec![GENERIC_ACP_ID]);
    }

    #[rstest]
    #[case("nonesuch")]
    #[case("none")]
    #[case("")]
    fn an_unknown_status_selection_is_refused(#[case] requested: &str) {
        let error = run_args(&["backend", "status", "--backend", requested])
            .expect_err("there is nothing to report on");
        assert_eq!(error.code(), "VIA_BACKEND_UNSUPPORTED");
    }

    #[test]
    fn a_known_status_selection_reaches_the_phase_two_refusal() {
        let error = run_args(&["backend", "status", "--backend", "PI"])
            .expect_err("the body lands in phase 2");
        assert_eq!(error.code(), CODE_NOT_IMPLEMENTED);
    }

    #[test]
    fn none_is_the_catalogs_sentinel_not_a_backend() {
        assert_eq!(via_catalog::BACKEND_NONE_SENTINEL, "none");
        assert!(backend_definition(via_catalog::BACKEND_NONE_SENTINEL).is_none());
    }
}
