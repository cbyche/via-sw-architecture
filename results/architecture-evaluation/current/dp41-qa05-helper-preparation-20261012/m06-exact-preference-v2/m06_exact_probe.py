"""Preserved M06/S1-03 oracle probe against an explicit isolated candidate.

Scripted native A actions or B frames verify functionality, not LLM accuracy.
The evaluator observes the candidate's actual owner read; it never projects
private memory into that port or repairs a missing write/read integration.
"""
from __future__ import annotations
import argparse
import asyncio
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import traceback

INPUT = "앞으로 비교 결과는 표로 보여주는 걸 선호한다고 기억해줘."
RESPONSE = "비교 결과는 표로 보여주는 선호를 기억했다."
USER = "current-user"
KEY = "comparison-result-format"
VALUE = {"user_id": USER, "applies_to": "COMPARISON_RESULT", "format": "TABLE",
         "scope": "FUTURE_COMPARISON_RESULTS"}


def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def hashes(source):
    return {str(p.relative_to(source)): sha(p) for p in source.rglob("*")
            if p.is_file() and "__pycache__" not in p.parts}
def save(path, value):
    path = Path(path)
    with path.open("x", encoding="utf-8") as out:
        out.write(json.dumps(value, ensure_ascii=False, indent=2, default=str) + "\n")


async def phase(source, variant):
    source = Path(source).resolve()
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(source))
    from prototypes.candidates.dp41_45_poc.shared import runtime as rm, owners
    from prototypes.candidates.dp41_45_poc.dp41 import contract
    from prototypes.candidates.dp41_45_poc.dp41 import a, b
    for module in (rm, owners, contract, a, b):
        assert Path(module.__file__).resolve().is_relative_to(source)

    class Evidence:
        def __init__(self): self.origin, self.rows = time.monotonic(), []
        def seconds(self): return time.monotonic() - self.origin
        def write(self, table, row):
            self.rows.append({"table": table, **copy.deepcopy(row)})
            return row

    def goal(key, value, refs=None):
        return {"id": "register-" + key, "kind": "DIRECT_RESPONSE", "operation": "REGISTER_MEMORY",
                "target": None, "arguments": {"key": key, "value": copy.deepcopy(value),
                                              "source_refs": refs or []},
                "instruction": "현재 사용자의 명시적 지속 선호를 VIA User Memory에 등록"}

    def native(g, response=None):
        return {"version": contract.MEANING_VERSION, "goals": [g], "relations": [],
                "constraints": [], "preservations": [], "response": response,
                "evidence_refs": [], "unresolved": []}

    class Model:
        def __init__(self): self.calls, self.document = [], None
        async def call(self, messages, role, **kwargs):
            self.calls.append({"role": role, "messages": copy.deepcopy(messages),
                               "request_id": kwargs.get("request_id")})
            if role == "CRESPONSE":
                facts = json.loads(messages[1]["content"])
                def contains_value(item):
                    if isinstance(item, dict):
                        return item == VALUE or any(contains_value(child) for child in item.values())
                    if isinstance(item, list): return any(contains_value(child) for child in item)
                    return False
                if kwargs.get("request_id") == "preference-request":
                    self.receipt_facts = copy.deepcopy(facts)
                    return {"text": RESPONSE if facts.get("registered") is True and contains_value(facts)
                            else "등록된 비교 형식 선호를 확인할 수 없다."}
                return {"text": "제공된 실제 처리 상태를 확인했다."}
            if role != "CINTERPRET":
                raise AssertionError("unexpected scripted semantic role")
            result = copy.deepcopy(self.document)
            if variant == "A": return {"action": "PROPOSE", "meaning": result}
            result["version"] = contract.FRAME_VERSION
            result["references"] = []
            for goal in result["goals"]:
                arguments = goal["arguments"]
                source_fields = []
                for index, actual in enumerate(arguments.get("source_refs", [])):
                    field = "source-field-" + str(index)
                    result["references"].append({"id": field, "selector": {"op": "KNOWN_REF", "source_ref": actual},
                                                 "tool": "context_retrieve", "cardinality": "ONE"})
                    source_fields.append(field)
                arguments["source_refs"] = source_fields
            result["input_ref"] = json.loads(messages[1]["content"])["input_id"]
            return result

    language = {"source_id": "M언어", "revision": 1, "kind": "USER_MEMORY",
                "title": "설명 언어", "text": "설명 언어=한국어",
                "metadata": {"user_id": USER, "scope": "EXPLANATION_LANGUAGE", "value": "한국어"},
                "permission": {"read": True, "model": True, "agent": True}}
    private = {"source_id": "private-document", "revision": 1, "kind": "DOCUMENT",
               "title": "철회 대상", "text": "철회 대상 원문.", "metadata": {},
               "permission": {"read": True, "model": True, "agent": True}}
    other = {"source_id": "permitted-document", "revision": 1, "kind": "DOCUMENT",
             "title": "다른 허용 자료", "text": "계속 사용할 수 있는 원문.", "metadata": {},
             "permission": {"read": True, "model": True, "agent": True}}
    unrelated = {"source_id": "existing-short-preference", "revision": 1, "kind": "USER_MEMORY",
                 "title": "설명 길이", "text": "짧은 설명 선호",
                 "metadata": {"user_id": USER, "scope": "EXPLANATION_LENGTH", "value": "SHORT"},
                 "permission": {"read": True, "model": True, "agent": True}}
    foreign = {"source_id": "foreign-preference", "revision": 1, "kind": "USER_MEMORY",
               "title": "다른 사용자의 선호", "text": "other-user-sensitive-text",
               "metadata": {"user_id": "other-user", "scope": "EXPLANATION_LENGTH", "value": "LONG"},
               "permission": {"read": True, "model": True, "agent": True}}
    initial = {"sources": [language, unrelated, foreign, private, other], "tasks": [], "voice": {"on": False},
               "current_user_id": USER, "policy": {"user_memory_registration_allowed": True}}
    evidence, model = Evidence(), Model()
    runtime = rm.LocalVIA(initial, model, evidence, variant,
        voice_profile={"output_onset_seconds": .001, "characters_per_second": 100000., "stop_seconds": .001})
    checks, errors, raws = {}, [], {}
    def check(name, passed, details=None):
        checks[name] = {"passed": bool(passed), "details": details}
    try:
        language_before = copy.deepcopy(runtime.owners.lookup("M언어@v1"))
        unrelated_before = copy.deepcopy(runtime.owners.lookup("existing-short-preference@v1"))
        model.document = native(goal(KEY, VALUE), None)
        await runtime.input_started("preference-input", advance_generation=False)
        # A real simulated speech interval is retained. No QA02/03 timing score
        # is computed by this supplemental functional probe.
        await asyncio.sleep(3.)
        await runtime.speech_end("preference-input")
        result = await runtime.resolve(INPUT, request_id="preference-request", input_id="preference-input")
        while runtime.jobs:
            await asyncio.wait_for(asyncio.gather(*list(runtime.jobs), return_exceptions=True), 2.)
            await asyncio.sleep(0)
        raws["resolution"] = result
        adopted = runtime.requests.get("preference-request", {}).get("meaning", {})
        g = adopted.get("goals", [{}])[0] if len(adopted.get("goals", [])) == 1 else {}
        args = g.get("arguments", {})
        value = args.get("value")
        own = runtime.memory_records.get(KEY)
        reads = rm.Ports(runtime, "memory-query-request", "memory-query-input", "현재 사용자의 등록된 비교 형식 조회")
        read = await reads.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": KEY}})
        raws["common_memory_read"] = read
        actual_items = [item.get("metadata", item) for item in read.get("items", [])]
        found_values = [item.get("value", item.get("content", item.get("text"))) for item in actual_items]
        # The exact owner registration format is this fixture's native JSON
        # value. The common port must return it; private dict inspection alone
        # cannot satisfy canonical C5.
        common_observable = any(item == VALUE for item in found_values) or any(
            item.get("metadata", {}).get("value") == VALUE or item.get("value") == VALUE
            for item in read.get("items", []))
        actual_owner_scope = any(item.get("metadata", {}).get("user_id") == USER
            and item.get("metadata", {}).get("scope") == VALUE["scope"]
            and item.get("metadata", {}).get("persistent") is True for item in read.get("items", []))
        registrations = [e for e in evidence.rows if e.get("kind") == "MEMORY_REGISTERED" and e.get("key") == KEY]
        deliveries = [d for d in runtime.deliveries if d.get("request_id") == "preference-request"]
        delivered_texts = [d.get("text", "") for d in deliveries if d.get("delivered_receipt") and d.get("text_presented_s") is not None]
        language_now = runtime.owners.lookup("M언어@v1")
        check("C1", result.get("status") == "ADOPTED" and g.get("operation") == "REGISTER_MEMORY"
              and g.get("kind") == "DIRECT_RESPONSE", {"adopted_goal": g})
        check("C2", value == VALUE and args.get("source_refs") == []
              and all(USER == v.get("user_id") for v in [value] if isinstance(v, dict)), {"transcript": INPUT, "value": value})
        check("C3", g.get("target") is None and not runtime.owners.tasks and not runtime.effects,
              {"tasks": runtime.owners.tasks, "external_effect_count": len(runtime.effects)})
        check("C4", value == VALUE and language_now == language_before,
              {"stored_scope": value, "Mlanguage_before": language_before, "Mlanguage_after": language_now})
        check("C5", common_observable and actual_owner_scope and own is not None and own.get("value") == VALUE
              and len(registrations) == 1 and not runtime.effects and language_now == language_before
              and runtime.owners.lookup("existing-short-preference@v1") == unrelated_before,
              {"common_read": read, "private_owner_record": own, "registration_count": len(registrations),
               "actual_owner_user_and_persistent_scope": actual_owner_scope,
               "private_storage_is_not_common_read_evidence": True})
        check("C6", RESPONSE in delivered_texts and all("모든 답변" not in text and "다른 사용자" not in text for text in delivered_texts),
              {"actual_delivered_texts": delivered_texts, "actual_response_facts": getattr(model, "receipt_facts", None),
               "candidate_explicit_response": adopted.get("response")})
        foreign_read = await reads.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "foreign-preference@v1"}})
        language_read = await reads.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "M언어@v1"}})
        check("current_user_partition_and_language_common_read", not foreign_read.get("items")
              and bool(language_read.get("items")) and all("other-user-sensitive-text" not in str(c["messages"]) for c in model.calls),
              {"foreign_read": foreign_read, "language_read": language_read})
        refs = [item.get("ref") for item in read.get("items", [])]
        check("registered_owner_ref_and_revision", common_observable and bool(refs) and
              all(isinstance(ref, str) and ref for ref in refs) and
              all(isinstance(item.get("metadata", {}).get("revision", item.get("revision", 1)), int)
                  for item in read.get("items", [])), {"actual_owner_items": read.get("items", [])})
        raws["registration_boundary"] = {"private_memory_after": copy.deepcopy(runtime.memory_records),
            "language_after": copy.deepcopy(language_now), "deliveries": copy.deepcopy(runtime.deliveries)}
        # Candidate-generated derived registration uses actual allowed source
        # provenance. The evaluator does not fill an expected derived record.
        model.document = native(goal("derived-private", {"relation": "derived relation"}, ["private-document@v1"]))
        derived = await runtime.resolve("이 제공 문서의 관계 정보를 기억해줘.",
            request_id="derived-request", input_id="derived-input")
        raws["derived_resolution"] = derived
        before_revoke = copy.deepcopy(runtime.memory_records)
        await runtime.external_event({"event_id": "actual-revoke", "type": "PERMISSION_REVOKED",
                                      "payload": {"source_id": "private-document"}})
        revoked = await reads.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "private-document@v1"}})
        allowed = await reads.read({"tool": "context_retrieve", "selector": {"op": "KNOWN_REF", "source_ref": "permitted-document@v1"}})
        check("revocation_and_derived_deletion", derived.get("status") == "ADOPTED"
              and before_revoke.get("derived-private", {}).get("value") is not None
              and runtime.memory_records.get("derived-private", {}).get("value") is None
              and runtime.memory_records.get("derived-private", {}).get("invalidated") == "POLICY_REVOCATION"
              and not revoked.get("items"), {"derived_before": before_revoke.get("derived-private"),
                                          "derived_after": runtime.memory_records.get("derived-private"), "revoked_read": revoked})
        receipt = [e for e in evidence.rows if e.get("kind") == "REVOCATION_FENCE_APPLIED"]
        check("honest_receipt_and_unrelated_preference_retained", bool(receipt) and
              receipt[-1].get("receipt") == "LOCAL_CONSUMERS_FENCED_REMOTE_SENT_COPIES_NOT_RETRACTED" and
              runtime.memory_records.get(KEY, {}).get("value") == VALUE and
              runtime.owners.lookup("M언어@v1") == language_before and bool(allowed.get("items")),
              {"receipt": receipt, "allowed_read": allowed})
    except Exception as error:
        errors.append({"type": type(error).__name__, "detail": str(error), "traceback": traceback.format_exc()})
    finally:
        for job in list(runtime.jobs): job.cancel()
        await asyncio.gather(*list(runtime.jobs), return_exceptions=True)
    return {"status": "PASSED" if not errors and len(checks) == 10 and all(c["passed"] for c in checks.values()) else "FAILED",
        "variant": variant, "case_id": "S1-03-M06-PRESERVATION", "input": INPUT,
        "initial": initial, "checks": checks, "errors": errors, "raws": raws,
        "events": evidence.rows, "effects": runtime.effects, "memory_records": runtime.memory_records,
        "owner_sources": runtime.owners.sources, "deliveries": runtime.deliveries, "model_calls": model.calls,
        "imported_modules": {m.__name__: str(Path(m.__file__).resolve()) for m in (rm, owners, contract, a, b)},
        "api_calls": 0, "evidence_level": "SCRIPTED_EXACT_ORACLE_FUNCTIONAL_REGRESSION",
        "actual_model_semantic_accuracy_claim": False, "P21_accuracy_denominator_addition": 0,
        "responsibility_count": None}


def run_supplement(source, variant, output):
    source, output = Path(source).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = hashes(source)
    try:
        process = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--source", str(source),
            "--variant", variant, "--output", str(output), "--phase"], capture_output=True, text=True,
            timeout=20, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}, cwd=source)
        try: raw = json.loads(process.stdout)
        except json.JSONDecodeError: raw = {"status": "FAILED", "error": "INVALID_PROBE_STDOUT"}
        save(output / "raw.json", {"result": raw, "returncode": process.returncode,
             "stdout": process.stdout, "stderr": process.stderr})
    except subprocess.TimeoutExpired as error:
        raw = {"status": "FAILED", "error": "PROCESS_WATCHDOG_TIMEOUT"}
        save(output / "raw.json", {"result": raw, "stdout": str(error.stdout), "stderr": str(error.stderr)})
    after = hashes(source)
    result = {"status": "PASSED" if raw.get("status") == "PASSED" and before == after else "FAILED",
              "source_unchanged": before == after, "source": str(source), "variant": variant,
              "source_manifest": before, "source_digest": hashlib.sha256(json.dumps(before, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
              "helper_sha256": sha(__file__), "raw_ref": str(output / "raw.json"), "checks": raw.get("checks", {}),
              "api_calls": 0, "responsibility_review": "AWAITING_INDEPENDENT_REVIEW", "responsibility_count": None,
              "evidence_level": "SCRIPTED_EXACT_ORACLE_FUNCTIONAL_REGRESSION"}
    save(output / "result.json", result)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True)
    parser.add_argument("--variant", required=True, choices=["A", "B"])
    parser.add_argument("--output", required=True)
    parser.add_argument("--phase", action="store_true")
    args = parser.parse_args()
    try:
        result = asyncio.run(phase(args.source, args.variant)) if args.phase else run_supplement(args.source, args.variant, args.output)
    except Exception as error:
        result = {"status": "FAILED", "error": str(error), "traceback": traceback.format_exc(), "api_calls": 0}
    print(json.dumps(result, ensure_ascii=False, default=str))
    sys.exit(0 if result.get("status") == "PASSED" else 1)
