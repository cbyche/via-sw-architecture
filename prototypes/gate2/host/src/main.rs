use clap::{Parser, ValueEnum};
use gate2_contracts::{
    AgentBackend, NativeReply, PReply, QReply, SubmitRequest, WorkerRequest, WorkerResponse,
};
use gate2_fixture::{AgentShape, DeterministicAgent};
use gate2_runtime::exec::{IntegrationBridge, ProcessBridge};
use std::path::PathBuf;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Mode {
    Shared,
    Isolated,
}

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum)]
    mode: Mode,
    #[arg(long)]
    worker: Option<PathBuf>,
    #[arg(long)]
    agent_state_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match args.mode {
        Mode::Shared => run_shared(args.agent_state_file).await?,
        Mode::Isolated => {
            let worker = args
                .worker
                .ok_or("--worker is required for isolated mode")?;
            run_isolated(worker, args.agent_state_file).await?;
        }
    }
    Ok(())
}

async fn write_response(
    stdout: &mut tokio::io::Stdout,
    response: WorkerResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    stdout
        .write_all(serde_json::to_string(&response)?.as_bytes())
        .await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}

async fn write_value(
    stdout: &mut tokio::io::Stdout,
    response: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    stdout
        .write_all(serde_json::to_string(response)?.as_bytes())
        .await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}

fn static_probe_value(capability: &str) -> Option<&'static str> {
    match capability {
        "CAP-TEXT-INTERACTION" => Some("text:ready"),
        "CAP-S2S-DIRECT" => Some("s2s:direct-ready"),
        "CAP-SEMANTIC-TEXT" => Some("semantic:ready"),
        "CAP-BOUNDED-DOC-READ" => Some("doc-budget:education-budget"),
        "CAP-BOUNDED-MAIL-READ" => Some("mail:education-schedule"),
        "CAP-LOCAL-MEMORY-READ" => Some("memory:table-first"),
        "CAP-LOCAL-TASK-CARD-READ" => Some("task:T-PPT:running"),
        _ => None,
    }
}

fn agent_probe_spec(capability: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match capability {
        "CAP-AGENT-DOC-TASK" => Some(("agent-doc", "T-PPT", "agent-doc:T-PPT:running")),
        "CAP-AGENT-MAIL-TASK" => Some(("agent-mail", "T-MAIL", "agent-mail:T-MAIL:running")),
        _ => None,
    }
}

fn accepted_run_id(reply: &NativeReply) -> Option<String> {
    match reply {
        NativeReply::P(PReply::Accepted { run_id }) => Some(run_id.clone()),
        NativeReply::Q(QReply::Accepted { run_id, .. }) => Some(run_id.clone()),
        _ => None,
    }
}

fn is_running_snapshot(reply: &NativeReply) -> bool {
    match reply {
        NativeReply::P(PReply::Snapshot { state, .. })
        | NativeReply::Q(QReply::Snapshot { state, .. }) => state == "running",
        _ => false,
    }
}

fn probe_response(capability: &str, value: &str, evidence: &str) -> serde_json::Value {
    serde_json::json!({
        "status":"w10_capability_probe",
        "capability":capability,
        "value":value,
        "evidence":evidence
    })
}

fn probe_error(capability: &str, message: impl ToString) -> serde_json::Value {
    serde_json::json!({
        "status":"error",
        "capability":capability,
        "message":message.to_string()
    })
}

fn core_probe_response(request: &serde_json::Value) -> Option<serde_json::Value> {
    if request.get("op").and_then(serde_json::Value::as_str) != Some("core_probe") {
        return None;
    }
    let capability = request
        .get("capability")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let Some(value) = static_probe_value(capability) else {
        return Some(probe_error(
            capability,
            format!("unknown W-10 core probe capability: {capability}"),
        ));
    };
    Some(serde_json::json!({
        "status":"core_probe",
        "capability":capability,
        "value":value
    }))
}

async fn begin_external_fault(
    request: &serde_json::Value,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    if request.get("op").and_then(serde_json::Value::as_str) != Some("w10_begin_external_fault") {
        return Ok(None);
    }
    let mode = request
        .get("mode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let address = request
        .get("address")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| std::io::Error::other("W-10 fault address missing"))?;

    let response = match mode {
        "connection_refused" => {
            let observed = TcpStream::connect(address).await.is_err();
            serde_json::json!({
                "status":"w10_fault_started",
                "mode":mode,
                "fault_observed":observed
            })
        }
        "no_reply" => {
            let mut stream = TcpStream::connect(address).await?;
            stream.write_all(b"w10").await?;
            tokio::spawn(async move {
                let mut byte = [0_u8; 1];
                let _ = stream.read(&mut byte).await;
            });
            serde_json::json!({
                "status":"w10_fault_started",
                "mode":mode,
                "fault_observed":true
            })
        }
        other => serde_json::json!({
            "status":"error",
            "message":format!("unknown W-10 external fault mode: {other}")
        }),
    };
    Ok(Some(response))
}

async fn shared_capability_probe(
    agent: &DeterministicAgent,
    request: &serde_json::Value,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    if request.get("op").and_then(serde_json::Value::as_str) != Some("w10_capability_probe") {
        return Ok(None);
    }
    let capability = request
        .get("capability")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if let Some(value) = static_probe_value(capability) {
        return Ok(Some(probe_response(
            capability,
            value,
            "candidate_host_core_surface",
        )));
    }

    let Some((agent_id, task_id, value)) = agent_probe_spec(capability) else {
        return Ok(Some(probe_error(
            capability,
            format!("unknown W-10 capability probe: {capability}"),
        )));
    };
    let accepted = agent
        .submit(SubmitRequest {
            task_id: task_id.into(),
            submission_key: format!("w10-{agent_id}-{task_id}"),
            goal: format!("W-10 health probe for {agent_id}"),
        })
        .await?;
    let run_id = accepted_run_id(&accepted)
        .ok_or_else(|| std::io::Error::other("W-10 agent probe was not accepted"))?;
    let snapshot = agent.query(&run_id).await?;
    if !is_running_snapshot(&snapshot) {
        return Ok(Some(probe_error(
            capability,
            "W-10 agent task is not running",
        )));
    }
    Ok(Some(probe_response(
        capability,
        value,
        "candidate_host_shared_agent_backend",
    )))
}

async fn isolated_capability_probe(
    bridge: &ProcessBridge,
    request: &serde_json::Value,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    if request.get("op").and_then(serde_json::Value::as_str) != Some("w10_capability_probe") {
        return Ok(None);
    }
    let capability = request
        .get("capability")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if let Some(value) = static_probe_value(capability) {
        return Ok(Some(probe_response(
            capability,
            value,
            "candidate_host_core_surface",
        )));
    }

    let Some((agent_id, task_id, value)) = agent_probe_spec(capability) else {
        return Ok(Some(probe_error(
            capability,
            format!("unknown W-10 capability probe: {capability}"),
        )));
    };
    let accepted = bridge
        .submit(SubmitRequest {
            task_id: task_id.into(),
            submission_key: format!("w10-{agent_id}-{task_id}"),
            goal: format!("W-10 health probe for {agent_id}"),
        })
        .await?;
    let run_id = accepted_run_id(&accepted)
        .ok_or_else(|| std::io::Error::other("W-10 isolated agent probe was not accepted"))?;
    let snapshot = bridge.query(&run_id).await?;
    if !is_running_snapshot(&snapshot) {
        return Ok(Some(probe_error(
            capability,
            "W-10 isolated agent task is not running",
        )));
    }
    Ok(Some(probe_response(
        capability,
        value,
        "candidate_host_isolated_agent_backend",
    )))
}

async fn run_shared(agent_state_file: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let agent = match agent_state_file {
        Some(path) => DeterministicAgent::persistent(AgentShape::Q, path)?,
        None => DeterministicAgent::new(AgentShape::Q),
    };
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let raw: serde_json::Value = serde_json::from_str(&line)?;
        if let Some(response) = core_probe_response(&raw) {
            write_value(&mut stdout, &response).await?;
            continue;
        }
        if let Some(response) = begin_external_fault(&raw).await? {
            write_value(&mut stdout, &response).await?;
            continue;
        }
        if let Some(response) = shared_capability_probe(&agent, &raw).await? {
            write_value(&mut stdout, &response).await?;
            continue;
        }
        let request: WorkerRequest = serde_json::from_value(raw)?;
        if matches!(request, WorkerRequest::AbortHost) {
            std::process::abort();
        }
        let response = match request {
            WorkerRequest::Submit { request } => match agent.submit(request).await {
                Ok(reply) => WorkerResponse::Reply { reply },
                Err(error) => WorkerResponse::Error {
                    message: error.to_string(),
                },
            },
            WorkerRequest::Query { run_id } => match agent.query(&run_id).await {
                Ok(reply) => WorkerResponse::Reply { reply },
                Err(error) => WorkerResponse::Error {
                    message: error.to_string(),
                },
            },
            WorkerRequest::FollowUp { run_id, text } => {
                match agent.follow_up(&run_id, text).await {
                    Ok(reply) => WorkerResponse::Reply { reply },
                    Err(error) => WorkerResponse::Error {
                        message: error.to_string(),
                    },
                }
            }
            WorkerRequest::Cancel { run_id } => match agent.cancel(&run_id).await {
                Ok(reply) => WorkerResponse::Reply { reply },
                Err(error) => WorkerResponse::Error {
                    message: error.to_string(),
                },
            },
            WorkerRequest::EventsSince {
                run_id,
                after_revision,
            } => match agent.events_since(&run_id, after_revision).await {
                Ok(events) => WorkerResponse::Events { events },
                Err(error) => WorkerResponse::Error {
                    message: error.to_string(),
                },
            },
            WorkerRequest::Ping => WorkerResponse::Pong,
            WorkerRequest::AbortHost => unreachable!(),
        };
        write_response(&mut stdout, response).await?;
    }
    Ok(())
}

async fn run_isolated(
    worker: PathBuf,
    agent_state_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bridge = match &agent_state_file {
        Some(path) => ProcessBridge::spawn_with_state_file(&worker, path).await?,
        None => ProcessBridge::spawn(&worker).await?,
    };
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let raw: serde_json::Value = serde_json::from_str(&line)?;
        if let Some(response) = core_probe_response(&raw) {
            write_value(&mut stdout, &response).await?;
            continue;
        }
        if let Some(response) = begin_external_fault(&raw).await? {
            write_value(&mut stdout, &response).await?;
            continue;
        }
        if let Some(response) = isolated_capability_probe(&bridge, &raw).await? {
            write_value(&mut stdout, &response).await?;
            continue;
        }
        let request: WorkerRequest = serde_json::from_value(raw)?;
        let response = match request {
            WorkerRequest::Submit { request } => match bridge.submit(request).await {
                Ok(reply) => WorkerResponse::Reply { reply },
                Err(error) => WorkerResponse::Error {
                    message: error.to_string(),
                },
            },
            WorkerRequest::Query { run_id } => match bridge.query(&run_id).await {
                Ok(reply) => WorkerResponse::Reply { reply },
                Err(error) => WorkerResponse::Error {
                    message: error.to_string(),
                },
            },
            WorkerRequest::FollowUp { run_id, text } => {
                match bridge.follow_up(&run_id, text).await {
                    Ok(reply) => WorkerResponse::Reply { reply },
                    Err(error) => WorkerResponse::Error {
                        message: error.to_string(),
                    },
                }
            }
            WorkerRequest::Cancel { run_id } => match bridge.cancel(&run_id).await {
                Ok(reply) => WorkerResponse::Reply { reply },
                Err(error) => WorkerResponse::Error {
                    message: error.to_string(),
                },
            },
            WorkerRequest::EventsSince {
                run_id,
                after_revision,
            } => match bridge.events_since(&run_id, after_revision).await {
                Ok(events) => WorkerResponse::Events { events },
                Err(error) => WorkerResponse::Error {
                    message: error.to_string(),
                },
            },
            WorkerRequest::Ping => WorkerResponse::Pong,
            WorkerRequest::AbortHost => {
                bridge.abort_host().await?;
                bridge = match &agent_state_file {
                    Some(path) => ProcessBridge::spawn_with_state_file(&worker, path).await?,
                    None => ProcessBridge::spawn(&worker).await?,
                };
                WorkerResponse::Pong
            }
        };
        write_response(&mut stdout, response).await?;
    }
    Ok(())
}
