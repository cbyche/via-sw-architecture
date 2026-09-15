//! [`ContextEngine`] — the three pieces, held together for one session.
//!
//! `docs/architecture.md` §2: *"In the other three modes the Context Engine
//! assembles context; in `interface` it also resolves targets."* That sentence
//! is this type's entire shape:
//!
//! - [`pack`](ContextEngine::pack) works in every mode. It is the assembly half.
//! - [`refresh_surface`](ContextEngine::refresh_surface) consumes the injected
//!   [`SurfaceSource`] only in [`SessionMode::Interface`]; in any other mode it
//!   reports [`SurfaceStatus::NotApplicable`] and binds nothing.
//! - [`resolve`](ContextEngine::resolve) works in every mode, over whatever is
//!   bound. Outside `interface` that is the conversation's own referents — an
//!   `input_N`, a `work_id`, the working directory — which need no screen at
//!   all.
//!
//! # Failure is a state, not a fallback
//!
//! Every path that cannot vouch for the screen calls
//! [`ReferentRegistry::forget_surface`], which turns every screen referent into
//! [`Staleness::SurfaceUnknown`](crate::referent::Staleness::SurfaceUnknown).
//! A source that errors, a generation the source contradicted, a session whose
//! mode does not mount a screen — all three land in the same place, and none of
//! them leaves the last good snapshot in play. Acting on a screen nobody can
//! confirm is the failure this crate exists to make impossible.
//!
//! # Ownership
//!
//! Plain state with `&mut self` methods. `docs/architecture.md` §11 gives an
//! owning task to each of the four invariants that need FIFO order across
//! tasks; a per-session referent registry is not one of them, so it lives in
//! whichever task owns the session rather than acquiring a lock of its own.

use std::fmt;
use std::sync::Arc;

use serde::Serialize;
use via_protocol::SessionMode;

use crate::assemble::{FrontendPackInputs, frontend_builder};
use crate::deixis::{DeixisView, Ordinal, SurfaceSource};
use crate::error::ContextError;
use crate::fence::FenceSource;
use crate::pack::{ContextPack, SectionId, Tier};
use crate::referent::{ReferentBinding, ReferentRegistry, Resolution};

/// Where the session's screen source stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum SurfaceStatus {
    /// This mode does not mount a screen. Every mode but
    /// [`SessionMode::Interface`].
    NotApplicable,
    /// `interface`, and no [`SurfaceSource`] was injected.
    ///
    /// The honest state for a Gateway a host has not wired a screen into: the
    /// mode is real, referent resolution over the conversation works, and
    /// nothing on screen resolves because there is no screen.
    Unbound,
    /// A snapshot was read and enumerated.
    Bound {
        /// The snapshot's generation.
        generation: u64,
        /// How many objects were enumerated.
        objects: usize,
        /// Whether the snapshot held more than
        /// [`MAX_SURFACE_OBJECTS`](crate::deixis::MAX_SURFACE_OBJECTS).
        truncated: bool,
    },
    /// The source could not answer, or contradicted itself.
    Failed {
        /// What went wrong.
        detail: String,
    },
}

impl SurfaceStatus {
    /// A stable machine token, for `/api/health` or a log record.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Unbound => "unbound",
            Self::Bound { .. } => "bound",
            Self::Failed { .. } => "failed",
        }
    }

    /// Whether anything on screen can be resolved.
    #[must_use]
    pub const fn is_bound(&self) -> bool {
        matches!(self, Self::Bound { .. })
    }
}

/// One session's Context Engine.
pub struct ContextEngine {
    mode: SessionMode,
    referents: ReferentRegistry,
    source: Option<Arc<dyn SurfaceSource>>,
    view: Option<DeixisView>,
    status: SurfaceStatus,
}

impl fmt::Debug for ContextEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextEngine")
            .field("mode", &self.mode)
            .field("referents", &self.referents.len())
            .field("source", &self.source.is_some())
            .field("status", &self.status)
            .finish()
    }
}

impl ContextEngine {
    /// An engine for `mode`, with no screen source.
    #[must_use]
    pub fn new(mode: SessionMode) -> Self {
        Self {
            mode,
            referents: ReferentRegistry::new(),
            source: None,
            view: None,
            status: initial_status(mode),
        }
    }

    /// An engine for `mode`, reading the screen through `source`.
    #[must_use]
    pub fn with_surface(mode: SessionMode, source: Arc<dyn SurfaceSource>) -> Self {
        Self {
            mode,
            referents: ReferentRegistry::new(),
            source: Some(source),
            view: None,
            status: initial_status(mode),
        }
    }

    /// The session's mode.
    #[must_use]
    pub const fn mode(&self) -> SessionMode {
        self.mode
    }

    /// Whether this mode resolves on-screen targets.
    ///
    /// True only for [`SessionMode::Interface`] — `docs/architecture.md` §2.
    #[must_use]
    pub const fn resolves_targets(&self) -> bool {
        matches!(self.mode, SessionMode::Interface)
    }

    /// The referent registry.
    #[must_use]
    pub const fn referents(&self) -> &ReferentRegistry {
        &self.referents
    }

    /// The referent registry, to bind into.
    pub const fn referents_mut(&mut self) -> &mut ReferentRegistry {
        &mut self.referents
    }

    /// The current enumeration, if there is one.
    #[must_use]
    pub const fn view(&self) -> Option<&DeixisView> {
        self.view.as_ref()
    }

    /// Where the screen source stands.
    #[must_use]
    pub const fn surface_status(&self) -> &SurfaceStatus {
        &self.status
    }

    /// Advance the turn, expiring what the turn ended.
    pub fn begin_turn(&mut self) -> u64 {
        self.referents.begin_turn()
    }

    /// Bind a referent that did not come from the screen.
    ///
    /// # Errors
    ///
    /// [`ContextError::UnbindableReferent`] when the binding has no target or a
    /// blank identity.
    pub fn bind(&mut self, binding: ReferentBinding) -> Result<String, ContextError> {
        self.referents.bind(binding)
    }

    /// Read the screen and re-enumerate it.
    ///
    /// # Errors
    ///
    /// [`ContextError::Surface`] when the source could not answer, and
    /// [`ContextError::SurfaceGenerationReused`] when it repeated a generation
    /// with different objects. Both leave the engine with **no** screen: the
    /// previous view is dropped and every screen referent becomes
    /// [`SurfaceUnknown`](crate::referent::Staleness::SurfaceUnknown). The
    /// status is updated before the error is returned, so a caller that only
    /// wants to report can read [`surface_status`](Self::surface_status).
    pub fn refresh_surface(&mut self) -> Result<&SurfaceStatus, ContextError> {
        if !self.resolves_targets() {
            self.drop_surface(SurfaceStatus::NotApplicable);
            return Ok(&self.status);
        }
        let Some(source) = self.source.clone() else {
            self.drop_surface(SurfaceStatus::Unbound);
            return Ok(&self.status);
        };
        let snapshot = match source.snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                let detail = error.to_string();
                self.drop_surface(SurfaceStatus::Failed {
                    detail: detail.clone(),
                });
                return Err(ContextError::Surface { detail });
            }
        };
        match DeixisView::build(&mut self.referents, &snapshot) {
            Ok(view) => {
                self.status = SurfaceStatus::Bound {
                    generation: view.generation(),
                    objects: view.len(),
                    truncated: view.is_truncated(),
                };
                self.view = Some(view);
                Ok(&self.status)
            }
            Err(error) => {
                self.drop_surface(SurfaceStatus::Failed {
                    detail: error.to_string(),
                });
                Err(error)
            }
        }
    }

    /// Resolve a spoken phrase against everything bound.
    #[must_use]
    pub fn resolve(&self, phrase: &str) -> Resolution {
        self.referents.resolve(phrase)
    }

    /// Resolve a handle the model sent back.
    #[must_use]
    pub fn resolve_handle(&self, handle: &str) -> Resolution {
        self.referents.resolve_handle(handle)
    }

    /// Resolve an ordinal against the current enumeration.
    ///
    /// With no enumeration this is [`Resolution::NotFound`] carrying whatever
    /// *is* live, never a fallback to a previous screen.
    #[must_use]
    pub fn resolve_ordinal(&self, ordinal: Ordinal) -> Resolution {
        match &self.view {
            Some(view) => view.resolve(&self.referents, ordinal),
            None => Resolution::NotFound {
                candidates: self.referents.candidates(),
            },
        }
    }

    /// Assemble the pack: the frontend sections, plus the enumeration when
    /// there is one.
    ///
    /// The enumeration is appended as a fenced [`Tier::SemiStatic`] section —
    /// see [`DeixisView::section`] for why that tier and why fenced.
    ///
    /// # Errors
    ///
    /// [`ContextError`] from the builder. Unreachable with the ids this
    /// function uses, all of which are [`SectionId::KNOWN`] and each used once.
    pub fn pack(&self, inputs: &FrontendPackInputs<'_>) -> Result<ContextPack, ContextError> {
        let mut builder = frontend_builder(inputs);
        if let Some(view) = &self.view
            && !view.is_empty()
        {
            builder = builder.push_untrusted(
                SectionId::ON_SCREEN,
                Tier::SemiStatic,
                FenceSource::Screen,
                &view.body(),
            );
        }
        builder.build()
    }

    fn drop_surface(&mut self, status: SurfaceStatus) {
        self.view = None;
        self.referents.forget_surface();
        self.status = status;
    }
}

/// The status before anything has been read.
///
/// `interface` starts [`Unbound`](SurfaceStatus::Unbound) whether or not a
/// source was injected, because a source that has not been *asked* has not
/// vouched for anything. `Bound` is reachable only through
/// [`ContextEngine::refresh_surface`].
const fn initial_status(mode: SessionMode) -> SurfaceStatus {
    match mode {
        SessionMode::Interface => SurfaceStatus::Unbound,
        _ => SurfaceStatus::NotApplicable,
    }
}
