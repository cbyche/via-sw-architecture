use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc, time::Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use via_contracts::{AgentObservation, ObservationKind, TaskCommand, TaskOp, TaskView};
use via_runtime::{
    repository::Repository,
    task::{PerTaskSupervisors, SharedTaskService, TaskAuthority},
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Candidate {
    SharedService,
    PerTaskSupervisor,
}

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum)]
    candidate: Candidate,
    #[arg(long)]
    database: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Request {
    Apply {
        command: TaskCommand,
    },
    Get {
        task_id: String,
    },
    Ping,
    RaceCancelResult {
        task_id: String,
        run_id: String,
        expected_revision: u64,
        command_prefix: String,
        source_revision: u64,
        artifact: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Response {
    View {
        view: TaskView,
        elapsed_ns: u128,
    },
    Race {
        cancel: Box<OperationResult>,
        result: Box<OperationResult>,
        final_view: Box<TaskView>,
        elapsed_ns: u128,
    },
    Pong,
    Error {
        message: String,
        elapsed_ns: u128,
    },
}

#[derive(Debug, Serialize)]
struct OperationResult {
    ok: bool,
    view: Option<TaskView>,
    error: Option<String>,
}

impl OperationResult {
    fn from_result(result: Result<TaskView, via_runtime::repository::ApplyError>) -> Self {
        match result {
            Ok(view) => Self {
                ok: true,
                view: Some(view),
                error: None,
            },
            Err(error) => Self {
                ok: false,
                view: None,
                error: Some(error.to_string()),
            },
        }
    }
}

async fn dispatch(
    authority: Arc<dyn TaskAuthority>,
    repository: Repository,
    request: Request,
) -> Response {
    let started = Instant::now();
    match request {
        Request::Apply { command } => match authority.apply(command).await {
            Ok(view) => Response::View {
                view,
                elapsed_ns: started.elapsed().as_nanos(),
            },
            Err(error) => Response::Error {
                message: error.to_string(),
                elapsed_ns: started.elapsed().as_nanos(),
            },
        },
        Request::Get { task_id } => match repository.get(&task_id).await {
            Ok(view) => Response::View {
                view,
                elapsed_ns: started.elapsed().as_nanos(),
            },
            Err(error) => Response::Error {
                message: error.to_string(),
                elapsed_ns: started.elapsed().as_nanos(),
            },
        },
        Request::Ping => Response::Pong,
        Request::RaceCancelResult {
            task_id,
            run_id,
            expected_revision,
            command_prefix,
            source_revision,
            artifact,
        } => {
            let cancel_authority = authority.clone();
            let result_authority = authority.clone();
            let cancel_task = task_id.clone();
            let result_task = task_id.clone();
            let cancel_prefix = command_prefix.clone();
            let result_prefix = command_prefix;
            let cancel = tokio::spawn(async move {
                cancel_authority
                    .apply(TaskCommand {
                        command_id: format!("{cancel_prefix}:cancel"),
                        task_id: cancel_task,
                        expected_revision,
                        op: TaskOp::Cancel,
                    })
                    .await
            });
            let result = tokio::spawn(async move {
                result_authority
                    .apply(TaskCommand {
                        command_id: format!("{result_prefix}:result"),
                        task_id: result_task,
                        expected_revision,
                        op: TaskOp::ApplyObservation {
                            observation: AgentObservation {
                                run_id,
                                source_revision,
                                kind: ObservationKind::Result { artifact },
                            },
                        },
                    })
                    .await
            });
            let cancel_result = match cancel.await {
                Ok(value) => OperationResult::from_result(value),
                Err(error) => OperationResult {
                    ok: false,
                    view: None,
                    error: Some(error.to_string()),
                },
            };
            let result_result = match result.await {
                Ok(value) => OperationResult::from_result(value),
                Err(error) => OperationResult {
                    ok: false,
                    view: None,
                    error: Some(error.to_string()),
                },
            };
            match repository.get(&task_id).await {
                Ok(final_view) => Response::Race {
                    cancel: Box::new(cancel_result),
                    result: Box::new(result_result),
                    final_view: Box::new(final_view),
                    elapsed_ns: started.elapsed().as_nanos(),
                },
                Err(error) => Response::Error {
                    message: error.to_string(),
                    elapsed_ns: started.elapsed().as_nanos(),
                },
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let repository = Repository::open(args.database)?;
    let authority: Arc<dyn TaskAuthority> = match args.candidate {
        Candidate::SharedService => Arc::new(SharedTaskService::new(repository.clone())),
        Candidate::PerTaskSupervisor => Arc::new(PerTaskSupervisors::new(repository.clone())),
    };
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();
    while let Some(line) = lines.next_line().await? {
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(request) => dispatch(authority.clone(), repository.clone(), request).await,
            Err(error) => Response::Error {
                message: error.to_string(),
                elapsed_ns: 0,
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
