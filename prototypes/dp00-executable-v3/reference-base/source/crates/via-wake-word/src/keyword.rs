//! The wake phrase, its token line, and the per-locale table of both.
//!
//! # The phrase is configuration, never a literal
//!
//! `docs/architecture.md` §16 is explicit: *"The phrase is a configuration
//! value, never a literal; the keyword model is a downloadable checksummed
//! artifact resolved per phrase; and wake word is per-locale, so three locales
//! may mean three models."*
//!
//! Upstream hard-codes one phrase and one token line at
//! `server/src/core/config.mjs:503` and
//! `server/src/voice/wake-word/model-manager.mjs:95`. That phrase belongs to
//! the upstream product, `docs/rebrand.md` rows 147-149 mark it RENAME, and
//! **VIA has not chosen its own yet**. So there is no default in this module —
//! not a constant, not a `Default` impl, not a fallback. A deployment that
//! configures nothing gets
//! [`DisabledReason::NoPhraseConfigured`](crate::DisabledReason::NoPhraseConfigured),
//! which is a state the caller handles, not a panic and not a phrase nobody
//! picked.
//!
//! # A keyword file line has three parts, not two
//!
//! ```text
//! n ǐ h ǎo q iān w èn @<label>
//! └──────── tokens ──────┘ └──┘
//! ```
//!
//! **Tokens.** The phrase spelled in the *model's own modelling unit*. For the
//! catalogued zh-en model that is `cjkchar`: tone-marked pinyin syllables for
//! Chinese, ARPAbet phonemes with stress digits for English (`L AY1 T AH1 P`).
//! It is configuration, and it has to be, because the spelling is a property of
//! the model's token inventory — a phrase cannot be tokenized without the model
//! that will match it. [`crate::tokens`] checks it against that inventory
//! before the engine ever sees it.
//!
//! **Label.** What the engine reports on a match. `sherpa-onnx` splits the
//! whole line on whitespace and takes `@`-prefixed *word* as the label
//! (`sherpa-onnx/csrc/utils.cc`, `EncodeBase`), so **the label cannot contain a
//! space** — a second word would be read as another token and fail to encode.
//! The model's own sample keyword file shows the convention: its
//! `keywords_raw.txt` line `LIGHT UP @LIGHT_UP` becomes `keywords.txt`'s
//! `L AY1 T AH1 P @LIGHT_UP`. So [`WakePhrase::label`] joins the phrase's words
//! with `_`, exactly as the vendor does.
//!
//! **Phrase.** The human text — *"say “hey via” to wake me"*. It may contain
//! spaces, because it is what a person reads and hears, and it is the only one
//! of the three that reaches [`via_i18n`].
//!
//! For a single-word phrase all three coincide, which is why the catalogued
//! upstream line round-trips byte for byte through [`WakePhrase`].

use std::collections::BTreeMap;

use via_i18n::Locale;

use crate::artifact::ModelArtifact;
use crate::error::KeywordError;

/// The separator between the token column and the display label.
///
/// **External contract** — `model-manager.mjs:95`, catalogued as
/// `generated keywords.txt content`.
pub const KEYWORD_DISPLAY_SEPARATOR: &str = " @";

/// What [`WakePhrase::label`] replaces a space in the phrase with.
///
/// **External contract** — the model's own `keywords_raw.txt` /`keywords.txt`
/// pair (`LIGHT UP` → `LIGHT_UP`). Any character would do as far as the parser
/// is concerned; this one is the vendor's.
pub const LABEL_WORD_SEPARATOR: char = '_';

/// The greatest number of keyword models VIA can have configured at once.
///
/// One per locale, and there are three locales (`docs/architecture.md` §16).
/// Derived from [`Locale::ALL`] rather than written down, so adding a fourth
/// locale moves this number without anyone remembering to.
pub const MAX_KEYWORD_MODELS: usize = Locale::ALL.len();

/// A wake phrase and the token line that matches it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WakePhrase {
    text: String,
    tokens: String,
}

impl WakePhrase {
    /// Build a phrase from its display text and its token line.
    ///
    /// Both are trimmed. Both must survive trimming, contain no line break and
    /// contain no `@`; no word of the token line may begin with `:` or `#`.
    ///
    /// # Errors
    ///
    /// [`KeywordError`], naming which of the two columns was wrong. None of the
    /// checks is stylistic — each one is a shape the keyword-file parser reads
    /// as something other than what was meant:
    ///
    /// | Rejected | What the parser would do |
    /// | --- | --- |
    /// | an empty column | register a keyword that matches nothing |
    /// | a line break | register a *second* keyword nobody configured |
    /// | `@` | move the token/label boundary |
    /// | a token word starting with `:` or `#` | read it as a boost score or a threshold, and `std::stof` the rest |
    pub fn new(text: impl Into<String>, tokens: impl Into<String>) -> Result<Self, KeywordError> {
        let text = validate("phrase", text.into(), false)?;
        let tokens = validate("tokens", tokens.into(), true)?;
        Ok(Self { text, tokens })
    }

    /// The human phrase — what a person is told to say, and what
    /// [`WakeWordSettings::asleep_message`](crate::WakeWordSettings::asleep_message)
    /// names.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The token line, in the model's modelling unit.
    #[must_use]
    pub fn tokens(&self) -> &str {
        &self.tokens
    }

    /// The display label — the phrase as one whitespace-free word.
    ///
    /// This is what the engine reports on a match, and therefore what
    /// [`KeywordSet::locale_of`] matches against. For a phrase with no spaces
    /// it is the phrase.
    #[must_use]
    pub fn label(&self) -> String {
        self.text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(&LABEL_WORD_SEPARATOR.to_string())
    }

    /// The keyword file line, trailing newline included.
    ///
    /// **External contract** — `model-manager.mjs:95`, catalogued as
    /// `generated keywords.txt content`: token line, one space, `@`, display
    /// label, newline.
    #[must_use]
    pub fn keyword_line(&self) -> String {
        format!(
            "{}{KEYWORD_DISPLAY_SEPARATOR}{}\n",
            self.tokens,
            self.label()
        )
    }
}

/// Trim and reject the shapes a column cannot have.
fn validate(
    field: &'static str,
    value: String,
    reject_markers: bool,
) -> Result<String, KeywordError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(KeywordError::Empty { field });
    }
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(KeywordError::LineBreak { field });
    }
    if trimmed.contains('@') {
        return Err(KeywordError::Separator { field });
    }
    if reject_markers {
        for word in trimmed.split_whitespace() {
            if let Some(marker @ (':' | '#')) = word.chars().next() {
                return Err(KeywordError::Marker { field, marker });
            }
        }
    }
    Ok(trimmed.to_owned())
}

/// A phrase bound to the model artifact that can hear it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keyword {
    phrase: WakePhrase,
    artifact: ModelArtifact,
}

impl Keyword {
    /// Bind a phrase to an artifact.
    #[must_use]
    pub fn new(phrase: WakePhrase, artifact: ModelArtifact) -> Self {
        Self { phrase, artifact }
    }

    /// Bind a phrase to the catalogued zh-en artifact.
    ///
    /// # Errors
    ///
    /// [`KeywordError`] from [`WakePhrase::new`].
    pub fn zh_en(text: impl Into<String>, tokens: impl Into<String>) -> Result<Self, KeywordError> {
        Ok(Self::new(
            WakePhrase::new(text, tokens)?,
            ModelArtifact::ZH_EN_3M,
        ))
    }

    /// The phrase.
    #[must_use]
    pub fn phrase(&self) -> &WakePhrase {
        &self.phrase
    }

    /// The human phrase, which is [`WakePhrase::text`].
    #[must_use]
    pub fn text(&self) -> &str {
        self.phrase.text()
    }

    /// The display label, which is [`WakePhrase::label`].
    #[must_use]
    pub fn label(&self) -> String {
        self.phrase.label()
    }

    /// The artifact that carries this keyword.
    #[must_use]
    pub fn artifact(&self) -> &ModelArtifact {
        &self.artifact
    }
}

/// The per-locale keyword table: at most [`MAX_KEYWORD_MODELS`] entries.
///
/// A `BTreeMap` keyed by [`Locale`], so the cap is structural rather than
/// checked — there is no way to put a fourth locale in — and iteration order is
/// `en`, `zh`, `ko` no matter what order the entries were added in. That
/// matters: it is the order keyword lines are written to disk in, and a stable
/// file is a file whose rewrite is a no-op.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeywordSet {
    entries: BTreeMap<Locale, Keyword>,
}

impl KeywordSet {
    /// An empty table — which is a valid configuration meaning "no wake word".
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or replace the keyword for `locale`, taking `self` by value.
    #[must_use]
    pub fn with(mut self, locale: Locale, keyword: Keyword) -> Self {
        self.insert(locale, keyword);
        self
    }

    /// Add or replace the keyword for `locale`, returning what it displaced.
    pub fn insert(&mut self, locale: Locale, keyword: Keyword) -> Option<Keyword> {
        self.entries.insert(locale, keyword)
    }

    /// The keyword configured for `locale`, if any.
    #[must_use]
    pub fn get(&self, locale: Locale) -> Option<&Keyword> {
        self.entries.get(&locale)
    }

    /// How many locales have a keyword. Never more than
    /// [`MAX_KEYWORD_MODELS`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every `(locale, keyword)` pair, in [`Locale::ALL`] order.
    pub fn iter(&self) -> impl Iterator<Item = (Locale, &Keyword)> {
        self.entries
            .iter()
            .map(|(locale, keyword)| (*locale, keyword))
    }

    /// The distinct artifacts this table needs installed, in locale order.
    ///
    /// Deduplicated by [`ModelArtifact::id`], because two locales sharing one
    /// artifact — which is exactly what `en` and `zh` do on the catalogued
    /// zh-en model — must produce one install and one download, not two.
    #[must_use]
    pub fn artifacts(&self) -> Vec<&ModelArtifact> {
        let mut seen: Vec<&ModelArtifact> = Vec::new();
        for keyword in self.entries.values() {
            if !seen.iter().any(|known| known.id == keyword.artifact.id) {
                seen.push(&keyword.artifact);
            }
        }
        seen
    }

    /// The `keywords.txt` content for one artifact.
    ///
    /// One line per configured phrase that resolves to `artifact_id`, in locale
    /// order, with identical lines collapsed: two locales configured with the
    /// same phrase on the same model are one keyword, and registering it twice
    /// is at best redundant.
    ///
    /// An artifact nothing resolves to yields the empty string, which is a
    /// keyword file with no keywords in it — deliberately not an error, because
    /// the caller that asks is installing an artifact it already resolved.
    #[must_use]
    pub fn keywords_file(&self, artifact_id: &str) -> String {
        let mut lines: Vec<String> = Vec::new();
        for keyword in self.entries.values() {
            if keyword.artifact.id != artifact_id {
                continue;
            }
            let line = keyword.phrase.keyword_line();
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        lines.concat()
    }

    /// Which locale's phrase a detected label belongs to.
    ///
    /// The engine reports the **label** — the `@` column of the line it matched
    /// — so that is what is matched here, and the human phrase is then read off
    /// the keyword. The first locale in [`Locale::ALL`] order wins when two
    /// share a label, which is the same tie-break [`Self::keywords_file`] uses
    /// when it collapses the duplicate line.
    #[must_use]
    pub fn locale_of(&self, label: &str) -> Option<Locale> {
        self.entries
            .iter()
            .find(|(_, keyword)| keyword.label() == label)
            .map(|(locale, _)| *locale)
    }
}

impl FromIterator<(Locale, Keyword)> for KeywordSet {
    fn from_iter<T: IntoIterator<Item = (Locale, Keyword)>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}
