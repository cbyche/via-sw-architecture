use serde_json::{Value, json};
use std::io;
use std::time::Instant;
use via_dp00_executable_v3::{
    JsonChild, append_array, local_bounded_result, monotonic_ns, process_events, serve,
};

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
        let capability = sem
            .pointer("/semantic/capability")
            .and_then(Value::as_str)
            .unwrap_or("");
        let read_only = sem
            .pointer("/semantic/read_only")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let local = read_only && capability == "bounded-read";
        let dispatch_offset_ns = monotonic_ns(start);
        let (agent, agent_tx, agent_rx, selected_role, ipc_count, hops) = if local {
            (
                local_bounded_result(&request),
                0,
                0,
                "r1-via-bounded-read-tactic",
                2,
                1,
            )
        } else {
            let selected = if capability.contains("specialist") {
                &mut specialist
            } else {
                &mut general
            };
            let role = selected.role;
            let (answer, tx, rx) = selected.request(&request).expect("agent IPC");
            (answer, tx, rx, role, 3, 2)
        };
        let feedback_emitted_offset_ns =
            dispatch_offset_ns + agent["elapsed_ns"].as_u64().unwrap_or(0) as u128;
        let mut result = json!({
            "case_id":request.get("case_id").cloned().unwrap_or(Value::Null),
            "entry_component":"r1-via-control-plane-with-bounded-read-tactic",
            "topology":{"root_pid":parent,"processes":[
                {"role":"r1-via-control-plane-with-bounded-read-tactic","pid":parent,"parent_pid":null},
                {"role":"semantic-model","pid":semantic.pid(),"parent_pid":parent},
                {"role":"policy-engine","pid":policy.pid(),"parent_pid":parent},
                {"role":"r1-general-agent","pid":general.pid(),"parent_pid":parent},
                {"role":"specialist-agent","pid":specialist.pid(),"parent_pid":parent}
            ],"selected_path":["scenario-driver","r1-via-control-plane-with-bounded-read-tactic",selected_role]},
            "state_owner":selected_role,
            "via_state":{"user_task":request.get("task_id"),"result_binding":true,"workflow_truth":local,"bounded_read_only":local,"event_state":process_events(&request)},
            "agent":agent,
            "semantic":sem["semantic"],
            "policy":grant,
            "scope_crossings":[{"boundary":"VIA -> R1 Agent or bounded tactic","principal":request.get("principal"),"purpose":request.get("purpose"),"task":request.get("task_id"),"approval_version":request.pointer("/policy_context/approval_version"),"scopes":grant["granted"]}],
            "elapsed_ns":monotonic_ns(start),
            "ipc_count":ipc_count,
            "process_hop_count":hops,
            "serialization_bytes":sem_tx+sem_rx+pol_tx+pol_rx+agent_tx+agent_rx,
            "recovery":{"fault_target":recovery_target,"safe_continuation":true,"recovery_ns":monotonic_ns(start)},
            "feedback":{"emitted_offset_ns":feedback_emitted_offset_ns,"delivered_offset_ns":monotonic_ns(start)},
            "trace":[
                {"span":"interaction","parent":null,"component":"r1-via-control-plane-with-bounded-read-tactic"},
                {"span":"semantic","parent":"interaction","component":"semantic-model"},
                {"span":"policy","parent":"semantic","component":"policy-engine"},
                {"span":"dispatch","parent":"policy","component":selected_role},
                {"span":"result.bind","parent":"agent.result","component":"r1-via-control-plane-with-bounded-read-tactic"},
                {"span":"delivery","parent":"result.bind","component":"r1-via-control-plane-with-bounded-read-tactic"}
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
