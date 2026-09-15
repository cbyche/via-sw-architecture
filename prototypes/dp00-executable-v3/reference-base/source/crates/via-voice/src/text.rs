//! The string primitives the voice layer is built from.
//!
//! Upstream reaches for `String(value || '').replace(/\s+/g, ' ').trim()` and
//! `.slice(0, n)` in a dozen files. Both hide a unit: JavaScript's `slice`
//! counts UTF-16 code units, `[...string].slice(0, n)` counts code points, and
//! neither is bytes. Every catalogued bound in this crate says which one it
//! means, so the two are separate functions here rather than one `truncate`.

/// Collapse every run of whitespace to one space and trim the ends.
///
/// **External contract** — `frontend-agent-context.mjs:13-15` and
/// `tool-call-handler.mjs:510`, the same expression in both. JavaScript's
/// `\s` covers the Unicode space separators plus `\t\n\r\x0b\x0c` and
/// `﻿`; [`char::is_whitespace`] covers `White_Space`, which differs only
/// on `﻿` — a zero-width no-break space that is not `White_Space` in
/// Unicode. It is handled explicitly so a BOM pasted into an objective
/// collapses the same way it does upstream.
#[must_use]
pub fn clean(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_run = false;
    for character in value.chars() {
        if is_js_whitespace(character) {
            in_run = true;
            continue;
        }
        if in_run && !out.is_empty() {
            out.push(' ');
        }
        in_run = false;
        out.push(character);
    }
    out
}

/// `String.prototype.trim()`.
///
/// Trims exactly what [`clean`] collapses, so the two agree about what
/// whitespace is.
#[must_use]
pub fn trim(value: &str) -> &str {
    value.trim_matches(is_js_whitespace)
}

/// Whether `character` is whitespace to JavaScript's `\s` and `trim`.
///
/// Re-exported from [`via_core::text`]. This crate previously approximated the
/// set as `char::is_whitespace() || U+FEFF`, which is a **superset**: it also
/// matches U+0085 NEXT LINE, which Rust classes as `White_Space` and
/// ECMAScript's `\s` does not. That made `clean` collapse a character upstream
/// leaves alone. The shared definition enumerates the specified set, so the two
/// disagreements with Rust — U+FEFF in, U+0085 out — are both reproduced.
pub use via_core::text::is_js_whitespace;

/// `[...value].slice(0, max).join('')` — a **code point** bound.
///
/// Used wherever upstream spreads the string first: the prompt documents, the
/// announcement's result truncation, the batch's size probe.
#[must_use]
pub fn bounded_code_points(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

/// The number of code points in `value`.
#[must_use]
pub fn code_point_len(value: &str) -> usize {
    value.chars().count()
}

/// `value.slice(0, max)` — a **UTF-16 code unit** bound.
///
/// Used wherever upstream slices the string directly: the objective clip, the
/// result clip, the activity detail, the holder name. A bound that would land
/// inside a surrogate pair stops before it rather than splitting one, which is
/// the only place this can differ from JavaScript — and JavaScript's answer
/// there is an unpaired surrogate, which Rust has no way to hold.
#[must_use]
pub fn bounded_utf16(value: &str, max: usize) -> String {
    let mut used = 0usize;
    let mut out = String::new();
    for character in value.chars() {
        let width = character.len_utf16();
        if used + width > max {
            break;
        }
        used += width;
        out.push(character);
    }
    out
}

/// The number of UTF-16 code units in `value` — JavaScript's `.length`.
#[must_use]
pub fn utf16_len(value: &str) -> usize {
    value.chars().map(char::len_utf16).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_collapses_runs_and_trims_both_ends() {
        assert_eq!(clean("  a \n\t b  "), "a b");
        assert_eq!(clean(""), "");
        assert_eq!(clean("   "), "");
        // A BOM is `\s` to JavaScript but not `White_Space` to Unicode.
        assert_eq!(clean("a\u{feff}b"), "a b");
    }

    #[test]
    fn the_two_bounds_disagree_exactly_where_they_should() {
        // U+1F600 is one code point and two UTF-16 units.
        let emoji = "😀😀😀";
        assert_eq!(bounded_code_points(emoji, 2), "😀😀");
        assert_eq!(bounded_utf16(emoji, 2), "😀");
        assert_eq!(code_point_len(emoji), 3);
        assert_eq!(utf16_len(emoji), 6);
    }

    #[test]
    fn a_utf16_bound_never_splits_a_surrogate_pair() {
        assert_eq!(bounded_utf16("a😀", 2), "a");
        assert_eq!(bounded_utf16("a😀", 3), "a😀");
    }
}
