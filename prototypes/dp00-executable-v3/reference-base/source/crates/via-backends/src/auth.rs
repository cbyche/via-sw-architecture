//! Read-only inspection of a backend's own authentication.
//!
//! Ported from `shared/backend-auth-status.mjs`. Upstream's own header states
//! the rule the whole module exists to keep:
//!
//! > Authentication belongs to each backend, and is not part of ACP. This only
//! > runs official, read-only status checks; when it cannot tell, it **must**
//! > stay `unknown` — a failed check is not "logged out".
//!
//! `docs/reference/contracts.json` (`state-name/authentication status values`)
//! says why: *"collapsing unknown to unauthenticated would nag users who are in
//! fact logged in via keychain."* Every branch below either proves a state or
//! answers [`AuthStatus::Unknown`], and `tests/auth_status.rs` walks the
//! inconclusive cases specifically.
//!
//! # The files this module reads belong to other products
//!
//! `~/.qwen/settings.json`, `~/.pi/agent/settings.json`, `~/.dsh/.credentials.yaml`,
//! CodeBuddy's `auth` directory and `~/.openclaw/openclaw.json` are third-party
//! layouts (`file-path/third-party credential probe paths`). They are read, never
//! written, never relocated and never rebranded.

use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use via_catalog::backend::{AuthProbe, CommandParser, ProbeKind};
use via_core::EnvMap;

use crate::platform::HostPlatform;

/// How long a backend's status command may run.
///
/// **External contract** — `shared/backend-auth-status.mjs:187`
/// (`timeoutMs = 8_000`).
pub const PROBE_TIMEOUT: Duration = Duration::from_millis(8_000);

/// How much of a status command's output is read.
///
/// **External contract** — `shared/backend-auth-status.mjs:8`
/// (`MAX_OUTPUT = 256 * 1024`).
pub const MAX_PROBE_OUTPUT: usize = 262_144;

/// The environment variables that count as Qwen Code credentials.
///
/// **External contract** — `shared/backend-auth-status.mjs:37-42`. All four are
/// KEEP: they belong to Alibaba's Qwen Code CLI and to DashScope, not to VIA.
pub const QWEN_CREDENTIAL_KEYS: [&str; 4] = [
    "DASHSCOPE_API_KEY",
    "OPENAI_API_KEY",
    "QWEN_API_KEY",
    "QWEN_OAUTH_TOKEN",
];

/// What VIA believes about a backend's sign-in.
///
/// **External contract** — `state-name/authentication status values`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthStatus {
    /// Proven signed in.
    Authenticated,
    /// Proven signed out.
    Unauthenticated,
    /// Not determinable. **The answer whenever a probe is inconclusive.**
    Unknown,
}

impl AuthStatus {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authenticated => "authenticated",
            Self::Unauthenticated => "unauthenticated",
            Self::Unknown => "unknown",
        }
    }
}

/// One status-command invocation.
#[derive(Debug, Clone)]
pub struct ProbeRequest {
    /// The backend's own executable, already resolved.
    pub command: String,
    /// Its arguments, from the catalogued probe.
    pub arguments: Vec<String>,
    /// The child environment.
    pub environment: EnvMap,
    /// The deadline.
    pub timeout: Duration,
}

/// What a status command produced.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProbeOutcome {
    /// Whether it exited zero.
    ///
    /// Read by upstream only for the Pi probe's JSON branch; the text parsers
    /// deliberately ignore it, because several CLIs report "not logged in" with
    /// a non-zero status.
    pub ok: bool,
    /// stdout and stderr, merged.
    pub output: String,
}

/// Running a backend's own status command.
#[async_trait]
pub trait ProbeRunner: Send + Sync {
    /// Run it. Must not fail: a probe that could not run produced no output,
    /// and no output is [`AuthStatus::Unknown`].
    async fn run(&self, request: ProbeRequest) -> ProbeOutcome;
}

/// A runner that never runs anything.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoProbeRunner;

#[async_trait]
impl ProbeRunner for NoProbeRunner {
    async fn run(&self, _request: ProbeRequest) -> ProbeOutcome {
        ProbeOutcome::default()
    }
}

/// The read-only filesystem view the credential probes need.
///
/// A trait because every one of these paths belongs to a product that may or
/// may not be installed, and a test must be able to describe the layout it is
/// asserting about.
pub trait CredentialFiles: Send + Sync + std::fmt::Debug {
    /// The file's contents, or `None` if it cannot be read.
    fn read(&self, path: &Path) -> Option<String>;
    /// Whether the path exists.
    fn exists(&self, path: &Path) -> bool;
    /// Every entry under `path`, recursively. Empty when it cannot be listed.
    fn list_recursive(&self, path: &Path) -> Vec<PathBuf>;
}

/// The real filesystem.
#[derive(Debug, Clone, Copy, Default)]
pub struct HostCredentialFiles;

impl CredentialFiles for HostCredentialFiles {
    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn list_recursive(&self, path: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut queue = vec![path.to_path_buf()];
        while let Some(directory) = queue.pop() {
            let Ok(entries) = std::fs::read_dir(&directory) else {
                continue;
            };
            for entry in entries.flatten() {
                let child = entry.path();
                if child.is_dir() {
                    queue.push(child.clone());
                }
                found.push(child);
            }
        }
        found
    }
}

/// Everything a probe needs beyond the backend id.
pub struct AuthInspection<'a> {
    /// The backend's own executable, or empty when it was not found. An empty
    /// command makes the `command` and `pi-auth-check` probes inconclusive
    /// rather than failing them.
    pub command: &'a str,
    /// The Gateway environment.
    pub env: &'a EnvMap,
    /// The platform whose credential layout applies.
    pub platform: HostPlatform,
    /// Running status commands.
    pub runner: &'a dyn ProbeRunner,
    /// Reading credential files.
    pub files: &'a dyn CredentialFiles,
}

/// `inspectBackendAuthentication(id, options)` —
/// `shared/backend-auth-status.mjs:226-280`.
///
/// Dispatches on the catalogued [`ProbeKind`]. A backend with no probe, an
/// unrecognised parser, or a `command` probe with no executable to run all
/// answer [`AuthStatus::Unknown`].
pub async fn inspect_backend_authentication(
    id: &str,
    inspection: &AuthInspection<'_>,
) -> AuthStatus {
    let probe = crate::onboarding::backend_onboarding_adapter(
        id,
        inspection.env,
        Some(inspection.platform),
        via_i18n::Locale::default(),
    )
    .configuration
    .probe;
    let Some(probe) = probe else {
        return AuthStatus::Unknown;
    };
    match probe.kind {
        ProbeKind::QwenSettings => qwen_status(inspection),
        ProbeKind::PiAuthCheck => pi_status(inspection).await,
        ProbeKind::DeepseekCredentials => deepseek_status(inspection),
        ProbeKind::CodebuddyCredentials => codebuddy_status(inspection),
        ProbeKind::OpenclawState => openclaw_status(inspection),
        ProbeKind::Command => command_status(&probe, inspection).await,
    }
}

/// The `command` probe: run the backend's own status command and parse it.
async fn command_status(probe: &AuthProbe, inspection: &AuthInspection<'_>) -> AuthStatus {
    if inspection.command.trim().is_empty() {
        return AuthStatus::Unknown;
    }
    let Some(parser) = probe.parser else {
        return AuthStatus::Unknown;
    };
    let outcome = inspection
        .runner
        .run(ProbeRequest {
            command: inspection.command.to_owned(),
            arguments: probe.args.iter().map(|arg| (*arg).to_owned()).collect(),
            environment: inspection.env.clone(),
            timeout: PROBE_TIMEOUT,
        })
        .await;
    parse_status(parser, &clean_output(&outcome.output))
}

/// The three `STATUS_PARSERS` — `shared/backend-auth-status.mjs:10-35`.
#[must_use]
pub fn parse_status(parser: CommandParser, output: &str) -> AuthStatus {
    match parser {
        CommandParser::CredentialCount => parse_credential_count(output),
        CommandParser::QoderStatus => parse_qoder_status(output),
        CommandParser::CodexStatus => parse_codex_status(output),
    }
}

/// The seven upstream patterns, as `Option` so this module carries no `unwrap`.
///
/// Every pattern is a literal, and `every_pattern_compiles` asserts all seven do.
/// A `None` would therefore be a build-time mistake, and it degrades to
/// *"matches nothing"* — which for each of these means [`AuthStatus::Unknown`],
/// never a false "logged out".
static CREDENTIAL_COUNT: Lazy<Option<Regex>> =
    Lazy::new(|| Regex::new(r"(?i)(\d+)\s+credentials?").ok());

static NOT_SIGNED_IN: Lazy<Option<Regex>> = Lazy::new(|| {
    Regex::new(r"(?i)not (?:logged in|authenticated)|please (?:log in|login|sign in)").ok()
});

static LOGGED_IN: Lazy<Option<Regex>> = Lazy::new(|| Regex::new(r"(?i)logged in").ok());

static NOT_LOGGED_IN: Lazy<Option<Regex>> =
    Lazy::new(|| Regex::new(r"(?i)not logged in|not authenticated").ok());

static ANSI: Lazy<Option<Regex>> = Lazy::new(|| Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").ok());

static PI_READY: Lazy<Option<Regex>> = Lazy::new(|| Regex::new(r"(?i)\bready\b").ok());

static PI_MISSING: Lazy<Option<Regex>> = Lazy::new(|| {
    Regex::new(r"(?i)missing|not (?:configured|authenticated)|no (?:credential|api key)").ok()
});

fn matches(pattern: &Lazy<Option<Regex>>, text: &str) -> bool {
    pattern
        .as_ref()
        .is_some_and(|pattern| pattern.is_match(text))
}

/// `'credential-count'` — `shared/backend-auth-status.mjs:11-17`.
///
/// No number at all is [`AuthStatus::Unknown`]: a CLI that changed its wording
/// has not told us the user is logged out.
fn parse_credential_count(output: &str) -> AuthStatus {
    let Some(capture) = CREDENTIAL_COUNT
        .as_ref()
        .and_then(|pattern| pattern.captures(output))
    else {
        return AuthStatus::Unknown;
    };
    let count = capture
        .get(1)
        .and_then(|value| value.as_str().parse::<u64>().ok());
    match count {
        Some(count) if count > 0 => AuthStatus::Authenticated,
        Some(_) => AuthStatus::Unauthenticated,
        None => AuthStatus::Unknown,
    }
}

/// `'qoder-status'` — `shared/backend-auth-status.mjs:18-26`.
///
/// The `Account:` line carries a negative lookahead upstream —
/// `/^Account:\s*(?!Not (?:logged in|authenticated)\b)\S+/im` — which Rust's
/// `regex` deliberately does not support. It is reproduced here line by line:
/// an `Account:` whose value *is* "Not logged in" proves nothing, and must not
/// be read as a username.
fn parse_qoder_status(output: &str) -> AuthStatus {
    for line in output.lines() {
        if let Some(rest) = strip_field(line, &["Username", "Email"])
            && !rest.is_empty()
        {
            return AuthStatus::Authenticated;
        }
        if let Some(rest) = strip_field(line, &["Account"]) {
            let lowered = rest.to_lowercase();
            let denied = ["not logged in", "not authenticated"].iter().any(|phrase| {
                lowered.strip_prefix(phrase).is_some_and(|tail| {
                    // `\b` after the phrase: the next character must not
                    // continue the word.
                    tail.chars()
                        .next()
                        .is_none_or(|next| !next.is_alphanumeric() && next != '_')
                })
            });
            if !denied && !rest.is_empty() {
                return AuthStatus::Authenticated;
            }
        }
    }
    if matches(&NOT_SIGNED_IN, output) {
        AuthStatus::Unauthenticated
    } else {
        AuthStatus::Unknown
    }
}

/// The value after `<field>:` on a line, with leading whitespace removed, or
/// `None` when the line is not that field. Case-insensitive, as the `i` flag is.
fn strip_field<'a>(line: &'a str, fields: &[&str]) -> Option<&'a str> {
    for field in fields {
        if line.len() > field.len()
            && line[..field.len()].eq_ignore_ascii_case(field)
            && line.as_bytes().get(field.len()) == Some(&b':')
        {
            return Some(line[field.len() + 1..].trim_start());
        }
    }
    None
}

/// `'codex-status'` — `shared/backend-auth-status.mjs:27-34`.
///
/// `logged in` **and not** `not logged in`, in that order: "not logged in"
/// contains "logged in", so testing only the positive would invert the answer.
fn parse_codex_status(output: &str) -> AuthStatus {
    if matches(&LOGGED_IN, output) && !matches(&NOT_LOGGED_IN, output) {
        return AuthStatus::Authenticated;
    }
    if matches(&NOT_LOGGED_IN, output) {
        AuthStatus::Unauthenticated
    } else {
        AuthStatus::Unknown
    }
}

/// `cleanOutput(value)` — `shared/backend-auth-status.mjs:110-112`.
#[must_use]
pub fn clean_output(value: &str) -> String {
    let bounded: String = value.chars().take(MAX_PROBE_OUTPUT).collect();
    match ANSI.as_ref() {
        Some(pattern) => pattern.replace_all(&bounded, "").trim().to_owned(),
        None => bounded.trim().to_owned(),
    }
}

/// `qwenAuthenticationStatus` — `shared/backend-auth-status.mjs:44-70`.
///
/// Qwen Code can also sign in through the OS keychain, whose credentials are
/// intentionally unreadable here. A configured auth type alone is therefore
/// inconclusive rather than proof that login is missing — the sentence in
/// upstream's own comment, and the reason this returns `Unknown` at the end.
fn qwen_status(inspection: &AuthInspection<'_>) -> AuthStatus {
    let env = inspection.env;
    if QWEN_CREDENTIAL_KEYS
        .iter()
        .any(|name| !env.get_trimmed(name).is_empty())
    {
        return AuthStatus::Authenticated;
    }
    let home = home_directory(env);
    if home.is_empty() {
        return AuthStatus::Unknown;
    }
    let base = non_empty(env.get_trimmed("QWEN_HOME"))
        .map_or_else(|| PathBuf::from(&home).join(".qwen"), PathBuf::from);
    let Some(settings) = read_json(inspection.files, &base.join("settings.json")) else {
        return AuthStatus::Unknown;
    };
    let configured = settings.get("env");
    let signed_in = QWEN_CREDENTIAL_KEYS.iter().any(|name| {
        configured
            .and_then(|env| env.get(*name))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
    });
    if signed_in {
        AuthStatus::Authenticated
    } else {
        AuthStatus::Unknown
    }
}

/// `piAuthenticationStatus` — `shared/backend-auth-status.mjs:72-108`.
async fn pi_status(inspection: &AuthInspection<'_>) -> AuthStatus {
    let env = inspection.env;
    let home = home_directory(env);
    if inspection.command.trim().is_empty() || home.is_empty() {
        return AuthStatus::Unknown;
    }
    let base = non_empty(env.get_trimmed("PI_CODING_AGENT_DIR")).map_or_else(
        || PathBuf::from(&home).join(".pi").join("agent"),
        PathBuf::from,
    );
    let settings = read_json(inspection.files, &base.join("settings.json"));
    let field = |name: &str| {
        settings
            .as_ref()
            .and_then(|value| value.get(name))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };
    let provider = field("defaultProvider");
    let model = field("defaultModel");
    if provider.is_empty() && model.is_empty() {
        // Pi with neither a provider nor a model selected has genuinely not
        // been set up; this is the one place a missing file *is* proof.
        return AuthStatus::Unauthenticated;
    }
    let selector = if provider.is_empty() {
        vec!["--model".to_owned(), model]
    } else {
        vec!["--provider".to_owned(), provider]
    };
    let mut arguments = vec!["auth".to_owned(), "check".to_owned()];
    arguments.extend(selector);
    arguments.push("--no-refresh".to_owned());
    arguments.push("--json".to_owned());
    let outcome = inspection
        .runner
        .run(ProbeRequest {
            command: inspection.command.to_owned(),
            arguments,
            environment: env.clone(),
            timeout: PROBE_TIMEOUT,
        })
        .await;

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(outcome.output.trim()) {
        match parsed.get("status").and_then(serde_json::Value::as_str) {
            Some("ready") => return AuthStatus::Authenticated,
            Some("missing" | "unavailable" | "unauthenticated") => {
                return AuthStatus::Unauthenticated;
            }
            // Older Pi builds may not support JSON output. Fall through to the
            // conservative text parser rather than treating the probe as
            // logged out.
            _ => {}
        }
    }
    if matches(&PI_READY, &outcome.output) {
        return AuthStatus::Authenticated;
    }
    if matches(&PI_MISSING, &outcome.output) {
        return AuthStatus::Unauthenticated;
    }
    AuthStatus::Unknown
}

/// `deepSeekCredentialStatus` — `shared/backend-auth-status.mjs:163-181`.
fn deepseek_status(inspection: &AuthInspection<'_>) -> AuthStatus {
    let env = inspection.env;
    if !env.get_trimmed("DEEPSEEK_API_KEY").is_empty() {
        return AuthStatus::Authenticated;
    }
    let dsh_home = env.get_trimmed("DSH_HOME");
    let home = if dsh_home.is_empty() {
        home_directory(env)
    } else {
        dsh_home.to_owned()
    };
    if home.is_empty() {
        return AuthStatus::Unauthenticated;
    }
    let path = if dsh_home.is_empty() {
        PathBuf::from(&home).join(".dsh").join(".credentials.yaml")
    } else {
        PathBuf::from(&home).join(".credentials.yaml")
    };
    match inspection.files.read(&path) {
        Some(content) if has_deepseek_key(&content) => AuthStatus::Authenticated,
        _ => AuthStatus::Unauthenticated,
    }
}

/// `/^\s*DEEPSEEK_API_KEY\s*:\s*(?!(?:["']{2})\s*$)\S+/m`.
///
/// The negative lookahead means *"the value must not be an empty quoted
/// string"*: `DEEPSEEK_API_KEY: ""` is a placeholder `dsh web` writes before
/// the user pastes a key, and reading it as a credential would report a
/// logged-out user as signed in.
fn has_deepseek_key(content: &str) -> bool {
    content.lines().any(|line| {
        let Some(rest) = line.trim_start().strip_prefix("DEEPSEEK_API_KEY") else {
            return false;
        };
        let Some(rest) = rest.trim_start().strip_prefix(':') else {
            return false;
        };
        let value = rest.trim_start();
        let Some(first) = value.chars().next() else {
            return false;
        };
        if first.is_whitespace() {
            return false;
        }
        for quote in ['"', '\''] {
            let empty = std::format!("{quote}{quote}");
            if let Some(tail) = value.strip_prefix(&empty)
                && tail.trim().is_empty()
            {
                return false;
            }
        }
        true
    })
}

/// `codeBuddyCredentialFiles` + its verdict —
/// `shared/backend-auth-status.mjs:114-161,266-272`.
///
/// CodeBuddy has no read-only login-status command. An empty credential
/// directory proves the user is signed out, but the presence of files may only
/// be an expired token or leftovers from an uninstall, so it can never prove
/// the opposite.
fn codebuddy_status(inspection: &AuthInspection<'_>) -> AuthStatus {
    let files = codebuddy_credential_directory(inspection.env, inspection.platform)
        .map(|directory| inspection.files.list_recursive(&directory))
        .unwrap_or_default();
    if files.is_empty() {
        AuthStatus::Unauthenticated
    } else {
        AuthStatus::Unknown
    }
}

/// Where CodeBuddy keeps its credentials on each platform.
///
/// **External contract** — `file-path/third-party credential probe paths`.
#[must_use]
pub fn codebuddy_credential_directory(env: &EnvMap, platform: HostPlatform) -> Option<PathBuf> {
    let home = home_directory(env);
    if home.is_empty()
        && env.get_trimmed("LOCALAPPDATA").is_empty()
        && env.get_trimmed("XDG_DATA_HOME").is_empty()
    {
        return None;
    }
    let base = match platform {
        HostPlatform::Windows => non_empty(env.get_trimmed("LOCALAPPDATA")).map_or_else(
            || PathBuf::from(&home).join("AppData").join("Local"),
            PathBuf::from,
        ),
        HostPlatform::Darwin => PathBuf::from(&home)
            .join("Library")
            .join("Application Support"),
        HostPlatform::Linux => non_empty(env.get_trimmed("XDG_DATA_HOME")).map_or_else(
            || PathBuf::from(&home).join(".local").join("share"),
            PathBuf::from,
        ),
    };
    Some(
        base.join("CodeBuddyExtension")
            .join("Data")
            .join("Public")
            .join("auth"),
    )
}

/// `openClawInitializationStatus` — `shared/backend-auth-status.mjs:143-161`.
///
/// `OPENCLAW_STATE_DIR` and `OPENCLAW_CONFIG_PATH` here are OpenClaw's **own**
/// variables (KEEP), not VIA's `VIA_OPENCLAW_STATE_DIR`: this probe inspects an
/// existing user installation of a third-party product.
fn openclaw_status(inspection: &AuthInspection<'_>) -> AuthStatus {
    let env = inspection.env;
    let home = home_directory(env);
    let state_directory = non_empty(env.get_trimmed("OPENCLAW_STATE_DIR")).or_else(|| {
        if home.is_empty() {
            None
        } else {
            Some(
                PathBuf::from(&home)
                    .join(".openclaw")
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    });
    let config_path = non_empty(env.get_trimmed("OPENCLAW_CONFIG_PATH")).or_else(|| {
        state_directory.as_ref().map(|directory| {
            PathBuf::from(directory)
                .join("openclaw.json")
                .to_string_lossy()
                .into_owned()
        })
    });
    let model_path = state_directory.as_ref().map(|directory| {
        PathBuf::from(directory)
            .join("agents")
            .join("main")
            .join("agent")
            .join("models.json")
    });
    let (Some(config_path), Some(model_path)) = (config_path, model_path) else {
        return AuthStatus::Unknown;
    };
    let configured = inspection.files.exists(Path::new(&config_path));
    let has_model = inspection.files.exists(&model_path);
    match (configured, has_model) {
        (true, true) => AuthStatus::Authenticated,
        (false, false) => AuthStatus::Unauthenticated,
        // A config with no model, or a model with no config, is a half-finished
        // onboarding rather than either answer.
        _ => AuthStatus::Unknown,
    }
}

fn read_json(files: &dyn CredentialFiles, path: &Path) -> Option<serde_json::Value> {
    files
        .read(path)
        .and_then(|text| serde_json::from_str(&text).ok())
}

fn home_directory(env: &EnvMap) -> String {
    ["HOME", "USERPROFILE"]
        .iter()
        .map(|name| env.get_trimmed(name))
        .find(|value| !value.is_empty())
        .unwrap_or_default()
        .to_owned()
}

fn non_empty(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_count_stays_unknown_when_the_wording_changes() {
        assert_eq!(
            parse_credential_count("2 credentials stored"),
            AuthStatus::Authenticated
        );
        assert_eq!(
            parse_credential_count("1 credential stored"),
            AuthStatus::Authenticated
        );
        assert_eq!(
            parse_credential_count("0 credentials"),
            AuthStatus::Unauthenticated
        );
        assert_eq!(parse_credential_count("no keys here"), AuthStatus::Unknown);
        assert_eq!(parse_credential_count(""), AuthStatus::Unknown);
    }

    #[test]
    fn qoder_reads_an_account_line_but_not_a_not_logged_in_one() {
        assert_eq!(
            parse_qoder_status("Username: ada"),
            AuthStatus::Authenticated
        );
        assert_eq!(
            parse_qoder_status("Email: ada@example.test"),
            AuthStatus::Authenticated
        );
        assert_eq!(
            parse_qoder_status("Account: ada"),
            AuthStatus::Authenticated
        );
        assert_eq!(
            parse_qoder_status("Account: Not logged in"),
            AuthStatus::Unauthenticated,
            "the negative lookahead is the whole point of this parser"
        );
        assert_eq!(
            parse_qoder_status("Account: Not authenticated"),
            AuthStatus::Unauthenticated
        );
        assert_eq!(
            parse_qoder_status("Please sign in first"),
            AuthStatus::Unauthenticated
        );
        assert_eq!(parse_qoder_status("Qoder 1.2.3"), AuthStatus::Unknown);
    }

    #[test]
    fn a_not_logged_in_account_value_that_only_starts_with_the_phrase_is_a_name() {
        // `\b` after the phrase: `Notational` is a username, not a refusal.
        assert_eq!(
            parse_qoder_status("Account: Not logged inbox"),
            AuthStatus::Authenticated
        );
    }

    #[test]
    fn codex_does_not_read_not_logged_in_as_logged_in() {
        assert_eq!(
            parse_codex_status("Logged in as ada"),
            AuthStatus::Authenticated
        );
        assert_eq!(
            parse_codex_status("Not logged in"),
            AuthStatus::Unauthenticated
        );
        assert_eq!(
            parse_codex_status("not authenticated"),
            AuthStatus::Unauthenticated
        );
        assert_eq!(parse_codex_status("codex 0.146.0"), AuthStatus::Unknown);
    }

    #[test]
    fn ansi_escapes_are_stripped_before_parsing() {
        let coloured = "\u{1b}[32mLogged in\u{1b}[0m as ada\n";
        assert_eq!(clean_output(coloured), "Logged in as ada");
        assert_eq!(
            parse_status(CommandParser::CodexStatus, &clean_output(coloured)),
            AuthStatus::Authenticated
        );
    }

    #[test]
    fn an_empty_deepseek_placeholder_is_not_a_credential() {
        assert!(has_deepseek_key("DEEPSEEK_API_KEY: sk-real"));
        assert!(has_deepseek_key("  DEEPSEEK_API_KEY : sk-real"));
        assert!(!has_deepseek_key("DEEPSEEK_API_KEY: \"\""));
        assert!(!has_deepseek_key("DEEPSEEK_API_KEY: ''"));
        assert!(!has_deepseek_key("DEEPSEEK_API_KEY:"));
        assert!(!has_deepseek_key("OTHER_KEY: sk-real"));
        // A quoted value that is not empty still counts.
        assert!(has_deepseek_key("DEEPSEEK_API_KEY: \"sk-real\""));
    }

    #[test]
    fn every_pattern_compiles() {
        for pattern in [
            &CREDENTIAL_COUNT,
            &NOT_SIGNED_IN,
            &LOGGED_IN,
            &NOT_LOGGED_IN,
            &ANSI,
            &PI_READY,
            &PI_MISSING,
        ] {
            assert!(
                pattern.as_ref().is_some(),
                "every upstream pattern compiles"
            );
        }
    }

    #[test]
    fn every_status_has_its_catalogued_spelling() {
        assert_eq!(AuthStatus::Authenticated.as_str(), "authenticated");
        assert_eq!(AuthStatus::Unauthenticated.as_str(), "unauthenticated");
        assert_eq!(AuthStatus::Unknown.as_str(), "unknown");
    }
}
