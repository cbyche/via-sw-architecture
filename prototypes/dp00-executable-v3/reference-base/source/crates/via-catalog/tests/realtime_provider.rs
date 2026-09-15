//! Contract assertions for the realtime provider registry.
//!
//! Values from `shared/realtime-provider-catalog.mjs:18-33,61-118` and
//! `server/src/core/config.mjs:509-512`.

use pretty_assertions::assert_eq;
use via_catalog::realtime_provider::{
    DASHSCOPE_WORKSPACE_REALTIME_URL_SUFFIX, DEFAULT_DASHSCOPE_REALTIME_URL,
    DEFAULT_OPENAI_REALTIME_URL, DEFAULT_REALTIME_PROVIDER, DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL,
    IdentityShape, dashscope_workspace_realtime_url, normalize_dashscope_base_url,
    normalize_realtime_provider, normalize_realtime_provider_with_fallback,
    normalize_speech_to_speech_base_url, realtime_provider_definition, realtime_provider_names,
    realtime_providers, realtime_url,
};

/// `shared/realtime-provider-catalog.mjs:22-33`. The two upstream providers,
/// with their exact keys, labels and alias lists.
#[test]
fn upstream_providers_are_reproduced_exactly() {
    let dashscope = realtime_provider_definition("dashscope").expect("registered");
    assert_eq!(dashscope.key, "dashscope");
    assert_eq!(dashscope.label, "DashScope");
    assert_eq!(dashscope.aliases, &["qwen"]);

    let s2s = realtime_provider_definition("speech-to-speech").expect("registered");
    assert_eq!(s2s.key, "speech-to-speech");
    assert_eq!(s2s.label, "Hugging Face Speech-to-Speech");
    assert_eq!(s2s.aliases, &["s2s"]);
}

/// `docs/rebrand.md` KEEP: `qwen` names the vendor's realtime runtime, clients
/// still send `provider: "qwen"` on the connect event, and existing configs
/// contain it. Renaming it would break both.
#[test]
fn the_qwen_alias_is_kept() {
    assert_eq!(
        normalize_realtime_provider("qwen").expect("alias"),
        "dashscope"
    );
    assert_eq!(
        normalize_realtime_provider("QWEN").expect("alias"),
        "dashscope"
    );
    assert_eq!(
        normalize_realtime_provider("  Qwen  ").expect("alias"),
        "dashscope"
    );
    assert_eq!(
        normalize_realtime_provider("s2s").expect("alias"),
        "speech-to-speech"
    );
    assert_eq!(
        normalize_realtime_provider("S2S").expect("alias"),
        "speech-to-speech"
    );
}

/// `shared/realtime-provider-catalog.mjs:65-77`: the fallback substitutes for a
/// *falsy* value, before trimming — so an empty string becomes the default while
/// a whitespace-only string is a hard error.
#[test]
fn empty_falls_back_but_whitespace_does_not() {
    assert_eq!(DEFAULT_REALTIME_PROVIDER, "dashscope");
    assert_eq!(
        normalize_realtime_provider("").expect("default"),
        "dashscope"
    );
    assert_eq!(
        normalize_realtime_provider_with_fallback("", "s2s").expect("explicit fallback"),
        "speech-to-speech"
    );
    assert!(normalize_realtime_provider("   ").is_err());
}

/// An unrecognised provider is refused, including one selected by a client on the
/// connect event.
#[test]
fn an_unknown_provider_is_refused() {
    let error = normalize_realtime_provider("gemini").expect_err("not registered");
    assert_eq!(error.code(), "VIA_REALTIME_PROVIDER_UNSUPPORTED");
    assert_eq!(
        error,
        via_catalog::CatalogError::UnsupportedRealtimeProvider {
            requested: "gemini".to_owned(),
        }
    );
}

/// VIA ships five providers (`docs/architecture.md` §7). The two upstream entries
/// stay first, so the ordered name list keeps upstream's prefix.
#[test]
fn via_extends_the_registry_without_reordering_it() {
    assert_eq!(
        realtime_provider_names(),
        vec![
            "dashscope",
            "speech-to-speech",
            "openai",
            "local-omni",
            "mock",
        ]
    );
    assert_eq!(realtime_providers().len(), 5);

    // The three additions carry no aliases: an alias is a compatibility promise
    // to existing configuration, and these have no existing configuration.
    for key in ["openai", "local-omni", "mock"] {
        let provider = realtime_provider_definition(key).expect("registered");
        assert!(provider.aliases.is_empty(), "{key} declares an alias");
    }
}

/// Which identity shape each provider hashes. The two upstream branches are
/// contract; the three additions reuse whichever branch fits.
#[test]
fn identity_shapes_match_the_upstream_branches() {
    let shape = |key: &str| {
        realtime_provider_definition(key)
            .expect("registered")
            .identity_shape
    };
    assert_eq!(shape("dashscope"), IdentityShape::ModelAndVoice);
    assert_eq!(shape("speech-to-speech"), IdentityShape::EndpointOnly);
    assert_eq!(shape("openai"), IdentityShape::ModelAndVoice);
    assert_eq!(shape("local-omni"), IdentityShape::ModelAndVoice);
    assert_eq!(shape("mock"), IdentityShape::EndpointOnly);
}

/// `shared/realtime-provider-catalog.mjs:19-20` and `:94`. Vendor endpoints —
/// KEEP.
#[test]
fn default_endpoints_are_exact() {
    assert_eq!(
        DEFAULT_DASHSCOPE_REALTIME_URL,
        "wss://dashscope.aliyuncs.com/api-ws/v1/realtime"
    );
    assert_eq!(
        DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL,
        "ws://127.0.0.1:8765/v1/realtime"
    );
    assert_eq!(
        DASHSCOPE_WORKSPACE_REALTIME_URL_SUFFIX,
        ".cn-beijing.maas.aliyuncs.com/api-ws/v1/realtime"
    );
    assert_eq!(
        dashscope_workspace_realtime_url("llm-abc123"),
        "wss://llm-abc123.cn-beijing.maas.aliyuncs.com/api-ws/v1/realtime"
    );

    // Ported from ARGO rather than from qwen: openai_live.rs's candidate walk
    // starts here.
    assert_eq!(
        DEFAULT_OPENAI_REALTIME_URL,
        "wss://api.openai.com/v1/realtime"
    );

    assert_eq!(
        realtime_provider_definition("dashscope")
            .expect("registered")
            .default_url,
        Some(DEFAULT_DASHSCOPE_REALTIME_URL)
    );
    assert_eq!(
        realtime_provider_definition("speech-to-speech")
            .expect("registered")
            .default_url,
        Some(DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL)
    );
    // `local-omni:pipeline` needs no endpoint and `:endpoint` must be told one;
    // `mock` replays from a fixture. Neither gets a default.
    assert_eq!(
        realtime_provider_definition("local-omni")
            .expect("registered")
            .default_url,
        None
    );
    assert_eq!(
        realtime_provider_definition("mock")
            .expect("registered")
            .default_url,
        None
    );
}

/// `shared/realtime-provider-catalog.mjs:89-110`: the DashScope base URL has
/// trailing `?` stripped and the speech-to-speech base URL has trailing `/`
/// stripped, both after trimming.
#[test]
fn base_urls_are_normalized_per_provider() {
    assert_eq!(
        normalize_dashscope_base_url("  wss://host/api-ws/v1/realtime??  "),
        "wss://host/api-ws/v1/realtime"
    );
    assert_eq!(
        normalize_dashscope_base_url("wss://host/api-ws/v1/realtime"),
        "wss://host/api-ws/v1/realtime"
    );
    // A trailing slash is meaningful to DashScope and is left alone.
    assert_eq!(
        normalize_dashscope_base_url("wss://host/realtime/"),
        "wss://host/realtime/"
    );

    assert_eq!(
        normalize_speech_to_speech_base_url(" ws://127.0.0.1:8765/v1/realtime// "),
        "ws://127.0.0.1:8765/v1/realtime"
    );
    // Symmetrically, a trailing `?` is left alone for speech-to-speech.
    assert_eq!(
        normalize_speech_to_speech_base_url("ws://host/realtime?"),
        "ws://host/realtime?"
    );
}

/// `server/src/core/config.mjs:509-512` —
/// `` `${base}${base.includes('?') ? '&' : '?'}model=${encodeURIComponent(model)}` ``.
#[test]
fn realtime_url_appends_the_model_as_a_query_parameter() {
    assert_eq!(
        realtime_url(
            DEFAULT_DASHSCOPE_REALTIME_URL,
            "qwen-audio-3.0-realtime-plus"
        ),
        "wss://dashscope.aliyuncs.com/api-ws/v1/realtime?model=qwen-audio-3.0-realtime-plus"
    );
    // An existing query string switches the separator to `&`.
    assert_eq!(
        realtime_url(
            "wss://gateway.internal/v1/realtime?api-version=2026-01-01",
            "m"
        ),
        "wss://gateway.internal/v1/realtime?api-version=2026-01-01&model=m"
    );
    // encodeURIComponent's unreserved set is `A-Z a-z 0-9 - _ . ! ~ * ' ( )`;
    // everything else is percent-encoded over UTF-8 with uppercase hex.
    assert_eq!(realtime_url("wss://h/r", "a b"), "wss://h/r?model=a%20b");
    assert_eq!(realtime_url("wss://h/r", "a/b"), "wss://h/r?model=a%2Fb");
    assert_eq!(
        realtime_url("wss://h/r", "a-_.!~*'()"),
        "wss://h/r?model=a-_.!~*'()"
    );
    assert_eq!(
        realtime_url("wss://h/r", "模型"),
        "wss://h/r?model=%E6%A8%A1%E5%9E%8B"
    );
}
