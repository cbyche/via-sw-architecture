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

#[allow(dead_code, reason = "QA03 diagnostic metadata must not affect routing")]
pub fn resource_class_for_diagnostics(
    descriptor: &bench_core::DownstreamCapabilityDescriptor,
) -> Option<bench_core::ResourceClass> {
    descriptor.resource_class
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
