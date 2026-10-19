"""COMMON9 host-observed causal critical path; no interval-union subtraction.

Vertices have actual monotonic timestamps. Counted edges retain their physical
intervals, so a goal root clips prior work rather than inheriting its full cost.
External/source endpoints join their actual parents with zero cost. Model slot
predecessors join at actual acquisition (legacy fallback:send), not after decode,
and compete with queue edges by MAX.
"""
from dataclasses import dataclass


@dataclass
class Edge:
    parent: str
    start: int
    end: int
    counted: bool = True
    label: str = "VIA"


class Graph:
    def __init__(self):
        self.times = {}
        self.edges = {}
        self.issues = {}

    def vertex(self, name, at, edges=()):
        if name in self.times:
            self.issues.setdefault(name, []).append("duplicate_vertex")
        self.times[name] = at
        self.edges[name] = list(edges)

    def join(self, name, at, parents, counted=True, label="VIA_GLUE"):
        # Resolve only after all operation endpoints exist. One shared glue
        # interval follows the latest ACTUAL parent, not each excluded wait.
        known = [self.times[p] for p in parents if p in self.times]
        latest = max(known, default=at)
        if latest > at:
            self.issues.setdefault(name, []).append("parent_after_child")
        self.times[name] = at
        self.edges[name] = [Edge(p, latest, at, counted, label) for p in parents]

    def measure(self, end, root_ns, root_id=None):
        memo, visiting = {}, set()
        def walk(node):
            if node in memo:
                return memo[node]
            if node not in self.times:
                raise ValueError("missing_parent:" + str(node))
            if node in visiting:
                raise ValueError("causal_cycle:" + str(node))
            if self.issues.get(node):
                raise ValueError(node + ":" + ",".join(self.issues[node]))
            visiting.add(node)
            best, path = 0, []
            for edge in self.edges.get(node, []):
                if edge.end < edge.start:
                    raise ValueError("negative_edge:" + node)
                prior, prior_path = walk(edge.parent)
                weight = max(0, edge.end - max(root_ns, edge.start)) if edge.counted else 0
                if prior + weight >= best:
                    best = prior + weight
                    path = prior_path + [{"from": edge.parent, "to": node,
                        "start_ns": edge.start, "end_ns": edge.end,
                        "counted_ns": weight, "role": edge.label}]
            visiting.remove(node)
            memo[node] = best, path
            return memo[node]
        try:
            length, path = walk(end)
            if root_id is not None and root_id not in memo:
                raise ValueError("original_source_not_causal_ancestor:" + root_id)
            wall = self.times[end] - root_ns
            if wall < 0 or length > wall:
                raise ValueError("critical_path_exceeds_observed_wall")
            return {"status": "OBSERVED", "via_ns": length, "wall_ns": wall,
                    "critical_path": path, "root_ns": root_ns, "terminal_ns": self.times[end]}
        except ValueError as error:
            return {"status": "UNRESOLVED", "via_ns": None, "reason": str(error),
                    "root_ns": root_ns, "terminal_ns": self.times.get(end)}


def parents(event):
    return event.get("parent_ids", event.get("data", {}).get("parent_ids", []))


def build_graph(trace):
    graph = Graph()
    events = trace.get("events", [])
    starts, ends = {}, {}
    for event in events:
        kind, data = event.get("kind", ""), event.get("data", {})
        if kind in ("OPERATION_START", "LOCAL_OPERATION_START"):
            starts[data["node_id"]] = event
        elif kind in ("OPERATION_END", "LOCAL_OPERATION_END"):
            ends[data["node_id"]] = event
    calls = {}
    call_uuid_nodes = {}
    for call in trace.get("calls", []):
        node = call.get("causal_trace", {}).get("node_id")
        if node:
            if node in calls:
                graph.issues.setdefault(node, []).append("multiple_calls_for_operation")
            calls[node] = call
            call_uuid_nodes[call.get("call_uuid")] = node
    pending = []
    resources = []
    for node, start_event in starts.items():
        start = start_event.get("observed_ns")
        finish_event = ends.get(node)
        if start is None or finish_event is None or finish_event.get("observed_ns") is None:
            # An incomplete node is intentionally absent; a terminal depending
            # on it becomes unresolved rather than receiving a fabricated zero.
            continue
        finish = finish_event["observed_ns"]
        graph.vertex(node + ":start", start)
        pending.append((node + ":start", start, parents(start_event), True, "VIA_GLUE"))
        call = calls.get(node)
        role = start_event.get("data", {}).get("role", "VIA")
        if call:
            keys = ("enqueue_ns", "send_ns", "terminal_ns")
            if any(call.get(key) is None for key in keys):
                graph.vertex(node, finish)
                graph.issues.setdefault(node, []).append("missing_call_timestamps")
                continue
            enqueue, send, terminal = [call[key] for key in keys]
            chain = [(":enqueue", enqueue, ":start", start, True, "VIA_DISPATCH")]
            wait_begin, acquired = call.get("before_slot_wait_ns"), call.get("slot_acquired_ns")
            if wait_begin is not None and acquired is not None:
                chain += [(":wait", wait_begin, ":enqueue", enqueue, True, "VIA_REQUEST_SERIALIZATION"),
                    (":acquired", acquired, ":wait", wait_begin, True, "VIA_SLOT_QUEUE"),
                    (":send", send, ":acquired", acquired, True, "VIA_SEND_DISPATCH")]
                resource_vertex, resource_at, resource_wait = node + ":acquired", acquired, wait_begin
            else:
                chain += [(":send", send, ":enqueue", enqueue, True, "VIA_DISPATCH_AND_SLOT_QUEUE")]
                resource_vertex, resource_at, resource_wait = node + ":send", send, enqueue
            chain += [(":terminal", terminal, ":send", send,
                call.get("role", role) != "RECEIVER", call.get("role", role)),
                ("", finish, ":terminal", terminal, True, "VIA_DECODE_IPC")]
            for suffix, at, prior, began, counted, label in chain:
                graph.vertex(node + suffix, at, [Edge(node + prior, began, at, counted, label)])
            predecessor = call.get("resource_predecessor_call_uuid")
            if predecessor:
                resources.append((resource_vertex, resource_at, predecessor, resource_wait))
        elif start_event["kind"] == "LOCAL_OPERATION_START":
            graph.vertex(node, finish, [Edge(node + ":start", start, finish,
                         role not in ("RECEIVER", "USER_WAIT", "EXTERNAL"), role)])
        else:
            graph.vertex(node, finish)
            graph.issues.setdefault(node, []).append("missing_bridge_call")
    # Event points retain the actual publication/ingress prefix CPU. They also
    # let a source anchored at an in-progress publication avoid a backwards edge
    # to the publication's eventual LOCAL_OPERATION_END.
    points = []
    for event in events:
        if event.get("observed_ns") is not None and event.get("kind") in (
                "DISPLAY_PROXY", "AGENT_INGRESS", "CONTROL_INGRESS",
                "RECEIVER_RESULT_AVAILABLE", "GOAL_COMPLETE", "RESPONSE_FINAL"):
            name = "point:" + event["event_id"]
            graph.vertex(name, event["observed_ns"])
            points.append((name, event["observed_ns"], parents(event), True, "VIA_EVENT_GLUE"))
    source_pending = []
    for source in trace.get("source_nodes", []):
        at, source_parents = source["end_ns"], list(source.get("parents", []))
        anchor = source.get("start_ns")
        # The harness reports the exact event anchor time. Replace its retained
        # context parent with that actual event point only when both agree.
        anchors = [event for event in events if event.get("observed_ns") == anchor
                   and event.get("kind") in ("DISPLAY_PROXY", "RESPONSE_FINAL", "AGENT_INGRESS", "CONTROL_INGRESS")]
        if len(anchors) == 1:
            anchor_event = anchors[0]
            old = set(parents(anchor_event))
            source_parents = [p for p in source_parents if p not in old]
            source_parents.append("point:" + anchor_event["event_id"])
        if source.get("role") == "EXTERNAL":
            # Actual receiver receipt may follow its call terminal by VIA
            # decoding/publication work. Preserve that point before the zero
            # external endpoint. Never append the receiver duration serially.
            rewritten = []
            for parent in source_parents:
                receipts = [event for event in events
                    if event.get("kind") == "RECEIVER_RESULT_AVAILABLE"
                    and parent in parents(event) and event.get("observed_ns", at + 1) <= at]
                rewritten.append("point:" + max(receipts, key=lambda e:e["observed_ns"])["event_id"]
                                 if receipts else parent)
            source_parents = rewritten
        graph.vertex(source["id"], at)
        source_pending.append((source["id"], at, source_parents, False, source.get("role", "EXTERNAL")))
    # All endpoint times now exist; parent order in the raw event stream does
    # not affect graph construction.
    for name, at, deps, counted, label in pending + points + source_pending:
        graph.join(name, at, deps, counted, label)
    for send_node, send, predecessor, wait_begin in resources:
        owner = call_uuid_nodes.get(predecessor)
        parent = owner + ":terminal" if owner else "missing_slot_owner:" + predecessor
        if parent in graph.times and graph.times[parent] <= wait_begin:
            # last_slot_owner is historical bookkeeping, not automatically a
            # causal dependency: a slot released before this caller waited
            # cannot make unrelated earlier work part of the goal path.
            continue
        if parent in graph.times and graph.times[parent] > send:
            graph.issues.setdefault(send_node, []).append("slot_release_after_send")
        # Preserve the actual scheduling tail after both a slot was released
        # and this caller began waiting. Do not charge idle time before this
        # caller's wait, or add predecessor service to the full queue serially.
        tail_start = max(graph.times.get(parent, send), wait_begin)
        graph.edges[send_node].append(Edge(parent, tail_start, send, True, "RESOURCE_RELEASE_TO_ACQUIRE"))
    return graph
