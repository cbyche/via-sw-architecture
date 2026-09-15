//! Install paths: resolution order, the config/data split, and the shard rule.

mod common;

use std::path::Path;

use common::env;
use pretty_assertions::assert_eq;
use via_core::config::names;
use via_core::paths::{
    CONFIG_DIRECTORY_NAME, InstallPaths, owner_shard, owner_sharded_path, resolve_path,
};
use via_core::search_path::{Platform, command_directory, merge_search_path};

const HOME: &str = "/home/via";
const CWD: &str = "/srv/via";

fn paths(pairs: &[(&str, &str)]) -> InstallPaths {
    InstallPaths::from_env(&env(pairs), Path::new(HOME), Path::new(CWD))
}

#[test]
fn the_config_directory_resolution_order_is_three_deep() {
    assert_eq!(
        paths(&[]).config_directory(),
        Path::new("/home/via/.config/via")
    );
    assert_eq!(
        paths(&[("XDG_CONFIG_HOME", "/xdg")]).config_directory(),
        Path::new("/xdg/via")
    );
    assert_eq!(
        paths(&[
            ("XDG_CONFIG_HOME", "/xdg"),
            (names::CONFIG_DIR, "/explicit")
        ])
        .config_directory(),
        Path::new("/explicit")
    );
    // The last segment is the rebranded one.
    assert_eq!(CONFIG_DIRECTORY_NAME, "via");
    assert!(
        !paths(&[])
            .config_directory()
            .to_string_lossy()
            .contains("qwaudio")
    );
}

#[test]
fn an_empty_directory_variable_is_treated_as_unset() {
    assert_eq!(
        paths(&[(names::CONFIG_DIR, "")]).config_directory(),
        Path::new("/home/via/.config/via")
    );
    assert_eq!(
        paths(&[(names::DATA_DIR, "")]).data_directory(),
        Path::new("/home/via/.config/via")
    );
}

#[test]
fn the_data_directory_defaults_to_the_config_directory() {
    let shared = paths(&[]);
    assert_eq!(shared.data_directory(), shared.config_directory());

    let split = paths(&[(names::CONFIG_DIR, "/run"), (names::DATA_DIR, "/assets")]);
    assert_eq!(split.config_directory(), Path::new("/run"));
    assert_eq!(split.data_directory(), Path::new("/assets"));
}

#[test]
fn every_file_lands_in_the_directory_that_owns_it() {
    let paths = InstallPaths::new("/run".into(), "/assets".into());

    for path in [
        paths.config_file(),
        paths.state_file(),
        paths.user_model_file(),
        paths.assistant_profile_file(),
        paths.memory_file(),
        paths.legacy_frontend_memory_file(),
        paths.frontend_notes_file(),
        paths.shared_workspace(),
        paths.memory_migration_marker(),
    ] {
        assert!(
            path.starts_with("/assets"),
            "{} is an asset and belongs in the data directory",
            path.display()
        );
    }

    for path in [
        paths.gateway_lock_file(),
        paths.task_state_file(),
        paths.memory_audit_file(),
        paths.cli_lock_file(),
        paths.gateway_service_file(),
        paths.log_directory(),
        paths.gateway_log_file(),
        paths.gateway_console_log_file(),
        paths.backend_session_state_file(),
        paths.openclaw_state_directory(),
        paths.openclaw_token_file(),
        paths.openclaw_config_file(),
        paths.deepseek_session_root(),
        paths.wake_word_model_directory(),
        paths.skins_directory(),
    ] {
        assert!(
            path.starts_with("/run"),
            "{} is runtime state and belongs in the config directory",
            path.display()
        );
    }
}

#[test]
fn nested_contract_paths_become_real_path_segments() {
    let paths = InstallPaths::new("/run".into(), "/assets".into());
    assert_eq!(
        paths.backend_session_state_file(),
        Path::new("/run").join("state").join("acp-sessions.json")
    );
    assert_eq!(
        paths.openclaw_token_file(),
        Path::new("/run")
            .join("backends")
            .join("openclaw")
            .join("state")
            .join("gateway-token")
    );
    assert_eq!(
        paths.gateway_log_file().file_name().map(|n| n.to_str()),
        Some(Some("gateway.log"))
    );
}

#[test]
fn the_legacy_workspace_directories_are_searched_in_both_directories() {
    let shared = InstallPaths::new("/one".into(), "/one".into());
    assert_eq!(shared.legacy_backend_workspaces("opencode").len(), 1);

    let split = InstallPaths::new("/run".into(), "/assets".into());
    let both = split.legacy_backend_workspaces("opencode");
    assert_eq!(both.len(), 2);
    assert!(both.contains(&Path::new("/run/workspaces/opencode").to_path_buf()));
    assert!(both.contains(&Path::new("/assets/workspaces/opencode").to_path_buf()));
}

#[test]
fn resolve_path_folds_dot_segments_lexically() {
    assert_eq!(
        resolve_path(Path::new("/base"), "a/../b/./c"),
        Path::new("/base/b/c")
    );
    assert_eq!(
        resolve_path(Path::new("/base"), "/absolute"),
        Path::new("/absolute")
    );
    assert_eq!(resolve_path(Path::new("/base"), ".."), Path::new("/"));
}

#[test]
fn owner_sharding_is_sixteen_hex_characters_of_sha256() {
    let shard = owner_shard("user_abc");
    assert_eq!(shard.len(), 16);
    assert!(
        shard
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
    );
    // Stable across calls and specific to the owner.
    assert_eq!(shard, owner_shard("user_abc"));
    assert_ne!(shard, owner_shard("user_abd"));
    assert_eq!(
        owner_sharded_path(Path::new("/assets/MEMORY.md"), "user_abc"),
        Path::new("/assets/users").join(&shard).join("MEMORY.md")
    );
    // The known digest of "user_abc", truncated — computed independently of
    // the implementation.
    assert_eq!(shard, {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(b"user_abc");
        digest
            .iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    });
}

#[test]
fn search_paths_merge_per_platform() {
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
        merge_search_path(
            "C:\\Windows;C:\\Node",
            "c:\\node;D:\\Tools",
            Platform::Windows,
            true
        ),
        "D:\\Tools;C:\\Windows;C:\\Node"
    );
    // POSIX comparison is case-sensitive, so these are two entries.
    assert_eq!(
        merge_search_path("/usr/Bin", "/usr/bin", Platform::Posix, true),
        "/usr/bin:/usr/Bin"
    );
    assert_eq!(
        command_directory("/opt/node/bin/npm", Platform::Posix),
        "/opt/node/bin"
    );
    assert_eq!(command_directory("npm", Platform::Posix), ".");
    assert_eq!(command_directory("", Platform::Posix), ".");
    assert_eq!(
        command_directory("C:\\Node\\npm.cmd", Platform::Windows),
        "C:\\Node"
    );
    // Windows accepts either separator.
    assert_eq!(
        command_directory("C:/Node/npm.cmd", Platform::Windows),
        "C:/Node"
    );
}
