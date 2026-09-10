use bench_core::{
    ArchitectureEvent, ArchitectureObservation, ArchitectureUnderTest, ExecutionPort,
    ExecutionRequest, ExecutionRoute, ExecutionRouteKind, InitialProductState, ModelPort,
    ObservationPort, ProductCorrelation, ProductFact, ReferentRole, TaskId, TaskRelation, TurnId,
    UserTurn,
};

use crate::{
    agent_harness, agent_router, context_engine, intent_refiner, task_manager::TaskManager,
};

pub struct ThinVia<M, O, E> {
    model: M,
    observations: O,
    executor: E,
    tasks: TaskManager,
    capability_facts: Vec<ProductFact>,
    policy_facts: Vec<ProductFact>,
    clarification_pending: Option<TurnId>,
}

impl<M, O, E> ThinVia<M, O, E> {
    pub fn new(model: M, observations: O, executor: E) -> Self {
        Self {
            model,
            observations,
            executor,
            tasks: TaskManager::default(),
            capability_facts: Vec::new(),
            policy_facts: Vec::new(),
            clarification_pending: None,
        }
    }
}

impl<M, O, E> ArchitectureUnderTest for ThinVia<M, O, E>
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
        let turn_id = turn.turn_id.clone();
        self.emit_for_turn(ArchitectureEvent::ProcessingStarted, turn_id.clone());
        let context = context_engine::package(turn);
        let intent = intent_refiner::refine(&self.model, &context)?;

        if intent == intent_refiner::NormalizedIntent::OpenRightDocument {
            self.emit_for_turn(
                ArchitectureEvent::ReferentBound {
                    referent_role: ReferentRole::Source,
                    resolved_referent_id: "doc-right".into(),
                },
                turn_id.clone(),
            );
        }

        if intent == intent_refiner::NormalizedIntent::AmbiguousDocument {
            self.clarification_pending = Some(turn_id.clone());
            self.emit_for_turn(
                ArchitectureEvent::ClarificationRequested {
                    prompt: "Which document?".into(),
                    reason: bench_core::ClarificationReason::AmbiguousReferent,
                },
                turn_id,
            );
            return Ok(());
        }
        if let Some(request_turn_id) = self.clarification_pending.take() {
            self.emit_for_turn(
                ArchitectureEvent::ClarificationResolved {
                    request_turn_id,
                    response_turn_id: turn_id.clone(),
                },
                turn_id.clone(),
            );
        }

        if intent == intent_refiner::NormalizedIntent::ContinueTask {
            return self.continue_task(TaskId::from("T1"));
        }

        let executor_id = agent_router::select(
            &self.model,
            &intent,
            &self.capability_facts,
            &self.policy_facts,
        )?;
        let mut route = ExecutionRoute {
            route_kind: ExecutionRouteKind::ExecutorDirect,
            initial_executor_id: executor_id.clone(),
            final_executor_id_if_known: Some(executor_id.clone()),
            delegation_chain: vec![executor_id],
        };
        if self.accept_candidate(&route).is_err() {
            let fallback = agent_router::select(
                &self.model,
                &intent,
                &self.capability_facts,
                &self.policy_facts,
            )?;
            route = ExecutionRoute {
                route_kind: ExecutionRouteKind::ExecutorDirect,
                initial_executor_id: fallback.clone(),
                final_executor_id_if_known: Some(fallback.clone()),
                delegation_chain: vec![fallback],
            };
            self.accept_candidate(&route)?;
        }
        let task_id = self.tasks.create(route.clone());
        self.emit(ArchitectureEvent::TaskCreated, Some(task_id.clone()));
        self.emit(
            ArchitectureEvent::TaskAssociated {
                task_relation: TaskRelation::New,
            },
            Some(task_id.clone()),
        );
        self.commit_and_execute(task_id, route, format!("{intent:?}"))
    }

    fn teardown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M, O, E> ThinVia<M, O, E>
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
        self.emit(
            ArchitectureEvent::TaskAssociated {
                task_relation: TaskRelation::FollowUp,
            },
            Some(task_id.clone()),
        );
        self.accept_candidate(&route)?;
        self.commit_and_execute(task_id, route, "ContinueTask".into())
    }

    fn accept_candidate(&self, route: &ExecutionRoute) -> Result<(), String> {
        self.emit(
            ArchitectureEvent::RouteCandidateObserved {
                route: route.clone(),
            },
            None,
        );
        self.executor.accept_route(route).map_err(|error| {
            self.emit(
                ArchitectureEvent::RouteCandidateRejected {
                    route: route.clone(),
                    reason: format!("synchronous dispatch reject: {error:?}"),
                },
                None,
            );
            format!("executor rejected route: {error:?}")
        })
    }

    fn commit_and_execute(
        &self,
        task_id: TaskId,
        route: ExecutionRoute,
        semantic_action: String,
    ) -> Result<(), String> {
        self.emit(
            ArchitectureEvent::RouteCommitted {
                route: route.clone(),
            },
            Some(task_id.clone()),
        );
        let result = agent_harness::execute(
            &self.executor,
            ExecutionRequest {
                route,
                task_id: task_id.clone(),
                semantic_action,
            },
        )?;
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

    fn emit_for_turn(&self, event: ArchitectureEvent, turn_id: TurnId) {
        self.observations.emit(ArchitectureObservation {
            event,
            product_correlation: ProductCorrelation {
                turn_id: Some(turn_id),
                ..ProductCorrelation::default()
            },
        });
    }
}
