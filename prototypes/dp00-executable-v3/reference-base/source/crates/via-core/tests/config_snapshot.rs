//! An executable inventory of every default.
//!
//! The snapshot below is the whole resolved configuration under an **empty
//! environment**, with the three host facts pinned so the result is identical
//! on every machine. `docs/reference/contracts.json` catalogues 75 default
//! values; upstream evaluates them at module load, scattered across four files,
//! and consequently has no test that can see them all at once.
//!
//! Changing any default changes this file, and the diff names the setting, its
//! old value and its new one. That is the point: a default is a contract, and a
//! contract should not be able to move without a reviewer seeing it move.
//!
//! Credentials serialize as `[REDACTED]` (they are empty here, so they render
//! as `""`), because [`via_core::Secret`] does not hand out its contents.

mod common;

use common::{env, overrides};
use via_core::config::{BackendSelection, GatewayOptions, resolve};

#[test]
fn the_resolved_defaults_are_an_inventory() {
    let config = resolve(&env(&[]), None, &overrides()).expect("an empty environment resolves");
    insta::assert_json_snapshot!("config_defaults", config);
}

#[test]
fn a_fully_configured_environment_resolves_end_to_end() {
    // Not a default inventory — a second snapshot proving that every override
    // path is wired, including the ones an empty environment never exercises:
    // the split data directory, the legacy two-name path chains, an external
    // backend, and a per-backend workspace override.
    let config = resolve(
        &env(&[
            ("VIA_CONFIG_DIR", "/run/via"),
            ("VIA_DATA_DIR", "/assets/via"),
            ("HOST", "0.0.0.0"),
            ("PORT", "8080"),
            ("DASHSCOPE_API_KEY", "sk-dashscope"),
            ("DASHSCOPE_WORKSPACE_ID", "ws-42"),
            ("VIA_REALTIME_MODEL", "qwen3.5-omni-plus-realtime"),
            ("VIA_OMNI_REALTIME_VOICE", "Ethan-custom"),
            ("VIA_ALLOWED_ORIGINS", "https://voice.example.com, "),
            ("VIA_IDENTITY_MODE", "browser"),
            ("VIA_PERSONAL_OWNER_ID", "user_shared"),
            ("AGENT_PROTOCOL", "openclaw"),
            ("OPENCLAW_BASE_URL", "wss://openclaw.example.com///"),
            ("VIA_BACKEND_MODEL", "bailian/qwen3.7-max"),
            ("VIA_BACKEND_PERMISSION_MODE", "FULL"),
            ("VIA_OPENCLAW_WORKSPACE", "work/openclaw"),
            ("ACP_ARGS", "[\"--acp\",\"--verbose\"]"),
            ("ACP_LABEL", "  "),
            ("VIA_USER_PROFILE_PATH", "profiles/USER.md"),
            ("VIA_FRONTEND_MEMORY_PATH", "profiles/MEMORY.md"),
            ("VIA_SLEEP_TIMEOUT_SECONDS", "90"),
            ("VIA_WAKE_WORD_ENABLED", "TRUE"),
            ("VIA_WAKE_WORD", "hey via"),
            ("VIA_COMPUTER_USE", "off"),
            ("VIA_LOCALE", "ko"),
        ]),
        None,
        &overrides(),
    )
    .expect("a well-formed environment resolves");
    insta::assert_json_snapshot!("config_configured", config);
}

#[test]
fn host_options_sit_above_the_environment_and_the_file() {
    let mut overrides = overrides();
    overrides.gateway = GatewayOptions {
        config_dir: Some("/override/config".into()),
        host: Some("192.0.2.10".to_owned()),
        port: Some(4242),
        backend: Some(BackendSelection::None),
        wake_word: Some(true),
        owner: Some("service".to_owned()),
        log_console: Some(false),
    };
    let config = resolve(
        &env(&[("HOST", "10.0.0.1"), ("AGENT_PROTOCOL", "openclaw")]),
        Some("PORT=9999\nVIA_GATEWAY_OWNER=cli\n"),
        &overrides,
    )
    .expect("overrides resolve");

    assert_eq!(config.host, "192.0.2.10");
    assert_eq!(config.port, 4242);
    assert_eq!(config.agent_protocol, "");
    assert!(config.wake_word_enabled);
    assert_eq!(config.gateway_owner, "service");
    assert_eq!(
        config.config_directory(),
        std::path::Path::new("/override/config")
    );
}
