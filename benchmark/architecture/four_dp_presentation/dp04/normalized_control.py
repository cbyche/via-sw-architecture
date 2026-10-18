"""Approved common A/B normalized-output sensitivity. No A2/A5/B4 tactics."""
import argparse,copy,fcntl,hashlib,json,os,platform,shutil,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import control
from control import BoundedClient,ROOT,RESULT,OLD,REPO,PYTHON,sha,read,dump,harness
PHASE=RESULT/'normalized'
CRATE=HERE/'normalized_candidate'

class NormalizedClient(BoundedClient):
    def budget(self,id,amount,state):
        with (PHASE/'budget.json').open('a+') as f:
            fcntl.flock(f,fcntl.LOCK_EX);f.seek(0);text=f.read();data=json.loads(text) if text else {'limit_usd':.20,'calls':{}}
            if state=='RESERVED' and sum(x['amount_usd'] for x in data['calls'].values())+amount>.20:raise RuntimeError('Common normalized24 $0.20 budget stop')
            super().budget(id,amount,state)
            data['calls'][id]={'amount_usd':amount,'state':state};f.seek(0);f.truncate();json.dump(data,f);f.flush();os.fsync(f.fileno())

def offline(probe='normal'):
    """Actual Rust wire/gate acceptance; mock geometry never reaches live client."""
    import offline_dp04
    folder=RESULT/('normalized-offline-'+probe);folder.mkdir(parents=True,exist_ok=True)
    binary=folder/'frozen/bin/four_dp_poc';binary.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(CRATE/'target/debug/four_dp_poc',binary)
    dump(folder/'freeze.json',{'binary':{'path':str(binary.resolve()),'sha256':sha(binary)}})
    class Probe(harness.OfflineClient):
        def call(self,payload,meta):
            # The old test-only binder expects pixel boxes. Preserve candidate
            # input bytes; adapt only its private copy so malformed normalized
            # proposals reach the real host gate rather than crashing the mock.
            payload=copy.deepcopy(payload)
            if 'candidate_binding' in meta['stage']:
                for c in payload['input']['screen']['candidates']:
                    if 'bbox' not in c:c['bbox']=c.get('bbox_2d',[-1,-1,-1,-1])
            value=offline_dp04.respond(payload,{**meta,'dp04_probe_bad_initial':probe=='normal'})
            if 'joint' in meta['stage']:
                frames=payload['input']['original']['observations']['frames'];extents=[c for a in value['adoptions'] for c in a.get('extents',[])]
            elif 'screen' in meta['stage']:
                frames=payload['input']['frames'];extents=value['candidates']
            else:frames=[];extents=[]
            for c in extents:
                frame=next(f for f in frames if f['frame_id']==c['frame_id']);b=c.pop('bbox')
                c['bbox_2d']=[round(b[0]/frame['width']*1000),round(b[1]/frame['height']*1000),round(b[2]/frame['width']*1000),round(b[3]/frame['height']*1000)];c['coordinate_space']='qwen_normalized_0_1000'
                if probe!='normal' and 'screen' in meta['stage']:
                    c['bbox']=b
                    if probe=='reject_pixel':c.pop('bbox_2d');c['coordinate_space']='frame_pixels'
            return value,{'call_uuid':'OFFLINE-'+os.urandom(8).hex(),'stage':meta['stage'],'status':'OFFLINE','cost_usd':0,'input_tokens':0,'output_tokens':0}
    records=[]
    for episode in read(OLD/'fixtures/dp04.json'):
        for variant in (['A','B'] if probe=='normal' else ['B']):
            x=harness.execute(Probe(folder),episode,variant,1,'OFFLINE_NORMALIZED')
            complete=[e for e in x['events'] if e.get('kind')=='GOAL_COMPLETE'];failure=[e for e in x['events'] if e.get('kind') in ['EPISODE_ERROR','GOAL_FAILED']]
            ready=[e for e in x['events'] if e.get('kind')=='ADOPTION' and e.get('data',{}).get('targets')]
            expanded=sum(e.get('kind')=='PRODUCER_EXPANSION_REQUEST' for e in x['events']);rejected=sum(e.get('kind')=='NORMALIZED_WIRE_REJECTED' for e in x['events'])
            passed=(bool(complete) and not failure) if probe=='normal' else (bool(failure) and not complete and not ready and expanded>=1 and rejected>=2)
            records.append({'run_id':x['run_id'],'variant':variant,'episode':episode['id'],'calls':len(x['calls']),'source_expansion_or_repair':sum(e.get('kind') in ['BOUNDED_SOURCE_REREAD','PRODUCER_EXPANSION_REQUEST'] for e in x['events']),'normalization_events':sum(e.get('kind')=='COORDINATES_NORMALIZED' for e in x['events']),'wire_rejections':rejected,'ready_adoptions':len(ready),'pass':passed})
            dump(RESULT/('normalized-offline-acceptance.json' if probe=='normal' else 'normalized-offline-'+probe+'-acceptance.json'),{'executions':records,'model_calls':0,'probe':probe,'scope':'Test-only authored-geometry service; actual Rust wire/recovery/gate exercised; not quality measurements'})
            if not records[-1]['pass']:raise RuntimeError('Offline normalized acceptance failed; raw retained')
    print(json.dumps({'executions':len(records),'all_pass':True,'model_calls':0}))

def prepare():
    if PHASE.exists():raise RuntimeError('Use a new immutable phase; refuse replacement')
    plan=read(RESULT/'normalized-plan.json')
    if not plan['approved'] or not all(x['pass'] for name in ['normalized-offline-acceptance.json','normalized-offline-reject_pixel-acceptance.json','normalized-offline-reject_duplicate-acceptance.json'] for x in read(RESULT/name)['executions']):raise RuntimeError('Plan/offline acceptance missing')
    PHASE.mkdir();files=[p for p in OLD.rglob('*') if p.is_file() and '__pycache__' not in p.parts]
    files += [p for p in HERE.rglob('*') if p.is_file() and 'target' not in p.parts and '__pycache__' not in p.parts]
    for source in files:
        dest=PHASE/'frozen/source'/source.relative_to(REPO);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,dest)
    binary=PHASE/'frozen/bin/four_dp_poc';binary.parent.mkdir(parents=True);shutil.copy2(CRATE/'target/debug/four_dp_poc',binary)
    schedule=[]
    for rep in range(1,4):
        for episode in read(OLD/'fixtures/dp04.json'):
            schedule.extend({'dp':'04','episode':episode['id'],'variant':v,'repetition':rep} for v in (['A','B'] if rep%2 else ['B','A']))
    dump(PHASE/'schedule.json',schedule)
    manifest={'source':{str(p.relative_to(REPO)):sha(p) for p in files},'binary':{'path':str(binary.resolve()),'sha256':sha(binary)},'model':'qwen3.7-plus-2026-05-26','thinking':False,'temperature':0,'coordinate_wire':'bbox_2d integer0..1000 original frame, round nearest ties even to original pixel; no clamp or unit inference','plan_sha256':sha(RESULT/'normalized-plan.json'),'schedule_sha256':sha(PHASE/'schedule.json'),'phase':'BASIC_NORMALIZED','environment':{'python':platform.python_version(),'python_executable':sys.executable,'platform':platform.platform(),'rustc':subprocess.check_output(['rustc','--version'],text=True).strip()},'rates':'fixed comparison input$0.40/output$1.60 per1M actual usage, not invoice/current-price claim','scope':'Common representation sensitivity; no A2/A5/B4; old pixel outputs and scores immutable'}
    dump(PHASE/'freeze.json',manifest);dump(PHASE/'MODEL-CALLS-AUTHORIZED.json',{'phase':'BASIC_NORMALIZED','allowed_jobs':schedule,'freeze_sha256':sha(PHASE/'freeze.json'),'local_additional_cap':.20,'dp04_total_cap':.55,'parent_shared_cap':1.50})
    print(json.dumps({'jobs':len(schedule),'freeze_sha256':sha(PHASE/'freeze.json'),'binary_sha256':sha(binary)}))

def worker(index):
    auth=read(PHASE/'MODEL-CALLS-AUTHORIZED.json');manifest=read(PHASE/'freeze.json')
    if sha(PHASE/'freeze.json')!=auth['freeze_sha256'] or sha(PHASE/'schedule.json')!=manifest['schedule_sha256']:raise RuntimeError('Frozen authorization/schedule mismatch')
    job=read(PHASE/'schedule.json')[index]
    if job not in auth['allowed_jobs']:raise RuntimeError('Unplanned job')
    fixture=next(e for e in read(OLD/'fixtures/dp04.json') if e['id']==job['episode'])
    x=harness.execute(NormalizedClient(PHASE,limit=1.50,ledger_root=ROOT),fixture,job['variant'],job['repetition'],'BASIC_NORMALIZED')
    print(json.dumps({'run_id':x['run_id'],'calls':len(x['calls']),'failed_events':sum(e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR'] for e in x['events'])}),flush=True)

def run():
    done=set()
    if (PHASE/'runs.jsonl').exists():done={(r['episode'],r['variant'],r['repetition']) for r in map(json.loads,(PHASE/'runs.jsonl').read_text().splitlines())}
    for i,job in enumerate(read(PHASE/'schedule.json')):
        if (job['episode'],job['variant'],job['repetition']) in done:continue
        c=subprocess.run([PYTHON,str(__file__),'worker','--job',str(i)],capture_output=True,text=True)
        with (PHASE/'orchestration.log').open('a') as f:f.write(c.stdout+c.stderr)
        print(c.stdout.strip(),flush=True)
        if c.returncode:raise RuntimeError('Infrastructure failure retained; no hidden rerun')
        x=json.loads((PHASE/'runs.jsonl').read_text().splitlines()[-1])
        if any(c.get('http_status') not in (None,200) for c in x['calls']):raise RuntimeError('Provider HTTP failure retained; inspect before resuming unexecuted jobs')
    import analyze
    analyze.analyze(PHASE)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['offline','prepare','run','worker']);p.add_argument('--job',type=int);p.add_argument('--probe',choices=['normal','reject_pixel','reject_duplicate'],default='normal');a=p.parse_args()
    {'offline':lambda:offline(a.probe),'prepare':prepare,'run':run,'worker':lambda:worker(a.job)}[a.action]()
