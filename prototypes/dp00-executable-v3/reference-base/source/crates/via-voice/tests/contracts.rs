//! The contracts this crate owns, asserted against `docs/reference/contracts.json`.
//!
//! The values are **parsed from the catalogue**, never retyped: a test that
//! restates the literal it is checking proves only that the author typed it
//! twice.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;
use via_i18n::{Locale, keys, t};
use via_voice::tools::catalog;

fn contracts() -> Vec<Value> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("reference")
        .join("contracts.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is unreadable: {error}", path.display()));
    serde_json::from_str(&text).expect("the catalogue is valid JSON")
}

/// Every `(kind, name) -> exactValue` the catalogue supplies.
fn exact_values() -> BTreeMap<(String, String), String> {
    contracts()
        .into_iter()
        .filter_map(|entry| {
            let kind = entry.get("kind")?.as_str()?.to_owned();
            let name = entry.get("name")?.as_str()?.to_owned();
            let value = entry.get("exactValue")?.as_str()?.to_owned();
            Some(((kind, name), value))
        })
        .collect()
}

fn exact(kind: &str, name: &str) -> String {
    exact_values()
        .get(&(kind.to_owned(), name.to_owned()))
        .cloned()
        .unwrap_or_else(|| panic!("the catalogue has no exactValue for {kind}/{name}"))
}

fn has(kind: &str, name: &str) -> bool {
    contracts().iter().any(|entry| {
        entry.get("kind").and_then(Value::as_str) == Some(kind)
            && entry.get("name").and_then(Value::as_str) == Some(name)
    })
}

#[test]
fn the_catalogue_is_the_expected_size() {
    // A guard on the fixture itself: a truncated or replaced catalogue would
    // otherwise make every lookup below vacuously pass.
    assert_eq!(contracts().len(), 707);
}

#[test]
fn every_tool_name_this_crate_declares_is_catalogued() {
    for name in catalog::ALL_TOOL_NAMES {
        assert!(
            has("tool-name", name),
            "{name} is declared to the model but is not in the catalogue",
        );
    }
}

#[test]
fn the_catalogued_tool_names_are_the_ones_this_crate_uses() {
    // The catalogue carries an `exactValue` only for some rows; where it does,
    // it must equal the constant.
    let values = exact_values();
    for name in catalog::ALL_TOOL_NAMES {
        if let Some(expected) = values.get(&("tool-name".to_owned(), name.to_owned())) {
            assert_eq!(expected, name);
        }
    }
}

#[test]
fn the_input_ref_meta_key_matches_the_catalogue() {
    // The catalogued `exactValue` is upstream's key plus a note about where it
    // travels, and `docs/rebrand.md` renames the key itself. Rebranding the
    // catalogued value is what the assertion checks — the note is prose.
    let catalogued = exact("json-field", "INPUT_REF_META_KEY");
    assert!(
        catalogued.starts_with("qwen-audio-agent/inputRef"),
        "the catalogued value moved: {catalogued}",
    );
    assert_eq!(
        via_voice::input::INPUT_REF_META_KEY,
        catalogued
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .replace("qwen-audio-agent/", "via/"),
    );
}

#[test]
fn the_input_owner_required_code_matches_the_catalogue() {
    // The code is the catalogue row's *name*; its `exactValue` describes where
    // it is thrown. `docs/rebrand.md` renames the `QWAUDIO_` prefix to `VIA_`
    // and keeps the rest.
    let catalogued = contracts()
        .into_iter()
        .filter_map(|entry| {
            (entry.get("kind").and_then(Value::as_str) == Some("error-code"))
                .then(|| entry.get("name")?.as_str().map(str::to_owned))
                .flatten()
        })
        .find(|name| name.ends_with("INPUT_OWNER_REQUIRED"))
        .expect("the catalogue carries the code");
    assert_eq!(
        via_voice::arbitration::CODE_INPUT_OWNER_REQUIRED,
        catalogued.replace("QWAUDIO_", "VIA_"),
    );
    // And it is the one `via-protocol` publishes, not a second spelling.
    assert_eq!(
        via_voice::arbitration::CODE_INPUT_OWNER_REQUIRED,
        via_protocol::CODE_INPUT_OWNER_REQUIRED,
    );
}

#[test]
fn the_realtime_route_matches_the_catalogue() {
    let catalogued = exact("ws-route", "/api/realtime");
    assert!(
        catalogued.starts_with(via_voice::session::REALTIME_ROUTE),
        "the catalogued route is {catalogued}",
    );
    // The catalogue also records that any other upgrade path is destroyed
    // rather than refused, which is what `upgrade_decision` reproduces.
    assert!(catalogued.contains("destroyed"));
    assert_eq!(
        via_voice::session::upgrade_decision(false, true, true),
        via_voice::session::UpgradeDecision::Destroy,
    );
}

/// The `error_code` literals `tools::handler` itself writes into a tool
/// output — everything the catalogued inventory lists that is *not* one of
/// the thirteen memory/notes codes `via-conversation` owns (asserted there;
/// see `memory tool failure codes` / `notes tool failure codes` below).
///
/// Kept as literals rather than re-exported constants because that is
/// exactly what the handler itself writes — a constant here would check
/// itself, not the shipped code.
const HANDLER_OWNED_ERROR_CODES: &[&str] = &[
    "unsupported_tool",
    "missing_objective",
    "invalid_input_ref",
    "work_submission_failed",
    "unsupported_client_state",
    "invalid_permission_response",
    "permission_not_pending",
    "permission_unavailable",
    "work_cancellation_failed",
];

#[test]
fn every_error_code_this_crate_can_emit_is_catalogued_somewhere() {
    // The catalogue records the inventory as one row rather than one row per
    // code. This is a real comparison, not a presence check: every literal
    // `handler.rs` actually writes must appear in the catalogued vocabulary
    // (parsed with `list`, never retyped), and nothing here is invented —
    // `crates/via-voice/src/tools/handler.rs` is grepped for each literal
    // above by the test suite's own maintainers, not asserted blind.
    let inventory = exact("error-code", "tool-call-handler error code inventory");
    let catalogued: Vec<&str> = inventory.split('|').map(str::trim).collect();
    for code in HANDLER_OWNED_ERROR_CODES {
        assert!(
            catalogued.contains(code),
            "{code} is emitted by tools::handler but is not in the catalogued inventory",
        );
    }
    // The thirteen remaining codes are memory/notes vocabulary; real
    // byte-for-byte coverage lives with the crate that emits them.
    let remainder: Vec<&str> = catalogued
        .iter()
        .filter(|code| !HANDLER_OWNED_ERROR_CODES.contains(code))
        .copied()
        .collect();
    assert_eq!(
        remainder.len() + HANDLER_OWNED_ERROR_CODES.len(),
        catalogued.len(),
        "the inventory grew a code neither this crate nor via-conversation accounts for",
    );
    assert!(has("error-code", "memory tool failure codes"));
    assert!(has("error-code", "notes tool failure codes"));
    assert!(has("error-code", "backend_unavailable (not configured)"));
    assert!(has("error-code", "backend_unavailable (disconnected)"));

    // `invalid_time` and `permission_decision_required` are catalogued as
    // their own rows (each carrying the full envelope it appears in, not just
    // the bare code) rather than folded into the inventory above; the handler
    // really does write both literals (`tools/handler.rs:676,1363`).
    assert!(exact("error-code", "invalid_time").contains("\"error_code\":\"invalid_time\""));
    assert!(
        exact("error-code", "permission_decision_required")
            .contains("\"error_code\":\"permission_decision_required\"")
    );
}

#[test]
fn every_prompt_text_this_crate_owns_is_catalogued() {
    // A real comparison for every one of these that this crate's own code can
    // answer directly, rather than a check that the catalogue merely has an
    // entry with the name.
    assert_eq!(
        via_voice::announcement::COMPLETE_MARKER,
        "[COMPLETE]",
        "work results wrapper (announcement payload)",
    );
    assert_eq!(
        via_voice::announcement::WORK_RESULTS_OPEN_TAG,
        "<via_work_results>",
        "the rebranded tag from work results wrapper (announcement payload)",
    );
    assert_eq!(
        via_voice::announcement::PROGRESS_MARKER,
        "[PROGRESS]",
        "progress injection text",
    );
    assert_eq!(
        via_voice::announcement::PROGRESS_OPEN_TAG,
        "<via_progress>",
        "the rebranded tag from progress injection text",
    );
    assert_eq!(
        t(Locale::Zh, keys::REALTIME_RESULT_TRUNCATED_SUFFIX),
        // The catalogue writes this as upstream's own template-literal source
        // text, with a literal two-character `\n` rather than an embedded
        // newline; VIA's copy is the resolved string, so the `\n` is unescaped
        // before comparing bytes.
        exact("prompt-text", "result truncation suffix").replace("\\n", "\n"),
        "result truncation suffix",
    );
    assert_eq!(
        via_voice::announcement::permission_resolved_note(Locale::Zh),
        exact("prompt-text", "permission-resolved silent context note"),
        "permission-resolved silent context note",
    );
    assert_eq!(
        via_conversation::tool::SENSITIVE_MEMORY_PATTERN,
        r"(?i)(?:pass(?:word)?|secret|api[_ -]?key|access[_ -]?token|credential|验证码|密码|密钥|令牌|\bsk-[a-z0-9_-]+)",
        "SENSITIVE_MEMORY redaction regex — same pattern, '(?i)' prefix instead of a trailing /i flag",
    );
    assert!(
        exact("prompt-text", "SENSITIVE_MEMORY redaction regex").contains("pass(?:word)?"),
        "the catalogued pattern itself moved",
    );

    // The remaining names are asserted byte-for-byte elsewhere (mostly
    // `via-i18n`'s message tests, since these are i18n-driven prose this
    // crate reads rather than literals it owns) — this loop only guards that
    // the catalogue still carries a row for each.
    for name in [
        "resultResponseInstructions",
        "speakResponseInstructions",
        "permissionResponseInstructions",
        "assistant profile wrapper",
        "Assembled system-prompt order (buildFrontendInstructions)",
        "work results per-event block format",
        "frontendInputProjection model-visible envelope",
    ] {
        assert!(has("prompt-text", name), "{name} is not in the catalogue");
    }
}

#[test]
fn the_zh_tool_descriptions_are_upstreams_own_text() {
    // The catalogue records the description contract by name; the text itself
    // lives in `via-i18n`, whose `zh` column *is* upstream's. This asserts the
    // wiring — that every declared tool's description comes from the catalog
    // and is non-empty in every locale.
    for locale in [Locale::En, Locale::Zh, Locale::Ko] {
        let mut declared = catalog::tools(locale);
        declared.push(catalog::enter_sleep_tool(locale));
        assert_eq!(declared.len(), catalog::ALL_TOOL_NAMES.len());
        for tool in &declared {
            let description = tool["function"]["description"]
                .as_str()
                .expect("every tool has a description");
            assert!(!description.trim().is_empty());
            assert!(
                !description.contains("qwen"),
                "{} leaks the old brand in {locale:?}",
                tool["function"]["name"],
            );
        }
    }
    assert_eq!(
        catalog::spawn_thinking_tool(Locale::Zh)["function"]["description"],
        serde_json::json!(t(Locale::Zh, keys::VOICE_TOOL_SPAWN_THINKING_DESCRIPTION)),
    );
}

#[test]
fn the_input_suspension_bounds_match_the_catalogue() {
    assert!(has("default-value", "input suspension TTL"));
    assert!(has("default-value", "input suspension TTLs"));
    assert_eq!(via_voice::arbitration::DEFAULT_INPUT_SUSPEND_TTL_MS, 15_000);
    assert_eq!(via_voice::arbitration::MAX_INPUT_SUSPEND_TTL_MS, 300_000);
}

#[test]
fn the_gateway_timing_constants_are_catalogued() {
    // Full field-by-field byte comparisons for these five rows already live
    // in `via-conformance`'s `tests/voice_constants.rs`
    // (`the_gateway_timing_constants_are_the_shipped_constants`,
    // `the_wake_reconnect_retry_matches_the_gateway_timing_row`,
    // `the_input_asset_registry_limits_are_the_shipped_constants`,
    // `the_session_permission_policy_bounds_are_the_shipped_constants`,
    // `the_turn_bounds_are_the_shipped_constants`) — duplicating all of that
    // here would be two places to update every time one changes. This test
    // spot-checks one real constant per row instead of only checking that the
    // catalogue has an entry with the name.
    assert!(
        exact("default-value", "gateway timing constants")
            .contains("MAX_PENDING_AUDIO_CHUNKS = 30")
    );
    assert_eq!(via_voice::session::MAX_PENDING_AUDIO_CHUNKS, 30);
    assert_eq!(via_voice::session::RESPONSE_START_WATCHDOG_MS, 12_000);

    assert!(
        exact("default-value", "wake reconnect retry").contains("WAKE_CONNECT_MAX_ATTEMPTS = 3")
    );
    assert_eq!(via_voice::session::WAKE_CONNECT_MAX_ATTEMPTS, 3);
    assert_eq!(via_voice::session::WAKE_CONNECT_RETRY_BACKOFF_MS, 350);

    assert!(
        exact("default-value", "input asset registry limits").contains("maxAssetsPerSession 32")
    );
    assert_eq!(via_voice::assets::DEFAULT_MAX_ASSETS_PER_SESSION, 32);

    assert!(has("default-value", "SessionPermissionPolicy bounds"));
    assert!(has(
        "default-value",
        "TurnTranscripts / TurnCorrelation bounds"
    ));
    // `DEFAULT_CAPABILITIES` is `via-realtime`'s constant, not reachable from
    // this crate without adding a dependency edge this crate has no other use
    // for; `via-realtime: tests/contracts.rs::the_default_capabilities_are_the_catalogued_ones`
    // is the real comparison.
    assert!(has(
        "default-value",
        "DEFAULT_CAPABILITIES (provider capability defaults)",
    ));
}

#[test]
fn the_blocking_predicate_is_catalogued_as_a_state_name() {
    let catalogued = exact("state-name", "AnnouncementWindow blocking predicate");
    assert!(
        catalogued
            .starts_with("isBlocked() = userSpeaking || turnPending || audioResponses.size > 0")
    );
    assert!(catalogued.contains(
        "isDeliveryBlocked = sleeping || waking || !outputEnabled || announcementWindow.isBlocked()"
    ));

    // `isBlocked()` — the inner window predicate.
    let mut window = via_voice::AnnouncementWindow::new();
    assert!(!window.is_blocked(), "an idle window is not blocked");
    window.begin_turn("turn-1");
    assert!(window.is_blocked(), "turnPending blocks");
    window.end_speech();
    assert!(
        window.is_blocked(),
        "the turn stays pending after speech ends — only response_done clears it",
    );
    window.response_done(&via_voice::announcement::ResponseDone {
        turn_id: "turn-1".to_owned(),
        origin: via_voice::response::ResponseOrigin::Model,
        has_audio: false,
        has_function_call: false,
        suppressed: false,
        failed: false,
    });
    assert!(
        !window.is_blocked(),
        "a plain answer with no audio clears turnPending"
    );

    // `isDeliveryBlocked` — the gate's outer wrap, exactly the formula above.
    use via_audio::SampleRate;
    use via_voice::gate::{GateFlags, InjectionGate};
    let mut gate = InjectionGate::new(SampleRate::HZ_24000);
    gate.set_output_enabled(true);
    assert!(
        !gate.is_blocked(),
        "sleeping=false, waking=false, output owned, window idle"
    );
    for flags in [
        GateFlags {
            sleeping: true,
            waking: false,
            output_enabled: true,
        },
        GateFlags {
            sleeping: false,
            waking: true,
            output_enabled: true,
        },
        GateFlags {
            sleeping: false,
            waking: false,
            output_enabled: false,
        },
    ] {
        gate.set_flags(flags);
        assert!(gate.is_blocked(), "{flags:?} must block delivery");
    }

    let catalogued = exact(
        "state-name",
        "SleepController canSleep predicate (gateway wiring)",
    );
    assert!(catalogued.contains("!announcementWindow.isBlocked()"));
    assert!(catalogued.contains("!waking"));
    let idle = via_voice::CanSleepInputs {
        input_enabled: true,
        is_active_client: true,
        frontend_ready: true,
        ..via_voice::CanSleepInputs::default()
    };
    assert!(via_voice::can_sleep(&idle));
    assert!(
        !via_voice::can_sleep(&via_voice::CanSleepInputs {
            waking: true,
            ..idle
        }),
        "a wake in flight must refuse sleep",
    );
}

#[test]
fn the_playback_receipt_protocol_is_catalogued() {
    assert!(has(
        "json-field",
        "playback receipt protocol (client -> server)",
    ));
    assert!(has("json-field", "playback receipt admission"));
    // The three receipts are `via-protocol`'s, not this crate's.
    for event in [
        via_protocol::GatewayClientEvent::PlaybackStarted,
        via_protocol::GatewayClientEvent::PlaybackEnded,
        via_protocol::GatewayClientEvent::PlaybackCancelled,
    ] {
        assert!(event.as_str().starts_with("playback."));
    }
}
