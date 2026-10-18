"""C1 raw selected-source prefetch; C2 exact read serving, independent tactics."""
import argparse,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path.insert(0,str(HERE.parent/'dp01'))
from experiment_common import *
HERE=Path(__file__).resolve().parent
RESULT=ROOT/'dp05'
def prepare():
 case=next(e for e in read(OLD/'fixtures/dp05.json') if e['id']=='dp05-e3')
 for e in case['events']:
  if e['kind']=='adopt':e['state']['condition_source_refs']=[{'source_id':'delivered-guidance','revision':1,'start_line':1,'end_line':2}]
 dump(HERE/'inputs/dp05-e3.json',case)
 schedule=[{'variant':'C','tactic':t,'repetition':r} for r in range(1,4) for t in (['NONE','C1'] if r%2 else ['C1','NONE'])]
 dump(HERE/'schedule.json',schedule);freeze('dp05')
def worker(job):
 spec=read(HERE/'schedule.json')[job];case=read(HERE/'inputs/dp05-e3.json');client=BoundedClient(RESULT,limit=1.50,ledger_root=ROOT)
 record=harness.execute(client,case,'C',spec['repetition'],'PREFETCH_TACTIC',None if spec['tactic']=='NONE' else 'C1')
 print(json.dumps({'job':job,'run_id':record['run_id'],'calls':len(record['calls']),'cost':sum(c.get('cost_usd') or 0 for c in record['calls']),'failed':sum(e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR'] for e in record['events'])}),flush=True)
def main():
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','worker','replay']);p.add_argument('--job',type=int);p.add_argument('--tactic',default='basic');a=p.parse_args()
 if a.action=='prepare':prepare()
 elif a.action=='worker':worker(a.job)
 else:replay('dp05',a.job,a.tactic)
if __name__=='__main__':main()
