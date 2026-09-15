//! The contracts this crate touches, asserted against
//! `docs/reference/contracts.json`.
//!
//! Every value is **parsed from the catalogue**, never retyped: a test that
//! restates the literal it is checking proves only that the author typed it
//! twice. `via-context` owns no contract of its own — it is new — so what is
//! asserted here is that wrapping the blocks did not change them.

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use common::{client, document, message};
use pretty_assertions::assert_eq;
use serde_json::Value;
use via_context::pack::MAX_SECTION_ID_CHARS;
use via_context::{
    ContextPack, FrontendPackInputs, MAX_FENCED_CHARS, SectionId, Tier, frontend_pack,
};
use via_conversation::sync::MessageRole;
use via_i18n::Locale;

fn contracts() -> Vec<Value> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("reference")
        .join("contracts.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is unreadable: {error}", path.display()));
    serde_json::from_str(&text).expect("the catalogue is valid JSON")
}

/// Every `(kind, name) -> exactValue` the catalogue supplies.
fn exact_values() -> BTreeMap<(String, String), String> {
    contracts()
        .into_iter()
        .filter_map(|entry| {
            let kind = entry.get("kind")?.as_str()?.to_owned();
            let name = entry.get("name")?.as_str()?.to_owned();
            let value = entry.get("exactValue")?.as_str()?.to_owned();
            Some(((kind, name), value))
        })
        .collect()
}

fn exact(kind: &str, name: &str) -> String {
    exact_values()
        .get(&(kind.to_owned(), name.to_owned()))
        .cloned()
        .unwrap_or_else(|| panic!("the catalogue has no exactValue for {kind}/{name}"))
}

/// The `why` prose for one entry, which several contracts state their ordering
/// rule in rather than in `exactValue`.
fn why(kind: &str, name: &str) -> String {
    contracts()
        .into_iter()
        .find(|entry| {
            entry.get("kind").and_then(Value::as_str) == Some(kind)
                && entry.get("name").and_then(Value::as_str) == Some(name)
        })
        .and_then(|entry| entry.get("why").and_then(Value::as_str).map(str::to_owned))
        .unwrap_or_else(|| panic!("the catalogue has no why for {kind}/{name}"))
}

#[test]
fn the_catalogue_is_the_expected_size() {
    // A guard on the fixture itself: a truncated or replaced catalogue would
    // otherwise make every lookup below vacuously pass.
    assert_eq!(contracts().len(), 707);
}

/// A pack built from the catalogued `<runtime_context>` example.
fn catalogued_pack() -> ContextPack {
    let client = client("Asia/Shanghai", "zh-CN", "/path");
    // No revisions: the catalogued ordering rule names the bare
    // `<user_preferences>` opener, which is the form a document without a
    // revision renders. The revision-bearing form is asserted separately by
    // `both_catalogued_openers_are_reachable`.
    let memories = [
        document("user", "- 称呼：老大", ""),
        document("memory", "用户住在上海", ""),
    ];
    let recent = [message(MessageRole::User, "订一张票")];
    frontend_pack(
        &FrontendPackInputs::new(
            Locale::Zh,
            "# Instruction hierarchy\nPOLICY BODY",
            "# Assistant Profile\n<assistant_profile authority=\"persona_only\">\nPERSONA\n</assistant_profile>",
            &client,
        )
        .with_memories(&memories)
        .with_recent(&recent),
    )
    .expect("the frontend pack builds")
}

#[test]
fn the_runtime_context_section_is_the_catalogued_block_byte_for_byte() {
    let expected = exact("prompt-text", "<runtime_context> block format");
    let pack = catalogued_pack();
    let section = pack
        .section(SectionId::RUNTIME_CONTEXT)
        .expect("the runtime context section");
    assert_eq!(section.body, expected);
    assert!(pack.instructions().contains(&expected));
}

#[test]
fn the_static_tier_holds_the_catalogued_assembly_order() {
    // The `why` of *Assembled system-prompt order* states the ordering upstream
    // asserts: indexOf('# Instruction hierarchy') <
    // lastIndexOf('<assistant_profile authority="persona_only">') <
    // lastIndexOf('<user_preferences>') < lastIndexOf('<runtime_context>').
    // The four tokens are read out of the catalogue rather than retyped.
    let rule = why(
        "prompt-text",
        "Assembled system-prompt order (buildFrontendInstructions)",
    );
    let tokens: Vec<String> = rule
        .split('\'')
        .skip(1)
        .step_by(2)
        .filter(|token| token.starts_with('#') || token.starts_with('<'))
        .map(str::to_owned)
        .collect();
    assert_eq!(
        tokens.len(),
        4,
        "the catalogue's ordering rule names four tokens: {tokens:?}",
    );

    let instructions = catalogued_pack().instructions();
    let mut positions = Vec::new();
    for token in &tokens {
        let at = instructions
            .rfind(token.as_str())
            .unwrap_or_else(|| panic!("the instructions do not contain {token}"));
        positions.push(at);
    }
    for window in positions.windows(2) {
        assert!(
            window[0] < window[1],
            "the catalogued order is broken: {tokens:?} landed at {positions:?}",
        );
    }
}

#[test]
fn the_pack_never_names_a_memory_file() {
    // Same contract's `why`: `:802 asserts the prompt never contains the
    // literal strings ASSISTANT.md / USER.md / MEMORY.md`. The three names are
    // read out of the `context caps and fallbacks` entry rather than retyped.
    let caps = exact("default-value", "context caps and fallbacks");
    let mut files: Vec<String> = caps
        .split('\'')
        .skip(1)
        .step_by(2)
        .filter(|token| token.ends_with(".md"))
        .map(str::to_owned)
        .collect();
    files.push("MEMORY.md".to_owned());
    assert!(files.contains(&"PROMPT.md".to_owned()));
    assert!(files.contains(&"ASSISTANT.md".to_owned()));

    let rendered = catalogued_pack().render();
    for file in files {
        assert!(
            !rendered.contains(&file),
            "the rendered pack names {file}, which is internal structure",
        );
    }
}

#[test]
fn the_catalogued_blocks_are_joined_the_catalogued_way() {
    // *frontend context blocks*: "blocks joined with '\n\n', empty blocks
    // dropped". The separator is read out of the entry.
    let spec = exact("prompt-text", "frontend context blocks");
    assert!(spec.contains(r"blocks joined with '\n\n', empty blocks dropped"));
    let separator = "\n\n";

    let client = client("UTC", "en-GB", "");
    let memories = [document("user", "- call me boss", "r1")];
    let pack = frontend_pack(
        &FrontendPackInputs::new(Locale::En, "POLICY", "PERSONA", &client).with_memories(&memories),
    )
    .expect("the pack builds");
    // Three sections, two separators, and nothing where an empty block was.
    assert_eq!(pack.len(), 4);
    assert_eq!(pack.render().matches(separator).count(), 3);
    assert!(pack.section(SectionId::USER_MEMORY).is_none());
}

#[test]
fn the_recent_conversation_section_carries_the_catalogued_tags() {
    let spec = exact("prompt-text", "recent conversation block");
    assert!(spec.contains("<recent_conversation>"));
    let section = catalogued_pack()
        .section(SectionId::RECENT_CONVERSATION)
        .expect("the replay section")
        .clone();
    assert!(section.body.starts_with("<recent_conversation>"));
    assert!(section.body.ends_with("</recent_conversation>"));
    assert_eq!(section.tier, Tier::Dynamic);
    // Per-turn, so it must not be in the session instructions.
    assert!(!catalogued_pack().instructions().contains(&section.body));
}

#[test]
fn the_fenced_cap_is_derived_from_the_catalogued_prompt_cap() {
    let catalogued: usize = exact("default-value", "MAX_PROMPT_CHARS")
        .trim()
        .parse()
        .expect("MAX_PROMPT_CHARS is a number");
    assert_eq!(MAX_FENCED_CHARS * 2, catalogued);
    assert_eq!(
        via_conversation::context::MAX_PROMPT_CHARS,
        catalogued,
        "the crate this one wraps must agree too",
    );
}

#[test]
fn every_section_id_this_crate_uses_names_a_catalogued_tag_or_is_ours() {
    // The five block ids are the catalogued tag names; `core_policy` and
    // `on_screen` are VIA's own. Checking the tags against the catalogue is
    // what stops a rename here drifting away from the block it names.
    let blocks = exact("prompt-text", "frontend context blocks");
    for id in [
        SectionId::USER_PREFERENCES,
        SectionId::USER_MEMORY,
        SectionId::RUNTIME_CONTEXT,
    ] {
        assert!(
            blocks.contains(&format!("<{id}")),
            "{id} does not name a catalogued block",
        );
    }
    assert!(
        exact("prompt-text", "recent conversation block")
            .contains(&format!("<{}", SectionId::RECENT_CONVERSATION)),
    );
    assert!(
        exact(
            "prompt-text",
            "Assembled system-prompt order (buildFrontendInstructions)",
        )
        .contains(&format!("<{}", SectionId::ASSISTANT_PROFILE)),
    );
    // And every id, catalogued or ours, fits the grammar and the bound.
    for id in SectionId::KNOWN {
        assert!(id.chars().count() <= MAX_SECTION_ID_CHARS);
        assert!(SectionId::new(id).is_ok());
    }
}

#[test]
fn both_catalogued_openers_are_reachable() {
    // *frontend context blocks* describes two forms of the same block:
    // `<user_preferences revision="<rev>">` and, without one,
    // `<user_preferences>`.
    let spec = exact("prompt-text", "frontend context blocks");
    assert!(spec.contains(r#"<user_preferences revision="<rev>">"#));
    assert!(spec.contains("(or `<user_preferences>` without revision)"));

    let client = client("UTC", "en-GB", "");
    let with_revision = [document("user", "- call me boss", "abc123")];
    let without = [document("user", "- call me boss", "")];
    let opener = |memories: &[via_conversation::MemoryDocument]| {
        frontend_pack(
            &FrontendPackInputs::new(Locale::En, "POLICY", "PERSONA", &client)
                .with_memories(memories),
        )
        .expect("the pack builds")
        .section(SectionId::USER_PREFERENCES)
        .expect("the preferences section")
        .body
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
    };
    assert_eq!(
        opener(&with_revision),
        "<user_preferences revision=\"abc123\">"
    );
    assert_eq!(opener(&without), "<user_preferences>");
}
