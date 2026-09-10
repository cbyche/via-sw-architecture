use crate::{
    agent_harness, agent_router, context_engine,
    fast_eligibility::{self, FastCapability},
    fast_executor, intent_refiner,
    task_manager::TaskManager,
};
use bench_core::{
    ArchitectureEvent, ArchitectureObservation, ArchitectureUnderTest, ExecutionPort,
    ExecutionRequest, ExecutionRoute, ExecutionRouteKind, ExecutorId, InitialProductState,
    ModelPort, ObservationPort, ProductCorrelation, ProductFact, TaskId, UserTurn,
};

pub struct HybridVia<M, O, E> {
    model: M,
    observations: O,
    executor: E,
    tasks: TaskManager,
    capability_facts: Vec<ProductFact>,
    policy_facts: Vec<ProductFact>,
    clarification_pending: bool,
}
impl<M, O, E> HybridVia<M, O, E> {
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

impl<M, O, E> ArchitectureUnderTest for HybridVia<M, O, E>
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
            return self.continue_task(TaskId::from("T1"));
        }

        let (route, action, is_fast) = if let Some(capability) =
            fast_eligibility::evaluate(&intent, &self.capability_facts, &self.policy_facts)
        {
            let executor_id = match capability {
                FastCapability::Volume => ExecutorId::from("VIA_LOCAL_VOLUME"),
                FastCapability::DocumentOpen => ExecutorId::from("VIA_LOCAL_DOCUMENT"),
            };
            (
                ExecutionRoute {
                    route_kind: ExecutionRouteKind::LocalDirect,
                    initial_executor_id: executor_id.clone(),
                    final_executor_id_if_known: Some(executor_id.clone()),
                    delegation_chain: vec![executor_id],
                },
                format!("{intent:?}"),
                true,
            )
        } else {
            let executor_id = agent_router::select(
                &self.model,
                &intent,
                &self.capability_facts,
                &self.policy_facts,
            )?;
            (
                ExecutionRoute {
                    route_kind: ExecutionRouteKind::ExecutorDirect,
                    initial_executor_id: executor_id.clone(),
                    final_executor_id_if_known: Some(executor_id.clone()),
                    delegation_chain: vec![executor_id],
                },
                format!("{intent:?}"),
                false,
            )
        };
        let task_id = self.tasks.create(route.clone());
        self.emit(ArchitectureEvent::TaskCreated, Some(task_id.clone()));
        self.commit_and_execute(task_id, route, action, is_fast)
    }
    fn teardown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M, O, E> HybridVia<M, O, E>
where
    O: ObservationPort,
    E: ExecutionPort,
    E::Error: std::fmt::Debug,
{
    fn continue_task(&self, task_id: TaskId) -> Result<(), String> {
        let route = self
            .tasks
            .existing(&task_id)
            .cloned()
            .ok_or_else(|| "T1 route missing".to_owned())?;
        self.emit(ArchitectureEvent::TaskReused, Some(task_id.clone()));
        self.commit_and_execute(task_id, route, "ContinueTask".into(), false)
    }
    fn commit_and_execute(
        &self,
        task_id: TaskId,
        route: ExecutionRoute,
        semantic_action: String,
        fast: bool,
    ) -> Result<(), String> {
        self.emit(
            ArchitectureEvent::RouteCommitted {
                route: route.clone(),
            },
            Some(task_id.clone()),
        );
        let request = ExecutionRequest {
            route,
            task_id: task_id.clone(),
            semantic_action,
        };
        let result = if fast {
            fast_executor::execute(&self.executor, request)?
        } else {
            agent_harness::execute(&self.executor, request)?
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
