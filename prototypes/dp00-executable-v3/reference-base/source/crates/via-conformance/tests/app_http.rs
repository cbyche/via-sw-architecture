//! `via-app`'s remaining Pending rows, closed against the shipped crate.
//!
//! Every other `via-app` contract is either `Behavioural` — the comparison
//! lives in `crates/via-app/tests/` itself, next to the router it exercises —
//! or stays `Pending` because nothing in the tree yet builds the value the
//! contract describes. This file holds the two rows that neither of those
//! paths fit: one whose real implementation lives in a crate `via-app` does
//! not itself depend on ([`service_endpoint_port_resolution_matches_the_scheme_table`]),
//! and one deliberately dropped surface whose divergence is already recorded
//! in `docs/deviations/phase-5-via-app.md`
//! ([`skins_static_is_dropped_and_the_404_body_is_reused`]).

use via_conformance::expect_contract;

#[test]
fn service_endpoint_port_resolution_matches_the_scheme_table() {
    let contract = expect_contract("default-value", "service endpoint port resolution");
    // "wss→443, https→443, ws→80, explicit port preserved; paths are stripped
    // ('wss://h/gateway' → 'wss://h')" — every clause of that prose has a
    // matching assertion below, against `via-process::endpoint`, which is
    // where the identical scheme table actually lives
    // (`server/src/process/backend-drivers/shared.mjs:16-19`).
    for clause in ["wss→443", "https→443", "ws→80", "explicit port preserved"] {
        assert!(
            contract.exact_value.contains(clause),
            "the catalogued value no longer contains `{clause}`: {}",
            contract.exact_value,
        );
    }

    assert_eq!(via_process::service_endpoint_port("wss://h").unwrap(), 443);
    assert_eq!(
        via_process::service_endpoint_port("https://h").unwrap(),
        443
    );
    assert_eq!(via_process::service_endpoint_port("ws://h").unwrap(), 80);
    assert_eq!(via_process::service_endpoint_port("http://h").unwrap(), 80);
    // An explicit port always wins over the scheme default.
    assert_eq!(
        via_process::service_endpoint_port("wss://h:9999").unwrap(),
        9999
    );
    assert_eq!(
        via_process::service_endpoint_port("http://h:80").unwrap(),
        80
    );

    // "paths are stripped ('wss://h/gateway' → 'wss://h')" — the catalogue's
    // own example, reproduced literally rather than with a value this test
    // invented.
    assert_eq!(
        via_process::normalize_service_endpoint(
            "wss://h/gateway",
            via_process::DEFAULT_SERVICE_PROTOCOLS
        )
        .unwrap(),
        "wss://h",
    );
}

#[test]
fn skins_static_is_dropped_and_the_404_body_is_reused() {
    let contract = expect_contract("http-route", "GET /skins/* (static)");
    // Upstream's literal: `express.static(...); miss -> 404
    // {"error":"not found"}`. VIA ships no web UI at all — see
    // `crates/via-app/src/http/mod.rs`'s module documentation and
    // `docs/deviations/phase-5-via-app.md`'s "Not ported" section, which name
    // this route by name — so there is no `/skins/*` route to compare against,
    // only the 404 body it left behind for VIA's fallback to reuse.
    assert!(
        contract.exact_value.contains(r#"{"error":"not found"}"#),
        "the catalogued 404 body changed: {}",
        contract.exact_value,
    );

    // VIA's real, shipped value: every unmatched path is a JSON 404
    // (`via_app::http::FALLBACK_IS_JSON_404`) built from exactly the string
    // upstream's own `/skins` miss produced — not a new literal invented for
    // the fallback.
    assert_eq!(via_app::NOT_FOUND_BODY, "not found");
    let served = serde_json::json!({ "error": via_app::NOT_FOUND_BODY }).to_string();
    assert_eq!(served, r#"{"error":"not found"}"#);
}
