"""Serial, counterbalanced, fresh-process trials. No implicit API authorization."""
import argparse,json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('root',type=Path);p.add_argument('--phase',default='PRIMARY');a=p.parse_args()
auth=json.loads((a.root/'MODEL-CALLS-AUTHORIZED.json').read_text());plan=json.loads((a.root/'jobs.json').read_text())
completed=set()
if(a.root/'runs.jsonl').exists():
 for line in(a.root/'runs.jsonl').read_text().splitlines():
  r=json.loads(line);completed.add((r['episode'],r['variant'],r['repetition']))
for j in plan['jobs']:
 if(j['episode'],j['variant'],j['repetition'])in completed:continue
 subprocess.run([sys.executable,str(HERE/'run.py'),'--skip-build','--dp','02','--episode',j['episode'],'--variant',j['variant'],'--single-repetition',str(j['repetition']),'--phase',a.phase,'--results',str(a.root),'--single-trial-worker'],check=True)
 ledger=a.root.parent/'budget.json'
 if ledger.exists():
  x=json.loads(ledger.read_text());total=sum(v['amount_usd']for v in x['calls'].values())
  print(json.dumps({'completed':j,'budget_with_reservations':total}),flush=True)
