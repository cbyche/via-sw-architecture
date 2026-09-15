//! Whether there is a wake word at all, and which one.
//!
//! Every question this module answers has the same shape: *given a locale, is
//! this Gateway listening for a phrase, and if so which?* The answer is
//! [`Resolution`] — an enum with a reason attached to the negative case, never
//! a panic and never an empty string standing in for "no".
//!
//! # Why "disabled" is a first-class answer
//!
//! Four different configurations produce no wake word, and a caller has to be
//! able to tell them apart:
//!
//! | Configuration | [`DisabledReason`] |
//! | --- | --- |
//! | `VIA_WAKE_WORD_ENABLED` is not `true` | [`NotEnabled`](DisabledReason::NotEnabled) |
//! | enabled, but no phrase configured | [`NoPhraseConfigured`](DisabledReason::NoPhraseConfigured) |
//! | enabled with a phrase, no keyword model for this locale | [`NoKeywordModel`](DisabledReason::NoKeywordModel) |
//! | the configured phrase is not the one the keyword model matches | [`PhraseMismatch`](DisabledReason::PhraseMismatch) |
//!
//! The last one is the reason this type exists rather than an `Option<&str>`.
//! `VIA_WAKE_WORD` is what the sleep message says out loud, and the keyword
//! model is what actually opens the microphone's ears. If they disagree, the
//! Gateway tells the user to say one phrase while listening for another, and
//! nothing anywhere fails — it just never wakes. Reporting the disagreement as
//! *disabled, because the two disagree* turns a silent product failure into a
//! log line.

use std::path::{Path, PathBuf};

use via_core::Config;
use via_i18n::{Locale, format as i18n_format, keys};

use crate::artifact::ModelArtifact;
use crate::keyword::{Keyword, KeywordSet};

/// Why there is no wake word to listen for.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DisabledReason {
    /// Wake-word detection is switched off.
    ///
    /// **External contract** — `VIA_WAKE_WORD_ENABLED`, upstream
    /// `QWEN_AUDIO_WAKE_WORD_ENABLED`, default off
    /// (`server/src/core/config.mjs:500-502`).
    NotEnabled,
    /// Detection is on, but no phrase has been configured.
    ///
    /// This is the state `docs/architecture.md` §16 leaves VIA in until a
    /// phrase is chosen. It is not an error.
    NoPhraseConfigured,
    /// A phrase is configured, but this locale has no keyword model.
    NoKeywordModel {
        /// The locale that was asked for.
        locale: Locale,
    },
    /// The announced phrase and the keyword model's phrase are different
    /// strings.
    PhraseMismatch {
        /// The locale whose configuration disagrees with itself.
        locale: Locale,
        /// What `VIA_WAKE_WORD` says the user should say.
        announced: String,
        /// What the keyword model is actually listening for.
        model: String,
    },
}

impl core::fmt::Display for DisabledReason {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotEnabled => f.write_str("wake-word detection is not enabled"),
            Self::NoPhraseConfigured => f.write_str("no wake phrase is configured"),
            Self::NoKeywordModel { locale } => {
                write!(f, "no wake-word keyword model is configured for {locale}")
            }
            Self::PhraseMismatch {
                locale,
                announced,
                model,
            } => write!(
                f,
                "the wake phrase configured for {locale} (`{announced}`) is not the phrase its \
                 keyword model matches (`{model}`)"
            ),
        }
    }
}

/// The answer to "is this Gateway listening, and for what?".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution<'a> {
    /// Listening, for this keyword.
    Enabled(&'a Keyword),
    /// Not listening, for this reason.
    Disabled(DisabledReason),
}

impl<'a> Resolution<'a> {
    /// The keyword, when there is one.
    #[must_use]
    pub fn keyword(&self) -> Option<&'a Keyword> {
        match self {
            Self::Enabled(keyword) => Some(keyword),
            Self::Disabled(_) => None,
        }
    }

    /// The reason, when there is not.
    #[must_use]
    pub fn disabled_reason(&self) -> Option<&DisabledReason> {
        match self {
            Self::Enabled(_) => None,
            Self::Disabled(reason) => Some(reason),
        }
    }

    /// Whether a wake word is live for this locale.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled(_))
    }
}

/// What, if anything, `Config` said the wake phrase is.
///
/// Three states rather than an `Option`, because "this settings value was not
/// built from a `Config` at all" and "it was, and the phrase was empty" are
/// different configurations with different answers. Collapsing them would make
/// [`WakeWordSettings::new`] behave as though `VIA_WAKE_WORD` were unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Announced {
    /// Not built from a `Config`; the keyword table is the whole
    /// configuration.
    #[default]
    Untracked,
    /// Built from a `Config` whose `VIA_WAKE_WORD` was empty or blank.
    Unconfigured,
    /// Built from a `Config` that named this phrase, for this locale.
    Configured(Locale, String),
}

/// The whole wake-word configuration: the switch, the install root and the
/// per-locale keyword table.
#[derive(Clone, Debug, Default)]
pub struct WakeWordSettings {
    enabled: bool,
    model_root: PathBuf,
    keywords: KeywordSet,
    announced: Announced,
}

impl WakeWordSettings {
    /// Build settings directly, for a caller that is not reading a `Config`.
    #[must_use]
    pub fn new(enabled: bool, model_root: impl Into<PathBuf>, keywords: KeywordSet) -> Self {
        Self {
            enabled,
            model_root: model_root.into(),
            keywords,
            announced: Announced::Untracked,
        }
    }

    /// Take the switch, the install root and the announced phrase from
    /// `Config`, and the keyword table from the caller.
    ///
    /// `Config` carries a phrase *text* (`VIA_WAKE_WORD`) and no token line,
    /// because a token line is a property of the model's own token inventory
    /// and `via-core` has no business knowing one. The caller supplies the
    /// table; this constructor supplies the cross-check that the two agree.
    ///
    /// A blank `config.wake_word` leaves the settings in
    /// [`DisabledReason::NoPhraseConfigured`] whatever the table holds — the
    /// phrase is the configuration, and an unconfigured phrase is a clean
    /// *disabled*.
    #[must_use]
    pub fn from_config(config: &Config, keywords: KeywordSet) -> Self {
        let phrase = config.wake_word.trim();
        let announced = if phrase.is_empty() {
            Announced::Unconfigured
        } else {
            Announced::Configured(config.locale, phrase.to_owned())
        };
        Self {
            enabled: config.wake_word_enabled,
            model_root: config.wake_word_model_directory.clone(),
            keywords,
            announced,
        }
    }

    /// Whether the switch is on. Says nothing about whether a phrase resolves.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Where keyword models install, one directory per artifact id.
    ///
    /// **External contract** — `<config>/models/wake-word`, upstream
    /// `QWEN_AUDIO_WAKE_WORD_MODEL_DIR` (`config.mjs:504-506`), which
    /// `via_core::InstallPaths::wake_word_model_directory` already resolves.
    #[must_use]
    pub fn model_root(&self) -> &Path {
        &self.model_root
    }

    /// The keyword table.
    #[must_use]
    pub fn keywords(&self) -> &KeywordSet {
        &self.keywords
    }

    /// Is this Gateway listening in `locale`, and for what?
    #[must_use]
    pub fn resolve(&self, locale: Locale) -> Resolution<'_> {
        if !self.enabled {
            return Resolution::Disabled(DisabledReason::NotEnabled);
        }
        if self.announced == Announced::Unconfigured || self.keywords.is_empty() {
            return Resolution::Disabled(DisabledReason::NoPhraseConfigured);
        }
        let Some(keyword) = self.keywords.get(locale) else {
            return Resolution::Disabled(DisabledReason::NoKeywordModel { locale });
        };
        if let Announced::Configured(announced_locale, announced) = &self.announced
            && *announced_locale == locale
            && announced != keyword.text()
        {
            return Resolution::Disabled(DisabledReason::PhraseMismatch {
                locale,
                announced: announced.clone(),
                model: keyword.text().to_owned(),
            });
        }
        Resolution::Enabled(keyword)
    }

    /// The phrase this Gateway would tell a `locale` user to say.
    #[must_use]
    pub fn phrase(&self, locale: Locale) -> Option<&str> {
        match self.resolve(locale) {
            Resolution::Enabled(keyword) => Some(keyword.text()),
            Resolution::Disabled(_) => None,
        }
    }

    /// The "I am asleep, say the phrase" sentence, in `locale`.
    ///
    /// **External contract** — `realtime.asleep`, upstream
    /// `'已休眠，请先说“${config.wakeWord}”唤醒。'`
    /// (`realtime-gateway.mjs:2046-2048`). Upstream interpolates its own
    /// hard-coded phrase; the VIA key takes a `{wake_word}` placeholder instead
    /// (recorded on the key in `via-i18n`'s crate documentation), and this is
    /// the call site that fills it — with the phrase that locale actually
    /// resolves to, so the sentence can never name a phrase that would not wake
    /// anything.
    ///
    /// `None` when [`Self::resolve`] is disabled: there is no honest sentence
    /// to render when there is no phrase.
    #[must_use]
    pub fn asleep_message(&self, locale: Locale) -> Option<String> {
        let phrase = self.phrase(locale)?;
        Some(i18n_format(
            locale,
            keys::REALTIME_ASLEEP,
            &[("wake_word", phrase)],
        ))
    }

    /// Every artifact that has to be installed for the configured locales.
    ///
    /// Deduplicated, so `en` and `zh` sharing the catalogued zh-en model is one
    /// download. At most [`crate::MAX_KEYWORD_MODELS`] entries, and empty when
    /// nothing resolves — a disabled wake word downloads nothing.
    #[must_use]
    pub fn artifacts(&self) -> Vec<&ModelArtifact> {
        Locale::ALL
            .iter()
            .filter_map(|locale| self.resolve(*locale).keyword())
            .fold(Vec::new(), |mut seen, keyword| {
                if !seen
                    .iter()
                    .any(|known: &&ModelArtifact| known.id == keyword.artifact().id)
                {
                    seen.push(keyword.artifact());
                }
                seen
            })
    }
}
