use std::collections::HashMap;

use bench_core::{ExecutionRoute, InitialProductState, TaskId};

#[derive(Default)]
pub struct TaskManager {
    routes: HashMap<TaskId, ExecutionRoute>,
    next_task: u64,
}

impl TaskManager {
    pub fn seed(&mut self, state: InitialProductState) {
        self.routes = state
            .tasks
            .into_iter()
            .filter_map(|task| task.prior_route.map(|route| (task.task_id, route)))
            .collect();
    }

    pub fn existing(&self, task_id: &TaskId) -> Option<&ExecutionRoute> {
        self.routes.get(task_id)
    }

    pub fn create(&mut self, route: ExecutionRoute) -> TaskId {
        self.next_task += 1;
        let task_id = TaskId(format!("A-T{}", self.next_task));
        self.routes.insert(task_id.clone(), route);
        task_id
    }

    pub fn create_parent(&mut self) -> TaskId {
        self.next_task += 1;
        TaskId(format!("A-T{}", self.next_task))
    }
}
