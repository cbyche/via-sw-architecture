//! Integration contracts. Evaluator goal IDs and expected target sets never enter these APIs.
use serde_json::Value;
#[derive(Clone, Debug, Default)]
pub struct EvidenceResult {
    pub context: Value,
    pub passages: Vec<Value>,
    pub bindings: Vec<Value>,
}
#[derive(Clone, Debug)]
pub struct GroundingInput {
    pub event_id: String,
    pub revision: u64,
    pub transcript: String,
    pub frames: Vec<Value>,
    pub cues: Vec<Value>,
    pub prior: Value,
    pub evidence: Value,
}
#[derive(Clone, Debug)]
pub struct HandoffRequest {
    pub goal_id: String,
    pub event_id: String,
    pub raw_text: String,
    pub revision: u64,
    pub adoption: Value,
    pub evidence: EvidenceResult,
    pub grounding: Value,
    pub public_state: Value,
}
#[derive(Clone, Debug)]
pub struct Configuration {
    pub id: String,
    pub evidence: String,
    pub execution: String,
    pub grounding: String,
    pub handoff: String,
}
impl Configuration {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let get = |key: &str| {
            v[key]
                .as_str()
                .map(String::from)
                .ok_or_else(|| format!("configuration {} missing", key))
        };
        Ok(Self {
            id: get("id")?,
            evidence: get("evidence")?,
            execution: get("execution")?,
            grounding: get("grounding")?,
            handoff: get("handoff")?,
        })
    }
}
