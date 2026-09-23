use bench_core::{
    DecisionOwner, ExecutorId, ModelPort, ModelProfile, ModelRequest, ProductFact,
    SemanticResponsibility,
};

use crate::intent_refiner::NormalizedIntent;

pub fn select<M>(
    model: &M,
    intent: &NormalizedIntent,
    capabilities: &[ProductFact],
    policies: &[ProductFact],
) -> Result<ExecutorId, String>
where
    M: ModelPort,
    M::Error: std::fmt::Debug,
{
    let response = model
        .generate(ModelRequest {
            decision_owner: DecisionOwner::from("A.AgentRouter"),
            semantic_responsibilities: vec![SemanticResponsibility::AgentSelection],
            semantic_input: format!(
                "intent={intent:?};capabilities={capabilities:?};policies={policies:?}"
            ),
            expected_output_schema: "executor-selection.v0".into(),
            model_profile: ModelProfile {
                id: "dp00-base".into(),
                version: "v0".into(),
            },
        })
        .map_err(|error| format!("model error: {error:?}"))?;

    let output = response
        .completed_output()
        .map_err(|status| format!("model generation did not complete: {status:?}"))?;

    if output.contains("NETWORK_AGENT") {
        Ok(ExecutorId::from("NetworkAgent"))
    } else if output.contains("ARGO") {
        Ok(ExecutorId::from("ARGO"))
    } else {
        Err(format!("unsupported executor selection: {}", output))
    }
}
