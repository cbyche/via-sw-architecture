use bench_core::{ExecutionPort, ExecutionRequest, ExecutionResult};

pub fn execute<E>(executor: &E, request: ExecutionRequest) -> Result<ExecutionResult, String>
where
    E: ExecutionPort,
    E::Error: std::fmt::Debug,
{
    executor
        .execute(request)
        .map_err(|error| format!("executor error: {error:?}"))
}
