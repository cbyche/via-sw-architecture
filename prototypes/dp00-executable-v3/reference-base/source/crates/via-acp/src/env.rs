//! The credential boundary every ACP child process is spawned behind.
//!
//! Ported from `shared/backend-environment.mjs`, catalogued three times
//! (*"child-process environment filter"*, *"backend child environment
//! allowlist"*, *"backend env allowlist internals"*) and described by
//! `docs/architecture.md` §17 as a review question in its own right:
//!
//! > Did a child process spawn without `.env_clear()`? Rust's
//! > inherit-by-default `Command` breaks the credential boundary silently.
//!
//! That asymmetry is the whole reason this module exists as a type rather than
//! a function. Node's `spawn(cmd, args, { env })` **replaces** the child's
//! environment, so upstream's filter is enforced by the shape of the call.
//! Rust's [`std::process::Command`] **inherits** unless told otherwise, so the
//! same code written naively leaks the Gateway auth secret, the realtime API
//! key, the memory-extractor key and the speech-to-speech token into every
//! third-party agent — and leaks them silently, because nothing fails.
//!
//! So: [`BackendEnv`] is a newtype with no public constructor except
//! [`BackendEnv::project`], which takes a [`EnvironmentPolicy`] from
//! `via-catalog`; [`crate::process`] is the only spawn site in the crate; and
//! it calls `.env_clear()` before applying a `BackendEnv` and nothing else.
//! There is no way to hand the spawn path a raw map.
//!
//! # What crosses
//!
//! 1. [`SYSTEM_NAMES`] — the portable operating-system context a program needs
//!    in order to run at all.
//! 2. [`SYSTEM_PREFIXES`] — locale (`LC_`) and the package-runner's own
//!    configuration.
//! 3. [`INTERNAL_NAMES`] — VIA settings the **child** reads.
//! 4. The selected profile's own declared names and prefixes — its credential
//!    namespace, and only its own.
//! 5. Names the operator explicitly listed in the policy's
//!    [`explicit_list_environment`](EnvironmentPolicy::explicit_list_environment)
//!    variable, for the generic entry point that has no vendor namespace.
//!
//! Everything else — every other backend's credentials included — is absent
//! from the child's environment, not merely unused by it.

use std::collections::BTreeMap;

use via_catalog::EnvironmentPolicy;
use via_core::EnvMap;

/// The portable operating-system context a child process is given.
///
/// **External contract** — `shared/backend-environment.mjs:7-47`, reproduced
/// verbatim (all 39 names are OS- or vendor-owned, so `docs/rebrand.md` renames
/// none of them). Sorted as upstream sorts them: ASCII, uppercase before
/// lowercase, which puts the three lowercase proxy aliases last.
///
/// The catalogue's prose says "47 OS names"; the list it then enumerates, and
/// the upstream source, both hold 39. The list is authoritative — see
/// `tests/backend_environment.rs`.
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

/// Prefixes whose every variable crosses.
///
/// **External contract** — `shared/backend-environment.mjs:63`. `LC_` is the
/// locale family; the two package-runner spellings matter because several
/// backends are launched through one.
pub const SYSTEM_PREFIXES: &[&str] = &["LC_", "npm_config_", "NPM_CONFIG_"];

/// VIA's own settings that the **child** reads.
///
/// **External contract** — `shared/backend-environment.mjs:49-61`, with the
/// `QWEN_AUDIO_AGENT_*` → `VIA_*` rename mandated by `docs/rebrand.md` row
/// "QWEN_AUDIO_AGENT_ENV_LOADED, _NODE, _ROOT, _RUNTIME_ROOT, _SOURCE_ROOT".
/// The catalogue notes why the rename is a coordinated change rather than an
/// internal one: these names are read by *other programs*.
///
/// Eleven names, matching the upstream set exactly. `VIA_COMPUTER_USE` is
/// deliberately **not** here: upstream reads it in `builtin-mcp.mjs:54` to
/// decide whether to declare a server and passes it explicitly, so it crosses
/// as an addition when it crosses at all, and adding it to the allow-list would
/// widen the boundary for every backend.
pub const INTERNAL_NAMES: &[&str] = &[
    via_core::config::names::BACKEND_AGENT,
    via_core::config::names::BACKEND_MODEL,
    via_core::config::names::BACKEND_OWNERSHIP,
    via_core::config::names::BACKEND_PERMISSION_MODE,
    DESKTOP,
    DESKTOP_INSTALLED_ONLY,
    ENV_LOADED,
    NODE,
    ROOT,
    via_core::config::names::RUNTIME_ROOT,
    SOURCE_ROOT,
];

/// `VIA_DESKTOP` — the child is running under the desktop shell.
pub const DESKTOP: &str = "VIA_DESKTOP";
/// `VIA_DESKTOP_INSTALLED_ONLY` — refuse package-runner fallbacks.
pub const DESKTOP_INSTALLED_ONLY: &str = "VIA_DESKTOP_INSTALLED_ONLY";
/// `VIA_ENV_LOADED` — stamped `1` on every projected environment.
pub const ENV_LOADED: &str = "VIA_ENV_LOADED";
/// `VIA_NODE` — the Node interpreter a launcher shim should re-exec.
///
/// Upstream always stamps `process.execPath`. VIA has no interpreter of its
/// own, so this crate never stamps a value; the name stays on the allow-list,
/// and a launch spec that genuinely needs a Node runtime supplies it as an
/// addition. Recorded in `docs/deviations/phase-2.md`.
pub const NODE: &str = "VIA_NODE";
/// `VIA_ROOT` — the installation root.
pub const ROOT: &str = "VIA_ROOT";
/// `VIA_SOURCE_ROOT` — the source checkout, when running from one.
pub const SOURCE_ROOT: &str = "VIA_SOURCE_ROOT";

/// The value [`ENV_LOADED`] is stamped with.
///
/// **External contract** — `shared/backend-environment.mjs:99`
/// (`QWEN_AUDIO_AGENT_ENV_LOADED: '1'`). A launcher shim reads it to know the
/// configuration has already been loaded and must not be loaded again.
pub const ENV_LOADED_VALUE: &str = "1";

/// The environment one child process will receive — and nothing else.
///
/// Constructed **only** by [`BackendEnv::project`], which requires a
/// [`EnvironmentPolicy`]. That is the invariant the whole module exists for:
/// there is no `From<HashMap>`, no `insert`, and no way to reach
/// [`crate::process`] with a map that did not pass the filter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BackendEnv {
    vars: BTreeMap<String, String>,
}

impl BackendEnv {
    /// Project a parent environment through one backend's policy.
    ///
    /// **Contract** — `backendEnvironment(protocol, { env, additions })`,
    /// `shared/backend-environment.mjs:84-103`. Order of application, which is
    /// observable because later writes win:
    ///
    /// 1. every allowed name from `env`,
    /// 2. `VIA_ENV_LOADED=1`,
    /// 3. `additions`.
    ///
    /// So an addition may override a stamped or inherited value, and the stamp
    /// may override an inherited `VIA_ENV_LOADED` — exactly as upstream's
    /// object spread does.
    ///
    /// `additions` are values the caller **computed** (a workspace directory, a
    /// token file path, a runtime override), never values it read back out of
    /// the parent environment. Reading the parent environment is this
    /// function's job, and it is the only place it happens.
    #[must_use]
    pub fn project(policy: &EnvironmentPolicy, env: &EnvMap, additions: &[(&str, &str)]) -> Self {
        let explicit = explicit_names(policy, env);
        let mut vars: BTreeMap<String, String> = env
            .iter()
            .filter(|(name, _)| allowed(name, policy, &explicit))
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect();
        vars.insert(ENV_LOADED.to_owned(), ENV_LOADED_VALUE.to_owned());
        for (name, value) in additions {
            vars.insert((*name).to_owned(), (*value).to_owned());
        }
        Self { vars }
    }

    /// One variable's value.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(String::as_str)
    }

    /// Whether a name crossed the boundary at all.
    ///
    /// The question a security test asks; distinct from `get(..) == Some("")`,
    /// because an empty value that crossed is still a leak of the fact that the
    /// variable is set.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// Every variable, in name order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// How many variables the child receives.
    #[must_use]
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// Whether the child receives nothing at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }
}

/// The names an operator listed in the policy's explicit-forward variable.
///
/// **Contract** — `explicitNames`, `shared/backend-environment.mjs:69-73`:
/// comma-separated, each entry trimmed, empties dropped. The variable that
/// holds the list is itself **not** forwarded, because it is not on any
/// allow-list — which is asserted, because "the opt-in leaks itself" would be a
/// quiet information disclosure.
fn explicit_names(policy: &EnvironmentPolicy, env: &EnvMap) -> Vec<String> {
    let Some(name) = policy.explicit_list_environment else {
        return Vec::new();
    };
    let name = name.trim();
    if name.is_empty() {
        return Vec::new();
    }
    via_core::env::comma_list(env.get(name))
}

/// `allowed(name, policy, explicit)` — `shared/backend-environment.mjs:75-82`.
///
/// Case-sensitive throughout, as upstream's `Set.has` and `startsWith` are.
fn allowed(name: &str, policy: &EnvironmentPolicy, explicit: &[String]) -> bool {
    SYSTEM_NAMES.contains(&name)
        || INTERNAL_NAMES.contains(&name)
        || SYSTEM_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
        || policy.allows(name)
        || explicit.iter().any(|entry| entry == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic policy. This crate must never name a backend
    /// (`docs/architecture.md` §9; upstream's `dependency-boundaries.test.mjs`
    /// asserts the same rule with a regex over source text), so the fixtures
    /// here are invented namespaces rather than catalogue entries. The
    /// verbatim port of `server/test/backend-environment.test.mjs`, which does
    /// name backends, belongs with the catalogue that declares them.
    fn policy(
        names: &'static [&'static str],
        prefixes: &'static [&'static str],
    ) -> EnvironmentPolicy {
        EnvironmentPolicy {
            names,
            prefixes,
            explicit_list_environment: None,
        }
    }

    fn parent() -> EnvMap {
        [
            ("PATH", "/usr/bin"),
            ("HOME", "/home/user"),
            ("LANG", "zh_CN.UTF-8"),
            ("LC_ALL", "zh_CN.UTF-8"),
            ("npm_config_registry", "https://registry.example.test"),
            ("VIA_BACKEND_MODEL", "example-model"),
            ("EXAMPLE_ONE_API_KEY", "one-secret"),
            ("EXAMPLE_TWO_API_KEY", "two-secret"),
            ("VIA_AUTH_SECRET", "identity-secret"),
            ("VIA_REALTIME_API_KEY", "realtime-secret"),
            ("VIA_MEMORY_API_KEY", "memory-secret"),
            ("SPEECH_TO_SPEECH_AUTH_TOKEN", "speech-secret"),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn projects_only_operating_system_and_the_selected_namespace() {
        let projected = BackendEnv::project(&policy(&["EXAMPLE_ONE_API_KEY"], &[]), &parent(), &[]);

        assert_eq!(projected.get("PATH"), Some("/usr/bin"));
        assert_eq!(projected.get("HOME"), Some("/home/user"));
        assert_eq!(projected.get("LANG"), Some("zh_CN.UTF-8"));
        assert_eq!(projected.get("EXAMPLE_ONE_API_KEY"), Some("one-secret"));

        // The other namespace, and every Gateway secret, is absent.
        assert!(!projected.contains("EXAMPLE_TWO_API_KEY"));
        assert!(!projected.contains("VIA_AUTH_SECRET"));
        assert!(!projected.contains("VIA_REALTIME_API_KEY"));
        assert!(!projected.contains("VIA_MEMORY_API_KEY"));
        assert!(!projected.contains("SPEECH_TO_SPEECH_AUTH_TOKEN"));
    }

    #[test]
    fn each_credential_namespace_stays_isolated() {
        let one = BackendEnv::project(&policy(&["EXAMPLE_ONE_API_KEY"], &[]), &parent(), &[]);
        let two = BackendEnv::project(&policy(&[], &["EXAMPLE_TWO_"]), &parent(), &[]);

        assert_eq!(one.get("EXAMPLE_ONE_API_KEY"), Some("one-secret"));
        assert!(!one.contains("EXAMPLE_TWO_API_KEY"));
        assert_eq!(two.get("EXAMPLE_TWO_API_KEY"), Some("two-secret"));
        assert!(!two.contains("EXAMPLE_ONE_API_KEY"));
    }

    #[test]
    fn additional_names_cross_only_when_explicitly_requested() {
        // The third case of server/test/backend-environment.test.mjs, ported
        // against the generic policy shape rather than a named backend.
        let opt_in = EnvironmentPolicy {
            names: &[],
            prefixes: &["ACP_"],
            explicit_list_environment: Some("VIA_ACP_FORWARD_ENV"),
        };
        let mut env = parent();
        env.set("CUSTOM_AGENT_TOKEN", "custom-secret");
        env.set("VIA_ACP_FORWARD_ENV", "CUSTOM_AGENT_TOKEN");

        let projected = BackendEnv::project(&opt_in, &env, &[]);
        assert_eq!(projected.get("CUSTOM_AGENT_TOKEN"), Some("custom-secret"));
        assert!(!projected.contains("EXAMPLE_ONE_API_KEY"));
        assert!(
            !projected.contains("VIA_ACP_FORWARD_ENV"),
            "the opt-in list must not forward itself"
        );

        let without = BackendEnv::project(&opt_in, &parent(), &[]);
        assert!(!without.contains("CUSTOM_AGENT_TOKEN"));
    }

    #[test]
    fn the_explicit_list_is_comma_separated_and_trimmed() {
        let opt_in = EnvironmentPolicy {
            names: &[],
            prefixes: &[],
            explicit_list_environment: Some("VIA_ACP_FORWARD_ENV"),
        };
        let env: EnvMap = [
            ("A_TOKEN", "a"),
            ("B_TOKEN", "b"),
            ("C_TOKEN", "c"),
            ("VIA_ACP_FORWARD_ENV", " A_TOKEN , ,B_TOKEN "),
        ]
        .into_iter()
        .collect();
        let projected = BackendEnv::project(&opt_in, &env, &[]);
        assert_eq!(projected.get("A_TOKEN"), Some("a"));
        assert_eq!(projected.get("B_TOKEN"), Some("b"));
        assert!(!projected.contains("C_TOKEN"));
    }

    #[test]
    fn prefix_and_name_matching_are_case_sensitive() {
        let env: EnvMap = [
            ("EXAMPLE_KEY", "upper"),
            ("example_key", "lower"),
            ("lc_all", "lower locale"),
        ]
        .into_iter()
        .collect();
        let projected = BackendEnv::project(&policy(&["EXAMPLE_KEY"], &["EXAMPLE_"]), &env, &[]);
        assert_eq!(projected.get("EXAMPLE_KEY"), Some("upper"));
        assert!(!projected.contains("example_key"));
        assert!(
            !projected.contains("lc_all"),
            "the LC_ prefix is uppercase; `lc_all` is not a locale variable"
        );
    }

    #[test]
    fn env_loaded_is_always_stamped_and_additions_win() {
        let mut env = parent();
        env.set(ENV_LOADED, "stale");
        let projected = BackendEnv::project(&policy(&[], &[]), &env, &[]);
        assert_eq!(projected.get(ENV_LOADED), Some("1"));

        let overridden = BackendEnv::project(&policy(&[], &[]), &env, &[(ENV_LOADED, "0")]);
        assert_eq!(
            overridden.get(ENV_LOADED),
            Some("0"),
            "additions are applied after the stamp"
        );
    }

    #[test]
    fn internal_names_cross_but_gateway_names_do_not() {
        let projected = BackendEnv::project(&policy(&[], &[]), &parent(), &[]);
        assert_eq!(projected.get("VIA_BACKEND_MODEL"), Some("example-model"));
        assert!(!projected.contains("VIA_AUTH_SECRET"));
        assert_eq!(projected.get("LC_ALL"), Some("zh_CN.UTF-8"));
        assert_eq!(
            projected.get("npm_config_registry"),
            Some("https://registry.example.test")
        );
    }

    #[test]
    fn an_empty_value_is_forwarded_rather_than_dropped() {
        // Upstream filters `value !== undefined`, not falsiness: an explicitly
        // empty `PATH` masks the inherited one, and must keep doing so.
        let env: EnvMap = [("PATH", "")].into_iter().collect();
        let projected = BackendEnv::project(&policy(&[], &[]), &env, &[]);
        assert_eq!(projected.get("PATH"), Some(""));
    }

    #[test]
    fn a_policy_with_no_explicit_variable_forwards_no_extras() {
        let env: EnvMap = [
            ("CUSTOM_AGENT_TOKEN", "custom"),
            ("VIA_ACP_FORWARD_ENV", "CUSTOM_AGENT_TOKEN"),
        ]
        .into_iter()
        .collect();
        let projected = BackendEnv::project(&policy(&[], &[]), &env, &[]);
        assert!(!projected.contains("CUSTOM_AGENT_TOKEN"));
    }
}
