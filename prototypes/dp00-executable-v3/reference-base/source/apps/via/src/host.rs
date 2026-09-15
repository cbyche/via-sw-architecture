//! The process facts every command starts from.
//!
//! `via-core`'s resolver is a pure function: it reads no environment, no
//! filesystem and no clock, and the two host facts it cannot invent — the home
//! directory and the working directory — arrive as parameters. Something has
//! to be impure, and this is it. [`Host::from_process`] is the *only* place in
//! the binary that samples the real environment.
//!
//! That is not tidiness for its own sake. `std::env::set_var` is `unsafe` in
//! edition 2024, so a test that wanted to exercise a command against a
//! synthetic environment would otherwise have to either use `unsafe` or
//! serialize against every other test in the binary. With the sample taken
//! once and threaded through, `Host::new(env, home, cwd)` builds any
//! environment a test likes and the tests run in parallel.

use std::path::{Path, PathBuf};

use via_core::config::{GatewayOptions, Overrides, names};
use via_core::runtime::{RuntimeEnvironment, RuntimeOptions, load_runtime_environment};
use via_core::{Config, EnvMap, InstallPaths, paths, resolve};
use via_i18n::{Locale, resolve_locale};

use crate::error::CliError;

/// One process's view of the world.
#[derive(Clone, Debug)]
pub struct Host {
    env: EnvMap,
    home_directory: PathBuf,
    working_directory: PathBuf,
    locale: Locale,
}

impl Host {
    /// Sample the real process.
    ///
    /// `dirs::home_dir()` and `std::env::current_dir()` fall back to `.`,
    /// which is the one branch Node cannot reach — `os.homedir()` throws
    /// instead. `via-core`'s `Overrides::from_process` makes the same choice
    /// and this defers to it rather than repeating the fallbacks.
    #[must_use]
    pub fn from_process() -> Self {
        let overrides = Overrides::from_process();
        Self::new(
            EnvMap::from_process(),
            overrides.home_directory,
            overrides.working_directory,
        )
    }

    /// Build a host from explicit facts.
    ///
    /// The locale is resolved immediately, from `env` alone: `VIA_LOCALE` →
    /// OS locale → `en` (`docs/architecture.md` §16). It is resolved *again*
    /// by [`Self::load_runtime`], because `config.env` may define
    /// `VIA_LOCALE` and upstream's `config.env` is loaded into the process
    /// environment before anything reads it. The first answer is what a
    /// failure *before* the file is read is rendered in.
    #[must_use]
    pub fn new(env: EnvMap, home_directory: PathBuf, working_directory: PathBuf) -> Self {
        let locale = resolve_locale(&env.reader());
        Self {
            env,
            home_directory,
            working_directory,
            locale,
        }
    }

    /// The environment as it currently stands.
    #[must_use]
    pub fn env(&self) -> &EnvMap {
        &self.env
    }

    /// The locale every message is rendered in.
    #[must_use]
    pub const fn locale(&self) -> Locale {
        self.locale
    }

    /// The OS home directory.
    #[must_use]
    pub fn home_directory(&self) -> &Path {
        &self.home_directory
    }

    /// The process working directory.
    #[must_use]
    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    /// The configuration and data directories this environment resolves to.
    #[must_use]
    pub fn install_paths(&self) -> InstallPaths {
        InstallPaths::from_env(&self.env, &self.home_directory, &self.working_directory)
    }

    /// The installation root — upstream's `sourceRoot`.
    ///
    /// **External contract** — `server/src/core/config.mjs:19`,
    /// `VIA_RUNTIME_ROOT` overriding it. This has to be answered *before*
    /// [`load_runtime_environment`] runs, because `.env.local` and `.env` are
    /// looked for inside it, and `via-core`'s resolver answers it only
    /// *after*. The duplication is deliberate and narrow, and
    /// `tests/host.rs::the_runtime_root_agrees_with_via_cores` asserts the two
    /// answers are the same for every shape of input.
    #[must_use]
    pub fn runtime_root(&self) -> PathBuf {
        match self.env.get_truthy(names::RUNTIME_ROOT) {
            Some(configured) => paths::resolve_path(&self.working_directory, configured),
            None => self.working_directory.clone(),
        }
    }

    /// Load `.env.local`, `.env` and `config.env` into the environment, and
    /// scaffold what is missing.
    ///
    /// **External contract** — `shared/runtime-environment.mjs`, through
    /// `via-core`'s port of it. `read_only` is upstream's own flag
    /// (`cli/src/launcher.mjs:185-187`): a command that only *inspects* a
    /// machine must not create directories on it.
    ///
    /// The locale is re-resolved afterwards, because the files just merged may
    /// have defined `VIA_LOCALE`.
    ///
    /// # Errors
    ///
    /// [`CliError::Core`] for any filesystem failure the loader reports.
    pub fn load_runtime(&mut self, read_only: bool) -> Result<RuntimeEnvironment, CliError> {
        let options = RuntimeOptions {
            root: self.runtime_root(),
            home_directory: self.home_directory.clone(),
            working_directory: self.working_directory.clone(),
            // The templates are seeded in the locale resolved so far. On a
            // first run there is no `config.env` to have said otherwise, so
            // the two answers cannot disagree there; on a later run the file
            // already exists and is not re-seeded.
            locale: self.locale,
            generate_secret: !read_only,
            read_only,
        };
        let environment = load_runtime_environment(&mut self.env, &options)?;
        self.locale = resolve_locale(&self.env.reader());
        Ok(environment)
    }

    /// Overlay host-facing Gateway options onto the environment.
    ///
    /// **External contract** — `shared/gateway-options.mjs:14-41`, through
    /// `via-core`'s `GatewayOptions::to_environment`. Applied here rather than
    /// inside [`Self::resolve_config`] so that the setup gate, which reads the
    /// environment directly, sees exactly what the resolver sees.
    pub fn apply_gateway_options(&mut self, options: &GatewayOptions) {
        self.env.overlay(&options.to_environment());
    }

    /// Set one environment entry.
    ///
    /// Used for the values `GatewayOptions` has no field for —
    /// `VIA_BACKEND_OWNERSHIP`, `VIA_BACKEND_PERMISSION_MODE`,
    /// `VIA_BACKEND_AGENT` and a backend's `*_BASE_URL`
    /// (`cli/src/launcher.mjs:55-86`).
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.set(key, value);
    }

    /// Remove one environment entry.
    ///
    /// **External contract** — `cli/src/launcher.mjs:75-81` deletes three keys
    /// when no backend is selected. Deleting is not the same as setting an
    /// empty value: `AGENT_PROTOCOL` is deliberately set to `""` on the same
    /// path, because deleting *that* one would let the file layer restore a
    /// backend the user asked to disable.
    pub fn remove_env(&mut self, key: &str) {
        self.env.remove(key);
    }

    /// Resolve the configuration from the environment as it now stands.
    ///
    /// `file` is `None`: [`Self::load_runtime`] has already merged
    /// `config.env` into the environment, exactly as upstream merges it into
    /// `process.env`, so passing it again would apply the lowest layer twice.
    ///
    /// # Errors
    ///
    /// [`CliError::Core`] for a malformed configuration — an unsupported
    /// backend or realtime provider, an unknown realtime model id, an invalid
    /// permission mode, or unparseable `ACP_ARGS`.
    pub fn resolve_config(&self) -> Result<Config, CliError> {
        let overrides = Overrides {
            home_directory: self.home_directory.clone(),
            working_directory: self.working_directory.clone(),
            runtime_root: Some(self.runtime_root()),
            // Already overlaid onto `self.env`; applying them again would be
            // a second, silent layer.
            gateway: GatewayOptions::default(),
        };
        Ok(resolve(&self.env, None, &overrides)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(pairs: &[(&str, &str)]) -> Host {
        Host::new(
            pairs.iter().copied().collect(),
            PathBuf::from("/home/via"),
            PathBuf::from("/srv/via"),
        )
    }

    #[test]
    fn the_locale_defaults_to_english() {
        assert_eq!(host(&[]).locale(), Locale::En);
    }

    #[test]
    fn an_explicit_locale_wins_over_the_os_one() {
        assert_eq!(
            host(&[("VIA_LOCALE", "ko"), ("LANG", "zh_CN.UTF-8")]).locale(),
            Locale::Ko
        );
        assert_eq!(host(&[("LANG", "zh_CN.UTF-8")]).locale(), Locale::Zh);
    }

    #[test]
    fn the_runtime_root_is_the_working_directory_unless_overridden() {
        assert_eq!(host(&[]).runtime_root(), PathBuf::from("/srv/via"));
        assert_eq!(
            host(&[("VIA_RUNTIME_ROOT", "/opt/via")]).runtime_root(),
            PathBuf::from("/opt/via")
        );
        // A relative override anchors to the working directory, matching
        // Node's single-argument `path.resolve`.
        assert_eq!(
            host(&[("VIA_RUNTIME_ROOT", "runtime")]).runtime_root(),
            PathBuf::from("/srv/via/runtime")
        );
    }

    #[test]
    fn the_runtime_root_agrees_with_via_cores() {
        for pairs in [
            [("DASHSCOPE_API_KEY", "k")].as_slice(),
            &[("DASHSCOPE_API_KEY", "k"), ("VIA_RUNTIME_ROOT", "/opt/via")],
            &[("DASHSCOPE_API_KEY", "k"), ("VIA_RUNTIME_ROOT", "runtime")],
            &[("DASHSCOPE_API_KEY", "k"), ("VIA_RUNTIME_ROOT", "")],
        ] {
            let host = host(pairs);
            let config = host.resolve_config().expect("a resolvable configuration");
            assert_eq!(
                config.root,
                host.runtime_root(),
                "the two answers drifted for {pairs:?}"
            );
        }
    }

    #[test]
    fn gateway_options_reach_the_resolved_configuration() {
        let mut host = host(&[("DASHSCOPE_API_KEY", "k")]);
        host.apply_gateway_options(&GatewayOptions {
            host: Some("0.0.0.0".to_owned()),
            port: Some(9000),
            ..GatewayOptions::default()
        });
        let config = host.resolve_config().expect("resolvable");
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9000);
    }

    #[test]
    fn removing_a_key_is_not_the_same_as_blanking_it() {
        let mut host = host(&[("VIA_BACKEND_AGENT", "build")]);
        host.set_env("AGENT_PROTOCOL", "");
        host.remove_env("VIA_BACKEND_AGENT");
        assert_eq!(host.env().get("AGENT_PROTOCOL"), Some(""));
        assert_eq!(host.env().get("VIA_BACKEND_AGENT"), None);
    }
}
