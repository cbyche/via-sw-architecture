//! VIA's **Context Engine** — what the model is given, and what "that one"
//! means.
//!
//! `docs/architecture.md` §5 marks this box *"OURS · NO QWEN EQUIVALENT"*, and
//! the verification of both upstreams agreed: **referents and ordered deixis
//! have no implementation in either.** qwen-audio-agent has no Context Engine at
//! all; ARGO has prompt assembly, memory and three narrow fencing conventions,
//! and nothing to extend for the rest. So this crate is designed rather than
//! transcribed — and where an upstream *did* solve something, it is reused
//! rather than re-derived.
//!
//! ```
//! use std::sync::Arc;
//! use via_context::{ContextEngine, Ordinal, ScriptedSurface, SurfaceObject, SurfaceSnapshot};
//! use via_protocol::SessionMode;
//!
//! let screen = SurfaceSnapshot::new(
//!     7,
//!     vec![
//!         SurfaceObject::new("row-a", "Inbox mail").with_actions(["open"]),
//!         SurfaceObject::new("row-b", "Archived mail").with_actions(["open", "delete"]),
//!     ],
//! );
//! let mut engine =
//!     ContextEngine::with_surface(SessionMode::Interface, Arc::new(ScriptedSurface::fixed(screen)));
//! engine.refresh_surface()?;
//!
//! // "the second one" resolves to a concrete object — the order is the contract.
//! let second = engine.resolve_ordinal(Ordinal::Nth { position: 2 });
//! assert_eq!(second.found().map(|r| r.label.as_str()), Some("Archived mail"));
//!
//! // The label resolves to the same referent, through the same ladder.
//! assert_eq!(
//!     engine.resolve("archived mail").found().map(|r| r.handle.as_str()),
//!     second.found().map(|r| r.handle.as_str()),
//! );
//!
//! // A phrase that fits both reports candidates. It never picks.
//! let ambiguous = engine.resolve("mail");
//! assert_eq!(ambiguous.as_str(), "ambiguous");
//! assert!(ambiguous.found().is_none());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # The four pieces
//!
//! | Module | What | Status against the upstreams |
//! | --- | --- | --- |
//! | [`pack`] | [`ContextPack`] — per-section provenance, trust, tier and token cost | **new**, wrapping ARGO's `PromptTier` vocabulary and `via-conversation`'s blocks |
//! | [`referent`] | a stable handle bound to a concrete object, and the ladder that resolves it | **new** — nothing to port |
//! | [`deixis`] | an enumerated, stably-ordered view of on-screen objects | **new** — nothing to port |
//! | [`fence`] | provenance-driven fencing | ARGO's mechanism, extended with the classification it was missing |
//! | [`engine`] | the three held together for one session | — |
//!
//! # The three claims this crate makes, and where each is proved
//!
//! **1. The pack is a thin layer, and the proof is an equality.**
//! [`ContextPack::instructions`] reproduces `via-voice`'s
//! `assemble_frontend_instructions` byte for byte for the same inputs. Nothing
//! here re-renders a block: `via-conversation` renders `<user_preferences>`,
//! `<user_memory>`, `<runtime_context>` and `<recent_conversation>`, `via-voice`
//! renders `PROMPT.md`, the persona wrapper and `<input_parts>`, and
//! [`assemble`] joins what they produced. The assertion lives in `via-voice`'s
//! `tests/context_pack.rs`, because Layer 1 may depend on Layer 2 and never the
//! reverse (`docs/architecture.md` §9).
//!
//! **2. Resolution never guesses.** Every ladder in [`referent`] ends in
//! candidates rather than a pick, which is `via-conversation`'s notes rule
//! (`frontend-notes.mjs:50-83`) reused for its reason — *a wrong guess on a
//! destructive action is unrecoverable.* A stale referent is reported stale,
//! with which of four things went wrong, rather than silently resolved or
//! silently dropped.
//!
//! **3. The fence follows the bytes, not the caller.**
//! [`Provenance::classify`] takes *both* [`ToolTrust`] and [`ContentSource`],
//! and a first-party tool returning external bytes is
//! [`Provenance::ThirdParty`]. That is the one thing
//! `docs/architecture.md` §5 says was missing. Provenance then floors
//! [`Trust`], and [`Trust::Untrusted`] fences inside
//! [`ContextSection::new`] — so there is no ordering of calls in which
//! third-party bytes reach a rendered prompt unfenced.
//!
//! # `SessionMode::Interface`
//!
//! This crate is what stops that mode degrading. `via-voice`'s `ModePlan` used
//! to resolve `interface` to `direct` unconditionally and report
//! `context_engine_unavailable` on `/api/health`; it now takes the Context
//! Engine's availability as a fact, and the degradation is reported only when a
//! caller says there is no engine. What a session gets in `interface` mode
//! without a host-registered screen is [`SurfaceStatus::Unbound`] — referents
//! over the conversation resolve, nothing on screen does — which is a state, not
//! a silent lie.
//!
//! # What this crate builds on rather than restating
//!
//! - [`via_conversation::context`] owns every frontend block and its catalogued
//!   format; [`via_conversation::text`] owns `clean` / `trim` /
//!   `bounded_code_points`; [`via_conversation::notes`] owns
//!   `MAX_RESULT_CANDIDATES` and `MAX_ITEM_CHARS`, which the referent ladder
//!   reuses because a spoken referent label is the same kind of thing as a
//!   spoken note item.
//! - [`via_i18n`] owns the two model-facing sentences here: the fence's framing
//!   line and its truncation marker.
//! - [`via_protocol`] owns [`SessionMode`](via_protocol::SessionMode).
//!
//! Deviations are recorded in `docs/deviations/phase-7.md`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod assemble;
pub mod deixis;
pub mod engine;
pub mod error;
pub mod fence;
pub mod pack;
pub mod referent;

pub use assemble::{FrontendPackInputs, frontend_builder, frontend_pack};
pub use deixis::{
    DeicticEntry, DeixisView, MAX_SURFACE_OBJECTS, Ordinal, ScriptedSurface, SurfaceError,
    SurfaceObject, SurfaceSnapshot, SurfaceSource,
};
pub use engine::{ContextEngine, SurfaceStatus};
pub use error::ContextError;
pub use fence::{FENCE_CLOSE, FENCE_OPEN, FenceSource, MAX_FENCED_CHARS, neutralize};
pub use pack::{
    ContentSource, ContextPack, ContextPackBuilder, ContextSection, Provenance, SectionId, Tier,
    ToolTrust, Trust, estimate_tokens,
};
pub use referent::{
    Candidate, Referent, ReferentBinding, ReferentRegistry, ReferentTarget, Resolution, Staleness,
    SurfaceFingerprint, SurfaceMark,
};
