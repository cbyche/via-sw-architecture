"""Small frozen-candidate executor; recorded responses are never provider measurements."""
import fcntl,hashlib,json,os,shutil,sys,threading,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
OLD=REPO/'benchmark/architecture/four_dp_poc_v2'
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019'
HISTORY=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
sys.path.insert(0,str(OLD))
import run as harness
from bridge import Client,dump,MODEL
from memory import LogicalBuffers

def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def freeze(dp):
 here=HERE.parent/dp;result=ROOT/dp;plan=read(result/'plan.json')
 files=[p for p in here.rglob('*') if p.is_file() and not {'target','__pycache__'}.intersection(p.parts)]
 files += [HERE/'experiment_common.py'] if dp!='dp01' else []
 files += [p for p in OLD.glob('*.py')]
 for p in files:
  out=result/'frozen/source'/p.relative_to(REPO);out.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,out)
 binary=here/'candidate/target/debug/four_dp_poc';out=result/'frozen/bin/four_dp_poc';out.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(binary,out)
 manifest={'source':{str(p.relative_to(REPO)):sha(p) for p in files},'binary':{'path':str(out),'sha256':sha(out)},'plan_sha256':sha(result/'plan.json'),'model':MODEL,'thinking':False,'temperature':0,'prior_raw_modified':False}
 if (result/'freeze.json').exists():raise RuntimeError('No replacement of existing freeze')
 dump(result/'freeze.json',manifest)
 return manifest
class ActualResponseReplay:
 def __init__(self,root,old):self.root=Path(root);self.original=old;self.at=0;self.lock=threading.Lock();self.buffers=LogicalBuffers();self.requests=[]
 def call(self,payload,meta):
  with self.lock:
   call=self.original['calls'][self.at];self.at+=1
   if call['stage']!=meta['stage']:raise RuntimeError('Recorded stage mismatch; preserve failure, no canned fallback')
   value=call['response_value'];at=time.monotonic_ns();actual={k:v for k,v in payload.items() if k!='_trace'};self.requests.append(actual)
   dump(self.root/'replayed-requests'/meta['run_id']/f'{self.at:03}.json',{'source_call_uuid':call.get('call_uuid'),'payload':actual,'source_response':value})
   return value,{**meta,'call_uuid':'REPLAY-'+str(self.at),'role':payload.get('role','VIA'),'enqueue_ns':at,'send_ns':at,'terminal_ns':time.monotonic_ns(),'status':'ACTUAL_RECORDED_RESPONSE_REPLAY','input_tokens':0,'output_tokens':0,'cost_usd':0,'original_call_uuid':call.get('call_uuid'),'original_input_tokens':call.get('input_tokens'),'original_output_tokens':call.get('output_tokens'),'not_provider_measurement':True,'payload_sha256':hashlib.sha256(json.dumps(actual,sort_keys=True,ensure_ascii=False).encode()).hexdigest()}
class BoundedClient(Client):
 def budget(self,id,amount,state):
  path=ROOT/'dp01-dp05-budget.json'
  with path.open('a+') as f:
   fcntl.flock(f,fcntl.LOCK_EX);f.seek(0);data=json.loads(f.read() or '{"limit_usd":0.85,"calls":{}}')
   if state=='RESERVED' and sum(x['amount_usd'] for x in data['calls'].values())+amount>.85:raise RuntimeError('DP01/DP05 combined $0.85 budget stop')
   super().budget(id,amount,state)
   data['calls'][id]={'amount_usd':amount,'state':state};f.seek(0);f.truncate();json.dump(data,f,indent=2);f.flush();os.fsync(f.fileno())
def old_rows(dp):
 phase=HISTORY/('basic-v1' if dp=='dp01' else 'basic-dp05-reuse-union-corrected-v2')
 # Only load compact names here; workers read one actual record.
 return [(str(p),read(p)['episode'],read(p)['variant'],read(p)['repetition']) for p in sorted((phase/'raw/runs').glob('*/trace.json')) if read(p)['dp']==dp[-2:]]
def replay(dp,index,tactic):
 result=ROOT/dp/'replay'/tactic;result.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/dp/'freeze.json',result/'freeze.json')
 rows=[r for r in old_rows(dp) if r[2]==('A' if dp=='dp01' else 'C')];old=read(rows[index][0]);case=read(old['input']);variant=case.pop('variant');case.pop('tactic',None)
 client=ActualResponseReplay(result,old);record=harness.execute(client,case,variant,old['repetition'],'ACTUAL_RESPONSE_REPLAY',None if tactic=='basic' else tactic)
 summary={'run_id':record['run_id'],'source_trace':rows[index][0],'source_trace_sha256':sha(rows[index][0]),'replayed_calls':client.at,'original_calls':len(old['calls']),'payload_hashes':[c['payload_sha256'] for c in record['calls']],'returncode':record['returncode'],'failed_events':[e for e in record['events'] if e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR']],'source_reads':sum(e.get('kind')=='SOURCE_READ' for e in record['events']),'cache_hits':sum(e.get('kind')=='C2_EXACT_READ_HIT' for e in record['events']),'working_view_evictions':[e['data'] for e in record['events'] if e.get('kind')=='WORKING_VIEW_BUDGET_EVICTION'],'not_fresh_model_measurement':True}
 dump(result/'summaries'/f'{index:02}.json',summary);print(json.dumps(summary,ensure_ascii=False))
