"""Independent M04 representation alternative; never patches the baseline in place.

The ordinary B binder remains byte-identical. A reserved, finite reference
namespace is validated by the common contract and resolved to an actually
dispatched producer by the common Request-scoped continuation runtime.
"""
from __future__ import annotations

import ast
import hashlib
import importlib.util
from pathlib import Path


def _one(text, old, new):
    if text.count(old) != 1:
        raise ValueError("GOALREF source anchor changed: " + old[:100])
    return text.replace(old, new, 1)


def patch_goalref_continuation(source: Path, tools_root: Path) -> dict:
    source, tools_root = Path(source).resolve(), Path(tools_root).resolve()
    if source.name in {"frozen-source", "evaluator-tools"} or (source / ".git").exists():
        raise ValueError("patch requires an independent change-job source copy")
    patch_ref = tools_root / "benchmark/architecture/dp41_45_poc/change_patches.py"
    spec = importlib.util.spec_from_file_location("recorded_m04_common_patch", patch_ref)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    root = source / "prototypes/candidates/dp41_45_poc"
    binder = root / "dp41/b.py"
    before_binder = hashlib.sha256(binder.read_bytes()).hexdigest()
    # The A patch contains the shared producer-failure mechanism. The different
    # producer-reference encoding is then defined explicitly below.
    result = module.patch_failure_continuation(source, "A")

    runtime = root / "shared/runtime.py"
    text = runtime.read_text()
    text = _one(text,
        '        producer = predicate.get("producer_goal_id")\n'
        '        ref = self.goal_tasks.get((request_id, producer)) if producer else predicate.get("reference_id")\n',
        '        reference = predicate.get("reference_id")\n'
        '        producer = reference[5:] if isinstance(reference, str) and reference.startswith("GOAL:") else None\n'
        '        if producer and self.owners.lookup(reference) is not None:\n'
        '            raise ValueError("reserved producer reference collides with an actual owner handle")\n'
        '        ref = self.goal_tasks.get((request_id, producer)) if producer else reference\n')
    text = _one(text, 'p.get("producer_goal_id") == dependency',
        'p.get("reference_id") == "GOAL:" + dependency')
    text = _one(text, '        relations = meaning.get("relations", [])\n',
        '        relations = meaning.get("relations", [])\n'
        '        for relation in relations:\n'
        '            ref = relation.get("predicate", {}).get("reference_id")\n'
        '            if (relation.get("op") == "WHEN" and isinstance(ref, str) and ref.startswith("GOAL:")\n'
        '                    and self.owners.lookup(ref) is not None):\n'
        '                raise ValueError("reserved producer reference collides with an actual owner handle")\n')
    ast.parse(text)
    runtime.write_text(text)

    contract = root / "dp41/contract.py"
    text = contract.read_text()
    start = text.index('            if "producer_goal_id" in p:\n')
    end = text.index('            if (not isinstance(p, dict)', start)
    text = text[:start] + '''            if "producer_goal_id" in p:
                raise ContractError("GOALREF uses only the finite reference_id namespace")
            ref = p.get("reference_id")
            if isinstance(ref, str) and ref.upper().startswith("GOAL:"):
                if ref != "GOAL:" + source or p.get("field") != "STATE":
                    raise ContractError("GOALREF must name this relation's actual producer goal")
                terminal = ("FAILED", "COMPLETED")
                expected, comparison = p.get("value"), p.get("comparison")
                valid = comparison == "EQ" and expected in terminal
                if comparison == "IN":
                    valid = (isinstance(expected, list) and 1 <= len(expected) <= 2
                             and all(v in terminal for v in expected) and len(set(expected)) == len(expected))
                if set(p) != {"reference_id", "field", "comparison", "value"} or not valid:
                    raise ContractError("unsupported finite GOALREF terminal predicate")
''' + text[end:]
    text = _one(text, '    edges = {i: [] for i in ids}\n',
        '    if any(i.upper().startswith("GOAL:") for i in ids):\n'
        '        raise ContractError("goal IDs cannot use the reserved GOALREF namespace")\n'
        '    edges = {i: [] for i in ids}\n')
    text = _one(text, '        reference_ids = set(ids)\n',
        '        if any(i.upper().startswith("GOAL:") for i in ids):\n'
        '            raise ContractError("frame reference IDs cannot use the reserved GOALREF namespace")\n'
        '        reference_ids = set(ids)\n')
    text = _one(text,
        '            if (relation["op"] == "WHEN" and "producer_goal_id" not in relation["predicate"]\n'
        '                    and relation["predicate"].get("reference_id") not in reference_ids):\n',
        '            if (relation["op"] == "WHEN"\n'
        '                    and not str(relation["predicate"].get("reference_id", "")).startswith("GOAL:")\n'
        '                    and relation["predicate"].get("reference_id") not in reference_ids):\n')
    ast.parse(text)
    contract.write_text(text)

    prompts = root / "dp41/prompts.py"
    text = prompts.read_text()
    text = _one(text,
        "For a current goal's terminal state, use producer_goal_id:from_goal instead of reference_id,\n",
        "For a current goal's terminal state, use reference_id:'GOAL:' + from_goal. This reserved\n"
        "namespace names only a current goal; never declare it as a Frame reference or use an owner ID.\n")
    ast.parse(text)
    prompts.write_text(text)
    after_binder = hashlib.sha256(binder.read_bytes()).hexdigest()
    if before_binder != after_binder:
        raise ValueError("GOALREF changed the B resolution/binding responsibility")
    claims = result["responsibility_claims"]
    claims["U01"]["after_contract"] = "finite GOAL:<from_goal> typed terminal WHEN"
    claims["U03"]["reason"] = "Strict reserved GOAL namespace, own producer identity, terminal enum and Frame reference collision rejection."
    claims["U03"]["after_contract"] = "finite GOALREF terminal predicate with disjoint owner/Frame namespaces"
    claims["U08"]["reason"] = "Map current Request/goal to actual dispatched producer Task; reject actual owner collision and preserve successful-result dependencies."
    claims["U08"]["after_contract"] = "actual Request-scoped producer readiness resolved from finite GOALREF"
    for claim in claims.values():
        claim["functional_evidence"] = ["goalref-runtime-checks.json", "regression.json", "b-binder-integrity.json"]
    result["unchanged_responsibilities"] = {
        "U02": "Existing WHEN reference binding preserves undeclared strings; reserved namespace is checked in U03 and resolved by common U08. B binder byte hash is unchanged."
    }
    result["representation_alternative"] = "GOALREF-v1"
    result["b_binder_sha256"] = {"before": before_binder, "after": after_binder, "identical": True}
    return result
