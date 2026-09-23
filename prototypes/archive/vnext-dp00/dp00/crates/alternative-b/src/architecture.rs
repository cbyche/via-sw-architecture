use bench_core::{
    ArchitectureEvent, ArchitectureObservation, ArchitectureUnderTest, ExecutionPort,
    ExecutionRequest, ExecutionRoute, ExecutionRouteKind, ExecutorId, InitialProductState,
    ModelPort, ObservationPort, ProductCorrelation, ProductFact, ReferentRole, SubgoalId, TaskId,
    TaskRelation, TurnId, UserTurn,
};

use crate::{
    argo_primary_controller::{self, ArgoDecision},
    task_projection::TaskProjection,
    thin_context_packager,
};

pub struct ArgoCentric<M, O, E> {
    model: M,
    observations: O,
    executor: E,
    tasks: TaskProjection,
    capability_facts: Vec<ProductFact>,
    policy_facts: Vec<ProductFact>,
    clarification_pending: Option<TurnId>,
}

impl<M, O, E> ArgoCentric<M, O, E> {
    pub fn new(model: M, observations: O, executor: E) -> Self {
        Self {
            model,
            observations,
            executor,
            tasks: TaskProjection::default(),
            capability_facts: Vec::new(),
            policy_facts: Vec::new(),
            clarification_pending: None,
        }
    }
}

impl<M, O, E> ArchitectureUnderTest for ArgoCentric<M, O, E>
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
        let context = thin_context_packager::package(turn);
        let decision = argo_primary_controller::decide(
            &self.model,
            &context,
            &self.capability_facts,
            &self.policy_facts,
        )?;

        if matches!(
            &decision,
            ArgoDecision::Direct { action } | ArgoDecision::Delegate { action, .. }
                if action == "OPEN_RIGHT_DOCUMENT"
        ) {
            self.emit_for_turn(
                ArchitectureEvent::ReferentBound {
                    referent_role: ReferentRole::Source,
                    resolved_referent_id: "doc-right".into(),
                },
                turn_id.clone(),
            );
        }

        if decision == ArgoDecision::Clarify {
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
        if decision == ArgoDecision::ContinueT1 {
            let task_id = TaskId::from("T1");
            let route = self
                .tasks
                .existing(&task_id)
                .cloned()
                .ok_or_else(|| "T1 ARGO mapping missing".to_owned())?;
            self.emit(ArchitectureEvent::TaskReused, Some(task_id.clone()));
            self.emit(
                ArchitectureEvent::TaskAssociated {
                    task_relation: TaskRelation::FollowUp,
                },
                Some(task_id.clone()),
            );
            return self.commit_and_execute(task_id, route, "CONTINUE_T1".into());
        }
        if decision == ArgoDecision::CompoundMediaDownloads {
            return self.handle_compound();
        }

        let route =
            argo_primary_controller::route(&decision).ok_or_else(|| "route missing".to_owned())?;
        let action = match decision {
            ArgoDecision::Direct { action } | ArgoDecision::Delegate { action, .. } => action,
            ArgoDecision::ContinueT1
            | ArgoDecision::Clarify
            | ArgoDecision::CompoundMediaDownloads => unreachable!(),
        };
        let task_id = self.tasks.create(route.clone());
        self.emit(ArchitectureEvent::TaskCreated, Some(task_id.clone()));
        self.emit(
            ArchitectureEvent::TaskAssociated {
                task_relation: TaskRelation::New,
            },
            Some(task_id.clone()),
        );
        self.commit_and_execute(task_id, route, action)
    }

    fn teardown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M, O, E> ArgoCentric<M, O, E>
where
    O: ObservationPort,
    E: ExecutionPort,
    E::Error: std::fmt::Debug,
{
    fn handle_compound(&mut self) -> Result<(), String> {
        let route = ExecutionRoute {
            route_kind: ExecutionRouteKind::ExecutorDirect,
            initial_executor_id: ExecutorId::from("ARGO"),
            final_executor_id_if_known: Some(ExecutorId::from("ARGO")),
            delegation_chain: vec![ExecutorId::from("ARGO")],
        };
        let parent = self.tasks.create_parent();
        let s1_task = self.tasks.create(route.clone());
        let s2_task = self.tasks.create(route.clone());
        self.emit_compound(ArchitectureEvent::TaskCreated, &parent, None, None);
        for (task, subgoal) in [(&s1_task, "S1"), (&s2_task, "S2")] {
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
                .accept_route(&route)
                .map_err(|error| format!("executor rejected compound route: {error:?}"))?;
        }
        for (task, subgoal) in [(&s1_task, "S1"), (&s2_task, "S2")] {
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
        self.execute_compound_child(&parent, s1_task, "S1", route.clone(), "PAUSE_MEDIA")?;
        self.execute_compound_child(&parent, s2_task, "S2", route, "ORGANIZE_DOWNLOADS")
    }

    fn execute_compound_child(
        &self,
        parent: &TaskId,
        child: TaskId,
        subgoal: &str,
        route: ExecutionRoute,
        action: &str,
    ) -> Result<(), String> {
        let result = self
            .executor
            .execute(ExecutionRequest {
                route,
                task_id: child.clone(),
                parent_task_id: Some(parent.clone()),
                subgoal_id: Some(SubgoalId::from(subgoal)),
                semantic_action: action.into(),
            })
            .map_err(|error| format!("executor error: {error:?}"))?;
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
                subgoal_id: None,
            },
            Some(task_id.clone()),
        );
        let result = self
            .executor
            .execute(ExecutionRequest {
                route,
                task_id: task_id.clone(),
                parent_task_id: None,
                subgoal_id: None,
                semantic_action,
            })
            .map_err(|error| format!("executor error: {error:?}"))?;
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
