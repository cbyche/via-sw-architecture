//! The signed owner identity.
//!
//! Ported from `server/src/core/identity.mjs`. An owner id is minted by the
//! server, never accepted from a client: the cookie carries
//! `user_<uuid>.<base64url(HMAC-SHA256(secret, ownerId))>` and a value whose
//! signature does not verify resolves to *nothing* rather than to the owner it
//! names. That is the only thing separating one browser identity from another.
//!
//! # Two properties that are easy to lose in a port
//!
//! **The comparison must be constant time.** Upstream calls
//! `crypto.timingSafeEqual`. A `==` on the two byte strings returns as soon as
//! the first differing byte is found, which leaks the length of the matching
//! prefix; an attacker who can measure that recovers the signature byte by byte
//! and mints any owner id they like. [`constant_time_eq`] compares every byte.
//!
//! **The split is at the LAST `.`, not the first.** An owner id is
//! `user_<uuid>`, which contains no dot today — but splitting at the first dot
//! would make the parse depend on that, and a future owner-id shape would
//! silently start verifying the wrong substring.
//!
//! # Modes
//!
//! `personal` (the default) never issues or checks a cookie: every request
//! resolves to `VIA_PERSONAL_OWNER_ID`, so a WebSocket upgrade never returns
//! 401. `browser` is the multi-tenant mode where the cookie is the identity.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::CoreError;

/// The identity cookie's name.
///
/// **External contract** — upstream `qwen_audio_agent_identity`
/// (`server/src/core/identity.mjs:7`), renamed per `docs/rebrand.md`.
pub const IDENTITY_COOKIE_NAME: &str = "via_identity";

/// `Max-Age` on the identity cookie, in seconds: seven days.
///
/// **External contract** — `server/src/core/identity.mjs:8`
/// (`7 * 24 * 60 * 60`).
pub const IDENTITY_COOKIE_MAX_AGE_SECONDS: u32 = 604_800;

/// The prefix every owner id must carry.
///
/// **External contract** — `server/src/core/identity.mjs:59`. A token whose
/// owner id does not start with it is refused before the signature is trusted.
pub const OWNER_ID_PREFIX: &str = "user_";

/// Shortest acceptable `VIA_AUTH_SECRET`.
///
/// **External contract** — `server/src/core/identity.mjs:35`.
pub const AUTH_SECRET_MIN_LENGTH: usize = 32;

/// The refusal thrown for a short secret, verbatim.
///
/// **External contract** — `server/src/core/identity.mjs:36`, with the
/// variable name renamed per `docs/rebrand.md`. Reproduced as a literal rather
/// than localized: it was already English upstream, it names an environment
/// variable rather than prose, and `contracts.json` records that it "reaches
/// the operator".
pub const AUTH_SECRET_LENGTH_MESSAGE: &str = "VIA_AUTH_SECRET must contain at least 32 characters";

/// The fixed `Set-Cookie` attributes, in upstream's order.
///
/// **External contract** — `server/src/core/identity.mjs:69-75`:
/// `Path=/; HttpOnly; SameSite=Strict; Max-Age=604800`, with `; Secure`
/// appended over TLS.
pub const IDENTITY_COOKIE_ATTRIBUTES: &[&str] = &["Path=/", "HttpOnly", "SameSite=Strict"];

/// Which identity scheme the Gateway runs.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum IdentityMode {
    /// One fixed owner for every request; no cookie is issued or checked.
    #[default]
    Personal,
    /// Per-browser owners, minted and verified through the signed cookie.
    Browser,
}

impl IdentityMode {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Browser => "browser",
        }
    }

    /// Upstream's resolution rule, reproduced exactly.
    ///
    /// **External contract** — `server/src/core/config.mjs:201-203`:
    /// `(env || 'personal').toLowerCase() === 'browser' ? 'browser' : 'personal'`.
    /// **Any** other value silently means `personal`; there is no error path,
    /// so a typo fails closed onto the single-owner mode rather than opening a
    /// multi-tenant one.
    #[must_use]
    pub fn from_env_value(value: Option<&str>) -> Self {
        let raw = value.filter(|text| !text.is_empty()).unwrap_or("personal");
        if raw.to_lowercase() == "browser" {
            Self::Browser
        } else {
            Self::Personal
        }
    }
}

/// A resolved owner.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Identity {
    /// The owner id, always `user_`-prefixed.
    #[serde(rename = "ownerId")]
    pub owner_id: String,
}

/// What resolving an HTTP request produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpIdentity {
    /// The request already carried a valid identity, or the Gateway is in
    /// personal mode. No cookie is set.
    Existing(Identity),
    /// A fresh identity was minted; `set_cookie` is the `Set-Cookie` value.
    Issued {
        /// The minted owner.
        identity: Identity,
        /// The complete `Set-Cookie` header value.
        set_cookie: String,
    },
}

impl HttpIdentity {
    /// The owner either way.
    #[must_use]
    pub fn identity(&self) -> &Identity {
        match self {
            Self::Existing(identity) | Self::Issued { identity, .. } => identity,
        }
    }

    /// The `Set-Cookie` value, when one must be sent.
    #[must_use]
    pub fn set_cookie(&self) -> Option<&str> {
        match self {
            Self::Existing(_) => None,
            Self::Issued { set_cookie, .. } => Some(set_cookie),
        }
    }
}

/// Mints and verifies owner identities.
#[derive(Clone)]
pub struct IdentityManager {
    secret: String,
    cookie_name: String,
    mode: IdentityMode,
    personal_owner_id: String,
}

impl core::fmt::Debug for IdentityManager {
    /// Deliberately omits the secret. A `#[derive(Debug)]` here would put the
    /// HMAC key into any log line that formats a config.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IdentityManager")
            .field("secret", &"[REDACTED]")
            .field("cookie_name", &self.cookie_name)
            .field("mode", &self.mode)
            .field("personal_owner_id", &self.personal_owner_id)
            .finish()
    }
}

impl IdentityManager {
    /// Build a manager.
    ///
    /// `personal_owner_id` has no default here on purpose: its default
    /// (`user_personal`) is a configuration default and lives in
    /// `impl Default for Config`, which is the single place this crate spells
    /// one.
    ///
    /// # Errors
    ///
    /// [`CoreError::AuthSecretTooShort`] when the secret is shorter than
    /// [`AUTH_SECRET_MIN_LENGTH`]. Upstream measures UTF-16 code units; this
    /// measures Unicode scalar values, which differ only for astral characters
    /// and never for the 64 hex characters VIA generates.
    pub fn new(
        secret: &str,
        mode: IdentityMode,
        personal_owner_id: &str,
    ) -> Result<Self, CoreError> {
        if secret.chars().count() < AUTH_SECRET_MIN_LENGTH {
            return Err(CoreError::AuthSecretTooShort(AUTH_SECRET_LENGTH_MESSAGE));
        }
        Ok(Self {
            secret: secret.to_owned(),
            cookie_name: IDENTITY_COOKIE_NAME.to_owned(),
            mode,
            personal_owner_id: personal_owner_id.to_owned(),
        })
    }

    /// Override the cookie name. Upstream exposes the same seam; nothing in
    /// VIA uses it outside tests.
    #[must_use]
    pub fn with_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// The configured mode.
    #[must_use]
    pub fn mode(&self) -> IdentityMode {
        self.mode
    }

    /// The cookie name this manager reads and writes.
    #[must_use]
    pub fn cookie_name(&self) -> &str {
        &self.cookie_name
    }

    /// The identity every request resolves to in [`IdentityMode::Personal`].
    #[must_use]
    pub fn personal_identity(&self) -> Identity {
        Identity {
            owner_id: self.personal_owner_id.clone(),
        }
    }

    /// `base64url(HMAC-SHA256(secret, owner_id))`, unpadded.
    ///
    /// **External contract** — `server/src/core/identity.mjs:46-48`. Node's
    /// `digest('base64url')` emits the URL alphabet with **no** `=` padding.
    #[must_use]
    pub fn sign(&self, owner_id: &str) -> String {
        // `new_from_slice` is infallible for HMAC — the construction accepts a
        // key of any length — so the error arm is unreachable rather than
        // unwrapped.
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()) else {
            return String::new();
        };
        mac.update(owner_id.as_bytes());
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    }

    /// The cookie value for an owner: `<owner_id>.<signature>`.
    #[must_use]
    pub fn token(&self, owner_id: &str) -> String {
        format!("{owner_id}.{}", self.sign(owner_id))
    }

    /// Verify a `Cookie` header and return the owner it proves.
    ///
    /// **External contract** — `server/src/core/identity.mjs:50-64`, in order:
    /// split at the **last** `.`; require the `user_` prefix; require equal
    /// byte lengths; then compare in constant time. Any failure is `None`; a
    /// forged value never mints an owner.
    #[must_use]
    pub fn resolve_token(&self, cookie_header: Option<&str>) -> Option<Identity> {
        let token = parse_cookies(cookie_header.unwrap_or_default())
            .into_iter()
            .filter(|(name, _)| name == &self.cookie_name)
            .map(|(_, value)| value)
            // `Object.fromEntries` keeps the last occurrence of a repeated key.
            .next_back()?;
        if token.is_empty() {
            return None;
        }
        let separator = token.rfind('.')?;
        let owner_id = &token[..separator];
        let supplied = &token.as_bytes()[separator + 1..];
        let expected = self.sign(owner_id);
        if !owner_id.starts_with(OWNER_ID_PREFIX)
            || supplied.len() != expected.len()
            || !constant_time_eq(supplied, expected.as_bytes())
        {
            return None;
        }
        Some(Identity {
            owner_id: owner_id.to_owned(),
        })
    }

    /// Build the `Set-Cookie` value for an owner.
    ///
    /// **External contract** — `server/src/core/identity.mjs:66-79`: the value
    /// is `encodeURIComponent`d, then `Path=/; HttpOnly; SameSite=Strict;
    /// Max-Age=604800`, with `Secure` appended last when the connection is TLS
    /// or `x-forwarded-proto: https`.
    #[must_use]
    pub fn set_cookie_value(&self, owner_id: &str, secure: bool) -> String {
        let mut attributes = vec![format!(
            "{}={}",
            self.cookie_name,
            encode_uri_component(&self.token(owner_id))
        )];
        attributes.extend(IDENTITY_COOKIE_ATTRIBUTES.iter().map(|s| (*s).to_owned()));
        attributes.push(format!("Max-Age={IDENTITY_COOKIE_MAX_AGE_SECONDS}"));
        if secure {
            attributes.push("Secure".to_owned());
        }
        attributes.join("; ")
    }

    /// Mint an identity for a specific owner id.
    ///
    /// Pure: the caller supplies the id. [`Self::issue`] is the impure wrapper
    /// that generates one.
    #[must_use]
    pub fn issue_for(&self, owner_id: &str, secure: bool) -> HttpIdentity {
        HttpIdentity::Issued {
            identity: Identity {
                owner_id: owner_id.to_owned(),
            },
            set_cookie: self.set_cookie_value(owner_id, secure),
        }
    }

    /// Mint a fresh identity: `user_<uuid v4>`.
    ///
    /// **External contract** — `server/src/core/identity.mjs:67`. Lowercase
    /// UUID v4 **with** dashes.
    #[must_use]
    pub fn issue(&self, secure: bool) -> HttpIdentity {
        let owner_id = format!("{OWNER_ID_PREFIX}{}", uuid::Uuid::new_v4());
        self.issue_for(&owner_id, secure)
    }

    /// Resolve an HTTP request, minting an identity if it carries none.
    ///
    /// In [`IdentityMode::Personal`] no cookie is ever read or written.
    #[must_use]
    pub fn resolve_http(&self, cookie_header: Option<&str>, secure: bool) -> HttpIdentity {
        if self.mode == IdentityMode::Personal {
            return HttpIdentity::Existing(self.personal_identity());
        }
        match self.resolve_token(cookie_header) {
            Some(identity) => HttpIdentity::Existing(identity),
            None => self.issue(secure),
        }
    }

    /// Resolve a WebSocket upgrade, which may **not** mint an identity.
    ///
    /// An upgrade has no response headers to carry a `Set-Cookie`, so an
    /// unrecognised client is refused (`401 identity required`) rather than
    /// silently given a new owner.
    #[must_use]
    pub fn resolve_upgrade(&self, cookie_header: Option<&str>) -> Option<Identity> {
        if self.mode == IdentityMode::Personal {
            return Some(self.personal_identity());
        }
        self.resolve_token(cookie_header)
    }
}

/// Compare two byte strings in time independent of their contents.
///
/// Returns `false` immediately for a length mismatch — as
/// `crypto.timingSafeEqual` does, which throws on unequal lengths and is
/// therefore guarded by an explicit length check upstream too. Lengths are not
/// secret here: the signature length is fixed by SHA-256.
///
/// The accumulator is passed through [`core::hint::black_box`] so the optimizer
/// cannot rewrite the fold into an early return.
#[must_use]
pub fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        difference |= a ^ b;
    }
    core::hint::black_box(difference) == 0
}

/// Parse a `Cookie` header into name/value pairs, in header order.
///
/// **External contract** — `server/src/core/identity.mjs:10-26`: split on `;`,
/// trim, drop empties, split each at the **first** `=`, and
/// `decodeURIComponent` the value — with a malformed escape yielding the empty
/// string rather than throwing. A part with no `=` becomes `(part, "")`.
#[must_use]
pub fn parse_cookies(header: &str) -> Vec<(String, String)> {
    header
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| match part.find('=') {
            None => (part.to_owned(), String::new()),
            Some(separator) => (
                part[..separator].to_owned(),
                decode_uri_component(&part[separator + 1..]).unwrap_or_default(),
            ),
        })
        .collect()
}

/// `decodeURIComponent`, returning `None` where JavaScript throws `URIError`.
///
/// Two failure modes, both of which upstream catches into an empty string: a
/// truncated or non-hex `%` escape (`%E0%A4%A`), and a sequence of escapes that
/// does not decode as UTF-8.
#[must_use]
pub fn decode_uri_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).copied().and_then(hex_value)?;
            let low = bytes.get(index + 2).copied().and_then(hex_value)?;
            out.push(high * 16 + low);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    (byte as char).to_digit(16).map(|digit| digit as u8)
}

/// `encodeURIComponent`: percent-encode every byte outside
/// `A-Z a-z 0-9 - _ . ! ~ * ' ( )`, uppercase hex over UTF-8.
///
/// Every character a VIA token can contain is already unreserved — the base64url
/// alphabet plus `_` and `.` — so in practice this is the identity function. It
/// is reproduced anyway because a future owner-id shape would silently start
/// needing it.
#[must_use]
pub fn encode_uri_component(value: &str) -> String {
    const UNRESERVED: &[u8] = b"-_.!~*'()";
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        if byte.is_ascii_alphanumeric() || UNRESERVED.contains(byte) {
            out.push(*byte as char);
        } else {
            out.push('%');
            out.push_str(&format!("{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-that-is-longer-than-32-characters";

    fn manager() -> IdentityManager {
        IdentityManager::new(SECRET, IdentityMode::Browser, "user_personal")
            .expect("a 45-character secret is above the minimum")
    }

    #[test]
    fn a_short_secret_is_refused_with_the_upstream_sentence() {
        let error = IdentityManager::new("short", IdentityMode::Browser, "user_personal")
            .expect_err("31 characters is below the minimum");
        assert_eq!(error.to_string(), AUTH_SECRET_LENGTH_MESSAGE);
    }

    #[test]
    fn a_forged_token_resolves_to_nothing() {
        let manager = manager();
        assert_eq!(
            manager.resolve_token(Some("via_identity=attacker-selected-client-id")),
            None
        );
    }

    #[test]
    fn a_malformed_escape_does_not_panic() {
        assert_eq!(manager().resolve_token(Some("via_identity=%E0%A4%A")), None);
        assert_eq!(decode_uri_component("%E0%A4%A"), None);
    }

    #[test]
    fn constant_time_eq_agrees_with_equality() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
