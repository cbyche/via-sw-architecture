//! Ordered deixis — an enumerated, stably-ordered view of on-screen objects.
//!
//! `docs/architecture.md` §5 marks this **new**, and §2 makes it the reason
//! `interface` exists: *"Referents and ordered deixis are what turn 'click that
//! one' into an action against a concrete on-screen object."*
//!
//! # The order is the contract
//!
//! "The third one" must mean the same thing twice. Three rules make that true,
//! and each one is a refusal to be clever:
//!
//! 1. **The order is the source's own.** Objects are enumerated in the order
//!    the host gave them. Nothing here sorts, groups or re-ranks — the host
//!    knows its own reading order and VIA does not, so inventing one would put
//!    a layout opinion between the user's eyes and their words.
//! 2. **A generation change is the only thing that renumbers.**
//!    [`SurfaceSnapshot::generation`] is the version of the screen. A source
//!    that changes its objects *without* advancing it is contradicting itself,
//!    and [`ReferentRegistry::observe_surface`](crate::referent::ReferentRegistry::observe_surface)
//!    reports [`ContextError::SurfaceGenerationReused`] rather than quietly
//!    accepting either version. Silently renumbering is exactly the failure
//!    this whole module exists to prevent.
//! 3. **An object with no id is not enumerated.** Ordinals are handles onto
//!    identities; an object the host cannot name again next snapshot cannot be
//!    tracked for staleness, so it is dropped rather than given a position it
//!    could lose.
//!
//! # VIA has no screen
//!
//! [`SurfaceSource`] is an injected trait and this crate must not grow a screen
//! reader. The trait is deliberately synchronous: a host that has to cross an
//! IPC boundary caches the last frame it was pushed and answers from the cache,
//! which is what a screen source *is* — the host already knows what is on
//! screen. Making it `async` would invite a blocking round-trip inside prompt
//! assembly.
//!
//! [`ScriptedSurface`] ships for tests, and is compiled unconditionally rather
//! than behind a feature: it is a few dozen lines with no dependencies, and a
//! host wiring up its own affordances needs it to test that wiring.
//!
//! # Everything on screen is third-party
//!
//! A label is whatever the screen says. The host is first-party, the *bytes*
//! are not — which is the classification `docs/architecture.md` §5 says is
//! missing, in its most literal form. So [`DeixisView::section`] is
//! [`Provenance::ThirdParty`](crate::pack::Provenance::ThirdParty) and fenced,
//! and every label, kind and action is whitespace-collapsed first so a label
//! carrying a newline cannot forge a second enumerated line.

use std::collections::VecDeque;
use std::sync::Mutex;

use serde::Serialize;
use via_conversation::context::INPUT_FIELD_SEPARATOR;
use via_conversation::text::clean_bounded;
use via_i18n::Locale;

use crate::error::ContextError;
use crate::fence::FenceSource;
use crate::pack::{ContextSection, Provenance, SectionId, Tier, Trust};
use crate::referent::{
    MAX_LABEL_CHARS, Referent, ReferentBinding, ReferentRegistry, ReferentTarget, Resolution,
    SurfaceFingerprint, SurfaceMark,
};

/// How many objects one view enumerates.
///
/// Fifty. Beyond that an ordinal stops being something a person says and the
/// enumeration starts to outweigh the policy that governs it. A snapshot with
/// more is **not** an error — it is truncated, and the view says so, because a
/// host paging a long list is normal and refusing it would make the feature
/// unusable on a real screen.
pub const MAX_SURFACE_OBJECTS: usize = 50;

/// The code-point cap on a label, a kind or an action name.
pub const MAX_SURFACE_TEXT_CHARS: usize = MAX_LABEL_CHARS;

/// The surface source could not answer.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct SurfaceError(pub String);

impl SurfaceError {
    /// A failure with `detail` as its message.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self(detail.into())
    }
}

/// One thing on screen.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SurfaceObject {
    /// The host's own stable identity for it. An object with a blank id is not
    /// enumerated.
    pub id: String,
    /// What it says — a row's text, a button's caption.
    pub label: String,
    /// What kind of thing it is, in the host's own vocabulary.
    pub kind: String,
    /// The affordances the host will accept against it: `open`, `select`,
    /// `delete`.
    pub actions: Vec<String>,
}

impl SurfaceObject {
    /// An object with an id and a label.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: String::new(),
            actions: Vec::new(),
        }
    }

    /// Declare what may be done to it.
    #[must_use]
    pub fn with_actions<I, S>(mut self, actions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.actions = actions.into_iter().map(Into::into).collect();
        self
    }

    /// Declare what kind of thing it is.
    #[must_use]
    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    /// The same object with every free-form field whitespace-collapsed and
    /// clipped.
    ///
    /// Applied before anything is enumerated or fingerprinted, so a label
    /// carrying a newline cannot forge a second line of the enumeration and a
    /// label that grew a trailing space cannot register as
    /// [`Staleness::Moved`](crate::referent::Staleness::Moved).
    #[must_use]
    pub fn cleaned(&self) -> Self {
        Self {
            id: self.id.trim().to_owned(),
            label: clean_bounded(&self.label, MAX_SURFACE_TEXT_CHARS),
            kind: clean_bounded(&self.kind, MAX_SURFACE_TEXT_CHARS),
            actions: self
                .actions
                .iter()
                .map(|action| clean_bounded(action, MAX_SURFACE_TEXT_CHARS))
                .filter(|action| !action.is_empty())
                .collect(),
        }
    }
}

/// What is on screen, as of one version of it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SurfaceSnapshot {
    /// The version of the screen. Advancing it is what permits renumbering.
    pub generation: u64,
    /// The objects, in the host's own order.
    pub objects: Vec<SurfaceObject>,
}

impl SurfaceSnapshot {
    /// A snapshot of `objects` at `generation`.
    #[must_use]
    pub fn new(generation: u64, objects: Vec<SurfaceObject>) -> Self {
        Self {
            generation,
            objects,
        }
    }

    /// The enumerable objects, cleaned, deduplicated by id and capped.
    ///
    /// The single canonicalisation. Both [`DeixisView::build`] and
    /// [`SurfaceSnapshot::fingerprints`] read it, so what is enumerated and
    /// what is fingerprinted can never disagree — and a disagreement there
    /// would make every screen referent look [`Moved`](crate::referent::Staleness::Moved)
    /// forever.
    ///
    /// First occurrence of an id wins: an id is an identity, so a repeat is the
    /// host contradicting itself and the earlier position is the one the user
    /// would have heard.
    #[must_use]
    pub fn enumerated(&self) -> Vec<(usize, SurfaceObject)> {
        let mut seen: Vec<String> = Vec::new();
        let mut out: Vec<(usize, SurfaceObject)> = Vec::new();
        for object in &self.objects {
            let object = object.cleaned();
            if object.id.is_empty() || seen.contains(&object.id) {
                continue;
            }
            if out.len() >= MAX_SURFACE_OBJECTS {
                break;
            }
            seen.push(object.id.clone());
            out.push((out.len() + 1, object));
        }
        out
    }

    /// Whether [`enumerated`](Self::enumerated) dropped objects for want of
    /// room.
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        self.enumerated().len() < self.distinct_object_count()
    }

    /// Every enumerable object's id and fingerprint.
    pub fn fingerprints(&self) -> impl Iterator<Item = (String, SurfaceFingerprint)> {
        self.enumerated()
            .into_iter()
            .map(|(ordinal, object)| (object.id.clone(), SurfaceFingerprint::of(&object, ordinal)))
    }

    /// How many distinct, non-blank ids the raw snapshot carries.
    pub(crate) fn distinct_object_count(&self) -> usize {
        let mut seen: Vec<String> = Vec::new();
        for object in &self.objects {
            let id = object.id.trim();
            if !id.is_empty() && !seen.iter().any(|known| known == id) {
                seen.push(id.to_owned());
            }
        }
        seen.len()
    }
}

/// Where a snapshot comes from.
///
/// Injected. VIA has no screen capture and this crate must not grow one.
pub trait SurfaceSource: Send + Sync {
    /// What is on screen now.
    ///
    /// # Errors
    ///
    /// [`SurfaceError`] when the host cannot say. A session whose screen cannot
    /// be read resolves nothing on screen — it does not fall back to the last
    /// snapshot, because the whole point of the generation is that an
    /// unconfirmed screen is not a screen.
    fn snapshot(&self) -> Result<SurfaceSnapshot, SurfaceError>;
}

/// Which position the user named.
///
/// Parsing *"the third one"*, *"두 번째"* or *"最后一个"* out of speech is the
/// model's job, not this crate's — VIA has no NLU here, and pretending
/// otherwise would put a language-specific ordinal table in a crate that has no
/// business owning one. The model emits an [`Ordinal`]; this resolves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case", tag = "ordinal")]
pub enum Ordinal {
    /// The first.
    First,
    /// The `n`th, 1-based. Zero resolves to nothing, because there is no
    /// zeroth one.
    Nth {
        /// The 1-based position.
        position: usize,
    },
    /// The last.
    ///
    /// Refused on a truncated view: the last object is genuinely not in the
    /// enumeration, and answering with the last *visible* one would be the
    /// guess this crate never makes.
    Last,
}

/// One enumerated object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeicticEntry {
    /// Its 1-based position.
    pub ordinal: usize,
    /// The referent handle it was bound to.
    pub handle: String,
    /// The object, cleaned.
    pub object: SurfaceObject,
}

/// An enumerated, frozen view of one screen generation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct DeixisView {
    generation: u64,
    entries: Vec<DeicticEntry>,
    truncated: bool,
}

impl DeixisView {
    /// Enumerate `snapshot`, binding every object into `registry`.
    ///
    /// # Errors
    ///
    /// [`ContextError::SurfaceGenerationReused`] when the snapshot repeats a
    /// generation with different contents, and
    /// [`ContextError::UnbindableReferent`] if an object survives
    /// canonicalisation with no identity — which
    /// [`SurfaceSnapshot::enumerated`] already excludes, so it is a guard
    /// against a future change rather than a reachable state today.
    pub fn build(
        registry: &mut ReferentRegistry,
        snapshot: &SurfaceSnapshot,
    ) -> Result<Self, ContextError> {
        registry.observe_surface(snapshot)?;
        let enumerated = snapshot.enumerated();
        let truncated = enumerated.len() < snapshot.distinct_object_count();
        let mut entries = Vec::with_capacity(enumerated.len());
        for (ordinal, object) in enumerated {
            let mark = SurfaceMark {
                generation: snapshot.generation,
                fingerprint: SurfaceFingerprint::of(&object, ordinal),
            };
            let handle = registry.bind_surface_object(
                ReferentBinding::new(
                    ReferentTarget::Screen {
                        object_id: object.id.clone(),
                    },
                    object.label.clone(),
                )
                .from_surface(ordinal, mark),
            )?;
            entries.push(DeicticEntry {
                ordinal,
                handle,
                object,
            });
        }
        Ok(Self {
            generation: snapshot.generation,
            entries,
            truncated,
        })
    }

    /// The screen version this view froze.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// How many objects are enumerated.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is enumerated.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether the snapshot had more objects than [`MAX_SURFACE_OBJECTS`].
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// The enumeration, in order.
    #[must_use]
    pub fn entries(&self) -> &[DeicticEntry] {
        &self.entries
    }

    /// The handle at a 1-based position.
    #[must_use]
    pub fn handle_at(&self, ordinal: usize) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.ordinal == ordinal)
            .map(|entry| entry.handle.as_str())
    }

    /// Resolve an ordinal through `registry`, so staleness applies to it
    /// exactly as it does to a phrase.
    #[must_use]
    pub fn resolve(&self, registry: &ReferentRegistry, ordinal: Ordinal) -> Resolution {
        let position = match ordinal {
            Ordinal::First => 1,
            Ordinal::Nth { position } => position,
            Ordinal::Last if self.truncated => {
                return Resolution::NotFound {
                    candidates: registry.candidates(),
                };
            }
            Ordinal::Last => self.entries.len(),
        };
        match self.handle_at(position) {
            Some(handle) => registry.resolve_handle(handle),
            None => Resolution::NotFound {
                candidates: registry.candidates(),
            },
        }
    }

    /// The enumeration as prompt text, unfenced.
    ///
    /// One line per object: `N. ref_N · label · kind · action · action`, with
    /// empty parts dropped — the same shape and the same
    /// [`INPUT_FIELD_SEPARATOR`](via_conversation::context::INPUT_FIELD_SEPARATOR)
    /// `via-conversation` already uses to summarise an input, so a model sees
    /// one convention rather than two.
    ///
    /// Every part is already whitespace-collapsed by
    /// [`SurfaceObject::cleaned`], which is what stops a label forging a second
    /// numbered line. The fence in [`section`](Self::section) is defence in
    /// depth on top of that, not instead of it.
    #[must_use]
    pub fn body(&self) -> String {
        self.entries
            .iter()
            .map(|entry| {
                let parts = [
                    entry.handle.as_str(),
                    &entry.object.label,
                    &entry.object.kind,
                ]
                .into_iter()
                .chain(entry.object.actions.iter().map(String::as_str))
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(INPUT_FIELD_SEPARATOR);
                format!("{}. {parts}", entry.ordinal)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The enumeration as a fenced [`ContextSection`].
    ///
    /// [`Tier::SemiStatic`] because it is invalidated by a new screen, not by a
    /// new turn — which is ARGO's own definition of the tier
    /// (`tinicore/src/prompt/builder.rs:470-473`) and a real cache property: a
    /// user who talks for five turns without touching the screen re-sends
    /// nothing.
    #[must_use]
    pub fn section(&self, locale: Locale) -> ContextSection {
        ContextSection::new(
            locale,
            SectionId::on_screen(),
            Provenance::ThirdParty,
            Trust::Untrusted,
            Tier::SemiStatic,
            FenceSource::Screen,
            &self.body(),
        )
    }

    /// Every referent this view bound that is still live in `registry`.
    pub fn live<'a>(
        &'a self,
        registry: &'a ReferentRegistry,
    ) -> impl Iterator<Item = &'a Referent> {
        self.entries
            .iter()
            .filter_map(|entry| registry.get(&entry.handle))
            .filter(|referent| registry.staleness(referent).is_none())
    }
}

/// A surface source that answers from a script.
///
/// Each call takes the next scripted answer; once the script is exhausted the
/// last answer repeats, so a test that only cares about one screen writes one
/// entry.
#[derive(Debug)]
pub struct ScriptedSurface {
    state: Mutex<ScriptState>,
}

#[derive(Debug)]
struct ScriptState {
    queue: VecDeque<Result<SurfaceSnapshot, SurfaceError>>,
    last: Option<Result<SurfaceSnapshot, SurfaceError>>,
}

impl ScriptedSurface {
    /// A source that always answers with `snapshot`.
    #[must_use]
    pub fn fixed(snapshot: SurfaceSnapshot) -> Self {
        Self::scripted([Ok(snapshot)])
    }

    /// A source that always fails with `detail`.
    #[must_use]
    pub fn failing(detail: impl Into<String>) -> Self {
        Self::scripted([Err(SurfaceError::new(detail))])
    }

    /// A source that answers each call from `script` in order.
    #[must_use]
    pub fn scripted<I>(script: I) -> Self
    where
        I: IntoIterator<Item = Result<SurfaceSnapshot, SurfaceError>>,
    {
        Self {
            state: Mutex::new(ScriptState {
                queue: script.into_iter().collect(),
                last: None,
            }),
        }
    }

    /// How many scripted answers are left.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.state.lock().map_or(0, |state| state.queue.len())
    }
}

impl SurfaceSource for ScriptedSurface {
    fn snapshot(&self) -> Result<SurfaceSnapshot, SurfaceError> {
        // A poisoned lock answers with a failure rather than panicking: a
        // source that cannot be read is a screen VIA cannot vouch for, which is
        // already a first-class outcome here.
        let Ok(mut state) = self.state.lock() else {
            return Err(SurfaceError::new("the scripted surface lock is poisoned"));
        };
        match state.queue.pop_front() {
            Some(answer) => {
                state.last = Some(answer.clone());
                answer
            }
            None => state
                .last
                .clone()
                .unwrap_or_else(|| Err(SurfaceError::new("the surface script is empty"))),
        }
    }
}
