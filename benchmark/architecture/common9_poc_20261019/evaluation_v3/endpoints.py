"""Approved source-backed progress replacement, without answer-key prose."""

REPLACEMENT_OBLIGATIONS = frozenset(('S11-A01', 'S18-A03', 'S19-A01'))
RESULT_PHASES = frozenset(('ACTUAL_AGENT_FACT', 'AGENT_RESULT'))


def integer(value):
    return isinstance(value, int) and not isinstance(value, bool)


def capture_index(trace):
    result = {}
    for event in sorted(trace.get('events', []), key=lambda e: e.get('observed_ns', 0)):
        if event.get('kind') == 'INPUT_CAPTURED':
            result.setdefault(event.get('data', {}).get('stimulus_id'), event)
    return result


def provision_at(source, capture, at):
    """Proof of availability at card, not authored source due time alone."""
    received = capture.get('observed_ns') if capture else None
    if integer(received) and received <= at:
        return 'BEFORE'
    begin, end = source.get('send_begin_ns'), source.get('send_end_ns')
    if integer(begin) and integer(end) and begin <= end:
        if end <= at:
            return 'BEFORE'
        if begin > at:
            return 'AFTER'
    # A future source cannot supersede the card; an older due time alone
    # cannot prove actual provision. A malformed send bracket is ambiguous.
    if integer(source.get('source_ns')) and source['source_ns'] > at:
        return 'AFTER'
    return 'UNRESOLVED'


def replacement(trace, original, callback, captures):
    provided = trace.get('provided_sources', {})
    old = captures.get(original, {})
    olddata = old.get('data', {})
    data = callback.get('data', {})
    eid = data.get('stimulus_id')
    received = captures.get(eid, {})
    actual = received.get('data', {})
    source, prior = provided.get(eid), provided.get(original)
    at = callback.get('observed_ns')
    def fail(reason):
        return False, reason
    if olddata.get('input_class') != 'agent' or olddata.get('event_class') != 'PROGRESS' or not prior:
        return fail('ORIGINAL_PROGRESS_NOT_ACTUALLY_CAPTURED')
    task = olddata.get('task_id')
    if not task or actual.get('task_id') != task or data.get('task_id') != task:
        return fail('DIFFERENT_OR_MISSING_ACTUAL_TASK')
    if actual.get('input_class') != 'agent' or actual.get('event_class') != 'RESULT':
        return fail('NOT_ACTUAL_RESULT_CAPTURE')
    if not source:
        return fail('RESULT_SOURCE_NOT_PROVIDED')
    clocks = [prior.get('source_ns'), source.get('source_ns'), received.get('observed_ns'), at]
    if not all(integer(t) for t in clocks):
        return fail('MISSING_RESULT_SOURCE_CLOCK')
    if clocks[1] <= clocks[0]:
        return fail('RESULT_NOT_NEWER_THAN_PROGRESS')
    if not clocks[1] <= clocks[2] <= clocks[3]:
        return fail('CARD_BEFORE_ACTUAL_RESULT_CAPTURE_OR_SOURCE')
    if data.get('phase') not in RESULT_PHASES:
        return fail('WRONG_RESULT_EMISSION_PHASE')
    if str(data.get('text') or '') != str(actual.get('text') or '').strip():
        return fail('CARD_DIFFERS_FROM_ACTUAL_RESULT_TEXT')
    uncertain = []
    for other_id, other in captures.items():
        meta, supplied = other.get('data', {}), provided.get(other_id)
        if supplied is None or meta.get('input_class') != 'agent' or meta.get('event_class') not in ('PROGRESS', 'RESULT') or meta.get('task_id') != task:
            continue
        newer = supplied.get('source_ns')
        if not integer(newer):
            if provision_at(supplied, other, at) != 'AFTER':
                uncertain.append(other_id + ':MISSING_SOURCE_CLOCK')
            continue
        if newer <= source['source_ns']:
            continue
        availability = provision_at(supplied, other, at)
        if availability == 'BEFORE':
            return fail('KNOWN_NEWER_SAME_TASK_STATUS')
        if availability == 'UNRESOLVED':
            uncertain.append(other_id)
    if uncertain:
        return None, 'UNRESOLVED_PROVISION_ORDER:' + ','.join(sorted(uncertain))
    return True, 'ACTUAL_NEWER_SAME_TASK_RESULT_COVERS_PROGRESS'


def select(trace, expected, substantive, deadline, strict=False, publication_proofs=None):
    eid = expected['event_id']
    captures = capture_index(trace)
    provided = trace.get('provided_sources', {})
    source = provided.get(eid, {}).get('source_ns')
    audits, certain, possible = [], [], []
    for event in sorted((e for e in trace.get('events', []) if e.get('kind') == 'DISPLAY_PROXY'), key=lambda e: e.get('observed_ns', 0)):
        data, at = event.get('data', {}), event.get('observed_ns')
        own = data.get('stimulus_id') == eid
        if not own and eid not in REPLACEMENT_OBLIGATIONS:
            continue
        utterance = captures.get(eid, {}).get('data', {})
        yes, reason = substantive(data, utterance.get('text', '') if own and utterance.get('input_class') == 'user' else '')
        if not integer(source):
            yes, reason = False, 'SOURCE_NOT_PROVIDED'
        elif not integer(at) or at < source:
            yes, reason = False, 'BEFORE_ACTUAL_SOURCE'
        elif deadline is None:
            yes, reason = None, 'UNRESOLVED_OBSERVATION_DEADLINE'
        elif at > deadline:
            yes, reason = False, 'AFTER_OBSERVATION_DEADLINE'
        elif yes and own and strict and (not captures.get(eid) or not integer(captures[eid].get('observed_ns')) or captures[eid]['observed_ns'] > at):
            yes, reason = False, 'OWN_CARD_BEFORE_ACTUAL_CAPTURE_OR_CAPTURE_MISSING'
        elif yes and own and strict and provision_at(provided[eid], captures.get(eid), at) != 'BEFORE':
            yes, reason = None, 'UNRESOLVED_OWN_SOURCE_PROVISION'
        elif yes and not own:
            yes, reason = replacement(trace, eid, event, captures)
        if yes is True and publication_proofs is not None and not publication_proofs.get(event.get('event_id')):
            yes, reason = None, 'UNRESOLVED_UNIQUE_PUBLICATION_ADMISSION'
        audits.append({'event_id': event.get('event_id'), 'observed_ns': at, 'text': data.get('text'),
                       'phase': data.get('phase'), 'eligible': yes, 'reason': reason, 'own_stimulus': own})
        if yes is True:
            certain.append(event)
        elif yes is None and integer(at):
            possible.append(event)
    accepted = certain[0] if certain else None
    earlier = [e for e in possible if accepted is None or e['observed_ns'] < accepted['observed_ns']]
    unresolved = bool(earlier) or deadline is None
    return {**expected, 'status': 'UNRESOLVED_EARLIEST_BOUNDARY' if unresolved else 'OBSERVED' if accepted else 'NOT_OBSERVED',
        'source_ns': source, 'latency_ns': accepted['observed_ns'] - source if accepted and not unresolved else None,
        'accepted_callback_id': accepted.get('event_id') if accepted and not unresolved else None,
        'first_accepted_raw_text': accepted.get('data', {}).get('text') if accepted and not unresolved else None,
        'possible_first_callback_id': earlier[0].get('event_id') if earlier else None,
        'possible_latency_lower_bound_ns': earlier[0]['observed_ns'] - source if earlier and integer(source) else None,
        'certain_callback_id': accepted.get('event_id') if accepted else None,
        'certain_latency_upper_bound_ns': accepted['observed_ns'] - source if accepted and integer(source) else None,
        'filter_version': 'COMMON9-ENDPOINT-v3', 'callback_audit': audits,
        'raw_publication_count': sum(e.get('kind') == 'DISPLAY_PROXY' and e.get('data', {}).get('stimulus_id') == eid for e in trace.get('events', []))}
