"""Host observed endpoints and actual state-effect clocks are separate proofs."""
from .endpoints import integer
from . import publications


def audit(trace, mode='strict'):
    if mode not in ('strict', 'legacy_diagnostic'):
        raise ValueError('mode must be strict or legacy_diagnostic')
    cutoff = trace.get('observation_deadline_ns')
    issues, violations = [], []
    events = trace.get('events', [])
    proof = trace.get('host_clock_proof') or {}
    probe_events = [e for e in events if e.get('kind') == 'HOST_CLOCK_PROBE_ACK' and e.get('event_id') == proof.get('event_id')]
    clock_verified = False
    proof_observed = None
    if len(probe_events) == 1:
        event = probe_events[0]
        begin = trace.get('clock_probe_send', {}).get('send_begin_ns')
        probe = event.get('data', {}).get('host_monotonic_ns')
        emission, observed = event.get('host_monotonic_ns'), event.get('observed_ns')
        clock_verified = all(integer(v) for v in (begin, probe, emission, observed)) and begin <= probe <= emission <= observed
        proof_observed = observed if clock_verified else None
    if mode == 'strict' and not clock_verified:
        issues.append('HOST_CLOCK_DOMAIN_NOT_INDEPENDENTLY_VERIFIED')
    if not integer(cutoff):
        cutoff = None
        if mode == 'legacy_diagnostic':
            roots = [s.get('source_ns') for s in trace.get('provided_sources', {}).values() if integer(s.get('source_ns'))]
            if roots:
                cutoff = max(roots) + round(60 * trace.get('authored_time_scale', 1) * 1e9)
        issues.append('MISSING_ACTUAL_OBSERVATION_DEADLINE' if mode == 'strict' else 'DERIVED_LEGACY_CUTOFF_NO_CANCELLATION_COUNTERFACTUAL')
    technical_abort = bool(trace.get('technical_abort_reason') or trace.get('technical_failures'))
    if technical_abort:
        issues.append('TECHNICAL_ABORT_OR_FAILURE_NOT_CERTIFIED')
    effective_stop = trace.get('effective_admission_stop_ns')
    if not integer(effective_stop):
        effective_stop = cutoff
        if mode == 'strict':
            issues.append('MISSING_EFFECTIVE_ADMISSION_STOP')
    updates = trace.get('observation_deadline_updates', [])
    if mode == 'strict' and not updates:
        issues.append('MISSING_DEADLINE_UPDATE_LEDGER')
    previous = None
    accepted_acks = [e for e in events if e.get('kind') == 'OBSERVATION_DEADLINE_ACK' and e.get('data', {}).get('accepted') is True]
    first_arm = min((e['data'].get('update_admitted_ns', e['data'].get('host_monotonic_ns')) for e in accepted_acks if integer(e.get('data', {}).get('update_admitted_ns', e.get('data', {}).get('host_monotonic_ns')))), default=None)
    allowed_gates = {u.get('deadline_ns') for u in updates if integer(u.get('deadline_ns'))}
    for update in updates:
        at, last, end = (update.get(k) for k in ('at_ns', 'last_actual_source_ns', 'deadline_ns'))
        if not all(integer(v) for v in (at, last, end)):
            issues.append('MISSING_DEADLINE_UPDATE_CLOCK')
            continue
        if previous is not None and (at >= previous or end < previous):
            violations.append({'kind': 'DEADLINE_REVIVAL_OR_SHORTENING', 'update': update, 'previous_deadline_ns': previous})
        if end != last + round(60 * trace.get('authored_time_scale', 1) * 1e9):
            violations.append({'kind': 'DEADLINE_NOT_ACTUAL_SOURCE_PLUS_60_SECONDS', 'update': update})
        provisions = [s['provided_at_ns'] for s in trace.get('provided_sources', {}).values() if integer(s.get('provided_at_ns')) and s['provided_at_ns'] <= at]
        if not provisions or last != max(provisions):
            violations.append({'kind': 'DEADLINE_ROOT_NOT_LAST_ACTUAL_PROVISION', 'update': update, 'latest_actual_provision_ns': max(provisions, default=None)})
        acks = [e for e in accepted_acks if e.get('data', {}).get('control', {}).get('deadline_ns') == end and e.get('data', {}).get('control', {}).get('at_ns') == at]
        if mode == 'strict' and len(acks) != 1:
            issues.append('DEADLINE_UPDATE_ACK_MISSING_OR_AMBIGUOUS:' + str(end))
        previous = end
    if previous is not None and previous != cutoff:
        violations.append({'kind': 'FINAL_DEADLINE_DIFFERS_FROM_LEDGER', 'ledger': previous, 'cutoff': cutoff})
    source_checks = []
    for eid, source in trace.get('provided_sources', {}).items():
        times = [source.get(k) for k in ('source_ns', 'send_begin_ns', 'send_end_ns', 'provided_at_ns')]
        valid = all(integer(v) for v in times) and times[0] <= times[1] <= times[2] == times[3]
        source_checks.append({'event_id': eid, 'status': 'CONSISTENT' if valid else 'UNRESOLVED_OR_INCONSISTENT', 'clocks': dict(zip(('source_ns', 'send_begin_ns', 'send_end_ns', 'provided_at_ns'), times))})
        if not valid:
            if all(integer(v) for v in times):
                violations.append({'kind': 'SOURCE_SEND_CLOCK_INCONSISTENT', 'event_id': eid, 'clocks': times})
            else:
                issues.append('SOURCE_SEND_CLOCK_UNPROVED:' + eid)
    effects = []
    for event in trace.get('events', []):
        if event.get('kind') != 'STATE_EFFECT_COMMIT':
            continue
        data = event.get('data', {})
        at, gate = data.get('linearization_ns'), data.get('deadline_ns')
        prearm = gate is None and integer(at) and (integer(first_arm) and at < first_arm or
            first_arm is None and technical_abort and integer(effective_stop) and at < effective_stop)
        valid = integer(at) and (prearm or integer(gate) and gate in allowed_gates and at < gate) and integer(effective_stop) and at < effective_stop
        effects.append({'event_id': event.get('event_id'), 'effect_kind': data.get('effect_kind'), 'linearization_ns': at, 'deadline_ns': gate, 'status': 'UNARMED_BEFORE_TECHNICAL_ABORT' if valid and prearm and first_arm is None else 'PRE_ARM_WITHIN_FINAL_CUTOFF' if valid and prearm else 'WITHIN_GATE' if valid else 'INVALID_OR_LATE', 'data': data.get('data')})
        if not valid:
            violations.append({'kind': 'UNPROVED_OR_LATE_STATE_EFFECT', 'effect': effects[-1]})
    if mode == 'strict' and not effects:
        issues.append('MISSING_STATE_EFFECT_LEDGER')
    effect_coverage = []
    for event in events:
        kind, data = event.get('kind'), event.get('data', {})
        accepted_kinds = {'SEMANTIC_ADOPTION': ('SEMANTIC_ADOPTION',),
                          'AGENT_INGRESS': ('AGENT_INGRESS',), 'CONTROL_INGRESS': ('CONTROL_INGRESS',)}.get(kind)
        if not accepted_kinds:
            continue
        wire = data.get('request', data.get('envelope', {}).get('request', data)) if kind in ('AGENT_INGRESS', 'CONTROL_INGRESS') else data
        eid = wire.get('stimulus_id')
        emission = event.get('host_monotonic_ns')
        proofs = []
        for effect in effects:
            if effect['effect_kind'] not in accepted_kinds or effect['data'].get('stimulus_id') != eid or not integer(effect.get('linearization_ns')) or not integer(emission) or effect['linearization_ns'] > emission:
                continue
            if kind == 'DISPLAY_PROXY':
                phase = data.get('phase')
                if phase == 'ACTUAL_AGENT_FACT':
                    if effect['effect_kind'] != 'ACTUAL_AGENT_FACT':
                        continue
                elif effect['effect_kind'] != 'PUBLICATION' or effect['data'].get('phase') != phase:
                    continue
                if any(data.get(f) != effect['data'].get(f) for f in ('goal_id', 'revision') if f in data or f in effect['data']):
                    continue
            proofs.append(effect)
        # INPUT_CAPTURED is raw receipt instrumentation, emitted before the
        # semantic input state commit. It is deliberately outside this list.
        effect_coverage.append({'event_id': event.get('event_id'), 'kind': kind, 'stimulus_id': eid,
            'matching_commit_ids': [p['event_id'] for p in proofs], 'status': 'PROVED' if proofs else 'UNRESOLVED'})
        if mode == 'strict' and not proofs:
            issues.append('STATE_EFFECT_EVENT_WITHOUT_PRIOR_COMMIT:' + str(event.get('event_id')))
    cancel = trace.get('cancellation_requested_ns')
    settlement = trace.get('settlement_deadline_ns')
    if mode == 'strict' and not integer(cancel):
        issues.append('MISSING_ACTUAL_CANCELLATION_REQUEST')
    if integer(cutoff) and integer(cancel) and cancel < cutoff and not technical_abort:
        violations.append({'kind': 'CANCEL_BEFORE_REQUIRED_OBSERVATION_WINDOW', 'at_ns': cancel, 'cutoff_ns': cutoff})
    if mode == 'strict' and not integer(settlement):
        issues.append('MISSING_SETTLEMENT_DEADLINE')
    stop_bounds = [v for v in (cutoff, cancel) if integer(v)]
    pending_previous = [u.get('previous_deadline_ns') for u in updates if u.get('ack_status') not in (None, 'ACCEPTED') and integer(u.get('previous_deadline_ns'))]
    pending_at_cancel = trace.get('pending_deadline_update_cutoffs_at_cancel')
    if isinstance(pending_at_cancel, dict):
        pending_previous.extend(v for v in pending_at_cancel.values() if integer(v))
    elif isinstance(pending_at_cancel, list):
        pending_previous.extend(v for v in pending_at_cancel if integer(v))
    elif mode == 'strict':
        issues.append('PENDING_DEADLINE_CUTOFF_SNAPSHOT_MISSING')
    stop_bounds.extend(pending_previous)
    if stop_bounds and integer(effective_stop) and effective_stop != min(stop_bounds):
        violations.append({'kind': 'EFFECTIVE_STOP_NOT_EARLIEST_CUTOFF_OR_CANCELLATION', 'actual': effective_stop, 'bounds_ns': stop_bounds})
    if integer(effective_stop) and integer(settlement) and settlement != effective_stop + round(120 * trace.get('authored_time_scale', 1) * 1e9):
        violations.append({'kind': 'SETTLEMENT_DEADLINE_MISMATCH', 'actual': settlement, 'effective_stop_ns': effective_stop})
    aliases = ('contract_observation_deadline_ns', 'proposed_observation_deadline_ns', 'last_accepted_observation_deadline_ns', 'technical_abort_requested_ns', 'settlement_anchor_ns')
    if mode == 'strict':
        issues.extend('MISSING_ABORT_DEADLINE_ALIAS:' + key for key in aliases if key not in trace)
    if 'contract_observation_deadline_ns' in trace and trace['contract_observation_deadline_ns'] != cutoff:
        violations.append({'kind': 'CONTRACT_DEADLINE_OVERWRITTEN_OR_INCONSISTENT', 'record': trace['contract_observation_deadline_ns'], 'observation_deadline_ns': cutoff})
    if 'proposed_observation_deadline_ns' in trace and trace['proposed_observation_deadline_ns'] != (updates[-1].get('deadline_ns') if updates else None):
        violations.append({'kind': 'PROPOSED_DEADLINE_ALIAS_MISMATCH'})
    acknowledged = [u.get('deadline_ns') for u in updates if u.get('ack_status') in (None, 'ACCEPTED') and any(
        e.get('data', {}).get('control', {}).get('deadline_ns') == u.get('deadline_ns') and
        e.get('data', {}).get('control', {}).get('at_ns') == u.get('at_ns') and integer(e.get('observed_ns')) and
        (not integer(cancel) or e['observed_ns'] <= cancel) for e in accepted_acks)]
    last_accepted = acknowledged[-1] if acknowledged else None
    if 'last_accepted_observation_deadline_ns' in trace and trace['last_accepted_observation_deadline_ns'] != last_accepted:
        violations.append({'kind': 'LAST_ACCEPTED_DEADLINE_ALIAS_MISMATCH', 'actual': trace['last_accepted_observation_deadline_ns'], 'proved': last_accepted})
    if 'technical_abort_requested_ns' in trace and trace['technical_abort_requested_ns'] != (cancel if trace.get('technical_abort_reason') else None):
        violations.append({'kind': 'TECHNICAL_ABORT_REQUEST_CLOCK_MISMATCH'})
    if 'settlement_anchor_ns' in trace and trace['settlement_anchor_ns'] != effective_stop:
        violations.append({'kind': 'SETTLEMENT_ANCHOR_RESET_OR_MISMATCH', 'actual': trace['settlement_anchor_ns'], 'effective_stop_ns': effective_stop})
    if integer(trace.get('end_ns')) and integer(effective_stop) and trace['end_ns'] < effective_stop:
        violations.append({'kind': 'OBSERVATION_ENDED_BEFORE_REQUIRED_CUTOFF', 'end_ns': trace['end_ns'], 'cutoff_ns': cutoff})
    cancellation_acks = [e for e in events if e.get('kind') == 'WORK_CANCELLATION_ACK']
    if mode == 'strict' and not cancellation_acks:
        issues.append('MISSING_CANDIDATE_CANCELLATION_ACK')
    late_endpoints = [{'event_id': e.get('event_id'), 'kind': e.get('kind'), 'observed_ns': e.get('observed_ns')}
                      for e in trace.get('events', []) if e.get('kind') in ('DISPLAY_PROXY', 'GOAL_COMPLETE') and integer(cutoff) and integer(e.get('observed_ns')) and e['observed_ns'] > cutoff]
    dispatch_checks = []
    for call in trace.get('calls', []):
        # A reservation, queue entry or blocked request is not an actual
        # dispatch. In-flight service may finish and report cost after cutoff.
        if call.get('actual_api_sent') is not True:
            continue
        at = call.get('http_dispatch_registered_ns')
        valid = integer(at) and integer(effective_stop) and at < effective_stop and integer(proof_observed) and proof_observed <= at
        dispatch_checks.append({'call_uuid': call.get('call_uuid'), 'http_dispatch_registered_ns': at,
            'status': 'ADMITTED_BEFORE_CUTOFF' if valid else 'UNPROVED_OR_LATE_DISPATCH',
            'timestamp_semantics': call.get('http_dispatch_timestamp_semantics')})
        if not valid:
            (issues if not integer(at) else violations).append('UNPROVED_DISPATCH_CLOCK' if not integer(at) else {'kind': 'DISPATCH_OUTSIDE_VERIFIED_WINDOW', 'call_uuid': call.get('call_uuid'), 'at_ns': at, 'clock_proof_observed_ns': proof_observed})
    local_dispatch_checks = []
    for call in trace.get('calls', []):
        if call.get('evidence_kind') not in ('LOCAL_MODEL', 'LOCAL_MODEL_FAILED'):
            continue
        admissions = call.get('local_dispatch_admissions')
        if all(key in call for key in ('local_spawn_admitted_ns', 'local_parse_dispatch_admitted_ns')):
            admissions = [{'kind': kind, 'admitted_ns': call[key]} for key, kind in
                (('local_spawn_admitted_ns', 'PROCESS_SPAWN'), ('local_parse_dispatch_admitted_ns', 'PARSE_REQUEST')) if call[key] is not None]
        if not isinstance(admissions, list):
            issues.append('LOCAL_DISPATCH_ADMISSION_LEDGER_MISSING:' + str(call.get('call_uuid')))
            continue
        for admission in admissions:
            at = admission.get('admitted_ns')
            valid = integer(at) and integer(effective_stop) and at < effective_stop and integer(proof_observed) and proof_observed <= at
            item = {'call_uuid': call.get('call_uuid'), **admission, 'status': 'ADMITTED_AFTER_CLOCK_PROOF_BEFORE_CUTOFF' if valid else 'UNPROVED_OR_OUTSIDE_DISPATCH_WINDOW'}
            local_dispatch_checks.append(item)
            if not valid:
                violations.append({'kind': 'LOCAL_DISPATCH_OUTSIDE_VERIFIED_WINDOW', 'dispatch': item})
    publication_audit = publications.audit(trace, cutoff, effective_stop)
    if mode == 'strict':
        issues.extend(publication_audit['issues'])
        violations.extend(publication_audit['violations'])
    return {'version': 'COMMON9-DEADLINE-AUDIT-v3', 'mode': mode, 'cutoff_ns': cutoff,
        'status': 'VIOLATION' if violations else 'UNRESOLVED' if issues else 'CONSISTENT',
        'issues': sorted(set(issues)), 'violations': violations, 'clock_domain_verified': clock_verified, 'source_clock_checks': source_checks,
        'state_effect_checks': effects, 'effect_coverage': effect_coverage, 'dispatch_checks': dispatch_checks, 'local_dispatch_checks': local_dispatch_checks, 'late_endpoint_observations': late_endpoints,
        'publication_audit': publication_audit,
        'cancellation_requested_ns': cancel, 'settlement_deadline_ns': settlement,
        'effective_admission_stop_ns': effective_stop, 'technical_abort': technical_abort,
        'campaign_stop_required': technical_abort,
        'deadline_history': {'contract_deadline_ns': trace.get('contract_observation_deadline_ns', cutoff),
            'proposed_deadline_ns': trace.get('proposed_observation_deadline_ns', updates[-1].get('deadline_ns') if updates else None),
            'last_accepted_deadline_ns': trace.get('last_accepted_observation_deadline_ns', last_accepted),
            'actual_cancellation_requested_ns': cancel, 'pending_previous_cutoffs_ns': pending_previous},
        'legacy_cost_rss_counterfactual': 'NOT_RECONSTRUCTABLE' if mode == 'legacy_diagnostic' else 'ACTUAL_FULL_TRACE_ONLY'}
