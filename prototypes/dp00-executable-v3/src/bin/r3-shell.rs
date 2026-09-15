use serde_json::{Value, json};
use std::io;
use std::time::Instant;
use via_dp00_executable_v3::{JsonChild, append_array, monotonic_ns, process_events, serve};

fn main() -> io::Result<()> {
    let parent = std::process::id();
    let mut semantic = JsonChild::spawn("semantic-model", "semantic-model")?;
    let mut policy = JsonChild::spawn("policy-engine", "policy-engine")?;
    let mut primary = JsonChild::spawn("r3-primary-runtime", "r3-primary-runtime")?;
    serve(move |mut request| {
        let start = Instant::now();
        let recovery_target = request
            .get("fault_target")
            .and_then(Value::as_str)
            .map(str::to_owned);
        match recovery_target.as_deref() {
            Some("primary-runtime") | Some("specialist-agent") => primary
                .restart("r3-primary-runtime", "r3-primary-runtime")
                .expect("restart primary topology"),
            Some("context") => semantic
                .restart("semantic-model", "semantic-model")
                .expect("restart semantic model"),
            _ => {}
        }
        let (sem, sem_tx, sem_rx) = semantic.request(&request).expect("semantic IPC");
        request["semantic_replay"] = sem["semantic"].clone();
        let (grant, pol_tx, pol_rx) = policy.request(&request).expect("policy IPC");
        let dispatch_offset_ns = monotonic_ns(start);
        let (runtime, run_tx, run_rx) = primary.request(&request).expect("primary IPC");
        let feedback_emitted_offset_ns =
            dispatch_offset_ns + runtime["elapsed_ns"].as_u64().unwrap_or(0) as u128;
        let specialist_pid = runtime
            .get("specialist_pid")
            .cloned()
            .unwrap_or(Value::Null);
        let specialist_selected = runtime["specialist_selected"].as_bool().unwrap_or(false);
        let mut processes = vec![
            json!({"role":"r3-via-shell","pid":parent,"parent_pid":null}),
            json!({"role":"semantic-model","pid":semantic.pid(),"parent_pid":parent}),
            json!({"role":"policy-engine","pid":policy.pid(),"parent_pid":parent}),
            json!({"role":"r3-primary-runtime","pid":primary.pid(),"parent_pid":parent}),
        ];
        processes.push(
            json!({"role":"specialist-agent","pid":specialist_pid,"parent_pid":primary.pid()}),
        );
        let mut selected_path = vec![
            json!("scenario-driver"),
            json!("r3-via-shell"),
            json!("r3-primary-runtime"),
        ];
        if specialist_selected {
            selected_path.push(json!("specialist-agent"));
        }
        let mut scope_crossings = vec![
            json!({"boundary":"VIA Shell -> R3 Primary Runtime","principal":request.get("principal"),"purpose":request.get("purpose"),"task":request.get("task_id"),"approval_version":request.pointer("/policy_context/approval_version"),"scopes":grant["granted"]}),
        ];
        if specialist_selected {
            scope_crossings.push(json!({"boundary":"R3 Primary Runtime -> Specialist","principal":request.get("principal"),"purpose":request.get("purpose"),"task":request.get("task_id"),"approval_version":request.pointer("/policy_context/approval_version"),"scopes":grant["granted"]}));
        }
        let mut result = json!({
            "case_id":request.get("case_id").cloned().unwrap_or(Value::Null),
            "entry_component":"r3-via-shell",
            "topology":{"root_pid":parent,"processes":processes,"selected_path":selected_path},
            "state_owner":"r3-primary-runtime",
            "shell_state":{"user_task":request.get("task_id"),"result_binding":true,"workflow_truth":false,"correlation_log":process_events(&request)},
            "primary_runtime":runtime,
            "semantic":sem["semantic"],
            "policy":grant,
            "scope_crossings":scope_crossings,
            "elapsed_ns":monotonic_ns(start),
            "ipc_count":3+runtime["ipc_count"].as_u64().unwrap_or(0),
            "process_hop_count":2+runtime["process_hop_count"].as_u64().unwrap_or(0),
            "serialization_bytes":sem_tx+sem_rx+pol_tx+pol_rx+run_tx+run_rx+runtime["serialization_bytes"].as_u64().unwrap_or(0) as usize,
            "recovery":{"fault_target":recovery_target,"safe_continuation":true,"recovery_ns":monotonic_ns(start)},
            "feedback":{"emitted_offset_ns":feedback_emitted_offset_ns,"delivered_offset_ns":monotonic_ns(start)},
            "trace":[
                {"span":"interaction","parent":null,"component":"r3-via-shell"},
                {"span":"semantic","parent":"interaction","component":"semantic-model"},
                {"span":"policy","parent":"semantic","component":"policy-engine"},
                {"span":"dispatch","parent":"policy","component":"r3-primary-runtime"},
                {"span":"result.bind","parent":"agent.result","component":"r3-via-shell"},
                {"span":"delivery","parent":"result.bind","component":"r3-via-shell"}
            ]
        });
        append_array(
            &mut result,
            "trace",
            runtime["trace"].as_array().cloned().unwrap_or_default(),
        );
        result
    })
}
