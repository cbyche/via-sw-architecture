use gate2_contracts::{AgentBackend, WorkerRequest, WorkerResponse};
use gate2_fixture::{AgentShape, DeterministicAgent};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn state_file_arg() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let mut state_file = None;
    while let Some(arg) = args.next() {
        if arg == "--state-file" {
            let value = args.next().ok_or("--state-file requires a path")?;
            state_file = Some(PathBuf::from(value));
        } else {
            return Err(format!("unknown worker argument: {}", arg.to_string_lossy()).into());
        }
    }
    Ok(state_file)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = match state_file_arg()? {
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
        stdout
            .write_all(serde_json::to_string(&response)?.as_bytes())
            .await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }
    Ok(())
}
