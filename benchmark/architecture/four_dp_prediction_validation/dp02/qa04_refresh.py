"""Four independent real Rust patches and acceptance against frozen DP02 source. API-free."""
import ast,copy,difflib,hashlib,json,shutil,subprocess,sys
from pathlib import Path
H=Path(__file__).resolve().parent;R=H.parents[3];OUT=R/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp02/qa04';ORIG=R/'prototypes/candidates/four_dp_prediction_validation/dp02';OLD=R/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/qa04'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def prepare():
 if OUT.exists():raise RuntimeError('Preserve prior task generation')
 OUT.mkdir(parents=True);freeze=R/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp02/primary/freeze.json'
 constants=json.loads((OLD/'controls-corrected-v1/control-constants.json').read_text())
 for task in ['C01','C03','C04','C06']:
  folder=OUT/task;crate=folder/'candidate'
  if task in ['C01','C03']:
   shutil.copytree(ORIG,crate,ignore=shutil.ignore_patterns('target'))
   if task=='C01':(crate/'src/model_wire.rs').write_text(constants['WIRE'])
   else:
    p=crate/'src/source_contract.rs';s=p.read_text();p.write_text(s[:s.index('pub fn normalize(')]+constants['NORMALIZER'])
    p=crate/'src/main.rs';s=p.read_text();needle='let input:Value=serde_json::from_str(&line).unwrap();';assert s.count(needle)==1;p.write_text(s.replace(needle,needle+'\n    let input=source_contract::normalize_input(&input);'))
  else:
   s=(OLD/'dp02'/('prepare_c04_v2.py'if task=='C04'else'prepare_c06_v2.py')).read_text()
   s=s.replace("BASE=ROOT/'calibration-v2/frozen/source/prototypes/candidates/four_dp_poc_v2'",'BASE=ORIG')
   s=s.replace("TASK=Path(__file__).resolve().parent/'C04-v2'","TASK=OUT/'C04'").replace("TASK=Path(__file__).resolve().parent/'C06-v2'","TASK=OUT/'C06'")
   s=s.replace("hashlib.sha256((ROOT/'calibration-v2/freeze.json').read_bytes()).hexdigest()",'sha(freeze)')
   exec(compile(s,str(folder/'reapplied-template.py'),'exec'),{'__file__':str(OLD/'dp02/prepare_c04_v2.py'),'ORIG':ORIG,'OUT':OUT,'freeze':freeze,'sha':sha})
  changes=[];diff=[]
  for p in sorted((ORIG/'src').rglob('*.rs')):
   after=crate/p.relative_to(ORIG)
   if p.read_bytes()!=after.read_bytes():
    changes.append(str(p.relative_to(ORIG)));diff.extend(difflib.unified_diff(p.read_text().splitlines(True),after.read_text().splitlines(True),fromfile='frozen/'+str(p.relative_to(ORIG)),tofile=task+'/'+str(p.relative_to(ORIG))))
  (folder/'actual.patch').write_text(''.join(diff));save(folder/'manifest.json',{'task':task,'baseline_freeze_sha256':sha(freeze),'baseline_source_sha256':{str(p.relative_to(ORIG)):sha(p)for p in sorted((ORIG/'src').rglob('*.rs'))},'changes':changes,'count':len(changes),'alternatives':['A','B'],'new_implementation_not_inferred_from_old_counts':True,'template_provenance':str(OLD),'API_calls':0})
 print('Prepared four independent task crates')
def migrate(v):
 if isinstance(v,list):return[migrate(x)for x in v]
 if not isinstance(v,dict):return v
 x={k:migrate(z)for k,z in v.items()}
 if 'id'in x and 'revision'in x and('path'in x or'text'in x):x['version']={'number':x.pop('revision')}
 return x
def accept():
 sys.path.insert(0,str(H));import run,offline_dp02
 logs=[];checks=[];fixture=next(x for x in json.loads((H/'fixtures/dp02.json').read_text())if x['id']=='02-N-none-independent-K2')
 class Probe(run.OfflineClient):
  def call(self,payload,meta):
   x=payload.get('input',{});resp,receipt=super().call(payload,meta)
   if 'response'in meta['stage']:
    if any('[WITHDRAWN RANGE]'in s.get('text','')for s in x.get('sources',[])):
     resp={'answer':'The Cedar price was withdrawn. Birch costs 64 dollars monthly. Cedar checkpoints every 12 seconds.','sections':[{'id':'allowed','text':'Birch costs 64 dollars monthly; Cedar checkpoints every 12 seconds.'}],'applied_revision':x['current_revision'],'source_claims':[{'source_id':'D7','revision':3,'quote':'Birch costs 64 dollars monthly'}]}
    else:resp['sections'].append({'id':'recovery','text':'Cedar checkpoint 12 seconds; Birch checkpoint 28 seconds.'})
   receipt['qa04_payload']=payload;return resp,receipt
 for task in ['C01','C03','C04','C06']:
  folder=OUT/task;run.CRATE=folder/'candidate'
  for action in ['build','test']:
   args=['cargo',action,'--offline','--manifest-path',str(run.CRATE/'Cargo.toml')];p=subprocess.run(args,capture_output=True,text=True);logs.append({'command':args,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr});save(OUT/'commands.json',logs);assert p.returncode==0
  for variant in ['A','B']:
   case=copy.deepcopy(fixture);case['id']='qa04-'+task+'-'+variant
   if task=='C03':case=migrate(case)
   if task=='C04':case['incremental_updates']=[{'goal_id':'R1','section_id':'cost','section_revision':2,'text':'Updated cost: Cedar95; Birch63.'},{'goal_id':'R1','section_id':'cost','section_revision':1,'text':'STALE'}]
   if task=='C06':
    t=case['sources'][0]['text'];a=t.index('Cedar costs 96 dollars monthly');b=a+len('Cedar costs 96 dollars monthly');case['source_range_revocations']=[{'source_id':'D7','revision':3,'start_byte':a,'end_byte':b,'trigger':'FIRST_DRAFT_READY','authority':'user_access_event'}]
   trace=run.execute(Probe(folder/'acceptance'),case,variant,1,'QA04_OFFLINE');ev=trace['events'];get=lambda kind:[e['data']for e in ev if e.get('kind')==kind]
   assert not get('GOAL_FAILED')and not get('EPISODE_ERROR');assert len(get('GOAL_COMPLETE'))==2
   result=get('EPISODE_RESULT')[-1];assert any(o['goal_id']=='U2'for o in result['outputs'])
   passed=['actual_patched_Rust_execution','original_goal_and_independent_goal_preserved','no_failures']
   if task=='C01':assert len(trace['calls'])>=2;passed+=['wire_v2_actual_request_and_usage_roundtrip']
   if task=='C03':assert any(o['goal_id']=='R1'and '96'in o['text']for o in result['outputs']);passed+=['nested_source_version_normalized_before_interpretation','D7revision3_source_quotes_accepted']
   if task=='C04':
    patches=get('CONTENT_SECTION_PATCH');assert len(patches)==1;actual=patches[0];assert actual['goal_revision']==1
    sections=actual['full_current_delivery']['result']['sections'];assert next(s['text']for s in sections if s['id']=='cost')=='Updated cost: Cedar95; Birch63.';assert next(s['text']for s in sections if s['id']=='recovery')=='Cedar checkpoint 12 seconds; Birch checkpoint 28 seconds.';assert len(get('SECTION_PATCH_REJECTED'))==1
    refs=actual['pending_voice'];assert any(v['section_id']=='cost'and v['section_revision']==2 for v in refs);assert any(v['section_id']=='recovery'and v['section_revision']==1 for v in refs)
    passed+=['target_section_only_updated','other_section_preserved','stale_rejected','pending_voice_reference_revised']
   if task=='C06':
    assert len(get('SOURCE_RANGE_REVOKED'))==1 and len(get('REVOKED_DRAFT_REJECTED'))==1
    final=next(x for x in get('GOAL_COMPLETE')if x['goal_id']=='R1');assert'96'not in final['text']and'12 seconds'in final['text'];passed+=['actual_draft_invalidated_before_publication','real_bounded_rejudgment','allowed_range_retained','withdrawn_price_not_delivered']
   checks.append({'task':task,'variant':variant,'pass':True,'checks':passed,'run_id':trace['run_id'],'calls':'offline doubles only'});save(folder/'acceptance-summary.json',[c for c in checks if c['task']==task])
 summary={'checks':checks,'API_calls':0,'tasks':[{**json.loads((OUT/task/'manifest.json').read_text()),'accepted':True}for task in ['C01','C03','C04','C06']],'mean_by_alternative':{v:sum(json.loads((OUT/t/'manifest.json').read_text())['count']for t in ['C01','C03','C04','C06'])/4 for v in ['A','B']},'scope':'Actual independent new patches and code-function acceptance against new frozen baseline; offline double timings are not model latency.'};save(OUT/'summary.json',summary);print(json.dumps({'checks':len(checks),'mean_by_alternative':summary['mean_by_alternative'],'API_calls':0}))
if __name__=='__main__':
 if sys.argv[-1]=='prepare':prepare()
 elif sys.argv[-1]=='accept':accept()
 else:raise SystemExit('Use prepare or accept')
