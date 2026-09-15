//! The pure half of `via chat`: the URL, the command set, and the two
//! catalogued output shapes.
//!
//! Ported from `tui/src/text-cli.mjs` and `tui/src/terminal-commands.mjs`.
//! Everything here is a function of its arguments, so the shapes a user reads
//! are asserted directly rather than through a socket.

use via_i18n::{Locale, format, keys, t};
use via_work::PublicWork;

/// The exit commands.
///
/// **External contract** — `tui/src/terminal-commands.mjs:1`:
/// `new Set(['/exit', '/quit', '/q'])`.
pub const EXIT_COMMANDS: [&str; 3] = ["/exit", "/quit", "/q"];

/// The help command.
pub const HELP_COMMAND: &str = "/help";

/// The task-list command.
pub const TASKS_COMMAND: &str = "/tasks";

/// The cancel command.
pub const CANCEL_COMMAND: &str = "/cancel";

/// The statuses `/cancel` will target when given no id.
///
/// **External contract** — `text-cli.mjs:58-60`:
/// `['queued', 'running', 'delegated', 'finalizing']`. Note it is **not**
/// `via_voice::tools::CANCELLABLE_STATUSES`, which additionally includes
/// `scheduled`: the model may cancel a reminder that has not fired, and this
/// command deliberately will not, because *"the first cancellable one"* picked
/// out of a list a user cannot see should never be a timer they set on purpose.
pub const CANCELLABLE_STATUSES: [via_protocol::WorkStatus; 4] = [
    via_protocol::WorkStatus::Queued,
    via_protocol::WorkStatus::Running,
    via_protocol::WorkStatus::Delegated,
    via_protocol::WorkStatus::Finalizing,
];

/// Whether `command` quits.
#[must_use]
pub fn is_exit_command(command: &str) -> bool {
    EXIT_COMMANDS.contains(&command)
}

/// The WebSocket URL for a Gateway origin and a session.
///
/// **External contract** — `text-cli.mjs:34-39`:
/// `new URL('/api/realtime', baseUrl)`, the scheme swapped `https:`→`wss:` and
/// everything else `ws:`, then `searchParams.set('sessionId', sessionId)`.
///
/// # Errors
///
/// The origin does not parse, or names no host.
pub fn websocket_url(origin: &str, session_id: &str) -> Result<String, url::ParseError> {
    let mut url = url::Url::parse(origin)?.join(via_voice::REALTIME_ROUTE)?;
    let scheme = if url.scheme() == "https" { "wss" } else { "ws" };
    url.set_scheme(scheme)
        .map_err(|()| url::ParseError::InvalidDomainCharacter)?;
    url.query_pairs_mut()
        .clear()
        .append_pair("sessionId", session_id);
    Ok(url.to_string())
}

/// One line of `/tasks`.
///
/// **External contract** — `text-cli.mjs:51-54`:
/// `` `${task.id}  ${task.status}  ${seconds}s  ${task.objective}` ``, where
/// `seconds` is `Math.max(0, Math.round((task.elapsedMs || 0) / 1000))`. Two
/// spaces between each field, and the status is the **wire** status rather than
/// a localized word, because a user reading it is going to type it into
/// `/cancel` or quote it in a bug report.
#[must_use]
pub fn task_line(task: &PublicWork) -> String {
    std::format!(
        "{}  {}  {}s  {}",
        task.id,
        task.status.as_str(),
        elapsed_seconds(task.elapsed_ms),
        task.objective,
    )
}

/// `Math.max(0, Math.round(ms / 1000))`.
///
/// JavaScript's `Math.round` is round-half-**up** rather than Rust's
/// round-half-away-from-zero, but the clamp to zero happens after, so the two
/// differ only for negative input — which the clamp erases.
#[must_use]
pub fn elapsed_seconds(elapsed_ms: i64) -> i64 {
    if elapsed_ms <= 0 {
        return 0;
    }
    (elapsed_ms + 500) / 1000
}

/// Which task `/cancel` targets.
///
/// **External contract** — `text-cli.mjs:56-61`: an explicit id matches by id
/// and nothing else — *not* "the first cancellable one" — so a user who names
/// a finished task is told there is nothing to cancel rather than having a
/// different task cancelled out from under them.
#[must_use]
pub fn select_cancellable<'tasks>(
    tasks: &'tasks [PublicWork],
    requested: Option<&str>,
) -> Option<&'tasks PublicWork> {
    match requested.map(str::trim).filter(|id| !id.is_empty()) {
        Some(id) => tasks.iter().find(|task| task.id == id),
        None => tasks
            .iter()
            .find(|task| CANCELLABLE_STATUSES.contains(&task.status)),
    }
}

/// The help text.
#[must_use]
pub fn help(locale: Locale) -> &'static str {
    t(locale, keys::CHAT_HELP)
}

/// One line of `via chat`'s command surface, split into a verb and its
/// argument.
///
/// **External contract** — `text-cli.mjs:171`: `text.split(/\s+/)`, so the
/// argument is the *second whitespace-separated token* and anything after it is
/// dropped. A Work id has no spaces in it, so this loses nothing a user could
/// have meant.
#[must_use]
pub fn split_command(line: &str) -> (&str, Option<&str>) {
    let mut parts = line.split_whitespace();
    (parts.next().unwrap_or_default(), parts.next())
}

/// The connect frame `via chat` sends.
///
/// **External contract** — `text-cli.mjs:105-112`, with one field stated
/// differently and the difference is deliberate. Upstream sends
/// `voiceEnabled: true` and comments *"enable the output channel to receive
/// task announcements; the CLI sends no audio and ignores audio frames"* —
/// it overloads the legacy single flag because it has to. VIA sends the
/// **explicit pair** `active-voice-clients.mjs:46-64` provides for exactly
/// this: `voiceEnabled: false` (there is no voice), `outputEnabled: true` (a
/// text client still hears announcements read to it), `textOnly: true` (which
/// vetoes capture and the voice slot but **not** output).
///
/// [`via_voice::client_voice_capabilities`] resolves both spellings to the same
/// answer — `{input: false, output: true, arbitration: false}` — so this is a
/// clearer statement of upstream's behaviour rather than a change to it.
#[must_use]
pub fn connect_frame(takeover: bool, time_zone: &str, locale: Locale) -> serde_json::Value {
    serde_json::json!({
        "type": via_protocol::GatewayClientEvent::Connect.as_str(),
        "voiceEnabled": false,
        "outputEnabled": true,
        "textOnly": true,
        "takeover": takeover,
        "clientType": via_protocol::ClientType::Cli.as_str(),
        "timeZone": time_zone,
        "locale": locale.as_str(),
    })
}

/// The line printed for a task-plane frame.
///
/// **VIA's own.** Upstream's text CLI drops every `task.*` frame on the floor:
/// it renders the announcement when the model speaks it and nothing before
/// that, so a delegation that takes two minutes looks like a hang. `via chat`
/// is the end-to-end harness for the Work queue, and a harness that cannot see
/// `queued → running → completed` cannot be used to debug it.
#[must_use]
pub fn task_event_line(locale: Locale, id: &str, status: &str) -> String {
    format(
        locale,
        keys::CHAT_TASK_EVENT,
        &[("id", id), ("status", status)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use via_protocol::WorkStatus;

    fn task(id: &str, status: WorkStatus, elapsed_ms: i64) -> PublicWork {
        let mut task = via_work::testing::blank_record(id, "user_1").to_public(0);
        task.status = status;
        task.elapsed_ms = elapsed_ms;
        task.objective = "summarise the diff".to_owned();
        task
    }

    #[rstest]
    #[case("/exit", true)]
    #[case("/quit", true)]
    #[case("/q", true)]
    #[case("/exi", false)]
    #[case("exit", false)]
    #[case("/EXIT", false)]
    #[case("", false)]
    fn only_the_three_spellings_quit(#[case] command: &str, #[case] quits: bool) {
        assert_eq!(is_exit_command(command), quits, "{command}");
    }

    #[test]
    fn the_websocket_url_swaps_the_scheme_and_carries_the_session() {
        assert_eq!(
            websocket_url("http://127.0.0.1:3101", "voice-1").expect("a url"),
            "ws://127.0.0.1:3101/api/realtime?sessionId=voice-1",
        );
        assert_eq!(
            websocket_url("https://voice.example.com", "voice-1").expect("a url"),
            "wss://voice.example.com/api/realtime?sessionId=voice-1",
        );
    }

    #[test]
    fn a_session_id_that_needs_escaping_is_escaped() {
        // `searchParams.set` percent-encodes; a session id is a user-supplied
        // string and `--session 'a b&c=d'` must not become three parameters.
        let url = websocket_url("http://127.0.0.1:3101", "a b&c=d").expect("a url");
        assert!(url.ends_with("sessionId=a+b%26c%3Dd"), "{url}");
    }

    #[test]
    fn a_task_line_is_id_status_seconds_objective() {
        assert_eq!(
            task_line(&task("work_1", WorkStatus::Running, 61_400)),
            "work_1  running  61s  summarise the diff",
        );
    }

    #[rstest]
    #[case(0, 0)]
    #[case(499, 0)]
    #[case(500, 1)]
    #[case(1_499, 1)]
    #[case(1_500, 2)]
    // Upstream's `(task.elapsedMs || 0)` turns a negative into a negative and
    // then `Math.max(0, …)` erases it.
    #[case(-5_000, 0)]
    fn the_seconds_are_rounded_then_clamped(#[case] elapsed_ms: i64, #[case] seconds: i64) {
        assert_eq!(elapsed_seconds(elapsed_ms), seconds, "{elapsed_ms}");
    }

    #[test]
    fn an_explicit_id_matches_by_id_and_nothing_else() {
        let tasks = vec![
            task("work_1", WorkStatus::Running, 0),
            task("work_2", WorkStatus::Completed, 0),
        ];
        assert_eq!(
            select_cancellable(&tasks, Some("work_2")).map(|task| task.id.as_str()),
            Some("work_2"),
            "a named terminal task is still the one named",
        );
        assert!(
            select_cancellable(&tasks, Some("work_9")).is_none(),
            "an id nothing answers to must not fall back to another task",
        );
    }

    #[test]
    fn with_no_id_the_first_cancellable_status_wins() {
        let tasks = vec![
            task("work_1", WorkStatus::Completed, 0),
            task("work_2", WorkStatus::Delegated, 0),
            task("work_3", WorkStatus::Running, 0),
        ];
        assert_eq!(
            select_cancellable(&tasks, None).map(|task| task.id.as_str()),
            Some("work_2"),
        );
    }

    #[test]
    fn a_scheduled_reminder_is_never_cancelled_by_accident() {
        let tasks = vec![task("work_1", WorkStatus::Scheduled, 0)];
        assert!(
            select_cancellable(&tasks, None).is_none(),
            "`/cancel` with no id must not silently cancel a timer the user set",
        );
        assert!(
            via_voice::tools::CANCELLABLE_STATUSES.contains(&WorkStatus::Scheduled),
            "…even though the model's own cancel tool may",
        );
    }

    #[test]
    fn a_blank_argument_reads_as_no_argument() {
        let tasks = vec![task("work_1", WorkStatus::Running, 0)];
        assert_eq!(
            select_cancellable(&tasks, Some("   ")).map(|task| task.id.as_str()),
            Some("work_1"),
        );
    }

    #[rstest]
    #[case("/cancel work_1", "/cancel", Some("work_1"))]
    #[case("/cancel", "/cancel", None)]
    #[case("/cancel   work_1   extra", "/cancel", Some("work_1"))]
    #[case("   /tasks  ", "/tasks", None)]
    fn a_command_line_splits_on_whitespace(
        #[case] line: &str,
        #[case] command: &str,
        #[case] argument: Option<&str>,
    ) {
        assert_eq!(split_command(line), (command, argument));
    }

    #[test]
    fn the_connect_frame_declares_output_without_voice() {
        let frame = connect_frame(true, "Asia/Shanghai", Locale::En);
        assert_eq!(frame["type"], "connect");
        assert_eq!(frame["voiceEnabled"], serde_json::Value::Bool(false));
        assert_eq!(frame["outputEnabled"], serde_json::Value::Bool(true));
        assert_eq!(frame["textOnly"], serde_json::Value::Bool(true));
        assert_eq!(frame["takeover"], serde_json::Value::Bool(true));
        assert_eq!(frame["clientType"], "cli");

        // …and the Gateway resolves it to upstream's own answer.
        let capabilities = via_voice::client_voice_capabilities(via_voice::DeclaredCapabilities {
            voice_enabled: false,
            input_enabled: None,
            output_enabled: Some(true),
            text_only: true,
        });
        let upstream = via_voice::client_voice_capabilities(via_voice::DeclaredCapabilities {
            voice_enabled: true,
            input_enabled: None,
            output_enabled: None,
            text_only: true,
        });
        assert_eq!(capabilities, upstream);
        assert!(capabilities.output_enabled, "announcements must still land");
        assert!(!capabilities.input_enabled);
        assert!(!capabilities.participates_in_voice_arbitration);
    }

    #[test]
    fn the_help_text_names_every_command() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let text = help(locale);
            for command in [TASKS_COMMAND, CANCEL_COMMAND, HELP_COMMAND, "/exit"] {
                assert!(text.contains(command), "{locale}: {text}");
            }
            for alias in ["/quit", "/q"] {
                assert!(text.contains(alias), "{locale}: {text}");
            }
        }
    }
}
