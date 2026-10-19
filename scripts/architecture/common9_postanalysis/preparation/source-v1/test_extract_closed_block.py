"""Tiny synthetic fixtures only. Never opens the actual campaign corpus."""
import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import time
import unittest

import extract_closed_block as x


def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value), encoding='utf-8')


class Fixture:
    def __init__(self, root):
        self.root = root
        oracle = {'S02': {'responses': [{'event_id': 'U1', 'response_id': 'R1', 'required_content': 'plain answer'}],
                         'goals': [{'id': 'G1', 'start_event': 'U1', 'member_events': ['U1'], 'completion': 'done'}], 'bindings': []}}
        self.sources = []
        for relative in (x.ORACLES['COMMON9'], *x.REFERENCE_SOURCES):
            path = root / 'frozen/source' / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(oracle) if relative.endswith('.json') else 'synthetic reference only\n')
            self.sources.append({'relativePath': relative, 'frozen_path': str(path), 'sha256': x.digest(path.read_bytes())})
        self.freeze = {'version': x.GENERATION, 'sources': self.sources, 'binary': {'sha256': 'synthetic-binary'},
                       'jobs': [dict(population='COMMON9', configuration=cfg, episode='S02', rep=1) for cfg in x.CONFIGS]}
        dump(root / 'freeze.json', self.freeze)
        self.freeze_hash = x.digest((root / 'freeze.json').read_bytes())
        self.closed = {'status': 'SYNTHETIC_BLOCK_CLOSED', 'population': 'COMMON9', 'episode': 'S02', 'rep': 1,
                       'rows': [{'configuration': cfg, 'run_id': cfg + '-S02-r1-test'} for cfg in x.CONFIGS]}
        self.closure = root / 'closure.json'
        dump(self.closure, self.closed)
        self.first = root / 'raw/runs/AAAA-S02-r1-test/trace.json'
        self.payload = 'data:image/png;base64,' + 'QUJD' * 100
        for cfg in x.CONFIGS:
            run_id = cfg + '-S02-r1-test'
            run = root / 'raw/runs' / run_id
            dump(run / 'candidate-initial.json', {'frames': [{'id': 'F1', 'text': 'source plain text', 'image_url': self.payload}], 'payload': self.payload})
            trace = dict(run_id=run_id, configuration=cfg, episode='S02', rep=1, phase='FORMAL', binary_sha256='synthetic-binary',
                         candidate_input_file=str(run / 'candidate-initial.json'), returncode=0, end_ns=100,
                         events=[{'event_id': 'adopt', 'kind': 'SEMANTIC_ADOPTION', 'observed_ns': 5,
                                  'data': {'stimulus_id': 'U1', 'goal_id': 'goal-U1', 'task_id': None, 'revision': 1, 'plan': {'purpose': 'plain purpose'}}},
                                 {'event_id': 'display', 'kind': 'DISPLAY_PROXY', 'observed_ns': 10,
                                  'data': {'stimulus_id': 'U1', 'text': 'plain answer', 'image_url': self.payload}}],
                         host_display_receipts=[], calls=[], provided_sources=[], source_nodes=[])
            dump(run / 'trace.json', trace)

    def build(self, **kwargs):
        return x.build_bundle(self.root, self.closure, 'COMMON9', 'S02', 1, self.freeze_hash, **kwargs)

    def change_trace(self, change):
        trace = json.loads(self.first.read_text())
        change(trace)
        dump(self.first, trace)


class SyntheticTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='common9-report-synthetic-')
        self.f = Fixture(Path(self.temp.name))

    def tearDown(self):
        self.temp.cleanup()

    def test_wrong_freeze_root(self):
        with self.assertRaisesRegex(x.EvidenceError, 'Wrong frozen root'):
            x.build_bundle(self.f.root, self.f.closure, 'COMMON9', 'S02', 1, 'another-root-hash')

    def test_wrong_candidate_root(self):
        self.f.change_trace(lambda t: t.update(candidate_input_file='/not-this-synthetic-root/input.json'))
        with self.assertRaisesRegex(x.EvidenceError, 'outside exact root'):
            self.f.build()

    def test_mixed_phase(self):
        self.f.change_trace(lambda t: t.update(phase='OFFLINE'))
        with self.assertRaisesRegex(x.EvidenceError, 'Mixed phase'):
            self.f.build()

    def test_mixed_configuration(self):
        self.f.change_trace(lambda t: t.update(configuration='AAAB'))
        with self.assertRaisesRegex(x.EvidenceError, 'Mixed phase/configuration'):
            self.f.build()

    def test_duplicate_run(self):
        self.f.closed['rows'][1]['run_id'] = self.f.closed['rows'][0]['run_id']
        dump(self.f.closure, self.f.closed)
        with self.assertRaisesRegex(x.EvidenceError, 'Duplicate/invalid run'):
            self.f.build()

    def test_duplicate_configuration(self):
        self.f.closed['rows'][1]['configuration'] = 'AAAA'
        dump(self.f.closure, self.f.closed)
        with self.assertRaisesRegex(x.EvidenceError, 'Duplicate/unplanned configuration'):
            self.f.build()

    def test_missing_run_retains_declared_obligations(self):
        self.f.closed['rows'].pop()
        dump(self.f.closure, self.f.closed)
        manifest, _, rows = self.f.build()
        self.assertEqual(len(rows), 14)
        missing = [r for r in rows if r['key']['configuration'] == 'AAAC']
        self.assertEqual(len(missing), 2)
        self.assertTrue(all(r['execution_state'] == 'MISSING_RUN_RECORD' and r['key']['run_id'] is None for r in missing))
        self.assertEqual(manifest['status'], 'INCOMPLETE_EVIDENCE_UNREVIEWED')

    def test_missing_trace_is_not_unexecuted_or_semantic_fail(self):
        self.f.first.unlink()
        _, _, rows = self.f.build()
        row = next(r for r in rows if r['key']['configuration'] == 'AAAA')
        self.assertEqual(row['execution_state'], 'MISSING_TRACE')
        self.assertTrue(all(v['status'] == 'UNREVIEWED' for v in row['review'].values()))

    def test_ambiguous_callbacks_preserved(self):
        self.f.change_trace(lambda t: t['events'].append(dict(t['events'][-1], event_id='second-display')))
        _, _, rows = self.f.build()
        row = rows[0]
        self.assertEqual(row['raw_callback_state'], 'AMBIGUOUS_RAW_CALLBACK')
        self.assertIsNone(row['first_raw_callback'])
        self.assertEqual(len(row['all_matching_display_refs']), 2)

    def test_oversized_trace(self):
        with self.assertRaisesRegex(x.EvidenceError, 'byte limit'):
            self.f.build(max_trace=10)

    def test_no_base64_copy_and_immutable_output(self):
        bundle = self.f.build()
        out = self.f.root / 'postanalysis/synthetic-bundle'
        x.write_bundle(out, bundle)
        combined = ''.join((out / name).read_text() for name in ['manifest.json', 'worklist.json', 'evidence.json', 'review.md'])
        self.assertNotIn(self.f.payload, combined)
        self.assertIn('source plain text', combined)
        self.assertIn('plain answer', combined)
        before = (out / 'manifest.json').read_bytes()
        with self.assertRaisesRegex(x.EvidenceError, 'Immutable output'):
            x.write_bundle(out, bundle)
        self.assertEqual(before, (out / 'manifest.json').read_bytes())

    def test_missing_saved_evaluation_explicit(self):
        manifest, _, rows = self.f.build()
        self.assertTrue(all(r['saved_evaluation_state'] == 'MISSING_SAVED_EVALUATION' for r in rows))
        self.assertEqual(rows[0]['saved_QA01_selection']['status'], 'MISSING_SAVED_EVALUATION')
        self.assertTrue(all(v['status'] == 'UNREVIEWED' for r in rows for v in r['review'].values()))
        self.assertEqual(manifest['declared_obligations'], {'response': 7, 'goal_relation': 7})

    def test_saved_selected_callback_does_not_replace_raw_first(self):
        self.f.change_trace(lambda t: t['events'].append(dict(t['events'][-1], event_id='selected-later', observed_ns=20)))
        saved = dict(run_id='AAAA-S02-r1-test', configuration='AAAA', episode='S02', rep=1, phase='FORMAL',
                     primary_measurement_eligibility='ELIGIBLE', deadline_audit={'status': 'CONSISTENT'},
                     qa01=[{'response_id': 'R1', 'accepted_callback_id': 'selected-later', 'latency_ns': 20}], qa03=[])
        path = self.f.root / 'saved-evaluation.json'
        dump(path, saved)
        index = self.f.root / 'saved-index.json'
        dump(index, {'AAAA-S02-r1-test': {'path': str(path), 'sha256': x.digest(path.read_bytes())}})
        _, _, rows = self.f.build(saved_index=index)
        self.assertEqual(rows[0]['first_raw_callback']['event_id'], 'display')
        self.assertEqual(rows[0]['saved_QA01_selection']['selected']['event_id'], 'selected-later')
        self.assertTrue(all(v['status'] == 'UNREVIEWED' for v in rows[0]['review'].values()))

    def test_no_output_stays_explicit_unreviewed(self):
        self.f.change_trace(lambda t: t.update(events=t['events'][:1]))
        _, _, rows = self.f.build()
        self.assertEqual(rows[0]['raw_callback_state'], 'NOT_OBSERVED')
        self.assertTrue(all(v['status'] == 'UNREVIEWED' for v in rows[0]['review'].values()))


if __name__ == '__main__':
    started = time.perf_counter()
    output = io.StringIO()
    result = unittest.TextTestRunner(stream=output, verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(SyntheticTests))
    elapsed = time.perf_counter() - started
    record = {'status': 'PASS' if result.wasSuccessful() else 'FAIL', 'scope': 'Temporary synthetic fixtures only; no actual corpus/evaluator/candidate/model/parser/build/network execution',
              'tests': result.testsRun, 'elapsed_seconds': elapsed, 'stdout': output.getvalue(),
              'source_sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(x.__file__), Path(__file__)]}}
    target = Path(__file__).with_name('synthetic-check-v1.json')
    with target.open('x') as handle:
        json.dump(record, handle, indent=2)
        handle.write('\n')
    print(json.dumps({k: v for k, v in record.items() if k != 'stdout'}))
    if not result.wasSuccessful():
        print(output.getvalue())
        raise SystemExit(1)
