"""New finite condition-balanced DP04 campaign. Requires root's explicit API window marker."""
import argparse,copy,fcntl,hashlib,json,os,platform,select,shutil,subprocess,sys,time,uuid
from pathlib import Path
BASE=Path(__file__).resolve().parent;REPO=BASE.parents[3];CRATE=REPO/'prototypes/candidates/four_dp_prediction_validation/dp04';ROOT=REPO/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019';RESULT=ROOT/'dp04';PHASE=RESULT/'main';PYTHON='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3'
sys.path.insert(0,str(BASE))
import run as harness
from bridge import Client,ModelError,dump,MODEL

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(p):return json.loads(Path(p).read_text())
class LiveClient(Client):
 def __init__(self,root):
  super().__init__(root,limit=1.4,ledger_root=RESULT);self.parser_process=None;self.parser_cache={};self.parser_log=None;self.process_role_history={}
 def local_parser(self,payload,meta):
  start=time.monotonic_ns();call_id='LOCAL-'+uuid.uuid4().hex;created=False
  if self.parser_process is None:
   created=True;self.parser_log=(self.root/'raw'/('parser-'+meta['run_id']+'.stderr')).open('w');self.parser_process=subprocess.Popen(['/private/tmp/via-dp04-omniparser-venv/bin/python',str(BASE/'parser/service.py')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.parser_log,text=True,bufsize=1);self.process_roles[self.parser_process.pid]='real_local_detector_ocr_caption';self.process_role_history[self.parser_process.pid]='real_local_detector_ocr_caption'
   if not select.select([self.parser_process.stdout],[],[],180)[0]:raise RuntimeError('Parser initialization timeout180s')
   ready=json.loads(self.parser_process.stdout.readline())
   if not ready.get('ready'):raise RuntimeError('Parser failed startup')
  frames=payload['input']['frames'];needed=[f for f in frames if (f['frame_id'],f['source_revision'],f['sha256']) not in self.parser_cache]
  if needed:
   self.parser_process.stdin.write(json.dumps({'frames':needed})+'\n');self.parser_process.stdin.flush()
   if not select.select([self.parser_process.stdout],[],[],240)[0]:raise RuntimeError('Parser frame batch timeout240s')
   result=json.loads(self.parser_process.stdout.readline())
   if result.get('parser_error'):raise RuntimeError(result['parser_error'])
   for f in result['frames']:self.parser_cache[(f['frame_id'],f['source_revision'],f['source_sha256'])]=f
  result={'parser':'Real OmniParser-v2.0 detector/EasyOCR/Florence2 CPU','frames':[self.parser_cache[(f['frame_id'],f['source_revision'],f['sha256'])] for f in frames],'fallible_derived_interpretation':True,'not_native_evidence':True}
  record={**meta,'call_uuid':call_id,'enqueue_ns':start,'send_ns':start,'terminal_ns':time.monotonic_ns(),'status':'LOCAL_PARSER_COMPLETED','evidence_kind':'LOCAL_PARSER_NOT_CLOUD_MODEL_CALL','cost_usd':0,'cloud_api_cost_applicable':False,'local_compute_cost':'NOT_MONETIZED','input_tokens':0,'output_tokens':0,'parser_initialized_here':created,'new_frames':len(needed),'reused_frames':len(frames)-len(needed),'parser_pid':self.parser_process.pid,'initialization':ready if created else None,'inference_ns_new_frames':sum(f.get('parse_ns',0) for f in result['frames'] if any(n['frame_id']==f['frame_id'] and n['sha256']==f['source_sha256'] for n in needed)),'response_value':result}
  dump(self.root/'raw'/'parser'/(call_id+'.json'),record)
  return result,record
 def call(self,payload,meta):
  if meta['stage']=='dp04.local_screen_parser':
   start=time.monotonic_ns()
   try:return self.local_parser(payload,meta)
   except Exception as error:
    record={**meta,'call_uuid':'LOCAL-FAILED-'+uuid.uuid4().hex,'enqueue_ns':start,'send_ns':start,'terminal_ns':time.monotonic_ns(),'status':'LOCAL_PARSER_FAILED','error':type(error).__name__+': '+str(error),'evidence_kind':'LOCAL_PARSER_NOT_CLOUD_MODEL_CALL','cost_usd':0,'cloud_api_cost_applicable':False,'local_compute_cost':'NOT_MONETIZED','input_tokens':0,'output_tokens':0}
    dump(self.root/'raw'/'parser'/(record['call_uuid']+'.json'),record);self.close_parser();raise ModelError(record) from error
  return super().call(payload,meta)
 def close_parser(self):
  if self.parser_process:
   try:
    self.parser_process.stdin.close();self.parser_process.wait(timeout=10)
   except (subprocess.TimeoutExpired,BrokenPipeError):
    self.parser_process.kill();self.parser_process.wait()
   self.process_roles.pop(self.parser_process.pid,None);self.parser_log.close();self.parser_process=None

def prepare():
 if PHASE.exists():raise RuntimeError('Refuse overwrite frozen phase')
 plan=read(RESULT/'plan.json')
 if not plan.get('preregistered'):raise RuntimeError('Preregistration missing')
 PHASE.mkdir(parents=True);files=[p for p in BASE.rglob('*') if p.is_file() and '__pycache__' not in p.parts and p.name!='vision_ocr' and not p.name.endswith('.omniparser.json')]+[p for p in CRATE.rglob('*') if p.is_file() and 'target' not in p.parts]
 for source in files:
  dest=PHASE/'frozen/source'/source.relative_to(REPO);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,dest)
 binary=PHASE/'frozen/bin/four_dp_poc';binary.parent.mkdir(parents=True);shutil.copy2(CRATE/'target/debug/four_dp_poc',binary)
 schedule=[]
 for rep in range(1,4):
  fixtures=read(BASE/'fixtures/dp04.json')
  if PHASE.name=='bq':fixtures=[e for e in fixtures if e['id'] in ['dp04-held-dense','dp04-held-region']]
  for i,episode in enumerate(fixtures):
   variants=['A','BQ'] if PHASE.name=='bq' else ['A','B','A4','B4'];offset=(rep+i-1)%len(variants);order=variants[offset:]+variants[:offset]
   if rep%2==0:order=list(reversed(order))
   schedule.extend(dict(dp='04',episode=episode['id'],variant=v,repetition=rep) for v in order)
 dump(PHASE/'schedule.json',schedule)
 manifest={'source':{str(p.relative_to(REPO)):sha(p) for p in files},'binary':{'path':str(binary.resolve()),'sha256':sha(binary)},'model':MODEL,'thinking':False,'temperature':0,'plan_sha256':sha(RESULT/'plan.json'),'schedule_sha256':sha(PHASE/'schedule.json'),'coordinate_wire':'integer0..1000 both owners -> original pixels ties-even no clamp; raw cues original pixels','parser':read(RESULT/'parser-inventory.json'),'memory':'50ms simultaneous Python+Rust+real local parser PID; shared-page dedup unverified; cold helper init included','environment':{'platform':platform.platform(),'python':platform.python_version()},'max_cloud_slots':2,'cost_rates_usd_per_million':{'input':.4,'output':1.6},'local_budget_usd':1.4}
 dump(PHASE/'freeze.json',manifest);print(json.dumps({'jobs':len(schedule),'freeze_sha256':sha(PHASE/'freeze.json')}))

def worker(index):
 window=read(PHASE/'API-WINDOW-GRANTED.json');freeze=read(PHASE/'freeze.json');job=read(PHASE/'schedule.json')[index]
 if window['freeze_sha256']!=sha(PHASE/'freeze.json'):raise RuntimeError('Root window freeze mismatch')
 if sha(PHASE/'schedule.json')!=freeze['schedule_sha256']:raise RuntimeError('Schedule mismatch')
 for absolute,expected in freeze['parser'].get('external_source_sha256',{}).items():
  if sha(absolute)!=expected:raise RuntimeError('Real parser dependency source changed: '+absolute)
 fixture=next(e for e in read(BASE/'fixtures/dp04.json') if e['id']==job['episode']);client=LiveClient(PHASE)
 record=harness.execute(client,fixture,job['variant'],job['repetition'],'PREDICTION_VALIDATION',tactic=job['variant'] if job['variant'] in ['A4','B4'] else None)
 print(json.dumps({'run_id':record['run_id'],'calls':len(record['calls']),'failed_events':sum(e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR'] for e in record['events'])}),flush=True)
def run():
 done=set()
 if (PHASE/'runs.jsonl').exists():done={(r['episode'],r['variant'],r['repetition']) for r in map(json.loads,(PHASE/'runs.jsonl').read_text().splitlines())}
 for i,j in enumerate(read(PHASE/'schedule.json')):
  if (j['episode'],j['variant'],j['repetition']) in done:continue
  c=subprocess.run([PYTHON,str(__file__),'worker','--phase',PHASE.name,'--index',str(i)],capture_output=True,text=True);(PHASE/'orchestration.log').open('a').write(c.stdout+c.stderr);print(c.stdout.strip(),flush=True)
  if c.returncode:raise RuntimeError('Worker failure; preserve and inspect')
  r=json.loads((PHASE/'runs.jsonl').read_text().splitlines()[-1])
  if any(x.get('http_status') not in (None,200) for x in r['calls']):raise RuntimeError('HTTP failure retained; pause unexecuted jobs')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','run','worker']);p.add_argument('--index',type=int);p.add_argument('--phase',choices=['main','bq'],default='main');a=p.parse_args();PHASE=RESULT/a.phase;{'prepare':prepare,'run':run,'worker':lambda:worker(a.index)}[a.action]()
