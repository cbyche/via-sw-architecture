"""Independent, evidence-backed QA05 change jobs.

This module does not infer modifiability from a file count. A completed job has
an actual source delta (or a tested baseline capability), regression evidence,
and independent review of functional responsibility claims.
"""
from __future__ import annotations

import ast
import difflib
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
from typing import Callable
import uuid


REGISTRY_VERSION = "ASR-QA-v6-U01-U18"
RESPONSIBILITIES = {
    "U01": "Natural-language semantic production prompts and interpretation rules",
    "U02": "Finite representation dispatch, item resolution and code binding",
    "U03": "Common meaning/frame/condition validation contract",
    "U04": "Request adoption, presented-question focus and input hold",
    "U05": "Input/delivery clocks and actual playback control",
    "U06": "Response content, publication and delivery ledger",
    "U07": "Task/Execution/command/question currentness",
    "U08": "Central continuation or Stage/window execution flow",
    "U09": "Source adapter and normalized original-source port",
    "U10": "Request-wise original composition/search/cache semantics",
    "U11": "Common past-relation production/coverage/invalidation",
    "U12": "Published relation reading and uncovered-range completion",
    "U13": "Dialogue/work command, acceptance and reflection contract",
    "U14": "Persistent schema migration and restore",
    "U15": "Cloud-provider wire, usage and cancellation adapter",
    "U16": "External Agent protocol/capability adapter",
    "U17": "Permission/consent/deletion policy decisions",
    "U18": "Cross-owner derived-consumer deletion/correction completion contract",
}

TASKS = {
    "M01": {
        "requirement": "Replace semantic provider wire request/response/usage while preserving normalized capabilities and upper port.",
        "completion": ["new wire input/output/cancel/usage normalized"],
        "regressions": ["P21", "schema_rejection", "late_cancel_usage"],
        "baseline_overlap_audit": False,
    },
    "M02": {
        "requirement": "Replace mail-Agent submit/cancel envelope and receipt wire, preserving command IDs and upper acceptance/unknown/completion semantics.",
        "completion": ["new envelope and receipt normalized"],
        "regressions": ["P21", "duplicate_command", "reverse_receipt", "cancel_unknown"],
        "baseline_overlap_audit": False,
    },
    "M03": {
        "requirement": "Replace calendar fields/timezone envelope while retaining normalized interval/version/permission port.",
        "completion": ["new raw times normalized without changing meaning"],
        "regressions": ["P21", "S3-03_BUSY_NO_CREATE", "stale_source", "permission_denied", "new_calendar_port_busy_and_free"],
        "baseline_overlap_audit": False,
    },
    "M04": {
        "requirement": "Continue mail-draft goal after report failure; test both terminal outcomes and preserve SEND prohibition without inventing a successful report result.",
        "completion": ["failure_continues_draft_once", "success_branch_preserved", "SEND_zero"],
        "regressions": ["P21", "AFTER", "USE_RESULT", "duplicate_failure", "late_success", "unrelated_task"],
        "baseline_overlap_audit": True,
    },
    "M05": {
        "requirement": "Task owner creates/checks allowed-answer-scope token; dialogue returns only the token actually presented for current question ID/revision.",
        "completion": ["current_presented_token_accepted_once", "wrong_task_token_rejected", "stale_token_rejected", "restore_keeps_binding"],
        "regressions": ["P21", "S6-04", "S6-02", "duplicate_answer", "reverse_event", "input_hold", "restored_initial_question_with_token"],
        "baseline_overlap_audit": False,
    },
    "M06": {
        "requirement": "Audit baseline document-consent use fence before adding any genuine derivative/context/processing-receipt delta; preserve other allowed data.",
        "completion": ["original_derived_pending_late_use_blocked", "honest_processing_receipt", "other_data_retained"],
        "regressions": ["P21", "cache_invalidation", "late_publication", "explicit_preference_registration", "consent_restore"],
        "baseline_overlap_audit": True,
    },
}

IGNORED = {".git", "__pycache__", ".pytest_cache", ".venv", "results"}


def source_snapshot(root: Path) -> dict[str, dict]:
    root = Path(root).resolve()
    snapshot = {}
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root)
        if any(p in IGNORED for p in relative.parts):
            continue
        if path.is_symlink():
            raise ValueError(f"baseline source must not contain symlink: {relative}")
        if path.is_file():
            data = path.read_bytes()
            snapshot[str(relative)] = {"sha256": hashlib.sha256(data).hexdigest(),
                                      "bytes": len(data), "mode": path.stat().st_mode & 0o777}
    return snapshot


def snapshot_digest(snapshot: dict) -> str:
    return hashlib.sha256(json.dumps(snapshot, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def prepare_jobs(variants: list[str] | None = None) -> list[dict]:
    """Create job *specifications*, not measured changes or predicted counts."""
    variants = variants or ["A", "B"]
    return [{"job_id": f"CH-DP41-{variant}-{task}", "variant": variant, "task_id": task,
             "registry_version": REGISTRY_VERSION, "specification": spec,
             "status": "NOT_RUN", "changed_unit_set": None, "count": None}
            for variant in variants for task, spec in TASKS.items()]


def _diff(before_root: Path, after_root: Path, before: dict, after: dict) -> tuple[str, list[str]]:
    output = []
    changed = []
    for relative in sorted(set(before) | set(after)):
        if before.get(relative) == after.get(relative):
            continue
        changed.append(relative)
        old = (before_root / relative).read_bytes() if relative in before else b""
        new = (after_root / relative).read_bytes() if relative in after else b""
        try:
            old_text, new_text = old.decode().splitlines(keepends=True), new.decode().splitlines(keepends=True)
            output.extend(difflib.unified_diff(old_text, new_text, fromfile=f"before/{relative}", tofile=f"after/{relative}"))
        except UnicodeDecodeError:
            output.append(f"Binary file changed: {relative}\n")
        if before.get(relative, {}).get("mode") != after.get(relative, {}).get("mode"):
            output.append(f"Mode changed {relative}: {before.get(relative, {}).get('mode')} -> {after.get(relative, {}).get('mode')}\n")
    return "".join(output), changed


def _checks_passed(checks: dict, required: list[str]) -> bool:
    return all(isinstance(checks.get(name), dict) and checks[name].get("passed") is True
               and bool(checks[name].get("evidence")) for name in required)


def validate_responsibility_review(claims: dict, reviewers: list[dict], changed_paths: list[str]) -> tuple[list[str], bool]:
    """Reject unsupported/duplicated counting; humans review functional meaning."""
    unknown = set(claims) - set(RESPONSIBILITIES)
    if unknown:
        raise ValueError(f"unknown responsibilities {sorted(unknown)}")
    for unit, claim in claims.items():
        if not claim.get("reason") or not claim.get("before_contract") or not claim.get("after_contract"):
            raise ValueError(f"{unit} requires reason and actual before/after contract")
        evidence_paths = set(claim.get("changed_paths", []))
        if not evidence_paths or not evidence_paths <= set(changed_paths):
            raise ValueError(f"{unit} claim has no corresponding actual source diff")
        if not claim.get("functional_evidence"):
            raise ValueError(f"{unit} lacks functional evidence")
    # U18 and U14 must justify a separate cross-owner/migration responsibility;
    # a cache field changed inside U10-U12 is not an extra unit.
    if "U18" in claims and not claims["U18"].get("distinct_cross_owner_contract"):
        raise ValueError("U18 needs a distinct cross-owner deletion/completion delta")
    if "U14" in claims and not claims["U14"].get("distinct_persistent_migration"):
        raise ValueError("U14 needs a distinct persistent schema migration/restore delta")
    claimed_paths = {path for claim in claims.values() for path in claim.get("changed_paths", [])}
    uncovered = {path for path in changed_paths if path not in claimed_paths
                 and "tests" not in Path(path).parts and Path(path).suffix not in (".md", ".txt")}
    if uncovered:
        raise ValueError(f"source delta without a reviewed functional responsibility: {sorted(uncovered)}")
    chosen = sorted(claims)
    distinct = {r.get("reviewer_id") for r in reviewers if r.get("reviewer_id")}
    accepted = len(distinct) >= 2 and all(r.get("accepted") is True and
                                        sorted(r.get("changed_unit_set", [])) == chosen and
                                        bool(r.get("evidence")) for r in reviewers)
    return chosen, accepted


def run_change_job(
    baseline: Path,
    output_root: Path,
    variant: str,
    task_id: str,
    patch: Callable[[Path], dict],
    regressions: Callable[[Path], dict],
    review: Callable[[Path, dict], list[dict]],
    baseline_audit: Callable[[Path], dict] | None = None,
) -> dict:
    """Execute a change on a fresh copy and preserve all failure evidence.

    ``patch`` returns functional responsibility claims (not a file count).
    ``regressions`` returns named checks, each with passed/evidence fields.
    ``review`` independently validates the recorded source and claim set.
    No network or model call is made by this framework.
    """
    task_id = task_id.replace("-", "")
    if task_id not in TASKS:
        raise ValueError(f"unknown change task {task_id}")
    baseline, output_root = Path(baseline).resolve(), Path(output_root).resolve()
    if output_root == baseline or output_root.is_relative_to(baseline):
        raise ValueError("change output must be outside baseline source")
    job_id = f"CH-DP41-{variant}-{task_id}"
    directory = output_root / job_id
    if directory.exists():
        raise FileExistsError(f"never overwrite change evidence: {directory}")
    directory.mkdir(parents=True)
    before = source_snapshot(baseline)
    baseline_hash = snapshot_digest(before)
    (directory / "baseline-manifest.json").write_text(json.dumps(before, indent=2) + "\n")
    source = directory / "source"
    shutil.copytree(baseline, source, ignore=shutil.ignore_patterns(*IGNORED))
    if snapshot_digest(source_snapshot(source)) != baseline_hash:
        raise ValueError("copied baseline differs from original")
    spec = TASKS[task_id]
    record = {"job_id": job_id, "variant": variant, "task_id": task_id,
              "registry_version": REGISTRY_VERSION, "requirement": spec,
              "baseline_source_hash": baseline_hash, "baseline_source_path": str(baseline), "status": "RUNNING",
              "evidence_level": "MEASURED_CHANGE_EXPERIMENT",
              "completion_passed": False, "regression_passed": False,
              "patch_verified": False, "responsibility_review_passed": False,
              "baseline_support_audit_passed": False, "changed_unit_set": [],
              "result_ref": str(directory / "result.json")}
    try:
        audit = baseline_audit(source) if baseline_audit else {"supports_requirement": False, "evidence": []}
        if spec["baseline_overlap_audit"] and baseline_audit is None:
            raise ValueError(f"{task_id} requires baseline capability audit before patching")
        (directory / "baseline-audit.json").write_text(json.dumps(audit, indent=2) + "\n")
        if snapshot_digest(source_snapshot(source)) != baseline_hash:
            raise ValueError("baseline audit modified copied source before the recorded patch")
        patch_result = patch(source)
        after = source_snapshot(source)
        text, paths = _diff(baseline, source, before, after)
        (directory / "patch.diff").write_text(text)
        (directory / "after-source-manifest.json").write_text(json.dumps(after, indent=2) + "\n")
        record.update({"after_source_hash": snapshot_digest(after), "changed_paths": paths,
                       "patch_sha256": hashlib.sha256(text.encode()).hexdigest(), "patch_ref": str(directory / "patch.diff")})
        checks = regressions(source)
        (directory / "regression.json").write_text(json.dumps(checks, indent=2) + "\n")
        if snapshot_digest(source_snapshot(source)) != record["after_source_hash"]:
            raise ValueError("regression runner modified the source after the recorded patch")
        record["completion_passed"] = _checks_passed(checks, spec["completion"])
        record["regression_passed"] = _checks_passed(checks, spec["regressions"])
        claims = patch_result.get("responsibility_claims", {})
        if not paths:
            # Honest zero is possible, but it must prove the unchanged baseline
            # already satisfied the *entire* requirement and all regressions.
            if claims:
                raise ValueError("no source diff cannot claim changed responsibilities")
            record["baseline_support_audit_passed"] = audit.get("supports_requirement") is True and bool(audit.get("evidence"))
            if not record["baseline_support_audit_passed"]:
                raise ValueError("zero change requires tested baseline capability evidence")
        elif not claims:
            raise ValueError("source changed without functional responsibility claims")
        record["patch_verified"] = True
        record["responsibility_claims"] = claims
        unchanged = patch_result.get("unchanged_responsibilities", {})
        record["responsibility_assessment"] = {
            unit: {"status": "CHANGED", **claims[unit]} if unit in claims else {
                "status": "UNCHANGED", "reason": unchanged.get(unit, "Same responsibility contract retained; independent diff and regression review required."),
                "before_contract": RESPONSIBILITIES[unit], "after_contract": RESPONSIBILITIES[unit],
                "evidence": ["baseline-manifest.json", "after-source-manifest.json", "regression.json"],
            } for unit in RESPONSIBILITIES
        }
        review_rows = review(source, record)
        (directory / "reviews.json").write_text(json.dumps(review_rows, indent=2) + "\n")
        if snapshot_digest(source_snapshot(source)) != record["after_source_hash"]:
            raise ValueError("responsibility review modified the tested source")
        units, accepted = validate_responsibility_review(claims, review_rows, paths)
        record["changed_unit_set"] = units
        record["responsibility_review_passed"] = accepted
        record["count"] = len(units) if record["completion_passed"] and record["regression_passed"] and accepted else None
        record["status"] = ("COMPLETED" if record["count"] is not None else "AWAITING_RESPONSIBILITY_REVIEW"
                            if record["completion_passed"] and record["regression_passed"] else "FAILED")
    except Exception as error:
        record.update({"status": "FAILED", "error": f"{type(error).__name__}: {error}", "count": None})
    finally:
        unchanged = snapshot_digest(source_snapshot(baseline)) == baseline_hash
        record["original_baseline_unchanged"] = unchanged
        if not unchanged:
            record.update({"status": "FAILED", "error": "original baseline source was modified", "count": None,
                           "patch_verified": False})
        (directory / "result.json").write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n")
    return record


def finalize_responsibility_review(result_ref: Path, reviewers: list[dict]) -> dict:
    """Append a reviewed result without replacing the preserved first execution record."""
    result_ref = Path(result_ref).resolve()
    record = json.loads(result_ref.read_text())
    directory = result_ref.parent
    target = directory / "reviewed-result.json"
    if target.exists():
        raise FileExistsError(f"never overwrite finalized review: {target}")
    if record.get("status") != "AWAITING_RESPONSIBILITY_REVIEW":
        raise ValueError("only successfully executed jobs awaiting review can be finalized")
    source_hash = snapshot_digest(source_snapshot(directory / "source"))
    baseline_hash = snapshot_digest(source_snapshot(Path(record["baseline_source_path"])))
    if source_hash != record["after_source_hash"] or baseline_hash != record["baseline_source_hash"]:
        raise ValueError("source or original baseline changed before responsibility review")
    patch_hash = hashlib.sha256(Path(record["patch_ref"]).read_bytes()).hexdigest()
    if patch_hash != record["patch_sha256"]:
        raise ValueError("recorded patch changed before responsibility review")
    units, accepted = validate_responsibility_review(record["responsibility_claims"], reviewers, record["changed_paths"])
    attempt_ref = directory / f"review-attempt-{uuid.uuid4().hex}.json"
    attempt_ref.write_text(json.dumps(reviewers, indent=2, ensure_ascii=False) + "\n")
    if not accepted:
        return {**record, "responsibility_review_passed": False, "count": None,
                "review_attempt_ref": str(attempt_ref), "status": "AWAITING_RESPONSIBILITY_REVIEW"}
    (directory / "final-reviews.json").write_text(json.dumps(reviewers, indent=2, ensure_ascii=False) + "\n")
    updated = {**record, "changed_unit_set": units, "responsibility_review_passed": accepted,
               "count": len(units) if accepted else None, "status": "COMPLETED" if accepted else "FAILED",
               "initial_result_ref": str(result_ref), "result_ref": str(target)}
    target.write_text(json.dumps(updated, indent=2, ensure_ascii=False) + "\n")
    return updated


def _adapters_path(source: Path) -> Path:
    choices = [source / "prototypes/candidates/dp41_45_poc/shared/adapters.py",
               source / "shared/adapters.py", source / "adapters.py"]
    found = [p for p in choices if p.is_file()]
    if len(found) != 1:
        raise ValueError("frozen source must contain exactly one known adapters.py port")
    return found[0]


def _replace_method(text: str, class_name: str, method_name: str, transform: Callable[[str], str]) -> str:
    tree = ast.parse(text)
    owner = next(node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == class_name)
    method = next(node for node in owner.body if isinstance(node, ast.FunctionDef) and node.name == method_name)
    lines = text.splitlines(keepends=True)
    before = "".join(lines[method.lineno - 1:method.end_lineno])
    after = transform(before)
    if after == before:
        raise ValueError(f"change requirement did not alter {class_name}.{method_name}")
    return "".join(lines[:method.lineno - 1]) + after + "".join(lines[method.end_lineno:])


def _once(text: str, old: str, new: str) -> str:
    if text.count(old) != 1:
        raise ValueError("frozen adapter contract differs from the recorded change requirement")
    return text.replace(old, new, 1)


def patch_adapter_requirement(source: Path, task_id: str, variant: str = "A") -> dict:
    """Apply actual alternate wire/scope contracts on the independent copied baseline."""
    task_id = task_id.replace("-", "")
    if task_id == "M04":
        from benchmark.architecture.dp41_45_poc.change_patches import patch_failure_continuation
        return patch_failure_continuation(source, variant)
    if task_id == "M06":
        return {}  # Accepted only with complete runtime baseline audit, never inferred.
    path = _adapters_path(Path(source))
    relative = str(path.relative_to(source))
    text = path.read_text()
    if task_id == "M01":
        old = '''            return {"model": self.model_id, "messages": messages, "temperature": 0,
                    "max_tokens": max_tokens, "enable_thinking": False,
                    "response_format": {"type": "json_object"}}'''
        new = '''            return {"model": self.model_id, "conversation": messages,
                    "generation": {"temperature": 0, "max_tokens": max_tokens,
                                   "thinking": False, "format": "json_object"}}'''
        text = _replace_method(text, "ProviderWireAdapter", "encode", lambda body: _once(body, old, new))
        text = _replace_method(text, "ProviderWireAdapter", "decode", lambda body:
            _once(_once(body, 'raw_usage = response.get("usage")',
                        'raw_usage = response.get("metering") if self.provider == "qwen" else response.get("usage")'),
                  'text = response["choices"][0]["message"].get("content") or ""',
                  'text = response["result"].get("text") or ""'))
        units = {"U15": ("Provider API serialization and usage normalization changed while upper messages/text/usage port is stable.",
                           "Qwen-compatible messages/choices/usage wire", "alternate conversation/generation/result/metering wire")}
    elif task_id == "M02":
        text = _once(text, 'protocol = "agent-wire-v1"', 'protocol = "agent-wire-v2"')
        bodies = {
            "encode_command": '''    def encode_command(self, normalized):
        value = self._validate(normalized, command=True)
        route = {key: value[key] for key in ("command_id", "task_id", "execution_id", "request_revision")}
        return {"version": self.protocol, "routing": route, "action": value["operation"], "input": value["payload"],
                "extensions": {k: v for k, v in value.items() if k not in {*route, "operation", "payload"}}}
''',
            "decode_command": '''    def decode_command(self, wire):
        if not isinstance(wire, dict) or wire.get("version") != self.protocol:
            raise AdapterError("Agent command protocol mismatch")
        value = {**wire.get("extensions", {}), **wire.get("routing", {}),
                 "operation": wire.get("action"), "payload": wire.get("input")}
        return self._validate(value, command=True)
''',
            "encode_receipt": '''    def encode_receipt(self, normalized):
        value = self._validate(normalized, command=False)
        route = {key: value[key] for key in ("command_id", "task_id", "execution_id", "request_revision")}
        return {"version": self.protocol, "routing": route,
                "result": {k: v for k, v in value.items() if k not in route}}
''',
            "decode_receipt": '''    def decode_receipt(self, wire):
        if not isinstance(wire, dict) or wire.get("version") != self.protocol:
            raise AdapterError("Agent receipt protocol mismatch")
        return self._validate({**wire.get("result", {}), **wire.get("routing", {})}, command=False)
''',
        }
        for method, body in bodies.items():
            text = _replace_method(text, "AgentWireAdapter", method, lambda old, new=body: new)
        units = {"U16": ("Gateway transport contract changes; original command routing, operation, payload and receipt status are preserved.",
                           "v1 protocol with command/receipt body", "v2 routing/action/input/result envelope")}
    elif task_id == "M03":
        preface = '''        if raw_source.get("resource_type") == "CALENDAR_V2":
            envelope = raw_source["calendar"]
            metadata = {**copy.deepcopy(envelope.get("metadata_extension", {})), "date": envelope["day"],
                        "timezone": envelope["zone"], "start": envelope.get("start_utc"), "end": envelope.get("end_utc"),
                        "state": envelope.get("availability"), "interval_state": envelope.get("availability"),
                        "events": [{"start": e["begins_at"], "end": e["ends_at"], "title": e["label"]}
                                   for e in envelope.get("appointments", [])]}
            raw_source = {"source_id": raw_source["calendar_uid"], "revision": raw_source["etag_revision"],
                          "kind": "CALENDAR", "title": raw_source.get("label", raw_source["calendar_uid"]),
                          "text": raw_source["original_text"], "permission": copy.deepcopy(raw_source["access"]),
                          "metadata": metadata}
'''
        text = _replace_method(text, "CalendarSourceAdapter", "normalize", lambda body:
            _once(body, "        result = copy.deepcopy(raw_source)\n", preface + "        result = copy.deepcopy(raw_source)\n"))
        units = {"U09": ("Calendar source adapter accepts changed external fields/timezone envelope without changing stable original-source meaning.",
                           "source_id/revision/text/metadata local-time fields", "calendar_uid/etag_revision/original_text/calendar UTC envelope")}
    elif task_id == "M05":
        text = _once(text, "import copy\n", "import copy\nimport secrets\n")
        text = _replace_method(text, "QuestionAuthority", "issue", lambda body:
            _once(body, "        return result\n", '        result.setdefault("allowed_answer_scope_token", secrets.token_hex(16))\n        return result\n'))
        text = _replace_method(text, "QuestionAuthority", "validate_answer", lambda body:
            _once(body, '        qid = question.get("question_id")\n',
                  '        qid = question.get("question_id")\n'
                  '        if not question.get("allowed_answer_scope_token") or answer.get("allowed_answer_scope_token") != question["allowed_answer_scope_token"]:\n'
                  '            return {"accepted": False, "reason": "INVALID_SCOPE_TOKEN", "question_id": qid}\n'))
        text = _replace_method(text, "PresentedQuestionBinding", "reflect_answer", lambda body:
            _once(body, '        return {"question_ref": question_id, "question_revision": revision,\n',
                  '        if not question.get("allowed_answer_scope_token"):\n'
                  '            raise AdapterError("presented question lacks Task-issued scope token")\n'
                  '        return {"question_ref": question_id, "question_revision": revision,\n'
                  '                "allowed_answer_scope_token": question["allowed_answer_scope_token"],\n'))
        runtime_path = Path(source) / "prototypes/candidates/dp41_45_poc/shared/runtime.py"
        runtime_relative = None
        if runtime_path.is_file():
            runtime_text = runtime_path.read_text()
            runtime_text = _once(runtime_text, '        for question in self.owners.questions.values():\n',
                '        for question_id, stored_question in list(self.owners.questions.items()):\n'
                '            question = self.question_authority.issue(stored_question, task_id=stored_question.get("task_id"),\n'
                '                execution_id=stored_question.get("execution_id"))\n'
                '            self.owners.questions[question_id] = question\n')
            ast.parse(runtime_text)
            runtime_path.write_text(runtime_text)
            runtime_relative = str(runtime_path.relative_to(source))
        units = {
            "U07": ("Task question owner issues/checks token tied to its current question identity/revision/scope.",
                     "question identity/revision/scope authority without token", "Task-issued token required in addition to existing checks"),
            "U04": ("Dialogue adoption reflects only the token in the actually presented question and preserves it across restore.",
                     "actually presented identity/revision reflection", "actually presented identity/revision/token reflection"),
        }
    else:
        raise ValueError(f"unsupported adapter change task {task_id}")
    ast.parse(text)
    path.write_text(text)
    claims = {unit: {"reason": values[0], "before_contract": values[1],
                                           "after_contract": values[2], "changed_paths": [relative],
                                           "functional_evidence": ["adapter-runtime-checks.json", "regression.json"]}
              for unit, values in units.items()}
    if task_id == "M05" and runtime_relative:
        claims["U07"]["changed_paths"].append(runtime_relative)
        claims["U07"]["reason"] += " Existing owner question records are issued the new token before restored presentation binding."
    return {"responsibility_claims": claims}


def execute_adapter_requirement(source: Path, task_id: str) -> dict:
    """Actual deterministic transport/scope execution, explicitly not semantic quality."""
    task_id = task_id.replace("-", "")
    path = _adapters_path(Path(source))
    spec = importlib.util.spec_from_file_location("qa05_adapter_" + uuid.uuid4().hex, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    observations = []
    if task_id == "M01":
        adapter = module.ProviderWireAdapter("qwen", "alternate-provider-fixture")
        context = [{"role": "user", "content": [{"type": "text", "text": "actual input"},
                                                 {"type": "image_url", "image_url": {"url": "fixture://image"}}]}]
        wire = adapter.encode(context, max_tokens=200)
        assert wire["conversation"] == context and wire["generation"]["max_tokens"] == 200
        assert wire["generation"]["format"] == "json_object"
        decoded = adapter.decode({"id": "v2-call", "result": {"text": '{"target":"wrong-original-output"}'},
                                  "metering": {"input_tokens": 80, "output_tokens": 20,
                                               "input_tokens_details": {"cached_tokens": 10}}})
        assert decoded["text"] == '{"target":"wrong-original-output"}'
        assert decoded["usage"]["input_tokens"] == 80 and decoded["usage"]["output_tokens"] == 20
        assert decoded["usage"]["input_tokens_details"]["cached_tokens"] == 10
        late = adapter.decode({"id": "late-call", "result": {"text": "late-output"},
                               "metering": {"input_tokens": 90, "output_tokens": 10}})
        assert late["usage"]["input_tokens"] == 90 and late["usage"]["output_tokens"] == 10
        unknown = adapter.decode({"result": {"text": "unreported-usage"}})
        assert unknown["usage"] is None
        observations.extend([wire, decoded, {"late_response_normalized": late,
                                               "caller_disposition": "NOT_ADOPTED_BY_TEST_TRANSPORT", "unknown_usage": unknown}])
    elif task_id == "M02":
        adapter = module.AgentWireAdapter()
        command = {"command_id": "C1", "task_id": "T1", "execution_id": "X1", "request_revision": 2,
                   "operation": "SEND_MAIL", "payload": {"recipient": "wrong@example.invalid", "body": "original wrong payload"}}
        wire = adapter.encode_command(command)
        assert wire["version"] == "agent-wire-v2" and wire["action"] == "SEND_MAIL"
        assert adapter.decode_command(wire) == command
        received = []
        for status in ("COMPLETED", "ACCEPTED", "ACCEPTED", "CANCEL_ACCEPTED_TERMINATION_UNKNOWN"):
            receipt = {**{k: command[k] for k in ("command_id", "task_id", "execution_id", "request_revision")},
                       "status": status, "terminated": False}
            new_wire = adapter.encode_receipt(receipt)
            assert adapter.decode_receipt(new_wire) == receipt
            received.append(new_wire)
        observations.extend([wire, {"reverse_duplicate_and_cancel_receipts": received}])
    elif task_id == "M03":
        raw = {"resource_type": "CALENDAR_V2", "calendar_uid": "Cal16", "etag_revision": 2,
               "original_text": "2026-10-16 15:00~16:00 BUSY; alternative 16:00~17:00",
               "access": {"read": "DENIED", "model": "DENIED", "agent": "DENIED"},
               "calendar": {"day": "2026-10-16", "zone": "Asia/Seoul", "availability": "BUSY",
                            "start_utc": "2026-10-16T06:00:00+00:00", "end_utc": "2026-10-16T07:00:00+00:00",
                            "appointments": [{"begins_at": "2026-10-16T06:00:00+00:00", "ends_at": "2026-10-16T07:00:00+00:00", "label": "existing meeting"}]}}
        normalized = module.CalendarSourceAdapter().normalize(raw)
        assert normalized["source_id"] == "Cal16" and normalized["revision"] == 2
        assert normalized["text"] == raw["original_text"] and normalized["permission"] == raw["access"]
        assert normalized["metadata"]["start"] == "2026-10-16T15:00:00+09:00"
        assert normalized["metadata"]["state"] == "BUSY"
        observations.extend([raw, normalized])
    elif task_id == "M05":
        authority, binding = module.QuestionAuthority(), module.PresentedQuestionBinding()
        question = authority.issue({"question_id": "Q1", "revision": 7, "status": "VALID", "delivered_receipt": True,
                                    "text": "Use first-half period?", "scope": {"property": "PERIOD"}}, task_id="T1", execution_id="X1")
        receipt = {"publication_id": "P1", "text_presented_s": 0, "voice_on": False}
        binding.present(question, receipt=receipt)
        answer = binding.reflect_answer("Q1", 7, task_id="T1", answer="yes", properties={"PERIOD": "2026-H1"})
        assert authority.validate_answer(question, answer, task_id="T1")["accepted"]
        other = authority.issue({"question_id": "Q2", "revision": 7, "status": "VALID", "text": "Other task?",
                                 "delivered_receipt": True}, task_id="T2", execution_id="X2")
        assert not authority.validate_answer(question, {**answer, "allowed_answer_scope_token": other["allowed_answer_scope_token"]}, task_id="T1")["accepted"]
        assert not authority.validate_answer(question, {**answer, "question_revision": 6}, task_id="T1")["accepted"]
        restored = module.PresentedQuestionBinding()
        restored.restore(binding.snapshot())
        assert restored.reflect_answer("Q1", 7, task_id="T1", answer="yes", properties={"PERIOD": "2026-H1"}) == answer
        assert authority.issue(question, task_id="T1", execution_id="X1")["allowed_answer_scope_token"] == answer["allowed_answer_scope_token"]
        authority.validate_answer(question, answer, task_id="T1", consume=True)
        assert not authority.validate_answer(question, answer, task_id="T1")["accepted"]
        observations.extend([question, answer, {"wrong_task_token_rejected": True, "stale_token_rejected": True,
                                               "restore_keeps_binding": True, "duplicate_answer_rejected": True}])
    elif task_id not in ("M04", "M06"):
        raise ValueError(f"unknown adapter requirement {task_id}")
    return {"passed": True, "observations": observations, "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION"}


def execute_runtime_requirement(source: Path, variant: str, task_id: str, *, audit: bool = False) -> dict:
    """Execute only the copied candidate; record stdout/errors outside that source."""
    source = Path(source).resolve()
    checker = Path(__file__).with_name("change_runtime_checks.py")
    prefix = "baseline-runtime-requirement-checks" if audit else "runtime-requirement-checks"
    ref = source.parent / (prefix + ".json")
    try:
        process = subprocess.run([sys.executable, "-B", str(checker), str(source), variant, task_id],
                                 cwd=source, capture_output=True, text=True, timeout=30)
        if process.returncode:
            record = {"checks": {}, "returncode": process.returncode, "stdout": process.stdout,
                      "stderr": process.stderr, "source_root": str(source), "api_calls": 0,
                      "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION"}
        else:
            record = json.loads(process.stdout)
            record["stderr"] = process.stderr
    except subprocess.TimeoutExpired as error:
        def decoded(value):
            return value.decode(errors="replace") if isinstance(value, bytes) else value
        record = {"checks": {}, "collection_error": "RUNTIME_PROBE_TIMEOUT", "timeout_seconds": 30,
                  "stdout": decoded(error.stdout), "stderr": decoded(error.stderr), "source_root": str(source),
                  "api_calls": 0, "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION"}
    ref.write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n")
    return {name: {**row, "evidence": [str(ref) + "#" + name]}
            for name, row in record.get("checks", {}).items()}


def run_dp41_change_jobs(baseline: Path, output_root: Path, regression_runner: Callable[[Path, str, str], dict],
                        review_runner: Callable[[Path, dict], list[dict]] | None = None) -> list[dict]:
    """Run 12 real independent jobs after baseline freeze; never invokes a model API.

    The supplied runner must execute and preserve P21/state/tail regressions.
    Scripted replay supports QA05 structural regression, not semantic accuracy.
    Two independent responsibility reviews can be finalized after execution.
    """
    records = []
    for job in prepare_jobs():
        variant, task = job["variant"], job["task_id"]
        def checks(source, variant=variant, task=task):
            actual = execute_adapter_requirement(source, task)
            ref = Path(source).parent / "adapter-runtime-checks.json"
            ref.write_text(json.dumps(actual, indent=2, ensure_ascii=False) + "\n")
            supplied = regression_runner(source, variant, task)
            supplied.update(execute_runtime_requirement(source, variant, task))
            if task in ("M01", "M02", "M03"):
                supplied[TASKS[task]["completion"][0]] = {"passed": actual["passed"], "evidence": [str(ref)]}
            elif task == "M05":
                for name in TASKS[task]["completion"]:
                    supplied[name] = {"passed": actual["passed"], "evidence": [str(ref)]}
            return supplied
        def audit(source, variant=variant, task=task):
            observed = regression_runner(source, variant, task)
            observed.update(execute_runtime_requirement(source, variant, task, audit=True))
            ref = Path(source).parent / "baseline-runtime-checks.json"
            ref.write_text(json.dumps(observed, indent=2, ensure_ascii=False) + "\n")
            return {"supports_requirement": _checks_passed(observed, TASKS[task]["completion"] + TASKS[task]["regressions"]),
                    "evidence": [str(ref)], "evidence_level": "SCRIPTED_FUNCTIONAL_REGRESSION"}
        records.append(run_change_job(baseline, output_root, variant, task,
                                      lambda source, task=task, variant=variant: patch_adapter_requirement(source, task, variant), checks,
                                      review_runner or (lambda source, record: []),
                                      audit if TASKS[task]["baseline_overlap_audit"] else None))
    return records
