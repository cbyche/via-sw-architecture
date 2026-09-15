//! The argument surface.
//!
//! `docs/architecture.md` §10 is the verb set; `docs/reference/contracts.json`
//! is the flag set. This module is where the two meet, and it is written with
//! `clap`'s derive API for one reason above all the others: **`#[arg(env =
//! "VIA_…")]` puts the environment surface into `--help`**. Upstream's
//! environment inputs are catalogued as *env-var/CLI env inputs* precisely
//! because they are invisible in its own help text — a reader has to find
//! `cli/src/arguments.mjs:103-114` to learn that `--url` has an environment
//! twin. Here the help output *is* the catalogue entry, and
//! `tests/help_surface.rs` snapshots it so a flag cannot be added, renamed or
//! silently dropped without the diff showing up in review.
//!
//! # Where VIA's verbs differ from upstream's
//!
//! Upstream's eight are `gateway, tui, webui, status, config, setup, install,
//! skill` (*cli-commands/COMMANDS*). §10's six are `gateway, chat, config,
//! backend, mcp-serve, service`. The mapping, recorded in
//! `docs/deviations/phase-1.md`:
//!
//! | Upstream | VIA | Why |
//! | --- | --- | --- |
//! | `gateway run` | `via gateway` | The foreground Gateway is the verb; the service verbs move under `service`. |
//! | `gateway install\|start\|stop` | `via service install\|start\|stop` | §10. `restart`, `status` and `uninstall` have no §10 verb and are owed. |
//! | `tui` | `via chat` | `tui/` is deferred (`docs/fidelity.md`); `via chat` is the port of `tui/src/text-cli.mjs`. |
//! | `webui` | — | `web/` is dropped, so `--no-open` goes with it. |
//! | `status` | — | An alias for `gateway status`, which is owed with it. |
//! | `setup` | `via backend status` | Same job: read-only backend readiness, including `--json`. |
//! | `install NAME` | `via backend install NAME` | Same job, including `--yes`. |
//! | `skill` | — | A branded passthrough over `skills.sh`; §10 has no verb for it, so `--skill` and `--list` go with it. |
//!
//! # Where `clap` differs from a hand-rolled parser
//!
//! Upstream accepts every flag on every command and then rejects the
//! misplaced ones one at a time — `--json 只适用于 setup`, `--yes 只适用于
//! install`, `--audio-mode 只适用于 tui`. Declaring each flag on the command
//! that owns it gets the same refusals structurally, and gets them *before*
//! any body runs. The message is then `clap`'s rather than the catalogued
//! sentence; the constraint is identical and is asserted directly
//! (`tests/flag_surface.rs`).

use clap::{Args, Parser, Subcommand};

use crate::origin::DEFAULT_GATEWAY_URL;

/// The verb a bare `via` runs.
///
/// **External contract** — `docs/reference/contracts.json`
/// *cli-commands/COMMANDS*, `cli/src/arguments.mjs:57`: *"default when
/// argv\[0\] is absent or starts with '-': gateway"*.
pub const DEFAULT_COMMAND: &str = "gateway";

/// The flags that are the binary's own rather than a command's, and therefore
/// must not trigger [`DEFAULT_COMMAND`] insertion.
///
/// Upstream has no such exception because its `--help` is handled inside the
/// defaulted `gateway` command. Here `via --help` prints the *top-level* help,
/// which is the one that lists the six verbs — the more useful answer, and the
/// one a reader typing `via --help` is asking for.
pub const BINARY_FLAGS: [&str; 4] = ["-h", "--help", "-V", "--version"];

/// The environment variable naming the Gateway to talk to.
///
/// **External contract** — `cli/src/arguments.mjs:103`, renamed by
/// `docs/rebrand.md` row 93.
pub const URL_ENV: &str = "VIA_URL";

/// VIA — the Voice Interaction Agent gateway.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(
    name = crate::BINARY_NAME,
    version,
    about = "VIA — the Voice Interaction Agent gateway",
    long_about = "VIA — the Voice Interaction Agent gateway.\n\n\
                  Run `via gateway` to serve, `via chat` to talk to a running \
                  Gateway, and `via config` to see or change what either of \
                  them will use.",
    term_width = 100,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// The verb.
    #[command(subcommand)]
    pub command: Command,
}

/// `docs/architecture.md` §10's six verbs.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Run the Gateway in the foreground.
    ///
    /// Resolves the configuration, refuses to start unconfigured, and takes
    /// the single-instance lease before anything else happens.
    #[command(
        about = "Run the Gateway in the foreground",
        long_about = "Run the Gateway in the foreground.\n\n\
                      Resolves the configuration, refuses to start unconfigured, \
                      and takes the single-instance lease before anything else \
                      happens."
    )]
    Gateway(GatewayArgs),

    /// Text client: drive a running Gateway with no audio and no model
    /// weights.
    #[command(about = "Text client: drive a running Gateway with no audio and no model weights")]
    Chat(ChatArgs),

    /// Show or edit `config.env`.
    #[command(about = "Show or edit config.env")]
    Config(ConfigArgs),

    /// Backend agent lifecycle.
    #[command(about = "Backend agent lifecycle: install, status, auth")]
    Backend(BackendArgs),

    /// stdio MCP server exposing the five coordination tools.
    #[command(
        name = "mcp-serve",
        about = "stdio MCP server exposing the five coordination tools"
    )]
    McpServe(McpServeArgs),

    /// launchd / systemd unit for the Gateway.
    #[command(about = "launchd / systemd unit for the Gateway")]
    Service(ServiceArgs),
}

impl Command {
    /// The command's name as a user spells it, for a refusal that names it.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Gateway(_) => "gateway".to_owned(),
            Self::Chat(_) => "chat".to_owned(),
            Self::Config(_) => "config".to_owned(),
            Self::Backend(args) => std::format!("backend {}", args.action.name()),
            Self::McpServe(_) => "mcp-serve".to_owned(),
            Self::Service(args) => std::format!("service {}", args.action.name()),
        }
    }

    /// Whether this command only *inspects* the machine.
    ///
    /// **External contract** — `cli/src/launcher.mjs:185-187`:
    /// `['setup', 'install'].includes(argv[0]) || (argv[0] === 'config' &&
    /// argv[1] === 'show')`. A read-only command must not scaffold a
    /// configuration directory on a machine somebody is only asking about.
    #[must_use]
    pub fn is_read_only(&self) -> bool {
        match self {
            Self::Config(args) => matches!(args.action, Some(ConfigAction::Show)),
            Self::Backend(args) => matches!(
                args.action,
                BackendAction::Status(_) | BackendAction::Install(_)
            ),
            _ => false,
        }
    }
}

/// `via gateway` — the foreground Gateway.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GatewayArgs {
    /// Gateway address. Only `http` and `https` are accepted; the path,
    /// query and fragment are discarded and the origin is kept.
    ///
    /// **External contract** — *cli-flag/--url*.
    #[arg(
        long,
        value_name = "URL",
        env = URL_ENV,
        hide_env_values = true,
        default_value = DEFAULT_GATEWAY_URL,
        help = "Gateway address (http or https; normalised to its origin)",
        long_help = "Gateway address (http or https; normalised to its origin)",
    )]
    pub url: String,

    /// Backend agent id, or `none` for voice chat only.
    ///
    /// **External contract** — *cli-flag/--backend*. The value is trimmed and
    /// lowercased, and `none` in any case means the same as unset.
    #[arg(
        long,
        value_name = "NAME",
        env = "AGENT_PROTOCOL",
        hide_env_values = true,
        help = "Backend agent id, or `none` for voice chat only",
        long_help = "Backend agent id, or `none` for voice chat only"
    )]
    pub backend: Option<String>,

    /// `native` (the backend asks for itself) or `full` (the Gateway's unified
    /// highest permission).
    ///
    /// **External contract** — *cli-flag/--backend-permission-mode*.
    #[arg(
        long = "backend-permission-mode",
        value_name = "MODE",
        env = "VIA_BACKEND_PERMISSION_MODE",
        hide_env_values = true,
        help = "native (default) or full (the Gateway's unified highest permission)",
        long_help = "native (default) or full (the Gateway's unified highest permission)"
    )]
    pub backend_permission_mode: Option<String>,

    /// Address of an already-running backend server. Only meaningful for a
    /// backend that declares one; passing it makes the backend `external`.
    ///
    /// **External contract** — *cli-flag/--backend-url*. Deliberately without
    /// an `env` twin: upstream reads the backend's *own* variable
    /// (`OPENCODE_BASE_URL`, `OPENCLAW_BASE_URL`) for the default, and
    /// "specified on the command line" is what flips ownership.
    #[arg(
        long = "backend-url",
        value_name = "URL",
        help = "Address of an already-running backend server; passing it makes the backend external",
        long_help = "Address of an already-running backend server; passing it makes the backend external"
    )]
    pub backend_url: Option<String>,

    /// Coordinator agent id forwarded to the backend.
    ///
    /// **External contract** — *cli-flag/--backend-agent*. Trimmed.
    #[arg(
        long = "backend-agent",
        value_name = "ID",
        env = "VIA_BACKEND_AGENT",
        hide_env_values = true,
        help = "Coordinator agent id forwarded to the backend",
        long_help = "Coordinator agent id forwarded to the backend"
    )]
    pub backend_agent: Option<String>,
}

/// `via chat` — the text client.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ChatArgs {
    /// Gateway address.
    ///
    /// **External contract** — *cli-flag/--url*, and
    /// `tui/src/text-cli.mjs:19`, which reads the same variable.
    #[arg(
        long,
        value_name = "URL",
        env = URL_ENV,
        hide_env_values = true,
        default_value = DEFAULT_GATEWAY_URL,
        help = "Gateway address (http or https; normalised to its origin)",
        long_help = "Gateway address (http or https; normalised to its origin)",
    )]
    pub url: String,

    /// Voice session to join. Defaults to a fresh `voice-<32 hex>` id.
    ///
    /// **External contract** — *cli-flag/--session*.
    #[arg(
        long,
        value_name = "ID",
        env = crate::session::SESSION_ID_ENV,
        hide_env_values = true,
        help = "Voice session to join (default: a fresh voice-<32 hex> id)",
        long_help = "Voice session to join (default: a fresh voice-<32 hex> id)",
    )]
    pub session: Option<String>,

    /// Take voice control from whichever client currently holds it.
    ///
    /// **External contract** — *cli-flag/--takeover*, catalogued as `tui` and
    /// `webui` only. `webui` is dropped and `tui` is deferred; `via chat` is
    /// the client that replaces them, so the flag lives here.
    #[arg(
        long,
        help = "Take voice control from whichever client currently holds it",
        long_help = "Take voice control from whichever client currently holds it"
    )]
    pub takeover: bool,

    /// Refuse rather than start a Gateway when none is running.
    ///
    /// **VIA's own.** Upstream has no flag for it because it has no choice to
    /// express: `tui` always refuses (`cli/src/launcher.mjs:427-432`) and the
    /// desktop path always starts one (`ensureRuntime`). `via chat` is both
    /// clients at once, so which one it is has to be sayable — and the default
    /// is the useful one, because a user who typed `via chat` wants to chat.
    #[arg(
        long = "no-autostart",
        help = "Refuse rather than start a Gateway when none is running",
        long_help = "Refuse rather than start a Gateway when none is running"
    )]
    pub no_autostart: bool,
}

/// `via config` — show or edit `config.env`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ConfigArgs {
    /// `show`, `set`, or nothing at all.
    #[command(subcommand)]
    pub action: Option<ConfigAction>,
}

/// The `config` actions.
///
/// **External contract** — *cli-commands/config actions*,
/// `cli/src/arguments.mjs:59-63`: *"show, set (no action = print the config
/// file path); unknown raises `unknown config command: <x>`"*. The unknown arm
/// is a real variant rather than a `clap` "unrecognized subcommand" error,
/// because that message is catalogued and a script may match on it.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ConfigAction {
    /// Show the effective Realtime model. Prints no credentials.
    #[command(about = "Show the effective Realtime model (prints no credentials)")]
    Show,

    /// Update the Realtime model in `config.env`.
    #[command(about = "Update the Realtime model in config.env")]
    Set {
        /// The model id. Must be one the Realtime catalog knows.
        ///
        /// **External contract** — *cli-flag/--realtime-model*: *"legal only
        /// with `config set`, and `config set` without it raises `config set
        /// needs --realtime-model`"*. Optional here so that the second half of
        /// that sentence is VIA's own catalogued message rather than `clap`'s
        /// "required argument was not provided".
        #[arg(
            long = "realtime-model",
            value_name = "ID",
            help = "Realtime model id; must be one the Realtime catalog knows",
            long_help = "Realtime model id; must be one the Realtime catalog knows"
        )]
        realtime_model: Option<String>,
    },

    /// Anything else, reported as `unknown config command: <x>`.
    #[command(external_subcommand)]
    Unknown(Vec<String>),
}

/// `via backend` — backend agent lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct BackendArgs {
    /// `install`, `status` or `auth`.
    #[command(subcommand)]
    pub action: BackendAction,
}

/// The `backend` actions (`docs/architecture.md` §10).
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum BackendAction {
    /// Install a backend agent in one step, with the ACP adapter it needs.
    #[command(about = "Install a backend agent, with the ACP adapter it needs")]
    Install(BackendInstallArgs),
    /// Read-only readiness check. Installs nothing, signs in to nothing.
    #[command(about = "Read-only readiness check; installs nothing, signs in to nothing")]
    Status(BackendStatusArgs),
    /// Run a backend agent's own authentication flow.
    #[command(about = "Run a backend agent's own authentication flow")]
    Auth(BackendAuthArgs),
}

impl BackendAction {
    /// The action's name, for a refusal that names the command.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Install(_) => "install",
            Self::Status(_) => "status",
            Self::Auth(_) => "auth",
        }
    }
}

/// `via backend install`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct BackendInstallArgs {
    /// Backend id to install. Generic `acp` is not installable.
    #[arg(
        value_name = "NAME",
        help = "Backend id to install; generic acp is not installable",
        long_help = "Backend id to install; generic acp is not installable"
    )]
    pub name: Option<String>,

    /// Stop confirming each script-based install step. Use with care.
    ///
    /// **External contract** — *cli-flag/--yes / -y*, catalogued as `install`
    /// only.
    #[arg(
        long,
        short = 'y',
        help = "Stop confirming each script-based install step (use with care)",
        long_help = "Stop confirming each script-based install step (use with care)"
    )]
    pub yes: bool,
}

/// `via backend status`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct BackendStatusArgs {
    /// Check one backend only. Every backend by default.
    #[arg(
        long,
        value_name = "NAME",
        help = "Check one backend only; every backend by default",
        long_help = "Check one backend only; every backend by default"
    )]
    pub backend: Option<String>,

    /// Emit the report as JSON for a script.
    ///
    /// **External contract** — *cli-flag/--json*, catalogued as `setup` only;
    /// `via backend status` is `setup`.
    #[arg(
        long,
        help = "Emit the report as JSON for a script",
        long_help = "Emit the report as JSON for a script"
    )]
    pub json: bool,
}

/// `via backend auth`.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct BackendAuthArgs {
    /// Backend id to authenticate.
    #[arg(
        value_name = "NAME",
        help = "Backend id to authenticate",
        long_help = "Backend id to authenticate"
    )]
    pub name: Option<String>,
}

/// `via mcp-serve` — the stdio MCP server.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct McpServeArgs {
    /// Gateway the coordination tools act against.
    #[arg(
        long,
        value_name = "URL",
        env = URL_ENV,
        hide_env_values = true,
        default_value = DEFAULT_GATEWAY_URL,
        help = "Gateway the coordination tools act against",
        long_help = "Gateway the coordination tools act against",
    )]
    pub url: String,
}

/// `via service` — the launchd / systemd unit.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ServiceArgs {
    /// `install`, `start` or `stop`.
    #[command(subcommand)]
    pub action: ServiceAction,
}

/// The `service` actions (`docs/architecture.md` §10).
///
/// **Partial contract** — *cli-commands/GATEWAY_ACTIONS* lists seven verbs.
/// `run` became `via gateway`; `restart`, `status` and `uninstall` have no §10
/// verb and are recorded as owed in `docs/deviations/phase-1.md`. The unknown
/// arm carries the catalogued refusal rather than `clap`'s.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ServiceAction {
    /// Install the unit and start it.
    #[command(about = "Install the unit and start it")]
    Install,
    /// Start the installed unit.
    #[command(about = "Start the installed unit")]
    Start,
    /// Stop the running unit.
    #[command(about = "Stop the running unit")]
    Stop,
    /// Anything else, reported as `unsupported Gateway service action: <x>`.
    #[command(external_subcommand)]
    Unknown(Vec<String>),
}

impl ServiceAction {
    /// The action's name, for a refusal that names the command.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Install => "install".to_owned(),
            Self::Start => "start".to_owned(),
            Self::Stop => "stop".to_owned(),
            Self::Unknown(words) => words.first().cloned().unwrap_or_default(),
        }
    }
}

/// Whether [`DEFAULT_COMMAND`] has to be inserted ahead of `args`.
///
/// **External contract** — *cli-commands/COMMANDS*: the default applies when
/// the first argument is absent **or starts with `-`**, so `via --backend
/// openclaw` is a Gateway run, not a usage error. [`BINARY_FLAGS`] are the
/// exception; see its documentation.
#[must_use]
pub fn needs_default_command(args: &[String]) -> bool {
    match args.first() {
        None => true,
        Some(first) => first.starts_with('-') && !BINARY_FLAGS.contains(&first.as_str()),
    }
}

/// Parse `args` — the arguments *after* the program name.
///
/// # Errors
///
/// A [`clap::Error`]. `--help` and `--version` are errors in `clap`'s sense
/// and carry exit code 0; everything else is a usage failure. `main` maps both
/// onto the catalogued exit codes in [`crate::error`].
pub fn parse_from<I, S>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut argv: Vec<String> = args.into_iter().map(Into::into).collect();
    if needs_default_command(&argv) {
        argv.insert(0, DEFAULT_COMMAND.to_owned());
    }
    argv.insert(0, crate::BINARY_NAME.to_owned());
    Cli::try_parse_from(argv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use rstest::rstest;

    fn parse(args: &[&str]) -> Cli {
        parse_from(args.iter().map(|s| (*s).to_owned()))
            .unwrap_or_else(|error| panic!("{args:?} should parse: {error}"))
    }

    #[test]
    fn the_declared_surface_is_internally_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn a_bare_invocation_runs_the_gateway() {
        assert!(matches!(parse(&[]).command, Command::Gateway(_)));
    }

    #[rstest]
    #[case(&["--backend", "none"])]
    #[case(&["--backend-agent", "build"])]
    fn a_leading_flag_still_runs_the_gateway(#[case] args: &[&str]) {
        assert!(matches!(parse(args).command, Command::Gateway(_)));
    }

    #[rstest]
    #[case("-h")]
    #[case("--help")]
    #[case("-V")]
    #[case("--version")]
    fn a_binary_flag_is_not_defaulted_into_a_command(#[case] flag: &str) {
        assert!(!needs_default_command(&[flag.to_owned()]));
        let error = parse_from([flag]).expect_err("help and version are clap errors");
        assert_eq!(error.exit_code(), 0, "{flag} must not be a usage failure");
    }

    #[test]
    fn every_verb_in_section_ten_parses() {
        assert!(matches!(parse(&["gateway"]).command, Command::Gateway(_)));
        assert!(matches!(parse(&["chat"]).command, Command::Chat(_)));
        assert!(matches!(parse(&["config"]).command, Command::Config(_)));
        assert!(matches!(
            parse(&["backend", "status"]).command,
            Command::Backend(_)
        ));
        assert!(matches!(
            parse(&["mcp-serve"]).command,
            Command::McpServe(_)
        ));
        assert!(matches!(
            parse(&["service", "start"]).command,
            Command::Service(_)
        ));
    }

    #[rstest]
    // Upstream's verbs that VIA does not have. Each is a refusal, not a
    // silently-defaulted Gateway run.
    #[case("tui")]
    #[case("webui")]
    #[case("status")]
    #[case("setup")]
    #[case("install")]
    #[case("skill")]
    fn a_dropped_upstream_verb_is_a_usage_failure(#[case] verb: &str) {
        let error = parse_from([verb]).expect_err("not a VIA verb");
        assert_ne!(error.exit_code(), 0);
    }

    #[test]
    fn config_has_three_shapes() {
        assert_eq!(
            parse(&["config"]).command,
            Command::Config(ConfigArgs { action: None })
        );
        assert_eq!(
            parse(&["config", "show"]).command,
            Command::Config(ConfigArgs {
                action: Some(ConfigAction::Show)
            })
        );
        assert_eq!(
            parse(&["config", "set", "--realtime-model", "m"]).command,
            Command::Config(ConfigArgs {
                action: Some(ConfigAction::Set {
                    realtime_model: Some("m".to_owned())
                })
            })
        );
    }

    #[test]
    fn an_unknown_config_action_is_captured_rather_than_rejected_by_clap() {
        assert_eq!(
            parse(&["config", "bogus"]).command,
            Command::Config(ConfigArgs {
                action: Some(ConfigAction::Unknown(vec!["bogus".to_owned()]))
            })
        );
    }

    #[test]
    fn an_unknown_service_action_is_captured_too() {
        // `restart`, `status` and `uninstall` are upstream verbs §10 does not
        // list; they land here rather than in a `clap` error, so the refusal
        // names them.
        for action in ["restart", "status", "uninstall", "bogus"] {
            let Command::Service(args) = parse(&["service", action]).command else {
                panic!("{action} should parse as a service action");
            };
            assert_eq!(args.action.name(), action);
        }
    }

    #[test]
    fn realtime_model_is_legal_only_with_config_set() {
        for args in [
            vec!["gateway", "--realtime-model", "m"],
            vec!["chat", "--realtime-model", "m"],
            vec!["config", "show", "--realtime-model", "m"],
            vec!["backend", "status", "--realtime-model", "m"],
            vec!["service", "start", "--realtime-model", "m"],
        ] {
            let error = parse_from(args.clone()).expect_err("--realtime-model is config set only");
            assert_ne!(error.exit_code(), 0, "{args:?}");
        }
    }

    #[rstest]
    // `--json` is `setup` only; `via backend status` is `setup`.
    #[case(&["gateway", "--json"])]
    #[case(&["chat", "--json"])]
    #[case(&["backend", "install", "codex", "--json"])]
    // `--yes` is `install` only.
    #[case(&["gateway", "--yes"])]
    #[case(&["backend", "status", "--yes"])]
    #[case(&["backend", "status", "-y"])]
    // `--takeover` belongs to the client.
    #[case(&["gateway", "--takeover"])]
    #[case(&["backend", "status", "--takeover"])]
    // Configuration flags are not accepted by a service verb at all, which is
    // the structural form of `error-code/service action with config flags`.
    #[case(&["service", "start", "--url", "http://127.0.0.1:3101"])]
    #[case(&["service", "install", "--backend", "openclaw"])]
    #[case(&["service", "stop", "--backend-permission-mode", "full"])]
    // Flags whose surface VIA dropped with the command that owned them.
    #[case(&["chat", "--audio-mode", "half"])]
    #[case(&["chat", "--no-open"])]
    #[case(&["chat", "--skill", "x"])]
    #[case(&["chat", "--list"])]
    // Upstream's own removed flag, still removed.
    #[case(&["gateway", "--backend-mode", "compatible"])]
    fn a_misplaced_flag_is_a_usage_failure(#[case] args: &[&str]) {
        let error = parse_from(args.iter().map(|s| (*s).to_owned()))
            .expect_err("the flag does not exist on that command");
        assert_ne!(error.exit_code(), 0, "{args:?}");
    }

    #[rstest]
    #[case(&["gateway", "--url"])]
    #[case(&["gateway", "--backend"])]
    #[case(&["gateway", "--backend-agent"])]
    #[case(&["chat", "--session"])]
    #[case(&["config", "set", "--realtime-model"])]
    fn a_flag_without_its_value_is_a_usage_failure(#[case] args: &[&str]) {
        let error =
            parse_from(args.iter().map(|s| (*s).to_owned())).expect_err("the flag takes a value");
        assert_ne!(error.exit_code(), 0, "{args:?}");
    }

    #[test]
    fn the_gateway_url_defaults_to_the_catalogued_address() {
        // Only meaningful when the developer's own `VIA_URL` is unset; when it
        // is set, `clap` is supposed to prefer it, and that is asserted by the
        // integration tests, which control the child's environment.
        if std::env::var_os(URL_ENV).is_none() {
            let Command::Gateway(args) = parse(&["gateway"]).command else {
                panic!("gateway");
            };
            assert_eq!(args.url, DEFAULT_GATEWAY_URL);
        }
    }

    #[test]
    fn the_backend_flags_are_optional_and_absent_by_default() {
        let Command::Gateway(args) = parse(&["gateway"]).command else {
            panic!("gateway");
        };
        assert_eq!(args.backend_url, None);
    }

    #[test]
    fn read_only_is_exactly_config_show_backend_status_and_backend_install() {
        assert!(parse(&["config", "show"]).command.is_read_only());
        assert!(parse(&["backend", "status"]).command.is_read_only());
        assert!(
            parse(&["backend", "install", "codex"])
                .command
                .is_read_only()
        );
        assert!(!parse(&["config"]).command.is_read_only());
        assert!(
            !parse(&["config", "set", "--realtime-model", "m"])
                .command
                .is_read_only()
        );
        assert!(!parse(&["gateway"]).command.is_read_only());
        assert!(!parse(&["chat"]).command.is_read_only());
        assert!(!parse(&["backend", "auth", "codex"]).command.is_read_only());
        assert!(!parse(&["service", "start"]).command.is_read_only());
        assert!(!parse(&["mcp-serve"]).command.is_read_only());
    }

    #[test]
    fn a_command_names_itself_including_its_action() {
        assert_eq!(parse(&["gateway"]).command.name(), "gateway");
        assert_eq!(parse(&["backend", "auth"]).command.name(), "backend auth");
        assert_eq!(parse(&["service", "stop"]).command.name(), "service stop");
        assert_eq!(parse(&["service", "bogus"]).command.name(), "service bogus");
    }
}
