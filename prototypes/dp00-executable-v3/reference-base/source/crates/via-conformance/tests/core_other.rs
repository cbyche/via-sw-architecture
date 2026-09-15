//! `via-core` contracts that are not environment variables.
//!
//! `tests/core_environment.rs` (a separate agent's file) covers the `env-var`
//! rows this crate owns. This file covers the rest of what was left
//! `Pending` for `Crate::ViaCore`: default values, an error code, file paths,
//! two HTTP-route contracts, and three JSON-field shapes.
//!
//! As everywhere else in this crate, nothing here retypes a catalogue value —
//! every literal is either pulled out of the catalogued `exactValue` with
//! `via_conformance::value` helpers, or read out of `via-core`'s own public
//! constants and functions, and the two are compared against each other.

use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;
use via_conformance::expect_contract;
use via_conformance::value::rebranded;
use via_core::config::backend::VIA_BACKEND_AGENT_ID;
use via_core::config::{
    BackendModels, Config, GatewayOptions, Overrides, names, resolve, resolve_backend_models,
    resolve_opencode_coordinator_agent,
};
use via_core::env::EnvMap;
use via_core::identity::{IdentityManager, IdentityMode};
use via_core::paths::InstallPaths;
use via_core::security::is_allowed_origin;
use via_core::setup::{FIELD_DASHSCOPE_API_KEY, FIELD_SPEECH_TO_SPEECH_URL, gateway_setup_status};
use via_i18n::{Locale, keys, t};

fn env(pairs: &[(&str, &str)]) -> EnvMap {
    pairs.iter().copied().collect()
}

/// The first integer written after `marker` in `value`.
///
/// Mirrors `tests/voice_constants.rs::number_after`: the catalogue spells
/// bounds as prose (`resultContextMaxChars 6000 (min 256)`), so the number is
/// found by the name beside it rather than by position.
///
/// # Panics
///
/// When `marker` is absent, or no digits follow it — both mean the catalogued
/// text was reworded and this helper is no longer reading what it claims to.
#[track_caller]
fn number_after(value: &str, marker: &str) -> i64 {
    let at = value
        .find(marker)
        .unwrap_or_else(|| panic!("the catalogued value no longer names `{marker}`: {value}"));
    let rest = &value[at + marker.len()..];
    let start = rest
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or_else(|| panic!("no number follows `{marker}` in: {value}"));
    let digits: String = rest[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits
        .parse()
        .unwrap_or_else(|error| panic!("`{digits}` after `{marker}` is not a number: {error}"))
}

// ── default-value ────────────────────────────────────────────────────────────

/// The magic OpenCode sentinel, renamed, and the precedence rule around it.
#[test]
fn the_opencode_coordinator_agent_sentinel_matches_its_contract() {
    let contract = expect_contract("default-value", "OpenCode coordinator agent sentinel");
    assert!(contract.exact_value.contains("qwen-audio-agent-backend"));
    assert!(
        contract
            .exact_value
            .contains("QWEN_AUDIO_AGENT_BACKEND_AGENT takes precedence")
    );

    // `docs/rebrand.md` row 78: `qwen-audio-agent-backend` -> `via-backend`.
    assert_eq!(VIA_BACKEND_AGENT_ID, "via-backend");

    // The sentinel means "use OpenCode's own default".
    assert_eq!(
        resolve_opencode_coordinator_agent(&env(&[(
            names::OPENCODE_COORDINATOR_AGENT,
            VIA_BACKEND_AGENT_ID
        )])),
        ""
    );
    // Any other value is explicit.
    assert_eq!(
        resolve_opencode_coordinator_agent(&env(&[(
            names::OPENCODE_COORDINATOR_AGENT,
            "custom-agent"
        )])),
        "custom-agent"
    );
    // `VIA_BACKEND_AGENT` (upstream `QWEN_AUDIO_AGENT_BACKEND_AGENT`) wins.
    assert_eq!(
        resolve_opencode_coordinator_agent(&env(&[
            (names::OPENCODE_COORDINATOR_AGENT, "custom-agent"),
            (names::BACKEND_AGENT, "explicit-agent"),
        ])),
        "explicit-agent"
    );
}

/// The seven announcement numbers `via-core` resolves into [`Config`].
///
/// The row's remaining three fields — `retryBaseMs`, `retryMaxMs` and the
/// `leaseRenewIntervalMs` formula — are
/// `via_voice::announcement::manager::AnnouncementManagerConfig`'s constants,
/// not anything `via-core` reads or writes; only their presence in the
/// catalogued text is checked here, so the row stays [`Partial`] rather than
/// claiming coverage that belongs to `via-voice`.
///
/// [`Partial`]: via_conformance::Coverage::Partial
#[test]
fn the_announcement_tuning_defaults_via_core_share_matches_its_contract() {
    let contract = expect_contract("default-value", "announcement tuning defaults");
    let config = Config::default();

    assert_eq!(
        config.result_context_max_chars,
        number_after(&contract.exact_value, "resultContextMaxChars ")
    );
    assert_eq!(
        config.announcement_batch_ms,
        number_after(&contract.exact_value, "announcementBatchMs ")
    );
    assert_eq!(
        config.announcement_max_batch_items,
        number_after(&contract.exact_value, "announcementMaxBatchItems ")
    );
    assert_eq!(
        config.announcement_quiet_ms,
        number_after(&contract.exact_value, "announcementQuietMs ")
    );
    assert_eq!(
        config.announcement_acknowledgement_timeout_ms,
        number_after(
            &contract.exact_value,
            "announcementAcknowledgementTimeoutMs "
        )
    );
    assert_eq!(
        config.announcement_max_retry_attempts,
        number_after(&contract.exact_value, "announcementMaxRetryAttempts ")
    );
    assert!(contract.exact_value.contains("announceIntoContext true"));
    assert!(config.announce_into_context);

    // The Work notification claim TTL this crate does own, and the formula's
    // input agrees with it.
    assert_eq!(
        config.task_notification_claim_ttl_ms,
        number_after(&contract.exact_value, "taskNotificationClaimTtlMs default ")
    );

    // The remainder, left to `via-voice`: checked for presence, not value.
    for owed in [
        "retryBaseMs 1000",
        "retryMaxMs 10000",
        "leaseRenewIntervalMs = max(1000",
    ] {
        assert!(
            contract.exact_value.contains(owed),
            "`{owed}` left the catalogued value; the `via-voice` remainder note has gone stale"
        );
    }
}

/// Both wake-word default rows: VIA deliberately ships no hard-coded phrase.
#[test]
fn wake_word_defaults_deliberately_diverge_from_upstreams_hardcoded_phrase() {
    // Upstream's own hard-coded phrase, quoted once and only for comparison.
    let upstream_phrase = "你好千问";
    for name in ["wake word (voice layer)", "wakeWord phrase"] {
        let contract = expect_contract("default-value", name);
        assert!(
            contract.exact_value.contains(upstream_phrase),
            "the catalogued upstream phrase left `{name}`"
        );
    }
    // VIA's real, shipped value: the empty string, not a placeholder that
    // would ship (`docs/deviations/phase-1.md`).
    assert_eq!(Config::default().wake_word, "");
}

// ── error-code ───────────────────────────────────────────────────────────────

/// The Gateway setup refusal code, renamed per `docs/rebrand.md`.
#[test]
fn the_gateway_setup_refusal_code_matches_the_renamed_upstream_code() {
    let contract = expect_contract("error-code", "Gateway setup refusal code");
    assert_eq!(contract.exact_value, "QWAUDIO_GATEWAY_SETUP_REQUIRED");
    assert_eq!(
        rebranded(&contract.exact_value),
        via_protocol::CODE_GATEWAY_SETUP_REQUIRED
    );
    assert_eq!(
        via_protocol::CODE_GATEWAY_SETUP_REQUIRED,
        "VIA_GATEWAY_SETUP_REQUIRED"
    );

    // The real, shipped refusal really does carry it.
    let error = via_core::setup::assert_gateway_setup(&env(&[]), Locale::En)
        .expect_err("an empty environment must be refused");
    assert_eq!(error.code(), via_protocol::CODE_GATEWAY_SETUP_REQUIRED);
}

// ── file-path ────────────────────────────────────────────────────────────────

/// The three-deep resolution order, and where `config.env` lands.
#[test]
fn the_default_config_and_data_directory_resolution_matches_its_contract() {
    let contract = expect_contract("file-path", "default config/data directory");
    assert!(contract.exact_value.contains("QWAUDIO_CONFIG_DIR"));
    assert!(contract.exact_value.contains("QWAUDIO_DATA_DIR"));
    assert!(contract.exact_value.contains("config.env"));
    assert_eq!(via_core::paths::ENV_CONFIG_DIR, "VIA_CONFIG_DIR");
    assert_eq!(via_core::paths::ENV_DATA_DIR, "VIA_DATA_DIR");

    let shared = InstallPaths::from_env(&env(&[]), Path::new("/home/via"), Path::new("/cwd"));
    assert_eq!(
        shared.config_directory(),
        Path::new("/home/via/.config/via")
    );
    assert_eq!(shared.data_directory(), shared.config_directory());
    assert_eq!(
        shared.config_file(),
        shared.data_directory().join("config.env")
    );

    let split = InstallPaths::from_env(
        &env(&[("VIA_CONFIG_DIR", "/run"), ("VIA_DATA_DIR", "/assets")]),
        Path::new("/home/via"),
        Path::new("/cwd"),
    );
    assert_eq!(split.config_directory(), Path::new("/run"));
    assert_eq!(split.data_directory(), Path::new("/assets"));
    assert_eq!(split.config_file(), Path::new("/assets/config.env"));
}

/// Every default state/asset path the catalogue names, `PROMPT.md` included.
#[test]
fn the_default_state_and_asset_paths_match_their_contract() {
    let contract = expect_contract("file-path", "default state/asset paths");
    for name in [
        "tasks.json",
        "frontend-notes.json",
        "USER.md",
        "MEMORY.md",
        "ASSISTANT.md",
        "memory-audit.jsonl",
        "PROMPT.md",
        "config/frontend-agent",
    ] {
        assert!(
            contract.exact_value.contains(name),
            "`{name}` left the catalogued list"
        );
    }

    let paths = InstallPaths::new(PathBuf::from("/run"), PathBuf::from("/assets"));
    assert_eq!(paths.task_state_file(), Path::new("/run/tasks.json"));
    assert_eq!(
        paths.frontend_notes_file(),
        Path::new("/assets/frontend-notes.json")
    );
    assert_eq!(paths.user_model_file(), Path::new("/assets/USER.md"));
    assert_eq!(paths.memory_file(), Path::new("/assets/MEMORY.md"));
    assert_eq!(
        paths.assistant_profile_file(),
        Path::new("/assets/ASSISTANT.md")
    );
    assert_eq!(
        paths.memory_audit_file(),
        Path::new("/run/memory-audit.jsonl")
    );

    // `PROMPT.md` resolves against the runtime root, not either directory.
    let overrides = Overrides {
        home_directory: PathBuf::from("/home/via"),
        working_directory: PathBuf::from("/srv/via"),
        runtime_root: Some(PathBuf::from("/opt/via")),
        gateway: GatewayOptions::default(),
    };
    let resolved = resolve(&env(&[]), None, &overrides).expect("the defaults resolve");
    assert_eq!(
        resolved
            .frontend_prompt_dir
            .join(via_core::paths::PROMPT_FILE_NAME),
        Path::new("/opt/via/config/frontend-agent/PROMPT.md")
    );
}

/// Every managed file name, and the two file-system modes.
#[test]
fn the_managed_files_list_matches_its_contract() {
    let contract = expect_contract(
        "file-path",
        "managed files inside the config/data directories",
    );
    for name in [
        via_core::paths::CONFIG_FILE_NAME,
        via_core::paths::STATE_FILE_NAME,
        via_core::paths::USER_MODEL_FILE_NAME,
        via_core::paths::ASSISTANT_PROFILE_FILE_NAME,
        via_core::paths::MEMORY_FILE_NAME,
        via_core::paths::LEGACY_FRONTEND_MEMORY_FILE_NAME,
        via_core::paths::FRONTEND_NOTES_FILE_NAME,
        via_core::paths::TASK_STATE_FILE_NAME,
        via_core::paths::GATEWAY_LOCK_FILE_NAME,
        via_core::paths::LOG_DIRECTORY_NAME,
        via_core::paths::SHARED_WORKSPACE_DIRECTORY_NAME,
        via_core::paths::OPENCLAW_STATE_DIRECTORY,
    ] {
        assert!(
            contract.exact_value.contains(name),
            "`{name}` is not in the catalogued list"
        );
    }
    assert!(contract.exact_value.contains(&format!(
        "{}/<backendId>/",
        via_core::paths::LEGACY_WORKSPACES_DIRECTORY_NAME
    )));
    assert!(
        contract
            .exact_value
            .contains(via_core::paths::MEMORY_MIGRATION_MARKER)
    );

    assert!(
        contract
            .exact_value
            .contains(&format!("{:#o}", via_core::paths::DIRECTORY_MODE))
    );
    assert!(
        contract
            .exact_value
            .contains(&format!("{:#o}", via_core::paths::FILE_MODE))
    );
    assert_eq!(via_core::paths::DIRECTORY_MODE, 0o700);
    assert_eq!(via_core::paths::FILE_MODE, 0o600);

    assert_eq!(via_core::runtime::AUTH_SECRET_HEX_LENGTH, 64);
    assert!(contract.exact_value.contains(&format!(
        "{} hex chars",
        via_core::runtime::AUTH_SECRET_HEX_LENGTH
    )));
}

/// The real, seeded `config.env`: its path and its first line.
#[test]
fn the_runtime_config_file_matches_its_contract() {
    let contract = expect_contract("file-path", "runtime config file (the real one)");
    assert!(contract.exact_value.contains("config.env"));
    assert!(contract.exact_value.contains("qwen-audio-agent 用户配置"));

    let paths = InstallPaths::new(PathBuf::from("/run"), PathBuf::from("/run"));
    assert_eq!(paths.config_file(), Path::new("/run/config.env"));

    // The header line is `via-i18n`'s own, with only the identity renamed.
    let header = t(Locale::Zh, keys::RUNTIME_CONFIG_HEADER);
    assert_eq!(header, "# VIA 用户配置");
    let template = via_core::runtime::user_config_template(Locale::Zh);
    assert_eq!(template.lines().next(), Some(header));
}

/// The config directory's own resolution rule and its mode.
#[test]
fn the_user_config_directory_matches_its_contract() {
    let contract = expect_contract("file-path", "user config directory");
    assert!(contract.exact_value.contains("QWAUDIO_CONFIG_DIR"));
    assert!(contract.exact_value.contains("0o700"));
    assert_eq!(via_core::paths::DIRECTORY_MODE, 0o700);
    assert_eq!(via_core::paths::CONFIG_DIRECTORY_NAME, "via");

    let paths = InstallPaths::from_env(&env(&[]), Path::new("/home/via"), Path::new("/cwd"));
    assert_eq!(paths.config_directory(), Path::new("/home/via/.config/via"));
}

/// The data directory's default, and which files live in which directory.
#[test]
fn the_user_data_directory_matches_its_contract() {
    let contract = expect_contract("file-path", "user data directory");
    assert!(contract.exact_value.contains("QWAUDIO_DATA_DIR"));

    let shared = InstallPaths::from_env(&env(&[]), Path::new("/home/via"), Path::new("/cwd"));
    assert_eq!(shared.data_directory(), shared.config_directory());

    let split = InstallPaths::new(PathBuf::from("/run"), PathBuf::from("/assets"));
    for path in [
        split.config_file(),
        split.state_file(),
        split.user_model_file(),
        split.assistant_profile_file(),
        split.memory_file(),
        split.frontend_notes_file(),
        split.shared_workspace(),
    ] {
        assert!(
            path.starts_with("/assets"),
            "{} must live in the data directory",
            path.display()
        );
    }
    for path in [
        split.task_state_file(),
        split.gateway_lock_file(),
        split.log_directory(),
        split.wake_word_model_directory(),
        split.skins_directory(),
        split.deepseek_session_root(),
        split.backend_session_state_file(),
    ] {
        assert!(
            path.starts_with("/run"),
            "{} must live in the config directory",
            path.display()
        );
    }
}

/// `webDistributionPath()` is deliberately not ported: VIA ships no web UI.
#[test]
fn web_distribution_path_is_deliberately_not_ported() {
    let contract = expect_contract("file-path", "webDistributionPath()");
    assert!(contract.exact_value.contains("web/dist"));
    // The capability is dropped, not approximated with a directory that would
    // resolve to nothing (`docs/deviations/phase-1.md`).
    assert!(
        via_protocol::DROPPED_UPSTREAM_CAPABILITIES.contains(&"web.same-origin-ui"),
        "the dropped-capability list no longer records why there is nothing to resolve"
    );
}

// ── http-route ───────────────────────────────────────────────────────────────

/// Every rule in the origin allow-list table, by the contract's own examples.
#[test]
fn the_origin_allow_list_rules_match_their_contract() {
    let contract = expect_contract("http-route", "origin allow-list rules");
    assert!(contract.exact_value.contains("attacker.example:3101"));
    assert!(contract.exact_value.contains("192.168.1.20:3101"));

    // loopback host with no Origin -> allowed
    assert!(is_allowed_origin(Some("localhost:3101"), None, &[]));
    // loopback host + matching Origin -> allowed
    assert!(is_allowed_origin(
        Some("localhost:3101"),
        Some("http://localhost:3101"),
        &[]
    ));
    // mismatched Origin -> rejected
    assert!(!is_allowed_origin(
        Some("localhost:3101"),
        Some("https://attacker.example"),
        &[]
    ));
    // non-loopback Host -> rejected by default
    assert!(!is_allowed_origin(Some("attacker.example:3101"), None, &[]));
    assert!(!is_allowed_origin(Some("192.168.1.20:3101"), None, &[]));
    // ...unless explicitly allow-listed
    assert!(is_allowed_origin(
        Some("attacker.example"),
        None,
        &["https://attacker.example".to_owned()],
    ));
    // an allow-list entry matches scheme-sensitively
    assert!(!is_allowed_origin(
        Some("voice.example.com"),
        Some("http://voice.example.com"),
        &["http://voice.example.com".to_owned()],
    ));
    assert!(is_allowed_origin(
        Some("voice.example.com"),
        Some("https://voice.example.com"),
        &["https://voice.example.com".to_owned()],
    ));
}

// ── json-field ───────────────────────────────────────────────────────────────

/// The only two missing-key entries the setup gate ever built upstream.
#[test]
fn the_gateway_setup_missing_key_entries_match_their_contract() {
    let contract = expect_contract(
        "json-field",
        "gatewaySetupStatus missing-key entries (the only two)",
    );
    assert!(contract.exact_value.contains(FIELD_DASHSCOPE_API_KEY));
    assert!(contract.exact_value.contains(names::DASHSCOPE_API_KEY));
    assert!(contract.exact_value.contains(FIELD_SPEECH_TO_SPEECH_URL));
    assert!(
        contract
            .exact_value
            .contains(names::SPEECH_TO_SPEECH_REALTIME_URL)
    );

    // The dashscope branch is reachable, and really does carry this pair.
    let status = gateway_setup_status(&env(&[]), Locale::En).expect("the defaults resolve");
    assert_eq!(status.missing.len(), 1);
    assert_eq!(status.missing[0].field, FIELD_DASHSCOPE_API_KEY);
    assert_eq!(status.missing[0].key, names::DASHSCOPE_API_KEY);
}

/// The identity cookie's forgery resistance and its personal-mode collapse.
#[test]
fn the_identity_cookie_json_field_matches_its_contract() {
    let contract = expect_contract("json-field", "identity cookie");
    assert!(contract.exact_value.contains("qwen_audio_agent_identity"));
    assert_eq!(via_core::identity::IDENTITY_COOKIE_NAME, "via_identity");

    let browser = IdentityManager::new(
        "a-secret-that-is-certainly-longer-than-32",
        IdentityMode::Browser,
        "user_personal",
    )
    .expect("the secret is long enough");

    let cookie = browser.set_cookie_value("user_1", false);
    for attribute in ["HttpOnly", "SameSite=Strict"] {
        assert!(contract.exact_value.contains(attribute) && cookie.contains(attribute));
    }

    // An unsigned/attacker-chosen value resolves to null.
    assert_eq!(
        browser.resolve_token(Some("via_identity=attacker-selected-client-id")),
        None
    );

    // Malformed percent-encoding resolves to null, without throwing.
    assert!(contract.exact_value.contains("%E0%A4%A"));
    assert_eq!(browser.resolve_token(Some("via_identity=%E0%A4%A")), None);

    // Personal mode collapses every request to the configured personalOwnerId.
    assert!(contract.exact_value.contains("user_my_assistant"));
    let personal = IdentityManager::new(
        "a-secret-that-is-certainly-longer-than-32",
        IdentityMode::Personal,
        "user_my_assistant",
    )
    .expect("the secret is long enough");
    assert_eq!(personal.personal_identity().owner_id, "user_my_assistant");
    assert_eq!(
        personal.resolve_http(None, false).identity().owner_id,
        "user_my_assistant"
    );
}

/// One configured model, spelled the twelve ways the catalogue states.
#[test]
fn the_unified_backend_model_mapping_matches_its_contract() {
    let contract = expect_contract("json-field", "unified backend model mapping");
    assert!(contract.exact_value.contains("qwen3.7-plus"));
    assert!(contract.exact_value.contains("alibaba-cn/qwen3.7-plus"));
    assert!(contract.exact_value.contains("bailian/qwen3.7-plus"));
    assert!(contract.exact_value.contains("OPENCODE_MODEL"));
    assert!(contract.exact_value.contains("QODER_MODEL"));

    let models = resolve_backend_models(&env(&[(names::BACKEND_MODEL, "qwen3.7-plus")]));
    assert_eq!(models.common, "qwen3.7-plus");
    assert_eq!(models.open_code, "alibaba-cn/qwen3.7-plus");
    assert_eq!(models.open_claw, "bailian/qwen3.7-plus");
    for bare in [
        &models.qoder,
        &models.qwen,
        &models.code_buddy,
        &models.codex,
    ] {
        assert_eq!(bare, "qwen3.7-plus");
    }
    for whole in [
        &models.kimi,
        &models.hermes,
        &models.claude,
        &models.pi,
        &models.acp,
    ] {
        assert_eq!(whole, "qwen3.7-plus");
    }
    assert_eq!(models.deep_seek_harness, "");

    // VIA reads only `VIA_BACKEND_MODEL`: the backend-native override names
    // the catalogue mentions are not plumbed in at all, so the "all-empty"
    // half of the contract cannot regress into reading them by accident.
    let native = resolve_backend_models(&env(&[
        ("OPENCODE_MODEL", "should-be-ignored"),
        ("QODER_MODEL", "should-be-ignored"),
    ]));
    assert_eq!(native, BackendModels::default());
}
