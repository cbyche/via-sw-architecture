"""macOS kernel task accounting, sampled within the measured process."""
from __future__ import annotations

import ctypes
import os
import platform
import threading
import time
import uuid


def task_memory():
    if platform.system() != "Darwin":
        raise RuntimeError("This campaign requires macOS task_info; no RSS substitution")
    library = ctypes.CDLL("/usr/lib/libSystem.B.dylib")
    port = ctypes.c_uint.in_dll(library, "mach_task_self_").value
    buffer = (ctypes.c_uint64 * 128)()
    count = ctypes.c_uint(256)
    function = library.task_info
    function.argtypes = [ctypes.c_uint, ctypes.c_uint, ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint)]
    function.restype = ctypes.c_int
    code = function(port, 22, buffer, ctypes.byref(count))
    if code or count.value < 38:
        raise RuntimeError(f"TASK_VM_INFO unavailable (code={code}, count={count.value})")
    return {"resident_size_bytes": buffer[2], "phys_footprint_bytes": buffer[18],
            "task_info_count": count.value, "task_info_flavor": 22}


class MemorySampler:
    def __init__(self, evidence, interval=.05):
        self.evidence, self.interval = evidence, interval
        self.generation = str(uuid.uuid4())
        self.phase = "PREPARATION"
        self.stop_event = threading.Event()
        self.thread = threading.Thread(target=self._run, name="via-memory-sampler", daemon=True)

    def _run(self):
        sequence = 0
        due = time.monotonic()
        while not self.stop_event.is_set():
            sequence += 1
            start = time.monotonic_ns()
            try:
                values = {**task_memory(), "success": True}
            except Exception as error:
                values = {"phys_footprint_bytes": None, "resident_size_bytes": None,
                          "success": False, "error": str(error)}
            self.evidence.write("memory", {"sample_id": sequence, "process_id": os.getpid(),
                                           "generation": self.generation, "included_in_via_sum": True,
                                           "phase": self.phase, "sample_start_ns": start,
                                           "sample_end_ns": time.monotonic_ns(), **values})
            due += self.interval
            self.stop_event.wait(max(0., due - time.monotonic()))

    def start(self):
        task_memory()  # Fail before measurement when the ABI cannot be observed.
        self.thread.start()

    def stop(self):
        self.stop_event.set()
        self.thread.join(timeout=2)
