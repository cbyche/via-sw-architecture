use clap::{Parser, ValueEnum};
use gate2_contracts::{
    AgentBackend, WorkerRequest, WorkerResponse,
};
use gate2_fixture::{AgentShape, DeterministicAgent};
use gate2_runtime::exec::{IntegrationBridge, ProcessBridge};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
            let worker = args.worker.ok_or("--worker is required for isolated mode")?;
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

async fn run_shared(
    agent_state_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent = match agent_state_file {
        Some(path) => DeterministicAgent::persistent(AgentShape::Q, path)?,
        None => DeterministicAgent::new(AgentShape::Q),
    };
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let request: WorkerRequest = serde_json::from_str(&line)?;
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
        let request: WorkerRequest = serde_json::from_str(&line)?;
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
