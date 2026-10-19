"""Tiny source-shaped synthetic checks; never reads real campaign evidence."""
from collections import Counter
from copy import deepcopy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import time
from types import SimpleNamespace
import unittest
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
SOURCE=HERE.parent.parent/'extract_dp1_dp4_mechanism_evidence.py'
spec=importlib.util.spec_from_file_location('mechanism_sidecar_stage02',SOURCE)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
LABEL=sys.argv[1] if len(sys.argv)>1 else 'attempt-01'
if LABEL not in ('attempt-01','attempt-02','attempt-03'):raise ValueError('Bounded attempt label required')
ROOT=HERE/('fake-inputs-'+LABEL);ROOT.mkdir(exist_ok=False)
NAMED=set();VIRTUAL_COUNTS=[]
KEY=('COMMON9','AAAA','S02',1)
SPAN={'source_id':'DOC','revision':1,'start_line':1,'end_line':1}
PASSAGE=dict(SPAN,text='SECRET ORIGINAL',acquisition_node='read1')
RECORD={'record_id':'R','producer_node':'producer1','source_spans':[SPAN],
        'supporting_passages':[PASSAGE],'interpretation':'SECRET INTERPRETATION'}
MANDATORY={'goal_id':'g','request_revision':1,'purpose':'SECRET PURPOSE','constraints':['SECRET CONSTRAINT']}
CONTRACT={'mandatory':MANDATORY,'capsule':{'records':[RECORD]}}


def save(name,value,raw=False):
    p=ROOT/name;p.parent.mkdir(parents=True,exist_ok=True)
    data=value if raw else m.encoded(value);p.write_bytes(data);NAMED.add(p)
    return {'path':str(p),'bytes':len(data),'sha256':m.digest(data)}


def event(kind,data,n):
    return {'kind':kind,'event_id':'e'+str(n),'elapsed_ns':n,'parent_ids':[],'goal_ids':['g'],'data':data}


def trace(events,calls=()):return {'events':events,'calls':list(calls),'end_ns':99,'returncode':1}
def items(row,name):return row['collections'][name]['items']
def extract(t):
    ref=save('tiny-trace.json',t);row=m.empty_row(KEY,run_id='FAKE')
    return m.extract_run(t,ref,row,m.Reader(ROOT),{'recorded_path':'FAKE_REFERENCE_ONLY'})
def commit():return event('HANDOFF_RECORD_COMMITTED',{'activity_seq':1,'records':[RECORD]},0)
def call(uuid='u',node='receiver1',parents=None,stage='receiver_select_or_complete',payload=None,array=False,actual=True):
    text=json.dumps({'contract':CONTRACT} if payload is None else payload)
    content=[{'type':'image_url','image_url':{'url':'data:DO_NOT_DECODE'}},{'type':'text','text':text}] if array else text
    ref=save('raw/model/'+uuid+'-request.json',{'messages':[{}, {'role':'user','content':content}]})
    causal={'node_id':node,'parent_ids':['producer1'] if parents is None else parents,'stage':stage,'role':'RECEIVER'}
    return {'call_uuid':uuid,'stage':stage,'role':'RECEIVER','actual_api_sent':actual,
            'status':'FAILED','request_file':ref['path'],'causal_trace':causal}


class Sink(io.BytesIO):
    def __exit__(self,*args):return False


def virtual_rows():
    original=Path.open;sinks=[]
    def opened(p,*args,**kwargs):
        if p.name=='mechanism-evidence.jsonl' and args and args[0]=='xb':
            sink=Sink();sinks.append(sink);return sink
        return original(p,*args,**kwargs)
    return patch.object(Path,'open',opened),sinks


def fixture():
    jobs=[m.key_object((p,c,e,r)) for p,(cases,reps) in m.POPULATIONS.items()
          for c in m.CONFIGS for e in cases for r in range(1,reps+1)]
    sources=[]
    for name in ('19-common9-evaluation-contract.md','20-common-supplement-evaluation-contract.md'):
        ref=save('frozen/source/docs/'+name,b'FAKE CONTRACT\n',raw=True)
        sources.append({'relativePath':'docs/'+name,'frozen_path':ref['path'],'sha256':ref['sha256']})
    frozen=save('freeze.json',{'version':'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1',
        'jobs':jobs,'sources':sources,'binary':{'sha256':'f'*64}})
    approved=save('approval/APPROVAL-BINDING.json',{'formal_freeze_sha256':frozen['sha256']})
    observation=save('fake-process-observation.txt',b'FAKE STOPPED\n',raw=True)
    q=save('quiescence.json',{'version':'JOINT441_QUIESCENT_SNAPSHOT_v1','controller_state':'STOPPED',
        'no_active_episode_processes':True,'evidence_refs':[observation]})
    index=save('result-index.jsonl',b'',raw=True)
    inp=save('adapter-input.json',{'version':'JOINT441_QUANTITATIVE_INPUT_v1','formal_root':str(ROOT),
        'artifacts':{'freeze':frozen,'approval':approved,'quiescence':q,'result_index':index}})
    report=save('accepted-report/manifest.json',{'version':'JOINT441_QUANTITATIVE_MANIFEST_v1',
        'report_version':'JOINT441_QUANTITATIVE_REPORT_v1','input_manifest':inp,
        'outputs':{'calls.jsonl':{'sha256':'c'*64,'bytes':12}},
        'sources':[{'path':'/fake/report_adapter.py','sha256':m.ADAPTER_SHA,'bytes':1}]})
    args=SimpleNamespace(formal_root=ROOT,input_manifest=Path(inp['path']),input_sha256=inp['sha256'],
        accepted_report_manifest=Path(report['path']),accepted_report_sha256=report['sha256'],
        output=ROOT/'postanalysis/rejected')
    return args,jobs,frozen,approved,report


class Checks(unittest.TestCase):
    def test_01_shared_reader_budget_and_failed_hash(self):
        ref=save('bound.json',{'n':1});r=m.Reader(ROOT);r.total=m.CAPS['aggregate_input_bytes']-1
        with self.assertRaises(m.EvidenceError):r.read(ref,100)
        self.assertEqual(r.attempts[-1]['bytes_read'],0)
        r=m.Reader(ROOT)
        with self.assertRaises(m.EvidenceError):r.read(dict(ref,sha256='0'*64),100)
        self.assertEqual(r.total,ref['bytes']);self.assertEqual(r.attempts[-1]['status'],'CONFLICT')

    def test_02_growing_file_cannot_overread_aggregate(self):
        ref=save('grow.bin',b'x',raw=True);p=Path(ref['path']);original=Path.open
        def opened(path,*args,**kwargs):
            if path==p and args and args[0]=='rb':
                with original(path,'wb') as h:h.write(b'0123456789')
            return original(path,*args,**kwargs)
        r=m.Reader(ROOT);r.total=m.CAPS['aggregate_input_bytes']-5
        with patch.object(Path,'open',opened),self.assertRaises(m.EvidenceError):r.read(ref,100)
        self.assertEqual(r.total,m.CAPS['aggregate_input_bytes'])
        self.assertEqual(r.attempts[-1]['bytes_read'],5)
        self.assertEqual(r.attempts[-1]['hash_scope'],'BYTES_ACTUALLY_READ_NOT_YET_FULL_FILE_VERIFIED')

    def test_03_near_exhausted_report_metadata(self):
        args,jobs,frozen,approved,report=fixture();r=m.Reader(ROOT);context={}
        real=r.pinned
        def pinned(name,sha,limit):
            if name==args.accepted_report_manifest:r.total=m.CAPS['aggregate_input_bytes']-1
            return real(name,sha,limit)
        with patch.object(m,'FREEZE_SHA',frozen['sha256']),patch.object(m,'APPROVAL_SHA',approved['sha256']),patch.object(r,'pinned',pinned):
            with self.assertRaises(m.EvidenceError):m.load_boundary(args,r,context)
        self.assertEqual(len(context['keys']),441)
        self.assertEqual(r.attempts[-1]['bytes_read'],0)
        self.assertEqual(r.attempts[-1]['recorded_path'],str(args.accepted_report_manifest))

    def test_04_saved_late_rejection_receipt_and_441_rows(self):
        args,jobs,frozen,approved,report=fixture();args.accepted_report_sha256='0'*64
        manager,sinks=virtual_rows()
        with patch.object(m,'FREEZE_SHA',frozen['sha256']),patch.object(m,'APPROVAL_SHA',approved['sha256']),manager:
            receipt=m.execute(args)
        p=args.output/'execution-receipt.json';NAMED.add(p);NAMED.add(args.output/'execution-receipt.sha256')
        saved=json.loads(p.read_bytes())
        self.assertEqual(saved['status'],'REJECTED_INPUT');self.assertEqual(saved['counts']['planned'],441)
        self.assertEqual(saved['input_bytes_read'],sum(x['bytes_read'] for x in saved['input_read_attempts']))
        self.assertEqual(sum(x['recorded_path']==str(args.input_manifest) for x in saved['input_read_attempts']),1)
        self.assertTrue(any(x['source_self'] for x in saved['input_read_attempts']))
        self.assertEqual(saved['input_read_attempts'][-1]['status'],'CONFLICT')
        rows=[json.loads(x) for x in sinks[0].getvalue().splitlines()]
        self.assertEqual(len(rows),441);self.assertTrue(all(x['extraction_status']!='COMPLETE_WITHIN_DECLARED_SCOPE' for x in rows))
        VIRTUAL_COUNTS.append({'case':'late_boundary_rejection','rows':len(rows),'bytes':len(sinks[0].getvalue())})
        with self.assertRaises(m.EvidenceError):m.execute(args)

    def test_05_exact_ingress_association_pointer(self):
        row=extract(trace([commit(),event('AGENT_INGRESS',{'request':{'contract':CONTRACT,'purpose':'SECRET P'}},1)]))
        purpose=items(row,'record_occurrences')[0]['association']['purpose']
        self.assertEqual(purpose['raw_ref']['pointer'],'/events/1/data/request/purpose')

    def test_06_exact_receiver_array_association_pointer(self):
        row=extract(trace([commit()],[call(array=True)]));association=items(row,'record_occurrences')[0]['association']
        self.assertEqual(association['purpose']['decoded_input_pointer'],'/contract/mandatory/purpose')
        self.assertEqual(association['purpose']['raw_ref']['pointer'],'/messages/1/content/1/text')
        self.assertEqual(association['constraints'][0]['decoded_input_pointer'],'/contract/mandatory/constraints/0')

    def test_07_exact_work_records_association_pointer(self):
        c=call(stage='handoff_select_details',payload={'work_records':[RECORD],'mandatory':MANDATORY})
        row=extract(trace([commit()],[c]));association=items(row,'record_occurrences')[0]['association']
        self.assertEqual(association['purpose']['decoded_input_pointer'],'/mandatory/purpose')

    def test_08_malformed_parent_ids_fail_closed(self):
        c=call();c['causal_trace']['parent_ids']=None
        row=extract(trace([commit()],[c]))
        self.assertNotEqual(row['collections']['record_occurrences']['status'],m.COMPLETE)
        self.assertNotEqual(row['collections']['call_links']['status'],m.COMPLETE)
        self.assertEqual(row['observed_generation_exposures'][0]['distinct_receiver_operations'],0)

    def test_09_malformed_supporting_passages_preserves_records(self):
        record=deepcopy(RECORD);record['supporting_passages']=None
        row=extract(trace([event('HANDOFF_RECORD_COMMITTED',{'records':[record]},0)]))
        self.assertEqual(len(items(row,'record_generations')),1)
        self.assertEqual(row['collections']['record_generations']['status'],'UNSUPPORTED')

    def test_10_malformed_index_keeps_all_planned_rows(self):
        args,jobs,frozen,approved,report=fixture();keys=m.validate_plan(jobs)
        malformed={'identity':dict(m.key_object(KEY),population=['INVALID'])}
        by_key,conflicts=m.partition_index([malformed],keys)
        self.assertEqual(len(conflicts),1)
        badcall=call();badcall['causal_trace']['parent_ids']=None
        record=deepcopy(RECORD);record['supporting_passages']=None
        indexed=[malformed]
        for k,run,t in [(keys[0],'fake-call',trace([commit()],[badcall])),
                        (keys[1],'fake-record',trace([event('HANDOFF_RECORD_COMMITTED',{'records':[record]},0)]))]:
            t.update(configuration=k[1],episode=k[2],rep=k[3],run_id=run,phase='FORMAL',binary_sha256='f'*64)
            ref=save('raw/runs/'+run+'/trace.json',t)
            indexed.append({'identity':m.key_object(k),'run_id':run,'trace_ref':ref})
        out=ROOT/'virtual-index';receipt={'outputs':{}};manager,sinks=virtual_rows()
        with manager:m.write_rows(out,keys,m.Reader(ROOT),receipt,index=indexed,
            freeze={'binary':{'sha256':'f'*64}},boundary={'accepted_calls_ref':{'recorded_path':'FAKE_REFERENCE'}})
        rows=[json.loads(x) for x in sinks[0].getvalue().splitlines()]
        self.assertEqual(len(rows),441)
        self.assertEqual(rows[0]['collections']['record_occurrences']['status'],'UNSUPPORTED')
        self.assertEqual(rows[1]['collections']['record_generations']['status'],'UNSUPPORTED')
        self.assertEqual(receipt['counts']['planned'],441)
        VIRTUAL_COUNTS.append({'case':'malformed_index','rows':441,'bytes':len(sinks[0].getvalue())})

    def test_11_row_serialization_failure_is_contained(self):
        row=m.empty_row(KEY);row['bad']={1,2}
        for name in m.COLLECTIONS:m.finish_collection(row,name,m.COMPLETE)
        data,status=m.serialize_row(row,KEY)
        self.assertEqual(status,'UNSUPPORTED');saved=json.loads(data)
        self.assertEqual(saved['bad']['unsupported_projection_type'],'set')
        self.assertTrue(all(c['status']!='COMPLETE_WITHIN_DECLARED_SCOPE' for c in saved['collections'].values()))

    def test_12_same_uuid_conflicting_causal_metadata(self):
        a=call();b=deepcopy(a);b['causal_trace']['parent_ids']=['OTHER']
        row=extract(trace([commit()],[a,b]));exposure=row['observed_generation_exposures'][0]
        self.assertEqual(exposure['distinct_receiver_operations'],0)
        self.assertEqual(len(exposure['unresolved_consumer_occurrences']),2)
        self.assertEqual(exposure['recorded_distinct_dispatch_uuids'],['u'])

    def test_13_same_node_different_uuid_is_conflict(self):
        row=extract(trace([commit()],[call('u'),call('v')]))
        exposure=row['observed_generation_exposures'][0]
        self.assertEqual(exposure['distinct_receiver_operations'],0)
        self.assertEqual(exposure['recorded_distinct_dispatch_uuids'],['u','v'])

    def test_14_missing_operation_node_not_counted(self):
        c=call();del c['causal_trace']['node_id']
        row=extract(trace([commit()],[c]));exposure=row['observed_generation_exposures'][0]
        self.assertEqual(exposure['distinct_receiver_operations'],0)
        self.assertEqual(exposure['unresolved_consumer_occurrences'][0]['status'],'MISSING')

    def test_15_conflicting_ingress_id_not_counted(self):
        a=event('AGENT_INGRESS',{'request':{'contract':CONTRACT}},1);b=deepcopy(a);b['data']['request']['other']='changed'
        row=extract(trace([commit(),a,b]));exposure=row['observed_generation_exposures'][0]
        self.assertEqual(exposure['distinct_actual_ingresses'],0)
        self.assertEqual(len(exposure['unresolved_consumer_occurrences']),2)

    def test_16_identical_failed_dispatch_deduplicates(self):
        c=call();row=extract(trace([commit()],[c,deepcopy(c)]));exposure=row['observed_generation_exposures'][0]
        self.assertEqual(exposure['distinct_receiver_operations'],1)
        self.assertEqual(exposure['raw_receiver_record_occurrences'],2)
        self.assertEqual(exposure['established_distinct_dispatch_uuids'],['u'])

    def test_17_future_read_cannot_prove_earlier_reuse(self):
        row=extract(trace([event('EVIDENCE_ORIGINAL_REUSED',{'passage':PASSAGE},0),
            event('SOURCE_RANGE_READ',{'passage':PASSAGE,'consumer':'HANDOFF_WORK_RECORD_PRODUCER'},1)]))
        reuse=items(row,'source_reuses')[0]
        self.assertEqual(reuse['acquisition_link']['status'],'MISSING');self.assertIsNone(reuse['provenance_owner'])

    def test_18_prior_read_and_ambiguous_prior_read(self):
        read=event('SOURCE_RANGE_READ',{'passage':PASSAGE,'consumer':'HANDOFF_WORK_RECORD_PRODUCER'},0)
        reuse=event('EVIDENCE_ORIGINAL_REUSED',{'passage':PASSAGE},2)
        self.assertEqual(items(extract(trace([read,reuse])),'source_reuses')[0]['provenance_owner'],'DP4_HANDOFF_PREFIX')
        self.assertEqual(items(extract(trace([read,deepcopy(read),reuse])),'source_reuses')[0]['acquisition_link']['status'],'CONFLICT')

    def test_19_future_read_cannot_prove_commit_support(self):
        row=extract(trace([commit(),event('SOURCE_RANGE_READ',{'passage':PASSAGE,'consumer':'HANDOFF_WORK_RECORD_PRODUCER'},1)]))
        self.assertEqual(items(row,'record_generations')[0]['supporting_acquisition_links'][0]['status'],'MISSING')

    def test_20_dp1_actual_producer_keys_and_prose_hashes(self):
        c=call(stage='evidence_publish_relations',payload={'observed_evidence':[PASSAGE],
            'actual_delivered':[{'text':'SECRET HISTORY','source_spans':[SPAN]}],'catalog':[SPAN]})
        row=extract(trace([],[c]));fields=items(row,'call_links')[0]['input_exposure']['decoded_input_fields']
        self.assertIn('observed_evidence',fields);self.assertIn('actual_delivered',fields)
        self.assertEqual(fields['observed_evidence'][0]['text']['decoded_input_pointer'],'/observed_evidence/0/text')
        self.assertNotIn('SECRET',json.dumps(row))

    def test_21_unsent_and_stale_built_remain_distinct(self):
        row=extract(trace([commit(),event('HANDOFF_CONTRACT_BUILT',CONTRACT,1),
            event('STALE_HANDOFF_DISCARDED',{},2)],[call(actual=False)]))
        self.assertEqual(row['observed_generation_exposures'],[])
        self.assertEqual({x['phase'] for x in items(row,'record_occurrences')},{'AVAILABLE_IN_INPUT','SELECTED_IN_BUILT_CAPSULE'})

    def test_22_incomplete_trace_never_complete_empty(self):
        t=trace([]);del t['end_ns'];row=extract(t)
        self.assertEqual(row['collections']['record_occurrences']['status'],'MISSING')
        self.assertFalse(row['trace_complete_for_absence_claims'])

    def test_23_missing_dispatch_uuid_is_unestablished(self):
        c=call();del c['call_uuid']
        identity=m.canonical_call_identities([c],'FAKE')[0]
        self.assertEqual(identity['status'],'MISSING');self.assertIsNone(identity['operation_key'])

    def test_24_missing_ingress_id_is_unestablished(self):
        ingress=event('AGENT_INGRESS',{'request':{'contract':CONTRACT}},1);del ingress['event_id']
        row=extract(trace([commit(),ingress]));exposure=row['observed_generation_exposures'][0]
        self.assertEqual(exposure['distinct_actual_ingresses'],0)
        self.assertEqual(exposure['unresolved_consumer_occurrences'][0]['status'],'MISSING')

    def test_25_identical_ingress_copy_deduplicates(self):
        ingress=event('AGENT_INGRESS',{'request':{'contract':CONTRACT}},1)
        row=extract(trace([commit(),ingress,deepcopy(ingress)]));exposure=row['observed_generation_exposures'][0]
        self.assertEqual(exposure['distinct_actual_ingresses'],1)
        self.assertEqual(exposure['raw_ingress_record_occurrences'],2)

    def test_26_commit_publication_order_is_not_universal_causality(self):
        ingress=event('AGENT_INGRESS',{'request':{'contract':CONTRACT}},0)
        row=extract(trace([ingress,commit()]))
        self.assertIsNotNone(items(row,'record_occurrences')[0]['generation_ref'])
        self.assertEqual(row['observed_generation_exposures'][0]['distinct_actual_ingresses'],1)


if __name__=='__main__':
    started=time.monotonic();suite=unittest.defaultTestLoader.loadTestsFromTestCase(Checks)
    result=unittest.TextTestRunner(verbosity=1).run(suite);elapsed=time.monotonic()-started
    inventory=[]
    for p in sorted(NAMED):
        b=p.read_bytes();inventory.append({'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
    total=sum(x['bytes'] for x in inventory)
    output={'kind':'SYNTHETIC_ONLY','tests':result.testsRun,'failures':len(result.failures),
        'errors':len(result.errors),'runtime_seconds':elapsed,'fake_file_bytes':total,'files':inventory,
        'virtual_output_evidence':VIRTUAL_COUNTS,'source_sha256':m.digest(SOURCE.read_bytes()),
        'under_1MiB_fake_files':total<=1024*1024,'under_1second_test_target':elapsed<1,
        'real_campaign_inputs_read':False,'actual_sidecar_cli_executed':False,
        'virtual_rows_note':'Full441 synthetic row writes use bounded memory sinks; no real extraction.'}
    with (HERE/('fake-results-'+LABEL+'.json')).open('xb') as h:h.write(m.encoded(output))
    raise SystemExit(0 if result.wasSuccessful() and total<=1024*1024 and elapsed<1 else 1)
