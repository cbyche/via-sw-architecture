import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from benchmark.architecture.dp41_45_poc.campaign import run_workers, service_failure_calls, verify_input_filter_transience


class ServiceFailureTests(unittest.TestCase):
    def record(self, directory, call, response=None):
        directory.mkdir(exist_ok=True)
        if response is not None:
            (directory / 'response.json').write_text(json.dumps(response))
            call['output'] = {'path': 'response.json'}
        (directory / 'model-calls.jsonl').write_text(json.dumps(call) + '\n')

    def test_identical_context_provider_filter_rejection_is_preserved_service_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            self.record(directory, {'call_id': 'failed', 'status': 'HTTP_ERROR', 'http_status': 400,
                                     'usage': None, 'cost_usd': None},
                        {'body': json.dumps({'error': {'code': 'data_inspection_failed'}})})
            self.assertEqual([c['call_id'] for c in service_failure_calls(directory)], ['failed'])

    def test_invalid_wire_parameter_needs_audit_instead_of_automatic_replay(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            self.record(directory, {'call_id': 'bad-wire', 'status': 'HTTP_ERROR', 'http_status': 400,
                                     'usage': None, 'cost_usd': None},
                        {'body': json.dumps({'error': {'code': 'invalid_parameter'}})})
            with self.assertRaisesRegex(ValueError, 'Unclassified missing usage'):
                service_failure_calls(directory)

    def test_known_usage_model_schema_failure_is_a_candidate_observation(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            self.record(directory, {'call_id': 'bad-frame', 'status': 'COMPLETED', 'http_status': 200,
                                     'usage': {'input_tokens': 5, 'output_tokens': 2},
                                     'cost_usd': .001, 'decode_error': 'Invalid model JSON'})
            self.assertEqual(service_failure_calls(directory), [])

    def test_local_queue_cancellation_has_known_zero_usage_and_is_retained(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            self.record(directory, {'call_id': 'cancelled', 'status': 'CANCELLED_BEFORE_SEND',
                                     'usage': {'input_tokens': 0, 'output_tokens': 0}, 'cost_usd': 0})
            self.assertEqual(service_failure_calls(directory), [])

    def filter_health(self, root, accepted):
        worker = root / 'workers/A1'
        worker.mkdir(parents=True)
        original = {'messages': [{'role': 'user', 'content': 'identical benign input'}], 'max_tokens': 4096,
                    'model': 'qwen3.6-35b-a3b', 'temperature': 0, 'enable_thinking': False,
                    'response_format': {'type': 'json_object'}}
        (worker / 'original.json').write_text(json.dumps(original))
        failure = {'call_id': 'filter-call', 'http_status': 400, 'context': {'path': 'original.json'}, 'case_id': 'S1-01'}
        class HealthClient:
            def __init__(self, evidence, *args, **kwargs):
                self.evidence, self.ordinal = evidence, 0
            async def call(self, messages, role, **kwargs):
                self.ordinal += 1
                self_outer.assertEqual(messages, original['messages'])
                self_outer.assertEqual(kwargs['purpose'], 'SERVICE_HEALTH')
                context = self.evidence.artifact('model-contexts', str(self.ordinal), original)
                self.evidence.write('model-calls', {'call_id': f'health-{self.ordinal}', 'http_status': 200 if accepted else 400,
                    'usage': {'input_tokens': 2, 'output_tokens': 2} if accepted else None,
                    'cost_usd': .001 if accepted else None, 'context': context})
                if not accepted:
                    raise RuntimeError('preserved provider rejection')
                return {'unused': 'health output is not adopted'}
        self_outer = self
        with patch('prototypes.candidates.dp41_45_poc.shared.providers.ModelClient', HealthClient):
            return verify_input_filter_transience(worker, failure, root, root / 'budget.json')

    def test_filter_replay_requires_identical_accepted_context_and_never_adopts_output(self):
        with tempfile.TemporaryDirectory() as temp:
            summary = Path(self.filter_health(Path(temp), True))
            result = json.loads(summary.read_text())
            self.assertEqual(result['status'], 'CONFIRMED_TRANSIENT_IDENTICAL_CONTEXT')
            self.assertFalse(result['candidate_meaning_adopted'])
            self.assertFalse(result['included_in_final_population'])

    def test_persistent_input_filter_rejection_needs_audit_instead_of_hiding_candidate_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            with self.assertRaisesRegex(ValueError, 'Persistent provider input rejection'):
                self.filter_health(root, False)
            result = json.loads((root / 'service-health/filter-call/summary.json').read_text())
            self.assertEqual(result['status'], 'PERSISTENT_REJECTION_NEEDS_AUDIT')

    def test_service_failure_replays_both_complete_workers_without_overwriting_attempt(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'frozen-source').mkdir()
            manifest = {'pair_order': [['A', 'B'], ['B', 'A'], ['A', 'B'], ['B', 'A'], ['A', 'B']],
                        'python_executable': 'specified-test-worker', 'frozen_source_hashes': {}}
            (root / 'manifest.json').write_text(json.dumps(manifest))
            observed = []
            class CompletedWorker:
                stdout = ['actual test worker output\n']
                def wait(self):
                    return 0
            def worker(command, **kwargs):
                directory = Path(command[command.index('--output') + 1])
                observed.append(directory.name)
                directory.mkdir(parents=True)
                (directory / 'worker-summary.jsonl').write_text('{"status":"COMPLETE"}\n')
                failed = len(observed) == 1
                call = {'call_id': f'call-{len(observed)}', 'status': 'HTTP_ERROR' if failed else 'COMPLETED',
                        'http_status': 429 if failed else 200, 'usage': None if failed else {'input_tokens': 1, 'output_tokens': 1},
                        'cost_usd': None if failed else .001}
                self.record(directory, call)
                return CompletedWorker()
            with patch('benchmark.architecture.dp41_45_poc.campaign.subprocess.Popen', worker), contextlib.redirect_stdout(io.StringIO()):
                run_workers(root, root / 'test-budget.json')
            self.assertEqual(observed[:4], ['A1', 'B1', 'A1', 'B1'])
            self.assertEqual(len(observed), 12)
            preserved = root / 'invalid-worker-attempts/pair-1-attempt-1'
            self.assertEqual(json.loads((preserved / 'A1/model-calls.jsonl').read_text())['http_status'], 429)
            self.assertEqual(json.loads((preserved / 'B1/model-calls.jsonl').read_text())['http_status'], 200)
            invalid = json.loads((preserved / 'invalid-pair.json').read_text())
            self.assertFalse(invalid['included_in_final_population'])
            self.assertEqual({p.name for p in (root / 'workers').iterdir()}, {f'{v}{r}' for v in 'AB' for r in range(1, 6)})
            progress = [json.loads(line) for line in (root / 'campaign-progress.jsonl').read_text().splitlines()]
            self.assertEqual(sum(r['kind'] == 'PAIR_ACCEPTED' for r in progress), 5)
            self.assertEqual(progress[-1]['kind'], 'ALL_WORKERS_COMPLETE')
