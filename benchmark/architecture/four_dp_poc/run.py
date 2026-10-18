"""Execute Rust alternatives, actual cloud calls, DISPLAY_PROXY and macOS RSS evidence."""
import argparse,csv,hashlib,json,os,platform,subprocess,threading,time,uuid
from pathlib import Path
from bridge import Client,ModelError,dump,MODEL,PRICE
BASE=Path(__file__).resolve().parent
REPO=BASE.parents[2]
CRATE=REPO/'prototypes/candidates/four_dp_poc'
DEFAULT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-20261018'

def read(path):return json.loads(Path(path).read_text())
def digest(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def freeze(root):
    files=sorted([p for p in BASE.rglob('*') if p.is_file() and '__pycache__' not in str(p)]+[p for p in CRATE.rglob('*') if p.is_file() and 'target/' not in str(p)])
    manifest={'model':MODEL,'thinking':False,'prices':PRICE,'voice':'OFF','display':'DISPLAY_PROXY','repetitions':3,'budget_global_usd':5,'source':{str(p.relative_to(REPO)):digest(p) for p in files},'environment':{'platform':platform.platform(),'python':platform.python_version(),'rust':subprocess.check_output(['rustc','--version'],text=True).strip()},'claim_scope':'Authored finite fixtures; no product population or local Omni claims'}
    dump(root/'freeze.json',manifest);return manifest

def sampler(proc,stop,records):
    while not stop.is_set():
        at=time.monotonic_ns()
        p=subprocess.run(['ps','-o','pid=,rss=','-p',f'{os.getpid()},{proc.pid}'],capture_output=True,text=True)
        values={}
        for line in p.stdout.splitlines():
            cells=line.split()
            if len(cells)==2:values[cells[0]]=int(cells[1])*1024
        records.append({'at_ns':at,'rss_bytes_by_pid':values,'concurrent_sum_bytes':sum(values.values()),'method':'macOS ps RSS; decimal MB; shared-page deduplication unverified','scope':'Python bridge/runner + Rust candidate; browser UI excluded; cloud model excluded'})
        stop.wait(.1)

def execute(client,episode,variant,rep,phase='BASIC',tactic=None):
    case={**episode,'variant':variant}
    if tactic:case['tactic']=tactic
    run_id=f'{phase.lower()}-{episode["id"]}-{variant}-r{rep}-{uuid.uuid4().hex[:6]}'
    folder=client.root/'raw'/'runs'/run_id;folder.mkdir(parents=True)
    dump(folder/'input.json',case)
    t0=time.monotonic_ns();proc=subprocess.Popen([str(CRATE/'target/debug/four_dp_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1)
    proc.stdin.write(json.dumps(case,ensure_ascii=False)+'\n');proc.stdin.flush()
    events=[];calls=[];threads=[];write_lock=threading.Lock();memory=[];stop=threading.Event()
    memthread=threading.Thread(target=sampler,args=(proc,stop,memory),daemon=True);memthread.start()
    client.semaphore=threading.Semaphore(case.get('k',2))
    def model(msg):
        meta={'run_id':run_id,'phase':phase,'dp':case['dp'],'variant':variant,'episode':case['id'],'repetition':rep,'stage':msg['stage'],'rust_call_id':msg['call_id']}
        try:
            result,record=client.call(msg.get('payload',msg.get('request',{}).get('context',{})),meta);response={'call_id':msg['call_id'],'result':result,'usage':record.get('raw_usage')}
            if msg.get('wire_version')==2 and 'result' in response:response={'call_id':msg['call_id'],'provider_response':{'json':response['result'],'usage':{'input_tokens':record.get('input_tokens'),'output_tokens':record.get('output_tokens')}}}
        except ModelError as error:record=error.record;response={'call_id':msg['call_id'],'error':str(error)}
        except Exception as error:record={**meta,'status':'BRIDGE_ERROR','error':str(error)};response={'call_id':msg['call_id'],'error':str(error)}
        calls.append(record)
        with write_lock:
            try:proc.stdin.write(json.dumps(response,ensure_ascii=False)+'\n');proc.stdin.flush()
            except BrokenPipeError:record['discarded_after_process_end']=True
    for line in proc.stdout:
        try:msg=json.loads(line)
        except Exception:events.append({'type':'INVALID_STDOUT','line':line});continue
        msg['observed_ns']=time.monotonic_ns()
        if msg.get('type')=='model_call':
            thread=threading.Thread(target=model,args=(msg,),daemon=True);threads.append(thread);thread.start()
        else:events.append(msg)
    for thread in threads:thread.join()
    proc.wait();stop.set();memthread.join();stderr=proc.stderr.read()
    record={'run_id':run_id,'dp':case['dp'],'episode':case['id'],'variant':variant,'repetition':rep,'phase':phase,'tactic':tactic,'candidate_binary_sha256':digest(CRATE/'target/debug/four_dp_poc'),'freeze_sha256':digest(client.root/'freeze.json') if (client.root/'freeze.json').exists() else None,'start_ns':t0,'end_ns':time.monotonic_ns(),'events':events,'calls':calls,'memory':memory,'stderr':stderr,'returncode':proc.returncode,'input':str(folder/'input.json')}
    dump(folder/'trace.json',record)
    with (client.root/'runs.jsonl').open('a') as f:f.write(json.dumps(record,ensure_ascii=False)+'\n')
    return record

def main():
    p=argparse.ArgumentParser();p.add_argument('--results',default=str(DEFAULT));p.add_argument('--dp',choices=['01','02','04','05']);p.add_argument('--variant');p.add_argument('--episode');p.add_argument('--reps',type=int,default=3);p.add_argument('--single-repetition',type=int);p.add_argument('--phase',default='BASIC');p.add_argument('--tactic');p.add_argument('--freeze-only',action='store_true');p.add_argument('--skip-build',action='store_true');a=p.parse_args();root=Path(a.results);root.mkdir(parents=True,exist_ok=True)
    if a.dp=='05' and a.phase=='BASIC' and (root/'DP05-V2-PENDING').exists():
        print('DP05_BASIC_DEFERRED_TO_SEPARATE_V2_FREEZE',flush=True);raise SystemExit(75)
    if not a.skip_build:subprocess.run(['cargo','build','--offline','--manifest-path',str(CRATE/'Cargo.toml')],check=True)
    if a.freeze_only:freeze(root);print('Frozen '+str(root/'freeze.json'));return
    client=Client(root);dps=[a.dp] if a.dp else ['01','02','04','05']
    for dp in dps:
        episodes=read(BASE/'fixtures'/f'dp{dp}.json');variants=list('ABC' if dp in ('01','05') else 'AB')
        if a.variant:variants=[a.variant]
        for rep in ([a.single_repetition] if a.single_repetition else range(1,a.reps+1)):
            order=variants[(rep-1)%len(variants):]+variants[:(rep-1)%len(variants)]
            for episode in episodes:
                if a.episode and episode['id']!=a.episode:continue
                for variant in order:
                    record=execute(client,episode,variant,rep,a.phase,a.tactic)
                    budget=read(root/'budget.json');total=sum(v['amount_usd'] for v in budget['calls'].values())
                    error=any(e.get('kind')=='EPISODE_ERROR' for e in record['events'])
                    print(json.dumps({'run_id':record['run_id'],'calls':len(record['calls']),'status':'ERROR' if error else 'COMPLETED','ledger_total_usd':round(total,6),'total_calls':len(budget['calls'])}),flush=True)
                    if total>4.5:raise RuntimeError('Budget approaching $5; root review needed')
if __name__=='__main__':main()
