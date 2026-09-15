//! Response guards — the one correction a finished response may earn.
//!
//! Ported from `server/src/voice/response-guards/index.mjs` and
//! `server/src/voice/response-guards/action-promise.mjs`.
//!
//! A guard inspects a response that has already completed and, at most once,
//! asks the model to reconsider. It never executes a tool, never mutates Work
//! state, and never speaks on its own — it supplies `instructions` for one
//! extra `response.create`, and the gateway declines to send even that unless
//! [`is_response_guard_turn_current`] still holds.
//!
//! Registration is deliberately static: adding a guard is a reviewed code
//! change, not runtime configuration, and **the first match is the only
//! correction allowed for one response**.

use once_cell::sync::Lazy;
use regex::Regex;
use via_i18n::{Locale, keys, t};

use crate::response::ResponseOrigin;

/// What a finished response looked like.
///
/// **External contract** — `response-guards/action-promise.mjs:53-59`, the
/// destructured `matches({...})` argument.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuardObservation {
    /// Where the response came from.
    pub origin: ResponseOrigin,
    /// Whether it authored a tool call.
    pub has_function_call: bool,
    /// Whether `response.done` reported a failure status.
    pub failed: bool,
    /// Whether the response was cancelled or superseded.
    pub suppressed: bool,
    /// The settled assistant transcript.
    pub transcript: String,
}

/// A guard's verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardDecision {
    /// Which guard matched.
    pub guard_id: &'static str,
    /// The instructions for the one corrective response.
    pub instructions: String,
}

/// One registered guard.
pub trait ResponseGuard: Send + Sync {
    /// A stable identifier, reported on the decision.
    fn id(&self) -> &'static str;
    /// The instructions this guard supplies, in `locale`.
    fn instructions(&self, locale: Locale) -> String;
    /// Whether this observation earns a correction.
    fn matches(&self, observation: &GuardObservation) -> bool;
}

/// The action-promise guard's id.
///
/// **External contract** — `response-guards/action-promise.mjs:48`.
pub const ACTION_PROMISE_GUARD_ID: &str = "action-promise";

/// The longest transcript the action-promise guard will look at.
///
/// **External contract** — `response-guards/action-promise.mjs:37`
/// (`ACTION_PROMISE_MAX_CHARS = 40`), a JavaScript `.length`, so it is a
/// UTF-16 bound. The guard recognizes one short, explicit promise; a long
/// sentence is by construction not one.
pub const ACTION_PROMISE_MAX_CHARS: usize = 40;

/// The promise pattern.
///
/// **External contract** — `response-guards/action-promise.mjs:5-22`,
/// reproduced alternation for alternation. Rust's `regex` has no lookaround,
/// and this pattern uses none.
static ACTION_PROMISE: Lazy<Option<Regex>> = Lazy::new(|| {
    Regex::new(concat!(
        "^",
        "(?:(?:好的?|好|行|明白|收到)[，,\\s]*)?",
        "(?:稍等[，,\\s]*)?",
        "(?:",
        "我(?:来|去|先去|马上|立刻|现在(?:就)?|这就)(?:帮你|替你)?",
        "|让我来",
        "|马上(?:去|来)?",
        "|现在就(?:去|来)?",
        ")",
        "(?:",
        "查(?:一下)?|查询|查找|看(?:一下)?|检查|确认|核实|搜索|排查|调查",
        "|处理|修改|调整|创建|新建|运行|跑(?:一下)?|测试|验证",
        ")",
        "[^，,。；;：:！？!?\\n]{0,28}",
        "[。！!]?",
        "$",
    ))
    .ok()
});

/// The "this is a question, not a promise" pattern.
///
/// **External contract** — `response-guards/action-promise.mjs:24-33`.
static CONFIRMATION_REQUEST: Lazy<Option<Regex>> = Lazy::new(|| {
    Regex::new(concat!(
        "[？?]\\s*$",
        "|好吗",
        "|可以吗",
        "|行吗",
        "|要不要",
        "|需要我",
        "|是否需要",
        "|要我(?:现在|先)?(?:去|来|帮)",
    ))
    .ok()
});

/// The "an answer was actually delivered" pattern.
///
/// **External contract** — `response-guards/action-promise.mjs:35`.
static DELIVERED_CONTENT: Lazy<Option<Regex>> = Lazy::new(|| {
    Regex::new(
        "(?:[：:]|结果|答案|查到|找到|发现|显示|已经(?:完成|处理|修改|创建|运行|测试)|原因是|因为)",
    )
    .ok()
});

/// Whether `transcript` is a bare promise to execute work.
///
/// **External contract** — `response-guards/action-promise.mjs:39-45`, in
/// order: bound the length, reject a question, reject a delivered answer, then
/// match the promise. The order is the point — a sentence that both promises
/// and delivers is a delivery, and a sentence that asks permission is not a
/// promise at all.
///
/// # If a pattern fails to compile
///
/// Every branch falls **closed** — no correction. The patterns are literals and
/// cannot in fact fail, but a guard that fired on a pattern it could not
/// compile would ask the model to reconsider an arbitrary response, which is
/// worse than never correcting it. Same direction as
/// [`via_conversation::tool::is_sensitive`], opposite answer, because the safe
/// side is opposite: there, refusing to write; here, staying quiet.
#[must_use]
pub fn promises_action(transcript: &str) -> bool {
    let content = crate::text::trim(transcript);
    if content.is_empty() || crate::text::utf16_len(content) > ACTION_PROMISE_MAX_CHARS {
        return false;
    }
    let (Some(promise), Some(question), Some(delivered)) = (
        ACTION_PROMISE.as_ref(),
        CONFIRMATION_REQUEST.as_ref(),
        DELIVERED_CONTENT.as_ref(),
    ) else {
        return false;
    };
    if question.is_match(content) {
        return false;
    }
    if delivered.is_match(content) {
        return false;
    }
    promise.is_match(content)
}

/// The guard that catches a promise the model never acted on.
///
/// It is deliberately **not** a general intent classifier: compound sentences
/// and delivered answers remain the model's own responsibility. Everything it
/// declines to match is a case where a second, unrequested response would be
/// worse than silence.
#[derive(Clone, Copy, Debug, Default)]
pub struct ActionPromiseGuard;

impl ResponseGuard for ActionPromiseGuard {
    fn id(&self) -> &'static str {
        ACTION_PROMISE_GUARD_ID
    }

    fn instructions(&self, locale: Locale) -> String {
        t(locale, keys::VOICE_INSTRUCTIONS_ACTION_PROMISE_BROKEN).to_owned()
    }

    fn matches(&self, observation: &GuardObservation) -> bool {
        if observation.origin != ResponseOrigin::Model {
            return false;
        }
        if observation.has_function_call || observation.failed || observation.suppressed {
            return false;
        }
        promises_action(&observation.transcript)
    }
}

/// The registered guards, in match order.
///
/// **External contract** — `response-guards/index.mjs:6-8`
/// (`RESPONSE_GUARDS`), frozen and containing exactly the action-promise
/// guard.
#[must_use]
pub fn default_guards() -> &'static [&'static (dyn ResponseGuard + 'static)] {
    static GUARDS: &[&(dyn ResponseGuard + 'static)] = &[&ActionPromiseGuard];
    GUARDS
}

/// Run the guards and return the first match's correction.
///
/// **External contract** — `response-guards/index.mjs:10-26`. A guard with an
/// empty `instructions` is skipped rather than returning a correction with
/// nothing in it.
#[must_use]
pub fn evaluate_response_guards(
    observation: &GuardObservation,
    locale: Locale,
    guards: &[&dyn ResponseGuard],
) -> Option<GuardDecision> {
    for guard in guards {
        let instructions = guard.instructions(locale);
        let instructions = crate::text::trim(&instructions);
        if guard.id().is_empty() || instructions.is_empty() {
            continue;
        }
        if !guard.matches(observation) {
            continue;
        }
        return Some(GuardDecision {
            guard_id: guard.id(),
            instructions: instructions.to_owned(),
        });
    }
    None
}

/// Run the [`default_guards`].
#[must_use]
pub fn evaluate_default_response_guards(
    observation: &GuardObservation,
    locale: Locale,
) -> Option<GuardDecision> {
    evaluate_response_guards(observation, locale, default_guards())
}

/// Whether the turn a guard wants to correct is still the live one.
///
/// **External contract** — `response-guards/index.mjs:28-45`. All six
/// conditions, and each one closes a real hole:
///
/// - `same_frontend` — the realtime connection was replaced, so the correction
///   would land in a session that never heard the promise.
/// - `output_enabled` — this client no longer owns the speaker.
/// - `!user_speaking` — correcting into a barge-in talks over the user.
/// - a non-empty `response_turn_id` equal to the committed one, and the same
///   generation — the user has moved on, and a correction for a turn that is
///   two turns old is noise.
#[must_use]
pub fn is_response_guard_turn_current(state: &GuardTurnState) -> bool {
    state.same_frontend
        && state.output_enabled
        && !state.user_speaking
        && !state.response_turn_id.is_empty()
        && state.response_turn_id == state.committed_turn_id
        && state.response_turn_generation == state.committed_turn_generation
}

/// The six inputs to [`is_response_guard_turn_current`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuardTurnState {
    /// The realtime frontend is still the one that produced the response.
    pub same_frontend: bool,
    /// This client still owns the speaker.
    pub output_enabled: bool,
    /// The user is talking right now.
    pub user_speaking: bool,
    /// The turn the response answered.
    pub response_turn_id: String,
    /// That turn's generation.
    pub response_turn_generation: i64,
    /// The turn the gateway has committed to.
    pub committed_turn_id: String,
    /// That turn's generation.
    pub committed_turn_generation: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One named way to break an otherwise-passing input.
    type Mutation = (&'static str, fn(&mut GuardTurnState));
    use pretty_assertions::assert_eq;

    fn promise(transcript: &str) -> GuardObservation {
        GuardObservation {
            origin: ResponseOrigin::Model,
            transcript: transcript.to_owned(),
            ..GuardObservation::default()
        }
    }

    #[test]
    fn a_bare_promise_matches() {
        for transcript in [
            "我来查一下",
            "好的，我马上帮你处理",
            "稍等，我这就运行",
            "让我来看一下",
            "马上去检查",
            "现在就来修改",
            "收到，我立刻替你验证一下配置",
        ] {
            assert!(promises_action(transcript), "{transcript} should match");
        }
    }

    #[test]
    fn a_question_is_not_a_promise() {
        for transcript in [
            "我来查一下好吗",
            "要不要我来查一下",
            "需要我去处理吗",
            "我来查一下？",
            "是否需要我来运行",
            "要我现在去查一下",
        ] {
            assert!(
                !promises_action(transcript),
                "{transcript} should not match"
            );
        }
    }

    #[test]
    fn a_delivered_answer_is_not_a_promise() {
        for transcript in [
            "我来查一下：结果是三个",
            "我马上处理，已经完成了",
            "我去查找，找到两处",
            "我来看一下，原因是端口冲突",
        ] {
            assert!(
                !promises_action(transcript),
                "{transcript} should not match"
            );
        }
    }

    #[test]
    fn a_delivery_that_would_otherwise_match_the_promise_pattern_is_still_refused() {
        // The cases above are also excluded by the tail's punctuation class, so
        // they do not exercise the delivered-content check on their own. These
        // match the promise pattern in full — no excluded punctuation anywhere —
        // and are rejected only because they say an answer was delivered.
        for transcript in [
            "我来查一下已经完成",
            "我去查找找到两处",
            "我马上检查发现三个问题",
            "我来看一下答案",
        ] {
            assert!(
                ACTION_PROMISE
                    .as_ref()
                    .is_some_and(|pattern| pattern.is_match(transcript)),
                "{transcript} must reach the delivered-content check",
            );
            assert!(
                !promises_action(transcript),
                "{transcript} delivers an answer and is not a bare promise",
            );
        }
    }

    #[test]
    fn the_length_bound_is_a_utf16_length_of_forty() {
        // The longest matchable prefix is 12 characters — `好的，` plus
        // `我现在就帮你` plus `查一下` — and the tail is capped at 28, so a
        // 41-character promise is reachable and is exactly what the bound is
        // there to stop.
        let head = "好的，我现在就帮你查一下";
        assert_eq!(crate::text::utf16_len(head), 12);
        let exactly_forty = format!("{head}{}", "阿".repeat(28));
        assert_eq!(crate::text::utf16_len(&exactly_forty), 40);
        assert!(promises_action(&exactly_forty));
        let forty_one = format!("{exactly_forty}。");
        assert_eq!(crate::text::utf16_len(&forty_one), 41);
        assert!(
            !promises_action(&forty_one),
            "the bound rejects it even though the pattern would match",
        );
    }

    #[test]
    fn an_empty_or_whitespace_transcript_never_matches() {
        assert!(!promises_action(""));
        assert!(!promises_action("   \n "));
    }

    #[test]
    fn the_guard_declines_every_non_model_origin() {
        for origin in [
            ResponseOrigin::Announcement,
            ResponseOrigin::Permission,
            ResponseOrigin::Agent,
            ResponseOrigin::Progress,
        ] {
            let observation = GuardObservation {
                origin,
                transcript: "我来查一下".to_owned(),
                ..GuardObservation::default()
            };
            assert!(
                evaluate_default_response_guards(&observation, Locale::Zh).is_none(),
                "{origin} earned a correction",
            );
        }
    }

    #[test]
    fn a_response_that_did_call_a_tool_earns_nothing() {
        let mut observation = promise("我来查一下");
        observation.has_function_call = true;
        assert!(evaluate_default_response_guards(&observation, Locale::Zh).is_none());
    }

    #[test]
    fn a_failed_or_suppressed_response_earns_nothing() {
        let mut failed = promise("我来查一下");
        failed.failed = true;
        assert!(evaluate_default_response_guards(&failed, Locale::Zh).is_none());
        let mut suppressed = promise("我来查一下");
        suppressed.suppressed = true;
        assert!(evaluate_default_response_guards(&suppressed, Locale::Zh).is_none());
    }

    #[test]
    fn a_match_reports_the_guard_id_and_the_localized_instructions() {
        let decision = evaluate_default_response_guards(&promise("我来查一下"), Locale::Zh)
            .expect("a bare promise earns a correction");
        assert_eq!(decision.guard_id, ACTION_PROMISE_GUARD_ID);
        assert_eq!(
            decision.instructions,
            t(Locale::Zh, keys::VOICE_INSTRUCTIONS_ACTION_PROMISE_BROKEN),
        );
        // The correction is localized, not pinned to `zh`.
        let english = evaluate_default_response_guards(&promise("我来查一下"), Locale::En)
            .expect("locale does not change what matches");
        assert_eq!(
            english.instructions,
            t(Locale::En, keys::VOICE_INSTRUCTIONS_ACTION_PROMISE_BROKEN),
        );
        assert_ne!(english.instructions, decision.instructions);
    }

    #[test]
    fn a_guard_with_empty_instructions_is_skipped_rather_than_returned() {
        struct Silent;
        impl ResponseGuard for Silent {
            fn id(&self) -> &'static str {
                "silent"
            }
            fn instructions(&self, _locale: Locale) -> String {
                "   ".to_owned()
            }
            fn matches(&self, _observation: &GuardObservation) -> bool {
                true
            }
        }
        let silent = Silent;
        let action = ActionPromiseGuard;
        let guards: [&dyn ResponseGuard; 2] = [&silent, &action];
        let decision = evaluate_response_guards(&promise("我来查一下"), Locale::Zh, &guards)
            .expect("the second guard still runs");
        assert_eq!(decision.guard_id, ACTION_PROMISE_GUARD_ID);
    }

    #[test]
    fn only_the_first_matching_guard_wins() {
        struct Always;
        impl ResponseGuard for Always {
            fn id(&self) -> &'static str {
                "always"
            }
            fn instructions(&self, _locale: Locale) -> String {
                "reconsider".to_owned()
            }
            fn matches(&self, _observation: &GuardObservation) -> bool {
                true
            }
        }
        let always = Always;
        let action = ActionPromiseGuard;
        let guards: [&dyn ResponseGuard; 2] = [&always, &action];
        let decision = evaluate_response_guards(&promise("我来查一下"), Locale::Zh, &guards)
            .expect("the first guard matches");
        assert_eq!(decision.guard_id, "always");
    }

    #[test]
    fn the_turn_currency_check_needs_all_six_conditions() {
        let current = GuardTurnState {
            same_frontend: true,
            output_enabled: true,
            user_speaking: false,
            response_turn_id: "voice-1".to_owned(),
            response_turn_generation: 3,
            committed_turn_id: "voice-1".to_owned(),
            committed_turn_generation: 3,
        };
        assert!(is_response_guard_turn_current(&current));

        let mutations: [Mutation; 6] = [
            ("frontend replaced", |state| state.same_frontend = false),
            ("output disabled", |state| state.output_enabled = false),
            ("user speaking", |state| state.user_speaking = true),
            ("no turn id", |state| state.response_turn_id.clear()),
            ("turn moved on", |state| {
                state.committed_turn_id = "voice-2".to_owned();
            }),
            ("generation bumped", |state| {
                state.committed_turn_generation = 4
            }),
        ];
        for (why, mutate) in mutations {
            let mut state = current.clone();
            mutate(&mut state);
            assert!(
                !is_response_guard_turn_current(&state),
                "{why} still passed"
            );
        }
    }

    #[test]
    fn the_default_registry_holds_exactly_the_action_promise_guard() {
        assert_eq!(default_guards().len(), 1);
        assert_eq!(default_guards()[0].id(), ACTION_PROMISE_GUARD_ID);
    }
}
