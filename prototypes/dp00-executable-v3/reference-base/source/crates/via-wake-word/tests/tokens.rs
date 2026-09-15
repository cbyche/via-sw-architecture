//! The token inventory, and the rule it reproduces.
//!
//! [`TokenInventory::unknown_tokens`] is a transcription of `EncodeBase` in
//! `sherpa-onnx/csrc/utils.cc`. It has to agree with that function exactly, in
//! both directions: rejecting a keyword file the engine would accept turns a
//! working wake word off, and accepting one the engine would reject ends the
//! process. So the tests below are written from the C++ control flow —
//! *contains first, marker second, otherwise out of vocabulary* — rather than
//! from what a token "looks like".

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_wake_word::{KEYWORD_MARKERS, TokenInventory, WakeWordError};

/// A `tokens.txt` in the real shape, with the three control symbols the
/// catalogued model opens with.
const TOKENS: &str =
    "<blk> 0\n<sos/eos> 1\n<unk> 2\nAY1 3\nT 4\nAH1 5\nP 6\nL 7\nn 8\nǐ 9\nh 10\nǎo 11\n";

fn inventory() -> TokenInventory {
    TokenInventory::parse(TOKENS)
}

// ── parsing ─────────────────────────────────────────────────────────────────

#[test]
fn the_symbol_is_the_first_field_of_each_line() {
    let inventory = inventory();
    assert_eq!(inventory.len(), 12);
    for symbol in ["<blk>", "<sos/eos>", "<unk>", "AY1", "T", "ǎo"] {
        assert!(inventory.contains(symbol), "{symbol} is missing");
    }
    assert!(!inventory.contains("0"), "an id is not a symbol");
    assert!(!inventory.contains("AY2"));
    assert!(!inventory.is_empty());
}

#[test]
fn matching_is_exact() {
    // ARPAbet stress digits are part of the symbol, and the model is
    // case-sensitive: `t` and `T` are a pinyin initial and an English phoneme.
    let inventory = inventory();
    assert!(inventory.contains("T"));
    assert!(!inventory.contains("t"));
    assert!(inventory.contains("n"));
    assert!(!inventory.contains("N"));
    assert!(!inventory.contains("AY"));
    assert!(!inventory.contains(" T"));
}

#[test]
fn a_line_without_an_id_adds_nothing() {
    // The library encodes the space symbol as a line carrying only its id.
    // Taking the first field there would admit a symbol whose name is a number.
    let inventory = TokenInventory::parse("<blk> 0\n7\nT 4\n");
    assert_eq!(inventory.len(), 2);
    assert!(!inventory.contains("7"));
    assert!(inventory.contains("T"));
}

#[test]
fn blank_lines_and_trailing_whitespace_are_tolerated() {
    let inventory = TokenInventory::parse("\n<blk> 0\n\n  T   4  \n\n");
    assert_eq!(inventory.len(), 2);
    assert!(inventory.contains("T"));
}

#[test]
fn an_empty_file_is_an_empty_inventory_rather_than_an_error() {
    // A model whose tokens.txt is empty is broken, but that is discovered as
    // "it can encode nothing", which is a diagnosis, not a parse failure.
    let inventory = TokenInventory::parse("");
    assert!(inventory.is_empty());
    assert_eq!(inventory.unknown_tokens("T @x\n"), vec!["T".to_owned()]);
}

// ── the parser rule ─────────────────────────────────────────────────────────

#[test]
fn a_keyword_file_of_known_tokens_is_accepted() {
    assert_eq!(
        inventory().validate("L AY1 T AH1 P @LIGHT_UP\n").ok(),
        Some(())
    );
    assert!(
        inventory()
            .unknown_tokens("n ǐ h ǎo @greeting\n")
            .is_empty()
    );
}

#[test]
fn every_line_of_a_multi_keyword_file_is_checked() {
    let unknown = inventory().unknown_tokens("L AY1 T @a\nn ǐ ZZZ @b\nT QQQ @c\n");
    assert_eq!(unknown, vec!["ZZZ".to_owned(), "QQQ".to_owned()]);
}

#[rstest]
#[case::boost(":2.0")]
#[case::threshold("#0.35")]
#[case::label("@LIGHT_UP")]
fn a_marker_is_not_a_token(#[case] marker: &str) {
    assert!(KEYWORD_MARKERS.contains(&marker.chars().next().expect("non-empty")));
    let line = format!("L AY1 T {marker}\n");
    assert!(
        inventory().unknown_tokens(&line).is_empty(),
        "`{marker}` was read as a token"
    );
}

#[test]
fn the_inventory_is_consulted_before_the_marker_table() {
    // `EncodeBase` calls `symbol_table.Contains(word)` first and only looks at
    // `word[0]` for a word it did not find. A model whose inventory contained a
    // marker-shaped symbol encodes it as a symbol — it does not try to read
    // `hash` as a threshold. Checking the marker table first would reject a
    // keyword file the engine accepts, which turns a working wake word off.
    let odd = TokenInventory::parse("<blk> 0\n#hash 1\n");
    assert!(odd.contains("#hash"));
    assert!(odd.unknown_tokens("#hash @x\n").is_empty());
    assert!(
        odd.malformed_markers("#hash @x\n").is_empty(),
        "`#hash` is a symbol here, not a threshold with a broken payload"
    );
    assert!(odd.validate("#hash @x\n").is_ok());

    // The same word against a model that does *not* know it is a marker, and
    // `hash` is not a number.
    assert_eq!(
        inventory().malformed_markers("#hash @x\n"),
        vec!["#hash".to_owned()]
    );
}

// ── the second way a keyword file ends the process ──────────────────────────

#[rstest]
#[case::boost_word(":later")]
#[case::threshold_word("#soon")]
#[case::empty_boost(":")]
#[case::sign_only(":-")]
#[case::dot_only(":.")]
#[case::infinity(":inf")]
fn a_marker_with_no_number_is_a_fault(#[case] word: &str) {
    // `EncodeBase` reaches `std::stof(word.substr(1))` with no `try`, so this
    // is an uncaught `std::invalid_argument`, not a rejected score.
    let line = format!("L AY1 T @x {word}\n");
    assert_eq!(inventory().malformed_markers(&line), vec![word.to_owned()]);

    let error = inventory()
        .validate(&line)
        .expect_err("the marker is malformed");
    match &error {
        WakeWordError::MalformedMarker { malformed } => {
            assert_eq!(*malformed, vec![word.to_owned()]);
        }
        other => panic!("expected a malformed marker, got {other:?}"),
    }
    assert!(error.to_string().contains(word));
}

#[rstest]
#[case::plain(":2")]
#[case::decimal(":2.0")]
#[case::leading_dot(":.5")]
#[case::signed(":-1.5")]
#[case::explicit_plus(":+1.5")]
#[case::threshold("#0.35")]
// `std::stof` converts a valid prefix and ignores the rest, so the library
// reads this as 2.0 and so must the guard: rejecting it would turn off a wake
// word the engine would have run.
#[case::trailing_garbage(":2.0dB")]
fn a_marker_with_a_number_is_not_a_fault(#[case] word: &str) {
    let line = format!("L AY1 T @x {word}\n");
    assert!(
        inventory().malformed_markers(&line).is_empty(),
        "`{word}` was rejected"
    );
    assert!(inventory().validate(&line).is_ok());
}

#[test]
fn an_at_marker_carries_any_string_at_all() {
    // Only `:` and `#` are numbers. `@` is the display label.
    assert!(
        inventory()
            .malformed_markers("L AY1 @anything-at-all\n")
            .is_empty()
    );
    assert!(inventory().malformed_markers("L AY1 @\n").is_empty());
}

#[test]
fn unknown_tokens_are_reported_before_malformed_markers() {
    // A file with both is more usefully described by the fault an operator will
    // recognise, and an unknown token is far and away the common one.
    let error = inventory()
        .validate("L ZZZ @x :later\n")
        .expect_err("both faults are present");
    assert!(
        matches!(error, WakeWordError::UnknownTokens { .. }),
        "expected unknown tokens first, got {error:?}"
    );
}

#[test]
fn an_unknown_token_is_reported_once_in_the_order_it_appeared() {
    let unknown = inventory().unknown_tokens("ZZZ T ZZZ QQQ ZZZ @x\n");
    assert_eq!(unknown, vec!["ZZZ".to_owned(), "QQQ".to_owned()]);
}

#[test]
fn validate_names_every_unknown_token_in_its_message() {
    let error = inventory()
        .validate("L AY1 ZZZ QQQ @x\n")
        .expect_err("two tokens are unknown");
    match &error {
        WakeWordError::UnknownTokens { unknown } => {
            assert_eq!(*unknown, vec!["ZZZ".to_owned(), "QQQ".to_owned()]);
        }
        other => panic!("expected unknown tokens, got {other:?}"),
    }
    let rendered = error.to_string();
    assert!(rendered.contains("ZZZ"));
    assert!(rendered.contains("QQQ"));
}

#[test]
fn an_empty_keyword_file_has_no_unknown_tokens() {
    // Emptiness is refused elsewhere — by `ModelManager::ensure` and
    // `SherpaDetector::open` — because it is a different fault with a different
    // name. This function's answer is only about encodability.
    assert!(inventory().unknown_tokens("").is_empty());
    assert!(inventory().unknown_tokens("   \n\n").is_empty());
}

#[test]
fn the_model_this_crate_catalogues_uses_two_alphabets() {
    // Both halves of a zh-en keyword model, in one file: ARPAbet with stress
    // digits for English, tone-marked pinyin for Chinese. It is why a token
    // line cannot be derived from a phrase without the model.
    let inventory = inventory();
    assert!(inventory.validate("L AY1 T AH1 P @LIGHT_UP\n").is_ok());
    assert!(inventory.validate("n ǐ h ǎo @greeting\n").is_ok());
    // And why a plausible-looking guess is not a token line.
    assert!(inventory.validate("l i g h t u p @LIGHT_UP\n").is_err());
}
