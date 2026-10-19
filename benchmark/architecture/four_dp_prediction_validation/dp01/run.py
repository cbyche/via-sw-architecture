"""Review-v2 fresh-per-trial Rust execution, real callbacks, causal events and RSS proxy."""
import argparse,hashlib,json,os,platform,subprocess,threading,time,uuid,shutil,sys
from pathlib import Path
from bridge import Client,ModelError,dump,MODEL,PRICE
from memory import LogicalBuffers,sample_processes
BASE=Path(__file__).resolve().parent
REPO=BASE.parents[3]
CRATE=REPO/'prototypes/candidates/four_dp_prediction_validation/dp01'
DEFAULT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'

def read(path):return json.loads(Path(path).read_text())
def digest(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def freeze(root):
    if (root/'freeze.json').exists():raise RuntimeError('Freeze already exists; use a new immutable phase directory')
    files=sorted([p for p in BASE.rglob('*') if p.is_file() and '__pycache__' not in str(p)]+[p for p in CRATE.rglob('*') if p.is_file() and 'target/' not in str(p)])
    binary=root/'frozen/bin/four_dp_poc';binary.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(CRATE/'target/debug/four_dp_poc',binary)
    for source in files:
        destination=root/'frozen/source'/source.relative_to(REPO);destination.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,destination)
    rubric=root.parent/'analysis/dp05-semantic-review-rubric.md';external={}
    if rubric.exists():
        destination=root/'frozen/evaluator/dp05-semantic-review-rubric.md';destination.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(rubric,destination);external[str(rubric)]={'sha256':digest(rubric),'snapshot':str(destination.resolve()),'candidate_input':False}
    manifest={'evaluator_external_sources':external,'source_snapshot':str((root/'frozen/source').resolve()),'binary':{'path':str(binary.resolve()),'sha256':digest(binary)},'generation':'USER_REVIEW_V2','model':MODEL,'thinking':False,'prices':PRICE,'voice':'OFF','display':'DISPLAY_PROXY_USER_ACCEPTED_CALLBACK','repetitions':3,'budget_global_usd':4,'source':{str(p.relative_to(REPO)):digest(p) for p in files},'environment':{'platform':platform.platform(),'python':platform.python_version(),'rust':subprocess.check_output(['rustc','--version'],text=True).strip()},'claim_scope':'Authored finite fixtures; fresh Python/Rust RSS proxy; no product population or local Omni claim'}
    dump(root/'freeze.json',manifest);return manifest

class OfflineClient:
    def __init__(self,root,probe=None):self.probe=probe;self.root=Path(root);self.lock=threading.Lock();self.semaphore=threading.Semaphore(2);self.buffers=LogicalBuffers();self.calls={}
    def call(self,payload,meta):
        import importlib
        module=importlib.import_module('offline_dp'+meta['dp'])
        at=time.monotonic_ns();result=module.respond(payload,{**meta,'offline_probe':self.probe})
        record={**meta,'call_uuid':'OFFLINE-'+uuid.uuid4().hex,'causal_trace':payload.get('_trace',{}),'enqueue_ns':at,'send_ns':at,'terminal_ns':time.monotonic_ns(),'role':payload.get('role','VIA'),'status':'OFFLINE_RESPONDER','cost_usd':0,'input_tokens':0,'output_tokens':0,'not_provider_measurement':True}
        return result,record

def verify_freeze(root):
    manifest=read(root/'freeze.json')
    for relative,expected in manifest['source'].items():
        source=REPO/relative
        if not source.exists() or digest(source)!=expected:raise RuntimeError('Frozen source changed: '+relative)
    return manifest

def execute(client,episode,variant,rep,phase='BASIC',tactic=None):
    if not isinstance(client,OfflineClient):verify_freeze(client.root)
    client.buffers=LogicalBuffers()
    case={**episode,'variant':variant}
    if tactic:case['tactic']=tactic
    run_id=f'{phase.lower()}-{episode["id"]}-{variant}-r{rep}-{uuid.uuid4().hex[:6]}'
    folder=client.root/'raw'/'runs'/run_id;folder.mkdir(parents=True)
    dump(folder/'input.json',case)
    buffers=client.buffers
    buffers.set('input_json',len(json.dumps(case,ensure_ascii=False).encode()),'input')
    baseline=sample_processes({os.getpid():'python_runner_bridge'},'BEFORE_RUST_START',buffers)
    binary=Path(read(client.root/'freeze.json')['binary']['path']) if (client.root/'freeze.json').exists() else CRATE/'target/debug/four_dp_poc'
    prestart_binary_sha256=digest(binary)
    if (client.root/'freeze.json').exists() and prestart_binary_sha256!=read(client.root/'freeze.json')['binary']['sha256']:raise RuntimeError('Frozen binary digest mismatch')
    t0=time.monotonic_ns();proc=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1)
    events=[];calls=[];threads=[];write_lock=threading.Lock();memory=[];stop=threading.Event();sample_ready=threading.Event();stage={'name':'RUST_WAITING_FOR_INPUT'}
    roles={os.getpid():'python_runner_bridge',proc.pid:'rust_candidate'}
    def sampler():
        while not stop.is_set():
            memory.append(sample_processes(roles,stage['name'],buffers));sample_ready.set();stop.wait(.05)
    memthread=threading.Thread(target=sampler,daemon=True);memthread.start()
    # Startup observation is outside user stimulus. No model/service delay is fabricated.
    sample_ready.wait(5)
    client.configure_slots(case.get('k',2)) if hasattr(client,'configure_slots') else setattr(client,'semaphore',threading.Semaphore(case.get('k',2)))
    proc.stdin.write(json.dumps(case,ensure_ascii=False)+'\n');proc.stdin.flush()
    def model(msg):
        payload=msg.get('payload',msg.get('request',{}).get('context',{}))
        meta={'run_id':run_id,'phase':phase,'dp':case['dp'],'variant':variant,'episode':case['id'],'repetition':rep,'stage':msg['stage'],'rust_call_id':msg['call_id']}
        stage['name']='MODEL:'+msg['stage']
        try:
            result,record=client.call(payload,meta);response={'call_id':msg['call_id'],'result':result,'usage':record.get('raw_usage')}
            record['response_value']=result
            if msg.get('wire_version')==2:response={'call_id':msg['call_id'],'provider_response':{'json':result,'usage':{'input_tokens':record.get('input_tokens'),'output_tokens':record.get('output_tokens')}}}
        except ModelError as error:record=error.record;response={'call_id':msg['call_id'],'error':str(error)}
        except Exception as error:record={**meta,'status':'BRIDGE_ERROR','error':str(error),'terminal_ns':time.monotonic_ns()};response={'call_id':msg['call_id'],'error':str(error)}
        calls.append(record)
        buffers.set('retained_call_records',len(json.dumps(calls,ensure_ascii=False).encode()),'retained_evidence')
        with write_lock:
            try:proc.stdin.write(json.dumps(response,ensure_ascii=False)+'\n');proc.stdin.flush()
            except BrokenPipeError:record['discarded_after_process_end']=True
    for line in proc.stdout:
        try:msg=json.loads(line)
        except Exception:events.append({'type':'INVALID_STDOUT','line':line});continue
        msg['observed_ns']=time.monotonic_ns()
        if msg.get('type')=='model_call':
            thread=threading.Thread(target=model,args=(msg,),daemon=True);threads.append(thread);thread.start()
        else:
            events.append(msg);stage['name']=msg.get('kind','EVENT')
            buffers.set('retained_events',sum(len(json.dumps(e,ensure_ascii=False).encode()) for e in events),'retained_evidence')
            if msg.get('kind')=='LOGICAL_BUFFER':
                d=msg['data'];name='rust:'+d['name']
                if d['action']=='RELEASE':buffers.release(name)
                else:buffers.set(name,d['bytes'],'rust_serialized','rust_candidate')
    for thread in threads:thread.join()
    proc.wait();stop.set();memthread.join();stderr=proc.stderr.read()
    record={'schema_version':2,'run_id':run_id,'dp':case['dp'],'episode':case['id'],'variant':variant,'repetition':rep,'phase':phase,'tactic':tactic,'candidate_binary_sha256':prestart_binary_sha256,'postrun_binary_sha256':digest(binary),'candidate_binary_path':str(binary),'freeze_sha256':digest(client.root/'freeze.json') if (client.root/'freeze.json').exists() else None,'start_ns':t0,'end_ns':time.monotonic_ns(),'events':events,'calls':calls,'memory':memory,'memory_baseline':baseline,'logical_buffer_events':buffers.events,'process_registry':[{'pid':p,'role':r} for p,r in roles.items()],'memory_nominal_poll_ms':50,'stderr':stderr,'returncode':proc.returncode,'input':str(folder/'input.json'),'offline_responder':isinstance(client,OfflineClient)}
    if record['postrun_binary_sha256']!=prestart_binary_sha256:record['binary_mutation_error']=True
    dump(folder/'trace.json',record)
    with (client.root/'runs.jsonl').open('a') as f:f.write(json.dumps(record,ensure_ascii=False)+'\n')
    return record

def main():
    p=argparse.ArgumentParser();p.add_argument('--results',default=str(DEFAULT));p.add_argument('--dp',choices=['01','02','04','05']);p.add_argument('--variant');p.add_argument('--episode');p.add_argument('--reps',type=int,default=3);p.add_argument('--single-repetition',type=int);p.add_argument('--phase',default='BASIC');p.add_argument('--tactic');p.add_argument('--freeze-only',action='store_true');p.add_argument('--skip-build',action='store_true');p.add_argument('--offline',action='store_true');p.add_argument('--offline-probe');p.add_argument('--single-trial-worker',action='store_true');a=p.parse_args();root=Path(a.results);root.mkdir(parents=True,exist_ok=True)
    if not a.skip_build:subprocess.run(['cargo','build','--offline','--manifest-path',str(CRATE/'Cargo.toml')],check=True)
    if a.freeze_only:freeze(root);print('Frozen '+str(root/'freeze.json'));return
    if not a.offline and not (root/'MODEL-CALLS-AUTHORIZED.json').exists():raise RuntimeError('No model call: parent freeze authorization marker missing')
    dps=[a.dp] if a.dp else ['01','02','04','05'];jobs=[]
    for dp in dps:
        episodes=read(BASE/'fixtures'/f'dp{dp}.json');variants=[a.variant] if a.variant else list('ABC' if dp in ('01','05') else 'AB')
        for rep in ([a.single_repetition] if a.single_repetition else range(1,a.reps+1)):
            order=variants[(rep-1)%len(variants):]+variants[:(rep-1)%len(variants)]
            for episode in episodes:
                if a.episode and episode['id']!=a.episode:continue
                jobs.extend((dp,episode,variant,rep) for variant in order)
    if len(jobs)>1 and not a.single_trial_worker:
        for dp,episode,variant,rep in jobs:
            args=[sys.executable,str(Path(__file__).resolve()),'--results',str(root),'--dp',dp,'--episode',episode['id'],'--variant',variant,'--single-repetition',str(rep),'--phase',a.phase,'--skip-build','--single-trial-worker']
            if a.offline:args+=['--offline']
            if a.offline_probe:args+=['--offline-probe',a.offline_probe]
            if a.tactic:args+=['--tactic',a.tactic]
            subprocess.run(args,check=True)
        return
    if len(jobs)!=1:raise RuntimeError('Worker requires exactly one trial')
    auth=read(root/'MODEL-CALLS-AUTHORIZED.json') if not a.offline else {}
    if not a.offline:
        key={'dp':jobs[0][0],'episode':jobs[0][1]['id'],'variant':jobs[0][2],'repetition':jobs[0][3]}
        if auth.get('phase')!=a.phase or key not in auth.get('allowed_jobs',[]):raise RuntimeError('This exact phase/job is not authorized')
        if digest(root/'freeze.json')!=auth.get('freeze_sha256'):raise RuntimeError('Authorized freeze digest mismatch')
    client=OfflineClient(root,a.offline_probe) if a.offline else Client(root,limit=4,ledger_root=auth['campaign_root'])
    dp,episode,variant,rep=jobs[0]
    record=execute(client,episode,variant,rep,'OFFLINE' if a.offline else a.phase,a.tactic)
    error=any(e.get('kind') in ('EPISODE_ERROR','GOAL_FAILED') for e in record['events'])
    print(json.dumps({'run_id':record['run_id'],'calls':len(record['calls']),'status':'ERROR' if error else 'TERMINAL_RECORDED','not_provider_measurement':a.offline}),flush=True)
if __name__=='__main__':main()
