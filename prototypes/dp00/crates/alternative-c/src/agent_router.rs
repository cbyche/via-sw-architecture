use crate::intent_refiner::NormalizedIntent;
use bench_core::{
    DecisionOwner, ExecutorId, ModelPort, ModelProfile, ModelRequest, ProductFact,
    SemanticResponsibility,
};

pub fn select<M: ModelPort>(
    model: &M,
    intent: &NormalizedIntent,
    capabilities: &[ProductFact],
    policies: &[ProductFact],
) -> Result<ExecutorId, String>
where
    M::Error: std::fmt::Debug,
{
    let output = model
        .generate(ModelRequest {
            decision_owner: DecisionOwner::from("C.AgentRouter"),
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
        .map_err(|error| format!("model error: {error:?}"))?
        .composite_output;
    if output.contains("NETWORK_AGENT") {
        Ok(ExecutorId::from("NetworkAgent"))
    } else if output.contains("ARGO") {
        Ok(ExecutorId::from("ARGO"))
    } else {
        Err(format!("unsupported executor selection: {output}"))
    }
}
