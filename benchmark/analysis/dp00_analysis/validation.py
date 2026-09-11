"""Strict shape, schema, provenance, and cross-stream validation."""

from __future__ import annotations

from typing import Any, Iterable

from .models import ValidationIssue

CANONICAL_EVENT_VERSION = "canonical-event-v3"
SUPPORTED_CANONICAL_EVENT_VERSIONS = {"canonical-event-v2", CANONICAL_EVENT_VERSION}
MODEL_CALL_VERSION = "model-call-v1"
RUN_PROVENANCE_VERSION = "dp00-pilot-provenance-v3"
SUPPORTED_RUN_PROVENANCE_VERSIONS = {"dp00-pilot-provenance-v2", RUN_PROVENANCE_VERSION}
CAMPAIGN_PROVENANCE_VERSION = "dp00-pilot-campaign-provenance-v1"

EMITTERS = {
    "BENCHMARK", "ARCHITECTURE_UNDER_TEST", "MODEL_FIXTURE", "AGENT_FIXTURE",
    "TOOL_FIXTURE", "OUTCOME_PROBE", "INTERACTION_FIXTURE",
}
ALTERNATIVES = {"A", "B", "C", "D"}
CALL_CLASSES = {"ORCHESTRATION", "DOMAIN", "MIXED"}
MODEL_STATUSES = {"COMPLETED", "FAILED", "TIMED_OUT", "NO_RESPONSE", "MALFORMED"}
RESPONSIBILITIES = {
    "INTENT_INTERPRETATION", "REFERENT_RESOLUTION", "TASK_ASSOCIATION",
    "CLARIFICATION_DECISION", "EXECUTION_ROUTE_SELECTION", "AGENT_SELECTION",
    "RESULT_ASSOCIATION",
}
ROUTE_KINDS = {"LOCAL_DIRECT", "EXECUTOR_DIRECT", "EXECUTOR_DELEGATED"}
EFFECT_TYPES = {
    "VOLUME_CHANGED", "FILE_INSPECTED", "WIFI_STATUS_OBSERVED", "DOCUMENT_OPENED",
    "DNS_CHECK_OBSERVED", "DOWNLOADS_ORGANIZED", "MEDIA_PAUSED", "DIAGNOSIS_STARTED",
}
FAILURE_REASONS = {
    "MODEL_MALFORMED", "MODEL_TIMEOUT", "MODEL_NO_RESPONSE", "INVALID_ROUTE",
    "DISPATCH_REJECTED", "EXECUTION_FAILURE",
}
CORRELATION_KEYS = {
    "turn_id", "task_id", "parent_task_id", "child_task_id", "subgoal_id",
    "execution_id", "dispatch_id", "result_id", "clarification_id"
}
LEGACY_CORRELATION_KEYS = {
    "turn_id", "task_id", "execution_id", "dispatch_id", "result_id", "clarification_id"
}
EVENT_KEYS = {
    "schema_version", "event_id", "run_id", "episode_id", "scenario_id",
    "scenario_version", "alternative_id", "benchmark_version", "source_git_commit",
    "sequence_number", "monotonic_timestamp", "emitter", "product_correlation", "event",
}
MODEL_CALL_KEYS = {
    "schema_version", "model_call_id", "run_id", "episode_id", "scenario_id",
    "alternative_id", "logical_sequence", "attempt", "decision_owner",
    "semantic_responsibilities", "status", "semantic_output_reference",
    "route_committed_before_call", "route_committed_after_call", "logical_start",
    "first_output", "completion", "failure", "call_class", "classification_reason",
    "qa04_primary_included", "route_commit_event_id",
}
FIXTURE_EVENT_KEYS = {
    "run_id", "episode_id", "sequence_number", "monotonic_timestamp_nanos",
    "fixture_kind", "action", "subject_id", "outcome", "observable_effect",
}
LEGACY_RUN_KEYS = {
    "provenance_schema_version", "run_id", "campaign_id", "campaign_profile_sequence_index",
    "official", "source_git_commit", "working_tree_clean", "pilot_corpus_id",
    "pilot_corpus_version", "alternative", "scenario_id", "scenario_version",
    "semantic_behavior_plan_id", "semantic_behavior_plan_version", "latency_profile",
    "warmup_count", "measured_repetition_count", "measurement_population", "order_policy",
    "order_cycle", "sequence_position", "repetition_index", "instrumentation_mode",
    "episode_elapsed_nanos", "event_count", "capture_append_cost_nanos", "model_profile",
    "prompt_profile", "cache_policy", "rust_toolchain", "rustc_version", "cargo_version",
    "target", "build_profile", "tokio_resolved_version", "runtime_worker_policy",
    "cargo_lock_identity", "os", "machine_architecture", "canonical_event_schema_version",
    "model_call_schema_version",
}
RUN_KEYS = LEGACY_RUN_KEYS | {
    "source_sha", "calibration_id", "cycle_id", "pair_id", "order_slot",
    "mode_order_slot", "repetition_id", "attempted_event_count",
    "measurement_spine_event_count",
}
CAMPAIGN_KEYS = {
    "provenance_schema_version", "campaign_id", "source_git_commit",
    "initial_working_tree_clean", "pilot_corpus_id", "pilot_corpus_version",
    "profile_sequence", "campaign_configuration", "runner_version",
}
RUNTIME_SCENARIO_KEYS = {
    "schema_version", "asset_id", "asset_version", "scenario_id", "scenario_version", "title",
    "description", "catalog_scenario_ref", "catalog_version", "scenario_class",
    "architecture_sensitivity_tags", "stimulus", "dependency_fixture_refs",
    "semantic_behavior_plan_ref", "evaluator_oracle_ref", "evaluator_oracle_refs",
    "qa_eligibility", "termination", "provenance",
}
BEHAVIOR_PLAN_KEYS = {
    "schema_version", "asset_id", "asset_version", "behavior_plan_id", "behavior_plan_version",
    "scenario_id", "scenario_version", "semantic_schema_version", "payload_registry_version",
    "allowed_owner_mapping_version", "operations",
}
ORACLE_KEYS = {
    "schema_version", "asset_id", "asset_version", "scenario_id", "ground_truth",
    "constraint_manifest", "expected_result_binding", "success_predicate",
}


class StrictValidationError(ValueError):
    pass


def validate_runtime_scenario(value: Any) -> dict[str, Any]:
    obj = _object(value, "runtime scenario")
    _exact(obj, RUNTIME_SCENARIO_KEYS, "runtime scenario")
    if obj["schema_version"] != "runtime-scenario-pilot-v0":
        raise StrictValidationError("runtime scenario: unsupported schema version")
    for key in ("asset_id", "asset_version", "scenario_id", "scenario_version", "scenario_class"):
        _nonempty(obj[key], f"runtime scenario.{key}")
    if obj["scenario_class"] not in {f"R{i}" for i in range(1, 11)}:
        raise StrictValidationError("runtime scenario: invalid scenario_class")
    qa = _object(obj["qa_eligibility"], "runtime scenario.qa_eligibility")
    qa04 = _object(qa.get("qa04"), "runtime scenario.qa_eligibility.qa04")
    _enum(
        qa04.get("route_commit_expectation"),
        {"REQUIRED", "OPTIONAL", "FORBIDDEN"},
        "runtime scenario.qa_eligibility.qa04.route_commit_expectation",
    )
    required_subgoals = qa04.get("required_subgoal_ids")
    if not isinstance(required_subgoals, list) or not all(
        isinstance(value, str) and value for value in required_subgoals
    ) or len(required_subgoals) != len(set(required_subgoals)):
        raise StrictValidationError(
            "runtime scenario.qa_eligibility.qa04.required_subgoal_ids: "
            "expected unique non-empty string array"
        )
    return obj


def validate_behavior_plan(value: Any) -> dict[str, Any]:
    obj = _object(value, "behavior plan")
    _exact(obj, BEHAVIOR_PLAN_KEYS, "behavior plan")
    if obj["schema_version"] != "semantic-behavior-plan-pilot-v0":
        raise StrictValidationError("behavior plan: unsupported schema version")
    if not isinstance(obj["operations"], list) or not obj["operations"]:
        raise StrictValidationError("behavior plan.operations: expected non-empty array")
    op_keys = {"operation_key", "turn_id", "responsibility", "input_contract_id", "output_schema_id", "operation_group", "attempts"}
    attempt_keys = {"attempt", "behavior_class", "payload_ref", "failure_behavior"}
    for index, operation in enumerate(obj["operations"]):
        operation = _object(operation, f"behavior plan.operations[{index}]")
        _exact(operation, op_keys, f"behavior plan.operations[{index}]")
        _enum(operation["responsibility"], RESPONSIBILITIES | {"RESULT_BINDING"}, f"behavior plan.operations[{index}].responsibility")
        if not isinstance(operation["attempts"], list) or not operation["attempts"]:
            raise StrictValidationError("behavior plan operation has no attempts")
        for attempt in operation["attempts"]:
            attempt = _object(attempt, "behavior plan attempt")
            _exact(attempt, attempt_keys, "behavior plan attempt")
            _enum(attempt["behavior_class"], {"CORRECT", "AMBIGUOUS", "WRONG_CANDIDATE", "MALFORMED", "TIMEOUT", "NO_RESPONSE"}, "behavior plan attempt.behavior_class")
    return obj


def validate_oracle(value: Any) -> dict[str, Any]:
    obj = _object(value, "oracle")
    _exact(obj, ORACLE_KEYS, "oracle")
    if obj["schema_version"] != "oracle-asset-pilot-v0":
        raise StrictValidationError("oracle: unsupported schema version")
    for key in ("asset_id", "asset_version", "scenario_id"):
        _nonempty(obj[key], f"oracle.{key}")
    manifest = _object(obj["constraint_manifest"], "oracle.constraint_manifest")
    constraints = manifest.get("constraints")
    if not isinstance(constraints, list) or not constraints:
        raise StrictValidationError("oracle.constraint_manifest.constraints: expected non-empty array")
    for constraint in constraints:
        constraint = _object(constraint, "oracle constraint")
        _enum(
            constraint.get("constraint_set"),
            {"REQUIRED", "ALLOWED", "FORBIDDEN"},
            "oracle constraint.constraint_set",
        )
    return obj


def _object(value: Any, where: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise StrictValidationError(f"{where}: expected object")
    return value


def _exact(obj: dict[str, Any], keys: set[str], where: str) -> None:
    missing = keys - obj.keys()
    unknown = obj.keys() - keys
    if missing:
        raise StrictValidationError(f"{where}: missing required fields {sorted(missing)}")
    if unknown:
        raise StrictValidationError(f"{where}: unknown fields {sorted(unknown)}")


def _nonempty(value: Any, where: str) -> None:
    if not isinstance(value, str) or not value:
        raise StrictValidationError(f"{where}: expected non-empty string")


def _enum(value: Any, allowed: set[str], where: str) -> None:
    if value not in allowed:
        raise StrictValidationError(f"{where}: invalid enum {value!r}")


def _route(value: Any, where: str) -> None:
    obj = _object(value, where)
    _exact(obj, {"route_kind", "initial_executor_id", "final_executor_id_if_known", "delegation_chain"}, where)
    _enum(obj["route_kind"], ROUTE_KINDS, f"{where}.route_kind")
    _nonempty(obj["initial_executor_id"], f"{where}.initial_executor_id")
    if obj["final_executor_id_if_known"] is not None:
        _nonempty(obj["final_executor_id_if_known"], f"{where}.final_executor_id_if_known")
    if not isinstance(obj["delegation_chain"], list) or not all(isinstance(x, str) and x for x in obj["delegation_chain"]):
        raise StrictValidationError(f"{where}.delegation_chain: expected string array")


def _effect(value: Any, where: str) -> None:
    obj = _object(value, where)
    keys = {"effect_type", "capability_id", "subject_id", "target_id", "value", "before_value", "after_value", "state", "executor_id", "authoritative_source"}
    _exact(obj, keys, where)
    _enum(obj["effect_type"], EFFECT_TYPES, f"{where}.effect_type")
    _enum(obj["authoritative_source"], EMITTERS, f"{where}.authoritative_source")
    _nonempty(obj["capability_id"], f"{where}.capability_id")
    for key in ("subject_id", "target_id", "value", "state", "executor_id"):
        if obj[key] is not None and not isinstance(obj[key], str):
            raise StrictValidationError(f"{where}.{key}: expected string or null")
    for key in ("before_value", "after_value"):
        if obj[key] is not None and (not isinstance(obj[key], int) or isinstance(obj[key], bool)):
            raise StrictValidationError(f"{where}.{key}: expected integer or null")
    if obj["effect_type"] == "VOLUME_CHANGED" and (obj["before_value"] is None or obj["after_value"] is None):
        raise StrictValidationError(f"{where}: missing state transition boundary")


def validate_canonical_event(value: Any) -> dict[str, Any]:
    obj = _object(value, "canonical event")
    _exact(obj, EVENT_KEYS, "canonical event")
    if obj["schema_version"] not in SUPPORTED_CANONICAL_EVENT_VERSIONS:
        raise StrictValidationError(f"canonical event: unsupported schema version {obj['schema_version']!r}")
    for key in ("event_id", "run_id", "episode_id", "scenario_id", "scenario_version", "alternative_id", "benchmark_version", "source_git_commit"):
        _nonempty(obj[key], f"canonical event.{key}")
    _enum(obj["alternative_id"], ALTERNATIVES, "canonical event.alternative_id")
    _enum(obj["emitter"], EMITTERS, "canonical event.emitter")
    if not isinstance(obj["sequence_number"], int) or not isinstance(obj["monotonic_timestamp"], int) or obj["sequence_number"] < 0 or obj["monotonic_timestamp"] < 0:
        raise StrictValidationError("canonical event: sequence/timestamp must be non-negative integers")
    correlation = _object(obj["product_correlation"], "product_correlation")
    _exact(
        correlation,
        CORRELATION_KEYS
        if obj["schema_version"] == CANONICAL_EVENT_VERSION
        else LEGACY_CORRELATION_KEYS,
        "product_correlation",
    )
    event = obj["event"]
    if isinstance(event, str):
        if event not in {"AcousticEos", "ModelGenerationStarted", "ModelGenerationCompleted", "EpisodeCompleted"}:
            raise StrictValidationError(f"canonical event.event: invalid variant {event!r}")
        return obj
    event = _object(event, "canonical event.event")
    if len(event) != 1:
        raise StrictValidationError("canonical event.event: expected one variant")
    variant, payload = next(iter(event.items()))
    if variant == "Architecture":
        _validate_architecture(payload, obj["schema_version"])
        if (
            obj["schema_version"] == CANONICAL_EVENT_VERSION
            and isinstance(payload, dict)
            and "RouteCommitted" in payload
        ):
            committed_subgoal = payload["RouteCommitted"]["subgoal_id"]
            if committed_subgoal is not None:
                if (
                    correlation["parent_task_id"] is None
                    or correlation["child_task_id"] is None
                    or correlation["task_id"] != correlation["child_task_id"]
                ):
                    raise StrictValidationError(
                        "Architecture.RouteCommitted: incomplete compound task correlation"
                    )
                if correlation["subgoal_id"] != committed_subgoal:
                    raise StrictValidationError(
                        "Architecture.RouteCommitted: subgoal payload/correlation mismatch"
                    )
    elif variant == "ExecutionStarted":
        payload = _object(payload, "ExecutionStarted")
        _exact(payload, {"invocation"}, "ExecutionStarted")
        invocation = _object(payload["invocation"], "ExecutionStarted.invocation")
        _exact(invocation, {"capability_id", "executor_id"}, "ExecutionStarted.invocation")
        _nonempty(invocation["capability_id"], "ExecutionStarted.invocation.capability_id")
        _nonempty(invocation["executor_id"], "ExecutionStarted.invocation.executor_id")
    elif variant == "UsefulOutcomeObserved":
        payload = _object(payload, "UsefulOutcomeObserved")
        _exact(payload, {"effect"}, "UsefulOutcomeObserved")
        _effect(payload["effect"], "UsefulOutcomeObserved.effect")
    elif variant == "EpisodeFailed":
        payload = _object(payload, "EpisodeFailed")
        _exact(payload, {"reason"}, "EpisodeFailed")
        _enum(payload["reason"], FAILURE_REASONS, "EpisodeFailed.reason")
    else:
        raise StrictValidationError(f"canonical event.event: invalid variant {variant!r}")
    return obj


def _validate_architecture(value: Any, schema_version: str) -> None:
    if isinstance(value, str):
        if value not in {"ProcessingStarted", "TaskCreated", "TaskReused", "ResultBound", "CancelPropagated"}:
            raise StrictValidationError(f"Architecture: invalid variant {value!r}")
        return
    obj = _object(value, "Architecture")
    if len(obj) != 1:
        raise StrictValidationError("Architecture: expected one variant")
    variant, payload = next(iter(obj.items()))
    fields = {
        "ReferentBound": {"referent_role", "resolved_referent_id"},
        "TaskAssociated": {"task_relation"},
        "ClarificationRequested": {"prompt", "reason"},
        "ClarificationResolved": {"request_turn_id", "response_turn_id"},
        "RouteCandidateObserved": {"route"},
        "RouteCandidateRejected": {"route", "reason"},
        "RouteCommitted": (
            {"route", "subgoal_id"}
            if schema_version == CANONICAL_EVENT_VERSION
            else {"route"}
        ),
    }
    if variant not in fields:
        raise StrictValidationError(f"Architecture: invalid variant {variant!r}")
    payload = _object(payload, f"Architecture.{variant}")
    _exact(payload, fields[variant], f"Architecture.{variant}")
    if variant == "ReferentBound":
        _enum(payload["referent_role"], {"SOURCE", "DESTINATION"}, f"Architecture.{variant}.referent_role")
        _nonempty(payload["resolved_referent_id"], f"Architecture.{variant}.resolved_referent_id")
    elif variant == "TaskAssociated":
        _enum(payload["task_relation"], {"NEW", "FOLLOW_UP"}, f"Architecture.{variant}.task_relation")
    elif variant == "ClarificationRequested":
        _nonempty(payload["prompt"], f"Architecture.{variant}.prompt")
        _enum(payload["reason"], {"AMBIGUOUS_REFERENT"}, f"Architecture.{variant}.reason")
    elif variant == "ClarificationResolved":
        _nonempty(payload["request_turn_id"], f"Architecture.{variant}.request_turn_id")
        _nonempty(payload["response_turn_id"], f"Architecture.{variant}.response_turn_id")
    elif "route" in payload:
        _route(payload["route"], f"Architecture.{variant}.route")
        if variant == "RouteCommitted" and schema_version == CANONICAL_EVENT_VERSION:
            if payload["subgoal_id"] is not None:
                _nonempty(payload["subgoal_id"], f"Architecture.{variant}.subgoal_id")
        if variant == "RouteCandidateRejected":
            _nonempty(payload["reason"], f"Architecture.{variant}.reason")


def validate_model_call(value: Any) -> dict[str, Any]:
    obj = _object(value, "model call")
    _exact(obj, MODEL_CALL_KEYS, "model call")
    if obj["schema_version"] != MODEL_CALL_VERSION:
        raise StrictValidationError(f"model call: unsupported schema version {obj['schema_version']!r}")
    for key in ("model_call_id", "run_id", "episode_id", "scenario_id", "alternative_id", "decision_owner", "classification_reason"):
        _nonempty(obj[key], f"model call.{key}")
    _enum(obj["alternative_id"], ALTERNATIVES, "model call.alternative_id")
    _enum(obj["status"], MODEL_STATUSES, "model call.status")
    _enum(obj["call_class"], CALL_CLASSES, "model call.call_class")
    responsibilities = obj["semantic_responsibilities"]
    if not isinstance(responsibilities, list) or not responsibilities:
        raise StrictValidationError("model call.semantic_responsibilities: expected non-empty array")
    for responsibility in responsibilities:
        _enum(responsibility, RESPONSIBILITIES, "model call.semantic_responsibilities")
    for key in ("logical_sequence", "attempt", "logical_start", "completion"):
        if not isinstance(obj[key], int) or isinstance(obj[key], bool) or obj[key] < 0:
            raise StrictValidationError(f"model call.{key}: expected non-negative integer")
    if obj["first_output"] is not None and not isinstance(obj["first_output"], int):
        raise StrictValidationError("model call.first_output: expected integer or null")
    if obj["failure"] is not None and not isinstance(obj["failure"], int):
        raise StrictValidationError("model call.failure: expected integer or null")
    if obj["logical_start"] > obj["completion"] or (obj["first_output"] is not None and not obj["logical_start"] <= obj["first_output"] <= obj["completion"]) or (obj["failure"] is not None and not obj["logical_start"] <= obj["failure"] <= obj["completion"]):
        raise StrictValidationError("model call: invalid monotonic timing order")
    reference = obj["semantic_output_reference"]
    if reference is not None:
        reference = _object(reference, "model call.semantic_output_reference")
        _exact(reference, {"value_kind", "value"}, "model call.semantic_output_reference")
        _enum(reference["value_kind"], {"EXECUTOR_CANDIDATE"}, "model call.semantic_output_reference.value_kind")
        _nonempty(reference["value"], "model call.semantic_output_reference.value")
    return obj


def validate_fixture_event(value: Any) -> dict[str, Any]:
    obj = _object(value, "fixture event")
    _exact(obj, FIXTURE_EVENT_KEYS, "fixture event")
    for key in ("run_id", "episode_id", "fixture_kind", "action", "subject_id", "outcome"):
        _nonempty(obj[key], f"fixture event.{key}")
    if obj["observable_effect"] is not None:
        _effect(obj["observable_effect"], "fixture event.observable_effect")
    return obj


def validate_run_provenance(value: Any) -> dict[str, Any]:
    obj = _object(value, "run provenance")
    version = obj.get("provenance_schema_version")
    if version not in SUPPORTED_RUN_PROVENANCE_VERSIONS:
        raise StrictValidationError("run provenance: unsupported schema version")
    _exact(
        obj,
        RUN_KEYS if version == RUN_PROVENANCE_VERSION else LEGACY_RUN_KEYS,
        "run provenance",
    )
    if obj["canonical_event_schema_version"] not in SUPPORTED_CANONICAL_EVENT_VERSIONS or obj["model_call_schema_version"] != MODEL_CALL_VERSION:
        raise StrictValidationError("run provenance: stream schema version mismatch")
    _enum(obj["alternative"], ALTERNATIVES, "run provenance.alternative")
    for key in ("run_id", "source_git_commit", "pilot_corpus_id", "pilot_corpus_version", "scenario_id", "scenario_version", "semantic_behavior_plan_id", "semantic_behavior_plan_version", "rust_toolchain", "target", "build_profile", "runtime_worker_policy", "cargo_lock_identity"):
        _nonempty(obj[key], f"run provenance.{key}")
    profile = _object(obj["latency_profile"], "run provenance.latency_profile")
    _exact(profile, {"profile_id", "version", "model_delay_micros", "agent_delay_micros", "tool_delay_micros", "calibration_status"}, "run provenance.latency_profile")
    _enum(profile["profile_id"], {"Z", "C"}, "run provenance.latency_profile.profile_id")
    for key in ("model_delay_micros", "agent_delay_micros", "tool_delay_micros"):
        if not isinstance(profile[key], int) or isinstance(profile[key], bool) or profile[key] < 0:
            raise StrictValidationError(f"run provenance.latency_profile.{key}: expected non-negative integer")
    if obj["official"] and not obj["working_tree_clean"]:
        raise StrictValidationError("run provenance: official evidence has dirty source")
    if obj["official"] and (not obj["campaign_id"] or not isinstance(obj["campaign_profile_sequence_index"], int)):
        raise StrictValidationError("run provenance: official evidence lacks campaign correlation")
    if version == RUN_PROVENANCE_VERSION:
        if obj["source_sha"] != obj["source_git_commit"]:
            raise StrictValidationError("run provenance: source_sha/source_git_commit mismatch")
        calibration_keys = (
            "calibration_id", "cycle_id", "pair_id", "order_slot",
            "mode_order_slot", "repetition_id",
        )
        present = [obj[key] is not None for key in calibration_keys]
        if any(present) and not all(present):
            raise StrictValidationError("run provenance: incomplete paired calibration identity")
        for key in ("event_count", "attempted_event_count", "measurement_spine_event_count"):
            if not isinstance(obj[key], int) or isinstance(obj[key], bool) or obj[key] < 0:
                raise StrictValidationError(f"run provenance.{key}: expected non-negative integer")
        if obj["event_count"] < obj["measurement_spine_event_count"] or obj["attempted_event_count"] < obj["event_count"]:
            raise StrictValidationError("run provenance: invalid event retention counts")
        if obj["mode_order_slot"] is not None and obj["mode_order_slot"] not in {0, 1}:
            raise StrictValidationError("run provenance.mode_order_slot: expected 0 or 1")
        for key in ("calibration_id", "cycle_id", "pair_id", "repetition_id"):
            if obj[key] is not None:
                _nonempty(obj[key], f"run provenance.{key}")
    return obj


def validate_campaign_provenance(value: Any) -> dict[str, Any]:
    obj = _object(value, "campaign provenance")
    _exact(obj, CAMPAIGN_KEYS, "campaign provenance")
    if obj["provenance_schema_version"] != CAMPAIGN_PROVENANCE_VERSION:
        raise StrictValidationError("campaign provenance: unsupported schema version")
    if not obj["initial_working_tree_clean"]:
        raise StrictValidationError("campaign provenance: initial working tree was dirty")
    for key in ("campaign_id", "source_git_commit", "pilot_corpus_id", "pilot_corpus_version", "runner_version"):
        _nonempty(obj[key], f"campaign provenance.{key}")
    profiles = obj["profile_sequence"]
    if not isinstance(profiles, list) or len(profiles) != 2:
        raise StrictValidationError("campaign provenance: ordered profiles must be Z then C")
    for profile in profiles:
        _exact(_object(profile, "campaign provenance.profile_sequence"), {"profile_id", "version", "model_delay_micros", "agent_delay_micros", "tool_delay_micros", "calibration_status"}, "campaign provenance.profile_sequence")
    if [p["profile_id"] for p in profiles] != ["Z", "C"]:
        raise StrictValidationError("campaign provenance: ordered profiles must be Z then C")
    config = _object(obj["campaign_configuration"], "campaign provenance.campaign_configuration")
    _exact(config, {"alternatives", "scenario_ids", "warmup_count", "measured_repetition_count", "order_policy", "instrumentation_mode"}, "campaign provenance.campaign_configuration")
    return obj


def cross_stream_issues(provenance: dict[str, Any], events: Iterable[dict[str, Any]], calls: Iterable[dict[str, Any]], fixtures: Iterable[dict[str, Any]]) -> list[ValidationIssue]:
    events, calls, fixtures = list(events), list(calls), list(fixtures)
    issues: list[ValidationIssue] = []
    identity = (provenance["run_id"], provenance["scenario_id"], provenance["alternative"])
    episode_ids: set[str] = set()
    for stream, rows in (("canonical", events), ("model", calls), ("fixture", fixtures)):
        for row in rows:
            row_identity = (row["run_id"], identity[1], identity[2]) if stream == "fixture" else (row["run_id"], row["scenario_id"], row["alternative_id"])
            if row_identity != identity:
                issues.append(ValidationIssue("CROSS_STREAM_IDENTITY_MISMATCH", f"{stream} identity {row_identity} != {identity}"))
            episode_ids.add(row["episode_id"])
    for event in events:
        if event["source_git_commit"] != provenance["source_git_commit"] or event["scenario_version"] != provenance["scenario_version"]:
            issues.append(ValidationIssue("EVENT_PROVENANCE_MISMATCH", event["event_id"]))
    if len(episode_ids) > 1:
        issues.append(ValidationIssue("EPISODE_ID_MISMATCH", f"multiple episode ids: {sorted(episode_ids)}"))
    sequences = [e["sequence_number"] for e in events]
    if len(sequences) != len(set(sequences)):
        issues.append(ValidationIssue("DUPLICATE_CANONICAL_SEQUENCE", "duplicate canonical sequence"))
    if sequences != sorted(sequences):
        issues.append(ValidationIssue("CANONICAL_SEQUENCE_REVERSAL", "canonical sequence is not ordered"))
    timestamps = [e["monotonic_timestamp"] for e in events]
    if timestamps != sorted(timestamps):
        issues.append(ValidationIssue("TIMESTAMP_REVERSAL", "canonical timestamps reverse"))
    variants = [_variant(e["event"]) for e in events]
    route_events = [e for e in events if _variant(e["event"]) == "RouteCommitted"]
    route_subgoals = [
        e["event"]["Architecture"]["RouteCommitted"].get("subgoal_id")
        for e in route_events
    ]
    non_null_subgoals = [value for value in route_subgoals if value is not None]
    if (
        len(non_null_subgoals) != len(set(non_null_subgoals))
        or (non_null_subgoals and len(non_null_subgoals) != len(route_subgoals))
        or (not non_null_subgoals and len(route_events) > 1)
    ):
        issues.append(ValidationIssue("DUPLICATE_INITIAL_ROUTE_COMMIT", "duplicate root or subgoal route commit"))
    terminals = [v for v in variants if v in {"EpisodeCompleted", "EpisodeFailed"}]
    if len(terminals) != 1:
        issues.append(ValidationIssue("MISSING_OR_CONFLICTING_TERMINAL", f"terminal count {len(terminals)}"))
    elif variants[-1] not in {"EpisodeCompleted", "EpisodeFailed"}:
        issues.append(ValidationIssue("TERMINAL_NOT_FINAL", "terminal event is not final"))
    if "EpisodeCompleted" in terminals and not route_events:
        issues.append(ValidationIssue("COMPLETED_WITHOUT_ROUTE_COMMIT", "completed episode requires a route commit"))
    if len(non_null_subgoals) > 1:
        last_commit = max(i for i, value in enumerate(variants) if value == "RouteCommitted")
        first_execution = next((i for i, value in enumerate(variants) if value == "ExecutionStarted"), None)
        if first_execution is not None and first_execution <= last_commit:
            issues.append(ValidationIssue("EXECUTION_BEFORE_INITIAL_ROUTE_PLAN_BARRIER", "compound execution started before all observed subgoal commits"))
    outcome_pos = next((i for i, v in enumerate(variants) if v == "UsefulOutcomeObserved"), None)
    execution_pos = next((i for i, v in enumerate(variants) if v == "ExecutionStarted"), None)
    if outcome_pos is not None and (execution_pos is None or outcome_pos < execution_pos):
        issues.append(ValidationIssue("OUTCOME_BEFORE_EXECUTION", "useful outcome precedes execution"))
    for event in events:
        if _variant(event["event"]) == "UsefulOutcomeObserved":
            payload = event["event"]["UsefulOutcomeObserved"]["effect"]
            if event["emitter"] != "OUTCOME_PROBE" or payload["authoritative_source"] != "OUTCOME_PROBE":
                issues.append(ValidationIssue("NON_AUTHORITATIVE_USEFUL_OUTCOME", event["event_id"]))
    result_events = [e for e in events if _variant(e["event"]) == "ResultBound"]
    invocation_ids = {e["product_correlation"]["execution_id"] for e in events if _variant(e["event"]) == "ExecutionStarted"}
    for result in result_events:
        c = result["product_correlation"]
        if not c["result_id"] or not c["task_id"] or not c["execution_id"] or c["execution_id"] not in invocation_ids:
            issues.append(ValidationIssue("INCONSISTENT_RESULT_BINDING", "result/task/execution correlation is incomplete or unknown"))
    return issues


def _variant(event: Any) -> str:
    if isinstance(event, str):
        return event
    key = next(iter(event))
    if key == "Architecture":
        inner = event[key]
        return inner if isinstance(inner, str) else next(iter(inner))
    return key
