"""Actual new C01/C03 scratch execution; normalized fixture semantics preserved."""
from pathlib import Path
import json,copy,subprocess,importlib.util,sys
H=Path(__file__).resolve().parent;sys.path.insert(0,str(H));import qa04_feature_accept as feature
R=H.parents[3];OUT=R/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp04/qa04'
spec=importlib.util.spec_from_file_location('prior_control_migration',R/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/qa04/controls-corrected-v1/rebase_controls.py');migration=importlib.util.module_from_spec(spec);spec.loader.exec_module(migration)
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
checks=[]
for task in ['C01','C03']:
 for variant in ['A','B']:
  folder=OUT/task/variant;crate=folder/'candidate'
  commands=[]
  for action in ['build','test']:
   args=['cargo',action,'--offline','--manifest-path',str(crate/'Cargo.toml')];p=subprocess.run(args,capture_output=True,text=True);commands.append({'command':args,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr});save(folder/'commands.json',commands);assert p.returncode==0
  case=copy.deepcopy(next(f for f in feature.FIXTURES if f['late_result_replay']));case['variant']=variant
  if task=='C03':case=migration.migrate(case)
  trace=feature.drive(crate,case);save(folder/'actual-acceptance-trace.json',trace)
  assert not any(e.get('kind') in ['EPISODE_ERROR','GOAL_FAILED']for e in trace['events']);assert len(trace['result']['outputs'])==2
  assert trace['result']['outputs'][0]['targets']==['C1','C2'];assert trace['result']['outputs'][1]['targets']==['C1','C3']
  if task=='C01':assert all(c['wire_version']==2 for c in trace['calls'])
  if task=='C03':assert all(c['wire_version']==1 for c in trace['calls']);assert trace['result']['outputs'][-1]['source_revisions']['document@correction']==1
  checks.append({'task':task,'variant':variant,'pass':True,'acceptance':['actualpatchedRust','two revisions andexactactualadoptions retained','actualwire2roundtrip'if task=='C01'else'newnestedframeversion/coordinate/capturedatnormalized beforeproduction'],'provider_calls':0});save(folder/'acceptance-summary.json',checks[-1])
save(OUT/'controls-summary.json',{'checks':checks,'provider_calls':0});print(json.dumps({'checks':len(checks),'all_pass':True}))
