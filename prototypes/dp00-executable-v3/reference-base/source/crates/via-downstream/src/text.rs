//! The two string primitives every Layer-3 crate needs, ported exactly.
//!
//! Upstream defines both in `server/src/agent/acp-backend-session-utils.mjs:1-7`
//! and repeats `clean` in `server/src/agent/backends/shared.mjs:10-12`:
//!
//! ```js
//! function clean(value) { return String(value || '').trim() }
//! function bounded(value, max = 300) {
//!   return clean(value).replace(/\s+/g, ' ').slice(0, max)
//! }
//! ```
//!
//! They look incidental and are not. [`clean`] is what makes the coordinator
//! session key stable across a caller that passes `" owner "`, and [`bounded`]
//! is the *only* thing standing between a hostile backend and an unbounded
//! string on VIA's public progress surface — see [`crate::event`].
//!
//! # Why the whitespace set is spelled out
//!
//! `\s` in ECMAScript and [`char::is_whitespace`] in Rust are not the same set.
//! ECMAScript's is `WhiteSpace ∪ LineTerminator ∪ U+FEFF`; Rust's is the Unicode
//! `White_Space` property. They disagree in both directions — U+0085 (NEL) is
//! `White_Space` but not `\s`, and U+FEFF (zero-width no-break space) is `\s`
//! but not `White_Space`. A U+FEFF that Rust declined to collapse would sit
//! inside a "bounded" detail as an invisible character, which is exactly the
//! kind of smuggling this function exists to stop, so [`is_js_whitespace`]
//! reproduces ECMAScript's set rather than approximating it.

/// The default bound `bounded()` applies when a call site names no other:
/// `server/src/agent/acp-backend-session-utils.mjs:5`.
pub const DEFAULT_BOUND: usize = 300;

/// Whether `c` is matched by ECMAScript's `\s`.
///
/// Re-exported from [`via_core::text`], which is where
/// `docs/deviations/phase-4.md` said the predicate belonged: it was defined
/// three times over — here, in `via-conversation` and in `via-voice` — and the
/// third copy had quietly drifted into a *different* set. Kept in this module's
/// namespace so every call site reads unchanged.
pub use via_core::text::is_js_whitespace;

/// `String(value || '').trim()`.
///
/// Upstream `clean`, `server/src/agent/acp-backend-session-utils.mjs:1-3`. The
/// `|| ''` arm is Rust's `&str` by construction — there is no `null` to
/// coerce — so what remains is the trim, over ECMAScript's whitespace set.
#[must_use]
pub fn clean(value: &str) -> &str {
    value.trim_matches(is_js_whitespace)
}

/// `clean(value).replace(/\s+/g, ' ').slice(0, max)`.
///
/// Upstream `bounded`, `server/src/agent/acp-backend-session-utils.mjs:5-7`.
///
/// # The one deviation
///
/// `String.prototype.slice` counts **UTF-16 code units**, so upstream will
/// happily cut an astral character in half and emit a lone surrogate. A Rust
/// `String` cannot hold one. This counts UTF-16 code units too — so every
/// bound agrees with upstream's byte for byte on any text inside the Basic
/// Multilingual Plane — but stops one character early rather than splitting a
/// surrogate pair. Recorded in `docs/deviations/phase-2.md`.
#[must_use]
pub fn bounded(value: &str, max: usize) -> String {
    let trimmed = clean(value);
    let mut out = String::with_capacity(trimmed.len().min(max.saturating_mul(4)));
    let mut units = 0usize;
    let mut in_run = false;
    for c in trimmed.chars() {
        let (c, width) = if is_js_whitespace(c) {
            if in_run {
                continue;
            }
            in_run = true;
            (' ', 1)
        } else {
            in_run = false;
            (c, c.len_utf16())
        };
        if units + width > max {
            break;
        }
        units += width;
        out.push(c);
    }
    out
}

/// `encodeURIComponent(value)`.
///
/// ECMA-262 §19.2.6.5: every code point outside
/// `A–Z a–z 0–9 - _ . ! ~ * ' ( )` becomes `%XX` per UTF-8 byte, with
/// **upper-case** hex digits. Used by [`SessionKey`](crate::SessionKey), where
/// the encoding is part of a persisted identity and not a display choice.
///
/// Upstream can throw `URIError` on a lone surrogate; a `&str` is valid UTF-8,
/// so this cannot.
#[must_use]
pub fn encode_uri_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => out.push(char::from(byte)),
            _ => {
                out.push('%');
                out.push(char::from(HEX[usize::from(byte >> 4)]));
                out.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_trims_the_ecmascript_whitespace_set() {
        assert_eq!(clean("  owner  "), "owner");
        assert_eq!(clean("\t\n\r\u{b}\u{c}owner\u{feff}"), "owner");
        assert_eq!(clean("\u{3000}owner\u{2028}"), "owner");
        // U+0085 is Rust `White_Space` but not ECMAScript `\s`, so it stays.
        assert_eq!(clean("\u{85}owner"), "\u{85}owner");
    }

    #[test]
    fn bounded_collapses_runs_and_cuts_at_the_bound() {
        assert_eq!(bounded("  a \t\n  b  ", DEFAULT_BOUND), "a b");
        assert_eq!(bounded("abcdef", 3), "abc");
        assert_eq!(bounded("", DEFAULT_BOUND), "");
        // A collapsed run counts as the one space it became.
        assert_eq!(bounded("a\u{a0}\u{2003}b", 3), "a b");
    }

    #[test]
    fn bounded_never_splits_an_astral_character() {
        // "\u{1f600}" is two UTF-16 code units. A bound of 1 must drop it
        // rather than emit half of a surrogate pair.
        assert_eq!(bounded("\u{1f600}", 1), "");
        assert_eq!(bounded("\u{1f600}", 2), "\u{1f600}");
        assert_eq!(bounded("a\u{1f600}b", 2), "a");
    }

    #[test]
    fn encode_uri_component_matches_the_unreserved_set() {
        assert_eq!(encode_uri_component("owner one"), "owner%20one");
        assert_eq!(encode_uri_component("-_.!~*'()"), "-_.!~*'()");
        assert_eq!(encode_uri_component(":/?#[]@"), "%3A%2F%3F%23%5B%5D%40");
        assert_eq!(encode_uri_component("\u{4f60}"), "%E4%BD%A0");
        assert_eq!(encode_uri_component("a+b"), "a%2Bb");
    }
}
