"""Unique host display facts and later receipt-authorized state are distinct."""
from .endpoints import integer


def audit(trace, cutoff, effective_stop=None):
    effective_stop = cutoff if effective_stop is None else effective_stop
    events = trace.get('events', [])
    displays = [e for e in events if e.get('kind') == 'DISPLAY_PROXY']
    receipts = trace.get('host_display_receipts', [])
    effects = [e for e in events if e.get('kind') == 'STATE_EFFECT_COMMIT']
    rows, issues, violations = [], [], []
    for display in displays:
        data, eid = display.get('data', {}), display.get('event_id')
        publication = data.get('publication_id')
        same_id = [e for e in displays if e.get('data', {}).get('publication_id') == publication]
        dispatches = [e for e in events if e.get('kind') == 'PUBLICATION_DISPATCH_ADMITTED' and
            e.get('data', {}).get('publication_id') == publication and e.get('data', {}).get('display_event_id') == eid]
        at = display.get('observed_ns')
        dispatch = dispatches[0] if len(dispatches) == 1 else {}
        admitted = dispatch.get('data', {}).get('linearization_ns')
        proved = bool(isinstance(publication, str) and publication and len(same_id) == 1 and len(dispatches) == 1 and
            integer(admitted) and integer(at) and integer(effective_stop) and admitted < effective_stop and admitted <= at)
        row = {'publication_id': publication, 'display_event_id': eid, 'observed_ns': at,
            'dispatch_admitted_ns': admitted, 'dispatch_event_id': dispatch.get('event_id'),
            'admission_proved': proved, 'raw_display_within_cutoff': integer(at) and integer(cutoff) and at <= cutoff,
            'receipt_status': 'NOT_OBSERVED', 'candidate_receipt_ids': [], 'delivery_state_commit_ids': []}
        if not proved:
            issues.append('PUBLICATION_ID_OR_ADMISSION_UNPROVED:' + str(eid))
        host = [r for r in receipts if r.get('publication_id') == publication]
        candidate = [e for e in events if e.get('kind') == 'PUBLICATION_HOST_RECEIPT' and e.get('data', {}).get('publication_id') == publication]
        row['candidate_receipt_ids'] = [e.get('event_id') for e in candidate]
        canonical = host[0] if len(host) == 1 else None
        host_valid = bool(canonical and canonical.get('display_event_id') == eid and canonical.get('observed_ns') == at and
            isinstance(canonical.get('accepted'), bool) and all(integer(canonical.get(k)) for k in ('send_begin_ns', 'send_end_ns', 'provided_at_ns')) and
            at <= canonical['send_begin_ns'] <= canonical['send_end_ns'] == canonical['provided_at_ns'])
        row['host_receipt'] = canonical
        candidate_valid = []
        if canonical:
            for event in candidate:
                actual = event.get('data', {})
                clock = event.get('host_monotonic_ns')
                if host_valid and all(actual.get(k) == canonical.get(k) for k in ('publication_id', 'display_event_id', 'observed_ns', 'accepted', 'observation_deadline_ns')) and integer(clock) and clock >= canonical['send_begin_ns'] and clock >= at:
                    candidate_valid.append(event)
        if len(host) > 1 or len(candidate_valid) > 1:
            row['receipt_status'] = 'AMBIGUOUS_DUPLICATE_RECEIPT'
            issues.append('DUPLICATE_PUBLICATION_RECEIPT:' + str(publication))
        elif len(candidate_valid) == 1 and canonical.get('accepted') is True:
            row['receipt_status'] = 'ACK_RECEIVED' if integer(effective_stop) and candidate_valid[0]['host_monotonic_ns'] < effective_stop else 'ACK_RECEIVED_AFTER_CUTOFF'
        elif len(candidate_valid) == 1:
            row['receipt_status'] = 'HOST_REJECTED_RECEIPT'
        elif host and not host_valid:
            row['receipt_status'] = 'MALFORMED_HOST_RECEIPT'
            issues.append('PUBLICATION_RECEIPT_SOURCE_MISMATCH:' + str(publication))
        elif host:
            row['receipt_status'] = 'HOST_SENT_NO_CANDIDATE_RECEIPT'
        delivery = [e for e in effects if e.get('data', {}).get('effect_kind') == 'PUBLICATION_DELIVERY_STATE' and e.get('data', {}).get('data', {}).get('publication_id') == publication]
        valid_delivery = []
        valid_history, valid_fact = [], []
        for event in delivery:
            meta, commit = event['data'].get('data', {}), event['data'].get('linearization_ns')
            identities = all(meta.get(f) == data.get(f) for f in ('stimulus_id', 'goal_id', 'revision') if f in meta or f in data)
            phase = 'phase' not in meta or meta['phase'] == data.get('phase')
            captured = [e.get('data', {}) for e in events if e.get('kind') == 'INPUT_CAPTURED' and e.get('data', {}).get('stimulus_id') == data.get('stimulus_id')]
            fact = data.get('phase') == 'ACTUAL_AGENT_FACT' and len(captured) == 1 and captured[0].get('input_class') == 'agent' and \
                meta.get('class') == captured[0].get('event_class') == data.get('event_class') and data.get('task_id') == captured[0].get('task_id') and \
                ('task_id' not in meta or meta['task_id'] == data.get('task_id')) and \
                ('task_fact_state_recorded' not in meta or meta['task_fact_state_recorded'] is True) and \
                not meta.get('complete') and 'goal_id' not in meta and 'revision' not in meta
            history = meta.get('historical_display_recorded') is True
            valid = bool(proved and row['raw_display_within_cutoff'] and len(candidate_valid) == 1 and len(host) == 1 and
                canonical.get('accepted') is True and identities and phase and meta.get('actual_host_observed_ns') == at and (history or fact) and
                integer(commit) and candidate_valid[0]['host_monotonic_ns'] <= commit < effective_stop)
            if valid:
                valid_delivery.append(event)
                if history:
                    valid_history.append(event)
                if fact:
                    valid_fact.append(event)
            else:
                violations.append({'kind': 'PUBLICATION_DELIVERY_WITHOUT_CURRENT_RECEIPT_PROOF', 'event_id': event.get('event_id'), 'publication_id': publication})
        row['delivery_state_commit_ids'] = [e.get('event_id') for e in valid_delivery]
        if len(delivery) > 1:
            violations.append({'kind': 'DUPLICATE_PUBLICATION_DELIVERY_STATE', 'publication_id': publication})
        row['delivery_state_proved'] = len(valid_delivery) == 1 and len(delivery) == 1
        row['historical_display_state_proved'] = row['delivery_state_proved'] and len(valid_history) == 1
        row['task_fact_state_proved'] = row['delivery_state_proved'] and len(valid_fact) == 1
        row['completion_proved'] = row['historical_display_state_proved'] and valid_delivery[0]['data']['data'].get('complete') is True
        rows.append(row)
    by_id = {r['publication_id']: r for r in rows if isinstance(r['publication_id'], str)}
    for event in events:
        if event.get('kind') == 'PUBLICATION_HOST_RECEIPT' and event.get('data', {}).get('publication_id') not in by_id:
            issues.append('RECEIPT_ACCEPTED_WITHOUT_DISPATCHED_DISPLAY:' + str(event.get('event_id')))
    terminal_checks = []
    for event in events:
        if event.get('kind') != 'GOAL_COMPLETE':
            continue
        data = event.get('data', {})
        publication = data.get('publication_id')
        row = by_id.get(publication)
        matching = [e for e in effects if e.get('event_id') in (row or {}).get('delivery_state_commit_ids', [])]
        valid = bool(row and row['completion_proved'] and data.get('goal_complete') is True and len(matching) == 1 and
            matching[0]['data']['data'].get('complete') is True and
            all(matching[0]['data']['data'].get(f) == data.get(f) for f in ('stimulus_id', 'goal_id', 'revision')) and
            integer(event.get('host_monotonic_ns')) and matching[0]['data']['linearization_ns'] <= event['host_monotonic_ns'])
        terminal_checks.append({'event_id': event.get('event_id'), 'publication_id': publication, 'proved': valid})
        if not valid:
            issues.append('GOAL_WITHOUT_MATCHING_RECEIPT_DELIVERY_COMMIT:' + str(event.get('event_id')))
    for effect in effects:
        meta = effect.get('data', {}).get('data', {})
        publication = meta.get('publication_id')
        if publication is None or effect.get('data', {}).get('effect_kind') == 'PUBLICATION_DELIVERY_STATE':
            continue
        row = by_id.get(publication)
        kind = effect.get('data', {}).get('effect_kind')
        if not row or not row['delivery_state_proved'] or kind == 'SOURCE_REGISTER' and not row['completion_proved']:
            violations.append({'kind': 'POST_PUBLICATION_STATE_WITHOUT_DELIVERY_PROOF', 'event_id': effect.get('event_id'), 'publication_id': publication})
    return {'rows': rows, 'terminal_checks': terminal_checks, 'issues': issues, 'violations': violations,
        'rejected_receipts': [e for e in events if e.get('kind') == 'PUBLICATION_RECEIPT_REJECTED'],
        'receipt_timeouts': [e for e in events if e.get('kind') == 'PUBLICATION_RECEIPT_TIMEOUT'],
        'scope': 'Host observed display may qualify QA01 without a later delivery/goal commit. Missing, late, rejected or cancelled ACK is never fabricated.'}
