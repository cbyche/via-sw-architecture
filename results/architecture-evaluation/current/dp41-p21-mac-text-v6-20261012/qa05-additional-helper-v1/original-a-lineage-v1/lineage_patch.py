"""Original A M04 semantics with corrected actual producer dispatch lineage.

Apply only to a fresh copy of the common frozen baseline. A's semantic loop,
producer_goal_id representation, prompts and enum support remain unchanged
relative to its original M04 delta. Only its U08 mapping gains revision and
actual Task/Execution guards.
"""
from __future__ import annotations
import ast
import hashlib
import importlib.util
from pathlib import Path


def _one(text, old, new):
    if text.count(old) != 1: raise ValueError("A lineage patch anchor changed: " + old[:100])
    return text.replace(old, new, 1)


def patch_corrected_a_failure(source, tools_root):
    source, tools_root = Path(source).resolve(), Path(tools_root).resolve()
    if source.name in {"frozen-source", "evaluator-tools"} or (source / ".git").exists():
        raise ValueError("patch requires an independent source copy")
    root = source / "prototypes/candidates/dp41_45_poc"
    semantic = root / "dp41/a.py"
    before_a = hashlib.sha256(semantic.read_bytes()).hexdigest()
    spec = importlib.util.spec_from_file_location("original_a_m04_patch",
        tools_root / "benchmark/architecture/dp41_45_poc/change_patches.py")
    common = importlib.util.module_from_spec(spec); spec.loader.exec_module(common)
    result = common.patch_failure_continuation(source, "A")
    runtime = root / "shared/runtime.py"
    original_delta_runtime = runtime.read_text()
    text = _one(original_delta_runtime,
        '    def _predicate(self, predicate, request_id=None):\n'
        '        producer = predicate.get("producer_goal_id")\n'
        '        ref = self.goal_tasks.get((request_id, producer)) if producer else predicate.get("reference_id")\n'
        '        item = self.owners.lookup(ref)\n',
        '    def _predicate(self, predicate, request_scope=None):\n'
        '        producer = predicate.get("producer_goal_id")\n'
        '        binding = self.goal_tasks.get((*request_scope, producer)) if producer and request_scope else None\n'
        '        item = self.owners.lookup(binding["task_id"]) if binding else (None if producer else self.owners.lookup(predicate.get("reference_id")))\n'
        '        if binding and item and (item.get("execution_id") != binding["execution_id"]\n'
        '                or item.get("request_revision", 1) != binding["request_revision"]):\n'
        '            return None  # A later Task execution/revision is not this producer terminal.\n')
    text = text.replace('self._predicate(predicate, ports.request_id)',
                        'self._predicate(predicate, (ports.request_id, ports.revision))')
    text = _one(text, '        self.goal_tasks[(ports.request_id, goal["id"])] = target\n',
        '        self.goal_tasks[(ports.request_id, ports.revision, goal["id"])] = {"task_id": target,\n'
        '            "execution_id": task["execution_id"], "request_revision": task.get("request_revision", 1)}\n')
    ast.parse(text); runtime.write_text(text)
    if hashlib.sha256(semantic.read_bytes()).hexdigest() != before_a:
        raise ValueError("lineage correction changed A semantic interpretation")
    result["responsibility_claims"]["U08"]["reason"] = (
        "Bind failed producer readiness to current Request revision and actual Task/Execution/Task revision, "
        "preserving UNKNOWN for absent or superseded dispatches and successful-result dependencies.")
    result["responsibility_claims"]["U08"]["after_contract"] = (
        "original finite producer_goal_id continuation with actual Request-revision dispatch lineage")
    for claim in result["responsibility_claims"].values():
        claim["functional_evidence"] = ["runtime-requirement-checks.json", "producer-lineage-checks.json", "regression.json"]
    result["representation_alternative"] = "ORIGINAL-A-LINEAGE-v1"
    result["a_semantic_sha256"] = {"before": before_a, "after": before_a, "identical": True}
    result["correction_relative_to_original_a_m04"] = {
        "responsibilities": ["U08"], "original_delta_runtime_sha256": hashlib.sha256(original_delta_runtime.encode()).hexdigest(),
        "corrected_runtime_sha256": hashlib.sha256(text.encode()).hexdigest(),
        "representation": "original producer_goal_id",
        "candidate_algorithm_changed": False}
    return result
