use bench_core::{ExecutionPort, ExecutionRequest, ExecutionResult};
pub fn execute<E: ExecutionPort>(
    executor: &E,
    request: ExecutionRequest,
) -> Result<ExecutionResult, String>
where
    E::Error: std::fmt::Debug,
{
    executor
        .execute(request)
        .map_err(|error| format!("agent client error: {error:?}"))
}
