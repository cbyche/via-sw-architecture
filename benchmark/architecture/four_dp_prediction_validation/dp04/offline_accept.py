"""24 actual Rust paths with a structural semantic double. No provider quality claim."""
from pathlib import Path
import sys,json,shutil,hashlib
BASE=Path(__file__).resolve().parent;sys.path.insert(0,str(BASE));import run
from bridge import dump
ROOT=BASE.parents[3]/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp04/offline'
ROOT.mkdir(parents=True,exist_ok=True)
binary=ROOT/'frozen/bin/four_dp_poc';binary.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(run.CRATE/'target/debug/four_dp_poc',binary);dump(ROOT/'freeze.json',{'binary':{'path':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}})
checks=[]
for case in json.loads((BASE/'fixtures/dp04.json').read_text()):
 for variant in ['A','B','A4','B4']:
  record=run.execute(run.OfflineClient(ROOT),case,variant,1,'OFFLINE_STRUCTURAL')
  failures=[e for e in record['events']if e.get('kind')in['EPISODE_ERROR','GOAL_FAILED']];complete=[e for e in record['events']if e.get('kind')=='GOAL_COMPLETE']
  good=not failures and len(complete)==len(case['revisions']);checks.append({'episode':case['id'],'variant':variant,'pass':good,'run_id':record['run_id'],'scope':'ActualRustsource/revision/gate/wire/display paths only; external authoredsource semanticdouble; no modelaccuracy/timing','failures':failures});dump(ROOT/'acceptance.json',{'checks':checks,'provider_calls':0})
  assert good,checks[-1]
print(json.dumps({'paths':len(checks),'all_pass':True,'provider_calls':0}))
