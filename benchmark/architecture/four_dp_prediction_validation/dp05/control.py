"""Frozen fresh prediction-validation. No oracle is passed to the candidate/provider."""
import argparse,hashlib,json,os,shutil,subprocess,sys,time,platform
from pathlib import Path
HERE=Path(__file__).resolve().parent;REPO=HERE.parents[3];DP=HERE.name
RESULT=REPO/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019'/DP
CRATE=REPO/'prototypes/candidates/four_dp_prediction_validation'/DP
sys.path.insert(0,str(HERE));import run as harness
from bridge import Client,dump

def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def freeze():
 if (RESULT/'freeze.json').exists():raise RuntimeError('Immutable freeze already exists')
 files=[f for root in [HERE,CRATE] for f in root.rglob('*') if f.is_file() and not {'target','__pycache__'}.intersection(f.parts)]
 binary=RESULT/'frozen/bin/four_dp_poc';binary.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(CRATE/'target/debug/four_dp_poc',binary)
 for f in files:
  out=RESULT/'frozen/source'/f.relative_to(REPO);out.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(f,out)
 manifest={'generation':'PREDICTION_VALIDATION_V1','source':{str(f.relative_to(REPO)):sha(f) for f in files},'binary':{'path':str(binary),'sha256':sha(binary)},'repetitions':3,'prior_pooling':'FORBIDDEN_CHANGED_INPUT_POLICY_IMPLEMENTATION','environment':{'platform':platform.platform(),'python':platform.python_version()},'model':'qwen3.7-plus-2026-05-26','temperature':0,'thinking':False,'voice':'OFF','display':'USER_ACCEPTED_CALLBACK_PROXY'}
 dump(RESULT/'freeze.json',manifest)
class Probe(harness.OfflineClient):
 def call(self,payload,meta):
  self.requests.append({'meta':meta,'payload':payload})
  # Protocol probe deliberately ignores semantic oracle; exercises every supplied source.
  return super().call(payload,meta)
def worker(index,offline=False,bridge=False):
 specs=read(HERE/('bridge-schedule.json' if bridge else 'schedule.json'));spec=specs[index]
 case=read(HERE/'legacy-bridge.json') if bridge else next(c for c in read(HERE/'fixtures.json') if c['id']==spec['episode'])
 if offline:
  root=RESULT/'offline';root.mkdir(parents=True,exist_ok=True);client=Probe(root);client.requests=[]
 else:
  auth=read(RESULT/'API-WINDOW.json')
  if auth['freeze_sha256']!=sha(RESULT/'freeze.json'):raise RuntimeError('Wrong API window freeze')
  if bridge and not auth.get('bridge',False):raise RuntimeError('Bridge API not authorized')
  client=Client(RESULT,limit=1.60,ledger_root=RESULT.parent)
 record=harness.execute(client,case,spec['variant'],1 if offline else spec['repetition'],'LEGACY_BRIDGE' if bridge else 'PREDICTION_VALIDATION')
 if offline:dump(RESULT/'offline'/('requests-'+str(index)+'.json'),client.requests)
 print(json.dumps({'index':index,'run_id':record['run_id'],'offline':offline,'calls':len(record['calls']),'cost':sum(c.get('cost_usd') or 0 for c in record['calls']),'failures':[e.get('data') for e in record['events'] if e.get('kind') in ('GOAL_FAILED','EPISODE_ERROR')]}),flush=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('action',choices=['freeze','worker']);p.add_argument('--index',type=int,default=0);p.add_argument('--offline',action='store_true');p.add_argument('--bridge',action='store_true');a=p.parse_args()
 if a.action=='freeze':freeze()
 else:worker(a.index,a.offline,a.bridge)
if __name__=='__main__':main()
