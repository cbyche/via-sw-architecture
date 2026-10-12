"""Selected QA05 evidence must stay tied to one reviewed change per task."""
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from benchmark.architecture.dp41_45_poc.change_experiments import source_snapshot, snapshot_digest
from benchmark.architecture.dp41_45_poc.finalize import selected_change_records


class SelectionIntegrityTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.campaign = Path(self.temporary.name)
        baseline = self.campaign / 'frozen-source'
        baseline.mkdir()
        (baseline / 'port.py').write_text('WIRE = 1\n')
        self.baseline_hash = snapshot_digest(source_snapshot(baseline))
        (self.campaign / 'manifest.json').write_text(json.dumps({'frozen_source_digest':'frozen-baseline'}))
        selected = []
        for variant in 'AB':
            for number in range(1, 7):
                task = f'M{number:02d}'
                directory = self.campaign / 'changes-selected' / f'{variant}-{task}'
                directory.mkdir(parents=True)
                shutil.copytree(baseline, directory / 'source')
                (directory / 'source/port.py').write_text('WIRE = 2\n')
                patch = directory / 'patch.diff'
                patch.write_text('-WIRE = 1\n+WIRE = 2\n')
                path = directory / 'reviewed-result.json'
                record = {'variant':variant, 'task_id':task, 'status':'COMPLETED',
                          'baseline_source_path':str(baseline), 'baseline_source_hash':self.baseline_hash,
                          'after_source_hash':snapshot_digest(source_snapshot(directory / 'source')),
                          'patch_ref':str(patch), 'patch_sha256':hashlib.sha256(patch.read_bytes()).hexdigest(),
                          'result_ref':str(path), **{flag:True for flag in ('completion_passed', 'regression_passed',
                              'patch_verified', 'responsibility_review_passed', 'original_baseline_unchanged')}}
                path.write_text(json.dumps(record))
                selected.append({'variant':variant, 'task_id':task,
                    'reviewed_result_ref':str(path.relative_to(self.campaign)),
                    'reviewed_result_sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
        self.selection = {'frozen_source_digest':'frozen-baseline', 'selected':selected}
        self.save_selection()

    def save_selection(self):
        (self.campaign / 'qa05-selection-manifest.json').write_text(json.dumps(self.selection))

    def test_only_pinned_selected_jobs_count(self):
        extra = self.campaign / 'changes/CH-DP41-B-M04'
        extra.mkdir(parents=True)
        (extra / 'reviewed-result.json').write_text('{"status":"preserved-unselected-attempt"}')
        records = selected_change_records(self.campaign)
        self.assertEqual(len(records), 12)
        self.assertEqual({(r['variant'], r['task_id']) for r in records},
                         {(v, f'M{n:02d}') for v in 'AB' for n in range(1, 7)})

    def test_duplicate_and_missing_jobs_rejected(self):
        self.selection['selected'][-1] = self.selection['selected'][0]
        self.save_selection()
        with self.assertRaisesRegex(ValueError, 'Duplicate'):
            selected_change_records(self.campaign)

    def test_changed_reviewed_source_rejected(self):
        first = self.campaign / self.selection['selected'][0]['reviewed_result_ref']
        (first.parent / 'source/port.py').write_text('WIRE = 3\n')
        with self.assertRaisesRegex(ValueError, 'changed source'):
            selected_change_records(self.campaign)

    def test_different_baseline_and_result_hash_rejected(self):
        self.selection['frozen_source_digest'] = 'other-baseline'
        self.save_selection()
        with self.assertRaisesRegex(ValueError, 'different measured baseline'):
            selected_change_records(self.campaign)
        self.selection['frozen_source_digest'] = 'frozen-baseline'
        self.selection['selected'][0]['reviewed_result_sha256'] = 'invalid'
        self.save_selection()
        with self.assertRaisesRegex(ValueError, 'reviewed result changed'):
            selected_change_records(self.campaign)


if __name__ == '__main__':
    unittest.main()
