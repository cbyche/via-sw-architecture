use crate::intent_refiner::NormalizedIntent;
use bench_core::{
    DecisionOwner, ExecutionRoute, ExecutionRouteKind, ExecutorId, ModelPort, ModelProfile,
    ModelRequest, ProductFact, SemanticResponsibility,
};

pub fn select<M: ModelPort>(
    model: &M,
    intent: &NormalizedIntent,
    capabilities: &[ProductFact],
    policies: &[ProductFact],
) -> Result<ExecutionRoute, String>
where
    M::Error: std::fmt::Debug,
{
    let response = model
        .generate(ModelRequest {
            decision_owner: DecisionOwner::from("D.ExecutionPathSelector"),
            semantic_responsibilities: vec![
                SemanticResponsibility::ExecutionRouteSelection,
                SemanticResponsibility::AgentSelection,
            ],
            semantic_input: format!(
                "intent={intent:?};capabilities={capabilities:?};policies={policies:?}"
            ),
            expected_output_schema: "execution-route.v0".into(),
            model_profile: ModelProfile {
                id: "dp00-base".into(),
                version: "v0".into(),
            },
        })
        .map_err(|error| format!("model error: {error:?}"))?;
    let output = response
        .completed_output()
        .map_err(|status| format!("model generation did not complete: {status:?}"))?;

    let (kind, executor) = if output.contains("LOCAL_VOLUME") {
        (ExecutionRouteKind::LocalDirect, "VIA_LOCAL_VOLUME")
    } else if output.contains("LOCAL_DOCUMENT") {
        (ExecutionRouteKind::LocalDirect, "VIA_LOCAL_DOCUMENT")
    } else if output.contains("NETWORK_AGENT") {
        (ExecutionRouteKind::ExecutorDirect, "NetworkAgent")
    } else if output.contains("ARGO") {
        (ExecutionRouteKind::ExecutorDirect, "ARGO")
    } else {
        return Err(format!("unsupported execution path: {output}"));
    };
    let executor = ExecutorId::from(executor);
    Ok(ExecutionRoute {
        route_kind: kind,
        initial_executor_id: executor.clone(),
        final_executor_id_if_known: Some(executor.clone()),
        delegation_chain: vec![executor],
    })
}

pub fn select_compound<M: ModelPort>(
    model: &M,
    intent: &NormalizedIntent,
    capabilities: &[ProductFact],
    policies: &[ProductFact],
) -> Result<(ExecutionRoute, ExecutionRoute), String>
where
    M::Error: std::fmt::Debug,
{
    let response = model
        .generate(ModelRequest {
            decision_owner: DecisionOwner::from("D.ExecutionPathSelector"),
            semantic_responsibilities: vec![
                SemanticResponsibility::ExecutionRouteSelection,
                SemanticResponsibility::AgentSelection,
            ],
            semantic_input: format!(
                "intent={intent:?};capabilities={capabilities:?};policies={policies:?}"
            ),
            expected_output_schema: "compound-execution-route.v1".into(),
            model_profile: ModelProfile {
                id: "dp00-base".into(),
                version: "v0".into(),
            },
        })
        .map_err(|error| format!("model error: {error:?}"))?;
    let output = response
        .completed_output()
        .map_err(|status| format!("model generation did not complete: {status:?}"))?;
    if !output.contains("COMPOUND_LOCAL_AND_GENERAL") || !output.contains("ARGO") {
        return Err(format!("unsupported compound execution path: {output}"));
    }
    let local = ExecutorId::from("VIA_LOCAL_MEDIA");
    let argo = ExecutorId::from("ARGO");
    Ok((
        ExecutionRoute {
            route_kind: ExecutionRouteKind::LocalDirect,
            initial_executor_id: local.clone(),
            final_executor_id_if_known: Some(local.clone()),
            delegation_chain: vec![local],
        },
        ExecutionRoute {
            route_kind: ExecutionRouteKind::ExecutorDirect,
            initial_executor_id: argo.clone(),
            final_executor_id_if_known: Some(argo.clone()),
            delegation_chain: vec![argo],
        },
    ))
}
