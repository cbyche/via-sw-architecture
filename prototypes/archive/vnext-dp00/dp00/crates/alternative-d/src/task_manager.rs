use bench_core::{ExecutionRoute, InitialProductState, TaskId};
use std::collections::HashMap;
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
        let id = TaskId(format!("D-T{}", self.next_task));
        self.routes.insert(id.clone(), route);
        id
    }
    pub fn create_parent(&mut self) -> TaskId {
        self.next_task += 1;
        TaskId(format!("D-T{}", self.next_task))
    }
}
