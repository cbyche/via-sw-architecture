use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use alternative_d::AdaptiveVia;
use bench_core::{
    ArchitectureObservation, ArchitectureUnderTest, ExecutionId, ExecutionPort, ExecutionRequest,
    ExecutionResult, InitialProductState, InputModality, ModelPort, ModelRequest, ModelResponse,
    ModelStatus, ObservationPort, ResultId, TurnId, UserTurn,
};
use bench_fixtures::context_sources::active_window_title;

#[derive(Clone)]
struct Model {
    outputs: Arc<Mutex<VecDeque<String>>>,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}

impl Model {
    fn scripted(outputs: &[&str]) -> Self {
        Self {
            outputs: Arc::new(Mutex::new(
                outputs.iter().map(|value| (*value).to_owned()).collect(),
            )),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl ModelPort for Model {
    type Error = String;

    fn generate(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error> {
        self.requests.lock().unwrap().push(request);
        Ok(ModelResponse {
            composite_output: self.outputs.lock().unwrap().pop_front().unwrap(),
            model_status: ModelStatus::Completed,
        })
    }
}

#[derive(Clone, Default)]
struct Executor;
impl ExecutionPort for Executor {
    type Error = String;
    fn execute(&self, _request: ExecutionRequest) -> Result<ExecutionResult, Self::Error> {
        Ok(ExecutionResult {
            execution_id: ExecutionId::from("e-context"),
            result_id: ResultId::from("r-context"),
            payload: "ok".into(),
        })
    }
}

#[derive(Clone, Default)]
struct Observations;
impl ObservationPort for Observations {
    fn emit(&self, _observation: ArchitectureObservation) {}
}

#[test]
fn active_window_title_maps_once_with_provenance_and_is_consumed_by_context_seam() {
    let mut evidence = vec![bench_core::ContextEvidence {
        source: "existing".into(),
        kind: "existing-kind".into(),
        value: "existing-value".into(),
        observed_at_product_revision: Some(40),
    }];
    evidence.extend(active_window_title::map("Architecture.md", 41));
    assert_eq!(evidence.len(), 2);
    assert_eq!(evidence[0].value, "existing-value");
    assert_eq!(evidence[1].source, "ACTIVE_WINDOW_TITLE");
    assert_eq!(evidence[1].kind, "WINDOW_TITLE");
    assert_eq!(evidence[1].value, "Architecture.md");
    assert_eq!(evidence[1].observed_at_product_revision, Some(41));

    let model = Model::scripted(&["GENERAL_FILE_WORK", "ARGO"]);
    let requests = model.requests.clone();
    let mut architecture = AdaptiveVia::new(model, Observations, Executor);
    architecture.setup(InitialProductState::default()).unwrap();
    architecture
        .handle_user_turn(UserTurn {
            turn_id: TurnId::from("turn-context"),
            modality: InputModality::Text,
            content: "organize files".into(),
            context_evidence: evidence,
        })
        .unwrap();

    let semantic_input = requests.lock().unwrap()[0].semantic_input.clone();
    assert!(semantic_input.contains("existing-value"));
    assert!(semantic_input.contains("Architecture.md"));
    assert!(semantic_input.contains("ACTIVE_WINDOW_TITLE"));
}

