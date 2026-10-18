from pathlib import Path
import sys,json,subprocess,copy
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2/run.py').exists());sys.path.insert(0,str(REPO/'benchmark/architecture/four_dp_poc_v2'))
import run
ROOT=Path(__file__).resolve().parent;checks=[];commands=[]
def command(args):
 p=subprocess.run(args,capture_output=True,text=True);commands.append({'args':args,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr});assert p.returncode==0,p.stderr;return p

def migrate(value):
 if isinstance(value,list):return [migrate(v)for v in value]
 if not isinstance(value,dict):return value
 x={k:migrate(v)for k,v in value.items()}
 if 'frame_id'in x and'image_path'in x:
  x['version']={'number':x.pop('source_revision')};x['coordinate']={'unit':'pixels','dimensions':[x.pop('width'),x.pop('height')]};x.pop('coordinate_space');x['captured_at']={'value':x.pop('observed_at_ms')*1000,'unit':'microseconds'}
 elif'id'in x and'revision'in x and('path'in x or'text'in x):x['version']={'number':x.pop('revision')}
 return x
class CaptureClient(run.OfflineClient):
 def __init__(self,root):super().__init__(root);self.requests=[]
 def call(self,payload,meta):self.requests.append({'meta':meta,'payload':payload});return super().call(payload,meta)
command([str(ROOT/'C01/candidate/target/debug/contract_probe')])
for dp,variants in [('01','ABC'),('02','AB'),('04','AB'),('05','ABC')]:
 episodes=json.loads((run.BASE/'fixtures'/f'dp{dp}.json').read_text());episode=episodes[1] if dp=='05'else episodes[0]
 for variant in variants:
  for task in ['C01','C03']:
   run.CRATE=ROOT/task/'candidate';case=copy.deepcopy(episode);outgoing=migrate(case)if task=='C03'else case
   if task=='C03':
    inp=ROOT/'canonical-probes'/f'{dp}-{variant}-migrated.json';expected=inp.with_name(f'{dp}-{variant}-expected.json');inp.parent.mkdir(exist_ok=True);inp.write_text(json.dumps({**outgoing,'variant':variant},ensure_ascii=False));expected.write_text(json.dumps({**case,'variant':variant},ensure_ascii=False));command([str(run.CRATE/'target/debug/contract_probe'),str(inp),str(expected)])
   client=CaptureClient(ROOT/task/'acceptance-evidence');trace=run.execute(client,outgoing,variant,1,'QA04_CONTROL_OFFLINE');assert not any(e.get('kind')in['EPISODE_ERROR','GOAL_FAILED']for e in trace['events'])
   assert any(e.get('kind')=='GOAL_COMPLETE'for e in trace['events']);assert len(trace['calls'])>0
   requests=ROOT/task/'requests';requests.mkdir(exist_ok=True);(requests/(trace['run_id']+'.json')).write_text(json.dumps(client.requests,ensure_ascii=False,indent=2))
   checks.append({'task':task,'dp':dp,'variant':variant,'pass':True,'run_id':trace['run_id'],'deterministic_calls':len(trace['calls']),'provider_calls':0})
(ROOT/'acceptance-summary.json').write_text(json.dumps({'checks':checks,'provider_calls':0,'scope':'Actual patched runtime under finite deterministic contract double; not model accuracy or latency'},indent=2));(ROOT/'commands.json').write_text(json.dumps(commands,indent=2));print(json.dumps({'checks':len(checks),'pass':all(c['pass']for c in checks),'provider_calls':0}))
