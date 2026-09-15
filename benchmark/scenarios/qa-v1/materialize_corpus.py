#!/usr/bin/env python3
"""Materialize the frozen QA-v1 corpus without network access or randomness."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


GENERATION_RULE_VERSION = "qa-v1-materializer-1.0.0"


def encoded(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode()


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def write_json(root: Path, relative: str, value: Any) -> str:
    payload = encoded(value)
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(payload)
    return digest_bytes(payload)


def slug(value: str) -> str:
    return value.lower().replace(" / ", "-").replace("/", "-").replace(" ", "-").replace("_", "-")


def common_case(case_id: str, family_id: str, index: int) -> dict[str, Any]:
    return {
        "case_id": case_id,
        "family_id": family_id,
        "variation_id": f"V{index + 1:02d}",
        "generation_rule_version": GENERATION_RULE_VERSION,
        "semantic_reason": f"Frozen semantic variation {index + 1} for {family_id}; no wording-only mutation.",
    }


GOAL_FAMILIES = {
    "fast_bounded": [
        ("device-status-read", "Read a current device state without mutation"),
        ("current-time-read", "Resolve a locale-aware current time fact"),
        ("battery-state-read", "Read battery charge and charging state"),
        ("network-status-read", "Read active network type and reachability"),
        ("media-state-read", "Read current media identity and playback state"),
        ("calendar-next-event-read", "Read the next locally available event"),
        ("file-metadata-read", "Read metadata for a selected file"),
        ("cached-weather-read", "Read a frozen cached weather observation"),
        ("task-status-read", "Read status of an existing UserTask"),
        ("screen-selection-read", "Read facts about a bounded selected screen object"),
    ],
    "short_general_agent": [
        ("memory-store", "Store an approved preference with provenance and expiry"),
        ("memory-retrieve-and-use", "Use an applicable stored preference in a later goal"),
        ("memory-modify", "Modify an existing memory and use only the new version"),
        ("memory-delete", "Delete a memory and prevent later retrieval"),
        ("expired-memory-not-used", "Ignore an expired memory during a relevant goal"),
        ("conflicting-memory-resolution", "Resolve conflicting memories by provenance and version"),
        ("memory-provenance-consent", "Respect memory provenance and consent boundaries"),
        ("browser-context-use", "Use authorized browser context for a bounded request"),
        ("context-resolvable-request", "Resolve an omitted value from sufficient current context"),
        ("clarification-required-request", "Request clarification only when context is insufficient"),
    ],
    "specialist_agent": [
        ("network-diagnosis", "Delegate bounded network diagnosis to a specialist"),
        ("mail-search", "Search a frozen mailbox index"),
        ("file-search", "Search a frozen document index"),
        ("calendar-lookup", "Query a frozen calendar through a specialist contract"),
        ("media-catalog", "Query a frozen media catalog"),
        ("document-summary", "Summarize a frozen specialist-owned document"),
        ("contacts-lookup", "Resolve a contact from a frozen directory"),
        ("device-inventory", "Query a specialist device inventory"),
        ("task-history-lookup", "Query frozen task history"),
        ("photo-metadata", "Inspect metadata through a photo specialist"),
    ],
    "temporal_grounding_referent_correction": [
        ("pointer-source-destination", "Bind sequential source and destination pointer evidence"),
        ("pointer-group-destination", "Bind a pointed object group and later destination"),
        ("pointer-movement", "Resolve referents across meaningful pointer movement"),
        ("selection-focus-change", "Resolve evidence across selection and focus changes"),
        ("active-window-change", "Use the correct active-window version during the utterance"),
        ("transcript-revision-pointer", "Rebind pointer evidence after transcript revision"),
        ("source-content-update", "Use the correct source content version after an update"),
        ("ambiguous-source-destination", "Clarify genuinely ambiguous source or destination"),
        ("explicit-pointer-correction", "Apply an explicit correction to a pointed referent"),
        ("multi-referent-timeline", "Bind several referents to ordered temporal evidence"),
    ],
    "multi_turn_concurrent_approval": [
        ("voice-to-text-correction", "Preserve task identity from Voice request to Text correction"),
        ("text-to-voice-followup", "Preserve task identity from Text request to Voice follow-up"),
        ("voice-result-text-followup", "Bind a Text follow-up to a Voice-delivered result"),
        ("voice-interruption-text-continuation", "Continue the same task in Text after Voice interruption"),
        ("existing-task-followup", "Attach a follow-up to the correct existing UserTask"),
        ("concurrent-results", "Bind interleaved results to two concurrent tasks"),
        ("approval-required", "Obtain scoped approval before a state-changing step"),
        ("wrong-principal-defense", "Reject approval belonging to another principal"),
        ("cancel-inflight", "Cancel only the intended in-flight task"),
        ("exclusive-resource-conflict", "Serialize conflicting exclusive tasks without blocking a read-only task"),
    ],
    "compound_longer_agent": [
        ("independent-subgoals", "Decompose independent subgoals without false dependencies"),
        ("sequential-subgoals", "Preserve an explicitly ordered subgoal relationship"),
        ("data-dependent-subgoals", "Feed one subgoal result into the next subgoal"),
        ("conditional-subgoals", "Execute a subgoal only when its stated condition holds"),
        ("navigation-open", "Navigate to and open a constrained application or page"),
        ("search-candidate-selection", "Search, present candidates, and select the constrained result"),
        ("communication-send", "Send approved content to the correct recipient"),
        ("external-transaction-constraints", "Perform a constrained transaction without forbidden completion"),
        ("document-transformation", "Transform content into a required document result"),
        ("creative-ui-automation", "Complete a multi-step creative UI goal through an eligible Agent"),
    ],
}


UC_TITLES = {
    "UC-01": "Real-time Voice Interaction",
    "UC-02": "Screen-pointing Context Interaction",
    "UC-03": "Personal Context-aware Request",
    "UC-04": "Downstream Agent Delegation",
    "UC-05": "PC/Application/Service Action Request",
    "UC-06": "Sensitive Data/Action Approval",
    "UC-07": "Stateful / Long-running Task",
    "UC-08": "Immediate Interrupt / Cancel / Correction",
    "UC-09": "Session / Task Recovery",
    "UC-10": "Result / Progress Delivery",
    "UC-11": "Compound Utterance Handling",
    "UC-12": "Underspecified / Contextual Resolution",
    "UC-13": "Task Continuation & Follow-up",
    "UC-14": "Personalization & Memory Control",
    "UC-15": "Mixed-Modality Conversation",
    "UC-16": "Concurrent Task Handling",
}
UC_SOURCE_PATH = "docs/requirements/requirements-v1.1.md"


def use_case_provenance(tags: list[str], note: str) -> dict[str, Any]:
    return {
        "source_use_case": tags[0],
        "historical_source_path": UC_SOURCE_PATH,
        "derivation_note": note,
        "architecture_assumptions_removed": [
            "local Fast Path or mandatory delegation route",
            "named Context Engine, Intent Refiner, or Agent Router placement",
            "specific Agent product or internal tool-call orchestration",
        ],
    }


def goal_use_case_tags(stratum: str, family_slug: str) -> list[str]:
    tags: set[str] = {"UC-10"}
    if stratum == "fast_bounded":
        tags.add("UC-01")
        if family_slug in {"file-metadata-read", "calendar-next-event-read", "screen-selection-read"}:
            tags.add("UC-03")
        if family_slug in {"media-state-read", "network-status-read", "device-status-read"}:
            tags.add("UC-05")
    elif stratum == "short_general_agent":
        if family_slug.startswith("memory-") or family_slug in {"expired-memory-not-used", "conflicting-memory-resolution"}:
            tags.update({"UC-03", "UC-14"})
        if family_slug == "memory-provenance-consent":
            tags.add("UC-06")
        if family_slug == "browser-context-use":
            tags.update({"UC-03", "UC-12"})
        if family_slug in {"context-resolvable-request", "clarification-required-request"}:
            tags.add("UC-12")
    elif stratum == "specialist_agent":
        tags.update({"UC-04", "UC-05"})
        if family_slug in {"mail-search", "file-search", "calendar-lookup", "document-summary", "contacts-lookup", "photo-metadata"}:
            tags.add("UC-03")
    elif stratum == "temporal_grounding_referent_correction":
        tags.update({"UC-02", "UC-12"})
        if family_slug in {"transcript-revision-pointer", "explicit-pointer-correction"}:
            tags.add("UC-08")
    elif stratum == "multi_turn_concurrent_approval":
        tags.add("UC-13")
        if family_slug in {
            "voice-to-text-correction", "text-to-voice-followup",
            "voice-result-text-followup", "voice-interruption-text-continuation",
        }:
            tags.add("UC-15")
        if family_slug in {"voice-to-text-correction", "voice-interruption-text-continuation", "cancel-inflight"}:
            tags.add("UC-08")
        if family_slug in {"concurrent-results", "exclusive-resource-conflict", "cancel-inflight"}:
            tags.add("UC-16")
        if family_slug in {"approval-required", "wrong-principal-defense"}:
            tags.add("UC-06")
        if family_slug == "existing-task-followup":
            tags.add("UC-09")
    elif stratum == "compound_longer_agent":
        tags.update({"UC-04", "UC-05", "UC-07"})
        if family_slug in {"independent-subgoals", "sequential-subgoals", "data-dependent-subgoals", "conditional-subgoals"}:
            tags.add("UC-11")
        if family_slug in {"communication-send", "external-transaction-constraints"}:
            tags.add("UC-06")
    return sorted(tags)


def goal_temporal_events(stratum: str, family_slug: str, modality: str, variation: int) -> list[dict[str, Any]]:
    events = [
        {"event": "input_begin", "offset_ms": 0},
        {"event": "acoustic_eos" if modality == "voice" else "explicit_input_commit", "offset_ms": 600 + variation * 20},
    ]
    if stratum == "temporal_grounding_referent_correction":
        events[1:1] = [
            {"event": "pointer_or_selection_evidence", "role": "source", "object": f"source-{variation % 4}", "offset_ms": 120},
            {"event": "pointer_moved", "offset_ms": 260},
            {"event": "focus_or_window_version", "version": f"v{1 + variation % 3}", "offset_ms": 330},
            {"event": "pointer_or_selection_evidence", "role": "destination", "object": f"destination-{variation % 5}", "offset_ms": 440},
        ]
        if family_slug in {"transcript-revision-pointer", "explicit-pointer-correction"}:
            events.insert(-1, {"event": "transcript_or_referent_revision", "offset_ms": 510})
        if family_slug == "source-content-update":
            events.insert(-1, {"event": "source_content_version_updated", "version": f"v{2 + variation % 3}", "offset_ms": 500})
    return events


def goal_semantic_obligations(family_slug: str, goal_id: str, variation: int) -> dict[str, Any]:
    obligations: dict[str, Any] = {
        "architecture_route": "unconstrained",
        "candidate_may_use_any_contract_compliant_topology": True,
    }
    compound = {
        "independent-subgoals": "independent",
        "sequential-subgoals": "sequential",
        "data-dependent-subgoals": "data-dependent",
        "conditional-subgoals": "conditional",
    }
    if family_slug in compound:
        relation = compound[family_slug]
        obligations["compound_oracle"] = {
            "atomic_tasks": [f"task:{goal_id}:a", f"task:{goal_id}:b"],
            "dependency_relation": relation,
            "required_result_bindings": [
                [f"task:{goal_id}:a", f"result:{goal_id}:a"],
                [f"task:{goal_id}:b", f"result:{goal_id}:b"],
            ],
            "individual_action_occurrence_alone_is_insufficient": True,
        }
    if family_slug in {"context-resolvable-request", "clarification-required-request", "ambiguous-source-destination"}:
        omitted = ["app_or_source", "target", "parameter", "referent"][variation % 4]
        obligations["resolution_oracle"] = {
            "underspecification_kind": omitted,
            "context_sufficient": family_slug == "context-resolvable-request",
            "clarification_required": family_slug != "context-resolvable-request",
            "unnecessary_clarification_is_failure": True,
        }
    if family_slug.startswith("memory-") or family_slug in {"expired-memory-not-used", "conflicting-memory-resolution"}:
        operation = {
            "memory-store": "store", "memory-retrieve-and-use": "retrieve_and_use",
            "memory-modify": "modify", "memory-delete": "delete",
            "expired-memory-not-used": "expiry", "conflicting-memory-resolution": "conflict_resolution",
            "memory-provenance-consent": "provenance_consent",
        }[family_slug]
        obligations["memory_oracle"] = {
            "operation": operation,
            "memory_identity": f"memory:{goal_id}",
            "later_interaction_effect_required": operation in {"retrieve_and_use", "modify"},
            "subsequent_use_forbidden": operation in {"delete", "expiry"},
            "principal_and_task_isolation_required": True,
            "principal_fixture": f"principal-{1 + variation % 3}",
            "wrong_principal_probe": variation in {7, 8},
            "wrong_task_probe": variation == 9,
            "provenance_and_consent_required": True,
        }
    if family_slug == "exclusive-resource-conflict":
        obligations["resource_oracle"] = {
            "tasks": {
                "T1": {"resource": "mouse-keyboard", "mode": "exclusive"},
                "T2": {"resource": "mouse-keyboard", "mode": "exclusive"},
                "T3": {"resource": None, "mode": "read-only"},
            },
            "required": [
                "T1 and T2 do not overlap exclusive ownership",
                "T3 is not blocked by the T1/T2 conflict",
                "cancel affects only its intended task",
                "progress/result/approval binding remains exact",
            ],
            "agent_internal_tool_calls_are_not_via_scheduled": True,
        }
    if family_slug == "existing-task-followup":
        obligations["recovery_oracle"] = {
            "session_reconnect_variation": variation % 2 == 1,
            "same_logical_task_required": True,
            "duplicate_side_effect_forbidden": True,
        }
    action_domains = {
        "navigation-open": "navigation_open",
        "search-candidate-selection": "search_candidate_selection",
        "communication-send": "communication_send",
        "external-transaction-constraints": "external_transaction",
        "document-transformation": "document_transformation",
        "creative-ui-automation": "creative_ui_automation",
        "file-search": "local_file_content",
        "media-catalog": "media_system_control",
    }
    if family_slug in action_domains:
        obligations["action_domain"] = action_domains[family_slug]
        obligations["downstream_execution_boundary"] = (
            "Eligible downstream capability owns task reasoning and tool execution; "
            "the corpus does not prescribe a named Agent or VIA routing component."
        )
    return obligations


def make_goals() -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, list[str]]]:
    families: list[dict[str, Any]] = []
    goals: list[dict[str, Any]] = []
    memberships = {"QA-01": [], "QA-02": []}
    family_number = 0
    for stratum, entries in GOAL_FAMILIES.items():
        for family_slug, concern in entries:
            family_number += 1
            family_id = f"GF-{family_number:03d}-{family_slug}"
            use_case_tags = goal_use_case_tags(stratum, family_slug)
            families.append(
                {
                    "family_id": family_id,
                    "qa02_stratum": stratum,
                    "product_concern": concern,
                    "legacy_seed_usage": (
                        "E2 semantic seed: historical pilot/design-reference concept; no historical observation reused"
                        if family_slug in {
                            "battery-state-read",
                            "network-status-read",
                            "media-state-read",
                            "file-metadata-read",
                            "current-time-read",
                            "compare-local-options",
                            "summarize-recent-activity",
                            "file-search",
                            "mail-search",
                            "network-diagnosis",
                            "pointed-document",
                            "existing-task-followup",
                            "concurrent-results",
                            "approval-required",
                            "specialist-agent",
                        }
                        else "none"
                    ),
                    "variation_dimensions": [
                        "modality",
                        "fixture identity",
                        "context version",
                        "capability profile",
                        "result-fact cardinality",
                    ],
                    "use_case_tags": use_case_tags,
                    "use_case_provenance": use_case_provenance(
                        use_case_tags,
                        f"Derived from {', '.join(use_case_tags)} product semantics; only observable goal, context, consent, task, and delivery obligations retained.",
                    ),
                }
            )
            for variation in range(10):
                goal_id = f"GOAL-{family_number:03d}-{variation + 1:02d}"
                modality = "voice" if variation % 2 == 0 else "text"
                capability = (
                    "bounded-read"
                    if stratum == "fast_bounded"
                    else "specialist-v1"
                    if stratum == "specialist_agent"
                    else "general-v1"
                )
                agent_profile = (
                    "limited-session-v1"
                    if stratum == "fast_bounded"
                    else "specialist-v1"
                    if stratum == "specialist_agent"
                    else "general-v1"
                )
                membership = ["QA-02"]
                if stratum in {"fast_bounded", "short_general_agent", "specialist_agent"} and variation < 5:
                    membership.append("QA-01")
                    memberships["QA-01"].append(goal_id)
                memberships["QA-02"].append(goal_id)
                required_facts = [f"{goal_id}:fact:identity", f"{goal_id}:fact:state"]
                if variation % 3 == 2:
                    required_facts.append(f"{goal_id}:fact:version")
                goals.append(
                    {
                        "goal_id": goal_id,
                        "family_id": family_id,
                        "qa02_stratum": stratum,
                        "qa_population_memberships": membership,
                        "use_case_tags": use_case_tags,
                        "use_case_provenance": use_case_provenance(
                            use_case_tags,
                            f"Materialized from {family_id}; historical routing and component-placement language removed.",
                        ),
                        "modality": modality,
                        "user_input": {
                            "voice": f"Frozen voice request for {concern}, fixture {variation + 1}.",
                            "text": f"Frozen text request for {concern}, fixture {variation + 1}.",
                            "committed_representation": f"{family_slug}:fixture-{variation + 1}:v{1 + variation % 3}",
                        },
                        "temporal_interaction_events": goal_temporal_events(stratum, family_slug, modality, variation),
                        "context_fixture_references": [f"benchmark/fixtures/qa-v1/reference-fixtures-v1.json#/contexts/context-{variation + 1:02d}"],
                        "agent_capability_requirements": [capability],
                        "agent_profile_reference": f"benchmark/agent-stubs/qa-v1/capability-profiles-v1.json#/profiles/{ {'general-v1': 0, 'specialist-v1': 1, 'limited-session-v1': 2}[agent_profile] }",
                        "expected_task_semantics": {
                            "intent": family_slug,
                            "read_only": stratum == "fast_bounded",
                            "referent_version": f"v{1 + variation % 3}",
                            "constraints": [f"fixture={variation + 1}", f"result_complexity={1 + variation % 3}"],
                            "consent_required": stratum in {"multi_turn_concurrent_approval", "compound_longer_agent"} and variation % 2 == 1,
                            **goal_semantic_obligations(family_slug, goal_id, variation),
                        },
                        "required_result_facts": required_facts,
                        "required_voice_summary_facts": required_facts[:2],
                        "required_text_detail_fields": ["result.identity", "result.state"] + (["result.version"] if len(required_facts) == 3 else []),
                        "deadline_seconds": 8 if stratum in {"fast_bounded", "short_general_agent", "specialist_agent"} else 30,
                        "goal_oracle": {
                            "correct_goal": family_slug,
                            "correct_referent": f"fixture-{variation + 1}:v{1 + variation % 3}",
                            "explicit_constraints": [f"fixture={variation + 1}", f"result_complexity={1 + variation % 3}"],
                            "required_consent": stratum in {"multi_turn_concurrent_approval", "compound_longer_agent"} and variation % 2 == 1,
                            "task_result_binding": f"task:{goal_id}",
                            "required_result_facts": required_facts,
                            "voice_text_consistency_required": True,
                            "deadline_seconds": 8 if stratum in {"fast_bounded", "short_general_agent", "specialist_agent"} else 30,
                        },
                        "semantic_signature": f"{family_id}|fixture-{variation + 1}|{modality}|context-v{1 + variation % 3}|{capability}|facts-{len(required_facts)}",
                        "generation_rule_version": GENERATION_RULE_VERSION,
                    }
                )
    return families, goals, memberships


CONTINUITY_FAMILIES = [
    "existing-task-follow-up", "concurrent-task-interleaving", "voice-to-text-correction",
    "result-during-other-turn", "approval-correlation", "cancel-vs-complete-race",
    "response-interruption", "delayed-progress", "duplicate-progress", "late-agent-event",
    "delivery-continuation", "reconnect-mid-delivery", "revision-after-dispatch",
    "approval-revocation", "task-focus-switch", "exclusive-resource-contention",
    "memory-version-continuity", "queued-to-running-transition", "partial-delivery-resume",
    "follow-up-before-result",
]

INTERLEAVINGS = [
    "baseline-ordered", "result-before-progress", "duplicate-progress-before-result", "correction-before-dispatch",
    "correction-after-dispatch", "approval-before-progress", "approval-after-progress", "cancel-before-result",
    "cancel-concurrent-result", "result-during-other-response", "reconnect-before-result", "reconnect-after-result",
    "late-event-after-complete", "two-task-round-robin", "two-task-burst", "delivery-interrupted-once",
    "delivery-interrupted-twice", "focus-switch-before-result", "focus-switch-after-result", "revision-and-cancel",
]


def qa03() -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    families = []
    for i, name in enumerate(CONTINUITY_FAMILIES):
        tags = qa_family_tags("QA03", name)
        families.append({
            "family_id": f"Q03-F{i + 1:02d}",
            "name": name,
            "variation_dimension": "causally distinct event ordering",
            "use_case_tags": tags,
            "use_case_provenance": use_case_provenance(tags, f"Continuity obligations derived from {', '.join(tags)} without fixing component ownership."),
        })
    cases, oracles = [], {}
    for fi, family in enumerate(families):
        for vi, ordering in enumerate(INTERLEAVINGS):
            case_id = f"QA03-{fi + 1:02d}-{vi + 1:02d}"
            item = common_case(case_id, family["family_id"], vi)
            item.update({"event_interleaving": ordering, "actors": {
                "turn": f"turn:{case_id}", "task": f"task:{case_id}", "execution": f"execution:{case_id}",
                "result": f"result:{case_id}", "response": f"response:{case_id}", "delivery": f"delivery:{case_id}",
                "approval": f"approval:{case_id}", "cancel": f"cancel:{case_id}",
            }, "use_case_tags": family["use_case_tags"], "use_case_provenance": family["use_case_provenance"]})
            if family["name"] == "exclusive-resource-contention":
                item["resource_requirements"] = {
                    "T1": {"resource": "mouse-keyboard", "mode": "exclusive"},
                    "T2": {"resource": "mouse-keyboard", "mode": "exclusive"},
                    "T3": {"resource": None, "mode": "read-only"},
                    "agent_internal_tool_calls_via_orchestrated": False,
                }
            if family["name"] == "memory-version-continuity":
                item["memory_lifecycle"] = {
                    "stored_version": f"v{1 + vi % 3}",
                    "terminal_operation": ["modify", "delete", "expire", "conflict-resolve"][vi % 4],
                    "later_turn_must_use_current_valid_state": True,
                    "same_principal_and_task_binding_required": True,
                }
            cases.append(item)
            oracles[case_id] = {
                "required_relations": {
                    "turn_task": True, "task_execution": True, "execution_result": True,
                    "result_response": True, "response_delivery": True, "approval_task": True,
                    "cancel_task": True, "follow_up_task": True,
                },
                "forbidden_relation_patterns": ["cross-task-result-binding", "cross-principal-approval", "delivery-of-superseded-response"],
            }
            if family["name"] == "exclusive-resource-contention":
                oracles[case_id]["resource_invariants"] = [
                    "T1 and T2 exclusive ownership never overlaps",
                    "T3 is not blocked solely by T1/T2 contention",
                    "cancellation changes only the addressed task",
                    "progress/result/approval bindings remain exact",
                ]
    return families, cases, oracles


def generic_grid(prefix: str, names: list[str], variants: int, extra) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    families, cases, oracles = [], [], {}
    for fi, name in enumerate(names):
        family_id = f"{prefix}-F{fi + 1:02d}"
        tags = qa_family_tags(prefix, name)
        provenance = use_case_provenance(tags, f"Derived from {', '.join(tags)} product behavior; no candidate route or named component retained.")
        families.append({"family_id": family_id, "name": name, "use_case_tags": tags, "use_case_provenance": provenance})
        for vi in range(variants):
            case_id = f"{prefix}-{fi + 1:02d}-{vi + 1:02d}"
            case = common_case(case_id, family_id, vi)
            case["use_case_tags"] = tags
            case["use_case_provenance"] = provenance
            case_extra, oracle = extra(name, fi, vi, case_id)
            case.update(case_extra)
            cases.append(case)
            oracles[case_id] = oracle
    return families, cases, oracles


def qa_family_tags(prefix: str, name: str) -> list[str]:
    if prefix == "QA03":
        tags = {"UC-13"}
        if any(token in name for token in ("concurrent", "two-task", "exclusive-resource", "cancel")):
            tags.add("UC-16")
        if any(token in name for token in ("voice-to-text", "delivery-continuation", "partial-delivery")):
            tags.add("UC-15")
        if any(token in name for token in ("correction", "cancel", "interruption")):
            tags.add("UC-08")
        if any(token in name for token in ("reconnect", "stale", "late")):
            tags.add("UC-09")
        if "approval" in name:
            tags.add("UC-06")
        if "memory" in name:
            tags.add("UC-14")
        if any(token in name for token in ("delivery", "response", "result", "progress")):
            tags.add("UC-10")
        return sorted(tags)
    if prefix == "QA04":
        return ["UC-04", "UC-05"]
    if prefix == "QA08":
        tags = {"UC-09"}
        if name == "delivery-interruption":
            tags.update({"UC-08", "UC-10"})
        if name in {"duplicate-event", "late-event", "task-runtime-restart"}:
            tags.add("UC-07")
        return sorted(tags)
    if prefix == "QA09":
        tags = {"UC-03", "UC-06"}
        if name in {"user-memory", "approval-scope"}:
            tags.add("UC-14")
        if name in {"task-identity", "cross-device-scope"}:
            tags.update({"UC-13", "UC-16"})
        return sorted(tags)
    if prefix == "QA10":
        tags = {"UC-10"}
        if "memory" in name:
            tags.add("UC-14")
        if any(token in name for token in ("reconnect", "fault", "late", "duplicate")):
            tags.add("UC-09")
        if any(token in name for token in ("multi-turn", "correction")):
            tags.update({"UC-13", "UC-15"})
        if "concurrent" in name or "contention" in name:
            tags.add("UC-16")
        return sorted(tags)
    if prefix == "QA11":
        tags = {"UC-10"}
        if name == "approval_needed":
            tags.add("UC-06")
        if name in {"meaningful_progress", "completion", "blocked_failure"}:
            tags.add("UC-07")
        return sorted(tags)
    if prefix == "QA12":
        return ["UC-08"]
    return ["UC-05"]


def qa04():
    names = ["agent-onboarding", "agent-replacement", "session-contract", "progress-event", "result-schema", "approval-semantics", "cancel-behavior", "reconnect-resume", "protocol-version", "capability-extension-removal"]
    def extra(name, _fi, vi, _case_id):
        return ({
            "change_request": f"{name} variation {vi + 1} with contract profile cp-{vi + 1:02d}",
            "allowed_agent_integration_ownership_areas": ["Agent adapter", "protocol mapping", "capability manifest/extension", "approved Agent-integration seam"],
            "approved_extension_seams": [f"agent-integration:{name}", f"contract-profile:cp-{vi + 1:02d}"],
            "forbidden_core_semantic_zones": ["Z1", "Z2", "Z3", "Z5", "Z6", "Z7", "Z8"],
            "required_product_regressions": ["goal completion", "continuity", "scope boundary", "trace reconstruction"],
        }, {"pass_conditions": ["requested_change_completed", "common_regressions_pass", "contained_in_agent_integration"]})
    return generic_grid("QA04", names, 6, extra)


ZONE_CHANGES = {
    "Z1": ["S2S provider replacement", "ASR revision event", "turn-end detector", "barge-in playback controller", "voice locale negotiation", "audio route change", "partial transcript policy", "voice session handoff"],
    "Z2": ["context provider addition", "context-version contract", "evidence provenance field", "referent ranking policy", "context freshness rule", "cross-device evidence adapter", "screen-region evidence", "temporal cache invalidation"],
    "Z3": ["semantic decision schema", "constraint representation", "decision replay format", "capability selection rule", "clarification policy", "semantic error taxonomy", "decision version migration", "compound-goal planner"],
    "Z4": ["Agent protocol change", "Agent capability manifest", "progress mapping", "result mapping", "approval mapping", "session reconnect", "cancel mapping", "Agent replacement"],
    "Z5": ["Task persistence evolution", "execution retry record", "task-result binding version", "concurrent task index", "task resume token", "task cancellation state", "task history projection"],
    "Z6": ["response-plan format", "voice summary policy", "text detail projection", "delivery versioning", "feedback channel routing", "response interruption state", "fact consistency check"],
    "Z7": ["inference runtime change", "memory accounting provider", "buffer allocator", "model residency policy", "runtime contention telemetry", "cache sharing policy", "resource attribution event"],
    "Z8": ["new device capability", "mobile lifecycle adapter", "TV remote adapter", "robot sensor adapter", "paired display adapter", "device permission bridge", "platform context provider"],
}


def qa05():
    zone_tags = {
        "Z1": ["UC-01", "UC-08", "UC-15"], "Z2": ["UC-02", "UC-03", "UC-12"],
        "Z3": ["UC-04", "UC-11", "UC-12"], "Z4": ["UC-04", "UC-05"],
        "Z5": ["UC-07", "UC-09", "UC-13", "UC-16"], "Z6": ["UC-10", "UC-15"],
        "Z7": ["UC-01", "UC-07", "UC-16"], "Z8": ["UC-05", "UC-10", "UC-15"],
    }
    families = [{
        "family_id": f"QA05-{zone}", "zone": zone, "name": f"{zone} standard evolution",
        "use_case_tags": zone_tags[zone],
        "use_case_provenance": use_case_provenance(zone_tags[zone], f"Evolution coverage derived from product concerns associated with {zone}."),
    } for zone in ZONE_CHANGES]
    cases, oracles = [], {}
    for zone, names in ZONE_CHANGES.items():
        for vi, name in enumerate(names):
            case_id = f"QA05-{zone}-{vi + 1:02d}"
            item = common_case(case_id, f"QA05-{zone}", vi)
            item.update({
                "change_request": name,
                "expected_ownership_zones": [zone],
                "approved_extension_seams": [f"{zone.lower()}:{slug(name)}"],
                "required_regression_obligations": ["common goal contract", "continuity contract", "trace contract"],
                "use_case_tags": zone_tags[zone],
                "use_case_provenance": use_case_provenance(zone_tags[zone], f"Evolution case for {zone}; use-case tags do not prescribe its implementation."),
            })
            cases.append(item)
            oracles[case_id] = {"pass_conditions": ["requested_functionality_completed", "common_regressions_pass", "semantic_changes_authorized", "no_semantic_dependency_leak"]}
    return families, cases, oracles


DEVICE_REQUIREMENTS = {
    "mobile": [
        "touch-selected referent", "camera context", "coarse location", "foreground transition", "background task feedback",
        "battery state", "network transition", "active app context", "share-sheet input", "notification referent",
        "screen rotation", "microphone route", "paired wearable context", "permission revocation", "offline cached context",
        "mobile text detail", "mobile voice summary", "multi-window context", "camera permission scope", "handoff to paired display",
    ],
    "tv": [
        "remote directional input", "remote voice input", "large-screen text detail", "current media state", "playback position",
        "profile context", "multi-user approval", "limited text input", "focus navigation", "paired-phone text entry",
        "content catalog context", "HDMI source context", "ambient playback feedback", "screen-safe scope", "remote disconnect",
        "household principal", "media interruption", "subtitle state", "watchlist referent", "TV background task feedback",
    ],
    "robot": [
        "sensor context", "physical location", "camera environment", "mobility capability", "manipulator capability",
        "battery dock state", "safety stop state", "paired display output", "remote operator approval", "map referent",
        "obstacle context", "room identity", "payload state", "physical action scope", "sensor freshness",
        "navigation progress", "physical cancel", "camera egress scope", "capability degradation", "task result on paired display",
    ],
}


def qa06():
    tags = ["UC-05", "UC-10", "UC-15"]
    families = [{
        "family_id": f"QA06-{device.upper()}", "device_family": device,
        "profile_ref": f"benchmark/fixtures/qa-v1/reference-fixtures-v1.json#/device_profiles/{device}",
        "use_case_tags": tags,
        "use_case_provenance": use_case_provenance(tags, "Device-domain obligations retained without prescribing platform adapter placement."),
    } for device in DEVICE_REQUIREMENTS]
    cases, oracles = [], {}
    for device, requirements in DEVICE_REQUIREMENTS.items():
        for vi, requirement in enumerate(requirements):
            case_id = f"QA06-{device.upper()}-{vi + 1:02d}"
            item = common_case(case_id, f"QA06-{device.upper()}", vi)
            item.update({
                "device_family": device,
                "requirement": requirement,
                "required_functionality": f"Deliver {requirement} through the platform-independent VIA task/result contract",
                "allowed_device_adapters": [f"{device}-input-adapter", f"{device}-context-provider", f"{device}-delivery-adapter"],
                "forbidden_change": "semantic modification of platform-independent VIA Core contracts",
                "use_case_tags": tags,
                "use_case_provenance": use_case_provenance(tags, "Device requirement cell; adapter naming is illustrative and not an architecture oracle."),
            })
            cases.append(item)
            oracles[case_id] = {"pass_conditions": ["required_functionality_delivered", "core_contract_semantics_unchanged"]}
    return families, cases, oracles


def qa08():
    names = ["agent-disconnect", "agent-crash", "adapter-crash", "frontend-reconnect", "task-runtime-restart", "duplicate-event", "late-event", "partial-projection-loss", "context-broker-failure", "delivery-interruption"]
    lifecycle = ["accepted", "queued", "dispatched", "running", "progress", "approval-needed", "approved", "result-ready", "response-planned", "delivery-active"]
    def extra(name, _fi, vi, case_id):
        point = lifecycle[vi % len(lifecycle)]
        return ({
            "fault_type": name,
            "fault_timing": {"lifecycle_point": point, "offset_ms": 100 + vi * 137},
            "ground_truth_fault_onset_event": f"fault:{case_id}",
            "safe_expected_continuation_state": {"same_user_task": f"task:{case_id}", "state": "continuation-ready", "executor_reconciled": True},
            "forbidden_unsafe_retries": ["duplicate state-changing call", "reuse approval for another principal", "deliver stale result"],
            "allowed_recovery_mechanisms": ["idempotent event replay", "adapter reconnect", "projection rebuild", "safe task resume"],
        }, {"pass_conditions": ["same logical UserTask retained", "executor state reconciled", "no duplicate/unsafe side effect", "continuation-ready safe state", "user-visible task state restored"]})
    return generic_grid("QA08", names, 20, extra)


def qa09():
    names = ["document-scope", "file-scope", "screen-region", "user-memory", "task-identity", "sensor-class", "context-class", "approval-scope", "speculative-read", "agent-egress", "calendar-scope", "mail-scope", "location-scope", "camera-scope", "cross-device-scope"]
    def extra(name, fi, vi, case_id):
        required = [f"scope:{name}:object-{vi % 5}", f"scope:task:{case_id}"]
        forbidden = [f"scope:{name}:object-{(vi + 1) % 5}", "scope:unrelated-principal:*"]
        gate = ["explicitly forbidden disclosure"] if vi == 17 else ["unauthorized state-changing action"] if vi == 18 else ["wrong-principal approval use"] if vi == 19 else []
        return ({
            "principal": f"principal-{1 + vi % 4}",
            "task": f"task:{case_id}",
            "purpose": f"complete-{name}-goal",
            "scope_version": f"v{1 + vi % 3}",
            "expiry_event": f"task:{case_id}:terminal",
            "required_scopes": required,
            "forbidden_scopes": forbidden,
            "hard_gate_probe": gate,
            "linked_goal_obligation": f"legitimate goal requires {required[0]}; deny-all is a QA-02 failure",
        }, {"exact_required_scopes": required, "forbidden_scopes": forbidden, "hard_gate_triggers": gate})
    return generic_grid("QA09", names, 20, extra)


TRACE_FAMILIES = [
    "bounded-read-success", "general-agent-success", "specialist-success", "approval-success", "multi-turn-success",
    "concurrent-success", "correction-success", "reconnect-success", "memory-lifecycle-success", "compound-success",
    "agent-fault", "adapter-fault", "context-stale-edge", "approval-denied-edge", "cancel-race-edge",
    "delivery-interruption", "duplicate-event-edge", "late-result-edge", "resource-contention-edge", "semantic-error-edge",
]


def qa10():
    def extra(name, fi, vi, case_id):
        outcome = "success" if fi < 10 else "controlled_fault_or_edge"
        nodes = ["input", "evidence", "decision", "agent_selection", "execution", "task", "approval_progress_result", "response", "delivery", "attribution"]
        edges = [[nodes[i], nodes[i + 1]] for i in range(len(nodes) - 1)]
        return ({
            "outcome_class": outcome,
            "execution_fixture": f"trace-fixture:{name}:{vi + 1}",
            "candidate_telemetry_must_exclude": ["oracle_label", "injected_fault_label", "expected_graph"],
        }, {"required_chain": {node: True for node in nodes}, "expected_graph": {"nodes": nodes, "edges": edges}, "hidden_oracle_labels_exposed": False})
    return generic_grid("QA10", TRACE_FAMILIES, 10, extra)


def qa11():
    names = ["accepted_queued", "meaningful_progress", "approval_needed", "completion", "blocked_failure"]
    def extra(name, _fi, vi, case_id):
        if name == "approval_needed":
            channel = "voice_and_text"
        elif name in {"accepted_queued", "meaningful_progress"}:
            channel = "text_or_card"
        elif name == "completion" and vi % 2:
            channel = "notification_and_persisted_text"
        else:
            channel = "voice_and_text"
        return ({
            "event_class": name,
            "truthful_reportable_event": {"event_id": f"event:{case_id}", "available_offset_ms": 1000 + vi * 50},
            "required_channel": channel,
            "required_semantic_feedback": {
                "facts": [f"{name}:task-identity", f"{name}:state", f"{name}:version-{1 + vi % 3}"],
                "generic_ack_is_sufficient": False,
                "stale_repetition_is_sufficient": False,
            },
            "delivery_contract": {
                "voice": "concise key summary when the frozen channel includes voice",
                "text": "detailed persisted result when the frozen channel includes text",
                "every_progress_event_must_be_spoken": False,
            },
        }, {"useful_feedback_requires": ["correct task", "truthful state", "current version", f"channel:{channel}"]})
    return generic_grid("QA11", names, 40, extra)


def qa12():
    names = ["shallow-buffer", "deep-buffer", "short-speech", "long-speech", "tts-generating", "s2s-generating", "semantic-inference", "agent-progress-arrival", "three-active-tasks", "runtime-contention"]
    def extra(name, _fi, vi, case_id):
        onset = 1000 + vi * 25
        buffer_ms = [40, 80, 120, 180, 260][vi % 5]
        return ({
            "playback_condition": name,
            "playback_buffer_depth_ms": buffer_ms,
            "ground_truth_user_speech_onset_ms": onset,
            "previous_response_audible_timeline": [
                {"segment": "pre-barge", "start_ms": 0, "end_ms": onset},
                {"segment": "buffer-drain", "start_ms": onset, "end_ms": onset + buffer_ms},
            ],
            "required_stop_event": {"event_type": "AudioPlaybackStopped", "previous_response_id": f"response:{case_id}"},
            "concurrent_state": {"active_tasks": 1 + vi % 4, "semantic_inference": vi % 2 == 0, "agent_progress_arrival": vi % 3 == 0},
        }, {"measurement_start": "ground_truth_user_speech_onset_ms", "measurement_end": "last audible sample of previous response", "api_cancel_completion_is_not_end": True})
    return generic_grid("QA12", names, 20, extra)


def reference_fixtures() -> dict[str, Any]:
    contexts = {
        f"context-{i:02d}": {
            "fixture_id": f"context-{i:02d}",
            "version": f"v{1 + (i - 1) % 3}",
            "object_id": f"object-{i:02d}",
            "source": ["screen", "task", "notification", "device", "local-cache"][(i - 1) % 5],
            "observed_at_ms": 1_000_000 + i * 1_000,
        }
        for i in range(1, 11)
    }
    return {
        "fixture_set_id": "qa-v1-reference-fixtures-v1",
        "generation_rule_version": GENERATION_RULE_VERSION,
        "contexts": contexts,
        "device_profiles": {
            "mobile": {"inputs": ["voice", "text", "touch", "camera"], "lifecycle": ["foreground", "background"], "context": ["location", "app", "network", "battery"]},
            "tv": {"inputs": ["voice", "remote", "paired-phone"], "outputs": ["voice", "large-screen"], "context": ["media", "profile", "focus"]},
            "robot": {"inputs": ["voice", "remote-operator"], "outputs": ["voice", "paired-display"], "context": ["sensors", "map", "camera", "physical-capability"]},
        },
    }


def agent_profiles() -> dict[str, Any]:
    return {
        "profile_set_id": "qa-v1-agent-capability-profiles-v1",
        "profiles": [
            {"profile_id": "general-v1", "capabilities": ["bounded-read", "planning", "drafting"], "session": "resumable", "progress_contract": "standard", "result_contract": "standard"},
            {"profile_id": "specialist-v1", "capabilities": ["specialist-query", "specialist-result"], "session": "resumable", "progress_contract": "standard", "result_contract": "typed"},
            {"profile_id": "limited-session-v1", "capabilities": ["bounded-read"], "session": "single-turn", "progress_contract": "none", "result_contract": "standard"},
            {"profile_id": "no-resume-v1", "capabilities": ["planning", "state-changing-with-approval"], "session": "no-resume", "progress_contract": "standard", "result_contract": "standard"},
            {"profile_id": "alternate-contract-v1", "capabilities": ["specialist-query"], "session": "reconnectable", "progress_contract": "delta-v2", "result_contract": "typed-v2"},
        ],
        "claim_limit": "Deterministic contract profiles; they are not claims about named products.",
    }


def semantic_replay() -> dict[str, Any]:
    return {
        "replay_set_id": "qa-v1-semantic-replay-v1",
        "variants": [
            {"variant_id": "correct-v1", "decision": "fixture-defined-correct", "controlled_error": None},
            {"variant_id": "wrong-referent-v1", "decision": "fixture-defined-wrong-referent", "controlled_error": "wrong_referent"},
            {"variant_id": "missing-constraint-v1", "decision": "fixture-defined-missing-constraint", "controlled_error": "constraint_omission"},
            {"variant_id": "stale-evidence-v1", "decision": "fixture-defined-stale-evidence", "controlled_error": "stale_evidence"},
        ],
        "selection_rule": "The test fixture explicitly selects a variant; no probability or candidate-specific accuracy is injected.",
    }


def qa07_workload() -> dict[str, Any]:
    components = ["voice-input-model", "voice-output-model", "semantic-model", "agent-runtime", "context-evidence", "user-task-state", "response-delivery-buffers", "provisional-state"]
    phases = [
        ("P0", "idle baseline", ["voice-input-model", "voice-output-model"]),
        ("P1", "active Voice interaction", ["voice-input-model", "voice-output-model", "response-delivery-buffers"]),
        ("P2", "Voice plus semantic request processing", ["voice-input-model", "voice-output-model", "semantic-model", "context-evidence", "response-delivery-buffers"]),
        ("P3", "Voice plus one active Agent Task", ["voice-input-model", "voice-output-model", "semantic-model", "agent-runtime", "context-evidence", "user-task-state", "response-delivery-buffers"]),
        ("P4", "Voice plus three active UserTasks and progress/result", components[:-1]),
        ("P5", "response generation plus context/evidence overlap and optional provisional state", components),
    ]
    return {
        "workload_id": "qa07-resource-workload-v1",
        "population_id": "qa07-resource-v1",
        "population_kind": "fixed_workload_timeline",
        "case_count": None,
        "generation_rule_version": GENERATION_RULE_VERSION,
        "use_case_tags": ["UC-01", "UC-07", "UC-10", "UC-16"],
        "use_case_provenance": use_case_provenance(
            ["UC-01", "UC-07", "UC-10", "UC-16"],
            "Concurrent required-stack workload derived from product activity; it does not prescribe candidate process topology.",
        ),
        "timebase_ms": 100,
        "phases": [
            {
                "phase_id": phase_id,
                "name": name,
                "start_ms": index * 1000,
                "end_ms": (index + 1) * 1000,
                "required_resident_components": resident,
                "active_user_tasks": 3 if phase_id in {"P4", "P5"} else 1 if phase_id == "P3" else 0,
                "candidate_may_report_provisional_state": phase_id == "P5",
                "use_case_tags": ["UC-01", "UC-07", "UC-10", "UC-16"],
            }
            for index, (phase_id, name, resident) in enumerate(phases)
        ],
        "measurement_obligation": {
            "numerator": "candidate peak physically resident required AI-stack memory across all phases",
            "denominator": "frozen minimum mandatory AI-stack memory for the same workload",
            "shared_physical_pages": "count once",
            "true_replicas": "count separately",
            "cross_process_memory": "included",
        },
        "claim_limit": "Evidence-anchored deterministic workload definition; it contains no measured device memory.",
    }


def self_test_fixture() -> dict[str, Any]:
    observations = {
        "QA-01": [{"observation_id": "st-01", "start_seconds": 0.0, "voice_facts_audible_seconds": 1.0, "text_details_available_seconds": 1.2, "useful_outcome_correct": True}],
        "QA-02": [{"observation_id": "st-02", "required_conditions": {"goal": True, "referent": True, "constraints": True, "consent": True, "binding": True, "facts": True, "consistency": True, "deadline": True}}],
        "QA-03": [{"observation_id": "st-03", "required_relations": {"turn_task": True, "task_execution": True, "execution_result": True, "result_response": True, "response_delivery": True, "approval_task": True, "cancel_task": True, "follow_up_task": True}}],
        "QA-04": [{"observation_id": "st-04", "requested_change_completed": True, "common_regressions_pass": True, "actual_semantic_ownership_changes": ["Agent adapter"], "allowed_agent_integration_ownership_areas": ["Agent adapter", "protocol mapping"], "actual_extension_seams": ["agent-integration:self-test"], "approved_extension_seams": ["agent-integration:self-test"], "actual_core_semantic_zone_changes": [], "forbidden_core_semantic_zones": ["Z1", "Z2", "Z3", "Z5", "Z6", "Z7", "Z8"]}],
        "QA-05": [{"observation_id": "st-05", "requested_functionality_completed": True, "common_regressions_pass": True, "actual_changed_semantic_zones": ["Z2"], "expected_ownership_zones": ["Z2"], "actual_extension_seams": ["z2:self-test"], "approved_extension_seams": ["z2:self-test"], "semantic_dependency_leaks": []}],
        "QA-06": [{"observation_id": "st-06", "device_family": "mobile", "required_functionality_delivered": True, "actual_core_semantic_changes": []}],
        "QA-07": [{"observation_id": "st-07", "phase_id": "P5", "candidate_resident_bytes": 1100, "minimum_mandatory_bytes": 1000}],
        "QA-08": [{"observation_id": "st-08", "safe_recovery_reached": True, "safe_recovery_seconds": 2.0, "unsafe_or_incorrect_recovery": False}],
        "QA-09": [{"observation_id": "st-09", "required_scopes": ["a"], "granted_or_exposed_scopes": ["a"], "confirmed_hard_gate_triggers": []}],
        "QA-10": [{"observation_id": "st-10", "required_chain": {"input": True, "evidence": True, "decision": True, "agent_selection": True, "execution": True, "task": True, "approval_progress_result": True, "response": True, "delivery": True, "attribution": True}, "hidden_oracle_labels_exposed": False}],
        "QA-11": [{"observation_id": "st-11", "event_class": "completion", "event_available_seconds": 0.0, "feedback_received_seconds": 0.5, "useful_feedback": True}],
        "QA-12": [{"observation_id": "st-12", "user_speech_onset_ms": 1000, "last_audible_sample_ms": 1100}],
    }
    return {"fixture_id": "qa-v1-evaluator-self-test-v1", "evidence_mode": "semantic_replay", "observations": observations}


def population_paths(qa_id: str) -> str:
    number = int(qa_id.split("-")[1])
    if number == 4:
        return "benchmark/evolution/qa-v1/qa04-agent-evolution-v1"
    if number == 5:
        return "benchmark/evolution/qa-v1/qa05-product-evolution-v1"
    if number == 6:
        return "benchmark/evolution/qa-v1/qa06-device-adaptation-v1"
    if number == 8:
        return "benchmark/failure-injection/qa-v1/qa08-recovery-v1"
    population = {
        1: "qa01-fast-v1", 2: "qa02-goals-v1", 3: "qa03-continuity-v1", 7: "qa07-resource-v1",
        9: "qa09-sensitive-scope-v1", 10: "qa10-trace-v1", 11: "qa11-feedback-v1", 12: "qa12-barge-in-v1",
    }[number]
    return f"benchmark/scenarios/qa-v1/{population}"


def write_population(root: Path, qa_id: str, population_id: str, families: list[dict[str, Any]], cases: Any, oracles: Any, fixture_hash: str) -> dict[str, Any]:
    base = population_paths(qa_id)
    family_hash = write_json(root, f"{base}/families.json", {"population_id": population_id, "families": families})
    instance_hash = write_json(root, f"{base}/instances.json", {"population_id": population_id, "instances": cases})
    oracle_hash = write_json(root, f"{base}/oracles.json", {"population_id": population_id, "oracles": oracles})
    actual_size = len(cases)
    manifest = {
        "population_id": population_id,
        "qa_id": qa_id,
        "status": "frozen",
        "population_kind": "materialized_cases",
        "actual_size": actual_size,
        "semantic_family_count": len(families),
        "generation_rule_version": GENERATION_RULE_VERSION,
        "family_manifest_hash": family_hash,
        "materialized_instance_hash": instance_hash,
        "oracle_hash": oracle_hash,
        "reference_fixture_hash": fixture_hash,
        "artifacts": {
            "families": f"{base}/families.json",
            "instances": f"{base}/instances.json",
            "oracles": f"{base}/oracles.json",
        },
    }
    manifest_path = f"{base}/manifest.json"
    manifest_hash = write_json(root, manifest_path, manifest)
    return {"path": manifest_path, "hash": manifest_hash, "actual_size": actual_size, "family_count": len(families)}


UC_VARIATIONS = {
    "UC-01": ["voice/text modality", "bounded result obligations", "delivery endpoint", "runtime contention"],
    "UC-02": ["multiple referents", "pointer movement", "selection/focus/window change", "transcript revision", "content version", "correction"],
    "UC-03": ["file/mail/calendar/browser source", "scope", "principal", "context version"],
    "UC-04": ["capability profile", "health/session contract", "protocol evolution", "multi-capability goal"],
    "UC-05": ["navigation", "search/selection", "file/content", "communication", "media/system", "transaction", "transformation", "creative UI"],
    "UC-06": ["purpose", "scope", "principal", "approval timing", "consent", "forbidden disclosure"],
    "UC-07": ["lifecycle phase", "progress", "waiting/approval", "resume", "fault timing"],
    "UC-08": ["physical audio interruption", "semantic correction", "cancel timing", "continuation modality"],
    "UC-09": ["fault type", "lifecycle timing", "reconnect", "projection recovery", "duplicate/late event"],
    "UC-10": ["Voice summary", "Text detail", "card/notification channel", "event class", "inactive delivery"],
    "UC-11": ["independent", "sequential", "data-dependent", "conditional", "result binding"],
    "UC-12": ["omitted source", "omitted target", "omitted parameter", "ambiguity", "sufficient/insufficient context"],
    "UC-13": ["follow-up timing", "task focus", "cancel/resume", "same/new task", "result arrival"],
    "UC-14": ["store", "retrieve/use", "modify", "delete", "expiry", "conflict", "principal/task isolation", "consent/provenance"],
    "UC-15": ["Voice→Text correction", "Text→Voice follow-up", "Voice result→Text follow-up", "interruption→Text continuation"],
    "UC-16": ["exclusive/shared/no resource", "three active tasks", "cancel isolation", "unblocked read-only task", "binding under contention"],
}


def build_use_case_coverage(
    root: Path,
    goals: list[dict[str, Any]],
    memberships: dict[str, list[str]],
    pop_records: dict[str, dict[str, Any]],
    workload: dict[str, Any],
) -> dict[str, Any]:
    per_qa: dict[str, tuple[str, list[dict[str, Any]]]] = {}
    goal_by_id = {goal["goal_id"]: goal for goal in goals}
    for qa_id in ("QA-01", "QA-02"):
        per_qa[qa_id] = (
            "qa01-fast-v1" if qa_id == "QA-01" else "qa02-goals-v1",
            [goal_by_id[goal_id] for goal_id in memberships[qa_id]],
        )
    for qa_id, record in pop_records.items():
        if qa_id in {"QA-01", "QA-02", "QA-07"}:
            continue
        population_manifest = json.loads((root / record["path"]).read_text())
        instances = json.loads((root / population_manifest["artifacts"]["instances"]).read_text())["instances"]
        per_qa[qa_id] = (population_manifest["population_id"], instances)
    per_qa["QA-07"] = ("qa07-resource-v1", [{
        "case_id": workload["workload_id"],
        "family_id": "QA07-FIXED-TIMELINE",
        "use_case_tags": workload["use_case_tags"],
    }])

    strengthened = {"UC-02", "UC-05", "UC-11", "UC-12", "UC-14", "UC-15", "UC-16"}
    use_cases: dict[str, Any] = {}
    for uc_id, title in UC_TITLES.items():
        population_coverage = []
        all_families: set[str] = set()
        total = 0
        for qa_id, (population_id, cases) in sorted(per_qa.items()):
            matching = [case for case in cases if uc_id in case.get("use_case_tags", [])]
            if not matching:
                continue
            families = sorted({case["family_id"] for case in matching})
            all_families.update(families)
            count = None if qa_id == "QA-07" else len(matching)
            if count is not None:
                total += count
            population_coverage.append({
                "qa_id": qa_id,
                "population_id": population_id,
                "semantic_family_ids": families,
                "frozen_instance_count": count,
            })
        use_cases[uc_id] = {
            "title": title,
            "qa_populations": [entry["population_id"] for entry in population_coverage],
            "population_coverage": population_coverage,
            "semantic_family_ids": sorted(all_families),
            "semantic_family_count": len(all_families),
            "frozen_instance_count": total,
            "important_variation_dimensions": UC_VARIATIONS[uc_id],
            "coverage_status": "required_covered" if all_families else "excluded_with_reason",
            "strengthened_coverage_gate": uc_id in strengthened,
            "known_limitations": [
                "Qualification corpus coverage is synthetic/replayed and is not a production-frequency estimate.",
                "Use-case tags carry no candidate routing or component-placement requirement.",
            ],
        }
    return {
        "coverage_id": "via-uc01-uc16-qa-v1-coverage",
        "coverage_version": "1.0.0",
        "qa_contract_id": "qa-evaluation-contract-v1",
        "corpus_manifest_id": "qa-v1-corpus-manifest-v1",
        "source_requirements": UC_SOURCE_PATH,
        "architecture_neutrality_policy": {
            "use_cases_are": "product scenario and coverage inputs",
            "use_cases_are_not": "architecture answers",
            "forbidden_oracle_requirements": ["R1", "R3", "R1+@", "Fast Path", "Agent Router", "named Agent product"],
        },
        "use_cases": use_cases,
    }


def materialize(source_root: Path, output_root: Path) -> list[str]:
    written_before = {p.relative_to(output_root).as_posix() for p in output_root.rglob("*") if p.is_file()} if output_root.exists() else set()
    fixture_hash = write_json(output_root, "benchmark/fixtures/qa-v1/reference-fixtures-v1.json", reference_fixtures())
    write_json(output_root, "benchmark/fixtures/qa-v1/semantic-replay-v1.json", semantic_replay())
    write_json(output_root, "benchmark/fixtures/qa-v1/evaluator-self-test-v1.json", self_test_fixture())
    write_json(output_root, "benchmark/agent-stubs/qa-v1/capability-profiles-v1.json", agent_profiles())
    workload = qa07_workload()
    workload_hash = write_json(output_root, "benchmark/fixtures/qa-v1/resource/qa07-resource-workload-v1.json", workload)

    goal_families, goals, memberships = make_goals()
    family_catalog_hash = write_json(output_root, "benchmark/scenarios/qa-v1/goals/families/catalog.json", {"family_count": len(goal_families), "families": goal_families})
    goal_instances_hash = write_json(output_root, "benchmark/scenarios/qa-v1/goals/instances/catalog.json", {"goal_count": len(goals), "instances": goals})
    goal_catalog = {
        "catalog_id": "qa-v1-master-goal-catalog-v1",
        "generation_rule_version": GENERATION_RULE_VERSION,
        "family_count": len(goal_families),
        "goal_count": len(goals),
        "family_manifest_hash": family_catalog_hash,
        "materialized_instance_hash": goal_instances_hash,
        "population_memberships": {key: {"count": len(value), "goal_ids": value} for key, value in memberships.items()},
    }
    write_json(output_root, "benchmark/scenarios/qa-v1/goals/catalog.json", goal_catalog)

    pop_records: dict[str, dict[str, Any]] = {}
    for qa_id, goal_ids in (("QA-01", memberships["QA-01"]), ("QA-02", memberships["QA-02"])):
        pop_id = "qa01-fast-v1" if qa_id == "QA-01" else "qa02-goals-v1"
        family_ids = sorted({goal["family_id"] for goal in goals if goal["goal_id"] in set(goal_ids)})
        family_index = {family["family_id"]: index for index, family in enumerate(goal_families)}
        goal_index = {goal["goal_id"]: index for index, goal in enumerate(goals)}
        family_refs = [{
            "family_id": family_id,
            "catalog_ref": f"benchmark/scenarios/qa-v1/goals/families/catalog.json#/families/{family_index[family_id]}",
            "legacy_seed_usage": goal_families[family_index[family_id]]["legacy_seed_usage"],
            "use_case_tags": goal_families[family_index[family_id]]["use_case_tags"],
            "use_case_provenance": goal_families[family_index[family_id]]["use_case_provenance"],
        } for family_id in family_ids]
        case_refs = [{
            "goal_id": goal_id,
            "family_id": goals[goal_index[goal_id]]["family_id"],
            "catalog_ref": f"benchmark/scenarios/qa-v1/goals/instances/catalog.json#/instances/{goal_index[goal_id]}",
            "use_case_tags": goals[goal_index[goal_id]]["use_case_tags"],
            "use_case_provenance": goals[goal_index[goal_id]]["use_case_provenance"],
        } for goal_id in goal_ids]
        oracle_refs = {goal_id: f"benchmark/scenarios/qa-v1/goals/instances/catalog.json#/instances/{goal_index[goal_id]}/goal_oracle" for goal_id in goal_ids}
        pop_records[qa_id] = write_population(output_root, qa_id, pop_id, family_refs, case_refs, oracle_refs, fixture_hash)

    builders = {"QA-03": qa03, "QA-04": qa04, "QA-05": qa05, "QA-06": qa06, "QA-08": qa08, "QA-09": qa09, "QA-10": qa10, "QA-11": qa11, "QA-12": qa12}
    pop_ids = {
        "QA-03": "qa03-continuity-v1", "QA-04": "qa04-agent-evolution-v1", "QA-05": "qa05-product-evolution-v1",
        "QA-06": "qa06-device-adaptation-v1", "QA-08": "qa08-recovery-v1", "QA-09": "qa09-sensitive-scope-v1",
        "QA-10": "qa10-trace-v1", "QA-11": "qa11-feedback-v1", "QA-12": "qa12-barge-in-v1",
    }
    for qa_id, builder in builders.items():
        families, cases, oracles = builder()
        seed_ref = {
            "QA-03": "E2 pilot-v0/failure-injection semantic seed; no historical observation reused",
            "QA-04": "E2 qa03-evolution scenario seed; no historical observation reused",
            "QA-05": "E2 qa03-evolution scenario seed; no historical observation reused",
            "QA-06": "E2 qa03-evolution scenario seed; no historical observation reused",
            "QA-08": "E2 failure-injection concept seed; no historical observation reused",
            "QA-09": "E2 pilot-v0 scope concept seed; no historical observation reused",
            "QA-10": "E2 pilot-v0/failure-injection trace seed; no historical observation reused",
            "QA-11": "E2 pilot-v0/failure-injection feedback seed; no historical observation reused",
        }.get(qa_id, "none")
        for family in families:
            family["legacy_seed_usage"] = seed_ref
        pop_records[qa_id] = write_population(output_root, qa_id, pop_ids[qa_id], families, cases, oracles, fixture_hash)

    qa07_base = population_paths("QA-07")
    qa07_family_hash = write_json(output_root, f"{qa07_base}/families.json", {"population_id": "qa07-resource-v1", "families": [{
        "family_id": "QA07-FIXED-TIMELINE", "name": "fixed concurrent resource residency timeline",
        "use_case_tags": workload["use_case_tags"], "use_case_provenance": workload["use_case_provenance"],
    }]})
    qa07_oracle_hash = write_json(output_root, f"{qa07_base}/oracles.json", {"population_id": "qa07-resource-v1", "oracles": {"QA07-FIXED-TIMELINE": workload["measurement_obligation"]}})
    qa07_manifest = {
        "population_id": "qa07-resource-v1", "qa_id": "QA-07", "status": "frozen",
        "population_kind": "fixed_workload_timeline", "actual_size": None, "workload_identity": workload["workload_id"],
        "semantic_family_count": 1, "generation_rule_version": GENERATION_RULE_VERSION,
        "family_manifest_hash": qa07_family_hash, "materialized_instance_hash": workload_hash,
        "oracle_hash": qa07_oracle_hash, "reference_fixture_hash": fixture_hash,
        "artifacts": {"families": f"{qa07_base}/families.json", "instances": "benchmark/fixtures/qa-v1/resource/qa07-resource-workload-v1.json", "oracles": f"{qa07_base}/oracles.json"},
    }
    qa07_manifest_path = f"{qa07_base}/manifest.json"
    qa07_manifest_hash = write_json(output_root, qa07_manifest_path, qa07_manifest)
    pop_records["QA-07"] = {"path": qa07_manifest_path, "hash": qa07_manifest_hash, "actual_size": None, "family_count": 1}

    use_case_coverage = build_use_case_coverage(output_root, goals, memberships, pop_records, workload)
    use_case_coverage_hash = write_json(
        output_root,
        "benchmark/contracts/qa-v1/use-case-coverage-v1.json",
        use_case_coverage,
    )

    source_manifest_path = source_root / "benchmark/contracts/qa-v1/corpus-manifest-v1.json"
    source_manifest = json.loads(source_manifest_path.read_text())
    populations = []
    for source_population in source_manifest["populations"]:
        qa_id = source_population["qa_id"]
        record = pop_records[qa_id]
        population = {
            key: source_population[key]
            for key in ("population_id", "qa_id", "planned_size", "size_unit", "strata", "ownership_directory")
            if key in source_population
        }
        if "minimum_semantic_families" in source_population:
            population["minimum_semantic_families"] = source_population["minimum_semantic_families"]
        population.update({
            "status": "frozen", "actual_size": record["actual_size"], "semantic_family_count": record["family_count"],
            "population_manifest": record["path"], "population_manifest_sha256": record["hash"],
        })
        if qa_id == "QA-07":
            population["population_kind"] = "fixed_workload_timeline"
            population["workload_identity"] = "qa07-resource-workload-v1"
            population["planning_note"] = "Governance-finalized as a non-counted fixed workload timeline; no integer denominator applies."
        populations.append(population)
    frozen_manifest = {
        "manifest_id": source_manifest["manifest_id"], "manifest_version": "1.1.0",
        "status": "FROZEN_CORPUS", "empirical_evaluation_status": "NOT_EVALUATED",
        "qa_contract_id": source_manifest["qa_contract_id"], "reference_environment_id": "via-reference-environment-v1",
        "generation_rule_version": GENERATION_RULE_VERSION, "master_goal_catalog": "benchmark/scenarios/qa-v1/goals/catalog.json",
        "use_case_coverage": {
            "path": "benchmark/contracts/qa-v1/use-case-coverage-v1.json",
            "sha256": use_case_coverage_hash,
        },
        "populations": populations,
        "claim_limits": [
            "Frozen synthetic/replay populations are an architecture qualification instrument, not a production-frequency model.",
            "No DP alternative has been evaluated by materializing this corpus.",
            "Historical observations are not QA-v1 observations.",
        ],
    }
    write_json(output_root, "benchmark/contracts/qa-v1/corpus-manifest-v1.json", frozen_manifest)

    for population in populations:
        plan_path = source_root / population["ownership_directory"] / "population-plan.json"
        if not plan_path.exists():
            continue
        plan = json.loads(plan_path.read_text())
        plan["status"] = "frozen"
        plan["frozen_instance_count"] = population["actual_size"]
        plan["population_manifest"] = population["population_manifest"]
        write_json(output_root, f"{population['ownership_directory']}/population-plan.json", plan)

    written_after = {p.relative_to(output_root).as_posix() for p in output_root.rglob("*") if p.is_file()}
    authoritative_prefixes = (
        "benchmark/contracts/qa-v1/corpus-manifest-v1.json",
        "benchmark/fixtures/qa-v1/",
        "benchmark/agent-stubs/qa-v1/",
        "benchmark/scenarios/qa-v1/goals/",
        "benchmark/scenarios/qa-v1/qa",
        "benchmark/evolution/qa0",
        "benchmark/failure-injection/qa08-",
    )
    return sorted(path for path in written_after - written_before if path.startswith(authoritative_prefixes))


def main() -> int:
    parser = argparse.ArgumentParser()
    repo = Path(__file__).resolve().parents[3]
    parser.add_argument("--source-root", type=Path, default=repo)
    parser.add_argument("--output-root", type=Path, default=repo)
    args = parser.parse_args()
    materialize(args.source_root.resolve(), args.output_root.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
