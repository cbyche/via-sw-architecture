"""Fresh DP04 A/B then A2+A5/B4, immutable phase pins and original metric code."""
import argparse,collections,hashlib,json,os,shutil,subprocess,sys,time,fcntl
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
OLD=REPO/'benchmark/architecture/four_dp_poc_v2'
sys.path.insert(0,str(OLD))
import run as harness
from bridge import Client,credential,MODEL,dump
from coordinate_views import render,map_extents
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019'
RESULT=ROOT/'dp04'
CRATE=HERE/'candidate'
PYTHON='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(p):return json.loads(Path(p).read_text())

class BoundedClient(Client):
    def budget(self,id,amount,state):
        with (RESULT/'budget.json').open('a+') as f:
            fcntl.flock(f,fcntl.LOCK_EX);f.seek(0);text=f.read();data=json.loads(text) if text else {'limit_usd':.55,'calls':{}}
            if state=='RESERVED' and sum(x['amount_usd'] for x in data['calls'].values())+amount>.55:raise RuntimeError('DP04 $0.55 budget stop')
            super().budget(id,amount,state)
            data['calls'][id]={'amount_usd':amount,'state':state};f.seek(0);f.truncate();json.dump(data,f);f.flush();os.fsync(f.fileno())
    def call(self,payload,meta):
        payload=dict(payload);revision=payload.pop('coordinate_view_request',None);views=[];timing=None
        if revision:
            views,timing=render(revision,self.root/'raw'/'coordinate-views'/meta['run_id']/str(revision['revision']))
            payload['images']=[*payload.get('images',[]),*[v['path'] for v in views]]
            payload['input']={**payload['input'],'coordinate_views':views}
            payload['instruction']+=' For this bounded local-repair call, output each repair_expression_id exactly once rather than all original expressions; host preserves and revalidates unaffected entries. If full_rejudgment:true is necessary, instead output all original expressions within this same call. Coordinate-view images are mechanical crops with original-coordinate grid labels; they contain no inferred targets. Their view_to_original maps local pixel coordinates. Prefer original frame pixel bboxes. If returning a local crop bbox, add coordinate_space:"view_pixels" and view_id to that extent; host applies the declared affine transform and rejects out-of-crop boxes without clamping. Unmarked bboxes remain original frame pixels.'
        value,record=super().call(payload,meta)
        if views:
            record['coordinate_view_render']=timing;record['coordinate_views']=views
            try:value,mapping=map_extents(value,views);record['coordinate_mappings']=mapping
            except ValueError as error:
                # Preserve provider success/usage; return explicit invalid original
                # geometry for the existing candidate gate instead of repairing it.
                record['coordinate_mapping_error']=str(error)
                for adoption in value.get('adoptions',[]):
                    for extent in adoption.get('extents',[]):extent['bbox']=[-1,-1,-1,-1]
        return value,record

def prepare():
    RESULT.mkdir(parents=True,exist_ok=True)
    plan=read(RESULT/'plan.json')
    if (RESULT/'basic/freeze.json').exists():raise RuntimeError('Already frozen; no replacement')
    build=subprocess.run(['cargo','build','--offline','--manifest-path',str(CRATE/'Cargo.toml')],capture_output=True,text=True)
    dump(RESULT/'build.json',{'returncode':build.returncode,'stdout':build.stdout,'stderr':build.stderr})
    if build.returncode:raise RuntimeError('Build failed; preserved log')
    fixtures=read(OLD/'fixtures/dp04.json');jobs={}
    for name,variants in [('basic',['A','B']),('tactics',['A2A5','B4'])]:
        phase=RESULT/name;phase.mkdir()
        files=[p for p in OLD.rglob('*') if p.is_file() and '__pycache__' not in p.parts]
        files += [p for p in HERE.rglob('*') if p.is_file() and 'target' not in p.parts and '__pycache__' not in p.parts]
        for source in files:
            dest=phase/'frozen/source'/source.relative_to(REPO);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,dest)
        binary=phase/'frozen/bin/four_dp_poc';binary.parent.mkdir(parents=True);shutil.copy2(CRATE/'target/debug/four_dp_poc',binary)
        manifest={'source':{str(p.relative_to(REPO)):sha(p) for p in files},'binary':{'path':str(binary),'sha256':sha(binary)},'model':MODEL,'thinking':False,'temperature':0,'scope':'Fresh matched DP04; unchanged v2 metric/evaluator code; A/B/B4 source paths same; A2A5 explicit recovery tactic','plan_sha256':sha(RESULT/'plan.json')}
        dump(phase/'freeze.json',manifest)
        schedule=[]
        for rep in range(1,4):
            order=variants if rep%2 else list(reversed(variants))
            for episode in fixtures:
                schedule.extend({'dp':'04','episode':episode['id'],'variant':v,'repetition':rep} for v in order)
        dump(phase/'schedule.json',schedule)
        dump(phase/'MODEL-CALLS-AUTHORIZED.json',{'phase':'BASIC' if name=='basic' else 'TACTIC','campaign_root':str(ROOT),'allowed_jobs':schedule,'freeze_sha256':sha(phase/'freeze.json'),'parent_authorization':'48 fresh DP04 trials, local $0.55, global $1.50, original failure/metric contract retained'})
        jobs[name]=len(schedule)
    print(json.dumps({'prepared':jobs,'binary_sha256':sha(CRATE/'target/debug/four_dp_poc')}))

def worker(phase,job,offline=False):
    if offline:phase=RESULT/'offline'/phase.name
    spec=read(phase/'schedule.json')[job];auth=read(phase/'MODEL-CALLS-AUTHORIZED.json')
    if sha(phase/'freeze.json')!=auth['freeze_sha256']:raise RuntimeError('Authorization freeze mismatch')
    fixture=next(e for e in read(OLD/'fixtures/dp04.json') if e['id']==spec['episode'])
    if offline:
        class Probe(harness.OfflineClient):
            def call(self,payload,meta):
                import offline_dp04
                # Test-only service gets authored-source geometry; live bridge
                # never imports this module or reads authoring SVG.
                p=dict(payload);revision=p.pop('coordinate_view_request',None)
                if revision:
                    views,_=render(revision,phase/'offline-views'/meta['run_id']);p['images']=[*p.get('images',[]),*[v['path'] for v in views]]
                result=offline_dp04.respond(p,{**meta,'dp04_probe_bad_initial':True})
                return result,{'call_uuid':'OFFLINE','stage':meta['stage'],'status':'OFFLINE','cost_usd':0,'input_tokens':0,'output_tokens':0}
        client=Probe(phase)
    else:client=BoundedClient(phase,limit=1.50,ledger_root=ROOT)
    record=harness.execute(client,fixture,spec['variant'],spec['repetition'],'OFFLINE' if offline else auth['phase'],tactic='A2+A5' if spec['variant']=='A2A5' else 'B4' if spec['variant']=='B4' else None)
    if offline and any(e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR'] for e in record['events']):raise RuntimeError('Offline structural acceptance failed; preserved trace')
    print(json.dumps({'run_id':record['run_id'],'calls':len(record['calls']),'failed_events':sum(e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR'] for e in record['events'])}),flush=True)

def run_phase(name):
    phase=RESULT/name;schedule=read(phase/'schedule.json')
    if name=='tactics' and len((RESULT/'basic/runs.jsonl').read_text().splitlines())!=24:raise RuntimeError('Finish all24 basic first')
    done={}
    if (phase/'runs.jsonl').exists():
        done={(r['episode'],r['variant'],r['repetition']):r for r in map(json.loads,(phase/'runs.jsonl').read_text().splitlines())}
    for index,job in enumerate(schedule):
        key=(job['episode'],job['variant'],job['repetition'])
        if key in done:continue
        command=[PYTHON,str(__file__),'worker','--phase',name,'--job',str(index)]
        result=subprocess.run(command,capture_output=True,text=True)
        with (phase/'orchestration.log').open('a') as f:f.write(result.stdout+result.stderr)
        print(result.stdout.strip(),flush=True)
        if result.returncode:raise RuntimeError('Worker infrastructure failed; no silent rerun')
        last=json.loads((phase/'runs.jsonl').read_text().splitlines()[-1])
        if any(c.get('http_status') not in (None,200) for c in last['calls']):raise RuntimeError('Provider endpoint/model HTTP failure retained; stop phase rather than spending on repeated invalid requests')
    import analyze
    analyze.analyze(phase)

def main():
    p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','run','worker','preflight']);p.add_argument('--phase',choices=['basic','tactics']);p.add_argument('--job',type=int);p.add_argument('--offline',action='store_true');a=p.parse_args()
    if a.action=='prepare':prepare()
    elif a.action=='run':run_phase(a.phase)
    elif a.action=='worker':worker(RESULT/a.phase,a.job,a.offline)
    else:
        # Only availability, no credential bytes, workspace hostname or URL in
        # outputs. First scheduled paid call verifies actual provider model.
        present=bool(credential('via-dashscope-api-key')) and bool(credential('via-dashscope-workspace-id'))
        print(json.dumps({'credentials_available':present,'model':MODEL,'thinking':False,'temperature':0,'provider_route':'configured DashScope Singapore compatible endpoint','calls':0,'model_existence_network_check':'first scheduled call; no uncounted smoke'}))
if __name__=='__main__':main()
