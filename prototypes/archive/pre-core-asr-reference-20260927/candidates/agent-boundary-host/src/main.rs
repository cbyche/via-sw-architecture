use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use via_contracts::{AgentObservation, CancelOutcome, SubmitRequest};
use via_fixture::RemoteReferenceAgent;
use via_runtime::agent::{AgentBoundary, CoreVisibleTyped, EdgeNormalized};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Candidate {
    EdgeNormalized,
    CoreVisibleTyped,
}

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum)]
    candidate: Candidate,
    #[arg(long)]
    agent_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Request {
    Submit { request: SubmitRequest },
    Query { run_id: String },
    FollowUp { run_id: String, text: String },
    Cancel { run_id: String },
    EventsSince { run_id: String, after_revision: u64 },
    Ping,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Response {
    Accepted {
        run_id: String,
        context_id: Option<String>,
    },
    Snapshot {
        run_id: String,
        source_revision: u64,
        state: String,
        artifact: Option<String>,
    },
    Cancel {
        outcome: CancelOutcome,
    },
    Events {
        events: Vec<AgentObservation>,
    },
    Pong,
    Error {
        message: String,
    },
}

async fn dispatch(boundary: &dyn AgentBoundary, request: Request) -> Response {
    let result: Result<Response, String> = async {
        Ok(match request {
            Request::Submit { request } => {
                let accepted = boundary
                    .submit(request)
                    .await
                    .map_err(|error| error.to_string())?;
                Response::Accepted {
                    run_id: accepted.run_id,
                    context_id: accepted.context_id,
                }
            }
            Request::Query { run_id } => {
                let snapshot = boundary
                    .query(&run_id)
                    .await
                    .map_err(|error| error.to_string())?;
                Response::Snapshot {
                    run_id: snapshot.run_id,
                    source_revision: snapshot.source_revision,
                    state: snapshot.state,
                    artifact: snapshot.artifact,
                }
            }
            Request::FollowUp { run_id, text } => {
                let accepted = boundary
                    .follow_up(&run_id, text)
                    .await
                    .map_err(|error| error.to_string())?;
                Response::Accepted {
                    run_id: accepted.run_id,
                    context_id: accepted.context_id,
                }
            }
            Request::Cancel { run_id } => Response::Cancel {
                outcome: boundary
                    .cancel(&run_id)
                    .await
                    .map_err(|error| error.to_string())?,
            },
            Request::EventsSince {
                run_id,
                after_revision,
            } => Response::Events {
                events: boundary
                    .events_since(&run_id, after_revision)
                    .await
                    .map_err(|error| error.to_string())?,
            },
            Request::Ping => Response::Pong,
        })
    }
    .await;
    result.unwrap_or_else(|message| Response::Error { message })
}

async fn serve(boundary: Box<dyn AgentBoundary>) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();
    while let Some(line) = lines.next_line().await? {
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(request) => dispatch(boundary.as_ref(), request).await,
            Err(error) => Response::Error {
                message: error.to_string(),
            },
        };
        stdout
            .write_all(serde_json::to_string(&response)?.as_bytes())
            .await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let backend = RemoteReferenceAgent::new(args.agent_address);
    let boundary: Box<dyn AgentBoundary> = match args.candidate {
        Candidate::EdgeNormalized => Box::new(EdgeNormalized::new(backend)),
        Candidate::CoreVisibleTyped => Box::new(CoreVisibleTyped::new(backend)),
    };
    serve(boundary).await
}
