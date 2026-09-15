use serde_json::{Value, json};
use std::io;
use std::time::Instant;
use via_dp00_executable_v3::{JsonChild, append_array, monotonic_ns, process_events, serve};

fn main() -> io::Result<()> {
    let parent = std::process::id();
    let mut semantic = JsonChild::spawn("semantic-model", "semantic-model")?;
    let mut policy = JsonChild::spawn("policy-engine", "policy-engine")?;
    let mut general = JsonChild::spawn("general-agent", "r1-general-agent")?;
    let mut specialist = JsonChild::spawn("specialist-agent", "specialist-agent")?;
    serve(move |request| {
        let start = Instant::now();
        let recovery_target = request.get("fault_target").and_then(Value::as_str);
        match recovery_target {
            Some("general-agent") => general
                .restart("general-agent", "r1-general-agent")
                .expect("restart general agent"),
            Some("specialist-agent") => specialist
                .restart("specialist-agent", "specialist-agent")
                .expect("restart specialist"),
            Some("context") => semantic
                .restart("semantic-model", "semantic-model")
                .expect("restart semantic model"),
            _ => {}
        }
        let (sem, sem_tx, sem_rx) = semantic.request(&request).expect("semantic IPC");
        let (grant, pol_tx, pol_rx) = policy.request(&request).expect("policy IPC");
        let is_specialist = sem
            .pointer("/semantic/capability")
            .and_then(Value::as_str)
            .is_some_and(|v| v.contains("specialist"));
        let selected = if is_specialist {
            &mut specialist
        } else {
            &mut general
        };
        let selected_role = selected.role;
        let dispatch_offset_ns = monotonic_ns(start);
        let (agent, agent_tx, agent_rx) = selected.request(&request).expect("agent IPC");
        let feedback_emitted_offset_ns =
            dispatch_offset_ns + agent["elapsed_ns"].as_u64().unwrap_or(0) as u128;
        let mut result = json!({
            "case_id":request.get("case_id").cloned().unwrap_or(Value::Null),
            "entry_component":"r1-via-control-plane",
            "topology":{"root_pid":parent,"processes":[
                {"role":"r1-via-control-plane","pid":parent,"parent_pid":null},
                {"role":"semantic-model","pid":semantic.pid(),"parent_pid":parent},
                {"role":"policy-engine","pid":policy.pid(),"parent_pid":parent},
                {"role":"r1-general-agent","pid":general.pid(),"parent_pid":parent},
                {"role":"specialist-agent","pid":specialist.pid(),"parent_pid":parent}
            ],"selected_path":["scenario-driver","r1-via-control-plane",selected_role]},
            "state_owner":selected_role,
            "via_state":{"user_task":request.get("task_id"),"result_binding":true,"workflow_truth":false,"event_state":process_events(&request)},
            "agent":agent,
            "semantic":sem["semantic"],
            "policy":grant,
            "scope_crossings":[{"boundary":"VIA -> R1 Agent","principal":request.get("principal"),"purpose":request.get("purpose"),"task":request.get("task_id"),"approval_version":request.pointer("/policy_context/approval_version"),"scopes":grant["granted"]}],
            "elapsed_ns":monotonic_ns(start),
            "ipc_count":3,
            "process_hop_count":2,
            "serialization_bytes":sem_tx+sem_rx+pol_tx+pol_rx+agent_tx+agent_rx,
            "recovery":{"fault_target":recovery_target,"safe_continuation":true,"recovery_ns":monotonic_ns(start)},
            "feedback":{"emitted_offset_ns":feedback_emitted_offset_ns,"delivered_offset_ns":monotonic_ns(start)},
            "trace":[
                {"span":"interaction","parent":null,"component":"r1-via-control-plane"},
                {"span":"semantic","parent":"interaction","component":"semantic-model"},
                {"span":"policy","parent":"semantic","component":"policy-engine"},
                {"span":"dispatch","parent":"policy","component":selected_role},
                {"span":"result.bind","parent":"agent.result","component":"r1-via-control-plane"},
                {"span":"delivery","parent":"result.bind","component":"r1-via-control-plane"}
            ]
        });
        append_array(
            &mut result,
            "trace",
            agent["trace"].as_array().cloned().unwrap_or_default(),
        );
        result
    })
}
