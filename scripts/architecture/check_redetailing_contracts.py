#!/usr/bin/env python3
"""Recorded design-example consistency; no runtime, protocol or semantic test.

Default: check available 44/45/42 files, supporting staged publication. Explicit
--dp requires its file. Checks validate example shapes, references, source spans
and annotated hold/CAS orderings, not actual execution or measured QA.
"""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACTS = ROOT / 'docs/architecture/12-decisions/decision-packages/contracts'
FILES = {44: 'dp44-execution-examples.json',
         45: 'dp45-evidence-examples.json', 42: 'dp42-handoff-examples.json'}


def need(condition, message):
    if not condition:
        raise ValueError(message)


def keys(value, names, where):
    need(isinstance(value, dict), f'{where}: expected object')
    expected = set(names.split())
    need(set(value) == expected, f'{where}: missing {sorted(expected-set(value))}; '
         f'unknown {sorted(set(value)-expected)}')


def integer(value, where, minimum=1):
    need(type(value) is int and value >= minimum, f'{where}: integer >= {minimum} required')


def unique(values, key, where):
    need(isinstance(values, list), f'{where}: expected array')
    refs = [v[key] for v in values]
    need(len(refs) == len(set(refs)), f'{where}: duplicate {key}')


def one(values, predicate, where):
    matches = [v for v in values if predicate(v)]
    need(len(matches) == 1, f'{where}: expected one matching recorded step')
    return matches[0]


def no_duplicate_keys(pairs):
    result = {}
    for key, value in pairs:
        need(key not in result, f'duplicate JSON key: {key}')
        result[key] = value
    return result


def check44(d):
    keys(d, 'contract_version status events a_continuation b_publication_window '
         'b_clarification dispositions', '44 example')
    need(d['contract_version'] == 1 and d['status'] == 'DESIGN_REVIEW_EXAMPLES_NOT_RUNTIME',
         '44 review status')
    payloads = {'INPUT_READY': 'text evidence_refs', 'TASK_NOTICE': 'task_ref notice_kind text',
                'JOB_COMPLETED': 'status candidate_ref for_input',
                'DELIVERY_RECEIPT': 'actual_range status'}
    unique(d['events'], 'event_id', '44 events')
    for event in d['events']:
        keys(event, 'event_id kind conversation_ref source_ref source_revision input_generation payload',
             '44 EventEnvelope')
        need(event['kind'] in payloads, '44 unknown event kind')
        integer(event['source_revision'], '44 source_revision')
        if event['input_generation'] is not None:
            integer(event['input_generation'], '44 input_generation')
        need(event['conversation_ref'] == 'Conversation1', '44 same-case conversation')
        keys(event['payload'], payloads[event['kind']], '44 event payload')
    a, b, clarify = (d[k] for k in ('a_continuation', 'b_publication_window', 'b_clarification'))
    keys(a, 'job_ref input_ref input_generation next waiting_refs source_refs status', '44 Continuation')
    keys(b, 'candidate_ref kind input_ref input_generation required_refs source_vector status', '44 Window')
    keys(clarify, 'candidate_ref kind input_ref input_generation required_refs requires_input_settled',
         '44 clarification')
    need(a['next'] == 'PREPARE_RESPONSE' and a['status'] == 'WAIT_JOB', '44 continuation example')
    need(a['input_ref'] == b['input_ref'] == clarify['input_ref'] and
         a['input_generation'] == b['input_generation'] == clarify['input_generation'],
         '44 current input/generation correspondence')
    integer(a['input_generation'], '44 current generation')
    need('AdoptedMeaning1' in a['waiting_refs'] and
         {'AdoptedMeaning1', 'InputSettled1', 'Admission1', 'PriorDeliverySnapshot1'} <= set(b['required_refs']),
         '44 adopted meaning and prior delivery dependencies')
    need(b['kind'] == 'CONTENT' and b['status'] == 'WAIT_DEPENDENCY', '44 content window')
    need(clarify['kind'] == 'CLARIFICATION' and clarify['requires_input_settled'] is False and
         'InputSettled1' not in clarify['required_refs'] and
         'ClarificationAdmission1' in clarify['required_refs'], '44 clarification admission')
    events = {e['event_id']: e for e in d['events']}
    keys(events, 'Event1 Event2 Event3 Event4', '44 same-case events')
    need(events['Event3']['input_generation'] < a['input_generation'] and
         events['Event3']['payload']['for_input'] != a['input_ref'], '44 superseded result example')
    need(events['Event2']['input_generation'] is None, '44 task fact is not an input-bound result')
    need(d['dispositions'] == {'Event3': 'DISCARD_SUPERSEDED',
         'Event2': 'KEEP_DURABLE_AND_PREPARE_WHEN_ADMITTED',
         'Event4': 'UPDATE_ACTUAL_DELIVERY_NOT_TASK_CANCEL'}, '44 recorded dispositions')
    print('PASS: 44 fixed envelope, continuation/window, stale example and clarification admission')


def check42(d):
    keys(d, 'format_version status purpose labels owners common_assumptions command_intent_example '
         'a_normal b_normal hold_request hold_first dispatch_first late_acceptance rebind '
         'uncertainty_rules b_initial_work_snapshot', '42 example')
    need(d['format_version'] == 1 and d['status'] == 'DESIGN_REVIEW_EXAMPLES_NOT_RUNTIME_API', '42 review status')
    need(d['owners'] == {'dialogue_intent': 'Request Controller', 'current_question_and_task': 'Task Manager',
         'work_gate': 'Task Manager', 'transmission_state': 'Agent Gateway',
         'actual_publication': 'Response Manager', 'input_and_local_stop': 'Interaction Manager',
         'unit_of_work_module': 'State Store'}, '42 state owners')
    common = d['common_assumptions']
    need(common['via_location'] == 'LOCAL_PC' and
         common['vad_location'] == 'Interaction Manager / Turn-Taking Control' and
         common['voice_and_semantic_models'] == 'CLOUD_DEPENDENCIES_VIA_MODEL_ACCESS' and
         common['network_calls_inside_transaction'] is False and common['model_calls_inside_transaction'] is False and
         common['external_agent_idempotency'] == 'CAPABILITY_DEPENDENT_NOT_ASSUMED', '42 common conditions')
    initial = d['b_initial_work_snapshot']
    keys(initial, 'conversation_id control_session applied_sequence gate_revision gate_state input_epoch '
         'task_id task_revision question_id question_revision question_local_state permission_revision', '42 snapshot')
    for field in ('control_session', 'applied_sequence', 'gate_revision', 'input_epoch',
                  'task_revision', 'question_revision', 'permission_revision'):
        integer(initial[field], '42 snapshot.' + field)
    need(initial['gate_state'] == initial['question_local_state'] == 'OPEN', '42 initial OPEN state')

    def command(c):
        keys(c, 'command_id request_id conversation_id control_session expected_gate_revision input_epoch '
             'meaning_revision task_id expected_task_revision question_id expected_question_revision '
             'permission_revision kind payload evidence_refs', '42 CommandIntent')
        for field in ('control_session', 'expected_gate_revision', 'input_epoch', 'meaning_revision',
                      'expected_task_revision', 'expected_question_revision', 'permission_revision'):
            integer(c[field], '42 CommandIntent.' + field)
        need(c['conversation_id'] == initial['conversation_id'] and c['task_id'] == initial['task_id'] and
             c['question_id'] == initial['question_id'], '42 same conversation/task/question')
        need(c['kind'] in {'ANSWER_QUESTION', 'REVISE_ANSWER'}, '42 command kind')
        keys(c['payload'], 'answer', '42 command payload')
        need(isinstance(c['payload']['answer'], str) and
             set(c['evidence_refs']) <= set(d['labels']), '42 answer and known evidence references')

    intent = d['command_intent_example']
    command(intent)
    for field, source in [('control_session', 'control_session'), ('expected_gate_revision', 'gate_revision'),
                          ('input_epoch', 'input_epoch'), ('expected_task_revision', 'task_revision'),
                          ('expected_question_revision', 'question_revision'), ('permission_revision', 'permission_revision')]:
        need(intent[field] == initial[source], '42 initial command ' + field)
    a, b = d['a_normal'], d['b_normal']
    joint = one(a, lambda s: s.get('atomic_across_dialogue_and_work') is True, '42 A joint commit')
    need(joint['owner'] == 'State Store' and joint['result'] == 'COMMITTED', '42 A joint local commit')
    acceptance = one(b, lambda s: s.get('transaction') == 'work_local_accept', '42 B acceptance')
    need(acceptance['owner'] == 'Task Manager' and acceptance['atomic_with_dialogue_transaction'] is False,
         '42 B independent work acceptance')
    transfer = one(b, lambda s: s.get('producer') == 'Request Controller' and
                   s.get('consumer') == 'Task Manager', '42 B transfer')
    need(transfer['data'] == intent, '42 transferred command matches example')
    receipt = one(b, lambda s: s.get('data', {}).get('kind') == 'AcceptanceReceipt', '42 receipt')
    for field in ('command_id', 'acceptance_status', 'accepted_question_revision', 'transmission_state', 'event_id'):
        need(receipt['data'][field] == acceptance['data'][field], '42 receipt.' + field)
    need(receipt['producer'] == 'Task Manager' and receipt['consumer'] == 'Request Controller' and
         acceptance['data']['transmission_state'] == 'PENDING', '42 receipt is internal, not external acceptance')
    for steps, cas_name in ((a, 'A_dispatch_CAS'), (b, 'work_local_dispatch_CAS')):
        cas = one(steps, lambda s: s.get('transaction') == cas_name, '42 annotated CAS')
        send = one(steps, lambda s: s.get('producer') == 'Agent Gateway' and
                   s.get('consumer') == 'Downstream Agent', '42 external send')
        external = one(steps, lambda s: s.get('data', {}).get('transmission_state') == 'EXTERNAL_ACCEPTED',
                       '42 external receipt')
        need(cas['owner'] == 'Task Manager + Agent Gateway' and cas['transmission_before'] == 'PENDING' and
             cas['transmission_after'] == 'DISPATCHING' and any('permission' in s for s in cas['checks']) and
             send['outside_transaction'] is True and steps.index(cas) < steps.index(send) < steps.index(external),
             '42 recorded CAS/send/receipt boundaries')
    reflected = one(b, lambda s: s.get('transaction') == 'dialogue_receipt_reflection', '42 dialogue reflection')
    need(b.index(acceptance) < b.index(receipt) < b.index(reflected) and
         reflected['data']['command_ref'] == intent['command_id'], '42 separately reflected internal receipt')
    hold = d['hold_request']
    keys(hold, 'kind conversation_id control_session control_sequence input_epoch reason', '42 Hold')
    need(hold['kind'] == 'Hold' and hold['conversation_id'] == initial['conversation_id'] and
         hold['control_session'] == initial['control_session'] and hold['control_sequence'] > initial['applied_sequence'] and
         hold['input_epoch'] > initial['input_epoch'] and hold['reason'] == 'INPUT_PENDING', '42 newer protective hold')
    first = d['hold_first']
    gate, ack, reconcile, result = (first[k] for k in
        ('work_transaction_result', 'hold_ack', 'reconciliation', 'work_local_result'))
    for value in (ack, d['dispatch_first']['hold_ack']):
        keys(value, 'kind control_session applied_sequence gate_revision pending_held_commands '
             'dispatch_started_commands unknown_external_commands conversation_id', '42 HoldAck')
        need(value['kind'] == 'HoldAck' and value['conversation_id'] == initial['conversation_id'] and
             value['control_session'] == hold['control_session'] and
             value['applied_sequence'] == hold['control_sequence'], '42 HoldAck identity and applied sequence')
    need(gate['gate_state'] == 'HOLD' and gate['command_states'][intent['command_id']] == 'HELD', '42 hold-first effect')
    for field in ('control_session', 'applied_sequence', 'gate_revision'):
        need(ack[field] == gate[field], '42 hold ACK.' + field)
    need(ack['applied_sequence'] == hold['control_sequence'] and ack['pending_held_commands'] == [intent['command_id']] and
         not ack['dispatch_started_commands'] and not ack['unknown_external_commands'], '42 hold-first ACK classification')
    keys(reconcile, 'kind reconciliation_id conversation_id control_session control_sequence expected_gate_revision '
         'settled_input_epoch meaning_revision revoke_pending_commands retain_pending_commands new_command_intents', '42 reconciliation')
    need(reconcile['kind'] == 'ReleaseAndReconcile' and reconcile['control_session'] == hold['control_session'] and
         reconcile['control_sequence'] > hold['control_sequence'] and reconcile['expected_gate_revision'] == gate['gate_revision'] and
         reconcile['settled_input_epoch'] == hold['input_epoch'] and reconcile['revoke_pending_commands'] == [intent['command_id']] and
         not reconcile['retain_pending_commands'] and len(reconcile['new_command_intents']) == 1, '42 settled reconciliation')
    replacement = reconcile['new_command_intents'][0]
    command(replacement)
    need(replacement['command_id'] != intent['command_id'] and replacement['kind'] == 'REVISE_ANSWER' and
         replacement['payload'] != intent['payload'] and replacement['expected_gate_revision'] == gate['gate_revision'] and
         replacement['input_epoch'] == reconcile['settled_input_epoch'] and replacement['meaning_revision'] == reconcile['meaning_revision'] and
         replacement['expected_question_revision'] == acceptance['data']['accepted_question_revision'], '42 replacement command conditions')
    need(result['gate_revision'] > gate['gate_revision'] and result['gate_state'] == 'OPEN' and
         result['command_states'][intent['command_id']] == 'REVOKED' and
         result['command_states'][replacement['command_id']] == 'PENDING' and
         result['accepted_question_revision'] > replacement['expected_question_revision'] and
         result['atomic_with_dialogue_transaction'] is False, '42 work-local reconciliation effect')
    started = d['dispatch_first']['hold_ack']
    need(started['pending_held_commands'] == [] and
         started['dispatch_started_commands'] == started['unknown_external_commands'] == [intent['command_id']],
         '42 dispatch-first retains unknown original command')
    late = d['late_acceptance']
    need(late['before']['command_id'] == late['late_receipt']['command_id'] == late['after']['command_id'] == intent['command_id'] and
         late['before']['attempt_id'] == late['late_receipt']['attempt_id'] and late['before']['transmission_state'] == 'UNKNOWN' and
         late['after']['transmission_state'] == 'EXTERNAL_ACCEPTED' and late['new_task_created'] is False,
         '42 late receipt remains with original command/attempt')
    rebind = d['rebind']
    need(rebind['previous_control_session'] == hold['control_session'] and rebind['new_control_session'] > rebind['previous_control_session'] and
         rebind['initial_gate_state'] == 'HOLD' and rebind['automatic_release'] is False and
         rebind['delayed_old_release']['control_session'] == rebind['previous_control_session'] and
         rebind['delayed_old_release']['result'] == 'REJECTED_STALE_SESSION', '42 old-session release fenced')
    print('PASS: 42 recorded owners, local boundaries, hold/CAS orderings, receipt and rebind references')


def check45(d):
    keys(d, 'contract_status utterance input_ref source_snapshot query a_owner_reads a_bundle query_receipt '
         'b_production_job b_semantic_job b_semantic_output b_memory_records b_coverage b_publish_result b_bundle '
         'not_covered unsupported_relation correction delete_or_revoke new_request_condition limits', '45 example')
    need(d['contract_status'] == 'DESIGN_REVIEW_V1_NOT_IMPLEMENTED_NOT_MEASURED', '45 review status')
    snapshots = d['source_snapshot']
    owners = {'E20': 'Response Manager', 'P20': 'Response Manager', 'T21': 'Task Manager', 'T22': 'Task Manager'}
    versions = {ref: (owners[ref], value['revision']) for ref, value in snapshots.items()}
    for value in snapshots.values():
        integer(value['revision'], '45 source revision')
        if 'result_ref' in value:
            integer(value['result_revision'], '45 result revision')
            versions[value['result_ref']] = ('Task Manager', value['result_revision'])
    delivery = snapshots['P20']
    need(delivery['response_ref'] == 'E20', '45 delivery links original response')

    def intervals(spans, ref, where):
        original = snapshots.get(ref, {}).get('response_ref', ref)
        text = snapshots.get(original, {}).get('text')
        need(isinstance(text, str) and isinstance(spans, list), f'{where}: known text coordinate source')
        for span in spans:
            need(isinstance(span, list) and len(span) == 2 and all(type(n) is int for n in span) and
                 0 <= span[0] < span[1] <= len(text), f'{where}: bounded Unicode [start,end)')
        return spans

    delivered = intervals(delivery['actual_delivered_spans'], 'P20', '45 delivered spans')

    def vector(values, where):
        unique(values, 'source_ref', where)
        for value in values:
            keys(value, 'owner source_ref revision', where)
            integer(value['revision'], where + '.revision')
            need(value['source_ref'] in versions and
                 (value['owner'], value['revision']) == versions[value['source_ref']], f'{where}: source owner/revision')
        return {(v['owner'], v['source_ref'], v['revision']) for v in values}

    def source_span(value, where):
        keys(value, 'owner source_ref revision start end unit text', where)
        need(value['source_ref'] == 'E20' and value['owner'] == 'Response Manager' and
             value['revision'] == snapshots['E20']['revision'] and value['unit'] == 'UNICODE_CODEPOINT', f'{where}: original coordinates')
        intervals([[value['start'], value['end']]], value['source_ref'], where)
        need(snapshots['E20']['text'][value['start']:value['end']] == value['text'], f'{where}: exact original substring')
        need(any(lo <= value['start'] and value['end'] <= hi for lo, hi in delivered), f'{where}: actually delivered span')

    allowed = {'PRESENTS', 'HAS_RESULT', 'REVISES', 'EXPLAINS_ITEM'}
    query = d['query']
    keys(query, 'schema_version query_id request_ref attempt_ref input_revision purpose relation_kinds '
         'source_ranges original_required budget_ref policy_epoch', '45 query')
    need(query['schema_version'] == 1 and query['purpose'] in {'REQUEST_UNDERSTANDING', 'DELEGATION_CONTEXT'} and
         set(query['relation_kinds']) <= allowed and query['original_required'] is True, '45 bounded query')
    integer(query['input_revision'], '45 input revision'); integer(query['policy_epoch'], '45 policy epoch')
    unique(d['a_owner_reads'], 'read_id', '45 owner reads')
    for read in d['a_owner_reads']:
        keys(read, 'read_id query_id owner selector', '45 owner read')
        need(read['query_id'] == query['query_id'], '45 owner read query correspondence')
        if read['owner'] == 'Response Manager':
            keys(read['selector'], 'delivery_ref include_delivered_excerpt', '45 response selector')
            need(read['selector']['delivery_ref'] == 'P20' and read['selector']['include_delivered_excerpt'] is True,
                 '45 actual-delivery read')
        else:
            need(read['owner'] == 'Task Manager', '45 permitted owner')
            keys(read['selector'], 'task_candidates include_result_refs', '45 task selector')
            need(set(read['selector']['task_candidates']) == {'T21', 'T22'} and read['selector']['include_result_refs'] is True,
                 '45 same candidate result read')
    for name in ('a_bundle', 'b_bundle'):
        bundle = d[name]
        keys(bundle, 'schema_version bundle_ref query_id status evidence relations source_vector issues receipt_ref '
             'policy_epoch acquisition', '45 bundle')
        need(bundle['schema_version'] == 1 and bundle['query_id'] == query['query_id'] and
             bundle['policy_epoch'] == query['policy_epoch'] and bundle['status'] == 'PARTIAL' and bundle['issues'],
             '45 partial output does not assert complete criteria')
        vector(bundle['source_vector'], '45 bundle vector')
        unique(bundle['evidence'], 'evidence_ref', '45 bundle evidence')
        for evidence in bundle['evidence']:
            if evidence['representation'] == 'ORIGINAL_EXCERPT':
                keys(evidence, 'evidence_ref source_ref delivery_ref source_span representation', '45 original evidence')
                need(evidence['source_ref'] == delivery['response_ref'] and evidence['delivery_ref'] == 'P20', '45 delivery refs')
                source_span(evidence['source_span'], '45 evidence span')
            else:
                keys(evidence, 'evidence_ref source_ref revision task_ref representation locator', '45 result evidence')
                task = snapshots[evidence['task_ref']]
                need(evidence['representation'] == 'RESULT_REFERENCE' and evidence['source_ref'] == task['result_ref'] and
                     evidence['revision'] == task['result_revision'] and
                     evidence['locator'] == f"result-ref:{evidence['source_ref']}:{evidence['revision']}", '45 confirmed result locator')
        for relation in bundle['relations']:
            keys(relation, 'kind from_ref to_ref to_revision', '45 explicit relation')
            task = snapshots[relation['from_ref']]
            need(relation['kind'] == 'HAS_RESULT' and relation['to_ref'] == task['result_ref'] and
                 relation['to_revision'] == task['result_revision'], '45 explicit task/result relation')
    a, b = d['a_bundle'], d['b_bundle']
    need(a['acquisition'] == 'OWNER_COMPOSITION' and b['acquisition'] == 'PUBLISHED_MEMORY' and
         {k: v for k, v in a.items() if k != 'acquisition'} == {k: v for k, v in b.items() if k != 'acquisition'},
         '45 same recorded consumer output, different acquisition')
    receipt = d['query_receipt']
    keys(receipt, 'schema_version receipt_ref query_id read_ranges checked_revisions source_cursors gaps excluded_ranges '
         'deadline_outcome', '45 receipt')
    need(receipt['query_id'] == query['query_id'] and receipt['receipt_ref'] == a['receipt_ref'] and
         vector(receipt['checked_revisions'], '45 checked vector') == vector(a['source_vector'], '45 input vector'), '45 receipt references')
    for excluded in receipt['excluded_ranges']:
        intervals(excluded['spans'], excluded['source_ref'], '45 excluded spans')
        need(excluded['reason'] == 'NOT_DELIVERED' and not any(max(lo, start) < min(hi, end)
             for lo, hi in excluded['spans'] for start, end in delivered), '45 undelivered range excluded')
    for read_range in receipt['read_ranges']:
        if 'spans' in read_range:
            intervals(read_range['spans'], read_range['source_ref'], '45 actually read range')
    for evidence in a['evidence']:
        if 'source_span' in evidence:
            excerpt = evidence['source_span']
            need(any(row.get('source_ref') == excerpt['source_ref'] and any(
                 start <= excerpt['start'] and excerpt['end'] <= end
                 for start, end in row.get('spans', [])) for row in receipt['read_ranges']),
                 '45 returned excerpt inside an actually read range')
    production = d['b_production_job']
    keys(production, 'job_id trigger requested_relation_kinds input_source_vector policy_epoch publication_expected_revision '
         'schema_version producer_version', '45 production job')
    need(production['schema_version'] == 1 and production['policy_epoch'] == query['policy_epoch'] and
         set(production['requested_relation_kinds']) <= allowed and
         vector(production['input_source_vector'], '45 producer vector') == vector(a['source_vector'], '45 A vector'), '45 producer inputs')
    trigger = production['trigger']
    keys(trigger, 'kind change_id source_ref revision cursor', '45 recorded production trigger')
    need(trigger['kind'] == 'SOURCE_CHANGED' and trigger['source_ref'] in versions and
         trigger['revision'] == versions[trigger['source_ref']][1], '45 trigger corresponds to known source')
    semantic = d['b_semantic_job']
    keys(semantic, 'job_id role source_spans allowed_relation_kinds allowed_from_refs allowed_to_refs policy_epoch '
         'schema_version producer_version', '45 semantic job')
    need(semantic['role'] == 'C-CONTEXT' and semantic['policy_epoch'] == query['policy_epoch'] and
         semantic['producer_version'] == production['producer_version'], '45 semantic role and permissions')
    for span in semantic['source_spans']:
        source_span(span, '45 semantic input span')
    output = d['b_semantic_output']
    keys(output, 'relation_kind from_ref to_ref source_spans value uncertain', '45 semantic output')
    need(output['relation_kind'] in semantic['allowed_relation_kinds'] and output['from_ref'] in semantic['allowed_from_refs'] and
         output['to_ref'] in semantic['allowed_to_refs'] and type(output['uncertain']) is bool and
         output['source_spans'] == semantic['source_spans'], '45 bounded semantic output references')
    unique(d['b_memory_records'], 'record_id', '45 memory records')
    for record in d['b_memory_records']:
        keys(record, 'schema_version record_id relation_kind from_ref to_ref source_spans value derivation uncertain '
             'producer_version dependency_vector policy_epoch published_revision validity', '45 memory record')
        need(record['schema_version'] == 1 and record['relation_kind'] in allowed and
             record['derivation'] in {'MODEL_DERIVED', 'EXPLICIT'} and type(record['uncertain']) is bool and
             record['policy_epoch'] == query['policy_epoch'] and record['validity'] == 'VALID' and
             record['producer_version'] == production['producer_version'], '45 record provenance')
        deps = vector(record['dependency_vector'], '45 record dependencies')
        need({record['from_ref'], record['to_ref']} <= {ref for _, ref, _ in deps}, '45 relation dependencies')
        if record['derivation'] == 'MODEL_DERIVED':
            need(record['source_spans'] and record['relation_kind'] == output['relation_kind'] and
                 record['from_ref'] == output['from_ref'] and record['to_ref'] == output['to_ref'] and
                 record['value'] == output['value'] and record['source_spans'] == output['source_spans'], '45 bounded record output')
            for span in record['source_spans']:
                source_span(span, '45 published source span')
        else:
            need(record['source_spans'] == [] and record['value'] is None and record['relation_kind'] == 'HAS_RESULT' and
                 snapshots[record['from_ref']]['result_ref'] == record['to_ref'], '45 explicit result without invented text')
    coverage, published = d['b_coverage'], d['b_publish_result']
    keys(coverage, 'schema_version coverage_ref source_ranges source_cursors gaps production_failures published_revision', '45 coverage')
    keys(published, 'job_id status published_revision coverage_ref rejected_dependencies', '45 publish result')
    need(published['job_id'] == production['job_id'] and published['status'] == 'PUBLISHED' and
         published['coverage_ref'] == coverage['coverage_ref'] and
         published['published_revision'] == coverage['published_revision'] == production['publication_expected_revision'] + 1 and
         all(r['published_revision'] == published['published_revision'] for r in d['b_memory_records']), '45 common publication revision')
    need(coverage['source_cursors'] == receipt['source_cursors'], '45 source cursors')
    for section in ('source_ranges', 'gaps'):
        for row in coverage[section]:
            if 'spans' in row:
                intervals(row['spans'], row['source_ref'], '45 coverage ' + section)
    for gap in coverage['gaps']:
        need(gap['reason'] == 'NOT_COVERED' and gap['relation_kind'] in allowed, '45 supported unproduced coverage gap')
        need(not any(row.get('source_ref') == gap['source_ref'] and
             gap['relation_kind'] in row.get('relation_kinds', []) and any(
             max(start, lo) < min(end, hi) for start, end in gap['spans']
             for lo, hi in row.get('spans', [])) for row in coverage['source_ranges']),
             '45 same span/kind cannot be both covered and not covered')
    for record in d['b_memory_records']:
        if record['derivation'] == 'MODEL_DERIVED':
            for excerpt in record['source_spans']:
                need(any(row.get('source_ref') == record['from_ref'] and
                     record['relation_kind'] in row.get('relation_kinds', []) and any(
                     lo <= excerpt['start'] and excerpt['end'] <= hi for lo, hi in row.get('spans', []))
                     for row in coverage['source_ranges']), '45 published excerpt covered by recorded publication')
    need(d['not_covered']['status'] == 'NOT_COVERED' and
         d['not_covered']['next_step'] == 'REQUEST_SUPPORTED_RANGE_PRODUCTION_AND_REREAD' and
         d['unsupported_relation']['status'] == 'UNSUPPORTED_RELATION' and
         d['unsupported_relation']['requested_kind'] not in allowed, '45 not-covered versus unsupported')
    change, revoke = d['correction'], d['delete_or_revoke']
    need(change['source_ref'] in versions and change['from_revision'] == versions[change['source_ref']][1] and
         change['to_revision'] > change['from_revision'], '45 changed source revision')
    need(revoke['policy_epoch_before'] == query['policy_epoch'] and revoke['policy_epoch_after'] > revoke['policy_epoch_before'] and
         revoke['first_step'] == 'FENCE_NEW_USE' and revoke['old_job_step'] == 'REJECT_OLD_EPOCH_PUBLICATION', '45 old epoch fenced')
    need(d['new_request_condition']['reuse_allowed'] == 'VALID_PAST_EVIDENCE_ONLY' and
         d['new_request_condition']['reuse_forbidden'] == 'PREVIOUS_REQUEST_BINDING_OR_DOMAIN_JUDGMENT', '45 evidence is not current meaning')
    print('PASS: 45 recorded original/delivered spans, owner vectors, bounded production and common output')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dp', type=int, choices=list(FILES), action='append',
                        help='Check only this DP; repeat to select multiple. Missing file is an error.')
    args = parser.parse_args()
    selected = args.dp if args.dp else [dp for dp in FILES if (CONTRACTS / FILES[dp]).is_file()]
    need(selected, 'No redetailing design examples found')
    checks = {44: check44, 45: check45, 42: check42}
    for dp in dict.fromkeys(selected):
        path = CONTRACTS / FILES[dp]
        checks[dp](json.loads(path.read_text(), object_pairs_hook=no_duplicate_keys))
    if args.dp is None:
        absent = [str(dp) for dp in FILES if dp not in selected]
        if absent:
            print('SKIP: unpublished optional examples ' + ', '.join(absent))
    print('Documentation examples only: no protocol execution, model/semantic test or measured QA result.')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, TypeError, FileNotFoundError, json.JSONDecodeError) as error:
        raise SystemExit(f'FAIL: {error}')
