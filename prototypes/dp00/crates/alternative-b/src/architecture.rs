use bench_core::{
    ArchitectureEvent, ArchitectureObservation, ArchitectureUnderTest, ExecutionPort,
    ExecutionRequest, ExecutionRoute, InitialProductState, ModelPort, ObservationPort,
    ProductCorrelation, ProductFact, TaskId, UserTurn,
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
    clarification_pending: bool,
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
            clarification_pending: false,
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
        self.emit(ArchitectureEvent::ProcessingStarted, None);
        let context = thin_context_packager::package(turn);
        let decision = argo_primary_controller::decide(
            &self.model,
            &context,
            &self.capability_facts,
            &self.policy_facts,
        )?;

        if decision == ArgoDecision::Clarify {
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
        if decision == ArgoDecision::ContinueT1 {
            let task_id = TaskId::from("T1");
            let route = self
                .tasks
                .existing(&task_id)
                .cloned()
                .ok_or_else(|| "T1 ARGO mapping missing".to_owned())?;
            self.emit(ArchitectureEvent::TaskReused, Some(task_id.clone()));
            return self.commit_and_execute(task_id, route, "CONTINUE_T1".into());
        }

        let route =
            argo_primary_controller::route(&decision).ok_or_else(|| "route missing".to_owned())?;
        let action = match decision {
            ArgoDecision::Direct { action } | ArgoDecision::Delegate { action, .. } => action,
            ArgoDecision::ContinueT1 | ArgoDecision::Clarify => unreachable!(),
        };
        let task_id = self.tasks.create(route.clone());
        self.emit(ArchitectureEvent::TaskCreated, Some(task_id.clone()));
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
        let result = self
            .executor
            .execute(ExecutionRequest {
                route,
                task_id: task_id.clone(),
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
}
