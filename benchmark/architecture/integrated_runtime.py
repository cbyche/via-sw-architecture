"""Reusable process/message adapters for integrated VIA Architecture campaigns."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import time
from typing import Any

from benchmark.architecture.run_dp06_evaluation import ReferenceAgent


ROOT = Path(__file__).resolve().parents[2]
HOST_BIN = ROOT / "prototypes/candidates/target/debug/via-host"
WORKER_BIN = ROOT / "prototypes/candidates/target/debug/via-worker"


class RustAgentHost:
    """Actual Rust VIA host with either same-process or isolated integration code.

    Evaluator-only Reference Agent control operations bypass the VIA host. Product-path
    submit/query/follow-up/cancel/event operations always cross the selected candidate.
    """

    def __init__(self, mode: str, source: ReferenceAgent) -> None:
        if mode not in {"shared", "isolated"}:
            raise ValueError(f"unsupported Rust host mode: {mode}")
        if not HOST_BIN.exists() or not WORKER_BIN.exists():
            raise RuntimeError("build via-host and via-worker before integrated campaigns")
        argv = [
            str(HOST_BIN),
            "--mode",
            mode,
            "--agent-address",
            source.address,
        ]
        if mode == "isolated":
            argv.extend(["--worker", str(WORKER_BIN)])
        self.mode = mode
        self.source = source
        self.process = subprocess.Popen(
            argv,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        if self.process.stdin is None or self.process.stdout is None:
            raise RuntimeError("Rust host stdio unavailable")
        self._stdin = self.process.stdin
        self._stdout = self.process.stdout

    @property
    def wall_to_mono_ns(self) -> int:
        return self.source.wall_to_mono_ns

    @property
    def clock_uncertainty_ns(self) -> int:
        return self.source.clock_uncertainty_ns

    def _host_request(self, request: dict[str, Any]) -> dict[str, Any]:
        if self.process.poll() is not None:
            raise RuntimeError(f"Rust host exited with {self.process.returncode}")
        self._stdin.write(json.dumps(request, ensure_ascii=False, sort_keys=True) + "\n")
        self._stdin.flush()
        line = self._stdout.readline()
        if not line:
            raise RuntimeError(f"Rust host closed stdout (exit={self.process.poll()})")
        response = json.loads(line)
        if response.get("status") == "error":
            raise RuntimeError(response["message"])
        return response

    def request(self, request: dict[str, Any]) -> dict[str, Any]:
        operation = request.get("op")
        if operation in {"complete", "emit_progress", "ask", "confirm_cancel", "fail"}:
            # Fixture-control operations are evaluator-only and intentionally absent
            # from the VIA-owned integration contract.
            return self.source.request(request)
        return self._host_request(request)

    def drain_events(self, settle_ms: int = 30) -> list[dict[str, Any]]:
        return self.source.drain_events(settle_ms)

    def ping(self) -> bool:
        return self._host_request({"op": "ping"}).get("status") == "pong"

    def core_probe(self, capability: str) -> dict[str, Any]:
        return self._host_request({"op": "core_probe", "capability": capability})

    def abort_integration_host(self) -> tuple[bool, int]:
        """Inject the frozen fatal boundary event and report whether Core survived."""
        started = time.monotonic_ns()
        try:
            response = self._host_request({"op": "abort_host"})
            survived = response.get("status") == "pong" and self.process.poll() is None
        except (BrokenPipeError, RuntimeError, json.JSONDecodeError):
            survived = False
        return survived, time.monotonic_ns() - started

    def close(self) -> None:
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=2)

    def __enter__(self) -> "RustAgentHost":
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


def integrated_agent(mode: str) -> tuple[ReferenceAgent, RustAgentHost]:
    source = ReferenceAgent()
    try:
        return source, RustAgentHost(mode, source)
    except Exception:
        source.close()
        raise


def close_integrated_agent(source: ReferenceAgent, host: RustAgentHost) -> None:
    host.close()
    source.close()
