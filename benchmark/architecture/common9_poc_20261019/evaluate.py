"""Frozen-population COMMON9 evaluator. Candidates never import this module.

Latency eligibility tests substantive output, independently of semantic truth.
Raw accepted/rejected callbacks, exact set judgments and causal paths are kept.
No missing observation is imputed as zero or a fabricated deadline penalty.
"""
import argparse
import json
import re
import statistics
from pathlib import Path
from causal_timing import build_graph

HERE = Path(__file__).resolve().parent
ORACLES = json.loads((HERE / "oracles.json").read_text())
CONFIGURATIONS = ("AAAA", "BAAA", "CAAA", "ABAA", "AABA", "AAAB", "AAAC")
FILTER_VERSION = "COMMON9-ENDPOINT-v1"
RSS_POLICY_VERSION = "COMMON9-RSS-v3-PREFORMAL"


def normalized(text):
    return re.sub(r"[\W_]+", "", str(text), flags=re.UNICODE).lower()


def substantive(data, utterance=""):
    """Pre-outcome endpoint rule; does not compare to expected answer content.

    A wrong concrete answer still qualifies. Phase/content_type labels alone
    cannot qualify an ack. Unresolved deictic-only promises do not qualify.
    A displayed resolved target card may establish the concrete referent.
    """
    text = str(data.get("text") or "").strip()
    compact = normalized(text)
    if not compact:
        return False, "EMPTY"
    if utterance and compact == normalized(utterance):
        return False, "RAW_INPUT_ECHO"
    acknowledgements = ("네", "예", "알겠습니다", "확인했습니다", "이해했습니다",
        "좋습니다", "잠시만요", "처리중입니다", "진행중입니다", "요청을확인했습니다",
        "요청을처리하겠습니다", "도와드리겠습니다", "답변하겠습니다",
        "지정한요청의실제내용을확인해답변하겠습니다", "곧답변드리겠습니다", "곧답변드릴게요",
        "okay", "ok", "sure", "understood", "gotit", "workingonit", "processingyourrequest",
        "iwilllookintothis", "illlookintothis", "好的", "明白了", "知道了", "正在处理中")
    if compact in acknowledgements or re.fullmatch(
            r"(?:네|예|알겠습니다|확인했습니다|잠시만기다려주세요|처리하겠습니다)+", compact):
        return False, "GENERIC_ACK_OR_PROMISE"
    if re.fullmatch(r"(?:네|예|알겠습니다)?(?:요청하신|지정한|말씀하신|해당|이)?"
                    r"(?:요청|작업|내용|업무)(?:을|를)?(?:확인|처리|진행|수행|검토)"
                    r"(?:하겠습니다|합니다|할게요|하고있습니다|중입니다)", compact):
        return False, "GENERIC_UNSPECIFIED_WORK_PROMISE"
    if re.fullmatch(r"(?:이|그|저|해당)(?:것|부분|문단|표|표들|자료|요청|결과)(?:을|를|은|는)?"
                    r"(?:확인|처리|수정|설명|보존|진행|전달)(?:하겠습니다|합니다|할게요)", compact):
        targets = data.get("targets") or []
        if not targets or not all(isinstance(t, str) and ":" in t for t in targets):
            return False, "UNRESOLVED_DEICTIC_PROMISE"
        return True, "CONCRETE_DISPLAYED_TARGET_CARD"
    # A proposition needs more than an orphan reference or punctuation. Exact
    # minimal facts/choices/questions remain eligible; no answer-key grading.
    if compact in ("이것", "그것", "저것", "이표", "저문단", "그결과", "처리중"):
        return False, "ORPHAN_REFERENCE"
    return True, "SUBSTANTIVE_TEXT_PROXY_SEMANTICS_SEPARATE"


def expression_match(actual, expected):
    actual = " ".join(str(actual).split())
    expected = " ".join(str(expected).split())
    return actual == expected or actual in [expected + suffix for suffix in
        ("은", "는", "을", "를", "만", "과", "와", "이", "가", "에서", "으로", "로", "에게")]


def _events(trace, kind):
    return sorted((e for e in trace.get("events", []) if e.get("kind") == kind),
                  key=lambda e: e.get("observed_ns", 0))


def memory_sample_state(sample, first_positive_by_pid=None):
    """Same pre-formal policy for every configuration, including exit races."""
    registered = sample.get("registered_processes", [])
    observed = sample.get("processes", [])
    required_pids = {r.get("pid") for r in registered}
    actual_pids = {r.get("pid") for r in observed}
    positive_pids = {r.get("pid") for r in observed
                    if isinstance(r.get("rss_bytes"), (int, float)) and r["rss_bytes"] > 0}
    base_valid = (isinstance(sample.get("concurrent_sum_bytes"), (int, float))
        and sample["concurrent_sum_bytes"] > 0
        and {"python_bridge", "rust_candidate"}.issubset({r.get("role") for r in registered})
        and not sample.get("error") and not sample.get("sample_error")
        and sample.get("ps_returncode", 0) == 0)
    lifecycle = sample.get("candidate_lifecycle", {})
    candidate = lifecycle.get("pid")
    boundary = (base_valid and lifecycle.get("alive_at_sample_start") is True
        and lifecycle.get("exit_confirmed_by_sample_end") is True
        and lifecycle.get("exitcode") == 0
        and candidate in sample.get("confirmed_exited_pids", [])
        and candidate in {r.get("pid") for r in registered if r.get("role") == "rust_candidate"}
        and actual_pids.issubset(required_pids)
        and (required_pids - positive_pids).issubset({candidate}))
    if first_positive_by_pid is not None and any(pid not in first_positive_by_pid for pid in required_pids):
        return "INVALID"
    if boundary:
        return "CONFIRMED_EXIT_BOUNDARY"
    if base_valid and sample.get("all_registered_observed") is True and required_pids == actual_pids == positive_pids:
        return "VALID"
    # A PID can be observed in the exec-start window with actual RSS0. This
    # exception needs the real Popen/registration bracket and later positive
    # evidence in this same episode; no delayed PID registration or fake wait.
    zero_pids = {r.get("pid") for r in observed if r.get("rss_bytes") == 0}
    metadata = sample.get("process_start_metadata", {})
    roles = {r.get("pid"):r.get("role") for r in registered}
    def genuine_start(pid):
        m = metadata.get(str(pid), {})
        times = [m.get(k) for k in ("spawn_begin_ns", "spawn_return_ns", "registered_at_ns")]
        positive_at = (first_positive_by_pid or {}).get(pid)
        return (roles.get(pid) != "python_bridge" and m.get("role") == roles.get(pid)
            and all(isinstance(at, int) for at in times)
            and times[0] <= times[1] <= times[2] <= sample.get("at_ns", -1)
            and positive_at is not None and positive_at > sample.get("at_ns", positive_at))
    if (base_valid and sample.get("all_registered_observed") is True
        and required_pids == actual_pids == positive_pids | zero_pids
        and zero_pids and all(genuine_start(pid) for pid in zero_pids)):
        return "STARTUP_ZERO_RSS"
    return "INVALID"


def semantic_diagnostics(trace, oracle):
    """Review ledger, deliberately not part of any QA numerator or latency.

    Literal field equality proves preservation only. It cannot establish that
    a purpose, Task association, prose answer or condition was understood.
    """
    adoptions = _events(trace, "SEMANTIC_ADOPTION")
    displays = _events(trace, "DISPLAY_PROXY")
    body, relations, handoffs = [], [], []
    for requirement in oracle["responses"]:
        eid = requirement["event_id"]
        actual = [e for e in displays if e.get("data", {}).get("stimulus_id") == eid]
        adopted = [e for e in adoptions if e.get("data", {}).get("stimulus_id") == eid]
        body.append({"response_id": requirement["response_id"], "event_id": eid,
            "expected_conditions": requirement["required_content"],
            "status": "NEEDS_REVIEW" if actual else "AUTO_VIOLATION",
            "reason": "semantic_body_conditions_require_independent_review" if actual else "no_actual_display",
            "actual_displays": [{"event_id": e["event_id"], "observed_ns": e.get("observed_ns"),
                "data": e.get("data")} for e in actual],
            "actual_adoptions": [{"event_id": e["event_id"], "data": e.get("data")} for e in adopted]})
    for goal in oracle["goals"]:
        members = goal["member_events"]
        actual = [e for e in adoptions if e.get("data", {}).get("stimulus_id") in members]
        relations.append({"diagnostic": "N-R", "goal": goal["id"],
            "expected_start_event": goal["start_event"], "expected_members": members,
            "expected_completion_conditions": goal["completion"],
            "status": "NEEDS_REVIEW" if actual else "AUTO_VIOLATION",
            "reason": "review_Task_association_revision_scope_and_control_targets" if actual else "no_user_semantic_adoption",
            "actual_relation_and_Task_decisions": [{"event_id": e["event_id"], "observed_ns": e.get("observed_ns"),
                "data": e.get("data")} for e in actual]})
    for event in trace.get("events", []):
        if event.get("kind") not in ("AGENT_INGRESS", "CONTROL_INGRESS"):
            continue
        data = event.get("data", {})
        request = data.get("request", {})
        # The v1 record lives directly under request. Unknown future formats
        # stay reviewable raw evidence instead of being normalized from oracle.
        mandatory = request.get("contract", {}).get("mandatory", {}) if isinstance(request, dict) else {}
        eid = request.get("stimulus_id") if isinstance(request, dict) else None
        adopted = [e for e in adoptions if e.get("data", {}).get("stimulus_id") == eid
                   and e.get("observed_ns", 0) <= event.get("observed_ns", 0)]
        source = adopted[-1].get("data", {}) if adopted else {}
        plan = source.get("plan", {})
        checks = {}
        if mandatory and source:
            for field in ("purpose", "constraints", "exclusions", "allowed_actions", "completed_actions", "artifact_type", "completion_condition"):
                if field in plan:
                    checks[field] = {"adopted": plan[field], "transferred": mandatory.get(field),
                                     "equal": plan[field] == mandatory.get(field)}
            checks["request_revision"] = {"adopted": source.get("revision"),
                "transferred": mandatory.get("request_revision"),
                "equal": source.get("revision") == mandatory.get("request_revision")}
        violation = any(not c["equal"] for c in checks.values())
        handoffs.append({"diagnostic": "N-H", "ingress_event_id": event["event_id"], "kind": event["kind"],
            "stimulus_id": eid, "actual_wire": data, "actual_adopted_plan": source,
            "preservation_status": "AUTO_VIOLATION" if violation else
                "AUTO_CONFIRMED_CONTRACT_FIELDS_ONLY" if checks else "NEEDS_REVIEW",
            "field_checks": checks, "semantic_status": "NEEDS_REVIEW",
            "semantic_review": "Compare actual recipient input/result with user purpose, constraints, exclusions and permissions; literal equality is not semantic correctness."})
    return {"body_conditions": body, "request_relations": relations, "handoff_conditions": handoffs,
        "scope": "Separate functional review ledger; no QA score or latency endpoint is derived from these semantic review statuses."}


def evaluate(trace, oracle=None):
    oracle = oracle if oracle is not None else ORACLES[trace["episode"]]
    provided = trace.get("provided_sources", {})
    captures = {e.get("data", {}).get("stimulus_id"): e.get("data", {})
                for e in _events(trace, "INPUT_CAPTURED")}
    callbacks = _events(trace, "DISPLAY_PROXY")
    qa01 = []
    for expected in oracle["responses"]:
        eid = expected["event_id"]
        source = provided.get(eid, {}).get("source_ns")
        candidates = [e for e in callbacks if e.get("data", {}).get("stimulus_id") == eid]
        audits, accepted = [], None
        for callback in candidates:
            capture = captures.get(eid, {})
            # Repeating an actually received Agent fact is a legitimate output;
            # echo rejection concerns the user's request, not raw source facts.
            utterance = capture.get("text", "") if capture.get("input_class") == "user" else ""
            yes, reason = substantive(callback.get("data", {}), utterance)
            if source is None:
                yes, reason = False, "SOURCE_NOT_PROVIDED"
            elif callback.get("observed_ns") is None or callback["observed_ns"] < source:
                yes, reason = False, "BEFORE_ACTUAL_SOURCE"
            audits.append({"event_id": callback.get("event_id"), "observed_ns": callback.get("observed_ns"),
                "text": callback.get("data", {}).get("text"), "eligible": yes, "reason": reason,
                "phase": callback.get("data", {}).get("phase")})
            if yes and accepted is None:
                accepted = callback
        # The catalog explicitly permits a newer completion to cover the same
        # background Task's progress response. Only actual Task metadata may
        # connect them; no phrase/ID-to-answer lookup in candidate execution.
        if accepted is None and source is not None and "더 최신 완료" in expected.get("required_content", ""):
            original_task = captures.get(eid, {}).get("task_id")
            if original_task:
                for callback in callbacks:
                    data = callback.get("data", {})
                    yes, reason = substantive(data)
                    if (yes and data.get("task_id") == original_task and
                        "완료" in data.get("text", "") and callback["observed_ns"] >= source):
                        accepted = callback
                        audits.append({"event_id": callback.get("event_id"), "observed_ns": callback["observed_ns"],
                            "text": data.get("text"), "eligible": True,
                            "reason": "ACTUAL_NEWER_SAME_TASK_COMPLETION_COVERS_PROGRESS"})
                        break
        qa01.append({**expected, "status": "OBSERVED" if accepted else "NOT_OBSERVED",
            "source_ns": source, "latency_ns": accepted["observed_ns"] - source if accepted else None,
            "accepted_callback_id": accepted.get("event_id") if accepted else None,
            "first_accepted_raw_text": accepted.get("data", {}).get("text") if accepted else None,
            "filter_version": FILTER_VERSION, "callback_audit": audits})
    adoptions = _events(trace, "SEMANTIC_ADOPTION")
    qa02 = []
    for expected in oracle["bindings"]:
        candidates = [e for e in adoptions if e.get("data", {}).get("stimulus_id") == expected["event_id"]]
        adoption = candidates[0] if candidates else None
        bindings = adoption.get("data", {}).get("plan", {}).get("bindings", []) if adoption else []
        matches = [b for b in bindings if expression_match(b.get("expression", ""), expected["expression"])]
        targets = matches[0].get("targets") if len(matches) == 1 else None
        valid = isinstance(targets, list) and all(isinstance(t, str) for t in targets)
        correct = valid and set(targets) == set(expected["expected_set"])
        qa02.append({**expected, "correct": bool(correct), "status": "ADOPTED" if adoption else "NOT_ADOPTED",
            "adoption_event_id": adoption.get("event_id") if adoption else None,
            "adopted_revision": adoption.get("data", {}).get("revision") if adoption else None,
            "matching_binding_count": len(matches), "actual_set": sorted(set(targets)) if valid else None,
            "actual_binding": matches[0] if len(matches) == 1 else matches,
            "duplicate_targets": bool(valid and len(targets) != len(set(targets))),
            "matching_rule": "exact_whitespace_or_one_trailing_Korean_case_particle"})
    graph = build_graph(trace)
    completions = _events(trace, "GOAL_COMPLETE")
    qa03 = []
    for expected in oracle["goals"]:
        root = provided.get(expected["start_event"], {}).get("source_ns")
        last = expected["member_events"][-1]
        final_source = provided.get(last, {}).get("source_ns")
        missing_sources = [eid for eid in expected["member_events"]
                           if provided.get(eid, {}).get("source_ns") is None]
        audits, accepted = [], None
        for event in completions:
            data = event.get("data", {})
            if data.get("origin_event_id") != expected["start_event"]:
                continue
            reason = None
            if data.get("stimulus_id") != last:
                reason = "PREMATURE_BEFORE_FINAL_REQUIRED_MEMBER"
            elif missing_sources:
                reason = "REQUIRED_SOURCE_NOT_PROVIDED"
            elif event.get("observed_ns", 0) < final_source:
                reason = "BEFORE_FINAL_REQUIRED_SOURCE"
            else:
                prior = [e for e in adoptions if e.get("data", {}).get("goal_id") == data.get("goal_id")
                         and e["observed_ns"] <= event["observed_ns"]]
                if prior and data.get("revision") != prior[-1]["data"].get("revision"):
                    reason = "STALE_GOAL_REVISION"
            audits.append({"event_id": event.get("event_id"), "reason": reason or "ELIGIBLE",
                "stimulus_id": data.get("stimulus_id"), "revision": data.get("revision")})
            if reason is None and accepted is None:
                accepted = event
        root_id = provided.get(expected["start_event"], {}).get("source_node")
        if accepted and not root_id:
            timing = {"status": "UNRESOLVED", "via_ns": None,
                "root_ns": root, "reason": "missing_original_source_node"}
        else:
            timing = graph.measure("point:" + accepted["event_id"], root, root_id) if accepted else {
                "status": "NOT_OBSERVED", "via_ns": None, "root_ns": root}
        qa03.append({**expected, **timing,
            "missing_required_member_sources": missing_sources,
            "accepted_terminal_event_id": accepted.get("event_id") if accepted else None,
            "terminal_audit": audits})
    calls = trace.get("calls", [])
    known = [c for c in calls if isinstance(c.get("cost_usd"), (int, float))]
    unknown = [c for c in calls if not isinstance(c.get("cost_usd"), (int, float))]
    roles = {}
    for call in calls:
        role = "LOCAL_MODEL" if call.get("evidence_kind") == "LOCAL_MODEL" else call.get("role", "VIA")
        row = roles.setdefault(role, {"calls": 0, "known_api_cost_usd": 0, "unknown_cost_calls": 0,
            "unknown_reservation_usd": 0, "known_input_tokens": 0, "known_output_tokens": 0,
            "unknown_token_usage_calls": 0, "initialization_ns": 0, "inference_ns": 0})
        row["calls"] += 1
        if isinstance(call.get("cost_usd"), (int, float)):
            row["known_api_cost_usd"] += call["cost_usd"]
        else:
            row["unknown_cost_calls"] += 1
            row["unknown_reservation_usd"] += call.get("reservation_usd", 0)
        if call.get("input_tokens") is None or call.get("output_tokens") is None:
            row["unknown_token_usage_calls"] += 1
        row["known_input_tokens"] += call.get("input_tokens") or 0
        row["known_output_tokens"] += call.get("output_tokens") or 0
        row["initialization_ns"] += call.get("initialization_ns") or 0
        row["inference_ns"] += call.get("inference_ns") or 0
    samples = trace.get("memory", [])
    first_positive = {}
    for sample in sorted(samples, key=lambda s:s.get("at_ns", 0)):
        if sample.get("ps_returncode", 0) != 0 or sample.get("error") or sample.get("sample_error"):
            continue
        for process in sample.get("processes", []):
            if isinstance(process.get("rss_bytes"), (int, float)) and process["rss_bytes"] > 0:
                first_positive.setdefault(process.get("pid"), sample.get("at_ns", 0))
    states = [memory_sample_state(s, first_positive) for s in samples]
    valid_samples = [s for s,state in zip(samples,states) if state in ("VALID", "STARTUP_ZERO_RSS")]
    invalid_samples = [s for s,state in zip(samples,states) if state == "INVALID"]
    boundary_samples = [s for s,state in zip(samples,states) if state == "CONFIRMED_EXIT_BOUNDARY"]
    startup_samples = [{"classification":"STARTUP_ZERO_RSS", "raw":s,
        "zero_pids":[p["pid"] for p in s.get("processes", []) if p.get("rss_bytes") == 0]}
        for s,state in zip(samples,states) if state == "STARTUP_ZERO_RSS"]
    sampler_failures = [e for e in trace.get("host_failures", []) if e.get("kind") == "MEMORY_SAMPLER_FAILED"]
    peak = max(valid_samples, key=lambda s: s["concurrent_sum_bytes"], default=None)
    failures = [e for e in trace.get("events", []) if e.get("kind") in
        ("WORKFLOW_FAILED", "SOURCE_PUBLICATION_REJECTED", "PUBLICATION_REJECTED", "PERMISSION_VIOLATION")]
    return {"version": "COMMON9-EVALUATOR-v1", "run_id": trace.get("run_id"),
        "episode": trace.get("episode"), "configuration": trace.get("configuration"),
        "rep": trace.get("rep"), "phase": trace.get("phase"),
        "observation": trace.get("observation"), "authored_time_scale": trace.get("authored_time_scale"),
        "qa01": qa01, "qa02": qa02, "qa03": qa03,
        "semantic_diagnostics": semantic_diagnostics(trace, oracle),
        "qa05": {"status": "COMPLETE_API_USAGE" if not unknown else "INCOMPLETE_COST",
            "known_api_cost_usd": sum(c["cost_usd"] for c in known),
            "unknown_call_count": len(unknown), "unknown_reservation_usd": sum(c.get("reservation_usd", 0) for c in unknown),
            "calls": len(calls), "roles": roles,
            "local_helper_calls": sum(c.get("evidence_kind") == "LOCAL_MODEL" for c in calls),
            "receiver_calls": sum(c.get("role") == "RECEIVER" for c in calls),
            "local_compute_monetized": "NOT_MEASURED", "all_calls_including_unused_and_failed": True},
        "qa06": {"status": "INCOMPLETE" if peak and (invalid_samples or sampler_failures) else
                "OBSERVED" if peak else "INVALID" if samples or sampler_failures else "NOT_OBSERVED",
            "policy_version": RSS_POLICY_VERSION,
            "metric": "observed sampled maximum RSS; not guaranteed continuous-time maximum",
            "nominal_sampling_interval_ms": trace.get("memory_interval_ms", 50),
            "peak_concurrent_sum_bytes": peak["concurrent_sum_bytes"] if peak else None,
            "peak_same_sample": peak, "sample_count": len(samples),
            "valid_sample_count": len(valid_samples), "invalid_sample_count": len(invalid_samples),
            "startup_zero_rss_count": len(startup_samples), "startup_zero_rss_samples": startup_samples,
            "first_positive_rss_observed_ns_by_pid": first_positive,
            "confirmed_exit_boundary_count": len(boundary_samples), "confirmed_exit_boundary_samples": boundary_samples,
            "invalid_samples": invalid_samples, "sampler_failures": sampler_failures,
            "peak_is_observed_lower_bound_if_incomplete": bool(invalid_samples or sampler_failures),
            "incomplete_registered_process_samples": sum(not s.get("all_registered_observed", False) for s in invalid_samples),
            "raw_incomplete_registered_process_samples": sum(not s.get("all_registered_observed", False) for s in samples),
            "max_sample_gap_ns": max((b["at_ns"] - a["at_ns"] for a,b in zip(samples, samples[1:])), default=None)},
        "functional_failures": failures, "host_failures": trace.get("host_failures", []),
        "returncode": trace.get("returncode")}


def regression(record, oracle=None):
    """Deterministic OFFLINE common9 regression gate, not model quality proof.

    One episode predicate. Caller aggregates all nine; timing values have no
    performance meaning when authored_time_scale !=1 or semantic doubles run.
    """
    result = evaluate(record, oracle)
    checks = {
        "all_required_substantive_callbacks": all(r["status"] == "OBSERVED" for r in result["qa01"]),
        "all_final_goals_observed": all(r["status"] == "OBSERVED" for r in result["qa03"]),
        "all_exact_referent_sets": all(r["correct"] for r in result["qa02"]),
        "no_automatic_handoff_field_violations": all(h["preservation_status"] != "AUTO_VIOLATION"
            for h in result["semantic_diagnostics"]["handoff_conditions"]),
        "no_recorded_functional_failures": not result["functional_failures"],
        "no_host_failures": not result["host_failures"], "process_exit_ok": record.get("returncode") == 0}
    return {"status": "PASS" if all(checks.values()) else "FAIL", "checks": checks,
        "counts": {"responses_planned": len(result["qa01"]),
            "responses_observed": sum(r["status"] == "OBSERVED" for r in result["qa01"]),
            "goals_planned": len(result["qa03"]), "goals_observed": sum(r["status"] == "OBSERVED" for r in result["qa03"]),
            "bindings_planned": len(result["qa02"]), "bindings_correct": sum(r["correct"] for r in result["qa02"])},
        "evaluation": result, "scope": "offline actual Rust+protocol regression; model semantic accuracy NOT_MEASURED"}


def summarize(results, repetitions=5):
    """Same fixed planned denominators even when traces are absent or fail."""
    groups = {}
    for phase in sorted({r.get("phase") for r in results}, key=str):
        for config in CONFIGURATIONS:
            rows = [r for r in results if r.get("phase") == phase and r.get("configuration") == config]
            identity = [(r.get("episode"), r.get("rep")) for r in rows]
            duplicate = len(identity) != len(set(identity))
            expected_ids = {(s, rep) for s in ORACLES for rep in range(1, repetitions + 1)}
            unexpected = sorted(set(identity) - expected_ids, key=str)
            valid_identity = not duplicate and not unexpected
            complete = set(identity) == expected_ids and not duplicate
            latencies = [x["latency_ns"] for r in rows for x in r["qa01"] if x["latency_ns"] is not None]
            durations = [x["via_ns"] for r in rows for x in r["qa03"] if x["via_ns"] is not None]
            bindings = [x for r in rows for x in r["qa02"]]
            peaks = [r["qa06"]["peak_concurrent_sum_bytes"] for r in rows if r["qa06"]["peak_concurrent_sum_bytes"] is not None]
            cost_known = sum(r["qa05"]["known_api_cost_usd"] for r in rows)
            unknown = sum(r["qa05"]["unknown_call_count"] for r in rows)
            groups[str(phase) + ":" + config] = {
                "episodes_observed": len(rows), "episodes_planned": 9 * repetitions,
                "population_complete": complete, "duplicate_episode_repetitions": duplicate,
                "unexpected_episode_repetitions": unexpected,
                "qa01": {"observed": len(latencies), "planned": 43 * repetitions,
                    "observed_partial_mean_ms": statistics.mean(latencies)/1e6 if latencies and valid_identity else None},
                "qa02": {"correct": sum(x["correct"] for x in bindings), "judgments_observed": len(bindings),
                    "planned": 15 * repetitions,
                    "fixed_denominator_accuracy": sum(x["correct"] for x in bindings)/(15 * repetitions) if valid_identity else None},
                "qa03": {"observed": len(durations), "planned": 23 * repetitions,
                    "observed_partial_mean_ms": statistics.mean(durations)/1e6 if durations and valid_identity else None},
                "qa05": {"known_total_usd": cost_known, "unknown_calls": unknown,
                    "unknown_reservation_usd": sum(r["qa05"]["unknown_reservation_usd"] for r in rows),
                    "mean_nine_episode_bundle_usd": cost_known/repetitions if complete and not unknown else None,
                    "status": "COMPLETE_API_USAGE" if complete and not unknown else "INCOMPLETE_COST_OR_POPULATION"},
                "qa06": {"maximum_same_sample_rss_bytes": max(peaks) if peaks else None,
                    "status": "COMPLETE_POPULATION" if complete and all(r["qa06"]["status"] == "OBSERVED" for r in rows)
                        else "INCOMPLETE_OBSERVATION_OR_POPULATION",
                    "invalid_samples": sum(r["qa06"]["invalid_sample_count"] for r in rows),
                    "confirmed_exit_boundary_samples": sum(r["qa06"]["confirmed_exit_boundary_count"] for r in rows),
                    "startup_zero_rss_samples": sum(r["qa06"]["startup_zero_rss_count"] for r in rows),
                    "sampler_failures": sum(len(r["qa06"]["sampler_failures"]) for r in rows),
                    "incomplete_samples": sum(r["qa06"]["incomplete_registered_process_samples"] for r in rows)}}
    return groups


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path, help="single trace JSON or runs.jsonl")
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--repetitions", type=int, default=5)
    args = parser.parse_args()
    text = args.input.read_text()
    traces = [json.loads(line) for line in text.splitlines() if line.strip()] if args.input.suffix == ".jsonl" else [json.loads(text)]
    results = [evaluate(trace) for trace in traces]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({"results": results, "summary": summarize(results, args.repetitions)}, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
