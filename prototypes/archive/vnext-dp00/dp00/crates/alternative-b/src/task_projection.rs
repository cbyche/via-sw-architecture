use std::collections::HashMap;

use bench_core::{ExecutionRoute, ExecutionRouteKind, ExecutorId, InitialProductState, TaskId};

#[derive(Default)]
pub struct TaskProjection {
    argo_routes: HashMap<TaskId, ExecutionRoute>,
    next_task: u64,
}

impl TaskProjection {
    pub fn seed(&mut self, state: InitialProductState) {
        self.argo_routes = state
            .tasks
            .into_iter()
            .filter_map(|task| {
                task.prior_route.map(|route| {
                    let final_executor = route
                        .final_executor_id_if_known
                        .unwrap_or(route.initial_executor_id);
                    let argo_route = ExecutionRoute {
                        route_kind: ExecutionRouteKind::ExecutorDelegated,
                        initial_executor_id: ExecutorId::from("ARGO"),
                        final_executor_id_if_known: Some(final_executor.clone()),
                        delegation_chain: vec![ExecutorId::from("ARGO"), final_executor],
                    };
                    (task.task_id, argo_route)
                })
            })
            .collect();
    }

    pub fn existing(&self, task_id: &TaskId) -> Option<&ExecutionRoute> {
        self.argo_routes.get(task_id)
    }

    pub fn create(&mut self, route: ExecutionRoute) -> TaskId {
        self.next_task += 1;
        let task_id = TaskId(format!("B-T{}", self.next_task));
        self.argo_routes.insert(task_id.clone(), route);
        task_id
    }

    pub fn create_parent(&mut self) -> TaskId {
        self.next_task += 1;
        TaskId(format!("B-T{}", self.next_task))
    }
}
