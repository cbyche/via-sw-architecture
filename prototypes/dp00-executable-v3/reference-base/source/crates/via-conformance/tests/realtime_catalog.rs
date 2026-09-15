//! The realtime model and provider catalogs.
//!
//! Every model id, voice id and endpoint here goes on the wire to a vendor, and
//! the whole model profile is echoed to clients as
//! `/api/health.realtimeModelCatalog`. All of it is KEEP under
//! `docs/rebrand.md` — it names the vendor's models, not VIA.
//!
//! One deliberate divergence runs through several of these contracts and is
//! asserted rather than merely noted. Upstream's
//! `resolveDashScopeRealtimeModelProfile()` answers an unrecognised id with an
//! all-capabilities-false profile, and `providers/dashscope.mjs:84-87` gates
//! `session.turn_detection` on `transportCapabilities.audioInput` — so an
//! unknown id opens a session that hears nothing. A local model id is by
//! definition unknown to that table, so on VIA's on-device path that fallback
//! sits on the happy path. `docs/architecture.md` §7 turns it into an error;
//! `unknown_model_id_is_an_error_not_an_all_false_profile` is where that is
//! locked down, together with the upstream behaviour it replaces.

use pretty_assertions::assert_eq;
use sha2::{Digest, Sha256};
use via_catalog::realtime_model::{
    DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL, DASHSCOPE_OMNI_FLASH_REALTIME_MODEL,
    DASHSCOPE_OMNI_PLUS_REALTIME_MODEL, DASHSCOPE_OMNI_REALTIME_VOICE, ModelFamily, ModelProfile,
    TurnDetection, TurnDetectionKind,
};
use via_catalog::realtime_provider::{
    DASHSCOPE_WORKSPACE_REALTIME_URL_SUFFIX, dashscope_workspace_realtime_url,
    normalize_dashscope_base_url, normalize_speech_to_speech_base_url, realtime_url,
};
use via_catalog::{
    DEFAULT_DASHSCOPE_REALTIME_MODEL, DEFAULT_DASHSCOPE_REALTIME_URL,
    DEFAULT_DASHSCOPE_REALTIME_VOICE, DEFAULT_REALTIME_PROVIDER,
    DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL, IdentityShape, RealtimeIdentity,
    dashscope_realtime_model_profiles, realtime_providers,
    resolve_dashscope_realtime_model_profile,
};
use via_conformance::records_for;
use via_conformance::value::{after, before, js_string_array, list, quoted_literals, unquote};

/// The wire spelling of a turn-detection mode, taken from the type's own
/// `Serialize` impl so the test reads the same string the provider is sent.
fn turn_detection_name(kind: TurnDetectionKind) -> String {
    let value = serde_json::to_value(kind).expect("TurnDetectionKind serialises");
    value
        .as_str()
        .expect("TurnDetectionKind serialises as a string")
        .to_owned()
}

/// The keys of a serialised struct, in insertion order.
///
/// `serde_json` is configured with `preserve_order`, so this is the order a
/// client sees in `/api/health` and the order a hash over the object would be
/// computed in.
fn serialised_keys<T: serde::Serialize>(value: &T) -> Vec<String> {
    let json = serde_json::to_value(value).expect("value serialises");
    match json {
        serde_json::Value::Object(map) => map.keys().cloned().collect(),
        other => panic!("expected an object, got {other}"),
    }
}

#[test]
fn model_catalog_order_and_labels() {
    let contract = via_conformance::expect_contract(
        "model-id",
        "DashScope realtime model catalog (exact order)",
    );

    let Some(expected_ids) = js_string_array(&contract.exact_value) else {
        panic!(
            "the catalogue entry is not an array literal: {}",
            contract.file
        );
    };
    let expected_labels: Vec<&str> = after(&contract.exact_value, "with labels ")
        .split(',')
        .map(unquote)
        .collect();

    let shipped = dashscope_realtime_model_profiles();
    let shipped_ids: Vec<&str> = shipped.iter().map(|p| p.id.as_ref()).collect();
    let shipped_labels: Vec<&str> = shipped.iter().map(|p| p.label.as_ref()).collect();

    assert_eq!(
        expected_ids, shipped_ids,
        "the four DashScope model ids, in catalog order ({})",
        contract.file
    );
    assert_eq!(
        expected_labels, shipped_labels,
        "the four DashScope model labels, in catalog order ({})",
        contract.file
    );
    assert_eq!(shipped.len(), 4);
}

#[test]
fn model_id_constants() {
    // Each `model-id` contract is a bare id; the shipped constant must equal it
    // exactly. The pairs below are the catalogue's own names for the constants.
    let constants: &[(&str, &str)] = &[
        (
            "DEFAULT_DASHSCOPE_REALTIME_MODEL",
            DEFAULT_DASHSCOPE_REALTIME_MODEL,
        ),
        (
            "DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL",
            DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL,
        ),
        (
            "DASHSCOPE_OMNI_FLASH_REALTIME_MODEL",
            DASHSCOPE_OMNI_FLASH_REALTIME_MODEL,
        ),
        (
            "DASHSCOPE_OMNI_PLUS_REALTIME_MODEL",
            DASHSCOPE_OMNI_PLUS_REALTIME_MODEL,
        ),
    ];
    for (name, shipped) in constants {
        let records = records_for("model-id", name);
        assert!(
            !records.is_empty(),
            "docs/reference/contracts.json has no `model-id` contract named `{name}`"
        );
        for contract in records {
            assert_eq!(
                unquote(&contract.exact_value),
                *shipped,
                "{name} ({})",
                contract.file
            );
        }
    }

    // The same default is also catalogued as a `default-value`, twice — once
    // quoted from the upstream test and once bare from the module.
    let defaults = records_for("default-value", "DEFAULT_DASHSCOPE_REALTIME_MODEL");
    assert_eq!(
        defaults.len(),
        2,
        "catalogued from the module and from its test"
    );
    for contract in defaults {
        assert_eq!(
            unquote(&contract.exact_value),
            DEFAULT_DASHSCOPE_REALTIME_MODEL,
            "{}",
            contract.file
        );
    }

    // An empty or whitespace-only id resolves to the default, as upstream does.
    for spelling in ["", "   "] {
        let profile = resolve_dashscope_realtime_model_profile(spelling)
            .expect("an empty model id resolves to the default");
        assert_eq!(profile.id.as_ref(), DEFAULT_DASHSCOPE_REALTIME_MODEL);
    }
}

#[test]
fn family_session_defaults() {
    // `voice: 'longanqian'; turnDetection: { type: 'smart_turn' }`
    let audio = via_conformance::expect_contract(
        "default-value",
        "audio-family session defaults (DEFAULT_DASHSCOPE_REALTIME_VOICE)",
    );
    assert_eq!(
        quoted_literals(&audio.exact_value),
        [DEFAULT_DASHSCOPE_REALTIME_VOICE, "smart_turn"],
        "audio-family session defaults ({})",
        audio.file
    );

    // `voice: 'Ethan'; turnDetection: { type: 'semantic_vad' }`
    let omni = via_conformance::expect_contract("default-value", "omni-family session defaults");
    assert_eq!(
        quoted_literals(&omni.exact_value),
        [DASHSCOPE_OMNI_REALTIME_VOICE, "semantic_vad"],
        "omni-family session defaults ({})",
        omni.file
    );

    // …and the shipped profiles really do carry them.
    for profile in dashscope_realtime_model_profiles() {
        let (voice, turn) = match profile.family {
            ModelFamily::Audio => (DEFAULT_DASHSCOPE_REALTIME_VOICE, "smart_turn"),
            ModelFamily::Omni => (DASHSCOPE_OMNI_REALTIME_VOICE, "semantic_vad"),
            ModelFamily::Local => panic!("the DashScope catalog has no local-family profile"),
        };
        assert_eq!(profile.session_defaults.voice.as_ref(), voice);
        assert_eq!(
            turn_detection_name(profile.session_defaults.turn_detection.kind),
            turn
        );
    }
}

#[test]
fn unknown_model_id_is_an_error_not_an_all_false_profile() {
    // ---- the four profiles, as prose, in catalog order --------------------
    let profiles = via_conformance::expect_contract(
        "default-value",
        "DashScope model catalog ids and profiles",
    );
    let mut previous = 0usize;
    for profile in dashscope_realtime_model_profiles() {
        let default_marker = if profile.id == DEFAULT_DASHSCOPE_REALTIME_MODEL {
            "DEFAULT, "
        } else {
            ""
        };
        let expected = format!(
            "'{}' ({}label '{}', family '{}', voice '{}', {})",
            profile.id,
            default_marker,
            profile.label,
            profile.family.as_str(),
            profile.session_defaults.voice,
            turn_detection_name(profile.session_defaults.turn_detection.kind),
        );
        let at = profiles.exact_value.find(&expected).unwrap_or_else(|| {
            panic!(
                "`{expected}` is not in the catalogued profile list ({})",
                profiles.file
            )
        });
        assert!(at >= previous, "profiles must appear in catalog order");
        previous = at;
    }

    // ---- the constants, as the catalogue spells them ----------------------
    let catalog =
        via_conformance::expect_contract("default-value", "DashScope realtime model catalog");
    for (name, value) in [
        (
            "DEFAULT_DASHSCOPE_REALTIME_MODEL",
            DEFAULT_DASHSCOPE_REALTIME_MODEL,
        ),
        (
            "DEFAULT_DASHSCOPE_REALTIME_VOICE",
            DEFAULT_DASHSCOPE_REALTIME_VOICE,
        ),
        (
            "DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL",
            DASHSCOPE_AUDIO_FLASH_REALTIME_MODEL,
        ),
        (
            "DASHSCOPE_OMNI_FLASH_REALTIME_MODEL",
            DASHSCOPE_OMNI_FLASH_REALTIME_MODEL,
        ),
        (
            "DASHSCOPE_OMNI_PLUS_REALTIME_MODEL",
            DASHSCOPE_OMNI_PLUS_REALTIME_MODEL,
        ),
    ] {
        let assignment = format!("{name}='{value}'");
        assert!(
            catalog.exact_value.contains(&assignment),
            "`{assignment}` is not in the catalogued constant list ({})",
            catalog.file
        );
    }

    // ---- turn detection by family, and the fields never sent --------------
    let turn = via_conformance::expect_contract("default-value", "turn detection by model family");
    assert_eq!(
        quoted_literals(&turn.exact_value),
        ["smart_turn", "semantic_vad"],
        "audio -> smart_turn, omni -> semantic_vad ({})",
        turn.file
    );
    assert!(
        turn.exact_value
            .contains("No threshold / silence_duration_ms fields are ever sent"),
        "the catalogue still records that turn_detection carries only `type`"
    );
    // `TurnDetection` has exactly one field, so neither can be sent.
    assert_eq!(
        serde_json::to_string(&TurnDetection::new(TurnDetectionKind::SmartTurn))
            .expect("TurnDetection serialises"),
        r#"{"type":"smart_turn"}"#
    );
    assert_eq!(
        serde_json::to_string(&TurnDetection::new(TurnDetectionKind::SemanticVad))
            .expect("TurnDetection serialises"),
        r#"{"type":"semantic_vad"}"#
    );

    // ---- the ordered session-default pairs --------------------------------
    let session =
        via_conformance::expect_contract("default-value", "model profile session defaults");
    assert_eq!(
        quoted_literals(&session.exact_value),
        [
            DASHSCOPE_OMNI_REALTIME_VOICE,
            "semantic_vad",
            DEFAULT_DASHSCOPE_REALTIME_VOICE,
            "smart_turn"
        ],
        "omni first, then audio ({})",
        session.file
    );

    // ---- the divergence itself --------------------------------------------
    // Every one of the four contracts above ends with upstream's unknown-id
    // fallback. VIA replaces it with an error, per docs/architecture.md §7.
    for contract in [profiles, catalog, session] {
        assert!(
            contract.exact_value.contains("unknown"),
            "the catalogue still records upstream's unknown-id fallback ({})",
            contract.file
        );
    }
    for unknown in [
        "not-a-model",
        // Capabilities are never inferred from the shape of an id: an id that
        // looks like a future omni model is still unknown.
        "qwen3.5-omni-plus-realtime-future",
        // A local model id, which is what made the upstream fallback dangerous.
        "Qwen3-30B-A3B-Q4_K_M.gguf",
    ] {
        assert!(
            resolve_dashscope_realtime_model_profile(unknown).is_err(),
            "`{unknown}` must be an error, not an all-capabilities-false profile"
        );
    }
    // And the replacement really does give the on-device path audio input,
    // which is the flag upstream gated `session.turn_detection` on.
    let local = via_catalog::local_realtime_model_profile("Qwen3-30B-A3B-Q4_K_M.gguf");
    assert_eq!(local.family, ModelFamily::Local);
    assert!(local.transport_capabilities.audio_input);
    assert!(local.model_capabilities.audio_input);
}

#[test]
fn capability_flag_names_and_order() {
    let flags =
        via_conformance::expect_contract("json-field", "model profile capability flag sets");
    let expected_model = list(before(
        after(&flags.exact_value, "modelCapabilities keys:"),
        ".",
    ));
    let expected_transport = list(before(
        after(&flags.exact_value, "transportCapabilities keys:"),
        ".",
    ));

    let profile: &ModelProfile = &dashscope_realtime_model_profiles()[0];
    assert_eq!(
        serialised_keys(&profile.model_capabilities),
        expected_model,
        "modelCapabilities field names and order ({})",
        flags.file
    );
    assert_eq!(
        serialised_keys(&profile.transport_capabilities),
        expected_transport,
        "transportCapabilities field names and order ({})",
        flags.file
    );
    assert_eq!(expected_model.len(), 7);
    assert_eq!(expected_transport.len(), 5);

    // The same two key sets are catalogued a second time, from the upstream
    // test that locks the fail-closed unknown-id behaviour.
    let shape = via_conformance::expect_contract(
        "json-field",
        "model capability shape (fail-closed for unknown ids)",
    );
    let shape_model = list(
        before(after(&shape.exact_value, "modelCapabilities {"), "}")
            .trim_matches(|c| c == '{' || c == '}'),
    );
    let shape_transport = list(
        before(after(&shape.exact_value, "transportCapabilities {"), "}")
            .trim_matches(|c| c == '{' || c == '}'),
    );
    assert_eq!(
        shape_model, expected_model,
        "the two catalogue records agree"
    );
    assert_eq!(
        shape_transport, expected_transport,
        "the two catalogue records agree"
    );

    // "Omni model caps: text/audio/image in true, video false, all outputs
    // true, functionCalling true. Omni transport caps: text+audio true, rest
    // false. Audio (legacy) model caps: image false."
    for profile in dashscope_realtime_model_profiles() {
        let caps = profile.model_capabilities;
        assert!(caps.text_input && caps.audio_input);
        assert!(!caps.video_input);
        assert!(caps.text_output && caps.audio_output && caps.function_calling);
        assert_eq!(
            caps.image_input,
            profile.family == ModelFamily::Omni,
            "only the omni family takes image input ({})",
            profile.id
        );

        let transport = profile.transport_capabilities;
        assert!(transport.text_input && transport.audio_input);
        assert!(!transport.image_input);
        assert!(!transport.observation_input);
        assert!(!transport.native_video_input);
    }

    // The divergence: `family: 'unknown'` with every flag false has no
    // representation here, because there is no unknown family to represent.
    assert!(
        shape
            .exact_value
            .contains("every field is false and family is 'unknown'"),
        "the catalogue still records the upstream fallback ({})",
        shape.file
    );
    assert!(resolve_dashscope_realtime_model_profile("not-a-model").is_err());
}

#[test]
fn provider_keys_labels_and_aliases() {
    // `dashscope (label 'DashScope', aliases ['qwen']); speech-to-speech
    //  (label 'Hugging Face Speech-to-Speech', aliases ['s2s'])`
    let contract =
        via_conformance::expect_contract("provider-id", "realtime provider keys and aliases");
    let literals = quoted_literals(&contract.exact_value);
    assert_eq!(
        literals,
        ["DashScope", "qwen", "Hugging Face Speech-to-Speech", "s2s"],
        "labels and aliases, in upstream order ({})",
        contract.file
    );

    // VIA ships five providers; upstream's two stay first and unchanged, so
    // the ordered name list keeps upstream's prefix (docs/architecture.md §7).
    let providers = realtime_providers();
    assert!(providers.len() >= 2);
    assert_eq!(providers[0].key, "dashscope");
    assert_eq!(providers[0].label, literals[0]);
    assert_eq!(providers[0].aliases, [literals[1]]);
    assert_eq!(providers[1].key, "speech-to-speech");
    assert_eq!(providers[1].label, literals[2]);
    assert_eq!(providers[1].aliases, [literals[3]]);
    assert_eq!(DEFAULT_REALTIME_PROVIDER, "dashscope");

    // The alias half of `provider aliases and configured gating`. The
    // `isConfigured` half is `via-realtime`'s, and this contract stays partial
    // until that crate lands.
    let gating =
        via_conformance::expect_contract("default-value", "provider aliases and configured gating");
    for expected in [
        "dashscope key 'dashscope', aliases ['qwen']",
        "speech-to-speech key 'speech-to-speech', aliases ['s2s']",
    ] {
        assert!(
            gating.exact_value.contains(expected),
            "`{expected}` is not in the catalogued gating rule ({})",
            gating.file
        );
    }
    // KEEP per docs/rebrand.md: `qwen` names the vendor's realtime runtime.
    assert_eq!(
        via_catalog::normalize_realtime_provider("qwen").expect("the alias resolves"),
        "dashscope"
    );
    assert_eq!(
        via_catalog::normalize_realtime_provider("S2S").expect("the alias resolves"),
        "speech-to-speech"
    );
}

#[test]
fn endpoint_defaults_and_url_construction() {
    let dashscope =
        via_conformance::expect_contract("http-route", "DEFAULT_DASHSCOPE_REALTIME_URL");
    assert_eq!(
        unquote(&dashscope.exact_value),
        DEFAULT_DASHSCOPE_REALTIME_URL,
        "{}",
        dashscope.file
    );

    let s2s =
        via_conformance::expect_contract("http-route", "DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL");
    assert_eq!(
        unquote(&s2s.exact_value),
        DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL,
        "{}",
        s2s.file
    );

    // `wss://${DASHSCOPE_WORKSPACE_ID}.cn-beijing.maas.aliyuncs.com/api-ws/v1/realtime`
    let workspace =
        via_conformance::expect_contract("http-route", "DashScope workspace endpoint template");
    let template = unquote(&workspace.exact_value);
    assert!(
        template.ends_with(DASHSCOPE_WORKSPACE_REALTIME_URL_SUFFIX),
        "the shipped suffix must be the template's tail ({})",
        workspace.file
    );
    assert_eq!(
        template.replace("${DASHSCOPE_WORKSPACE_ID}", "llm-abc123"),
        dashscope_workspace_realtime_url("llm-abc123"),
        "the workspace endpoint, with the id substituted ({})",
        workspace.file
    );

    // The `default-value` record restates all three plus the URL construction.
    let construction =
        via_conformance::expect_contract("default-value", "endpoint defaults and URL construction");
    for expected in [
        format!("DEFAULT_DASHSCOPE_REALTIME_URL = '{DEFAULT_DASHSCOPE_REALTIME_URL}'"),
        format!(
            "DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL = '{DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL}'"
        ),
        format!("workspace form = `{template}`"),
    ] {
        assert!(
            construction.exact_value.contains(&expected),
            "`{expected}` is not in the catalogued endpoint rule ({})",
            construction.file
        );
    }

    // `realtimeUrl(base, model) = `${base}${base.includes('?') ? '&' : '?'}model=${encodeURIComponent(model)}``
    assert_eq!(
        realtime_url("wss://host/v1/realtime", DEFAULT_DASHSCOPE_REALTIME_MODEL),
        format!("wss://host/v1/realtime?model={DEFAULT_DASHSCOPE_REALTIME_MODEL}")
    );
    assert_eq!(
        realtime_url("wss://host/v1/realtime?x=1", "a b"),
        "wss://host/v1/realtime?x=1&model=a%20b"
    );
    // `encodeURIComponent` leaves `- _ . ! ~ * ' ( )` alone and uppercases hex.
    assert_eq!(
        realtime_url("wss://h", "a-_.!~*'()/b"),
        "wss://h?model=a-_.!~*'()%2Fb"
    );

    // "base URL has trailing '?' stripped (dashscope) / trailing '/' stripped (s2s)"
    assert_eq!(
        normalize_dashscope_base_url("  wss://host/v1/realtime??  "),
        "wss://host/v1/realtime"
    );
    assert_eq!(
        normalize_speech_to_speech_base_url("  ws://host/v1/realtime//  "),
        "ws://host/v1/realtime"
    );
}

#[test]
fn signature_key_order() {
    // `sha256(JSON.stringify({provider, endpoint, model, voice, credential}))`
    let dashscope = via_conformance::expect_contract(
        "json-field",
        "realtime configuration signature input (dashscope branch)",
    );
    let expected_keys = list(before(
        after(&dashscope.exact_value, "JSON.stringify({"),
        "}",
    ));
    assert_eq!(
        expected_keys,
        ["provider", "endpoint", "model", "voice", "credential"],
        "the hashed key order ({})",
        dashscope.file
    );

    let identity = RealtimeIdentity::new(
        IdentityShape::ModelAndVoice,
        "dashscope",
        DEFAULT_DASHSCOPE_REALTIME_URL,
        DEFAULT_DASHSCOPE_REALTIME_MODEL,
        DEFAULT_DASHSCOPE_REALTIME_VOICE,
        "sk-secret",
    );
    let json = identity.canonical_json();
    assert_eq!(
        json,
        format!(
            r#"{{"provider":"dashscope","endpoint":"{DEFAULT_DASHSCOPE_REALTIME_URL}","model":"{DEFAULT_DASHSCOPE_REALTIME_MODEL}","voice":"{DEFAULT_DASHSCOPE_REALTIME_VOICE}","credential":"sk-secret"}}"#
        ),
        "JSON.stringify emits own string keys in insertion order; sorting them \
         would publish a signature no client could reproduce"
    );
    // Re-derived here rather than taken from the implementation, so this is not
    // a test of `signature()` against itself.
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    assert_eq!(identity.signature(), hex::encode(hasher.finalize()));
    assert_eq!(identity.signature().len(), 64);
    assert!(identity.signature().bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(
        identity.signature(),
        identity.signature().to_lowercase(),
        "the digest is published as lowercase hex"
    );

    // "the model and voice keys are ABSENT, not null"
    let s2s_contract = via_conformance::expect_contract(
        "json-field",
        "realtime configuration signature input (speech-to-speech branch)",
    );
    let s2s_keys = list(before(
        after(&s2s_contract.exact_value, "JSON.stringify({"),
        "}",
    ));
    assert_eq!(
        s2s_keys,
        ["provider", "endpoint", "credential"],
        "the speech-to-speech branch hashes three keys ({})",
        s2s_contract.file
    );

    let s2s = RealtimeIdentity::new(
        IdentityShape::EndpointOnly,
        "speech-to-speech",
        DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL,
        "ignored",
        "ignored",
        "token",
    );
    let s2s_json = s2s.canonical_json();
    assert_eq!(
        s2s_json,
        format!(
            r#"{{"provider":"speech-to-speech","endpoint":"{DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL}","credential":"token"}}"#
        )
    );
    assert!(!s2s_json.contains("model"), "`model` is absent, not null");
    assert!(!s2s_json.contains("voice"), "`voice` is absent, not null");
    assert_ne!(
        s2s.signature(),
        identity.signature(),
        "two shapes, two digests"
    );
}
