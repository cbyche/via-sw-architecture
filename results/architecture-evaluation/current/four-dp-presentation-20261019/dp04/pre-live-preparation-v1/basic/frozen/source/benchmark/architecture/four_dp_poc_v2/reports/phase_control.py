"""Freeze and run parent-authorized serial phase plans; no credential handling here."""
import argparse,hashlib,json,os,re,shutil,subprocess,sys,time
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
BASE=REPO/'benchmark/architecture/four_dp_poc_v2'
CRATE=REPO/'prototypes/candidates/four_dp_poc_v2'
sys.path.insert(0,str(BASE))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(p):return json.loads(Path(p).read_text())
def save(p,v):Path(p).parent.mkdir(parents=True,exist_ok=True);Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
RESPONSIBILITIES={
 'main.rs':'Input admission and DP dispatch', 'context.rs':'Real callbacks, causal tracing, bounded model IPC and local source operations',
 'model_wire.rs':'Rust/Python model wire encoding and usage decoding','source_contract.rs':'Source access, input-version and frame-metadata normalization',
 'dp01.rs':'Evidence-supply lifecycle and source/request events','dp01_evidence.rs':'Current source access rights and evidence validity','dp01_producer.rs':'Published evidence production and provenance','dp01_request.rs':'Request adoption, original reads and bounded answer completion',
 'dp02.rs':'Interpretation-execution lifecycle admission','dp02/state.rs':'Captured/public conversation, request revisions and task association state','dp02/interpretation.rs':'Current-input semantic intent adoption and source revision validation','dp02/response.rs':'Detailed response contract, source claims and bounded repair','dp02/execution.rs':'A step-boundary / B request-worker scheduling, revision publication and task receipts',
 'dp04.rs':'Grounding lifecycle and goal/revision admission','dp04/observations.rs':'Actual source-image reads and raw event overlay','dp04/joint.rs':'A joint multimodal semantic grounding','dp04/producers.rs':'B screen/speech evidence production','dp04/binding.rs':'B producer-state binding and source-owned expansion','dp04/geometry.rs':'Shared declared geometric contract and source geometry validation','dp04/gate.rs':'Grounding proposal revision gate and final adoption',
 'dp05.rs':'Handoff lifecycle, raw-input adoption and authoritative contract','dp05_sources.rs':'Current accessible bounded source references','dp05_record.rs':'Persistent B handoff record production and invalidation','dp05_handoff.rs':'A construction and C direct source-manifest transfer','dp05_receiver.rs':'Common receiver select/complete, reads, source-backed validation and scoped shim',
}
def registry():
 rows=[]
 for p in sorted((CRATE/'src').rglob('*.rs')):
  relative=str(p.relative_to(CRATE/'src'));text=p.read_text()
  if relative not in RESPONSIBILITIES:raise RuntimeError('Unregistered module '+relative)
  rows.append({'id':relative[:-3].replace('/','::'),'path':str(p.relative_to(REPO)),'sha256':sha(p),'responsibility':RESPONSIBILITIES[relative],'declared_functions':re.findall(r'\b(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)',text),'operational':True})
 return {'version':'V2_OPERATIONAL_OWNERSHIP_BEFORE_BASIC','rule':'Actual Rust source module with operating responsibility; tests/probes excluded. New operational module plus wiring changes count if required by a feature. File count is scoped modifiability evidence, not general maintainability.','modules':rows}
def jobs(kind):
 if kind=='calibration-v3':return [{'dp':'04','episode':'dp04-free-region','variant':v,'repetition':1} for v in 'AB']
 if kind!='basic-v1':raise ValueError(kind)
 out=[]
 for dp in ['01','02','04','05']:
  variants=list('ABC' if dp in ('01','05') else 'AB')
  for rep in range(1,4):
   order=variants[(rep-1)%len(variants):]+variants[:(rep-1)%len(variants)]
   for e in read(BASE/'fixtures'/f'dp{dp}.json'):
    out.extend({'dp':dp,'episode':e['id'],'variant':v,'repetition':rep}for v in order)
 return out
def prepare(kind):
 from run import freeze
 phase='CALIBRATION_V3' if kind=='calibration-v3' else 'BASIC'
 folder=ROOT/kind;folder.mkdir(exist_ok=True)
 plan=jobs(kind)
 if kind=='basic-v1':
  from timing import population
  counts={d:{'qa01':0,'qa03':0}for d in ['01','02','04','05']}
  for j in plan:
   p=population(j['dp'],j['episode']);counts[j['dp']]['qa01']+=len(p['qa01_stimuli']);counts[j['dp']]['qa03']+=len(p['qa03_goals'])
  assert sum(x['qa01']for x in counts.values())==261,counts
  assert sum(x['qa03']for x in counts.values())==243,counts
  assert len(plan)==120
  save(folder/'population.json',{'expected':counts,'qa01_total':261,'qa03_total':243,'episode_trials':120})
  save(folder/'operational-module-registry.json',registry())
  shutil.copytree(ROOT/'qa04',folder/'qa04',ignore=shutil.ignore_patterns('scratch','target'),dirs_exist_ok=True) if False else None
  (folder/'qa04').mkdir(exist_ok=True);shutil.copy2(ROOT/'qa04/summary.json',folder/'qa04/summary.json')
 manifest=freeze(folder)
 external=manifest['evaluator_external_sources']
 candidates=[ROOT/'analysis/evaluate_bounded_content_equivalence.py']
 if kind=='basic-v1':candidates += [folder/'operational-module-registry.json',folder/'population.json',folder/'qa04/summary.json']
 for source in candidates:
  if not source.exists():raise RuntimeError('Missing frozen external '+str(source))
  dest=folder/'frozen/evaluator'/source.name;dest.parent.mkdir(exist_ok=True);shutil.copy2(source,dest)
  external[str(source.resolve())]={'sha256':sha(source),'snapshot':str(dest.resolve()),'candidate_input':False}
 manifest['evaluator_equivalence_scope']='Separate parallel DP01/05 bounded content diagnostic; literal results retained. DP02 continues required-revision/receipt scorer.'
 save(folder/'freeze.json',manifest)
 auth={'phase':phase,'authorization':'Root explicitly authorized two free AB calibration trials and conditional full 120 BASIC after minimal shared relation clarification; actual semantic model failures preserved. No further prompt tuning.','campaign_root':str(ROOT),'global_budget_usd':4,'freeze_sha256':sha(folder/'freeze.json'),'allowed_jobs':plan}
 save(folder/'MODEL-CALLS-AUTHORIZED.json',auth);save(folder/'plan.json',{'phase':phase,'jobs':plan,'serial_episode_execution':True,'fresh_python_each_trial':True,'internal_structural_concurrency_only':True})
 (folder/'budget.json').symlink_to(ROOT/'budget.json')
 print(json.dumps({'phase':phase,'path':str(folder),'freeze_sha256':sha(folder/'freeze.json'),'binary_sha256':manifest['binary']['sha256'],'jobs':len(plan)}),flush=True)
def run(kind):
 folder=ROOT/kind;plan=read(folder/'plan.json');started=time.monotonic()
 donekeys=set()
 if (folder/'runs.jsonl').exists():
  for line in (folder/'runs.jsonl').read_text().splitlines():
   r=json.loads(line);donekeys.add((r['dp'],r['episode'],r['variant'],r['repetition']))
 for index,j in enumerate(plan['jobs'],1):
  if (j['dp'],j['episode'],j['variant'],j['repetition'])in donekeys:continue
  command=[sys.executable,str(BASE/'run.py'),'--results',str(folder),'--dp',j['dp'],'--episode',j['episode'],'--variant',j['variant'],'--single-repetition',str(j['repetition']),'--phase',plan['phase'],'--skip-build','--single-trial-worker']
  with (folder/'orchestration.log').open('a')as f:
   result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);f.write(result.stdout);f.flush()
  print(result.stdout.strip(),flush=True)
  ledger=read(ROOT/'budget.json')
  print(json.dumps({'completed':index,'planned':len(plan['jobs']),'global_calls':len(ledger.get('calls',{})),'global_ledger_usd':sum(c['amount_usd']for c in ledger['calls'].values()if c['state']!='RESERVED'),'reserved_usd':sum(c['amount_usd']for c in ledger['calls'].values()if c['state']=='RESERVED'),'returncode':result.returncode,'orchestrator_elapsed_s':time.monotonic()-started}),flush=True)
  if result.returncode:raise RuntimeError('Infrastructure failure; preserve current evidence and resolve explicitly')
def status(kind):
 folder=ROOT/kind;ledger=read(ROOT/'budget.json')['calls'];lines=(folder/'runs.jsonl').read_text().splitlines()if(folder/'runs.jsonl').exists()else[]
 print(json.dumps({'completed':len(lines),'planned':len(read(folder/'plan.json')['jobs']),'global_calls':len(ledger),'usage_confirmed_usd':sum(c['amount_usd']for c in ledger.values()if c['state']=='USAGE_RECORDED'),'inflight_reserved_usd':sum(c['amount_usd']for c in ledger.values()if c['state']=='RESERVED'),'unknown_reservation_usd':sum(c['amount_usd']for c in ledger.values()if c['state']not in('RESERVED','USAGE_RECORDED')),'global_limit_usd':4}))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','run','status']);p.add_argument('kind',choices=['calibration-v3','basic-v1']);a=p.parse_args();{'prepare':prepare,'run':run,'status':status}[a.action](a.kind)
