"""Pure synthetic adapter checks. Never reads the formal corpus."""
import argparse
import copy
import io
import json
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

import report_adapter as a


def synthetic_plan():
    response_counts = [6, 5, 5, 5, 5, 5, 4, 4, 4]
    goal_counts = [6, 3, 2, 2, 2, 2, 2, 2, 2]
    binding_counts = [0, 2, 2, 2, 2, 2, 2, 2, 1]
    oracles, jobs = {}, []
    for pop, rule in a.POPULATIONS.items():
        oracles[pop] = {}
        for index, case in enumerate(rule['cases']):
            rc = response_counts[index] if pop == 'COMMON9' else 2
            gc = goal_counts[index] if pop == 'COMMON9' else 2 if index < 2 else 1
            bc = binding_counts[index] if pop == 'COMMON9' else 1
            oracles[pop][case] = {'responses': [{'response_id': case + '-R' + str(i), 'event_id': case + '-U' + str(i), 'required_content': 'synthetic only'} for i in range(rc)],
                                'goals': [{'id': case + '-G' + str(i), 'start_event': case + '-U0', 'member_events': [case + '-U0'], 'completion': 'synthetic only'} for i in range(gc)],
                                'bindings': [{'id': case + '-B' + str(i)} for i in range(bc)]}
            for rep in range(1, rule['reps'] + 1):
                for cfg in a.CONFIGS:
                    jobs.append(dict(population=pop, configuration=cfg, episode=case, rep=rep))
    return jobs, oracles


def record(oracles, key=('COMMON9', 'AAAA', 'S02', 1), eligible=True, value=0):
    pop, cfg, case, rep = key
    oracle = oracles[pop][case]
    result = dict(version='COMMON9-EVALUATOR-v3', evaluation_mode='strict', phase='FORMAL', configuration=cfg, episode=case, rep=rep,
                  run_id='-'.join(map(str, key)),
                  primary_measurement_eligibility='ELIGIBLE' if eligible else 'NOT_CERTIFIED', deadline_audit={'status': 'CONSISTENT' if eligible else 'UNRESOLVED'},
                  qa01=[dict(row, status='OBSERVED', latency_ns=value, accepted_callback_id='synthetic-display') for row in oracle['responses']],
                  qa02=[dict(row, status='ADOPTED', correct=True) for row in oracle['bindings']],
                  qa03=[dict(row, status='OBSERVED', via_ns=value, accepted_terminal_event_id='synthetic-goal') for row in oracle['goals']],
                  qa05={'known_api_cost_usd': '0.1', 'unknown_call_count': 0, 'unknown_reservation_usd': 0},
                  qa06={'status': 'OBSERVED', 'peak_concurrent_sum_bytes': 100, 'peak_same_sample': {'concurrent_sum_bytes': 100, 'processes': [{'role': 'python', 'rss_bytes': 60}, {'role': 'rust', 'rss_bytes': 40}]}})
    return {'identity': a.key_object(key), 'run_id': '-'.join(map(str, key)), 'provenance_valid': True, 'result': result,
            'result_ref': {'path': 'synthetic-result', 'sha256': 'synthetic', 'bytes': 1}, 'trace_ref': {'path': 'synthetic-trace', 'sha256': 'synthetic', 'bytes': 1}}


def closed_progress(records):
    return [dict(r['identity'], state=state) for r in records for state in ['STARTED', 'DISPATCHED', 'TERMINATED']]


class CoreChecks(unittest.TestCase):
    def test_fixed441_oracle_denominators_and_duplicate_plan(self):
        jobs, oracles = synthetic_plan()
        self.assertEqual(len(a.validate_plan(jobs, oracles)), 441)
        with self.assertRaises(a.InputError):
            a.validate_plan(jobs[:-1] + [jobs[0]], oracles)

    def test_episode_states_stay_distinct(self):
        q = {'progress_complete': True, 'controller_stopped': True, 'no_active_episode_processes': True}
        self.assertEqual(a.episode_state([], [], q), 'UNEXECUTED_VERIFIED')
        self.assertEqual(a.episode_state([], ['STARTED'], q), 'STARTED_UNCLOSED')
        self.assertEqual(a.episode_state([], ['STARTED', 'DISPATCHED', 'TERMINATED'], q), 'TERMINATED_TRACE_MISSING')
        self.assertEqual(a.episode_state([{'provenance_valid': False}], ['STARTED', 'DISPATCHED', 'TERMINATED'], q), 'CLOSED_RESULT_UNVERIFIED')

    def test_timing_zero_null_and_partition(self):
        rows = []
        for raw, expected in [({'status': 'OBSERVED', 'latency_ns': 0}, 'CERTIFIED_OBSERVED'),
                              ({'status': 'NOT_OBSERVED', 'latency_ns': None}, 'CERTIFIED_NOT_OBSERVED'),
                              ({'status': 'UNRESOLVED_EARLIEST_BOUNDARY', 'latency_ns': None}, 'CERTIFIED_UNRESOLVED')]:
            state, reason = a.obligation_state('qa01', 'CLOSED_CERTIFIED', raw)
            self.assertEqual(state, expected)
            rows.append({'qa': 'qa01', 'reporting_state': state, 'reason': reason, 'raw_value': raw})
        summary = a.timing_summary(rows, 3)
        self.assertEqual((summary['sum_ns'], summary['mean_ms'], summary['observed_count']), (0, 0, 1))
        empty = a.timing_summary([], 0)
        self.assertIsNone(empty['sum_ns'])
        self.assertIsNone(empty['mean_ms'])
        for value in [None, -1, float('nan'), float('inf'), True, 0.5]:
            self.assertEqual(a.obligation_state('qa01', 'CLOSED_CERTIFIED', {'status': 'OBSERVED', 'latency_ns': value})[1], 'ABI_CONTRADICTION')

    def test_binding_categories_and_lower_bound(self):
        rows = []
        for raw, expected in [({'status': 'ADOPTED', 'correct': True}, 'KNOWN_CORRECT'), ({'status': 'ADOPTED', 'correct': False}, 'ADOPTED_INCORRECT'),
                              ({'status': 'NOT_ADOPTED', 'correct': False}, 'NOT_ADOPTED'), ({'status': 'UNRESOLVED_OR_LATE_ADOPTION', 'correct': False}, 'TECHNICALLY_UNJUDGEABLE_ADOPTION')]:
            state, _ = a.obligation_state('qa02', 'CLOSED_CERTIFIED', raw)
            self.assertEqual(state, expected)
            rows.append({'reporting_state': state})
        result = a.binding_summary(rows, 4)
        self.assertEqual(result['known_correct_over_planned_lower_bound'], 0.25)
        self.assertIsNone(result['fixed_denominator_accuracy'])

    def test_micro_average_and_uncertified_raw_diagnostic(self):
        rows = [{'qa': 'qa01', 'reporting_state': 'CERTIFIED_OBSERVED', 'raw_value': {'latency_ns': n}} for n in [0, 0, 9000000]]
        self.assertEqual(a.timing_summary(rows, 3)['mean_ms'], 3)
        state, _ = a.obligation_state('qa01', 'CLOSED_TECHNICALLY_UNCERTIFIED', {'status': 'OBSERVED', 'latency_ns': 100})
        self.assertEqual(state, 'TECHNICALLY_UNCERTIFIED')

    def test_explicit_artifact_hash_root_size_limits(self):
        with tempfile.TemporaryDirectory(prefix='adapter-synthetic-') as temp:
            root = Path(temp).resolve()
            path = root / 'artifact.json'
            path.write_text('{"value":0}')
            ref = {'path': str(path), 'sha256': a.sha(path.read_bytes()), 'bytes': path.stat().st_size}
            self.assertEqual(a.ArtifactReader(root).read(ref), {'value': 0})
            for changed in [dict(ref, sha256='0' * 64), dict(ref, bytes=0), dict(ref, path='/outside/explicit.json')]:
                with self.assertRaises(a.InputError):
                    a.ArtifactReader(root).read(changed)
            with self.assertRaises(a.InputError):
                a.ArtifactReader(root, file_limit=1).read(ref)

    def test_association_requires_exact_population_oracle_and_sources(self):
        _, oracles = synthetic_plan()
        r = record(oracles)
        ref = {'path': '/synthetic/oracle.json', 'sha256': 'oracle', 'bytes': 1}
        freeze = {'binary': {'sha256': 'binary'}, 'sources': [{'relativePath': 'eval.py', 'sha256': 'source'}]}
        assoc = {'version': 'JOINT441_RESULT_ASSOCIATION_v1', 'phase': 'FORMAL', 'mode': 'strict', 'evaluator_version': 'COMMON9-EVALUATOR-v3',
                 'formal_root': '/synthetic', 'formal_freeze_sha256': 'freeze', 'binary_sha256': 'binary', 'identity': r['identity'], 'run_id': r['run_id'],
                 'result_ref': r['result_ref'], 'trace_ref': r['trace_ref'], 'oracle_ref': ref, 'source_sha256': {'eval.py': 'source'},
                 'recorded_analysis_argv': ['synthetic-declared-never-executed'], 'recorded_by': 'synthetic independent recorder', 'recorded_at': 'synthetic'}
        self.assertTrue(a.validate_association(assoc, r, freeze, {'sha256': 'freeze'}, ref, Path('/synthetic')))
        for field, value in [('source_sha256', {}), ('oracle_ref', dict(ref, sha256='another')), ('mode', 'legacy_diagnostic'), ('binary_sha256', 'other')]:
            bad = dict(assoc, **{field: value})
            with self.assertRaises(a.InputError):
                a.validate_association(bad, r, freeze, {'sha256': 'freeze'}, ref, Path('/synthetic'))


class ReviewFixChecks(unittest.TestCase):
    def test_exact_unresolved_status_and_rep_types(self):
        for qa, status, field in [('qa01', 'UNRESOLVED_EARLIEST_BOUNDARY', 'latency_ns'), ('qa03', 'UNRESOLVED', 'via_ns')]:
            self.assertEqual(a.obligation_state(qa, 'CLOSED_CERTIFIED', {'status': status, field: None})[0], 'CERTIFIED_UNRESOLVED')
            for unsupported in ['UNRESOLVED_OTHER', 'UNRESOLVED_EARLIEST', status + '_invented']:
                self.assertEqual(a.obligation_state(qa, 'CLOSED_CERTIFIED', {'status': unsupported, field: None})[1], 'ABI_CONTRADICTION')
        jobs, oracles = synthetic_plan()
        jobs[0]['rep'] = True
        with self.assertRaises(a.InputError):
            a.validate_plan(jobs, oracles)
        self.assertNotEqual(a.identity(dict(population='COMMON9', configuration='AAAA', episode='S02', rep=True)), ('COMMON9', 'AAAA', 'S02', 1))

    def test_canonical_json_pointers(self):
        self.assertEqual(a.get_pointer({'a/b': [7]}, '/a~1b/0'), 7)
        self.assertEqual(a.get_pointer([1], ''), [1])
        for pointer in ['/-1', '/01', '/+0', '/1', '/00', '/~2']:
            with self.assertRaises(a.InputError):
                a.get_pointer([1], pointer)

    def test_bad_identity_declaration_and_progress_cannot_enter_pairs(self):
        jobs, oracles = synthetic_plan()
        for kind in ['run_id', 'population', 'rep', 'declaration', 'duplicate_terminated', 'unstarted_terminated']:
            r = record(oracles)
            progress = closed_progress([r])
            if kind == 'run_id':
                r['result']['run_id'] = 'wrong-run'
            elif kind == 'population':
                r['result']['population'] = 'COMMON_SUPPLEMENT6'
            elif kind == 'rep':
                r['result']['rep'] = True
            elif kind == 'declaration':
                r['result']['qa03'][0]['member_events'] = ['other-stimulus']
            else:
                progress = [dict(r['identity'], state=s) for s in (['STARTED', 'DISPATCHED', 'TERMINATED', 'TERMINATED'] if kind == 'duplicate_terminated' else ['TERMINATED'])]
            report, rows = a.build_report(jobs, oracles, [r], progress, {'progress_complete': True})
            self.assertEqual(report['status'], 'INVALID_REPORT_INPUT', kind)
            self.assertEqual(rows['planned-episodes.jsonl'][0]['state'], 'IDENTITY_OR_SOURCE_CONFLICT', kind)
            selected = [o for o in rows['obligations.jsonl'] if o['run_id'] == r['run_id']]
            self.assertTrue(all(o['reporting_state'] == 'EVIDENCE_UNAVAILABLE' for o in selected), kind)
            self.assertFalse(any(p.get('certified_intersection') for p in rows['paired.jsonl']), kind)
            self.assertEqual(report['populations'][0]['configurations'][0]['qa05']['known_expenditure_usd'], '0.1')


class ProjectionChecks(unittest.TestCase):
    def test_empty_fixed_partitions_conditions_and_unreviewed(self):
        jobs, oracles = synthetic_plan()
        report, rows = a.build_report(jobs, oracles, [], [], {'progress_complete': True, 'controller_stopped': True, 'no_active_episode_processes': True})
        self.assertEqual(report['planned_totals'], dict(episodes=441, responses=1757, bindings=651, goals=973))
        self.assertEqual(len(rows['planned-episodes.jsonl']), 441)
        self.assertEqual(len(rows['obligations.jsonl']), 3381)
        self.assertEqual(len(rows['paired.jsonl']), 4872)
        for pop in report['populations']:
            self.assertEqual(len(pop['configurations']), 7)
            for cfg in pop['configurations']:
                for qa in ['qa01', 'qa02', 'qa03']:
                    self.assertEqual(sum(cfg[qa]['partition'].values()), a.POPULATIONS[pop['population']][qa])
                self.assertIsNone(cfg['qa01']['mean_ms'])
                self.assertIsNone(cfg['qa05']['known_expenditure_usd'])
                self.assertIsNone(cfg['qa06']['peak_concurrent_sum_bytes'])
        self.assertEqual({r['condition'] for r in rows['case-repetition.jsonl'] if r.get('population') == 'COMMON_SUPPLEMENT6' and r['kind'] == 'CASE_REPETITION'}, set(a.CONDITIONS.values()))
        self.assertEqual(len(report['prediction_worklist']), 24)
        self.assertTrue(all(p['judgment_status'] == 'UNREVIEWED' and p['judgment'] is None for p in report['prediction_worklist']))

    def test_direct_BC_certified_intersection_and_semantic_not_filter(self):
        jobs, oracles = synthetic_plan()
        records = [record(oracles, ('COMMON9', cfg, 'S02', 1), eligible=cfg != 'AAAA', value=n) for cfg, n in [('AAAA', 1), ('BAAA', 7), ('CAAA', 17)]]
        review_key = ('COMMON9', 'BAAA', 'S02', 1, 'qa01', 'S02-R0')
        report, rows = a.build_report(jobs, oracles, records, closed_progress(records), {'progress_complete': True}, reviews={review_key: {'body': {'status': 'FAIL', 'reason': 'synthetic oracle/input disagreement'}, 'approval': {'status': 'NOT_APPLICABLE'}}}, raw_pairs={'legacy': 'two nonnull but uncertified'})
        pairs = [p for p in rows['paired.jsonl'] if p['qa'] == 'qa01' and p['episode'] == 'S02' and p['rep'] == 1 and p['obligation_id'] == 'S02-R0']
        bc = next(p for p in pairs if (p['left'], p['right']) == ('BAAA', 'CAAA'))
        self.assertEqual(bc['delta_ns_right_minus_left'], 10)
        self.assertTrue(bc['certified_intersection'])
        ab = next(p for p in pairs if (p['left'], p['right']) == ('AAAA', 'BAAA'))
        self.assertIsNone(ab['delta_ns_right_minus_left'])
        self.assertEqual(report['pairs']['raw_analyzer_paired_diagnostics']['legacy'], 'two nonnull but uncertified')
        self.assertEqual(report['populations'][0]['configurations'][1]['qa01']['observed_count'], 6)
        self.assertEqual(report['populations'][0]['configurations'][0]['qa05']['known_expenditure_usd'], '0.1')

    def test_duplicate_and_unknown_obligation_preserved_invalid(self):
        jobs, oracles = synthetic_plan()
        r = record(oracles)
        r['result']['qa01'][0]['response_id'] = 'UNKNOWN'
        report, rows = a.build_report(jobs, oracles, [r, copy.deepcopy(r)], [], {})
        self.assertEqual(report['status'], 'INVALID_REPORT_INPUT')
        self.assertEqual(rows['planned-episodes.jsonl'][0]['state'], 'IDENTITY_OR_SOURCE_CONFLICT')
        self.assertEqual(len(rows['obligations.jsonl']), 3381)

    def test_fixed_bundle_cost_missing_uncertified_unknown(self):
        eps = [{'state': 'CLOSED_CERTIFIED', 'qa05': {'known_api_cost_usd': '0.1', 'unknown_call_count': 0, 'unknown_reservation_usd': '0'}} for _ in range(9)]
        self.assertEqual(a.cost_summary(eps, 9, 1)['representative_fixed_bundle_mean_usd'], '0.9')
        for changed in [eps[:-1], [dict(eps[0], state='CLOSED_TECHNICALLY_UNCERTIFIED')] + eps[1:], [dict(eps[0], qa05={'known_api_cost_usd': '0.1', 'unknown_call_count': 1, 'unknown_reservation_usd': '0.3'})] + eps[1:]]:
            result = a.cost_summary(changed, 9, 1)
            self.assertIsNone(result['representative_fixed_bundle_mean_usd'])
            self.assertIsNotNone(result['known_expenditure_usd'])

    def test_same_sample_RSS_ties_not_role_or_episode_sum(self):
        eps = [dict(identity={'synthetic': i}, run_id=str(i), state='CLOSED_CERTIFIED', qa06={'status': 'OBSERVED', 'peak_concurrent_sum_bytes': 100, 'peak_same_sample': {'concurrent_sum_bytes': 100, 'processes': [{'rss_bytes': n}, {'rss_bytes': 100-n}]}}) for i, n in enumerate([80, 20])]
        result = a.rss_summary(eps, 2)
        self.assertEqual(result['peak_concurrent_sum_bytes'], 100)
        self.assertEqual(len(result['tied_same_sample_refs']), 2)
        self.assertTrue(result['comparison_complete'])
        eps[1]['qa06']['status'] = 'INCOMPLETE'
        self.assertEqual(a.rss_summary(eps, 2)['maximum_status'], 'OBSERVED_LOWER_BOUND_DIAGNOSTIC')

    def test_call_uuid_dedup_rates_unknown_late_settlement_all_cost(self):
        key = dict(population='COMMON9', configuration='AAAA', episode='S02', rep=1)
        raw = dict(call_uuid='u', actual_api_sent=True, role='VIA', stage='semantic', scope_root='/formal', cost_usd='0.01', cost_state='PROVIDER_USAGE_X_STANDARD_PRICE_NOT_INVOICE', raw_usage={'prompt_tokens': 10}, discarded=True, price={'input_per_million': .4, 'output_per_million': 1.6})
        cloud = a.project_call(raw, key, 'run', {'file': 'synthetic-a'})
        duplicate = a.project_call(raw, key, 'run', {'file': 'synthetic-b'})
        local = a.project_call(dict(evidence_kind='LOCAL_MODEL', actual_api_sent=False, cost_usd=0), key, 'run', {'file': 'synthetic-local'})
        admin = a.project_call(dict(actual_api_sent=False, cost_usd=0, cost_state='NO_REQUEST_SENT_NO_API_CHARGE'), key, 'run', {'file': 'synthetic-unsent'})
        ledger = {'limit_usd': 40, 'calls': {'u': {'amount_usd': '0.01', 'state': 'USAGE_RECORDED', 'scope_root': '/formal'}, 'prior': {'amount_usd': '.7', 'state': 'USAGE_RECORDED', 'scope_root': '/prior'}, 'unknown': {'amount_usd': '.2', 'state': 'UNKNOWN', 'scope_root': '/prior'}}}
        result = a.reconcile_calls([cloud, duplicate, local, admin], ledger, '/formal')
        self.assertEqual((result['input_record_count'], result['deduplicated_record_count'], result['actual_cloud_dispatch_count']), (4, 3, 1))
        self.assertEqual(result['global']['known_usd'], '0.71')
        self.assertEqual(result['global']['unknown_usd'], '0.2')
        self.assertFalse(result['reconciliation_complete'])
        self.assertEqual(len(result['calls'][0]['occurrences']), 2)
        bad = a.project_call(dict(raw, cost_usd='.02'), key, 'run', {'file': 'synthetic-bad'})
        self.assertTrue(a.reconcile_calls([cloud, bad], ledger, '/formal')['conflicts'])
        unresolved = dict(cloud, cost_usd=None, cost_state='UNKNOWN_RESERVATION_RETAINED')
        self.assertTrue(any(d['kind'] == 'LATE_LEDGER_SETTLEMENT_ABSENT_FROM_TRACE' for d in a.reconcile_calls([unresolved], ledger, '/formal')['discrepancies']))
        different = a.project_call(dict(raw, price={'input_per_million': 1, 'output_per_million': 2}), key, 'run', {'file': 'synthetic-other-rate'})
        self.assertTrue(a.reconcile_calls([different], ledger, '/formal')['groups'][0]['different_or_missing_fixed_comparison_rate'])


class Stage02FixChecks(unittest.TestCase):
    def test_exact_lifecycle_complete_conflicts_and_recoverable_incomplete(self):
        jobs, oracles = synthetic_plan()
        r = record(oracles)
        for states in [[], ['DISPATCHED'], ['STARTED'], ['STARTED','TERMINATED'], ['STARTED','DISPATCHED','DISPATCHED','TERMINATED'], ['STARTED','TERMINATED','DISPATCHED'], ['STARTED','UNKNOWN','TERMINATED']]:
            progress = [dict(r['identity'],state=state) for state in states]
            report, rows = a.build_report(jobs,oracles,[r],progress,{'progress_complete':True})
            self.assertEqual(rows['planned-episodes.jsonl'][0]['state'],'IDENTITY_OR_SOURCE_CONFLICT',states)
            self.assertFalse(any(p.get('certified_intersection') for p in rows['paired.jsonl']))
            self.assertEqual(report['populations'][0]['configurations'][0]['qa05']['known_expenditure_usd'],'0.1')
            if len(states)==len(set(states)) and all(s in ['STARTED','DISPATCHED','TERMINATED'] for s in states) and [s for s in ['STARTED','DISPATCHED','TERMINATED'] if s in states]==states:
                self.assertEqual(a.episode_state([r],states,{'progress_complete':False}),'CLOSED_RESULT_UNVERIFIED')
        self.assertEqual(a.episode_state([r],['STARTED','DISPATCHED','TERMINATED'],{'progress_complete':True}),'CLOSED_CERTIFIED')
        self.assertEqual(a.episode_state([],['STARTED','DISPATCHED'],{'progress_complete':True}),'STARTED_UNCLOSED')

    def test_unavailable_vs_contradictory_artifacts_and_missing_association(self):
        _, oracles = synthetic_plan()
        with tempfile.TemporaryDirectory(prefix='adapter-unavailable-synthetic-') as temp:
            root=Path(temp).resolve()
            missing={'path':str(root/'missing.json'),'sha256':'0'*64,'bytes':1}
            for artifact in ['trace','result','association']:
                r=record(oracles)
                r.update(unavailable_evidence=[],conflicts=[],provenance_valid=False)
                self.assertIsNone(a.read_record_artifact(a.ArtifactReader(root),{artifact+'_ref':missing},artifact,r))
                self.assertFalse(r['conflicts'])
                self.assertEqual(r['unavailable_evidence'][0]['artifact'],artifact)
                expected='CLOSED_RESULT_UNVERIFIED' if artifact=='association' else 'TERMINATED_TRACE_MISSING'
                self.assertEqual(a.episode_state([r],['STARTED','DISPATCHED','TERMINATED'],{'progress_complete':True}),expected)
            r=record(oracles)
            r.update(unavailable_evidence=[],conflicts=[],provenance_valid=False)
            for artifact in ['trace','result']:
                self.assertIsNone(a.read_record_artifact(a.ArtifactReader(root),{},artifact,r))
            self.assertEqual(a.episode_state([r],['STARTED','DISPATCHED','TERMINATED'],{'progress_complete':True}),'TERMINATED_TRACE_MISSING')
            path=root/'present.json'
            path.write_text('{}')
            r.update(unavailable_evidence=[],conflicts=[])
            bad=dict(missing,path=str(path),bytes=2)
            a.read_record_artifact(a.ArtifactReader(root),{'trace_ref':bad},'trace',r)
            self.assertTrue(r['conflicts'])
            self.assertFalse(r['unavailable_evidence'])
            self.assertEqual(a.episode_state([r],['STARTED','DISPATCHED','TERMINATED'],{'progress_complete':True}),'IDENTITY_OR_SOURCE_CONFLICT')

    def test_saved_RSS_arithmetic_cannot_certify_missing_or_mismatched_sample(self):
        _,oracles=synthetic_plan()
        q=record(oracles)['result']['qa06']
        for altered in [dict(q,peak_concurrent_sum_bytes=None),dict(q,peak_concurrent_sum_bytes=-1),dict(q,peak_same_sample=None),dict(q,peak_same_sample={'concurrent_sum_bytes':99,'processes':[{'rss_bytes':99}]}),dict(q,peak_same_sample={'concurrent_sum_bytes':100,'processes':[{'rss_bytes':99}]})]:
            ep=dict(identity={'synthetic':'rss'},state='CLOSED_CERTIFIED',qa06=altered)
            result=a.rss_summary([ep],1)
            self.assertFalse(result['comparison_complete'])
            self.assertNotEqual(result['maximum_status'],'COMPLETE_OBSERVED_MAXIMUM')
            self.assertTrue(result['unverified_peak_records'])
            self.assertEqual(result['unverified_peak_records'][0]['recorded_peak_concurrent_sum_bytes'],altered['peak_concurrent_sum_bytes'])
        jobs,oracles=synthetic_plan()
        records=[record(oracles,('COMMON9',cfg,'S02',1)) for cfg in ['AAAA','BAAA']]
        records[1]['result']['qa06']['peak_same_sample']=None
        report,rows=a.build_report(jobs,oracles,records,closed_progress(records),{'progress_complete':True})
        p=next(p for p in rows['paired.jsonl'] if p['qa']=='qa06' and p['episode']=='S02' and p['rep']==1 and p['left']=='AAAA' and p['right']=='BAAA')
        self.assertFalse(p['comparison_complete'])
        self.assertIsNone(p['comparable_delta_right_minus_left'])
        self.assertEqual(report['status'],'INVALID_REPORT_INPUT')

    def test_repetition_bundle_metrics_fixed_counts_and_micro_means(self):
        jobs,oracles=synthetic_plan()
        records=[record(oracles,('COMMON9','AAAA',case,1),value=value) for case,value in [('S02',0),('S08',9)]]
        _,rows=a.build_report(jobs,oracles,records,closed_progress(records),{'progress_complete':True})
        bundles=[r for r in rows['case-repetition.jsonl'] if r['kind']=='FIXED_REPETITION_BUNDLE']
        b=next(r for r in bundles if r['population']=='COMMON9' and r['configuration']=='AAAA' and r['rep']==1)
        self.assertEqual([b[q]['planned'] for q in ['qa01','qa02','qa03']],[43,15,23])
        self.assertEqual((b['qa01']['sum_ns'],b['qa01']['observed_count']),(45,11))
        self.assertEqual(b['qa01']['mean_ms'],45/11/1e6)
        for bundle in bundles:
            expected=[43,15,23] if bundle['population']=='COMMON9' else [12,6,8]
            self.assertEqual([bundle[q]['planned'] for q in ['qa01','qa02','qa03']],expected)
            self.assertTrue(all(sum(bundle[q]['partition'].values())==bundle[q]['planned'] for q in ['qa01','qa02','qa03']))

    def test_later_disposition_only_annotation_dedups_financial_call(self):
        key=dict(population='COMMON9',configuration='AAAA',episode='S02',rep=1)
        raw=dict(call_uuid='u',actual_api_sent=True,cost_usd='0.1',cost_state='USAGE_RECORDED',scope_root='/formal')
        old=a.project_call(raw,key,'run',{'file':'earlier-global'})
        later=a.project_call(dict(raw,discarded_after_candidate_end=True),key,'run',{'file':'later-trace'})
        ledger={'limit_usd':40,'calls':{'u':{'scope_root':'/formal','amount_usd':'0.1','state':'USAGE_RECORDED'}}}
        result=a.reconcile_calls([old,later],ledger,'/formal')
        self.assertFalse(result['conflicts'])
        self.assertEqual(result['deduplicated_record_count'],1)
        self.assertEqual(result['unique_formal_call_known_usd'],'0.1')
        call=result['calls'][0]
        self.assertEqual(call['recorded_work_disposition'],{'discarded_after_candidate_end':True})
        self.assertEqual(call['work_disposition_occurrences'][0]['recorded_fields'],{})
        self.assertEqual(call['work_disposition_occurrences'][1]['recorded_fields'],{'discarded_after_candidate_end':True})
        false=a.project_call(dict(raw,discarded_after_candidate_end=False),key,'run',{'file':'explicit-false'})
        result=a.reconcile_calls([old,false,later],ledger,'/formal')
        self.assertFalse(result['conflicts'])
        self.assertTrue(result['disposition_annotation_disagreements'])


class OutputChecks(unittest.TestCase):
    def test_immutable_all_outputs_hashed_without_model_bodies(self):
        jobs, oracles = synthetic_plan()
        report, rows = a.build_report(jobs, oracles, [], [], {})
        with tempfile.TemporaryDirectory(prefix='adapter-output-synthetic-') as temp:
            root = Path(temp).resolve()
            output = root / 'new'
            manifest = a.write_outputs(output, report, rows, ['synthetic-only-not-real-CLI'])
            self.assertEqual(len(manifest['outputs']), 8)
            self.assertIn('report.md', manifest['outputs'])
            for name, reference in manifest['outputs'].items():
                self.assertEqual(a.sha((output / name).read_bytes()), reference['sha256'])
                self.assertEqual((output / name).stat().st_size, reference['bytes'])
            with self.assertRaises(a.InputError):
                a.write_outputs(output, report, rows, [])
            with self.assertRaises(a.InputError):
                a.write_outputs(root / '..' / 'escape', report, rows, [])
            link = root / 'link'
            link.symlink_to(root, target_is_directory=True)
            with self.assertRaises(a.InputError):
                a.write_outputs(link / 'escape', report, rows, [])
            text = (output / 'report.md').read_text()
            self.assertIn('—', text)
            self.assertEqual(text.count('## COMMON'), 2)

    def test_reader_no_nonfinite_json_and_hard_limits(self):
        with tempfile.TemporaryDirectory(prefix='adapter-bad-synthetic-') as temp:
            root = Path(temp).resolve()
            path = root / 'bad.json'
            path.write_text('{"x":NaN}')
            ref = dict(path=str(path), sha256=a.sha(path.read_bytes()), bytes=path.stat().st_size)
            with self.assertRaises(a.InputError):
                a.ArtifactReader(root).read(ref)
            with self.assertRaises(a.InputError):
                a.ArtifactReader(root, total_limit=1).read(ref)


class LoaderChecks(unittest.TestCase):
    def test_synthetic_explicit_full_loader_and_QA04_final_roles(self):
        jobs, oracles = synthetic_plan()
        with tempfile.TemporaryDirectory(prefix='adapter-loader-synthetic-') as temp:
            root = Path(temp).resolve()
            def save(name, value, raw=False):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                data = value.encode() if raw else a.json_bytes(value)
                path.write_bytes(data)
                return {'path': str(path), 'sha256': a.sha(data), 'bytes': len(data)}
            oracle_refs = {}
            sources = []
            for pop, relative in [('COMMON9', 'benchmark/architecture/common9_poc_20261019/oracles.json'), ('COMMON_SUPPLEMENT6', 'benchmark/architecture/common_supplement6_20261019/oracles.json')]:
                ref = save('frozen/source/' + relative, oracles[pop])
                oracle_refs[pop] = ref
                sources.append(dict(relativePath=relative, frozen_path=ref['path'], sha256=ref['sha256']))
            for i in range(110):
                relative = 'synthetic-source-' + str(i) + '.txt'
                ref = save('frozen/source/' + relative, 'synthetic only', raw=True)
                sources.append(dict(relativePath=relative, frozen_path=ref['path'], sha256=ref['sha256']))
            freeze = {'version': 'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1', 'jobs': jobs, 'sources': sources, 'binary': {'sha256': '0'*64}, 'ledger_path': str(root / 'mutable-ledger.json')}
            freeze_ref = save('freeze.json', freeze)
            qa04 = {'accepted_independent_changes': 42, 'combined_regression_episodes': 630, 'actual_provider_api_calls': 0,
                    'rows': [dict(configuration=cfg, task='C0'+str(i)) for cfg in a.CONFIGS for i in range(1,7)],
                    'all_counts_by_configuration': {cfg:[1,1,3,3,1,2] for cfg in a.CONFIGS}, 'mean_changed_operational_modules': {cfg:11/6 for cfg in a.CONFIGS}}
            qa_refs = {'report': save('approval/qa04-report.json', qa04), 'classification': save('approval/qa04-count.json', {'synthetic': True}), 'raw630': save('approval/qa04-raw.json', {'synthetic': True})}
            review = {'status':'PASS_QA04_FUNCTIONAL_AND_MODULE_ACCEPTANCE_WITH_RETAINED_RSS_DIAGNOSTICS', 'input_sha256': {'report_json':qa_refs['report']['sha256'], 'count_classification_v2':qa_refs['classification']['sha256'], 'raw630_audit':qa_refs['raw630']['sha256']}}
            qa_refs['final_review'] = save('approval/qa04-review.json', review)
            qa_refs['checkpoint'] = save('approval/qa04-checkpoint.json', {'status':'STABLE_QA04_WRITES_CLOSED','writes_closed': True, 'report_json':qa_refs['report'], 'count_classification':qa_refs['classification'], 'independent_reviews':{'raw630':qa_refs['raw630'], 'final_qa04_acceptance':qa_refs['final_review']}})
            approval = {'formal_freeze_sha256':freeze_ref['sha256'], 'artifacts':[dict(role=k, original_path=v['path'], copied_path=v['path'], sha256=v['sha256'], bytes=v['bytes'], status='historical-label-retained') for k,v in qa_refs.items()]}
            approval_ref = save('approval/APPROVAL-BINDING.json', approval)
            r = record(oracles)
            result_ref = save('analysis/result.json', r['result'])
            trace_ref = save('raw/runs/'+r['run_id']+'/trace.json', dict(phase='FORMAL', configuration='AAAA', episode='S02', rep=1, run_id=r['run_id'], binary_sha256='0'*64, candidate_input_file=str(root/'raw/runs'/r['run_id']/'candidate-input.json'), returncode=0, end_ns=10, calls=[]))
            association = {'version':'JOINT441_RESULT_ASSOCIATION_v1', 'phase':'FORMAL','mode':'strict','evaluator_version':'COMMON9-EVALUATOR-v3', 'formal_root':str(root), 'formal_freeze_sha256':freeze_ref['sha256'], 'binary_sha256':'0'*64, 'identity':r['identity'], 'run_id':r['run_id'], 'result_ref':result_ref, 'trace_ref':trace_ref, 'oracle_ref':oracle_refs['COMMON9'], 'source_sha256':{s['relativePath']:s['sha256'] for s in sources}, 'recorded_by':'synthetic independent recorder', 'recorded_at':'synthetic', 'recorded_analysis_argv':['synthetic-never-executed']}
            assoc_ref = save('analysis/association.json', association)
            result_index = save('analysis/index.jsonl', __import__('json').dumps(dict(identity=r['identity'],run_id=r['run_id'],result_ref=result_ref,trace_ref=trace_ref,association_ref=assoc_ref))+'\n', raw=True)
            progress_ref = save('snapshots/progress.jsonl', ''.join(json.dumps(dict(r['identity'],state=state))+'\n' for state in ['STARTED','DISPATCHED','TERMINATED']), raw=True)
            progress_ref['snapshot_of'] = str(root/'campaign-progress.jsonl')
            status_ref = save('snapshots/status.json', {'planned':441,'completed':1})
            status_ref['snapshot_of'] = str(root/'status.json')
            ledger_ref = save('snapshots/ledger.json', {'limit_usd':40,'calls':{}})
            ledger_ref['snapshot_of'] = freeze['ledger_path']
            evidence = save('snapshots/stopped.txt', 'synthetic stopped evidence', raw=True)
            q_ref = save('snapshots/quiescence.json', {'version':'JOINT441_QUIESCENT_SNAPSHOT_v1','controller_state':'COMPLETED','no_active_episode_processes':True,'progress_complete':True,'evidence_refs':[evidence], 'progress_snapshot_sha256':progress_ref['sha256'],'status_snapshot_sha256':status_ref['sha256'],'ledger_snapshot_sha256':ledger_ref['sha256']})
            supplied = {'version':'JOINT441_QUANTITATIVE_INPUT_v1','formal_root':str(root),'artifacts':dict(freeze=freeze_ref, approval=approval_ref,quiescence=q_ref, progress=progress_ref,status=status_ref,ledger=ledger_ref,result_index=result_index),'oracles':oracle_refs,'qa04':{k:{'approval_role':k,'ref':v} for k,v in qa_refs.items()}}
            manifest = save('input.json',supplied)
            with patch.dict(a.QA04_HASHES,{k:v['sha256'] for k,v in qa_refs.items()},clear=True):
                kwargs = a.load_inputs(Path(manifest['path']),manifest['sha256'],expected_freeze=freeze_ref['sha256'],expected_approval=approval_ref['sha256'])
                self.assertTrue(kwargs['records'][0]['provenance_valid'])
                report,rows = a.build_report(**kwargs)
                self.assertEqual(rows['planned-episodes.jsonl'][0]['state'],'CLOSED_CERTIFIED')
                self.assertEqual(report['qa04_reference']['numerator'],11)
                for artifact in ['trace','result','association','placeholder','wrong_hash']:
                    entry=dict(identity=r['identity'],run_id=r['run_id'],result_ref=result_ref,trace_ref=trace_ref,association_ref=assoc_ref)
                    moved=None
                    if artifact in ['trace','result','association']:
                        ref={'trace':trace_ref,'result':result_ref,'association':assoc_ref}[artifact]
                        original=Path(ref['path'])
                        moved=original.with_suffix('.unavailable')
                        original.rename(moved)
                    elif artifact=='placeholder':
                        entry=dict(identity=r['identity'])
                    else:
                        entry['trace_ref']=dict(trace_ref,sha256='0'*64)
                    try:
                        index=save('analysis/variant-index.jsonl',json.dumps(entry)+'\n',raw=True)
                        variant=copy.deepcopy(supplied)
                        variant['artifacts']['result_index']=index
                        m=save('variant-input.json',variant)
                        k=a.load_inputs(Path(m['path']),m['sha256'],expected_freeze=freeze_ref['sha256'],expected_approval=approval_ref['sha256'])
                        rpt, projected=a.build_report(**k)
                        expected='IDENTITY_OR_SOURCE_CONFLICT' if artifact=='wrong_hash' else 'CLOSED_RESULT_UNVERIFIED' if artifact=='association' else 'TERMINATED_TRACE_MISSING'
                        self.assertEqual(projected['planned-episodes.jsonl'][0]['state'],expected,artifact)
                        if artifact!='wrong_hash':
                            self.assertNotEqual(rpt['status'],'INVALID_REPORT_INPUT',artifact)
                        if artifact in ['trace','association']:
                            self.assertEqual(projected['planned-episodes.jsonl'][0]['qa05']['known_api_cost_usd'],'0.1')
                        self.assertFalse(any(p.get('certified_intersection') for p in projected['paired.jsonl']))
                    finally:
                        if moved is not None:
                            moved.rename(original)
                # Wrong oracle/source identity never receives certification.
                bad = dict(association, oracle_ref=oracle_refs['COMMON_SUPPLEMENT6'])
                with self.assertRaises(a.InputError):
                    a.validate_association(bad,kwargs['records'][0],freeze,freeze_ref,oracle_refs['COMMON9'],root)
                # Even a known hash cannot replace a role's exact path/bytes.
                supplied['qa04']['report']['ref'] = dict(qa_refs['report'],path=str(root/'excluded.json'))
                with self.assertRaises(a.InputError):
                    a.approved_qa04(a.ArtifactReader(root),supplied['qa04'],approval)
                old = dict(qa_refs['report'])
                qa04['all_counts_by_configuration']['AAAA'] = [1,1,3,2,1,2]
                wrong = save('approval/old-report.json',qa04)
                badapproval = copy.deepcopy(approval)
                badapproval['artifacts'][0].update(original_path=wrong['path'],copied_path=wrong['path'],sha256=wrong['sha256'],bytes=wrong['bytes'])
                supplied['qa04']['report']['ref'] = wrong
                with patch.dict(a.QA04_HASHES,{'report':wrong['sha256']}):
                    with self.assertRaises(a.InputError):
                        a.approved_qa04(a.ArtifactReader(root),supplied['qa04'],badapproval)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--batch', choices=['core', 'review-fixes', 'projection', 'output', 'loader', 'stage02-fixes'], default='core')
    parser.add_argument('--record', type=Path, required=True)
    args = parser.parse_args()
    started = time.perf_counter()
    stream = io.StringIO()
    outcome = unittest.TextTestRunner(stream=stream, verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase({'core': CoreChecks, 'review-fixes': ReviewFixChecks, 'projection': ProjectionChecks, 'output': OutputChecks, 'loader': LoaderChecks, 'stage02-fixes': Stage02FixChecks}[args.batch]))
    elapsed = time.perf_counter() - started
    result = {'status': 'PASS' if outcome.wasSuccessful() else 'FAIL', 'batch': args.batch, 'tests': outcome.testsRun, 'elapsed_seconds': elapsed,
              'scope': 'Pure temporary synthetic data only; no corpus/evaluator/model/parser/candidate/compiler/network', 'stdout': stream.getvalue(),
              'source_sha256': {'report_adapter.py': a.sha(Path(a.__file__).read_bytes()), 'test_report_adapter.py': a.sha(Path(__file__).read_bytes())}}
    args.record.parent.mkdir(parents=True, exist_ok=True)
    with args.record.open('x') as handle:
        json.dump(result, handle, indent=2)
        handle.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k != 'stdout'}))
    if not outcome.wasSuccessful():
        print(stream.getvalue())
        raise SystemExit(1)
