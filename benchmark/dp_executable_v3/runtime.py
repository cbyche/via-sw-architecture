from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
PROTOTYPE = ROOT / "prototypes/dp00-executable-v3"
BIN = PROTOTYPE / "target/release"

BINARY_BY_REALIZATION = {
    "R1": "r1-via",
    "R3": "r3-shell",
    "R1+@": "r1-via-fastpath",
}


class ExecutableTopology:
    def __init__(self, realization: str, *, executable_override: str | None = None, bin_dir: Path | None = None):
        self.realization = realization
        self.binary = executable_override or BINARY_BY_REALIZATION[realization]
        self.bin_dir = bin_dir or BIN
        self._temp = tempfile.TemporaryDirectory(prefix="via-v3-candidate-")
        cwd = Path(self._temp.name)
        (cwd / "runtime-input").mkdir()
        (cwd / "semantic-replay").mkdir()
        (cwd / "agent-replay").mkdir()
        env = {**os.environ, "VIA_V3_STATE_DIR": str(cwd / "state")}
        self.process = subprocess.Popen(
            [str(self.bin_dir / self.binary)], cwd=cwd, env=env,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, bufsize=1,
        )

    @property
    def cwd(self) -> Path:
        return Path(self._temp.name)

    def execute(self, request: dict[str, Any]) -> dict[str, Any]:
        assert self.process.stdin and self.process.stdout
        wire = json.dumps(request, separators=(",", ":"), sort_keys=True)
        started = time.monotonic_ns()
        self.process.stdin.write(wire + "\n")
        self.process.stdin.flush()
        response_line = self.process.stdout.readline()
        ended = time.monotonic_ns()
        if not response_line:
            error = self.process.stderr.read() if self.process.stderr else ""
            raise RuntimeError(f"{self.binary} exited during request: {error}")
        response = json.loads(response_line)
        response["driver_wall_ns"] = ended - started
        response["driver_wire_bytes"] = len(wire.encode()) + len(response_line.encode()) + 1
        return response

    def memory_snapshot(self) -> list[dict[str, Any]]:
        probe = self.execute(base_request("memory-probe", capability="bounded-read"))
        pids = [process["pid"] for process in probe["topology"]["processes"]]
        command = ["ps", "-o", "pid=,rss=,vsz=,comm=", "-p", ",".join(map(str, pids))]
        rows = []
        for line in subprocess.run(command, check=True, capture_output=True, text=True).stdout.splitlines():
            pid, rss, vsz, command_name = line.strip().split(maxsplit=3)
            rows.append({"pid": int(pid), "rss_kib": int(rss), "vsz_kib": int(vsz), "command": command_name})
        return rows

    def close(self) -> None:
        if self.process.poll() is None and self.process.stdin:
            self.process.stdin.write('{"control":"shutdown"}\n')
            self.process.stdin.flush()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
        self._temp.cleanup()

    def __enter__(self) -> "ExecutableTopology":
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


class PlaybackProbe:
    def __init__(self):
        self.process = subprocess.Popen(
            [str(BIN / "playback-probe")], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, text=True, bufsize=1,
        )

    def execute(self, depth_ms: int) -> dict[str, Any]:
        assert self.process.stdin and self.process.stdout
        started = time.monotonic_ns()
        self.process.stdin.write(json.dumps({"playback_buffer_depth_ms": depth_ms}) + "\n")
        self.process.stdin.flush()
        result = json.loads(self.process.stdout.readline())
        result["driver_wall_ns"] = time.monotonic_ns() - started
        return result

    def close(self) -> None:
        if self.process.poll() is None and self.process.stdin:
            self.process.stdin.write('{"control":"shutdown"}\n')
            self.process.stdin.flush()
            self.process.wait(timeout=5)


def base_request(case_id: str, *, capability: str = "planning", read_only: bool = False) -> dict[str, Any]:
    return {
        "case_id": case_id,
        "task_id": f"task:{case_id}",
        "principal": "principal-local",
        "semantic_replay": {"goal": "probe", "referent": "local:v1", "constraints": [], "consent_required": False, "capability": capability, "read_only": read_only},
        "agent_replay": {"status": "success", "result_version": 1, "facts": [f"{case_id}:fact"]},
        "events": [{"type": "TURN_COMMIT", "task_id": f"task:{case_id}"}, {"type": "TASK_START", "task_id": f"task:{case_id}"}, {"type": "AGENT_RESULT", "task_id": f"task:{case_id}"}],
        "scope_request": [f"scope:task:{case_id}"],
        "delays": {"semantic_ms": 0, "agent_ms": 0, "tool_ms": 0},
    }


def normalized_behavior(result: dict[str, Any]) -> dict[str, Any]:
    ignored = {"pid", "root_pid", "parent_pid", "elapsed_ns", "driver_wall_ns", "recovery_ns", "serialization_bytes", "driver_wire_bytes", "emitted_offset_ns", "delivered_offset_ns", "specialist_pid"}
    def clean(value: Any) -> Any:
        if isinstance(value, dict):
            return {key: clean(item) for key, item in value.items() if key not in ignored and key != "topology"}
        if isinstance(value, list):
            return [clean(item) for item in value]
        return value
    return clean(result)
