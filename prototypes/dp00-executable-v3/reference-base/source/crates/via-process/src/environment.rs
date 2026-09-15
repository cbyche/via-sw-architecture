//! The child-process environment: a trust boundary, and the `PATH` a child
//! needs to find its own runtime.
//!
//! Ported from `shared/backend-environment.mjs:1-103` (the projection),
//! `shared/backend-install.mjs:206-231` (the `PATH` composition) and
//! `shared/path-environment.mjs` (the merge itself, which already lives in
//! [`via_core::search_path`] and is reused here rather than restated).
//!
//! # Why the allow-list is inverted
//!
//! Rust's `Command` inherits the parent environment by default, so the
//! *absence* of a call is a leak. Everything here is built the other way
//! round: [`backend_environment`] starts from nothing and copies in only what
//! a policy names, and [`crate::spawn::ProcessSpawner`] calls `env_clear()`
//! before applying it. `docs/architecture.md` §17 item 9 asks the reviewer
//! exactly this question — *"Did a child process spawn without
//! `.env_clear()`?"*
//!
//! What is deliberately **not** inherited: the Gateway auth secret, the
//! realtime API key, the memory-extractor credential and the
//! speech-to-speech token. `docs/reference/contracts.json`
//! (`env-var/backend and security environment`) records that upstream's own
//! test asserts `QWEN_AUDIO_AGENT_AUTH_SECRET` never appears in a spawned
//! backend's environment. Widening [`SYSTEM_NAMES`] or [`INTERNAL_NAMES`] is a
//! security regression, not a convenience.

use via_catalog::EnvironmentPolicy;
use via_core::EnvMap;
use via_core::search_path::{Platform, command_directory, merge_search_path};

/// Operating-system context every child agent receives.
///
/// **External contract** — `shared/backend-environment.mjs:7-45`, in upstream's
/// order. `docs/reference/contracts.json` carries the same list twice
/// (`env-var/child-process environment filter` and `env-var/backend child
/// environment allowlist`); the prose in the second says "47 OS names" while
/// the list it then gives holds 39, and the list is the contract.
pub const SYSTEM_NAMES: &[&str] = &[
    "APPDATA",
    "COLORTERM",
    "COMSPEC",
    "HOME",
    "HOMEDRIVE",
    "HOMEPATH",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "LANG",
    "LOCALAPPDATA",
    "LOGNAME",
    "NODE_EXTRA_CA_CERTS",
    "NO_BROWSER",
    "NO_PROXY",
    "PATH",
    "PATHEXT",
    "PWD",
    "SHELL",
    "SSL_CERT_DIR",
    "SSL_CERT_FILE",
    "SYSTEMDRIVE",
    "SYSTEMROOT",
    "TEMP",
    "TERM",
    "TMP",
    "TMPDIR",
    "USER",
    "USERDOMAIN",
    "USERNAME",
    "USERPROFILE",
    "WINDIR",
    "XDG_CACHE_HOME",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "XDG_RUNTIME_DIR",
    "XDG_STATE_HOME",
    "http_proxy",
    "https_proxy",
    "no_proxy",
];

/// VIA's own variables a child agent is allowed to see.
///
/// **External contract** — `shared/backend-environment.mjs:47-59`, with the
/// identity renames `docs/rebrand.md` line 107-108 mandates
/// (`QWEN_AUDIO_AGENT_* → VIA_*`). These carry no credential: they tell the
/// child which agent and model were selected, and where the installation is.
pub const INTERNAL_NAMES: &[&str] = &[
    "VIA_BACKEND_AGENT",
    "VIA_BACKEND_MODEL",
    "VIA_BACKEND_OWNERSHIP",
    "VIA_BACKEND_PERMISSION_MODE",
    "VIA_DESKTOP",
    "VIA_DESKTOP_INSTALLED_ONLY",
    "VIA_ENV_LOADED",
    "VIA_NODE",
    "VIA_ROOT",
    "VIA_RUNTIME_ROOT",
    "VIA_SOURCE_ROOT",
];

/// Prefixes whose every variable crosses the boundary.
///
/// **External contract** — `shared/backend-environment.mjs:61`. `LC_` is
/// locale, the two `npm_config`/`NPM_CONFIG` spellings are the package
/// manager's own configuration channel — a backend launched through `npx`
/// reads them.
pub const SYSTEM_PREFIXES: &[&str] = &["LC_", "npm_config_", "NPM_CONFIG_"];

/// Stamped into every child environment.
///
/// **External contract** — `shared/backend-environment.mjs:99`
/// (`QWEN_AUDIO_AGENT_ENV_LOADED`), renamed per `docs/rebrand.md`. It tells
/// the child that VIA has already layered `config.env`, so the child must not
/// layer it again.
pub const ENV_LOADED_NAME: &str = "VIA_ENV_LOADED";

/// The value [`ENV_LOADED_NAME`] is stamped with.
///
/// **External contract** — `shared/backend-environment.mjs:99`.
pub const ENV_LOADED_VALUE: &str = "1";

/// File extensions that mark a `PATH` entry as an executable rather than a
/// directory.
///
/// **External contract** — `shared/backend-install.mjs:219-226`. A
/// `PATH` holding `C:\tools\nodejs\npm.cmd` is a real thing users end up with;
/// the entry is dropped and the command's own directory takes its place.
/// Compared case-insensitively, as upstream lowercases before testing.
pub const EXECUTABLE_PATH_SUFFIXES: &[&str] = &[".cmd", ".exe", ".bat"];

/// Whether a policy, plus the shared allow-lists, forwards `name`.
///
/// **External contract** — `shared/backend-environment.mjs:74-81`. The order
/// of the tests is upstream's; all of them are case-sensitive, including the
/// prefixes, which is why `http_proxy` and `HTTP_PROXY` are both listed by
/// name.
#[must_use]
pub fn is_forwarded(name: &str, policy: &EnvironmentPolicy, explicit: &[String]) -> bool {
    SYSTEM_NAMES.contains(&name)
        || INTERNAL_NAMES.contains(&name)
        || SYSTEM_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
        || policy.allows(name)
        || explicit.iter().any(|allowed| allowed == name)
}

/// The names a user opted into through the policy's explicit-list variable.
///
/// **External contract** — `shared/backend-environment.mjs:67-72`. Only the
/// generic ACP backend declares one, because it has no vendor namespace of its
/// own to allow by prefix. Comma-separated, each entry trimmed, empties
/// dropped.
#[must_use]
pub fn explicit_names(policy: &EnvironmentPolicy, env: &EnvMap) -> Vec<String> {
    let Some(name) = policy.explicit_list_environment else {
        return Vec::new();
    };
    env.get_trimmed(name)
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Project the Gateway's environment down to what one backend may see.
///
/// **External contract** — `shared/backend-environment.mjs:83-103`.
/// `additions` are applied last and therefore win, which is how upstream's
/// `spawnSpec` stamps its extra entries.
///
/// # Deviation
///
/// Upstream also stamps `QWEN_AUDIO_AGENT_NODE = process.execPath` so a
/// `scripts/*.mjs` shim can re-exec the same Node. VIA spawns a real command,
/// not a Node script (`docs/architecture.md` §10), so there is no interpreter
/// path to publish and nothing is stamped. `VIA_NODE` stays on
/// [`INTERNAL_NAMES`], so an embedder that has a reason to set it still gets
/// it through; recorded in `docs/deviations/phase-2.md`.
#[must_use]
pub fn backend_environment(
    policy: &EnvironmentPolicy,
    env: &EnvMap,
    additions: &[(String, String)],
) -> EnvMap {
    let explicit = explicit_names(policy, env);
    let mut projected = EnvMap::new();
    for (name, value) in env.iter() {
        if is_forwarded(name, policy, &explicit) {
            projected.set(name, value);
        }
    }
    projected.set(ENV_LOADED_NAME, ENV_LOADED_VALUE);
    for (name, value) in additions {
        projected.set(name.as_str(), value.as_str());
    }
    projected
}

/// Compose the `PATH` a spawned child searches.
///
/// **External contract** — `shared/backend-install.mjs:206-231` (`npmRunEnv`),
/// which exists because a backend launched through `npx` runs hooks that
/// spawn `node`, and `node` lives beside `npx` under whatever version manager
/// installed it — nvm, fnm, volta, asdf, Homebrew. Two steps:
///
/// 1. Drop `PATH` entries that are executables rather than directories
///    ([`EXECUTABLE_PATH_SUFFIXES`]).
/// 2. Prepend the directory holding the resolved command, through
///    [`via_core::search_path::merge_search_path`], which dedupes
///    case-insensitively on Windows.
///
/// `command` is the *resolved* absolute path, not the configured name: the
/// point is to publish the directory the command was actually found in.
#[must_use]
pub fn compose_child_search_path(current: &str, command: &str, platform: Platform) -> String {
    let delimiter = platform.delimiter();
    let cleaned: Vec<&str> = current
        .split(delimiter)
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .filter(|entry| {
            let lowered = entry.to_lowercase();
            !EXECUTABLE_PATH_SUFFIXES
                .iter()
                .any(|suffix| lowered.ends_with(suffix))
        })
        .collect();
    let command_dir = command_directory(command, platform);
    merge_search_path(
        &cleaned.join(&delimiter.to_string()),
        &command_dir,
        platform,
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: EnvironmentPolicy = EnvironmentPolicy {
        names: &["FIXTURE_TOKEN"],
        prefixes: &["FIXTURE_"],
        explicit_list_environment: Some("VIA_FIXTURE_FORWARD_ENV"),
    };

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    #[test]
    fn forwards_only_what_a_policy_names() {
        let projected = backend_environment(
            &POLICY,
            &env(&[
                ("PATH", "/usr/bin"),
                ("FIXTURE_TOKEN", "keep"),
                ("FIXTURE_EXTRA", "keep"),
                ("VIA_AUTH_SECRET", "must not cross"),
                ("DASHSCOPE_API_KEY", "must not cross"),
                ("SOMETHING_ELSE", "must not cross"),
            ]),
            &[],
        );
        assert_eq!(projected.get("PATH"), Some("/usr/bin"));
        assert_eq!(projected.get("FIXTURE_TOKEN"), Some("keep"));
        assert_eq!(projected.get("FIXTURE_EXTRA"), Some("keep"));
        assert_eq!(projected.get("VIA_AUTH_SECRET"), None);
        assert_eq!(projected.get("DASHSCOPE_API_KEY"), None);
        assert_eq!(projected.get("SOMETHING_ELSE"), None);
        assert_eq!(projected.get(ENV_LOADED_NAME), Some(ENV_LOADED_VALUE));
    }

    #[test]
    fn the_explicit_list_is_opt_in_and_comma_separated() {
        let projected = backend_environment(
            &POLICY,
            &env(&[
                ("VIA_FIXTURE_FORWARD_ENV", " ONE , TWO ,, "),
                ("ONE", "1"),
                ("TWO", "2"),
                ("THREE", "3"),
            ]),
            &[],
        );
        assert_eq!(projected.get("ONE"), Some("1"));
        assert_eq!(projected.get("TWO"), Some("2"));
        assert_eq!(projected.get("THREE"), None);
    }

    #[test]
    fn additions_win_over_the_projection() {
        let projected = backend_environment(
            &POLICY,
            &env(&[("FIXTURE_TOKEN", "from parent")]),
            &[("FIXTURE_TOKEN".into(), "from driver".into())],
        );
        assert_eq!(projected.get("FIXTURE_TOKEN"), Some("from driver"));
    }

    #[test]
    fn composing_path_drops_executable_entries_and_leads_with_the_command() {
        assert_eq!(
            compose_child_search_path(
                "C:\\tools\\nodejs\\npm.cmd;C:\\Windows",
                "C:\\Program Files\\nodejs\\npx.cmd",
                Platform::Windows,
            ),
            "C:\\Program Files\\nodejs;C:\\Windows"
        );
        assert_eq!(
            compose_child_search_path(
                "/usr/bin:/bin",
                "/Users/x/.nvm/versions/node/v24.0.0/bin/npx",
                Platform::Posix,
            ),
            "/Users/x/.nvm/versions/node/v24.0.0/bin:/usr/bin:/bin"
        );
    }

    #[test]
    fn composing_path_does_not_duplicate_a_directory_already_present() {
        assert_eq!(
            compose_child_search_path(
                "/opt/node/bin:/usr/bin",
                "/opt/node/bin/npx",
                Platform::Posix
            ),
            "/opt/node/bin:/usr/bin"
        );
    }
}
