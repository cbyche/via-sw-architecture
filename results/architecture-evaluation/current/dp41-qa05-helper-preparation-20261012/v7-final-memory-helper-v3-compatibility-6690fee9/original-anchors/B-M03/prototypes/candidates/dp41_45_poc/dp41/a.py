"""Model-led bounded interpretation loop (DP41 A)."""
import asyncio

from .contract import (Budget, BudgetExceeded, ContractError, emit, failed_meaning,
                       json_text, model_input, validate_document, validate_read)
from .prompts import A_SYSTEM


async def resolve(interpretation_input: dict, ports) -> dict:
    budget = Budget.from_input(interpretation_input)
    messages = [{"role": "system", "content": A_SYSTEM},
                {"role": "user", "content": json_text(model_input(interpretation_input))}]
    evidence = []
    clarification_count = 0
    try:
        while True:
            reply = await budget.model(ports, messages, purpose="A_NEXT_ACTION")
            await emit(ports, "dp41_a_model_action", {"output": reply})
            messages.append({"role": "assistant", "content": json_text(reply)})
            try:
                action = reply.get("action", reply.get("kind"))
                if action == "READ":
                    reads = reply.get("reads", [])
                    if not isinstance(reads, list) or not reads or len(reads) > 8:
                        raise ContractError("READ needs 1..8 bounded read specs")
                    for spec in reads:
                        validate_read(spec)
                    # The model declared these independent reads; no strategy is added here.
                    results = await asyncio.gather(*(budget.read(ports, spec) for spec in reads))
                    evidence.extend(r.get("read_id") for r in results if r.get("read_id"))
                    messages.append({"role": "user", "content": json_text({"bounded_read_results": results})})
                elif action == "CLARIFY":
                    question = reply.get("question")
                    if not isinstance(question, str) or not question.strip():
                        raise ContractError("clarification question required")
                    if clarification_count >= 2:
                        raise BudgetExceeded("additional clarification budget exhausted")
                    clarification_count += 1
                    await emit(ports, "dp41_clarification_requested", {"question": question, "scope": reply.get("scope")})
                    answer = await ports.clarify(question, reply.get("scope"))
                    await emit(ports, "dp41_clarification_returned", {"answer": answer})
                    if not answer or answer.get("no_answer"):
                        return failed_meaning("NO_CLARIFICATION_ANSWER", evidence_refs=evidence)
                    messages.append({"role": "user", "content": json_text({"clarification_answer": answer})})
                elif action == "PROPOSE":
                    meaning = validate_document(reply.get("meaning"))
                    await emit(ports, "dp41_meaning_proposed", {"candidate": "A", "meaning": meaning})
                    return meaning
                else:
                    raise ContractError("allowed action is READ, CLARIFY or PROPOSE")
            except ContractError as error:
                await emit(ports, "dp41_output_rejected", {"candidate": "A", "reason": str(error), "output": reply})
                messages.append({"role": "user", "content": json_text({"validation_error": str(error),
                                  "instruction": "Repair your own output within the same bounded contract."})})
    except asyncio.CancelledError:
        await emit(ports, "dp41_resolution_cancelled", {"candidate": "A"})
        raise
    except Exception as error:
        await emit(ports, "dp41_resolution_failed", {"candidate": "A", "type": type(error).__name__, "reason": str(error)})
        return failed_meaning(f"{type(error).__name__}: {error}", evidence_refs=evidence)
