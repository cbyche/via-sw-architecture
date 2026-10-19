"""Bounded offline verification dispatcher. Does not patch source or invoke providers."""
import argparse,concurrent.futures,hashlib,json,subprocess,sys,time
from pathlib import Path
import orchestrate as o

def main():
 p=argparse.ArgumentParser();p.add_argument('--freeze',type=Path,required=True);p.add_argument('--digest',required=True);p.add_argument('--workers',type=int,default=2);a=p.parse_args()
 if not 1<=a.workers<=2:raise ValueError('At most2 offline verification workers')
 o.baseline(a.freeze,a.digest)
 dispatch=o.RESULT/'dispatcher'/time.strftime('%Y%m%dT%H%M%S');dispatch.mkdir(parents=True,exist_ok=False);jobs=[];skipped=[]
 for cfg in o.CONFIGS:
  for task in o.TASKS:
   taskroot=o.RESULT/cfg/task;pointer=o.read(taskroot/'latest-attempt.json');verification=Path(pointer.get('verification','/unavailable'));summary=verification/'summary.json'
   if pointer['status']=='PASS'and summary.exists()and o.read(verification/'status.json')['status']=='PASS'and o.read(summary)['baseline_freeze_sha256']==a.digest:skipped.append([cfg,task]);continue
   if pointer['status']!='PATCHED_NOT_VERIFIED':raise RuntimeError('Review non-pristine pending verification before resuming: '+cfg+'/'+task+' '+pointer['status'])
   jobs.append((cfg,task))
 o.dump(dispatch/'plan.json',{'jobs':jobs,'already_verified_skipped':skipped,'freeze_sha256':a.digest,'workers':a.workers,'provider_calls':0,'dispatcher_sha256':o.sha(Path(__file__)),'orchestrator_sha256':o.sha(Path(o.__file__))})
 def run(job):
  cfg,task=job;cmd=[sys.executable,str(o.HERE/'orchestrate.py'),'verify','--configuration',cfg,'--task',task,'--driver',str(o.HERE/'driver.py'),'--freeze',str(a.freeze),'--digest',a.digest];start=time.time();r=subprocess.run(cmd,capture_output=True,text=True);row={'configuration':cfg,'task':task,'args':cmd,'start_unix':start,'end_unix':time.time(),'returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr};o.dump(dispatch/f'{cfg}-{task}.json',row);return row
 remaining=iter(jobs);finished=[];failed=False
 with concurrent.futures.ThreadPoolExecutor(max_workers=a.workers)as pool:
  pending={}
  for _ in range(a.workers):
   job=next(remaining,None)
   if job:pending[pool.submit(run,job)]=job
  while pending:
   done,_=concurrent.futures.wait(pending,return_when=concurrent.futures.FIRST_COMPLETED)
   for future in done:
    job=pending.pop(future);row=future.result();finished.append(row);failed=failed or row['returncode']!=0
    print(json.dumps({'configuration':row['configuration'],'task':row['task'],'status':'PASS'if row['returncode']==0 else'FAIL','finished':len(finished),'planned':len(jobs),'elapsed_seconds':round(row['end_unix']-row['start_unix'],2)}),flush=True)
   if not failed:
    while len(pending)<a.workers:
     job=next(remaining,None)
     if not job:break
     pending[pool.submit(run,job)]=job
 o.dump(dispatch/'summary.json',{'status':'FAIL_STOPPED'if failed else'PASS','finished':len(finished),'planned':len(jobs),'already_verified':len(skipped),'failed_jobs':[[r['configuration'],r['task']]for r in finished if r['returncode']],'provider_calls':0})
 o.summarize(a)
 if failed:raise SystemExit(1)
if __name__=='__main__':main()
