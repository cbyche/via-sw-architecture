//! Cross-platform executable discovery.
//!
//! Ported from `findExecutable`, `shared/backend-setup.mjs:57-91`. Every setup,
//! install and onboarding decision rests on it, so it is injected as a trait
//! rather than called directly: upstream threads a `find` parameter through
//! `inspectBackendSetups`, `installBackend` and `inspectAdapter` for the same
//! reason.
//!
//! # Three behaviours `which` alone does not give
//!
//! 1. **`~` expansion.** `OPENCLAW_BUNDLE_BIN` defaults to
//!    `~/.openclaw-bundle/wrapper/openclaw` (`backend-setup.mjs:262-266`), and a
//!    tilde is not a path component to any OS.
//! 2. **A `PATH` entry that is itself the executable.** Windows installs in the
//!    wild put `C:\tools\nodejs\npm.cmd` on `PATH`
//!    (`backend-setup.mjs:82-89`). `which` would join and miss it.
//! 3. **The empty answer.** Upstream returns `''`, and every caller branches on
//!    truthiness. A `Result` here would turn "not installed" — an ordinary,
//!    reportable state — into an error path.
//!
//! Executability is `X_OK` on POSIX and mere existence on Windows
//! (`backend-setup.mjs:45-56`), because Windows has no execute bit and
//! `PATHEXT` already decides what is runnable.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::platform::HostPlatform;
use via_core::EnvMap;

/// Windows' `PATHEXT` default.
///
/// **External contract** — `shared/backend-setup.mjs:41-47`. Lower-cased before
/// use, as upstream lower-cases it.
pub const DEFAULT_PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";

/// Locating an executable.
///
/// A trait because the answer depends on a filesystem and a `PATH` that a test
/// must be able to supply — the same reason upstream passes `find` in.
pub trait ExecutableFinder: Send + Sync + fmt::Debug {
    /// The absolute path `command` resolves to, or the empty string.
    ///
    /// `command` may be a bare name, a relative or absolute path, or a
    /// `~`-rooted one.
    fn find(&self, command: &str) -> String;
}

/// The real finder: this machine's filesystem, a supplied environment.
#[derive(Debug, Clone)]
pub struct SystemFinder {
    env: EnvMap,
    platform: HostPlatform,
}

impl SystemFinder {
    /// A finder over `env`, resolving with `platform`'s conventions.
    #[must_use]
    pub fn new(env: EnvMap, platform: HostPlatform) -> Self {
        Self { env, platform }
    }
}

impl ExecutableFinder for SystemFinder {
    fn find(&self, command: &str) -> String {
        find_executable(command, &self.env, self.platform)
    }
}

/// A finder that answers `''` for everything.
///
/// The state a machine with nothing installed is in, and the fixture most
/// "not installed" tests want.
#[derive(Debug, Clone, Copy, Default)]
pub struct MissingFinder;

impl ExecutableFinder for MissingFinder {
    fn find(&self, _command: &str) -> String {
        String::new()
    }
}

/// `findExecutable(command, { env, platform })`.
///
/// **External contract** — `shared/backend-setup.mjs:57-91`. Returns an
/// absolute path, or `''` when there is nothing runnable by that name.
#[must_use]
pub fn find_executable(command: &str, env: &EnvMap, platform: HostPlatform) -> String {
    let value = expand_home(command.trim(), env);
    if value.is_empty() {
        return String::new();
    }
    let has_path = is_absolute(&value, platform) || value.contains('/') || value.contains('\\');
    let directories: Vec<String> = if has_path {
        vec![String::new()]
    } else {
        env.get_trimmed("PATH")
            .split(platform.paths().delimiter())
            .filter(|entry| !entry.is_empty())
            .map(str::to_owned)
            .collect()
    };
    let suffixes: Vec<String> = if platform.is_windows() && extension(&value).is_empty() {
        std::iter::once(String::new())
            .chain(executable_extensions(env))
            .collect()
    } else {
        vec![String::new()]
    };

    for directory in &directories {
        for suffix in &suffixes {
            let candidate = if directory.is_empty() {
                resolve_one(&format!("{value}{suffix}"), env)
            } else {
                resolve_two(directory, &format!("{value}{suffix}"), env)
            };
            if is_executable_file(Path::new(&candidate), platform) {
                return candidate;
            }
        }
        // A `PATH` entry may *be* the target file (`C:\tools\nodejs\npm.cmd`);
        // upstream compares the entry's basename against the sought command
        // rather than joining. `backend-setup.mjs:82-89`.
        if !directory.is_empty()
            && !has_path
            && win32_basename(directory).to_lowercase() == value.to_lowercase()
            && is_executable_file(Path::new(directory), platform)
        {
            return directory.clone();
        }
    }
    String::new()
}

/// `expandHome(value, env)` — `shared/backend-setup.mjs:32-39`.
///
/// A bare `~` becomes the home directory; `~/` and `~\` are rooted at it.
/// Anything else, including `~user`, is returned untouched — as upstream's is.
fn expand_home(value: &str, env: &EnvMap) -> String {
    let home = first_trimmed(env, &["HOME", "USERPROFILE"]);
    if value == "~" {
        return if home.is_empty() {
            value.to_owned()
        } else {
            home
        };
    }
    if !value.starts_with("~/") && !value.starts_with("~\\") {
        return value.to_owned();
    }
    if home.is_empty() {
        return value.to_owned();
    }
    resolve_two(&home, &value[2..], env)
}

/// `executableExtensions(platform, env)` — `shared/backend-setup.mjs:41-47`.
fn executable_extensions(env: &EnvMap) -> Vec<String> {
    let raw = env.get_trimmed("PATHEXT");
    let source = if raw.is_empty() { DEFAULT_PATHEXT } else { raw };
    source
        .split(';')
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// `executableFile(path, platform)` — `shared/backend-setup.mjs:49-56`.
///
/// `constants.X_OK` on POSIX; `constants.F_OK` on Windows, where there is no
/// execute bit to test.
fn is_executable_file(path: &Path, platform: HostPlatform) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        // `access()` succeeds on a directory too, but every caller then spawns
        // the result. Refusing a directory is the same answer one step earlier.
        return false;
    }
    if platform.is_windows() {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Node's `path.extname`, restricted to what the `PATHEXT` branch needs.
fn extension(value: &str) -> &str {
    let base = win32_basename(value);
    match base.rfind('.') {
        Some(index) if index > 0 => &base[index..],
        _ => "",
    }
}

/// Node's `path.win32.basename`, which accepts both separators.
fn win32_basename(value: &str) -> &str {
    let trimmed = value.trim_end_matches(['/', '\\']);
    match trimmed.rfind(['/', '\\']) {
        Some(index) => &trimmed[index + 1..],
        None => trimmed,
    }
}

fn is_absolute(value: &str, platform: HostPlatform) -> bool {
    if platform.is_windows() {
        let bytes = value.as_bytes();
        value.starts_with('\\')
            || value.starts_with('/')
            || (bytes.len() >= 3
                && bytes[0].is_ascii_alphabetic()
                && bytes[1] == b':'
                && (bytes[2] == b'\\' || bytes[2] == b'/'))
    } else {
        value.starts_with('/')
    }
}

/// Node's one-argument `path.resolve(value)` — against the process cwd.
fn resolve_one(value: &str, env: &EnvMap) -> String {
    let cwd = env.get_trimmed("PWD");
    if cwd.is_empty() {
        std::env::current_dir()
            .map(|dir| join_absolute(&dir, value))
            .unwrap_or_else(|_| value.to_owned())
    } else {
        join_absolute(Path::new(cwd), value)
    }
}

/// Node's two-argument `path.resolve(base, value)`.
fn resolve_two(base: &str, value: &str, env: &EnvMap) -> String {
    let base = if base.is_empty() {
        return resolve_one(value, env);
    } else {
        Path::new(base)
    };
    join_absolute(base, value)
}

fn join_absolute(base: &Path, value: &str) -> String {
    let candidate = PathBuf::from(value);
    if candidate.is_absolute() {
        candidate.to_string_lossy().into_owned()
    } else {
        base.join(candidate).to_string_lossy().into_owned()
    }
}

fn first_trimmed(env: &EnvMap, names: &[&str]) -> String {
    names
        .iter()
        .map(|name| env.get_trimmed(name))
        .find(|value| !value.is_empty())
        .unwrap_or_default()
        .to_owned()
}

/// The value of the first non-empty variable among `names`, with the name it
/// came from.
///
/// **External contract** — `environmentValue`, `shared/backend-setup.mjs:22-29`.
/// The *name* matters because it is interpolated into the refusal
/// (`QODERCLI_PATH 指定的命令不可用`), and when nothing is set upstream still
/// reports the first name so the message can say what to configure.
#[must_use]
pub fn environment_value<'a>(env: &EnvMap, names: &[&'a str]) -> (&'a str, String) {
    for name in names {
        let value = env.get_trimmed(name);
        if !value.is_empty() {
            return (name, value.to_owned());
        }
    }
    (names.first().copied().unwrap_or(""), String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn write_executable(directory: &Path, name: &str) -> PathBuf {
        let path = directory.join(name);
        let mut file = std::fs::File::create(&path).expect("create fixture");
        file.write_all(b"#!/bin/sh\n").expect("write fixture");
        drop(file);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod fixture");
        }
        path
    }

    #[test]
    fn finds_a_bare_command_on_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let expected = write_executable(dir.path(), "widget");
        let env: EnvMap = [("PATH", dir.path().to_string_lossy().as_ref())]
            .into_iter()
            .collect();

        assert_eq!(
            find_executable("widget", &env, HostPlatform::Linux),
            expected.to_string_lossy()
        );
        assert_eq!(find_executable("absent", &env, HostPlatform::Linux), "");
    }

    #[test]
    fn a_non_executable_file_is_not_found_on_posix() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("widget");
        std::fs::write(&path, b"data").expect("write");
        let env: EnvMap = [("PATH", dir.path().to_string_lossy().as_ref())]
            .into_iter()
            .collect();

        // POSIX asks for X_OK; Windows only asks that the file exist.
        #[cfg(unix)]
        assert_eq!(find_executable("widget", &env, HostPlatform::Linux), "");
        assert_eq!(
            find_executable("widget", &env, HostPlatform::Windows),
            path.to_string_lossy()
        );
    }

    #[test]
    fn expands_a_tilde_against_home() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nested = dir.path().join("bundle");
        std::fs::create_dir_all(&nested).expect("mkdir");
        let expected = write_executable(&nested, "openclaw");
        let env: EnvMap = [("HOME", dir.path().to_string_lossy().as_ref())]
            .into_iter()
            .collect();

        assert_eq!(
            find_executable("~/bundle/openclaw", &env, HostPlatform::Linux),
            expected.to_string_lossy()
        );
    }

    #[test]
    fn a_path_entry_that_is_itself_the_executable_still_resolves() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("npm.cmd");
        std::fs::write(&path, b"@echo off").expect("write");
        let env: EnvMap = [
            ("PATH", path.to_string_lossy().into_owned()),
            ("PATHEXT", DEFAULT_PATHEXT.to_owned()),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            find_executable("npm.cmd", &env, HostPlatform::Windows),
            path.to_string_lossy()
        );
    }

    #[test]
    fn windows_appends_pathext_when_the_command_has_no_extension() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("npm.cmd");
        std::fs::write(&path, b"@echo off").expect("write");
        let env: EnvMap = [
            ("PATH", dir.path().to_string_lossy().into_owned()),
            ("PATHEXT", DEFAULT_PATHEXT.to_owned()),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            find_executable("npm", &env, HostPlatform::Windows),
            path.to_string_lossy()
        );
        // On POSIX the same name has no extension appended, so nothing matches.
        assert_eq!(find_executable("npm", &env, HostPlatform::Linux), "");
    }

    #[test]
    fn an_empty_command_is_not_a_lookup() {
        let env = EnvMap::new();
        assert_eq!(find_executable("", &env, HostPlatform::Linux), "");
        assert_eq!(find_executable("   ", &env, HostPlatform::Linux), "");
    }

    #[test]
    fn environment_value_reports_the_first_name_when_nothing_is_set() {
        let env: EnvMap = [("QODER_CLI_PATH", "/opt/qoder")].into_iter().collect();
        assert_eq!(
            environment_value(&env, &["QODERCLI_PATH", "QODER_CLI_PATH"]),
            ("QODER_CLI_PATH", "/opt/qoder".to_owned())
        );
        assert_eq!(
            environment_value(&EnvMap::new(), &["QODERCLI_PATH", "QODER_CLI_PATH"]),
            ("QODERCLI_PATH", String::new())
        );
    }

    #[test]
    fn the_missing_finder_never_finds_anything() {
        assert_eq!(MissingFinder.find("npm"), "");
    }
}
