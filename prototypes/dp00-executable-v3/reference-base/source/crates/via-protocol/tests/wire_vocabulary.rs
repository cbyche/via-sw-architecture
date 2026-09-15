//! The Gateway WebSocket wire vocabulary, asserted string by string.
//!
//! Every literal in this file is copied from upstream
//! `shared/realtime-events.mjs` (`QwenAudio/qwen-audio-agent` v1.11.0) and is
//! listed in `docs/reference/contracts.json` under "GatewayClientEvent
//! constants", "GatewayServerEvent constants" and "GatewayTaskEvent constants".
//! None of the 52 names carries the upstream brand, so all 52 survive the
//! rebrand byte for byte.

use std::collections::BTreeSet;
use std::str::FromStr;

use pretty_assertions::assert_eq;
use via_protocol::{
    GatewayClientEvent as Client, GatewayOutboundEvent as Outbound, GatewayServerEvent as Server,
    GatewayTaskEvent as Task, ProtocolError,
};

/// `shared/realtime-events.mjs:1-21` — all 16, in declaration order.
const CLIENT_EVENTS: &[(Client, &str)] = &[
    (Client::Connect, "connect"),
    (Client::Unmute, "unmute"),
    (Client::Mute, "mute"),
    (Client::InputUnmute, "input.unmute"),
    (Client::InputMute, "input.mute"),
    (Client::AudioAppend, "audio.append"),
    (Client::TextMessage, "text.message"),
    (Client::InputMessage, "input.message"),
    (Client::InputParts, "input.parts"),
    (Client::Interrupt, "interrupt"),
    (Client::Sleep, "sleep"),
    (Client::Wake, "wake"),
    (Client::PlaybackStarted, "playback.started"),
    (Client::PlaybackEnded, "playback.ended"),
    (Client::PlaybackCancelled, "playback.cancelled"),
    (Client::InputSuspendAck, "input.suspend.ack"),
];

/// `shared/realtime-events.mjs:23-50` — all 22, in declaration order.
const SERVER_EVENTS: &[(Server, &str)] = &[
    (Server::GatewayConnected, "gateway.connected"),
    (Server::GatewayDisconnected, "gateway.disconnected"),
    (Server::VoiceConnection, "voice.connection"),
    (Server::VoiceReady, "voice.ready"),
    (Server::VoiceState, "voice.state"),
    (Server::VoiceOwnership, "voice.ownership"),
    (Server::VoiceDeactivated, "voice.deactivated"),
    (Server::VoiceSleep, "voice.sleep"),
    (Server::TurnStarted, "turn.started"),
    (Server::PlaybackClear, "playback.clear"),
    (Server::InputSuspend, "input.suspend"),
    (Server::InputResume, "input.resume"),
    (Server::AudioDelta, "audio.delta"),
    (Server::AudioDone, "audio.done"),
    (Server::ResponseStarted, "response.started"),
    (Server::ResponseInterrupted, "response.interrupted"),
    (Server::TranscriptDelta, "transcript.delta"),
    (Server::TranscriptFinal, "transcript.final"),
    (Server::TranscriptDiscard, "transcript.discard"),
    (Server::TimelineInline, "timeline.inline"),
    (Server::ClientState, "client.state"),
    (Server::Error, "error"),
];

/// `shared/realtime-events.mjs:52-67` — all 14, in declaration order.
const TASK_EVENTS: &[(Task, &str)] = &[
    (Task::Scheduled, "task.scheduled"),
    (Task::ScheduledFired, "task.scheduled.fired"),
    (Task::Running, "task.running"),
    (Task::Delegated, "task.delegated"),
    (Task::Finalizing, "task.finalizing"),
    (Task::Cancelling, "task.cancelling"),
    (Task::Progress, "task.progress"),
    (Task::ProgressCheck, "task.progress.check"),
    (Task::Completed, "task.completed"),
    (Task::Failed, "task.failed"),
    (Task::Cancelled, "task.cancelled"),
    (Task::PermissionRequested, "task.permission.requested"),
    (Task::PermissionResolved, "task.permission.resolved"),
    (Task::NotificationOffline, "task.notification.offline"),
];

/// The whole exercise: 16 + 22 + 14 = 52 names, every one pinned to its
/// variant through `as_str`, `Display`, `from_wire`, `FromStr` and serde.
macro_rules! assert_vocabulary {
    ($table:expr, $ty:ty, $count:expr, $vocabulary:literal) => {{
        let table = $table;
        assert_eq!(table.len(), $count, "{} table size", $vocabulary);
        assert_eq!(
            <$ty>::ALL.len(),
            $count,
            "{}::ALL size — a variant was added or removed",
            $vocabulary
        );
        assert_eq!(<$ty>::VOCABULARY, $vocabulary);

        // Declaration order is contract: ALL must match the upstream table.
        let declared: Vec<&str> = <$ty>::ALL.iter().map(|event| event.as_str()).collect();
        let expected: Vec<&str> = table.iter().map(|(_, wire)| *wire).collect();
        assert_eq!(declared, expected, "{} declaration order", $vocabulary);

        for (variant, wire) in table {
            assert_eq!(variant.as_str(), *wire, "as_str for {wire}");
            assert_eq!(variant.to_string(), *wire, "Display for {wire}");
            assert_eq!(
                <$ty>::from_wire(wire),
                Some(*variant),
                "from_wire for {wire}"
            );
            assert_eq!(<$ty>::from_str(wire), Ok(*variant), "FromStr for {wire}");
            assert_eq!(
                serde_json::to_value(variant).expect("serialize"),
                serde_json::Value::String((*wire).to_owned()),
                "serialize {wire}",
            );
            assert_eq!(
                serde_json::from_value::<$ty>(serde_json::Value::String((*wire).to_owned()))
                    .expect("deserialize"),
                *variant,
                "deserialize {wire}",
            );
        }

        // Every name is unique within its own vocabulary.
        let unique: BTreeSet<&str> = expected.iter().copied().collect();
        assert_eq!(unique.len(), $count, "{} names are unique", $vocabulary);
        expected
    }};
}

#[test]
fn client_vocabulary_is_the_upstream_sixteen() {
    assert_vocabulary!(CLIENT_EVENTS, Client, 16, "GatewayClientEvent");
}

#[test]
fn server_vocabulary_is_the_upstream_twenty_two() {
    assert_vocabulary!(SERVER_EVENTS, Server, 22, "GatewayServerEvent");
}

#[test]
fn task_vocabulary_is_the_upstream_fourteen() {
    assert_vocabulary!(TASK_EVENTS, Task, 14, "GatewayTaskEvent");
}

/// Upstream's `test/realtime-events.test.mjs:11-21` asserts that merging
/// `GatewayServerEvent` and `GatewayTaskEvent` into one `Set` loses nothing —
/// i.e. that the two vocabularies do not collide. Same property, asserted
/// directly.
#[test]
fn server_and_task_vocabularies_do_not_collide() {
    let mut merged: BTreeSet<&str> = BTreeSet::new();
    for (_, wire) in SERVER_EVENTS {
        merged.insert(wire);
    }
    for (_, wire) in TASK_EVENTS {
        merged.insert(wire);
    }
    assert_eq!(merged.len(), SERVER_EVENTS.len() + TASK_EVENTS.len());
    assert_eq!(merged.len(), 36);
}

/// The direction gate. Upstream keeps two `Set`s; here a client name never
/// parses as an outbound event and vice versa.
#[test]
fn direction_is_enforced_by_parsing() {
    for (_, wire) in CLIENT_EVENTS {
        assert_eq!(
            Server::from_wire(wire),
            None,
            "{wire} is not a server event"
        );
        assert_eq!(Task::from_wire(wire), None, "{wire} is not a task event");
        assert_eq!(
            Outbound::from_wire(wire),
            None,
            "{wire} is not an outbound event",
        );
    }
    let outbound_names = SERVER_EVENTS
        .iter()
        .map(|(_, wire)| *wire)
        .chain(TASK_EVENTS.iter().map(|(_, wire)| *wire));
    for wire in outbound_names {
        assert_eq!(
            Client::from_wire(wire),
            None,
            "{wire} is not a client event"
        );
    }
}

/// The three vocabularies are pairwise disjoint across all 52 names — the
/// property that makes "direction is a type" safe to rely on.
#[test]
fn all_fifty_two_names_are_distinct() {
    let mut every: BTreeSet<&str> = BTreeSet::new();
    for (_, wire) in CLIENT_EVENTS {
        every.insert(wire);
    }
    for (_, wire) in SERVER_EVENTS {
        every.insert(wire);
    }
    for (_, wire) in TASK_EVENTS {
        every.insert(wire);
    }
    assert_eq!(every.len(), 52);
}

/// `playback.*` receipts pair with `audio.*` deltas — upstream's
/// "shared playback acknowledgement lifecycle"
/// (`test/realtime-events.test.mjs:23-36`).
#[test]
fn playback_acknowledgement_lifecycle() {
    assert_eq!(
        [
            Server::AudioDelta.as_str(),
            Server::AudioDone.as_str(),
            Client::PlaybackStarted.as_str(),
            Client::PlaybackEnded.as_str(),
            Client::PlaybackCancelled.as_str(),
        ],
        [
            "audio.delta",
            "audio.done",
            "playback.started",
            "playback.ended",
            "playback.cancelled",
        ],
    );
}

/// Upstream's "shared sleep wake lifecycle"
/// (`test/realtime-events.test.mjs:39-51`).
#[test]
fn sleep_wake_lifecycle() {
    assert_eq!(
        [
            Client::Sleep.as_str(),
            Client::Wake.as_str(),
            Server::VoiceSleep.as_str(),
        ],
        ["sleep", "wake", "voice.sleep"],
    );
}

#[test]
fn outbound_event_is_the_union_set() {
    for (variant, wire) in SERVER_EVENTS {
        let outbound = Outbound::Session(*variant);
        assert_eq!(Outbound::from_wire(wire), Some(outbound));
        assert_eq!(outbound.as_str(), *wire);
        assert_eq!(outbound.to_string(), *wire);
        assert_eq!(Outbound::from(*variant), outbound);
        assert_eq!(
            serde_json::to_value(outbound).expect("serialize"),
            serde_json::Value::String((*wire).to_owned()),
        );
    }
    for (variant, wire) in TASK_EVENTS {
        let outbound = Outbound::Task(*variant);
        assert_eq!(Outbound::from_wire(wire), Some(outbound));
        assert_eq!(outbound.as_str(), *wire);
        assert_eq!(Outbound::from(*variant), outbound);
        assert_eq!(
            serde_json::from_value::<Outbound>(serde_json::Value::String((*wire).to_owned()))
                .expect("deserialize"),
            outbound,
        );
    }
}

#[test]
fn unknown_names_are_rejected_with_a_named_vocabulary() {
    assert_eq!(Client::from_wire("audio.deltaa"), None);
    assert_eq!(
        Client::from_str("desktop.orb-shell"),
        Err(ProtocolError::UnknownWireValue {
            vocabulary: "GatewayClientEvent",
            value: "desktop.orb-shell".to_owned(),
        }),
    );
    assert_eq!(
        Outbound::from_str("connect"),
        Err(ProtocolError::UnknownWireValue {
            vocabulary: "GatewayOutboundEvent",
            value: "connect".to_owned(),
        }),
    );
    // Case is significant on the wire.
    assert_eq!(Server::from_wire("Voice.Ready"), None);
    assert_eq!(Task::from_wire("task.Scheduled"), None);
}

/// An event's `type` field round-trips through a real frame without the enum
/// leaking a Rust-shaped tag.
#[test]
fn a_frame_carries_the_bare_wire_string() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
    struct Frame {
        #[serde(rename = "type")]
        kind: Client,
        audio: String,
    }

    let frame = Frame {
        kind: Client::AudioAppend,
        audio: "AAAA".to_owned(),
    };
    let json = serde_json::to_string(&frame).expect("serialize");
    assert_eq!(json, r#"{"type":"audio.append","audio":"AAAA"}"#);
    assert_eq!(
        serde_json::from_str::<Frame>(&json).expect("deserialize"),
        frame,
    );
}
