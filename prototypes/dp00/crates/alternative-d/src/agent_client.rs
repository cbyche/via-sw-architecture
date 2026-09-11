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

#[cfg(test)]
mod qa03_e5_acceptance {
    use super::*;

    #[test]
    fn io_bound_metadata_is_visible_to_delegation_diagnostics() {
        let descriptor = bench_core::DownstreamCapabilityDescriptor {
            executor_id: "DocumentSummaryAgent".into(),
            capability_id: "document.summarize".into(),
            resource_class: Some(bench_core::ResourceClass::IoBound),
        };
        assert_eq!(
            resource_class_for_diagnostics(&descriptor),
            Some(bench_core::ResourceClass::IoBound)
        );
    }
}
