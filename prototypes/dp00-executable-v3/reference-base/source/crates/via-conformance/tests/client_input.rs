//! `default-value/client input capability profiles` — what a client may put
//! into a turn.
//!
//! Upstream `shared/client-input-capabilities.mjs` is ten lines long and five
//! modules read it, which is exactly the shape of contract that gets ported
//! from memory and quietly loses a `false`. The three profiles are therefore
//! parsed out of the catalogue rather than retyped, including their key order,
//! and the fallback clause at the end of the `exactValue` is asserted as
//! behaviour rather than skipped as prose.

use pretty_assertions::assert_eq;
use via_conformance::expect_contract;
use via_protocol::{
    CLIENT_INPUT_CAPABILITY_FIELDS, ClientInputCapabilities, ClientType, DEFAULT_CLIENT_TYPE,
};

/// One catalogued profile: its client type and its ordered flags.
#[derive(Debug, PartialEq, Eq)]
struct CataloguedProfile<'a> {
    client_type: &'a str,
    flags: Vec<(&'a str, bool)>,
}

/// Parse `name {key:value,…}` out of one `;`-separated segment.
///
/// Returns `None` for a segment with no braces, which is how the trailing
/// prose clause (`unknown clientType falls back to web`) is told apart from a
/// profile without guessing.
fn parse_profile(segment: &str) -> Option<CataloguedProfile<'_>> {
    let open = segment.find('{')?;
    let close = segment.rfind('}')?;
    if close < open {
        return None;
    }
    let flags = segment[open + 1..close]
        .split(',')
        .map(|pair| {
            let (key, value) = pair
                .split_once(':')
                .unwrap_or_else(|| panic!("`{pair}` is not `key:value`"));
            let value = match value.trim() {
                "true" => true,
                "false" => false,
                other => panic!("`{other}` is not a boolean"),
            };
            (key.trim(), value)
        })
        .collect();
    Some(CataloguedProfile {
        client_type: segment[..open].trim(),
        flags,
    })
}

/// The shipped profile as ordered `(key, value)` pairs, read back through
/// `serde` so the assertion covers the *serialized* order rather than the
/// declaration order a reader hopes matches it.
fn shipped_flags(capabilities: ClientInputCapabilities) -> Vec<(String, bool)> {
    let value = serde_json::to_value(capabilities).expect("four bools always serialize");
    let object = value
        .as_object()
        .expect("ClientInputCapabilities serializes as an object")
        .clone();
    object
        .into_iter()
        .map(|(key, value)| {
            let flag = value
                .as_bool()
                .unwrap_or_else(|| panic!("`{key}` is not a boolean on the wire"));
            (key, flag)
        })
        .collect()
}

#[test]
fn client_input_capability_profiles() {
    let contract = expect_contract("default-value", "client input capability profiles");

    let segments: Vec<&str> = contract.exact_value.split(';').map(str::trim).collect();
    let catalogued: Vec<CataloguedProfile<'_>> =
        segments.iter().filter_map(|s| parse_profile(s)).collect();
    assert_eq!(
        catalogued.len(),
        3,
        "upstream publishes exactly three profiles ({})",
        contract.file
    );

    // The three client types, in the catalogue's order, are the vocabulary.
    let catalogued_types: Vec<&str> = catalogued.iter().map(|p| p.client_type).collect();
    let shipped_types: Vec<&str> = ClientType::ALL.iter().map(|t| t.as_str()).collect();
    assert_eq!(
        catalogued_types, shipped_types,
        "ClientType must be upstream's PROFILES keys, in upstream's order"
    );

    for profile in &catalogued {
        let client_type = ClientType::from_wire(profile.client_type)
            .unwrap_or_else(|| panic!("`{}` is not a ClientType", profile.client_type));
        let shipped = shipped_flags(ClientInputCapabilities::for_client_type(client_type));

        // Key order first: the profile is serialized into a frame, so a
        // reordered object is an observable change even when every value is
        // right.
        let catalogued_keys: Vec<&str> = profile.flags.iter().map(|(key, _)| *key).collect();
        let shipped_keys: Vec<&str> = shipped.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(
            catalogued_keys, shipped_keys,
            "flag order for `{}`",
            profile.client_type
        );
        assert_eq!(
            catalogued_keys,
            CLIENT_INPUT_CAPABILITY_FIELDS.to_vec(),
            "CLIENT_INPUT_CAPABILITY_FIELDS must name the same four flags in the same order"
        );

        // Then the values, one by one, so a failure names the flag.
        for ((catalogued_key, catalogued_value), (shipped_key, shipped_value)) in
            profile.flags.iter().zip(&shipped)
        {
            assert_eq!(catalogued_key, shipped_key);
            assert_eq!(
                *catalogued_value, *shipped_value,
                "{}.{catalogued_key}",
                profile.client_type
            );
        }
    }

    // The clause the parser deliberately skipped, asserted as behaviour.
    let fallback = segments
        .last()
        .copied()
        .expect("the exactValue is not empty");
    assert!(
        fallback.contains("unknown clientType falls back to web"),
        "the catalogue no longer states the fallback rule: {fallback}"
    );
    assert_eq!(DEFAULT_CLIENT_TYPE, ClientType::Web);
    for unknown in ["", "tui", "electron", "Desktop", "web "] {
        assert_eq!(
            ClientInputCapabilities::for_wire(Some(unknown)),
            ClientInputCapabilities::WEB,
            "`{unknown}` must resolve to the web profile"
        );
    }
    assert_eq!(
        ClientInputCapabilities::for_wire(None),
        ClientInputCapabilities::WEB,
        "an absent clientType is upstream's default parameter, also web"
    );

    // The one asymmetry the contract exists to protect: desktop is voice-only,
    // so it is the only profile that hides the composer.
    assert!(!ClientInputCapabilities::DESKTOP.supports_composer_input());
    assert!(ClientInputCapabilities::WEB.supports_composer_input());
    assert!(ClientInputCapabilities::CLI.supports_composer_input());
}
