"""Authorized post-BASIC27 trials. Separate immutable binaries/plans, one campaign budget."""
import argparse,difflib,hashlib,json,shutil,subprocess,sys,time
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
BASE=REPO/'benchmark/architecture/four_dp_poc_v2';BASIC=ROOT/'basic-v1';CRATE_REL=Path('prototypes/candidates/four_dp_poc_v2')
sys.path.insert(0,str(BASE))
from run import verify_freeze,OfflineClient,execute

def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,x):Path(p).parent.mkdir(parents=True,exist_ok=True);Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def basic_done():
 rs=[json.loads(x)for x in(BASIC/'runs.jsonl').read_text().splitlines()if x]
 expected=read(BASIC/'plan.json')['jobs'];key=lambda x:(x['dp'],x['episode'],x['variant'],x['repetition'])
 if len(rs)!=120 or {key(x)for x in rs}!={key(x)for x in expected}:raise RuntimeError('All exact120 BASIC must finish before tactic preparation/compile/run')
 verify_freeze(BASIC)

def jobs(name):
 if name=='dp01-A2':return [{'dp':'01','episode':'dp01-e3','variant':'A','repetition':rep}for rep in range(1,4)]
 variant=name.split('-')[1];episodes=read(BASIC/'frozen/source/benchmark/architecture/four_dp_poc_v2/fixtures/dp04.json')
 return [{'dp':'04','episode':e['id'],'variant':variant,'repetition':rep}for rep in range(1,4)for e in episodes]

def prepare(name):
 basic_done();folder=ROOT/('tactic-'+name)
 if folder.exists():raise RuntimeError('Do not replace an existing tactic phase')
 folder.mkdir();manifest=read(BASIC/'freeze.json');shutil.copytree(BASIC/'frozen',folder/'frozen')
 manifest['source_snapshot']=str((folder/'frozen/source').resolve());manifest['binary']['path']=str((folder/'frozen/bin/four_dp_poc').resolve())
 for pin in manifest['evaluator_external_sources'].values():pin['snapshot']=str(folder/Path(pin['snapshot']).relative_to(BASIC))
 tactic=name.split('-')[1];phase='DIAGNOSTIC'if tactic=='B4'else'TACTIC'
 manifest['baseline_freeze_sha256']=sha(BASIC/'freeze.json');manifest['tactic']=tactic
 for source in [Path(__file__).resolve()]:
  relative=source.relative_to(REPO);target=folder/'frozen/source'/relative;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target);manifest['source'][str(relative)]=sha(source)
 manifest['tactic_scope']='Same frozen four DP04 cases×3; actual A4/B4 variant path'if name!='dp01-A2'else'Same frozen dp01-e3×3; exact BASIC crate with only approved dp01.rs A2 fragment invalidation patch'
 if name=='dp01-A2':
  candidate=folder/'effective-candidate-source';shutil.copytree(BASIC/'frozen/source'/CRATE_REL,candidate)
  patch=ROOT/'tactics-preparation/dp01-A2/source/dp01.rs';assert sha(patch)=='b37e2427e1dd9e89e4f6e29cf35d704c9946b4baaf72340ec3de8098612065bb'
  old=(candidate/'src/dp01.rs').read_text();shutil.copy2(patch,candidate/'src/dp01.rs')
  (folder/'candidate.patch').write_text(''.join(difflib.unified_diff(old.splitlines(True),(candidate/'src/dp01.rs').read_text().splitlines(True),fromfile='BASIC/src/dp01.rs',tofile='A2/src/dp01.rs')))
  for p in candidate.rglob('*'):
   if p.is_file()and p.relative_to(candidate)!=Path('src/dp01.rs'):assert sha(p)==sha(BASIC/'frozen/source'/CRATE_REL/p.relative_to(candidate))
  logs=[]
  for command in [['cargo','build','--offline','--manifest-path',str(candidate/'Cargo.toml')],['cargo','test','--offline','--manifest-path',str(candidate/'Cargo.toml'),'dp01']]:
   c=subprocess.run(command,capture_output=True,text=True);logs.append({'command':command,'returncode':c.returncode,'stdout':c.stdout,'stderr':c.stderr})
   save(folder/'build-and-guard-checks.json',logs)
   if c.returncode:raise RuntimeError('A2 post-BASIC build/check failed; preserved logs')
  shutil.copy2(candidate/'target/debug/four_dp_poc',folder/'frozen/bin/four_dp_poc');manifest['binary']['sha256']=sha(folder/'frozen/bin/four_dp_poc')
  manifest['effective_candidate_source']={'snapshot':str(candidate.resolve()),'sha256':{str(p.relative_to(candidate)):sha(p)for p in candidate.rglob('*')if p.is_file()and 'target'not in p.relative_to(candidate).parts},'only_baseline_source_change':['src/dp01.rs'],'baseline_source_snapshot':str((BASIC/'frozen/source'/CRATE_REL).resolve()),'note':'manifest.source pins unchanged baseline harness/fixture/source inventory; this explicit effective source map pins the actual separately compiled A2 crate, and only its dp01.rs differs.'}
  # Deterministic actual executable flow, no provider calls and not numerical evidence.
  save(folder/'freeze.json',manifest)
  offline=folder/'offline-acceptance';offline.mkdir();save(offline/'freeze.json',manifest)
  episode=next(e for e in read(BASIC/'frozen/source/benchmark/architecture/four_dp_poc_v2/fixtures/dp01.json')if e['id']=='dp01-e3')
  for rep in [1,2,3]:
   rec=execute(OfflineClient(offline),episode,'A',rep,'OFFLINE',tactic='A2')
   assert rec['returncode']==0
   if rep==1:
    assert any(e.get('kind')=='WORKING_VIEW_FRAGMENT_INVALIDATION'for e in rec['events'])
    assert len([e for e in rec['events']if e.get('kind')=='GOAL_COMPLETE'])==2
  manifest['offline_acceptance_source']=str(offline.resolve())
 save(folder/'freeze.json',manifest)
 plan=jobs(name);auth={'phase':phase,'authorization':'Root explicitly approved after BASIC120: DP04 A4/B4 each same4cases×3; DP01 A2 sameE3×3. Preserve every failure and global $4 ledger.','campaign_root':str(ROOT),'global_budget_usd':4,'freeze_sha256':sha(folder/'freeze.json'),'allowed_jobs':plan,'allowed_tactic':tactic}
 save(folder/'MODEL-CALLS-AUTHORIZED.json',auth);save(folder/'plan.json',{'phase':phase,'tactic':tactic,'jobs':plan,'serial_episode_execution':True,'fresh_python_each_trial':True,'baseline':str(BASIC),'comparison_population':'Same episode only, matching BASIC3 repetitions. BASIC outcomes unchanged.'})
 (folder/'budget.json').symlink_to(ROOT/'budget.json')
 print(json.dumps({'prepared':name,'freeze_sha256':sha(folder/'freeze.json'),'binary_sha256':manifest['binary']['sha256'],'jobs':len(plan)}),flush=True)

def verify_effective(folder):
 m=read(folder/'freeze.json');extra=m.get('effective_candidate_source')
 if extra:
  for relative,wanted in extra['sha256'].items():assert sha(Path(extra['snapshot'])/relative)==wanted,'Effective source changed '+relative
 assert sha(m['binary']['path'])==m['binary']['sha256']

def run(name):
 basic_done();folder=ROOT/('tactic-'+name);plan=read(folder/'plan.json');done=set();key=lambda r:(r['dp'],r['episode'],r['variant'],r['repetition'])
 if(folder/'runs.jsonl').exists():done={key(json.loads(x))for x in(folder/'runs.jsonl').read_text().splitlines()if x}
 for index,j in enumerate(plan['jobs'],1):
  if key(j)in done:continue
  verify_freeze(folder);verify_effective(folder)
  cmd=[sys.executable,str(BASE/'run.py'),'--results',str(folder),'--dp',j['dp'],'--episode',j['episode'],'--variant',j['variant'],'--single-repetition',str(j['repetition']),'--phase',plan['phase'],'--tactic',plan['tactic'],'--skip-build','--single-trial-worker']
  r=subprocess.run(cmd,capture_output=True,text=True)
  with(folder/'orchestration.log').open('a')as f:f.write(r.stdout+r.stderr)
  print(r.stdout.strip(),flush=True);verify_effective(folder)
  ledger=read(ROOT/'budget.json')['calls'];print(json.dumps({'phase_folder':name,'completed':index,'planned':len(plan['jobs']),'global_calls':len(ledger),'confirmed_usd':sum(v['amount_usd']for v in ledger.values()if v['state']=='USAGE_RECORDED'),'reserved_usd':sum(v['amount_usd']for v in ledger.values()if v['state']=='RESERVED')}),flush=True)
  if r.returncode:raise RuntimeError('Infrastructure failure, not silent rerun; inspect preserved log')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','run']);p.add_argument('name',choices=['dp04-A4','dp04-B4','dp01-A2']);a=p.parse_args();{'prepare':prepare,'run':run}[a.action](a.name)
