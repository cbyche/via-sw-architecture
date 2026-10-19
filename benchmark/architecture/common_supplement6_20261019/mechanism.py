"""Observed activation diagnostics, never selection filters or oracle assistance.

Reports positive overlap from actual owned operations/call intervals. The current
runtime has no explicit worker-return event; gaps between operations cannot prove
that a whole workflow terminated and are labelled unresolved, not invented busy.
"""
import argparse
import json
from pathlib import Path


def overlap(a, b):
    return max(0, min(a[1], b[1]) - max(a[0], b[0]))


def operation_intervals(trace, sid):
    starts, spans, unresolved = {}, [], []
    for event in trace.get('events', []):
        if 'input:' + sid not in event.get('goal_ids', []):
            continue
        kind, data = event.get('kind'), event.get('data', {})
        node = data.get('node_id')
        if kind in ('OPERATION_START', 'LOCAL_OPERATION_START'):
            starts[node] = event
        elif kind in ('OPERATION_END', 'LOCAL_OPERATION_END'):
            start = starts.pop(node, None)
            if start and event['observed_ns'] >= start['observed_ns']:
                spans.append({'node_id': node, 'stage': data.get('stage'), 'start_ns': start['observed_ns'], 'end_ns': event['observed_ns'], 'role': data.get('role'), 'clock': 'ACTUAL_HOST_OBSERVED_OPERATION_BOUNDARIES'})
            else:
                unresolved.append({'node_id': node, 'reason': 'MISSING_OR_REVERSED_START'})
    unresolved.extend({'node_id': node, 'reason': 'NO_OBSERVED_OPERATION_TERMINAL'} for node in starts)
    return spans, unresolved


def call_interval(call):
    if not call.get('actual_api_sent'):
        return None
    start = call.get('http_dispatch_registered_ns')
    terminal = call.get('terminal_ns')
    if isinstance(start, int) and isinstance(terminal, int) and terminal >= start:
        return (start, terminal)
    return None


def extract(trace):
    episode = trace['episode']
    u1, u2 = episode + '-U01', episode + '-U02'
    events, calls = trace.get('events', []), trace.get('calls', [])
    captures = [e for e in events if e.get('kind') == 'INPUT_CAPTURED' and e.get('data', {}).get('stimulus_id') == u2]
    raw_capture_proxy = captures[0]['observed_ns'] if len(captures) == 1 else None
    commits = [e['data']['linearization_ns'] for e in events if e.get('kind') == 'STATE_EFFECT_COMMIT' and e.get('data', {}).get('effect_kind') == 'INPUT_CAPTURE' and e.get('data', {}).get('data', {}).get('stimulus_id') == u2]
    capture = commits[0] if len(commits) == 1 else raw_capture_proxy
    capture_clock = 'INPUT_CAPTURE_SEMANTIC_STATE_COMMIT_LINEARIZATION' if len(commits) == 1 else 'RAW_INPUT_CAPTURED_HOST_RECEIPT_PROXY_NOT_CAPTURE_COMMIT'
    spans1, missing1 = operation_intervals(trace, u1)
    spans2, missing2 = operation_intervals(trace, u2)
    pair_overlaps = [{'u1_node': a['node_id'], 'u2_node': b['node_id'], 'overlap_ns': overlap((a['start_ns'], a['end_ns']), (b['start_ns'], b['end_ns']))} for a in spans1 for b in spans2 if overlap((a['start_ns'], a['end_ns']), (b['start_ns'], b['end_ns'])) > 0 and a['role'] == 'VIA' and b['role'] == 'VIA']
    actual = [(c, call_interval(c)) for c in calls if c.get('status') == 'COMPLETED' and call_interval(c)]
    unproved_calls = [c['call_uuid'] for c in calls if c.get('actual_api_sent') and c.get('status') != 'COMPLETED']
    owned1 = [(c, span) for c, span in actual if 'input:' + u1 in c.get('causal_trace', {}).get('goal_ids', [])]
    owned2 = [(c, span) for c, span in actual if 'input:' + u2 in c.get('causal_trace', {}).get('goal_ids', [])]
    services = [{'u1_call_uuid': a['call_uuid'], 'u2_call_uuid': b['call_uuid'], 'overlap_ns': overlap(x, y)} for a, x in owned1 for b, y in owned2 if overlap(x, y) > 0]
    running = [c for c, span in actual if capture is not None and span[0] <= capture < span[1]]
    waiting = [c for c in calls if capture is not None and isinstance(c.get('before_slot_wait_ns'), int) and c['before_slot_wait_ns'] <= capture < c.get('slot_acquired_ns', c.get('terminal_ns', trace['end_ns']))]
    acquired = [c for c in calls if c.get('status') == 'COMPLETED' and capture is not None and isinstance(c.get('slot_acquired_ns'), int) and c['slot_acquired_ns'] <= capture < c.get('terminal_ns', trace['end_ns'])]
    unresolved_release = [c for c in calls if capture is not None and isinstance(c.get('slot_acquired_ns'), int) and c['slot_acquired_ns'] <= c.get('terminal_ns', trace['end_ns']) < capture and not any(n.get('resource_predecessor_call_uuid') == c.get('call_uuid') and isinstance(n.get('slot_acquired_ns'), int) and n['slot_acquired_ns'] <= capture for n in calls)]
    operation_active = [s for s in spans1 if capture is not None and s['start_ns'] <= capture < s['end_ns']]
    prior_final = [e for e in events if e.get('kind') == 'RESPONSE_FINAL' and e.get('data', {}).get('stimulus_id') == u1 and capture is not None and e['observed_ns'] <= capture]
    active = 'HOST_OPERATION_ENVELOPE_BRACKETS_CAPTURE_NOT_WORKER_LIFETIME_PROOF' if operation_active else 'FINAL_RESPONSE_ALREADY_OBSERVED_NOT_PROOF_OF_WORKER_RETURN' if prior_final else 'UNRESOLVED_BETWEEN_OPERATION_BOUNDARIES'
    adoptions = [e['data'] for e in events if e.get('kind') == 'SEMANTIC_ADOPTION' and e.get('data', {}).get('stimulus_id') in (u1, u2)]
    by_sid = {sid: [a for a in adoptions if a['stimulus_id'] == sid] for sid in (u1, u2)}
    correction = None
    if by_sid[u1] and by_sid[u2]:
        first, last = by_sid[u1][0], by_sid[u2][0]
        correction = {'same_goal': first['goal_id'] == last['goal_id'], 'first_revision': first['revision'], 'second_revision': last['revision'], 'second_relation': last['plan'].get('relation')}
    parser = [c for c in calls if c['stage'] == 'grounding.local_screen_parser']
    parser_admissions = [{'call_uuid': c['call_uuid'], 'status': c.get('status'), 'evidence_kind': c.get('evidence_kind'), 'local_spawn_admitted_ns': c.get('local_spawn_admitted_ns'), 'local_parse_dispatch_admitted_ns': c.get('local_parse_dispatch_admitted_ns'), 'cost_usd': c.get('cost_usd'), 'monetized_local_compute_cost_usd': c.get('monetized_local_compute_cost_usd'), 'admission_fields_present': all(key in c for key in ['local_spawn_admitted_ns', 'local_parse_dispatch_admitted_ns'])} for c in parser]
    native = [e for e in events if e.get('kind') == 'GROUNDING_NATIVE_CANDIDATES']
    has_real = trace.get('phase') != 'OFFLINE'
    return {'version': 'CONTRACT20_OBSERVED_ACTIVATION_v1', 'run_id': trace['run_id'], 'episode': episode, 'configuration': trace['configuration'], 'rep': trace['rep'], 'evidence_class': 'REAL_MODEL_TRACE' if has_real else 'OFFLINE_DOUBLE_DIAGNOSTIC_NOT_ACTIVATION_EVIDENCE', 'dp2': {'u2_capture_boundary_ns': capture, 'u2_capture_clock': capture_clock, 'u2_raw_INPUT_CAPTURED_host_observed_ns': raw_capture_proxy, 'u1_work_at_capture_status': active if capture is not None else 'INPUT_NOT_CAPTURED', 'u1_active_owned_operation_nodes_at_capture': [s['node_id'] for s in operation_active], 'u1_actual_model_interval_overlaps_capture': any(span[0] <= capture < span[1] for _, span in owned1) if capture is not None else None, 'common_k': 2, 'active_dispatched_cloud_calls_at_capture_lower_bound': len(running) if has_real and capture is not None else None, 'known_acquired_slots_at_capture_lower_bound': len(acquired) if has_real and capture is not None else None, 'noncompleted_acquired_records_unresolved': [c['call_uuid'] for c in calls if c.get('status') != 'COMPLETED' and isinstance(c.get('slot_acquired_ns'), int)], 'noncompleted_dispatched_interval_proofs_unresolved': unproved_calls, 'known_acquired_calls': [{'call_uuid': c['call_uuid'], 'resource_slot': c.get('resource_slot'), 'stage': c['stage']} for c in acquired], 'release_boundary_unresolved_calls': [c['call_uuid'] for c in unresolved_release], 'exact_free_K2_capacity': 'UNRESOLVED_NO_SLOT_RELEASE_CLOCK_IN_FROZEN_ABI', 'active_dispatched_slot_ids_at_capture': [c.get('resource_slot') for c in running], 'calls_at_capture': [{'call_uuid': c['call_uuid'], 'stage': c['stage'], 'role': c.get('role'), 'goal_ids': c.get('causal_trace', {}).get('goal_ids')} for c in running], 'queued_at_capture': [{'call_uuid': c['call_uuid'], 'stage': c['stage'], 'role': c.get('role')} for c in waiting], 'positive_owned_VIA_operation_overlaps': pair_overlaps, 'positive_actual_cloud_intervals': services, 'activation': 'PROVEN_ACTUAL_MODEL_OVERLAP' if has_real and services else 'MECHANISM_NOT_ACTIVATED_IN_OBSERVED_MODEL_INTERVALS' if has_real else 'NOT_MEASURED_OFFLINE', 'scheduler_A_boundaries': [e for e in events if e.get('kind') == 'SCHEDULER_A_BOUNDARY_ACCEPT'], 'adopted_goal_revision_relation': correction, 'stale_or_rejected_work': [e for e in events if e.get('kind') in ('WORKFLOW_FAILED', 'STALE_PUBLICATION_DISCARDED', 'STALE_HANDOFF_DISCARDED', 'MEANING_PROPOSAL_REJECTED')], 'unclosed_owned_operations': missing1 + missing2, 'limitations': 'Raw operation envelopes are host observation proxies, not exact worker lifetime clocks. No exact free K2 capacity is inferred without release timestamps. Only COMPLETED calls prove terminal was recorded within the owned slot; failed/cancelled outer terminal clocks remain unresolved. Dispatch-to-terminal cloud intervals include transport; no wire-service clock is claimed.'}, 'dp3': {'local_parser_stage_records': parser_admissions, 'actual_parse_dispatches': sum(isinstance(c.get('local_parse_dispatch_admitted_ns'), int) for c in parser) if has_real else None, 'actual_helper_spawns': sum(isinstance(c.get('local_spawn_admitted_ns'), int) for c in parser) if has_real else None, 'native_candidate_events': native, 'native_skip_observed': bool(native) and all(all(key in c for key in ['local_spawn_admitted_ns', 'local_parse_dispatch_admitted_ns']) for c in parser) and not any(isinstance(c.get('local_parse_dispatch_admitted_ns'), int) or isinstance(c.get('local_spawn_admitted_ns'), int) for c in parser) if has_real else None, 'all_grounding_stages': [{'stage': c['stage'], 'call_uuid': c['call_uuid'], 'status': c.get('status'), 'actual_api_sent': c.get('actual_api_sent'), 'cost_usd': c.get('cost_usd')} for c in calls if c['stage'].startswith('grounding.')], 'all_model_call_costs_retained': True}, 'not_a_population_filter': True}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('runs', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError('Preserve previous extraction')
    values = [extract(json.loads(line)) for line in args.runs.read_text().splitlines() if line.strip()]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({'version': 'CONTRACT20_OBSERVED_ACTIVATION_v1', 'traces': values}, ensure_ascii=False, indent=2) + '\n')
