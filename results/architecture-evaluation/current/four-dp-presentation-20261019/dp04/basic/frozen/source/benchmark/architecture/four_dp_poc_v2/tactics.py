"""Bounded selected tactics after BASIC120, with one GLOBAL budget and separate freezes."""
import argparse,hashlib,json,subprocess,sys
from pathlib import Path
from bridge import dump,MODEL,PRICE
BASE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--results',type=Path,required=True);p.add_argument('--plan',type=Path,required=True);a=p.parse_args();root=a.results.resolve();plan=json.loads(a.plan.read_text())
def loadlines(path):return [json.loads(x) for x in path.read_text().splitlines() if x] if path.exists() else []
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
basic=[r for r in loadlines(root/'runs.jsonl') if r['phase']=='BASIC'];unique={(r['dp'],r['variant'],r['episode'],r['repetition']) for r in basic}
if len(basic)!=120 or len(unique)!=120:raise RuntimeError('Tactics wait for exactly120 unique completed BASIC episode rows, including failures')
dump(root/'selected-tactic-plan.json',plan)
for item in plan:
    phase=item.get('phase','TACTIC');dp=item['dp'];crate=Path(item['crate']).resolve();label=item['tactic'];episodes=item.get('episodes',[item.get('episode')]);folder=root/'tactics'/f'dp{dp}-{label}';folder.mkdir(parents=True,exist_ok=True)
    budget=folder/'budget.json'
    if not budget.exists():budget.symlink_to(root/'budget.json')
    if budget.resolve()!=root/'budget.json':raise RuntimeError('Tactic must share the global budget file')
    subprocess.run(['cargo','build','--offline','--manifest-path',str(crate/'Cargo.toml')],check=True)
    files=sorted(p for p in crate.rglob('*') if p.is_file() and 'target/' not in str(p));freeze={'phase':phase,'selection':item,'repetitions':3,'model':MODEL,'thinking':False,'price':PRICE,'baseline_candidate_freeze_sha256':sha(root/'basic-dp05-v2/freeze.json') if dp=='05' else sha(root/'freeze.json'),'runtime_tools_sha256':{name:sha(BASE/name) for name in ['run.py','bridge.py','tactic_worker.py','tactics.py']},'base_fixture_freeze_sha256':sha(root/'freeze.json'),'scratch_sources':{str(p.relative_to(crate)):sha(p) for p in files},'binary_sha256':sha(crate/'target/debug/four_dp_poc'),'affected_fixtures':{e:sha(BASE/'fixtures'/f'dp{dp}.json') for e in episodes},'comparison_population':'Only these same affected episodes, baseline vs tactic, three individual repetitions; no whole-DP extrapolation','global_budget':str(root/'budget.json')};
    if (folder/'freeze.json').exists():
        preparation=folder/'preparation-freeze.json'
        if not preparation.exists():preparation.write_bytes((folder/'freeze.json').read_bytes())
        freeze['preparation_freeze_sha256']=sha(preparation)
    dump(folder/'freeze.json',freeze)
    for rep in range(1,4):
        for episode in episodes:
            cmd=[sys.executable,str(BASE/'tactic_worker.py'),'--crate',str(crate),'--skip-build','--results',str(folder),'--dp',dp,'--variant',item.get('variant',item['base_variant']),'--episode',episode,'--single-repetition',str(rep),'--reps','1','--phase',phase,'--tactic',label]
            subprocess.run(cmd,check=True)
            for name,key in [('runs.jsonl','run_id'),('model-calls.jsonl','call_uuid')]:
                existing={r[key] for r in loadlines(root/name)}
                with (root/name).open('a') as out:
                    for record in loadlines(folder/name):
                        if record[key] not in existing:out.write(json.dumps(record,ensure_ascii=False)+'\n');existing.add(record[key])
            print(json.dumps({'tactic':label,'dp':dp,'episode':episode,'repetition':rep,'status':'RECORDED','global_budget_usd':sum(x['amount_usd'] for x in json.loads((root/'budget.json').read_text())['calls'].values())}),flush=True)
