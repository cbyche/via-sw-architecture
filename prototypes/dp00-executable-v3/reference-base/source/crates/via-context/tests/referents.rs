//! The referent registry: the ladder, the lifetime, the four staleness
//! detectors, and the rule that it never guesses.

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_context::referent::{DEFAULT_RETENTION_TURNS, HANDLE_PREFIX, MAX_ALIASES, MAX_LABEL_CHARS};
use via_context::{
    ContextError, ReferentBinding, ReferentRegistry, ReferentTarget, Resolution, Staleness,
    SurfaceObject, SurfaceSnapshot,
};
use via_conversation::notes::MAX_RESULT_CANDIDATES;

fn work(id: &str, label: &str) -> ReferentBinding {
    ReferentBinding::new(
        ReferentTarget::Work {
            work_id: id.to_owned(),
        },
        label,
    )
}

fn bind(registry: &mut ReferentRegistry, id: &str, label: &str) -> String {
    registry.bind(work(id, label)).expect("a valid binding")
}

fn labels(resolution: &Resolution) -> Vec<String> {
    match resolution {
        Resolution::Ambiguous { candidates } | Resolution::NotFound { candidates } => {
            candidates.iter().map(|c| c.label.clone()).collect()
        }
        Resolution::Stale { candidates, .. } => {
            candidates.iter().map(|c| c.label.clone()).collect()
        }
        Resolution::Found(referent) => vec![referent.label.clone()],
    }
}

// ── what may be a referent ──────────────────────────────────────────────────

#[test]
fn every_target_kind_binds_and_reports_its_own_token() {
    let mut registry = ReferentRegistry::new();
    let cases = [
        (
            ReferentTarget::Screen {
                object_id: "row-1".to_owned(),
            },
            "screen",
        ),
        (
            ReferentTarget::Input {
                reference: "input_1".to_owned(),
            },
            "input",
        ),
        (
            ReferentTarget::Work {
                work_id: "w-1".to_owned(),
            },
            "work",
        ),
        (
            ReferentTarget::NoteList {
                key: "shopping".to_owned(),
            },
            "note_list",
        ),
        (
            ReferentTarget::Directory {
                path: "/srv/project".to_owned(),
            },
            "directory",
        ),
    ];
    for (target, kind) in cases {
        assert_eq!(target.kind(), kind);
        assert!(!target.identity().is_empty());
        let handle = registry
            .bind(ReferentBinding::new(target, format!("the {kind}")))
            .expect("a valid binding");
        assert!(handle.starts_with(HANDLE_PREFIX));
    }
    assert_eq!(registry.len(), 5);
}

#[rstest]
#[case("")]
#[case("   ")]
#[case("\n")]
fn a_target_with_no_identity_is_refused(#[case] id: &str) {
    let mut registry = ReferentRegistry::new();
    let error = registry
        .bind(ReferentBinding::new(
            ReferentTarget::Work {
                work_id: id.to_owned(),
            },
            "something",
        ))
        .expect_err("a referent must denote something");
    assert!(matches!(error, ContextError::UnbindableReferent { .. }));
    assert!(registry.is_empty());
}

#[test]
fn a_binding_with_no_target_at_all_is_refused() {
    let mut registry = ReferentRegistry::new();
    let error = registry
        .bind(ReferentBinding::default())
        .expect_err("a blank is not a referent");
    assert_eq!(
        error,
        ContextError::UnbindableReferent {
            reason: "no target",
        },
    );
}

#[test]
fn a_label_is_cleaned_and_bounded_and_aliases_are_capped() {
    let mut registry = ReferentRegistry::new();
    let handle = registry
        .bind(
            ReferentBinding::new(
                ReferentTarget::Work {
                    work_id: "w-1".to_owned(),
                },
                "  the\n\tfirst   one  ",
            )
            .with_aliases(["a", "", "  ", "b", "c", "d", "e", "f", "g", "h", "i"]),
        )
        .expect("a valid binding");
    let referent = registry.get(&handle).expect("bound");
    assert_eq!(referent.label, "the first one", "whitespace is collapsed");
    assert_eq!(referent.aliases.len(), MAX_ALIASES);
    assert!(!referent.aliases.iter().any(String::is_empty));

    let long = "x".repeat(MAX_LABEL_CHARS + 40);
    let handle = registry.bind(work("w-2", &long)).expect("a valid binding");
    assert_eq!(
        registry
            .get(&handle)
            .map(|referent| referent.label.chars().count()),
        Some(MAX_LABEL_CHARS),
    );
}

// ── the ladder ──────────────────────────────────────────────────────────────

#[test]
fn an_exact_label_wins_outright_over_a_substring() {
    let mut registry = ReferentRegistry::new();
    bind(&mut registry, "w-1", "report");
    bind(&mut registry, "w-2", "quarterly report");
    let resolved = registry.resolve("report");
    assert_eq!(
        resolved.found().map(|r| r.target.identity().to_owned()),
        Some("w-1".to_owned()),
    );
}

#[test]
fn matching_is_case_insensitive_the_way_speech_is() {
    let mut registry = ReferentRegistry::new();
    bind(&mut registry, "w-1", "Quarterly Report");
    assert!(registry.resolve("quarterly report").is_found());
    assert!(registry.resolve("QUARTERLY REPORT").is_found());
    assert!(registry.resolve("  quarterly   report  ").is_found());
}

#[test]
fn a_substring_match_is_bidirectional_and_only_accepted_when_unique() {
    let mut registry = ReferentRegistry::new();
    bind(&mut registry, "w-1", "quarterly report");
    // The user said less than the label.
    assert!(registry.resolve("quarterly").is_found());
    // The user said more than the label.
    assert!(registry.resolve("the quarterly report please").is_found());
}

#[test]
fn several_substring_matches_report_candidates_and_never_pick() {
    let mut registry = ReferentRegistry::new();
    bind(&mut registry, "w-1", "quarterly report");
    bind(&mut registry, "w-2", "annual report");
    let resolved = registry.resolve("report");
    assert_eq!(resolved.as_str(), "ambiguous");
    assert_eq!(
        labels(&resolved),
        vec!["quarterly report".to_owned(), "annual report".to_owned()],
    );
    assert!(resolved.found().is_none());
}

#[test]
fn several_exact_matches_skip_the_substring_pass_and_report_each_other() {
    // `via-conversation`'s asymmetry (`frontend-notes.mjs:72-83`), for its
    // reason: acting on the wrong one of two identically-named things is
    // unrecoverable.
    let mut registry = ReferentRegistry::new();
    bind(&mut registry, "w-1", "report");
    bind(&mut registry, "w-2", "report");
    bind(&mut registry, "w-3", "report addendum");
    let resolved = registry.resolve("report");
    assert_eq!(resolved.as_str(), "ambiguous");
    assert_eq!(
        labels(&resolved),
        vec!["report".to_owned(), "report".to_owned()],
        "only the exact matches, not the substring one",
    );
}

#[test]
fn an_alias_resolves_through_the_same_ladder_as_a_label() {
    let mut registry = ReferentRegistry::new();
    registry
        .bind(
            ReferentBinding::new(
                ReferentTarget::NoteList {
                    key: "shopping".to_owned(),
                },
                "购物清单",
            )
            .with_aliases(["shopping list", "groceries"]),
        )
        .expect("a valid binding");
    assert!(registry.resolve("groceries").is_found());
    assert!(registry.resolve("SHOPPING LIST").is_found());
    assert!(
        registry.resolve("购物").is_found(),
        "bidirectional substring"
    );
}

#[test]
fn nothing_matching_reports_what_there_is_rather_than_an_empty_shrug() {
    let mut registry = ReferentRegistry::new();
    bind(&mut registry, "w-1", "report");
    bind(&mut registry, "w-2", "invoice");
    let resolved = registry.resolve("spreadsheet");
    assert_eq!(resolved.as_str(), "not_found");
    assert_eq!(
        labels(&resolved),
        vec!["report".to_owned(), "invoice".to_owned()],
    );
}

#[rstest]
#[case("")]
#[case("   ")]
#[case("\n\t")]
fn an_empty_phrase_resolves_to_nothing_and_does_not_match_a_blank_label(#[case] phrase: &str) {
    let mut registry = ReferentRegistry::new();
    registry
        .bind(ReferentBinding::new(
            ReferentTarget::Screen {
                object_id: "row-1".to_owned(),
            },
            "",
        ))
        .expect("an unlabelled object is still an object");
    assert_eq!(registry.resolve(phrase).as_str(), "not_found");
}

#[test]
fn candidates_are_capped_at_the_shared_limit() {
    let mut registry = ReferentRegistry::new();
    for index in 0..(MAX_RESULT_CANDIDATES + 5) {
        bind(
            &mut registry,
            &format!("w-{index}"),
            &format!("item {index}"),
        );
    }
    let resolved = registry.resolve("nothing like this");
    assert_eq!(labels(&resolved).len(), MAX_RESULT_CANDIDATES);
}

// ── handles ─────────────────────────────────────────────────────────────────

#[test]
fn a_handle_resolves_directly_and_an_unknown_one_reports_candidates() {
    let mut registry = ReferentRegistry::new();
    let handle = bind(&mut registry, "w-1", "report");
    assert_eq!(
        registry.resolve(&handle).found().map(|r| r.handle.clone()),
        Some(handle.clone()),
    );
    assert_eq!(registry.resolve_handle(&handle).as_str(), "found");
    assert_eq!(registry.resolve_handle("ref_999").as_str(), "not_found");
    assert_eq!(registry.resolve("ref_999").as_str(), "not_found");
}

#[test]
fn a_label_that_merely_starts_like_a_handle_is_still_a_label() {
    // `ref_docs` is a plausible host label. Routing it to a handle lookup — on
    // the prefix alone — would turn a resolvable label into a guaranteed miss.
    let mut registry = ReferentRegistry::new();
    let handle = bind(&mut registry, "w-1", "ref_docs");
    assert_eq!(
        registry
            .resolve("ref_docs")
            .found()
            .map(|r| r.handle.clone()),
        Some(handle),
    );
    // A real handle shape still routes to the handle lookup, and an unknown one
    // misses rather than falling through to a substring match on the label.
    assert_eq!(registry.resolve("ref_404").as_str(), "not_found");
    // A bare prefix is not a handle shape, so it stays on the label ladder —
    // where it is an ordinary unique substring of `ref_docs`.
    assert!(registry.resolve("ref_").is_found());
}

#[test]
fn a_handle_is_never_reused_even_after_eviction() {
    let mut registry = ReferentRegistry::with_limits(8, 2).expect("valid limits");
    let first = bind(&mut registry, "w-1", "one");
    let second = bind(&mut registry, "w-2", "two");
    let third = bind(&mut registry, "w-3", "three");
    assert_eq!(registry.len(), 2, "the oldest was evicted");
    assert!(registry.get(&first).is_none());
    assert_ne!(third, first);
    assert_ne!(third, second);
    // The evicted handle answers "gone", never "here is a different object".
    assert_eq!(registry.resolve_handle(&first).as_str(), "not_found");
}

#[rstest]
#[case(0, 1)]
#[case(1, 0)]
fn a_zero_limit_is_refused(#[case] retention: u64, #[case] capacity: usize) {
    assert!(matches!(
        ReferentRegistry::with_limits(retention, capacity),
        Err(ContextError::InvalidLimit { .. }),
    ));
}

// ── lifetime ────────────────────────────────────────────────────────────────

#[test]
fn a_referent_survives_the_turn_that_bound_it_and_the_next_one() {
    let mut registry = ReferentRegistry::new();
    registry.begin_turn();
    let handle = bind(&mut registry, "w-1", "report");
    assert!(registry.resolve_handle(&handle).is_found(), "binding turn");
    registry.begin_turn();
    assert!(registry.resolve_handle(&handle).is_found(), "the next turn");
    registry.begin_turn();
    assert_eq!(
        registry.resolve_handle(&handle).as_str(),
        "not_found",
        "and no further — the expired binding is dropped, not reported stale",
    );
    assert_eq!(DEFAULT_RETENTION_TURNS, 2);
}

#[test]
fn expiry_is_enforced_by_the_predicate_as_well_as_by_the_sweep() {
    // Two guards, deliberately. The sweep on `begin_turn` keeps the map small;
    // the predicate on every resolve is what actually decides. Removing either
    // one has to be caught, so both are asserted on the same binding.
    let mut registry = ReferentRegistry::with_limits(1, 8).expect("valid limits");
    let handle = bind(&mut registry, "w-1", "report");
    let referent = registry.get(&handle).expect("bound").clone();
    registry.begin_turn();
    assert!(registry.is_empty(), "the sweep ran");
    assert_eq!(registry.resolve_handle(&handle).as_str(), "not_found");
    assert_eq!(
        registry.staleness(&referent),
        Some(Staleness::Expired),
        "the predicate refuses it independently of the sweep",
    );
}

#[test]
fn eviction_is_oldest_first() {
    let mut registry = ReferentRegistry::with_limits(8, 3).expect("valid limits");
    for index in 0..5 {
        bind(
            &mut registry,
            &format!("w-{index}"),
            &format!("item {index}"),
        );
    }
    assert_eq!(registry.len(), 3);
    assert_eq!(
        registry
            .live()
            .map(|referent| referent.label.clone())
            .collect::<Vec<_>>(),
        vec![
            "item 2".to_owned(),
            "item 3".to_owned(),
            "item 4".to_owned()
        ],
    );
}

// ── staleness ───────────────────────────────────────────────────────────────

fn screen(generation: u64, rows: &[(&str, &str)]) -> SurfaceSnapshot {
    SurfaceSnapshot::new(
        generation,
        rows.iter()
            .map(|(id, label)| SurfaceObject::new(*id, *label))
            .collect(),
    )
}

fn bound_to_screen(registry: &mut ReferentRegistry, snapshot: &SurfaceSnapshot) -> Vec<String> {
    via_context::DeixisView::build(registry, snapshot)
        .expect("the view builds")
        .entries()
        .iter()
        .map(|entry| entry.handle.clone())
        .collect()
}

#[test]
fn a_screen_referent_is_fresh_while_the_object_is_unchanged() {
    let mut registry = ReferentRegistry::new();
    let first = screen(1, &[("a", "Inbox"), ("b", "Drafts")]);
    let handles = bound_to_screen(&mut registry, &first);
    // A new generation that leaves the objects where they were is not a change.
    let same = screen(2, &[("a", "Inbox"), ("b", "Drafts")]);
    registry.observe_surface(&same).expect("a new generation");
    assert!(registry.resolve_handle(&handles[1]).is_found());
}

#[test]
fn a_reordered_object_is_moved_not_silently_renumbered() {
    let mut registry = ReferentRegistry::new();
    let first = screen(1, &[("a", "Inbox"), ("b", "Drafts")]);
    let handles = bound_to_screen(&mut registry, &first);
    let swapped = screen(2, &[("b", "Drafts"), ("a", "Inbox")]);
    registry
        .observe_surface(&swapped)
        .expect("a new generation");
    for handle in &handles {
        assert!(
            matches!(
                registry.resolve_handle(handle),
                Resolution::Stale {
                    reason: Staleness::Moved,
                    ..
                },
            ),
            "{handle} should be Moved",
        );
    }
}

#[test]
fn a_relabelled_object_is_moved_because_it_is_not_what_the_user_pointed_at() {
    let mut registry = ReferentRegistry::new();
    let handles = bound_to_screen(&mut registry, &screen(1, &[("a", "Inbox")]));
    registry
        .observe_surface(&screen(2, &[("a", "Archive")]))
        .expect("a new generation");
    assert!(matches!(
        registry.resolve_handle(&handles[0]),
        Resolution::Stale {
            reason: Staleness::Moved,
            ..
        },
    ));
}

#[test]
fn an_object_that_left_the_screen_is_gone() {
    let mut registry = ReferentRegistry::new();
    let handles = bound_to_screen(
        &mut registry,
        &screen(1, &[("a", "Inbox"), ("b", "Drafts")]),
    );
    registry
        .observe_surface(&screen(2, &[("a", "Inbox")]))
        .expect("a new generation");
    assert!(registry.resolve_handle(&handles[0]).is_found());
    assert!(matches!(
        registry.resolve_handle(&handles[1]),
        Resolution::Stale {
            reason: Staleness::Gone,
            ..
        },
    ));
}

#[test]
fn a_screen_referent_with_no_snapshot_is_refused_rather_than_trusted() {
    let mut registry = ReferentRegistry::new();
    let handles = bound_to_screen(&mut registry, &screen(1, &[("a", "Inbox")]));
    registry.forget_surface();
    assert!(matches!(
        registry.resolve_handle(&handles[0]),
        Resolution::Stale {
            reason: Staleness::SurfaceUnknown,
            ..
        },
    ));
    assert_eq!(registry.surface_generation(), None);
}

#[test]
fn a_non_screen_referent_is_unaffected_by_the_screen() {
    let mut registry = ReferentRegistry::new();
    let handle = bind(&mut registry, "w-1", "the report");
    registry.forget_surface();
    assert!(registry.resolve_handle(&handle).is_found());
}

#[test]
fn a_stale_match_is_reported_stale_rather_than_missing() {
    let mut registry = ReferentRegistry::new();
    bound_to_screen(&mut registry, &screen(1, &[("a", "Inbox")]));
    registry
        .observe_surface(&screen(2, &[("z", "Something else")]))
        .expect("a new generation");
    let resolved = registry.resolve("inbox");
    let Resolution::Stale { reason, .. } = &resolved else {
        panic!("expected Stale, got {}", resolved.as_str());
    };
    assert_eq!(*reason, Staleness::Gone);
}

#[test]
fn a_live_match_beats_a_stale_one_and_stale_candidates_are_never_offered() {
    let mut registry = ReferentRegistry::new();
    bound_to_screen(&mut registry, &screen(1, &[("a", "Inbox")]));
    // The screen moves on; the old `Inbox` referent is now Gone.
    registry
        .observe_surface(&screen(2, &[("z", "Other")]))
        .expect("a new generation");
    // A live, non-screen referent with the same label.
    let live = bind(&mut registry, "w-1", "Inbox");
    assert_eq!(
        registry.resolve("inbox").found().map(|r| r.handle.clone()),
        Some(live),
        "the live one wins outright",
    );
    // And the candidate list a miss reports contains only live referents.
    let resolved = registry.resolve("nothing at all");
    assert_eq!(labels(&resolved), vec!["Inbox".to_owned()]);
}

#[test]
fn a_stale_report_still_offers_what_is_live_instead() {
    let mut registry = ReferentRegistry::new();
    bound_to_screen(&mut registry, &screen(1, &[("a", "Inbox")]));
    registry
        .observe_surface(&screen(2, &[("z", "Other")]))
        .expect("a new generation");
    bind(&mut registry, "w-1", "the report");
    let resolved = registry.resolve("inbox");
    assert_eq!(resolved.as_str(), "stale");
    assert_eq!(labels(&resolved), vec!["the report".to_owned()]);
}

#[test]
fn every_staleness_reason_has_a_stable_token() {
    assert_eq!(
        Staleness::ALL.map(Staleness::as_str),
        ["expired", "moved", "gone", "surface_unknown"],
    );
}

// ── the source contradicting itself ─────────────────────────────────────────

#[test]
fn a_reused_generation_with_different_objects_is_reported_not_absorbed() {
    let mut registry = ReferentRegistry::new();
    registry
        .observe_surface(&screen(7, &[("a", "Inbox")]))
        .expect("the first observation");
    let error = registry
        .observe_surface(&screen(7, &[("b", "Drafts")]))
        .expect_err("the generation is the ordering contract");
    assert_eq!(
        error,
        ContextError::SurfaceGenerationReused { generation: 7 }
    );
    // And the registry kept the version it could vouch for.
    assert_eq!(registry.surface_generation(), Some(7));
}

#[test]
fn a_repeated_identical_snapshot_is_fine() {
    let mut registry = ReferentRegistry::new();
    let snapshot = screen(7, &[("a", "Inbox")]);
    registry.observe_surface(&snapshot).expect("first");
    registry
        .observe_surface(&snapshot)
        .expect("an idempotent re-observation is not a contradiction");
}
