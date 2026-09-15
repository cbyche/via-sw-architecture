//! `via chat` — the text client.
//!
//! A port of `tui/src/text-cli.mjs` (221 lines) over
//! `shared/gateway-client.mjs`. `docs/fidelity.md` puts its job exactly:
//! *"`tui/` — upstream's terminal UI … is deferred, not dropped. VIA ships
//! `via chat` (a port of upstream's `tui/src/text-cli.mjs`) so the core is
//! drivable end to end."*
//!
//! # What it is for
//!
//! `docs/architecture.md` §15's phase-5 milestone is *"`via chat` works end to
//! end — no audio hardware, no model weights"*. This is that client, and every
//! decision below follows from it:
//!
//! - it connects with **no voice** but with output enabled, so a delegated
//!   result still reaches it through the announcement window
//!   ([`protocol::connect_frame`]);
//! - it answers `audio.delta` / `audio.done` with playback receipts even though
//!   it plays nothing, because the receipts are the Injection Gate's only
//!   evidence that a response was heard;
//! - it prints the Work plane as it moves, which upstream's own text CLI does
//!   not — a harness that cannot see `queued → running → completed` cannot be
//!   used to debug the thing it exists to exercise.
//!
//! # The two fan-ins
//!
//! One task, selecting over the socket and over stdin. Reading the two in one
//! place is what makes a line typed while a response is streaming land *after*
//! that response rather than interleaved with it.

pub mod autostart;
pub mod client;
pub mod protocol;

use std::io::Write;

use futures::{SinkExt, StreamExt};
use tokio::io::AsyncBufReadExt;
use via_i18n::{Locale, format, keys, t};
use via_protocol::{GatewayServerEvent, GatewayTaskEvent};

use crate::commands::chat::ChatPlan;
use crate::error::{CODE_INVALID_ARGUMENT, CliError};
use crate::host::Host;

pub use client::GatewayClient;

/// The ANSI escapes upstream's text CLI paints with.
///
/// **External contract** — `text-cli.mjs:9-15`. They are reproduced because a
/// user who has scripted against the output already sees them, and because a
/// dim `·` prefix is what distinguishes the client's own chatter from the
/// assistant's words.
pub mod style {
    /// Dim — the client's own chatter.
    pub const DIM: &str = "\u{1b}[90m";
    /// Cyan — a timeline block.
    pub const CYAN: &str = "\u{1b}[36m";
    /// Yellow — a task line.
    pub const YELLOW: &str = "\u{1b}[33m";
    /// Red — a refusal.
    pub const RED: &str = "\u{1b}[31m";
    /// Bold — the assistant's speaker prefix.
    pub const BOLD: &str = "\u{1b}[1m";
    /// Reset.
    pub const RESET: &str = "\u{1b}[0m";
}

/// The prefix the assistant's stream is introduced with.
///
/// **External contract** — `text-cli.mjs:139`: `${BOLD}\u{1F916} ${RST}`.
pub const ASSISTANT_PREFIX: &str = "\u{1f916} ";

/// Run the client.
///
/// # Errors
///
/// * [`CliError::Refused`] when no Gateway answers and none can be started,
///   when the socket is refused, or when a command's request fails fatally.
/// * [`CliError::Io`] when stdout cannot be written.
pub async fn run(plan: &ChatPlan, host: &Host, out: &mut dyn Write) -> Result<(), CliError> {
    let locale = host.locale();
    let mut client = GatewayClient::new(&plan.url, locale)?;

    let mut started = None;
    if client.health().await?.is_none() {
        started = Some(autostart::start(plan, host, out, locale).await?);
        // The child publishes its own origin, which is the only way to learn
        // the port when the user asked for one that was taken.
        client = GatewayClient::new(&plan.url, locale)?;
        if client.health().await?.is_none() {
            return Err(CliError::refused_with(
                CODE_INVALID_ARGUMENT,
                locale,
                keys::CLI_GATEWAY_NOT_RUNNING,
                &[("url", &plan.url)],
            ));
        }
    }

    let outcome = converse(&client, plan, locale, out).await;
    if let Some(started) = started {
        started.stop().await;
    }
    outcome
}

/// The socket, and the loop around it.
async fn converse(
    client: &GatewayClient,
    plan: &ChatPlan,
    locale: Locale,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let url = protocol::websocket_url(&plan.url, &plan.session_id)
        .map_err(|error| refused(&error.to_string()))?;
    let mut request =
        tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(
            url.as_str(),
        )
        .map_err(|error| refused(&error.to_string()))?;
    if let Some(cookie) = client.cookie() {
        let value = cookie
            .parse()
            .map_err(|_| refused("the identity cookie is not a valid header value"))?;
        request
            .headers_mut()
            .insert(reqwest::header::COOKIE.as_str(), value);
    }
    let (socket, _response) = tokio_tungstenite::connect_async(request)
        .await
        .map_err(|error| refused(&error.to_string()))?;
    let (mut sink, mut stream) = socket.split();

    send(
        &mut sink,
        &protocol::connect_frame(plan.takeover, &time_zone(), locale),
    )
    .await?;
    line(
        out,
        &std::format!(
            "{}{}{}",
            style::DIM,
            format(
                locale,
                keys::CHAT_CONNECTED,
                &[("session", &plan.session_id)],
            ),
            style::RESET,
        ),
    )?;

    let mut screen = Screen::default();
    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    loop {
        tokio::select! {
            frame = stream.next() => {
                let Some(Ok(message)) = frame else {
                    line(out, &std::format!(
                        "{}{}{}",
                        style::RED,
                        t(locale, keys::CHAT_DISCONNECTED),
                        style::RESET,
                    ))?;
                    break;
                };
                let tokio_tungstenite::tungstenite::Message::Text(text) = message else {
                    continue;
                };
                // `try { event = JSON.parse(raw) } catch { return }` — an
                // unparseable frame is ignored, exactly as the Gateway ignores
                // an unparseable one from a client.
                let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) else {
                    continue;
                };
                if let Some(receipt) = screen.receipt(&event) {
                    send(&mut sink, &receipt).await?;
                }
                render(&event, locale, &mut screen, out)?;
            }
            input = lines.next_line() => {
                let Ok(Some(text)): std::io::Result<Option<String>> = input else { break };
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                if text.starts_with('/') {
                    let (command, argument) = protocol::split_command(text);
                    if protocol::is_exit_command(command) {
                        break;
                    }
                    command_line(client, plan, locale, command, argument, out).await?;
                    continue;
                }
                send(&mut sink, &serde_json::json!({
                    "type": via_protocol::GatewayClientEvent::InputMessage.as_str(),
                    "text": text,
                    "textOnly": true,
                })).await?;
            }
        }
    }

    let _ = sink.close().await;
    Ok(())
}

/// What the client is currently painting.
#[derive(Debug, Default)]
struct Screen {
    /// Whether an assistant stream is open and its prefix already written.
    streaming: bool,
    /// The responses a `playback.started` has been sent for.
    started: Vec<String>,
}

impl Screen {
    /// The playback receipt this frame calls for, if any.
    ///
    /// **External contract** — `text-cli.mjs:123-136`. A text client plays no
    /// audio and sends the receipts anyway, and upstream's comment says why:
    /// *"the gateway holds the assistant transcript back until
    /// `playback.started` — a design that keeps text in step with audio."*
    /// Without them the transcript never arrives, and the Injection Gate never
    /// learns the response was heard.
    fn receipt(&mut self, event: &serde_json::Value) -> Option<serde_json::Value> {
        let kind = event.get("type").and_then(serde_json::Value::as_str)?;
        let response_id = event
            .get("responseId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if response_id.is_empty() {
            return None;
        }
        if kind == GatewayServerEvent::AudioDelta.as_str() {
            if self.started.iter().any(|id| id == response_id) {
                return None;
            }
            self.started.push(response_id.to_owned());
            return Some(serde_json::json!({
                "type": via_protocol::GatewayClientEvent::PlaybackStarted.as_str(),
                "responseId": response_id,
            }));
        }
        if kind == GatewayServerEvent::AudioDone.as_str() {
            let before = self.started.len();
            self.started.retain(|id| id != response_id);
            if self.started.len() == before {
                return None;
            }
            return Some(serde_json::json!({
                "type": via_protocol::GatewayClientEvent::PlaybackEnded.as_str(),
                "responseId": response_id,
            }));
        }
        None
    }
}

/// Print one frame.
///
/// **External contract** — `text-cli.mjs:116-162`, arm for arm, plus the
/// task-plane arm VIA adds.
fn render(
    event: &serde_json::Value,
    locale: Locale,
    screen: &mut Screen,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let kind = event
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let role = event
        .get("role")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let content = event
        .get("content")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();

    if kind == GatewayServerEvent::TranscriptDelta.as_str() && role == "assistant" {
        if !screen.streaming {
            write(
                out,
                &std::format!("{}{ASSISTANT_PREFIX}{}", style::BOLD, style::RESET),
            )?;
            screen.streaming = true;
        }
        return write(out, content);
    }
    if kind == GatewayServerEvent::TranscriptFinal.as_str() && role == "assistant" {
        if screen.streaming {
            screen.streaming = false;
            return write(out, "\n");
        }
        if !content.is_empty() {
            return line(
                out,
                &std::format!("{}{ASSISTANT_PREFIX}{}{content}", style::BOLD, style::RESET),
            );
        }
        return Ok(());
    }
    if kind == GatewayServerEvent::TimelineInline.as_str() {
        let item = event.get("item");
        let body = item
            .and_then(|item| item.get("content"))
            .or_else(|| item.and_then(|item| item.get("markdown")))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if body.is_empty() {
            return Ok(());
        }
        return line(
            out,
            &std::format!(
                "{}{}{}\n{body}",
                style::CYAN,
                t(locale, keys::CHAT_TIMELINE),
                style::RESET,
            ),
        );
    }
    if kind == GatewayServerEvent::Error.as_str() {
        if screen.streaming {
            screen.streaming = false;
            write(out, "\n")?;
        }
        let message = event
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        return line(
            out,
            &std::format!(
                "{}{}{}",
                style::RED,
                format(locale, keys::CHAT_ERROR, &[("detail", message)]),
                style::RESET,
            ),
        );
    }
    // The Work plane. VIA's own — see the module documentation.
    if GatewayTaskEvent::from_wire(kind).is_some() {
        let task = event.get("task");
        let id = task
            .and_then(|task| task.get("id"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let status = task
            .and_then(|task| task.get("status"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if id.is_empty() {
            return Ok(());
        }
        return line(
            out,
            &std::format!(
                "{}{}{}",
                style::DIM,
                protocol::task_event_line(locale, id, status),
                style::RESET,
            ),
        );
    }
    Ok(())
}

/// Run one `/command`.
async fn command_line(
    client: &GatewayClient,
    plan: &ChatPlan,
    locale: Locale,
    command: &str,
    argument: Option<&str>,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    if command == protocol::HELP_COMMAND {
        return line(out, protocol::help(locale));
    }
    let outcome = match command {
        protocol::TASKS_COMMAND => tasks(client, plan, locale, out).await,
        protocol::CANCEL_COMMAND => cancel(client, plan, locale, argument, out).await,
        other => {
            return line(
                out,
                &std::format!(
                    "{}{}{}",
                    style::RED,
                    format(locale, keys::CHAT_UNKNOWN_COMMAND, &[("command", other)]),
                    style::RESET,
                ),
            );
        }
    };
    // `catch (error) { print('[命令失败] …') }` — a failed command reports and
    // the session continues. Only a failure to *write* ends the client.
    match outcome {
        Ok(()) => Ok(()),
        Err(CliError::Io { .. }) => outcome,
        Err(error) => line(
            out,
            &std::format!(
                "{}{}{}",
                style::RED,
                format(
                    locale,
                    keys::CHAT_COMMAND_FAILED,
                    &[("detail", &error.message(locale))],
                ),
                style::RESET,
            ),
        ),
    }
}

async fn tasks(
    client: &GatewayClient,
    plan: &ChatPlan,
    locale: Locale,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let tasks = client.tasks(&plan.session_id).await?;
    if tasks.is_empty() {
        return line(
            out,
            &std::format!(
                "{}{}{}",
                style::DIM,
                t(locale, keys::CHAT_NO_TASKS),
                style::RESET
            ),
        );
    }
    for task in &tasks {
        line(
            out,
            &std::format!(
                "  {}{}{}",
                style::YELLOW,
                protocol::task_line(task),
                style::RESET
            ),
        )?;
    }
    Ok(())
}

async fn cancel(
    client: &GatewayClient,
    plan: &ChatPlan,
    locale: Locale,
    argument: Option<&str>,
    out: &mut dyn Write,
) -> Result<(), CliError> {
    let tasks = client.tasks(&plan.session_id).await?;
    let Some(target) = protocol::select_cancellable(&tasks, argument) else {
        return line(
            out,
            &std::format!(
                "{}{}{}",
                style::RED,
                t(locale, keys::CHAT_NOTHING_TO_CANCEL),
                style::RESET,
            ),
        );
    };
    let id = target.id.clone();
    client.cancel(&id).await?;
    line(
        out,
        &std::format!(
            "{}{}{}",
            style::DIM,
            format(locale, keys::CHAT_CANCELLED, &[("id", &id)]),
            style::RESET,
        ),
    )
}

/// The client's IANA zone, or blank.
///
/// **External contract** — `text-cli.mjs:110`:
/// `Intl.DateTimeFormat().resolvedOptions().timeZone`. `via-conversation`
/// normalizes an unusable one to the host's zone and then to `UTC`, so a blank
/// value here is a supported answer rather than a failure.
fn time_zone() -> String {
    iana_time_zone::get_timezone().unwrap_or_default()
}

async fn send<S>(sink: &mut S, frame: &serde_json::Value) -> Result<(), CliError>
where
    S: SinkExt<tokio_tungstenite::tungstenite::Message> + Unpin,
    S::Error: std::fmt::Display,
{
    sink.send(tokio_tungstenite::tungstenite::Message::Text(
        frame.to_string().into(),
    ))
    .await
    .map_err(|error| refused(&error.to_string()))
}

fn write(out: &mut dyn Write, text: &str) -> Result<(), CliError> {
    out.write_all(text.as_bytes())
        .and_then(|()| out.flush())
        .map_err(|error| CliError::io("write", "<stdout>", error))
}

fn line(out: &mut dyn Write, text: &str) -> Result<(), CliError> {
    write(out, &std::format!("{text}\n"))
}

fn refused(message: &str) -> CliError {
    CliError::Refused {
        code: CODE_INVALID_ARGUMENT,
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn rendered(events: &[serde_json::Value]) -> String {
        let mut screen = Screen::default();
        let mut out = Vec::new();
        for event in events {
            render(event, Locale::En, &mut screen, &mut out).expect("a vec never fails");
        }
        String::from_utf8(out).expect("utf-8")
    }

    fn plain(text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars();
        while let Some(character) = chars.next() {
            if character == '\u{1b}' {
                for escape in chars.by_ref() {
                    if escape == 'm' {
                        break;
                    }
                }
                continue;
            }
            result.push(character);
        }
        result
    }

    #[test]
    fn an_assistant_stream_is_prefixed_once_and_closed_by_its_final() {
        let output = rendered(&[
            serde_json::json!({"type": "transcript.delta", "role": "assistant", "content": "On "}),
            serde_json::json!({"type": "transcript.delta", "role": "assistant", "content": "it."}),
            serde_json::json!({"type": "transcript.final", "role": "assistant", "content": "On it."}),
        ]);
        assert_eq!(plain(&output), std::format!("{ASSISTANT_PREFIX}On it.\n"));
    }

    #[test]
    fn a_final_with_no_stream_before_it_prints_the_whole_line() {
        let output = rendered(&[
            serde_json::json!({"type": "transcript.final", "role": "assistant", "content": "Done."}),
        ]);
        assert_eq!(plain(&output), std::format!("{ASSISTANT_PREFIX}Done.\n"));
    }

    #[test]
    fn a_user_transcript_is_not_echoed_back_at_the_user() {
        let output = rendered(&[
            serde_json::json!({"type": "transcript.delta", "role": "user", "content": "hello"}),
            serde_json::json!({"type": "transcript.final", "role": "user", "content": "hello"}),
        ]);
        assert_eq!(output, "");
    }

    #[test]
    fn an_error_closes_an_open_stream_before_it_prints() {
        let output = rendered(&[
            serde_json::json!({"type": "transcript.delta", "role": "assistant", "content": "hm"}),
            serde_json::json!({"type": "error", "message": "the provider refused"}),
        ]);
        let plain = plain(&output);
        assert!(
            plain.starts_with(&std::format!("{ASSISTANT_PREFIX}hm\n")),
            "{plain}"
        );
        assert!(plain.contains("the provider refused"), "{plain}");
    }

    #[test]
    fn a_timeline_block_takes_content_then_markdown() {
        let output = rendered(&[
            serde_json::json!({"type": "timeline.inline", "item": {"markdown": "# heading"}}),
        ]);
        assert!(plain(&output).contains("# heading"), "{output}");
        let empty = rendered(&[serde_json::json!({"type": "timeline.inline", "item": {}})]);
        assert_eq!(empty, "");
    }

    #[test]
    fn the_work_plane_prints_one_line_per_move() {
        let output = rendered(&[
            serde_json::json!({"type": "task.running", "task": {"id": "work_1", "status": "running"}}),
            serde_json::json!({"type": "task.completed", "task": {"id": "work_1", "status": "completed"}}),
        ]);
        let plain = plain(&output);
        assert!(plain.contains("work_1 running"), "{plain}");
        assert!(plain.contains("work_1 completed"), "{plain}");
    }

    #[test]
    fn a_task_frame_with_no_id_prints_nothing() {
        assert_eq!(
            rendered(&[serde_json::json!({"type": "task.running", "task": {}})]),
            "",
        );
    }

    #[test]
    fn a_frame_outside_the_vocabulary_prints_nothing() {
        assert_eq!(
            rendered(&[
                serde_json::json!({"type": "voice.ownership", "state": "available"}),
                serde_json::json!({"type": "nonsense"}),
                serde_json::json!({}),
            ]),
            "",
        );
    }

    #[test]
    fn one_playback_started_per_response_and_one_ended_after_it() {
        let mut screen = Screen::default();
        let delta = serde_json::json!({"type": "audio.delta", "responseId": "resp_1"});
        let first = screen
            .receipt(&delta)
            .expect("the first delta starts playback");
        assert_eq!(first["type"], "playback.started");
        assert_eq!(first["responseId"], "resp_1");
        assert!(
            screen.receipt(&delta).is_none(),
            "a second delta for the same response must not re-announce playback",
        );

        let done = serde_json::json!({"type": "audio.done", "responseId": "resp_1"});
        let ended = screen.receipt(&done).expect("the done ends playback");
        assert_eq!(ended["type"], "playback.ended");
        assert!(
            screen.receipt(&done).is_none(),
            "a second done has nothing left to end",
        );
    }

    #[test]
    fn a_done_for_a_response_that_never_started_is_ignored() {
        let mut screen = Screen::default();
        assert!(
            screen
                .receipt(&serde_json::json!({"type": "audio.done", "responseId": "resp_9"}))
                .is_none(),
        );
    }

    #[test]
    fn a_receipt_needs_a_response_id() {
        let mut screen = Screen::default();
        assert!(
            screen
                .receipt(&serde_json::json!({"type": "audio.delta"}))
                .is_none()
        );
        assert!(
            screen
                .receipt(&serde_json::json!({"type": "audio.delta", "responseId": ""}))
                .is_none()
        );
    }
}
