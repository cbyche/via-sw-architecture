use clap::Parser;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
use via_contracts::{AgentBackend, ReferenceAgentRequest, ReferenceAgentResponse};
use via_fixture::{AgentShape, DeterministicAgent};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:0")]
    listen: String,
    #[arg(long)]
    state_file: Option<PathBuf>,
}

async fn dispatch(
    agent: &DeterministicAgent,
    request: ReferenceAgentRequest,
) -> ReferenceAgentResponse {
    let result: Result<ReferenceAgentResponse, String> = async {
        Ok(match request {
            ReferenceAgentRequest::Submit { request } => {
                source_event(
                    "agent_request_available_at_agent_ingress",
                    serde_json::json!({
                        "task_id": request.task_id,
                        "submission_key": request.submission_key,
                    }),
                );
                ReferenceAgentResponse::Reply {
                    reply: agent
                        .submit(request)
                        .await
                        .map_err(|error| error.to_string())?,
                }
            }
            ReferenceAgentRequest::Query { run_id } => {
                source_event(
                    "agent_request_available_at_agent_ingress",
                    serde_json::json!({"run_id": run_id, "operation": "query"}),
                );
                ReferenceAgentResponse::Reply {
                    reply: agent
                        .query(&run_id)
                        .await
                        .map_err(|error| error.to_string())?,
                }
            }
            ReferenceAgentRequest::FollowUp { run_id, text } => ReferenceAgentResponse::Reply {
                reply: {
                    source_event(
                        "agent_request_available_at_agent_ingress",
                        serde_json::json!({"run_id": run_id, "operation": "follow_up"}),
                    );
                    agent
                        .follow_up(&run_id, text)
                        .await
                        .map_err(|error| error.to_string())?
                },
            },
            ReferenceAgentRequest::Cancel { run_id } => {
                source_event(
                    "agent_request_available_at_agent_ingress",
                    serde_json::json!({"run_id": run_id, "operation": "cancel"}),
                );
                ReferenceAgentResponse::Reply {
                    reply: agent
                        .cancel(&run_id)
                        .await
                        .map_err(|error| error.to_string())?,
                }
            }
            ReferenceAgentRequest::EventsSince {
                run_id,
                after_revision,
            } => ReferenceAgentResponse::Events {
                events: agent
                    .events_since(&run_id, after_revision)
                    .await
                    .map_err(|error| error.to_string())?,
            },
            ReferenceAgentRequest::EmitProgress { run_id, percent } => {
                agent
                    .emit_progress(&run_id, percent)
                    .map_err(|error| error.to_string())?;
                source_event(
                    "agent_status_available_at_source",
                    serde_json::json!({"run_id": run_id, "percent": percent}),
                );
                ReferenceAgentResponse::Ack
            }
            ReferenceAgentRequest::Ask {
                run_id,
                question_id,
                text,
            } => {
                agent
                    .ask(&run_id, question_id, text)
                    .map_err(|error| error.to_string())?;
                ReferenceAgentResponse::Ack
            }
            ReferenceAgentRequest::Complete { run_id, artifact } => {
                agent
                    .complete(&run_id, artifact.clone())
                    .map_err(|error| error.to_string())?;
                source_event(
                    "agent_result_available_at_source",
                    serde_json::json!({"run_id": run_id, "artifact": artifact}),
                );
                ReferenceAgentResponse::Ack
            }
            ReferenceAgentRequest::ConfirmCancel { run_id } => {
                agent
                    .confirm_cancel(&run_id)
                    .map_err(|error| error.to_string())?;
                ReferenceAgentResponse::Ack
            }
            ReferenceAgentRequest::Fail { run_id, reason } => {
                agent
                    .fail(&run_id, reason)
                    .map_err(|error| error.to_string())?;
                ReferenceAgentResponse::Ack
            }
            ReferenceAgentRequest::Ping => ReferenceAgentResponse::Pong,
        })
    }
    .await;
    result.unwrap_or_else(|message| ReferenceAgentResponse::Error { message })
}

fn source_event(name: &str, fields: serde_json::Value) {
    let wall_time_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before Unix epoch")
        .as_nanos();
    eprintln!(
        "{}",
        serde_json::json!({
            "schema_version": "via.reference-agent.source-event.v1",
            "event": name,
            "source_wall_time_ns": wall_time_ns.to_string(),
            "fields": fields,
        })
    );
}

async fn serve_connection(
    stream: TcpStream,
    agent: DeterministicAgent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await? {
        let response = match serde_json::from_str::<ReferenceAgentRequest>(&line) {
            Ok(request) => dispatch(&agent, request).await,
            Err(error) => ReferenceAgentResponse::Error {
                message: error.to_string(),
            },
        };
        writer
            .write_all(serde_json::to_string(&response)?.as_bytes())
            .await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let agent = match args.state_file {
        Some(path) => DeterministicAgent::persistent(AgentShape::Q, path)?,
        None => DeterministicAgent::new(AgentShape::Q),
    };
    let listener = TcpListener::bind(&args.listen).await?;
    println!(
        "{}",
        serde_json::json!({
            "status":"ready",
            "address":listener.local_addr()?.to_string(),
            "contract":"ReferenceAgentRequest-v1"
        })
    );

    let agent = Arc::new(agent);
    loop {
        let (stream, _) = listener.accept().await?;
        let agent = agent.clone();
        tokio::spawn(async move {
            if let Err(error) = serve_connection(stream, agent.as_ref().clone()).await {
                eprintln!("reference agent connection failed: {error}");
            }
        });
    }
}
