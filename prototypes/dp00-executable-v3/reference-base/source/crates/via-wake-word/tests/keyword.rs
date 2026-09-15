//! The phrase, the token line, and the per-locale table of both.
//!
//! The phrases below are fixtures. VIA has not chosen a wake phrase
//! (`docs/architecture.md` §16), and the upstream product's own phrase is
//! parsed out of `docs/reference/contracts.json` in `tests/contracts.rs` rather
//! than written anywhere in this tree.

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_i18n::Locale;
use via_wake_word::{
    KEYWORD_DISPLAY_SEPARATOR, Keyword, KeywordError, KeywordSet, LABEL_WORD_SEPARATOR,
    MAX_KEYWORD_MODELS, ModelArtifact, WakePhrase,
};

fn phrase(text: &str, tokens: &str) -> WakePhrase {
    WakePhrase::new(text, tokens).expect("a valid fixture phrase")
}

fn keyword(text: &str, tokens: &str) -> Keyword {
    Keyword::zh_en(text, tokens).expect("a valid fixture keyword")
}

// ── the two columns ─────────────────────────────────────────────────────────

#[test]
fn a_phrase_keeps_three_parts_and_renders_one_line() {
    // The token line is ARPAbet with stress digits, which is what the
    // catalogued zh-en model spells English in.
    let phrase = phrase("wake up", "W EY1 K AH1 P");
    assert_eq!(phrase.text(), "wake up", "the human phrase keeps its space");
    assert_eq!(phrase.tokens(), "W EY1 K AH1 P");
    assert_eq!(phrase.label(), "wake_up", "the label cannot contain one");
    assert_eq!(phrase.keyword_line(), "W EY1 K AH1 P @wake_up\n");
    assert!(phrase.keyword_line().contains(KEYWORD_DISPLAY_SEPARATOR));
    assert_eq!(KEYWORD_DISPLAY_SEPARATOR, " @");
    assert_eq!(LABEL_WORD_SEPARATOR, '_');
}

#[test]
fn a_single_word_phrase_is_its_own_label() {
    let phrase = phrase("你好维亚", "n ǐ h ǎo w éi y à");
    assert_eq!(phrase.label(), phrase.text());
    assert_eq!(phrase.keyword_line(), "n ǐ h ǎo w éi y à @你好维亚\n");
}

#[test]
fn a_run_of_whitespace_becomes_one_separator() {
    // A label with an empty word in it would be `wake__up`, which is a
    // different string from the one the engine reports.
    let phrase = phrase("wake  \t up", "W EY1 K AH1 P");
    assert_eq!(phrase.label(), "wake_up");
    assert_eq!(phrase.keyword_line().lines().count(), 1);
}

#[test]
fn both_columns_are_trimmed() {
    let phrase = phrase("  wake up \t", "\n  W EY1 K  ");
    assert_eq!(phrase.text(), "wake up");
    assert_eq!(phrase.tokens(), "W EY1 K");
    assert_eq!(phrase.keyword_line(), "W EY1 K @wake_up\n");
}

#[rstest]
#[case::empty_text("", "t o k e n s", "phrase")]
#[case::blank_text("   ", "t o k e n s", "phrase")]
#[case::tab_text("\t", "t o k e n s", "phrase")]
#[case::empty_tokens("wake up", "", "tokens")]
#[case::blank_tokens("wake up", " \t ", "tokens")]
fn an_empty_column_is_refused(
    #[case] text: &str,
    #[case] tokens: &str,
    #[case] field: &'static str,
) {
    assert_eq!(
        WakePhrase::new(text, tokens),
        Err(KeywordError::Empty { field })
    );
}

#[rstest]
#[case::newline_in_text("wake\nup", "t o k", "phrase")]
#[case::carriage_return_in_text("wake\rup", "t o k", "phrase")]
#[case::newline_in_tokens("wake up", "t o\nk", "tokens")]
#[case::carriage_return_in_tokens("wake up", "t o\rk", "tokens")]
fn a_line_break_is_refused_because_it_would_register_a_second_keyword(
    #[case] text: &str,
    #[case] tokens: &str,
    #[case] field: &'static str,
) {
    assert_eq!(
        WakePhrase::new(text, tokens),
        Err(KeywordError::LineBreak { field })
    );
}

#[rstest]
#[case::separator_in_text("wake@up", "t o k", "phrase")]
#[case::separator_in_tokens("wake up", "t @ o k", "tokens")]
fn the_column_separator_is_refused_inside_a_column(
    #[case] text: &str,
    #[case] tokens: &str,
    #[case] field: &'static str,
) {
    assert_eq!(
        WakePhrase::new(text, tokens),
        Err(KeywordError::Separator { field })
    );
}

#[test]
fn a_rejected_column_says_which_one_it_was() {
    // The diagnostic is the whole value of typing the failure: `phrase` and
    // `tokens` are configured in different places by different people.
    assert!(
        KeywordError::Empty { field: "tokens" }
            .to_string()
            .contains("tokens")
    );
    assert!(
        KeywordError::LineBreak { field: "phrase" }
            .to_string()
            .contains("phrase")
    );
    assert!(
        KeywordError::Separator { field: "phrase" }
            .to_string()
            .contains('@')
    );
}

#[test]
fn a_multi_byte_phrase_round_trips() {
    // The catalogued model is a zh-en keyword model, so a non-ASCII display
    // column is the normal case rather than the exotic one.
    let phrase = phrase("안녕 비아", "a n n y e o n g b i a");
    assert_eq!(phrase.keyword_line(), "a n n y e o n g b i a @안녕_비아\n");
    assert!(phrase.keyword_line().len() > phrase.keyword_line().chars().count());
}

// ── the table ───────────────────────────────────────────────────────────────

#[test]
fn the_table_holds_at_most_one_keyword_per_locale() {
    assert_eq!(MAX_KEYWORD_MODELS, 3);
    assert_eq!(MAX_KEYWORD_MODELS, Locale::ALL.len());

    let mut set = KeywordSet::new();
    assert!(set.is_empty());
    for locale in Locale::ALL {
        set.insert(*locale, keyword(&format!("phrase {locale}"), "t o k"));
    }
    assert_eq!(set.len(), MAX_KEYWORD_MODELS);

    // There is no fourth locale to add, and a repeat replaces rather than
    // grows: the cap is structural, not checked.
    let displaced = set.insert(Locale::En, keyword("another phrase", "t o k"));
    assert_eq!(
        displaced.map(|k| k.text().to_owned()),
        Some("phrase en".to_owned())
    );
    assert_eq!(set.len(), MAX_KEYWORD_MODELS);
    assert_eq!(
        set.get(Locale::En).map(Keyword::text),
        Some("another phrase")
    );
}

#[test]
fn the_table_iterates_in_locale_order_whatever_order_it_was_built_in() {
    let set = KeywordSet::new()
        .with(Locale::Ko, keyword("ko phrase", "k o"))
        .with(Locale::Zh, keyword("zh phrase", "z h"))
        .with(Locale::En, keyword("en phrase", "e n"));

    let order: Vec<Locale> = set.iter().map(|(locale, _)| locale).collect();
    assert_eq!(order, Locale::ALL.to_vec());
}

#[test]
fn a_missing_locale_is_none_not_a_fallback() {
    let set = KeywordSet::new().with(Locale::Zh, keyword("zh phrase", "z h"));
    assert!(set.get(Locale::Zh).is_some());
    assert_eq!(set.get(Locale::En), None);
    assert_eq!(set.get(Locale::Ko), None);
}

#[test]
fn a_table_can_be_collected() {
    let set: KeywordSet = [
        (Locale::En, keyword("en phrase", "e n")),
        (Locale::Ko, keyword("ko phrase", "k o")),
    ]
    .into_iter()
    .collect();
    assert_eq!(set.len(), 2);
    assert_eq!(set.get(Locale::Ko).map(Keyword::text), Some("ko phrase"));
}

// ── the keyword file ────────────────────────────────────────────────────────

#[test]
fn the_keyword_file_has_one_line_per_phrase_in_locale_order() {
    let set = KeywordSet::new()
        .with(Locale::Ko, keyword("ko phrase", "k o"))
        .with(Locale::En, keyword("en phrase", "e n"));

    assert_eq!(
        set.keywords_file(via_wake_word::WAKE_WORD_MODEL_NAME),
        "e n @en_phrase\nk o @ko_phrase\n"
    );
}

#[test]
fn two_locales_sharing_a_phrase_register_it_once() {
    // `en` and `zh` both resolving to the same phrase on the same model is a
    // real configuration — the catalogued artifact is a zh-en model. Writing
    // the line twice would register a duplicate keyword.
    let set = KeywordSet::new()
        .with(Locale::En, keyword("hello via", "h e l l o"))
        .with(Locale::Zh, keyword("hello via", "h e l l o"));

    assert_eq!(
        set.keywords_file(via_wake_word::WAKE_WORD_MODEL_NAME),
        "h e l l o @hello_via\n"
    );
    assert_eq!(set.len(), 2);
}

#[test]
fn the_same_phrase_with_different_tokens_is_two_keywords() {
    // Not a duplicate: two spellings of one phrase in two token inventories
    // are two things for the decoder to match.
    let set = KeywordSet::new()
        .with(Locale::En, keyword("hello via", "h e l l o"))
        .with(Locale::Zh, keyword("hello via", "h ə l ˈoʊ"));

    assert_eq!(
        set.keywords_file(via_wake_word::WAKE_WORD_MODEL_NAME),
        "h e l l o @hello_via\nh ə l ˈoʊ @hello_via\n"
    );
}

#[test]
fn the_keyword_file_carries_only_the_phrases_bound_to_that_artifact() {
    let other = ModelArtifact {
        id: std::borrow::Cow::Borrowed("some-other-kws-model"),
        ..ModelArtifact::ZH_EN_3M
    };
    let set = KeywordSet::new()
        .with(Locale::En, keyword("en phrase", "e n"))
        .with(
            Locale::Ko,
            Keyword::new(phrase("ko phrase", "k o"), other.clone()),
        );

    assert_eq!(
        set.keywords_file(via_wake_word::WAKE_WORD_MODEL_NAME),
        "e n @en_phrase\n"
    );
    assert_eq!(set.keywords_file(&other.id), "k o @ko_phrase\n");
    assert_eq!(set.keywords_file("a-model-nobody-configured"), "");
}

#[test]
fn every_keyword_file_line_parses_back_into_its_two_columns() {
    let set = KeywordSet::new()
        .with(Locale::En, keyword("en phrase", "e n"))
        .with(Locale::Zh, keyword("zh phrase", "z h"))
        .with(Locale::Ko, keyword("ko phrase", "k o"));

    let file = set.keywords_file(via_wake_word::WAKE_WORD_MODEL_NAME);
    let lines: Vec<&str> = file.lines().collect();
    assert_eq!(lines.len(), 3);
    for line in lines {
        let (tokens, text) = line
            .split_once(" @")
            .expect("every line separates its columns");
        assert!(!tokens.is_empty() && !text.is_empty());
        assert!(set.locale_of(text).is_some());
    }
}

// ── the artifacts ───────────────────────────────────────────────────────────

#[test]
fn locales_sharing_an_artifact_need_one_install() {
    let set = KeywordSet::new()
        .with(Locale::En, keyword("en phrase", "e n"))
        .with(Locale::Zh, keyword("zh phrase", "z h"));

    let artifacts = set.artifacts();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].id, ModelArtifact::ZH_EN_3M.id);
}

#[test]
fn three_locales_on_three_models_need_three_installs() {
    let second = ModelArtifact {
        id: std::borrow::Cow::Borrowed("second-kws-model"),
        ..ModelArtifact::ZH_EN_3M
    };
    let third = ModelArtifact {
        id: std::borrow::Cow::Borrowed("third-kws-model"),
        ..ModelArtifact::ZH_EN_3M
    };
    let set = KeywordSet::new()
        .with(Locale::En, keyword("en phrase", "e n"))
        .with(Locale::Zh, Keyword::new(phrase("zh phrase", "z h"), second))
        .with(Locale::Ko, Keyword::new(phrase("ko phrase", "k o"), third));

    let ids: Vec<&str> = set.artifacts().iter().map(|a| a.id.as_ref()).collect();
    assert_eq!(
        ids,
        vec![
            ModelArtifact::ZH_EN_3M.id.as_ref(),
            "second-kws-model",
            "third-kws-model",
        ]
    );
    assert!(set.artifacts().len() <= MAX_KEYWORD_MODELS);
}

#[test]
fn an_empty_table_needs_no_installs() {
    assert!(KeywordSet::new().artifacts().is_empty());
    assert_eq!(
        KeywordSet::new().keywords_file(via_wake_word::WAKE_WORD_MODEL_NAME),
        ""
    );
}

// ── attribution ─────────────────────────────────────────────────────────────

#[test]
fn a_detected_display_text_is_attributed_to_its_locale() {
    let set = KeywordSet::new()
        .with(Locale::Zh, keyword("zh phrase", "z h"))
        .with(Locale::Ko, keyword("ko phrase", "k o"));

    // The engine reports the label, so that is what is matched.
    assert_eq!(set.locale_of("zh_phrase"), Some(Locale::Zh));
    assert_eq!(set.locale_of("ko_phrase"), Some(Locale::Ko));
    assert_eq!(set.locale_of("something else"), None);
    // The human phrase is *not* what comes back from the engine.
    assert_eq!(set.locale_of("zh phrase"), None);
    // Matching is exact: the engine reports the label verbatim.
    assert_eq!(set.locale_of("zh_phrase "), None);
    assert_eq!(set.locale_of("ZH_PHRASE"), None);
}

#[test]
fn a_shared_phrase_is_attributed_to_the_first_locale_in_order() {
    let set = KeywordSet::new()
        .with(Locale::Zh, keyword("shared", "s h"))
        .with(Locale::Ko, keyword("shared", "s h"));
    // The same tie-break `keywords_file` uses when it collapses the duplicate.
    assert_eq!(set.locale_of("shared"), Some(Locale::Zh));

    let with_en = set.with(Locale::En, keyword("shared", "s h"));
    assert_eq!(with_en.locale_of("shared"), Some(Locale::En));
}

// ── the markers ─────────────────────────────────────────────────────────────

#[rstest]
#[case::boost_first(":2.0 W EY1 K")]
#[case::boost_middle("W :2.0 EY1 K")]
#[case::threshold("W EY1 #0.35 K")]
#[case::boost_not_a_number(":not-a-number W EY1")]
fn a_token_word_may_not_begin_with_a_keyword_file_marker(#[case] tokens: &str) {
    // `sherpa-onnx` reads `:x` as a per-keyword boost and `#x` as a threshold,
    // then `std::stof`s the rest — inside a C++ library that ends the process
    // when keyword encoding fails.
    let marker = tokens
        .split_whitespace()
        .find_map(|word| word.chars().next().filter(|c| *c == ':' || *c == '#'))
        .expect("the case carries a marker");
    assert_eq!(
        WakePhrase::new("wake up", tokens),
        Err(KeywordError::Marker {
            field: "tokens",
            marker
        })
    );
}

#[test]
fn a_marker_inside_a_word_is_not_a_marker() {
    // Only `word[0]` is examined, so a token containing `:` elsewhere is a
    // token — an odd one, but the parser will look it up rather than parse it.
    assert!(WakePhrase::new("wake up", "W E:Y1 K").is_ok());
}

#[test]
fn a_human_phrase_may_begin_with_a_marker_character() {
    // The label is emitted as `@<label>`, so `word[0]` is always `@` and the
    // marker table never sees the label's own first character.
    let phrase = phrase(":30 seconds", "W EY1 K");
    assert_eq!(phrase.label(), ":30_seconds");
    assert_eq!(phrase.keyword_line(), "W EY1 K @:30_seconds\n");
}

#[test]
fn a_rejected_marker_says_which_character_it_was() {
    assert!(
        KeywordError::Marker {
            field: "tokens",
            marker: '#'
        }
        .to_string()
        .contains('#')
    );
}
