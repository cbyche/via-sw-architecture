//! The protocol version and the capability list.
//!
//! Both are contracts VIA **deliberately does not reproduce**, which is exactly
//! why they need a test rather than a note. The risk with a recorded deviation
//! is not that it is wrong, it is that a later contributor cannot tell it apart
//! from a mistake — so each test here asserts three things at once: the upstream
//! value, VIA's value, and the relationship between them that makes the
//! deviation legitimate.
//!
//! - `GATEWAY_PROTOCOL_VERSION` — upstream `'2.0.0'`, VIA `1.0.0`. Upstream's
//!   own rule (`gateway-protocol.mjs:1-18`) is that a *removed* capability is a
//!   breaking change, and VIA advertises a strict subset, so inheriting `2.0.0`
//!   would lie to the clients the version exists for.
//! - `GATEWAY_CAPABILITIES` — upstream's sixteen entries have to partition
//!   exactly into the seven VIA honours and the nine it drops, **in upstream
//!   order**. That is the assertion that keeps "we dropped the GUI surfaces"
//!   from quietly becoming "we forgot one".

use pretty_assertions::assert_eq;
use via_conformance::records_for;
use via_conformance::value::{js_string_array, unquote};
use via_protocol::{
    DROPPED_UPSTREAM_CAPABILITIES, GATEWAY_CAPABILITIES, GATEWAY_PROTOCOL_VERSION,
    advertises_capability,
};

/// Whether `value` is `MAJOR.MINOR.PATCH` with numeric components.
///
/// `via-protocol` documents the shape as `^\d+\.\d+\.\d+$`; checked by hand
/// rather than with `regex`, which this crate has no other need for.
fn is_semver_triple(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

#[test]
fn protocol_version_starts_a_via_line() {
    let records = records_for("protocol-version", "GATEWAY_PROTOCOL_VERSION");
    assert_eq!(
        records.len(),
        2,
        "the catalogue records this contract twice, once from the module and \
         once from the upstream test that locks it"
    );

    // Both spellings — `'2.0.0'` from the module and `2.0.0` from the test —
    // have to denote the same value.
    for contract in &records {
        assert_eq!(
            unquote(&contract.exact_value),
            "2.0.0",
            "upstream GATEWAY_PROTOCOL_VERSION, {}",
            contract.file
        );
    }

    assert_eq!(
        GATEWAY_PROTOCOL_VERSION, "1.0.0",
        "VIA declares its own protocol line; see crates/via-protocol/src/gateway.rs"
    );
    assert_ne!(
        GATEWAY_PROTOCOL_VERSION,
        unquote(&records[0].exact_value),
        "the divergence from upstream's 2.0.0 is deliberate and must stay visible"
    );
    assert!(is_semver_triple(GATEWAY_PROTOCOL_VERSION));
    assert!(is_semver_triple(unquote(&records[0].exact_value)));
}

#[test]
fn capability_list_partitions_the_upstream_sixteen() {
    let records = records_for("capability-list", "GATEWAY_CAPABILITIES");
    assert_eq!(
        records.len(),
        2,
        "catalogued twice, with and without the freeze"
    );

    let upstream = {
        let mut parsed: Option<Vec<&str>> = None;
        for contract in &records {
            let Some(entries) = js_string_array(&contract.exact_value) else {
                panic!(
                    "`capability-list` / GATEWAY_CAPABILITIES ({}) is not an array literal",
                    contract.file
                );
            };
            if let Some(first) = &parsed {
                assert_eq!(
                    &entries, first,
                    "the two catalogue spellings of GATEWAY_CAPABILITIES disagree"
                );
            }
            parsed = Some(entries);
        }
        match parsed {
            Some(entries) => entries,
            None => panic!("the catalogue has no GATEWAY_CAPABILITIES contract"),
        }
    };
    assert_eq!(
        upstream.len(),
        16,
        "upstream advertises sixteen capabilities"
    );

    // Every entry matches the shape `via-protocol` documents, on both sides.
    for capability in upstream.iter().chain(GATEWAY_CAPABILITIES) {
        assert!(
            capability.contains('.')
                && capability.bytes().all(|b| b.is_ascii_lowercase()
                    || b.is_ascii_digit()
                    || b == b'-'
                    || b == b'.'),
            "`{capability}` is not a dotted lowercase capability name"
        );
    }

    // The partition. Filtering upstream's list keeps *upstream's* order, so
    // this asserts the order of both halves as well as their membership.
    let honoured: Vec<&str> = upstream
        .iter()
        .copied()
        .filter(|capability| GATEWAY_CAPABILITIES.contains(capability))
        .collect();
    let dropped: Vec<&str> = upstream
        .iter()
        .copied()
        .filter(|capability| DROPPED_UPSTREAM_CAPABILITIES.contains(capability))
        .collect();

    assert_eq!(
        honoured, GATEWAY_CAPABILITIES,
        "GATEWAY_CAPABILITIES must be upstream's list restricted to what VIA honours, in \
         upstream's own order"
    );
    assert_eq!(
        dropped, DROPPED_UPSTREAM_CAPABILITIES,
        "DROPPED_UPSTREAM_CAPABILITIES must be upstream's list restricted to what VIA drops, \
         in upstream's own order"
    );
    assert_eq!(
        honoured.len() + dropped.len(),
        upstream.len(),
        "the two lists must partition upstream's sixteen: no entry unaccounted for"
    );
    assert_eq!(GATEWAY_CAPABILITIES.len(), 7);
    assert_eq!(DROPPED_UPSTREAM_CAPABILITIES.len(), 9);

    // Disjoint, and the membership test clients are told to use agrees.
    for capability in GATEWAY_CAPABILITIES {
        assert!(advertises_capability(capability));
        assert!(
            !DROPPED_UPSTREAM_CAPABILITIES.contains(capability),
            "`{capability}` is both advertised and dropped"
        );
    }
    for capability in DROPPED_UPSTREAM_CAPABILITIES {
        assert!(
            !advertises_capability(capability),
            "`{capability}` is dropped but still advertised"
        );
    }
    assert!(!advertises_capability("gateway.does-not-exist"));
}
