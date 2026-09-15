//! VIA's trilingual message catalog.
//!
//! Three locales, per `docs/architecture.md` §16: `en` (the default), `zh` and
//! `ko`. Every other VIA crate localizes through this one, so it takes no VIA
//! dependency and no runtime dependency at all — the catalog is compiled in.
//!
//! ```
//! use via_i18n::{Locale, format, keys, t};
//!
//! assert_eq!(t(Locale::En, keys::LOCK_LEASE_EXHAUSTED), "could not acquire the Gateway instance lease");
//! assert_eq!(t(Locale::Zh, keys::LOCK_LEASE_EXHAUSTED), "无法获取 Gateway 实例租约");
//!
//! let message = format(
//!     Locale::En,
//!     keys::LOCK_GATEWAY_ALREADY_RUNNING_AT,
//!     &[("origin", "http://127.0.0.1:3101")],
//! );
//! assert_eq!(message, "a Gateway is already running: http://127.0.0.1:3101");
//! ```
//!
//! # Fidelity
//!
//! `zh` is **not** authored. It is upstream qwen-audio-agent's own text,
//! reproduced byte for byte, which is what keeps the catalogued
//! `prompt-text` / `error-code` contracts in `docs/reference/contracts.json`
//! assertable against a VIA build. The two exceptions are both mandated
//! elsewhere and both are recorded on the key that carries them:
//!
//! 1. **Identity.** `docs/rebrand.md` renames the product name, the binary
//!    name and the MCP tool names inside otherwise-verbatim sentences, so
//!    `请运行 qwenaudio config …` becomes `请运行 via config …`. The Chinese
//!    prose around the renamed token is untouched.
//! 2. **Configuration.** Upstream hard-codes its own wake phrase into the
//!    sleep message. VIA's wake phrase is configuration, never a literal
//!    (`docs/architecture.md` §16), so the key takes a `{wake_word}`
//!    placeholder.
//!
//! `en` and `ko` are authored peers written against `zh` as the
//! specification: same message set, same placeholder set, same register.
//!
//! # Discipline
//!
//! - Keys are generated `pub const`s, not strings. A key that is not in the
//!   catalog does not compile.
//! - A key that is missing a locale, or has an empty one, or whose locales
//!   disagree about their placeholders, fails `build.rs` — the build, not a
//!   test, and never a silent fallback to the key name.
//! - An unfilled or unused placeholder is a [`FormatError`], not a blank.
//!
//! # Scope
//!
//! This is the catalog of **messages** — what a person reads or hears, plus
//! the short model-facing scaffolding VIA composes at runtime. It is
//! deliberately not the home of the two prompt *documents*: `PROMPT.md` and
//! `ASSISTANT.md` ship as `assets/frontend-agent/{en,zh,ko}/` per
//! `docs/architecture.md` §16, because they are edited as documents and
//! diffed as documents.

mod catalog;
mod key;
mod locale;
mod render;

pub use catalog::keys;
pub use key::Key;
pub use locale::{LOCALE_ENV, Locale, OS_LOCALE_ENV_ORDER, UnknownLocale, resolve_locale};
pub use render::{FormatError, render};

/// Every key in the catalog, sorted by dotted name.
///
/// Exists so a test can walk the whole table — `via-conformance` asserts the
/// `zh` column against `docs/reference/contracts.json` this way, and this
/// crate's own tests use it to prove the three locales stay in lockstep.
#[must_use]
pub fn all_keys() -> &'static [Key] {
    &catalog::ALL
}

/// The number of keys in the catalog.
#[must_use]
pub fn key_count() -> usize {
    catalog::ALL.len()
}

/// Look up a key.
///
/// Total: a `Key` can only have come from a generated const, so there is no
/// miss to report. Use it for messages that take no placeholders; a message
/// that does take them will come back with the `{name}` holes still in it,
/// which is why [`Key::placeholders`] is public and why the crate's tests
/// assert that every caller-facing key with placeholders is reached through
/// [`format()`].
#[must_use]
pub fn t(locale: Locale, key: Key) -> &'static str {
    key.text(locale)
}

/// Look up a key and interpolate, reporting what went wrong.
///
/// This is the API to prefer. [`format()`] exists for call sites that cannot
/// carry a `Result`, and it turns an error into a visible diagnostic rather
/// than a plausible-looking sentence.
///
/// # Errors
///
/// Any [`FormatError`]: a placeholder with no argument, an argument the
/// message does not use, or a repeated argument name. A malformed *template*
/// cannot occur here — `build.rs` rejects one — but [`render`] answers for
/// that case.
pub fn try_format(locale: Locale, key: Key, args: &[(&str, &str)]) -> Result<String, FormatError> {
    render(t(locale, key), args)
}

/// Look up a key and interpolate.
///
/// The signature other crates code against. On success it is exactly
/// [`try_format`]; on failure it renders a diagnostic that names the key, the
/// locale and the fault:
///
/// ```
/// use via_i18n::{Locale, format, keys};
///
/// let broken = format(Locale::En, keys::LOCK_GATEWAY_ALREADY_RUNNING_AT, &[]);
/// assert_eq!(
///     broken,
///     "<via-i18n: cannot render `lock.gateway_already_running_at` in en: \
///      no value for placeholder `{origin}`>"
/// );
/// ```
///
/// It does not panic (`scripts/panic_filter.py` keeps its baseline) and it
/// does not return the half-filled sentence. Losing a value silently is the
/// failure mode this crate exists to prevent, so the fault is carried all the
/// way to whatever surface the message was headed for, where a human or a test
/// will see it.
#[must_use]
pub fn format(locale: Locale, key: Key, args: &[(&str, &str)]) -> String {
    match try_format(locale, key, args) {
        Ok(text) => text,
        Err(error) => std::format!(
            "<via-i18n: cannot render `{}` in {locale}: {error}>",
            key.as_str()
        ),
    }
}
