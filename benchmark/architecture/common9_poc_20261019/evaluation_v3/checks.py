"""Finite endpoint/addendum and deadline checks; no model or candidate calls."""
import copy
import unittest
from .endpoints import select
from .deadlines import audit
from .evaluate import legacy, evaluate, summarize


def fixture():
    def capture(eid, source, observed, event_class):
        return {'event_id': 'capture-' + eid, 'kind': 'INPUT_CAPTURED', 'observed_ns': observed,
            'data': {'stimulus_id': eid, 'task_id': 'BG', 'input_class': 'agent', 'event_class': event_class, 'text': ' actual result ' if event_class == 'RESULT' else 'progress 50%'}}
    def card(eid, at, text='actual result', phase='AGENT_RESULT'):
        return {'event_id': 'card-' + eid, 'kind': 'DISPLAY_PROXY', 'observed_ns': at,
            'data': {'stimulus_id': eid, 'task_id': 'BG', 'text': text, 'phase': phase}}
    trace = {'events': [capture('S11-A01', 100, 101, 'PROGRESS'), capture('R', 110, 111, 'RESULT'), card('R', 120)],
             'provided_sources': {'S11-A01': {'source_ns': 100}, 'R': {'source_ns': 110}}}
    return trace, capture, card


class Endpoints(unittest.TestCase):
    def choose(self, trace, deadline=200):
        return select(trace, {'event_id': 'S11-A01', 'response_id': 'response'}, legacy.substantive, deadline)

    def test_01_valid_result(self):
        row = self.choose(fixture()[0]); self.assertEqual(row['latency_ns'], 20)

    def test_02_older_delayed(self):
        t, _, _ = fixture(); t['provided_sources']['R']['source_ns'] = 90
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_03_handling_word(self):
        t, _, _ = fixture(); t['events'][-1]['data'].update(text='BG 작업의 완료 여부를 확인하겠습니다', phase='HANDLING')
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_04_other_task(self):
        t, _, _ = fixture(); t['events'][-1]['data']['task_id'] = 'OTHER'
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_05_not_provided(self):
        t, _, _ = fixture(); del t['provided_sources']['R']
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_06_wrong_phase(self):
        t, _, _ = fixture(); t['events'][-1]['data']['phase'] = 'DIRECT'
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_07_wrong_text(self):
        t, _, _ = fixture(); t['events'][-1]['data']['text'] = 'different concrete answer'
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_08_own_later(self):
        t, _, card = fixture(); t['events'].append(card('S11-A01', 130, 'progress 50%', 'ACTUAL_AGENT_FACT'))
        self.assertEqual(self.choose(t)['latency_ns'], 20)

    def test_09_own_first(self):
        t, _, card = fixture(); t['events'].append(card('S11-A01', 105, 'progress 50%'))
        self.assertEqual(self.choose(t)['latency_ns'], 5)

    def newer(self, source=115, captured=119):
        t, capture, _ = fixture(); t['provided_sources']['N'] = {'source_ns': source}
        t['events'].append(capture('N', source, captured, 'PROGRESS')); return t

    def test_10_known_newer(self):
        self.assertEqual(self.choose(self.newer())['status'], 'NOT_OBSERVED')

    def test_11_future_status(self):
        self.assertEqual(self.choose(self.newer(130, 132))['latency_ns'], 20)

    def test_12_ambiguous_before_own(self):
        t = self.newer(captured=135); _, _, card = fixture(); t['events'].append(card('S11-A01', 130, 'progress 50%'))
        row = self.choose(t); self.assertEqual(row['status'], 'UNRESOLVED_EARLIEST_BOUNDARY')
        self.assertIsNone(row['latency_ns']); self.assertEqual(row['possible_latency_lower_bound_ns'], 20); self.assertEqual(row['certain_latency_upper_bound_ns'], 30)

    def test_13_ambiguous_after_own(self):
        t = self.newer(captured=135); _, _, card = fixture(); t['events'].append(card('S11-A01', 105, 'progress 50%'))
        self.assertEqual(self.choose(t)['latency_ns'], 5)

    def test_14_before_capture(self):
        t, _, _ = fixture(); t['events'][1]['observed_ns'] = 121
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_deadline_inclusive_observation(self):
        self.assertEqual(self.choose(fixture()[0], 120)['status'], 'OBSERVED')
        self.assertEqual(self.choose(fixture()[0], 119)['status'], 'NOT_OBSERVED')

    def test_missing_deadline(self):
        self.assertIsNone(self.choose(fixture()[0], None)['latency_ns'])

    def test_actual_send_resolves_ambiguity(self):
        t = self.newer(captured=135); t['provided_sources']['N'].update(send_begin_ns=125, send_end_ns=126)
        self.assertEqual(self.choose(t)['latency_ns'], 20)
        t['provided_sources']['N'].update(send_begin_ns=116, send_end_ns=117)
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_inner_whitespace_not_normalized(self):
        t, _, _ = fixture(); t['events'][-1]['data']['text'] = 'actual  result'
        self.assertEqual(self.choose(t)['status'], 'NOT_OBSERVED')

    def test_send_bracket_straddling_card_is_ambiguous(self):
        t = self.newer(captured=135); t['provided_sources']['N'].update(send_begin_ns=119, send_end_ns=121)
        self.assertEqual(self.choose(t)['status'], 'UNRESOLVED_EARLIEST_BOUNDARY')

    def test_missing_potential_status_clock_is_not_silently_skipped(self):
        t = self.newer(captured=119); del t['provided_sources']['N']['source_ns']
        self.assertEqual(self.choose(t)['status'], 'UNRESOLVED_EARLIEST_BOUNDARY')


class Deadlines(unittest.TestCase):
    def record(self):
        return {'authored_time_scale': 1e-9, 'observation_deadline_ns': 160, 'effective_admission_stop_ns': 160,
            'contract_observation_deadline_ns': 160, 'proposed_observation_deadline_ns': 160, 'last_accepted_observation_deadline_ns': 160,
            'technical_abort_requested_ns': None, 'settlement_anchor_ns': 160, 'pending_deadline_update_cutoffs_at_cancel': {},
            'observation_deadline_updates': [{'at_ns': 102, 'last_actual_source_ns': 100, 'deadline_ns': 160}],
            'provided_sources': {'P': {'source_ns': 99, 'send_begin_ns': 99, 'send_end_ns': 100, 'provided_at_ns': 100}},
            'clock_probe_send': {'send_begin_ns': 1}, 'host_clock_proof': {'event_id': 'clock'},
            'cancellation_requested_ns': 161, 'settlement_deadline_ns': 280, 'end_ns': 200,
            'events': [{'kind': 'HOST_CLOCK_PROBE_ACK', 'event_id': 'clock', 'host_monotonic_ns': 3, 'observed_ns': 4, 'data': {'host_monotonic_ns': 2}},
                {'kind': 'OBSERVATION_DEADLINE_ACK', 'event_id': 'arm', 'host_monotonic_ns': 104, 'observed_ns': 105,
                 'data': {'accepted': True, 'host_monotonic_ns': 103, 'control': {'at_ns': 102, 'deadline_ns': 160}}},
                {'kind': 'WORK_CANCELLATION_ACK', 'event_id': 'cancel', 'host_monotonic_ns': 162, 'observed_ns': 163, 'data': {'host_monotonic_ns': 162}},
                {'kind': 'STATE_EFFECT_COMMIT', 'event_id': 'effect',
                'data': {'effect_kind': 'SEMANTIC_ADOPTION', 'linearization_ns': 159, 'deadline_ns': 160, 'data': {'stimulus_id': 'P'}}}]}

    def test_consistent(self): self.assertEqual(audit(self.record())['status'], 'CONSISTENT')
    def test_effect_at_cutoff(self):
        t = self.record(); t['events'][-1]['data']['linearization_ns'] = 160
        self.assertEqual(audit(t)['status'], 'VIOLATION')
    def test_no_revival(self):
        t = self.record(); t['observation_deadline_updates'].append({'at_ns': 161, 'last_actual_source_ns': 155, 'deadline_ns': 215}); t['observation_deadline_ns'] = 215
        self.assertEqual(audit(t)['status'], 'VIOLATION')
    def test_missing_send_clock(self):
        t = self.record(); del t['provided_sources']['P']['send_begin_ns']
        self.assertEqual(audit(t)['status'], 'UNRESOLVED')
    def test_bad_send_order(self):
        t = self.record(); t['provided_sources']['P']['send_end_ns'] = 99
        self.assertEqual(audit(t)['status'], 'VIOLATION')
    def test_legacy_is_never_primary_proof(self):
        t = self.record(); del t['observation_deadline_ns']
        self.assertEqual(audit(t, 'legacy_diagnostic')['legacy_cost_rss_counterfactual'], 'NOT_RECONSTRUCTABLE')

    def test_null_gate_only_before_arm(self):
        t = self.record(); t['events'][-1]['data'].update(linearization_ns=99, deadline_ns=None)
        self.assertEqual(audit(t)['state_effect_checks'][0]['status'], 'PRE_ARM_WITHIN_FINAL_CUTOFF')
        t['events'][-1]['data']['linearization_ns'] = 103
        self.assertEqual(audit(t)['status'], 'VIOLATION')

    def test_late_dispatch_rejected_but_late_terminal_allowed(self):
        t = self.record(); t['calls'] = [{'actual_api_sent': True, 'http_dispatch_registered_ns': 159, 'terminal_ns': 200}]
        self.assertEqual(audit(t)['status'], 'CONSISTENT')
        t['calls'][0]['http_dispatch_registered_ns'] = 160
        self.assertEqual(audit(t)['status'], 'VIOLATION')

    def test_clock_status_not_trusted_without_raw_probe(self):
        t = self.record(); t['host_clock_proof']['status'] = 'VERIFIED'; t['events'] = [e for e in t['events'] if e['kind'] != 'HOST_CLOCK_PROBE_ACK']
        self.assertIn('HOST_CLOCK_DOMAIN_NOT_INDEPENDENTLY_VERIFIED', audit(t)['issues'])

    def test_outside_clock_bracket(self):
        t = self.record(); t['events'][0]['data']['host_monotonic_ns'] = 5
        self.assertFalse(audit(t)['clock_domain_verified'])

    def test_due_not_actual_provision(self):
        t = self.record(); t['observation_deadline_updates'][0]['last_actual_source_ns'] = 99
        self.assertIn('DEADLINE_ROOT_NOT_LAST_ACTUAL_PROVISION', [v['kind'] for v in audit(t)['violations']])

    def test_cancellation_required(self):
        t = self.record(); del t['cancellation_requested_ns']
        self.assertIn('MISSING_ACTUAL_CANCELLATION_REQUEST', audit(t)['issues'])

    def test_missing_deadline_ack(self):
        t = self.record(); t['events'] = [e for e in t['events'] if e['kind'] != 'OBSERVATION_DEADLINE_ACK']
        self.assertTrue(any(x.startswith('DEADLINE_UPDATE_ACK_MISSING') for x in audit(t)['issues']))

    def test_early_termination(self):
        t = self.record(); t['end_ns'] = 159
        self.assertIn('OBSERVATION_ENDED_BEFORE_REQUIRED_CUTOFF', [v['kind'] for v in audit(t)['violations']])

    def test_unknown_gate_not_accepted(self):
        t = self.record(); t['events'][-1]['data']['deadline_ns'] = 999
        self.assertEqual(audit(t)['status'], 'VIOLATION')

    def test_cloud_dispatch_before_verified_proof(self):
        t = self.record(); t['calls'] = [{'actual_api_sent': True, 'http_dispatch_registered_ns': 3}]
        self.assertEqual(audit(t)['status'], 'VIOLATION')

    def test_local_dispatch_before_verified_proof(self):
        t = self.record(); t['calls'] = [{'evidence_kind': 'LOCAL_MODEL', 'local_spawn_admitted_ns': 3, 'local_parse_dispatch_admitted_ns': 110}]
        self.assertEqual(audit(t)['status'], 'VIOLATION')

    def test_local_cache_only_explicit_no_dispatch(self):
        t = self.record(); t['calls'] = [{'evidence_kind': 'LOCAL_MODEL', 'local_spawn_admitted_ns': None, 'local_parse_dispatch_admitted_ns': None}]
        self.assertEqual(audit(t)['status'], 'CONSISTENT')


class Integration(unittest.TestCase):
    def record(self):
        t = Deadlines().record()
        t.update(episode='S02', run_id='synthetic', configuration='AAAA', phase='OFFLINE', rep=1, returncode=0,
            source_nodes=[{'id': 'source:P', 'start_ns': 100, 'end_ns': 100, 'parents': [], 'role': 'INITIAL'}],
            calls=[{'cost_usd': .5, 'input_tokens': 10, 'output_tokens': 5, 'terminal_ns': 200, 'role': 'VIA'}])
        t['provided_sources']['P']['source_node'] = 'source:P'
        next(e for e in t['events'] if e['kind'] == 'STATE_EFFECT_COMMIT')['data']['linearization_ns'] = 139
        t['host_display_receipts'] = [{'type': 'publication_receipt', 'publication_id': 'pub-P', 'display_event_id': 'card-P',
            'observed_ns': 150, 'accepted': True, 'observation_deadline_ns': 160, 'send_begin_ns': 151, 'send_end_ns': 152, 'provided_at_ns': 152}]
        t['events'] += [{'kind': 'INPUT_CAPTURED', 'event_id': 'capture-P', 'observed_ns': 102,
                        'data': {'stimulus_id': 'P', 'input_class': 'user', 'text': 'what is that?'}},
                       {'kind': 'SEMANTIC_ADOPTION', 'event_id': 'adopt-P', 'observed_ns': 141, 'host_monotonic_ns': 140,
                        'data': {'stimulus_id': 'P', 'goal_id': 'G', 'revision': 1,
                                 'plan': {'bindings': [{'expression': 'that', 'targets': ['DOC:v1:P1']}]}}},
                       {'kind': 'STATE_EFFECT_COMMIT', 'event_id': 'publication', 'observed_ns': 156, 'host_monotonic_ns': 155,
                        'data': {'effect_kind': 'PUBLICATION_DELIVERY_STATE', 'linearization_ns': 155, 'deadline_ns': 160,
                                 'data': {'publication_id': 'pub-P', 'actual_host_observed_ns': 150, 'historical_display_recorded': True, 'complete': True, 'stimulus_id': 'P', 'phase': 'DIRECT', 'goal_id': 'G', 'revision': 1}}},
                       {'kind': 'DISPLAY_PROXY', 'event_id': 'card-P', 'observed_ns': 150, 'host_monotonic_ns': 147, 'parent_ids': ['source:P'],
                        'data': {'publication_id': 'pub-P', 'stimulus_id': 'P', 'phase': 'DIRECT', 'goal_id': 'G', 'revision': 1, 'text': 'The actual answer has concrete detail.'}},
                       {'kind': 'PUBLICATION_DISPATCH_ADMITTED', 'event_id': 'dispatch-P', 'observed_ns': 151, 'host_monotonic_ns': 150,
                        'data': {'publication_id': 'pub-P', 'display_event_id': 'card-P', 'linearization_ns': 149}},
                       {'kind': 'PUBLICATION_HOST_RECEIPT', 'event_id': 'receipt-P', 'observed_ns': 154, 'host_monotonic_ns': 153,
                        'data': {'type': 'publication_receipt', 'publication_id': 'pub-P', 'display_event_id': 'card-P', 'observed_ns': 150, 'observation_deadline_ns': 160, 'accepted': True}},
                       {'kind': 'GOAL_COMPLETE', 'event_id': 'goal-P', 'observed_ns': 157, 'host_monotonic_ns': 156, 'parent_ids': ['source:P'],
                        'data': {'publication_id': 'pub-P', 'goal_complete': True, 'stimulus_id': 'P', 'origin_event_id': 'P', 'goal_id': 'G', 'revision': 1}}]
        oracle = {'responses': [{'event_id': 'P', 'response_id': 'R', 'required_content': 'diagnostic'}],
                  'goals': [{'id': 'G', 'start_event': 'P', 'member_events': ['P'], 'completion': 'diagnostic'}],
                  'bindings': [{'id': 'B', 'event_id': 'P', 'expression': 'that', 'expected_set': ['DOC:v1:P1']}]}
        return t, oracle

    def test_actual_receipt_then_current_delivery_commit(self):
        t, o = self.record(); r = evaluate(t, o)
        self.assertEqual(r['qa01'][0]['status'], 'OBSERVED')
        self.assertEqual(r['qa03'][0]['status'], 'OBSERVED')
        self.assertTrue(r['qa02'][0]['correct'])

    def test_after_cutoff_endpoints(self):
        t, o = self.record()
        for e in t['events']:
            if e['kind'] in ('DISPLAY_PROXY', 'GOAL_COMPLETE'): e['observed_ns'] = 161
        r = evaluate(t, o)
        self.assertEqual(r['qa01'][0]['status'], 'NOT_OBSERVED')
        self.assertEqual(r['qa03'][0]['status'], 'NOT_OBSERVED')
        self.assertEqual(r['qa05']['known_api_cost_usd'], .5)

    def test_raw_input_unchanged(self):
        t, o = self.record(); before = copy.deepcopy(t); evaluate(t, o); self.assertEqual(before, t)

    def test_ambiguous_adoption_not_success(self):
        t, o = self.record(); t['events'].append(copy.deepcopy(next(e for e in t['events'] if e['kind'] == 'STATE_EFFECT_COMMIT')))
        self.assertFalse(evaluate(t, o)['qa02'][0]['correct'])

    def test_missing_deadline_is_not_zero_or_imputed_failure(self):
        t, o = self.record(); del t['observation_deadline_ns']; r = evaluate(t, o)
        self.assertIsNone(r['qa01'][0]['latency_ns']); self.assertIsNone(r['qa03'][0]['via_ns'])
        self.assertEqual(r['primary_measurement_eligibility'], 'NOT_CERTIFIED')

    def test_uncertified_raw_latency_not_primary_but_cost_preserved(self):
        t, o = self.record(); del t['host_clock_proof']; r = evaluate(t, o)
        s = summarize([r], 1)['OFFLINE:AAAA']
        self.assertEqual(s['qa01']['observed'], 0)
        self.assertEqual(s['raw_observation_diagnostics']['qa01']['observed'], 1)
        self.assertEqual(s['qa05']['known_total_usd'], .5)
        self.assertEqual(s['qa01']['planned'], 43)

    def test_ordinary_model_failure_not_technical_exclusion(self):
        t, o = self.record(); t['events'].append({'kind': 'WORKFLOW_FAILED', 'event_id': 'fail', 'observed_ns': 158, 'data': {'reason': 'actual model error'}})
        r = evaluate(t, o); self.assertEqual(r['primary_measurement_eligibility'], 'ELIGIBLE')
        self.assertEqual(summarize([r], 1)['OFFLINE:AAAA']['qa01']['observed'], 1)

    def test_technical_unjudgeable_not_semantic_incorrect(self):
        t, o = self.record(); t['events'] = [e for e in t['events'] if e.get('data', {}).get('effect_kind') != 'SEMANTIC_ADOPTION']
        s = summarize([evaluate(t, o)], 1)['OFFLINE:AAAA']['qa02']
        self.assertEqual(s['technically_unjudgeable'], 1)
        self.assertEqual(s['semantic_incorrect_judgments'], 0)
        self.assertIsNone(s['fixed_denominator_accuracy'])

    def test_all_missing_model_adoption_is_regular_wrong_set_when_technical_valid(self):
        t, o = self.record(); t['events'] = [e for e in t['events'] if e['kind'] != 'SEMANTIC_ADOPTION']
        r = evaluate(t, o); self.assertEqual(r['primary_measurement_eligibility'], 'ELIGIBLE')
        s = summarize([r], 1)['OFFLINE:AAAA']['qa02']
        self.assertEqual(s['semantic_incorrect_judgments'], 1)

    def test_handling_commit_cannot_cover_direct_publication(self):
        t, o = self.record(); next(e for e in t['events'] if e.get('event_id') == 'publication')['data']['data']['phase'] = 'HANDLING'
        r = evaluate(t, o)
        self.assertEqual(r['primary_measurement_eligibility'], 'NOT_CERTIFIED')
        self.assertTrue(any(x['kind'] == 'PUBLICATION_DELIVERY_WITHOUT_CURRENT_RECEIPT_PROOF' for x in r['deadline_audit']['violations']))

    def test_other_revision_commit_cannot_cover_publication(self):
        t, o = self.record(); next(e for e in t['events'] if e.get('event_id') == 'publication')['data']['data']['revision'] = 2
        next(e for e in t['events'] if e.get('event_id') == 'card-P')['data']['revision'] = 1
        self.assertEqual(evaluate(t, o)['primary_measurement_eligibility'], 'NOT_CERTIFIED')


class PublicationReceipts(unittest.TestCase):
    def record(self):
        return Integration().record()

    def without_delivery(self, t):
        t['events'] = [e for e in t['events'] if e.get('event_id') not in ('publication', 'goal-P')]

    def test_late_ack_preserves_actual_first_display_not_goal(self):
        t, o = self.record(); self.without_delivery(t)
        next(e for e in t['events'] if e['kind'] == 'PUBLICATION_HOST_RECEIPT')['host_monotonic_ns'] = 161
        r = evaluate(t, o)
        self.assertEqual(r['qa01'][0]['status'], 'OBSERVED'); self.assertEqual(r['qa03'][0]['status'], 'NOT_OBSERVED')
        self.assertEqual(r['deadline_audit']['publication_audit']['rows'][0]['receipt_status'], 'ACK_RECEIVED_AFTER_CUTOFF')
        self.assertEqual(r['deadline_audit']['status'], 'CONSISTENT')

    def test_missing_ack_never_invents_delivery(self):
        t, o = self.record(); self.without_delivery(t); t['host_display_receipts'] = []
        t['events'] = [e for e in t['events'] if e['kind'] != 'PUBLICATION_HOST_RECEIPT']
        r = evaluate(t, o); self.assertEqual(r['qa01'][0]['status'], 'OBSERVED')
        self.assertEqual(r['qa03'][0]['status'], 'NOT_OBSERVED')

    def test_duplicate_accepted_ack_unresolved(self):
        t, o = self.record(); t['events'].append(copy.deepcopy(next(e for e in t['events'] if e['kind'] == 'PUBLICATION_HOST_RECEIPT')))
        r = evaluate(t, o); self.assertEqual(r['primary_measurement_eligibility'], 'NOT_CERTIFIED')
        self.assertEqual(r['qa03'][0]['status'], 'NOT_OBSERVED')

    def test_unknown_ack_safely_rejected(self):
        t, o = self.record(); t['events'].append({'kind': 'PUBLICATION_RECEIPT_REJECTED', 'event_id': 'unknown',
            'data': {'receipt': {'publication_id': 'unknown'}, 'reason': 'UNKNOWN_DUPLICATE_OR_MALFORMED_PUBLICATION_RECEIPT'}})
        self.assertEqual(evaluate(t, o)['deadline_audit']['status'], 'CONSISTENT')

    def test_unknown_ack_consumption_is_not_certified(self):
        t, o = self.record(); t['events'].append({'kind': 'PUBLICATION_HOST_RECEIPT', 'event_id': 'unknown-consumed',
            'host_monotonic_ns': 153, 'data': {'publication_id': 'unknown', 'accepted': True}})
        self.assertEqual(evaluate(t, o)['primary_measurement_eligibility'], 'NOT_CERTIFIED')

    def test_duplicate_ack_safely_rejected_does_not_duplicate_state(self):
        t, o = self.record(); t['events'].append({'kind': 'PUBLICATION_RECEIPT_REJECTED', 'event_id': 'duplicate',
            'data': {'receipt': copy.deepcopy(t['host_display_receipts'][0]), 'reason': 'UNKNOWN_DUPLICATE_OR_MALFORMED_PUBLICATION_RECEIPT'}})
        self.assertEqual(evaluate(t, o)['qa03'][0]['status'], 'OBSERVED')

    def test_cancelled_receipt_wait_retains_raw_display(self):
        t, o = self.record(); self.without_delivery(t); t['host_display_receipts'] = []
        t['events'] = [e for e in t['events'] if e['kind'] != 'PUBLICATION_HOST_RECEIPT']
        t['events'].append({'kind': 'PUBLICATION_RECEIPT_TIMEOUT', 'event_id': 'wait-cancelled', 'data': {'publication_id': 'pub-P', 'reason': 'OBSERVATION_DEADLINE_CANCELLED'}})
        r = evaluate(t, o); self.assertEqual(r['qa01'][0]['status'], 'OBSERVED'); self.assertIsNone(r['qa03'][0]['via_ns'])

    def test_changed_revision_blocks_state_but_not_display_fact(self):
        t, o = self.record(); self.without_delivery(t)
        t['events'].append({'kind': 'PUBLICATION_REJECTED', 'event_id': 'stale', 'data': {'reason': 'STALE_GOAL_REVISION', 'publication_id': 'pub-P'}})
        r = evaluate(t, o); self.assertEqual(r['qa01'][0]['status'], 'OBSERVED'); self.assertEqual(r['qa03'][0]['status'], 'NOT_OBSERVED')

    def test_changed_authority_blocks_state_but_not_display_fact(self):
        t, o = self.record(); self.without_delivery(t)
        t['events'].append({'kind': 'PUBLICATION_REJECTED', 'event_id': 'withdrawn', 'data': {'reason': 'PUBLICATION_SOURCE_PERMISSION_CHANGED', 'publication_id': 'pub-P'}})
        self.assertEqual(evaluate(t, o)['qa03'][0]['status'], 'NOT_OBSERVED')

    def test_forged_receipt_observed_clock_not_accepted(self):
        t, o = self.record(); t['host_display_receipts'][0]['observed_ns'] = 149
        self.assertEqual(evaluate(t, o)['qa03'][0]['status'], 'NOT_OBSERVED')

    def test_duplicate_display_id_not_certified(self):
        t, o = self.record(); duplicate = copy.deepcopy(next(e for e in t['events'] if e['kind'] == 'DISPLAY_PROXY')); duplicate['event_id'] = 'duplicate-card'; t['events'].append(duplicate)
        self.assertEqual(evaluate(t, o)['primary_measurement_eligibility'], 'NOT_CERTIFIED')

    def test_late_delivery_commit_is_violation(self):
        t, o = self.record(); next(e for e in t['events'] if e['event_id'] == 'publication')['data']['linearization_ns'] = 160
        r = evaluate(t, o); self.assertEqual(r['deadline_audit']['status'], 'VIOLATION'); self.assertEqual(r['qa03'][0]['status'], 'NOT_OBSERVED')

    def test_unknown_publication_cannot_register_delivered_source(self):
        t, o = self.record(); t['events'].append({'kind': 'STATE_EFFECT_COMMIT', 'event_id': 'unknown-source',
            'data': {'effect_kind': 'SOURCE_REGISTER', 'linearization_ns': 158, 'deadline_ns': 160, 'data': {'publication_id': 'unknown', 'source_id': 'artifact'}}})
        self.assertEqual(evaluate(t, o)['deadline_audit']['status'], 'VIOLATION')

    def test_ack_wait_is_counted_as_via_after_zero_external_wait(self):
        t, o = self.record()
        t['source_nodes'].append({'id': 'external', 'start_ns': 100, 'end_ns': 147, 'parents': ['source:P'], 'role': 'EXTERNAL'})
        t['events'] += [{'kind': 'LOCAL_OPERATION_START', 'event_id': 'ack-start', 'observed_ns': 147,
            'parent_ids': ['external'], 'data': {'node_id': 'ack-wait', 'stage': 'publication_host_receipt_wait', 'role': 'VIA'}},
            {'kind': 'LOCAL_OPERATION_END', 'event_id': 'ack-end', 'observed_ns': 156,
            'parent_ids': ['external'], 'data': {'node_id': 'ack-wait', 'stage': 'publication_host_receipt_wait', 'role': 'VIA'}}]
        next(e for e in t['events'] if e['kind'] == 'GOAL_COMPLETE')['parent_ids'] = ['ack-wait']
        self.assertEqual(evaluate(t, o)['qa03'][0]['via_ns'], 10)

    def test_history_only_cannot_prove_current_goal_completion(self):
        t, o = self.record(); meta = next(e for e in t['events'] if e['event_id'] == 'publication')['data']['data']
        meta.update(complete=False, completion_ineligible_reason='STALE_GOAL_REVISION_AFTER_HOST_RECEIPT')
        r = evaluate(t, o); self.assertEqual(r['qa01'][0]['status'], 'OBSERVED'); self.assertEqual(r['qa03'][0]['status'], 'NOT_OBSERVED')

    def test_history_only_without_goal_is_legitimate_fact(self):
        t, o = self.record(); t['events'] = [e for e in t['events'] if e['kind'] != 'GOAL_COMPLETE']
        meta = next(e for e in t['events'] if e['event_id'] == 'publication')['data']['data']
        meta.update(complete=False, completion_ineligible_reason='AUTHORITY_CHANGED_AFTER_DISPLAY')
        r = evaluate(t, o); self.assertEqual(r['deadline_audit']['status'], 'CONSISTENT')
        self.assertTrue(r['deadline_audit']['publication_audit']['rows'][0]['delivery_state_proved'])
        self.assertFalse(r['deadline_audit']['publication_audit']['rows'][0]['completion_proved'])

    def test_history_only_cannot_register_new_delivered_source(self):
        t, o = self.record(); t['events'] = [e for e in t['events'] if e['kind'] != 'GOAL_COMPLETE']
        next(e for e in t['events'] if e['event_id'] == 'publication')['data']['data']['complete'] = False
        t['events'].append({'kind': 'STATE_EFFECT_COMMIT', 'event_id': 'bad-derived-source',
            'data': {'effect_kind': 'SOURCE_REGISTER', 'linearization_ns': 158, 'deadline_ns': 160, 'data': {'publication_id': 'pub-P', 'source_id': 'artifact'}}})
        self.assertEqual(evaluate(t, o)['deadline_audit']['status'], 'VIOLATION')

    def test_actual_agent_fact_state_is_not_usergoal_history(self):
        t, _ = self.record(); t['events'] = [e for e in t['events'] if e['kind'] != 'GOAL_COMPLETE']
        capture = next(e for e in t['events'] if e['kind'] == 'INPUT_CAPTURED')['data']
        capture.update(input_class='agent', event_class='RESULT', task_id='BG')
        display = next(e for e in t['events'] if e['kind'] == 'DISPLAY_PROXY')['data']
        display.update(phase='ACTUAL_AGENT_FACT', event_class='RESULT', task_id='BG')
        for key in ('goal_id', 'revision'): display.pop(key)
        meta = next(e for e in t['events'] if e['event_id'] == 'publication')['data']['data']
        for key in ('historical_display_recorded', 'complete', 'goal_id', 'revision', 'phase'): meta.pop(key)
        meta['class'] = 'RESULT'
        meta.update(task_id='BG', task_fact_state_recorded=True)
        row = audit(t)['publication_audit']['rows'][0]
        self.assertTrue(row['task_fact_state_proved']); self.assertFalse(row['historical_display_state_proved']); self.assertFalse(row['completion_proved'])
        meta['task_id'] = 'OTHER'
        self.assertFalse(audit(t)['publication_audit']['rows'][0]['task_fact_state_proved'])


class TechnicalAbort(unittest.TestCase):
    def record(self):
        t = Deadlines().record()
        t.update(technical_abort_reason='DEADLINE_CONTROL_REJECTED', technical_abort_requested_ns=150,
            cancellation_requested_ns=150, effective_admission_stop_ns=150, settlement_anchor_ns=150, settlement_deadline_ns=270)
        next(e for e in t['events'] if e['kind'] == 'STATE_EFFECT_COMMIT')['data']['linearization_ns'] = 139
        return t

    def test_abort_keeps_contract_and_stops_primary(self):
        r = audit(self.record())
        self.assertEqual(r['cutoff_ns'], 160); self.assertEqual(r['effective_admission_stop_ns'], 150)
        self.assertEqual(r['status'], 'UNRESOLVED'); self.assertTrue(r['campaign_stop_required'])

    def test_pending_previous_cutoff_cannot_be_reset_later(self):
        t = self.record(); t['pending_deadline_update_cutoffs_at_cancel'] = {'2': 140}
        t.update(effective_admission_stop_ns=140, settlement_anchor_ns=140, settlement_deadline_ns=260)
        self.assertFalse(any(v['kind'] == 'EFFECTIVE_STOP_NOT_EARLIEST_CUTOFF_OR_CANCELLATION' for v in audit(t)['violations']))
        t.update(effective_admission_stop_ns=150, settlement_anchor_ns=150, settlement_deadline_ns=270)
        self.assertTrue(any(v['kind'] == 'EFFECTIVE_STOP_NOT_EARLIEST_CUTOFF_OR_CANCELLATION' for v in audit(t)['violations']))

    def test_settlement_anchor_cannot_reset_to_contract_deadline(self):
        t = self.record(); t.update(settlement_anchor_ns=160, settlement_deadline_ns=280)
        self.assertTrue(any(v['kind'] == 'SETTLEMENT_ANCHOR_RESET_OR_MISMATCH' for v in audit(t)['violations']))

    def test_dispatch_after_abort_before_contract_is_blocked(self):
        t = self.record(); t['calls'] = [{'actual_api_sent': True, 'http_dispatch_registered_ns': 155}]
        self.assertTrue(any(v['kind'] == 'DISPATCH_OUTSIDE_VERIFIED_WINDOW' for v in audit(t)['violations']))

    def test_commit_after_abort_before_contract_is_violation(self):
        t = self.record(); next(e for e in t['events'] if e['kind'] == 'STATE_EFFECT_COMMIT')['data']['linearization_ns'] = 155
        self.assertEqual(audit(t)['status'], 'VIOLATION')

    def test_false_lastaccepted_alias_not_trusted(self):
        t = self.record(); t['last_accepted_observation_deadline_ns'] = 190
        self.assertTrue(any(v['kind'] == 'LAST_ACCEPTED_DEADLINE_ALIAS_MISMATCH' for v in audit(t)['violations']))

    def test_technical_failure_does_not_erase_actual_cost(self):
        t, o = Integration().record()
        t.update(technical_abort_reason='HOST_CLOCK_OR_DEADLINE_CONTROL_FAILURE', technical_abort_requested_ns=161)
        r = evaluate(t, o)
        self.assertEqual(r['primary_measurement_eligibility'], 'NOT_CERTIFIED')
        self.assertEqual(r['qa05']['known_api_cost_usd'], .5)

    def test_unarmed_initial_rejection_is_unknown_not_false_late_effect(self):
        t = self.record(); t['events'] = [e for e in t['events'] if e['kind'] != 'OBSERVATION_DEADLINE_ACK']
        t['last_accepted_observation_deadline_ns'] = None
        next(e for e in t['events'] if e['kind'] == 'STATE_EFFECT_COMMIT')['data']['deadline_ns'] = None
        r = audit(t)
        self.assertEqual(r['status'], 'UNRESOLVED')
        self.assertEqual(r['state_effect_checks'][0]['status'], 'UNARMED_BEFORE_TECHNICAL_ABORT')


if __name__ == '__main__':
    unittest.main(verbosity=2)
