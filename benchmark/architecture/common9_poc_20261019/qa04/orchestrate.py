"""Fresh42copy QA04 orchestration. Never imports old counts or executes provider calls."""
import argparse,difflib,hashlib,inspect,json,shutil,subprocess,sys,time
from pathlib import Path
from module_inventory import inventory,operational_tokens,PATHS,LIVE_PATHS
HERE=Path(__file__).resolve().parent;REPO=HERE.parents[3];RESULT=REPO/'results/architecture-evaluation/current/common9-poc-20261019/qa04'
CONFIGS={'AAAA':('A','A','A','A'),'BAAA':('B','A','A','A'),'CAAA':('C','A','A','A'),'ABAA':('A','B','A','A'),'AABA':('A','A','B','A'),'AAAB':('A','A','A','B'),'AAAC':('A','A','A','C')}
TASKS=['C01','C02','C03','C04','C05','C06'];CRATE_REL=Path('prototypes/candidates/common9_poc_20261019')
def read(p):return json.loads(Path(p).read_text())
def dump(p,x):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def baseline(freeze,expected):
 if sha(freeze)!=expected:raise RuntimeError('Baseline freeze hash mismatch')
 manifest=read(freeze);source=freeze.parent/'frozen/source'/CRATE_REL
 if not source.is_dir():raise RuntimeError('Frozen source snapshot absent; active source cannot substitute')
 entries=[entry for entry in manifest['sources']if entry['relativePath'].startswith(str(CRATE_REL)+'/')]
 if not entries:raise RuntimeError('Crate inventory absent from freeze')
 expected_files=set()
 for entry in entries:
  rel=Path(entry['relativePath']);p=freeze.parent/'frozen/source'/rel
  if p.resolve()!=Path(entry['frozen_path']).resolve():raise RuntimeError('Frozen relative-path mismatch: '+str(rel))
  if not p.is_file()or sha(p)!=entry['sha256']:raise RuntimeError('Frozen source mismatch: '+str(rel))
  expected_files.add(str(rel.relative_to(CRATE_REL)))
 if len(expected_files)!=len(entries):raise RuntimeError('Duplicate frozen crate source paths')
 actual_files={str(p.relative_to(source))for p in source.rglob('*')if p.is_file()}
 if actual_files!=expected_files:raise RuntimeError('Snapshot file inventory differs from frozen manifest')
 inventory(source)
 return source,manifest

def command(args,folder):
 started=time.time();p=subprocess.run(args,capture_output=True,text=True)
 with(folder/'commands.jsonl').open('a')as f:f.write(json.dumps({'args':args,'started_unix':started,'finished_unix':time.time(),'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})+'\n')
 if p.returncode:raise RuntimeError('Command failed, preserved stdout/stderr: '+str(args))
 return p

def counts(source,crate):
 changed=[];diff=[]
 for rel in sorted({p.relative_to(source)for p in(source/'src').rglob('*.rs')}|{p.relative_to(crate)for p in(crate/'src').rglob('*.rs')}):
  old=source/rel;new=crate/rel
  a=old.read_text()if old.exists()else'';b=new.read_text()if new.exists()else''
  if a==b:continue
  if operational_tokens(a)!=operational_tokens(b):changed.append(str(rel))
  diff.extend(difflib.unified_diff(a.splitlines(True),b.splitlines(True),fromfile='frozen/'+str(rel),tofile='patched/'+str(rel)))
 return changed,''.join(diff)

def module_review(source,crate,task,configuration):
 before=inventory(source);after=inventory(crate);changed,_=counts(source,crate)
 raw_changed=[]
 for rel in sorted({str(p.relative_to(source))for p in(source/'src').rglob('*.rs')}|{str(p.relative_to(crate))for p in(crate/'src').rglob('*.rs')}):
  a=source/rel;b=crate/rel
  if (a.read_text()if a.exists()else'')!=(b.read_text()if b.exists()else''):
   raw_changed.append({'file':rel,'operational_module_path':PATHS[rel],'counted':rel in changed,'reason':'Actual operational Rust tokens changed'if rel in changed else'Only test-module/comment/whitespace tokens changed; excluded'})
 return {'baseline_mapping':before,'candidate_mapping':after,'actual_diff_review':raw_changed,'counted_module_paths':sorted({PATHS[p]for p in changed}),'selected_configuration':configuration,'selected_live_path':LIVE_PATHS[task],'dormant_branch_only_changes':False,'mapping_limit':'File modules explicitly audited for this crate; no general file-count equals module-count claim'}

def prepare(args):
 source,manifest=baseline(args.freeze,args.digest)
 import patches
 import extensions
 targets=[(args.configuration,args.task)]if args.configuration and args.task else[(c,t)for c in CONFIGS for t in TASKS]
 for cfg,task in targets:
   task_root=RESULT/cfg/task;attempts=task_root/'attempts';attempts.mkdir(parents=True,exist_ok=True)
   folder=attempts/f'attempt-{len(list(attempts.glob("attempt-*")))+1:02d}';crate=folder/'candidate'
   dump(task_root/'latest-attempt.json',{'path':str(folder),'status':'PREPARING'})
   shutil.copytree(source,crate,ignore=shutil.ignore_patterns('target','__pycache__'))
   apply=getattr(patches,'apply_'+task.lower(),None)or getattr(extensions,'apply_'+task.lower())
   configuration=dict(zip(['evidence','execution','grounding','handoff'],CONFIGS[cfg]));configuration['id']=cfg
   try:apply(crate,configuration)if len(inspect.signature(apply).parameters)>1 else apply(crate)
   except Exception as error:
    changed,patch=counts(source,crate);(folder/'failed-partial.patch').write_text(patch)
    dump(folder/'failure.json',{'stage':'PATCH','type':type(error).__name__,'error':str(error),'modules_touched_before_failure':changed});dump(task_root/'latest-attempt.json',{'path':str(folder),'status':'PATCH_FAILED'});raise
   command(['cargo','fmt','--manifest-path',str(crate/'Cargo.toml')],folder)
   changed,patch=counts(source,crate);(folder/'actual.patch').write_text(patch)
   tools=folder/'tools';tools.mkdir();[shutil.copy2(p,tools/p.name)for p in HERE.glob('*.py')]
   dump(folder/'module-review.json',module_review(source,crate,task,configuration))
   configuration=dict(zip(['evidence','execution','grounding','handoff'],CONFIGS[cfg]));configuration['id']=cfg
   dump(task_root/'latest-attempt.json',{'path':str(folder),'status':'PATCHED_NOT_VERIFIED'})
   dump(folder/'configuration.json',configuration);dump(folder/'manifest.json',{'configuration':configuration,'task':task,'baseline_freeze':str(args.freeze.resolve()),'baseline_freeze_sha256':args.digest,'source_snapshot':str(source),'actual_changed_operational_modules':changed,'count':len(changed),'candidate_source_sha256':{str(p.relative_to(crate)):sha(p)for p in crate.rglob('*')if p.is_file()},'qa04_tool_sha256':{p.name:sha(p)for p in tools.glob('*.py')},'scope':'Independent whole frozen baseline; no provider calls; regression common9+finite task acceptance only'})
 print('Prepared '+str(len(targets))+' independent source changes; not acceptance PASS')

def verify_one(args,source,cfg,task):
 task_root=RESULT/cfg/task;folder=Path(read(task_root/'latest-attempt.json')['path']);crate=folder/'candidate'
 verifications=folder/'verifications';verifications.mkdir(exist_ok=True);verification=verifications/f'verify-{len(list(verifications.glob("verify-*")))+1:02d}';verification.mkdir()
 acceptance_folder=verification/'acceptance'
 pointer={'path':str(folder),'verification':str(verification),'acceptance':str(acceptance_folder),'status':'VERIFYING'}
 dump(task_root/'latest-attempt.json',pointer);dump(verification/'status.json',pointer)
 try:
  m=read(folder/'manifest.json')
  if m['baseline_freeze_sha256']!=args.digest:raise RuntimeError('Wrong independent baseline')
  for rel,digest in m['candidate_source_sha256'].items():
   if sha(crate/rel)!=digest:raise RuntimeError('Patched independent candidate changed; create a new preserved attempt: '+rel)
  command(['cargo','test','--offline','--manifest-path',str(crate/'Cargo.toml')],verification)
  command(['cargo','build','--offline','--manifest-path',str(crate/'Cargo.toml')],verification)
  if not args.driver:raise RuntimeError('No acceptance/regression driver; compiled source is not task acceptance')
  frozen_benchmark=args.freeze.parent/'frozen/source/benchmark/architecture/common9_poc_20261019'
  for entry in read(args.freeze)['sources']:
   if sha(entry['frozen_path'])!=entry['sha256']or sha(entry['path'])!=entry['sha256']:raise RuntimeError('Common frozen dependency changed: '+entry['relativePath'])
  driver=folder/'tools'/args.driver.name
  for name,digest in m['qa04_tool_sha256'].items():
   if sha(folder/'tools'/name)!=digest:raise RuntimeError('QA04 attempt tool snapshot changed: '+name)
  command([sys.executable,str(driver),'--benchmark',str(frozen_benchmark),'--crate',str(crate),'--configuration',str(folder/'configuration.json'),'--task',task,'--output',str(acceptance_folder)],verification)
  acceptance=read(acceptance_folder/'summary.json')
  if acceptance.get('provider_calls')!=0 or acceptance.get('task_acceptance')!='PASS'or acceptance.get('common9_regression')!='PASS':raise RuntimeError('Acceptance incomplete or unauthorized provider use')
  if acceptance.get('scheduled_common_counts')!={'responses':43,'goals':23,'bindings':15}:raise RuntimeError('Regression population changed')
  changed,patch=counts(source,crate);(verification/'final.patch').write_text(patch)
  review=module_review(source,crate,task,m['configuration']);dump(verification/'module-review.json',review)
  result={'configuration':cfg,'task':task,'status':'PASS','count':len(changed),'modules':changed,'operational_module_paths':review['counted_module_paths'],'module_review':str(verification/'module-review.json'),'selected_live_path':review['selected_live_path'],'dormant_branch_only_changes':False,'baseline_freeze_sha256':args.digest,'acceptance':str(acceptance_folder/'summary.json'),'provider_calls':0,'test_scope':'common9 plus task-specific deterministic code acceptance; not semantic/model timing evidence'}
  dump(verification/'summary.json',result);pointer['status']='PASS';dump(verification/'status.json',pointer);dump(task_root/'latest-attempt.json',pointer)
 except Exception as error:
  pointer.update(status='VERIFY_FAILED',error=type(error).__name__+': '+str(error));dump(verification/'status.json',pointer);dump(task_root/'latest-attempt.json',pointer);raise

def verify(args):
 source,_=baseline(args.freeze,args.digest)
 targets=[(args.configuration,args.task)]if args.configuration and args.task else[(c,t)for c in CONFIGS for t in TASKS]
 for cfg,task in targets:verify_one(args,source,cfg,task)
 print('Verified '+str(len(targets))+' independent changes')

def summarize(args):
 rows=[]
 for cfg in CONFIGS:
  for task in TASKS:
   latest=RESULT/cfg/task/'latest-attempt.json';attempt=read(latest)if latest.exists()else{};verification=Path(attempt.get('verification','/unavailable'));p=verification/'summary.json'
   valid=attempt.get('status')=='PASS'and p.exists()and read(verification/'status.json').get('status')=='PASS'
   rows.append(read(p)if valid else{'configuration':cfg,'task':task,'status':'NOT_VERIFIED','count':None,'latest_attempt':attempt})
 means={cfg:sum(r['count']for r in rows if r['configuration']==cfg)/6 if all(r['status']=='PASS'for r in rows if r['configuration']==cfg)else None for cfg in CONFIGS}
 dump(RESULT/'summary.json',{'rows':rows,'mean_changed_operational_modules':means,'denominator':6,'missing_tasks_retained':True,'scope':'Fresh independent42changes only; no old counts pooled. Shared operational modules counted.'})
 print(json.dumps(means))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','verify','summarize']);p.add_argument('--freeze',type=Path);p.add_argument('--digest');p.add_argument('--configuration',choices=CONFIGS);p.add_argument('--task',choices=TASKS);p.add_argument('--driver',type=Path);a=p.parse_args()
 if a.action=='prepare':prepare(a)
 elif a.action=='verify':verify(a)
 else:summarize(a)
