"""Client for the executable VIA-DP-14 Rust Task authority candidates."""

from __future__ import annotations

import json
from pathlib import Path
import select
import subprocess
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
TASK_HOST = ROOT / "prototypes/candidates/target/debug/via-task-host"


class TaskHostClient:
    def __init__(self, candidate: str, database: Path) -> None:
        if candidate not in {"shared_service", "per_task_supervisor"}:
            raise ValueError(f"unsupported Task candidate: {candidate}")
        if not TASK_HOST.exists():
            raise RuntimeError(f"build Task host first: {TASK_HOST}")
        self.candidate = candidate
        self.database = database
        self.process = subprocess.Popen(
            [
                str(TASK_HOST),
                "--candidate",
                candidate.replace("_", "-"),
                "--database",
                str(database),
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        response, _ = self.request({"op": "ping"})
        if response.get("status") != "pong":
            self.close()
            raise RuntimeError(f"Task host failed readiness: {response}")

    def request(
        self, value: dict[str, Any], timeout: float = 5.0
    ) -> tuple[dict[str, Any], int]:
        if self.process.stdin is None or self.process.stdout is None:
            raise RuntimeError("Task host pipes unavailable")
        if self.process.poll() is not None:
            raise RuntimeError(f"Task host exited with {self.process.returncode}")
        started = time.monotonic_ns()
        self.process.stdin.write(json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n")
        self.process.stdin.flush()
        ready, _, _ = select.select([self.process.stdout], [], [], timeout)
        if not ready:
            raise TimeoutError(f"Task host request timed out: {value.get('op')}")
        line = self.process.stdout.readline()
        elapsed = time.monotonic_ns() - started
        if not line:
            stderr = self.process.stderr.read() if self.process.stderr else ""
            raise RuntimeError(f"Task host closed stdout: {stderr[-1000:]}")
        return json.loads(line), elapsed

    def apply(self, command: dict[str, Any]) -> tuple[dict[str, Any], int]:
        response, elapsed = self.request({"op": "apply", "command": command})
        if response.get("status") != "view":
            raise RuntimeError(response.get("message", f"Task apply failed: {response}"))
        return response["view"], elapsed

    def get(self, task_id: str) -> tuple[dict[str, Any], int]:
        response, elapsed = self.request({"op": "get", "task_id": task_id})
        if response.get("status") != "view":
            raise RuntimeError(response.get("message", f"Task get failed: {response}"))
        return response["view"], elapsed

    def close(self) -> None:
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=2)

    def __enter__(self) -> "TaskHostClient":
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


def create_command(task_id: str, command_id: str, goal: str) -> dict[str, Any]:
    return {
        "command_id": command_id,
        "task_id": task_id,
        "expected_revision": 0,
        "op": {"kind": "create", "goal": goal},
    }


def accept_command(
    task_id: str, command_id: str, revision: int, run_id: str
) -> dict[str, Any]:
    return {
        "command_id": command_id,
        "task_id": task_id,
        "expected_revision": revision,
        "op": {"kind": "accept_execution", "run_id": run_id},
    }


def result_command(
    task_id: str,
    command_id: str,
    revision: int,
    run_id: str,
    source_revision: int,
    artifact: str,
) -> dict[str, Any]:
    return {
        "command_id": command_id,
        "task_id": task_id,
        "expected_revision": revision,
        "op": {
            "kind": "apply_observation",
            "observation": {
                "run_id": run_id,
                "source_revision": source_revision,
                "kind": {"kind": "result", "artifact": artifact},
            },
        },
    }


def progress_command(
    task_id: str,
    command_id: str,
    revision: int,
    run_id: str,
    source_revision: int,
    percent: int,
) -> dict[str, Any]:
    return {
        "command_id": command_id,
        "task_id": task_id,
        "expected_revision": revision,
        "op": {
            "kind": "apply_observation",
            "observation": {
                "run_id": run_id,
                "source_revision": source_revision,
                "kind": {"kind": "progress", "percent": percent},
            },
        },
    }


def cancelled_command(
    task_id: str,
    command_id: str,
    revision: int,
    run_id: str,
    source_revision: int,
) -> dict[str, Any]:
    return {
        "command_id": command_id,
        "task_id": task_id,
        "expected_revision": revision,
        "op": {
            "kind": "apply_observation",
            "observation": {
                "run_id": run_id,
                "source_revision": source_revision,
                "kind": {"kind": "cancelled"},
            },
        },
    }


def cancel_command(task_id: str, command_id: str, revision: int) -> dict[str, Any]:
    return {
        "command_id": command_id,
        "task_id": task_id,
        "expected_revision": revision,
        "op": {"kind": "cancel"},
    }
