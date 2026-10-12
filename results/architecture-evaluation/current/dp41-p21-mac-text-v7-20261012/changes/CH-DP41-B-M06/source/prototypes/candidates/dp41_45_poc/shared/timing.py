"""Observed dependency DAG and external-duration-only attribution.

All coordinates use Evidence.seconds(), including preparation and later scene
origin resets. Nodes describe actual service spans; resource queue time is an
edge to the actual preceding resource occupant, not another duplicate span.
"""
from __future__ import annotations

import copy
import math
import threading
import time
import uuid


EXCLUDED_KINDS = frozenset({"EXTERNAL_AGENT_WORK", "EXTERNAL_USER_WAIT", "EXTERNAL_USER_SPEECH"})


class TimingContractError(ValueError):
    pass


def _seconds(value):
    if isinstance(value, bool):
        raise TimingContractError("boolean is not a timestamp")
    try:
        result = float(value)
    except (ValueError, TypeError) as error:
        raise TimingContractError("timestamp must be numeric") from error
    if not math.isfinite(result):
        raise TimingContractError("timestamp must be finite")
    return result


class CausalTimeline:
    def __init__(self, evidence=None, clock=None):
        self.evidence = evidence
        self.clock = clock or (evidence.seconds if evidence is not None else time.monotonic)
        self.timeline_id = "TL:" + uuid.uuid4().hex
        self.nodes = {}
        self.sequence = 0
        self.lock = threading.RLock()

    def now(self):
        return _seconds(self.clock())

    def span(self, kind, owner, start_s, end_s, parents=None, evidence=None, request_id="", metadata=None, node_id=None):
        start_s, end_s = _seconds(start_s), _seconds(end_s)
        if end_s < start_s:
            raise TimingContractError("span ends before it starts")
        if not isinstance(kind, str) or not kind or not isinstance(owner, str) or not owner:
            raise TimingContractError("node kind and owner are required")
        parents = list(dict.fromkeys(parents or []))
        with self.lock:
            self.sequence += 1
            node_id = node_id or f"{self.timeline_id}:N{self.sequence}"
            if node_id in self.nodes:
                raise TimingContractError(f"duplicate node {node_id}")
            for parent in parents:
                if parent not in self.nodes:
                    raise TimingContractError(f"missing/cross-timeline parent {parent}")
                if self.nodes[parent]["end_s"] > start_s + 1e-6:
                    raise TimingContractError("parent must finish before child service starts; queue is an edge, not duplicate service")
            node = {"node_id": node_id, "timeline_id": self.timeline_id, "kind": kind, "owner": owner,
                    "start_s": start_s, "end_s": end_s, "duration": end_s - start_s,
                    "parents": parents, "request_id": request_id,
                    "evidence": copy.deepcopy(evidence or []), "metadata": copy.deepcopy(metadata or {})}
            self.nodes[node_id] = node
            if self.evidence is not None:
                self.evidence.write("causal-nodes", node)
            return node_id

    add_node = span

    def point(self, kind, at_s=None, parents=None, owner="LOCAL_VIA", evidence=None, request_id="", metadata=None, node_id=None):
        at_s = self.now() if at_s is None else at_s
        return self.span(kind, owner, at_s, at_s, parents=parents, evidence=evidence,
                         request_id=request_id, metadata=metadata, node_id=node_id)

    def attribute(self, start_s, endpoint_node_id):
        """Recompute longest dependency path with only the three external kinds at zero.

        Waiting/scheduling gaps not represented by an external node remain VIA
        time. Actual wall-clock reconstruction is checked before attribution.
        A wrong/missing endpoint must be rejected by the evaluator, not invented
        here; metric cap replacement is applied after this computation.
        """
        start_s = _seconds(start_s)
        with self.lock:
            source = copy.deepcopy(self.nodes)
        if endpoint_node_id not in source:
            raise TimingContractError("endpoint has no observed causal node")
        endpoint_s = source[endpoint_node_id]["end_s"]
        if endpoint_s < start_s:
            raise TimingContractError("endpoint precedes the measurement boundary")
        order, active, done = [], set(), set()
        def visit(node_id):
            if node_id in active:
                raise TimingContractError("causal dependency cycle")
            if node_id in done:
                return
            if node_id not in source:
                raise TimingContractError("causal node references missing parent")
            active.add(node_id)
            for parent in source[node_id]["parents"]:
                visit(parent)
            active.remove(node_id)
            done.add(node_id)
            order.append(node_id)
        visit(endpoint_node_id)
        actual, attributed, paths, rows = {}, {}, {}, []
        for node_id in order:
            node = source[node_id]
            parents = node["parents"]
            if any(source[p]["end_s"] > node["start_s"] + 1e-6 for p in parents):
                raise TimingContractError("invalid temporal dependency in preserved DAG")
            clipped_start = max(start_s, node["start_s"])
            clipped_end = max(start_s, node["end_s"])
            duration = max(0., clipped_end - clipped_start)
            required_available = max([start_s, *[source[p]["end_s"] for p in parents]])
            gap = max(0., node["start_s"] - required_available)
            actual_parent = max((actual[p] for p in parents), default=0.)
            attributed_parent = max((attributed[p] for p in parents), default=0.)
            actual[node_id] = actual_parent + gap + duration
            retained = 0. if node["kind"] in EXCLUDED_KINDS else duration
            attributed[node_id] = attributed_parent + gap + retained
            chosen = max(parents, key=lambda p: attributed[p]) if parents else None
            paths[node_id] = [*(paths[chosen] if chosen else []), node_id]
            rows.append({**node, "clipped_start_s": clipped_start, "clipped_end_s": clipped_end,
                         "clipped_duration_seconds": duration, "via_scheduling_gap_seconds": gap,
                         "retained_duration_seconds": retained, "excluded_duration_seconds": duration - retained,
                         "actual_recomputed_seconds": actual[node_id], "attributed_recomputed_seconds": attributed[node_id]})
        wall = endpoint_s - start_s
        reconstructed = actual[endpoint_node_id]
        if abs(wall - reconstructed) > max(1e-6, abs(wall) * 1e-9):
            raise TimingContractError("observed DAG does not reconstruct the actual endpoint wall clock")
        return {"timeline_id": self.timeline_id, "start_s": start_s, "endpoint_s": endpoint_s,
                "endpoint_node_id": endpoint_node_id, "wall_clock_seconds": wall,
                "attributed_seconds": attributed[endpoint_node_id], "actual_recomputed_seconds": reconstructed,
                "excluded_kinds": sorted(EXCLUDED_KINDS), "nodes": rows, "path": paths[endpoint_node_id],
                "attribution_rule": "SAME_OBSERVED_DAG_EXTERNAL_DURATIONS_ZERO_LONGEST_PATH",
                "gap_policy": "UNEXPLAINED_OBSERVED_SCHEDULING_GAP_IS_VIA_TIME"}

    def snapshot(self):
        with self.lock:
            return {"timeline_id": self.timeline_id, "clock_basis": "EVIDENCE_SECONDS_COMMON_ABSOLUTE",
                    "excluded_kinds": sorted(EXCLUDED_KINDS), "nodes": copy.deepcopy(list(self.nodes.values()))}
