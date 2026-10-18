"""Approved post120+27 E4×ABC×3 compact line-number diagnostic, fresh worker each trial."""
import argparse,hashlib,importlib.util,json,shutil,subprocess,sys,time,uuid
from pathlib import Path

def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,v):Path(p).parent.mkdir(parents=True,exist_ok=True);Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def prerequisites(root):
 stopped=root/'basic-dp05-reuse-corrected-v1';stop=read(stopped/'PHASE-STOPPED.json');assert stop['completed_trials']==11 and sha(stopped/'runs.jsonl')==stop['runs_sha256']
 expected={'basic-v1':120,'tactic-dp04-A4':12,'tactic-dp04-B4':12,'tactic-dp01-A2':3,'basic-dp05-reuse-union-corrected-v2':36}
 for name,n in expected.items():
  p=root/name;rs=[json.loads(x)for x in(p/'runs.jsonl').read_text().splitlines()if x];jobs=read(p/'MODEL-CALLS-AUTHORIZED.json')['allowed_jobs'];key=lambda r:(r['dp'],r['episode'],r['variant'],r['repetition'])
  assert len(rs)==n and {key(x)for x in rs}=={key(x)for x in jobs},'Must finish120+27+correctedDP05 36 before line diagnostic: '+name

def prepare(root):
 prerequisites(root);base=root/'basic-dp05-reuse-union-corrected-v2';folder=root/'diagnostic-dp05-numbered-lines';assert not folder.exists(),'New immutable phase required';folder.mkdir()
 m=read(base/'freeze.json');shutil.copytree(base/'frozen',folder/'frozen');m['source_snapshot']=str((folder/'frozen/source').resolve());m['binary']['path']=str((folder/'frozen/bin/four_dp_poc').resolve())
 for pin in m['evaluator_external_sources'].values():pin['snapshot']=str(folder/Path(pin['snapshot']).relative_to(base))
 harness=folder/'frozen/harness';harness.mkdir()
 canonical=Path(__file__).resolve().parent
 for name in ['line_number_diagnostic.py','compact_source_lines.py']:shutil.copy2(canonical/name,harness/name)
 repo=next(p for p in root.parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir())
 for source in [canonical/'line_number_diagnostic.py',canonical/'compact_source_lines.py']:
  relative=source.relative_to(repo);target=folder/'frozen/source'/relative;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target);m['source'][str(relative)]=sha(source)
 m['baseline_freeze_sha256']=sha(base/'freeze.json');m['diagnostic_scope']='Input representation only: compact actual original line numbers in model-payload copies; original Rust state/source/guards/oracles/model/caps/recovery unchanged. All ABC same E4×3. Does not replace failures in BASIC.'
 m['effective_harness_source']={'snapshot':str(harness.resolve()),'sha256':{p.name:sha(p)for p in harness.iterdir()},'execution':'Frozen line_number_diagnostic.py worker subclasses unchanged frozen-baseline Client.call. Frozen compact_source_lines.py formats deep copies only. Actual original payloads and transmitted requests retained per call.','candidate_binary_unchanged':True}
 save(folder/'freeze.json',m);jobs=[{'dp':'05','episode':'dp05-e4','variant':v,'repetition':rep}for rep in range(1,4)for v in ('ABC'[(rep-1)%3:]+'ABC'[:(rep-1)%3])]
 auth={'phase':'DIAGNOSTIC','authorization':'Root explicitly approved nine E4 ABC×3 compact original-line representation trials after BASIC120,27 comparison trials and correctedDP05 ABC36; no live tuning/rerun; same global $4.','campaign_root':str(root),'global_budget_usd':4,'freeze_sha256':sha(folder/'freeze.json'),'allowed_jobs':jobs,'allowed_tactic':'ORIGINAL_LINE_NUMBERS'}
 save(folder/'MODEL-CALLS-AUTHORIZED.json',auth);save(folder/'plan.json',{'phase':'DIAGNOSTIC','tactic':'ORIGINAL_LINE_NUMBERS','jobs':jobs,'serial_episode_execution':True,'fresh_python_each_trial':True,'baseline_same_case':str(base),'not_candidate_structure_comparison':True});(folder/'budget.json').symlink_to(root/'budget.json')
 print(json.dumps({'prepared':str(folder),'freeze_sha256':sha(folder/'freeze.json'),'binary_sha256':m['binary']['sha256'],'harness':m['effective_harness_source']}),flush=True)

def verify_harness(folder):
 m=read(folder/'freeze.json');pin=m['effective_harness_source'];assert sha(m['binary']['path'])==m['binary']['sha256']
 for name,wanted in pin['sha256'].items():assert sha(Path(pin['snapshot'])/name)==wanted,'Effective harness changed '+name
 return m

def worker(folder,variant,rep):
 root=folder.parent;prerequisites(root);m=verify_harness(folder);auth=read(folder/'MODEL-CALLS-AUTHORIZED.json');key={'dp':'05','episode':'dp05-e4','variant':variant,'repetition':rep};assert key in auth['allowed_jobs']and sha(folder/'freeze.json')==auth['freeze_sha256']
 repo=next(p for p in root.parents if(p/'benchmark/architecture/four_dp_poc_v2').exists());base=repo/'benchmark/architecture/four_dp_poc_v2';sys.path.insert(0,str(base))
 from bridge import Client,ModelError
 from run import execute,verify_freeze
 verify_freeze(folder)
 formatter_path=Path(m['effective_harness_source']['snapshot'])/'compact_source_lines.py';spec=importlib.util.spec_from_file_location('frozen_compact_source_lines',formatter_path);formatter=importlib.util.module_from_spec(spec);spec.loader.exec_module(formatter)
 class CompactClient(Client):
  def call(self,payload,meta):
   if meta['dp']!='05':raise RuntimeError('Diagnostic formatter is DP05 only')
   label='formatter-'+uuid.uuid4().hex;before=folder/'raw/payloads'/(label+'-original.json');save(before,{'meta':meta,'payload':payload})
   transformed,changes=formatter.transform(payload);transformed['instruction']=transformed.get('instruction','')+' '+formatter.EXPLANATION
   self.buffers.set(label,len(json.dumps(transformed,ensure_ascii=False).encode()),'formatter_model_input_copy')
   audit={'version':formatter.VERSION,'formatter_sha256':sha(formatter_path),'original_payload_file':str(before),'original_payload_sha256':sha(before),'changed_original_passages':changes,'instruction_appended_exactly_once':True,'host_state_source_guard_unchanged':True}
   try:
    result,record=super().call(transformed,meta);record['input_representation_diagnostic']=audit;return result,record
   except ModelError as error:error.record['input_representation_diagnostic']=audit;raise
   finally:self.buffers.release(label)
 episode=next(x for x in read(folder/'frozen/source/benchmark/architecture/four_dp_poc_v2/fixtures/dp05.json')if x['id']=='dp05-e4');client=CompactClient(folder,limit=4,ledger_root=root);record=execute(client,episode,variant,rep,'DIAGNOSTIC','ORIGINAL_LINE_NUMBERS');verify_harness(folder)
 print(json.dumps({'run_id':record['run_id'],'calls':len(record['calls']),'goal_failed_events':sum(e.get('kind')=='GOAL_FAILED'for e in record['events']),'returncode':record['returncode']}),flush=True)

def run(root):
 prerequisites(root);folder=root/'diagnostic-dp05-numbered-lines';plan=read(folder/'plan.json');key=lambda r:(r['dp'],r['episode'],r['variant'],r['repetition']);done={key(json.loads(x))for x in(folder/'runs.jsonl').read_text().splitlines()if x}if(folder/'runs.jsonl').exists()else set()
 for i,j in enumerate(plan['jobs'],1):
  if key(j)in done:continue
  verify_harness(folder);command=[sys.executable,str(folder/'frozen/harness/line_number_diagnostic.py'),'worker','--root',str(root),'--variant',j['variant'],'--repetition',str(j['repetition'])];r=subprocess.run(command,capture_output=True,text=True)
  with(folder/'orchestration.log').open('a')as f:f.write(r.stdout+r.stderr)
  print(r.stdout.strip(),flush=True);assert r.returncode==0,'Infrastructure failure; preserve raw/log and inspect, no silent retry'
  ledger=read(root/'budget.json')['calls'];print(json.dumps({'phase':'DIAGNOSTIC_LINE_NUMBERS','completed':i,'planned':9,'global_calls':len(ledger),'confirmed_usd':sum(x['amount_usd']for x in ledger.values()if x['state']=='USAGE_RECORDED')}),flush=True)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','run','worker']);p.add_argument('--root',type=Path,required=True);p.add_argument('--variant');p.add_argument('--repetition',type=int);a=p.parse_args();root=a.root.resolve()
 if a.action=='worker':worker(root/'diagnostic-dp05-numbered-lines',a.variant,a.repetition)
 elif a.action=='prepare':prepare(root)
 else:run(root)
