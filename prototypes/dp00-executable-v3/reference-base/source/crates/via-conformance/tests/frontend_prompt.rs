//! The two model-visible documents, byte for byte.
//!
//! `config/frontend-agent/PROMPT.md` and `config/frontend-agent/ASSISTANT.md`
//! are the only upstream files whose *whole text* is a contract: the model
//! reads them verbatim on every voice turn, so a dropped clause is a behaviour
//! change no type can catch. The catalogue records `PROMPT.md` as eight
//! `prompt-text` sections and `ASSISTANT.md` as one complete record.
//!
//! VIA packages them in `via-voice`
//! (`crates/via-voice/assets/frontend-agent/{en,zh,ko}/`), and `zh` is the
//! locale upstream wrote, so `zh` is the one comparable to the catalogue —
//! `en` and `ko` are authored peers (`docs/architecture.md` §16) and
//! `via-voice`'s own `the_three_locales_have_structural_parity` is what holds
//! those to the same shape.
//!
//! `via-voice` asserts that the three locales agree structurally, that no
//! packaged document carries the old brand, and that each fits its assembly
//! bound. What nothing asserted before this file is the thing the catalogue
//! actually records: **that the shipped `zh` bytes are upstream's**. These two
//! tests close that, which is why `via-conformance` grows an edge to
//! `via-voice` rather than classifying the nine rows against a test elsewhere.

use pretty_assertions::assert_eq;
use via_conformance::expect_contract;
use via_i18n::Locale;
use via_voice::prompt::{packaged_assistant_profile, packaged_prompt};

/// Every catalogued `PROMPT.md` section, in the catalogue's own file order.
///
/// The names carry the upstream line ranges in their `file` field, and
/// [`the_packaged_zh_prompt_is_the_catalogued_document`] asserts the sections
/// tile the shipped document in this order — so a section that moved, or one
/// the catalogue gained, fails rather than being silently skipped.
const PROMPT_SECTIONS: &[&str] = &[
    "PROMPT.md § # Role",
    "PROMPT.md § # Instruction hierarchy",
    "PROMPT.md § # Routing (paragraphs 1-2)",
    "PROMPT.md § # Routing (paragraphs 3-4)",
    "PROMPT.md § # Background work",
    "PROMPT.md § # Permission requests",
    "PROMPT.md § # Personalization and memory",
    "PROMPT.md § # Voice interaction",
];

/// Resolve the two-character escapes the catalogue writes for a newline and a
/// tab.
///
/// The surveyor quoted JavaScript source, where a multi-line string appears
/// with `\n` spelled out. Comparing against a real file means resolving them
/// first; every other backslash is left exactly as it is, so a literal `\d`
/// inside a quoted regex survives.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// The eight catalogued sections are the shipped `zh` `PROMPT.md`.
///
/// Byte for byte and **in order**: each section is located from where the last
/// one ended, so a reordered document fails even though every section would
/// still be present. What sits between two sections is asserted to be blank
/// separation only, which is what makes "these eight sections are the whole
/// file" a claim this test actually checks rather than one it assumes.
#[test]
fn the_packaged_zh_prompt_is_the_catalogued_document() {
    let shipped = packaged_prompt(Locale::Zh);
    let mut cursor = 0usize;

    for name in PROMPT_SECTIONS {
        let catalogued = unescape(&expect_contract("prompt-text", name).exact_value);
        let found = shipped[cursor..].find(&catalogued).unwrap_or_else(|| {
            panic!(
                "`{name}` is not in crates/via-voice/assets/frontend-agent/zh/PROMPT.md \
                 at or after byte {cursor}.\n  catalogued: {catalogued:?}\n  \
                 remaining:  {:?}",
                &shipped[cursor..],
            );
        });
        let between = &shipped[cursor..cursor + found];
        assert!(
            between.trim().is_empty(),
            "`{name}` is preceded by text the catalogue does not record: {between:?}"
        );
        cursor += found + catalogued.len();
    }

    assert!(
        shipped[cursor..].trim().is_empty(),
        "the shipped prompt carries text after the last catalogued section: {:?}",
        &shipped[cursor..]
    );
    // A guard on the fixture: eight sections that matched an empty document
    // would satisfy every assertion above.
    assert!(
        shipped.len() > 1_000,
        "the packaged prompt is suspiciously short"
    );
}

/// The persona is upstream's, with `docs/rebrand.md`'s one rename applied.
///
/// `docs/rebrand.md` renames the identity sentence and **only** that sentence
/// (`没有当前用户的个性化覆盖时，你叫 <name>。`, with the space before the name its own
/// instruction). The upstream name is lifted out of the catalogued text rather
/// than typed here, so this test cannot drift from the catalogue and
/// `scripts/brand_leak.py` has nothing to flag; the assertion is that applying
/// that one substitution to upstream's document yields ours exactly.
#[test]
fn the_packaged_zh_persona_is_upstreams_with_the_one_documented_rename() {
    const OPEN: &str = "你叫";
    const CLOSE: char = '。';
    const VIA_IDENTITY: &str = "你叫 VIA。";

    let catalogued = unescape(
        &expect_contract("prompt-text", "ASSISTANT.md (complete, model-visible)").exact_value,
    );
    // The surveyor quoted the file's text, not its terminating newline; the
    // packaged file is a real file and ends with one.
    let shipped = packaged_assistant_profile(Locale::Zh);
    assert!(
        shipped.ends_with('\n') && !shipped.ends_with("\n\n"),
        "the packaged persona should end with exactly one newline"
    );
    let shipped = shipped.trim_end_matches('\n');

    let at = catalogued
        .find(OPEN)
        .expect("the catalogued persona still names the assistant");
    let end = catalogued[at..]
        .find(CLOSE)
        .expect("the identity sentence still ends in a full stop");
    let upstream_identity = &catalogued[at..at + end + CLOSE.len_utf8()];

    // The rename is real: upstream's sentence is not already ours.
    assert_ne!(
        upstream_identity, VIA_IDENTITY,
        "the catalogued persona already names VIA — the rename has become a no-op"
    );
    assert_eq!(
        catalogued.replace(upstream_identity, VIA_IDENTITY),
        shipped,
        "the packaged zh persona is not upstream's with the documented rename applied"
    );
    // And nothing else moved: everything outside that sentence is identical.
    assert_eq!(
        catalogued.replacen(upstream_identity, "", 1),
        shipped.replacen(VIA_IDENTITY, "", 1),
    );
}
