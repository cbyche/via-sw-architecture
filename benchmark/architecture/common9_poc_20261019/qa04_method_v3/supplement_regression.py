"""Run the owned supplement6 CLI against an explicit QA04 candidate copy.

Fresh subprocesses keep supplemental oracles and Python module names isolated
from common9. Both populations retain their own exact obligation counts.
"""
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

COUNTS={'responses_planned':12,'responses_observed':12,'goals_planned':8,'goals_observed':8,'bindings_planned':6,'bindings_correct':6}


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def read(path):return json.loads(Path(path).read_text())
def dump(path,value):path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')


def inventory(benchmark):
    return {str(p.relative_to(benchmark)):sha(p)for p in sorted(benchmark.rglob('*'))if p.is_file()and '__pycache__'not in p.parts and p.suffix not in ('.pyc','.pyo')}


def frozen_benchmark(freeze,digest,baseline_digest=None):
    freeze=Path(freeze).resolve()
    assert sha(freeze)==digest,'Supplement exact freeze mismatch'
    manifest=read(freeze)
    if baseline_digest is not None:assert manifest['common_baseline_freeze_sha256']==baseline_digest
    benchmark=freeze.parent/'frozen/source/benchmark/architecture/common_supplement6_20261019'
    assert Path(manifest['benchmark_path']).resolve()==benchmark.resolve()
    expected={}
    for entry in manifest['sources']:
        p=Path(entry['frozen_path']);assert p.is_file()and sha(p)==entry['sha256'],'Supplement frozen dependency changed: '+entry['relativePath']
        expected[str(p.relative_to(benchmark))]=entry['sha256']
    assert inventory(benchmark)==expected,'Supplement snapshot inventory changed'
    return benchmark


def execute(crate,configuration,task,output,benchmark,freeze=None,digest=None,baseline_digest=None,development_only=False,workers=1):
    if not development_only:
        assert freeze is not None and digest is not None,'Final15 acceptance requires exact supplement dependency freeze'
        assert frozen_benchmark(freeze,digest,baseline_digest).resolve()==benchmark.resolve()
    assert 1<=workers<=2
    before=inventory(benchmark);binary=crate/'target/debug/common9_poc';binary_sha=sha(binary)
    command=[sys.executable,str(benchmark/'run_offline.py'),'--crate',str(crate),'--root',str(output/'supplement6'),'--configuration',configuration,'--workers',str(workers),'--time-scale','.05']
    if task=='C03':
        # The exact option is supplied by the supplement owner; fail closed if
        # that released CLI does not expose the actual C03 input transform.
        command.extend(['--input-transform-module',str(Path(__file__).parent/'patches.py')])
    started=time.monotonic_ns();result=subprocess.run(command,capture_output=True,text=True)
    dump(output/'supplement6-command.json',{'command':command,'start_ns':started,'end_ns':time.monotonic_ns(),'returncode':result.returncode,'stdout':result.stdout,'stderr':result.stderr,'development_only':development_only,'binary_sha256':binary_sha,'supplement_freeze':str(freeze)if freeze else None,'supplement_freeze_sha256':digest,'dependency_sha256':before,'provider_calls':0})
    assert before==inventory(benchmark),'Supplement dependencies changed during copied-candidate regression'
    assert result.returncode==0,'Supplement CLI failed; command output and raw failures retained'
    summary=read(output/'supplement6/summary.json');assert summary['status']=='PASS'and summary['completed']==summary['planned']==6 and summary['actual_API_calls']==0
    assert summary['binary_sha256']==binary_sha
    rows=[]
    fixture_map={e['id']:e for e in read(benchmark/'fixtures.json')};expected=set(fixture_map)
    assert len(expected)==6 and {r['episode']for r in summary['rows']}==expected
    for row in summary['rows']:
        assert row['configuration']==configuration and row['rep']==1
        trace_path=Path(row['trace']);trace=read(trace_path);check_path=trace_path.parents[3]/'regression.json'
        # Raw path: episodes/<configuration-case>/raw/runs/<run>/trace.json.
        checks=read(check_path)
        initial_path=trace_path.parent/'candidate-initial.json';actual_initial=read(initial_path)
        canonical={'id':row['episode'],'configuration':dict(zip(['id','evidence','execution','grounding','handoff'],[configuration,*configuration])),'initial':fixture_map[row['episode']]['initial']}
        transform_proof=None
        if task=='C03':
            from patches import migrate_c03
            assert actual_initial==migrate_c03(canonical),'C03 input wire did not actually migrate'
            transform_proof=read(trace_path.parents[3]/'input-transform-provenance.json')
            assert transform_proof['callable']=='migrate_c03'and transform_proof['sha256']==sha(Path(__file__).parent/'patches.py')
        else:assert actual_initial==canonical,'Unexpected supplemental input mutation'
        assert row['returncode']==0 and row['regression_status']=='PASS'and row['technical']=='CONSISTENT'
        assert checks['status']=='PASS'and checks['evaluation']['version']=='COMMON9-EVALUATOR-v3'and checks['evaluation']['evaluation_mode']=='strict'and checks['evaluation']['primary_measurement_eligibility']=='ELIGIBLE'
        assert trace['actual_API_calls']==0 and trace['binary_sha256']==binary_sha
        assert not any(c.get('actual_api_sent')is True for c in trace.get('calls',[]))
        rows.append({'population':'COMMON_SUPPLEMENT6','episode':row['episode'],'run_id':trace['run_id'],'trace':str(trace_path),'trace_sha256':sha(trace_path),'candidate_initial':str(initial_path),'candidate_initial_sha256':sha(initial_path),'input_transform':transform_proof,'regression':str(check_path),'regression_sha256':sha(check_path),'checks':checks,'provider_calls':0})
    totals={k:sum(row['checks']['counts'][k]for row in rows)for k in COUNTS}
    assert totals==COUNTS,totals
    dump(output/'supplement6-checks.json',{'population':'COMMON_SUPPLEMENT6','rows':rows,'totals':totals,'dependency_sha256':before,'supplement_freeze_sha256':digest,'binary_sha256':binary_sha,'development_only':development_only,'provider_calls':0})
    return rows
