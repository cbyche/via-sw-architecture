"""Finite frame production, reference resolution and code binding (DP41 B)."""
import asyncio
import copy

from .contract import (Budget, ContractError, FRAME_VERSION, MEANING_VERSION, emit,
                       failed_meaning, json_text, model_input, provided_references,
                       read_items, reference_id, reference_spec, validate_document)
from .prompts import B_SYSTEM, SELECT_SYSTEM


def _question_valid(item):
    record = {**item.get("metadata", {}), **item} if isinstance(item.get("metadata"), dict) else item
    if not ("question_id" in record or record.get("kind") == "QUESTION" or
            ("question" in record and "status" in record)):
        return True
    status = str(record.get("status", record.get("state", ""))).upper()
    presented = record.get("presented", record.get("delivered", record.get("delivered_receipt", record.get("publication_receipt"))))
    return status not in {"WITHDRAWN", "ANSWERED", "CANCELLED", "STALE"} and bool(presented)


def _replace_refs(value, bindings):
    if isinstance(value, str):
        return bindings.get(value, value)
    if isinstance(value, list):
        return [_replace_refs(item, bindings) for item in value]
    if isinstance(value, dict):
        if set(value) == {"ref"}:
            ref = value["ref"]
            if ref not in bindings:
                raise ContractError(f"unbound reference {ref}")
            return bindings[ref]
        return {key: _replace_refs(item, bindings) for key, item in value.items()}
    return value


def _symbolic_results(value, inp, goal_ids):
    if isinstance(value, dict):
        if set(value) == {"result_of"}:
            producer = value["result_of"]
            if producer not in goal_ids:
                raise ContractError("unknown current goal-result producer")
            return {"kind": "GOAL_RESULT", "producer_goal_id": producer,
                    "request_id": inp.get("request_id"), "producer_revision": inp.get("revision", 1)}
        if value.get("kind") == "GOAL_RESULT":
            producer = value.get("producer_goal_id")
            if producer not in goal_ids:
                raise ContractError("unknown symbolic producer")
            return {"kind": "GOAL_RESULT", "producer_goal_id": producer,
                    "request_id": inp.get("request_id"), "producer_revision": inp.get("revision", 1)}
        return {k: _symbolic_results(v, inp, goal_ids) for k, v in value.items()}
    if isinstance(value, list):
        return [_symbolic_results(v, inp, goal_ids) for v in value]
    return value


def _clarification_text(frame, unresolved, states):
    questions = []
    for state in states.values():
        for item in state.get("candidates", []):
            record = {**item.get("metadata", {}), **item} if isinstance(item.get("metadata"), dict) else item
            text = record.get("question", record.get("text", ""))
            if text and ("question_id" in record or record.get("kind") == "QUESTION"):
                questions.append(text)
    if questions:
        return "이전 질문의 현재 대상과 동의 범위를 다시 확인할게요. " + questions[-1]
    candidates = []
    for state in states.values():
        for item in state.get("candidates", []):
            label = item.get("title", item.get("instruction", item.get("name", reference_id(item))))
            if label:
                candidates.append(str(label))
    if candidates:
        return "다음 후보 중 어떤 대상이고 무엇을 변경할까요? " + "; ".join(candidates[:8])
    return "요청의 대상과 변경 내용 또는 필요한 동의 범위를 구체적으로 알려 주세요."


def _build_meaning(frame, bindings, inp, evidence):
    ids = {goal["id"] for goal in frame["goals"]}
    goals = []
    for raw in frame["goals"]:
        goal = copy.deepcopy(raw)
        target_ref = goal.pop("target_ref", goal.get("target"))
        if target_ref is not None and target_ref not in bindings:
            raise ContractError(f"unresolved target reference {target_ref}")
        goal["target"] = bindings.get(target_ref) if target_ref is not None else None
        goal.pop("source_span_ids", None)
        args = copy.deepcopy(goal["arguments"])
        # Only declared reference positions are executable binding sites. Literal
        # domain values that happen to equal a frame ID remain literal values.
        for field in ("source_refs", "recipient_ref", "question_ref"):
            if field in args:
                args[field] = _replace_refs(args[field], bindings)
        for field in ("body", "body_result_reference"):
            body = args.get(field)
            if isinstance(body, dict) and "source_ref" in body:
                body["source_ref"] = _replace_refs(body["source_ref"], bindings)
        goal["arguments"] = _symbolic_results(args, inp, ids)
        source_refs = goal["arguments"].get("source_refs")
        if isinstance(source_refs, list):
            goal["arguments"]["source_refs"] = [v for item in source_refs for v in (item if isinstance(item, list) else [item])]
        goals.append(goal)
    relations = copy.deepcopy(frame.get("relations", []))
    for relation in relations:
        if relation.get("op") == "WHEN":
            relation["predicate"]["reference_id"] = _replace_refs(relation["predicate"]["reference_id"], bindings)
    preservations = copy.deepcopy(frame.get("preservations", []))
    for preservation in preservations:
        preservation["reference_id"] = _replace_refs(preservation["reference_id"], bindings)
    result = {"version": MEANING_VERSION, "goals": goals,
              "relations": relations,
              "constraints": copy.deepcopy(frame.get("constraints", [])),
              "preservations": preservations,
              "response": frame.get("response"), "evidence_refs": list(dict.fromkeys(evidence)),
              "unresolved": copy.deepcopy(frame.get("unresolved", []))}
    return validate_document(result)


async def resolve(interpretation_input: dict, ports) -> dict:
    """The model produces frames/one-field selections; code produces final bindings."""
    inp = copy.deepcopy(interpretation_input)
    budget = Budget.from_input(inp)
    supplied = provided_references(inp)
    evidence = []
    messages = [{"role": "system", "content": B_SYSTEM},
                {"role": "user", "content": json_text(model_input(inp))}]
    clarifications = 0
    try:
        while True:
            frame = await budget.model(ports, messages, purpose="B_FRAME")
            await emit(ports, "dp41_b_frame_produced", {"frame": frame})
            try:
                validate_document(frame, frame=True)
            except ContractError as error:
                await emit(ports, "dp41_output_rejected", {"candidate": "B", "reason": str(error), "output": frame})
                messages.extend([{"role": "assistant", "content": json_text(frame)},
                                 {"role": "user", "content": json_text({"validation_error": str(error),
                                  "instruction": "Regenerate the unbound finite frame, not final actual bindings."})}])
                continue
            if frame.get("input_ref") not in (None, inp.get("input_id")):
                raise ContractError("frame input_ref differs from current input")
            bindings, states, unresolved = {}, {}, []

            async def fetch(ref):
                fid = ref["id"]
                selector = ref["selector"]
                await emit(ports, "dp41_field_state", {"field_id": fid, "state": "UNRESOLVED"})
                handle = selector.get("source_ref", selector.get("handle", selector.get("ref", selector.get("id"))))
                if selector["op"] == "KNOWN_REF" and handle in supplied:
                    item = supplied[handle]
                    # Source metadata lacks content and must still use the actual read port.
                    if (("task_id" in item or "question_id" in item or "publication_id" in item)
                            and not ("question_id" in item and clarifications)):
                        return ref, {"status": "OK", "items": [item], "read_id": "HOST_METADATA"}
                await emit(ports, "dp41_field_state", {"field_id": fid, "state": "READ_PENDING"})
                return ref, await budget.read(ports, reference_spec(ref))

            fetched = await asyncio.gather(*(fetch(ref) for ref in frame.get("references", [])))
            for ref, result in fetched:
                fid = ref["id"]
                if result.get("read_id"):
                    evidence.append(result["read_id"])
                candidates = read_items(result)
                state = {"status": result.get("status", "UNAVAILABLE"), "candidates": candidates,
                         "read_id": result.get("read_id"), "selector": ref["selector"]}
                states[fid] = state
                if state["status"] in {"DENIED", "ACCESS_DENIED"}:
                    state["status"] = "DENIED"
                    handle = ref["selector"].get("source_ref", ref["selector"].get("name"))
                    denial_ref = reference_id(candidates[0]) if len(candidates) == 1 else handle
                    if denial_ref:
                        bindings[fid] = denial_ref
                    unresolved.append({"field_ref": fid, "reason": "DENIED", "source_ref": denial_ref})
                elif state["status"] not in {"OK", "FOUND", "PARTIAL"}:
                    unresolved.append({"field_ref": fid, "reason": state["status"]})
                else:
                    valid_candidates = [c for c in candidates if reference_id(c) and _question_valid(c)]
                    state["valid_candidates"] = valid_candidates
                    if ref.get("cardinality", "ONE") == "MANY" and valid_candidates:
                        bindings[fid] = [reference_id(c) for c in valid_candidates]
                    elif len(valid_candidates) == 1:
                        bindings[fid] = reference_id(valid_candidates[0])
                    elif len(valid_candidates) > 1:
                        await emit(ports, "dp41_field_state", {"field_id": fid, "state": "PARTIAL_PENDING"})
                        partial_input = {"field_ref": fid, "transcript": inp.get("transcript"),
                                         "candidates": valid_candidates, "evidence_refs": [state["read_id"]]}
                        selection = await budget.model(ports,
                            [{"role": "system", "content": SELECT_SYSTEM},
                             {"role": "user", "content": json_text(partial_input)}], purpose="B_PARTIAL_SELECT")
                        await emit(ports, "dp41_b_partial_returned", {"field_ref": fid, "output": selection})
                        allowed = {"field_ref", "candidate_ref", "evidence_refs", "uncertain"}
                        refs = {reference_id(c) for c in valid_candidates}
                        if (set(selection) <= allowed and selection.get("field_ref") == fid and
                            selection.get("candidate_ref") in refs and selection.get("uncertain") is False and
                            set(selection.get("evidence_refs", [])) <= {state["read_id"]}):
                            bindings[fid] = selection["candidate_ref"]
                        else:
                            state["status"] = "AMBIGUOUS"
                            unresolved.append({"field_ref": fid, "reason": "AMBIGUOUS_OR_INVALID_PARTIAL"})
                    else:
                        state["status"] = "NO_VALID_CANDIDATE"
                        unresolved.append({"field_ref": fid, "reason": "NO_VALID_CANDIDATE"})
                await emit(ports, "dp41_field_state", {"field_id": fid,
                           "state": "BOUND" if fid in bindings and state["status"] != "DENIED" else state["status"],
                           "binding": bindings.get(fid), "read_id": state["read_id"]})

            unresolved.extend(frame.get("unresolved", []))
            if any(g["operation"] == "UNRESOLVED" or g["kind"] == "UNRESOLVED" for g in frame["goals"]):
                unresolved.append({"reason": "MISSING_GOAL_OPERATION_OR_VALUE"})
            recoverable = [u for u in unresolved if u.get("reason") not in {"DENIED", "UNAVAILABLE", "ACCESS_DENIED"}]
            if recoverable and clarifications < 2:
                question = _clarification_text(frame, recoverable, states)
                clarifications += 1
                scope = {"field_refs": [u.get("field_ref") for u in recoverable],
                         "candidate_refs": {k: [reference_id(c) for c in s["candidates"]] for k, s in states.items()}}
                question_refs = []
                for state in states.values():
                    for item in state["candidates"]:
                        record = {**item.get("metadata", {}), **item} if isinstance(item.get("metadata"), dict) else item
                        if record.get("question_id") and record.get("status") == "VALID":
                            question_refs.append({"question_ref": record["question_id"], "revision": record.get("revision", 1)})
                if question_refs:
                    scope["question_refs"] = question_refs
                await emit(ports, "dp41_clarification_requested", {"question": question, "scope": scope})
                answer = await ports.clarify(question, scope)
                await emit(ports, "dp41_clarification_returned", {"answer": answer})
                if not answer or answer.get("no_answer"):
                    return failed_meaning("NO_CLARIFICATION_ANSWER", evidence_refs=evidence)
                # The new model output remains a finite unbound frame. The code still binds it.
                messages.extend([{"role": "assistant", "content": json_text(frame)},
                                 {"role": "user", "content": json_text({"original_request": inp.get("transcript"),
                                  "actual_clarification_answer": answer, "previous_frame": frame,
                                  "read_evidence": states,
                                  "instruction": "Produce an updated unbound frame. Preserve original conditions and explicit exclusions."})}])
                inp["transcript"] = str(inp.get("transcript", "")) + "\n" + str(answer.get("transcript", ""))
                continue
            if recoverable:
                await emit(ports, "dp41_resolution_failed", {"candidate": "B", "unresolved": unresolved})
                return failed_meaning("UNRESOLVED_FIELDS", evidence_refs=evidence)
            meaning = _build_meaning(frame, bindings, inp, evidence)
            meaning["unresolved"] = unresolved
            await emit(ports, "dp41_meaning_proposed", {"candidate": "B", "meaning": meaning})
            return meaning
    except asyncio.CancelledError:
        await emit(ports, "dp41_resolution_cancelled", {"candidate": "B"})
        raise
    except Exception as error:
        await emit(ports, "dp41_resolution_failed", {"candidate": "B", "type": type(error).__name__, "reason": str(error)})
        return failed_meaning(f"{type(error).__name__}: {error}", evidence_refs=evidence)
