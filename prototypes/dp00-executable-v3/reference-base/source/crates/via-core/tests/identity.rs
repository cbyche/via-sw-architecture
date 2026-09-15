//! The signed owner cookie, including the forgeries it must refuse.
//!
//! Ported from `server/test/identity.test.mjs`, then extended: upstream tests
//! one forged value; a signature check is only as good as the shapes it
//! refuses, so every structural variation is exercised here.

use via_core::identity::{
    AUTH_SECRET_LENGTH_MESSAGE, HttpIdentity, IDENTITY_COOKIE_NAME, IdentityManager, IdentityMode,
    constant_time_eq, decode_uri_component, encode_uri_component, parse_cookies,
};

const SECRET: &str = "test-secret-that-is-longer-than-32-characters";

fn browser() -> IdentityManager {
    IdentityManager::new(SECRET, IdentityMode::Browser, "user_personal")
        .expect("a 45-character secret is above the minimum")
}

fn cookie_of(issued: &HttpIdentity) -> String {
    let header = issued.set_cookie().expect("a fresh identity sets a cookie");
    let value = header
        .split(';')
        .next()
        .expect("a Set-Cookie always has a name=value pair");
    value.to_owned()
}

#[test]
fn a_server_minted_identity_round_trips() {
    let manager = browser();
    let issued = manager.resolve_http(None, false);
    let owner = issued.identity().owner_id.clone();
    assert!(owner.starts_with("user_"));

    let recognized = manager
        .resolve_upgrade(Some(&cookie_of(&issued)))
        .expect("the manager recognises what it just signed");
    assert_eq!(recognized.owner_id, owner);
}

#[test]
fn the_same_secret_recognises_an_identity_across_a_restart() {
    let issued = browser().resolve_http(None, false);
    let restarted = browser();
    let recognized = restarted
        .resolve_upgrade(Some(&cookie_of(&issued)))
        .expect("the signature does not depend on process state");
    assert_eq!(recognized.owner_id, issued.identity().owner_id);
}

#[test]
fn separate_browsers_get_separate_owners() {
    let manager = browser();
    let first = manager.resolve_http(None, false);
    let second = manager.resolve_http(None, false);
    assert_ne!(first.identity().owner_id, second.identity().owner_id);
}

#[test]
fn every_forged_shape_resolves_to_nothing() {
    let manager = browser();
    let valid = manager.token("user_valid");
    let signature = valid
        .rsplit('.')
        .next()
        .expect("a token always has a signature");

    let forgeries = [
        // An attacker-chosen id with no signature at all.
        "attacker-selected-client-id".to_owned(),
        // A well-formed id with no signature.
        "user_attacker".to_owned(),
        // A dot but an empty signature.
        "user_attacker.".to_owned(),
        // Somebody else's signature.
        format!("user_attacker.{signature}"),
        // The right signature, one byte short.
        format!("user_valid.{}", &signature[..signature.len() - 1]),
        // The right signature, one byte long.
        format!("user_valid.{signature}A"),
        // The right signature with one character changed.
        format!("user_valid.{}X", &signature[..signature.len() - 1]),
        // A valid signature over an id without the mandatory prefix.
        manager.token("admin"),
        // An id that merely contains the prefix.
        manager.token("not_user_valid"),
        // Empty.
        String::new(),
    ];

    for forgery in forgeries {
        let header = format!("{IDENTITY_COOKIE_NAME}={forgery}");
        assert_eq!(
            manager.resolve_upgrade(Some(&header)),
            None,
            "{forgery:?} must not mint an owner"
        );
    }

    // The unforged article still works, so the test above is not vacuous.
    assert!(
        manager
            .resolve_upgrade(Some(&format!("{IDENTITY_COOKIE_NAME}={valid}")))
            .is_some()
    );
}

#[test]
fn an_owner_id_containing_a_dot_splits_at_the_last_one() {
    let manager = browser();
    let token = manager.token("user_a.b.c");
    let recognized = manager
        .resolve_upgrade(Some(&format!("{IDENTITY_COOKIE_NAME}={token}")))
        .expect("the split is at the last dot, not the first");
    assert_eq!(recognized.owner_id, "user_a.b.c");
}

#[test]
fn a_malformed_percent_escape_is_ignored_rather_than_fatal() {
    let manager = browser();
    for broken in ["%E0%A4%A", "%", "%ZZ", "%C3"] {
        assert_eq!(
            manager.resolve_upgrade(Some(&format!("{IDENTITY_COOKIE_NAME}={broken}"))),
            None,
            "{broken:?} must not panic or resolve"
        );
    }
    assert_eq!(decode_uri_component("%E0%A4%A"), None);
    assert_eq!(decode_uri_component("%C3%A9"), Some("é".to_owned()));
    assert_eq!(decode_uri_component("plain"), Some("plain".to_owned()));
}

#[test]
fn a_repeated_cookie_keeps_the_last_occurrence() {
    let manager = browser();
    let valid = manager.token("user_valid");
    let header = format!("{IDENTITY_COOKIE_NAME}=forged; {IDENTITY_COOKIE_NAME}={valid}");
    assert_eq!(
        manager
            .resolve_upgrade(Some(&header))
            .map(|identity| identity.owner_id),
        Some("user_valid".to_owned())
    );
}

#[test]
fn other_cookies_are_ignored() {
    let manager = browser();
    let valid = manager.token("user_valid");
    let header = format!("theme=dark; {IDENTITY_COOKIE_NAME}={valid}; consent");
    assert!(manager.resolve_upgrade(Some(&header)).is_some());

    // A part with no `=` at all becomes an empty value, not a panic.
    assert_eq!(
        parse_cookies("flag; a=b"),
        vec![
            ("flag".to_owned(), String::new()),
            ("a".to_owned(), "b".to_owned())
        ]
    );
}

#[test]
fn a_short_secret_is_refused_before_anything_is_signed() {
    let thirty_one = "a".repeat(31);
    let error = IdentityManager::new(&thirty_one, IdentityMode::Browser, "user_personal")
        .expect_err("31 characters is one short");
    assert_eq!(error.to_string(), AUTH_SECRET_LENGTH_MESSAGE);
    assert_eq!(error.code(), "VIA_IDENTITY_SECRET_TOO_SHORT");

    let thirty_two = "a".repeat(32);
    assert!(IdentityManager::new(&thirty_two, IdentityMode::Browser, "user_personal").is_ok());
}

#[test]
fn personal_mode_never_issues_or_checks_a_cookie() {
    let manager = IdentityManager::new(SECRET, IdentityMode::Personal, "user_my_assistant")
        .expect("the secret is long enough");

    let first = manager.resolve_http(None, false);
    let second = manager.resolve_http(Some("via_identity=whatever-forged"), true);
    let upgrade = manager
        .resolve_upgrade(None)
        .expect("personal mode never returns 401");

    assert_eq!(first.identity().owner_id, "user_my_assistant");
    assert_eq!(second.identity().owner_id, "user_my_assistant");
    assert_eq!(upgrade.owner_id, "user_my_assistant");
    assert_eq!(first.set_cookie(), None);
    assert_eq!(second.set_cookie(), None);
}

#[test]
fn a_browser_upgrade_with_no_cookie_is_refused_rather_than_minted() {
    // A WebSocket upgrade has no response headers to carry a Set-Cookie, so
    // minting there would hand out an owner the client could never present
    // again.
    assert_eq!(browser().resolve_upgrade(None), None);
    assert!(matches!(
        browser().resolve_http(None, false),
        HttpIdentity::Issued { .. }
    ));
}

#[test]
fn secure_is_appended_only_over_tls() {
    let manager = browser();
    let plain = manager.set_cookie_value("user_1", false);
    let secure = manager.set_cookie_value("user_1", true);
    assert!(!plain.contains("Secure"));
    assert!(secure.ends_with("; Secure"));
    // Order is upstream's: the pair, then the fixed attributes, then Max-Age,
    // then Secure.
    assert!(secure.starts_with(&format!("{IDENTITY_COOKIE_NAME}=user_1.")));
    assert!(secure.contains("; Path=/; HttpOnly; SameSite=Strict; Max-Age=604800; Secure"));
}

#[test]
fn a_different_secret_does_not_recognise_the_other_secrets_tokens() {
    let mine = browser();
    let theirs = IdentityManager::new(
        "a-completely-different-secret-of-sufficient-length",
        IdentityMode::Browser,
        "user_personal",
    )
    .expect("the secret is long enough");
    let token = theirs.token("user_theirs");
    assert_eq!(
        mine.resolve_upgrade(Some(&format!("{IDENTITY_COOKIE_NAME}={token}"))),
        None
    );
}

#[test]
fn constant_time_eq_is_total_over_length_and_content() {
    assert!(constant_time_eq(b"", b""));
    assert!(constant_time_eq(b"abc", b"abc"));
    assert!(!constant_time_eq(b"abc", b"abd"));
    assert!(!constant_time_eq(b"abc", b"ab"));
    assert!(!constant_time_eq(b"ab", b"abc"));
    // Differing in the *last* byte must be refused just as surely as the
    // first — the property an early-return comparison would leak.
    assert!(!constant_time_eq(&[0u8; 32], &{
        let mut other = [0u8; 32];
        other[31] = 1;
        other
    }));
}

#[test]
fn uri_encoding_round_trips_and_leaves_a_token_alone() {
    let manager = browser();
    let token = manager.token("user_abc");
    assert_eq!(encode_uri_component(&token), token);
    for value in ["a b", "é", "a/b?c=d", "100%"] {
        assert_eq!(
            decode_uri_component(&encode_uri_component(value)).as_deref(),
            Some(value)
        );
    }
}

#[test]
fn the_signature_is_deterministic_and_owner_specific() {
    let manager = browser();
    assert_eq!(manager.sign("user_a"), manager.sign("user_a"));
    assert_ne!(manager.sign("user_a"), manager.sign("user_b"));
    // base64url, unpadded.
    let signature = manager.sign("user_a");
    assert!(!signature.contains('='));
    assert!(!signature.contains('+'));
    assert!(!signature.contains('/'));
    // SHA-256 is 32 bytes; unpadded base64 of 32 bytes is 43 characters.
    assert_eq!(signature.len(), 43);
}
