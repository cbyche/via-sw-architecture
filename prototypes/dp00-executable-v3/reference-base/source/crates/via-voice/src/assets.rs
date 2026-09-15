//! The per-session input asset registry.
//!
//! Ported from `server/src/voice/input-asset-registry.mjs`.
//!
//! An attachment the user submitted three turns ago is still referable —
//! *"summarize that screenshot"* — but the bytes must not be replayed into
//! every prompt. The registry gives each distinct attachment a short
//! `input_N` handle, hands the model the handle plus a label, and resolves the
//! handle back to the bytes only when a delegation actually needs them.
//!
//! # Why the handle is per session and not global
//!
//! `resolve` is reachable from a model-authored `spawn_thinking.input_refs`.
//! Scoping the table to `(owner, session)` means a fabricated handle can only
//! ever name something that session already submitted — the model cannot reach
//! another user's file by guessing `input_2`.

use std::collections::HashMap;

use indexmap::IndexMap;
use sha2::{Digest, Sha256};
use via_i18n::{Locale, format as i18n_format, keys, t};

use crate::input::{InputPart, input_file_parts, input_part_label, part_bytes};

/// How many attachments one session holds.
///
/// **External contract** — `input-asset-registry.mjs:43`
/// (`maxAssetsPerSession = 32`).
pub const DEFAULT_MAX_ASSETS_PER_SESSION: usize = 32;

/// How many attachment bytes one session holds.
///
/// **External contract** — `input-asset-registry.mjs:44` (64 MiB).
pub const DEFAULT_MAX_BYTES_PER_SESSION: usize = 64 * 1024 * 1024;

/// How many sessions the registry holds.
///
/// **External contract** — `input-asset-registry.mjs:45` (`maxSessions = 500`).
pub const DEFAULT_MAX_SESSIONS: usize = 500;

/// How long an untouched session survives.
///
/// **External contract** — `input-asset-registry.mjs:46` (six hours).
pub const DEFAULT_SESSION_TTL_MS: i64 = 6 * 60 * 60 * 1_000;

/// The prefix of every input handle.
///
/// **External contract** — `input-asset-registry.mjs:116`
/// (`` `input_${state.nextRef++}` ``). The model round-trips it in
/// `spawn_thinking.input_refs`, so it is a frozen shape.
pub const INPUT_REF_PREFIX: &str = "input_";

/// What the model is told about one referable attachment.
///
/// **External contract** — `input-asset-registry.mjs:151-163`, field names and
/// order included: it is serialized into `<recent_conversation>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMetadata {
    /// The `input_N` handle.
    pub reference: String,
    /// `image` or `file`.
    pub kind: &'static str,
    /// The rendered anchor.
    pub label: String,
    /// The original filename, or `None`.
    pub filename: Option<String>,
    /// The MIME type.
    pub mime: String,
}

/// A handle that names nothing this session can reach.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct AssetError {
    /// The localized message.
    pub message: String,
    /// Whether the whole session was gone, rather than one handle.
    pub session_expired: bool,
}

#[derive(Clone, Debug)]
struct Asset {
    part: InputPart,
    fingerprint: String,
    bytes: usize,
    /// The last turn that referenced this asset. Kept because eviction is by
    /// insertion order and this is the only record of continued relevance.
    last_seen_turn_id: Option<String>,
    last_accessed_at: i64,
}

#[derive(Debug, Default)]
struct SessionAssets {
    assets: IndexMap<String, Asset>,
    by_fingerprint: HashMap<String, String>,
    next_reference: u64,
    total_bytes: usize,
    last_accessed_at: i64,
}

/// A monotonic millisecond clock.
pub type Clock = std::sync::Arc<dyn Fn() -> i64 + Send + Sync>;

/// The registry.
#[derive(Clone)]
pub struct InputAssetRegistry {
    max_assets_per_session: usize,
    max_bytes_per_session: usize,
    max_sessions: usize,
    session_ttl_ms: i64,
    clock: Clock,
    locale: Locale,
    sessions: std::sync::Arc<std::sync::Mutex<IndexMap<String, SessionAssets>>>,
}

impl std::fmt::Debug for InputAssetRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputAssetRegistry")
            .field("max_assets_per_session", &self.max_assets_per_session)
            .field("max_bytes_per_session", &self.max_bytes_per_session)
            .field("max_sessions", &self.max_sessions)
            .finish_non_exhaustive()
    }
}

fn session_key(owner_id: &str, session_id: &str) -> String {
    format!("{owner_id}\u{0}{session_id}")
}

/// `sha256(mime \0 url)`.
///
/// **External contract** — `input-asset-registry.mjs:29-35`. The NUL is what
/// stops `("image/pn", "gX")` and `("image/png", "X")` colliding.
#[must_use]
pub fn fingerprint(part: &InputPart) -> String {
    let mut hasher = Sha256::new();
    hasher.update(part.mime().as_bytes());
    hasher.update([0u8]);
    hasher.update(part.url().as_bytes());
    hex::encode(hasher.finalize())
}

impl InputAssetRegistry {
    /// A registry with the contract bounds and the system clock.
    #[must_use]
    pub fn new(locale: Locale) -> Self {
        Self::with_clock(
            locale,
            std::sync::Arc::new(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |elapsed| {
                        i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
                    })
            }),
        )
    }

    /// A registry driven by `clock`.
    #[must_use]
    pub fn with_clock(locale: Locale, clock: Clock) -> Self {
        Self {
            max_assets_per_session: DEFAULT_MAX_ASSETS_PER_SESSION,
            max_bytes_per_session: DEFAULT_MAX_BYTES_PER_SESSION,
            max_sessions: DEFAULT_MAX_SESSIONS,
            session_ttl_ms: DEFAULT_SESSION_TTL_MS,
            clock,
            locale,
            sessions: std::sync::Arc::new(std::sync::Mutex::new(IndexMap::new())),
        }
    }

    /// Override the per-session attachment cap.
    #[must_use]
    pub const fn max_assets_per_session(mut self, max: usize) -> Self {
        self.max_assets_per_session = max;
        self
    }

    /// Override the per-session byte cap.
    #[must_use]
    pub const fn max_bytes_per_session(mut self, max: usize) -> Self {
        self.max_bytes_per_session = max;
        self
    }

    /// Override the session cap.
    #[must_use]
    pub const fn max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = max;
        self
    }

    /// Override the session TTL.
    #[must_use]
    pub const fn session_ttl_ms(mut self, ttl_ms: i64) -> Self {
        self.session_ttl_ms = ttl_ms;
        self
    }

    fn with_sessions<R>(&self, body: impl FnOnce(&mut IndexMap<String, SessionAssets>) -> R) -> R {
        match self.sessions.lock() {
            Ok(mut sessions) => body(&mut sessions),
            Err(poisoned) => body(&mut poisoned.into_inner()),
        }
    }

    /// Bind an `input_N` handle to every attachment in `parts`.
    ///
    /// **External contract** — `input-asset-registry.mjs:104-134`. Two
    /// properties:
    ///
    /// - a fingerprint that is already registered **reuses** its handle, so the
    ///   same screenshot pasted twice is one asset and one reference;
    /// - text parts pass through untouched, because only attachments are
    ///   referable.
    pub fn register_parts(
        &self,
        owner_id: &str,
        session_id: &str,
        turn_id: &str,
        parts: &[InputPart],
    ) -> Vec<InputPart> {
        let now = (self.clock)();
        self.prune_at(now);
        self.with_sessions(|sessions| {
            let key = session_key(owner_id, session_id);
            Self::ensure_session(sessions, &key, self.max_sessions, now);
            let Some(state) = sessions.get_mut(&key) else {
                return parts.to_vec();
            };
            state.last_accessed_at = now;
            parts
                .iter()
                .map(|part| {
                    if !part.is_file() {
                        return part.clone();
                    }
                    let hash = fingerprint(part);
                    if let Some(reference) = state.by_fingerprint.get(&hash).cloned()
                        && let Some(known) = state.assets.get_mut(&reference)
                    {
                        if !turn_id.is_empty() {
                            known.last_seen_turn_id = Some(turn_id.to_owned());
                        }
                        known.last_accessed_at = now;
                        return part.clone().with_reference(&reference);
                    }
                    state.next_reference += 1;
                    let reference = format!("{INPUT_REF_PREFIX}{}", state.next_reference);
                    let enriched = part.clone().with_reference(&reference);
                    let bytes = part_bytes(part);
                    state.assets.insert(
                        reference.clone(),
                        Asset {
                            part: enriched.clone(),
                            fingerprint: hash.clone(),
                            bytes,
                            last_seen_turn_id: (!turn_id.is_empty()).then(|| turn_id.to_owned()),
                            last_accessed_at: now,
                        },
                    );
                    state.by_fingerprint.insert(hash, reference);
                    state.total_bytes = state.total_bytes.saturating_add(bytes);
                    Self::evict(
                        state,
                        self.max_assets_per_session,
                        self.max_bytes_per_session,
                    );
                    enriched
                })
                .collect()
        })
    }

    fn ensure_session(
        sessions: &mut IndexMap<String, SessionAssets>,
        key: &str,
        max_sessions: usize,
        now: i64,
    ) {
        if sessions.contains_key(key) {
            return;
        }
        while sessions.len() >= max_sessions {
            let oldest = sessions
                .iter()
                .min_by_key(|(id, state)| (state.last_accessed_at, (*id).clone()))
                .map(|(id, _)| id.clone());
            let Some(oldest) = oldest else { break };
            sessions.shift_remove(&oldest);
        }
        sessions.insert(
            key.to_owned(),
            SessionAssets {
                last_accessed_at: now,
                ..SessionAssets::default()
            },
        );
    }

    /// Evict oldest-first until both caps hold.
    ///
    /// **External contract** — `input-asset-registry.mjs:91-102`. Eviction is
    /// by insertion order, not by last access: an old attachment that is still
    /// being referenced is the one the model has been told about, and rotating
    /// on access would let a busy session keep a stale reference alive forever.
    fn evict(state: &mut SessionAssets, max_assets: usize, max_bytes: usize) {
        while state.assets.len() > max_assets || state.total_bytes > max_bytes {
            let Some((_, oldest)) = state.assets.shift_remove_index(0) else {
                break;
            };
            state.by_fingerprint.remove(&oldest.fingerprint);
            state.total_bytes = state.total_bytes.saturating_sub(oldest.bytes);
        }
    }

    /// Drop sessions that have not been touched within the TTL.
    pub fn prune(&self) {
        self.prune_at((self.clock)());
    }

    fn prune_at(&self, now: i64) {
        self.with_sessions(|sessions| {
            sessions.retain(|_, state| {
                now.saturating_sub(state.last_accessed_at) < self.session_ttl_ms
            });
        });
    }

    /// Resolve `references` back to their attachments.
    ///
    /// **External contract** — `input-asset-registry.mjs:136-149`. Empty input
    /// answers empty; an unknown session and an unknown handle are **different**
    /// errors, because the first means "start over" and the second means "you
    /// named the wrong one". Duplicates are collapsed, preserving first-seen
    /// order.
    ///
    /// # Errors
    ///
    /// [`AssetError`] with the localized message the model is shown.
    pub fn resolve(
        &self,
        owner_id: &str,
        session_id: &str,
        references: &[String],
    ) -> Result<Vec<InputPart>, AssetError> {
        if references.is_empty() {
            return Ok(Vec::new());
        }
        let now = (self.clock)();
        self.prune_at(now);
        self.with_sessions(|sessions| {
            let key = session_key(owner_id, session_id);
            let Some(state) = sessions.get_mut(&key) else {
                return Err(AssetError {
                    message: t(self.locale, keys::INPUT_REFERENCE_EXPIRED).to_owned(),
                    session_expired: true,
                });
            };
            state.last_accessed_at = now;
            let mut unique: Vec<String> = Vec::with_capacity(references.len());
            for reference in references {
                let trimmed = crate::text::trim(reference).to_owned();
                if trimmed.is_empty() || unique.contains(&trimmed) {
                    continue;
                }
                unique.push(trimmed);
            }
            unique
                .into_iter()
                .map(|reference| {
                    let Some(asset) = state.assets.get_mut(&reference) else {
                        return Err(AssetError {
                            message: i18n_format(
                                self.locale,
                                keys::INPUT_REFERENCE_NOT_FOUND,
                                &[("ref", &reference)],
                            ),
                            session_expired: false,
                        });
                    };
                    asset.last_accessed_at = now;
                    Ok(asset.part.clone())
                })
                .collect()
        })
    }

    /// The metadata for the attachments in `parts`.
    ///
    /// **External contract** — `input-asset-registry.mjs:151-163`. A part with
    /// no bound handle contributes nothing: the model cannot reference what it
    /// has no handle for, and listing it would invite a fabricated reference.
    #[must_use]
    pub fn metadata_for_parts(&self, parts: &[InputPart]) -> Vec<AssetMetadata> {
        input_file_parts(parts)
            .iter()
            .enumerate()
            .filter_map(|(index, part)| {
                let reference = part.reference();
                if reference.is_empty() {
                    return None;
                }
                Some(AssetMetadata {
                    reference: reference.to_owned(),
                    kind: if part.mime().starts_with("image/") {
                        "image"
                    } else {
                        "file"
                    },
                    label: input_part_label(part, index),
                    filename: part.filename().map(str::to_owned),
                    mime: part.mime().to_owned(),
                })
            })
            .collect()
    }

    /// How many attachments a session holds. Diagnostics only.
    #[must_use]
    pub fn asset_count(&self, owner_id: &str, session_id: &str) -> usize {
        self.with_sessions(|sessions| {
            sessions
                .get(&session_key(owner_id, session_id))
                .map_or(0, |state| state.assets.len())
        })
    }

    /// How many sessions the registry holds. Diagnostics only.
    #[must_use]
    pub fn session_count(&self) -> usize {
        self.with_sessions(|sessions| sessions.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, Ordering};

    fn clock() -> (Clock, Arc<AtomicI64>) {
        let now = Arc::new(AtomicI64::new(1_000));
        let handle = Arc::clone(&now);
        (Arc::new(move || handle.load(Ordering::SeqCst)), now)
    }

    fn image(seed: &str) -> InputPart {
        InputPart::file("image/png", format!("data:image/png;base64,{seed}"))
    }

    fn registry() -> (InputAssetRegistry, Arc<AtomicI64>) {
        let (clock, now) = clock();
        (InputAssetRegistry::with_clock(Locale::Zh, clock), now)
    }

    #[test]
    fn handles_are_minted_in_order_and_bound_to_the_part() {
        let (registry, _) = registry();
        let parts = registry.register_parts(
            "alice",
            "main",
            "voice-1",
            &[InputPart::text("look"), image("AAAA"), image("BBBB")],
        );
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].reference(), "", "text parts are untouched");
        assert_eq!(parts[1].reference(), "input_1");
        assert_eq!(parts[2].reference(), "input_2");
    }

    #[test]
    fn the_same_attachment_reuses_its_handle() {
        let (registry, _) = registry();
        registry.register_parts("alice", "main", "voice-1", &[image("AAAA")]);
        let again = registry.register_parts("alice", "main", "voice-2", &[image("AAAA")]);
        assert_eq!(again[0].reference(), "input_1");
        assert_eq!(registry.asset_count("alice", "main"), 1);
    }

    #[test]
    fn a_fingerprint_cannot_be_forged_across_the_separator() {
        let one = InputPart::file("image/pn", "data:image/png;base64,AA==");
        let two = InputPart::file("image/pn\u{0}", "data:image/png;base64,AA==");
        assert_ne!(fingerprint(&one), fingerprint(&two));
    }

    #[test]
    fn resolution_returns_the_registered_bytes() {
        let (registry, _) = registry();
        registry.register_parts("alice", "main", "voice-1", &[image("AAAA")]);
        let resolved = registry
            .resolve("alice", "main", &["input_1".to_owned()])
            .expect("a registered handle resolves");
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].url(), "data:image/png;base64,AAAA");
    }

    #[test]
    fn resolution_is_scoped_so_another_session_cannot_be_reached() {
        let (registry, _) = registry();
        registry.register_parts("alice", "main", "voice-1", &[image("AAAA")]);
        let error = registry
            .resolve("alice", "other", &["input_1".to_owned()])
            .expect_err("a different session holds nothing");
        assert!(error.session_expired);
        let error = registry
            .resolve("bob", "main", &["input_1".to_owned()])
            .expect_err("a different owner holds nothing");
        assert!(error.session_expired);
    }

    #[test]
    fn a_fabricated_handle_in_a_live_session_is_a_not_found() {
        let (registry, _) = registry();
        registry.register_parts("alice", "main", "voice-1", &[image("AAAA")]);
        let error = registry
            .resolve("alice", "main", &["input_99".to_owned()])
            .expect_err("the model invented a handle");
        assert!(!error.session_expired);
        assert!(error.message.contains("input_99"));
    }

    #[test]
    fn duplicates_collapse_and_blanks_are_dropped() {
        let (registry, _) = registry();
        registry.register_parts("alice", "main", "voice-1", &[image("AAAA"), image("BBBB")]);
        let resolved = registry
            .resolve(
                "alice",
                "main",
                &[
                    " input_2 ".to_owned(),
                    "input_1".to_owned(),
                    "input_2".to_owned(),
                    "  ".to_owned(),
                ],
            )
            .expect("resolves");
        assert_eq!(resolved.len(), 2);
        assert_eq!(
            resolved[0].url(),
            "data:image/png;base64,BBBB",
            "first-seen order"
        );
    }

    #[test]
    fn an_empty_reference_list_answers_empty_without_touching_the_session() {
        let (registry, _) = registry();
        assert_eq!(
            registry.resolve("nobody", "nowhere", &[]).expect("empty"),
            Vec::new()
        );
    }

    #[test]
    fn the_asset_cap_evicts_oldest_first() {
        let (clock, _) = clock();
        let registry = InputAssetRegistry::with_clock(Locale::Zh, clock).max_assets_per_session(2);
        for seed in ["AAAA", "BBBB", "CCCC"] {
            registry.register_parts("alice", "main", "voice-1", &[image(seed)]);
        }
        assert_eq!(registry.asset_count("alice", "main"), 2);
        assert!(
            registry
                .resolve("alice", "main", &["input_1".to_owned()])
                .is_err()
        );
        assert!(
            registry
                .resolve("alice", "main", &["input_3".to_owned()])
                .is_ok()
        );
    }

    #[test]
    fn the_byte_cap_evicts_too_and_the_fingerprint_goes_with_it() {
        let (clock, _) = clock();
        // Eight base64 characters decode to six bytes, so a six-byte budget
        // holds exactly one attachment.
        let registry = InputAssetRegistry::with_clock(Locale::Zh, clock).max_bytes_per_session(6);
        registry.register_parts("alice", "main", "voice-1", &[image("AAAAAAAA")]);
        registry.register_parts("alice", "main", "voice-1", &[image("BBBBBBBB")]);
        assert_eq!(registry.asset_count("alice", "main"), 1);
        // The evicted fingerprint was forgotten, so re-submitting mints a new
        // handle rather than resurrecting a dead one.
        let again = registry.register_parts("alice", "main", "voice-1", &[image("AAAAAAAA")]);
        assert_eq!(again[0].reference(), "input_3");
    }

    #[test]
    fn a_session_expires_after_the_ttl() {
        let (registry, now) = registry();
        registry.register_parts("alice", "main", "voice-1", &[image("AAAA")]);
        now.store(1_000 + DEFAULT_SESSION_TTL_MS, Ordering::SeqCst);
        let error = registry
            .resolve("alice", "main", &["input_1".to_owned()])
            .expect_err("expired");
        assert!(error.session_expired);
        assert_eq!(registry.session_count(), 0);
    }

    #[test]
    fn the_session_cap_evicts_the_least_recently_used() {
        let (clock, now) = clock();
        let registry = InputAssetRegistry::with_clock(Locale::Zh, clock).max_sessions(2);
        for (index, session) in ["a", "b"].into_iter().enumerate() {
            now.store(
                1_000 + i64::try_from(index).unwrap_or_default(),
                Ordering::SeqCst,
            );
            registry.register_parts("alice", session, "voice-1", &[image("AAAA")]);
        }
        now.store(1_010, Ordering::SeqCst);
        registry.register_parts("alice", "c", "voice-1", &[image("AAAA")]);
        assert_eq!(registry.session_count(), 2);
        assert!(
            registry
                .resolve("alice", "a", &["input_1".to_owned()])
                .is_err()
        );
        assert!(
            registry
                .resolve("alice", "c", &["input_1".to_owned()])
                .is_ok()
        );
    }

    #[test]
    fn metadata_lists_only_parts_that_have_a_handle() {
        let (registry, _) = registry();
        let registered = registry.register_parts("alice", "main", "voice-1", &[image("AAAA")]);
        let unregistered = InputPart::File {
            mime: "text/plain".to_owned(),
            filename: Some("notes.txt".to_owned()),
            url: "https://example.test/n".to_owned(),
            source: None,
            meta: None,
        };
        let mut parts = registered;
        parts.push(unregistered);
        let metadata = registry.metadata_for_parts(&parts);
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata[0].reference, "input_1");
        assert_eq!(metadata[0].kind, "image");
        assert_eq!(metadata[0].label, "[Image 1]");
        assert_eq!(metadata[0].mime, "image/png");
    }

    #[test]
    fn a_non_image_attachment_is_labelled_file() {
        let (registry, _) = registry();
        let part = InputPart::File {
            mime: "application/pdf".to_owned(),
            filename: Some("spec.pdf".to_owned()),
            url: "https://example.test/spec.pdf".to_owned(),
            source: None,
            meta: None,
        };
        let parts = registry.register_parts("alice", "main", "voice-1", &[part]);
        let metadata = registry.metadata_for_parts(&parts);
        assert_eq!(metadata[0].kind, "file");
        assert_eq!(metadata[0].label, "@spec.pdf");
        assert_eq!(metadata[0].filename.as_deref(), Some("spec.pdf"));
    }

    #[test]
    fn a_remote_attachment_costs_no_bytes_against_the_session_budget() {
        let (clock, _) = clock();
        let registry = InputAssetRegistry::with_clock(Locale::Zh, clock).max_bytes_per_session(1);
        let remote = InputPart::file("text/plain", "https://example.test/a");
        registry.register_parts("alice", "main", "voice-1", &[remote]);
        assert_eq!(
            registry.asset_count("alice", "main"),
            1,
            "zero bytes, not evicted"
        );
    }
}
