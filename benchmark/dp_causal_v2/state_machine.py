from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class TaskRecord:
    task_id: str
    status: str = "accepted"
    executions: set[str] = field(default_factory=set)
    current_result_id: str | None = None
    current_result_version: int = 0
    approval_id: str | None = None
    parent_task_id: str | None = None


class LifecycleStateMachine:
    """Candidate-side correlation rules for concurrent and reordered lifecycle input."""

    def __init__(self) -> None:
        self.tasks: dict[str, TaskRecord] = {}
        self.execution_task: dict[str, str] = {}
        self.voice_sessions: dict[str, dict[str, str]] = {}
        self.resource_owner: dict[str, str] = {}

    def accept_task(self, task_id: str, parent_task_id: str | None = None) -> TaskRecord:
        if task_id in self.tasks:
            return self.tasks[task_id]
        record = TaskRecord(task_id, parent_task_id=parent_task_id)
        self.tasks[task_id] = record
        return record

    def attach_execution(self, task_id: str, execution_id: str) -> None:
        task = self.tasks[task_id]
        task.executions.add(execution_id)
        self.execution_task[execution_id] = task_id

    def bind_result(self, task_id: str, execution_id: str, result_id: str, version: int) -> bool:
        task = self.tasks[task_id]
        if task.status == "cancelled" or self.execution_task.get(execution_id) != task_id:
            return False
        if version <= task.current_result_version:
            return False
        task.current_result_id, task.current_result_version, task.status = result_id, version, "completed"
        return True

    def correct_result(self, task_id: str, execution_id: str, result_id: str, version: int) -> bool:
        return self.bind_result(task_id, execution_id, result_id, version)

    def cancel_task(self, task_id: str) -> None:
        self.tasks[task_id].status = "cancelled"

    def require_approval(self, task_id: str, approval_id: str) -> None:
        self.tasks[task_id].approval_id = approval_id

    def associate_follow_up(self, task_id: str, follow_up_id: str) -> TaskRecord:
        return self.accept_task(follow_up_id, parent_task_id=task_id)

    def start_voice(self, session_id: str, task_id: str) -> None:
        self.voice_sessions[session_id] = {"task_id": task_id, "status": "playing"}

    def barge_in(self, session_id: str) -> None:
        self.voice_sessions[session_id]["status"] = "stopped"
        # Voice lifetime is deliberately independent of UserTask lifetime.

    def acquire_exclusive(self, resource_id: str, task_id: str) -> bool:
        owner = self.resource_owner.get(resource_id)
        if owner is not None and owner != task_id:
            return False
        self.resource_owner[resource_id] = task_id
        return True
