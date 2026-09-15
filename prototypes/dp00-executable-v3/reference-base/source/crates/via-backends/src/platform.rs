//! The three `process.platform` values the backend tables branch on.
//!
//! `via_core::search_path::Platform` is two-valued because path syntax has two
//! shapes. The backend tables need three: Hermes installs from a `curl | bash`
//! script on `darwin` and `linux` and from a PowerShell one-liner on `win32`
//! (`shared/backend-catalog.mjs:161-166`), and CodeBuddy's credential directory
//! differs on all three (`shared/backend-auth-status.mjs:120-140`).
//!
//! Like `via-core`'s, this is a parameter and never a `cfg`: upstream passes
//! `process.platform` explicitly through every one of these functions so the
//! Windows behaviour is testable from a developer's Mac, and the tests here do
//! exactly that.

use via_core::search_path::Platform;

/// A `process.platform` value.
///
/// Upstream's `['darwin', 'linux', 'win32'].includes(platform)`
/// (`shared/backend-onboarding.mjs:30`) is the closed set: anything else is an
/// unsupported platform, which is why there is no `Other` variant and why
/// [`HostPlatform::host`] answers `None` on, say, FreeBSD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostPlatform {
    /// macOS.
    Darwin,
    /// Linux.
    Linux,
    /// Windows — Node spells it `win32`.
    Windows,
}

impl HostPlatform {
    /// Every platform, in the order upstream lists them.
    pub const ALL: [Self; 3] = [Self::Darwin, Self::Linux, Self::Windows];

    /// The Node spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Darwin => "darwin",
            Self::Linux => "linux",
            Self::Windows => "win32",
        }
    }

    /// Parse a Node spelling.
    #[must_use]
    pub fn from_node(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|item| item.as_str() == value)
    }

    /// The platform this build runs on, or `None` when it is one upstream's
    /// closed set does not name.
    #[must_use]
    pub fn host() -> Option<Self> {
        if cfg!(target_os = "macos") {
            Some(Self::Darwin)
        } else if cfg!(target_os = "linux") {
            Some(Self::Linux)
        } else if cfg!(windows) {
            Some(Self::Windows)
        } else {
            None
        }
    }

    /// Whether path syntax follows Windows or POSIX rules.
    #[must_use]
    pub const fn paths(self) -> Platform {
        match self {
            Self::Windows => Platform::Windows,
            Self::Darwin | Self::Linux => Platform::Posix,
        }
    }

    /// Whether this is `win32`.
    ///
    /// The single predicate upstream writes as `platform === 'win32'` in
    /// eleven places; naming it once keeps the eleven from drifting.
    #[must_use]
    pub const fn is_windows(self) -> bool {
        matches!(self, Self::Windows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_node_spellings_round_trip() {
        for platform in HostPlatform::ALL {
            assert_eq!(HostPlatform::from_node(platform.as_str()), Some(platform));
        }
        assert_eq!(HostPlatform::from_node("windows"), None);
        assert_eq!(HostPlatform::from_node("freebsd"), None);
    }

    #[test]
    fn only_windows_uses_windows_path_syntax() {
        assert_eq!(HostPlatform::Windows.paths(), Platform::Windows);
        assert_eq!(HostPlatform::Darwin.paths(), Platform::Posix);
        assert_eq!(HostPlatform::Linux.paths(), Platform::Posix);
    }
}
