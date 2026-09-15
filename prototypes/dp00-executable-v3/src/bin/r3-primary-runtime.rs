use serde_json::{Value, json};
use std::io;
use std::time::Instant;
use via_dp00_executable_v3::{
    JsonChild, agent_result, append_array, monotonic_ns, persist_workflow, process_events, serve,
};

fn main() -> io::Result<()> {
    let parent = std::process::id();
    let mut specialist = JsonChild::spawn("specialist-agent", "specialist-agent")?;
    serve(move |request| {
        let start = Instant::now();
        let capability = request
            .pointer("/semantic_replay/capability")
            .and_then(Value::as_str)
            .unwrap_or("");
        let (execution, tx, rx, owner, specialist_selected) = if capability.contains("specialist") {
            let (answer, tx, rx) = specialist.request(&request).expect("specialist IPC");
            (answer, tx, rx, "specialist-agent", true)
        } else {
            (
                agent_result(request.clone(), "r3-primary-runtime"),
                0,
                0,
                "r3-primary-runtime",
                false,
            )
        };
        let workflow_events = process_events(&request);
        persist_workflow("r3-primary-runtime", &request, &workflow_events)
            .expect("persist primary workflow");
        let mut response = json!({
            "component":"r3-primary-runtime",
            "pid":parent,
            "specialist_pid":specialist.pid(),
            "specialist_selected":specialist_selected,
            "task_id":request.get("task_id"),
            "workflow_owner":"r3-primary-runtime",
            "execution_owner":owner,
            "workflow_state":{"status":"completed","version":1,"durably_written":true,"specialist_delegation":capability.contains("specialist"),"event_state":workflow_events},
            "execution":execution,
            "ipc_count":if specialist_selected{1}else{0},
            "process_hop_count":if specialist_selected{1}else{0},
            "serialization_bytes":tx+rx,
            "elapsed_ns":monotonic_ns(start),
            "trace":[
                {"span":"primary.interpret","parent":"dispatch","component":"r3-primary-runtime"},
                {"span":"primary.plan","parent":"primary.interpret","component":"r3-primary-runtime"},
                {"span":"agent.accept","parent":"primary.plan","component":owner},
                {"span":"agent.execute","parent":"agent.accept","component":owner},
                {"span":"agent.result","parent":"agent.execute","component":owner}
            ]
        });
        if specialist_selected {
            append_array(
                &mut response,
                "trace",
                execution["trace"].as_array().cloned().unwrap_or_default(),
            );
        }
        response
    })
}
