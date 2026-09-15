//! The opaque catalog key.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

use crate::locale::Locale;

/// One catalog row: the dotted name, the three translations, and the
/// placeholder names they all share.
///
/// Built only by `build.rs`. Crate-private, so the layout is free to change.
pub(crate) struct Entry {
    /// The dotted key, e.g. `gateway.already_running`.
    pub(crate) name: &'static str,
    /// The English text.
    pub(crate) en: &'static str,
    /// The Chinese text.
    pub(crate) zh: &'static str,
    /// The Korean text.
    pub(crate) ko: &'static str,
    /// The `{placeholder}` names, sorted and deduplicated. `build.rs` has
    /// already proved this set is identical in all three locales.
    pub(crate) placeholders: &'static [&'static str],
}

/// A key into the message catalog.
///
/// Opaque: the only values that exist are the [`keys`](crate::keys) consts
/// `build.rs` generates from `assets/i18n/*.json`. There is no constructor
/// that takes a string, so "key not found" is not a runtime state — a
/// mistyped key does not compile.
///
/// # Representation
///
/// A `Key` holds a `&'static Entry`, not the dotted name, so
/// [`t`](crate::t) is a field read rather than a lookup that could fail. This
/// is a deliberate deviation from the sketched
/// `pub struct Key(&'static str)`: with a name inside, every lookup would need
/// a fallible search and therefore a branch for the impossible miss — and the
/// only thing that branch could return is the key name, which is exactly the
/// silent fallback this crate exists to prevent. [`Key::as_str`] still hands
/// back the dotted name.
#[derive(Clone, Copy)]
pub struct Key(pub(crate) &'static Entry);

impl Key {
    /// The dotted name, e.g. `"gateway.already_running"`.
    ///
    /// Stable: it is the catalog's own identifier and is used as such by
    /// `via-conformance`. It is *not* a lookup handle — there is deliberately
    /// no `Key::from_str`.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        self.0.name
    }

    /// The text for one locale. Same thing as [`t`](crate::t), spelled as a
    /// method.
    #[must_use]
    pub fn text(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::En => self.0.en,
            Locale::Zh => self.0.zh,
            Locale::Ko => self.0.ko,
        }
    }

    /// The `{placeholder}` names this message takes, sorted and deduplicated.
    ///
    /// Identical in all three locales — `build.rs` refuses to emit a key whose
    /// locales disagree, which is the structural-parity half of
    /// `docs/architecture.md` §16 ("same `{{variable}}` set").
    ///
    /// Empty for a message that takes no arguments, which is also the exact
    /// condition under which [`format`](crate::format) and
    /// [`t`](crate::t) agree.
    #[must_use]
    pub fn placeholders(&self) -> &'static [&'static str] {
        self.0.placeholders
    }
}

// Identity is the dotted name. Two `Key`s for the same catalog row may hold
// different addresses (each const promotes its own `Entry`), so pointer
// equality would be wrong.
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}

impl Eq for Key {}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.name.cmp(other.0.name)
    }
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key({:?})", self.0.name)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.name)
    }
}
