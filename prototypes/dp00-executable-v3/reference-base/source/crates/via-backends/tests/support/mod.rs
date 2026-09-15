//! Shared fixtures for the integration tests.
//!
//! Every test here builds a real profile from a real configuration: the point of
//! this crate is the table, and a test that stubbed the table would assert
//! nothing.

// Each integration test binary compiles this module separately, so a helper
// only one of them uses reads as dead code in the others.
#![allow(dead_code)]

use std::path::PathBuf;

use via_backends::detect::{ExecutableFinder, MissingFinder};
use via_backends::{BackendProfile, BackendsError, LaunchContext, create_backend_profile};
use via_catalog::Ownership;
use via_core::{Config, EnvMap, Secret};

/// The installation root every test resolves relative paths against.
pub const ROOT: &str = "/opt/via";

/// A finder that answers a fixed table, so a driver's "where is the parent CLI"
/// lookup is deterministic.
#[derive(Debug, Default)]
pub struct TableFinder(pub Vec<(String, String)>);

impl TableFinder {
    pub fn new(pairs: &[(&str, &str)]) -> Self {
        Self(
            pairs
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
        )
    }
}

impl ExecutableFinder for TableFinder {
    fn find(&self, command: &str) -> String {
        self.0
            .iter()
            .find(|(name, _)| name == command)
            .map(|(_, path)| path.clone())
            .unwrap_or_default()
    }
}

/// A configuration with every backend's working directory set, so a profile's
/// `cwd` is observable.
#[must_use]
pub fn config() -> Config {
    let mut config = Config {
        root: PathBuf::from(ROOT),
        agent_timeout_ms: 120_000,
        ..Config::default()
    };
    let work = PathBuf::from("/work");
    config.backends.opencode.directory = work.clone();
    config.backends.opencode.base_url = "http://127.0.0.1:4096".to_owned();
    config.backends.openclaw.directory = work.clone();
    config.backends.openclaw.base_url = "http://127.0.0.1:18789".to_owned();
    config.backends.openclaw.token_file =
        PathBuf::from("/config/backends/openclaw/state/gateway-token");
    config.backends.qoder.directory = work.clone();
    config.backends.qwen.directory = work.clone();
    config.backends.kimi.directory = work.clone();
    config.backends.hermes.directory = work.clone();
    config.backends.codebuddy.directory = work.clone();
    config.backends.codex.directory = work.clone();
    config.backends.claude.directory = work.clone();
    config.backends.deepseek.directory = work.clone();
    config.backends.deepseek.session_root =
        PathBuf::from("/config/backends/deepseek-harness/sessions");
    config.backends.pi.directory = work.clone();
    config.backends.acp.directory = work;
    config
}

/// An OpenClaw configuration with a token and a direct bridge binary.
#[must_use]
pub fn openclaw_config(cli_path: &str, token: &str) -> Config {
    let mut config = config();
    config.backends.openclaw.cli_path = cli_path.to_owned();
    config.backends.openclaw.token = Secret::new(token);
    config
}

/// Build one backend's profile.
///
/// # Errors
///
/// Whatever the driver refuses with.
pub fn profile(
    id: &str,
    config: &Config,
    env: &EnvMap,
    permission_mode: &str,
) -> Result<BackendProfile, BackendsError> {
    profile_with(
        id,
        config,
        env,
        permission_mode,
        Ownership::Owned,
        &MissingFinder,
    )
}

/// Build one backend's profile with every knob supplied.
///
/// # Errors
///
/// Whatever the driver refuses with.
pub fn profile_with(
    id: &str,
    config: &Config,
    env: &EnvMap,
    permission_mode: &str,
    ownership: Ownership,
    finder: &dyn ExecutableFinder,
) -> Result<BackendProfile, BackendsError> {
    create_backend_profile(
        id,
        &LaunchContext {
            config,
            env,
            ownership,
            permission_mode,
            owner_id: "user_personal",
            finder,
        },
    )
}

/// An `EnvMap` from pairs.
#[must_use]
pub fn env(pairs: &[(&str, &str)]) -> EnvMap {
    pairs.iter().copied().collect()
}
