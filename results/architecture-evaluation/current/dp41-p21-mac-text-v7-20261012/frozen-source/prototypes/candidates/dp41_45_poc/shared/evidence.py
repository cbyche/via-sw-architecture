"""Append-only evidence; secrets never enter the persisted request envelope."""
from __future__ import annotations

import hashlib
import json
import time
import threading
from pathlib import Path


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def digest(value):
    return hashlib.sha256(canonical(value).encode()).hexdigest()


class Evidence:
    def __init__(self, directory, variant="", repeat=0):
        self.directory = Path(directory)
        self.directory.mkdir(parents=True, exist_ok=True)
        self.variant, self.repeat = variant, repeat
        self.origin = time.monotonic_ns()
        self.records = {}
        self.handles = {}
        self.sequence = 0
        self.lock = threading.RLock()

    def seconds(self):
        return (time.monotonic_ns() - self.origin) / 1e9

    def write(self, stream, value):
        with self.lock:
            return self._write(stream, value)

    def _write(self, stream, value):
        self.sequence += 1
        record = {**value, "schema_version": "dp41-evidence-1", "evidence_id": f"E{self.sequence}",
                  "monotonic_ns": time.monotonic_ns(), "at_seconds": self.seconds(),
                  "variant": value.get("variant", self.variant), "repeat": value.get("repeat", self.repeat)}
        if stream not in self.handles:
            self.handles[stream] = (self.directory / f"{stream}.jsonl").open("a", encoding="utf-8")
        encoded = canonical(record)
        self.handles[stream].write(encoded + "\n")
        self.handles[stream].flush()
        if stream != "memory":
            # Preserve the same immutable event-time snapshot in RAM and JSONL.
            # A later owner transition cannot rewrite an earlier question receipt.
            record = json.loads(encoded)
            self.records.setdefault(stream, []).append(record)
        return record

    def artifact(self, folder, name, value):
        directory = self.directory / folder
        directory.mkdir(exist_ok=True)
        path = directory / f"{name}.json"
        path.write_text(canonical(value) + "\n", encoding="utf-8")
        return {"path": str(path.relative_to(self.directory)), "sha256": digest(value)}

    def close(self):
        for handle in self.handles.values():
            handle.close()
