use crate::{
    agent_harness, agent_router, context_engine,
    fast_eligibility::{self, FastCapability},
    fast_executor, intent_refiner,
    task_manager::TaskManager,
};
use bench_core::{
    ArchitectureEvent, ArchitectureObservation, ArchitectureUnderTest, ExecutionPort,
    ExecutionRequest, ExecutionRoute, ExecutionRouteKind, ExecutorId, InitialProductState,
    ModelPort, ObservationPort, ProductCorrelation, ProductFact, ReferentRole, SubgoalId, TaskId,
    TaskRelation, TurnId, UserTurn,
};

pub struct HybridVia<M, O, E> {
    model: M,
    observations: O,
    executor: E,
    tasks: TaskManager,
    capability_facts: Vec<ProductFact>,
    policy_facts: Vec<ProductFact>,
    clarification_pending: Option<TurnId>,
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
            clarification_pending: None,
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
        let turn_id = turn.turn_id.clone();
        self.emit_for_turn(ArchitectureEvent::ProcessingStarted, turn_id.clone());
        let intent = intent_refiner::refine(&self.model, &context_engine::package(turn))?;
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
        if intent == intent_refiner::NormalizedIntent::CompoundMediaDownloads {
            return self.handle_compound(intent);
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
        self.emit(
            ArchitectureEvent::TaskAssociated {
                task_relation: TaskRelation::New,
            },
            Some(task_id.clone()),
        );
        self.commit_and_execute(task_id, route, action, is_fast)
    }
    fn teardown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M, O, E> HybridVia<M, O, E>
where
    M: ModelPort,
    M::Error: std::fmt::Debug,
    O: ObservationPort,
    E: ExecutionPort,
    E::Error: std::fmt::Debug,
{
    fn handle_compound(&mut self, intent: intent_refiner::NormalizedIntent) -> Result<(), String> {
        let local = ExecutorId::from("VIA_LOCAL_MEDIA");
        let s1_route = ExecutionRoute {
            route_kind: ExecutionRouteKind::LocalDirect,
            initial_executor_id: local.clone(),
            final_executor_id_if_known: Some(local.clone()),
            delegation_chain: vec![local],
        };
        let argo = agent_router::select(
            &self.model,
            &intent,
            &self.capability_facts,
            &self.policy_facts,
        )?;
        let s2_route = ExecutionRoute {
            route_kind: ExecutionRouteKind::ExecutorDirect,
            initial_executor_id: argo.clone(),
            final_executor_id_if_known: Some(argo.clone()),
            delegation_chain: vec![argo],
        };
        let parent = self.tasks.create_parent();
        let s1_task = self.tasks.create(s1_route.clone());
        let s2_task = self.tasks.create(s2_route.clone());
        self.emit_compound(ArchitectureEvent::TaskCreated, &parent, None, None);
        for (task, subgoal, route) in [(&s1_task, "S1", &s1_route), (&s2_task, "S2", &s2_route)] {
            self.emit_compound(
                ArchitectureEvent::TaskCreated,
                &parent,
                Some(task),
                Some(subgoal),
            );
            self.emit_compound(
                ArchitectureEvent::TaskAssociated {
                    task_relation: TaskRelation::New,
                },
                &parent,
                Some(task),
                Some(subgoal),
            );
            self.emit_compound(
                ArchitectureEvent::RouteCandidateObserved {
                    route: route.clone(),
                },
                &parent,
                Some(task),
                Some(subgoal),
            );
            self.executor
                .accept_route(route)
                .map_err(|error| format!("executor rejected compound route: {error:?}"))?;
        }
        for (task, subgoal, route) in [(&s1_task, "S1", &s1_route), (&s2_task, "S2", &s2_route)] {
            self.emit_compound(
                ArchitectureEvent::RouteCommitted {
                    route: route.clone(),
                    subgoal_id: Some(SubgoalId::from(subgoal)),
                },
                &parent,
                Some(task),
                Some(subgoal),
            );
        }
        self.execute_compound_child(&parent, s1_task, "S1", s1_route, "PAUSE_MEDIA", true)?;
        self.execute_compound_child(
            &parent,
            s2_task,
            "S2",
            s2_route,
            "ORGANIZE_DOWNLOADS",
            false,
        )
    }

    fn execute_compound_child(
        &self,
        parent: &TaskId,
        child: TaskId,
        subgoal: &str,
        route: ExecutionRoute,
        action: &str,
        fast: bool,
    ) -> Result<(), String> {
        let request = ExecutionRequest {
            route,
            task_id: child.clone(),
            parent_task_id: Some(parent.clone()),
            subgoal_id: Some(SubgoalId::from(subgoal)),
            semantic_action: action.into(),
        };
        let result = if fast {
            fast_executor::execute(&self.executor, request)?
        } else {
            agent_harness::execute(&self.executor, request)?
        };
        self.observations.emit(ArchitectureObservation {
            event: ArchitectureEvent::ResultBound,
            product_correlation: ProductCorrelation {
                task_id: Some(child.clone()),
                parent_task_id: Some(parent.clone()),
                child_task_id: Some(child),
                subgoal_id: Some(SubgoalId::from(subgoal)),
                execution_id: Some(result.execution_id),
                result_id: Some(result.result_id),
                ..ProductCorrelation::default()
            },
        });
        Ok(())
    }

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
                subgoal_id: None,
            },
            Some(task_id.clone()),
        );
        let request = ExecutionRequest {
            route,
            task_id: task_id.clone(),
            parent_task_id: None,
            subgoal_id: None,
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
    fn emit_for_turn(&self, event: ArchitectureEvent, turn_id: TurnId) {
        self.observations.emit(ArchitectureObservation {
            event,
            product_correlation: ProductCorrelation {
                turn_id: Some(turn_id),
                ..ProductCorrelation::default()
            },
        });
    }
    fn emit_compound(
        &self,
        event: ArchitectureEvent,
        parent: &TaskId,
        child: Option<&TaskId>,
        subgoal: Option<&str>,
    ) {
        self.observations.emit(ArchitectureObservation {
            event,
            product_correlation: ProductCorrelation {
                task_id: Some(child.unwrap_or(parent).clone()),
                parent_task_id: Some(parent.clone()),
                child_task_id: child.cloned(),
                subgoal_id: subgoal.map(SubgoalId::from),
                ..ProductCorrelation::default()
            },
        });
    }
}
