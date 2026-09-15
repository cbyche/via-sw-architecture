//! `SessionMode` — the mode table from `docs/architecture.md` §2, asserted.
//!
//! New to VIA: neither upstream has this type, so there is no upstream literal
//! to reproduce. The wire names and the layer table are the contract.

use std::str::FromStr;

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_protocol::{ProtocolError, SessionMode};

const MODES: &[(SessionMode, &str)] = &[
    (SessionMode::Dictation, "dictation"),
    (SessionMode::Direct, "direct"),
    (SessionMode::Agent, "agent"),
    (SessionMode::Interface, "interface"),
];

#[test]
fn there_are_exactly_four_modes_in_table_order() {
    assert_eq!(SessionMode::ALL.len(), 4);
    assert_eq!(
        SessionMode::ALL
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>(),
        vec!["dictation", "direct", "agent", "interface"],
    );
}

#[rstest]
#[case(SessionMode::Dictation, "dictation")]
#[case(SessionMode::Direct, "direct")]
#[case(SessionMode::Agent, "agent")]
#[case(SessionMode::Interface, "interface")]
fn each_mode_round_trips_through_serde(#[case] mode: SessionMode, #[case] wire: &str) {
    let json = serde_json::to_string(&mode).expect("serialize");
    assert_eq!(json, format!("\"{wire}\""));
    assert_eq!(
        serde_json::from_str::<SessionMode>(&json).expect("deserialize"),
        mode,
    );

    assert_eq!(mode.as_str(), wire);
    assert_eq!(mode.to_string(), wire);
    assert_eq!(SessionMode::from_wire(wire), Some(mode));
    assert_eq!(SessionMode::from_str(wire), Ok(mode));
}

/// The mode travels on the `connect` frame and in every stored Work record, so
/// it has to survive a nested round-trip, not just a bare string one.
#[test]
fn the_mode_round_trips_inside_a_connect_frame() {
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
    struct Connect {
        #[serde(rename = "type")]
        kind: String,
        mode: SessionMode,
    }

    let frame = Connect {
        kind: "connect".to_owned(),
        mode: SessionMode::Interface,
    };
    let json = serde_json::to_string(&frame).expect("serialize");
    assert_eq!(json, r#"{"type":"connect","mode":"interface"}"#);
    assert_eq!(
        serde_json::from_str::<Connect>(&json).expect("deserialize"),
        frame,
    );
}

#[test]
fn the_wire_name_is_the_lowercase_variant_and_nothing_else() {
    for (mode, wire) in MODES {
        assert_eq!(mode.as_str(), *wire);
        assert!(
            wire.bytes().all(|byte| byte.is_ascii_lowercase()),
            "{wire} must be plain lowercase",
        );
    }
    assert_eq!(SessionMode::from_wire("Agent"), None);
    assert_eq!(SessionMode::from_wire("AGENT"), None);
    assert_eq!(SessionMode::from_wire("harness"), None);
    assert_eq!(
        SessionMode::from_str("dictation "),
        Err(ProtocolError::UnknownWireValue {
            vocabulary: "SessionMode",
            value: "dictation ".to_owned(),
        }),
    );
}

#[test]
fn the_default_is_the_whole_stack() {
    assert_eq!(SessionMode::default(), SessionMode::Agent);
    assert_eq!(SessionMode::default().as_str(), "agent");
}

/// The §2 table, column by column.
#[rstest]
//                              layer2  harness  speaks  work queue
#[case(SessionMode::Dictation, false, false, false, false)]
#[case(SessionMode::Direct, true, false, true, false)]
#[case(SessionMode::Agent, true, true, true, true)]
#[case(SessionMode::Interface, true, false, true, false)]
fn the_layer_table_holds(
    #[case] mode: SessionMode,
    #[case] mounts_layer2: bool,
    #[case] mounts_harness: bool,
    #[case] speaks: bool,
    #[case] mounts_work_queue: bool,
) {
    assert_eq!(mode.mounts_layer2(), mounts_layer2, "{mode} mounts_layer2");
    assert_eq!(
        mode.mounts_harness(),
        mounts_harness,
        "{mode} mounts_harness"
    );
    assert_eq!(mode.speaks(), speaks, "{mode} speaks");
    assert_eq!(
        mode.mounts_work_queue(),
        mounts_work_queue,
        "{mode} mounts_work_queue",
    );
}

/// `dictation` is the one mode that mounts no model: no Layer 2, no harness,
/// no speech out.
#[test]
fn dictation_mounts_nothing_above_layer_one() {
    let mode = SessionMode::Dictation;
    assert!(!mode.mounts_layer2());
    assert!(!mode.mounts_harness());
    assert!(!mode.speaks());
    assert!(!mode.mounts_work_queue());
}

/// `agent` degrades to `direct` when no harness is configured, rather than
/// failing — what makes a fresh install useful before a backend is installed.
#[test]
fn agent_degrades_to_direct_and_nothing_else_moves() {
    assert_eq!(
        SessionMode::Agent.degraded_without_harness(),
        SessionMode::Direct,
    );
    for mode in [
        SessionMode::Dictation,
        SessionMode::Direct,
        SessionMode::Interface,
    ] {
        assert_eq!(
            mode.degraded_without_harness(),
            mode,
            "{mode} mounts no harness, so it cannot degrade",
        );
    }
}

/// The degradation is idempotent and always lands somewhere that needs no
/// harness — otherwise a degraded session could still fail for the same reason.
#[test]
fn degradation_is_idempotent_and_harness_free() {
    for mode in SessionMode::ALL {
        let degraded = mode.degraded_without_harness();
        assert!(
            !degraded.mounts_harness(),
            "{mode} degraded to {degraded}, which still wants a harness",
        );
        assert_eq!(degraded.degraded_without_harness(), degraded);
    }
}

/// A mode that mounts a harness must also mount Layer 2 — Layer 3 is reached
/// through the Work queue, never directly.
#[test]
fn a_harness_implies_layer_two() {
    for mode in SessionMode::ALL {
        if mode.mounts_harness() {
            assert!(mode.mounts_layer2(), "{mode} skips Layer 2");
            assert!(mode.mounts_work_queue(), "{mode} has no queue to dispatch");
        }
    }
}
