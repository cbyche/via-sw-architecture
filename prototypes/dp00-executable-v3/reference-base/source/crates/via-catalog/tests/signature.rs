//! Contract assertions for the realtime configuration signature.
//!
//! # Why field order matters, and why these digests are the test
//!
//! Upstream computes the signature as
//!
//! ```js
//! const signature = createHash('sha256')
//!   .update(JSON.stringify(identity))
//!   .digest('hex')
//! ```
//!
//! over an object *literal* (`shared/realtime-provider-catalog.mjs:122-137`).
//! `JSON.stringify` emits own string keys in **insertion order**, so the bytes
//! that are hashed are `{"provider":…,"endpoint":…,"model":…,"voice":…,"credential":…}`
//! — in that order, because that is the order the literal was written in. The
//! digest is published on `/api/health` as `realtimeConfigurationSignature` and
//! every client recomputes it locally to decide whether the gateway it reached is
//! the one it configured (`cli/src/runtime.mjs:188`,
//! `desktop/src/gateway-process.mjs:99`).
//!
//! Reordering the keys therefore does not "look different" — it silently breaks
//! every client, because the gateway publishes a digest nobody can reproduce and
//! every client reports the voice configuration as permanently changed. A
//! `HashMap` reorders by hash; a `BTreeMap`-backed `serde_json::Map` reorders
//! alphabetically. Both are wrong, and neither fails loudly. Hence
//! `serde_json`'s `preserve_order` feature in the workspace manifest, and hence
//! [`order_is_load_bearing`], which pins the alternative digest so a regression
//! names its own cause.
//!
//! The second trap is shape: the speech-to-speech branch has **three** keys, not
//! five with two nulls. [`absent_is_not_null`] pins that too.
//!
//! Every expected digest below was produced by running upstream's own expression
//! against the same input under Node 26:
//!
//! ```text
//! node -e "const {createHash}=require('node:crypto');
//!          console.log(createHash('sha256').update(JSON.stringify(o)).digest('hex'))"
//! ```

use pretty_assertions::assert_eq;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use via_catalog::realtime_provider::IdentityShape;
use via_catalog::signature::RealtimeIdentity;

const DASHSCOPE_ENDPOINT: &str = "wss://dashscope.aliyuncs.com/api-ws/v1/realtime";
const S2S_ENDPOINT: &str = "ws://127.0.0.1:8765/v1/realtime";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// The dashscope branch: five keys, `provider` first, `credential` last.
///
/// `JSON.stringify({provider:'dashscope', endpoint:'wss://dashscope.aliyuncs.com/api-ws/v1/realtime',
/// model:'qwen-audio-3.0-realtime-plus', voice:'', credential:''})`
#[test]
fn dashscope_branch_reproduces_the_upstream_digest() {
    let identity = RealtimeIdentity::new(
        IdentityShape::ModelAndVoice,
        "dashscope",
        DASHSCOPE_ENDPOINT,
        "qwen-audio-3.0-realtime-plus",
        "",
        "",
    );

    assert_eq!(
        identity.canonical_json(),
        concat!(
            r#"{"provider":"dashscope","#,
            r#""endpoint":"wss://dashscope.aliyuncs.com/api-ws/v1/realtime","#,
            r#""model":"qwen-audio-3.0-realtime-plus","#,
            r#""voice":"","credential":""}"#,
        )
    );
    assert_eq!(
        identity.signature(),
        "930efcc3c446a3d5c8edde015050d9de84e10c7c3e50aac79efcc87aca67f854"
    );
}

/// A configured key and a family-scoped voice override change the digest, which
/// is the entire point of the mechanism.
#[test]
fn dashscope_digests_track_every_field() {
    let configured = RealtimeIdentity::new(
        IdentityShape::ModelAndVoice,
        "dashscope",
        DASHSCOPE_ENDPOINT,
        "qwen-audio-3.0-realtime-plus",
        "",
        "sk-test-key",
    );
    assert_eq!(
        configured.signature(),
        "1b406bbd17d60b4b80c5af61ebc6f53dc9674fa827c6070ef331eaeb16b86988"
    );

    let omni = RealtimeIdentity::new(
        IdentityShape::ModelAndVoice,
        "dashscope",
        DASHSCOPE_ENDPOINT,
        "qwen3.5-omni-plus-realtime",
        "custom-omni",
        "sk-test-key",
    );
    assert_eq!(
        omni.signature(),
        "5af51a29f48b8394256df3c58e358765d5e2cc82ed8ef40da92915bf7a3513e7"
    );
    assert_ne!(configured.signature(), omni.signature());
}

/// The speech-to-speech branch: three keys. `model` and `voice` never appear.
#[test]
fn speech_to_speech_branch_reproduces_the_upstream_digest() {
    let identity = RealtimeIdentity::new(
        IdentityShape::EndpointOnly,
        "speech-to-speech",
        S2S_ENDPOINT,
        // Model and voice are supplied and must be ignored — upstream's branch
        // simply never reads them.
        "qwen-audio-3.0-realtime-plus",
        "longanqian",
        "",
    );

    assert_eq!(
        identity.canonical_json(),
        r#"{"provider":"speech-to-speech","endpoint":"ws://127.0.0.1:8765/v1/realtime","credential":""}"#
    );
    assert_eq!(
        identity.signature(),
        "ee75e5baf0f689db401781e84360ca72b6ffb09dbed0818dc34c2186d81fda7d"
    );

    let with_token = RealtimeIdentity::new(
        IdentityShape::EndpointOnly,
        "speech-to-speech",
        S2S_ENDPOINT,
        "",
        "",
        "s2s-token",
    );
    assert_eq!(
        with_token.signature(),
        "aeb7162fb4720c3b0d8d3535eeeae95193409caf81bec27c3a72a7cebb9bae77"
    );
}

/// **Key order is load-bearing.**
///
/// The same five values in alphabetical-ish order hash to something else
/// entirely. This test hashes a deliberately reordered map by hand and pins the
/// result, so that a future change to how the identity is serialized fails here
/// with a named cause rather than by mysteriously not matching a client.
#[test]
fn order_is_load_bearing() {
    let identity = RealtimeIdentity::new(
        IdentityShape::ModelAndVoice,
        "dashscope",
        DASHSCOPE_ENDPOINT,
        "qwen-audio-3.0-realtime-plus",
        "",
        "",
    );

    // `endpoint` before `provider`; every value identical.
    let mut reordered = Map::new();
    reordered.insert("endpoint".into(), Value::String(DASHSCOPE_ENDPOINT.into()));
    reordered.insert("provider".into(), Value::String("dashscope".into()));
    reordered.insert(
        "model".into(),
        Value::String("qwen-audio-3.0-realtime-plus".into()),
    );
    reordered.insert("voice".into(), Value::String(String::new()));
    reordered.insert("credential".into(), Value::String(String::new()));
    let reordered_json = serde_json::to_string(&Value::Object(reordered)).expect("map serializes");

    // Same key set, same values, different digest.
    assert_ne!(reordered_json, identity.canonical_json());
    assert_eq!(
        sha256_hex(reordered_json.as_bytes()),
        "db11ed0ed00ba7fb53cab36383560c92b294a3116988aded66dea2b9ad20f6dc"
    );
    assert_ne!(sha256_hex(reordered_json.as_bytes()), identity.signature());
}

/// **Absent is not null.**
///
/// Modelling the identity as one struct with `Option<String>` model and voice
/// fields would emit `"model":null,"voice":null` for the speech-to-speech branch
/// and produce this digest instead of the real one. The enum in
/// [`RealtimeIdentity`] makes that shape unconstructible; this test pins the
/// wrong answer so the distinction stays visible.
#[test]
fn absent_is_not_null() {
    let correct = RealtimeIdentity::new(
        IdentityShape::EndpointOnly,
        "speech-to-speech",
        S2S_ENDPOINT,
        "",
        "",
        "",
    );

    let mut with_nulls = Map::new();
    with_nulls.insert("provider".into(), Value::String("speech-to-speech".into()));
    with_nulls.insert("endpoint".into(), Value::String(S2S_ENDPOINT.into()));
    with_nulls.insert("model".into(), Value::Null);
    with_nulls.insert("voice".into(), Value::Null);
    with_nulls.insert("credential".into(), Value::String(String::new()));
    let with_nulls_json =
        serde_json::to_string(&Value::Object(with_nulls)).expect("map serializes");

    assert_eq!(
        sha256_hex(with_nulls_json.as_bytes()),
        "32c654d3a113ab082e9ae28b63375800f89a09784ee1c6622a494baea651378b"
    );
    assert_ne!(sha256_hex(with_nulls_json.as_bytes()), correct.signature());
}

/// `serde_json`'s `preserve_order` feature is a workspace-level setting, so a
/// well-meaning cleanup of the root manifest could remove it. Without it a
/// `serde_json::Map` sorts its keys and every digest above changes. Assert the
/// feature directly rather than only through its consequences.
#[test]
fn serde_json_preserves_insertion_order() {
    let mut map = Map::new();
    map.insert("zulu".into(), Value::from(1));
    map.insert("alpha".into(), Value::from(2));
    map.insert("mike".into(), Value::from(3));

    assert_eq!(
        serde_json::to_string(&Value::Object(map)).expect("map serializes"),
        r#"{"zulu":1,"alpha":2,"mike":3}"#,
        "serde_json lost insertion order — the `preserve_order` feature is off, \
         and every configuration signature is now wrong"
    );
}

/// The identity JSON is a plain object of strings, so the canonical form has no
/// whitespace and matches `JSON.stringify` byte for byte, including leaving `/`
/// unescaped and emitting non-ASCII as raw UTF-8.
#[test]
fn canonical_json_matches_json_stringify_escaping() {
    let identity = RealtimeIdentity::new(
        IdentityShape::EndpointOnly,
        "mock",
        "ws://127.0.0.1:1/路径",
        "",
        "",
        "",
    );
    assert_eq!(
        identity.canonical_json(),
        r#"{"provider":"mock","endpoint":"ws://127.0.0.1:1/路径","credential":""}"#
    );
}
