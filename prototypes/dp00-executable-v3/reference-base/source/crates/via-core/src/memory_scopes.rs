//! The two frontend memory documents.
//!
//! Ported from `server/src/core/memory-scopes.mjs`. The scope names are
//! model-visible through the memory tool and are the keys of
//! `/api/health.frontendMemory.documents`, so they are wire values; the aliases
//! exist so an upgrade does not lose memories written under the older
//! spellings.
//!
//! The `kind` distinction is the point of the table: a `directive` entry is a
//! standing instruction the user authored and the model follows with authority,
//! while `data` is passive material it treats as background. Collapsing the two
//! would let a remembered fact start behaving like an instruction.
//!
//! The Chinese labels are injected into model context, so they are `via-i18n`'s
//! (`memory.scope.user_label` / `memory.scope.memory_label`) rather than
//! literals here — see [`ScopeMeta::label`].

use via_i18n::{Key, Locale, keys, t};

/// A memory document's semantic class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopeKind {
    /// A user-authored standing instruction, injected with authority.
    Directive,
    /// Passive material the model treats as data.
    Data,
}

/// One row of the memory-scope table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopeMeta {
    /// The canonical scope name, as it appears on the wire.
    pub scope: &'static str,
    /// Directive or data.
    pub kind: ScopeKind,
    /// Cap on entries in this document.
    pub max_entries: usize,
    /// Cap on characters per entry.
    pub max_chars: usize,
    label_key: Key,
}

impl ScopeMeta {
    /// The label injected into model context, in `locale`.
    ///
    /// Upstream's literals are `用户偏好` and `长期记忆`
    /// (`server/src/core/memory-scopes.mjs:11,17`); `via-i18n`'s `zh` column
    /// carries them verbatim.
    #[must_use]
    pub fn label(&self, locale: Locale) -> &'static str {
        t(locale, self.label_key)
    }
}

/// The user-preferences document.
pub const USER_SCOPE: &str = "user";
/// The long-term-memory document.
pub const MEMORY_SCOPE: &str = "memory";

/// The internal sentinel meaning "no scope filter".
///
/// **External contract** — `server/src/core/memory-scopes.mjs:40`. It is
/// deliberately **not** part of the tool enum: the tool schema expresses "all"
/// by omitting the scope argument, so a model can never send it.
pub const ALL_SCOPE: &str = "all";

/// The two documents, in declaration order.
///
/// **External contract** — `server/src/core/memory-scopes.mjs:8-21`. The order
/// is `Object.keys(MEMORY_SCOPES)`, which is what `MEMORY_DOCUMENTS` publishes.
pub static MEMORY_SCOPES: [ScopeMeta; 2] = [
    ScopeMeta {
        scope: USER_SCOPE,
        kind: ScopeKind::Directive,
        max_entries: 32,
        max_chars: 500,
        label_key: keys::MEMORY_SCOPE_USER_LABEL,
    },
    ScopeMeta {
        scope: MEMORY_SCOPE,
        kind: ScopeKind::Data,
        max_entries: 32,
        max_chars: 500,
        label_key: keys::MEMORY_SCOPE_MEMORY_LABEL,
    },
];

/// Legacy scope spellings and what they mean now.
///
/// **External contract** — `server/src/core/memory-scopes.mjs:26-31`.
/// `profile` and `rules` used to overlap; both converge on `user`.
pub static SCOPE_ALIASES: [(&str, &str); 4] = [
    ("profile", USER_SCOPE),
    ("rules", USER_SCOPE),
    ("facts", MEMORY_SCOPE),
    ("long_term", MEMORY_SCOPE),
];

/// The document names, in table order.
#[must_use]
pub fn memory_documents() -> Vec<&'static str> {
    MEMORY_SCOPES.iter().map(|scope| scope.scope).collect()
}

/// Resolve a scope name through the alias table.
///
/// **External contract** — `server/src/core/memory-scopes.mjs:33-36`: trim,
/// lowercase, then map through the aliases. An unrecognised value is returned
/// as-is rather than rejected, so [`is_memory_document`] is the membership
/// test.
#[must_use]
pub fn canonical_scope(value: &str) -> String {
    let scope = value.trim().to_lowercase();
    SCOPE_ALIASES
        .iter()
        .find(|(alias, _)| *alias == scope)
        .map_or(scope, |(_, canonical)| (*canonical).to_owned())
}

/// The table row for a scope, alias-resolved.
#[must_use]
pub fn scope_meta(value: &str) -> Option<&'static ScopeMeta> {
    let canonical = canonical_scope(value);
    MEMORY_SCOPES.iter().find(|meta| meta.scope == canonical)
}

/// Whether a value names one of the two documents.
#[must_use]
pub fn is_memory_document(value: &str) -> bool {
    scope_meta(value).is_some()
}

/// Whether a scope's entries are standing instructions.
#[must_use]
pub fn is_directive_scope(value: &str) -> bool {
    scope_meta(value).is_some_and(|meta| meta.kind == ScopeKind::Directive)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_alias_resolves_to_a_real_document() {
        for (alias, canonical) in SCOPE_ALIASES {
            assert_eq!(canonical_scope(alias), canonical);
            assert!(is_memory_document(alias));
        }
    }

    #[test]
    fn the_all_sentinel_is_not_a_document() {
        assert!(!is_memory_document(ALL_SCOPE));
        assert_eq!(canonical_scope(ALL_SCOPE), ALL_SCOPE);
    }

    #[test]
    fn only_user_is_directive() {
        assert!(is_directive_scope("profile"));
        assert!(is_directive_scope(" USER "));
        assert!(!is_directive_scope("memory"));
        assert!(!is_directive_scope("facts"));
    }
}
