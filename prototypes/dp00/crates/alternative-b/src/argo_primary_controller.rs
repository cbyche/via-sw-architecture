use bench_core::{
    DecisionOwner, ExecutionRoute, ExecutionRouteKind, ExecutorId, ModelPort, ModelProfile,
    ModelRequest, ProductFact, SemanticResponsibility,
};

use crate::thin_context_packager::ContextPackage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgoDecision {
    Direct {
        action: String,
    },
    Delegate {
        executor: ExecutorId,
        action: String,
    },
    ContinueT1,
    Clarify,
}

pub fn decide<M>(
    model: &M,
    context: &ContextPackage,
    capabilities: &[ProductFact],
    policies: &[ProductFact],
) -> Result<ArgoDecision, String>
where
    M: ModelPort,
    M::Error: std::fmt::Debug,
{
    let response = model
        .generate(ModelRequest {
            decision_owner: DecisionOwner::from("B.ARGOPrimary"),
            semantic_responsibilities: vec![
                SemanticResponsibility::IntentInterpretation,
                SemanticResponsibility::ReferentResolution,
                SemanticResponsibility::ExecutionRouteSelection,
                SemanticResponsibility::AgentSelection,
            ],
            semantic_input: format!(
                "turn={};capabilities={capabilities:?};policies={policies:?}",
                context.turn.content
            ),
            expected_output_schema: "argo-primary-decision.v0".into(),
            model_profile: ModelProfile {
                id: "dp00-base".into(),
                version: "v0".into(),
            },
        })
        .map_err(|error| format!("model error: {error:?}"))?;
    let output = response.composite_output;

    if output.contains("AMBIGUOUS_DOCUMENT") {
        Ok(ArgoDecision::Clarify)
    } else if output.contains("CONTINUE_T1") {
        Ok(ArgoDecision::ContinueT1)
    } else if output.contains("NETWORK_AGENT") {
        Ok(ArgoDecision::Delegate {
            executor: ExecutorId::from("NetworkAgent"),
            action: action(&output),
        })
    } else if output.contains("ARGO_DIRECT") {
        Ok(ArgoDecision::Direct {
            action: action(&output),
        })
    } else {
        Err(format!("unsupported ARGO primary decision: {output}"))
    }
}

pub fn route(decision: &ArgoDecision) -> Option<ExecutionRoute> {
    match decision {
        ArgoDecision::Direct { .. } => Some(ExecutionRoute {
            route_kind: ExecutionRouteKind::ExecutorDirect,
            initial_executor_id: ExecutorId::from("ARGO"),
            final_executor_id_if_known: Some(ExecutorId::from("ARGO")),
            delegation_chain: vec![ExecutorId::from("ARGO")],
        }),
        ArgoDecision::Delegate { executor, .. } => Some(ExecutionRoute {
            route_kind: ExecutionRouteKind::ExecutorDelegated,
            initial_executor_id: ExecutorId::from("ARGO"),
            final_executor_id_if_known: Some(executor.clone()),
            delegation_chain: vec![ExecutorId::from("ARGO"), executor.clone()],
        }),
        ArgoDecision::ContinueT1 | ArgoDecision::Clarify => None,
    }
}

fn action(output: &str) -> String {
    for value in [
        "LOCAL_VOLUME",
        "GENERAL_FILE_WORK",
        "DIAGNOSE_WIFI",
        "OPEN_RIGHT_DOCUMENT",
    ] {
        if output.contains(value) {
            return value.into();
        }
    }
    "UNKNOWN".into()
}
