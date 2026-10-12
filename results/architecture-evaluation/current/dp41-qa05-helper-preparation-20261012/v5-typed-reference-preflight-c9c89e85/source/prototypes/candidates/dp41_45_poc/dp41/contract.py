"""Finite wire contracts shared by the two interpreters.

These checks validate structure and provenance, never evaluator correctness.
"""
from __future__ import annotations

import inspect
import json
from dataclasses import dataclass

MEANING_VERSION = "dp41-meaning-runtime-1"
FRAME_VERSION = "dp41-frame-runtime-1"
GOAL_KINDS = {"DIRECT_RESPONSE", "NEW_TASK", "UPDATE_TASK", "CANCEL_TASK",
              "ANSWER_QUESTION", "QUERY_STATUS", "UNRESOLVED", "DEFER"}
OPERATIONS = {"READ_FIELD", "READ_CALENDAR", "SUMMARIZE", "CREATE_REPORT",
              "CREATE_PROPOSAL", "CREATE_TABLE", "DRAFT_MAIL", "COMPARE", "EXPLAIN",
              "SEND_MAIL", "SET_PROPERTY", "CANCEL_TASK", "ANSWER_QUESTION",
              "RESULT_DELIVERY", "QUESTION_PRESENT", "CALENDAR_CONDITIONAL",
              "REGISTER_MEMORY", "QUERY_STATUS", "UNRESOLVED"}
SELECTORS = {"EXPLICIT_SOURCE", "RECENT_TASK", "RECENT_INTERACTION", "INPUT_SELECTION",
             "SOURCE_AT_REV", "PAST_DELIVERED", "TASK_RESULT", "CALENDAR_WINDOW",
             "CURRENT_QUESTION", "KNOWN_REF"}
RELATIONS = {"USE_RESULT", "AFTER", "INDEPENDENT", "WHEN"}
TOOLS = {"context_retrieve", "task_retrieve", "interaction_retrieve"}
GOAL_KEYS = {"id", "kind", "operation", "target", "target_ref", "arguments", "instruction",
             "source_span_ids"}
MEANING_KEYS = {"version", "goals", "relations", "constraints", "preservations", "response",
                "evidence_refs", "unresolved"}
FRAME_KEYS = MEANING_KEYS | {"references", "input_ref"}


class ContractError(ValueError):
    pass


class BudgetExceeded(RuntimeError):
    pass


def json_text(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), default=str)


def reference_id(item):
    if isinstance(item, str):
        return item
    if not isinstance(item, dict):
        return None
    for key in ("ref", "question_id", "result_id", "source_id", "publication_id", "task_id", "id"):
        if item.get(key) is not None:
            return str(item[key])
    return None


def provided_references(inp):
    result = {}
    metadata = inp.get("metadata", inp.get("initial", {}))
    if isinstance(metadata, dict):
        for key in ("tasks", "sources", "interactions", "questions", "results", "selections"):
            values = metadata.get(key, [])
            if isinstance(values, dict):
                values = list(values.values())
            for item in values:
                if isinstance(item, dict):
                    ref = reference_id(item)
                    if ref:
                        result[ref] = item
                    for question in item.get("questions", []):
                        qref = reference_id(question)
                        if qref:
                            result[qref] = question
    return result


def model_input(inp):
    """An explicit allowlist excludes case IDs, oracles and scheduled future events."""
    allowed = ("transcript", "input_id", "request_id", "revision", "spans", "metadata",
               "current_adopted_meaning", "current", "budgets", "conversation_id")
    return {key: inp[key] for key in allowed if key in inp}


def validate_read(spec):
    if (not isinstance(spec, dict) or not isinstance(spec.get("tool"), str)
            or spec["tool"] not in TOOLS):
        raise ContractError("read.tool must name an allowed bounded read port")
    selector = spec.get("selector", {})
    if (not isinstance(selector, dict) or not isinstance(selector.get("op"), str)
            or selector["op"] not in SELECTORS):
        raise ContractError("unsupported bounded selector")
    if set(spec) - {"tool", "selector", "query", "limit"}:
        raise ContractError("unexpected read envelope fields")
    selector_fields = {
        "EXPLICIT_SOURCE": {"name", "source_kind", "query"},
        "RECENT_TASK": {"query"},
        "RECENT_INTERACTION": {"query", "publication_id", "ref", "source_ref"},
        "INPUT_SELECTION": {"span_ids", "time_ns", "input_id"},
        "SOURCE_AT_REV": {"source_ref", "source_id", "name", "ref", "revision"},
        "PAST_DELIVERED": {"query", "source_ref", "publication_id", "ref", "revision"},
        "TASK_RESULT": {"query", "source_ref", "task_id", "ref", "revision"},
        "CALENDAR_WINDOW": {"date", "start", "end", "source_ref"},
        "CURRENT_QUESTION": {"question_ref", "question_id", "ref"},
        "KNOWN_REF": {"source_ref", "handle", "ref", "id"},
    }
    if set(selector) - ({"op"} | selector_fields[selector["op"]]):
        raise ContractError("unexpected selector fields")
    if not isinstance(spec.get("query", ""), str):
        raise ContractError("read.query must be a string")
    if "query" in selector and not isinstance(selector["query"], str):
        raise ContractError("selector.query must be a string")
    validate_value(spec)
    limit = spec.get("limit", 10)
    if isinstance(limit, bool) or not isinstance(limit, int) or not 1 <= limit <= 10:
        raise ContractError("read.limit must be 1..10")
    return spec


def read_items(result):
    items = result.get("items", result.get("candidates", []))
    if isinstance(items, dict):
        items = list(items.values())
    return [item for item in items if isinstance(item, dict)]


def validate_graph(goals, relations):
    ids = [goal.get("id") for goal in goals]
    if any(not isinstance(i, str) or not i for i in ids) or len(ids) != len(set(ids)):
        raise ContractError("goal IDs must be nonempty and unique")
    edges = {i: [] for i in ids}
    for relation in relations:
        if (not isinstance(relation, dict) or not isinstance(relation.get("op"), str)
                or relation["op"] not in RELATIONS):
            raise ContractError("unsupported relation")
        source, target = relation.get("from_goal"), relation.get("to_goal")
        if (not isinstance(source, str) or not isinstance(target, str)
                or source not in edges or target not in edges or source == target):
            raise ContractError("dangling/self relation")
        if relation["op"] != "INDEPENDENT":
            edges[source].append(target)
        if relation["op"] == "WHEN":
            p = relation.get("predicate", {})
            if not isinstance(p, dict):
                raise ContractError("WHEN predicate must be an object")
            if "producer_goal_id" in p:
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
            if (not isinstance(p, dict) or not isinstance(p.get("field"), str)
                    or not isinstance(p.get("comparison"), str)
                    or p["field"] not in {"STATE", "KIND", "YEAR"}
                    or p["comparison"] not in {"EQ", "NE", "IN"}):
                raise ContractError("unsupported finite WHEN predicate")
    active, done = set(), set()
    def visit(node):
        if node in active:
            raise ContractError("cyclic goal dependencies")
        if node in done:
            return
        active.add(node)
        for child in edges[node]:
            visit(child)
        active.remove(node)
        done.add(node)
    for node in edges:
        visit(node)


def validate_value(value):
    if isinstance(value, dict):
        if any(k in value for k in ("expression", "sql", "code", "eval", "bindings")):
            raise ContractError("executable/arbitrary binding expression is forbidden")
        for child in value.values():
            validate_value(child)
    elif isinstance(value, list):
        for child in value:
            validate_value(child)


def validate_document(document, *, frame=False):
    if not isinstance(document, dict):
        raise ContractError("expected a JSON object")
    expected = FRAME_VERSION if frame else MEANING_VERSION
    if document.get("version") != expected:
        raise ContractError(f"version must be {expected}")
    if set(document) - (FRAME_KEYS if frame else MEANING_KEYS):
        raise ContractError("unexpected document fields")
    goals = document.get("goals")
    if not isinstance(goals, list) or len(goals) > 4:
        raise ContractError("goals must be a list of at most 4 goals")
    for goal in goals:
        if not isinstance(goal, dict) or set(goal) - GOAL_KEYS:
            raise ContractError("unexpected goal fields")
        if (not isinstance(goal.get("kind"), str) or not isinstance(goal.get("operation"), str)
                or goal["kind"] not in GOAL_KINDS or goal["operation"] not in OPERATIONS):
            raise ContractError("unsupported goal kind/operation")
        if not isinstance(goal.get("arguments"), dict) or not isinstance(goal.get("instruction", ""), str):
            raise ContractError("goal arguments/instruction have invalid types")
        operation, args = goal["operation"], goal["arguments"]
        if "source_refs" in args and not isinstance(args["source_refs"], list):
            raise ContractError("source_refs must be a list")
        if operation == "SET_PROPERTY" and (not isinstance(args.get("property"), str)
                or args["property"] not in {"FORMAT", "PERIOD", "LANGUAGE", "AUDIENCE"} or args.get("value") is None):
            raise ContractError("SET_PROPERTY requires finite property and value")
        if operation == "SEND_MAIL" and not ("recipient" in args or "recipient_ref" in args):
            raise ContractError("SEND_MAIL requires explicit recipient")
        if operation == "SEND_MAIL" and not ("body" in args or "body_result_reference" in args or "source_refs" in args):
            raise ContractError("SEND_MAIL requires explicit body/result/source")
        if operation == "ANSWER_QUESTION" and not ("question_ref" in args and "answer" in args):
            raise ContractError("ANSWER_QUESTION requires question_ref and answer")
        if operation == "READ_FIELD" and (not isinstance(args.get("field"), str)
                or args["field"] not in {"DEADLINE", "AMOUNT", "VALUE", "STATUS"}):
            raise ContractError("READ_FIELD requires a supported field")
    for key in ("relations", "constraints", "preservations", "unresolved"):
        if not isinstance(document.get(key, []), list):
            raise ContractError(f"{key} must be a list")
    for item in document.get("unresolved", []):
        if not isinstance(item, dict) or set(item) - {"reason", "field_ref", "source_ref"}:
            raise ContractError("unresolved entries must be objects with reason and optional field_ref/source_ref")
        if not isinstance(item.get("reason"), str) or not item["reason"].strip():
            raise ContractError("unresolved.reason must be a nonempty string")
        for key in ("field_ref", "source_ref"):
            value = item.get(key)
            if value is not None and (not isinstance(value, str) or not value.strip()):
                raise ContractError(f"unresolved.{key} must be a nonempty string or null")
    graph_relations = list(document.get("relations", []))
    for goal in goals:
        body = goal["arguments"].get("body", goal["arguments"].get("body_result_reference"))
        if isinstance(body, dict) and ("result_of" in body or body.get("kind") == "GOAL_RESULT"):
            producer = body.get("producer_goal_id", body.get("result_of"))
            graph_relations.append({"op": "USE_RESULT", "from_goal": producer, "to_goal": goal["id"]})
    validate_graph(goals, graph_relations)
    for constraint in document.get("constraints", []):
        if (not isinstance(constraint, dict) or not isinstance(constraint.get("op"), str)
                or constraint["op"] not in {"FORBID_ACTION", "KEEP_PROPERTY"}):
            raise ContractError("unsupported constraint")
        if (constraint["op"] == "FORBID_ACTION" and (not isinstance(constraint.get("action"), str)
                or constraint["action"] not in {"SEND", "DELETE", "OVERWRITE"})):
            raise ContractError("unsupported forbidden action")
    for preservation in document.get("preservations", []):
        if not isinstance(preservation, dict):
            raise ContractError("preservations entries must be objects")
    validate_value(document)
    if frame:
        refs = document.get("references", [])
        if not isinstance(refs, list) or len(refs) > 8:
            raise ContractError("references must be a list of at most 8")
        ids = [ref.get("id") for ref in refs if isinstance(ref, dict)]
        if (len(ids) != len(refs) or any(not isinstance(i, str) or not i for i in ids)
                or len(ids) != len(set(ids))):
            raise ContractError("reference IDs must be unique")
        for ref in refs:
            if set(ref) - {"id", "selector", "tool", "query", "limit", "cardinality", "mention", "source_span_ids"}:
                raise ContractError("unexpected reference fields")
            cardinality = ref.get("cardinality", "ONE")
            if not isinstance(cardinality, str) or cardinality not in {"ONE", "MANY"}:
                raise ContractError("reference cardinality must be ONE or MANY")
            validate_read(reference_spec(ref))
        if set(ids) & {goal["id"] for goal in goals}:
            raise ContractError("goal IDs and reference IDs must be distinct")
        reference_ids = set(ids)
        for goal in goals:
            target = goal.get("target_ref", goal.get("target"))
            if target is not None and not isinstance(target, str):
                raise ContractError("frame target must be a reference ID string or null")
            if target is not None and target not in reference_ids:
                raise ContractError("frame target must be an unbound reference ID")
            args = goal["arguments"]
            for field in ("recipient_ref", "question_ref"):
                value = args.get(field)
                if value is not None and not isinstance(value, str):
                    raise ContractError(f"frame {field} must be a reference ID string or null")
                if value is not None and value not in reference_ids:
                    raise ContractError(f"frame {field} must name a frame reference")
            for value in args.get("source_refs", []):
                identifier = value.get("ref") if isinstance(value, dict) else value
                if not isinstance(identifier, str) or identifier not in reference_ids:
                    raise ContractError("frame source_refs must name declared frame references")
            body = args.get("body", args.get("body_result_reference"))
            if isinstance(body, dict) and body.get("source_ref") not in (None, *reference_ids):
                raise ContractError("frame body.source_ref must name a declared frame reference")
        for preservation in document.get("preservations", []):
            if (not isinstance(preservation.get("reference_id"), str)
                    or preservation["reference_id"] not in reference_ids or preservation.get("mode") != "NO_NEW_COMMAND"):
                raise ContractError("preservation requires declared reference and NO_NEW_COMMAND")
        for relation in document.get("relations", []):
            if (relation["op"] == "WHEN" and "reference_id" in relation["predicate"]
                    and not isinstance(relation["predicate"]["reference_id"], (str, dict))):
                raise ContractError("WHEN reference_id must be a reference ID string")
            if (relation["op"] == "WHEN" and not isinstance(relation["predicate"].get("reference_id"), dict)
                    and relation["predicate"].get("reference_id") not in reference_ids):
                raise ContractError("WHEN requires declared frame reference")
    return document


def reference_spec(ref):
    if not isinstance(ref, dict):
        raise ContractError("reference must be an object")
    selector = ref.get("selector")
    if (not isinstance(selector, dict) or not isinstance(selector.get("op"), str)
            or selector["op"] not in SELECTORS):
        raise ContractError("unsupported bounded selector")
    op = selector["op"]
    if "query" in selector and not isinstance(selector["query"], str):
        raise ContractError("selector.query must be a string")
    tool = ref.get("tool") or ("task_retrieve" if op in {"RECENT_TASK", "TASK_RESULT"} else
                              "interaction_retrieve" if op in {"RECENT_INTERACTION", "PAST_DELIVERED", "CURRENT_QUESTION"} else
                              "context_retrieve")
    query = ref["query"] if "query" in ref else selector.get("query", ref.get("mention", ""))
    return {"tool": tool, "selector": ref["selector"], "query": query,
            "limit": ref.get("limit", 10)}


async def emit(ports, kind, payload):
    value = ports.emit(kind, payload)
    if inspect.isawaitable(value):
        await value


@dataclass
class Budget:
    max_models: int = 8
    max_reads: int = 16
    models: int = 0
    reads: int = 0

    @classmethod
    def from_input(cls, inp):
        b = inp.get("budgets", {})
        return cls(min(8, int(b.get("model_calls", b.get("max_model_calls", 8)))),
                   min(16, int(b.get("read_calls", b.get("max_read_calls", 16)))))

    async def model(self, ports, messages, *, purpose, role="CINTERPRET"):
        if self.models >= self.max_models:
            raise BudgetExceeded("logical Request model-call budget exhausted")
        self.models += 1
        await emit(ports, "dp41_model_requested", {"purpose": purpose, "ordinal": self.models})
        result = await ports.model(messages, role=role)
        if not isinstance(result, dict):
            raise ContractError("model did not return a JSON object")
        return result

    async def read(self, ports, spec):
        validate_read(spec)
        if self.reads >= self.max_reads:
            raise BudgetExceeded("logical Request read budget exhausted")
        self.reads += 1
        ordinal = self.reads
        await emit(ports, "dp41_read_requested", {"ordinal": ordinal, "spec": spec})
        result = await ports.read(spec)
        if not isinstance(result, dict):
            raise ContractError("read did not return an object")
        await emit(ports, "dp41_read_returned", {"ordinal": ordinal, "spec": spec, "result": result})
        return result


def failed_meaning(reason, *, evidence_refs=()):
    return {"version": MEANING_VERSION, "goals": [], "relations": [], "constraints": [],
            "preservations": [], "response": None, "evidence_refs": list(evidence_refs),
            "unresolved": [{"reason": reason}]}
