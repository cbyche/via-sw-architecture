"""Compare each authorized tactic only with its matching BASIC case×3; preserve missing endpoints."""
import collections,csv,json,statistics
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(Path(p).read_text())
def fmt(x):return 'NOT_OBSERVED'if x is None else f'{x:.3f}'
def detail(rs):
 n1=sum(r['qa01_n']for r in rs);n3=sum(r['qa03_n']for r in rs)
 return {'episodes':len(rs),'qa01_ms':sum(r['qa01_sum_ms']for r in rs)/n1 if n1 else None,'qa01_observed':n1,'qa01_expected':sum(r['qa01_expected']for r in rs),'qa03_ms':sum(r['qa03_sum_ms']for r in rs)/n3 if n3 else None,'qa03_observed':n3,'qa03_expected':sum(r['qa03_expected']for r in rs),'qa03_full_elapsed_ms':sum(r['qa03_full_elapsed_sum_ms']for r in rs)/n3 if n3 else None,'receiver_excluded_ms':sum(r['receiver_shim_excluded_sum_ms']for r in rs)/n3 if n3 else None,'qa02_correct':sum(r['qa02_correct']for r in rs),'qa02_expected':sum(r['qa02_total']for r in rs),'calls':sum(r['model_calls']for r in rs),'via_usd':sum(r['via_cost_usd']for r in rs),'receiver_usd':sum(r['receiver_cost_usd']for r in rs),'max_rss_mb':max((r['rss_concurrent_peak_mb']or 0 for r in rs),default=None),'rss_incomplete_trials':sum(r['memory_state']!='SAMPLED_RSS_ESTIMATE'for r in rs),'failed_contract_or_completion_episodes':sum(r['functional_error']for r in rs),'literal_content_correct':sum(r['content_correct']for r in rs),'literal_content_expected':sum(r['content_total']for r in rs),'individual_rows':[{k:r[k]for k in ['run_id','rep','qa01_mean_ms','qa01_n','qa01_expected','qa03_mean_ms','qa03_n','qa03_expected','qa03_full_elapsed_mean_ms','model_calls','via_cost_usd','receiver_cost_usd','functional_error','content_correct','content_total']}for r in rs]}
def mechanism(rs):
 counts=collections.Counter();stages=collections.Counter();branch=collections.Counter();fragments=[];reads=[]
 for r in rs:
  for c in r['calls']:stages[c['stage']]+=1
  for e in r['events']:
   kind=e.get('kind');counts[kind]+=1;d=e.get('data',{})
   if kind=='EVIDENCE_BRANCH':branch[d.get('branch')]+=1
   if kind=='WORKING_VIEW_FRAGMENT_INVALIDATION':fragments.append({'run_id':r['run_id'],**d})
   if kind=='SOURCE_READ':reads.append({'run_id':r['run_id'],**d})
 return {'stage_call_counts':dict(stages),'evidence_branches':dict(branch),'source_file_read_calls':counts['SOURCE_READ'],'actual_fragment_invalidations':fragments,'source_file_reads':reads,'scope':'Actual source-file read calls, not physical disk I/O. Retained fragments are actual state, not assumed model reuse/hits; direct-complete branches and usage are shown separately.'}
def main():
 basic=read(ROOT/'basic-v1/episode-metrics.json');basicraw=[json.loads(x)for x in(ROOT/'basic-v1/runs.jsonl').read_text().splitlines()if x];out=[]
 for name in ['dp04-A4','dp04-B4','dp01-A2']:
  p=ROOT/('tactic-'+name)
  if not(p/'episode-metrics.json').exists():continue
  rows=read(p/'episode-metrics.json');raw=[json.loads(x)for x in(p/'runs.jsonl').read_text().splitlines()if x];basevariant='A'if name!='dp04-B4'else'B'
  for ep in sorted({r['episode']for r in rows}):
   after=[r for r in rows if r['episode']==ep];before=[r for r in basic if r['dp']==after[0]['dp']and r['episode']==ep and r['variant']==basevariant]
   assert len(before)==3
   entry={'tactic':name,'dp':after[0]['dp'],'episode':ep,'phase':after[0]['phase'],'baseline_variant':basevariant,'actual_variant':after[0]['variant'],'before':detail(before),'after':detail(after),'baseline_mechanism':mechanism([r for r in basicraw if r['run_id']in{x['run_id']for x in before}]),'tactic_mechanism':mechanism([r for r in raw if r['run_id']in{x['run_id']for x in after}]),'baseline_summary':str(ROOT/'basic-v1/case-summary.json'),'tactic_summary':str(p/'case-summary.json'),'scope':'Same authored case×3 only; missing endpoints remain missing and invalidate unconditional-winner claims. No significance test.'}
   pair=[]
   for field,idkey,timekey in [('timing_details_by_goal','goal_id','qa03_ms'),('timing_details_by_stimulus','stimulus_id','qa01_ms')]:
    maps=[]
    for rs in [before,after]:
     m={}
     for r in rs:
      values=r[field].values()if isinstance(r[field],dict)else r[field]
      for v in values:m[(r['rep'],v[idkey])]=v.get(timekey)
     maps.append(m)
    keys=set(maps[0])|set(maps[1]);both=[k for k in keys if all(m.get(k)is not None for m in maps)]
    pair.append({'metric':timekey,'fixed_keys':len(keys),'both_observed':len(both),'only_before':sum(maps[0].get(k)is not None and maps[1].get(k)is None for k in keys),'only_after':sum(maps[1].get(k)is not None and maps[0].get(k)is None for k in keys),'before_conditional_paired_ms':statistics.mean(maps[0][k]for k in both)if both else None,'after_conditional_paired_ms':statistics.mean(maps[1][k]for k in both)if both else None})
   entry['paired_observation_diagnostic']=pair;out.append(entry)
 result={'version':'SAME_CASE_ONLY_POST_BASIC_TACTICS','baseline_unchanged':True,'expected_post_basic_comparison_episodes':27,'expected_tactic_episodes':15,'expected_parallelism_diagnostic_episodes':12,'rows':out,'adoption_decision':'Parent/user review after actual effects; no automatic adoption.'}
 (ROOT/'tactic-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
 lines=['# 같은 사례의 기본안과 보완 비교','','각 행은 동일 사례의 BASIC 3회와 별도 보완 3회다. 기본안 전체12회 평균과 보완3회를 섞지 않는다. 관측값만의 평균은 조건부이며 누락·오답·계약 실패를 함께 읽는다. B4는 직렬/병행 정책 기여분 진단이며 기본 B를 대체하지 않는다. QA04 보완 후 변경과제는 별도 실행하지 않았으므로 이 표에 기본 QA04를 전이하지 않는다.','','|보완|사례|구분|첫 응답 ms (관측/예정)|완료 ms (관측/예정)|집합 정답/예정|호출|VIA USD|수신자 USD|최대 동시 MB|계약/완료 실패|','|---|---|---|---|---|---:|---:|---:|---:|---:|---:|']
 for e in out:
  for label,key in [('BASIC','before'),(e['tactic'],'after')]:
   x=e[key];lines.append(f"|{e['tactic']}|{e['episode']}|{label}|{fmt(x['qa01_ms'])} ({x['qa01_observed']}/{x['qa01_expected']})|{fmt(x['qa03_ms'])} ({x['qa03_observed']}/{x['qa03_expected']})|{x['qa02_correct']}/{x['qa02_expected']}|{x['calls']}|{x['via_usd']:.7f}|{x['receiver_usd']:.7f}|{fmt(x['max_rss_mb'])}|{x['failed_contract_or_completion_episodes']}/{x['episodes']}|")
 lines+=['','## 개별 반복과 실제 기전','','같은 키의 endpoint 관측 pairing과 전체 개별3회 원값은 tactic-comparison.json에 보존한다. 각 phase의 individual-trials.md에는 각3회·median/min/max·raw 링크가 있다. 의미 내용의 literal/equivalence/저자 검토는 구조 QA02 및 계측 완료와 분리한다.']
 for e in out:
  lines+=['',f"### {e['tactic']} · {e['episode']}",'',f"실제 SOURCE_READ 호출: BASIC {e['baseline_mechanism']['source_file_read_calls']} → 보완 {e['tactic_mechanism']['source_file_read_calls']}. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.",'',f"BASIC 기전: `{json.dumps(e['baseline_mechanism']['stage_call_counts'],ensure_ascii=False)}`",'',f"보완 기전: `{json.dumps(e['tactic_mechanism']['stage_call_counts'],ensure_ascii=False)}`"]
 (ROOT/'tactic-comparison.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'comparisons':len(out),'actual_post_basic_comparison_episodes':sum(len(r['after']['individual_rows'])for r in out)}))
if __name__=='__main__':main()
