//! `via-voice`-owned contracts whose real comparison needs `via-app`.
//!
//! `docs/deviations/phase-5-via-app.md` splits upstream's monolithic
//! `realtime-gateway.mjs` in two: the socket plumbing (which frame carries
//! which fields, how it is encoded) is `via-app`'s, and the decisions that
//! plumbing makes are `via-voice`'s. A handful of catalogued rows are the
//! *wire payload* half of that split — `via-voice` owns the value
//! conceptually (`docs/architecture.md` §9 band order), but the struct that
//! actually serializes it lives one band up. This crate is the only place
//! that can see both sides without either crate depending on the other for
//! anything that ships, so the comparisons that need both live here rather
//! than being asserted (incompletely) from either side alone.
//!
//! Everything here calls a real, pure function from `via-app`'s frame codec —
//! no mock server, no socket — so a rename or a dropped field fails this
//! test rather than silently drifting from the catalogue.

use via_app::realtime::clients::deactivation_frames;
use via_app::realtime::frames::{ClientDescriptor, ClientFrame, voice_ownership, voice_ready};
use via_conformance::expect_contract;
use via_protocol::ClientType;
use via_voice::input::{
    InputPart, MAX_LABEL_CHARS, MAX_SOURCE_CHARS, input_part_label, unique_attachment_references,
};

#[test]
fn attachment_reference_placeholders_match_the_catalogue() {
    let contract = expect_contract("default-value", "attachment reference placeholders");
    assert!(contract.exact_value.contains("[Image N]"));
    assert!(contract.exact_value.contains("@<filename>"));
    assert!(contract.exact_value.contains("[File N]"));
    assert!(contract.exact_value.contains("120 chars"));
    assert!(contract.exact_value.contains("2048"));
    assert_eq!(MAX_LABEL_CHARS, 120);
    assert_eq!(MAX_SOURCE_CHARS, 2048);

    let image = InputPart::File {
        mime: "image/png".to_owned(),
        filename: None,
        url: "https://example.com/a.png".to_owned(),
        source: None,
        meta: None,
    };
    let named = InputPart::File {
        mime: "application/pdf".to_owned(),
        filename: Some("report.pdf".to_owned()),
        url: "https://example.com/r.pdf".to_owned(),
        source: None,
        meta: None,
    };
    let unnamed = InputPart::File {
        mime: "application/pdf".to_owned(),
        filename: None,
        url: "https://example.com/r2.pdf".to_owned(),
        source: None,
        meta: None,
    };
    assert_eq!(input_part_label(&image, 0), "[Image 1]");
    assert_eq!(input_part_label(&named, 0), "@report.pdf");
    assert_eq!(input_part_label(&unnamed, 0), "[File 1]");

    // Duplicates resolve by incrementing the ordinal rather than colliding:
    // two files sharing a filename produce the same natural `@notes.pdf`
    // anchor regardless of position, so the second one falls back to
    // `[File N]` instead.
    let same_name = InputPart::File {
        mime: "application/pdf".to_owned(),
        filename: Some("notes.pdf".to_owned()),
        url: "https://example.com/n1.pdf".to_owned(),
        source: None,
        meta: None,
    };
    assert_eq!(
        unique_attachment_references(&[same_name.clone(), same_name]),
        vec!["@notes.pdf".to_owned(), "[File 2]".to_owned()],
    );
}

#[test]
fn voice_ready_carries_exactly_the_catalogued_three_fields() {
    let contract = expect_contract("json-field", "voice.ready");
    assert!(contract.exact_value.contains("inputSampleRate"));
    assert!(contract.exact_value.contains("\"provider\""));
    assert!(contract.exact_value.contains("providerLabel"));

    let frame = voice_ready(16_000, "local", "On-device").into_value();
    let object = frame.as_object().expect("an object");
    assert_eq!(
        object.keys().collect::<Vec<_>>(),
        vec!["type", "inputSampleRate", "provider", "providerLabel"],
        "field order ({})",
        contract.file,
    );
    assert_eq!(object["type"], "voice.ready");
    assert_eq!(object["inputSampleRate"], 16_000);
    assert_eq!(object["provider"], "local");
    assert_eq!(object["providerLabel"], "On-device");
}

#[test]
fn voice_ownership_and_voice_deactivated_match_the_catalogued_shapes() {
    let contract = expect_contract("json-field", "voice.ownership / voice.deactivated");
    for state in ["active", "busy", "available"] {
        assert!(
            contract.exact_value.contains(state),
            "the catalogue still lists {state} ({})",
            contract.file,
        );
    }

    // `voice.ownership` with a holder.
    let holder = ClientDescriptor {
        client_type: ClientType::Desktop,
        label: Some("kitchen".to_owned()),
        instance_id: Some("win-1".to_owned()),
    };
    let frame = voice_ownership("active", Some(&holder)).into_value();
    assert_eq!(frame["type"], "voice.ownership");
    assert_eq!(frame["state"], "active");
    assert_eq!(frame["holder"]["type"], "desktop");
    assert_eq!(frame["holder"]["label"], "kitchen");
    assert_eq!(frame["holder"]["instanceId"], "win-1");

    // `voice.ownership` with nobody holding — `holder` is explicitly `null`,
    // not an absent key.
    let empty = voice_ownership("available", None).into_value();
    assert_eq!(empty["type"], "voice.ownership");
    assert_eq!(empty["state"], "available");
    assert!(empty["holder"].is_null());

    // `VoiceClientRegistry::ownership_broadcast` (`via-app`'s
    // `src/realtime/clients.rs`) is what actually chooses `active` /
    // `busy` / `available` from a live registry; its own
    // `ownership_tells_the_holder_active_and_everyone_else_busy` and
    // `with_nobody_holding_every_connection_is_told_available` unit tests
    // exercise that selection end to end. This test's job is the frame
    // shape `voice_ownership` produces once a state has been chosen, which
    // is what it asserts above.

    // `voice.deactivated` — `{type, holder: <descriptor>|null}`, and always
    // paired with a `playback.clear` first (`realtime-gateway.mjs:12884`).
    let [clear, deactivated] = deactivation_frames(Some(&holder)).map(ServerFrameValue::from);
    assert_eq!(clear.0["type"], "playback.clear");
    assert_eq!(deactivated.0["type"], "voice.deactivated");
    assert_eq!(deactivated.0["holder"]["type"], "desktop");
    assert_eq!(deactivated.0["holder"]["instanceId"], "win-1");

    let [_, deactivated_nobody] = deactivation_frames(None).map(ServerFrameValue::from);
    assert!(deactivated_nobody.0["holder"].is_null());
}

#[test]
fn client_audio_append_decodes_exactly_the_catalogued_shape() {
    let contract = expect_contract("json-field", "client audio.append payload");
    assert!(contract.exact_value.contains("\"type\": \"audio.append\""));
    assert!(contract.exact_value.contains("\"audio\""));

    let decoded = ClientFrame::decode(r#"{"type":"audio.append","audio":"QUFB"}"#)
        .expect("a recognised frame");
    assert_eq!(
        decoded,
        ClientFrame::AudioAppend {
            audio: "QUFB".to_owned()
        }
    );

    // The chunk is forwarded, never decoded, on the way through — this is
    // the crate's own documented reason (`frames.rs`'s `AudioAppend` doc
    // comment) and is what the equality above actually exercises: the
    // base64 text survives untouched.
}

#[test]
fn client_connect_frame_matches_the_catalogued_shape() {
    let contract = expect_contract("ws-event-payload", "client connect frame");
    for field in [
        "timeZone",
        "locale",
        "voiceEnabled",
        "inputEnabled",
        "outputEnabled",
        "clientType",
        "clientLabel",
        "clientInstanceId",
    ] {
        assert!(
            contract.exact_value.contains(field),
            "{field} ({})",
            contract.file,
        );
    }

    // Every field the catalogue names, decoded through the real codec — not a
    // hand-built `Connect`, so a renamed or dropped field fails here.
    let decoded = ClientFrame::decode(
        &serde_json::json!({
            "type": "connect",
            "timeZone": "Asia/Shanghai",
            "locale": "zh-CN",
            "voiceEnabled": true,
            "inputEnabled": true,
            "outputEnabled": false,
            "clientType": "desktop",
            "clientLabel": "kitchen",
            "clientInstanceId": "win-1",
        })
        .to_string(),
    )
    .expect("a connect frame with exactly the catalogued fields decodes");
    let ClientFrame::Connect(connect) = decoded else {
        panic!("expected a connect frame");
    };
    assert_eq!(connect.time_zone, "Asia/Shanghai");
    assert_eq!(connect.locale, "zh-CN");
    assert!(connect.voice_enabled);
    assert_eq!(connect.input_enabled, Some(true));
    assert_eq!(connect.output_enabled, Some(false));
    assert_eq!(connect.descriptor.client_type, ClientType::Desktop);
    assert_eq!(connect.descriptor.label.as_deref(), Some("kitchen"));
    assert_eq!(connect.descriptor.instance_id.as_deref(), Some("win-1"));
}

/// A thin newtype so `.map` can convert `[ServerFrame; 2]` to values without
/// fighting closures over a non-`Copy` type.
struct ServerFrameValue(serde_json::Value);

impl From<via_app::realtime::frames::ServerFrame> for ServerFrameValue {
    fn from(frame: via_app::realtime::frames::ServerFrame) -> Self {
        Self(frame.into_value())
    }
}
