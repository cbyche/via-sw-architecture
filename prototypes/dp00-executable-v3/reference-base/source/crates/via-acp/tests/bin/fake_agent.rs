//! A scripted ACP agent: a real process speaking NDJSON on stdio.
//!
//! This is the fixture `tests/process_client.rs` runs the client against. It is
//! a separate executable on purpose. A mock transport would exercise the
//! client's own code and nothing else; the interesting failures in a process
//! client are the ones that live *between* the processes — framing, an
//! inherited environment variable, a child that ignores `SIGTERM`, a stdout
//! that closes before the child does. None of those can happen in-process.
//!
//! # The script
//!
//! Behaviour is decided entirely by environment variables, because that is also
//! how the credential boundary is observed: an agent that can only see what was
//! deliberately projected into it is an agent that can *report* what it saw.
//!
//! | Variable | Effect |
//! | --- | --- |
//! | `FAKE_PROTOCOL_VERSION` | the `protocolVersion` to answer `initialize` with (default `1`) |
//! | `FAKE_CAPABILITIES` | JSON deep-merged into `agentCapabilities` |
//! | `FAKE_STDERR` | written to stderr at start-up |
//! | `FAKE_EXIT_BEFORE_INITIALIZE` | exit with this code before answering anything |
//! | `FAKE_SLEEP_MS` | delay before answering `session/prompt` |
//! | `FAKE_UPDATES` | JSON array of `session/update` payloads emitted before a prompt is answered |
//! | `FAKE_STOP_REASON` | the `stopReason` to answer a prompt with (default `end_turn`) |
//! | `FAKE_PROMPT_ERROR` | answer `session/prompt` with this JSON-RPC error message instead |
//! | `FAKE_ERROR_DETAILS` | `data.details` on that error |
//! | `FAKE_REQUEST_PERMISSION` | JSON option array; asks for permission and waits for the reply |
//! | `FAKE_IGNORE_SIGTERM` | install a `SIGTERM` handler that does nothing, and park rather than exit on stdin EOF |
//! | `FAKE_ECHO_ENV` | comma-separated names, reported back as the prompt's text |
//! | `FAKE_SESSION_PREFIX` | prefix for minted session ids (default `fake-session`) |
//! | `FAKE_PID_FILE` | write this process's pid here at start-up, so a test can prove it died |
//!
//! Nothing here names a backend: the fixture is a generic ACP agent, which is
//! the only kind this crate is allowed to know about.

use std::io::{BufRead as _, Write as _};

use serde_json::{Map, Value, json};

/// One prompt that is waiting for a permission answer before it may reply.
struct PendingPrompt {
    id: Value,
    session_id: Value,
}

/// Run the scripted agent until stdin closes.
fn main() {
    if let Some(text) = env("FAKE_STDERR") {
        eprintln!("{text}");
        let _ = std::io::stderr().flush();
    }

    if let Some(path) = env("FAKE_PID_FILE") {
        let _ = std::fs::write(path, std::process::id().to_string());
    }

    if let Some(code) = env("FAKE_EXIT_BEFORE_INITIALIZE") {
        std::process::exit(code.parse().unwrap_or(1));
    }

    #[cfg(unix)]
    if env("FAKE_IGNORE_SIGTERM").is_some() {
        ignore_sigterm();
    }

    let stdin = std::io::stdin();
    let mut sessions = 0_u32;
    let mut pending: Option<PendingPrompt> = None;

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }
        let Ok(message) = serde_json::from_str::<Value>(&line) else {
            continue;
        };

        let Some(method) = message.get("method").and_then(Value::as_str) else {
            // A response to the permission request we sent: the prompt that was
            // waiting on it may now answer.
            if let Some(prompt) = pending.take() {
                let outcome = message
                    .get("result")
                    .and_then(|result| result.get("outcome"))
                    .cloned()
                    .unwrap_or(Value::Null);
                finish_prompt(Some(prompt.id), &prompt.session_id, Some(outcome));
            }
            continue;
        };
        let id = message.get("id").cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);

        match method {
            "initialize" => respond(id, initialize_result()),
            "session/new" | "session/resume" | "session/load" => {
                sessions += 1;
                let prefix =
                    env("FAKE_SESSION_PREFIX").unwrap_or_else(|| "fake-session".to_owned());
                let session_id = params
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .map_or_else(|| format!("{prefix}-{sessions}"), str::to_owned);
                respond(id, json!({ "sessionId": session_id }));
            }
            "session/list" => respond(id, list_result(&params)),
            "session/prompt" => {
                let session_id = params.get("sessionId").cloned().unwrap_or(Value::Null);
                if let Some(options) = env("FAKE_REQUEST_PERMISSION")
                    .and_then(|value| serde_json::from_str::<Value>(&value).ok())
                {
                    ask_permission(&session_id, &options);
                    if let Some(id) = id {
                        pending = Some(PendingPrompt { id, session_id });
                    }
                    continue;
                }
                finish_prompt(id, &session_id, None);
            }
            "session/set_config_option" => respond(id, json!({ "configOptions": [] })),
            "session/set_model" | "session/close" => respond(id, json!({})),
            // A notification: no reply, ever.
            "session/cancel" => {}
            _ => {
                if id.is_some() {
                    write_line(&json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": "Method not found" },
                    }));
                }
            }
        }
    }

    // A wrapper launcher that exits and leaves the real agent running is the
    // case the teardown ladder exists for: stdin EOF is not enough, and neither
    // is SIGTERM. Park until something escalates.
    if env("FAKE_IGNORE_SIGTERM").is_some() {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn initialize_result() -> Value {
    let version: u16 = env("FAKE_PROTOCOL_VERSION")
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    let mut capabilities = json!({
        "loadSession": false,
        "promptCapabilities": { "image": false, "audio": false, "embeddedContext": false },
        "sessionCapabilities": {},
    });
    if let Some(extra) =
        env("FAKE_CAPABILITIES").and_then(|value| serde_json::from_str::<Value>(&value).ok())
    {
        merge(&mut capabilities, &extra);
    }
    json!({
        "protocolVersion": version,
        "agentInfo": { "name": "fake-acp-agent", "version": "0.0.1" },
        "agentCapabilities": capabilities,
    })
}

fn merge(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target), Value::Object(source)) => {
            for (key, value) in source {
                match target.get_mut(key) {
                    Some(existing) if existing.is_object() && value.is_object() => {
                        merge(existing, value);
                    }
                    _ => {
                        target.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (target, source) => *target = source.clone(),
    }
}

/// Two pages of sessions, echoing the requested `_meta.limit` back so the
/// caller's pagination arithmetic is observable from outside the process.
fn list_result(params: &Value) -> Value {
    let limit = params
        .get("_meta")
        .and_then(|meta| meta.get("limit"))
        .and_then(Value::as_u64)
        .unwrap_or(1);
    let cursor = params
        .get("cursor")
        .and_then(Value::as_str)
        .unwrap_or("page-0")
        .to_owned();
    let page: u64 = cursor
        .rsplit('-')
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    let sessions: Vec<Value> = (0..limit)
        .map(|index| {
            json!({
                "sessionId": format!("s-{page}-{index}"),
                "cwd": "/work",
                "title": format!("Session {page}-{index}"),
                "updatedAt": "2026-08-22T00:00:00Z",
                "_meta": { "requestedLimit": limit },
            })
        })
        .collect();
    let next = if page < 1 {
        Value::String(format!("page-{}", page + 1))
    } else {
        Value::Null
    };
    json!({ "sessions": sessions, "nextCursor": next })
}

fn ask_permission(session_id: &Value, options: &Value) {
    write_line(&json!({
        "jsonrpc": "2.0",
        "id": "perm-1",
        "method": "session/request_permission",
        "params": {
            "sessionId": session_id,
            "toolCall": {
                "toolCallId": "tool-1",
                "name": "shell",
                "title": "Run something",
                "rawInput": { "command": "true", "description": "a description" },
            },
            "options": options,
        },
    }));
}

fn finish_prompt(id: Option<Value>, session_id: &Value, outcome: Option<Value>) {
    if let Some(updates) =
        env("FAKE_UPDATES").and_then(|value| serde_json::from_str::<Vec<Value>>(&value).ok())
    {
        for update in updates {
            write_line(&json!({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": { "sessionId": session_id, "update": update },
            }));
        }
    }

    if let Some(names) = env("FAKE_ECHO_ENV") {
        let mut seen = Map::new();
        for name in names.split(',').map(str::trim).filter(|n| !n.is_empty()) {
            seen.insert(
                name.to_owned(),
                std::env::var(name).map_or(Value::Null, Value::String),
            );
        }
        write_line(&json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": session_id,
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": { "type": "text", "text": Value::Object(seen).to_string() },
                },
            },
        }));
    }

    if let Some(millis) = env("FAKE_SLEEP_MS").and_then(|value| value.parse::<u64>().ok()) {
        std::thread::sleep(std::time::Duration::from_millis(millis));
    }

    if let Some(message) = env("FAKE_PROMPT_ERROR") {
        let mut error = json!({ "code": -32603, "message": message });
        if let Some(details) = env("FAKE_ERROR_DETAILS") {
            error["data"] = json!({ "details": details });
        }
        write_line(&json!({ "jsonrpc": "2.0", "id": id, "error": error }));
        return;
    }

    let stop_reason = env("FAKE_STOP_REASON").unwrap_or_else(|| "end_turn".to_owned());
    let mut result = Map::new();
    result.insert("stopReason".to_owned(), Value::String(stop_reason));
    if let Some(outcome) = outcome {
        result.insert("_meta".to_owned(), json!({ "permissionOutcome": outcome }));
    }
    respond(id, Value::Object(result));
}

fn respond(id: Option<Value>, result: Value) {
    let Some(id) = id else { return };
    write_line(&json!({ "jsonrpc": "2.0", "id": id, "result": result }));
}

fn write_line(message: &Value) {
    let mut stdout = std::io::stdout();
    let _ = writeln!(stdout, "{message}");
    let _ = stdout.flush();
}

/// Install a `SIGTERM` handler that does nothing.
///
/// The fixture that proves the teardown ladder escalates rather than hanging.
/// `tokio`'s signal API is used because it is safe; `nix::sigaction` is
/// `unsafe`, and this crate forbids `unsafe`.
#[cfg(unix)]
fn ignore_sigterm() {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        runtime.block_on(async move {
            let Ok(mut stream) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            else {
                return;
            };
            let _ = ready_tx.send(());
            loop {
                stream.recv().await;
            }
        });
    });
    // Do not start reading stdin until the handler is actually installed, or a
    // fast test could signal into the default disposition.
    let _ = ready_rx.recv_timeout(std::time::Duration::from_secs(5));
}
