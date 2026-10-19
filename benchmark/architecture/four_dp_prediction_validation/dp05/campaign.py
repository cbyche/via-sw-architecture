"""Sequential fresh-process scheduled primary trials; explicit API window required."""
import json,subprocess,sys,time
from control import HERE,RESULT,read,sha
if not(RESULT/'API-WINDOW.json').exists():raise RuntimeError('Root API window absent')
if read(RESULT/'API-WINDOW.json')['freeze_sha256']!=sha(RESULT/'freeze.json'):raise RuntimeError('Root API window not bound to this freeze')
completed=set()
if(RESULT/'runs.jsonl').exists():
 for line in(RESULT/'runs.jsonl').read_text().splitlines():
  r=json.loads(line)
  if r['phase']=='PREDICTION_VALIDATION':completed.add((r['episode'],r['variant'],r['repetition']))
for index,spec in enumerate(read(HERE/'schedule.json')):
 if(spec['episode'],spec['variant'],spec['repetition'])in completed:continue
 cmd=[sys.executable,str(HERE/'control.py'),'worker','--index',str(index)]
 start=time.time();p=subprocess.run(cmd,capture_output=True,text=True)
 with(RESULT/'commands.jsonl').open('a')as f:f.write(json.dumps({'command':cmd,'start_unix':start,'end_unix':time.time(),'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})+'\n')
 print(p.stdout.strip(),flush=True)
 if p.returncode:raise RuntimeError('Worker failed; preserved command stderr, do not silently retry: '+p.stderr)
 ledger=read(RESULT.parent/'dp01-dp05-budget.json');total=sum(c['amount_usd']for c in ledger['calls'].values());print(json.dumps({'done':spec,'cost_plus_unknown_reservations_usd':total}),flush=True)
 last=json.loads((RESULT/'runs.jsonl').read_text().splitlines()[-1]);budget_stop=any('BUDGET'in c.get('error','').upper()for c in last['calls'])
 if total>=1.58 or budget_stop:raise RuntimeError('DP01/DP05 combined budget stop; obtain root scope/budget decision before more')
