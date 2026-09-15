//! The voice session's numeric bounds, read out of the catalogue.
//!
//! `via-voice`'s own `tests/contracts.rs` guards five of these rows with
//! `assert!(has(kind, name))` — a check that the catalogue still carries the
//! row, not that the shipped constant equals the value in it. A presence check
//! reads like coverage and is not, so the registry cannot honestly point at it.
//! These tests do the comparison the rows were waiting for: every expected
//! number is scanned out of the catalogued `exactValue` and compared against the
//! `pub const` the crate ships.
//!
//! Two of the five stay [`Partial`] afterwards, and the row says which crate
//! owns the remainder: the WebSocket payload cap in `gateway timing constants`
//! is `via-app`'s frame limit, and the `capacity_busy` classification name in
//! `wake reconnect retry` is `via-realtime`'s error vocabulary. Asserting the
//! seven numbers and pretending the eighth is covered is exactly the failure
//! this crate exists to prevent.
//!
//! [`Partial`]: via_conformance::Coverage::Partial

use pretty_assertions::assert_eq;
use sha2::{Digest, Sha256};
use via_conformance::expect_contract;
use via_conformance::value::{after, before, quoted_literals};
use via_i18n::Locale;
use via_voice::assets::{
    DEFAULT_MAX_ASSETS_PER_SESSION, DEFAULT_MAX_BYTES_PER_SESSION, DEFAULT_MAX_SESSIONS,
    DEFAULT_SESSION_TTL_MS, fingerprint,
};
use via_voice::input::InputPart;
use via_voice::permission::{
    DEFAULT_MAX_SESSIONS as PERMISSION_MAX_SESSIONS, DEFAULT_SESSION_ID,
    DEFAULT_TTL_MS as PERMISSION_TTL_MS, PermissionMode, SessionPermissionPolicy,
};
use via_voice::session::{
    MAX_PENDING_AUDIO_CHUNKS, PERMISSION_RESPONSE_GRACE_MS, REALTIME_STABLE_CONNECTION_MS,
    RESPONSE_CONTEXT_CLEANUP_MS, RESPONSE_START_WATCHDOG_MS, WAKE_CONNECT_MAX_ATTEMPTS,
    WAKE_CONNECT_RETRY_BACKOFF_MS,
};
use via_voice::tools::handler::{MAX_DEFERRED_RESPONSES, MAX_PROCESSED_CALLS, MAX_TURN_TASKS};
use via_voice::tools::transcripts::{DEFAULT_MAX_TURNS, DEFAULT_WAIT_MS};
use via_voice::turn::DEFAULT_MAX_ITEMS;

/// The first integer written after `marker` in `value`.
///
/// The catalogue spells these bounds as prose — `maxSessions 500`,
/// `RESPONSE_START_WATCHDOG_MS = 12000`, `processedCalls cap 500` — so the
/// number is located by the name beside it rather than by position. Separators
/// (`=`, spaces, the word `cap`) are skipped; the first run of digits wins, and
/// a marker the catalogue no longer carries panics rather than defaulting.
///
/// # Panics
///
/// When `marker` is absent, or when no digits follow it. Both mean the
/// catalogued value was reworded and this test is no longer reading what it
/// claims to read.
#[track_caller]
fn number_after(value: &str, marker: &str) -> u64 {
    let at = value
        .find(marker)
        .unwrap_or_else(|| panic!("the catalogued value no longer names `{marker}`: {value}"));
    let rest = &value[at + marker.len()..];
    let start = rest
        .find(|character: char| character.is_ascii_digit())
        .unwrap_or_else(|| panic!("no number follows `{marker}` in: {value}"));
    let digits: String = rest[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits
        .parse()
        .unwrap_or_else(|error| panic!("`{digits}` after `{marker}` is not a number: {error}"))
}

/// Seven of the eight `gateway timing constants`.
///
/// The eighth — `WebSocketServer maxPayload` — is the frame limit `via-app`
/// applies when it opens the socket, and `via-app`'s
/// `the_realtime_route_and_its_payload_cap_are_catalogued` asserts it. This row
/// is therefore `Partial`, not `Asserted`.
#[test]
fn the_gateway_timing_constants_are_the_shipped_constants() {
    let catalogued = &expect_contract("default-value", "gateway timing constants").exact_value;

    assert_eq!(
        MAX_PENDING_AUDIO_CHUNKS as u64,
        number_after(catalogued, "MAX_PENDING_AUDIO_CHUNKS")
    );
    assert_eq!(
        RESPONSE_START_WATCHDOG_MS,
        number_after(catalogued, "RESPONSE_START_WATCHDOG_MS")
    );
    assert_eq!(
        PERMISSION_RESPONSE_GRACE_MS,
        number_after(catalogued, "PERMISSION_RESPONSE_GRACE_MS")
    );
    assert_eq!(
        RESPONSE_CONTEXT_CLEANUP_MS,
        number_after(catalogued, "RESPONSE_CONTEXT_CLEANUP_MS")
    );
    assert_eq!(
        REALTIME_STABLE_CONNECTION_MS,
        number_after(catalogued, "REALTIME_STABLE_CONNECTION_MS")
    );
    assert_eq!(
        u64::from(WAKE_CONNECT_MAX_ATTEMPTS),
        number_after(catalogued, "WAKE_CONNECT_MAX_ATTEMPTS")
    );
    assert_eq!(
        WAKE_CONNECT_RETRY_BACKOFF_MS,
        number_after(catalogued, "WAKE_CONNECT_RETRY_BACKOFF_MS")
    );

    // The owed half is still catalogued, so the `Partial` row cannot go stale.
    assert!(
        catalogued.contains("maxPayload"),
        "the payload cap left to via-app has gone from the catalogued value"
    );
}

/// Both `wake reconnect retry` numbers, which are the same two constants
/// spelled a second time in the catalogue.
///
/// The row's remaining clause — that the retry runs *only* for
/// `classifyError() === 'capacity_busy'` — names `via-realtime`'s error
/// vocabulary, which has no wire spelling `via-voice` exposes. The row is
/// `Partial` for that reason.
#[test]
fn the_wake_reconnect_retry_matches_the_gateway_timing_row() {
    let catalogued = &expect_contract("default-value", "wake reconnect retry").exact_value;

    assert_eq!(
        u64::from(WAKE_CONNECT_MAX_ATTEMPTS),
        number_after(catalogued, "WAKE_CONNECT_MAX_ATTEMPTS")
    );
    assert_eq!(
        WAKE_CONNECT_RETRY_BACKOFF_MS,
        number_after(catalogued, "WAKE_CONNECT_RETRY_BACKOFF_MS")
    );
    assert!(
        catalogued.contains("capacity_busy"),
        "the classification the retry is gated on has gone from the catalogued value"
    );
}

/// Every `input asset registry limit`, including the fingerprint formula.
///
/// The catalogued formula is `sha256(mime + '\0' + url)` — the separator is a
/// **NUL**, which the catalogue carries as a raw `U+0000` and which therefore
/// renders as nothing at all in most viewers. That is worth stating, because a
/// reader who takes it for a space would conclude `via-voice` diverges here,
/// and it does not. The separator is read out of the catalogued text rather
/// than typed, and the digest is recomputed independently with `sha2`, so this
/// is a comparison rather than a restatement of the implementation.
#[test]
fn the_input_asset_registry_limits_are_the_shipped_constants() {
    let catalogued = &expect_contract("default-value", "input asset registry limits").exact_value;

    assert_eq!(
        DEFAULT_MAX_ASSETS_PER_SESSION as u64,
        number_after(catalogued, "maxAssetsPerSession")
    );
    assert_eq!(
        DEFAULT_MAX_BYTES_PER_SESSION as u64,
        number_after(catalogued, "maxBytesPerSession")
    );
    assert_eq!(
        DEFAULT_MAX_SESSIONS as u64,
        number_after(catalogued, "maxSessions")
    );
    assert_eq!(
        DEFAULT_SESSION_TTL_MS as u64,
        number_after(catalogued, "sessionTtlMs")
    );

    // The separator, taken from the catalogued formula rather than typed.
    let formula = catalogued
        .split_once("sha256(mime + '")
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split_once("' + url)"))
        .map(|(separator, _)| separator)
        .expect("the catalogued value still states the fingerprint formula");
    assert_eq!(
        formula, "\u{0}",
        "the catalogued separator moved; it is a raw NUL, not a space"
    );

    let digest = |separator: &str, mime: &str, url: &str| {
        let mut hasher = Sha256::new();
        hasher.update(mime.as_bytes());
        hasher.update(separator.as_bytes());
        hasher.update(url.as_bytes());
        hex::encode(hasher.finalize())
    };

    let part = InputPart::file("image/png", "https://example.invalid/one.png");
    assert_eq!(
        fingerprint(&part),
        digest(formula, "image/png", "https://example.invalid/one.png"),
    );
    // The separator is load-bearing: without it the two halves run together
    // and the boundary can be moved, which is what it exists to stop.
    assert_ne!(
        fingerprint(&part),
        digest("", "image/png", "https://example.invalid/one.png"),
    );
    assert_ne!(
        fingerprint(&InputPart::file(
            "image/pn",
            "ghttps://example.invalid/one.png"
        )),
        fingerprint(&part),
    );
}

/// The `SessionPermissionPolicy` mode vocabulary, its two bounds, and its key.
///
/// The key is `${ownerId}\0${sessionId||'main'}` — again a raw NUL in the
/// catalogue, and again invisible. `key` is private, so both of its stated
/// properties are asserted through the public API instead of by reading it:
/// an empty session id resolves to `main`, and the NUL separator is the reason
/// two differently-split ids share one entry.
/// `docs/deviations/phase-5-via-voice.md` records that non-injectivity as
/// upstream's and reachable by nothing that can produce a NUL.
#[test]
fn the_session_permission_policy_bounds_are_the_shipped_constants() {
    let catalogued =
        &expect_contract("default-value", "SessionPermissionPolicy bounds").exact_value;

    // `modes 'ask' (default) | 'auto_allow'; …` — the clause before the first
    // `;`, so the `'main'` inside the key clause is not read as a mode.
    let modes = quoted_literals(before(catalogued, ";"));
    assert_eq!(
        modes,
        vec![
            PermissionMode::Ask.as_str(),
            PermissionMode::AutoAllow.as_str()
        ],
        "the catalogued mode vocabulary is not the shipped one"
    );
    // `'ask' (default)` — the first is the default, and so is ours.
    assert_eq!(PermissionMode::default().as_str(), modes[0]);

    assert_eq!(
        PERMISSION_MAX_SESSIONS as u64,
        number_after(catalogued, "maxSessions")
    );
    assert_eq!(PERMISSION_TTL_MS as u64, number_after(catalogued, "ttlMs"));

    // `${sessionId||'main'}` — the fallback name, read out of the catalogued
    // key clause rather than typed.
    let fallback = quoted_literals(after(catalogued, "key"))
        .first()
        .copied()
        .expect("the catalogued key clause still names the fallback session");
    assert_eq!(fallback, DEFAULT_SESSION_ID);

    let mut policy = SessionPermissionPolicy::new();
    policy.set_mode("alice", "", PermissionMode::AutoAllow);
    assert_eq!(
        policy.mode("alice", fallback),
        PermissionMode::AutoAllow,
        "an empty session id must resolve to the catalogued fallback"
    );
    assert_eq!(policy.len(), 1);

    // `${ownerId}\0${sessionId}` — the separator is NUL, so an owner ending in
    // NUL and a session beginning with one land on the same entry. That is
    // upstream's encoding, and demonstrating it here is what pins the shape of
    // a key the type does not expose.
    let separator = after(catalogued, "${ownerId}")
        .split("${sessionId")
        .next()
        .expect("the catalogued key still has two halves");
    assert_eq!(separator, "\u{0}", "the catalogued key separator moved");

    let mut collide = SessionPermissionPolicy::new();
    collide.set_mode(
        &format!("bob{separator}one"),
        "two",
        PermissionMode::AutoAllow,
    );
    assert_eq!(
        collide.mode("bob", &format!("one{separator}two")),
        PermissionMode::AutoAllow,
    );
    assert_eq!(collide.len(), 1, "the two spellings share one entry");
}

/// Every number in `TurnTranscripts / TurnCorrelation bounds` — six of them,
/// across three types.
#[test]
fn the_turn_bounds_are_the_shipped_constants() {
    let catalogued =
        &expect_contract("default-value", "TurnTranscripts / TurnCorrelation bounds").exact_value;

    assert_eq!(DEFAULT_WAIT_MS, number_after(catalogued, "waitMs"));
    assert_eq!(
        DEFAULT_MAX_TURNS as u64,
        number_after(catalogued, "maxTurns")
    );
    assert_eq!(
        DEFAULT_MAX_ITEMS as u64,
        number_after(catalogued, "maxItems")
    );
    assert_eq!(
        MAX_PROCESSED_CALLS as u64,
        number_after(catalogued, "processedCalls")
    );
    assert_eq!(MAX_TURN_TASKS as u64, number_after(catalogued, "turnTasks"));
    assert_eq!(
        MAX_DEFERRED_RESPONSES as u64,
        number_after(catalogued, "deferredToolResponses")
    );
}

/// A guard on the fixture: the locale argument the registry takes is the one
/// `via-i18n` publishes, so this file's `via-i18n` edge is a real one.
#[test]
fn the_asset_registry_speaks_the_shipped_locale_type() {
    let registry = via_voice::assets::InputAssetRegistry::new(Locale::En);
    assert_eq!(registry.session_count(), 0);
}
