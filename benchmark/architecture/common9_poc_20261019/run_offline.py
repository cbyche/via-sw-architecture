"""Fresh-process common9×7 deterministic regression, no API or real parser."""
import argparse,concurrent.futures,json,subprocess,sys,time
from pathlib import Path
import harness
from bridge import dump

def main():
 p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);p.add_argument('--workers',type=int,default=2);p.add_argument('--time-scale',type=float,default=.05);a=p.parse_args()
 assert 1<=a.workers<=2 and 0<a.time_scale<=1
 a.root.mkdir(parents=True,exist_ok=False)
 episodes=[e['id']for e in harness.read(harness.H/'fixtures.json')];jobs=[{'configuration':c,'episode':e,'rep':1}for e in episodes for c in harness.CONFIGS];dump(a.root/'planned-jobs.json',jobs)
 def execute(job):
  child=a.root/'episodes'/(job['configuration']+'-'+job['episode']);cmd=[sys.executable,str(harness.H/'harness.py'),'--root',str(child),'--configuration',job['configuration'],'--episode',job['episode'],'--rep','1','--offline','--time-scale',str(a.time_scale)]
  begin=time.monotonic_ns();run=subprocess.run(cmd,capture_output=True,text=True);end=time.monotonic_ns();dump(child/'command.json',{'command':cmd,'start_ns':begin,'end_ns':end,'returncode':run.returncode,'stdout':run.stdout,'stderr':run.stderr})
  traces=list((child/'raw/runs').glob('*/trace.json'));return {**job,'returncode':run.returncode,'trace':str(traces[0])if len(traces)==1 else None,'command':str(child/'command.json')}
 rows=[]
 with concurrent.futures.ThreadPoolExecutor(max_workers=a.workers)as pool:
  for row in pool.map(execute,jobs):
   rows.append(row)
   if row['trace']:
    trace=harness.read(row['trace'])
    with(a.root/'runs.jsonl').open('a')as f:f.write(json.dumps(trace,ensure_ascii=False)+'\n')
   dump(a.root/'progress.json',{'completed':len(rows),'planned':len(jobs),'actual_API_calls':0,'rows':rows})
   print(json.dumps({'completed':len(rows),'planned':len(jobs),**row}),flush=True)
 dump(a.root/'summary.json',{'status':'PROCESSES_FINISHED_EVALUATOR_PENDING','completed':len(rows),'planned':len(jobs),'actual_API_calls':0,'rows':rows,'scope':'Deterministic/scaled functional regression, fresh Python/Rust per episode; no paid model/helper inference or performance comparison'})

if __name__=='__main__':main()
