"""Combine already-executed corrected-DP05 four-task acceptance; no inferred execution."""
import hashlib,json
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());R=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 c=R/'qa04/controls-reuse-union-corrected-v2/acceptance-summary.json';features={t:R/f'qa04/baseline-dp05-reuse-union-corrected-v2/dp05-{t}/summary.json'for t in ['C04','C06']};control=read(c);assert len(control['checks'])==6 and all(x['pass']for x in control['checks'])
 inputs=[c,*features.values()];options=[]
 for v in 'ABC':
  tasks=[]
  for x in control['changes']:
   assert any(y['task']==x['task']and y['variant']==v and y['pass']for y in control['checks']);tasks.append({'id':x['task'],'modules':x['changed_operational_modules'],'count':x['count'],'status':'PASS','evidence':str(c.resolve()),'evidence_sha256':sha(c)})
  for t,p in features.items():
   x=read(p);assert x['acceptance']=='PASS';tasks.append({'id':t,'modules':x['changed_modules_by_variant'][v],'count':x['changed_module_count_each_variant'][v],'status':'PASS','evidence':str(p.resolve()),'evidence_sha256':sha(p)})
  assert {x['id']for x in tasks}=={'C01','C03','C04','C06'}
  options.append({'dp':'05','variant':v,'tasks':tasks,'mean_changed_operational_modules':sum(t['count']for t in tasks)/4,'functional_tasks_mean':sum(t['count']for t in tasks if t['id']in ['C04','C06'])/2})
 out={'version':'REUSE_UNION_CORRECTED_DP05_FOUR_TASK_ACTUAL_ACCEPTANCE','options':options,'source_evidence':[{'path':str(p.resolve()),'sha256':sha(p)}for p in inputs],'control_baseline_source_inventory_sha256':control['baseline_source_inventory_sha256'],'scope':'Same predeclared C01/C03/C04/C06 equal-weight main mean; functional pair separately. Actual offline patched Rust acceptance only, no model semantic/latency inference.','combiner_source_sha256':sha(Path(__file__))};p=R/'qa04/reuse-union-corrected-dp05-four-task-summary.json';p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'summary':str(p.resolve()),'means':{x['variant']:x['mean_changed_operational_modules']for x in options}}))
if __name__=='__main__':main()
