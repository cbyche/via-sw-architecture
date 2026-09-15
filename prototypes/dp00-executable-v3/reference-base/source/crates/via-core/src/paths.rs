//! Every directory and file the product creates, with its exact name.
//!
//! Upstream's `server/src/core/install-paths.mjs` resolves exactly one thing —
//! `web/dist` — while the real install surface lives in
//! `shared/runtime-environment.mjs`, `shared/logger.mjs` and
//! `shared/gateway-instance-lock.mjs`. `docs/reference/contracts.json` says so
//! in as many words ("install-paths.mjs itself only resolves web/dist; the real
//! install surface is runtime-environment + logger + lease"), so this module is
//! the union of all four.
//!
//! # The split that must not be got wrong
//!
//! There are **two** directories, and which file lives in which is a contract:
//!
//! | | Holds | Default |
//! | --- | --- | --- |
//! | config directory | runtime state: `gateway.lock`, `tasks.json`, `logs/`, `state/`, `backends/`, `models/` | `VIA_CONFIG_DIR` → `$XDG_CONFIG_HOME/via` → `~/.config/via` |
//! | data directory | user assets: `config.env`, `state.env`, `USER.md`, `MEMORY.md`, `ASSISTANT.md`, `frontend-notes.json`, `workspace/` | `VIA_DATA_DIR` → the config directory |
//!
//! They are identical by default. Upstream separates them so the desktop app
//! and the CLI can share one set of *assets* while keeping their runtime state
//! apart. `contracts.json` records the consequence of confusing them: "Getting
//! this split wrong silently loses user memories."
//!
//! # Rebrand
//!
//! The last path segment moves `~/.config/qwaudio` → `~/.config/via`, and the
//! two directory variables `QWAUDIO_CONFIG_DIR` / `QWAUDIO_DATA_DIR` become
//! `VIA_CONFIG_DIR` / `VIA_DATA_DIR` (`docs/rebrand.md`). **Every filename
//! inside those directories is unchanged** — `docs/rebrand.md` KEEPs them
//! explicitly, so the migration is a pure directory move.
//!
//! # Not ported
//!
//! `webDistributionPath()` (`server/src/core/install-paths.mjs:4-8`) resolves
//! the packaged React SPA. VIA drops `web/` (`docs/fidelity.md`, "Dropped, and
//! why") and does not advertise `web.same-origin-ui`, so there is no directory
//! to locate. Recorded in `docs/deviations/phase-1.md`.

use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::env::EnvMap;

/// Mode for every directory VIA creates.
///
/// **External contract** — `shared/runtime-environment.mjs:477` and every
/// `mkdirSync` beside it pass `{ mode: 0o700 }`. Applied on Unix only; on
/// Windows the directory inherits the parent ACL, as in `via-log` and
/// `via-store`.
pub const DIRECTORY_MODE: u32 = 0o700;

/// Mode for every file VIA creates.
///
/// **External contract** — `shared/runtime-environment.mjs:151` and peers pass
/// `{ mode: 0o600 }`.
pub const FILE_MODE: u32 = 0o600;

/// Environment variable naming the config directory.
///
/// Upstream `QWAUDIO_CONFIG_DIR` (`shared/runtime-environment.mjs:96`), renamed
/// per `docs/rebrand.md`. Shared with [`via_log::ENV_CONFIG_DIR`], which reads
/// the same variable to place `logs/`.
pub const ENV_CONFIG_DIR: &str = via_log::ENV_CONFIG_DIR;

/// Environment variable naming the data directory.
///
/// Upstream `QWAUDIO_DATA_DIR` (`shared/runtime-environment.mjs:111`).
pub const ENV_DATA_DIR: &str = "VIA_DATA_DIR";

/// The XDG base-directory variable consulted when [`ENV_CONFIG_DIR`] is unset.
pub const ENV_XDG_CONFIG_HOME: &str = via_log::ENV_XDG_CONFIG_HOME;

/// The last path segment of the default config directory.
///
/// Upstream `qwaudio` (`shared/runtime-environment.mjs:100`), renamed per
/// `docs/rebrand.md`; `~/.config/qwaudio` → `~/.config/via`.
pub const CONFIG_DIRECTORY_NAME: &str = via_log::CONFIG_DIRECTORY_NAME;

/// The `.config` segment under the home directory when `XDG_CONFIG_HOME` is
/// unset.
pub const XDG_CONFIG_FALLBACK_SEGMENT: &str = ".config";

/// User configuration file, in the **data** directory. KEEP.
pub const CONFIG_FILE_NAME: &str = "config.env";
/// Generated local-identity file, in the **data** directory. KEEP.
pub const STATE_FILE_NAME: &str = "state.env";
/// User personalization document, in the **data** directory. KEEP.
pub const USER_MODEL_FILE_NAME: &str = "USER.md";
/// Assistant persona document, in the **data** directory. KEEP.
pub const ASSISTANT_PROFILE_FILE_NAME: &str = "ASSISTANT.md";
/// Long-term memory document, in the **data** directory. KEEP.
pub const MEMORY_FILE_NAME: &str = "MEMORY.md";
/// Pre-Markdown memory store, in the **data** directory. Legacy; read only.
pub const LEGACY_FRONTEND_MEMORY_FILE_NAME: &str = "frontend-memory.json";
/// Notes store, in the **data** directory. KEEP.
pub const FRONTEND_NOTES_FILE_NAME: &str = "frontend-notes.json";
/// The one workspace every backend agent shares, in the **data** directory.
pub const SHARED_WORKSPACE_DIRECTORY_NAME: &str = "workspace";
/// Marker recording that the JSON→Markdown memory migration has run.
pub const MEMORY_MIGRATION_MARKER: &str = "state/frontend-memory-markdown-v1";

/// Gateway instance lease, in the **config** directory. Owned by `via-lock`.
pub const GATEWAY_LOCK_FILE_NAME: &str = "gateway.lock";
/// Work store, in the **config** directory. Owned by `via-work`.
pub const TASK_STATE_FILE_NAME: &str = "tasks.json";
/// Memory-extraction audit log, in the **config** directory.
pub const MEMORY_AUDIT_FILE_NAME: &str = "memory-audit.jsonl";
/// Concurrent-CLI detection lock, in the **config** directory.
pub const CLI_LOCK_FILE_NAME: &str = "cli.lock";
/// Installed-service metadata sidecar, in the **config** directory.
pub const GATEWAY_SERVICE_FILE_NAME: &str = "gateway-service.json";
/// Log directory, in the **config** directory. Owned by `via-log`.
pub const LOG_DIRECTORY_NAME: &str = via_log::LOG_DIRECTORY_NAME;
/// Gateway log file, inside [`LOG_DIRECTORY_NAME`].
pub const GATEWAY_LOG_FILE_NAME: &str = "gateway.log";
/// launchd/systemd console capture, inside [`LOG_DIRECTORY_NAME`].
pub const GATEWAY_CONSOLE_LOG_FILE_NAME: &str = "gateway-console.log";
/// ACP session index, in the **config** directory.
///
/// KEEP: `acp-sessions.json` names the protocol, not the product, and keeping
/// it lets VIA read an existing install's state file unchanged.
pub const BACKEND_SESSION_STATE_PATH: &str = "state/acp-sessions.json";
/// OpenClaw runtime state, in the **config** directory.
pub const OPENCLAW_STATE_DIRECTORY: &str = "backends/openclaw/state";
/// OpenClaw gateway token, inside [`OPENCLAW_STATE_DIRECTORY`].
pub const OPENCLAW_TOKEN_FILE_NAME: &str = "gateway-token";
/// Materialised OpenClaw bridge configuration, in the **config** directory.
pub const OPENCLAW_CONFIG_PATH: &str = "backends/openclaw/openclaw.json5";
/// DeepSeek Harness session root, in the **config** directory.
pub const DEEPSEEK_SESSION_ROOT: &str = "backends/deepseek-harness/sessions";
/// Wake-word model directory, in the **config** directory.
pub const WAKE_WORD_MODEL_DIRECTORY: &str = "models/wake-word";
/// Orb skin store, in the **config** directory. Present for on-disk
/// compatibility; VIA ships no skin store.
pub const SKINS_DIRECTORY_NAME: &str = "skins";
/// Parent of the pre-1.11 per-backend workspaces, in **both** directories.
///
/// Never migrated, only reported (`server/src/index.mjs:62-70`).
pub const LEGACY_WORKSPACES_DIRECTORY_NAME: &str = "workspaces";
/// Directory holding the assistant prompt assets, relative to the runtime root.
pub const FRONTEND_PROMPT_DIRECTORY: &str = "config/frontend-agent";
/// The immutable core policy prompt, inside [`FRONTEND_PROMPT_DIRECTORY`].
pub const PROMPT_FILE_NAME: &str = "PROMPT.md";
/// Directory holding per-owner memory shards, inside the data directory.
pub const OWNER_SHARD_DIRECTORY_NAME: &str = "users";
/// Hex characters of the SHA-256 owner digest used as a shard name.
///
/// **External contract** — `server/src/conversation/markdown-context-store.mjs:82-87`
/// (`sha256(ownerId).hex.slice(0, 16)`).
pub const OWNER_SHARD_HEX_LENGTH: usize = 16;

/// The two directories every persisted artifact lives under.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPaths {
    config_directory: PathBuf,
    data_directory: PathBuf,
}

impl InstallPaths {
    /// Resolve both directories from an environment.
    ///
    /// **External contract** — `shared/runtime-environment.mjs:92-113`:
    ///
    /// * config: `VIA_CONFIG_DIR` → `$XDG_CONFIG_HOME/via` → `<home>/.config/via`
    /// * data:   `VIA_DATA_DIR` → the config directory
    ///
    /// `home` and `working_directory` are parameters rather than lookups
    /// because Node's `os.homedir()` and `process.cwd()` are not environment
    /// reads; a pure resolver cannot invent them. `working_directory` is what a
    /// relative `VIA_CONFIG_DIR` is resolved against, matching Node's
    /// single-argument `path.resolve`.
    #[must_use]
    pub fn from_env(env: &EnvMap, home: &Path, working_directory: &Path) -> Self {
        let config_directory = match env.get_truthy(ENV_CONFIG_DIR) {
            Some(configured) => resolve_path(working_directory, configured),
            None => {
                let base = match env.get_truthy(ENV_XDG_CONFIG_HOME) {
                    Some(xdg) => resolve_path(working_directory, xdg),
                    None => resolve_path(home, XDG_CONFIG_FALLBACK_SEGMENT),
                };
                base.join(CONFIG_DIRECTORY_NAME)
            }
        };
        let data_directory = match env.get_truthy(ENV_DATA_DIR) {
            Some(configured) => resolve_path(working_directory, configured),
            None => config_directory.clone(),
        };
        Self {
            config_directory,
            data_directory,
        }
    }

    /// Build from two already-resolved directories.
    #[must_use]
    pub fn new(config_directory: PathBuf, data_directory: PathBuf) -> Self {
        Self {
            config_directory,
            data_directory,
        }
    }

    /// Runtime state lives here.
    #[must_use]
    pub fn config_directory(&self) -> &Path {
        &self.config_directory
    }

    /// User assets live here. Equal to the config directory unless
    /// `VIA_DATA_DIR` says otherwise.
    #[must_use]
    pub fn data_directory(&self) -> &Path {
        &self.data_directory
    }

    /// `<data>/config.env`
    #[must_use]
    pub fn config_file(&self) -> PathBuf {
        self.data_directory.join(CONFIG_FILE_NAME)
    }

    /// `<data>/state.env` — the generated auth secret.
    #[must_use]
    pub fn state_file(&self) -> PathBuf {
        self.data_directory.join(STATE_FILE_NAME)
    }

    /// `<data>/USER.md`
    #[must_use]
    pub fn user_model_file(&self) -> PathBuf {
        self.data_directory.join(USER_MODEL_FILE_NAME)
    }

    /// `<data>/ASSISTANT.md`
    #[must_use]
    pub fn assistant_profile_file(&self) -> PathBuf {
        self.data_directory.join(ASSISTANT_PROFILE_FILE_NAME)
    }

    /// `<data>/MEMORY.md`
    #[must_use]
    pub fn memory_file(&self) -> PathBuf {
        self.data_directory.join(MEMORY_FILE_NAME)
    }

    /// `<data>/frontend-memory.json` — legacy, read once and migrated.
    #[must_use]
    pub fn legacy_frontend_memory_file(&self) -> PathBuf {
        self.data_directory.join(LEGACY_FRONTEND_MEMORY_FILE_NAME)
    }

    /// `<data>/frontend-notes.json`
    #[must_use]
    pub fn frontend_notes_file(&self) -> PathBuf {
        self.data_directory.join(FRONTEND_NOTES_FILE_NAME)
    }

    /// `<data>/workspace` — the workspace every backend agent shares.
    ///
    /// **External contract** — `shared/runtime-environment.mjs:286-288`. The
    /// pre-1.11 per-backend directories are reported, never migrated.
    #[must_use]
    pub fn shared_workspace(&self) -> PathBuf {
        self.data_directory.join(SHARED_WORKSPACE_DIRECTORY_NAME)
    }

    /// `<data>/state/frontend-memory-markdown-v1`
    #[must_use]
    pub fn memory_migration_marker(&self) -> PathBuf {
        join_relative(&self.data_directory, MEMORY_MIGRATION_MARKER)
    }

    /// `<config>/gateway.lock`
    #[must_use]
    pub fn gateway_lock_file(&self) -> PathBuf {
        self.config_directory.join(GATEWAY_LOCK_FILE_NAME)
    }

    /// `<config>/tasks.json`
    #[must_use]
    pub fn task_state_file(&self) -> PathBuf {
        self.config_directory.join(TASK_STATE_FILE_NAME)
    }

    /// `<config>/memory-audit.jsonl`
    #[must_use]
    pub fn memory_audit_file(&self) -> PathBuf {
        self.config_directory.join(MEMORY_AUDIT_FILE_NAME)
    }

    /// `<config>/cli.lock`
    #[must_use]
    pub fn cli_lock_file(&self) -> PathBuf {
        self.config_directory.join(CLI_LOCK_FILE_NAME)
    }

    /// `<config>/gateway-service.json`
    #[must_use]
    pub fn gateway_service_file(&self) -> PathBuf {
        self.config_directory.join(GATEWAY_SERVICE_FILE_NAME)
    }

    /// `<config>/logs`
    #[must_use]
    pub fn log_directory(&self) -> PathBuf {
        self.config_directory.join(LOG_DIRECTORY_NAME)
    }

    /// `<config>/logs/gateway.log`
    #[must_use]
    pub fn gateway_log_file(&self) -> PathBuf {
        self.log_directory().join(GATEWAY_LOG_FILE_NAME)
    }

    /// `<config>/logs/gateway-console.log`
    #[must_use]
    pub fn gateway_console_log_file(&self) -> PathBuf {
        self.log_directory().join(GATEWAY_CONSOLE_LOG_FILE_NAME)
    }

    /// `<config>/state/acp-sessions.json`
    #[must_use]
    pub fn backend_session_state_file(&self) -> PathBuf {
        join_relative(&self.config_directory, BACKEND_SESSION_STATE_PATH)
    }

    /// `<config>/backends/openclaw/state`
    #[must_use]
    pub fn openclaw_state_directory(&self) -> PathBuf {
        join_relative(&self.config_directory, OPENCLAW_STATE_DIRECTORY)
    }

    /// `<config>/backends/openclaw/state/gateway-token`
    #[must_use]
    pub fn openclaw_token_file(&self) -> PathBuf {
        self.openclaw_state_directory()
            .join(OPENCLAW_TOKEN_FILE_NAME)
    }

    /// `<config>/backends/openclaw/openclaw.json5`
    #[must_use]
    pub fn openclaw_config_file(&self) -> PathBuf {
        join_relative(&self.config_directory, OPENCLAW_CONFIG_PATH)
    }

    /// `<config>/backends/deepseek-harness/sessions`
    #[must_use]
    pub fn deepseek_session_root(&self) -> PathBuf {
        join_relative(&self.config_directory, DEEPSEEK_SESSION_ROOT)
    }

    /// `<config>/models/wake-word`
    #[must_use]
    pub fn wake_word_model_directory(&self) -> PathBuf {
        join_relative(&self.config_directory, WAKE_WORD_MODEL_DIRECTORY)
    }

    /// `<config>/skins`
    #[must_use]
    pub fn skins_directory(&self) -> PathBuf {
        self.config_directory.join(SKINS_DIRECTORY_NAME)
    }

    /// `<directory>/workspaces/<backend_id>` — the pre-1.11 layout.
    ///
    /// Upstream looks for it under **both** directories, because a desktop
    /// install that split them may have left files in either
    /// (`shared/runtime-environment.mjs:541-546`).
    #[must_use]
    pub fn legacy_backend_workspaces(&self, backend_id: &str) -> Vec<PathBuf> {
        let mut directories = vec![
            self.config_directory
                .join(LEGACY_WORKSPACES_DIRECTORY_NAME)
                .join(backend_id),
        ];
        if self.data_directory != self.config_directory {
            directories.push(
                self.data_directory
                    .join(LEGACY_WORKSPACES_DIRECTORY_NAME)
                    .join(backend_id),
            );
        }
        directories
    }
}

/// Where a non-personal owner's copy of a document lives.
///
/// **External contract** — `server/src/conversation/markdown-context-store.mjs:82-87`:
/// `join(dirname(path), 'users', sha256(ownerId).hex.slice(0, 16), basename(path))`.
/// The personal owner (`VIA_PERSONAL_OWNER_ID`, default `user_personal`) uses
/// the unsharded path, which is why this function is not applied to it.
#[must_use]
pub fn owner_sharded_path(path: &Path, owner_id: &str) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = path.file_name().unwrap_or_default();
    parent
        .join(OWNER_SHARD_DIRECTORY_NAME)
        .join(owner_shard(owner_id))
        .join(file_name)
}

/// The 16-hex-character shard name for an owner id.
#[must_use]
pub fn owner_shard(owner_id: &str) -> String {
    let digest = Sha256::digest(owner_id.as_bytes());
    let mut hex = String::with_capacity(OWNER_SHARD_HEX_LENGTH);
    for byte in digest.iter().take(OWNER_SHARD_HEX_LENGTH / 2) {
        use std::fmt::Write as _;
        // Writing to a `String` cannot fail; the result is discarded rather
        // than unwrapped so this stays free of `expect` in non-test code.
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// Node's two-argument `path.resolve(base, value)`.
///
/// An absolute `value` replaces `base` entirely; a relative one is appended.
/// The result is normalised **lexically** — `.` and `..` are folded without
/// touching the filesystem, as Node does — so a configured
/// `VIA_TASK_STATE_PATH=../tasks.json` lands where upstream puts it.
#[must_use]
pub fn resolve_path(base: &Path, value: &str) -> PathBuf {
    normalize(&base.join(value))
}

/// Join a `/`-separated relative contract path onto a directory.
///
/// The constants in this module spell nested paths with `/` because that is how
/// upstream writes them; this converts each segment so Windows gets a native
/// path rather than a single segment containing slashes.
#[must_use]
fn join_relative(base: &Path, relative: &str) -> PathBuf {
    let mut path = base.to_path_buf();
    for segment in relative.split('/').filter(|s| !s.is_empty()) {
        path.push(segment);
    }
    path
}

/// Fold `.` and `..` without consulting the filesystem.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    #[test]
    fn defaults_to_the_rebranded_xdg_directory() {
        let paths = InstallPaths::from_env(&env(&[]), Path::new("/home/via"), Path::new("/cwd"));
        assert_eq!(paths.config_directory(), Path::new("/home/via/.config/via"));
        assert_eq!(paths.data_directory(), paths.config_directory());
    }

    #[test]
    fn xdg_config_home_replaces_only_the_base() {
        let paths = InstallPaths::from_env(
            &env(&[("XDG_CONFIG_HOME", "/xdg")]),
            Path::new("/home/via"),
            Path::new("/cwd"),
        );
        assert_eq!(paths.config_directory(), Path::new("/xdg/via"));
    }

    #[test]
    fn a_relative_config_dir_resolves_against_the_working_directory() {
        let paths = InstallPaths::from_env(
            &env(&[("VIA_CONFIG_DIR", "state/../via-state")]),
            Path::new("/home/via"),
            Path::new("/cwd"),
        );
        assert_eq!(paths.config_directory(), Path::new("/cwd/via-state"));
    }

    #[test]
    fn the_data_directory_can_be_split_from_the_config_directory() {
        let paths = InstallPaths::from_env(
            &env(&[("VIA_CONFIG_DIR", "/run"), ("VIA_DATA_DIR", "/assets")]),
            Path::new("/home/via"),
            Path::new("/cwd"),
        );
        assert_eq!(paths.task_state_file(), Path::new("/run/tasks.json"));
        assert_eq!(paths.config_file(), Path::new("/assets/config.env"));
        assert_eq!(paths.memory_file(), Path::new("/assets/MEMORY.md"));
        assert_eq!(paths.gateway_lock_file(), Path::new("/run/gateway.lock"));
    }

    #[test]
    fn owner_shards_are_sixteen_hex_characters() {
        let shard = owner_shard("user_abc");
        assert_eq!(shard.len(), OWNER_SHARD_HEX_LENGTH);
        assert!(shard.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(
            owner_sharded_path(Path::new("/data/MEMORY.md"), "user_abc"),
            Path::new("/data/users").join(&shard).join("MEMORY.md"),
        );
    }
}
