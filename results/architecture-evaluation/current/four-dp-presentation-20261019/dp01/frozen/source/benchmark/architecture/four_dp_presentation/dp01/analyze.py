"""Derived summaries only; replay times are explicitly excluded from provider QA metrics."""
import collections,json,statistics,sys
from pathlib import Path
from experiment_common import *
import analyze as old_analyze
# Import this file via main, so the canonical module name remains the old analyzer.
def replay_summary(dp,tactic):
 result=ROOT/dp;groups={}
 for mode in ['basic',tactic]:
  records=[read(p)for p in sorted((result/'replay'/mode/'raw/runs').glob('*/trace.json'))];by={}
  for r in records:by[(r['episode'],r['repetition'])]=r
  groups[mode]=by
 pairs=[]
 for key,a in groups['basic'].items():
  b=groups[tactic].get(key)
  if b is None:continue
  def payloads(r):return [c.get('payload_sha256')for c in r['calls']]
  def outputs(r):return [(e['data'].get('goal_id'),e['data'].get('revision'),e['data'].get('text'))for e in r['events']if e.get('kind')=='GOAL_COMPLETE']
  def peak(r):return max((m['concurrent_sum_bytes']for m in r['memory']if m.get('all_registered_observed')),default=None)
  def reads(r):return sum(e.get('kind')=='SOURCE_READ'and e['data']['status']=='OK'for e in r['events'])
  def serving(r):
   starts=[];dur=[]
   for e in r['events']:
    if e.get('kind')=='SERVING_START':starts.append(e['elapsed_ns'])
    if e.get('kind')=='SERVING_END'and starts:dur.append(e['elapsed_ns']-starts.pop(0))
   return sum(dur)
  def bufferpeak(r,name):return max((e['data']['bytes']for e in r['events']if e.get('kind')=='LOGICAL_BUFFER'and e['data']['name']==name),default=0)
  pairs.append({'episode':key[0],'repetition':key[1],'payload_values_equal':payloads(a)==payloads(b),'actual_completion_texts_equal':outputs(a)==outputs(b),'source_read_to_string_basic':reads(a),'source_read_to_string_tactic':reads(b),'cache_hits':sum(e.get('kind')=='C2_EXACT_READ_HIT'for e in b['events']),'serving_ns_basic':serving(a),'serving_ns_tactic':serving(b),'same_sample_peak_bytes_basic':peak(a),'same_sample_peak_bytes_tactic':peak(b),'cache_serialized_peak_bytes':bufferpeak(b,'dp05_C2_exact_cache'),'A3_events':sum(e.get('kind')=='WORKING_VIEW_BUDGET_EVICTION'for e in b['events']),'basic_run_id':a['run_id'],'tactic_run_id':b['run_id']})
 out={'version':1,'pairs':pairs,'expected_pairs':12,'observed_pairs':len(pairs),'all_model_payload_values_equal':all(p['payload_values_equal']for p in pairs),'all_actual_completion_texts_equal':all(p['actual_completion_texts_equal']for p in pairs),'API_calls':0,'replay_latency_not_provider_latency':True,'source_read_to_string_basic':sum(p['source_read_to_string_basic']for p in pairs),'source_read_to_string_tactic':sum(p['source_read_to_string_tactic']for p in pairs),'cache_hits':sum(p['cache_hits']for p in pairs),'serving_ns_basic':sum(p['serving_ns_basic']for p in pairs),'serving_ns_tactic':sum(p['serving_ns_tactic']for p in pairs)}
 for mode in ['basic','tactic']:
  values=[p[f'same_sample_peak_bytes_{mode}']for p in pairs if p[f'same_sample_peak_bytes_{mode}']is not None];out['replay_'+mode+'_max_RSS_MB']=max(values)/1e6 if values else None;out['replay_'+mode+'_mean_trial_peak_RSS_MB']=statistics.mean(values)/1e6 if values else None
 dump(result/'replay-summary.json',out);return out

def fresh_summary(dp):
 root=ROOT/dp;sys.modules.pop('analyze',None);sys.path.insert(0,str(OLD));import analyze
 analyze.analyze(root);d=read(root/'summary.json');out=[]
 for row in d['rows']:
  rs=[read(p)for p in(root/'raw/runs').glob('*/trace.json')];rs=[r for r in rs if r['variant']==row['variant']and r['tactic']==row['tactic']]
  row['input_tokens']=sum(c.get('input_tokens')or 0 for r in rs for c in r['calls']);row['output_tokens']=sum(c.get('output_tokens')or 0 for r in rs for c in r['calls']);row['total_cost_usd']=row['qa05_via_usd']+row['qa05_receiver_usd'];row['source_reads']=sum(e.get('kind')=='SOURCE_READ'and e['data']['status']=='OK'for r in rs for e in r['events']);row['stage_calls']=dict(collections.Counter(c['stage']for r in rs for c in r['calls']));out.append(row)
 dump(root/'fresh-summary.json',{'rows':out,'metric_source':'unchanged review-v2 frozen endpoint/evaluator code','scope':'DP01 E3 output-policy sensitivity or DP05 E3 prefetch paired; not BASIC120 replacement'})
 return out
if __name__=='__main__':
 p=__import__('argparse').ArgumentParser();p.add_argument('action',choices=['replay','fresh']);p.add_argument('--dp',choices=['dp01','dp05'],required=True);a=p.parse_args()
 print(json.dumps(replay_summary(a.dp,'A3'if a.dp=='dp01'else'C2')if a.action=='replay'else fresh_summary(a.dp),ensure_ascii=False))
