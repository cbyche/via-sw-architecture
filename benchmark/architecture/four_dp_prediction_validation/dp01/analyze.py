"""Read frozen authored oracle only in evaluator; retain all failures and condition strata."""
import json,statistics,hashlib,collections
from pathlib import Path
from control import HERE,RESULT,read,sha
from timing import analyze_timing
from score_dp01 import score as body_score

def mean(v):return statistics.mean(v) if v else None
def avg(row,key):return mean([r[key] for r in row if r.get(key)is not None])
def main():
 cases={c['id']:c for c in read(HERE/'fixtures.json')};oracles=read(HERE/'oracles.json');rows=[];audits=[]
 for p in sorted((RESULT/'raw/runs').glob('*/trace.json')):
  rec=read(p)
  if rec['phase']!='PREDICTION_VALIDATION':continue
  case=cases[rec['episode']];oracle=oracles[rec['episode']];events=rec['events'];goals=oracle['goals'];by_goal={g['goal_id']:g for g in goals}
  expected={'qa01_stimuli':[g['goal_id']for g in goals],'qa03_goals':[{'goal_id':g['goal_id'],'origin_stimulus_id':g['goal_id'],'final_revision':next((e.get('delegation_revision',e.get('revision',1))for e in case['events']if e.get('goal_id')==g['goal_id']),1)}for g in goals]}
  # Preserve existing QA01 actual host-accepted callback. Exact target mismatches are separate diagnostics, never an oracle-dependent timing endpoint.
  filtered=dict(rec);filtered['events']=[];wrong=[]
  for e in events:
   d=e.get('data',{});g=by_goal.get(d.get('stimulus_id'))
   if e.get('kind')=='DISPLAY_PROXY' and g and not g.get('close') and d.get('content_type')=='ADOPTED_HANDLING' and set(d.get('targets',[]))!=set(g.get('targets',[])):
    wrong.append(e);continue
   filtered['events'].append(e)
  timing=analyze_timing(rec,expected)
  result=next((e['data']for e in reversed(events)if e.get('kind')=='EPISODE_RESULT'),{})
  result['outputs']=[e['data']for e in events if e.get('kind')=='GOAL_COMPLETE']
  content=body_score(result,{'content_checks':[g for g in goals if not g.get('close')]})
  qa02=[]
  if rec['dp']=='01':
   for g in goals:
    adopt=next((e['data']for e in events if e.get('kind')=='SEMANTIC_ADOPTION' and e['data'].get('expression_id')==g['goal_id']+'-ref'),None)
    versions={s['source_id']:s['revision']for s in (adopt or {}).get('source_revisions',[])}
    qa02.append({'goal_id':g['goal_id'],'correct':adopt is not None and set(adopt.get('targets',[]))==set(g['targets']) and versions==g['required_source_revisions'],'actual':adopt})
  nh=[]
  if rec['dp']=='05':
   admissions=[e['data']for e in events if e.get('kind')=='HANDOFF_ADMISSION'];seen=set();state={};current={}
   for event in case['events']:
    if event['kind']=='source':current[event['source']['id']]=event['source']
    elif event['kind']=='adopt':state=event['state']
    elif event['kind']=='revoke':current[event['source_id']]={**current[event['source_id']],'revoked':True}
    elif event['kind'] in ('delegate','observe'):
     gid=event['goal_id'];found=[h for h in admissions if h['mandatory'].get('goal_id')==gid];errors=[];g=by_goal[gid]
     if g.get('close'):
      if found:errors.append('unexpected_handoff_for_close')
     elif len(found)!=1:errors.append('missing_or_duplicate_handoff')
     else:
      h=found[0];m=h['mandatory'];manifest={s['id']:s for s in h['manifest']}
      for k in ['constraints','exclusions','allowed_actions','already_completed_actions','unknown_actions','request_revision','condition_revision','access_epoch','required_source_ids']:
       if m.get(k)!=state.get(k):errors.append('mandatory_'+k)
      if set(m.get('targets',[]))!=set(g['targets']):errors.append('targets')
      for sid,rev in g['required_source_revisions'].items():
       if manifest.get(sid,{}).get('revision')!=rev:errors.append('current_manifest_'+sid)
      if any(s.get('revoked')and sid in manifest for sid,s in current.items()):errors.append('revoked_manifest')
     nh.append({'goal_id':gid,'correct':not errors,'errors':errors})
  prep=[];start=None
  for e in events:
   if e.get('kind')=='PREPARATION_START':start=e['elapsed_ns']
   elif e.get('kind')=='PREPARATION_COMPLETE'and start is not None:prep.append((e['elapsed_ns']-start)/1e6);start=None
  complete_samples=[s for s in rec['memory']if s.get('all_registered_observed')]
  peak_sample=max(complete_samples,key=lambda s:s['concurrent_sum_bytes'])if complete_samples else None
  peaks=[s['concurrent_sum_bytes']for s in complete_samples]
  state_peaks=[sum(x['bytes']for x in s.get('logical_buffers',{}).values()if x.get('owner')=='rust_candidate')for s in rec['memory']]
  branches=collections.Counter((e['kind']+':'+str(e.get('data',{}).get('branch')))for e in events if e.get('kind')in ['HANDOFF_BUILD_BRANCH','RECEIVER_BRANCH','EVIDENCE_BRANCH'])
  costs=[c.get('cost_usd')for c in rec['calls']];total=sum(c or 0 for c in costs)
  row={'run_id':rec['run_id'],'condition':case['condition'],'variant':rec['variant'],'repetition':rec['repetition'],'qa01_ms':mean(timing['qa01_ms']),'qa01_sum_ms':sum(timing['qa01_ms']),'qa01_observed':len(timing['qa01_ms']),'qa01_expected':timing['qa01_expected'],'qa03_ms':mean(timing['qa03_ms']),'qa03_sum_ms':sum(timing['qa03_ms']),'qa03_observed':len(timing['qa03_ms']),'qa03_expected':timing['qa03_expected'],'qa02':qa02 if rec['dp']=='01' else 'NOT_APPLICABLE','body_diagnostic':content,'handoff_diagnostic':nh,'full_goal_wall_ms':mean([g['elapsed_ms']for g in timing['goals']if g['elapsed_ms']is not None]),'preparation_ms':sum(prep),'preparation_calls':sum('publish' in c['stage'] or 'produce_work_record'in c['stage']for c in rec['calls']),'calls':len(costs),'cost_usd':total,'unknown_cost_calls':sum(c is None for c in costs),'same_time_registered_rss_peak_bytes':max(peaks)if peaks else None,'same_time_peak_sample':peak_sample,'rust_logical_peak_serialized_bytes':max(state_peaks)if state_peaks else None,'timing':timing,'wrong_target_first_displays':wrong,'branches':dict(branches),'failure_events':[e for e in events if e.get('kind')in ['GOAL_FAILED','EPISODE_ERROR']],'returncode':rec['returncode'],'trace':str(p),'trace_sha256':sha(p)}
  rows.append(row)
  for g in timing['stimuli']:
   calls=[c for c in rec['calls']if g['stimulus_id']in c.get('causal_trace',{}).get('goal_ids',[])]
   first=calls[0]if calls else {};audits.append({'run_id':rec['run_id'],'goal':g['stimulus_id'],'first_call':{k:first.get(k)for k in ['stage','input_payload_sha256','input_payload_bytes','input_field_names','input_tokens','output_tokens','request_file']},'first_display':g['first_content'],'later_branches':dict(branches)})
 groups=[]
 for condition in ['ALL_FINITE_FIXED_GOALS']+list(dict.fromkeys(c['condition']for c in cases.values())):
  for v in 'ABC':
   rr=[r for r in rows if r['variant']==v and (condition=='ALL_FINITE_FIXED_GOALS'or r['condition']==condition)]
   n1=sum(r['qa01_observed']for r in rr);n3=sum(r['qa03_observed']for r in rr)
   q2=[x for r in rr for x in r['qa02']]if HERE.name=='dp01' else []
   groups.append({'condition':condition,'variant':v,'trajectories':len(rr),'QA01_ms':sum(r['qa01_sum_ms']for r in rr)/n1 if n1 else None,'QA01_observed':n1,'QA01_expected':sum(r['qa01_expected']for r in rr),'QA02_correct':sum(x['correct']for x in q2)if q2 else None,'QA02_total':len(q2)if q2 else None,'QA03_ms':sum(r['qa03_sum_ms']for r in rr)/n3 if n3 else None,'QA03_observed':n3,'QA03_expected':sum(r['qa03_expected']for r in rr),'QA05_total_usd':sum(r['cost_usd']for r in rr),'QA05_mean_per_trajectory_usd':avg(rr,'cost_usd'),'full_wall_ms':avg(rr,'full_goal_wall_ms'),'prep_ms_mean':avg(rr,'preparation_ms'),'QA06_RSS_peak_MB':max((r['same_time_registered_rss_peak_bytes']for r in rr if r['same_time_registered_rss_peak_bytes']is not None),default=0)/1e6 if rr else None,'QA06_RSS_mean_peak_MB_diagnostic':avg(rr,'same_time_registered_rss_peak_bytes')/1e6 if rr else None,'logical_serialized_mean_peak_bytes_diagnostic':avg(rr,'rust_logical_peak_serialized_bytes'),'body_correct':sum(r['body_diagnostic']['correct']for r in rr),'body_total':sum(r['body_diagnostic']['total']for r in rr),'NH_correct':sum(x['correct']for r in rr for x in r['handoff_diagnostic']),'NH_total':sum(len(r['handoff_diagnostic'])for r in rr),'unknown_cost_calls':sum(r['unknown_cost_calls']for r in rr)})
 output={'rows':rows,'groups':groups,'pooling':'NO_OLD_DATA_POOLING_CHANGED_FIXTURES_POLICY_IMPLEMENTATION','QA04':'SEPARATE_FRESH_CHANGE_EXERCISES','claim':'Finite authored fixtures, actual model and accepted display proxy; no product/local Omni claim. Condition means complement unchanged fixed-goal QA arithmetic.'}
 (RESULT/'summary.json').write_text(json.dumps(output,ensure_ascii=False,indent=2));(RESULT/'first-input-audit.json').write_text(json.dumps(audits,ensure_ascii=False,indent=2))
 lines=['# Fresh prediction validation', '',output['claim'],'','Old data pooling: prohibited. New fixed-goal totals and condition strata shown separately. Missing endpoints retain expected denominator; conditional means cannot imply completion for missing goals.','', '|Condition|Alt|QA01 ms (n/N)|QA02|QA03 ms (n/N)|Cost USD|Wall ms|Prep ms|RSS peak MB|Body diagnostic|', '|---|---|---|---|---|---|---|---|---|---|']
 fmt=lambda x:'N/A'if x is None else f'{x:.3f}'
 for g in groups:lines.append(f"|{g['condition']}|{g['variant']}|{fmt(g['QA01_ms'])} ({g['QA01_observed']}/{g['QA01_expected']})|{g['QA02_correct']}/{g['QA02_total']}|{fmt(g['QA03_ms'])} ({g['QA03_observed']}/{g['QA03_expected']})|{g['QA05_total_usd']:.6f}|{fmt(g['full_wall_ms'])}|{fmt(g['prep_ms_mean'])}|{fmt(g['QA06_RSS_peak_MB'])}|{g['body_correct']}/{g['body_total']}|")
 (RESULT/'report.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'runs':len(rows),'groups':len(groups)}))
if __name__=='__main__':main()
