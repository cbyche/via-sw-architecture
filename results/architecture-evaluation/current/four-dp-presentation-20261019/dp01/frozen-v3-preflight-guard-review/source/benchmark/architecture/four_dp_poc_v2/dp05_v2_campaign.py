"""Run36 DP05 BASIC rows from the pre-DP05 fairness v2 freeze, after original84."""
import argparse,hashlib,json,subprocess,sys
from pathlib import Path
from bridge import dump,MODEL,PRICE
BASE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--results',type=Path,required=True);p.add_argument('--crate',type=Path,required=True);p.add_argument('--freeze-only',action='store_true');a=p.parse_args();root=a.results.resolve();crate=a.crate.resolve();folder=root/'basic-dp05-v2';folder.mkdir(exist_ok=True)
def lines(path):return [json.loads(x) for x in path.read_text().splitlines() if x] if path.exists() else []
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
budget=folder/'budget.json'
if not budget.exists():budget.symlink_to(root/'budget.json')
if budget.resolve()!=root/'budget.json':raise RuntimeError('DP05 v2 must share the global budget')
subprocess.run(['cargo','build','--offline','--manifest-path',str(crate/'Cargo.toml')],check=True)
files=sorted(p for p in crate.rglob('*') if p.is_file() and 'target/' not in str(p))
freeze={'version':'DP05_BASIC_V2_RECEIVER_SELECT_OR_COMPLETE','reason':json.loads((root/'dp05-v2-plan.json').read_text()),'base_freeze_sha256':sha(root/'freeze.json'),'source':{str(p.relative_to(crate)):sha(p) for p in files},'candidate_binary_sha256':sha(crate/'target/debug/four_dp_poc'),'fixture_sha256':sha(BASE/'fixtures/dp05.json'),'oracle_sha256':sha(BASE/'oracles/dp05.json'),'model':MODEL,'thinking':False,'price':PRICE,'episode_ids':[x['id'] for x in json.loads((BASE/'fixtures/dp05.json').read_text())],'variants':['A','B','C'],'repetitions':3,'phase':'BASIC','global_budget':str(root/'budget.json'),'receiver_protocol':'Same select-or-complete for ABC; completion needs supplied source-backed evidence, not manifest metadata; every repair/read call retained'}
if a.freeze_only:dump(folder/'freeze.json',freeze);print(str(folder/'freeze.json'));raise SystemExit(0)
old=lines(root/'runs.jsonl');basic=[r for r in old if r['phase']=='BASIC'];unique={(r['dp'],r['variant'],r['episode'],r['repetition']) for r in basic}
if len(basic)!=84 or len(unique)!=84 or any(r['dp']=='05' for r in basic):raise RuntimeError('DP05 v2 starts after exactly84 unique original DP01/02/04 BASIC rows and no DP05 BASIC')
if (folder/'freeze.json').exists() and json.loads((folder/'freeze.json').read_text())!=freeze:raise RuntimeError('Prepared DP05 v2 freeze changed; review before execution')
dump(folder/'freeze.json',freeze)
for rep in range(1,4):
    order=list('ABC')[(rep-1)%3:]+list('ABC')[:(rep-1)%3]
    for episode in json.loads((BASE/'fixtures/dp05.json').read_text()):
        for variant in order:
            cmd=[sys.executable,str(BASE/'tactic_worker.py'),'--crate',str(crate),'--skip-build','--results',str(folder),'--dp','05','--variant',variant,'--episode',episode['id'],'--single-repetition',str(rep),'--reps','1','--phase','BASIC']
            subprocess.run(cmd,check=True)
            for name,key in [('runs.jsonl','run_id'),('model-calls.jsonl','call_uuid')]:
                known={r[key] for r in lines(root/name)}
                with (root/name).open('a') as out:
                    for r in lines(folder/name):
                        if r[key] not in known:out.write(json.dumps(r,ensure_ascii=False)+'\n');known.add(r[key])
            count=sum(r['phase']=='BASIC' for r in lines(root/'runs.jsonl'));print(json.dumps({'basic_complete':count,'dp':'05','version':'v2','episode':episode['id'],'variant':variant,'rep':rep}),flush=True)
