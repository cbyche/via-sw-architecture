//! The coordinator and project key formats, including the adversarial cases.
//!
//! `docs/reference/contracts.json` calls the coordinator key *"THE fixed
//! identity that survives voice sessions, Work IDs, and Gateway restarts"*, and
//! it is a map key, a lock key prefix and a persisted registry key at once.
//! Two owner ids that must be different identities and render the same key
//! would let one owner reach another's coordinator session; two renderings of
//! one owner id would orphan a stored session on every restart. Both are tested
//! here.

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_downstream::{
    COORDINATOR_KEY_SUFFIX, DEFAULT_OWNER_ID, KEY_SEPARATOR, SessionKey, SessionKeyScope,
};

/// `encodeURIComponent`'s unreserved set, and everything else escaped.
#[rstest]
#[case::plain("ana", "ana")]
#[case::space("owner one", "owner%20one")]
#[case::unreserved_punctuation("-_.!~*'()", "-_.!~*'()")]
#[case::colon("a:b", "a%3Ab")]
#[case::percent("100%", "100%25")]
#[case::slash("a/b", "a%2Fb")]
#[case::plus("a+b", "a%2Bb")]
#[case::han("\u{4f60}\u{597d}", "%E4%BD%A0%E5%A5%BD")]
#[case::emoji("\u{1f600}", "%F0%9F%98%80")]
#[case::newline("a\nb", "a%0Ab")]
fn owner_ids_are_percent_encoded(#[case] owner: &str, #[case] encoded: &str) {
    assert_eq!(
        SessionKey::coordinator("acp", owner).as_str(),
        format!("acp{KEY_SEPARATOR}{encoded}{KEY_SEPARATOR}{COORDINATOR_KEY_SUFFIX}"),
    );
}

/// No owner id, however hostile, can produce a key with more or fewer than
/// three segments — which is what makes the segment structure parseable and a
/// forged `:backend` suffix impossible.
#[rstest]
#[case("")]
#[case("   ")]
#[case("a:b:c:d")]
#[case(":backend")]
#[case("ana:backend")]
#[case("acp:ana:backend")]
#[case("%3A")]
#[case("\u{feff}")]
fn a_coordinator_key_always_has_exactly_three_segments(#[case] owner: &str) {
    let key = SessionKey::coordinator("acp", owner);
    let segments: Vec<&str> = key.as_str().split(KEY_SEPARATOR).collect();
    assert_eq!(segments.len(), 3, "{}", key.as_str());
    assert_eq!(segments[0], "acp");
    assert_eq!(segments[2], COORDINATOR_KEY_SUFFIX);
}

/// Distinct owner ids must not collide, including the pairs an encoder that
/// stopped at ASCII would collapse.
#[test]
fn distinct_owners_get_distinct_keys() {
    let owners = [
        "ana",
        "Ana",
        "ANA",
        "ana ",
        " ana",
        "an a",
        "an%20a",
        "an+a",
        "a:na",
        "personal",
        "",
        "   ",
        "\u{4f60}",
        "\u{1f600}",
    ];
    let mut keys: Vec<String> = owners
        .iter()
        .map(|owner| SessionKey::coordinator("acp", owner).as_str().to_owned())
        .collect();
    keys.sort();
    keys.dedup();
    // `ana `, ` ana` and `ana` are one identity by design (the trim); `""` and
    // `"   "` are both `personal`, which the literal `personal` also is. So
    // three collapses are expected and everything else must stay distinct.
    assert_eq!(keys.len(), owners.len() - 4, "{keys:?}");
}

/// The trim is upstream's `clean`, and it happens before the default.
#[rstest]
#[case("ana", "ana")]
#[case("  ana  ", "ana")]
#[case("\tana\n", "ana")]
#[case("\u{3000}ana\u{2028}", "ana")]
#[case("", DEFAULT_OWNER_ID)]
#[case("\u{feff}", DEFAULT_OWNER_ID)]
fn the_owner_is_trimmed_then_defaulted(#[case] owner: &str, #[case] expected: &str) {
    let key = SessionKey::coordinator("acp", owner);
    assert_eq!(key.owner_id(), Some(expected));
    assert_eq!(key.session_id(), None);
    assert!(key.is_coordinator());
    assert_eq!(
        key.scope(),
        &SessionKeyScope::Coordinator {
            owner_id: expected.to_owned(),
        },
    );
}

/// A project key is two segments, is not encoded, and is trimmed.
#[test]
fn a_project_key_is_the_protocol_and_the_backends_own_id() {
    let key = SessionKey::project("openclaw", "  sess/42  ");
    assert_eq!(key.as_str(), "openclaw:sess/42");
    assert_eq!(key.session_id(), Some("sess/42"));
    assert_eq!(key.owner_id(), None);
    assert!(!key.is_coordinator());
    assert_eq!(key.protocol(), "openclaw");
}

/// The two shapes never collide with each other, even when a backend hands out
/// a session id that looks like a coordinator key.
#[test]
fn a_project_key_cannot_impersonate_a_coordinator_key() {
    let coordinator = SessionKey::coordinator("acp", "ana");
    let impostor = SessionKey::project("acp", "ana:backend");
    assert_eq!(coordinator.as_str(), impostor.as_str());
    // The *rendered* strings can coincide — that is upstream's format, and
    // upstream keeps the two in separate maps. What must not coincide is the
    // typed scope, which is what VIA dispatches on.
    assert_ne!(coordinator.scope(), impostor.scope());
    assert_ne!(coordinator, impostor);
    assert!(coordinator.is_coordinator());
    assert!(!impostor.is_coordinator());
}

/// The rendered key is what `Display`, `as_str` and `Serialize` all produce, so
/// a map key and a log line and a stored record cannot disagree.
#[test]
fn every_rendering_agrees() {
    let key = SessionKey::coordinator("openclaw", "owner one");
    assert_eq!(key.to_string(), key.as_str());
    assert_eq!(
        serde_json::to_string(&key).expect("serializable"),
        format!("\"{}\"", key.as_str()),
    );
}

/// The protocol segment is passed through untouched, because it has already
/// been normalised by the catalog before a key is built. Documented so that a
/// future caller does not assume the key normalises for them.
#[test]
fn the_protocol_segment_is_not_normalised_here() {
    let key = SessionKey::coordinator("  Codex  ", "ana");
    assert_eq!(key.as_str(), "  Codex  :ana:backend");
    assert_eq!(key.protocol(), "  Codex  ");
}
