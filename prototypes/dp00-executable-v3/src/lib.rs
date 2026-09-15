#![forbid(unsafe_code)]

pub mod ownership;

use serde_json::{Value, json};
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

pub struct JsonChild {
    pub child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
    pub role: &'static str,
}

impl JsonChild {
    pub fn spawn(binary: &str, role: &'static str) -> io::Result<Self> {
        let path = sibling_binary(binary)?;
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = BufWriter::new(child.stdin.take().expect("piped stdin"));
        let output = BufReader::new(child.stdout.take().expect("piped stdout"));
        Ok(Self {
            child,
            input,
            output,
            role,
        })
    }

    pub fn request(&mut self, value: &Value) -> io::Result<(Value, usize, usize)> {
        let encoded = serde_json::to_vec(value)?;
        self.input.write_all(&encoded)?;
        self.input.write_all(b"\n")?;
        self.input.flush()?;
        let mut response = String::new();
        self.output.read_line(&mut response)?;
        if response.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("{} exited", self.role),
            ));
        }
        let value: Value = serde_json::from_str(&response)?;
        Ok((value, encoded.len() + 1, response.len()))
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn terminate(&mut self) -> io::Result<()> {
        self.child.kill()?;
        let _ = self.child.wait();
        Ok(())
    }

    pub fn restart(&mut self, binary: &str, role: &'static str) -> io::Result<()> {
        self.terminate()?;
        *self = Self::spawn(binary, role)?;
        Ok(())
    }
}

impl Drop for JsonChild {
    fn drop(&mut self) {
        let _ = self.input.write_all(b"{\"control\":\"shutdown\"}\n");
        let _ = self.input.flush();
        let _ = self.child.wait();
    }
}

fn sibling_binary(name: &str) -> io::Result<PathBuf> {
    let current = env::current_exe()?;
    let dir = current
        .parent()
        .ok_or_else(|| io::Error::other("executable has no parent"))?;
    Ok(dir.join(name))
}

pub fn serve(mut handler: impl FnMut(Value) -> Value) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = BufWriter::new(io::stdout());
    for line in stdin.lock().lines() {
        let line = line?;
        let request: Value = serde_json::from_str(&line)?;
        if request.get("control").and_then(Value::as_str) == Some("shutdown") {
            break;
        }
        serde_json::to_writer(&mut stdout, &handler(request))?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
    }
    Ok(())
}

pub fn monotonic_ns(start: Instant) -> u128 {
    start.elapsed().as_nanos()
}

pub fn common_delay(request: &Value, key: &str) {
    let ms = request
        .get("delays")
        .and_then(|v| v.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if ms > 0 {
        std::thread::sleep(Duration::from_millis(ms));
    }
}

pub fn policy_decision(request: &Value) -> Value {
    let required = request
        .get("scope_request")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let principal = request
        .get("principal")
        .and_then(Value::as_str)
        .unwrap_or("principal-local");
    let task = request
        .get("task_id")
        .and_then(Value::as_str)
        .unwrap_or("task:unknown");
    let resource_kind = request
        .pointer("/policy_context/resource_kind")
        .and_then(Value::as_str);
    let object_id = request
        .pointer("/policy_context/object_id")
        .and_then(Value::as_str);
    let authorized_resource = resource_kind
        .zip(object_id)
        .map(|(kind, object)| format!("scope:{kind}:{object}"));
    let granted: Vec<Value> = required
        .into_iter()
        .filter(|scope| {
            let text = scope.as_str().unwrap_or("");
            !text.contains("unrelated-principal")
                && (text.contains(task)
                    || text.contains(principal)
                    || authorized_resource
                        .as_deref()
                        .is_some_and(|allowed| text == allowed)
                    || (authorized_resource.is_none() && !text.contains("principal:")))
        })
        .collect();
    json!({"granted": granted, "principal": principal, "task": task, "decision": "least-privilege"})
}

pub fn agent_result(mut request: Value, role: &str) -> Value {
    let start = Instant::now();
    common_delay(&request, "agent_ms");
    if request.get("fault").and_then(Value::as_str) == Some("close-ipc") {
        std::process::exit(86);
    }
    let task = request.get("task_id").cloned().unwrap_or(Value::Null);
    let replay = request
        .get_mut("agent_replay")
        .map(Value::take)
        .unwrap_or(Value::Null);
    let event_state = process_events(&request);
    persist_workflow(role, &request, &event_state).expect("persist workflow state");
    let feedback_event = match request.get("feedback_event_class").and_then(Value::as_str) {
        Some("accepted_queued") => "accepted",
        Some("meaningful_progress") => "progress",
        Some("approval_needed") => "approval-needed",
        Some("blocked_failure") => "failure",
        _ => "completion",
    };
    json!({
        "component": role,
        "pid": std::process::id(),
        "task_id": task,
        "workflow_owner": role,
        "workflow_state": {"status":"completed", "version":1, "durably_written":true, "event_state":event_state},
        "events": [feedback_event],
        "authoritative_event":feedback_event,
        "result": replay,
        "elapsed_ns": monotonic_ns(start),
        "trace": [
            {"span":"agent.accept", "parent":"dispatch"},
            {"span":"agent.execute", "parent":"agent.accept"},
            {"span":"agent.result", "parent":"agent.execute"}
        ]
    })
}

pub fn persist_workflow(role: &str, request: &Value, state: &Value) -> io::Result<()> {
    let directory = env::var_os("VIA_V3_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{role}-workflow.ndjson"));
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(
        &mut file,
        &json!({
            "task_id":request.get("task_id"),
            "state":state,
            "pid":std::process::id()
        }),
    )?;
    file.write_all(b"\n")?;
    file.sync_data()?;
    Ok(())
}

pub fn semantic_result(request: &Value) -> Value {
    common_delay(request, "semantic_ms");
    json!({
        "component":"semantic-model",
        "pid":std::process::id(),
        "semantic":request.get("semantic_replay").cloned().unwrap_or(Value::Null),
        "event":"interpreted"
    })
}

pub fn append_array(target: &mut Value, key: &str, values: impl IntoIterator<Item = Value>) {
    let array = target
        .as_object_mut()
        .unwrap()
        .entry(key)
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .unwrap();
    array.extend(values);
}

pub fn process_events(request: &Value) -> Value {
    let mut tasks = serde_json::Map::new();
    let events = request
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for (sequence, event) in events.iter().enumerate() {
        let kind = event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("TEXT_TURN");
        let task = event
            .get("task_id")
            .and_then(Value::as_str)
            .or_else(|| request.get("task_id").and_then(Value::as_str))
            .unwrap_or("task:unknown");
        let record = tasks.entry(task.to_string()).or_insert_with(|| json!({
            "status":"created", "result_version":0, "approval_version":0, "cancelled":false, "events":[]
        }));
        record["events"]
            .as_array_mut()
            .unwrap()
            .push(json!({"sequence":sequence,"type":kind}));
        match kind {
            "TASK_START" | "TURN_COMMIT" | "TEXT_TURN" => record["status"] = json!("running"),
            "AGENT_APPROVAL_REQUIRED" => {
                record["status"] = json!("approval-required");
                record["approval_version"] = json!(1);
            }
            "APPROVAL_RESPONSE" => record["status"] = json!("running"),
            "AGENT_RESULT" => {
                record["status"] = json!("completed");
                record["result_version"] = json!(1);
            }
            "LATE_AGENT_RESULT" => record["late_result_rejected"] = json!(true),
            "CANCEL" => {
                record["status"] = json!("cancelled");
                record["cancelled"] = json!(true);
            }
            "FOLLOW_UP" => record["follow_up_correlated"] = json!(true),
            "VOICE_REVISION" | "CONTEXT_VERSION_CHANGE" => record["revision_applied"] = json!(true),
            "CLARIFICATION_RESPONSE" => record["clarification_resolved"] = json!(true),
            "RESOURCE_ACQUIRE" => record["resource_held"] = json!(true),
            "RESOURCE_RELEASE" => record["resource_held"] = json!(false),
            "BARGE_IN" => record["playback_cancelled"] = json!(true),
            "FAULT" => record["fault_observed"] = json!(true),
            "RECOVERY" => record["recovery_ready"] = json!(true),
            _ => {}
        }
    }
    Value::Object(tasks)
}

pub fn local_bounded_result(request: &Value) -> Value {
    common_delay(request, "tool_ms");
    json!({
        "component":"r1-via-bounded-read-tactic",
        "pid":std::process::id(),
        "task_id":request.get("task_id").cloned().unwrap_or(Value::Null),
        "workflow_owner":"r1-via-bounded-read-tactic",
        "workflow_state":{"status":"completed","version":1,"durably_written":false},
        "events":["accepted","completion"],
        "result":request.get("agent_replay").cloned().unwrap_or(Value::Null),
        "trace":[{"span":"local-read.execute","parent":"semantic"}]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_denies_unrelated_principal() {
        let request = json!({"principal":"p1","task_id":"task:t1","scope_request":["scope:task:task:t1","scope:unrelated-principal:*"]});
        let decision = policy_decision(&request);
        let granted = decision["granted"].as_array().unwrap();
        assert_eq!(granted, &vec![json!("scope:task:task:t1")]);
    }

    #[test]
    fn policy_limits_resource_object() {
        let request = json!({
            "principal":"p1", "task_id":"task:t1",
            "policy_context":{"resource_kind":"file-scope","object_id":"object-0"},
            "scope_request":["scope:file-scope:object-0","scope:file-scope:object-1","scope:task:task:t1"]
        });
        let decision = policy_decision(&request);
        assert_eq!(
            decision["granted"],
            json!(["scope:file-scope:object-0", "scope:task:task:t1"])
        );
    }

    #[test]
    fn event_order_drives_state() {
        let request = json!({"task_id":"task:t","events":[{"type":"TASK_START"},{"type":"CANCEL"},{"type":"LATE_AGENT_RESULT"}]});
        let state = process_events(&request);
        assert_eq!(state["task:t"]["status"], "cancelled");
        assert_eq!(state["task:t"]["late_result_rejected"], true);
    }

    #[test]
    fn tactic_is_read_only_and_non_durable() {
        let result = local_bounded_result(&json!({"task_id":"task:t","agent_replay":{"facts":[]}}));
        assert_eq!(result["workflow_state"]["durably_written"], false);
    }

    #[test]
    fn ownership_surfaces_are_linked() {
        assert_eq!(ownership::common_regression_probe(), 14);
    }
}
