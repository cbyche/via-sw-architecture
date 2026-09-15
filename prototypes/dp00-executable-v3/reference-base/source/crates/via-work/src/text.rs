//! The one string primitive the Work record needs that Layer 3 does not.
//!
//! `via_downstream::text::bounded` is the *sanitising* bound: it collapses
//! whitespace runs before cutting, because it stands between a hostile backend
//! and VIA's public progress surface. The Work record's own bounds are not
//! that. `server/src/task/task-manager.mjs:79,84` writes
//!
//! ```js
//! String(task.delegation.title || '').slice(0, 160)
//! ```
//!
//! — a plain `slice`, with no collapsing. Reusing `bounded` here would rewrite
//! a delegation title the coordinator chose, so the two are kept apart.

/// `String.prototype.slice(0, max)`.
///
/// Counts **UTF-16 code units**, as JavaScript does, so every bound agrees with
/// upstream's for any text in the Basic Multilingual Plane.
///
/// # The one deviation
///
/// Upstream will cut an astral character in half and emit a lone surrogate; a
/// Rust `String` cannot hold one, so this stops one character early instead.
/// The same trade `via_downstream::text::bounded` records, for the same reason.
#[must_use]
pub fn slice_units(value: &str, max: usize) -> String {
    let mut out = String::with_capacity(value.len().min(max.saturating_mul(4)));
    let mut units = 0usize;
    for character in value.chars() {
        let width = character.len_utf16();
        if units + width > max {
            break;
        }
        units += width;
        out.push(character);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::slice_units;

    #[test]
    fn slices_by_utf16_code_units() {
        assert_eq!(slice_units("abcdef", 3), "abc");
        assert_eq!(slice_units("abc", 10), "abc");
        assert_eq!(slice_units("", 5), "");
    }

    #[test]
    fn does_not_collapse_whitespace_the_way_bounded_does() {
        assert_eq!(slice_units("a \t\n b", 6), "a \t\n b");
        assert_eq!(via_downstream::text::bounded("a \t\n b", 6), "a b");
    }

    #[test]
    fn never_splits_an_astral_character() {
        assert_eq!(slice_units("\u{1f600}", 1), "");
        assert_eq!(slice_units("\u{1f600}", 2), "\u{1f600}");
        assert_eq!(slice_units("a\u{1f600}b", 2), "a");
    }

    #[test]
    fn a_chinese_character_is_one_unit() {
        assert_eq!(slice_units("任务状态", 2), "任务");
    }
}
