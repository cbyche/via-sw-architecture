"""Pure synthetic adapter checks. Never reads the formal corpus."""
import argparse
import copy
import io
import json
from pathlib import Path
import tempfile
import time
import unittest

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
                  primary_measurement_eligibility='ELIGIBLE' if eligible else 'NOT_CERTIFIED', deadline_audit={'status': 'CONSISTENT' if eligible else 'UNRESOLVED'},
                  qa01=[dict(row, status='OBSERVED', latency_ns=value, accepted_callback_id='synthetic-display') for row in oracle['responses']],
                  qa02=[dict(row, status='ADOPTED', correct=True) for row in oracle['bindings']],
                  qa03=[dict(row, status='OBSERVED', via_ns=value, accepted_terminal_event_id='synthetic-goal') for row in oracle['goals']],
                  qa05={'known_api_cost_usd': '0.1', 'unknown_call_count': 0, 'unknown_reservation_usd': 0},
                  qa06={'status': 'OBSERVED', 'peak_concurrent_sum_bytes': 100, 'peak_same_sample': {'concurrent_sum_bytes': 100, 'processes': [{'role': 'python', 'rss_bytes': 60}, {'role': 'rust', 'rss_bytes': 40}]}})
    return {'identity': a.key_object(key), 'run_id': '-'.join(map(str, key)), 'provenance_valid': True, 'result': result,
            'result_ref': {'path': 'synthetic-result', 'sha256': 'synthetic', 'bytes': 1}, 'trace_ref': {'path': 'synthetic-trace', 'sha256': 'synthetic', 'bytes': 1}}


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
        self.assertEqual(a.episode_state([], ['STARTED', 'TERMINATED'], q), 'TERMINATED_TRACE_MISSING')
        self.assertEqual(a.episode_state([{'provenance_valid': False}], [], q), 'CLOSED_RESULT_UNVERIFIED')

    def test_timing_zero_null_and_partition(self):
        rows = []
        for raw, expected in [({'status': 'OBSERVED', 'latency_ns': 0}, 'CERTIFIED_OBSERVED'),
                              ({'status': 'NOT_OBSERVED', 'latency_ns': None}, 'CERTIFIED_NOT_OBSERVED'),
                              ({'status': 'UNRESOLVED_EARLIEST', 'latency_ns': None}, 'CERTIFIED_UNRESOLVED')]:
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


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--batch', choices=['core'], default='core')
    parser.add_argument('--record', type=Path, required=True)
    args = parser.parse_args()
    started = time.perf_counter()
    stream = io.StringIO()
    outcome = unittest.TextTestRunner(stream=stream, verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(CoreChecks))
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
