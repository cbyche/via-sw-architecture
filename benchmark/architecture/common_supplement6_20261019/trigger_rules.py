"""Contract20 host-observed response-anchor boundary, independent of polling."""
import threading
import time


def final_anchor(provided_at_ns, observed_final_ns, now_ns, timeout_ns):
    deadline = provided_at_ns + timeout_ns
    if observed_final_ns is not None and observed_final_ns <= deadline:
        return {'kind': 'ACTUAL_FINAL', 'at_ns': observed_final_ns, 'anchor_deadline_ns': deadline}
    if now_ns >= deadline:
        return {'kind': 'MISSING_AT_30_SECONDS', 'at_ns': deadline, 'anchor_deadline_ns': deadline, 'late_actual_final_ns': observed_final_ns}
    return None


class FinalAnchorRegistry:
    """Stamp/publish receipt and choose its boundary under one short lock.

    No IO, waits, semantic work or user callback occurs in these critical paths.
    The measured receipt timestamp is captured after acquiring this mutex, so a
    polling choice cannot overtake an already timestamped but unpublished fact.
    """
    def __init__(self, clock=time.monotonic_ns):
        self.clock = clock
        self.lock = threading.Lock()
        self.records = {}
        self.choices = {}

    def observe(self, stimulus_id, parent_node):
        with self.lock:
            at = self.clock()
            self.records.setdefault(stimulus_id, {'observed_ns': at, 'parent_node': parent_node})
        return at

    def choose(self, stimulus_id, provided_at_ns, timeout_ns):
        with self.lock:
            if stimulus_id in self.choices:
                return None
            record = self.records.get(stimulus_id)
            choice = final_anchor(provided_at_ns, record['observed_ns'] if record else None, self.clock(), timeout_ns)
            if choice is not None:
                choice['parent_node'] = record['parent_node'] if choice['kind'] == 'ACTUAL_FINAL' else None
                self.choices[stimulus_id] = choice
            return choice

    def snapshot(self):
        with self.lock:
            return {key: dict(value) for key, value in self.records.items()}
