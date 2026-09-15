//! The string primitives the conversation surface is built from.
//!
//! Four functions, each a port of a specific upstream helper, and each one
//! observable:
//!
//! | Rust | Upstream |
//! | --- | --- |
//! | [`clean`] | `conversation-sync.mjs:5-7`, `frontend-agent-context.mjs:13-15` |
//! | [`clean_bounded`] | `frontend-notes.mjs:21-25` |
//! | [`bounded_code_points`] | `frontend-agent-context.mjs:71,80` |
//! | [`normalize`] | `markdown-context-store.mjs:16-18`, `memory-extractor.mjs:86` |
//!
//! # Why the whitespace set is spelled out again
//!
//! `\s` in ECMAScript and [`char::is_whitespace`] in Rust are different sets —
//! U+0085 is `White_Space` but not `\s`, U+FEFF is `\s` but not `White_Space` —
//! and a U+FEFF that went uncollapsed would sit invisibly inside a list item or
//! a memory bullet. [`is_js_whitespace`] therefore reproduces ECMAScript's set
//! (ECMA-262 §12.2: `WhiteSpace ∪ LineTerminator ∪ U+FEFF`).
//!
//! The predicate itself is no longer defined here. It was duplicated in
//! `via-downstream` for a good reason — `conversation → conversation, core,
//! shared` has no `agent` edge — and `docs/deviations/phase-4.md` said the
//! answer was to hoist it to `via-core`, which both crates already depend on.
//! That is done: [`is_js_whitespace`] is `via_core::text`'s, re-exported.
//!
//! # Two different truncation units, on purpose
//!
//! Upstream truncates with two different operators and they do not agree:
//! `[...value].slice(0, n)` spreads into **code points**, while
//! `value.slice(0, n)` counts **UTF-16 code units**. Every truncation in this
//! crate's scope uses the spread form, so [`clean_bounded`] and
//! [`bounded_code_points`] count code points; [`utf16_len`] exists for the one
//! place a *budget* is measured with `String.prototype.length`
//! (`memory-extractor.mjs:145`, `frontend-agent-context.mjs:130`).

/// Whether `c` is matched by ECMAScript's `\s`.
///
/// Re-exported from [`via_core::text`]. This module used to carry its own copy
/// and say so; `docs/deviations/phase-4.md` recorded that the predicate's real
/// home was `via-core`, and it now lives there. Kept in this module's namespace
/// so every call site reads unchanged.
pub use via_core::text::is_js_whitespace;

/// `String(value || '').replace(/\s+/g, ' ').trim()`.
///
/// Contract — `server/src/conversation/conversation-sync.mjs:5-7` and
/// `server/src/conversation/frontend-agent-context.mjs:13-15`. Every run of
/// whitespace collapses to one U+0020 and the result is trimmed, so a message
/// that arrived with an embedded newline cannot forge a second transcript line.
#[must_use]
pub fn clean(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut pending_space = false;
    for c in value.chars() {
        if is_js_whitespace(c) {
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(c);
    }
    out
}

/// `[...String(value || '').replace(/\s+/g, ' ').trim()].slice(0, max).join('')`.
///
/// Contract — `server/src/conversation/frontend-notes.mjs:21-25`. The spread
/// makes the bound a **code-point** count, which is what a 30-code-point list
/// name and a 100-code-point item mean.
#[must_use]
pub fn clean_bounded(value: &str, max: usize) -> String {
    bounded_code_points(&clean(value), max)
}

/// `[...value].slice(0, max).join('')`.
///
/// Contract — `server/src/conversation/frontend-agent-context.mjs:71,80`
/// (`MAX_PROMPT_CHARS`, `MAX_ASSISTANT_CHARS`),
/// `markdown-context-store.mjs:92` and `memory-extractor.mjs:88`.
#[must_use]
pub fn bounded_code_points(value: &str, max: usize) -> String {
    match value.char_indices().nth(max) {
        Some((offset, _)) => value[..offset].to_owned(),
        None => value.to_owned(),
    }
}

/// `value.slice(0, max)` — a **UTF-16 code unit** bound.
///
/// The spelling upstream uses where it does *not* spread first:
/// `frontend-agent-context.mjs:28,38` (`locale`, `workingDirectory`).
///
/// # The one deviation
///
/// `String.prototype.slice` will cut an astral character in half and emit a
/// lone surrogate, which a Rust `String` cannot hold. This stops one character
/// short instead, so it agrees with upstream byte for byte on any text inside
/// the Basic Multilingual Plane. Identical to `via_downstream::text::bounded`'s
/// recorded behaviour.
#[must_use]
pub fn bounded_utf16(value: &str, max: usize) -> String {
    let mut out = String::with_capacity(value.len().min(max.saturating_mul(4)));
    let mut units = 0usize;
    for c in value.chars() {
        let width = c.len_utf16();
        if units + width > max {
            break;
        }
        units += width;
        out.push(c);
    }
    out
}

/// The number of code points in `value` — JavaScript's `[...value].length`.
#[must_use]
pub fn code_point_len(value: &str) -> usize {
    value.chars().count()
}

/// The number of UTF-16 code units in `value` — JavaScript's `value.length`.
///
/// Used only where upstream measures a *budget* with `String.prototype.length`:
/// the transcript budget (`memory-extractor.mjs:145`) and the recent-context
/// budget (`frontend-agent-context.mjs:130`).
#[must_use]
pub fn utf16_len(value: &str) -> usize {
    value.chars().map(char::len_utf16).sum()
}

/// `String(value || '').replaceAll('\0', '').replace(/\r\n?/g, '\n')`.
///
/// Contract — `server/src/conversation/markdown-context-store.mjs:16-18` and
/// `memory-extractor.mjs:86`. NUL is stripped because these strings become
/// file *contents* and a path or a document with an embedded NUL truncates
/// silently at the syscall boundary; CRLF and lone CR both fold to LF so an
/// `old_text` typed on Windows still matches a document written on Unix.
#[must_use]
pub fn normalize(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\0' => {}
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push('\n');
            }
            other => out.push(other),
        }
    }
    out
}

/// `String.prototype.trimEnd()` over ECMAScript's whitespace set.
#[must_use]
pub fn trim_end(value: &str) -> &str {
    value.trim_end_matches(is_js_whitespace)
}

/// `String.prototype.trim()` over ECMAScript's whitespace set.
#[must_use]
pub fn trim(value: &str) -> &str {
    value.trim_matches(is_js_whitespace)
}

/// How many non-overlapping times `needle` occurs in `haystack`.
///
/// Contract — `server/src/conversation/markdown-context-store.mjs:35-44`. The
/// scan advances by `needle.length`, so `aa` occurs **once** in `aaa`, and
/// that is what decides `ambiguous_edit` versus a clean replace. An empty
/// needle counts zero, matching upstream's `if (!needle) return 0`.
#[must_use]
pub fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut offset = 0;
    while let Some(found) = haystack[offset..].find(needle) {
        count += 1;
        offset += found + needle.len();
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_collapses_every_ecmascript_whitespace_run() {
        assert_eq!(clean("  a \t\n  b  "), "a b");
        assert_eq!(clean("\u{feff}a\u{3000}b\u{2028}"), "a b");
        assert_eq!(clean(""), "");
        assert_eq!(clean("   "), "");
        // U+0085 is Rust `White_Space` but not ECMAScript `\s`, so it survives.
        assert_eq!(clean("\u{85}a"), "\u{85}a");
    }

    #[test]
    fn clean_bounded_counts_code_points_not_utf16_units() {
        // Two astral characters are four UTF-16 units but two code points.
        assert_eq!(clean_bounded("\u{1f600}\u{1f601}", 2), "\u{1f600}\u{1f601}");
        assert_eq!(clean_bounded("\u{1f600}\u{1f601}", 1), "\u{1f600}");
        assert_eq!(utf16_len("\u{1f600}\u{1f601}"), 4);
        assert_eq!(code_point_len("\u{1f600}\u{1f601}"), 2);
    }

    #[test]
    fn normalize_strips_nul_and_folds_both_newline_spellings() {
        assert_eq!(normalize("a\0b"), "ab");
        assert_eq!(normalize("a\r\nb\rc\nd"), "a\nb\nc\nd");
        assert_eq!(normalize("trailing\r"), "trailing\n");
    }

    #[test]
    fn count_occurrences_advances_past_the_match() {
        assert_eq!(count_occurrences("aaa", "aa"), 1);
        assert_eq!(count_occurrences("abab", "ab"), 2);
        assert_eq!(count_occurrences("abc", ""), 0);
        assert_eq!(count_occurrences("", "a"), 0);
        assert_eq!(count_occurrences("称呼称呼", "称呼"), 2);
    }
}
