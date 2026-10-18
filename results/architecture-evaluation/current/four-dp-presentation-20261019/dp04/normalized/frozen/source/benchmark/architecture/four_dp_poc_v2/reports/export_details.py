"""Derived case/stage/endpoint/memory exports. Reads immutable episode evidence only."""
import argparse,csv,hashlib,importlib.util,json,statistics,sys
from collections import defaultdict
from itertools import combinations
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018';BASE=REPO/'benchmark/architecture/four_dp_poc_v2'
sys.path.insert(0,str(BASE))
from evidence import read_evaluator
from analyze import analyze
def read(p):return json.loads(Path(p).read_text())
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def csvsave(p,rows):
 with Path(p).open('w')as f:
  names=list(dict.fromkeys(k for r in rows for k in r));w=csv.DictWriter(f,fieldnames=names);w.writeheader();w.writerows({k:json.dumps(v,ensure_ascii=False)if isinstance(v,(dict,list))else v for k,v in r.items()}for r in rows)
def avg(x):return statistics.mean(x)if x else None
def describe(xs):
 xs=[x for x in xs if x is not None];return {'observed_n':len(xs),'mean':avg(xs),'median':statistics.median(xs)if xs else None,'min':min(xs)if xs else None,'max':max(xs)if xs else None,'values':xs}
def fmt(x):return 'NOT_OBSERVED'if x is None else f'{x:.3f}'if isinstance(x,float)else str(x)
def main(folder):
 analyze(folder);rows=read(folder/'episode-metrics.json');records=[json.loads(x)for x in(folder/'runs.jsonl').read_text().splitlines()if x]
 by={r['run_id']:r for r in records};stage=[];goals=[];stimuli=[];memory=[];equivalence=[];failed_attempts=[];waits=[];failures=[];preparations=[];rejections=[];all_failure_callbacks=[]
 manifest=read(folder/'freeze.json');external=manifest.get('evaluator_external_sources',{})
 pin=next((v for k,v in external.items()if k.endswith('/evaluate_bounded_content_equivalence.py')),None)
 module=None
 if pin:
  p=Path(pin['snapshot']);assert hashlib.sha256(p.read_bytes()).hexdigest()==pin['sha256']
  spec=importlib.util.spec_from_file_location('frozen_bounded_equivalence',p);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
 for row in rows:
  rec=by[row['run_id']];base={k:row[k]for k in ['run_id','phase','dp','episode','variant','rep','tactic']};base['raw_trace']=str(Path(rec['input']).with_name('trace.json'));base['raw_input']=rec['input']
  for event in rec['events']:
   if 'REJECT'not in event.get('kind',''):continue
   d=event.get('data',{});gid=d.get('goal_id');sid=d.get('stimulus_id');goalmatch=next((g for g in row['timing_details_by_goal']if g['goal_id']==gid),None);stim=row['timing_details_by_stimulus'].get(sid,{})
   start=goalmatch.get('start_elapsed_ns')if goalmatch else stim.get('input_elapsed_ns')
   rejections.append({**base,'kind':event['kind'],'elapsed_ns':event.get('elapsed_ns'),'goal_id':gid,'stimulus_id':sid,'input_to_rejection_ms':(event['elapsed_ns']-start)/1e6 if start is not None else None,'goal_causes':event.get('goal_ids'),'raw_reason':d,'actual_causal_parents':event.get('parent_ids'),'causal_call_uuids':[c['call_uuid']for c in rec['calls']if c.get('causal_trace',{}).get('node_id')in event.get('parent_ids',[])],'subsequent_required_completion_observed':goalmatch['status']=='OBSERVED'if goalmatch else None})
  failure_events=[e for e in rec['events']if e.get('kind')in('GOAL_FAILED','EPISODE_ERROR')]
  captures={e['data']['stimulus_id']:e['elapsed_ns']for e in reversed(rec['events'])if e.get('kind')=='INPUT_CAPTURED'and e.get('data',{}).get('stimulus_id')}
  for e in failure_events:
   d=e.get('data',{});g=next((g for g in row['timing_details_by_goal']if g['goal_id']==d.get('goal_id')),None);sids=[s[len('input:'):]for s in e.get('goal_ids',[])if s.startswith('input:')];sid=d.get('stimulus_id')or(sids[0]if len(sids)==1 else None);start=g.get('start_elapsed_ns')if g else captures.get(sid)
   all_failure_callbacks.append({**base,'kind':e['kind'],'actual_failure_goal_id':d.get('goal_id'),'actual_input_scope_ids':sids,'stimulus_id':sid,'elapsed_ns':e.get('elapsed_ns'),'input_elapsed_ns':start,'input_to_failure_ms':(e['elapsed_ns']-start)/1e6 if start is not None else None,'mapping':'fixed expected goal origin'if g else'actual captured input scope only; unresolved goal is not adopted'if start is not None else'NO_INPUT_MAPPING_OBSERVED','raw_reason':d,'not_successful_qa03':True})
  raw_reasons=json.dumps([e.get('data')for e in failure_events],ensure_ascii=False);lower=raw_reasons.lower()
  tags=[]
  if rec['returncode']!=0:tags.append('PROCESS_NONZERO_EXIT')
  if any(e['kind']=='EPISODE_ERROR'for e in failure_events):tags.append('UNHANDLED_EPISODE_ERROR_EVENT')
  if any(e['kind']=='GOAL_FAILED'for e in failure_events):tags.append('BOUNDED_GOAL_CONTRACT_OR_COMPLETION_FAILURE')
  if any(t in lower for t in ['quote','span','supplied_current','source_revision']):tags.append('SOURCE_SPAN_OR_CURRENTNESS_CONTRACT')
  if row['dp']=='04'and any(t in lower for t in ['count','bbox','geometr','point_','region','cue','contract_failures']):tags.append('GEOMETRY_COUNT_OR_RELATION_CONTRACT')
  failed_calls=[{'call_uuid':c.get('call_uuid'),'status':c.get('status'),'error':c.get('error')}for c in rec['calls']if c.get('status')not in('SUCCESS','USAGE_RECORDED','HTTP_200','OK')and c.get('error')]
  if failure_events or failed_calls or rec['returncode']!=0:failures.append({**base,'returncode':rec['returncode'],'stderr':rec['stderr'],'tags_are_raw_reason_keyword_diagnostics_not_semantic_judgments':tags,'actual_failure_events':failure_events,'provider_or_bridge_call_errors':failed_calls,'completed_goals':row['qa03_n'],'expected_goals':row['qa03_expected'],'meaning_diagnostics_separate':{'qa02_wrong':row['qa02_total']-row['qa02_correct'],'nr_wrong':row['nr_total']-row['nr_correct'],'nh_wrong':row['nh_total']-row['nh_correct'],'literal_content_wrong':row['content_total']-row['content_correct']}})
  pending_preparations={}
  for event in rec['events']:
   kind=event.get('kind');data=event.get('data',{});pid=data.get('preparation_id')
   if kind=='PREPARATION_START':pending_preparations[pid]=event
   elif kind=='PREPARATION_COMPLETE'and pid in pending_preparations:
    begin=pending_preparations.pop(pid);lo=begin['elapsed_ns'];hi=event['elapsed_ns'];nodes={e['data'].get('node_id')for e in rec['events']if e.get('kind')=='OPERATION_START'and lo<=e['elapsed_ns']<=hi};prep_calls=[c for c in rec['calls']if c.get('causal_trace',{}).get('node_id')in nodes]
    after=[e for e in rec['events']if e.get('elapsed_ns',-1)>=hi];next_input=next((e for e in after if e.get('kind')=='INPUT_CAPTURED'),None);next_display=next((e for e in after if e.get('kind')=='DISPLAY_PROXY'),None)
    preparations.append({**base,'preparation_id':pid,'start_elapsed_ns':lo,'end_elapsed_ns':hi,'actual_preparation_ms':(hi-lo)/1e6,'next_input_elapsed_ns':next_input.get('elapsed_ns')if next_input else None,'preparation_finished_before_next_input':bool(next_input),'next_display_elapsed_ns':next_display.get('elapsed_ns')if next_display else None,'preparation_start_to_next_display_ms':(next_display['elapsed_ns']-lo)/1e6 if next_display else None,'model_calls':len(prep_calls),'input_tokens':sum(c.get('input_tokens')or 0 for c in prep_calls),'output_tokens':sum(c.get('output_tokens')or 0 for c in prep_calls),'cost_usd':sum(c.get('cost_usd')or 0 for c in prep_calls),'call_uuids':[c['call_uuid']for c in prep_calls],'scope':'Diagnostic actual pre-request source-production interval; QA01/03 request time must not imply this preparation was free or overlapped the query.'})
  for pid,begin in pending_preparations.items():
   lo=begin['elapsed_ns'];nodes={e['data'].get('node_id')for e in rec['events']if e.get('kind')=='OPERATION_START'and e['elapsed_ns']>=lo and not e['data'].get('goal_ids')};prep_calls=[c for c in rec['calls']if c.get('causal_trace',{}).get('node_id')in nodes]
   preparations.append({**base,'preparation_id':pid,'start_elapsed_ns':lo,'end_elapsed_ns':None,'actual_preparation_ms':None,'status':'PREPARATION_COMPLETE_NOT_OBSERVED','model_calls':len(prep_calls),'cost_usd':sum(c.get('cost_usd')or 0 for c in prep_calls),'call_uuids':[c['call_uuid']for c in prep_calls]})
  pending_waits={}
  for event in rec['events']:
   kind=event.get('kind','');d=event.get('data',{});sid=d.get('stimulus_id',d.get('task_id',d.get('goal_id')));family=kind[:-6]if kind.endswith('_START')else kind[:-4]if kind.endswith('_END')else None
   if family in ('POLICY_WAIT','USER_WAIT'):
    key=(family,sid)
    if kind.endswith('_START'):pending_waits[key]=event
    elif key in pending_waits:
     beginning=pending_waits.pop(key);waits.append({**base,'kind':family,'stimulus_or_task_or_goal_id':sid,'wait_scope':'step-boundary'if d.get('stimulus_id')else'pending-input publication gate'if d.get('goal_id')else'external user wait','start_elapsed_ns':beginning['elapsed_ns'],'end_elapsed_ns':event['elapsed_ns'],'elapsed_ms':(event['elapsed_ns']-beginning['elapsed_ns'])/1e6,'start_data':beginning['data'],'end_data':d,'included_in_goal_time_unless_explicitly_excluded_by_timing_contract':True})
  for call in rec['calls']:
   stage.append({**base,'stage':call.get('stage'),'role':call.get('role','VIA'),'call_uuid':call.get('call_uuid'),'status':call.get('status'),'input_tokens':call.get('input_tokens'),'output_tokens':call.get('output_tokens'),'cost_usd':call.get('cost_usd'),'enqueue_to_send_ms':(call['send_ns']-call['enqueue_ns'])/1e6 if call.get('send_ns')and call.get('enqueue_ns')else None,'send_to_terminal_ms':(call['terminal_ns']-call['send_ns'])/1e6 if call.get('terminal_ns')and call.get('send_ns')else None,'goal_causes':call.get('causal_trace',{}).get('goal_ids',[]),'resource_slot':call.get('resource_slot'),'request_path':call.get('request_file'),'response_path':call.get('response_file'),'response_action':call.get('response_value',{}).get('action')if isinstance(call.get('response_value'),dict)else None,'response_mode':call.get('response_value',{}).get('mode')if isinstance(call.get('response_value'),dict)else None})
  for g in row['timing_details_by_goal']:
   goals.append({**base,**g})
   for failure in g['failed_attempts']:failed_attempts.append({**base,'goal_id':g['goal_id'],'required_final_revision':g.get('final_revision'),'completion_status':g['status'],**failure})
  for sid,s in row['timing_details_by_stimulus'].items():
   final=[e for e in rec['events']if e.get('kind')=='DISPLAY_PROXY'and e.get('data',{}).get('stimulus_id')==sid and e['data'].get('content_type')=='DETAILED_RESULT']
   end=final[-1]['elapsed_ns']if final else None;first=s['first_elapsed_ns'];start=s['input_elapsed_ns']
   stimuli.append({**base,**s,'detailed_elapsed_ns':end,'input_to_detailed_ms':(end-start)/1e6 if end is not None and start is not None else None,'first_to_detailed_ms':(end-first)/1e6 if end is not None and first is not None else None,'first_and_final_may_naturally_coincide':row['dp']=='04'})
  peak=row['rss_peak_same_sample']or {};initial=row['rss_runtime_initial_sample']or {};logical=peak.get('logical_buffers',{});categories=defaultdict(int)
  for buffer in logical.values():categories[buffer['category']]+=buffer['bytes']
  memory.append({**base,'sample_count':row['memory_sample_count'],'both_process_sample_count':row['memory_both_process_sample_count'],'incomplete':row['memory_state']!='SAMPLED_RSS_ESTIMATE','nominal_poll_ms':50,'actual_max_sample_gap_ms':row['memory_max_sample_gap_ms'],'peak_stage':peak.get('stage'),'peak_sample_start_ns':peak.get('at_ns'),'peak_sample_end_ns':peak.get('sample_end_ns'),'same_sample_sum_mb':row['rss_concurrent_peak_mb'],'peak_all_registered_observed':peak.get('all_registered_observed'),'peak_python_mb':peak['rss_bytes_by_role']['python_runner_bridge']/1e6 if 'python_runner_bridge'in peak.get('rss_bytes_by_role',{})else None,'peak_rust_mb':peak['rss_bytes_by_role']['rust_candidate']/1e6 if 'rust_candidate'in peak.get('rss_bytes_by_role',{})else None,'initial_simultaneous_mb':row['rss_concurrent_first_simultaneous_mb'],'initial_python_mb':initial['rss_bytes_by_role']['python_runner_bridge']/1e6 if 'python_runner_bridge'in initial.get('rss_bytes_by_role',{})else None,'initial_rust_mb':initial['rss_bytes_by_role']['rust_candidate']/1e6 if 'rust_candidate'in initial.get('rss_bytes_by_role',{})else None,'same_role_delta_bytes':{role:value-initial['rss_bytes_by_role'][role]if role in initial.get('rss_bytes_by_role',{})else None for role,value in peak.get('rss_bytes_by_role',{}).items()},'before_rust_start_sample':rec['memory_baseline'],'peak_process_pid_ppid_roles':peak.get('processes'),'logical_peak_categories_bytes':dict(categories),'harness_retained_evidence_logical_bytes':categories.get('retained_evidence',0),'logical_peak_buffers':logical,'logical_bytes_not_rss':True,'shared_page_dedup_unverified':True})
  if module is not None and row['dp']in('01','05'):
   oracle=read_evaluator(rec,Path('oracles')/('dp'+row['dp']+'.json'));oracle=next(o for o in oracle if o['id']==row['episode'])if isinstance(oracle,list)else oracle[row['episode']]
   equivalence.append({**base,**module.evaluate(row['result'],oracle)})
 csvsave(folder/'all-failure-callbacks.csv',all_failure_callbacks);csvsave(folder/'proposal-rejections.csv',rejections);csvsave(folder/'preparation.csv',preparations);save(folder/'failure-causes.json',failures);csvsave(folder/'policy-waits.csv',waits);csvsave(folder/'failed-attempts.csv',failed_attempts);csvsave(folder/'stage-calls.csv',stage);csvsave(folder/'goal-endpoints.csv',goals);csvsave(folder/'response-endpoints.csv',stimuli);csvsave(folder/'memory-breakdown.csv',memory)
 save(folder/'memory-breakdown.json',memory);save(folder/'bounded-content-equivalence.json',{'version':module.VERSION if module else None,'sha256':pin['sha256'] if pin else None,'not_frozen_in_this_phase':pin is None,'scope':'DP01/05 parallel bounded marker diagnostic only. DP02 required revision/receipt scorer unchanged. Actual meaning and conditional claims need separate manual review. Literal metrics are preserved.','rows':equivalence})
 grouped=defaultdict(list)
 for row in rows:grouped[(row['dp'],row['variant'],row['episode'])].append(row)
 cases=[]
 for (dp,v,episode),rs in sorted(grouped.items()):
  n1=sum(r['qa01_n']for r in rs);n3=sum(r['qa03_n']for r in rs)
  calls=[c for r in rs for c in by[r['run_id']]['calls']]
  cases.append({'dp':dp,'variant':v,'episode':episode,'reps':len(rs),'qa01_ms':sum(r['qa01_sum_ms']for r in rs)/n1 if n1 else None,'qa01_observed':n1,'qa01_expected':sum(r['qa01_expected']for r in rs),'qa03_ms':sum(r['qa03_sum_ms']for r in rs)/n3 if n3 else None,'qa03_full_elapsed_ms':sum(r['qa03_full_elapsed_sum_ms']for r in rs)/n3 if n3 else None,'receiver_excluded_ms':sum(r['receiver_shim_excluded_sum_ms']for r in rs)/n3 if n3 else None,'qa03_observed':n3,'qa03_expected':sum(r['qa03_expected']for r in rs),'comparison_status':'INCOMPLETE'if any(r['comparison_status']=='INCOMPLETE'for r in rs)else'COMPLETE','qa02_correct':sum(r['qa02_correct']for r in rs),'qa02_expected':sum(r['qa02_total']for r in rs),'model_calls':len(calls),'input_tokens':sum(c.get('input_tokens')or 0 for c in calls),'output_tokens':sum(c.get('output_tokens')or 0 for c in calls),'via_usd':sum(r['via_cost_usd']for r in rs),'receiver_usd':sum(r['receiver_cost_usd']for r in rs),'rss_max_mb':max(r['rss_concurrent_peak_mb']or 0 for r in rs),'rss_incomplete_trials':sum(r['memory_state']!='SAMPLED_RSS_ESTIMATE'for r in rs),'host_failed_trials':sum(r['functional_error']for r in rs),'literal_content_correct':sum(r['content_correct']for r in rs),'literal_content_expected':sum(r['content_total']for r in rs),'individual_runs':[{k:r[k]for k in ['run_id','rep','qa01_mean_ms','qa01_n','qa01_expected','qa03_mean_ms','qa03_n','qa03_expected','via_cost_usd','receiver_cost_usd','model_calls','rss_concurrent_peak_mb','functional_error']}for r in rs]})
 for c in cases:
  rs=grouped[(c['dp'],c['variant'],c['episode'])]
  c['individual_trial_mean_descriptive']={metric:describe([r[metric]for r in rs])for metric in ['qa01_mean_ms','qa03_mean_ms']}
  c['all_observed_endpoint_descriptive']={metric:describe([x for r in rs for x in r['timing_samples'][metric]])for metric in ['qa01_ms','qa03_ms']}
  c['failure_input_to_terminal_ms']=describe([f['input_to_failure_ms']for r in rs for g in r['timing_details_by_goal']for f in g['failed_attempts']])
  c['descriptive_scope']=str(len(rs))+' recorded finite trials (final plan: three repetitions). Observed conditional means and endpoint distributions; missing values stay missing. No significance test or population claim.'
 paired=[]
 for dp,episode in sorted({(r['dp'],r['episode'])for r in rows}):
  variants=sorted({r['variant']for r in rows if r['dp']==dp and r['episode']==episode})
  for a,b in combinations(variants,2):
   for metric,collection,idkey,valuekey in [('QA01',stimuli,'stimulus_id','qa01_ms'),('QA03',goals,'goal_id','qa03_ms')]:
    maps={v:{(r['rep'],r[idkey]):r.get(valuekey)for r in collection if r['dp']==dp and r['episode']==episode and r['variant']==v}for v in (a,b)};keys=set(maps[a])|set(maps[b]);both=[key for key in keys if maps[a].get(key)is not None and maps[b].get(key)is not None]
    paired.append({'dp':dp,'episode':episode,'metric':metric,'variant_a':a,'variant_b':b,'fixed_identical_endpoint_keys':len(keys),'both_observed':len(both),'only_a_observed':sum(maps[a].get(k)is not None and maps[b].get(k)is None for k in keys),'only_b_observed':sum(maps[b].get(k)is not None and maps[a].get(k)is None for k in keys),'neither_observed':sum(maps[a].get(k)is None and maps[b].get(k)is None for k in keys),'paired_a_ms':describe([maps[a][k]for k in sorted(both)]),'paired_b_ms':describe([maps[b][k]for k in sorted(both)]),'paired_a_minus_b_ms':describe([maps[a][k]-maps[b][k]for k in sorted(both)]),'scope':'Same authored episode/repetition/goal or stimulus only. Endpoint-observed pairing is not semantic correctness or full-success qualification; incomplete conditional means cannot establish an overall winner.'})
 save(folder/'pairwise-endpoint-coverage.json',paired);csvsave(folder/'pairwise-endpoint-coverage.csv',paired)
 save(folder/'case-summary.json',cases);csvsave(folder/'case-summary.csv',[{k:v for k,v in x.items()if k!='individual_runs'}for x in cases])
 lines=['# 사례별 실제 측정값','','단위 ms, 비용 USD. 평균은 관측된 callback/완료에 조건부입니다. INCOMPLETE를 빠른 승자로 해석하지 않습니다. DP05는 API-boundary receiver shim 내부 구간을 제외한 값과 전체 경과를 함께 표시합니다.','','|DP|안|사례|반복|첫 응답 ms (관측/예정)|VIA 완료 ms (관측/예정)|전체 경과 ms|수신자 제외 ms|대상 집합 정답/예정|VIA $|수신자 $|호출|상태|','|---|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---|']
 for c in cases:lines.append(f"|{c['dp']}|{c['variant']}|{c['episode']}|{c['reps']}|{fmt(c['qa01_ms'])} ({c['qa01_observed']}/{c['qa01_expected']})|{fmt(c['qa03_ms'])} ({c['qa03_observed']}/{c['qa03_expected']})|{fmt(c['qa03_full_elapsed_ms'])}|{fmt(c['receiver_excluded_ms'])}|{c['qa02_correct']}/{c['qa02_expected']}|{c['via_usd']:.7f}|{c['receiver_usd']:.7f}|{c['model_calls']}|{c['comparison_status']}|")
 lines+=['','각 3회 개별 값과 raw 경로는 case-summary.json, episode-metrics.csv, response-endpoints.csv, goal-endpoints.csv에 보존합니다. stage-calls.csv는 실제 call/token/cost와 bridge admission+직렬화/기록+slot 대기를 합친 enqueue→send 구간을 표시합니다. 순수 slot 대기는 별도 timestamp가 없어 분리하지 않습니다. policy-waits.csv는 실제 Rust step 대기를 따로 보존하고 preparation.csv는 요청 전 실제 생산 비용/시간을 보존합니다. memory-breakdown.csv는 같은 시각 Python/Rust RSS와 같은 실행 초기값/차이, 논리 버퍼 분류를 표시합니다. retained_evidence는 실험 기록 보관 비용이며 설계 상태와 분리합니다. 논리 직렬화 바이트는 RSS 세부 할당 측정이 아니며 더해서 RSS를 설명할 수 없습니다. 50 ms는 명목 polling이며 실제 표본 간격도 별도 기록했습니다.']
 individual=['# 사례별 개별 반복과 기술 통계','','단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.']
 for c in cases:
  individual+=['',f"## DP{c['dp']} {c['variant']} · {c['episode']}",'','|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|','|---:|---|---|---:|---:|---:|---|---|']
  for r in sorted(c['individual_runs'],key=lambda x:x['rep']):
   trace=Path(by[r['run_id']]['input']).with_name('trace.json')
   individual.append(f"|{r['rep']}|{fmt(r['qa01_mean_ms'])} ({r['qa01_n']}/{r['qa01_expected']})|{fmt(r['qa03_mean_ms'])} ({r['qa03_n']}/{r['qa03_expected']})|{r['via_cost_usd']:.7f}|{r['receiver_cost_usd']:.7f}|{r['model_calls']}|{r['functional_error']}|[trace]({trace})|")
  individual+=['','|반복별 평균의 분포|관측 반복 수|median|min|max|','|---|---:|---:|---:|---:|']
  for metric,d in c['individual_trial_mean_descriptive'].items():individual.append('|'+metric+'|'+str(d['observed_n'])+'|'+fmt(d['median'])+'|'+fmt(d['min'])+'|'+fmt(d['max'])+'|')
  d=c['failure_input_to_terminal_ms'];individual+=['',f"실제 실패 callback의 입력→실패 시간: n={d['observed_n']}, median={fmt(d['median'])}, min={fmt(d['min'])}, max={fmt(d['max'])}. 성공 완료 시간에 섞지 않았습니다."]
 (folder/'individual-trials.md').write_text('\n'.join(individual)+'\n')
 original=(folder/'report.md').read_text();(folder/'report-analyzer-original.md').write_text(original);(folder/'report.md').write_text(original.replace('Runtime errors','계약/완료 실패 episode').replace('errors are separate','contract/completion failures and actual process exits are separate'))
 if (folder/'PHASE-STOPPED.json').exists():
  stopped=read(folder/'PHASE-STOPPED.json');report=folder/'report.md';report.write_text(report.read_text()+f'\n\n이 phase는 명시적 지시에 따라 {stopped["completed_trials"]}/{stopped["planned_trials"]}에서 경계 정지한 중간 진단 이력이다. 미실행 {stopped["unexecuted_trials"]}개 job은 시작하지 않았고 진행 중 API를 취소하지 않았다. 최종 effective120에 사용하지 않는다. PHASE-STOPPED.json의 원인·고정목록·hash를 따른다.\n')
 save(folder/'export-provenance.json',{'version':'EXTERNAL_PRESENTATION_EXPORT_V2','export_code_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'core_metrics_changed':False,'label_change':'errors means functional contract/completion failure episodes; actual process exit and raw rejection causes separated in failure-causes.json','retained_evidence_scope':'Experiment events/call record retention logical bytes are separate from candidate state; serialized bytes do not partition RSS'})
 (folder/'case-report.md').write_text('\n'.join(lines)+'\n')
 print(json.dumps({'rows':len(rows),'cases':len(cases),'calls':len(stage),'goal_population':len(goals),'response_population':len(stimuli),'memory_rows':len(memory)}))
if __name__=='__main__':p=argparse.ArgumentParser();p.add_argument('phase',type=Path);main(p.parse_args().phase)
