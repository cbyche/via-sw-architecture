//! Contract assertions for the realtime model catalog.
//!
//! Values are taken from `shared/realtime-model-catalog.mjs` and from upstream's
//! own deep-equality test, `test/realtime-provider-catalog.test.mjs:44-107`.

use pretty_assertions::assert_eq;
use via_catalog::realtime_model::{
    AUDIO_FAMILY_VOICE_ENV, DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL,
    DASHSCOPE_OMNI_FLASH_REALTIME_MODEL, DASHSCOPE_OMNI_PLUS_REALTIME_MODEL,
    DASHSCOPE_OMNI_REALTIME_VOICE, DEFAULT_DASHSCOPE_REALTIME_MODEL,
    DEFAULT_DASHSCOPE_REALTIME_VOICE, LOCAL_FAMILY_VOICE_ENV, ModelCapabilities, ModelFamily,
    OMNI_FAMILY_VOICE_ENV, TransportCapabilities, TurnDetectionKind,
    dashscope_realtime_model_profiles, default_dashscope_realtime_model_profile,
    local_realtime_model_profile, resolve_dashscope_realtime_model_profile,
};

/// `shared/realtime-model-catalog.mjs:1-6` — the exact vendor identifiers.
#[test]
fn model_ids_are_the_vendor_strings() {
    assert_eq!(
        DEFAULT_DASHSCOPE_REALTIME_MODEL,
        "qwen-audio-3.0-realtime-plus"
    );
    assert_eq!(
        DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL,
        "qwen-audio-3.0-realtime-flash"
    );
    assert_eq!(
        DASHSCOPE_OMNI_FLASH_REALTIME_MODEL,
        "qwen3.5-omni-flash-realtime"
    );
    assert_eq!(
        DASHSCOPE_OMNI_PLUS_REALTIME_MODEL,
        "qwen3.5-omni-plus-realtime"
    );
    assert_eq!(DEFAULT_DASHSCOPE_REALTIME_VOICE, "longanqian");
    assert_eq!(DASHSCOPE_OMNI_REALTIME_VOICE, "Ethan");
}

/// `test/realtime-provider-catalog.test.mjs:44-107` asserts this list by deep
/// equality, so both the membership and the order are contract. `via config`
/// prints the models in this order too.
#[test]
fn catalog_is_the_exact_upstream_list_in_catalog_order() {
    let listed: Vec<(&str, &str, ModelFamily)> = dashscope_realtime_model_profiles()
        .iter()
        .map(|p| (p.id.as_ref(), p.label.as_ref(), p.family))
        .collect();

    assert_eq!(
        listed,
        vec![
            (
                "qwen3.5-omni-flash-realtime",
                "Qwen3.5 Omni Flash Realtime",
                ModelFamily::Omni
            ),
            (
                "qwen3.5-omni-plus-realtime",
                "Qwen3.5 Omni Plus Realtime",
                ModelFamily::Omni
            ),
            (
                "qwen-audio-3.0-realtime-plus",
                "Qwen Audio 3.0 Realtime Plus",
                ModelFamily::Audio
            ),
            (
                "qwen-audio-3.0-realtime-flash",
                "Qwen Audio 3.0 Realtime Flash",
                ModelFamily::Audio
            ),
        ]
    );
}

/// `shared/realtime-model-catalog.mjs:32-39`. These two strings are written into
/// `session.voice` and `session.turn_detection.type` on the wire.
#[test]
fn session_defaults_are_family_scoped() {
    for id in [
        DASHSCOPE_OMNI_FLASH_REALTIME_MODEL,
        DASHSCOPE_OMNI_PLUS_REALTIME_MODEL,
    ] {
        let profile = resolve_dashscope_realtime_model_profile(id).expect("catalog id");
        assert_eq!(profile.session_defaults.voice, "Ethan");
        assert_eq!(
            profile.session_defaults.turn_detection.kind,
            TurnDetectionKind::SemanticVad
        );
    }
    for id in [
        DEFAULT_DASHSCOPE_REALTIME_MODEL,
        DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL,
    ] {
        let profile = resolve_dashscope_realtime_model_profile(id).expect("catalog id");
        assert_eq!(profile.session_defaults.voice, "longanqian");
        assert_eq!(
            profile.session_defaults.turn_detection.kind,
            TurnDetectionKind::SmartTurn
        );
    }
}

/// `shared/realtime-model-catalog.mjs:8-23`. The Audio (legacy) set differs from
/// the Omni set in exactly one flag, `imageInput`.
#[test]
fn capability_flag_sets_are_exact() {
    let omni = resolve_dashscope_realtime_model_profile(DASHSCOPE_OMNI_PLUS_REALTIME_MODEL)
        .expect("catalog id");
    assert_eq!(
        omni.model_capabilities,
        ModelCapabilities {
            text_input: true,
            audio_input: true,
            image_input: true,
            video_input: false,
            text_output: true,
            audio_output: true,
            function_calling: true,
        }
    );
    assert_eq!(
        omni.transport_capabilities,
        TransportCapabilities {
            text_input: true,
            audio_input: true,
            image_input: false,
            observation_input: false,
            native_video_input: false,
        }
    );

    let audio = resolve_dashscope_realtime_model_profile(DEFAULT_DASHSCOPE_REALTIME_MODEL)
        .expect("catalog id");
    assert_eq!(
        audio.model_capabilities,
        ModelCapabilities {
            text_input: true,
            audio_input: true,
            image_input: false,
            video_input: false,
            text_output: true,
            audio_output: true,
            function_calling: true,
        }
    );
    assert_eq!(
        audio.transport_capabilities,
        TransportCapabilities {
            text_input: true,
            audio_input: true,
            image_input: false,
            observation_input: false,
            native_video_input: false,
        }
    );
}

/// `test/realtime-provider-catalog.test.mjs:129-135`.
#[test]
fn the_legacy_audio_plus_model_is_the_default() {
    assert_eq!(
        default_dashscope_realtime_model_profile().id,
        "qwen-audio-3.0-realtime-plus"
    );
    // Upstream's `resolveDashScopeRealtimeModelProfile()` with no argument.
    assert_eq!(
        resolve_dashscope_realtime_model_profile("")
            .expect("empty resolves to the default")
            .id,
        DEFAULT_DASHSCOPE_REALTIME_MODEL
    );
    assert_eq!(
        resolve_dashscope_realtime_model_profile("   ")
            .expect("whitespace resolves to the default")
            .id,
        DEFAULT_DASHSCOPE_REALTIME_MODEL
    );
    // Ids are matched exactly, after trimming.
    assert_eq!(
        resolve_dashscope_realtime_model_profile("  qwen3.5-omni-flash-realtime ")
            .expect("trimmed id")
            .id,
        DASHSCOPE_OMNI_FLASH_REALTIME_MODEL
    );
}

/// **The bug that is not ported.**
///
/// `qwen3.5-omni-plus-realtime-future` is upstream's own test input
/// (`test/realtime-provider-catalog.test.mjs:166-202`), where it produces
/// `family: 'unknown'` and every capability flag `false`. Combined with
/// `providers/dashscope.mjs:84-87`, which writes
/// `session.turn_detection = transportCapabilities.audioInput ? … : null`, that
/// profile opens a session with no audio input and no turn detection: it
/// connects and then never hears the user.
///
/// VIA refuses instead. The fail-closed property upstream cared about —
/// capabilities are never inferred from the *shape* of an id — is preserved, and
/// strengthened: there is no all-false profile to hand out at all.
#[test]
fn an_unknown_model_id_is_an_error_not_an_all_false_profile() {
    let error = resolve_dashscope_realtime_model_profile("qwen3.5-omni-plus-realtime-future")
        .expect_err("an unrecognised id must not resolve");
    assert_eq!(error.code(), "VIA_REALTIME_MODEL_UNKNOWN");
    assert_eq!(
        error,
        via_catalog::CatalogError::UnknownRealtimeModel {
            model: "qwen3.5-omni-plus-realtime-future".to_owned(),
        }
    );

    // Not name-inferred in either direction: a prefix of a catalog id is no more
    // acceptable than a suffix.
    assert!(resolve_dashscope_realtime_model_profile("qwen3.5-omni").is_err());
    assert!(resolve_dashscope_realtime_model_profile("gpt-realtime-2.1").is_err());

    // And no catalog entry is all-false, so nothing in the table can stand in
    // for the fallback upstream returned.
    for profile in dashscope_realtime_model_profiles() {
        assert!(
            profile.transport_capabilities.audio_input,
            "{} would open a deaf session",
            profile.id
        );
    }
}

/// The local family is the other half of the fix: on-device ids are by
/// definition absent from the DashScope table, so they need a home with **real**
/// flags rather than the all-false fallback.
#[test]
fn the_local_family_has_real_capabilities() {
    let profile = local_realtime_model_profile("qwen3-omni-30b-a3b-q4_k_m.gguf");

    assert_eq!(profile.family, ModelFamily::Local);
    assert_eq!(profile.id, "qwen3-omni-30b-a3b-q4_k_m.gguf");
    assert_eq!(profile.label, profile.id, "label mirrors the id");

    // The flag the deafness bug hinges on.
    assert!(profile.transport_capabilities.audio_input);
    assert!(profile.model_capabilities.audio_input);
    assert!(profile.model_capabilities.audio_output);
    assert!(profile.model_capabilities.function_calling);
    // The componentized pipeline is audio-only.
    assert!(!profile.model_capabilities.image_input);
    assert!(!profile.model_capabilities.video_input);
    assert!(!profile.transport_capabilities.native_video_input);

    // A local id still resolves to nothing in the DashScope table.
    assert!(resolve_dashscope_realtime_model_profile("qwen3-omni-30b-a3b-q4_k_m.gguf").is_err());
}

/// The whole profile is embedded in `/api/health` as `realtimeModelProfile` and
/// `realtimeModelCatalog`, so both the key names and their order are observable.
/// Order here is upstream's object-literal order
/// (`shared/realtime-model-catalog.mjs:46-53`), not alphabetical.
#[test]
fn serialized_profile_reproduces_the_upstream_key_order() {
    let profile = resolve_dashscope_realtime_model_profile(DASHSCOPE_OMNI_FLASH_REALTIME_MODEL)
        .expect("catalog id");

    assert_eq!(
        serde_json::to_string(profile).expect("profile serializes"),
        concat!(
            r#"{"id":"qwen3.5-omni-flash-realtime","#,
            r#""label":"Qwen3.5 Omni Flash Realtime","#,
            r#""family":"omni","#,
            r#""sessionDefaults":{"voice":"Ethan","turnDetection":{"type":"semantic_vad"}},"#,
            r#""modelCapabilities":{"textInput":true,"audioInput":true,"imageInput":true,"#,
            r#""videoInput":false,"textOutput":true,"audioOutput":true,"functionCalling":true},"#,
            r#""transportCapabilities":{"textInput":true,"audioInput":true,"imageInput":false,"#,
            r#""observationInput":false,"nativeVideoInput":false}}"#,
        )
    );
}

/// The audio-family profile, including the `smart_turn` wire string.
#[test]
fn serialized_audio_profile_matches() {
    let profile = resolve_dashscope_realtime_model_profile(DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL)
        .expect("catalog id");

    assert_eq!(
        serde_json::to_string(&profile.session_defaults).expect("defaults serialize"),
        r#"{"voice":"longanqian","turnDetection":{"type":"smart_turn"}}"#
    );
    assert_eq!(
        serde_json::to_string(&profile.family).expect("family serializes"),
        r#""audio""#
    );
}

/// `shared/realtime-provider-catalog.mjs:51-59` — the two voice overrides are
/// family-scoped so switching models never clobbers the other family's stored
/// preference. `via-catalog` publishes the rule; the caller reads the variable.
#[test]
fn voice_override_variables_are_family_scoped() {
    assert_eq!(
        ModelFamily::Audio.voice_override_env(),
        AUDIO_FAMILY_VOICE_ENV
    );
    assert_eq!(
        ModelFamily::Omni.voice_override_env(),
        OMNI_FAMILY_VOICE_ENV
    );
    assert_eq!(
        ModelFamily::Local.voice_override_env(),
        LOCAL_FAMILY_VOICE_ENV
    );

    // Upstream QWEN_AUDIO_REALTIME_VOICE / QWEN_OMNI_REALTIME_VOICE, renamed per
    // docs/rebrand.md, with the Audio/Omni distinction preserved.
    assert_eq!(AUDIO_FAMILY_VOICE_ENV, "VIA_REALTIME_VOICE");
    assert_eq!(OMNI_FAMILY_VOICE_ENV, "VIA_OMNI_REALTIME_VOICE");
    assert_ne!(AUDIO_FAMILY_VOICE_ENV, OMNI_FAMILY_VOICE_ENV);
}

/// The wire strings for the family discriminant.
#[test]
fn family_wire_names() {
    assert_eq!(ModelFamily::Omni.as_str(), "omni");
    assert_eq!(ModelFamily::Audio.as_str(), "audio");
    assert_eq!(ModelFamily::Local.as_str(), "local");
}

/// `server/test/realtime-provider.test.mjs:353-380` is test-locked on
/// `assert.equal(session.turn_detection.threshold, undefined)`: the turn-detection
/// object carries exactly one key.
#[test]
fn turn_detection_carries_only_a_type() {
    let value = serde_json::to_value(via_catalog::TurnDetection::new(
        TurnDetectionKind::SemanticVad,
    ))
    .expect("turn detection serializes");
    let object = value.as_object().expect("an object");
    assert_eq!(object.len(), 1);
    assert_eq!(
        object.get("type").and_then(|v| v.as_str()),
        Some("semantic_vad")
    );
    assert!(object.get("threshold").is_none());
    assert!(object.get("silence_duration_ms").is_none());
}
