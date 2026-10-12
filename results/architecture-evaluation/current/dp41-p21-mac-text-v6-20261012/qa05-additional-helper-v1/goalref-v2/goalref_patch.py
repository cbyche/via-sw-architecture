"""Typed producer-reference M04 alternative with byte-identical B binder."""
from __future__ import annotations
import ast
import hashlib
import importlib.util
from pathlib import Path


def _one(text, old, new):
    if text.count(old) != 1: raise ValueError("typed GOALREF anchor changed: " + old[:100])
    return text.replace(old, new, 1)


def patch_goalref_continuation(source, tools_root):
    source, tools_root = Path(source).resolve(), Path(tools_root).resolve()
    if source.name in {"frozen-source", "evaluator-tools"} or (source / ".git").exists():
        raise ValueError("patch requires an independent source copy")
    spec = importlib.util.spec_from_file_location("original_m04_common_patch",
        tools_root / "benchmark/architecture/dp41_45_poc/change_patches.py")
    common = importlib.util.module_from_spec(spec); spec.loader.exec_module(common)
    root = source / "prototypes/candidates/dp41_45_poc"
    binder = root / "dp41/b.py"
    binder_sha = hashlib.sha256(binder.read_bytes()).hexdigest()
    result = common.patch_failure_continuation(source, "A")
    runtime = root / "shared/runtime.py"
    text = runtime.read_text()
    text = _one(text,
        '    def _predicate(self, predicate, request_id=None):\n'
        '        producer = predicate.get("producer_goal_id")\n'
        '        ref = self.goal_tasks.get((request_id, producer)) if producer else predicate.get("reference_id")\n'
        '        item = self.owners.lookup(ref)\n',
        '    def _predicate(self, predicate, request_scope=None):\n'
        '        ref = predicate.get("reference_id")\n'
        '        producer = ref.get("producer_goal_id") if isinstance(ref, dict) else None\n'
        '        binding = self.goal_tasks.get((*request_scope, producer)) if producer and request_scope else None\n'
        '        item = self.owners.lookup(binding["task_id"]) if binding else (None if producer else self.owners.lookup(ref))\n'
        '        if binding and item and (item.get("execution_id") != binding["execution_id"]\n'
        '                or item.get("request_revision", 1) != binding["request_revision"]):\n'
        '            return None  # A later Task execution/revision is not this producer terminal.\n')
    text = text.replace('self._predicate(predicate, ports.request_id)',
                        'self._predicate(predicate, (ports.request_id, ports.revision))')
    text = _one(text, 'p.get("producer_goal_id") == dependency',
        'isinstance(p.get("reference_id"), dict) and p["reference_id"].get("producer_goal_id") == dependency')
    text = _one(text, '        self.goal_tasks[(ports.request_id, goal["id"])] = target\n',
        '        self.goal_tasks[(ports.request_id, ports.revision, goal["id"])] = {"task_id": target,\n'
        '            "execution_id": task["execution_id"], "request_revision": task.get("request_revision", 1)}\n')
    ast.parse(text); runtime.write_text(text)
    contract = root / "dp41/contract.py"
    text = contract.read_text()
    start = text.index('            if "producer_goal_id" in p:\n')
    end = text.index('            if (not isinstance(p, dict)', start)
    text = text[:start] + '''            if "producer_goal_id" in p:
                raise ContractError("producer identity belongs inside the typed reference_id")
            ref = p.get("reference_id")
            if isinstance(ref, dict):
                if (set(ref) != {"producer_goal_id"} or not isinstance(ref.get("producer_goal_id"), str)
                        or ref["producer_goal_id"] != source or p.get("field") != "STATE"):
                    raise ContractError("typed producer reference must name its own current finite goal")
                terminal = ("FAILED", "COMPLETED")
                expected, comparison = p.get("value"), p.get("comparison")
                valid = comparison == "EQ" and expected in terminal
                if comparison == "IN":
                    valid = (isinstance(expected, list) and 1 <= len(expected) <= 2
                             and all(v in terminal for v in expected) and len(set(expected)) == len(expected))
                if set(p) != {"reference_id", "field", "comparison", "value"} or not valid:
                    raise ContractError("unsupported typed producer terminal predicate")
''' + text[end:]
    text = _one(text, '                    and not isinstance(relation["predicate"]["reference_id"], str)):\n',
        '                    and not isinstance(relation["predicate"]["reference_id"], (str, dict))):\n')
    text = _one(text,
        '            if (relation["op"] == "WHEN" and "producer_goal_id" not in relation["predicate"]\n'
        '                    and relation["predicate"].get("reference_id") not in reference_ids):\n',
        '            if (relation["op"] == "WHEN" and not isinstance(relation["predicate"].get("reference_id"), dict)\n'
        '                    and relation["predicate"].get("reference_id") not in reference_ids):\n')
    ast.parse(text); contract.write_text(text)
    prompts = root / "dp41/prompts.py"
    text = prompts.read_text()
    text = _one(text, "For a current goal's terminal state, use producer_goal_id:from_goal instead of reference_id,\n",
        "For a current goal's terminal state, use reference_id:{{producer_goal_id:from_goal}}.\n"
        "This is an exact singleton tagged object naming this relation's current goal, never an owner string or Frame reference ID.\n")
    ast.parse(text); prompts.write_text(text)
    if hashlib.sha256(binder.read_bytes()).hexdigest() != binder_sha:
        raise ValueError("typed producer reference changed the B binder")
    claims = result["responsibility_claims"]
    claims["U01"]["after_contract"] = "finite tagged reference_id producer terminal WHEN"
    claims["U03"]["reason"] = "Exact singleton producer tag, own from_goal, terminal enum and existing owner-string/frame disjointness validation."
    claims["U03"]["after_contract"] = "owner string or exact typed producer reference, STATE terminal EQ/IN only for producers"
    claims["U08"]["reason"] = "Resolve the actual producer Task/Execution from Request revision and goal; preserve UNKNOWN for absent/superseded producers and success-result dependencies."
    claims["U08"]["after_contract"] = "typed producer readiness bound to actual Request-revision dispatch lineage"
    for claim in claims.values(): claim["functional_evidence"] = ["goalref-runtime-checks.json", "regression.json", "b-binder-integrity.json"]
    result["unchanged_responsibilities"] = {"U02": "Existing recursive B binding preserves the producer tag because goal IDs and Frame reference IDs are disjoint; binder byte hash is unchanged."}
    result["representation_alternative"] = "TYPED-GOALREF-v2"
    result["b_binder_sha256"] = {"before": binder_sha, "after": binder_sha, "identical": True}
    return result
