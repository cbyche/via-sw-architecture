//! The Gateway WebSocket vocabulary — 52 names, byte for byte.
//!
//! `shared/realtime-events.mjs` publishes three frozen objects and the
//! catalogue records each of them **four times**: once as a `json-field`
//! contract from the module, and three times as `ws-event` contracts spelled
//! differently (comma-separated twice, pipe-separated once). Every spelling has
//! to yield the same list, so the tests here iterate all four records rather
//! than picking one — a catalogue entry that drifts from its siblings fails
//! here instead of being averaged away.
//!
//! One test per vocabulary, iterating a table. Fifty-two tests would say the
//! same thing fifty-two times and report the same failure once.
//!
//! The expected values are read out of `docs/reference/contracts.json` rather
//! than retyped. A retyped literal is a copy that drifts; a parsed one cannot.

use pretty_assertions::assert_eq;
use via_conformance::records_for;
use via_conformance::value::list;
use via_protocol::{
    GatewayClientEvent, GatewayOutboundEvent, GatewayServerEvent, GatewayTaskEvent,
};

/// Every catalogue record that spells out the client → server vocabulary.
///
/// External contract — `shared/realtime-events.mjs:1-21`.
const CLIENT_RECORDS: &[(&str, &str)] = &[
    ("json-field", "GatewayClientEvent constants"),
    (
        "ws-event",
        "GatewayClientEvent (client -> server vocabulary)",
    ),
    ("ws-event", "GatewayClientEvent (client -> server)"),
    ("ws-event", "GatewayClientEvent (client to server, all 16)"),
];

/// Every catalogue record that spells out the server → client vocabulary.
///
/// External contract — `shared/realtime-events.mjs:23-50`.
const SERVER_RECORDS: &[(&str, &str)] = &[
    ("json-field", "GatewayServerEvent constants"),
    (
        "ws-event",
        "GatewayServerEvent (server -> client vocabulary)",
    ),
    ("ws-event", "GatewayServerEvent (server -> client)"),
    ("ws-event", "GatewayServerEvent (server to client, all 22)"),
];

/// Every catalogue record that spells out the task-plane vocabulary.
///
/// External contract — `shared/realtime-events.mjs:52-67`.
const TASK_RECORDS: &[(&str, &str)] = &[
    ("json-field", "GatewayTaskEvent constants"),
    (
        "ws-event",
        "GatewayTaskEvent (server -> client, task plane)",
    ),
    (
        "ws-event",
        "GatewayTaskEvent (server -> client, shares the server namespace)",
    ),
    (
        "ws-event",
        "GatewayTaskEvent (server to client task lifecycle, all 14)",
    ),
];

/// Assert one vocabulary against every catalogue record that spells it out.
///
/// The whole ordered list is compared in one go, because order is contract:
/// upstream's objects are declaration-ordered, the catalogue reproduces that
/// order, and `via-protocol`'s `ALL` publishes it. Each name is then round-
/// tripped through `from_wire`, which is the runtime gate upstream performs
/// with `GATEWAY_CLIENT_EVENT_TYPES` / `GATEWAY_SERVER_EVENT_TYPES`.
fn assert_vocabulary(
    records: &[(&str, &str)],
    shipped: &[&'static str],
    from_wire: fn(&str) -> Option<&'static str>,
) {
    for (kind, name) in records {
        let catalogued = records_for(kind, name);
        assert!(
            !catalogued.is_empty(),
            "docs/reference/contracts.json has no `{kind}` contract named `{name}`; \
             this test would otherwise assert nothing"
        );
        for contract in catalogued {
            let expected = list(&contract.exact_value);
            assert_eq!(
                expected, shipped,
                "`{kind}` / `{name}` ({}) does not match the shipped vocabulary",
                contract.file
            );
            for wire in &expected {
                assert_eq!(
                    from_wire(wire),
                    Some(*wire),
                    "`{wire}` does not parse back to itself"
                );
            }
        }
    }
}

#[test]
fn client_event_vocabulary() {
    let shipped: Vec<&'static str> = GatewayClientEvent::ALL
        .iter()
        .map(|event| event.as_str())
        .collect();
    assert_eq!(shipped.len(), 16, "the client vocabulary is 16 names");
    assert_vocabulary(CLIENT_RECORDS, &shipped, |wire| {
        GatewayClientEvent::from_wire(wire).map(GatewayClientEvent::as_str)
    });
}

#[test]
fn server_event_vocabulary() {
    let shipped: Vec<&'static str> = GatewayServerEvent::ALL
        .iter()
        .map(|event| event.as_str())
        .collect();
    assert_eq!(shipped.len(), 22, "the server vocabulary is 22 names");
    assert_vocabulary(SERVER_RECORDS, &shipped, |wire| {
        GatewayServerEvent::from_wire(wire).map(GatewayServerEvent::as_str)
    });
}

#[test]
fn task_event_vocabulary() {
    let shipped: Vec<&'static str> = GatewayTaskEvent::ALL
        .iter()
        .map(|event| event.as_str())
        .collect();
    assert_eq!(shipped.len(), 14, "the task vocabulary is 14 names");
    assert_vocabulary(TASK_RECORDS, &shipped, |wire| {
        GatewayTaskEvent::from_wire(wire).map(GatewayTaskEvent::as_str)
    });
}

/// The two outbound vocabularies share one socket and must not collide.
///
/// Upstream locks this by asserting that the merged `Set`'s size equals the sum
/// of the two objects' key counts (`test/realtime-events.test.mjs:11-21`);
/// asserted here directly, and again through `GatewayOutboundEvent::from_wire`,
/// which must resolve every outbound name to the side it came from and reject
/// every inbound one.
#[test]
fn outbound_namespace_has_no_collision() {
    let mut union: Vec<&str> = GatewayServerEvent::ALL
        .iter()
        .map(|event| event.as_str())
        .collect();
    union.extend(GatewayTaskEvent::ALL.iter().map(|event| event.as_str()));
    let mut deduped = union.clone();
    deduped.sort_unstable();
    deduped.dedup();
    assert_eq!(
        deduped.len(),
        union.len(),
        "GatewayServerEvent and GatewayTaskEvent share the outbound namespace"
    );
    assert_eq!(union.len(), 36, "22 server names plus 14 task names");

    for event in GatewayServerEvent::ALL {
        assert_eq!(
            GatewayOutboundEvent::from_wire(event.as_str()),
            Some(GatewayOutboundEvent::Session(*event))
        );
    }
    for event in GatewayTaskEvent::ALL {
        assert_eq!(
            GatewayOutboundEvent::from_wire(event.as_str()),
            Some(GatewayOutboundEvent::Task(*event))
        );
    }
    for event in GatewayClientEvent::ALL {
        assert_eq!(
            GatewayOutboundEvent::from_wire(event.as_str()),
            None,
            "`{}` is a client event and must not parse as outbound",
            event.as_str()
        );
    }
}
