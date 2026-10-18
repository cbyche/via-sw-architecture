"""Read-only derived decomposition; never mutates candidates, raw or ledgers."""
import collections,hashlib,json,statistics
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir())
R=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019/dp05'
def load(p):return json.loads(p.read_text())
def dump(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
records=[load(p)for p in(R/'raw/runs').glob('*/trace.json')];metrics=load(R/'episode-metrics.json');groups={};requests={};prefixes=[]
for mode in [None,'C1']:
 rs=[r for r in records if r['tactic']==mode];stages={};reads=collections.Counter();branches=collections.Counter();prep=[];served=[];prep_source_ns=[];prep_file_bytes=0
 for r in rs:
  for c in r['calls']:
   x=stages.setdefault(c['stage'],{'role':c['role'],'calls':0,'input_tokens':0,'output_tokens':0,'cost_usd':0,'HTTP_ms':0});x['calls']+=1
   for k in ['input_tokens','output_tokens']:x[k]+=c.get(k)or 0
   x['cost_usd']+=c.get('cost_usd')or 0;x['HTTP_ms']+=(c['terminal_ns']-c['send_ns'])/1e6
   if c['stage']=='dp05_interpret_user_request':
    goal=c['causal_trace']['goal_ids'][0];v=load(Path(c['request_file']));requests[(mode,r['repetition'],goal)]={'sha256':hashlib.sha256(json.dumps(v,sort_keys=True,ensure_ascii=False).encode()).hexdigest(),'call_uuid':c['call_uuid']}
  first_input=min(e['elapsed_ns']for e in r['events']if e['kind']=='INPUT_CAPTURED')
  local_starts={}
  for e in r['events']:
   a=e['data']
   if e['elapsed_ns']<first_input:
    if e['kind']=='SOURCE_READ'and a.get('status')=='OK':prep_file_bytes+=a['bytes']
    if e['kind']=='LOCAL_OPERATION_START':local_starts[a['node_id']]=e['elapsed_ns']
    if e['kind']=='LOCAL_OPERATION_END'and a['node_id']in local_starts:prep_source_ns.append(e['elapsed_ns']-local_starts[a['node_id']])
   if e['kind']=='HANDOFF_SOURCE_READ':reads[(a['consumer'],a['source_id'])]+=1;served.append({'run_id':r['run_id'],'goal_ids':e.get('goal_ids'),'consumer':a['consumer'],'source_id':a['source_id'],'start_line':a['start_line'],'end_line':a['end_line'],'revision':a['revision']})
   if e['kind']=='RECEIVER_BRANCH':branches[a['branch']]+=1
   if e['kind']=='C1_PREFETCH_COMMIT':prep.append({'run_id':r['run_id'],'elapsed_ns':e['elapsed_ns'],'spans':a})
   if e['kind']=='ADOPTION_COMMITTED':prefixes.append({'run_id':r['run_id'],'tactic':mode,'repetition':r['repetition'],'goal_id':a['goal_id'],'actual_display_text':a['text'],'handling':a['handling'],'purpose':a['purpose'],'completion_condition':a['completion_condition'],'recipient':a['recipient'],'targets':a['targets'],'source_revisions':a['source_revisions'],'review':'CONDITIONAL_SCOPE_STRENGTHENED'if r['run_id'].endswith('35dfbe')and a['goal_id']=='h3-mail'else'SOURCE_PURPOSE_RECIPIENT_DRAFT_ONLY_CONSISTENT','reason':'any comparative statements weakens price-equality antecedent'if r['run_id'].endswith('35dfbe')and a['goal_id']=='h3-mail'else'Purpose/source binding and draft-only scope checked, not proof of detailed answer correctness'})
 groups[str(mode)]={'pre_input_source_read_local_ns':prep_source_ns,'pre_input_source_read_local_ns_total':sum(prep_source_ns),'pre_input_actual_full_file_read_bytes':prep_file_bytes,'episodes':len(rs),'stage_calls':stages,'read_counts':[{'consumer':k[0],'source_id':k[1],'count':v}for k,v in sorted(reads.items())],'receiver_branches':dict(branches),'prefetch_commits':prep,'served_original_spans':served,'prefetch_serialized_peak_bytes':max((e['data']['bytes']for r in rs for e in r['events']if e['kind']=='LOGICAL_BUFFER'and e['data']['name']=='dp05_C1_prefetched_originals'),default=0)}
by={}
for m in metrics:
 for g in m['timing_details_by_goal']:by[(m['tactic'],m['rep'],g['goal_id'])]=g
pairs=[]
for rep in [1,2,3]:
 for goal in ['h3-deck','h3-mail','h3-meeting']:
  a=by[(None,rep,goal)];b=by[('C1',rep,goal)];pairs.append({'repetition':rep,'goal_id':goal,'both_completed':a['qa03_ms']is not None and b['qa03_ms']is not None,'C':{k:a[k]for k in ['qa03_ms','elapsed_ms']},'C1':{k:b[k]for k in ['qa03_ms','elapsed_ms']},'prefix_request_values_equal':requests[(None,rep,goal)]['sha256']==requests[('C1',rep,goal)]['sha256'],'requests':{'C':requests[(None,rep,goal)],'C1':requests[('C1',rep,goal)]}})
common=[p for p in pairs if p['both_completed']];out={'groups':groups,'paired_goals':pairs,'all_9_prefix_actual_request_values_equal':all(p['prefix_request_values_equal']for p in pairs),'common_completed_population':{'goals':len(common),'C':{k:statistics.mean(p['C'][k]for p in common)for k in ['qa03_ms','elapsed_ms']},'C1':{k:statistics.mean(p['C1'][k]for p in common)for k in ['qa03_ms','elapsed_ms']}},'failure_classification':{'run_id':'prefetch_tactic-dp05-e3-C-r2-35dfbe','goal_id':'h3-deck','host_error':'missing_structured_facts_or_provenance','actual_reason':'15 nonempty sourced facts exceeds host facts.len()>14 guard, initial and repair; not missing all facts','numeric_cap_explicit_in_frozen_receiver_prompt':False,'displayed':False},'aggregation_compatibility_note':'Old analyzer sorted null and C1 directly, which Python rejects. New report wrapper uses null-safe group ordering only; original raw/group metrics unchanged.'}
dump(R/'mechanism-breakdown.json',out);dump(R/'prefix-review.json',{'fixed_denominator':18,'rows':prefixes,'known_conditional_scope_errors':1,'no_model_judge_calls':True,'meaning_and_callback_times_not_recomputed':True});print(json.dumps({'common':out['common_completed_population'],'identical_prefix_requests':out['all_9_prefix_actual_request_values_equal'],'groups':groups},ensure_ascii=False))
