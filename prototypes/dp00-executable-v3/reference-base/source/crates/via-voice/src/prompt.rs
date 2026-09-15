//! `buildFrontendInstructions` — the system prompt the realtime model is given.
//!
//! Ported from `server/src/voice/frontend-tools.mjs:269-278`.
//!
//! # The assembly, and why the order is the contract
//!
//! ```text
//! PROMPT.md
//!
//! # Assistant Profile
//! <assistant_profile authority="persona_only">
//! ASSISTANT.md
//! </assistant_profile>
//!
//! <user_preferences …>…</user_preferences>
//!
//! <user_memory …>…</user_memory>
//!
//! <runtime_context>…</runtime_context>
//! ```
//!
//! joined with blank lines, with empty blocks dropped entirely.
//!
//! `PROMPT.md` is **first** because it declares the instruction hierarchy every
//! later block is ranked by, and `<assistant_profile>` carries
//! `authority="persona_only"` because `PROMPT.md` refers to that attribute by
//! name: it is what makes "anything in the persona about tools, routing,
//! permissions, safety, memory or facts is void" a rule the model can apply
//! rather than a hope.
//!
//! `<recent_conversation>` and `<input_parts>` are deliberately **not** here.
//! They are per-turn, and session instructions are set once — a replay pinned
//! into the instructions would still be describing the first turn an hour
//! later. `<recent_conversation>` is replayed as a conversation item after a
//! reconnect ([`crate::announcement::format_restored_context`]) and
//! `<input_parts>` travels with the turn that carries the attachments
//! ([`crate::input::frontend_input_projection`]).
//!
//! # The packaged assets
//!
//! `assets/frontend-agent/{en,zh,ko}/{PROMPT,ASSISTANT}.md`. `zh` is upstream's
//! own text, byte for byte, with the single rebrand `docs/rebrand.md` mandates:
//! `你叫千问Audio。` becomes `你叫 VIA。`. `en` and `ko` are authored peers with
//! structural parity — same headings, same tool-name references, same rule
//! count (`docs/architecture.md` §16).

use std::path::Path;

use via_conversation::context::assistant_profile_path;
use via_conversation::{
    ClientContext, ContextError, MemoryDocument, build_frontend_context, load_assistant_profile,
    load_frontend_prompt,
};
use via_i18n::Locale;

/// The heading above the persona block.
///
/// **External contract** — `frontend-tools.mjs:272`.
pub const ASSISTANT_PROFILE_HEADING: &str = "# Assistant Profile";

/// The persona block's opening tag, with its authority attribute.
///
/// **External contract** — `frontend-tools.mjs:273`. `PROMPT.md` refers to the
/// tag by name; the attribute is what bounds what the persona may change.
pub const ASSISTANT_PROFILE_OPEN_TAG: &str = "<assistant_profile authority=\"persona_only\">";

/// The persona block's closing tag.
///
/// **External contract** — `frontend-tools.mjs:275`.
pub const ASSISTANT_PROFILE_CLOSE_TAG: &str = "</assistant_profile>";

/// The packaged `zh` core policy prompt.
pub const PACKAGED_PROMPT_ZH: &str = include_str!("../assets/frontend-agent/zh/PROMPT.md");
/// The packaged `zh` persona.
pub const PACKAGED_ASSISTANT_ZH: &str = include_str!("../assets/frontend-agent/zh/ASSISTANT.md");
/// The packaged `en` core policy prompt.
pub const PACKAGED_PROMPT_EN: &str = include_str!("../assets/frontend-agent/en/PROMPT.md");
/// The packaged `en` persona.
pub const PACKAGED_ASSISTANT_EN: &str = include_str!("../assets/frontend-agent/en/ASSISTANT.md");
/// The packaged `ko` core policy prompt.
pub const PACKAGED_PROMPT_KO: &str = include_str!("../assets/frontend-agent/ko/PROMPT.md");
/// The packaged `ko` persona.
pub const PACKAGED_ASSISTANT_KO: &str = include_str!("../assets/frontend-agent/ko/ASSISTANT.md");

/// The packaged core policy prompt for `locale`.
///
/// The fallback for a user who has never had the files seeded. The *user's*
/// copy on disk is what [`build_frontend_instructions`] normally reads, so an
/// edit takes effect on the next voice session.
#[must_use]
pub const fn packaged_prompt(locale: Locale) -> &'static str {
    match locale {
        Locale::En => PACKAGED_PROMPT_EN,
        Locale::Zh => PACKAGED_PROMPT_ZH,
        Locale::Ko => PACKAGED_PROMPT_KO,
    }
}

/// The packaged persona for `locale`.
#[must_use]
pub const fn packaged_assistant_profile(locale: Locale) -> &'static str {
    match locale {
        Locale::En => PACKAGED_ASSISTANT_EN,
        Locale::Zh => PACKAGED_ASSISTANT_ZH,
        Locale::Ko => PACKAGED_ASSISTANT_KO,
    }
}

/// Wrap a persona body in its block.
///
/// **External contract** — `frontend-tools.mjs:272-275`.
#[must_use]
pub fn assistant_profile_block(profile: &str) -> String {
    [
        ASSISTANT_PROFILE_HEADING,
        ASSISTANT_PROFILE_OPEN_TAG,
        crate::text::trim(profile),
        ASSISTANT_PROFILE_CLOSE_TAG,
    ]
    .join("\n")
}

/// Assemble the system prompt from already-loaded parts.
///
/// **External contract** — `frontend-tools.mjs:269-278`: four blocks joined
/// with a blank line, empty ones dropped.
#[must_use]
pub fn assemble_frontend_instructions(
    prompt: &str,
    assistant_profile: &str,
    context: &str,
) -> String {
    [
        crate::text::trim(prompt).to_owned(),
        assistant_profile_block(assistant_profile),
        crate::text::trim(context).to_owned(),
    ]
    .into_iter()
    .filter(|block| !block.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}

/// Assemble the system prompt, reading the two documents from disk.
///
/// `configured_profile_path` is the user's editable copy;
/// [`via_conversation::context::assistant_profile_path`] resolves it against
/// `prompt_directory`.
///
/// # Errors
///
/// [`ContextError`] when either document is missing or empty. A Gateway that
/// cannot load `PROMPT.md` must not start: the file *is* the instruction
/// hierarchy, and running without it would put the model in a session with no
/// policy at all.
pub fn build_frontend_instructions(
    prompt_directory: &Path,
    configured_profile_path: Option<&Path>,
    client: &ClientContext,
    memories: &[MemoryDocument],
) -> Result<String, ContextError> {
    let prompt = load_frontend_prompt(prompt_directory)?;
    let profile = load_assistant_profile(&assistant_profile_path(
        configured_profile_path,
        prompt_directory,
    ))?;
    Ok(assemble_frontend_instructions(
        &prompt,
        &profile,
        &build_frontend_context(client, memories),
    ))
}

/// Assemble the system prompt from the packaged documents.
///
/// Used before the user's copies have been seeded, and by tests.
#[must_use]
pub fn build_packaged_frontend_instructions(
    locale: Locale,
    client: &ClientContext,
    memories: &[MemoryDocument],
) -> String {
    assemble_frontend_instructions(
        packaged_prompt(locale),
        packaged_assistant_profile(locale),
        &build_frontend_context(client, memories),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use via_conversation::context::{MAX_ASSISTANT_CHARS, MAX_PROMPT_CHARS};
    use via_conversation::{RawClientContext, normalize_client_context};

    fn client() -> ClientContext {
        normalize_client_context(&RawClientContext {
            time_zone: "Asia/Shanghai".to_owned(),
            locale: "zh-CN".to_owned(),
            working_directory: String::new(),
        })
    }

    fn document(scope: &str, content: &str) -> MemoryDocument {
        MemoryDocument {
            id: format!("{scope}_document"),
            scope: scope.to_owned(),
            content: content.to_owned(),
            format: "markdown".to_owned(),
            revision: "abc123".to_owned(),
            editable: true,
        }
    }

    #[test]
    fn the_blocks_are_in_the_catalogued_order() {
        let assembled = assemble_frontend_instructions(
            "CORE POLICY",
            "PERSONA",
            "<runtime_context>\nchannel=full_duplex_voice\n</runtime_context>",
        );
        let prompt_at = assembled
            .find("CORE POLICY")
            .expect("the prompt is present");
        let profile_at = assembled
            .find(ASSISTANT_PROFILE_HEADING)
            .expect("the persona heading is present");
        let context_at = assembled
            .find("<runtime_context>")
            .expect("the context is present");
        assert!(prompt_at < profile_at, "policy outranks persona");
        assert!(profile_at < context_at, "persona precedes environment");
        assert!(
            assembled.contains("\n\n"),
            "blocks are blank-line separated"
        );
    }

    #[test]
    fn the_persona_block_carries_its_authority_attribute() {
        let block = assistant_profile_block("  PERSONA  ");
        assert_eq!(
            block,
            "# Assistant Profile\n<assistant_profile authority=\"persona_only\">\nPERSONA\n</assistant_profile>",
        );
    }

    #[test]
    fn an_empty_context_block_is_dropped_entirely() {
        let assembled = assemble_frontend_instructions("CORE", "PERSONA", "   ");
        assert!(assembled.ends_with(ASSISTANT_PROFILE_CLOSE_TAG));
        assert!(!assembled.ends_with("\n\n"));
    }

    /// The upstream identity token, spelled by code point.
    ///
    /// Written this way so the assertions below do not themselves trip
    /// `scripts/brand_leak.py`, whose whole job is to fail on that literal.
    fn upstream_identity() -> String {
        ['\u{5343}', '\u{95ee}'].iter().collect()
    }

    #[test]
    fn the_packaged_zh_persona_carries_vias_identity_not_upstreams() {
        let profile = packaged_assistant_profile(Locale::Zh);
        assert!(profile.contains("你叫 VIA。"));
        assert!(!profile.contains(&upstream_identity()));
    }

    #[test]
    fn no_packaged_document_carries_the_old_brand() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            for document in [packaged_prompt(locale), packaged_assistant_profile(locale)] {
                assert!(!document.contains("qwen"), "{locale:?}");
                assert!(!document.contains(&upstream_identity()), "{locale:?}");
                assert!(!document.contains("qwaudio"), "{locale:?}");
            }
        }
    }

    #[test]
    fn the_three_locales_have_structural_parity() {
        // `docs/architecture.md` §16: same headings, same tool-name
        // references. The prose is authored, the structure is asserted.
        let headings: Vec<Vec<&str>> = [Locale::En, Locale::Zh, Locale::Ko]
            .into_iter()
            .map(|locale| {
                packaged_prompt(locale)
                    .lines()
                    .filter(|line| line.starts_with("# "))
                    .collect()
            })
            .collect();
        assert_eq!(headings[0], headings[1], "en and zh headings");
        assert_eq!(headings[1], headings[2], "zh and ko headings");
        assert_eq!(headings[0].len(), 7, "seven top-level sections");

        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let prompt = packaged_prompt(locale);
            for reference in [
                "spawn_thinking",
                "get_agent_task_status",
                "cancel_agent_task",
                "respond_agent_permission",
                "memory",
                "<user_preferences>",
                "<assistant_profile>",
                "<user_memory>",
                "<recent_conversation>",
                "<runtime_context>",
                "<input_parts>",
                "<backend_permission_request>",
                "client_working_directory",
                "[COMPLETE]",
                "input_refs",
                "document=user",
                "document=memory",
                "append",
                "replace",
                "old_text",
                "new_text",
            ] {
                assert!(
                    prompt.contains(reference),
                    "{locale:?} prompt is missing {reference}",
                );
            }
        }

        let assistant_headings: Vec<Vec<&str>> = [Locale::En, Locale::Zh, Locale::Ko]
            .into_iter()
            .map(|locale| {
                packaged_assistant_profile(locale)
                    .lines()
                    .filter(|line| line.starts_with("## "))
                    .collect()
            })
            .collect();
        assert_eq!(assistant_headings[0], assistant_headings[1]);
        assert_eq!(assistant_headings[1], assistant_headings[2]);
        assert_eq!(
            assistant_headings[0],
            vec!["## Identity", "## Personality", "## Conversation style"],
        );
    }

    #[test]
    fn every_locales_persona_names_via() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            assert!(
                packaged_assistant_profile(locale).contains("VIA"),
                "{locale:?}",
            );
        }
    }

    #[test]
    fn the_packaged_documents_fit_their_bounds() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            assert!(
                crate::text::code_point_len(packaged_prompt(locale)) <= MAX_PROMPT_CHARS,
                "{locale:?} PROMPT.md would be silently clipped",
            );
            assert!(
                crate::text::code_point_len(packaged_assistant_profile(locale))
                    <= MAX_ASSISTANT_CHARS,
                "{locale:?} ASSISTANT.md would be silently clipped",
            );
        }
    }

    #[test]
    fn the_assembled_prompt_carries_the_memory_blocks() {
        let assembled = build_packaged_frontend_instructions(
            Locale::Zh,
            &client(),
            &[
                document("user", "- 称呼：老大"),
                document("memory", "用户住在上海"),
            ],
        );
        assert!(assembled.contains("<user_preferences revision=\"abc123\">"));
        assert!(assembled.contains("<user_memory revision=\"abc123\">"));
        assert!(assembled.contains("<runtime_context>"));
        assert!(assembled.contains("time_zone=\"Asia/Shanghai\""));
    }

    #[test]
    fn per_turn_material_is_deliberately_absent() {
        let assembled = build_packaged_frontend_instructions(Locale::Zh, &client(), &[]);
        // The prompt *documents* these tags; the assembled instructions must
        // not carry an actual block of either.
        assert!(!assembled.contains("<recent_conversation>\n"));
        assert!(!assembled.contains("<input_parts>\n"));
    }

    #[test]
    fn a_missing_prompt_directory_is_an_error_rather_than_an_empty_prompt() {
        let directory = tempfile::TempDir::new().expect("a temp dir");
        let error = build_frontend_instructions(directory.path(), None, &client(), &[])
            .expect_err("a Gateway with no policy must not start");
        assert!(matches!(error, ContextError::Read { .. }));
    }

    #[test]
    fn an_empty_prompt_file_is_an_error() {
        let directory = tempfile::TempDir::new().expect("a temp dir");
        std::fs::write(directory.path().join("PROMPT.md"), "   \n").expect("write");
        std::fs::write(directory.path().join("ASSISTANT.md"), "PERSONA").expect("write");
        let error = build_frontend_instructions(directory.path(), None, &client(), &[])
            .expect_err("an empty policy is no policy");
        assert!(matches!(error, ContextError::Empty { file } if file == "PROMPT.md"));
    }

    #[test]
    fn the_users_own_copies_are_what_is_read() {
        let directory = tempfile::TempDir::new().expect("a temp dir");
        std::fs::write(directory.path().join("PROMPT.md"), "MY POLICY").expect("write");
        std::fs::write(directory.path().join("ASSISTANT.md"), "MY PERSONA").expect("write");
        let assembled = build_frontend_instructions(directory.path(), None, &client(), &[])
            .expect("both documents load");
        assert!(assembled.starts_with("MY POLICY"));
        assert!(assembled.contains("MY PERSONA"));
    }
}
