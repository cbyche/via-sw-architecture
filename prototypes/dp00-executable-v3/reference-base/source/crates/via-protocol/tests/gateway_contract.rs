//! The protocol version and the capability list.
//!
//! Upstream locks these with `test/gateway-contract.test.mjs`; the literals
//! below come from `server/src/core/gateway-protocol.mjs:19,21-74` and from
//! `docs/reference/contracts.json` ("GATEWAY_PROTOCOL_VERSION",
//! "GATEWAY_CAPABILITIES").

use std::collections::BTreeSet;

use pretty_assertions::assert_eq;
use via_protocol::{
    DROPPED_UPSTREAM_CAPABILITIES, GATEWAY_CAPABILITIES, GATEWAY_PROTOCOL_VERSION,
    advertises_capability,
};

/// Upstream's sixteen capabilities, in upstream's declaration order.
/// `server/src/core/gateway-protocol.mjs:21-74`.
const UPSTREAM_CAPABILITIES: &[&str] = &[
    "web.same-origin-ui",
    "web.skin-assets",
    "gateway.instance-lease",
    "gateway.setup-gate",
    "gateway.settings-store",
    "host.electron-entry",
    "host.gateway-process",
    "input.suspend-protocol",
    "input.suspend-clears-playback",
    "input.suspend-ttl",
    "input.suspend-ack",
    "desktop.orb-shell",
    "desktop.orb-window-factory",
    "desktop.orb-placement",
    "desktop.orb-position-store",
    "desktop.skin-store",
];

/// VIA declares its own version line at `1.0.0` rather than inheriting
/// upstream's `2.0.0`, because it advertises a strict subset of upstream's
/// capabilities and a removed capability is a breaking change under upstream's
/// own rule (`docs/fidelity.md`, "Dropped").
#[test]
fn protocol_version_is_vias_own_one_point_zero() {
    assert_eq!(GATEWAY_PROTOCOL_VERSION, "1.0.0");
    assert_ne!(
        GATEWAY_PROTOCOL_VERSION, "2.0.0",
        "VIA must not claim upstream's version while dropping capabilities",
    );
}

/// `/api/health.protocolVersion` must match `^\d+\.\d+\.\d+$`
/// (`test/gateway-contract.test.mjs:26`).
#[test]
fn protocol_version_is_three_numeric_components() {
    let parts: Vec<&str> = GATEWAY_PROTOCOL_VERSION.split('.').collect();
    assert_eq!(parts.len(), 3, "expected major.minor.patch");
    for part in parts {
        assert!(!part.is_empty(), "empty version component");
        assert!(
            part.bytes().all(|byte| byte.is_ascii_digit()),
            "non-numeric version component {part:?}",
        );
    }
}

/// Exact strings and exact order — both are echoed as
/// `/api/health.capabilities`.
#[test]
fn capabilities_are_the_seven_via_honours_in_upstream_order() {
    assert_eq!(
        GATEWAY_CAPABILITIES,
        &[
            "gateway.instance-lease",
            "gateway.setup-gate",
            "gateway.settings-store",
            "input.suspend-protocol",
            "input.suspend-clears-playback",
            "input.suspend-ttl",
            "input.suspend-ack",
        ],
    );
}

/// Every advertised capability is one upstream declared, and it keeps
/// upstream's relative order.
#[test]
fn capabilities_are_an_ordered_subset_of_upstream() {
    let mut upstream = UPSTREAM_CAPABILITIES.iter();
    for capability in GATEWAY_CAPABILITIES {
        assert!(
            upstream.any(|candidate| candidate == capability),
            "{capability} is not an upstream capability, or is out of upstream order",
        );
    }
}

/// The advertised list and the dropped list partition upstream's sixteen
/// exactly: nothing invented, nothing forgotten.
#[test]
fn advertised_and_dropped_partition_upstreams_sixteen() {
    assert_eq!(UPSTREAM_CAPABILITIES.len(), 16);
    assert_eq!(GATEWAY_CAPABILITIES.len(), 7);
    assert_eq!(DROPPED_UPSTREAM_CAPABILITIES.len(), 9);

    let advertised: BTreeSet<&str> = GATEWAY_CAPABILITIES.iter().copied().collect();
    let dropped: BTreeSet<&str> = DROPPED_UPSTREAM_CAPABILITIES.iter().copied().collect();
    let upstream: BTreeSet<&str> = UPSTREAM_CAPABILITIES.iter().copied().collect();

    assert_eq!(advertised.len(), 7, "advertised capabilities are unique");
    assert_eq!(dropped.len(), 9, "dropped capabilities are unique");
    assert!(
        advertised.is_disjoint(&dropped),
        "a capability cannot be both advertised and dropped",
    );
    assert_eq!(
        advertised.union(&dropped).copied().collect::<BTreeSet<_>>(),
        upstream,
    );
}

/// VIA advertises exactly the `gateway.*` and `input.*` families, which is the
/// rule the drop was derived from.
#[test]
fn only_gateway_and_input_families_are_advertised() {
    for capability in GATEWAY_CAPABILITIES {
        assert!(
            capability.starts_with("gateway.") || capability.starts_with("input."),
            "{capability} is outside the gateway.*/input.* families",
        );
    }
    for capability in DROPPED_UPSTREAM_CAPABILITIES {
        assert!(
            capability.starts_with("web.")
                || capability.starts_with("desktop.")
                || capability.starts_with("host."),
            "{capability} was dropped but is not a web/desktop/host surface",
        );
    }
}

/// Upstream requires every capability to match
/// `^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)+$`. Checked by hand rather than with a
/// regex dependency, since the shape is simple and this crate stays leaf-clean.
#[test]
fn capability_names_have_the_contract_shape() {
    fn segment_is_valid(segment: &str) -> bool {
        let mut bytes = segment.bytes();
        let Some(first) = bytes.next() else {
            return false;
        };
        first.is_ascii_lowercase()
            && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    }

    for capability in GATEWAY_CAPABILITIES {
        let segments: Vec<&str> = capability.split('.').collect();
        assert!(
            segments.len() >= 2,
            "{capability} must have at least one dot",
        );
        for segment in segments {
            assert!(
                segment_is_valid(segment),
                "{capability} has an invalid segment {segment:?}",
            );
        }
    }
}

/// None of the capability strings carries the upstream brand — the reason
/// `docs/rebrand.md` classifies the whole list as rebrand-neutral.
#[test]
fn capability_names_survive_the_rebrand_unchanged() {
    for capability in GATEWAY_CAPABILITIES {
        assert!(
            !capability.contains("qwen") && !capability.contains("qwaudio"),
            "{capability} carries the upstream brand",
        );
        assert!(
            !capability.contains("via"),
            "{capability} must stay brand-free, not be rebranded to VIA",
        );
    }
}

#[test]
fn membership_is_the_client_facing_check() {
    assert!(advertises_capability("gateway.setup-gate"));
    assert!(advertises_capability("input.suspend-ack"));
    for capability in DROPPED_UPSTREAM_CAPABILITIES {
        assert!(
            !advertises_capability(capability),
            "{capability} must not be advertised",
        );
    }
    assert!(!advertises_capability("gateway.embedded-lifecycle"));
    assert!(!advertises_capability(""));
}
