//! `PATH` merging for spawned children.
//!
//! Ported from `shared/path-environment.mjs`. When VIA launches a backend agent
//! it may need to prepend the directory holding a user-selected executable, and
//! doing that by string concatenation gets two things wrong: the delimiter is
//! `;` on Windows and `:` everywhere else, and Windows path comparison is
//! case-insensitive, so `C:\Node` and `c:\node` are one entry, not two.
//!
//! The platform is a parameter rather than a `cfg`, exactly as upstream passes
//! `process.platform`, so both behaviours are testable on either host — and
//! `via-process` will need to project a *Windows* child environment while
//! running its tests on a developer's Mac.

/// Which platform's path conventions to apply.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Platform {
    /// `;` delimiter, case-insensitive comparison, `\` separators.
    Windows,
    /// `:` delimiter, case-sensitive comparison, `/` separators.
    #[default]
    Posix,
}

impl Platform {
    /// The platform this build runs on.
    #[must_use]
    pub fn host() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Posix
        }
    }

    /// The `PATH` delimiter.
    ///
    /// **External contract** — `shared/path-environment.mjs:3-5`.
    #[must_use]
    pub const fn delimiter(self) -> char {
        match self {
            Self::Windows => ';',
            Self::Posix => ':',
        }
    }
}

/// The directory holding `command`.
///
/// **External contract** — `shared/path-environment.mjs:7-11`. Windows uses
/// `path.win32.dirname` even when the host is not Windows, so a projected
/// Windows child environment is correct from any host.
#[must_use]
pub fn command_directory(command: &str, platform: Platform) -> String {
    match platform {
        Platform::Windows => win32_dirname(command),
        Platform::Posix => posix_dirname(command),
    }
}

/// Merge `incoming` into `current`, dropping duplicates.
///
/// **External contract** — `shared/path-environment.mjs:13-36`. Entries are
/// trimmed, empties dropped, and an incoming entry already present in `current`
/// is skipped — case-insensitively on Windows. `prepend` puts the survivors in
/// front, which is what makes a user-selected executable win over one on the
/// system `PATH`.
#[must_use]
pub fn merge_search_path(
    current: &str,
    incoming: &str,
    platform: Platform,
    prepend: bool,
) -> String {
    let delimiter = platform.delimiter();
    let key = |value: &str| match platform {
        Platform::Windows => value.to_lowercase(),
        Platform::Posix => value.to_owned(),
    };

    let current: Vec<String> = split_entries(current, delimiter);
    let mut known: Vec<String> = current.iter().map(|value| key(value)).collect();
    let mut kept: Vec<String> = Vec::new();
    for value in split_entries(incoming, delimiter) {
        let normalized = key(&value);
        if known.contains(&normalized) {
            continue;
        }
        known.push(normalized);
        kept.push(value);
    }

    let merged: Vec<String> = if prepend {
        kept.into_iter().chain(current).collect()
    } else {
        current.into_iter().chain(kept).collect()
    };
    merged.join(&delimiter.to_string())
}

fn split_entries(value: &str, delimiter: char) -> Vec<String> {
    value
        .split(delimiter)
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Node's `path.posix.dirname`.
fn posix_dirname(path: &str) -> String {
    dirname_with(path, &['/'])
}

/// Node's `path.win32.dirname`, which accepts both separators.
fn win32_dirname(path: &str) -> String {
    dirname_with(path, &['/', '\\'])
}

fn dirname_with(path: &str, separators: &[char]) -> String {
    if path.is_empty() {
        return ".".to_owned();
    }
    let trimmed = path.trim_end_matches(|c| separators.contains(&c));
    // A path that was nothing but separators is the root itself.
    if trimmed.is_empty() {
        return path[..1].to_owned();
    }
    match trimmed.rfind(|c| separators.contains(&c)) {
        None => ".".to_owned(),
        Some(0) => trimmed[..1].to_owned(),
        Some(index) => trimmed[..index].to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_posix_paths_with_colon_delimiters() {
        assert_eq!(
            merge_search_path(
                "/usr/bin:/bin",
                "/opt/homebrew/bin:/usr/bin",
                Platform::Posix,
                true
            ),
            "/opt/homebrew/bin:/usr/bin:/bin"
        );
        assert_eq!(
            command_directory("/opt/homebrew/bin/npm", Platform::Posix),
            "/opt/homebrew/bin"
        );
    }

    #[test]
    fn merges_windows_paths_case_insensitively() {
        assert_eq!(
            merge_search_path(
                "C:\\Windows;C:\\Node",
                "c:\\node;D:\\Tools",
                Platform::Windows,
                true
            ),
            "D:\\Tools;C:\\Windows;C:\\Node"
        );
        assert_eq!(
            command_directory("C:\\Node\\npm.cmd", Platform::Windows),
            "C:\\Node"
        );
    }

    #[test]
    fn can_append_instead_of_prepending() {
        assert_eq!(
            merge_search_path("/usr/bin:/bin", "/opt/node/bin", Platform::Posix, false),
            "/usr/bin:/bin:/opt/node/bin"
        );
        assert_eq!(
            merge_search_path("C:\\Windows;C:\\Node", "c:\\node", Platform::Windows, false),
            "C:\\Windows;C:\\Node"
        );
    }
}
