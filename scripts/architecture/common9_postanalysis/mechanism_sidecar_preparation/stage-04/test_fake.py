"""Exact host lifecycle marker compatibility; synthetic in-memory traces only."""
import hashlib
import importlib.util
import json
from pathlib import Path
import time
import unittest

HERE=Path(__file__).resolve().parent
SOURCE=HERE.parent.parent/'extract_dp1_dp4_mechanism_evidence.py'
spec=importlib.util.spec_from_file_location('sidecar_stage04',SOURCE)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
KEY=('COMMON9','AAAA','S02',1)
PASSAGE={'source_id':'DOC','revision':1,'start_line':1,'end_line':1,
         'text':'SYNTHETIC ORIGINAL','acquisition_node':'read1'}
READ={'event_id':'e0','kind':'SOURCE_RANGE_READ','data':{
      'consumer':'REQUEST_EVIDENCE_ACQUISITION','passage':PASSAGE}}
REF={'path':str(HERE/'NOT_A_REAL_INPUT.json'),'bytes':0,'sha256':'0'*64}
MARKER={'kind':'PROCESS_STDOUT_CLOSED','observed_ns':99}

def extract(events):
    trace={'events':[READ,*events],'calls':[],'end_ns':100,'returncode':0}
    return m.extract_run(trace,REF,m.empty_row(KEY,run_id='SYNTHETIC'),
                         m.Reader(HERE),{'recorded_path':'SYNTHETIC_ONLY'})

class Checks(unittest.TestCase):
    def failed(self,row):
        self.assertEqual(row['extraction_status'],'UNSUPPORTED')
        for collection in row['collections'].values():
            self.assertEqual(collection['status'],'UNSUPPORTED')
            self.assertEqual(collection['processing_state'],'INTERRUPTED')
            self.assertTrue(collection['coverage_issue'])
        self.assertEqual(len(row['collections']['source_reads']['items']),1)

    def test_exact_host_marker_supported(self):
        row=extract([MARKER])
        self.assertEqual(row['extraction_status'],m.COMPLETE)
        self.assertEqual(len(row['collections']['source_reads']['items']),1)
        self.assertFalse(row['trace_complete_for_absence_claims'])
        for collection in row['collections'].values():
            self.assertEqual(collection['status'],m.COMPLETE)
            self.assertEqual(collection['processing_state'],'FINISHED')
        self.assertEqual(row['trace_ref']['sha256'],REF['sha256'])

    def test_null_data_not_reclassified(self):
        self.failed(extract([dict(MARKER,data=None)]))

    def test_boolean_clock_not_reclassified(self):
        self.failed(extract([dict(MARKER,observed_ns=True)]))

    def test_negative_clock_not_reclassified(self):
        self.failed(extract([dict(MARKER,observed_ns=-1)]))

    def test_unknown_missing_data_not_reclassified(self):
        self.failed(extract([{'kind':'UNKNOWN_HOST_EVENT','observed_ns':99}]))

    def test_extra_fields_not_silently_ignored(self):
        self.failed(extract([dict(MARKER,unexpected='value')]))

    def test_later_invalid_event_still_interrupts(self):
        self.failed(extract([MARKER,'NOT_AN_EVENT']))

    def test_candidate_null_data_still_interrupts(self):
        self.failed(extract([{'kind':'SOURCE_RANGE_READ','data':None},MARKER]))

if __name__=='__main__':
    begin=time.monotonic()
    result=unittest.TextTestRunner(verbosity=1).run(unittest.defaultTestLoader.loadTestsFromTestCase(Checks))
    elapsed=time.monotonic()-begin
    payload={'kind':'SYNTHETIC_ONLY_NO_INPUT_FILES_OR_PROCESS','tests':result.testsRun,
             'failures':len(result.failures),'errors':len(result.errors),'runtime_seconds':elapsed,
             'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
             'real_campaign_input_reads':0,'extractor_cli_runs':0,'model_calls':0,
             'caps_unchanged':m.CAPS,'expected_fake_input_files':0}
    with (HERE/'fake-results-attempt-01.json').open('x') as f:json.dump(payload,f,indent=2);f.write('\n')
    raise SystemExit(0 if result.wasSuccessful() and elapsed<1 else 1)
