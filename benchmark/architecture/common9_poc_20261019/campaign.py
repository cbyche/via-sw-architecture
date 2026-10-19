"""Reviewable freeze and fresh-process campaign driver; never self-authorizes APIs."""
import argparse,contextlib,fcntl,hashlib,json,os,shutil,subprocess,sys,time
from pathlib import Path
H=Path(__file__).resolve().parent;REPO=H.parents[2];CRATE=REPO/'prototypes/candidates/common9_poc_20261019';RESULT=REPO/'results/architecture-evaluation/current/common9-poc-20261019'
CONFIGS=['AAAA','BAAA','CAAA','ABAA','AABA','AAAB','AAAC'];EPISODES=['S02','S08','S11','S12','S15','S18','S19','S20','S26'];CONTRACT=REPO/'docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md'
def sha(p):
 h=hashlib.sha256()
 with Path(p).open('rb')as f:
  for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
 return h.hexdigest()
def dump(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def controller_event(root,event):
 row={"controller_pid":os.getpid(),"at_ns":time.monotonic_ns(),**event}
 fd=os.open(root/'controller-lock-events.jsonl',os.O_WRONLY|os.O_APPEND|os.O_CREAT,0o600)
 try:os.write(fd,(json.dumps(row,ensure_ascii=False)+'\n').encode());os.fsync(fd)
 finally:os.close(fd)

@contextlib.contextmanager
def controller_lock(root):
 # Resolve aliases before selecting the lock. Never unlink the lock file: doing so
 # could let a second inode acquire a parallel lock while the first is held.
 root=Path(root).resolve();root.mkdir(parents=True,exist_ok=True)
 with (root/'controller.lock').open('a+')as handle:
  try:fcntl.flock(handle,fcntl.LOCK_EX|fcntl.LOCK_NB)
  except BlockingIOError as error:
   handle.seek(0);owner=handle.read()
   controller_event(root,{'state':'CONTROLLER_LOCK_REJECTED','observed_owner':owner,'reason':'Another controller holds this campaign root; no validation or dispatch performed'})
   raise RuntimeError('Campaign controller already active for root: '+str(root))from error
  metadata={'controller_pid':os.getpid(),'acquired_ns':time.monotonic_ns(),'campaign_root':str(root)}
  handle.seek(0);handle.truncate();handle.write(json.dumps(metadata)+'\n');handle.flush();os.fsync(handle.fileno())
  controller_event(root,{'state':'CONTROLLER_LOCK_ACQUIRED','owner':metadata})
  outcome='RETURNED'
  try:yield
  except BaseException as error:
   outcome=type(error).__name__+': '+str(error)
   raise
  finally:
   controller_event(root,{'state':'CONTROLLER_LOCK_RELEASED','outcome':outcome})
   fcntl.flock(handle,fcntl.LOCK_UN)

def seed_budget():
 path=RESULT/'budget.json';path.parent.mkdir(parents=True,exist_ok=True)
 with path.open('a+')as f:
  fcntl.flock(f,fcntl.LOCK_EX);f.seek(0);text=f.read();data=json.loads(text)if text else{'limit_usd':4,'calls':{}}
  prior={'amount_usd':.1016136,'state':'PRIOR_CAMPAIGN_USAGE_RECORDED','provenance':'four-dp-prediction-validation-20261019/dp02:48of72;120calls;knowncost;unknown0'}
  old=data['calls'].get('prior-dp02-48of72')
  if old and old!=prior:raise RuntimeError('Previous campaign seed differs')
  data['calls']['prior-dp02-48of72']=prior;f.seek(0);f.truncate();json.dump(data,f,indent=2);f.flush();os.fsync(f.fileno())
 return data

def source_files():
 files=[]
 for base in[H,CRATE]:
  for p in base.rglob('*'):
   if not p.is_file():continue
   rel=p.relative_to(base)
   if any(x in ['target','__pycache__','qa04','qa04_protocol_v2','qa04_method_v3']for x in rel.parts):continue
   files.append(p)
 files += [RESULT/'parser-feasibility/dependency-inventory.json',RESULT/'parser-feasibility/packages.lock.txt',RESULT/'parser-feasibility/upstream-adapter.patch',CONTRACT,REPO/'docs/architecture/12-decisions/decision-packages/qa-candidate-package/common-scenarios.json']
 return sorted(set(files))
def jobs():
 # Cyclic order rotates configuration position by repetition and episode.
 out=[]
 for rep in range(1,6):
  for i,ep in enumerate(EPISODES):
   shift=(rep-1+i)%len(CONFIGS)
   for config in CONFIGS[shift:]+CONFIGS[:shift]:out.append({'configuration':config,'episode':ep,'rep':rep})
 return out

def external_dependencies():
 inventory=json.loads((RESULT/'parser-feasibility/dependency-inventory.json').read_text());items=[]
 for w in inventory['weights_loaded']:
  p=Path(w['path']);assert sha(p)==w['sha256'];stat=p.stat();items.append({'path':str(p),'sha256':w['sha256'],'bytes':stat.st_size,'mtime_ns':stat.st_mtime_ns,'large_weight':True})
 for key in ['external_runtime_source_sha256','remote_code_loaded_sha256','checkpoint_config_sha256']:
  for name,digest in inventory[key].items():
   p=Path(name);assert sha(p)==digest;stat=p.stat();items.append({'path':name,'sha256':digest,'bytes':stat.st_size,'mtime_ns':stat.st_mtime_ns,'large_weight':False})
 return items

def prepare(root,purpose='FORMAL_REVIEW_CANDIDATE',probe=False):
 if (root/'freeze.json').exists():raise RuntimeError('Freeze exists; preserve it and choose a fresh directory')
 root.mkdir(parents=True,exist_ok=True);sources=[]
 for p in source_files():
  rel=p.relative_to(REPO);dest=root/'frozen/source'/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,dest)
  sources.append({'path':str(p),'relativePath':str(rel),'sha256':sha(p),'frozen_path':str(dest.resolve())})
 binary=CRATE/'target/debug/common9_poc';dest=root/'frozen/bin/common9_poc';dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(binary,dest)
 manifest={'version':'COMMON9-FREEZE-v1','purpose':purpose,'created_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'contract':{'path':str(CONTRACT),'sha256':sha(CONTRACT)},'sources':sources,'external_dependencies':external_dependencies(),'binary':{'path':str(dest.resolve()),'sha256':sha(dest)},'jobs':jobs(),'common_k':2,'per_episode_max_cloud_calls':100,'repetitions':5,'per_repetition':{'responses':43,'goals':23,'bindings':15},'per_config':{'episode_runs':45,'responses':215,'goals':115,'bindings':75},'unique_configurations':CONFIGS,'total_episode_runs':315,'shared_control':'AAAA reused across4DPcomparisons; correlated, not independent replicas','ledger_path':str(RESULT/'budget.json'),'authorization':'Separate parent freeze review and MODEL-CALLS-AUTHORIZED.json required; this manifest grants nothing'}
 if probe:
  assert purpose=='DEVELOPMENT_COMPATIBILITY_PROBE_REVIEW_CANDIDATE'
  plan=[{'configuration':'AAAA','episode':ep,'rep':1}for ep in ['S02','S18']]
  oracle=json.loads((H/'oracles.json').read_text());counts={key:sum(len(oracle[j['episode']][key])for j in plan)for key in ['responses','goals','bindings']}
  manifest.update({'jobs':plan,'repetitions':1,'per_repetition':counts,'per_config':{'episode_runs':2,**counts},'unique_configurations':['AAAA'],'total_episode_runs':2,'shared_control':'Development protocol check only; no DP ranking or primary pooling','scope_limit_usd':1.0,'total_max_cloud_calls':200,'assessment_role':'DEVELOPMENT_COMPATIBILITY_ONLY_NOT_PRIMARY','probe_plan':{'path':str(RESULT/'preflight/compatibility-probe-v3-PLAN.md'),'sha256':sha(RESULT/'preflight/compatibility-probe-v3-PLAN.md')}})
 dump(root/'freeze.json',manifest);dump(root/'planned-jobs.json',manifest['jobs']);seed_budget();return manifest

def validate(root,verify_large=False):
 freeze=root/'freeze.json';manifest=json.loads(freeze.read_text())
 for s in manifest['sources']:
  if sha(s['path'])!=s['sha256']or sha(s['frozen_path'])!=s['sha256']:raise RuntimeError('Frozen source mismatch: '+s['relativePath'])
 for item in manifest.get('external_dependencies',[]):
  path=Path(item['path']);stat=path.stat()
  if stat.st_size!=item['bytes']or stat.st_mtime_ns!=item['mtime_ns']:raise RuntimeError('External dependency stat changed: '+str(path))
  if (verify_large or not item['large_weight'])and sha(path)!=item['sha256']:raise RuntimeError('External dependency hash changed: '+str(path))
 if sha(manifest['binary']['path'])!=manifest['binary']['sha256']:raise RuntimeError('Frozen binary mismatch')
 return manifest

def run(root):
 root=Path(root).resolve()
 with controller_lock(root):_run_locked(root)

def _run_locked(root):
 manifest=validate(root,verify_large=True);assert manifest['purpose']in ['FORMAL_REVIEW_CANDIDATE','DEVELOPMENT_COMPATIBILITY_PROBE_REVIEW_CANDIDATE'];auth=json.loads((root/'MODEL-CALLS-AUTHORIZED.json').read_text());assert auth['freeze_sha256']==sha(root/'freeze.json')
 seed_budget();done=set();started_jobs=set();progress=root/'campaign-progress.jsonl'
 if progress.exists():
  for line in progress.read_text().splitlines():
   r=json.loads(line)
   key=(r['configuration'],r['episode'],r['rep'])
   if r.get('state')=='STARTED':started_jobs.add(key)
   if r.get('state')=='TERMINATED':done.add(key)
 orphans=started_jobs-done
 if orphans:
  dump(root/'ORPHANED-STARTED-JOBS.json',{'reason':'Crash/interruption left STARTED without TERMINATED. No automatic rerun; retain raw/cost and obtain explicit recovery decision.','jobs':[{'configuration':c,'episode':e,'rep':r}for c,e,r in sorted(orphans)]})
  raise RuntimeError('Orphaned formal identity preserved; explicit recovery decision required')
 for job in manifest['jobs']:
  key=(job['configuration'],job['episode'],job['rep'])
  if key in done:continue
  if (root/'METHOD-REVIEW-HOLD').exists():break
  validate(root)
  if job not in auth['allowed_jobs']:raise RuntimeError('Job outside isolated window authorization')
  cmd=[sys.executable,str(H/'harness.py'),'--root',str(root),'--configuration',job['configuration'],'--episode',job['episode'],'--rep',str(job['rep'])]
  with progress.open('a')as f:f.write(json.dumps({**job,'state':'STARTED','command':cmd,'at_ns':time.monotonic_ns()})+'\n')
  # Each episode has a fresh Python bridge and Rust process; outer driver excluded from RSS.
  p=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  with progress.open('a')as f:f.write(json.dumps({**job,'state':'DISPATCHED','harness_pid':p.pid,'at_ns':time.monotonic_ns()})+'\n')
  stdout,stderr=p.communicate()
  record={**job,'state':'TERMINATED','harness_pid':p.pid,'returncode':p.returncode,'stdout':stdout,'stderr':stderr,'at_ns':time.monotonic_ns()}
  with progress.open('a')as f:f.write(json.dumps(record,ensure_ascii=False)+'\n')
  print(json.dumps(record,ensure_ascii=False),flush=True)
  if p.returncode:raise RuntimeError('Host failure preserved; campaign stops for review without retry')
  summary=json.loads(stdout.strip().splitlines()[-1])if stdout.strip()else{}
  if summary.get('technical_failure'):
   dump(root/'TECHNICAL-STOP.json',{'job':job,'reason':'Technical episode failure preserved; planned population unchanged, no automatic retry. Independent review and explicit recovery required.','summary':summary});break
  if summary.get('budget_stop'):
   dump(root/'BUDGET-STOP.json',{'job':job,'reason':'Administrative budget reservation failed; no further jobs dispatched. Allplanned315identities preserved.','completed_job_has_partial_failure_evidence':True});break
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['seed','snapshot','prepare','prepare-probe','validate','run']);p.add_argument('--root',type=Path,default=RESULT/'formal-v1');a=p.parse_args()
 if a.action=='seed':print(json.dumps(seed_budget()))
 elif a.action in ['prepare','snapshot']:print(json.dumps({'freeze':str(a.root/'freeze.json'),'sources':len(prepare(a.root,'PREFORMAL_QA04_SOURCE_SNAPSHOT'if a.action=='snapshot'else'FORMAL_REVIEW_CANDIDATE')['sources']),'authorization':False}))
 elif a.action=='prepare-probe':print(json.dumps({'freeze':str(a.root/'freeze.json'),'sources':len(prepare(a.root,'DEVELOPMENT_COMPATIBILITY_PROBE_REVIEW_CANDIDATE',True)['sources']),'authorization':False,'scope_limit_usd':1.0,'planned':2}))
 elif a.action=='validate':print(json.dumps({'valid':bool(validate(a.root))}))
 else:run(a.root)
