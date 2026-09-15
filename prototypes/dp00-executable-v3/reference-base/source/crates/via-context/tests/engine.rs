//! The engine: which half of it each mode mounts, and what happens when the
//! screen cannot be vouched for.

mod common;

use std::sync::Arc;

use common::client;
use pretty_assertions::assert_eq;
use rstest::rstest;
use via_context::{
    ContextEngine, ContextError, FrontendPackInputs, Ordinal, ReferentBinding, ReferentTarget,
    Resolution, ScriptedSurface, SectionId, Staleness, SurfaceError, SurfaceObject,
    SurfaceSnapshot, SurfaceStatus, Tier,
};
use via_i18n::Locale;
use via_protocol::SessionMode;

const POLICY: &str = "CORE POLICY";
const PERSONA: &str = "# Assistant Profile\n<assistant_profile authority=\"persona_only\">\nPERSONA\n</assistant_profile>";

fn screen() -> SurfaceSnapshot {
    SurfaceSnapshot::new(
        7,
        vec![
            SurfaceObject::new("row-a", "Inbox").with_actions(["open"]),
            SurfaceObject::new("row-b", "Drafts").with_actions(["open", "delete"]),
        ],
    )
}

fn interface_engine() -> ContextEngine {
    ContextEngine::with_surface(
        SessionMode::Interface,
        Arc::new(ScriptedSurface::fixed(screen())),
    )
}

fn pack_of(engine: &ContextEngine) -> via_context::ContextPack {
    let client = client("Asia/Shanghai", "zh-CN", "");
    engine
        .pack(&FrontendPackInputs::new(
            Locale::En,
            POLICY,
            PERSONA,
            &client,
        ))
        .expect("the pack builds")
}

// ── which half each mode mounts ─────────────────────────────────────────────

#[rstest]
#[case(SessionMode::Dictation, false)]
#[case(SessionMode::Direct, false)]
#[case(SessionMode::Agent, false)]
#[case(SessionMode::Interface, true)]
fn only_interface_resolves_on_screen_targets(#[case] mode: SessionMode, #[case] resolves: bool) {
    let mut engine = ContextEngine::with_surface(mode, Arc::new(ScriptedSurface::fixed(screen())));
    assert_eq!(engine.mode(), mode);
    assert_eq!(engine.resolves_targets(), resolves);
    engine.refresh_surface().expect("a refresh is always safe");
    assert_eq!(engine.surface_status().is_bound(), resolves);
    assert_eq!(engine.view().is_some(), resolves);
    if !resolves {
        assert_eq!(*engine.surface_status(), SurfaceStatus::NotApplicable);
        assert_eq!(engine.resolve("inbox").as_str(), "not_found");
    }
}

#[rstest]
#[case(SessionMode::Dictation)]
#[case(SessionMode::Direct)]
#[case(SessionMode::Agent)]
#[case(SessionMode::Interface)]
fn every_mode_assembles_context_and_resolves_conversation_referents(#[case] mode: SessionMode) {
    // `docs/architecture.md` §2: *"In the other three modes the Context Engine
    // assembles context."* An `input_N` or a `work_id` needs no screen.
    let mut engine = ContextEngine::new(mode);
    let handle = engine
        .bind(ReferentBinding::new(
            ReferentTarget::Input {
                reference: "input_1".to_owned(),
            },
            "the photo",
        ))
        .expect("a valid binding");
    assert_eq!(
        engine.resolve("photo").found().map(|r| r.handle.clone()),
        Some(handle),
    );
    assert!(!pack_of(&engine).is_empty());
}

#[test]
fn a_non_interface_mode_reports_not_applicable_rather_than_a_failure() {
    let engine = ContextEngine::new(SessionMode::Agent);
    assert_eq!(*engine.surface_status(), SurfaceStatus::NotApplicable);
    assert_eq!(engine.surface_status().as_str(), "not_applicable");
}

#[test]
fn interface_without_an_injected_source_is_unbound_not_broken() {
    // The honest state for a Gateway a host has not wired a screen into.
    let mut engine = ContextEngine::new(SessionMode::Interface);
    assert_eq!(*engine.surface_status(), SurfaceStatus::Unbound);
    engine.refresh_surface().expect("no source is not an error");
    assert_eq!(*engine.surface_status(), SurfaceStatus::Unbound);
    assert!(engine.view().is_none());
    // …and conversation referents still resolve.
    engine
        .bind(ReferentBinding::new(
            ReferentTarget::Directory {
                path: "/srv/project".to_owned(),
            },
            "this directory",
        ))
        .expect("a valid binding");
    assert!(engine.resolve("this directory").is_found());
}

#[test]
fn a_source_is_never_read_before_it_is_asked() {
    let engine = interface_engine();
    assert_eq!(
        *engine.surface_status(),
        SurfaceStatus::Unbound,
        "constructing an engine reads nothing",
    );
    assert!(engine.view().is_none());
}

// ── the happy path ──────────────────────────────────────────────────────────

#[test]
fn a_refreshed_interface_session_resolves_ordinals_labels_and_handles() {
    let mut engine = interface_engine();
    engine.refresh_surface().expect("the screen reads");
    assert_eq!(
        *engine.surface_status(),
        SurfaceStatus::Bound {
            generation: 7,
            objects: 2,
            truncated: false,
        },
    );
    let by_ordinal = engine.resolve_ordinal(Ordinal::Nth { position: 2 });
    let by_label = engine.resolve("drafts");
    let handle = by_ordinal
        .found()
        .map(|referent| referent.handle.clone())
        .expect("the second one");
    assert_eq!(
        by_label.found().map(|r| r.handle.clone()),
        Some(handle.clone())
    );
    assert!(engine.resolve_handle(&handle).is_found());
}

#[test]
fn an_ambiguous_phrase_reports_candidates_the_model_can_ask_about() {
    let mut engine = ContextEngine::with_surface(
        SessionMode::Interface,
        Arc::new(ScriptedSurface::fixed(SurfaceSnapshot::new(
            3,
            vec![
                SurfaceObject::new("row-a", "Inbox mail"),
                SurfaceObject::new("row-b", "Archived mail"),
            ],
        ))),
    );
    engine.refresh_surface().expect("the screen reads");
    let resolved = engine.resolve("mail");
    let Resolution::Ambiguous { candidates } = &resolved else {
        panic!("expected Ambiguous, got {}", resolved.as_str());
    };
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].ordinal, Some(1));
    assert_eq!(candidates[1].ordinal, Some(2));
    assert!(candidates.iter().all(|c| c.kind == "screen"));
    assert!(resolved.found().is_none(), "it never picks");
    // …and the unambiguous half of the same phrase still resolves.
    assert!(engine.resolve("inbox mail").is_found());
}

#[test]
fn the_pack_gains_the_enumeration_only_when_there_is_one() {
    let mut engine = interface_engine();
    assert!(pack_of(&engine).section(SectionId::ON_SCREEN).is_none());
    engine.refresh_surface().expect("the screen reads");
    let pack = pack_of(&engine);
    let section = pack
        .section(SectionId::ON_SCREEN)
        .expect("the enumeration is in the pack");
    assert_eq!(section.tier, Tier::SemiStatic);
    assert!(section.is_fenced());
    assert!(section.body.contains("Drafts"));
    // The instructions are untouched by it: the screen is not session policy.
    assert!(!pack.instructions().contains("Drafts"));
    assert_eq!(pack.tokens_in(Tier::SemiStatic), section.tokens);
}

#[test]
fn the_engines_section_is_the_views_section() {
    // Two paths reach the same block — `DeixisView::section` for a caller that
    // holds the view, and `ContextEngine::pack` for one that holds the engine.
    // They must not drift apart, because only one of them is fenced-by-
    // construction in anybody's head.
    let mut engine = interface_engine();
    engine.refresh_surface().expect("the screen reads");
    let view = engine.view().expect("a view");
    let from_view = view.section(Locale::En);
    let from_pack = pack_of(&engine)
        .section(SectionId::ON_SCREEN)
        .expect("the enumeration")
        .clone();
    assert_eq!(from_pack, from_view);
}

#[test]
fn an_empty_screen_adds_no_section_at_all() {
    let mut engine = ContextEngine::with_surface(
        SessionMode::Interface,
        Arc::new(ScriptedSurface::fixed(SurfaceSnapshot::new(1, Vec::new()))),
    );
    engine
        .refresh_surface()
        .expect("an empty screen is a screen");
    assert!(engine.view().is_some());
    assert!(pack_of(&engine).section(SectionId::ON_SCREEN).is_none());
}

// ── failure is a state, not a fallback ──────────────────────────────────────

#[test]
fn a_source_that_fails_drops_the_screen_rather_than_keeping_the_last_one() {
    let mut engine = ContextEngine::with_surface(
        SessionMode::Interface,
        Arc::new(ScriptedSurface::scripted([
            Ok(screen()),
            Err(SurfaceError::new("the host stopped sharing")),
        ])),
    );
    engine.refresh_surface().expect("the first read works");
    let handle = engine
        .resolve_ordinal(Ordinal::First)
        .found()
        .map(|referent| referent.handle.clone())
        .expect("the first one");

    let error = engine.refresh_surface().expect_err("the second read fails");
    assert_eq!(
        error,
        ContextError::Surface {
            detail: "the host stopped sharing".to_owned(),
        },
    );
    assert_eq!(
        *engine.surface_status(),
        SurfaceStatus::Failed {
            detail: "the host stopped sharing".to_owned(),
        },
    );
    assert_eq!(engine.surface_status().as_str(), "failed");
    assert!(engine.view().is_none(), "the stale view is dropped");
    assert!(matches!(
        engine.resolve_handle(&handle),
        Resolution::Stale {
            reason: Staleness::SurfaceUnknown,
            ..
        },
    ));
    assert_eq!(engine.resolve_ordinal(Ordinal::First).as_str(), "not_found");
    // And the pack stops claiming to know what is on screen.
    assert!(pack_of(&engine).section(SectionId::ON_SCREEN).is_none());
}

#[test]
fn a_source_that_contradicts_its_own_generation_drops_the_screen_too() {
    let mut engine = ContextEngine::with_surface(
        SessionMode::Interface,
        Arc::new(ScriptedSurface::scripted([
            Ok(screen()),
            Ok(SurfaceSnapshot::new(
                7,
                vec![SurfaceObject::new("row-z", "Something else")],
            )),
        ])),
    );
    engine.refresh_surface().expect("the first read works");
    let error = engine
        .refresh_surface()
        .expect_err("the generation is the ordering contract");
    assert_eq!(
        error,
        ContextError::SurfaceGenerationReused { generation: 7 },
    );
    assert!(engine.view().is_none());
    assert_eq!(engine.surface_status().as_str(), "failed");
}

#[test]
fn re_reading_an_unchanged_screen_churns_no_handles_and_stays_unambiguous() {
    // Re-enumerating the same screen must not mint a second `ref_N` per row.
    // If it did, the registry would fill with duplicates until a phrase that
    // resolved on the first turn came back Ambiguous on the third.
    let mut engine = interface_engine();
    engine.refresh_surface().expect("first");
    let first = engine.view().map(via_context::DeixisView::body);
    let bound = engine.referents().len();
    for _ in 0..5 {
        engine
            .refresh_surface()
            .expect("re-reading the same generation is not a contradiction");
    }
    assert_eq!(engine.view().map(via_context::DeixisView::body), first);
    assert_eq!(engine.referents().len(), bound, "no duplicate bindings");
    assert!(engine.resolve("inbox").is_found());
    assert_eq!(
        engine
            .resolve_ordinal(Ordinal::First)
            .found()
            .map(|r| r.label.clone()),
        Some("Inbox".to_owned()),
    );
}

#[test]
fn an_unchanged_row_keeps_its_handle_across_generations_and_a_moved_one_does_not() {
    let mut engine = ContextEngine::with_surface(
        SessionMode::Interface,
        Arc::new(ScriptedSurface::scripted([
            Ok(SurfaceSnapshot::new(
                1,
                vec![
                    SurfaceObject::new("row-a", "Inbox"),
                    SurfaceObject::new("row-b", "Drafts"),
                ],
            )),
            // `row-a` is untouched; `row-b` is relabelled.
            Ok(SurfaceSnapshot::new(
                2,
                vec![
                    SurfaceObject::new("row-a", "Inbox"),
                    SurfaceObject::new("row-b", "Archive"),
                ],
            )),
        ])),
    );
    engine.refresh_surface().expect("first");
    let unchanged = engine
        .resolve("inbox")
        .found()
        .map(|r| r.handle.clone())
        .expect("Inbox");
    let moved = engine
        .resolve("drafts")
        .found()
        .map(|r| r.handle.clone())
        .expect("Drafts");

    engine.refresh_surface().expect("second");
    assert_eq!(
        engine.resolve("inbox").found().map(|r| r.handle.clone()),
        Some(unchanged),
        "an unchanged row keeps the handle the model was already given",
    );
    assert!(
        matches!(
            engine.resolve_handle(&moved),
            Resolution::Stale {
                reason: Staleness::Moved,
                ..
            },
        ),
        "a relabelled row gets a new handle and the old one says why",
    );
    assert!(engine.resolve("archive").is_found());
}

// ── turns ───────────────────────────────────────────────────────────────────

#[test]
fn a_turn_boundary_expires_what_the_turn_ended() {
    let mut engine = interface_engine();
    engine.begin_turn();
    engine.refresh_surface().expect("the screen reads");
    assert!(engine.resolve("inbox").is_found());
    engine.begin_turn();
    assert!(engine.resolve("inbox").is_found(), "one turn of grace");
    engine.begin_turn();
    assert_eq!(
        engine.resolve("inbox").as_str(),
        "not_found",
        "and no more, even though the screen has not changed",
    );
}

#[test]
fn the_engine_is_debuggable_without_leaking_the_screen() {
    let engine = interface_engine();
    let rendered = format!("{engine:?}");
    assert!(rendered.contains("ContextEngine"));
    assert!(rendered.contains("Interface"));
    assert!(
        !rendered.contains("Inbox"),
        "a Debug line is not a transcript"
    );
}
