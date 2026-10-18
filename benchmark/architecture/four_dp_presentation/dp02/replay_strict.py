"""DP02 B/B1 actual-response replay and same-sample memory. No API/network code.

Authored input arrivals and recorded capture-before-return/order constraints are
preserved. Elapsed replay time is never QA01/QA03 model latency.
"""
import argparse,collections,hashlib,json,os,subprocess,sys,threading,time
from pathlib import Path

HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if(p/'benchmark/architecture/four_dp_presentation').is_dir())
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019/dp02/strict-control-v2'
BINARY=HERE/'candidate/target/debug/four_dp_poc'
def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,x):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def normalized(x,key=''):
    if isinstance(x,dict):return {k:normalized(v,k)for k,v in x.items()}
    if isinstance(x,list):
        v=[normalized(a)for a in x]
        if key in ['requests','pending_uninterpreted_inputs','associations','outputs']:v.sort(key=lambda a:json.dumps(a,sort_keys=True))
        return v
    return x
def callkey(stage,x):return(stage,x.get('event',{}).get('id'),x.get('goal_id'),x.get('current_revision'))
def rss(roles,stage):
    start=time.monotonic_ns();r=subprocess.run(['ps','-o','pid=,ppid=,rss=','-p',','.join(map(str,roles))],capture_output=True,text=True,check=True)
    values=[]
    for line in r.stdout.splitlines():
        p,ppid,kb=map(int,line.split());values.append({'pid':p,'ppid':ppid,'role':roles[p],'rss_bytes':kb*1024})
    return {'start_ns':start,'end_ns':time.monotonic_ns(),'stage':stage,'processes':values,'sum_bytes':sum(v['rss_bytes']for v in values),'by_role':{v['role']:v['rss_bytes']for v in values}}

def worker(mode,n,episode,rep,phase='MEASURED_REPLAY'):
    capsule=read(ROOT/'inputs'/f'{episode}-r{rep}.json');schedule=read(ROOT/'schedules'/f'{episode}-r{rep}.json');schedule_by_id={c['original_call_uuid']:c for c in schedule['calls']};capsule['calls'].sort(key=lambda c:schedule_by_id[c['original_call_uuid']]['terminal_order']);case=capsule['input'];folder=ROOT/('trials'if phase=='MEASURED_REPLAY'else'preflight-host-order-v1')/f'{mode}-{episode}-N{n}-r{rep}'
    if folder.exists():raise ValueError('Immutable trial already exists: '+str(folder))
    folder.mkdir(parents=True);prehash=sha(BINARY);samples=[];stop=threading.Event();sample_lock=threading.Lock();stage={'name':'BEFORE_RUST_START'}
    baseline=rss({os.getpid():'python_replay'},stage['name']);proc=subprocess.Popen([str(BINARY)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1)
    roles={os.getpid():'python_replay',proc.pid:'rust_candidate'}
    def sample_loop():
        while not stop.is_set():
            try:s=rss(roles,stage['name'])
            except subprocess.CalledProcessError:break
            with sample_lock:samples.append(s)
            stop.wait(.01)
    sampler=threading.Thread(target=sample_loop);sampler.start()
    proc.stdin.write(json.dumps({'mode':mode,'N':n,'input':case})+'\n');proc.stdin.flush()
    bykey=collections.defaultdict(list)
    for i,c in enumerate(capsule['calls']):bykey[callkey(c['stage'],c['request_input'])].append(i)
    states={s:{'captures':set(),'barriers':set(),'pending':{},'assigned':set(),'next':0,'result':None,'failures':[],'callbacks':[],'payloads':[]}for s in range(1,n+1)}
    issues=[];heap_peak=0;first_wave=set();wave_sampled=False;calls=0
    def release_ready():
        nonlocal calls
        if not wave_sampled:return
        for sid,s in states.items():
            while s['next']<len(capsule['calls']):
                index=s['next'];expected=capsule['calls'][index];message=s['pending'].get(index)
                if message is None or not set(schedule_by_id[expected['original_call_uuid']]['required_barriers']).issubset(s['barriers']):break
                response={'call_id':message['call_id'],'result':expected['response'],'usage':expected['usage']}
                proc.stdin.write(json.dumps(response)+'\n');proc.stdin.flush();del s['pending'][index];s['next']+=1;calls+=1
    start=time.monotonic_ns()
    with(folder/'events.jsonl').open('w')as raw:
        for line in proc.stdout:
            raw.write(line);message=json.loads(line);heap_peak=max(heap_peak,message.get('allocator_peak_bytes',0));sid=message['session_id'];stage['name']=message.get('stage',message.get('kind','EVENT'))
            if sid==0:continue
            s=states[sid];kind=message.get('kind');d=message.get('data',{})
            if kind=='INPUT_CAPTURED':s['captures'].add(d['stimulus_id']);s['barriers'].add(f"{kind}:{d['stimulus_id']}")
            if kind=='ADOPTION_COMMITTED':s['barriers'].add(f"{kind}:{d['adoption_id']}")
            if kind in ['GOAL_COMPLETE','GOAL_REVISION']:s['barriers'].add(f"{kind}:{d['goal_id']}:{d['revision']}")
            if kind=='DOWNSTREAM_SHIM_RECEIPT':s['barriers'].add(f"{kind}:{d['goal_id']}")
            if kind=='EPISODE_RESULT':s['result']=d
            if kind in ['GOAL_FAILED','EPISODE_ERROR']:s['failures'].append({'kind':kind,'data':d})
            if kind in ['ADOPTION_COMMITTED','GOAL_COMPLETE','DOWNSTREAM_SHIM_RECEIPT']:s['callbacks'].append({'kind':kind,'data':d})
            if message.get('type')=='model_call':
                first_wave.add(sid);payload=message['payload'];key=callkey(message['stage'],payload['input']);indices=[i for i in bykey.get(key,[])if i not in s['assigned']]
                if not indices:
                    issues.append({'session':sid,'reason':'No recorded response for call key','key':key});proc.stdin.write(json.dumps({'call_id':message['call_id'],'error':'RECORDED_RESPONSE_UNAVAILABLE'})+'\n');proc.stdin.flush()
                else:
                    index=indices[0];s['assigned'].add(index);s['pending'][index]=message;s['barriers'].add('CALL_PREPARED:'+capsule['calls'][index]['original_call_uuid'])
                    s['payloads'].append({'stage':message['stage'],'input':payload['input'],'instruction':payload['instruction'],'max_tokens':payload['max_tokens']})
            if not wave_sampled and len(first_wave)==n:
                sample=rss(roles,'DECLARED_FIRST_CALL_WAVE_OBSERVATION_GATE')
                with sample_lock:samples.append(sample)
                wave_sampled=True
            release_ready()
            if time.monotonic_ns()-start>30_000_000_000:raise TimeoutError('Replay stalled; preserve trial, no silent retry')
    proc.wait(timeout=10);stop.set();sampler.join();stderr=proc.stderr.read();save(folder/'samples.json',samples)
    expected=normalized(capsule['original_result'])
    session_results=[]
    for sid,s in states.items():
        same=s['result']is not None and normalized(s['result'])==expected
        if not same:issues.append({'session':sid,'reason':'Result differs from actual original B result'})
        if s['next']!=len(capsule['calls']):issues.append({'session':sid,'reason':'Recorded calls not fully replayed','returned':s['next'],'expected':len(capsule['calls'])})
        if normalized(s['callbacks'],'outputs')!=normalized(schedule['original_callbacks'],'outputs'):issues.append({'session':sid,'reason':'Actual callback contents differ from recorded B'})
        if s['failures']!=capsule['original_failures']:issues.append({'session':sid,'reason':'Failure contract differs','actual':s['failures']})
        session_results.append({'session':sid,'matches_original':same,'result':s['result'],'failures':s['failures'],'callbacks':s['callbacks'],'payloads':s['payloads']})
    peak=max(samples,key=lambda x:x['sum_bytes']);first_both=next((s for s in samples if len(s['processes'])==2),None)
    out={'phase':phase,'mode':mode,'episode':episode,'rep':rep,'N':n,'binary_sha256':prehash,'postrun_binary_sha256':sha(BINARY),'capsule_sha256':sha(ROOT/'inputs'/f'{episode}-r{rep}.json'),'actual_calls_replayed':calls,'expected_calls':len(capsule['calls'])*n,'API_calls':0,'same_sample_peak':peak,'initial_python':baseline,'first_simultaneous':first_both,'rss_samples':len(samples),'max_actual_sample_gap_ms':max(((b['start_ns']-a['start_ns'])/1e6 for a,b in zip(sorted(samples,key=lambda s:s['start_ns']),sorted(samples,key=lambda s:s['start_ns'])[1:])),default=None),'allocator_requested_peak_bytes':heap_peak,'returncode':proc.returncode,'stderr':stderr,'issues':issues,'functional_equal_to_original':not issues and proc.returncode==0,'sessions':session_results,'memory_scope':'Fresh Python actual-response replay+Rust sampled RSS; no HTTP/TLS/cloud or product UI. Requested allocator bytes are not RSS. Fixed first-call observation gate is instrumentation, replay elapsed is not QA01/QA03.'}
    save(folder/'result.json',out);return out

def run(phase):
    plan=read(ROOT/'plan.json');jobs=[]
    for episode in plan['episodes']:
        for n in plan['N']:
            for rep in plan['replay_seeds']:
                for mode in (['B','B1']if rep%2 else ['B1','B']):jobs.append((mode,n,episode,rep))
    if phase=='PILOT':jobs=[('B',1,'02-E2',1),('B1',1,'02-E2',1),('B',2,'02-E4',3),('B1',2,'02-E4',3)]
    for mode,n,episode,rep in jobs:
        command=[sys.executable,__file__,'worker','--mode',mode,'--N',str(n),'--episode',episode,'--rep',str(rep),'--phase',phase]
        r=subprocess.run(command,capture_output=True,text=True);print(r.stdout.strip(),flush=True)
        if r.returncode:print(r.stderr,flush=True);raise RuntimeError('Preserve trial and inspect; no silent retry')
    print(json.dumps({'completed_trials':len(jobs),'phase':phase,'API_calls':0}))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['worker','pilot','run']);p.add_argument('--mode');p.add_argument('--N',type=int);p.add_argument('--episode');p.add_argument('--rep',type=int);p.add_argument('--phase',default='MEASURED_REPLAY');a=p.parse_args()
    if a.action=='worker':
        x=worker(a.mode,a.N,a.episode,a.rep,a.phase);print(json.dumps({k:x[k]for k in ['mode','episode','rep','N','functional_equal_to_original','actual_calls_replayed','allocator_requested_peak_bytes','issues']}))
    else:run('PILOT'if a.action=='pilot'else'MEASURED_REPLAY')
