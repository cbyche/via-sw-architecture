//! The six verbs, and the one entry point that dispatches to them.

pub mod backend;
pub mod chat;
pub mod config;
pub mod gateway;
pub mod mcp_serve;
pub mod service;

use std::io::Write;

use via_i18n::{Locale, keys, t};

use crate::cli::{Cli, Command};
use crate::configfile::REALTIME_MODEL_KEY;
use crate::error::CliError;
use crate::host::Host;

/// What the process environment said *before* `config.env` was merged into it.
///
/// **External contract** — `cli/src/launcher.mjs:182-184` captures
/// `String(env.<PREFIX>_REALTIME_MODEL || '').trim()` on the first line of
/// `main`, *before* `prepareEnvironment` runs. That ordering is the whole
/// point of the value: after the merge, a model set in `config.env` is
/// indistinguishable from one exported in the shell, and `config set` would
/// then warn about an environment override that does not exist.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Prelude {
    /// The trimmed `VIA_REALTIME_MODEL` from the process environment alone.
    pub realtime_model_override: String,
}

impl Prelude {
    /// Capture the values that must be read before any file is loaded.
    #[must_use]
    pub fn capture(host: &Host) -> Self {
        Self {
            realtime_model_override: host
                .env()
                .get(REALTIME_MODEL_KEY)
                .unwrap_or_default()
                .trim()
                .to_owned(),
        }
    }
}

/// Join `items` with the locale's list separator.
///
/// **External contract** — upstream interpolates
/// `backendNames().join('、')` into three catalogued messages
/// (*error-code/install target validation*, and the two `--backend` help
/// lines). The separator is Chinese punctuation, so it is a `via-i18n` value
/// (`cli.list_separator`) rather than a literal here: `en` and `ko` join with
/// `", "`, `zh` with `、`, and the `zh` rendering is byte-identical to
/// upstream's.
#[must_use]
pub fn join_list(locale: Locale, items: &[&str]) -> String {
    items.join(t(locale, keys::CLI_LIST_SEPARATOR))
}

/// Run whichever command was parsed.
///
/// The order is upstream's `main` (`cli/src/launcher.mjs:182-188`), and each
/// step is here because the next one depends on it:
///
/// 1. capture the [`Prelude`] — before any file touches the environment;
/// 2. load the runtime environment, read-only for a command that only
///    inspects the machine;
/// 3. dispatch.
///
/// # Errors
///
/// Whatever the command refuses with. Four of the six always refuse: their
/// argument surface is real and their body belongs to a later phase.
pub fn dispatch(cli: &Cli, host: &mut Host, out: &mut dyn Write) -> Result<(), CliError> {
    let prelude = Prelude::capture(host);
    let runtime = host.load_runtime(cli.command.is_read_only())?;
    match &cli.command {
        Command::Gateway(args) => gateway::run(args, host, out),
        Command::Chat(args) => chat::run(args, host, out),
        Command::Config(args) => config::run(args, host, &runtime, &prelude, out),
        Command::Backend(args) => backend::run(args, host),
        Command::McpServe(args) => mcp_serve::run(args, host),
        Command::Service(args) => service::run(args, host),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_separator_reproduces_upstreams_ideographic_comma_in_chinese() {
        assert_eq!(join_list(Locale::Zh, &["a", "b", "c"]), "a、b、c");
        assert_eq!(join_list(Locale::En, &["a", "b", "c"]), "a, b, c");
        assert_eq!(join_list(Locale::Ko, &["a", "b", "c"]), "a, b, c");
        assert_eq!(join_list(Locale::Zh, &["only"]), "only");
        assert_eq!(join_list(Locale::Zh, &[]), "");
    }

    #[test]
    fn the_prelude_reads_the_process_environment_and_trims_it() {
        let host = Host::new(
            [("VIA_REALTIME_MODEL", "  m  ")].into_iter().collect(),
            "/home/via".into(),
            "/srv/via".into(),
        );
        assert_eq!(Prelude::capture(&host).realtime_model_override, "m");
    }

    #[test]
    fn an_absent_override_captures_as_empty() {
        let host = Host::new(Default::default(), "/home/via".into(), "/srv/via".into());
        assert_eq!(Prelude::capture(&host).realtime_model_override, "");
    }
}
