use crate::{
    agent_client, context_engine, execution_path_selector, fast_executor, intent_refiner,
    task_manager::TaskManager,
};
use bench_core::{
    ArchitectureEvent, ArchitectureObservation, ArchitectureUnderTest, ExecutionPort,
    ExecutionRequest, ExecutionRoute, ExecutionRouteKind, InitialProductState, ModelPort,
    ObservationPort, ProductCorrelation, ProductFact, TaskId, UserTurn,
};

pub struct AdaptiveVia<M, O, E> {
    model: M,
    observations: O,
    executor: E,
    tasks: TaskManager,
    capability_facts: Vec<ProductFact>,
    policy_facts: Vec<ProductFact>,
    clarification_pending: bool,
}
impl<M, O, E> AdaptiveVia<M, O, E> {
    pub fn new(model: M, observations: O, executor: E) -> Self {
        Self {
            model,
            observations,
            executor,
            tasks: TaskManager::default(),
            capability_facts: Vec::new(),
            policy_facts: Vec::new(),
            clarification_pending: false,
        }
    }
}

impl<M, O, E> ArchitectureUnderTest for AdaptiveVia<M, O, E>
where
    M: ModelPort + Send,
    M::Error: std::fmt::Debug,
    O: ObservationPort + Send,
    E: ExecutionPort + Send,
    E::Error: std::fmt::Debug,
{
    type Error = String;
    fn setup(&mut self, state: InitialProductState) -> Result<(), Self::Error> {
        self.capability_facts.clone_from(&state.capability_facts);
        self.policy_facts.clone_from(&state.policy_facts);
        self.tasks.seed(state);
        Ok(())
    }
    fn handle_user_turn(&mut self, turn: UserTurn) -> Result<(), Self::Error> {
        self.emit(ArchitectureEvent::ProcessingStarted, None);
        let intent = intent_refiner::refine(&self.model, &context_engine::package(turn))?;
        if intent == intent_refiner::NormalizedIntent::AmbiguousDocument {
            self.clarification_pending = true;
            self.emit(
                ArchitectureEvent::ClarificationRequested {
                    prompt: "Which document?".into(),
                },
                None,
            );
            return Ok(());
        }
        if self.clarification_pending {
            self.clarification_pending = false;
            self.emit(ArchitectureEvent::ClarificationResolved, None);
        }
        if intent == intent_refiner::NormalizedIntent::ContinueTask {
            let task_id = TaskId::from("T1");
            let route = self
                .tasks
                .existing(&task_id)
                .cloned()
                .ok_or_else(|| "T1 route missing".to_owned())?;
            self.emit(ArchitectureEvent::TaskReused, Some(task_id.clone()));
            return self.commit_and_execute(task_id, route, "ContinueTask".into());
        }
        let route = execution_path_selector::select(
            &self.model,
            &intent,
            &self.capability_facts,
            &self.policy_facts,
        )?;
        let task_id = self.tasks.create(route.clone());
        self.emit(ArchitectureEvent::TaskCreated, Some(task_id.clone()));
        self.commit_and_execute(task_id, route, format!("{intent:?}"))
    }
    fn teardown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M, O, E> AdaptiveVia<M, O, E>
where
    O: ObservationPort,
    E: ExecutionPort,
    E::Error: std::fmt::Debug,
{
    fn commit_and_execute(
        &self,
        task_id: TaskId,
        route: ExecutionRoute,
        semantic_action: String,
    ) -> Result<(), String> {
        self.emit(
            ArchitectureEvent::RouteCandidateObserved {
                route: route.clone(),
            },
            Some(task_id.clone()),
        );
        if let Err(error) = self.executor.accept_route(&route) {
            self.emit(
                ArchitectureEvent::RouteCandidateRejected {
                    route: route.clone(),
                    reason: format!("synchronous dispatch reject: {error:?}"),
                },
                Some(task_id),
            );
            return Err(format!("executor rejected route: {error:?}"));
        }
        self.emit(
            ArchitectureEvent::RouteCommitted {
                route: route.clone(),
            },
            Some(task_id.clone()),
        );
        let request = ExecutionRequest {
            route: route.clone(),
            task_id: task_id.clone(),
            semantic_action,
        };
        let result = if route.route_kind == ExecutionRouteKind::LocalDirect {
            fast_executor::execute(&self.executor, request)?
        } else {
            agent_client::execute(&self.executor, request)?
        };
        self.observations.emit(ArchitectureObservation {
            event: ArchitectureEvent::ResultBound,
            product_correlation: ProductCorrelation {
                task_id: Some(task_id),
                execution_id: Some(result.execution_id),
                result_id: Some(result.result_id),
                ..ProductCorrelation::default()
            },
        });
        Ok(())
    }
    fn emit(&self, event: ArchitectureEvent, task_id: Option<TaskId>) {
        self.observations.emit(ArchitectureObservation {
            event,
            product_correlation: ProductCorrelation {
                task_id,
                ..ProductCorrelation::default()
            },
        });
    }
}
