"""Presentation evidence for fixed DP02 cases; no provider calls or candidate edits."""
import argparse,json,statistics
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(Path(p).read_text())
def stats(xs):
 xs=[x for x in xs if x is not None];return {'n':len(xs),'mean':statistics.mean(xs)if xs else None,'median':statistics.median(xs)if xs else None,'min':min(xs)if xs else None,'max':max(xs)if xs else None,'values':xs}
def union(spans):
 total=0;end=None
 for a,b in sorted(spans):
  if end is None or a>end:total+=b-a;end=b
  elif b>end:total+=b-end;end=b
 return total
def fmt(x):return '미관측'if x is None else f'{x:.2f}'
def main(folder):
 rows=[r for r in read(folder/'episode-metrics.json')if r['dp']=='02'];records={r['run_id']:r for r in map(json.loads,(folder/'runs.jsonl').read_text().splitlines())if r['dp']=='02'}
 details=[]
 for row in rows:
  rec=records[row['run_id']];calls=rec['calls'];spans=[(c['send_ns'],c['terminal_ns'])for c in calls if c.get('send_ns')and c.get('terminal_ns')];origin=row['timing_contract']['clock_origin_alignment_ns'];waits=[];starts={}
  for e in rec['events']:
   if e.get('kind')=='POLICY_WAIT_START':starts[(e['data'].get('stimulus_id'),e['data'].get('goal_id'))]=e
   elif e.get('kind')=='POLICY_WAIT_END'and(e['data'].get('stimulus_id'),e['data'].get('goal_id'))in starts:
    start=starts.pop((e['data'].get('stimulus_id'),e['data'].get('goal_id')));waits.append({'stimulus_id':e['data'].get('stimulus_id'),'goal_id':e['data'].get('goal_id'),'reason':start['data'].get('reason'),'scope':'step-boundary'if start['data'].get('stimulus_id')else'pending-input publication gate','elapsed_ms':(e['elapsed_ns']-start['elapsed_ns'])/1e6,'start_elapsed_ns':start['elapsed_ns'],'end_elapsed_ns':e['elapsed_ns']})
  details.append({'run_id':row['run_id'],'episode':row['episode'],'variant':row['variant'],'rep':row['rep'],'raw_input':rec['input'],'raw_trace':str(Path(rec['input']).with_name('trace.json')),'k':read(rec['input']).get('k',2),'qa01':list(row['timing_details_by_stimulus'].values()),'qa03':row['timing_details_by_goal'],'actual_policy_waits':waits,'calls':[{'stage':c['stage'],'call_uuid':c['call_uuid'],'goal_causes':c.get('causal_trace',{}).get('goal_ids'),'slot':c.get('resource_slot'),'resource_predecessor':c.get('resource_predecessor_call_uuid'),'admission_serialization_recording_plus_slot_wait_ms':(c['send_ns']-c['enqueue_ns'])/1e6 if c.get('send_ns')else None,'send_to_terminal_ms':(c['terminal_ns']-c['send_ns'])/1e6 if c.get('send_ns')else None,'send_elapsed_ms':(c['send_ns']-origin)/1e6 if c.get('send_ns')else None,'terminal_elapsed_ms':(c['terminal_ns']-origin)/1e6,'input_tokens':c.get('input_tokens'),'output_tokens':c.get('output_tokens'),'cost_usd':c.get('cost_usd'),'status':c.get('status')}for c in calls],'provider_occupied_union_ms':union(spans)/1e6,'provider_parallel_overlap_ms':(sum(b-a for a,b in spans)-union(spans))/1e6,'semantic_rejections':[e for e in rec['events']if e.get('kind')=='SEMANTIC_STAGE_REJECTED'],'stale_drafts':[e for e in rec['events']if e.get('kind')=='STALE_RESULT_RETAINED_NOT_ADOPTED'],'response_branches':[e for e in rec['events']if e.get('kind')=='RESPONSE_BRANCH'],'nr_details':row['nr_details'],'content':row['content_diagnostic'],'comparison_status':row['comparison_status']})
 groups=[]
 for episode in sorted({r['episode']for r in rows}):
  for variant in 'AB':
   rs=[r for r in rows if r['episode']==episode and r['variant']==variant];ds=[d for d in details if d['episode']==episode and d['variant']==variant]
   if not rs:continue
   firsts={sid:stats([r['timing_details_by_stimulus'].get(sid,{}).get('qa01_ms')for r in rs])for sid in sorted({sid for r in rs for sid in r['timing_details_by_stimulus']})}
   finals={gid:stats([next((g['qa03_ms']for g in r['timing_details_by_goal']if g['goal_id']==gid),None)for r in rs])for gid in sorted({g['goal_id']for r in rs for g in r['timing_details_by_goal']})}
   c=[c for d in ds for c in d['calls']]
   groups.append({'episode':episode,'variant':variant,'trials':len(rs),'qa01_observed':sum(r['qa01_n']for r in rs),'qa01_expected':sum(r['qa01_expected']for r in rs),'qa03_observed':sum(r['qa03_n']for r in rs),'qa03_expected':sum(r['qa03_expected']for r in rs),'first_by_stimulus':firsts,'completion_by_goal':finals,'actual_step_waits_ms':stats([w['elapsed_ms']for d in ds for w in d['actual_policy_waits']if w['scope']=='step-boundary']),'actual_pending_publication_waits_ms':stats([w['elapsed_ms']for d in ds for w in d['actual_policy_waits']if w['scope']=='pending-input publication gate']),'bridge_admission_and_queue_ms':stats([x['admission_serialization_recording_plus_slot_wait_ms']for x in c]),'provider_call_ms':stats([x['send_to_terminal_ms']for x in c]),'parallel_provider_overlap_ms':stats([d['provider_parallel_overlap_ms']for d in ds]),'model_calls':len(c),'input_tokens':sum(x['input_tokens']or 0 for x in c),'output_tokens':sum(x['output_tokens']or 0 for x in c),'cost_usd':sum(x['cost_usd']or 0 for x in c),'nr_correct':sum(r['nr_correct']for r in rs),'nr_total':sum(r['nr_total']for r in rs),'content_correct':sum(r['content_correct']for r in rs),'content_total':sum(r['content_total']for r in rs),'rejections':sum(len(d['semantic_rejections'])for d in ds),'stale_drafts_not_adopted':sum(len(d['stale_drafts'])for d in ds),'comparison_status':'INCOMPLETE'if any(r['comparison_status']=='INCOMPLETE'for r in rs)else'COMPLETE'})
 data={'scope':'24 planned finite actual episodes; no statistical test. Wrong meaningful callback times retained. Failed callbacks separate from successful completion. Step waits precede model admission; enqueue-to-send includes serialization/recording and slot wait, not pure queue. Provider parallel overlap is measured HTTP dependency overlap, not cloud compute overlap.','groups':groups,'individual_runs':details};(ROOT/'analysis/dp02-case-review.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
 lines=['# DP02 사례별 실행 증거','','A는 실제 현재 모델 호출의 step 경계에서 같은 대화를 이어갑니다. B는 요청별 worker가 계속 실행하며 공통 채택 상태에 연결됩니다. 두 안은 동일한 실제 입력, 의도 채택/상세 응답 계약, source 검증과 제한된 복구를 사용합니다.','','|사례|안|반복|QA01 관측/예정|QA03 관측/예정|호출|입력 token|출력 token|VIA $|N-R|내용 marker|거부·복구 시도|상태|','|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|']
 for g in groups:lines.append(f"|{g['episode']}|{g['variant']}|{g['trials']}|{g['qa01_observed']}/{g['qa01_expected']}|{g['qa03_observed']}/{g['qa03_expected']}|{g['model_calls']}|{g['input_tokens']}|{g['output_tokens']}|{g['cost_usd']:.7f}|{g['nr_correct']}/{g['nr_total']}|{g['content_correct']}/{g['content_total']}|{g['rejections']}|{g['comparison_status']}|")
 for g in groups:
  lines+=['',f"## {g['episode']} {g['variant']}",'','|endpoint|관측 n|평균 ms|median ms|min ms|max ms|개별 ms|','|---|---:|---:|---:|---:|---:|---|']
  for kind,values in [('첫 응답',g['first_by_stimulus']),('원래 목표/전달 하위목표 완료',g['completion_by_goal'])]:
   for key,d in values.items():lines.append(f"|{kind} {key}|{d['n']}|{fmt(d['mean'])}|{fmt(d['median'])}|{fmt(d['min'])}|{fmt(d['max'])}|{', '.join(fmt(x)for x in d['values'])}|")
  lines+=['',f"실제 step 대기 표본: {g['actual_step_waits_ms']}; pending-input publication gate: {g['actual_pending_publication_waits_ms']}; bridge admission+직렬화/기록+slot 대기: {g['bridge_admission_and_queue_ms']}; HTTP 구간 중첩: {g['parallel_provider_overlap_ms']}. 이 숫자는 순수 cloud 추론/순수 slot 대기 분해가 아닙니다."]
 lines+=['','E1은 겹치지 않는 같은 코드/문맥 경로이므로 관측 차이를 재사용 구조의 이익으로 단정하지 않습니다. E2는 K=2에서 계속 지시와 별도 짧은 질문을 함께 담은 실제 U2입니다. E3은 같은 입력을 K=1로 실행합니다. E4는 R1 정정, 기존 Task의 Agent 질문 통지, U3의 답변 전달/receipt를 함께 다룹니다. U2는 새 QA03 목표가 아니고 Agent 통지도 새 목표가 아닙니다. U3는 기존 Task 전체 완료가 아니라 ANSWER_DELIVERY_TO_EXISTING_TASK 하위구간입니다. QA02는 N/A이며 N-R는 별도 관계 진단입니다.','', '출처/판본/빈 응답 계약의 bounded 복구와 실제 실패 시각은 원시 trace 및 failed-attempts.csv에 보존합니다. 고정 분모를 유지하며 미완료 조건부 평균을 전체 승리로 해석하지 않습니다.']
 (ROOT/'analysis/dp02-case-review.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'dp02_rows':len(rows),'groups':len(groups)}))
if __name__=='__main__':p=argparse.ArgumentParser();p.add_argument('phase',type=Path);main(p.parse_args().phase)
