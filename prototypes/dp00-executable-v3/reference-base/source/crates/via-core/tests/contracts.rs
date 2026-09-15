//! Every contract value this crate owns, asserted against
//! `docs/reference/contracts.json`.
//!
//! Nothing here retypes a catalogue value. The numbers are scanned out of the
//! catalogue's prose, the upstream environment names are pushed through
//! `docs/rebrand.md`'s rename rule programmatically, and the resulting table is
//! compared against what [`via_core::config::resolve`] actually produces — both
//! the default *and* the clamp at each bound.

mod common;

use std::collections::BTreeSet;

use common::{catalogued_settings, contract_value, contract_why, env, overrides};
use pretty_assertions::assert_eq;
use via_core::config::{Config, names, resolve};
use via_core::identity::{
    IDENTITY_COOKIE_MAX_AGE_SECONDS, IDENTITY_COOKIE_NAME, IdentityManager, IdentityMode,
};
use via_core::paths::InstallPaths;
use via_core::runtime::AUTH_SECRET_HEX_LENGTH;
use via_core::security::{
    LOOPBACK_HOSTS, ORIGIN_REJECTION_BODY, ORIGIN_REJECTION_STATUS, parse_allowed_origins,
};
use via_core::setup::{
    ALLOW_UNCONFIGURED_VALUE, FIELD_DASHSCOPE_API_KEY, FIELD_SPEECH_TO_SPEECH_URL,
};

/// Reads one numeric setting off a resolved config.
type Reader = fn(&Config) -> i64;

/// Every numeric setting `via-core` owns, keyed by its VIA environment name.
///
/// Only the *names* and the *accessors* live here. Every number comes from the
/// catalogue.
fn numeric_settings() -> Vec<(&'static str, Reader)> {
    vec![
        (names::AGENT_TIMEOUT_MS, |c| c.agent_timeout_ms),
        (names::RESULT_CONTEXT_MAX_CHARS, |c| {
            c.result_context_max_chars
        }),
        (names::ANNOUNCEMENT_BATCH_MS, |c| c.announcement_batch_ms),
        (names::ANNOUNCEMENT_MAX_BATCH_ITEMS, |c| {
            c.announcement_max_batch_items
        }),
        (names::ANNOUNCEMENT_QUIET_MS, |c| c.announcement_quiet_ms),
        (names::ANNOUNCEMENT_ACK_TIMEOUT_MS, |c| {
            c.announcement_acknowledgement_timeout_ms
        }),
        (names::ANNOUNCEMENT_MAX_RETRIES, |c| {
            c.announcement_max_retry_attempts
        }),
        (names::TASK_TERMINAL_TTL_MS, |c| c.task_terminal_ttl_ms),
        (names::TASK_NOTIFICATION_TTL_MS, |c| {
            c.task_pending_notification_ttl_ms
        }),
        (names::TASK_NOTIFICATION_CLAIM_TTL_MS, |c| {
            c.task_notification_claim_ttl_ms
        }),
        (names::MAX_TERMINAL_TASKS_PER_OWNER, |c| {
            c.max_terminal_tasks_per_owner
        }),
        (names::TASK_MAX_CONCURRENT, |c| c.task_max_concurrent),
        (names::TASK_MAX_CONCURRENT_PER_OWNER, |c| {
            c.task_max_concurrent_per_owner
        }),
        (names::SESSION_TTL_MS, |c| c.conversation_session_ttl_ms),
        (names::MAX_SESSIONS, |c| c.max_conversation_sessions),
        (names::MEMORY_OWNER_TTL_MS, |c| {
            c.frontend_memory_owner_ttl_ms
        }),
        (names::MAX_MEMORY_OWNERS, |c| c.max_frontend_memory_owners),
        (names::REMINDER_MAX_PER_OWNER, |c| c.reminder_max_per_owner),
        (names::SCHEDULED_TASK_TIMEOUT_MS, |c| {
            c.scheduled_task_timeout_ms
        }),
        (names::BACKGROUND_TASK_PROGRESS_CHECK_MS, |c| {
            c.background_task_progress_check_ms
        }),
        (names::OFFLINE_NOTIFICATION_DELAY_MS, |c| {
            c.offline_notification_delay_ms
        }),
        (names::REMINDER_STAGGER_MS, |c| c.reminder_stagger_ms),
    ]
}

fn resolved(pairs: &[(&str, &str)]) -> Config {
    resolve(&env(pairs), None, &overrides()).expect("a well-formed environment resolves")
}

#[test]
fn the_catalogue_scan_is_not_vacuous() {
    let settings = catalogued_settings();
    assert!(
        settings.len() >= 20,
        "the catalogue scan found only {} settings; the parser has drifted from the \
         catalogue's prose and every default assertion below would pass vacuously",
        settings.len()
    );
    // A representative sample, each stated in a different catalogue entry.
    for name in [
        names::TASK_MAX_CONCURRENT,
        names::ANNOUNCEMENT_QUIET_MS,
        names::MAX_SESSIONS,
        names::REMINDER_STAGGER_MS,
    ] {
        assert!(settings.contains_key(name), "{name} vanished from the scan");
    }
}

/// Settings the catalogue states that another crate owns.
///
/// The logging family belongs to `via-log`, which reads it through
/// [`via_log::EnvSettings`]. It is asserted in
/// [`the_logging_defaults_belong_to_via_log`] rather than skipped.
fn owned_elsewhere(name: &str) -> bool {
    name.starts_with("VIA_LOG_")
}

#[test]
fn the_logging_defaults_belong_to_via_log() {
    let catalogued = catalogued_settings();
    let settings = via_log::EnvSettings::from_env(&env(&[]), std::path::Path::new("/home/via"));

    let max_bytes = catalogued
        .get("VIA_LOG_MAX_BYTES")
        .expect("the catalogue states a log rotation size");
    assert_eq!(i64::try_from(settings.max_bytes), Ok(max_bytes.default));
    assert_eq!(
        i64::try_from(via_log::DEFAULT_MAX_BYTES),
        Ok(max_bytes.default)
    );

    let max_files = catalogued
        .get("VIA_LOG_MAX_FILES")
        .expect("the catalogue states a log retention count");
    assert_eq!(i64::from(settings.max_files), max_files.default);
    assert_eq!(i64::from(via_log::DEFAULT_MAX_FILES), max_files.default);

    // And `via-core` resolves the same directory `via-log` will write into.
    assert_eq!(
        settings.directory,
        resolved(&[]).config_directory().join("logs")
    );
}

#[test]
fn every_catalogued_default_is_the_shipped_default() {
    let catalogued = catalogued_settings();
    let config = resolved(&[]);
    let readers: Vec<_> = numeric_settings();
    let known: BTreeSet<&str> = readers.iter().map(|(name, _)| *name).collect();

    for (name, setting) in &catalogued {
        if owned_elsewhere(name) {
            continue;
        }
        assert!(
            known.contains(name.as_str()),
            "the catalogue states a default for `{name}` that `via-core` does not resolve"
        );
        let reader = readers
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, reader)| *reader)
            .expect("membership was just asserted");
        assert_eq!(
            reader(&config),
            setting.default,
            "{name} default disagrees with the catalogue"
        );
    }
}

#[test]
fn every_catalogued_bound_clamps_rather_than_rejecting() {
    let catalogued = catalogued_settings();
    let readers = numeric_settings();

    for (name, setting) in &catalogued {
        if owned_elsewhere(name) {
            continue;
        }
        let Some((_, reader)) = readers.iter().find(|(candidate, _)| candidate == name) else {
            continue;
        };
        if let Some(min) = setting.min {
            let below = (min - 1_000).to_string();
            assert_eq!(
                reader(&resolved(&[(name.as_str(), below.as_str())])),
                min,
                "{name} must clamp up to its catalogued minimum, not reject"
            );
        }
        if let Some(max) = setting.max {
            let above = (max + 1_000).to_string();
            assert_eq!(
                reader(&resolved(&[(name.as_str(), above.as_str())])),
                max,
                "{name} must clamp down to its catalogued maximum, not reject"
            );
        }
        // A value that is not a number at all falls back to the default.
        assert_eq!(
            reader(&resolved(&[(name.as_str(), "not-a-number")])),
            setting.default,
            "{name} must fall back to its default for an unparseable value"
        );
    }
}

#[test]
fn host_and_port_defaults_come_from_the_catalogue() {
    let host = contract_value("default-value", "HOST");
    let port = contract_value("default-value", "PORT");
    let config = resolved(&[]);

    assert!(
        host.contains(&format!("'{}'", config.host)),
        "the catalogued HOST default is {host:?}, the shipped one is {:?}",
        config.host
    );
    assert!(
        port.contains(&config.port.to_string()),
        "the catalogued PORT default is {port:?}, the shipped one is {}",
        config.port
    );
    // PORT=0 is the one value that is not clamped into [1, 65535].
    assert!(port.contains("PORT).trim() === '0' -> 0"));
    assert_eq!(resolved(&[("PORT", "0")]).port, 0);
    assert_eq!(resolved(&[("PORT", " 0 ")]).port, 0);
    assert_eq!(resolved(&[("PORT", "70000")]).port, 65_535);
    assert_eq!(resolved(&[("PORT", "-5")]).port, 1);
}

#[test]
fn the_identity_cookie_matches_its_contract() {
    let catalogued = contract_value("cookie", "identity cookie");

    // Upstream's name, renamed per docs/rebrand.md.
    assert!(catalogued.contains("qwen_audio_agent_identity"));
    assert_eq!(IDENTITY_COOKIE_NAME, "via_identity");
    assert!(catalogued.contains(&format!("Max-Age={IDENTITY_COOKIE_MAX_AGE_SECONDS}")));

    let manager = IdentityManager::new(
        "a-secret-that-is-certainly-longer-than-32",
        IdentityMode::Browser,
        "user_personal",
    )
    .expect("the secret is long enough");
    let cookie = manager.set_cookie_value("user_1", false);
    for attribute in ["Path=/", "HttpOnly", "SameSite=Strict"] {
        assert!(
            catalogued.contains(attribute) && cookie.contains(attribute),
            "{attribute} must be in both the catalogue and the header"
        );
    }
    assert!(!cookie.contains("Secure"));
    assert!(
        manager
            .set_cookie_value("user_1", true)
            .ends_with("; Secure")
    );
}

#[test]
fn the_origin_rejection_matches_its_contract() {
    let catalogued = contract_value("http-error", "origin rejection");
    assert!(catalogued.contains(&ORIGIN_REJECTION_STATUS.to_string()));
    assert!(catalogued.contains(ORIGIN_REJECTION_BODY));
}

#[test]
fn the_allowed_origins_rule_matches_its_contract() {
    let catalogued = contract_value("env-var", "QWEN_AUDIO_AGENT_ALLOWED_ORIGINS");
    assert!(catalogued.contains("comma-separated string; default ''"));
    assert!(
        contract_why("env-var", "QWEN_AUDIO_AGENT_ALLOWED_ORIGINS").contains("silently DROPPED")
    );

    // Every loopback host the catalogue names.
    for host in LOOPBACK_HOSTS {
        assert!(
            catalogued.contains(host),
            "{host} is a loopback host in code but not in the catalogue"
        );
    }

    assert!(resolved(&[]).allowed_origins.is_empty());
    assert_eq!(
        parse_allowed_origins(Some(" https://a.example , ,https://b.example ")),
        vec![
            "https://a.example".to_owned(),
            "https://b.example".to_owned()
        ]
    );
}

#[test]
fn the_setup_gate_fields_match_their_contract() {
    let catalogued = contract_value("json-field", "gatewaySetupStatus missing entries");
    assert!(catalogued.contains(FIELD_DASHSCOPE_API_KEY));
    assert!(catalogued.contains(FIELD_SPEECH_TO_SPEECH_URL));
    assert!(catalogued.contains(names::DASHSCOPE_API_KEY));
    assert!(catalogued.contains(names::SPEECH_TO_SPEECH_REALTIME_URL));

    let opt_out = contract_value("env-var", "QWEN_AUDIO_ALLOW_UNCONFIGURED");
    assert!(opt_out.contains(&format!("'{ALLOW_UNCONFIGURED_VALUE}'")));
}

#[test]
fn the_generated_secret_matches_its_contract() {
    let catalogued = contract_value("default-value", "generated auth secret");
    assert!(catalogued.contains(&format!("{AUTH_SECRET_HEX_LENGTH} lowercase hex chars")));
    assert!(catalogued.contains("flag 'wx'"));
    assert!(catalogued.contains("mode 0o600"));
    // The variable it is written under, renamed.
    assert!(catalogued.contains("QWEN_AUDIO_AGENT_AUTH_SECRET"));
    assert_eq!(names::AUTH_SECRET, "VIA_AUTH_SECRET");
}

#[test]
fn the_directory_layout_matches_its_contract() {
    let catalogued = contract_value("file-path", "user config / data directory resolution order");
    assert!(catalogued.contains("QWAUDIO_CONFIG_DIR"));
    assert!(catalogued.contains("QWAUDIO_DATA_DIR"));
    assert!(catalogued.contains("${XDG_CONFIG_HOME}/qwaudio"));
    assert_eq!(names::CONFIG_DIR, "VIA_CONFIG_DIR");
    assert_eq!(names::DATA_DIR, "VIA_DATA_DIR");

    let paths = InstallPaths::from_env(
        &env(&[]),
        std::path::Path::new("/home/via"),
        std::path::Path::new("/srv/via"),
    );
    assert_eq!(
        paths.config_directory(),
        std::path::Path::new("/home/via/.config/via")
    );
}

#[test]
fn the_config_and_data_split_matches_its_contract() {
    // The catalogue names, file by file, which directory each lives in, split
    // by the literal "In configDirectory:".
    let catalogued = contract_value("file-path", "files and directories the product creates");
    let (data_half, config_half) = catalogued
        .split_once("In configDirectory:")
        .expect("the catalogue states the split with this exact phrase");
    let paths = InstallPaths::new("/run".into(), "/assets".into());
    for (name, path) in [
        ("config.env", paths.config_file()),
        ("state.env", paths.state_file()),
        ("USER.md", paths.user_model_file()),
        ("ASSISTANT.md", paths.assistant_profile_file()),
        ("MEMORY.md", paths.memory_file()),
        ("frontend-notes.json", paths.frontend_notes_file()),
        ("workspace/", paths.shared_workspace()),
        (
            "state/frontend-memory-markdown-v1",
            paths.memory_migration_marker(),
        ),
        ("frontend-memory.json", paths.legacy_frontend_memory_file()),
    ] {
        assert!(
            data_half.contains(name),
            "{name} is not on the catalogue's data-directory side"
        );
        assert!(
            path.starts_with("/assets"),
            "{name} must live in the data directory, not {}",
            path.display()
        );
    }
    for (name, path) in [
        ("gateway.lock", paths.gateway_lock_file()),
        ("tasks.json", paths.task_state_file()),
        ("memory-audit.jsonl", paths.memory_audit_file()),
        (
            "state/acp-sessions.json",
            paths.backend_session_state_file(),
        ),
        ("models/wake-word/", paths.wake_word_model_directory()),
        ("skins/", paths.skins_directory()),
        (
            "backends/deepseek-harness/sessions",
            paths.deepseek_session_root(),
        ),
        ("gateway-token", paths.openclaw_token_file()),
    ] {
        assert!(
            config_half.contains(name),
            "{name} is not on the catalogue's config-directory side"
        );
        assert!(
            path.starts_with("/run"),
            "{name} must live in the config directory, not {}",
            path.display()
        );
    }
}

#[test]
fn the_memory_scope_table_matches_its_contract() {
    let catalogued = contract_value("default-value", "MEMORY_SCOPES");
    assert!(catalogued.contains("ALL_SCOPE='all'"));
    assert_eq!(
        via_core::memory_scopes::memory_documents(),
        vec!["user", "memory"]
    );
    for scope in via_core::memory_scopes::MEMORY_SCOPES {
        assert!(catalogued.contains(&format!("maxEntries:{}", scope.max_entries)));
        assert!(catalogued.contains(&format!("maxChars:{}", scope.max_chars)));
    }
    for (alias, canonical) in via_core::memory_scopes::SCOPE_ALIASES {
        assert!(
            catalogued.contains(&format!("{alias}->{canonical}")),
            "the alias {alias}->{canonical} left the catalogue"
        );
    }
}

#[test]
fn the_memory_extractor_defaults_match_their_contract() {
    let catalogued = contract_value("env-var", "memory extractor");
    let config = resolved(&[]);
    assert!(catalogued.contains(&format!("default '{}'", config.memory_model)));
    assert!(catalogued.contains(&format!("default '{}'", config.memory_base_url)));
    assert!(catalogued.contains("disabled only when lowercase === 'off'"));
    assert!(config.memory_auto_enabled);
    assert!(!resolved(&[(names::MEMORY_AUTO, "OFF")]).memory_auto_enabled);
    assert!(resolved(&[(names::MEMORY_AUTO, "anything-else")]).memory_auto_enabled);
}

#[test]
fn the_identity_mode_rule_matches_its_contract() {
    let catalogued = contract_value("env-var", "QWEN_AUDIO_AGENT_IDENTITY_MODE");
    assert!(catalogued.contains("ANY other value silently means personal"));
    assert_eq!(resolved(&[]).identity_mode, IdentityMode::Personal);
    assert_eq!(
        resolved(&[(names::IDENTITY_MODE, "BROWSER")]).identity_mode,
        IdentityMode::Browser
    );
    for value in ["browserr", "", "cookie", "Personal", "1"] {
        assert_eq!(
            resolved(&[(names::IDENTITY_MODE, value)]).identity_mode,
            IdentityMode::Personal,
            "{value:?} must fail closed onto personal"
        );
    }
    let personal_owner = contract_value("env-var", "QWEN_AUDIO_AGENT_PERSONAL_OWNER_ID");
    assert!(personal_owner.contains(&format!("default '{}'", resolved(&[]).personal_owner_id)));
}
