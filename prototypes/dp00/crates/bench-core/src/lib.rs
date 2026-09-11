#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

string_id!(TurnId);
string_id!(TaskId);
string_id!(ExecutionId);
string_id!(ExecutorId);
string_id!(DispatchId);
string_id!(ResultId);
string_id!(ClarificationId);
string_id!(SubgoalId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputModality {
    Voice,
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserTurn {
    pub turn_id: TurnId,
    pub modality: InputModality,
    pub content: String,
    pub context_evidence: Vec<ContextEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextEvidence {
    pub source: String,
    pub kind: String,
    pub value: String,
    pub observed_at_product_revision: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialProductState {
    pub conversation_facts: Vec<ProductFact>,
    pub tasks: Vec<InitialTaskState>,
    pub capability_facts: Vec<ProductFact>,
    pub policy_facts: Vec<ProductFact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductFact {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialTaskState {
    pub task_id: TaskId,
    pub execution_id: Option<ExecutionId>,
    pub prior_route: Option<ExecutionRoute>,
    pub status: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionRouteKind {
    LocalDirect,
    ExecutorDirect,
    ExecutorDelegated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRoute {
    pub route_kind: ExecutionRouteKind,
    pub initial_executor_id: ExecutorId,
    pub final_executor_id_if_known: Option<ExecutorId>,
    pub delegation_chain: Vec<ExecutorId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SemanticResponsibility {
    IntentInterpretation,
    ReferentResolution,
    TaskAssociation,
    ClarificationDecision,
    ExecutionRouteSelection,
    AgentSelection,
    ResultAssociation,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DecisionOwner(pub String);

impl From<&str> for DecisionOwner {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecisionMechanism {
    DeterministicRule,
    GenAiDedicated,
    GenAiMixed,
    ExecutorInternal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRequest {
    pub decision_owner: DecisionOwner,
    pub semantic_responsibilities: Vec<SemanticResponsibility>,
    pub semantic_input: String,
    pub expected_output_schema: String,
    pub model_profile: ModelProfile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelProfile {
    pub id: String,
    pub version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelStatus {
    Completed,
    Failed,
    TimedOut,
    NoResponse,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelResponse {
    pub composite_output: String,
    pub model_status: ModelStatus,
}

impl ModelResponse {
    /// Returns model output only when the logical generation completed normally.
    /// Architecture code must not interpret timeout/malformed/failure payloads as
    /// valid semantic decisions.
    pub fn completed_output(&self) -> Result<&str, ModelStatus> {
        if self.model_status == ModelStatus::Completed {
            Ok(&self.composite_output)
        } else {
            Err(self.model_status)
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductCorrelation {
    pub turn_id: Option<TurnId>,
    pub task_id: Option<TaskId>,
    #[serde(default)]
    pub parent_task_id: Option<TaskId>,
    #[serde(default)]
    pub child_task_id: Option<TaskId>,
    #[serde(default)]
    pub subgoal_id: Option<SubgoalId>,
    pub execution_id: Option<ExecutionId>,
    pub dispatch_id: Option<DispatchId>,
    pub result_id: Option<ResultId>,
    pub clarification_id: Option<ClarificationId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferentRole {
    Source,
    Destination,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskRelation {
    New,
    FollowUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClarificationReason {
    AmbiguousReferent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureObservation {
    pub event: ArchitectureEvent,
    pub product_correlation: ProductCorrelation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchitectureEvent {
    ProcessingStarted,
    ReferentBound {
        referent_role: ReferentRole,
        resolved_referent_id: String,
    },
    TaskAssociated {
        task_relation: TaskRelation,
    },
    ClarificationRequested {
        prompt: String,
        reason: ClarificationReason,
    },
    ClarificationResolved {
        request_turn_id: TurnId,
        response_turn_id: TurnId,
    },
    TaskCreated,
    TaskReused,
    RouteCandidateObserved {
        route: ExecutionRoute,
    },
    RouteCandidateRejected {
        route: ExecutionRoute,
        reason: String,
    },
    RouteCommitted {
        route: ExecutionRoute,
        #[serde(default)]
        subgoal_id: Option<SubgoalId>,
    },
    ResultBound,
    CancelPropagated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MonotonicTimestamp(pub u64);

pub trait Clock: Send + Sync {
    fn now(&self) -> MonotonicTimestamp;
}

pub trait ObservationPort: Send + Sync {
    fn emit(&self, observation: ArchitectureObservation);
}

pub trait ModelPort: Send + Sync {
    type Error;

    fn generate(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRequest {
    pub route: ExecutionRoute,
    pub task_id: TaskId,
    pub parent_task_id: Option<TaskId>,
    pub subgoal_id: Option<SubgoalId>,
    pub semantic_action: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub execution_id: ExecutionId,
    pub result_id: ResultId,
    pub payload: String,
}

pub trait ExecutionPort: Send + Sync {
    type Error;

    fn accept_route(&self, _route: &ExecutionRoute) -> Result<(), Self::Error> {
        Ok(())
    }

    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchitectureDependencies<M, O> {
    pub model: M,
    pub observations: O,
}

pub trait ArchitectureUnderTest: Send {
    type Error;

    fn setup(&mut self, state: InitialProductState) -> Result<(), Self::Error>;
    fn handle_user_turn(&mut self, turn: UserTurn) -> Result<(), Self::Error>;
    fn teardown(&mut self) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_request_serialization_contains_only_architecture_semantics() {
        let request = ModelRequest {
            decision_owner: DecisionOwner("test.owner".into()),
            semantic_responsibilities: vec![SemanticResponsibility::IntentInterpretation],
            semantic_input: "turn content".into(),
            expected_output_schema: "intent.v1".into(),
            model_profile: ModelProfile {
                id: "qualification".into(),
                version: "v0".into(),
            },
        };
        let value = serde_json::to_value(request).expect("request serializes");
        let object = value.as_object().expect("request is an object");

        assert_eq!(
            object
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            [
                "decision_owner",
                "expected_output_schema",
                "model_profile",
                "semantic_input",
                "semantic_responsibilities",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect()
        );
    }

    #[test]
    fn observation_has_no_benchmark_provenance_fields() {
        let observation = ArchitectureObservation {
            event: ArchitectureEvent::ProcessingStarted,
            product_correlation: ProductCorrelation {
                turn_id: None,
                task_id: None,
                execution_id: None,
                dispatch_id: None,
                result_id: None,
                clarification_id: None,
                ..ProductCorrelation::default()
            },
        };
        let json = serde_json::to_string(&observation).expect("observation serializes");
        for forbidden in [
            "scenario_id",
            "alternative_id",
            "run_id",
            "sequence_number",
            "monotonic_timestamp",
            "qa_score",
        ] {
            assert!(!json.contains(forbidden));
        }
    }

    #[test]
    fn execution_route_vocabulary_is_topology_neutral() {
        let all = [
            ExecutionRouteKind::LocalDirect,
            ExecutionRouteKind::ExecutorDirect,
            ExecutionRouteKind::ExecutorDelegated,
        ];
        assert_eq!(all.len(), 3);
        let json = serde_json::to_string(&all).expect("route kinds serialize");
        for architecture_name in ["ALTERNATIVE_A", "ALTERNATIVE_B", "ARGO_PRIMARY"] {
            assert!(!json.contains(architecture_name));
        }
    }
}
