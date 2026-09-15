//! What the backend agent is told about itself, on every single turn.
//!
//! Two pieces, both **model-visible and both catalogued verbatim**:
//!
//! * [`BACKEND_AGENT_INSTRUCTIONS`] — `server/src/agent/backend-agent-instructions.mjs:1-17`,
//!   fifteen lines joined by `\n` (see
//!   [`BACKEND_AGENT_INSTRUCTION_LINE_COUNT`]);
//! * [`coordinator_instructions`] — the wrapper
//!   `server/src/agent/acp-backend-adapter.mjs:940-955` puts around them and
//!   around the turn's own text.
//!
//! # The only literal in this crate
//!
//! `BACKEND_AGENT_INSTRUCTIONS` is the one model-visible string here that does
//! not come from `via-i18n`, and that is deliberate: upstream does not localize
//! it either, `docs/reference/contracts.json` pins the English text
//! (`prompt-text` / *BACKEND_AGENT_INSTRUCTIONS*), and translating it would
//! break the assertion it exists to make. `via-backends` records the same
//! reasoning for the three per-profile `sessionInstructions` paragraphs.
//!
//! # Three lines carry the product name
//!
//! `docs/rebrand.md` row 143 renames lines 1, 4 and 13 (upstream's source lines
//! 2, 5 and 14) —
//! `qwen-audio-agent` → `VIA` — and row 30 renames the wrapper tag
//! `<qwen_audio_agent_backend_instructions>` → [`BACKEND_INSTRUCTIONS_OPEN_TAG`].
//! The rename has to be applied in both places at once: the envelope's
//! `protocol` field ([`crate::envelope::COORDINATION_PROTOCOL`]) and this tag
//! are what tell the model that the envelope in front of it and the
//! instructions above it belong to the same system.

use via_downstream::text::clean;

/// The opening tag of the instruction wrapper.
///
/// **External contract**, renamed. Upstream
/// `<qwen_audio_agent_backend_instructions>`
/// (`server/src/agent/acp-backend-adapter.mjs:949`); `docs/rebrand.md` row 30.
pub const BACKEND_INSTRUCTIONS_OPEN_TAG: &str = "<via_backend_instructions>";

/// The closing tag of the instruction wrapper.
///
/// **External contract**, renamed. Upstream
/// `</qwen_audio_agent_backend_instructions>`
/// (`acp-backend-adapter.mjs:952`).
pub const BACKEND_INSTRUCTIONS_CLOSE_TAG: &str = "</via_backend_instructions>";

/// The fifteen lines every coordinator prompt is prefixed with.
///
/// **External contract, verbatim** — `docs/reference/contracts.json`
/// (`prompt-text` / *BACKEND_AGENT_INSTRUCTIONS*), from
/// `server/src/agent/backend-agent-instructions.mjs:1-17`. Every line is a
/// behavioural constraint the delegation protocol depends on; three of them
/// carry the product name and are rebranded per `docs/rebrand.md` row 143.
///
/// Reproduced from the catalogued `exactValue`, which is the authority. Its
/// prose says *"16 lines"* and its value has **fifteen** — the array literal
/// runs from source line 2 to line 16 and line 17 is the `join`. `via-acp`
/// records the same class of discrepancy for the environment allow-list
/// ("47 OS names", 39 listed) and resolves it the same way: the value wins.
///
/// [`BACKEND_AGENT_INSTRUCTION_LINES`] is the same text as the array it was
/// joined from, so a test can assert the count as well as the bytes.
pub const BACKEND_AGENT_INSTRUCTIONS: &str = "You are the backend Agent for VIA.
The user experiences the realtime voice frontend and your work as one assistant.
Use the tools, project context, memory, and permissions already available to you.
Treat the VIA request envelope as the current user request.
Follow the response contract inside that envelope exactly.
Preserve the requested action level. Implementation and continuation requests remain execution requests unless planning was requested or an indispensable choice is missing.
Preserve how the requested work relates to existing work. Create a project Session for new independent work, continue the matching Session for prior work, and otherwise work in the coordinator Session. Decide from the full meaning of the request, not isolated keywords.
A new independent task requires a new Session in the coordinator project, not a new directory. To work on a previous project, locate and continue its existing Session.
Do not create, choose, or prepare directories for Session routing. The Gateway owns Session directory resolution; file organization happens only inside the Session responsible for the work.
Continue an existing project Session when the user refers to prior work; do not silently create a replacement.
Send only natural task text to project Sessions. Never copy request envelopes, work IDs, or routing instructions into project history.
Only work in the coordinator workspace when no separate or previous project Session is needed.
Do not modify VIA itself unless explicitly requested.
Only claim completion after the responsible tool or external system confirms success.
Do not expose backend routing, protocol fields, Agent IDs, or Session IDs.";

/// How many lines [`BACKEND_AGENT_INSTRUCTIONS`] has.
///
/// **Fifteen**, counted from the catalogued `exactValue` rather than from its
/// prose — see [`BACKEND_AGENT_INSTRUCTIONS`]. The constant exists so the count
/// is asserted rather than eyeballed, and `tests/contracts.rs` asserts it
/// against the catalogue's own value.
pub const BACKEND_AGENT_INSTRUCTION_LINE_COUNT: usize = 15;

/// The instruction lines, as the array upstream joins.
///
/// Derived from [`BACKEND_AGENT_INSTRUCTIONS`] rather than written twice: a
/// second copy of fifteen contract lines is a second thing to keep in sync.
#[must_use]
pub fn backend_agent_instruction_lines() -> Vec<&'static str> {
    BACKEND_AGENT_INSTRUCTIONS.split('\n').collect()
}

/// Wrap one turn's text in the backend-instruction block.
///
/// **External contract** — `docs/reference/contracts.json` (`prompt-text` /
/// *backend instructions wrapper*), `acp-backend-adapter.mjs:940-956`:
///
/// ```text
/// <via_backend_instructions>
/// <BACKEND_AGENT_INSTRUCTIONS>
/// <session instructions>
/// </via_backend_instructions>
///
/// <the turn's own text>
/// ```
///
/// `session_instructions` is the profile's paragraph — for a backend that
/// reaches Layer 3 through VIA's MCP tools that is
/// [`via_mcp_tools::DEFAULT_SESSION_INSTRUCTIONS`], and three shipped profiles
/// override it with their own (`via-backends`). It is passed in rather than
/// chosen here because choosing it would mean naming a backend.
///
/// Upstream applies this through `transformPromptText`, which rewrites **only
/// the first text block** of a multi-part prompt so an attached image is not
/// swallowed by the wrapper. VIA's [`via_downstream::PromptRequest`] keeps text
/// and attachments in separate fields, so the same property holds by
/// construction and there is nothing to transform.
#[must_use]
pub fn coordinator_instructions(session_instructions: &str, content: &str) -> String {
    format!(
        "{BACKEND_INSTRUCTIONS_OPEN_TAG}\n\
         {BACKEND_AGENT_INSTRUCTIONS}\n\
         {session_instructions}\n\
         {BACKEND_INSTRUCTIONS_CLOSE_TAG}\n\
         \n\
         {content}",
    )
}

/// Whether `text` already carries the instruction wrapper.
///
/// Used by [`crate::Coordinator`] to keep the wrapper from being applied twice
/// when a control turn is composed from an already-wrapped prompt. Upstream has
/// no equivalent because its wrapper is applied at exactly one call site; the
/// predicate is here so that stays checkable rather than merely true today.
#[must_use]
pub fn is_wrapped(text: &str) -> bool {
    clean(text).starts_with(BACKEND_INSTRUCTIONS_OPEN_TAG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_instructions_are_fifteen_lines() {
        let lines = backend_agent_instruction_lines();
        assert_eq!(lines.len(), BACKEND_AGENT_INSTRUCTION_LINE_COUNT);
        assert_eq!(lines[0], "You are the backend Agent for VIA.");
        assert_eq!(
            lines[BACKEND_AGENT_INSTRUCTION_LINE_COUNT - 1],
            "Do not expose backend routing, protocol fields, Agent IDs, or Session IDs.",
        );
        assert!(
            lines.iter().all(|line| !line.is_empty()),
            "a blank line would change what the model sees",
        );
    }

    #[test]
    fn the_three_rebranded_lines_name_via_and_nothing_else() {
        let lines = backend_agent_instruction_lines();
        let naming: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|line| line.contains("VIA"))
            .collect();
        assert_eq!(
            naming,
            [
                "You are the backend Agent for VIA.",
                "Treat the VIA request envelope as the current user request.",
                "Do not modify VIA itself unless explicitly requested.",
            ],
            "docs/rebrand.md row 143 renames exactly these three",
        );
    }

    #[test]
    fn the_wrapper_puts_a_blank_line_before_the_turn() {
        let wrapped = coordinator_instructions("SESSION", "USER TEXT");
        let lines: Vec<&str> = wrapped.split('\n').collect();
        assert_eq!(lines[0], BACKEND_INSTRUCTIONS_OPEN_TAG);
        assert_eq!(lines[1], "You are the backend Agent for VIA.");
        assert_eq!(lines[lines.len() - 4], "SESSION");
        assert_eq!(lines[lines.len() - 3], BACKEND_INSTRUCTIONS_CLOSE_TAG);
        assert_eq!(lines[lines.len() - 2], "");
        assert_eq!(lines[lines.len() - 1], "USER TEXT");
        assert!(is_wrapped(&wrapped));
        assert!(!is_wrapped("USER TEXT"));
        assert!(
            is_wrapped(&format!("  \n{wrapped}")),
            "the predicate cleans first, exactly as every other check here does",
        );
    }

    #[test]
    fn a_multi_line_turn_is_appended_whole() {
        let wrapped = coordinator_instructions("S", "line one\nline two");
        assert!(wrapped.ends_with("\n\nline one\nline two"));
    }
}
