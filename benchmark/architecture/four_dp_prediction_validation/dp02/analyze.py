"""Frozen profile-level prediction analysis. Never changes endpoints or QA definitions."""
import argparse,json,statistics,itertools
from pathlib import Path
from timing import analyze_timing
from score_dp02 import score
HERE=Path(__file__).resolve().parent

def read(p):return json.loads(Path(p).read_text())
def nr(result,relation):
 actual={sid:[]for sid in ['R1','U2']}
 for a in result.get('associations',[]):
  i=a.get('intent',{});actual.setdefault(a.get('event_id'),[]).append([i.get('relation'),i.get('target_request_id'),i.get('task_id'),i.get('question_id')])
 expected={'R1':[['NEW',None,None,None]],'U2':[['CORRECTION','R1',None,None]] if relation=='correction' else [['CONTINUE','R1',None,None],['NEW',None,None,None]]}
 return {'correct':sum(sorted(actual[s],key=str)==sorted(expected[s],key=str)for s in expected),'total':2,'actual':actual,'expected':expected,'scope':'N-R exact request relations, not QA02'}

def analyze(root):
 fixtures={c['id']:c for c in read(HERE/'fixtures/dp02.json')};goals=read(HERE/'goal_manifests/dp02.json');oracles=read(HERE/'oracles/dp02.json')
 records=[json.loads(s)for s in (root/'runs.jsonl').read_text().splitlines()];rows=[]
 for r in records:
  case=fixtures[r['episode']];t=analyze_timing(r,goals[r['episode']]);result=next((e['data']for e in reversed(r['events'])if e.get('kind')=='EPISODE_RESULT'),{});calls=r['calls'];capture=next(e['observed_ns']for e in r['events']if e.get('kind')=='INPUT_CAPTURED'and e['data']['stimulus_id']=='U2')
  first=next((c for c in sorted(calls,key=lambda c:c.get('enqueue_ns',0))if c['stage']=='dp02_adopt_intents'),{})
  overlap=[]
  for a,b in itertools.combinations(calls,2):
   if a.get('send_ns')and b.get('send_ns'):overlap.append(max(0,min(a['terminal_ns'],b['terminal_ns'])-max(a['send_ns'],b['send_ns']))/1e6)
  wait={};start={}
  for e in r['events']:
   if e.get('kind')=='POLICY_WAIT_START':start[json.dumps(e['data'],sort_keys=True)]=e['elapsed_ns']
  samples=[m for m in r['memory']if len(m.get('processes',[]))>=2];peak=max(samples,key=lambda m:m.get('rss_total_bytes',m.get('sum_bytes',0)),default={})
  # memory.py defines sum_rss_bytes; retain the full actual peak record too.
  if samples:peak=max(samples,key=lambda m:m.get('concurrent_sum_bytes',0))
  rows.append({'run_id':r['run_id'],'episode':r['episode'],'variant':r['variant'],'rep':r['repetition'],'stratum':case['stratum'],'timing':t,'qa02':'N/A','nr':nr(result,case['stratum']['relation']),'finite_content':score(result,oracles[r['episode']]),'qa05_known_usd':sum(c.get('cost_usd')or 0 for c in calls),'qa05_unknown_count':sum(c.get('cost_usd')is None for c in calls),'qa05_unknown_reserved_usd':sum(c.get('reservation_usd',0)for c in calls if c.get('cost_usd')is None),'calls':len(calls),'call_statuses':[c.get('status')for c in calls],'input_tokens':sum(c.get('input_tokens')or 0 for c in calls),'output_tokens':sum(c.get('output_tokens')or 0 for c in calls),'first_call_ms':(first.get('terminal_ns',0)-first.get('send_ns',first.get('terminal_ns',0)))/1e6,'U2_arrived_before_first_return':capture<first.get('terminal_ns',0),'first_call_remaining_at_U2_ms':max(0,first.get('terminal_ns',0)-capture)/1e6,'max_actual_API_overlap_ms':max(overlap,default=0),'slot_wait_ms':sum((c.get('slot_acquired_ns',0)-c.get('before_slot_wait_ns',0))/1e6 for c in calls),'qa06_same_sample_peak':peak,'failures':[e for e in r['events']if e.get('kind')in ['EPISODE_ERROR','GOAL_FAILED']],'result':result})
 groups=[]
 for eid in fixtures:
  for variant in ['A','B']:
   rs=[r for r in rows if r['episode']==eid and r['variant']==variant]
   if not rs:continue
   q1=[x for r in rs for x in r['timing']['qa01_ms']];q3=[x for r in rs for x in r['timing']['qa03_ms']]
   groups.append({'episode':eid,'variant':variant,'runs':len(rs),'qa01_mean_ms':statistics.mean(q1)if q1 else None,'qa01_observed':len(q1),'qa01_planned':sum(r['timing']['qa01_expected']for r in rs),'qa03_mean_ms':statistics.mean(q3)if q3 else None,'qa03_observed':len(q3),'qa03_planned':sum(r['timing']['qa03_expected']for r in rs),'known_cost_usd':sum(r['qa05_known_usd']for r in rs),'unknown_count':sum(r['qa05_unknown_count']for r in rs),'nr_correct':sum(r['nr']['correct']for r in rs),'nr_total':sum(r['nr']['total']for r in rs),'finite_content_correct':sum(r['finite_content']['correct']for r in rs),'finite_content_total':sum(r['finite_content']['total']for r in rs),'overlap_ms':[r['max_actual_API_overlap_ms']for r in rs],'U2first_ms':[next((s['qa01_ms']for s in r['timing']['stimuli']if s['stimulus_id']=='U2'),None)for r in rs]})
 out={'status':'MEASURED_MODEL'if records and not records[0]['offline_responder']else'OFFLINE_PIPELINE_ONLY','observed_runs':len(rows),'planned_primary_runs':72,'old_new_pooling':'FORBIDDEN: changed fixtures and common quiescent-correction implementation; condition-level primary','groups':groups,'rows':rows}
 (root/'summary.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');return out
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('root',type=Path);a=p.parse_args();o=analyze(a.root);print(json.dumps({'runs':o['observed_runs'],'groups':o['groups']},indent=2))
