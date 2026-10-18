"""Explicitly invoked offline C01/C03 rebase on an immutable corrected DP05 crate.

Writing this adapter does not execute it. Run only in the coordinator-approved
model-free preparation window; no provider Client is constructed.
"""
import argparse
import ast
import copy
import difflib
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

REPO=next(p for p in Path(__file__).resolve().parents if (p/'benchmark/architecture/four_dp_poc_v2/run.py').is_file())
BASE=REPO/'benchmark/architecture/four_dp_poc_v2'
OLD=REPO/'results/architecture-evaluation/current/four-dp-qwen37-20261018/qa04/exercise.py'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def read(p):return json.loads(p.read_text())
def inventory(p):return {str(f.relative_to(p)):sha(f) for f in sorted(p.rglob('*')) if f.is_file() and not any(x in f.relative_to(p).parts for x in ('target','__pycache__'))}
def migrate(v):
    if isinstance(v,list):return [migrate(x) for x in v]
    if not isinstance(v,dict):return v
    x={k:migrate(z) for k,z in v.items()}
    if 'frame_id' in x and 'image_path' in x:
        x['version']={'number':x.pop('source_revision')};x['coordinate']={'unit':'pixels','dimensions':[x.pop('width'),x.pop('height')]};x.pop('coordinate_space');x['captured_at']={'value':x.pop('observed_at_ms')*1000,'unit':'microseconds'}
    elif 'id' in x and 'revision' in x and ('path' in x or 'text' in x):x['version']={'number':x.pop('revision')}
    return x

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--baseline-crate',type=Path,required=True);p.add_argument('--fixture-source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--baseline-freeze',type=Path);p.add_argument('--execute',action='store_true',help='Only after explicit model-free preparation-window authorization');a=p.parse_args()
    if not a.execute:raise SystemExit('No action taken. --execute is reserved for the authorized model-free preparation window.')
    baseline=a.baseline_crate.resolve();out=a.output.resolve()
    if out.exists():raise SystemExit('Preserve existing evidence; use a new output directory')
    out.mkdir(parents=True);before=inventory(baseline);save(out/'baseline-source-sha256.json',before)
    constants={}
    for node in ast.parse(OLD.read_text()).body:
        if isinstance(node,ast.Assign) and len(node.targets)==1 and isinstance(node.targets[0],ast.Name) and node.targets[0].id in ('WIRE','NORMALIZER','C01PROBE','C03PROBE'):constants[node.targets[0].id]=ast.literal_eval(node.value)
    assert len(constants)==4;save(out/'control-constants.json',constants);shutil.copy2(Path(__file__),out/'rebase_controls.py')
    commands=[];changes=[];checks=[]
    def command(args):
        result=subprocess.run(args,capture_output=True,text=True);commands.append({'args':list(map(str,args)),'returncode':result.returncode,'stdout':result.stdout,'stderr':result.stderr});save(out/'commands.json',commands)
        if result.returncode:raise RuntimeError('Offline control command failed; raw logs preserved')
    sys.path.insert(0,str(BASE));import run
    class CaptureClient(run.OfflineClient):
        def __init__(self,root):super().__init__(root);self.requests=[]
        def call(self,payload,meta):
            self.requests.append({'meta':copy.deepcopy(meta),'payload':copy.deepcopy(payload)});result,record=super().call(payload,meta)
            # Deliberate contract probe, not a model result: A/B must traverse the
            # actual VIA read/compose/retention path; C retains receiver ownership.
            if meta['stage'] in ('dp05_select_handoff_details','dp05_extract_record','dp05_repair_handoff_selection'):
                sources=payload['input']['manifest'];wanted=payload['input']['mandatory']['required_source_ids']
                result={'mode':'READ','summary':'OFFLINE C01/C03 actual VIA-read contract probe','references':[],'selected_evidence_spans':[],'selected_record_blocks':[],'reads':[{'source_id':s['id'],'start_line':s['allowed_lines'][0],'end_line':s['allowed_lines'][1]} for s in sources if s['id'] in wanted and not s.get('revoked')],'unknowns':[]}
            return result,record
    episode=next(e for e in read(a.fixture_source.resolve()) if e['id']=='dp05-e2')
    for task in ('C01','C03'):
        folder=out/task;crate=folder/'candidate';shutil.copytree(baseline,crate,ignore=shutil.ignore_patterns('target','__pycache__'));(crate/'src/bin').mkdir(exist_ok=True)
        if task=='C01':(crate/'src/model_wire.rs').write_text(constants['WIRE']);probe=constants['C01PROBE']
        else:
            source=crate/'src/source_contract.rs';text=source.read_text();source.write_text(text[:text.index('pub fn normalize(')]+constants['NORMALIZER'])
            main=crate/'src/main.rs';text=main.read_text();needle='let input:Value=serde_json::from_str(&line).unwrap();';assert text.count(needle)==1;main.write_text(text.replace(needle,needle+'\n    let input=source_contract::normalize_input(&input);'));probe=constants['C03PROBE']
        (crate/'src/bin/contract_probe.rs').write_text(probe);diff=[];changed=[]
        for source in sorted(baseline.rglob('*.rs')):
            relative=source.relative_to(baseline)
            if 'target' in relative.parts:continue
            patched=crate/relative
            if source.read_bytes()!=patched.read_bytes():changed.append(str(relative));diff.extend(difflib.unified_diff(source.read_text().splitlines(True),patched.read_text().splitlines(True),fromfile='a/'+str(relative),tofile='b/'+str(relative)))
        assert changed==(['src/model_wire.rs'] if task=='C01' else ['src/main.rs','src/source_contract.rs'])
        (folder/'actual.patch').write_text(''.join(diff));save(folder/'source-sha256.json',inventory(crate));changes.append({'task':task,'changed_operational_modules':changed,'count':len(changed),'new_test_only_probe_excluded':'src/bin/contract_probe.rs'})
        command(['cargo','build','--offline','--manifest-path',str(crate/'Cargo.toml'),'--bins'])
        if task=='C01':command([str(crate/'target/debug/contract_probe')])
        for variant in 'ABC':
            case=copy.deepcopy(episode);outgoing=migrate(case) if task=='C03' else case
            if task=='C03':
                inp=folder/f'{variant}-migrated.json';expected=folder/f'{variant}-expected.json';save(inp,{**outgoing,'variant':variant});save(expected,{**case,'variant':variant});command([str(crate/'target/debug/contract_probe'),str(inp),str(expected)])
            run.CRATE=crate;client=CaptureClient(folder/'acceptance-evidence');trace=run.execute(client,outgoing,variant,1,'QA04_CONTROL_OFFLINE')
            failures=[e for e in trace['events'] if e['kind'] in ('GOAL_FAILED','EPISODE_ERROR')];reads=[e['data'] for e in trace['events'] if e['kind']=='HANDOFF_SOURCE_READ'];retained=[e['data'] for e in trace['events'] if e['kind']=='COMMON_ORIGINAL_EVIDENCE_RETAINED']
            assert trace['returncode']==0 and not failures and any(e['kind']=='GOAL_COMPLETE' for e in trace['events']) and trace['calls'] and reads
            if variant in 'AB':assert any(x['consumer']=='VIA_COMPOSER' for x in reads) and any(x['actual_via_read_spans'] for x in retained)
            else:assert any(x['consumer']=='RECEIVER' for x in reads)
            save(folder/'requests'/(trace['run_id']+'.json'),client.requests);checks.append({'task':task,'dp':'05','variant':variant,'pass':True,'run_id':trace['run_id'],'deterministic_calls':len(trace['calls']),'actual_source_reads':len(reads),'common_retention_events':len(retained),'provider_calls':0})
    assert inventory(baseline)==before
    save(out/'acceptance-summary.json',{'baseline_crate':str(baseline),'baseline_source_inventory_sha256':sha(out/'baseline-source-sha256.json'),'original_baseline_freeze_sha256':sha(a.baseline_freeze) if a.baseline_freeze else None,'baseline_relation':'Original BASIC fixture/freeze plus separately pinned corrected candidate source inventory; not a claim that the original freeze already contained the retention fix.','fixture_source':str(a.fixture_source.resolve()),'fixture_sha256':sha(a.fixture_source),'control_constant_source':str(OLD),'control_constant_source_sha256':sha(OLD),'adapter_sha256':sha(Path(__file__)),'changes':changes,'checks':checks,'provider_calls':0,'scope':'Six actual patched Rust executions with deterministic offline responses, plus actual compatibility probes. C01=Rust/Python model wire and usage; C03=input-admission source/frame metadata. Source original-read retention path exercised for A/B, receiver-read path for C. No semantic quality/latency/cost inference. Other options were not rerun here.'})
    print(json.dumps({'output':str(out),'checks':len(checks),'pass':all(x['pass'] for x in checks),'counts':{x['task']:x['count'] for x in changes},'provider_calls':0}))
if __name__=='__main__':main()
