"""Temporary synthetic-only assembly checks; every analyzer runner is fake."""
import argparse
from contextlib import ExitStack
import copy
import io
import json
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

import report_adapter as a
import record_joint_report_inputs as g
from test_report_adapter import synthetic_plan, record


class Fixture:
    def __init__(self, root):
        self.root=root
        self.jobs,self.oracles=synthetic_plan()
        self.refs={}
        self.source_refs=[]
        self.oracles_refs={}
        for pop,relative in [('COMMON9','benchmark/architecture/common9_poc_20261019/oracles.json'),('COMMON_SUPPLEMENT6','benchmark/architecture/common_supplement6_20261019/oracles.json')]:
            reference=self.save('frozen/source/'+relative,self.oracles[pop])
            self.oracles_refs[pop]=reference
            self.source_refs.append(dict(relativePath=relative,frozen_path=reference['path'],sha256=reference['sha256']))
        analyzer=self.save('frozen/source/'+g.ANALYZER_RELATIVE,'# synthetic: never executed\n',raw=True)
        self.analyzer_sha=analyzer['sha256']
        self.source_refs.append(dict(relativePath=g.ANALYZER_RELATIVE,frozen_path=analyzer['path'],sha256=analyzer['sha256']))
        for i in range(109):
            relative='synthetic-dependency-'+str(i)+'.txt'
            reference=self.save('frozen/source/'+relative,'fake source only',raw=True)
            self.source_refs.append(dict(relativePath=relative,frozen_path=reference['path'],sha256=reference['sha256']))
        self.freeze={'version':'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1','jobs':self.jobs,'sources':self.source_refs,'binary':{'sha256':'b'*64},'ledger_path':str(root/'never-read-live-budget.json')}
        self.refs['freeze']=self.save('freeze.json',self.freeze)
        qa04={'accepted_independent_changes':42,'combined_regression_episodes':630,'actual_provider_api_calls':0,
            'rows':[dict(configuration=cfg,task='C0'+str(i)) for cfg in a.CONFIGS for i in range(1,7)],
            'all_counts_by_configuration':{cfg:[1,1,3,3,1,2] for cfg in a.CONFIGS},'mean_changed_operational_modules':{cfg:11/6 for cfg in a.CONFIGS}}
        self.qa_refs={'report':self.save('approval/report.json',qa04),'classification':self.save('approval/count.json',{'fake':True}),'raw630':self.save('approval/raw.json',{'fake':True})}
        review={'status':'PASS_QA04_FUNCTIONAL_AND_MODULE_ACCEPTANCE_WITH_RETAINED_RSS_DIAGNOSTICS','input_sha256':{'report_json':self.qa_refs['report']['sha256'],'count_classification_v2':self.qa_refs['classification']['sha256'],'raw630_audit':self.qa_refs['raw630']['sha256']}}
        self.qa_refs['final_review']=self.save('approval/review.json',review)
        self.qa_refs['checkpoint']=self.save('approval/checkpoint.json',{'status':'STABLE_QA04_WRITES_CLOSED','writes_closed':True,'report_json':self.qa_refs['report'],'count_classification':self.qa_refs['classification'],'independent_reviews':{'raw630':self.qa_refs['raw630'],'final_qa04_acceptance':self.qa_refs['final_review']}})
        self.refs['approval']=self.save('approval/APPROVAL-BINDING.json',{'formal_freeze_sha256':self.refs['freeze']['sha256'],'artifacts':[dict(role=key,original_path=reference['path'],copied_path=reference['path'],sha256=reference['sha256'],bytes=reference['bytes'],status='historical-label-not-final-gate') for key,reference in self.qa_refs.items()]})
        self.records=[record(self.oracles,('COMMON9','AAAA','S02',1),eligible=False),record(self.oracles,('COMMON_SUPPLEMENT6','AAAA','S80',1))]
        self.records[0]['result']['functional_failures']=[{'kind':'WORKFLOW_FAILED','synthetic':True}]
        traces=[]
        provenance=[]
        for r in self.records:
            trace=dict(r['identity'],phase='FORMAL',run_id=r['run_id'],binary_sha256='b'*64,candidate_input_file=str(root/'raw/runs'/r['run_id']/'candidate-input.json'),returncode=0,end_ns=10,calls=[])
            reference=self.save('raw/runs/'+r['run_id']+'/trace.json',trace)
            traces.append(trace)
            provenance.append({'run_id':r['run_id'],'trace_path':reference['path'],'sha256':reference['sha256']})
        self.refs['runs']=self.save('runs.jsonl',''.join(json.dumps(t)+'\n' for t in traces),raw=True)
        progress=[dict(r['identity'],state=state) for r in self.records for state in ['STARTED','DISPATCHED','TERMINATED']]
        self.refs['progress']=self.save('snapshots/progress.jsonl',''.join(json.dumps(r)+'\n' for r in progress),raw=True)
        self.refs['progress']['snapshot_of']=str(root/'campaign-progress.jsonl')
        self.refs['status']=self.save('snapshots/status.json',{'planned':441,'completed':2})
        self.refs['status']['snapshot_of']=str(root/'status.json')
        self.refs['ledger']=self.save('snapshots/ledger.json',{'limit_usd':40,'calls':{'historical-unknown':{'state':'UNKNOWN','scope_root':'/prior','amount_usd':'.2'}}})
        self.refs['ledger']['snapshot_of']=self.freeze['ledger_path']
        evidence=self.save('snapshots/quiescence-evidence.txt','fake quiescence evidence; no PID inspection',raw=True)
        self.refs['quiescence']=self.save('snapshots/quiescence.json',{'version':'JOINT441_QUIESCENT_SNAPSHOT_v1','controller_state':'STOPPED','no_active_episode_processes':True,'progress_complete':True,'evidence_refs':[evidence],**{name+'_snapshot_sha256':self.refs[name]['sha256'] for name in ['progress','status','ledger']}})
        observed={a.identity(r['identity']) for r in self.records}
        self.joint={'version':'JOINT441_SEPARATE_POPULATION_ANALYSIS_v1','freeze_sha256':self.refs['freeze']['sha256'],
            'primary_common9':{'planned_episodes':315,'observed_episodes':1,'results':[self.records[0]['result']],'summary':{'FORMAL:AAAA':{'original_summary_marker':'keep primary'}}},
            'supplement6':{'planned_episodes':126,'observed_episodes':1,'results':[self.records[1]['result']],'summary':{'FORMAL:AAAA':{'original_summary_marker':'keep supplement'}},'mechanism_activation':[dict(version='CONTRACT20_OBSERVED_ACTIVATION_v1',run_id=self.records[1]['run_id'],configuration='AAAA',episode='S80',rep=1,dp2={'activation':'SYNTHETIC_NOT_A_JUDGMENT'})]},
            'trace_provenance':provenance,'total_actual_recorded_API_cost_usd':.2,'unknown_call_count':0,
            'unexecuted_planned_identities':[{field:j[field] for field in ['configuration','episode','rep']} for j in self.jobs if a.identity(j) not in observed]}
        self.plan={'version':g.PLAN_VERSION,'formal_root':str(root),'artifacts':self.refs,'oracles':self.oracles_refs,'qa04':{key:{'approval_role':key,'ref':reference} for key,reference in self.qa_refs.items()}}
        self.plan_ref=self.save('postanalysis/synthetic-plan.json',self.plan)
        self.calls=[]

    def save(self,name,value,raw=False):
        path=self.root/name
        path.parent.mkdir(parents=True,exist_ok=True)
        data=value.encode() if raw else a.json_bytes(value)
        path.write_bytes(data)
        return dict(path=str(path),sha256=a.sha(data),bytes=len(data))

    def patches(self):
        stack=ExitStack()
        stack.enter_context(patch.object(g,'FREEZE_SHA',self.refs['freeze']['sha256']))
        stack.enter_context(patch.object(g,'APPROVAL_SHA',self.refs['approval']['sha256']))
        stack.enter_context(patch.object(g,'ANALYZER_SHA',self.analyzer_sha))
        stack.enter_context(patch.dict(a.QA04_HASHES,{key:ref['sha256'] for key,ref in self.qa_refs.items()},clear=True))
        return stack

    def runner(self,argv,stdout,stderr,cwd):
        self.calls.append(argv)
        assert argv == [str(Path(__import__('sys').executable).resolve()),str(self.root/'frozen/source'/g.ANALYZER_RELATIVE),'--root',str(self.root),'--output',argv[-1]]
        assert '--offline' not in argv
        assert cwd == str((self.root/'frozen/source'/g.ANALYZER_RELATIVE).parent)
        Path(argv[-1]).write_bytes(a.json_bytes(self.joint))
        stdout.write(b'synthetic fake runner; no process\n')
        return 0

    def execute(self,runner=None,name='assembly'):
        return g.record_and_assemble(Path(self.plan_ref['path']),self.plan_ref['sha256'],self.root/'postanalysis'/name,runner=runner or self.runner)


class SplitChecks(unittest.TestCase):
    def test_unchanged_split_and_associations_missing_jobs_both_populations(self):
        with tempfile.TemporaryDirectory(prefix='input-assembly-synthetic-') as temp:
            f=Fixture(Path(temp).resolve())
            with f.patches():
                manifest=f.execute()
                self.assertEqual(manifest['status'],'ASSEMBLED_NOT_REPORTED')
                self.assertEqual(len(f.calls),1)
                reference=manifest['adapter_input_ref']
                supplied=json.loads(Path(reference['path']).read_text())
                mappings=json.loads((Path(reference['path']).parent/'fragment-provenance.json').read_text())
                original=json.loads(Path(mappings['original_analysis_ref']['path']).read_text())
                for mapping in mappings['fragments']:
                    copy_value=json.loads(Path(mapping['fragment_ref']['path']).read_text())
                    if 'original_json_pointer' in mapping:
                        self.assertEqual(copy_value,a.get_pointer(original,mapping['original_json_pointer']))
                    else:
                        self.assertEqual(copy_value,{field:a.get_pointer(original,pointer) for field,pointer in mapping['original_field_selectors'].items()})
                input_kwargs=a.load_inputs(Path(reference['path']),reference['sha256'],expected_freeze=f.refs['freeze']['sha256'],expected_approval=f.refs['approval']['sha256'])
                self.assertEqual(len(input_kwargs['records']),2)
                self.assertTrue(all(r['provenance_valid'] for r in input_kwargs['records']))
                self.assertEqual(input_kwargs['records'][0]['result']['functional_failures'],f.records[0]['result']['functional_failures'])
                self.assertEqual(input_kwargs['records'][0]['result']['primary_measurement_eligibility'],'NOT_CERTIFIED')
                self.assertNotIn('raw_pairs',supplied['artifacts'])
                self.assertEqual(len((Path(reference['path']).parent/'planned-identities.jsonl').read_text().splitlines()),441)
                receipt=json.loads((Path(reference['path']).parent/'analysis-execution.json').read_text())
                self.assertFalse(receipt['actual_process_invoked'])
                self.assertEqual(receipt['invocation_kind'],'SYNTHETIC_FAKE_NO_PROCESS')
                self.assertEqual(len(receipt['source_sha256']),112)
                for name,ref in manifest['outputs'].items():
                    self.assertEqual(a.sha(Path(ref['path']).read_bytes()),ref['sha256'],name)

    def test_duplicate_mismatched_phase_job_and_trace_rejected_without_selection(self):
        with tempfile.TemporaryDirectory(prefix='input-assembly-synthetic-') as temp:
            f=Fixture(Path(temp).resolve())
            for kind in ['duplicate','phase','job','trace','missing-label']:
                bad=copy.deepcopy(f.joint)
                if kind=='duplicate':
                    bad['primary_common9']['results'].append(copy.deepcopy(bad['primary_common9']['results'][0]))
                    bad['primary_common9']['observed_episodes']=2
                elif kind=='phase':
                    bad['primary_common9']['results'][0]['phase']='OFFLINE'
                elif kind=='job':
                    bad['primary_common9']['results'][0]['episode']='S80'
                elif kind=='trace':
                    bad['trace_provenance'][0]['trace_path']='/other-generation/trace.json'
                else:
                    bad['unexecuted_planned_identities']=[]
                with f.patches(),self.assertRaises(a.InputError):
                    g.validate_joint(bad,f.freeze,f.root)


class FailureChecks(unittest.TestCase):
    def test_failed_analysis_receipt_and_partial_output_preserved(self):
        with tempfile.TemporaryDirectory(prefix='input-assembly-failure-synthetic-') as temp:
            f=Fixture(Path(temp).resolve())
            def failed(argv,stdout,stderr,cwd):
                f.calls.append(argv)
                stderr.write(b'synthetic analysis failure\n')
                Path(argv[-1]).write_text('partial original output, not JSON')
                return 7
            with f.patches():
                manifest=f.execute(failed)
            self.assertEqual(manifest['status'],'ASSEMBLY_FAILED_EVIDENCE_PRESERVED')
            output=f.root/'postanalysis/assembly'
            receipt=json.loads((output/'analysis-execution.json').read_text())
            self.assertEqual(receipt['returncode'],7)
            self.assertEqual((output/'joint-analysis.json').read_text(),'partial original output, not JSON')
            self.assertEqual(receipt['output_ref']['sha256'],a.sha((output/'joint-analysis.json').read_bytes()))
            self.assertFalse((output/'adapter-input.json').exists())
            self.assertIn('synthetic analysis failure',(output/'analysis.stderr.log').read_text())

    def test_invalid_success_output_keeps_execution_and_original(self):
        with tempfile.TemporaryDirectory(prefix='input-assembly-invalid-synthetic-') as temp:
            f=Fixture(Path(temp).resolve())
            f.joint['primary_common9']['results'][0]['run_id']='unmatched-run'
            with f.patches():
                manifest=f.execute()
            self.assertEqual(manifest['status'],'ASSEMBLY_FAILED_EVIDENCE_PRESERVED')
            self.assertTrue((f.root/'postanalysis/assembly/joint-analysis.json').exists())
            self.assertEqual(json.loads((f.root/'postanalysis/assembly/analysis-execution.json').read_text())['returncode'],0)
            self.assertFalse((f.root/'postanalysis/assembly/adapter-input.json').exists())

    def test_source_mismatch_blocks_runner_and_output_is_immutable(self):
        with tempfile.TemporaryDirectory(prefix='input-assembly-input-synthetic-') as temp:
            f=Fixture(Path(temp).resolve())
            Path(f.source_refs[-1]['frozen_path']).write_text('wrong source')
            with f.patches():
                manifest=f.execute()
                self.assertEqual(manifest['status'],'ASSEMBLY_FAILED_EVIDENCE_PRESERVED')
                self.assertEqual(f.calls,[])
                with self.assertRaises(a.InputError):
                    f.execute()
            self.assertFalse((f.root/'postanalysis/assembly/analysis-execution.json').exists())

    def test_oversized_analysis_and_runner_exception_are_preserved(self):
        with tempfile.TemporaryDirectory(prefix='input-assembly-bounds-synthetic-') as temp:
            f=Fixture(Path(temp).resolve())
            f.plan['analysis_max_bytes']=1
            f.plan_ref=f.save('postanalysis/synthetic-plan.json',f.plan)
            with f.patches():
                result=f.execute(name='oversize')
                self.assertEqual(result['status'],'ASSEMBLY_FAILED_EVIDENCE_PRESERVED')
                receipt=json.loads((f.root/'postanalysis/oversize/analysis-execution.json').read_text())
                self.assertIsNone(receipt['output_ref'])
                self.assertIsNotNone(receipt['output_error'])
                def raises(argv,stdout,stderr,cwd):
                    raise OSError('synthetic failed invocation')
                result=f.execute(raises,name='exception')
                self.assertEqual(result['status'],'ASSEMBLY_FAILED_EVIDENCE_PRESERVED')
                receipt=json.loads((f.root/'postanalysis/exception/analysis-execution.json').read_text())
                self.assertEqual(receipt['exception']['type'],'OSError')


if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--batch',choices=['split','failures'],required=True)
    parser.add_argument('--record',type=Path,required=True)
    args=parser.parse_args()
    started=time.perf_counter()
    stream=io.StringIO()
    outcome=unittest.TextTestRunner(stream=stream,verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase({'split':SplitChecks,'failures':FailureChecks}[args.batch]))
    elapsed=time.perf_counter()-started
    result={'status':'PASS' if outcome.wasSuccessful() else 'FAIL','tests':outcome.testsRun,'elapsed_seconds':elapsed,'batch':args.batch,
        'scope':'TEMPORARY_SYNTHETIC_ONLY_FAKE_RUNNER_NO_PROCESS_NO_CORPUS_OR_ANALYZER_EVALUATOR_MODEL_PARSER_BUILD_NETWORK',
        'source_sha256':{'record_joint_report_inputs.py':a.sha(Path(g.__file__).read_bytes()),'test_record_joint_report_inputs.py':a.sha(Path(__file__).read_bytes()),'accepted_report_adapter.py':a.sha(Path(a.__file__).read_bytes()),'accepted_synthetic_fixture_helper.py':a.sha(Path(__file__).with_name('test_report_adapter.py').read_bytes())},'stdout':stream.getvalue()}
    args.record.parent.mkdir(parents=True,exist_ok=True)
    with args.record.open('x') as handle:
        json.dump(result,handle,indent=2)
        handle.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k!='stdout'}))
    if not outcome.wasSuccessful():
        print(stream.getvalue())
        raise SystemExit(1)
