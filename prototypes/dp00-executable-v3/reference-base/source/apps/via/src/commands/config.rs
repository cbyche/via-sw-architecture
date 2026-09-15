//! `via config` — the one command phase 1 implements end to end.
//!
//! Ported from `cli/src/config-command.mjs` plus its three call sites in
//! `cli/src/launcher.mjs:193-215`. Three shapes:
//!
//! | Invocation | Output |
//! | --- | --- |
//! | `via config` | the `config.env` path |
//! | `via config show` | the effective Realtime model and the catalog |
//! | `via config set --realtime-model ID` | rewrites `config.env`, then a follow-up |
//!
//! `config show` is **read-only** (`cli/src/launcher.mjs:186`): asking a
//! machine what model it would use must not create a configuration directory
//! on it. `via config` with no action and `via config set` both scaffold,
//! which is upstream's split and is why the bare form can answer with a path
//! that exists.

use std::io::Write;

use via_core::runtime::RuntimeEnvironment;
use via_i18n::{Locale, format, keys, t};

use crate::cli::{ConfigAction, ConfigArgs};
use crate::commands::Prelude;
use crate::configfile::{read_config_file, resolve_config_model, update_realtime_model};
use crate::error::{CODE_INVALID_ARGUMENT, CliError};
use crate::host::Host;

/// Run `via config`.
///
/// # Errors
///
/// * [`CliError::Refused`] for an unknown action, or `config set` with no
///   `--realtime-model`.
/// * [`CliError::Core`] for a model id the Realtime catalog does not have.
/// * [`CliError::Io`] for a read, write or lock failure on `config.env`.
pub fn run(
    args: &ConfigArgs,
    host: &Host,
    runtime: &RuntimeEnvironment,
    prelude: &Prelude,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let locale = host.locale();
    let path = runtime.config_path.as_path();
    match &args.action {
        // `stdout.write(`${configPath}\n`)` — `cli/src/launcher.mjs:197`.
        None => write_line(out, &path.display().to_string(), path),
        Some(ConfigAction::Show) => {
            let contents = read_config_file(path)?;
            let report = show(host, &contents);
            write_line(out, &report, path)
        }
        Some(ConfigAction::Set { realtime_model }) => {
            // `!options.realtimeModel` — an empty value is as absent as no
            // value at all (`cli/src/launcher.mjs:177-179`).
            let model = realtime_model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    CliError::refused(
                        CODE_INVALID_ARGUMENT,
                        locale,
                        keys::CLI_CONFIG_SET_NEEDS_REALTIME_MODEL,
                    )
                })?;
            update_realtime_model(path, model, locale)?;
            write_follow_up(out, locale, prelude, model, path)
        }
        Some(ConfigAction::Unknown(words)) => Err(CliError::refused_with(
            CODE_INVALID_ARGUMENT,
            locale,
            keys::CLI_UNKNOWN_CONFIG_COMMAND,
            &[("command", words.first().map_or("", String::as_str))],
        )),
    }
}

/// The `config show` report.
///
/// **External contract** — `docs/reference/contracts.json` *prompt-text/config
/// show output*, `cli/src/config-command.mjs:98-105`:
///
/// ```text
/// Realtime 模型：<model>
/// 可用 Realtime 模型：
/// - <id>（<label>）
/// ```
///
/// The ids and labels are the DashScope catalog's, **in catalog order**,
/// because they are provider-facing names sent to DashScope verbatim. They
/// come from `via-catalog`; nothing here retypes one.
///
/// No credential appears in this output, which is the reason the command
/// exists in the first place — `via config` alone prints a path a user might
/// then `cat`, and this is the answer for the question they usually meant.
#[must_use]
pub fn show(host: &Host, contents: &str) -> String {
    let locale = host.locale();
    let model = resolve_config_model(host.env(), contents);
    let mut lines = vec![
        format(locale, keys::CLI_CONFIG_SHOW_MODEL, &[("model", &model)]),
        t(locale, keys::CLI_CONFIG_SHOW_AVAILABLE).to_owned(),
    ];
    lines.extend(
        via_catalog::dashscope_realtime_model_profiles()
            .iter()
            .map(|profile| {
                format(
                    locale,
                    keys::CLI_CONFIG_SHOW_MODEL_ITEM,
                    &[("id", &profile.id), ("label", &profile.label)],
                )
            }),
    );
    lines.join("\n")
}

/// The sentence printed after a successful `config set`.
///
/// **External contract** — `docs/reference/contracts.json` *prompt-text/config
/// set follow-up*, `cli/src/launcher.mjs:201-212`. Two sentences, and which
/// one is printed is the useful part: when the *process* environment pins a
/// different model, editing the file changes nothing the Gateway will see, and
/// saying "restart and you're done" would be wrong.
///
/// The comparison is against the value captured in the [`Prelude`], before
/// `config.env` was merged into the environment — a model configured in the
/// file is not an override of the file.
fn write_follow_up(
    out: &mut dyn Write,
    locale: Locale,
    prelude: &Prelude,
    model: &str,
    path: &std::path::Path,
) -> Result<(), CliError> {
    let override_differs =
        !prelude.realtime_model_override.is_empty() && prelude.realtime_model_override != model;
    if override_differs {
        // This catalog value already ends with its newline; the other does
        // not. That asymmetry is upstream's own — one sentence is written as a
        // template literal ending in `\n`, the other is a constant the caller
        // appends to — and it is reproduced rather than normalised so the two
        // stay byte-comparable against the catalogue.
        let message = t(locale, keys::CLI_CONFIG_UPDATED_ENV_OVERRIDES);
        out.write_all(message.as_bytes())
            .map_err(|error| CliError::io("write", path, error))
    } else {
        write_line(out, t(locale, keys::CLI_CONFIG_UPDATED_RESTART), path)
    }
}

/// Write `text` and one newline.
fn write_line(out: &mut dyn Write, text: &str, path: &std::path::Path) -> Result<(), CliError> {
    writeln!(out, "{text}").map_err(|error| CliError::io("write", path, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::parse_from;
    use crate::cli::{Cli, Command};
    use crate::commands::dispatch;
    use pretty_assertions::assert_eq;
    use std::path::{Path, PathBuf};

    /// A host pointed at a throwaway configuration directory.
    fn host_in(dir: &Path, extra: &[(&str, &str)]) -> Host {
        let mut pairs: Vec<(&str, String)> = vec![
            ("VIA_CONFIG_DIR", dir.display().to_string()),
            ("VIA_DATA_DIR", dir.display().to_string()),
            // Keep the seeded templates and every message in one language, so
            // an assertion about the output is an assertion about this code.
            ("VIA_LOCALE", "en".to_owned()),
        ];
        pairs.extend(extra.iter().map(|(k, v)| (*k, (*v).to_owned())));
        Host::new(
            pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
            PathBuf::from("/home/via"),
            dir.to_path_buf(),
        )
    }

    fn run_cli(
        dir: &Path,
        extra: &[(&str, &str)],
        args: &[&str],
    ) -> (String, Result<(), CliError>) {
        let cli: Cli = parse_from(args.iter().map(|s| (*s).to_owned())).expect("parses");
        let mut host = host_in(dir, extra);
        let mut out: Vec<u8> = Vec::new();
        let outcome = dispatch(&cli, &mut host, &mut out);
        (String::from_utf8(out).expect("utf-8"), outcome)
    }

    #[test]
    fn the_bare_form_prints_the_config_env_path_and_scaffolds_it() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let (stdout, outcome) = run_cli(dir.path(), &[], &["config"]);
        outcome.expect("`via config` succeeds");
        let expected = dir.path().join("config.env");
        assert_eq!(stdout, std::format!("{}\n", expected.display()));
        assert!(expected.exists(), "the bare form is not read-only");
    }

    #[test]
    fn show_does_not_scaffold() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let nested = dir.path().join("never-created");
        let (_, outcome) = run_cli(&nested, &[], &["config", "show"]);
        outcome.expect("`via config show` succeeds on an unconfigured machine");
        assert!(
            !nested.exists(),
            "a read-only command created a directory on a machine it was only asked about"
        );
    }

    #[test]
    fn show_reports_the_default_model_and_the_whole_catalog_in_order() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let (stdout, outcome) = run_cli(dir.path(), &[], &["config", "show"]);
        outcome.expect("succeeds");
        let mut lines = stdout.lines();
        assert_eq!(
            lines.next(),
            Some(
                std::format!(
                    "Realtime model: {}",
                    via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL
                )
                .as_str()
            )
        );
        assert_eq!(lines.next(), Some("Available Realtime models:"));
        for profile in via_catalog::dashscope_realtime_model_profiles() {
            assert_eq!(
                lines.next(),
                Some(std::format!("- {} ({})", profile.id, profile.label).as_str())
            );
        }
        assert_eq!(lines.next(), None, "nothing follows the catalog");
    }

    #[test]
    fn show_prints_no_credential_even_when_one_is_configured() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(
            dir.path().join("config.env"),
            "DASHSCOPE_API_KEY=sk-do-not-print-me\n",
        )
        .expect("seed");
        let (stdout, outcome) = run_cli(dir.path(), &[], &["config", "show"]);
        outcome.expect("succeeds");
        assert!(!stdout.contains("sk-do-not-print-me"), "{stdout}");
        assert!(!stdout.contains("DASHSCOPE"), "{stdout}");
    }

    #[test]
    fn show_reads_the_model_out_of_the_config_file() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let model = via_catalog::realtime_model::DASHSCOPE_OMNI_FLASH_REALTIME_MODEL;
        std::fs::write(
            dir.path().join("config.env"),
            std::format!("VIA_REALTIME_MODEL={model}\n"),
        )
        .expect("seed");
        let (stdout, outcome) = run_cli(dir.path(), &[], &["config", "show"]);
        outcome.expect("succeeds");
        assert!(
            stdout.starts_with(&std::format!("Realtime model: {model}\n")),
            "{stdout}"
        );
    }

    #[test]
    fn set_rewrites_the_file_and_asks_for_a_restart() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = dir.path().join("config.env");
        std::fs::write(&path, "# hand written\nDASHSCOPE_API_KEY=sk\n").expect("seed");
        let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
        let (stdout, outcome) = run_cli(
            dir.path(),
            &[],
            &["config", "set", "--realtime-model", model],
        );
        outcome.expect("succeeds");
        assert_eq!(
            stdout,
            "Configuration updated; run `via gateway restart` so the Gateway uses the new model\n"
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            std::format!("# hand written\nDASHSCOPE_API_KEY=sk\nVIA_REALTIME_MODEL={model}\n")
        );
    }

    #[test]
    fn set_warns_when_the_process_environment_still_overrides_the_file() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let pinned = via_catalog::realtime_model::DASHSCOPE_OMNI_FLASH_REALTIME_MODEL;
        let written = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
        let (stdout, outcome) = run_cli(
            dir.path(),
            &[("VIA_REALTIME_MODEL", pinned)],
            &["config", "set", "--realtime-model", written],
        );
        outcome.expect("succeeds");
        assert!(
            stdout.contains("VIA_REALTIME_MODEL environment variable still overrides"),
            "{stdout}"
        );
        assert!(stdout.ends_with('\n'), "{stdout:?}");
    }

    #[test]
    fn set_does_not_warn_when_the_environment_pins_the_same_model() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
        let (stdout, outcome) = run_cli(
            dir.path(),
            &[("VIA_REALTIME_MODEL", model)],
            &["config", "set", "--realtime-model", model],
        );
        outcome.expect("succeeds");
        assert!(stdout.starts_with("Configuration updated;"), "{stdout}");
    }

    #[test]
    fn a_model_configured_in_the_file_is_not_an_environment_override() {
        // The regression this pins: if the override were read *after*
        // `config.env` is merged into the environment, every `config set` on a
        // machine that already had a model would print the wrong follow-up.
        let dir = tempfile::TempDir::new().expect("temp dir");
        let old = via_catalog::realtime_model::DASHSCOPE_OMNI_FLASH_REALTIME_MODEL;
        let new = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
        std::fs::write(
            dir.path().join("config.env"),
            std::format!("VIA_REALTIME_MODEL={old}\n"),
        )
        .expect("seed");
        let (stdout, outcome) =
            run_cli(dir.path(), &[], &["config", "set", "--realtime-model", new]);
        outcome.expect("succeeds");
        assert!(stdout.starts_with("Configuration updated;"), "{stdout}");
    }

    #[test]
    fn set_without_a_model_is_the_catalogued_refusal() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let (stdout, outcome) = run_cli(dir.path(), &[], &["config", "set"]);
        let error = outcome.expect_err("`config set` needs --realtime-model");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
        assert_eq!(
            error.message(Locale::En),
            "`config set` needs --realtime-model"
        );
        assert!(stdout.is_empty(), "nothing is printed on a refusal");

        let (_, chinese) = run_cli(dir.path(), &[("VIA_LOCALE", "zh")], &["config", "set"]);
        assert_eq!(
            chinese.expect_err("still refused").message(Locale::Zh),
            "config set 需要 --realtime-model"
        );
    }

    #[test]
    fn set_with_a_blank_model_is_refused_the_same_way() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let (_, outcome) = run_cli(
            dir.path(),
            &[],
            &["config", "set", "--realtime-model", "  "],
        );
        let error = outcome.expect_err("a blank value is as absent as no value");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
    }

    #[test]
    fn set_with_an_unknown_model_refuses_and_leaves_the_file_alone() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = dir.path().join("config.env");
        std::fs::write(&path, "A=1\n").expect("seed");
        let (_, outcome) = run_cli(
            dir.path(),
            &[],
            &["config", "set", "--realtime-model", "not-a-model"],
        );
        let error = outcome.expect_err("the catalog decides");
        assert_eq!(error.code(), "VIA_REALTIME_MODEL_UNKNOWN");
        assert_eq!(
            error.message(Locale::Zh),
            "不支持的 Realtime 模型：not-a-model"
        );
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "A=1\n");
    }

    #[test]
    fn an_unknown_action_names_itself_in_the_catalogued_sentence() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let (_, outcome) = run_cli(dir.path(), &[], &["config", "bogus"]);
        let error = outcome.expect_err("not a config action");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
        assert_eq!(error.message(Locale::En), "unknown config command: bogus");

        let (_, chinese) = run_cli(dir.path(), &[("VIA_LOCALE", "zh")], &["config", "bogus"]);
        assert_eq!(
            chinese
                .expect_err("not a config action")
                .message(Locale::Zh),
            "未知 config 命令：bogus"
        );
    }

    #[test]
    fn the_three_locales_render_the_show_report_differently() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let mut rendered = Vec::new();
        for locale in ["en", "zh", "ko"] {
            let (stdout, outcome) =
                run_cli(dir.path(), &[("VIA_LOCALE", locale)], &["config", "show"]);
            outcome.expect("succeeds");
            rendered.push(stdout);
        }
        assert_ne!(rendered[0], rendered[1]);
        assert_ne!(rendered[1], rendered[2]);
        // The `zh` item line is upstream's, fullwidth brackets included.
        let first = via_catalog::dashscope_realtime_model_profiles()
            .first()
            .expect("the catalog is not empty");
        assert!(
            rendered[1].contains(&std::format!("- {}（{}）", first.id, first.label)),
            "{}",
            rendered[1]
        );
    }

    #[test]
    fn the_command_dispatched_is_config() {
        let cli: Cli = parse_from(["config"]).expect("parses");
        assert!(matches!(cli.command, Command::Config(_)));
    }
}
