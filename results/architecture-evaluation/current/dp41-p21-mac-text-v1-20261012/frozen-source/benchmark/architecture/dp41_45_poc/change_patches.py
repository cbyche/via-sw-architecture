"""Actual M04 delta, applied only to an independently copied frozen baseline."""
from pathlib import Path


def _one(text, old, new):
    if text.count(old) != 1:
        raise ValueError("M04 frozen source contract changed: " + old[:90])
    return text.replace(old, new, 1)


def patch_failure_continuation(source: Path, variant: str) -> dict:
    source = Path(source)
    root = source / "prototypes/candidates/dp41_45_poc"
    if not root.is_dir():
        raise ValueError("M04 needs the complete frozen candidate package")
    changed = {}
    runtime = root / "shared/runtime.py"
    text = runtime.read_text()
    text = _one(text, "        self.memory_records = {}\n", "        self.memory_records = {}\n        self.goal_tasks, self.task_failures = {}, {}\n")
    text = _one(text, "    def _predicate(self, predicate):\n        item = self.owners.lookup(predicate.get(\"reference_id\"))",
        "    def _predicate(self, predicate, request_id=None):\n        producer = predicate.get(\"producer_goal_id\")\n        ref = self.goal_tasks.get((request_id, producer)) if producer else predicate.get(\"reference_id\")\n        item = self.owners.lookup(ref)")
    text = text.replace("self._predicate(predicate)", "self._predicate(predicate, ports.request_id)")
    text = _one(text, "            for dependency in dependencies[gid]:\n                await tasks[dependency]\n",
        "            for dependency in dependencies[gid]:\n                outcome = await tasks[dependency]\n                when_failure = any(p.get(\"producer_goal_id\") == dependency and p.get(\"field\") == \"STATE\"\n                    and p.get(\"comparison\") == \"EQ\" and p.get(\"value\") == \"FAILED\" for p in conditions[gid])\n                if isinstance(outcome, dict) and outcome.get(\"failed\") and not when_failure:\n                    raise ValueError(\"Dependent goal needs a successful producer result\")\n")
    text = _one(text, "        command_id, command_time = \"CMD:\" + str(uuid.uuid4()), self.now()\n",
        "        self.goal_tasks[(ports.request_id, goal[\"id\"])] = target\n        self.task_failures.setdefault(target, asyncio.get_running_loop().create_future())\n        command_id, command_time = \"CMD:\" + str(uuid.uuid4()), self.now()\n")
    text = _one(text, '        task["status"] = "CANCEL_ACCEPTED_TERMINATION_UNKNOWN" if operation == "CANCEL_TASK" else "RUNNING"\n',
        '        if task.get("status") != "FAILED":\n            task["status"] = "CANCEL_ACCEPTED_TERMINATION_UNKNOWN" if operation == "CANCEL_TASK" else "RUNNING"\n')
    text = _one(text, "        await asyncio.sleep(max(0., effect[\"ingress_accepted_s\"] + final_delay - self.now()))\n",
        "        failure_future = self.task_failures[target]\n        try:\n            failure = await asyncio.wait_for(asyncio.shield(failure_future),\n                timeout=max(0., effect[\"ingress_accepted_s\"] + final_delay - self.now()))\n        except asyncio.TimeoutError:\n            failure = None\n        if failure is not None or task.get(\"status\") == \"FAILED\":\n            failed = {\"failed\": True, \"task\": copy.deepcopy(task), \"failure\": copy.deepcopy(failure)}\n            key = (ports.request_id, goal[\"id\"])\n            self.goal_result_nodes[key] = self.timeline.point(\"PRODUCER_FAILED_READY\",\n                parents=[failure[\"causal_node_id\"]] if failure else [ingress_node], request_id=ports.request_id)\n            ready = self._ready(*key)\n            if not ready.done():\n                ready.set_result(failed)\n            return failed\n")
    text = _one(text, '        elif kind in ("RESULT", "ACCEPTED", "CANCEL_RECEIPT"):\n',
        '        elif kind in ("RESULT", "FAILURE", "ACCEPTED", "CANCEL_RECEIPT"):\n')
    text = _one(text, '            if kind != "RESULT":\n', '''            if kind == "FAILURE":
                if task.get("status") in ("FAILED", "COMPLETED"):
                    self.emit("RESULT_REJECTED", {"source_event_id": event_id, "reason": "ALREADY_TERMINAL"})
                    return
                task["status"], task["failure"] = "FAILED", {"reason": payload.get("reason", "EXTERNAL_FAILURE"), "event_id": event_id}
                task.pop("result", None)
                effect = next((e for e in self.effects if e["task_id"] == payload["task_id"] and e["execution_id"] == payload["execution_id"]), None)
                parents = list(event.get("causal_parents", []))
                if effect:
                    ingress = effect["ingress_node_id"]
                    work = self.timeline.add_node("EXTERNAL_AGENT_WORK", "EXTERNAL_AGENT", self.timeline.nodes[ingress]["end_s"],
                        self.timeline.now(), parents=[ingress], metadata={"terminal_event_id": event_id})
                    parents.append(work)
                node = self.timeline.point("TASK_FAILED", parents=parents, metadata={"task_id": payload["task_id"], "event_id": event_id})
                failure = {**copy.deepcopy(task["failure"]), "causal_node_id": node}
                future = self.task_failures.get(payload["task_id"])
                if future is not None and not future.done():
                    future.set_result(failure)
                self.emit("TASK_FAILED", {"task": copy.deepcopy(task), "source_event_id": event_id, "causal_node_id": node})
                self.state_changed.set()
                self.track(self.response({"task": copy.deepcopy(task), "receipt": "FAILED", "completion_confirmed": False,
                    "_causal_parents": [node]}, kind="FAILURE", source_event=event_id))
                return
            if kind != "RESULT":
''')
    text = _one(text, '            if task.get("status") == "COMPLETED" and task.get("result"):\n',
        '            if task.get("status") == "FAILED":\n                self.emit("RESULT_REJECTED", {"source_event_id": event_id, "reason": "FAILED_EXECUTION_TERMINAL"})\n                return\n            if task.get("status") == "COMPLETED" and task.get("result"):\n')
    changed[runtime] = text
    contract = root / "dp41/contract.py"
    text = contract.read_text()
    text = _one(text, '            p = relation.get("predicate", {})\n', '''            p = relation.get("predicate", {})
            if "producer_goal_id" in p:
                if "reference_id" in p or p["producer_goal_id"] != source or p.get("field") != "STATE":
                    raise ContractError("WHEN producer state must reference its own finite producer goal")
                if set(p) != {"producer_goal_id", "field", "comparison", "value"} or p.get("comparison") != "EQ" or p.get("value") not in ("FAILED", "COMPLETED"):
                    raise ContractError("unsupported producer terminal predicate")
''')
    changed[contract] = text
    prompts = root / "dp41/prompts.py"
    text = prompts.read_text()
    text = _one(text, "uses reference_id,field:STATE|KIND|YEAR,comparison:EQ|NE|IN,value. UNKNOWN is not true or false.\n",
        "uses reference_id,field:STATE|KIND|YEAR,comparison:EQ|NE|IN,value. UNKNOWN is not true or false.\n"
        "For a current goal's terminal state, use producer_goal_id:from_goal instead of reference_id,\n"
        "field:STATE,comparison:EQ,value:FAILED|COMPLETED. A failed goal has no successful artifact.\n"
        "WHEN FAILED may continue a draft only; AFTER/USE_RESULT still require successful results.\n")
    changed[prompts] = text
    if variant == "B":
        binder = root / "dp41/b.py"
        text = binder.read_text()
        text = _one(text, '        if relation.get("op") == "WHEN":\n',
            '        if relation.get("op") == "WHEN" and "reference_id" in relation.get("predicate", {}):\n')
        changed[binder] = text
    import ast
    for path, text in changed.items():
        ast.parse(text)
        path.write_text(text)
    runtime_ref, contract_ref, prompt_ref = (str(p.relative_to(source)) for p in (runtime, contract, prompts))
    definitions = {
        "U01": ("Natural-language producer-failure condition is explicit in both model contracts.", "existing owner-state WHEN", "finite current-producer terminal WHEN", [prompt_ref]),
        "U03": ("Validate producer identity and the bounded terminal-state predicate without arbitrary expressions.", "owner reference predicates", "exclusive owner/producer references and terminal enum validation", [contract_ref]),
        "U07": ("Task owner accepts actual terminal FAILURE with revision/Execution checks and fences duplicate/late success.", "successful RESULT and receipt ingress", "terminal FAILURE ingress and one-way FAILED state", [runtime_ref]),
        "U08": ("Dispatcher resolves actual failed producer readiness and runs failure continuation once while blocking success-dependent paths.", "successful producer completion readiness", "failure readiness and typed failure continuation scheduling", [runtime_ref]),
    }
    if variant == "B":
        definitions["U02"] = ("Frame binder preserves symbolic producer identity instead of requiring a pre-existing owner reference.",
            "all WHEN predicates bind reference_id", "only owner predicates bind reference_id; producer identity is finite and retained", [str(binder.relative_to(source))])
    return {"responsibility_claims": {u: {"reason": d[0], "before_contract": d[1], "after_contract": d[2],
        "changed_paths": d[3], "functional_evidence": ["runtime-requirement-checks.json", "regression.json"]} for u, d in definitions.items()}}
