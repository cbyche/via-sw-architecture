//! Every authentication probe, and every way one can be inconclusive.
//!
//! Ported from `shared/backend-auth-status.mjs`. The rule under test is one
//! sentence — *"when it cannot tell, it must stay `unknown`"* — and the tests
//! below are mostly about the cases where a careless port would answer
//! `unauthenticated` instead and nag a user who is in fact signed in.

mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use support::env;
use via_backends::auth::{
    CredentialFiles, ProbeOutcome, ProbeRequest, ProbeRunner, codebuddy_credential_directory,
};
use via_backends::{AuthInspection, AuthStatus, HostPlatform, inspect_backend_authentication};
use via_core::EnvMap;

/// A filesystem described by a test rather than found on disk.
#[derive(Debug, Default)]
struct FakeFiles {
    files: BTreeMap<PathBuf, String>,
    directories: BTreeMap<PathBuf, Vec<PathBuf>>,
}

impl FakeFiles {
    fn with_file(mut self, path: &str, contents: &str) -> Self {
        self.files.insert(PathBuf::from(path), contents.to_owned());
        self
    }

    fn with_directory(mut self, path: &str, entries: &[&str]) -> Self {
        self.directories.insert(
            PathBuf::from(path),
            entries.iter().map(PathBuf::from).collect(),
        );
        self
    }
}

impl CredentialFiles for FakeFiles {
    fn read(&self, path: &Path) -> Option<String> {
        self.files.get(path).cloned()
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path) || self.directories.contains_key(path)
    }

    fn list_recursive(&self, path: &Path) -> Vec<PathBuf> {
        self.directories.get(path).cloned().unwrap_or_default()
    }
}

/// A status command scripted by a test.
struct ScriptedRunner {
    output: String,
    expected: Option<Vec<String>>,
}

impl ScriptedRunner {
    fn saying(output: &str) -> Self {
        Self {
            output: output.to_owned(),
            expected: None,
        }
    }

    fn expecting(output: &str, arguments: &[&str]) -> Self {
        Self {
            output: output.to_owned(),
            expected: Some(arguments.iter().map(|value| (*value).to_owned()).collect()),
        }
    }
}

#[async_trait]
impl ProbeRunner for ScriptedRunner {
    async fn run(&self, request: ProbeRequest) -> ProbeOutcome {
        if let Some(expected) = &self.expected {
            assert_eq!(&request.arguments, expected);
        }
        ProbeOutcome {
            ok: true,
            output: self.output.clone(),
        }
    }
}

async fn status(
    id: &str,
    command: &str,
    environment: &EnvMap,
    runner: &dyn ProbeRunner,
    files: &dyn CredentialFiles,
) -> AuthStatus {
    inspect_backend_authentication(
        id,
        &AuthInspection {
            command,
            env: environment,
            platform: HostPlatform::Linux,
            runner,
            files,
        },
    )
    .await
}

#[tokio::test]
async fn a_command_probe_with_no_executable_stays_unknown() {
    // Nothing was found to run, so nothing was learned. Upstream:
    // `if (probe?.kind !== 'command' || !command) return { status: 'unknown' }`.
    assert_eq!(
        status(
            "codex",
            "",
            &EnvMap::new(),
            &ScriptedRunner::saying("Not logged in"),
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Unknown
    );
}

#[tokio::test]
async fn a_backend_with_no_probe_stays_unknown() {
    // Kimi, Hermes and Claude Code declare an onboarding command but no probe.
    for id in ["kimi", "hermes", "claude"] {
        assert_eq!(
            status(
                id,
                "/usr/bin/thing",
                &EnvMap::new(),
                &ScriptedRunner::saying("Logged in"),
                &FakeFiles::default()
            )
            .await,
            AuthStatus::Unknown,
            "{id}"
        );
    }
}

#[tokio::test]
async fn the_generic_acp_backend_has_nothing_to_probe() {
    assert_eq!(
        status(
            "acp",
            "/opt/agent",
            &EnvMap::new(),
            &ScriptedRunner::saying("Logged in"),
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Unknown
    );
}

#[tokio::test]
async fn opencode_counts_its_credentials() {
    let runner = ScriptedRunner::expecting("2 credentials stored", &["auth", "list"]);
    assert_eq!(
        status(
            "opencode",
            "/usr/bin/opencode",
            &EnvMap::new(),
            &runner,
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Authenticated
    );
    assert_eq!(
        status(
            "opencode",
            "/usr/bin/opencode",
            &EnvMap::new(),
            &ScriptedRunner::saying("0 credentials"),
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Unauthenticated
    );
    assert_eq!(
        status(
            "opencode",
            "/usr/bin/opencode",
            &EnvMap::new(),
            &ScriptedRunner::saying("opencode 1.18.5"),
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Unknown,
        "a CLI that changed its wording has not said the user is signed out"
    );
}

#[tokio::test]
async fn qoder_runs_its_status_command() {
    let runner = ScriptedRunner::expecting("Account: ada", &["status"]);
    assert_eq!(
        status(
            "qoder",
            "/usr/bin/qodercli",
            &EnvMap::new(),
            &runner,
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Authenticated
    );
}

#[tokio::test]
async fn codex_runs_login_status() {
    let runner = ScriptedRunner::expecting("Logged in as ada", &["login", "status"]);
    assert_eq!(
        status(
            "codex",
            "/usr/bin/codex",
            &EnvMap::new(),
            &runner,
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Authenticated
    );
}

#[tokio::test]
async fn qwen_reads_its_own_settings_and_stays_unknown_when_a_keychain_may_hold_the_key() {
    let runner = ScriptedRunner::saying("");
    let environment = env(&[("HOME", "/home/u")]);

    // An environment credential is proof.
    assert_eq!(
        status(
            "qwen",
            "",
            &env(&[("HOME", "/home/u"), ("QWEN_API_KEY", "sk-qwen")]),
            &runner,
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Authenticated
    );

    // A settings file with a key is proof.
    let files = FakeFiles::default().with_file(
        "/home/u/.qwen/settings.json",
        r#"{"env":{"DASHSCOPE_API_KEY":"sk-dash"}}"#,
    );
    assert_eq!(
        status("qwen", "", &environment, &runner, &files).await,
        AuthStatus::Authenticated
    );

    // A settings file *without* one is not proof of the opposite: Qwen Code can
    // also sign in through the OS keychain, which is not readable here.
    let files = FakeFiles::default().with_file(
        "/home/u/.qwen/settings.json",
        r#"{"selectedAuthType":"oauth"}"#,
    );
    assert_eq!(
        status("qwen", "", &environment, &runner, &files).await,
        AuthStatus::Unknown
    );

    // No settings file at all, and no home directory, are both unknown.
    assert_eq!(
        status("qwen", "", &environment, &runner, &FakeFiles::default()).await,
        AuthStatus::Unknown
    );
    assert_eq!(
        status("qwen", "", &EnvMap::new(), &runner, &FakeFiles::default()).await,
        AuthStatus::Unknown
    );
}

#[tokio::test]
async fn qwen_honours_its_own_home_override() {
    let files = FakeFiles::default().with_file(
        "/custom/settings.json",
        r#"{"env":{"QWEN_OAUTH_TOKEN":"token"}}"#,
    );
    let environment = env(&[("HOME", "/home/u"), ("QWEN_HOME", "/custom")]);
    assert_eq!(
        status(
            "qwen",
            "",
            &environment,
            &ScriptedRunner::saying(""),
            &files
        )
        .await,
        AuthStatus::Authenticated
    );
}

#[tokio::test]
async fn pi_asks_its_own_auth_check_for_the_selected_provider() {
    let files = FakeFiles::default().with_file(
        "/home/u/.pi/agent/settings.json",
        r#"{"defaultProvider":"anthropic"}"#,
    );
    let runner = ScriptedRunner::expecting(
        r#"{"status":"ready"}"#,
        &[
            "auth",
            "check",
            "--provider",
            "anthropic",
            "--no-refresh",
            "--json",
        ],
    );
    assert_eq!(
        status(
            "pi",
            "/usr/bin/pi",
            &env(&[("HOME", "/home/u")]),
            &runner,
            &files
        )
        .await,
        AuthStatus::Authenticated
    );
}

#[tokio::test]
async fn pi_falls_back_to_the_model_selector_and_to_the_text_parser() {
    let files = FakeFiles::default().with_file(
        "/home/u/.pi/agent/settings.json",
        r#"{"defaultModel":"claude-sonnet"}"#,
    );
    let runner = ScriptedRunner::expecting(
        "provider ready",
        &[
            "auth",
            "check",
            "--model",
            "claude-sonnet",
            "--no-refresh",
            "--json",
        ],
    );
    assert_eq!(
        status(
            "pi",
            "/usr/bin/pi",
            &env(&[("HOME", "/home/u")]),
            &runner,
            &files
        )
        .await,
        AuthStatus::Authenticated,
        "an older Pi that cannot emit JSON falls through to the text parser"
    );
}

#[tokio::test]
async fn pi_with_nothing_selected_is_the_one_case_a_missing_file_proves() {
    assert_eq!(
        status(
            "pi",
            "/usr/bin/pi",
            &env(&[("HOME", "/home/u")]),
            &ScriptedRunner::saying(""),
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Unauthenticated
    );
    // …but with no `pi` to run, nothing is known at all.
    assert_eq!(
        status(
            "pi",
            "",
            &env(&[("HOME", "/home/u")]),
            &ScriptedRunner::saying(""),
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Unknown
    );
}

#[tokio::test]
async fn pi_reports_an_unrecognised_answer_as_unknown() {
    let files = FakeFiles::default().with_file(
        "/home/u/.pi/agent/settings.json",
        r#"{"defaultProvider":"anthropic"}"#,
    );
    assert_eq!(
        status(
            "pi",
            "/usr/bin/pi",
            &env(&[("HOME", "/home/u")]),
            &ScriptedRunner::saying(r#"{"status":"weird"}"#),
            &files
        )
        .await,
        AuthStatus::Unknown
    );
}

#[tokio::test]
async fn deepseek_reads_its_credential_file() {
    let runner = ScriptedRunner::saying("");
    assert_eq!(
        status(
            "deepseek",
            "",
            &env(&[("DEEPSEEK_API_KEY", "sk-deep")]),
            &runner,
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Authenticated
    );

    let files =
        FakeFiles::default().with_file("/home/u/.dsh/.credentials.yaml", "DEEPSEEK_API_KEY: sk-x");
    assert_eq!(
        status(
            "deepseek",
            "",
            &env(&[("HOME", "/home/u")]),
            &runner,
            &files
        )
        .await,
        AuthStatus::Authenticated
    );

    // `dsh web` writes an empty placeholder before the user pastes a key.
    let placeholder =
        FakeFiles::default().with_file("/home/u/.dsh/.credentials.yaml", "DEEPSEEK_API_KEY: \"\"");
    assert_eq!(
        status(
            "deepseek",
            "",
            &env(&[("HOME", "/home/u")]),
            &runner,
            &placeholder
        )
        .await,
        AuthStatus::Unauthenticated
    );

    // `DSH_HOME` moves the file up one level.
    let moved =
        FakeFiles::default().with_file("/state/.credentials.yaml", "DEEPSEEK_API_KEY: sk-x");
    assert_eq!(
        status(
            "deepseek",
            "",
            &env(&[("HOME", "/home/u"), ("DSH_HOME", "/state")]),
            &runner,
            &moved
        )
        .await,
        AuthStatus::Authenticated
    );
}

#[tokio::test]
async fn codebuddy_can_prove_signed_out_but_never_signed_in() {
    let runner = ScriptedRunner::saying("");
    let environment = env(&[("HOME", "/home/u")]);
    let directory = "/home/u/.local/share/CodeBuddyExtension/Data/Public/auth";

    assert_eq!(
        status(
            "codebuddy",
            "/usr/bin/codebuddy",
            &environment,
            &runner,
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Unauthenticated,
        "an empty credential directory is proof"
    );

    let files = FakeFiles::default().with_directory(directory, &["token.json"]);
    assert_eq!(
        status(
            "codebuddy",
            "/usr/bin/codebuddy",
            &environment,
            &runner,
            &files
        )
        .await,
        AuthStatus::Unknown,
        "files may be expired or left over from an uninstall"
    );
}

#[tokio::test]
async fn openclaw_needs_both_a_config_and_a_model_to_be_initialised() {
    let runner = ScriptedRunner::saying("");
    let environment = env(&[("HOME", "/home/u")]);
    let config = "/home/u/.openclaw/openclaw.json";
    let models = "/home/u/.openclaw/agents/main/agent/models.json";

    assert_eq!(
        status(
            "openclaw",
            "",
            &environment,
            &runner,
            &FakeFiles::default()
                .with_file(config, "{}")
                .with_file(models, "{}")
        )
        .await,
        AuthStatus::Authenticated
    );
    assert_eq!(
        status("openclaw", "", &environment, &runner, &FakeFiles::default()).await,
        AuthStatus::Unauthenticated
    );
    assert_eq!(
        status(
            "openclaw",
            "",
            &environment,
            &runner,
            &FakeFiles::default().with_file(config, "{}")
        )
        .await,
        AuthStatus::Unknown,
        "a half-finished onboarding is neither answer"
    );
    // With no home and no state directory there is no path to look at.
    assert_eq!(
        status(
            "openclaw",
            "",
            &EnvMap::new(),
            &runner,
            &FakeFiles::default()
        )
        .await,
        AuthStatus::Unknown
    );
}

#[tokio::test]
async fn every_backend_answers_one_of_the_three_statuses() {
    let runner = ScriptedRunner::saying("");
    let files = FakeFiles::default();
    for id in via_catalog::backend_names() {
        let answer = status(id, "/usr/bin/thing", &EnvMap::new(), &runner, &files).await;
        assert!(
            matches!(
                answer,
                AuthStatus::Authenticated | AuthStatus::Unauthenticated | AuthStatus::Unknown
            ),
            "{id}"
        );
    }
}

/// `file-path/third-party credential probe paths`: five products' own
/// on-disk layouts, read-only. Each backend's probe is exercised at exactly
/// the catalogued path — a wrong path would leave `FakeFiles` empty and every
/// one of these would answer `Unknown` instead of proving the real fixture.
/// The individual behaviours (the DeepSeek placeholder, OpenClaw's
/// half-finished-onboarding case, ...) already have their own tests above;
/// this one exists to hold the five path conventions side by side as the one
/// contract that names all five together.
#[tokio::test]
async fn every_third_party_credential_probe_path_matches_the_catalogue() {
    // qwen: `${QWEN_HOME|~/.qwen}/settings.json`.
    let files = FakeFiles::default().with_file(
        "/home/u/.qwen/settings.json",
        r#"{"env":{"DASHSCOPE_API_KEY":"sk-dash"}}"#,
    );
    assert_eq!(
        status(
            "qwen",
            "",
            &env(&[("HOME", "/home/u")]),
            &ScriptedRunner::saying(""),
            &files
        )
        .await,
        AuthStatus::Authenticated,
        "qwen"
    );

    // pi: `${PI_CODING_AGENT_DIR|~/.pi/agent}/settings.json`, then
    // `pi auth check --provider|--model X --no-refresh --json`.
    let files = FakeFiles::default().with_file(
        "/home/u/.pi/agent/settings.json",
        r#"{"defaultProvider":"anthropic"}"#,
    );
    let runner = ScriptedRunner::expecting(
        r#"{"status":"ready"}"#,
        &[
            "auth",
            "check",
            "--provider",
            "anthropic",
            "--no-refresh",
            "--json",
        ],
    );
    assert_eq!(
        status(
            "pi",
            "/usr/bin/pi",
            &env(&[("HOME", "/home/u")]),
            &runner,
            &files
        )
        .await,
        AuthStatus::Authenticated,
        "pi"
    );

    // deepseek: `${DSH_HOME}/.credentials.yaml` or `~/.dsh/.credentials.yaml`.
    let files = FakeFiles::default().with_file(
        "/home/u/.dsh/.credentials.yaml",
        "DEEPSEEK_API_KEY: sk-real",
    );
    assert_eq!(
        status(
            "deepseek",
            "",
            &env(&[("HOME", "/home/u")]),
            &ScriptedRunner::saying(""),
            &files
        )
        .await,
        AuthStatus::Authenticated,
        "deepseek"
    );

    // codebuddy: win `%LOCALAPPDATA%/...`, darwin
    // `~/Library/Application Support/...`, else
    // `${XDG_DATA_HOME|~/.local/share}/...`.
    assert_eq!(
        codebuddy_credential_directory(&env(&[("HOME", "/home/u")]), HostPlatform::Darwin),
        Some(PathBuf::from(
            "/home/u/Library/Application Support/CodeBuddyExtension/Data/Public/auth"
        )),
        "codebuddy darwin"
    );
    assert_eq!(
        codebuddy_credential_directory(&env(&[("HOME", "/home/u")]), HostPlatform::Linux),
        Some(PathBuf::from(
            "/home/u/.local/share/CodeBuddyExtension/Data/Public/auth"
        )),
        "codebuddy linux default"
    );
    assert_eq!(
        codebuddy_credential_directory(
            &env(&[("HOME", "/home/u"), ("XDG_DATA_HOME", "/data")]),
            HostPlatform::Linux
        ),
        Some(PathBuf::from("/data/CodeBuddyExtension/Data/Public/auth")),
        "codebuddy linux XDG_DATA_HOME override"
    );
    assert_eq!(
        codebuddy_credential_directory(
            &env(&[("HOME", "/home/u"), ("LOCALAPPDATA", "/appdata/local")]),
            HostPlatform::Windows
        ),
        Some(PathBuf::from(
            "/appdata/local/CodeBuddyExtension/Data/Public/auth"
        )),
        "codebuddy windows LOCALAPPDATA"
    );

    // openclaw: `${OPENCLAW_CONFIG_PATH|${OPENCLAW_STATE_DIR|~/.openclaw}/openclaw.json}`
    // and `<stateDir>/agents/main/agent/models.json`.
    let files = FakeFiles::default()
        .with_file("/home/u/.openclaw/openclaw.json", "{}")
        .with_file("/home/u/.openclaw/agents/main/agent/models.json", "{}");
    assert_eq!(
        status(
            "openclaw",
            "",
            &env(&[("HOME", "/home/u")]),
            &ScriptedRunner::saying(""),
            &files
        )
        .await,
        AuthStatus::Authenticated,
        "openclaw"
    );
}
