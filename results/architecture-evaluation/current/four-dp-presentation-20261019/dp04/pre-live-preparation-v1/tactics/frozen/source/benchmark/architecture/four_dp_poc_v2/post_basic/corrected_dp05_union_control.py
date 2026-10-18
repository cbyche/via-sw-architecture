"""Authorized fair DP05 ABC36 rerun for actual VIA-read retention and gapless supplied-span union; preserve original BASIC."""
import argparse,hashlib,json,shutil,subprocess,sys
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018';BASE=REPO/'benchmark/architecture/four_dp_poc_v2';CRATE_REL=Path('prototypes/candidates/four_dp_poc_v2');BASIC=ROOT/'basic-v1';FOLDER=ROOT/'basic-dp05-reuse-union-corrected-v2'
sys.path.insert(0,str(BASE))
from run import verify_freeze,execute,OfflineClient

def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,x):Path(p).parent.mkdir(parents=True,exist_ok=True);Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def prerequisites():
 for name,n in {'basic-v1':120,'tactic-dp04-A4':12,'tactic-dp04-B4':12,'tactic-dp01-A2':3}.items():
  p=ROOT/name;rs=[json.loads(x)for x in(p/'runs.jsonl').read_text().splitlines()if x];key=lambda r:(r['dp'],r['episode'],r['variant'],r['repetition']);assert len(rs)==n and {key(x)for x in rs}=={key(x)for x in read(p/'plan.json')['jobs']},name+' not complete'
 assert (ROOT/'basic-dp05-reuse-corrected-v1/PHASE-STOPPED.json').exists(),'Preserve the boundary-stopped intermediate phase before new comparison'
 verify_freeze(BASIC)

def prepare(patch_root,qa04_summary):
 prerequisites();assert not FOLDER.exists(),'Use a new immutable phase, never replace';FOLDER.mkdir();m=read(BASIC/'freeze.json');shutil.copytree(BASIC/'frozen',FOLDER/'frozen');m['source_snapshot']=str((FOLDER/'frozen/source').resolve());m['binary']['path']=str((FOLDER/'frozen/bin/four_dp_poc').resolve())
 for pin in m['evaluator_external_sources'].values():pin['snapshot']=str(FOLDER/Path(pin['snapshot']).relative_to(BASIC))
 candidate=FOLDER/'effective-candidate-source';shutil.copytree(BASIC/'frozen/source'/CRATE_REL,candidate);changes=[]
 allowed={'src/dp05.rs','src/dp05_handoff.rs','src/dp05_sources.rs'}
 for p in (BASIC/'frozen/source'/CRATE_REL).rglob('*'):
  if not p.is_file():continue
  relative=p.relative_to(BASIC/'frozen/source'/CRATE_REL);proposed=patch_root/relative;assert proposed.exists(),'Missing source '+str(relative)
  if sha(p)!=sha(proposed):
   assert str(relative)in allowed,'Unapproved source difference '+str(relative);changes.append(str(relative));shutil.copy2(proposed,candidate/relative)
 assert changes and set(changes)<=allowed
 logs=[]
 for command in [['cargo','build','--offline','--manifest-path',str(candidate/'Cargo.toml')],['cargo','test','--offline','--manifest-path',str(candidate/'Cargo.toml'),'dp05']]:
  c=subprocess.run(command,capture_output=True,text=True);logs.append({'command':command,'returncode':c.returncode,'stdout':c.stdout,'stderr':c.stderr});save(FOLDER/'build-and-guard-checks.json',logs);assert c.returncode==0,'Corrected DP05 build/check failed'
 shutil.copy2(candidate/'target/debug/four_dp_poc',FOLDER/'frozen/bin/four_dp_poc');m['binary']['sha256']=sha(FOLDER/'frozen/bin/four_dp_poc')
 m['baseline_freeze_sha256']=sha(BASIC/'freeze.json');m['correction_scope']='ABC identical current-authorized actual VIA_COMPOSER read ledger retention, including subsequent construction/receiver failure; common prefix receives a temporary canonical raw view of actual common originals plus B current-valid original work-record supporting passages. No derived summaries/statements or receiver-read promotion, no permanent record-support cache. Aggregate source-spans accept gapless current same-source/revision/unit unions; original fact quote checks unchanged. Same fixtures/oracles/endpoints/model/caps/recovery.'
 m['effective_candidate_source']={'snapshot':str(candidate.resolve()),'sha256':{str(p.relative_to(candidate)):sha(p)for p in candidate.rglob('*')if p.is_file()and'target'not in p.relative_to(candidate).parts},'only_baseline_source_change':changes,'baseline_source_snapshot':str((BASIC/'frozen/source'/CRATE_REL).resolve()),'note':'Baseline harness/evaluator/source map unchanged; this explicit source map pins actual separately compiled correction.'}
 # Actual functional-change acceptance is required, never inferred from old module counts.
 qa04=read(qa04_summary);opts=qa04.get('options',[]);assert {x['variant']for x in opts if str(x['dp'])=='05'}==set('ABC')
 for x in opts:
  if str(x['dp'])=='05':assert {t['id']for t in x['tasks']}=={'C01','C03','C04','C06'}and all(t['status']=='PASS'for t in x['tasks'])
 (FOLDER/'qa04').mkdir();shutil.copy2(qa04_summary,FOLDER/'qa04/summary.json');target=FOLDER/'frozen/evaluator/corrected-dp05-qa04-summary.json';shutil.copy2(qa04_summary,target);m['evaluator_external_sources'][str(qa04_summary.resolve())]={'sha256':sha(qa04_summary),'snapshot':str(target.resolve()),'candidate_input':False}
 for source in [Path(__file__).resolve()]:
  relative=source.relative_to(REPO);target=FOLDER/'frozen/source'/relative;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target);m['source'][str(relative)]=sha(source)
 save(FOLDER/'freeze.json',m);offline=FOLDER/'offline-acceptance';offline.mkdir();save(offline/'freeze.json',m)
 episodes=read(BASIC/'frozen/source/benchmark/architecture/four_dp_poc_v2/fixtures/dp05.json')
 for ep in episodes:
  for v in 'ABC':
   rec=execute(OfflineClient(offline),ep,v,1,'OFFLINE');assert rec['returncode']==0
 jobs=[]
 for rep in range(1,4):
  variants='ABC'[(rep-1)%3:]+'ABC'[:(rep-1)%3]
  for ep in episodes:jobs.extend({'dp':'05','episode':ep['id'],'variant':v,'repetition':rep}for v in variants)
 auth={'phase':'BASIC_CORRECTED','authorization':'Root approved fair DP05 ABC fourcases×3 rerun for intrinsic actual-read retention plus supplied adjacent/overlapping aggregate span union. Previous boundary-stopped phase is diagnostic history, all original costs/failures retained; same global$4.','campaign_root':str(ROOT),'global_budget_usd':4,'freeze_sha256':sha(FOLDER/'freeze.json'),'allowed_jobs':jobs}
 save(FOLDER/'MODEL-CALLS-AUTHORIZED.json',auth);save(FOLDER/'plan.json',{'phase':'BASIC_CORRECTED','jobs':jobs,'fresh_python_each_trial':True,'serial_episode_execution':True,'effective_presentation':'Use these36 for DP05 with original01/02/04 84; originalDP05 remains history.'});(FOLDER/'budget.json').symlink_to(ROOT/'budget.json');print(json.dumps({'phase':str(FOLDER),'freeze_sha256':sha(FOLDER/'freeze.json'),'binary_sha256':m['binary']['sha256'],'changed_source':changes,'jobs':len(jobs)}),flush=True)

def run():
 prerequisites();m=read(FOLDER/'freeze.json');pin=m['effective_candidate_source'];key=lambda r:(r['dp'],r['episode'],r['variant'],r['repetition']);done={key(json.loads(x))for x in(FOLDER/'runs.jsonl').read_text().splitlines()if x}if(FOLDER/'runs.jsonl').exists()else set()
 for i,j in enumerate(read(FOLDER/'plan.json')['jobs'],1):
  if key(j)in done:continue
  if (FOLDER/'STOP-AFTER-EPISODE').exists():
   print(json.dumps({'boundary_stop_requested':True,'completed':len(done),'next_unexecuted_job':j}),flush=True);break
  verify_freeze(FOLDER)
  for relative,wanted in pin['sha256'].items():assert sha(Path(pin['snapshot'])/relative)==wanted
  command=[sys.executable,str(BASE/'run.py'),'--results',str(FOLDER),'--dp','05','--episode',j['episode'],'--variant',j['variant'],'--single-repetition',str(j['repetition']),'--phase','BASIC_CORRECTED','--skip-build','--single-trial-worker'];r=subprocess.run(command,capture_output=True,text=True)
  with(FOLDER/'orchestration.log').open('a')as f:f.write(r.stdout+r.stderr)
  print(r.stdout.strip(),flush=True);done.add(key(j));assert r.returncode==0,'Infrastructure failure; preserve evidence, no quiet rerun';ledger=read(ROOT/'budget.json')['calls'];print(json.dumps({'corrected_dp05_completed':i,'planned':36,'global_calls':len(ledger),'confirmed_usd':sum(x['amount_usd']for x in ledger.values()if x['state']=='USAGE_RECORDED')}),flush=True)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','run']);p.add_argument('--patch-root',type=Path);p.add_argument('--qa04-summary',type=Path);a=p.parse_args();prepare(a.patch_root.resolve(),a.qa04_summary.resolve())if a.action=='prepare'else run()
