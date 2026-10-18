"""DP01 actual-response replay, native resource tradeoff, and explicit output-policy sensitivity."""
import argparse,json,subprocess,sys,shutil
from pathlib import Path
from experiment_common import *
HERE=Path(__file__).resolve().parent;RESULT=ROOT/'dp01'
def prepare():
 fixtures=read(OLD/'fixtures/dp01.json');case=next(e for e in fixtures if e['id']=='dp01-e3');(HERE/'inputs').mkdir(exist_ok=True)
 dump(HERE/'inputs/dp01-e3.json',case)
 oldplan=read(RESULT/'plan.json');dump(RESULT/'plan-v1-proposal.json',oldplan)
 oldplan['version']=2;oldplan['selection']['QA01_AB']='UNRESOLVED: common callback location does not remove input-size and complete-answer output differences; raw A/B342ms gap is not established structural rank';oldplan['selection']['QA01_C']='UNRESOLVED_WITH_OUTPUT_POLICY_EFFECT: source preparation before requests is costed; record inputs and COMPLETE frequency differ, raw11.995s retained';oldplan['selection']['selected']='A_CONDITIONAL_ON_COMPLETION_AND_REUSE; strict QA01-priority winner not proved'
 oldplan['tactic']['definition']='A3 optional 8192-byte view retention budget + original locator, compared AFTER required hard14000-byte single-passage contract repair. Reread normal source when required. No semantic compression.';oldplan['tactic']['hard_cap_fix']='REQUIRED_BASELINE_CORRECTION, not tactic performance';oldplan['executions']['boundary_scope']='existing report first48lines range/price/schedule8398bytes; basic hardcap14KB vs optionalA3 8KiB,12each; plus full80line hardcap correction regression';oldplan['fresh_sensitivity']={'count':18,'episode':'dp01-e3','variants':['A','B','C'],'policies':['ORIGINAL','ADOPTION_ONLY'],'repetitions':3,'interpretation':'ADOPTION_ONLY adds a common earlier adoption JSON then retains original select-or-complete flow. Different response policy; does not establish pure structural effect or replace BASIC. All extra calls/costs counted.','authorization':'root approved24fresh episodes combined with DP05; .85 localcap'}
 dump(RESULT/'plan.json',oldplan)
 schedule=[]
 for rep in range(1,4):
  for variant in (['A','B','C'] if rep%2 else ['C','B','A']):
   for policy in (['ORIGINAL','ADOPTION_ONLY'] if rep%2 else ['ADOPTION_ONLY','ORIGINAL']):schedule.append({'variant':variant,'repetition':rep,'policy':policy})
 dump(HERE/'schedule.json',schedule);freeze('dp01')
def worker(job):
 spec=read(HERE/'schedule.json')[job];case=read(HERE/'inputs/dp01-e3.json');case['first_response_policy']=spec['policy'];client=BoundedClient(RESULT,limit=1.50,ledger_root=ROOT)
 record=harness.execute(client,case,spec['variant'],spec['repetition'],'OUTPUT_POLICY_SENSITIVITY',spec['policy'])
 print(json.dumps({'job':job,'run_id':record['run_id'],'calls':len(record['calls']),'cost':sum(c.get('cost_usd') or 0 for c in record['calls']),'failed':sum(e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR'] for e in record['events'])}),flush=True)
def native():
 source=OLD/'sources/dp05/large-public-report-v1.txt';binary=HERE/'candidate/target/debug/view_boundary';records=[]
 for rep in range(1,13):
  for strict in ([False,True] if rep%2 else [True,False]):
   p=subprocess.run(['/usr/bin/time','-l',str(binary)],input=json.dumps({'path':str(source),'strict':strict,'lines':48}),capture_output=True,text=True)
   records.append({'repetition':rep,'tactic':'A3' if strict else 'hardcap_fixed_basic','returncode':p.returncode,'result':json.loads(p.stdout) if p.returncode==0 else None,'stderr_resource_measurement':p.stderr})
 dump(RESULT/'native/view-retention.json',{'source_sha256':sha(source),'input':'natural report first48lines; scope/prices/schedule sections','model_calls':0,'runs':records,'not_new_LLM_latency_or_accuracy':True})
 for strict in [False,True]:
  p=subprocess.run([str(binary)],input=json.dumps({'path':str(source),'strict':strict,'lines':80}),capture_output=True,text=True);dump(RESULT/'native'/('hardcap-full80-'+str(strict)+'.json'),{'returncode':p.returncode,'result':json.loads(p.stdout)})
def main():
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','worker','replay','native']);p.add_argument('--job',type=int);p.add_argument('--tactic',default='basic');a=p.parse_args()
 if a.action=='prepare':prepare()
 elif a.action=='worker':worker(a.job)
 elif a.action=='native':native()
 else:replay('dp01',a.job,a.tactic)
if __name__=='__main__':main()
