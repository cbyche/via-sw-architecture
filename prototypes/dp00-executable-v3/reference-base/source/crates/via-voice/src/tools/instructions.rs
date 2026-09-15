//! The per-response instructions the voice layer attaches.
//!
//! Ported from `server/src/voice/frontend-tools.mjs:248-267` and the
//! `response.instructions` blocks of
//! `server/src/voice/tools/tool-call-handler.mjs`.
//!
//! Every string here is a catalogued `prompt-text` contract and is read from
//! [`via_i18n`]. They exist because the model's default behaviour after a tool
//! result is wrong in a specific way each time — it acknowledges, it repeats
//! itself, it reads out an id — and one sentence of instruction is cheaper
//! than a fine-tune.
//!
//! A provider whose
//! [`per_response_instructions`](crate::provider::ProviderCapabilities::per_response_instructions)
//! flag is unset never receives these, which is why the flag exists rather
//! than a provider-name check.

use via_i18n::{Locale, format as i18n_format, keys, t};

/// How a finished Work's result must be presented.
///
/// **External contract** — `frontend-tools.mjs:248-256`.
#[must_use]
pub fn result_response_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_RESULT_RESPONSE).to_owned()
}

/// Read `content` aloud without acting on it.
///
/// **External contract** — `frontend-tools.mjs:258-260`. The interpolation is
/// deliberate: the content is *inside* the instruction, so a provider that
/// drops conversation items still speaks the right thing.
#[must_use]
pub fn speak_response_instructions(content: &str, locale: Locale) -> String {
    i18n_format(
        locale,
        keys::VOICE_INSTRUCTIONS_SPEAK_RESPONSE,
        &[("content", content)],
    )
}

/// How a backend permission question must be asked.
///
/// **External contract** — `frontend-tools.mjs:262-267`. The *"do not
/// prescribe a specific answer"* rule is the load-bearing part: a model that
/// demands a fixed phrase makes the user repeat themselves, and
/// `respond_agent_permission` is explicitly built to judge natural agreement.
#[must_use]
pub fn permission_response_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_PERMISSION_RESPONSE).to_owned()
}

/// Confirm a scheduled reminder in one sentence.
///
/// **External contract** — `tool-call-handler.mjs:333-336`.
#[must_use]
pub fn reminder_confirmation_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_REMINDER_CONFIRMATION).to_owned()
}

/// What to do instead of delegating while a permission is pending.
///
/// **External contract** — `tool-call-handler.mjs:446-451`.
#[must_use]
pub fn permission_pending_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_PERMISSION_PENDING).to_owned()
}

/// What to say when no backend is configured.
///
/// **External contract** — `tool-call-handler.mjs:477-480`.
#[must_use]
pub fn backend_not_configured_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_BACKEND_NOT_CONFIGURED).to_owned()
}

/// What to say when the backend is configured but disconnected.
///
/// **External contract** — `tool-call-handler.mjs:499-502`. Upstream's second
/// and third sentences are the same two as the not-configured variant, and the
/// shipped [`via_i18n`] catalog keeps them only on that key — so the
/// disconnected message reuses them rather than duplicating the text. Recorded
/// in `docs/deviations/phase-5.md`.
#[must_use]
pub fn backend_disconnected_instructions(locale: Locale) -> String {
    let head = t(locale, keys::VOICE_INSTRUCTIONS_BACKEND_DISCONNECTED);
    let shared = t(locale, keys::VOICE_INSTRUCTIONS_BACKEND_NOT_CONFIGURED);
    match shared.split_once(' ') {
        Some((_, tail)) if !tail.is_empty() => format!("{head} {tail}"),
        _ => head.to_owned(),
    }
}

/// What to do after a duplicate submission.
///
/// **External contract** — `tool-call-handler.mjs:546-552`.
#[must_use]
pub fn duplicate_submission_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_DUPLICATE_SUBMISSION).to_owned()
}

/// What to do after Work was accepted.
///
/// **External contract** — `tool-call-handler.mjs:636-641`. The first two
/// sentences are shared with the duplicate case; the third and fourth are the
/// accepted-specific ones.
#[must_use]
pub fn accepted_instructions(locale: Locale) -> String {
    let duplicate = t(locale, keys::VOICE_INSTRUCTIONS_DUPLICATE_SUBMISSION);
    // Sentences two and three of the duplicate block are the shared pair.
    let shared: Vec<&str> = duplicate.split(' ').skip(1).take(2).collect();
    [
        shared.join(" "),
        t(locale, keys::VOICE_INSTRUCTIONS_ACCEPTED).to_owned(),
        t(locale, keys::VOICE_INSTRUCTIONS_ACCEPTED_NOT_FINISHED).to_owned(),
    ]
    .join(" ")
}

/// Confirm an approval in one short sentence.
///
/// **External contract** — `tool-call-handler.mjs:762-766`.
#[must_use]
pub fn permission_submitted_always_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_PERMISSION_SUBMITTED_ALWAYS).to_owned()
}

/// Confirm a refusal in one short sentence.
///
/// **External contract** — `tool-call-handler.mjs:767-771`.
#[must_use]
pub fn permission_submitted_reject_instructions(locale: Locale) -> String {
    t(locale, keys::VOICE_INSTRUCTIONS_PERMISSION_SUBMITTED_REJECT).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One named instruction builder.
    type Builder = (&'static str, fn(Locale) -> String);

    #[test]
    fn the_speak_instruction_carries_the_content_it_is_about() {
        let instructions = speak_response_instructions("三点有个会", Locale::Zh);
        assert!(instructions.contains("三点有个会"));
        assert!(
            instructions.starts_with(
                t(Locale::Zh, keys::VOICE_INSTRUCTIONS_SPEAK_RESPONSE)
                    .split("{content}")
                    .next()
                    .unwrap_or_default()
            )
        );
    }

    #[test]
    fn the_disconnected_instruction_keeps_the_shared_tail() {
        let disconnected = backend_disconnected_instructions(Locale::Zh);
        let not_configured = backend_not_configured_instructions(Locale::Zh);
        assert!(
            disconnected.starts_with(t(Locale::Zh, keys::VOICE_INSTRUCTIONS_BACKEND_DISCONNECTED))
        );
        // Both end with the same "do not claim the task was created" pair.
        let shared_tail = not_configured
            .split_once(' ')
            .map(|(_, tail)| tail)
            .unwrap_or_default();
        assert!(!shared_tail.is_empty());
        assert!(disconnected.ends_with(shared_tail));
        assert_ne!(disconnected, not_configured);
    }

    #[test]
    fn the_accepted_instruction_ends_with_the_not_finished_warning() {
        let accepted = accepted_instructions(Locale::Zh);
        assert!(accepted.ends_with(t(
            Locale::Zh,
            keys::VOICE_INSTRUCTIONS_ACCEPTED_NOT_FINISHED
        )));
        assert!(accepted.contains(t(Locale::Zh, keys::VOICE_INSTRUCTIONS_ACCEPTED)));
    }

    #[test]
    fn the_duplicate_instruction_forbids_calling_the_tool_again() {
        let duplicate = duplicate_submission_instructions(Locale::Zh);
        assert!(duplicate.contains("不要再次调用工具"));
    }

    #[test]
    fn every_instruction_is_non_empty_in_all_three_locales() {
        let builders: [Builder; 10] = [
            ("result", result_response_instructions),
            ("permission", permission_response_instructions),
            ("reminder", reminder_confirmation_instructions),
            ("permission_pending", permission_pending_instructions),
            (
                "backend_not_configured",
                backend_not_configured_instructions,
            ),
            ("backend_disconnected", backend_disconnected_instructions),
            ("duplicate", duplicate_submission_instructions),
            ("accepted", accepted_instructions),
            ("always", permission_submitted_always_instructions),
            ("reject", permission_submitted_reject_instructions),
        ];
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            for (name, build) in builders {
                let rendered = build(locale);
                assert!(!rendered.trim().is_empty(), "{name} in {locale:?}");
                assert!(
                    !rendered.contains('{'),
                    "{name} in {locale:?} has an unfilled slot"
                );
            }
            assert!(!speak_response_instructions("x", locale).contains("{content}"));
        }
    }

    #[test]
    fn the_permission_instruction_forbids_prescribing_an_answer() {
        let instructions = permission_response_instructions(Locale::Zh);
        assert!(instructions.contains("不要规定具体回答方式"));
    }
}
