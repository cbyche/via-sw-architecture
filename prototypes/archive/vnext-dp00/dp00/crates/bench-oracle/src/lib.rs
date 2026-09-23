#![forbid(unsafe_code)]

use bench_core::{ExecutionRoute, TaskId};
use bench_events::CanonicalEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluatorOracle {
    pub ground_truth: GroundTruth,
    pub constraints: Vec<ScenarioConstraint>,
    pub success_predicate: SuccessPredicate,
    pub qa_eligibility: QaEligibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundTruth {
    pub referent: Option<String>,
    pub task_association: Option<TaskId>,
    pub expected_route: Option<ExecutionRoute>,
    pub expected_result_binding: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConstraintDisposition {
    Required,
    Allowed,
    Forbidden,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioConstraint {
    pub id: String,
    pub disposition: ConstraintDisposition,
    pub predicate: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuccessPredicate {
    pub id: String,
    pub expression: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QaEligibility {
    pub qa01: bool,
    pub qa02: bool,
    pub qa04: bool,
}

pub trait OracleEvaluator {
    type Result;

    fn evaluate(&self, oracle: &EvaluatorOracle, evidence: &[CanonicalEvent]) -> Self::Result;
}
