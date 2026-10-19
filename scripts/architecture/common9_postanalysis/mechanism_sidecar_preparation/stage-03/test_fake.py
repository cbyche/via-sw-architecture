"""Residual MS2-01 synthetic checks; no campaign inputs or actual extraction."""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import time
import unittest
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
SOURCE=HERE.parent.parent/'extract_dp1_dp4_mechanism_evidence.py'
spec=importlib.util.spec_from_file_location('sidecar_stage03',SOURCE)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
LABEL=sys.argv[1] if len(sys.argv)>1 else 'attempt-01'
if LABEL not in ('attempt-01','attempt-02'):raise ValueError('Explicit bounded synthetic attempt required')
ROOT=HERE/('fake-inputs-'+LABEL);ROOT.mkdir(exist_ok=False)
NAMED=[];VIRTUAL=[];KEY=('COMMON9','AAAA','S02',1)
PASSAGE={'source_id':'DOC','revision':1,'start_line':1,'end_line':1,
         'text':'SECRET ORIGINAL','acquisition_node':'read1'}
READ={'event_id':'e0','kind':'SOURCE_RANGE_READ','elapsed_ns':1,'parent_ids':[],
      'goal_ids':['g'],'data':{'consumer':'REQUEST_EVIDENCE_ACQUISITION','passage':PASSAGE}}


def saved(name,value):
    p=ROOT/name;p.parent.mkdir(parents=True,exist_ok=True);data=m.encoded(value);p.write_bytes(data)
    NAMED.append(p)
    return {'path':str(p),'bytes':len(data),'sha256':m.digest(data)}


def terminal(invalid):
    return {'events':[READ,invalid],'calls':[],'end_ns':99,'returncode':0}


def extract(invalid,name):
    t=terminal(invalid);ref=saved(name,t);row=m.empty_row(KEY,run_id='FAKE')
    return m.extract_run(t,ref,row,m.Reader(ROOT),{'recorded_path':'FAKE_REFERENCE_ONLY'})


class Sink(io.BytesIO):
    def __exit__(self,*args):return False


class Checks(unittest.TestCase):
    def assert_failed_coverage(self,row):
        self.assertEqual(row['extraction_status'],'UNSUPPORTED')
        for name,collection in row['collections'].items():
            self.assertEqual(collection['status'],'UNSUPPORTED',name)
            self.assertTrue(collection['coverage_issue'],name)
            self.assertEqual(collection['processing_state'],'INTERRUPTED',name)
        self.assertEqual(len(row['collections']['source_reads']['items']),1)
        issues=row['collections']['join_issues']['items']
        self.assertTrue(any(x['detail']=='Invalid event/data shape' and
            x['source_ref']['pointer']=='/events/1' for x in issues))
        self.assertNotIn('SECRET ORIGINAL',json.dumps(row))

    def test_01_invalid_event_object(self):
        self.assert_failed_coverage(extract('NOT_AN_EVENT_OBJECT','invalid-event.json'))

    def test_02_null_data_object(self):
        invalid={'event_id':'e1','kind':'SOURCE_RANGE_READ','data':None}
        self.assert_failed_coverage(extract(invalid,'null-data.json'))

    def test_03_all_planned_rows_and_prior_facts_survive(self):
        jobs=[m.key_object((p,c,e,r)) for p,(cases,reps) in m.POPULATIONS.items()
              for c in m.CONFIGS for e in cases for r in range(1,reps+1)]
        keys=m.validate_plan(jobs);index=[]
        for k,run,invalid in ((keys[0],'bad-event','NOT_AN_EVENT_OBJECT'),
                              (keys[1],'null-data',{'kind':'SOURCE_RANGE_READ','data':None})):
            t=terminal(invalid);t.update(configuration=k[1],episode=k[2],rep=k[3],run_id=run,
                                         phase='FORMAL',binary_sha256='f'*64)
            ref=saved('raw/runs/'+run+'/trace.json',t)
            index.append({'identity':m.key_object(k),'run_id':run,'trace_ref':ref})
        sink=Sink();original=Path.open
        def opened(path,*args,**kwargs):
            if path.name=='mechanism-evidence.jsonl' and args and args[0]=='xb':return sink
            return original(path,*args,**kwargs)
        receipt={'outputs':{}}
        with patch.object(Path,'open',opened):
            m.write_rows(ROOT/'virtual-output',keys,m.Reader(ROOT),receipt,index=index,
                freeze={'binary':{'sha256':'f'*64}},boundary={'accepted_calls_ref':{'recorded_path':'FAKE_REFERENCE_ONLY'}})
        rows=[json.loads(x) for x in sink.getvalue().splitlines()]
        self.assertEqual(len(rows),441);self.assertEqual(receipt['counts']['planned'],441)
        self.assert_failed_coverage(rows[0]);self.assert_failed_coverage(rows[1])
        VIRTUAL.append({'rows':len(rows),'bytes':len(sink.getvalue()),'fixture':'MEMORY_SINK_ONLY'})


if __name__=='__main__':
    started=time.monotonic();result=unittest.TextTestRunner(verbosity=1).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(Checks));elapsed=time.monotonic()-started
    inventory=[]
    for p in NAMED:
        b=p.read_bytes();inventory.append({'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
    total=sum(x['bytes'] for x in inventory)
    payload={'kind':'SYNTHETIC_ONLY','tests':result.testsRun,'failures':len(result.failures),
        'errors':len(result.errors),'runtime_seconds':elapsed,'fake_file_bytes':total,'files':inventory,
        'virtual_output_evidence':VIRTUAL,'source_sha256':m.digest(SOURCE.read_bytes()),
        'under_1MiB_fake_files':total<=1024*1024,'under_1second_test_target':elapsed<1,
        'actual_sidecar_cli_executed':False,'real_campaign_inputs_read':False}
    with (HERE/('fake-results-'+LABEL+'.json')).open('xb') as h:h.write(m.encoded(payload))
    raise SystemExit(0 if result.wasSuccessful() and total<=1024*1024 and elapsed<1 else 1)
