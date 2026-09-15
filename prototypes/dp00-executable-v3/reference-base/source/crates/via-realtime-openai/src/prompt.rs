//! The model-visible text this provider composes.
//!
//! Four catalogued `prompt-text` contracts, and every one of them is a
//! [`via_i18n`] key rather than a literal:
//!
//! | What | Contract | Key |
//! | --- | --- | --- |
//! | speak-aloud instructions | `speakResponseInstructions(content)` | [`keys::VOICE_INSTRUCTIONS_SPEAK_RESPONSE`] |
//! | result-presentation instructions | `resultResponseInstructions` | [`keys::VOICE_INSTRUCTIONS_RESULT_RESPONSE`] |
//! | permission-question instructions | `permissionResponseInstructions` | [`keys::VOICE_INSTRUCTIONS_PERMISSION_RESPONSE`] |
//! | the permission item body | `<backend_permission_request>` injection | *not localized — see below* |
//!
//! # Why the accessors are here and not imported
//!
//! ARGO has none of this: its realtime provider never injects a finished result
//! or asks for an authorization, because ARGO's delegation results ride a
//! different path. These three sentences come from the **qwen** side of the
//! port, where both providers `import { … } from '../frontend-tools.mjs'`.
//!
//! In VIA that file is `via-voice`'s (`docs/architecture.md` §9 puts the
//! frontend prompt and the tool catalog there) and `via-voice` sits above the
//! provider crates, so a provider that imported it would invert the crate graph.
//! The repo's answer is already established — `via-voice/src/tools/instructions.rs`
//! and `via-realtime-dashscope/src/prompt.rs` each reach the same three keys
//! directly. **The sentence has one definition, in `via-i18n`**; what is repeated
//! is a two-line accessor, and repeating it is what keeps the graph acyclic.
//!
//! # The permission item is deliberately not localized
//!
//! `<backend_permission_request>`, `authorization_id=` and `operation=` are
//! **protocol vocabulary the prompt refers to by name**:
//! `config/frontend-agent/PROMPT.md` § Permission requests names the tag, and
//! `respond_agent_permission` consumes the id it carries. Translating the tag
//! would break the instruction hierarchy in exactly one locale, silently. The
//! *summary* inside it is composed upstream of here and arrives already in the
//! reader's language.

use via_i18n::{Locale, format, keys, t};
use via_realtime::PermissionRequest;

/// The opening tag of the permission item.
///
/// External contract — `dashscope.mjs:120`, `s2s.mjs:125`. Referenced by name in
/// `config/frontend-agent/PROMPT.md`, so it is prompt vocabulary rather than an
/// implementation detail.
pub const PERMISSION_REQUEST_OPEN_TAG: &str = "<backend_permission_request>";

/// The closing tag of the permission item.
pub const PERMISSION_REQUEST_CLOSE_TAG: &str = "</backend_permission_request>";

/// The field carrying the id the model must quote back.
///
/// External contract — `dashscope.mjs:121`. `respond_agent_permission` matches
/// on this name.
pub const PERMISSION_AUTHORIZATION_ID_FIELD: &str = "authorization_id";

/// The field carrying the one-line description of the operation.
pub const PERMISSION_OPERATION_FIELD: &str = "operation";

/// Read `content` aloud without acting on it.
///
/// External contract — `frontend-tools.mjs:258-260`. The content is interpolated
/// *inside* the instruction, which is what lets `build_speak_response` set
/// `conversation: 'none'` and still have the model say the right thing: there is
/// no conversation item to read it from.
#[must_use]
pub fn speak_response_instructions(content: &str, locale: Locale) -> String {
    format(
        locale,
        keys::VOICE_INSTRUCTIONS_SPEAK_RESPONSE,
        &[("content", content)],
    )
}

/// How a finished Work's result must be presented.
///
/// External contract — `frontend-tools.mjs:248-256`.
#[must_use]
pub fn result_response_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_RESULT_RESPONSE).to_owned()
}

/// How a backend permission question must be asked.
///
/// External contract — `frontend-tools.mjs:262-267`.
#[must_use]
pub fn permission_response_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_PERMISSION_RESPONSE).to_owned()
}

/// The body of the conversation item that asks for an authorization.
///
/// External contract — `dashscope.mjs:118-124`, identical at `s2s.mjs:123-129`:
/// four lines joined with `\n`, no trailing newline.
///
/// ```
/// use via_realtime::PermissionRequest;
/// use via_realtime_openai::permission_request_text;
///
/// let text = permission_request_text(&PermissionRequest {
///     id: "perm_1".into(),
///     summary: "run `rm -rf build`".into(),
/// });
/// assert_eq!(
///     text,
///     "<backend_permission_request>\n\
///      authorization_id=perm_1\n\
///      operation=run `rm -rf build`\n\
///      </backend_permission_request>",
/// );
/// ```
#[must_use]
pub fn permission_request_text(permission: &PermissionRequest) -> String {
    [
        PERMISSION_REQUEST_OPEN_TAG.to_owned(),
        format!("{PERMISSION_AUTHORIZATION_ID_FIELD}={}", permission.id),
        format!("{PERMISSION_OPERATION_FIELD}={}", permission.summary),
        PERMISSION_REQUEST_CLOSE_TAG.to_owned(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const LOCALES: [Locale; 3] = [Locale::En, Locale::Zh, Locale::Ko];

    #[test]
    fn the_speak_instruction_carries_the_content_in_every_locale() {
        for locale in LOCALES {
            let rendered = speak_response_instructions("三点有个会", locale);
            assert!(rendered.contains("三点有个会"), "{locale}: {rendered}");
            assert!(!rendered.contains("{content}"), "{locale}: {rendered}");
        }
    }

    #[test]
    fn the_zh_column_is_upstreams_own_text() {
        // The one assertion that proves this crate reaches the shared key rather
        // than a copy of the sentence: `via-i18n`'s `zh` column *is* upstream's
        // text, byte for byte.
        assert_eq!(
            speak_response_instructions("完成", Locale::Zh),
            "请以自然口语传达下面的信息，保持事实一致，不调用工具：\n完成"
        );
    }

    #[test]
    fn the_two_static_instructions_are_non_empty_and_unrendered_in_every_locale() {
        for locale in LOCALES {
            for rendered in [
                result_response_instructions(locale),
                permission_response_instructions(locale),
            ] {
                assert!(!rendered.is_empty(), "{locale}");
                assert!(!rendered.contains('{'), "{locale}: {rendered}");
            }
        }
    }

    #[test]
    fn the_permission_item_is_four_lines_and_never_translated() {
        let text = permission_request_text(&PermissionRequest {
            id: "perm_9".to_owned(),
            summary: "删除构建目录".to_owned(),
        });
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], PERMISSION_REQUEST_OPEN_TAG);
        assert_eq!(lines[1], "authorization_id=perm_9");
        assert_eq!(lines[2], "operation=删除构建目录");
        assert_eq!(lines[3], PERMISSION_REQUEST_CLOSE_TAG);
        assert!(!text.ends_with('\n'), "no trailing newline");
    }

    #[test]
    fn an_empty_permission_still_renders_both_fields() {
        // A blank summary is a Layer-2 bug, but the tag must still close: a
        // half-written block would be read by the model as prose.
        let text = permission_request_text(&PermissionRequest {
            id: String::new(),
            summary: String::new(),
        });
        assert!(text.contains("\nauthorization_id=\n"));
        assert!(text.ends_with(PERMISSION_REQUEST_CLOSE_TAG));
    }
}
