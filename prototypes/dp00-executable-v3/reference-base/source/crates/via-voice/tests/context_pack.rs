//! The proof that `via-context`'s [`ContextPack`] is a *thin* layer.
//!
//! `docs/architecture.md` §5: *"ContextPack is a thin typed layer, not a
//! rewrite … Rewriting prompt assembly would be re-deriving a working
//! implementation."*
//!
//! A paragraph cannot establish that. An equality can:
//! `pack.instructions()` must be byte-for-byte what
//! [`assemble_frontend_instructions`] produces for the same inputs. If someone
//! ever "improves" the pack's join, its ordering, or its trimming, this file
//! fails — in `via-voice`'s own suite, next to the function being protected.
//!
//! # Why here
//!
//! `docs/architecture.md` §9 gives Layer 1 the row `voice → conversation, core,
//! shared, task, voice`, so `via-voice` (Layer 1) may depend on `via-context`
//! (Layer 2) and `via-context` may not depend back. This is the only crate that
//! can see both sides of the equality, so it is where the equality lives. The
//! dependency is `[dev-dependencies]` only: nothing that ships is added.

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_context::{
    ContextPack, FrontendPackInputs, Provenance, SectionId, Tier, Trust, frontend_pack,
};
use via_conversation::sync::{InputReference, Message, MessageRole, MessageSource};
use via_conversation::{
    ClientContext, MemoryDocument, RawClientContext, build_frontend_context,
    normalize_client_context,
};
use via_i18n::Locale;
use via_voice::prompt::{
    ASSISTANT_PROFILE_CLOSE_TAG, ASSISTANT_PROFILE_HEADING, ASSISTANT_PROFILE_OPEN_TAG,
    assemble_frontend_instructions, assistant_profile_block, packaged_assistant_profile,
    packaged_prompt,
};

fn client(time_zone: &str, locale: &str, working_directory: &str) -> ClientContext {
    normalize_client_context(&RawClientContext {
        time_zone: time_zone.to_owned(),
        locale: locale.to_owned(),
        working_directory: working_directory.to_owned(),
    })
}

fn document(scope: &str, content: &str, revision: &str) -> MemoryDocument {
    MemoryDocument {
        id: format!("{scope}_document"),
        scope: scope.to_owned(),
        content: content.to_owned(),
        format: "markdown".to_owned(),
        revision: revision.to_owned(),
        editable: true,
    }
}

fn message(role: MessageRole, content: &str, inputs: Vec<InputReference>) -> Message {
    Message {
        seq: 1,
        id: "id".to_owned(),
        role,
        content: content.to_owned(),
        source: MessageSource::VoiceUser,
        turn_id: None,
        task_id: None,
        task_ids: Vec::new(),
        inputs,
        created_at: 0,
    }
}

/// Build both halves from one set of inputs.
fn both(
    locale: Locale,
    profile: &str,
    client: &ClientContext,
    memories: &[MemoryDocument],
) -> (String, ContextPack) {
    let prompt = packaged_prompt(locale);
    let voice =
        assemble_frontend_instructions(prompt, profile, &build_frontend_context(client, memories));
    let pack = frontend_pack(
        &FrontendPackInputs::new(locale, prompt, &assistant_profile_block(profile), client)
            .with_memories(memories),
    )
    .expect("the frontend pack builds");
    (voice, pack)
}

// ── the equality ────────────────────────────────────────────────────────────

#[rstest]
#[case(Locale::En)]
#[case(Locale::Zh)]
#[case(Locale::Ko)]
fn the_packs_instructions_are_byte_for_byte_what_via_voice_assembles(#[case] locale: Locale) {
    let client = client("Asia/Shanghai", "zh-CN", "/srv/project");
    let memories = [
        document("user", "- 称呼：老大", "abc123"),
        document("memory", "用户住在上海", "def456"),
    ];
    let (voice, pack) = both(
        locale,
        packaged_assistant_profile(locale),
        &client,
        &memories,
    );
    assert_eq!(pack.instructions(), voice);
}

#[rstest]
// Every combination of the two optional blocks, plus the optional
// `client_working_directory` line, because each one changes where a separator
// falls and a join that only agrees in the full case agrees by accident.
#[case(false, false, "")]
#[case(true, false, "")]
#[case(false, true, "")]
#[case(true, true, "")]
#[case(false, false, "/srv/project")]
#[case(true, true, "/srv/project")]
fn the_equality_holds_for_every_shape_of_the_optional_blocks(
    #[case] preferences: bool,
    #[case] memory: bool,
    #[case] working_directory: &str,
) {
    let client = client("Europe/Berlin", "de-DE", working_directory);
    let mut memories = Vec::new();
    if preferences {
        memories.push(document("user", "- call me boss", "r1"));
    }
    if memory {
        memories.push(document("memory", "lives in Berlin", ""));
    }
    let (voice, pack) = both(Locale::En, "PERSONA", &client, &memories);
    assert_eq!(pack.instructions(), voice);
    assert_eq!(
        pack.section(SectionId::USER_PREFERENCES).is_some(),
        preferences,
    );
    assert_eq!(pack.section(SectionId::USER_MEMORY).is_some(), memory);
}

#[rstest]
#[case("  PERSONA with leading and trailing space  ")]
#[case("PERSONA\n\nwith a blank line inside")]
#[case("PERSONA ending in a newline\n")]
fn the_equality_survives_the_whitespace_the_two_crates_trim(#[case] profile: &str) {
    let client = client("UTC", "en-GB", "");
    let (voice, pack) = both(Locale::En, profile, &client, &[]);
    assert_eq!(pack.instructions(), voice);
}

#[test]
fn an_empty_context_still_agrees() {
    // `via-voice` drops an empty context block entirely; the pack drops an
    // empty *section*. The two must produce the same string, not merely
    // similar ones.
    let empty_client = ClientContext {
        time_zone: "UTC".to_owned(),
        locale: "en".to_owned(),
        working_directory: None,
    };
    let (voice, pack) = both(Locale::En, "PERSONA", &empty_client, &[]);
    assert_eq!(pack.instructions(), voice);
    assert!(voice.ends_with("</runtime_context>"));
}

// ── the persona block is `via-voice`'s, wrapped, not rebuilt ────────────────

#[test]
fn the_pack_carries_via_voices_persona_wrapper_verbatim() {
    let client = client("UTC", "en-GB", "");
    let (_, pack) = both(Locale::En, "PERSONA", &client, &[]);
    let section = pack
        .section(SectionId::ASSISTANT_PROFILE)
        .expect("the persona section");
    assert_eq!(section.body, assistant_profile_block("PERSONA"));
    assert!(section.body.starts_with(ASSISTANT_PROFILE_HEADING));
    assert!(section.body.contains(ASSISTANT_PROFILE_OPEN_TAG));
    assert!(section.body.ends_with(ASSISTANT_PROFILE_CLOSE_TAG));
    // …and the pack classifies it without changing it.
    assert_eq!(section.source, Provenance::System);
    assert_eq!(section.trust, Trust::Persona);
    assert_eq!(section.tier, Tier::Static);
    assert!(!section.is_fenced());
}

#[test]
fn the_fences_tag_neutralisation_covers_the_tags_via_voice_actually_emits() {
    // `via-context` cannot import these constants (Layer 2 may not depend on
    // Layer 1), so this is the drift guard: whatever `via-voice` wraps the
    // persona in must be something the fence would escape inside an untrusted
    // body.
    for tag in [ASSISTANT_PROFILE_OPEN_TAG, ASSISTANT_PROFILE_CLOSE_TAG] {
        let neutralised = via_context::neutralize(tag);
        assert_ne!(neutralised, tag, "{tag} survived neutralisation");
        assert!(neutralised.starts_with("&lt;"), "{neutralised}");
    }
    // And the same for the per-turn block `via-voice` renders.
    assert!(via_context::neutralize("<input_parts>").starts_with("&lt;"));
}

// ── the per-turn blocks are in the pack, and out of the instructions ────────

#[test]
fn the_replay_and_the_attachments_are_dynamic_and_never_reach_the_instructions() {
    let client = client("UTC", "en-GB", "");
    let recent = [
        message(
            MessageRole::User,
            "look at this",
            vec![InputReference {
                reference: "input_1".to_owned(),
                kind: "image".to_owned(),
                label: "[Image 1]".to_owned(),
                filename: "shot.png".to_owned(),
                mime: "image/png".to_owned(),
            }],
        ),
        message(MessageRole::Assistant, "I see a screenshot.", Vec::new()),
    ];
    let parts = "<input_parts>\n[{\"id\":\"input_1\"}]\n</input_parts>";
    let pack = frontend_pack(
        &FrontendPackInputs::new(
            Locale::En,
            packaged_prompt(Locale::En),
            &assistant_profile_block("PERSONA"),
            &client,
        )
        .with_recent(&recent)
        .with_input_parts(parts),
    )
    .expect("the pack builds");

    // The instructions are exactly what `via-voice` would set once…
    let voice = assemble_frontend_instructions(
        packaged_prompt(Locale::En),
        "PERSONA",
        &build_frontend_context(&client, &[]),
    );
    assert_eq!(pack.instructions(), voice);
    // `PROMPT.md` names both tags as prose — that is the instruction hierarchy
    // talking about them — so the property is that no *content* of either block
    // is in the instructions, not that the tag names are absent.
    assert!(!pack.instructions().contains("input_1"));
    assert!(!pack.instructions().contains("look at this"));
    assert!(!pack.instructions().contains("I see a screenshot."));

    // …and the per-turn material is still in the pack, tiered as per-turn.
    for id in [SectionId::RECENT_CONVERSATION, SectionId::INPUT_PARTS] {
        let section = pack
            .section(id)
            .unwrap_or_else(|| panic!("{id} is missing"));
        assert_eq!(section.tier, Tier::Dynamic, "{id}");
    }
    assert_eq!(
        pack.section(SectionId::INPUT_PARTS)
            .map(|s| s.body.as_str()),
        Some(parts)
    );
    assert!(pack.render().contains("input_1"));
}

#[test]
fn the_pack_costs_more_than_the_instructions_it_contains() {
    let client = client("UTC", "en-GB", "");
    let recent = [message(MessageRole::User, "a spoken turn", Vec::new())];
    let pack = frontend_pack(
        &FrontendPackInputs::new(
            Locale::En,
            packaged_prompt(Locale::En),
            &assistant_profile_block("PERSONA"),
            &client,
        )
        .with_recent(&recent),
    )
    .expect("the pack builds");
    assert!(pack.tokens_in(Tier::Static) > 0);
    assert!(pack.tokens_in(Tier::Dynamic) > 0);
    assert_eq!(
        pack.tokens(),
        pack.tokens_in(Tier::Static)
            + pack.tokens_in(Tier::SemiStatic)
            + pack.tokens_in(Tier::Dynamic),
    );
}
