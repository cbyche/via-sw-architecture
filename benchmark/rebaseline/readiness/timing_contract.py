"""Validate raw latency endpoints; never estimate p95 or invent an S2S profile.
All endpoint differences require the same core/controller monotonic clock.
Replayed dependencies remain simulated even when their trace was measured.
"""
from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Iterable


@dataclass(frozen=True)
class Event:
    name: str
    at_ns: int
    clock: str

    def __post_init__(self) -> None:
        if not self.name or not self.clock or type(self.at_ns) is not int or self.at_ns < 0:
            raise ValueError("event requires a name, clock and non-negative integer timestamp")


def event_map(events: Iterable[Event]) -> dict[str, Event]:
    output: dict[str, Event] = {}
    for event in events:
        if event.name in output:
            raise ValueError(f"ambiguous repeated endpoint: {event.name}")
        output[event.name] = event
    return output


def duration(start: Event, end: Event) -> int:
    if start.clock != end.clock:
        raise ValueError("different monotonic clocks require an explicit mapping")
    if end.at_ns < start.at_ns:
        raise ValueError("negative endpoint interval")
    return end.at_ns - start.at_ns


def union_duration(intervals: Iterable[tuple[int, int]]) -> int:
    values = sorted(intervals)
    total = 0
    left: int | None = None
    right = 0
    for start, end in values:
        if type(start) is not int or type(end) is not int or start < 0 or end < start:
            raise ValueError("invalid interval")
        if left is None:
            left, right = start, end
        elif start > right:
            total += right - left
            left, right = start, end
        else:
            right = max(right, end)
    return total if left is None else total + right - left


def endpoint_sample(events: Iterable[Event], metric: str, modality: str,
                    s2s: dict[str, Any] | None,
                    human_waits: Iterable[tuple[int, int]] = (),
                    sink: str = "HEADLESS_TEST_ONLY") -> dict[str, Any]:
    """Return one raw sample. This function intentionally has no score API."""
    e = event_map(events)
    if modality not in {"voice", "text"}:
        raise ValueError("unknown modality")
    required = {
        "W-01": ("user_input_end", "first_meaningful_output"),
        "W-02": ("user_input_end", "agent_acceptance_confirmed", "recoverable_link_committed"),
        "W-03": ("agent_event_available", "first_meaningful_output"),
    }
    if metric not in required:
        raise ValueError("unsupported endpoint metric")
    try:
        selected = [e[name] for name in required[metric]]
    except KeyError as ex:
        raise ValueError(f"missing required endpoint: {ex}") from ex
    if len({event.clock for event in selected}) != 1:
        raise ValueError("different monotonic clocks")
    start = selected[0]
    end = max(selected[1:], key=lambda event: event.at_ns)
    raw = duration(start, end)
    waits = list(human_waits)
    if any(a < start.at_ns or b > end.at_ns for a, b in waits):
        raise ValueError("human wait outside the measured interval")
    wait_ns = union_duration(waits)
    if wait_ns and metric != "W-02":
        raise ValueError("human wait exclusion currently defined for W-02 only")
    applied_s2s = modality == "voice" and metric in {"W-01", "W-02"}
    residual = None
    if applied_s2s:
        if not s2s or not s2s.get("sample_id") or not s2s.get("trace_sha256"):
            raise ValueError("voice replay needs a frozen sample ID and trace digest")
        if s2s.get("kind") not in {"synthetic", "measured_trace_replay", "live"}:
            raise ValueError("unknown dependency evidence kind")
        if "core_input_ready" in e:
            duration(start, e["core_input_ready"])
            residual = duration(e["core_input_ready"], end)
    if modality == "text" and s2s is not None:
        raise ValueError("do not charge S2S delay to Text")
    evidence = "SIMULATED_E2E" if applied_s2s and s2s["kind"] != "live" else "RAW_ENDPOINT_OBSERVATION"
    if sink != "ACTUAL_USER_DELIVERY":
        evidence += "_HEADLESS"
    return {
        "metric": metric, "modality": modality, "wall_elapsed_ns": raw,
        "human_wait_union_ns": wait_ns, "net_endpoint_ns": raw - wait_ns,
        "post_core_ready_window_ns": residual,
        "s2s_sample_id": s2s["sample_id"] if applied_s2s else None,
        "evidence": evidence, "score": None, "p95": None,
        "note": "Elapsed endpoints, not a sum of nested spans. Residual is not pure VIA CPU time.",
    }


def paired_samples(left: dict[str, Any], right: dict[str, Any]) -> None:
    """A/B must see the same input schedule; no distribution comparison here."""
    for key in ("trial_id", "case_id", "input_sha256", "s2s_sample_id", "s2s_trace_sha256", "model_digest", "runtime_digest"):
        if key not in left or key not in right or left[key] != right[key]:
            raise ValueError(f"unpaired experimental condition: {key}")
