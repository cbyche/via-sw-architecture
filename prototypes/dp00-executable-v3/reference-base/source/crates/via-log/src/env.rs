//! Environment inputs: the `VIA_LOG_*` variables and log-directory
//! resolution.
//!
//! **External contract** — upstream `shared/logger.mjs:24-25,47-57,243-283`
//! and `server/src/core/logger.mjs:8-16`, with the `QWEN_AUDIO_LOG_*` /
//! `QWAUDIO_CONFIG_DIR` prefixes renamed to `VIA_*` per `docs/rebrand.md`.
//!
//! Two subtleties a naive port drops:
//!
//! * the log directory falls back to `$VIA_CONFIG_DIR/logs` **before**
//!   `$XDG_CONFIG_HOME`;
//! * `VIA_LOG_CONSOLE` / `VIA_LOG_FILE` disable their sink only when the value
//!   is exactly `"0"` — any other value, including `"false"`, leaves it on.

use crate::level::{DEFAULT_LOG_LEVEL, LogLevel};
use crate::sink::{
    DEFAULT_MAX_BYTES, DEFAULT_MAX_FILES, MAX_MAX_BYTES, MAX_MAX_FILES, MIN_MAX_BYTES,
    MIN_MAX_FILES, absolute_path,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Selects the level. Unrecognised values fall back to `info`.
///
/// **External contract** — upstream `QWEN_AUDIO_LOG_LEVEL`.
pub const ENV_LOG_LEVEL: &str = "VIA_LOG_LEVEL";

/// Overrides the log directory outright.
///
/// **External contract** — upstream `QWEN_AUDIO_LOG_DIR`.
pub const ENV_LOG_DIR: &str = "VIA_LOG_DIR";

/// Disables the console sink when exactly `"0"`.
///
/// **External contract** — upstream `QWEN_AUDIO_LOG_CONSOLE`.
pub const ENV_LOG_CONSOLE: &str = "VIA_LOG_CONSOLE";

/// Disables the file sink when exactly `"0"`.
///
/// **External contract** — upstream `QWEN_AUDIO_LOG_FILE`.
pub const ENV_LOG_FILE: &str = "VIA_LOG_FILE";

/// Rotation threshold in bytes; clamped to `[1024, 1073741824]`.
///
/// **External contract** — upstream `QWEN_AUDIO_LOG_MAX_BYTES`.
pub const ENV_LOG_MAX_BYTES: &str = "VIA_LOG_MAX_BYTES";

/// Retained file count; clamped to `[1, 100]`.
///
/// **External contract** — upstream `QWEN_AUDIO_LOG_MAX_FILES`.
pub const ENV_LOG_MAX_FILES: &str = "VIA_LOG_MAX_FILES";

/// The config directory; logs land in `<VIA_CONFIG_DIR>/logs`.
///
/// **External contract** — upstream `QWAUDIO_CONFIG_DIR`.
pub const ENV_CONFIG_DIR: &str = "VIA_CONFIG_DIR";

/// XDG base directory, consulted after [`ENV_CONFIG_DIR`].
///
/// **External contract** — kept verbatim; it belongs to the XDG spec.
pub const ENV_XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

/// Node's environment selector, checked for the literal `"test"`.
///
/// **External contract** — upstream `server/src/core/logger.mjs:8`
/// (`process.env.NODE_ENV === 'test'`). The name belongs to Node, not to this
/// product, so `docs/rebrand.md` does not rename it and neither does this
/// crate. See [`is_test_process`] for why a Rust build also needs
/// [`ENV_TEST_PROCESS`].
pub const ENV_NODE_ENV: &str = "NODE_ENV";

/// The one [`ENV_NODE_ENV`] value that means "this is a test run".
///
/// **External contract** — upstream compares with `===`, so `"TEST"` and
/// `"testing"` are not matches and are not treated as ones here either.
pub const NODE_ENV_TEST: &str = "test";

/// VIA's own "this process is a test run" signal.
///
/// **Not an upstream contract — new to VIA, and load-bearing.** Upstream's two
/// tests are `NODE_ENV === 'test'` and a `test/` or `tests/` segment in
/// `argv`. Neither fires for a Rust build: nothing sets `NODE_ENV`, and a
/// `cargo test` binary is invoked as `target/<profile>/deps/<name>-<hash>`,
/// which contains no `test/` path segment. Without a third arm
/// [`is_test_process`] would answer `false` for every VIA test run and the
/// Gateway logger would write into the developer's real log directory — which
/// is exactly the outcome the upstream check exists to prevent.
///
/// `via-core` (or a test harness) sets this; `docs/deviations/phase-0.md`
/// records the gap this closes.
pub const ENV_TEST_PROCESS: &str = "VIA_TEST";

/// The one [`ENV_TEST_PROCESS`] value that means "this is a test run".
///
/// Compared exactly, matching the crate's other boolean environment inputs
/// ([`ENV_LOG_CONSOLE`] / [`ENV_LOG_FILE`] disable their sink only on exactly
/// `"0"`). `"true"`, `"yes"` and `"TEST"` are not matches — a variable that
/// silences logging on a fuzzy match is one that silences it by accident.
pub const TEST_PROCESS_VALUE: &str = "1";

/// Final path segment under the config base.
///
/// **External contract** — upstream `'qwaudio/logs'`
/// (`shared/logger.mjs:56`), renamed per `docs/rebrand.md` to `via`.
pub const CONFIG_DIRECTORY_NAME: &str = "via";

/// Subdirectory holding log files.
pub const LOG_DIRECTORY_NAME: &str = "logs";

/// A read-only view of process environment variables.
///
/// Only lookups are performed, never iteration, so no ordering is observable
/// through this trait.
pub trait EnvSource {
    /// Read one variable. `None` and `Some("")` are treated alike by every
    /// caller in this crate, matching JavaScript's truthiness checks.
    fn get(&self, key: &str) -> Option<String>;

    /// Read one variable, mapping the empty string to `None`.
    fn get_non_empty(&self, key: &str) -> Option<String> {
        self.get(key).filter(|value| !value.is_empty())
    }
}

/// The real process environment.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessEnv;

impl EnvSource for ProcessEnv {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

impl EnvSource for BTreeMap<String, String> {
    fn get(&self, key: &str) -> Option<String> {
        BTreeMap::get(self, key).cloned()
    }
}

impl EnvSource for BTreeMap<&str, &str> {
    fn get(&self, key: &str) -> Option<String> {
        BTreeMap::get(self, key).map(|value| (*value).to_owned())
    }
}

impl<T: EnvSource + ?Sized> EnvSource for &T {
    fn get(&self, key: &str) -> Option<String> {
        (**self).get(key)
    }
}

/// Parse an integer the way JavaScript's `Number.parseInt(value, 10)` does,
/// then clamp it.
///
/// Leading whitespace is skipped, an optional sign is accepted, and parsing
/// stops at the first non-digit — so `"12abc"` is `12` and `"abc"` falls back.
/// Values too large for the range saturate rather than wrapping.
///
/// **External contract** — upstream `boundedInteger`
/// (`shared/logger.mjs:36-40`).
#[must_use]
pub fn bounded_integer(value: Option<&str>, fallback: i128, min: i128, max: i128) -> i128 {
    let Some(parsed) = parse_int_radix10(value.unwrap_or_default()) else {
        return fallback;
    };
    parsed.clamp(min, max)
}

fn parse_int_radix10(value: &str) -> Option<i128> {
    let trimmed = value.trim_start();
    let (negative, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    let leading: String = digits.chars().take_while(char::is_ascii_digit).collect();
    if leading.is_empty() {
        return None;
    }
    // A value beyond i128 stays "finite" in JavaScript and is clamped by the
    // caller, so saturating at the extreme reproduces the same outcome.
    let magnitude = leading.parse::<i128>().unwrap_or(i128::MAX);
    Some(if negative {
        magnitude.saturating_neg()
    } else {
        magnitude
    })
}

/// Resolve the log directory.
///
/// **External contract** — upstream `defaultLogDirectory`
/// (`shared/logger.mjs:47-57`), in order:
/// `VIA_LOG_DIR` > `$VIA_CONFIG_DIR/logs` > `$XDG_CONFIG_HOME/via/logs` >
/// `<home>/.config/via/logs`.
#[must_use]
pub fn default_log_directory(env: &dyn EnvSource, home: &Path) -> PathBuf {
    if let Some(directory) = env.get_non_empty(ENV_LOG_DIR) {
        return absolute_path(Path::new(&directory));
    }
    if let Some(config) = env.get_non_empty(ENV_CONFIG_DIR) {
        return absolute_path(&Path::new(&config).join(LOG_DIRECTORY_NAME));
    }
    let base = match env.get_non_empty(ENV_XDG_CONFIG_HOME) {
        Some(xdg) => absolute_path(Path::new(&xdg)),
        None => absolute_path(&home.join(".config")),
    };
    base.join(CONFIG_DIRECTORY_NAME).join(LOG_DIRECTORY_NAME)
}

/// [`default_log_directory`] against the real process environment.
///
/// Falls back to the current directory when no home directory can be found,
/// which is the only branch upstream cannot reach (`os.homedir()` throws
/// there).
#[must_use]
pub fn default_log_directory_from_process() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    default_log_directory(&ProcessEnv, &home)
}

/// Whether the current process looks like a test run.
///
/// **External contract, plus one VIA arm.** Upstream
/// `server/src/core/logger.mjs:8-9` is `NODE_ENV === 'test'` or any argv entry
/// containing a `test/` or `tests/` path segment. A match disables **both** the
/// console and the file sink.
///
/// Three tests, in order:
///
/// | Test | Source |
/// | --- | --- |
/// | `VIA_TEST == "1"` | VIA's own — see [`ENV_TEST_PROCESS`] |
/// | `NODE_ENV == "test"` | upstream, reproduced verbatim |
/// | an argv entry with a `test/` or `tests/` segment | upstream |
///
/// The two upstream arms are kept for fidelity and are asserted, but neither
/// fires for a Rust build: nothing sets `NODE_ENV`, and a `cargo test` binary
/// is `target/<profile>/deps/<name>-<hash>`, which has no `test/` segment. The
/// `VIA_TEST` arm is what actually carries the behaviour, and it is why
/// removing it would silently reinstate the bug the upstream check prevents.
#[must_use]
pub fn is_test_process<I, S>(env: &dyn EnvSource, args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    if env.get(ENV_TEST_PROCESS).as_deref() == Some(TEST_PROCESS_VALUE) {
        return true;
    }
    if env.get(ENV_NODE_ENV).as_deref() == Some(NODE_ENV_TEST) {
        return true;
    }
    args.into_iter().any(|argument| {
        let argument = argument.as_ref();
        argument.contains("/test/")
            || argument.contains("/tests/")
            || argument.contains("\\test\\")
            || argument.contains("\\tests\\")
    })
}

/// The `VIA_LOG_*` settings, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvSettings {
    /// Threshold level.
    pub level: LogLevel,
    /// Directory for the file sink.
    pub directory: PathBuf,
    /// Whether the console sink is mounted.
    pub console_enabled: bool,
    /// Whether the file sink is mounted.
    pub file_enabled: bool,
    /// Rotation threshold in bytes, already clamped.
    pub max_bytes: u64,
    /// Retained file count, already clamped.
    pub max_files: u32,
}

impl EnvSettings {
    /// Read every logging variable from `env`.
    #[must_use]
    pub fn from_env(env: &dyn EnvSource, home: &Path) -> Self {
        let max_bytes = bounded_integer(
            env.get(ENV_LOG_MAX_BYTES).as_deref(),
            DEFAULT_MAX_BYTES as i128,
            MIN_MAX_BYTES as i128,
            MAX_MAX_BYTES as i128,
        );
        let max_files = bounded_integer(
            env.get(ENV_LOG_MAX_FILES).as_deref(),
            i128::from(DEFAULT_MAX_FILES),
            i128::from(MIN_MAX_FILES),
            i128::from(MAX_MAX_FILES),
        );
        Self {
            level: LogLevel::normalize_opt(env.get(ENV_LOG_LEVEL).as_deref(), DEFAULT_LOG_LEVEL),
            directory: default_log_directory(env, home),
            // Upstream: `env.QWEN_AUDIO_LOG_CONSOLE !== '0'`.
            console_enabled: env.get(ENV_LOG_CONSOLE).as_deref() != Some("0"),
            file_enabled: env.get(ENV_LOG_FILE).as_deref() != Some("0"),
            // Both values are clamped into ranges that fit their target types.
            max_bytes: max_bytes.clamp(0, i128::from(u64::MAX)) as u64,
            max_files: max_files.clamp(0, i128::from(u32::MAX)) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn parse_int_follows_javascript() {
        assert_eq!(parse_int_radix10("42"), Some(42));
        assert_eq!(parse_int_radix10("  42  "), Some(42));
        assert_eq!(parse_int_radix10("12abc"), Some(12));
        assert_eq!(parse_int_radix10("0x10"), Some(0));
        assert_eq!(parse_int_radix10("-7"), Some(-7));
        assert_eq!(parse_int_radix10("abc"), None);
        assert_eq!(parse_int_radix10(""), None);
    }

    #[test]
    fn directory_prefers_config_dir_over_xdg() {
        let vars = env(&[
            (ENV_CONFIG_DIR, "/opt/via-config"),
            (ENV_XDG_CONFIG_HOME, "/opt/xdg"),
        ]);
        assert_eq!(
            default_log_directory(&vars, Path::new("/home/tester")),
            PathBuf::from("/opt/via-config/logs")
        );
    }

    #[test]
    fn test_process_detection_matches_upstream_predicate() {
        let vars = env(&[(ENV_NODE_ENV, NODE_ENV_TEST)]);
        assert!(is_test_process(&vars, Vec::<String>::new()));
        let empty = env(&[]);
        assert!(is_test_process(&empty, ["/repo/tests/logger.rs"]));
        assert!(is_test_process(&empty, [r"C:\repo\test\logger.rs"]));
        assert!(!is_test_process(&empty, ["/repo/src/main.rs"]));
    }

    #[test]
    fn the_via_arm_fires_on_exactly_one_value() {
        let on = env(&[(ENV_TEST_PROCESS, TEST_PROCESS_VALUE)]);
        assert!(is_test_process(&on, Vec::<String>::new()));

        for near_miss in ["", "0", "true", "yes", "TEST", "test", "2", " 1"] {
            let vars = env(&[(ENV_TEST_PROCESS, near_miss)]);
            assert!(
                !is_test_process(&vars, Vec::<String>::new()),
                "VIA_TEST={near_miss:?} must not silence logging"
            );
        }
    }

    #[test]
    fn node_env_is_compared_exactly() {
        for near_miss in ["Test", "TEST", "testing", "test ", "development", ""] {
            let vars = env(&[(ENV_NODE_ENV, near_miss)]);
            assert!(
                !is_test_process(&vars, Vec::<String>::new()),
                "NODE_ENV={near_miss:?} is not upstream's `=== 'test'`"
            );
        }
    }

    #[test]
    fn a_cargo_test_binary_needs_the_via_arm() {
        // The exact shape `cargo test` invokes: no NODE_ENV, no `test/`
        // segment. Both upstream arms miss it, which is the gap VIA_TEST
        // closes.
        let argv = ["/repo/target/debug/deps/via_log-3f1c2a9b4d5e6f70"];
        let empty = env(&[]);
        assert!(
            !is_test_process(&empty, argv),
            "neither upstream arm can see a cargo test binary"
        );
        let flagged = env(&[(ENV_TEST_PROCESS, TEST_PROCESS_VALUE)]);
        assert!(is_test_process(&flagged, argv));
    }
}
