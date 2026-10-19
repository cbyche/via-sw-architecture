"""Bounded synthetic-only checks. No campaign input or external program access."""
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import sys
import time
from types import SimpleNamespace
import unittest

HERE=Path(__file__).resolve().parent
SOURCE=HERE.parent.parent/'extract_dp1_dp4_mechanism_evidence.py'
spec=importlib.util.spec_from_file_location('mechanism_sidecar',SOURCE)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
LABEL=sys.argv[1] if len(sys.argv)>1 else 'attempt-01'
if LABEL not in ('attempt-01','attempt-02','attempt-03'):
    raise ValueError('Explicit bounded synthetic attempt label required')
FAKE=HERE/('fake-inputs-'+LABEL)
FAKE.mkdir(exist_ok=False)
(FAKE/'raw/model').mkdir(parents=True,exist_ok=True)
KEY=('COMMON9','AAAA','S02',1)


def save(name,value,raw=False):
    p=FAKE/name;p.parent.mkdir(parents=True,exist_ok=True)
    data=value if raw else m.encoded(value)
    p.write_bytes(data)
    return {'path':str(p),'sha256':m.digest(data),'bytes':len(data)}


def event(kind,data,number):
    return {'kind':kind,'event_id':'e'+str(number),'elapsed_ns':number,
            'parent_ids':[],'goal_ids':['g'],'data':data}


def trace(events,calls=()):
    return {'events':events,'calls':list(calls),'end_ns':99,'returncode':1}


def extract(value):
    ref=save('trace.json',value);row=m.empty_row(KEY,run_id='fake')
    return m.extract_run(value,ref,row,m.Reader(FAKE),{'recorded_path':'FAKE_CALLS_REFERENCE_ONLY'})


def items(row,name):
    return row['collections'][name]['items']


SPAN={'source_id':'DOC','revision':1,'start_line':1,'end_line':1}
PASSAGE=dict(SPAN,text='SECRET PROSE',acquisition_node='read1')
RECORD={'record_id':'R','producer_node':'producer1','source_spans':[SPAN],
        'supporting_passages':[PASSAGE],'interpretation':'SECRET INTERPRETATION'}
CONTRACT={'mandatory':{'goal_id':'g','request_revision':1},'capsule':{'records':[RECORD]}}


class FakeChecks(unittest.TestCase):
    def test_01_exact_plan(self):
        jobs=[m.key_object((p,c,e,r)) for p,(cases,reps) in m.POPULATIONS.items()
              for c in m.CONFIGS for e in cases for r in range(1,reps+1)]
        self.assertEqual(len(m.validate_plan(jobs)),441)
        with self.assertRaises(m.EvidenceError):m.validate_plan(jobs[:-1]+jobs[:1])

    def test_02_boolean_rep_rejected(self):
        with self.assertRaises(m.EvidenceError):m.identity(dict(m.key_object(KEY),rep=True))

    def test_03_rfc6901_escaping_and_prose(self):
        ref=save('pointer.json',{})
        result=m.project({'a/b~c':'SECRET'},ref)
        self.assertEqual(result['a/b~c']['raw_ref']['pointer'],'/a~1b~0c')
        self.assertNotIn('SECRET',json.dumps(result))

    def test_04_reuse_owner_from_read_not_config(self):
        row=extract(trace([event('SOURCE_RANGE_READ',{'consumer':'HANDOFF_WORK_RECORD_PRODUCER','passage':PASSAGE},0),
                           event('EVIDENCE_ORIGINAL_REUSED',{'variant':'A','passage':PASSAGE},1)]))
        self.assertEqual(items(row,'source_reuses')[0]['provenance_owner'],'DP4_HANDOFF_PREFIX')

    def test_05_ambiguous_acquisition_not_inferred(self):
        read={'consumer':'REQUEST_EVIDENCE_ACQUISITION','passage':PASSAGE}
        row=extract(trace([event('SOURCE_RANGE_READ',read,0),event('SOURCE_RANGE_READ',read,1),
                           event('EVIDENCE_ORIGINAL_REUSED',{'passage':PASSAGE},2)]))
        self.assertEqual(items(row,'source_reuses')[0]['acquisition_link']['status'],'CONFLICT')

    def test_06_generation_actual_ingress(self):
        row=extract(trace([event('HANDOFF_RECORD_COMMITTED',{'activity_seq':1,'records':[RECORD]},0),
            event('AGENT_INGRESS',{'request':{'stimulus_id':'u','contract':CONTRACT}},1)]))
        self.assertIsNotNone(items(row,'record_occurrences')[0]['generation_ref'])
        self.assertEqual(row['observed_generation_exposures'][0]['distinct_actual_ingresses'],1)

    def test_07_stale_built_is_not_ingress(self):
        row=extract(trace([event('HANDOFF_RECORD_COMMITTED',{'records':[RECORD]},0),
            event('HANDOFF_CONTRACT_BUILT',CONTRACT,1),event('STALE_HANDOFF_DISCARDED',{},2)]))
        self.assertEqual(row['observed_generation_exposures'],[])
        self.assertEqual(items(row,'record_occurrences')[0]['phase'],'SELECTED_IN_BUILT_CAPSULE')

    def test_08_same_id_new_generation(self):
        second=deepcopy(RECORD);second['producer_node']='producer2'
        contract=deepcopy(CONTRACT);contract['capsule']['records']=[second]
        row=extract(trace([event('HANDOFF_RECORD_COMMITTED',{'records':[RECORD]},0),
            event('HANDOFF_RECORD_COMMITTED',{'records':[second]},1),
            event('AGENT_INGRESS',{'request':{'contract':contract}},2)]))
        self.assertEqual(len(items(row,'record_generations')),2)
        self.assertEqual(items(row,'record_occurrences')[0]['generation_ref']['pointer'],'/events/1')

    def test_09_duplicate_generation_is_conflict(self):
        row=extract(trace([event('HANDOFF_RECORD_COMMITTED',{'records':[RECORD]},0),
            event('HANDOFF_RECORD_COMMITTED',{'records':[RECORD]},1),
            event('AGENT_INGRESS',{'request':{'contract':CONTRACT}},2)]))
        self.assertIsNone(items(row,'record_occurrences')[0]['generation_ref'])
        self.assertEqual(items(row,'record_occurrences')[0]['generation_join_status'],'CONFLICT')

    def test_10_partial_trace_unknown_absence(self):
        value=trace([]);del value['end_ns'];row=extract(value)
        self.assertEqual(row['collections']['source_reads']['status'],'MISSING')
        self.assertFalse(row['trace_complete_for_absence_claims'])

    def test_11_request_outer_string_and_array(self):
        for content,expected in [(json.dumps({'work_records':[]} ),'/messages/1/content'),
            ([{'type':'image_url','image_url':{'url':'data:DO_NOT_DECODE'}},
              {'type':'text','text':json.dumps({'work_records':[]})}],'/messages/1/content/1/text')]:
            value,p=m.decoded_input({'messages':[{}, {'role':'user','content':content}]})
            self.assertEqual(p,expected);self.assertEqual(value,{'work_records':[]})

    def test_12_receiver_failed_and_duplicate_call(self):
        ref=save('raw/model/u-request.json',{'messages':[{}, {'role':'user','content':[
            {'type':'text','text':json.dumps({'contract':CONTRACT})}]}]})
        call={'call_uuid':'u','stage':'receiver_select_or_complete','role':'RECEIVER',
              'actual_api_sent':True,'status':'FAILED','request_file':ref['path'],
              'causal_trace':{'node_id':'receiver1','parent_ids':['producer1']}}
        row=extract(trace([event('HANDOFF_RECORD_COMMITTED',{'records':[RECORD]},0)],[call,deepcopy(call)]))
        self.assertEqual(row['observed_generation_exposures'][0]['distinct_receiver_operations'],1)
        occurrence=items(row,'record_occurrences')[0]
        self.assertEqual(occurrence['source_ref']['pointer'],'/messages/1/content/0/text')
        self.assertEqual(occurrence['decoded_input_pointer'],'/contract/capsule/records/0')
        self.assertNotIn('SECRET',json.dumps(row))

    def test_13_unsent_saved_input_not_receiver_supply(self):
        ref=save('raw/model/v-request.json',{'messages':[{}, {'role':'user','content':json.dumps({'contract':CONTRACT})}]})
        call={'call_uuid':'v','stage':'receiver_select_or_complete','actual_api_sent':False,'request_file':ref['path']}
        row=extract(trace([event('HANDOFF_RECORD_COMMITTED',{'records':[RECORD]},0)],[call]))
        self.assertEqual(row['observed_generation_exposures'],[])
        self.assertEqual(items(row,'record_occurrences')[0]['phase'],'AVAILABLE_IN_INPUT')

    def test_14_hash_mismatch_and_cap(self):
        ref=save('bounded.json',{'n':1});reader=m.Reader(FAKE)
        with self.assertRaises(m.EvidenceError):reader.read(dict(ref,sha256='0'*64),100)
        with self.assertRaises(m.EvidenceError) as e:reader.read(ref,1)
        self.assertEqual(e.exception.status,'LIMIT_EXCEEDED')

    def test_15_no_response_or_other_request_read(self):
        call={'stage':'UNSELECTED','request_file':str(FAKE/'MUST_NOT_EXIST'),
              'response_file':str(FAKE/'ALSO_MUST_NOT_EXIST'),'call_uuid':'unused'}
        row=extract(trace([],[call]));self.assertEqual(row['extraction_status'],m.COMPLETE)
        self.assertFalse(items(row,'call_links')[0]['request_file_ref']['content_inspected'])

    def test_16_path_traversal_and_duplicate_json(self):
        with self.assertRaises(m.EvidenceError):m.safe_path(str(FAKE/'../escape'),FAKE)
        with self.assertRaises(m.EvidenceError):m.strict_json('{"x":1,"x":2}')

    def test_17_receiver_chain_explicit_contract(self):
        row=extract(trace([event('AGENT_INGRESS',{'request':{'stimulus_id':'u','contract':CONTRACT}},0),
            event('RECEIVER_RESULT_AVAILABLE',{'receipt':{'stimulus_id':'u',
                'actual_result':{'contract':CONTRACT,'accepted':False,'errors':['FAIL'],
                                 'actually_supplied_source_spans':[SPAN]}}},1)]))
        self.assertEqual(items(row,'receiver_support')[0]['actual_contract_ingress_join']['status'],m.COMPLETE)

    def test_18_aggregate_cap_stops_future_reads(self):
        ref=save('aggregate.json',{});reader=m.Reader(FAKE);reader.total=m.CAPS['aggregate_input_bytes']
        with self.assertRaises(m.EvidenceError) as e:reader.read(ref,100)
        self.assertEqual(e.exception.status,'LIMIT_EXCEEDED');self.assertTrue(reader.aggregate_exhausted)

    def test_19_accepted_manifest_boundary_abi(self):
        jobs=[m.key_object((p,c,e,r)) for p,(cases,reps) in m.POPULATIONS.items()
              for c in m.CONFIGS for e in cases for r in range(1,reps+1)]
        sources=[]
        for name in ('19-common9-evaluation-contract.md','20-common-supplement-evaluation-contract.md'):
            ref=save('frozen/source/docs/'+name,b'FAKE CONTRACT\n',raw=True)
            sources.append({'relativePath':'docs/'+name,'frozen_path':ref['path'],'sha256':ref['sha256']})
        freeze=save('freeze.json',{'version':'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1',
            'jobs':jobs,'sources':sources,'binary':{'sha256':'f'*64}})
        approval=save('approval/APPROVAL-BINDING.json',{'formal_freeze_sha256':freeze['sha256']})
        evidence=save('process-observation.txt',b'FAKE STOPPED OBSERVATION\n',raw=True)
        q=save('quiescence.json',{'version':'JOINT441_QUIESCENT_SNAPSHOT_v1','controller_state':'STOPPED',
            'no_active_episode_processes':True,'evidence_refs':[evidence]})
        index=save('result-index.jsonl',b'',raw=True)
        inp=save('adapter-input.json',{'version':'JOINT441_QUANTITATIVE_INPUT_v1','formal_root':str(FAKE),
            'artifacts':{'freeze':freeze,'approval':approval,'quiescence':q,'result_index':index}})
        report=save('accepted-report/manifest.json',{'version':'JOINT441_QUANTITATIVE_MANIFEST_v1',
            'report_version':'JOINT441_QUANTITATIVE_REPORT_v1','status':'FAKE',
            'input_manifest':inp,'outputs':{'calls.jsonl':{'sha256':'c'*64,'bytes':12}},
            'sources':[{'path':'/fake/report_adapter.py','sha256':m.ADAPTER_SHA,'bytes':1}]})
        args=SimpleNamespace(input_manifest=Path(inp['path']),input_sha256=inp['sha256'],
            accepted_report_manifest=Path(report['path']),accepted_report_sha256=report['sha256'])
        old=m.FREEZE_SHA,m.APPROVAL_SHA
        m.FREEZE_SHA,m.APPROVAL_SHA=freeze['sha256'],approval['sha256']
        try:
            root,reader,keys,frozen,index,boundary=m.load_boundary(args)
            self.assertEqual(len(keys),441)
            self.assertEqual(boundary['accepted_calls_ref']['recorded_path'],str(FAKE/'accepted-report/calls.jsonl'))
            self.assertFalse((FAKE/'accepted-report/calls.jsonl').exists())
            self.assertEqual(boundary['accepted_accounting_linkage'],'REFERENCE_ONLY_NOT_INDEPENDENT_ACCOUNTING_VERIFICATION')
            # Existing output is rejected before any corpus read or mutation.
            args.output=FAKE/'postanalysis/occupied';args.output.mkdir(parents=True,exist_ok=True)
            with self.assertRaises(m.EvidenceError):m.execute(args)
        finally:m.FREEZE_SHA,m.APPROVAL_SHA=old

    def test_20_selected_request_count_cap(self):
        calls=[{'call_uuid':'cap'+str(i),'stage':'receiver_select_or_complete','actual_api_sent':True,
                'request_file':str(FAKE/'raw/model'/('cap'+str(i)+'-request.json'))} for i in range(101)]
        row=extract(trace([],calls))
        self.assertEqual(row['collections']['call_links']['status'],'LIMIT_EXCEEDED')
        self.assertTrue(any(x['detail']=='Selected-request count cap exceeded' for x in items(row,'join_issues')))


if __name__=='__main__':
    start=time.monotonic()
    suite=unittest.defaultTestLoader.loadTestsFromTestCase(FakeChecks)
    result=unittest.TextTestRunner(verbosity=1).run(suite)
    duration=time.monotonic()-start
    # Exact named fake inventory; no directory discovery or campaign traversal.
    names=('trace.json','pointer.json','raw/model/u-request.json','raw/model/v-request.json',
           'bounded.json','aggregate.json','freeze.json','approval/APPROVAL-BINDING.json',
           'process-observation.txt','quiescence.json','result-index.jsonl','adapter-input.json',
           'accepted-report/manifest.json','frozen/source/docs/19-common9-evaluation-contract.md',
           'frozen/source/docs/20-common-supplement-evaluation-contract.md')
    inventory=[{'path':str(FAKE/n),'bytes':(FAKE/n).stat().st_size,
                'sha256':m.digest((FAKE/n).read_bytes())} for n in names]
    total=sum(x['bytes'] for x in inventory)
    payload={'kind':'SYNTHETIC_ONLY','tests':result.testsRun,'failures':len(result.failures),
             'errors':len(result.errors),'runtime_seconds':duration,'fake_input_bytes':total,
             'fake_inputs':inventory,'source_sha256':m.digest(SOURCE.read_bytes()),
             'under_1MiB_fake_inputs':total<=1024*1024,'under_1second_test_target':duration<1}
    with (HERE/('fake-results-'+LABEL+'.json')).open('xb') as handle:handle.write(m.encoded(payload))
    raise SystemExit(0 if result.wasSuccessful() and total<=1024*1024 and duration<1 else 1)
