//! The three locales and how one is chosen.
//!
//! `docs/architecture.md` §16: *"Locale resolves `VIA_LOCALE` → OS locale →
//! `en`"*. `en` is the default, `zh` carries upstream qwen-audio-agent's own
//! text, and `ko` is an authored peer.

use core::fmt;
use core::str::FromStr;

/// A locale VIA can render a message in.
///
/// The wire spelling is the two-letter code — `"en"`, `"zh"`, `"ko"` — and
/// nothing else. Regional variants are a *resolution* input (see
/// [`Locale::from_language_tag`]), never a catalog dimension: VIA ships one
/// Chinese and one Korean text, not one per region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Locale {
    /// English. The default, per `docs/architecture.md` §16.
    #[default]
    En,
    /// Chinese. Upstream qwen-audio-agent's own text, byte for byte.
    Zh,
    /// Korean. An authored peer of `zh`.
    Ko,
}

impl Locale {
    /// Every locale, in declaration order: `en`, `zh`, `ko`.
    ///
    /// `en` is first because it is the default and the fallback of last resort.
    pub const ALL: &'static [Locale] = &[Locale::En, Locale::Zh, Locale::Ko];

    /// The wire spelling: `"en"`, `"zh"` or `"ko"`.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Zh => "zh",
            Locale::Ko => "ko",
        }
    }

    /// Parse a wire value. Exact match only — this is the strict half.
    ///
    /// `"EN"`, `"en_US"` and `"en-GB"` are all `None`. A client that sends a
    /// locale on the wire sends one of the three spellings or none; anything
    /// else is a protocol error for the caller to report, not something to
    /// guess at. The lenient half is [`Locale::from_language_tag`].
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Locale> {
        match s {
            "en" => Some(Locale::En),
            "zh" => Some(Locale::Zh),
            "ko" => Some(Locale::Ko),
            _ => None,
        }
    }

    /// Parse an operating-system locale tag, leniently.
    ///
    /// Handles the shapes an environment actually produces:
    /// `zh_CN.UTF-8`, `zh-Hans-CN`, `ko_KR`, `en_US@euro`, `EN`. The codeset
    /// (`.UTF-8`), the modifier (`@euro`) and every subtag after the language
    /// are discarded, and the language subtag is matched case-insensitively.
    ///
    /// `C` and `POSIX` are *not* locales — they mean "no localization" — and
    /// return `None` so resolution moves on to the next source rather than
    /// pinning a language the user never chose.
    #[must_use]
    pub fn from_language_tag(tag: &str) -> Option<Locale> {
        let trimmed = tag.trim();
        // Codeset and modifier first: `zh_CN.UTF-8@pinyin` -> `zh_CN`.
        let without_codeset = trimmed.split(['.', '@']).next().unwrap_or(trimmed);
        let language = without_codeset
            .split(['_', '-'])
            .next()
            .unwrap_or(without_codeset);
        if language.is_empty() {
            return None;
        }
        let mut lowered = String::with_capacity(language.len());
        for ch in language.chars() {
            lowered.extend(ch.to_lowercase());
        }
        Locale::from_wire(&lowered)
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The error [`Locale::from_str`] returns for a value that is not `en`, `zh`
/// or `ko`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownLocale {
    /// The value that was offered.
    pub value: String,
}

impl fmt::Display for UnknownLocale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is not a VIA locale (expected en, zh or ko)",
            self.value
        )
    }
}

impl std::error::Error for UnknownLocale {}

impl FromStr for Locale {
    type Err = UnknownLocale;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Locale::from_wire(s).ok_or_else(|| UnknownLocale {
            value: s.to_owned(),
        })
    }
}

/// The environment variable that pins the locale explicitly.
///
/// VIA-owned name, per `docs/rebrand.md`'s `QWEN_AUDIO_AGENT_*` → `VIA_*` rule.
pub const LOCALE_ENV: &str = "VIA_LOCALE";

/// The operating-system locale variables, in the order they are consulted.
///
/// `LC_ALL` overrides everything, `LC_MESSAGES` is the category that governs
/// program messages, and `LANG` is the fallback — that is POSIX's own
/// precedence. `AppleLocale` is macOS's own store (`defaults read -g
/// AppleLocale`), consulted last because a Mac with a POSIX variable set means
/// the user set it deliberately.
pub const OS_LOCALE_ENV_ORDER: &[&str] = &["LC_ALL", "LC_MESSAGES", "LANG", "AppleLocale"];

/// Resolve the locale: `VIA_LOCALE` → OS locale → [`Locale::En`].
///
/// `env` is an injected reader — `&|name| std::env::var(name).ok()` in
/// production — so tests need no process-global mutation. Edition 2024 makes
/// `std::env::set_var` `unsafe`, and a test suite that mutated the real
/// environment would have to be serialized; this crate's locale tests run in
/// parallel because nothing they touch is shared.
///
/// A source that is absent, blank, or carries a value this build has no
/// catalog for is **skipped**, and resolution continues. That is deliberate: a
/// machine set to `fr_FR.UTF-8` should land on `en`, not fail, and a
/// `LANG=C.UTF-8` build server should not out-vote a `VIA_LOCALE` that was
/// never set. The consequence to know about is that a *typo* in `VIA_LOCALE`
/// falls through to the OS locale rather than erroring — use
/// [`Locale::from_wire`] directly if a caller needs to reject a bad value
/// loudly.
///
/// ```
/// use via_i18n::{Locale, resolve_locale};
///
/// let env = |name: &str| match name {
///     "LANG" => Some("ko_KR.UTF-8".to_owned()),
///     _ => None,
/// };
/// assert_eq!(resolve_locale(&env), Locale::Ko);
/// ```
#[must_use]
pub fn resolve_locale(env: &dyn Fn(&str) -> Option<String>) -> Locale {
    // The explicit pin is the wire spelling, matched strictly: `VIA_LOCALE=zh`
    // is a VIA setting, not an OS tag, and accepting `zh_CN` here would make
    // the same string mean two different things in two places.
    if let Some(value) = env(LOCALE_ENV)
        && let Some(locale) = Locale::from_wire(value.trim())
    {
        return locale;
    }
    for name in OS_LOCALE_ENV_ORDER {
        if let Some(value) = env(name)
            && let Some(locale) = Locale::from_language_tag(&value)
        {
            return locale;
        }
    }
    Locale::En
}
