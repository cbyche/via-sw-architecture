//! The origin allow-list, tested adversarially.
//!
//! Ported from `server/test/request-security.test.mjs` and then extended well
//! past it. Upstream's file has three tests; this is a security boundary whose
//! failure mode is "any web page the user has open can drive their Gateway", so
//! the interesting cases are the ones that *look* like they should pass.

use rstest::rstest;
use via_core::security::{is_allowed_origin, is_loopback_host, trusted_origins};

fn allow(entries: &[&str]) -> Vec<String> {
    entries.iter().map(|entry| (*entry).to_owned()).collect()
}

// ── the upstream cases ──────────────────────────────────────────────────────

#[test]
fn loopback_same_origin_and_non_browser_requests_are_allowed() {
    assert!(is_allowed_origin(Some("localhost:3101"), None, &[]));
    assert!(is_allowed_origin(
        Some("localhost:3101"),
        Some("http://localhost:3101"),
        &[]
    ));
    assert!(!is_allowed_origin(
        Some("localhost:3101"),
        Some("https://attacker.example"),
        &[]
    ));
}

#[test]
fn dns_rebinding_and_direct_network_access_are_refused_by_default() {
    // The rebinding case: Origin and Host agree, and both name a host the
    // attacker controls and has pointed at 127.0.0.1.
    assert!(!is_allowed_origin(
        Some("attacker.example:3101"),
        Some("http://attacker.example:3101"),
        &[]
    ));
    assert!(!is_allowed_origin(Some("192.168.1.20:3101"), None, &[]));
}

#[test]
fn only_an_explicitly_configured_reverse_proxy_origin_is_allowed() {
    let options = allow(&["https://voice.example.com"]);
    assert!(is_allowed_origin(
        Some("voice.example.com"),
        Some("https://voice.example.com"),
        &options
    ));
    assert!(is_allowed_origin(Some("voice.example.com"), None, &options));
    assert!(!is_allowed_origin(
        Some("other.example.com"),
        Some("https://other.example.com"),
        &options
    ));
    // Scheme-sensitive: an https entry does not admit an http origin, and an
    // http public entry is dropped from the allow-list entirely.
    assert!(!is_allowed_origin(
        Some("voice.example.com"),
        Some("http://voice.example.com"),
        &allow(&["http://voice.example.com"])
    ));
}

// ── adversarial ─────────────────────────────────────────────────────────────

#[rstest]
// A hostname that merely starts with a loopback address.
#[case("127.0.0.1.evil.com:3101", None)]
#[case("127.0.0.1.evil.com:3101", Some("http://127.0.0.1.evil.com:3101"))]
// …and one that ends with it.
#[case("evil.com.127.0.0.1:3101", None)]
// A username in the Host header, which WHATWG strips to the real host.
#[case("127.0.0.1@evil.com:3101", None)]
// The wildcard bind address is not loopback: reaching the Gateway on it means
// the request came in over the network.
#[case("0.0.0.0:3101", None)]
#[case("0.0.0.0:3101", Some("http://0.0.0.0:3101"))]
#[case("[::]:3101", None)]
// A LAN address.
#[case("10.1.2.3:3101", None)]
#[case("172.16.0.9:3101", None)]
// An unparseable Host.
#[case("", None)]
#[case(":::::", None)]
// A public host with a loopback Origin, and the reverse.
#[case("evil.example:3101", Some("http://127.0.0.1:3101"))]
#[case("127.0.0.1:3101", Some("http://evil.example:3101"))]
// A port mismatch between two loopback endpoints.
#[case("127.0.0.1:3101", Some("http://127.0.0.1:3102"))]
#[case("localhost:3101", Some("http://localhost"))]
// A different loopback *spelling* on each side is still a host mismatch.
#[case("127.0.0.1:3101", Some("http://localhost:3101"))]
// An opaque origin, which upstream turns into a 500.
#[case("127.0.0.1:3101", Some("data:text/html,<script>"))]
#[case("127.0.0.1:3101", Some("blob:https://evil.example/x"))]
fn a_request_that_should_not_be_trusted_is_refused(
    #[case] host: &str,
    #[case] origin: Option<&str>,
) {
    assert!(
        !is_allowed_origin(Some(host), origin, &[]),
        "Host {host:?} with Origin {origin:?} must be refused"
    );
}

#[rstest]
#[case("127.0.0.1:3101", None)]
#[case("localhost:3101", None)]
#[case("[::1]:3101", None)]
// WHATWG normalises the long form of the IPv6 loopback.
#[case("[0:0:0:0:0:0:0:1]:3101", None)]
#[case("127.0.0.1:3101", Some("http://127.0.0.1:3101"))]
#[case("[::1]:3101", Some("http://[::1]:3101"))]
// The default port is omitted from both sides, so they still match.
#[case("localhost:80", Some("http://localhost"))]
#[case("localhost", Some("http://localhost:80"))]
fn a_loopback_request_is_allowed(#[case] host: &str, #[case] origin: Option<&str>) {
    assert!(
        is_allowed_origin(Some(host), origin, &[]),
        "Host {host:?} with Origin {origin:?} must be allowed"
    );
}

#[test]
fn an_absent_origin_is_not_the_same_as_a_public_host() {
    // The no-Origin path exists for the CLI, which reaches the Gateway over
    // loopback. It must not become a way for a LAN client to skip the check.
    assert!(is_allowed_origin(Some("127.0.0.1:3101"), None, &[]));
    assert!(!is_allowed_origin(Some("gateway.lan:3101"), None, &[]));
    // …unless the operator allow-listed exactly that host.
    assert!(is_allowed_origin(
        Some("gateway.lan"),
        None,
        &allow(&["https://gateway.lan"])
    ));
    // But the allow-list matches on host *including port*.
    assert!(!is_allowed_origin(
        Some("gateway.lan:8443"),
        None,
        &allow(&["https://gateway.lan"])
    ));
}

#[test]
fn an_allow_listed_origin_must_still_match_the_host_it_arrived_on() {
    let options = allow(&["https://a.example", "https://b.example"]);
    assert!(is_allowed_origin(
        Some("a.example"),
        Some("https://a.example"),
        &options
    ));
    // Both are allow-listed, but a request claiming to be from `b` that
    // arrived at `a` is a confused-deputy setup.
    assert!(!is_allowed_origin(
        Some("a.example"),
        Some("https://b.example"),
        &options
    ));
}

#[test]
fn the_allow_list_filter_drops_rather_than_rejects() {
    // Every kind of unusable entry disappears silently; a Rust port that
    // errored here would refuse to start on a configuration upstream accepts.
    assert_eq!(
        trusted_origins(&allow(&[
            "https://ok.example",
            "http://public.example",
            "not a url",
            "",
            "ftp://files.example",
            "http://127.0.0.1:3101",
            // Default ports are dropped from the serialised origin.
            "https://ok2.example:443",
        ])),
        vec![
            "https://ok.example".to_owned(),
            "http://127.0.0.1:3101".to_owned(),
            "https://ok2.example".to_owned(),
        ]
    );
}

#[test]
fn an_empty_allow_list_is_the_default_and_admits_no_public_origin() {
    assert!(trusted_origins(&[]).is_empty());
    assert!(!is_allowed_origin(
        Some("voice.example.com"),
        Some("https://voice.example.com"),
        &[]
    ));
}

#[test]
fn loopback_membership_is_exactly_the_four_names() {
    for host in ["127.0.0.1", "localhost", "[::1]", "::1"] {
        assert!(is_loopback_host(host));
    }
    for host in [
        "127.0.0.2",
        "127.1",
        "0.0.0.0",
        "[::]",
        "localhost.",
        "LOCALHOST",
        "::2",
    ] {
        assert!(!is_loopback_host(host), "{host} must not count as loopback");
    }
}

#[test]
fn a_host_header_with_a_path_or_query_is_reduced_to_its_host() {
    // WHATWG parses `http://<host-header>` and stops at the first `/` or `?`,
    // so a Host header carrying junk cannot smuggle a different authority.
    assert!(is_allowed_origin(Some("127.0.0.1:3101/evil"), None, &[]));
    assert!(!is_allowed_origin(
        Some("evil.example/127.0.0.1"),
        None,
        &[]
    ));
}
