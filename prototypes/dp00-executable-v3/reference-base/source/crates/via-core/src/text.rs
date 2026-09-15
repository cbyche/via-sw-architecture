//! ECMAScript's whitespace set, in one place.
//!
//! Four crates needed "is this character whitespace *to JavaScript*" and, until
//! this module, four crates answered it separately: `via-downstream::text` and
//! `via-conversation::text` each enumerated the set, `via-voice::text`
//! approximated it as `char::is_whitespace() || U+FEFF`, and `via-core::env`
//! carried a private fourth copy for `Number()`. `docs/deviations/phase-4.md`
//! recorded the duplication and named `via-core` as the predicate's real home;
//! this is that home, and the other three now re-export it.
//!
//! # Why `char::is_whitespace` is not the answer
//!
//! `\s` in ECMAScript and Rust's [`char::is_whitespace`] are different sets and
//! they disagree in **both** directions:
//!
//! | Code point | ECMAScript `\s` | Rust `White_Space` |
//! | --- | --- | --- |
//! | U+0085 NEXT LINE | no | yes |
//! | U+FEFF ZWNBSP | yes | no |
//!
//! Both directions matter and neither is theoretical. A U+FEFF that Rust
//! declined to collapse sits invisibly inside a "bounded" progress detail or a
//! memory bullet — the smuggling the bound exists to stop. A U+0085 that Rust
//! *did* trim would let `"\u{85}owner"` and `"owner"` resolve to the same
//! coordinator session key where upstream keeps them apart, which is a
//! cross-owner collision on a key derived from user input.
//!
//! So the set is enumerated from the specification rather than approximated.

/// Whether `c` is matched by ECMAScript's `\s`.
///
/// The set is `WhiteSpace ∪ LineTerminator ∪ U+FEFF` (ECMA-262 §12.2, §22.2.2):
/// TAB, LF, VT, FF, CR, SPACE, NBSP, LS, PS, ZWNBSP, plus every code point in
/// the Unicode `Space_Separator` (`Zs`) category.
///
/// This is also JavaScript's `StrWhiteSpace` — what `Number()` and
/// `String.prototype.trim` strip — so the numeric parser in
/// [`crate::env`] uses it too.
///
/// ```
/// use via_core::text::is_js_whitespace;
///
/// assert!(is_js_whitespace('\u{feff}'));   // `\s`, not `White_Space`
/// assert!(!is_js_whitespace('\u{85}'));    // `White_Space`, not `\s`
/// assert!(is_js_whitespace(' '));
/// assert!(!is_js_whitespace('a'));
/// ```
#[must_use]
pub const fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        // WhiteSpace, the non-Zs half.
        '\u{9}' | '\u{b}' | '\u{c}' | '\u{feff}'
        // LineTerminator.
        | '\u{a}' | '\u{d}' | '\u{2028}' | '\u{2029}'
        // Zs.
        | '\u{20}' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

#[cfg(test)]
mod tests {
    use super::is_js_whitespace;

    /// The set, member by member, against ECMA-262 §12.2's table.
    #[test]
    fn every_member_of_the_specified_set_is_whitespace() {
        for c in [
            '\u{9}', '\u{b}', '\u{c}', '\u{feff}', // WhiteSpace, non-Zs
            '\u{a}', '\u{d}', '\u{2028}', '\u{2029}', // LineTerminator
            '\u{20}', '\u{a0}', '\u{1680}', '\u{202f}', '\u{205f}', '\u{3000}', // Zs
        ] {
            assert!(is_js_whitespace(c), "U+{:04X} should be `\\s`", c as u32);
        }
        // The whole U+2000..=U+200A run is `Zs`.
        for point in 0x2000..=0x200a_u32 {
            let c = char::from_u32(point).expect("a real code point");
            assert!(is_js_whitespace(c), "U+{point:04X} should be `\\s`");
        }
    }

    /// The two disagreements with Rust, asserted in both directions so a
    /// "simplification" to `char::is_whitespace` fails here.
    #[test]
    fn the_set_disagrees_with_rust_in_both_directions() {
        assert!(is_js_whitespace('\u{feff}') && !'\u{feff}'.is_whitespace());
        assert!(!is_js_whitespace('\u{85}') && '\u{85}'.is_whitespace());
    }

    #[test]
    fn ordinary_text_is_not_whitespace() {
        for c in ['a', 'Z', '0', '_', '-', '\u{200b}', '\u{4e00}'] {
            assert!(!is_js_whitespace(c), "U+{:04X} is not `\\s`", c as u32);
        }
    }
}
