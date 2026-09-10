#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use bench_core::{
    DecisionOwner, ModelPort, ModelRequest, ModelResponse, ModelStatus, SemanticResponsibility,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayContext {
    pub run_id: String,
    pub episode_id: String,
    pub scenario_id: String,
    pub scenario_version: String,
    pub alternative_id: String,
    pub current_turn_fixture: String,
    pub semantic_behavior_plan_id: String,
    pub semantic_behavior_plan_version: String,
    pub owner_responsibility_mapping_version: String,
    pub replay_payload_registry_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayAttempt {
    pub output: String,
    pub status: ModelStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayOperation {
    operation_key: String,
    responsibility: SemanticResponsibility,
    attempts: Vec<ReplayAttempt>,
}

impl ReplayOperation {
    pub fn new(
        operation_key: impl Into<String>,
        responsibility: SemanticResponsibility,
        attempts: Vec<ReplayAttempt>,
    ) -> Self {
        Self {
            operation_key: operation_key.into(),
            responsibility,
            attempts,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResponsibilityMapping {
    allowed: HashMap<DecisionOwner, HashSet<SemanticResponsibility>>,
}

impl ResponsibilityMapping {
    #[must_use]
    pub fn dp00_base() -> Self {
        use SemanticResponsibility::{
            AgentSelection, ExecutionRouteSelection, IntentInterpretation, ReferentResolution,
        };

        Self::default()
            .allow(
                "A.IntentRefiner".into(),
                [IntentInterpretation, ReferentResolution],
            )
            .allow("A.AgentRouter".into(), [AgentSelection])
            .allow(
                "B.ARGOPrimary".into(),
                [
                    IntentInterpretation,
                    ReferentResolution,
                    ExecutionRouteSelection,
                    AgentSelection,
                ],
            )
            .allow(
                "C.IntentRefiner".into(),
                [IntentInterpretation, ReferentResolution],
            )
            .allow("C.AgentRouter".into(), [AgentSelection])
            .allow(
                "D.IntentRefiner".into(),
                [IntentInterpretation, ReferentResolution],
            )
            .allow(
                "D.ExecutionPathSelector".into(),
                [ExecutionRouteSelection, AgentSelection],
            )
    }

    pub fn allow(
        mut self,
        owner: DecisionOwner,
        responsibilities: impl IntoIterator<Item = SemanticResponsibility>,
    ) -> Self {
        self.allowed
            .entry(owner)
            .or_default()
            .extend(responsibilities);
        self
    }

    fn validates(&self, request: &ModelRequest) -> bool {
        if request.semantic_responsibilities.is_empty() {
            return false;
        }
        self.allowed
            .get(&request.decision_owner)
            .is_some_and(|allowed| {
                request
                    .semantic_responsibilities
                    .iter()
                    .all(|responsibility| allowed.contains(responsibility))
            })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReplayError {
    ResponsibilityViolation,
    OperationNotFound(SemanticResponsibility),
    AttemptExhausted(SemanticResponsibility),
    PoisonedState,
}

#[derive(Debug, Default)]
struct ReplayState {
    consumed: HashMap<SemanticResponsibility, usize>,
    audit: Vec<ReplayAuditEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayAuditEntry {
    pub decision_owner: DecisionOwner,
    pub responsibility: SemanticResponsibility,
    pub operation_key: String,
    pub attempt: usize,
    pub status: ModelStatus,
}

pub struct ReplayAdapter {
    context: ReplayContext,
    mapping: ResponsibilityMapping,
    operations: HashMap<SemanticResponsibility, ReplayOperation>,
    state: Mutex<ReplayState>,
}

impl ReplayAdapter {
    pub fn new(
        context: ReplayContext,
        mapping: ResponsibilityMapping,
        operations: impl IntoIterator<Item = ReplayOperation>,
    ) -> Self {
        Self {
            context,
            mapping,
            operations: operations
                .into_iter()
                .map(|operation| (operation.responsibility, operation))
                .collect(),
            state: Mutex::new(ReplayState::default()),
        }
    }

    #[must_use]
    pub fn context(&self) -> &ReplayContext {
        &self.context
    }

    #[must_use]
    pub fn resolved_operation_key(&self, responsibility: SemanticResponsibility) -> Option<&str> {
        self.operations
            .get(&responsibility)
            .map(|operation| operation.operation_key.as_str())
    }

    #[must_use]
    pub fn audit_snapshot(&self) -> Vec<ReplayAuditEntry> {
        self.state
            .lock()
            .expect("replay state poisoned")
            .audit
            .clone()
    }
}

impl ModelPort for ReplayAdapter {
    type Error = ReplayError;

    fn generate(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error> {
        if !self.mapping.validates(&request) {
            return Err(ReplayError::ResponsibilityViolation);
        }

        let mut state = self.state.lock().map_err(|_| ReplayError::PoisonedState)?;
        let mut outputs = Vec::with_capacity(request.semantic_responsibilities.len());
        let mut response_status = ModelStatus::Completed;

        for responsibility in request.semantic_responsibilities {
            let operation = self
                .operations
                .get(&responsibility)
                .ok_or(ReplayError::OperationNotFound(responsibility))?;
            let consumed = state.consumed.entry(responsibility).or_default();
            let attempt = operation
                .attempts
                .get(*consumed)
                .ok_or(ReplayError::AttemptExhausted(responsibility))?;
            *consumed += 1;
            let attempt_number = *consumed;
            outputs.push(attempt.output.clone());
            if attempt.status != ModelStatus::Completed {
                response_status = attempt.status;
            }
            state.audit.push(ReplayAuditEntry {
                decision_owner: request.decision_owner.clone(),
                responsibility,
                operation_key: operation.operation_key.clone(),
                attempt: attempt_number,
                status: attempt.status,
            });
        }

        Ok(ModelResponse {
            composite_output: outputs.join("\n"),
            model_status: response_status,
        })
    }
}
