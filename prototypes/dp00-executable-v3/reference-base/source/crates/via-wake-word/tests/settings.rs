//! Resolution: is this Gateway listening, for what, and what does it say?
//!
//! The property under test throughout is that **nothing here panics and
//! nothing here guesses**. Every way a wake word can fail to exist produces a
//! named [`DisabledReason`], and the only string ever handed to a user comes
//! from `via-i18n` with the configured phrase in it.

use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_core::Config;
use via_i18n::Locale;
use via_wake_word::{
    DisabledReason, Keyword, KeywordSet, ModelArtifact, Resolution, WakePhrase, WakeWordSettings,
};

fn keyword(text: &str) -> Keyword {
    Keyword::zh_en(text, "t o k e n s").expect("a valid fixture keyword")
}

fn config(enabled: bool, phrase: &str, locale: Locale) -> Config {
    Config {
        wake_word_enabled: enabled,
        wake_word: phrase.to_owned(),
        wake_word_model_directory: PathBuf::from("/var/lib/via/models/wake-word"),
        locale,
        ..Config::default()
    }
}

// ── the resolution matrix ───────────────────────────────────────────────────

#[test]
fn a_switched_off_wake_word_resolves_disabled_in_every_locale() {
    let settings = WakeWordSettings::new(
        false,
        "/models",
        KeywordSet::new().with(Locale::En, keyword("wake up")),
    );
    for locale in Locale::ALL {
        assert_eq!(
            settings.resolve(*locale),
            Resolution::Disabled(DisabledReason::NotEnabled)
        );
        assert_eq!(settings.phrase(*locale), None);
        assert_eq!(settings.asleep_message(*locale), None);
    }
    assert!(!settings.is_enabled());
    assert!(
        settings.artifacts().is_empty(),
        "a disabled wake word downloads nothing"
    );
}

#[test]
fn an_empty_table_resolves_no_phrase_configured() {
    let settings = WakeWordSettings::new(true, "/models", KeywordSet::new());
    for locale in Locale::ALL {
        assert_eq!(
            settings.resolve(*locale),
            Resolution::Disabled(DisabledReason::NoPhraseConfigured)
        );
    }
    assert!(
        settings.is_enabled(),
        "the switch is on; there is just nothing to hear"
    );
}

#[test]
fn a_locale_without_a_keyword_model_says_so_by_name() {
    let settings = WakeWordSettings::new(
        true,
        "/models",
        KeywordSet::new().with(Locale::Zh, keyword("zh phrase")),
    );
    assert_eq!(
        settings.resolve(Locale::Zh),
        Resolution::Enabled(&keyword("zh phrase"))
    );
    for locale in [Locale::En, Locale::Ko] {
        assert_eq!(
            settings.resolve(locale),
            Resolution::Disabled(DisabledReason::NoKeywordModel { locale })
        );
        assert!(
            settings
                .resolve(locale)
                .disabled_reason()
                .expect("a reason")
                .to_string()
                .contains(locale.as_str()),
            "the diagnostic names the locale that has no model"
        );
    }
}

#[test]
fn an_enabled_locale_resolves_to_its_keyword() {
    let settings = WakeWordSettings::new(
        true,
        "/models",
        KeywordSet::new().with(Locale::Ko, keyword("깨어나")),
    );
    let resolution = settings.resolve(Locale::Ko);
    assert!(resolution.is_enabled());
    assert_eq!(resolution.keyword().map(Keyword::text), Some("깨어나"));
    assert_eq!(resolution.disabled_reason(), None);
    assert_eq!(settings.phrase(Locale::Ko), Some("깨어나"));
}

#[test]
fn the_install_root_comes_from_the_settings_not_from_this_crate() {
    let settings = WakeWordSettings::new(true, "/somewhere/else", KeywordSet::new());
    assert_eq!(settings.model_root(), Path::new("/somewhere/else"));
}

// ── reading a Config ────────────────────────────────────────────────────────

#[rstest]
#[case::off_and_unconfigured(false, "", DisabledReason::NotEnabled)]
#[case::off_but_configured(false, "wake up", DisabledReason::NotEnabled)]
#[case::on_but_unconfigured(true, "", DisabledReason::NoPhraseConfigured)]
#[case::on_but_blank(true, "   ", DisabledReason::NoPhraseConfigured)]
fn a_config_that_names_no_phrase_is_cleanly_disabled(
    #[case] enabled: bool,
    #[case] phrase: &str,
    #[case] expected: DisabledReason,
) {
    // Note the table is *not* empty: an unconfigured `VIA_WAKE_WORD` disables
    // the wake word whatever keyword models are available, because the phrase
    // is the configuration.
    let keywords = KeywordSet::new().with(Locale::En, keyword("wake up"));
    let settings = WakeWordSettings::from_config(&config(enabled, phrase, Locale::En), keywords);
    assert_eq!(settings.resolve(Locale::En), Resolution::Disabled(expected));
    assert_eq!(settings.asleep_message(Locale::En), None);
}

#[test]
fn a_config_supplies_the_switch_and_the_install_root() {
    let settings = WakeWordSettings::from_config(
        &config(true, "wake up", Locale::En),
        KeywordSet::new().with(Locale::En, keyword("wake up")),
    );
    assert!(settings.is_enabled());
    assert_eq!(
        settings.model_root(),
        Path::new("/var/lib/via/models/wake-word")
    );
    assert_eq!(settings.phrase(Locale::En), Some("wake up"));
}

#[test]
fn a_config_phrase_that_disagrees_with_the_keyword_model_disables_rather_than_lying() {
    // The failure this catches: the Gateway tells the user to say one thing
    // while the decoder listens for another. Nothing errors, nothing logs, and
    // the wake word simply never fires.
    let settings = WakeWordSettings::from_config(
        &config(true, "hey via", Locale::En),
        KeywordSet::new().with(Locale::En, keyword("wake up")),
    );

    assert_eq!(
        settings.resolve(Locale::En),
        Resolution::Disabled(DisabledReason::PhraseMismatch {
            locale: Locale::En,
            announced: "hey via".to_owned(),
            model: "wake up".to_owned(),
        })
    );
    let reason = settings.resolve(Locale::En);
    let diagnostic = reason.disabled_reason().expect("a reason").to_string();
    assert!(diagnostic.contains("hey via"));
    assert!(diagnostic.contains("wake up"));
    assert_eq!(settings.asleep_message(Locale::En), None);
}

#[test]
fn the_cross_check_applies_only_to_the_locale_the_config_named() {
    // `Config` carries one phrase, for its own locale. Another locale's
    // keyword is configured elsewhere and has nothing to disagree with.
    let settings = WakeWordSettings::from_config(
        &config(true, "hey via", Locale::En),
        KeywordSet::new()
            .with(Locale::En, keyword("hey via"))
            .with(Locale::Zh, keyword("你好")),
    );
    assert_eq!(settings.phrase(Locale::En), Some("hey via"));
    assert_eq!(settings.phrase(Locale::Zh), Some("你好"));
}

#[test]
fn a_config_phrase_is_trimmed_before_it_is_compared() {
    let settings = WakeWordSettings::from_config(
        &config(true, "  wake up  ", Locale::En),
        KeywordSet::new().with(Locale::En, keyword("wake up")),
    );
    assert_eq!(settings.phrase(Locale::En), Some("wake up"));
}

// ── the sentence a person hears ─────────────────────────────────────────────

#[test]
fn the_asleep_message_names_whatever_phrase_is_configured() {
    for phrase in ["wake up", "hey via", "안녕 비아", "salut via"] {
        let settings = WakeWordSettings::new(
            true,
            "/models",
            KeywordSet::new()
                .with(Locale::En, keyword(phrase))
                .with(Locale::Zh, keyword(phrase))
                .with(Locale::Ko, keyword(phrase)),
        );
        for locale in Locale::ALL {
            let message = settings
                .asleep_message(*locale)
                .expect("an enabled locale renders the sentence");
            assert!(
                message.contains(phrase),
                "the {locale} sleep message does not name `{phrase}`: {message}"
            );
            assert!(
                !message.contains("{wake_word}"),
                "the placeholder survived into {locale}: {message}"
            );
            assert!(
                !message.contains("via-i18n:"),
                "the render failed in {locale}: {message}"
            );
            assert_eq!(
                message,
                via_i18n::format(
                    *locale,
                    via_i18n::keys::REALTIME_ASLEEP,
                    &[("wake_word", phrase)]
                )
            );
        }
    }
}

#[test]
fn each_locale_hears_its_own_phrase() {
    let settings = WakeWordSettings::new(
        true,
        "/models",
        KeywordSet::new()
            .with(Locale::En, keyword("hey via"))
            .with(Locale::Zh, keyword("你好维亚"))
            .with(Locale::Ko, keyword("안녕 비아")),
    );

    assert!(
        settings
            .asleep_message(Locale::En)
            .expect("en")
            .contains("hey via")
    );
    assert!(
        settings
            .asleep_message(Locale::Zh)
            .expect("zh")
            .contains("你好维亚")
    );
    assert!(
        settings
            .asleep_message(Locale::Ko)
            .expect("ko")
            .contains("안녕 비아")
    );

    // And no locale leaks another's.
    assert!(
        !settings
            .asleep_message(Locale::En)
            .expect("en")
            .contains("안녕 비아")
    );
}

#[test]
fn the_sleep_message_is_the_catalogued_sentence_with_the_phrase_substituted() {
    // The key carries `{wake_word}` where upstream hard-coded its own phrase.
    // Asserting the surrounding prose is unchanged is what keeps the
    // substitution a substitution rather than a rewrite.
    let settings = WakeWordSettings::new(
        true,
        "/models",
        KeywordSet::new().with(Locale::Zh, keyword("PHRASE")),
    );
    let rendered = settings.asleep_message(Locale::Zh).expect("zh");
    let template = via_i18n::t(Locale::Zh, via_i18n::keys::REALTIME_ASLEEP);
    assert_eq!(rendered, template.replace("{wake_word}", "PHRASE"));
    assert!(template.contains("{wake_word}"));
}

// ── what has to be downloaded ───────────────────────────────────────────────

#[test]
fn the_artifacts_to_install_follow_the_locales_that_actually_resolve() {
    let second = ModelArtifact {
        id: std::borrow::Cow::Borrowed("second-kws-model"),
        ..ModelArtifact::ZH_EN_3M
    };
    let settings = WakeWordSettings::new(
        true,
        "/models",
        KeywordSet::new()
            .with(Locale::En, keyword("hey via"))
            .with(Locale::Zh, keyword("你好维亚"))
            .with(
                Locale::Ko,
                Keyword::new(
                    WakePhrase::new("안녕 비아", "a n n y e o n g").expect("valid"),
                    second,
                ),
            ),
    );

    let ids: Vec<&str> = settings.artifacts().iter().map(|a| a.id.as_ref()).collect();
    assert_eq!(
        ids,
        vec![ModelArtifact::ZH_EN_3M.id.as_ref(), "second-kws-model"],
        "en and zh share the zh-en model, ko needs its own"
    );
    assert!(settings.artifacts().len() <= via_wake_word::MAX_KEYWORD_MODELS);
}

#[test]
fn a_locale_disabled_by_a_phrase_mismatch_downloads_nothing_for_itself() {
    let settings = WakeWordSettings::from_config(
        &config(true, "hey via", Locale::En),
        KeywordSet::new().with(Locale::En, keyword("wake up")),
    );
    assert!(
        settings.artifacts().is_empty(),
        "the only configured locale does not resolve, so there is nothing to install"
    );
}

// ── the reasons read as sentences ───────────────────────────────────────────

#[rstest]
#[case(DisabledReason::NotEnabled, "not enabled")]
#[case(DisabledReason::NoPhraseConfigured, "no wake phrase")]
#[case(DisabledReason::NoKeywordModel { locale: Locale::Ko }, "ko")]
fn a_disabled_reason_reads_as_a_diagnostic(#[case] reason: DisabledReason, #[case] fragment: &str) {
    let rendered = reason.to_string();
    assert!(
        rendered.contains(fragment),
        "`{rendered}` does not mention `{fragment}`"
    );
    assert!(rendered.chars().next().is_some_and(char::is_lowercase) || rendered.starts_with("the"));
}

#[test]
fn default_settings_are_disabled() {
    let settings = WakeWordSettings::default();
    assert!(!settings.is_enabled());
    assert_eq!(
        settings.resolve(Locale::En),
        Resolution::Disabled(DisabledReason::NotEnabled)
    );
    assert!(settings.keywords().is_empty());
}
