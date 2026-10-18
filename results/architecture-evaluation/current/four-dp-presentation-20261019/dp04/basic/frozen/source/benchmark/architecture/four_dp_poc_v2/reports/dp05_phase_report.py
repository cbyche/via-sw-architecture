"""Quick whole-DP05 36-trial report while the separate line diagnostic proceeds."""
import collections,hashlib,json
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());R=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018';P=R/'basic-dp05-reuse-union-corrected-v2'
def read(p):return json.loads(p.read_text())
def matched_goals(rows):
 maps={v:{}for v in 'ABC'}
 for r in rows:
  for g in r['timing_details_by_goal']:maps[r['variant']][(r['episode'],r['rep'],g['goal_id'])]={'run_id':r['run_id'],**g}
 fixed=set(maps['A']);assert all(set(m)==fixed for m in maps.values())and len(fixed)==21
 common=sorted(k for k in fixed if all(m[k]['status']=='OBSERVED'for m in maps.values()))
 def summarize(keys):
  return {v:{'n':len(keys),'via_ms':sum(maps[v][k]['qa03_ms']for k in keys)/len(keys)if keys else None,'full_wall_ms':sum(maps[v][k]['elapsed_ms']for k in keys)/len(keys)if keys else None,'receiver_union_ms':sum(maps[v][k]['receiver_shim_internal_excluded_ms']for k in keys)/len(keys)if keys else None}for v in 'ABC'}
 return {'fixed_goal_rep_keys':21,'all_three_observed':len(common),'aggregate':summarize(common),'cases':[{'episode':ep,'fixed_keys':sum(k[0]==ep for k in fixed),'common_observed':sum(k[0]==ep for k in common),'variants':summarize([k for k in common if k[0]==ep])}for ep in sorted({k[0]for k in fixed})],'individual_common_goals':[{'episode':k[0],'rep':k[1],'goal_id':k[2],'variants':{v:{q:maps[v][k][q]for q in ['run_id','qa03_ms','elapsed_ms','receiver_shim_internal_excluded_ms']}for v in 'ABC'}}for k in common],'excluded_keys':[{'episode':k[0],'rep':k[1],'goal_id':k[2],'observed_by_variant':{v:maps[v][k]['status']for v in 'ABC'}}for k in sorted(fixed-set(common))],'scope':'Post hoc same-goal×repetition observed intersection only; does not replace fixed21 denominators or prove whole-population superiority.'}

def main():
 raw=[json.loads(l)for l in(P/'runs.jsonl').read_text().splitlines()if l];assert len(raw)==36;summary=read(P/'summary.json')['rows'];memory=read(P/'memory-option-peaks.json');out=[]
 for s in summary:
  rs=[r for r in raw if r['variant']==s['variant']];calls=[c for r in rs for c in r['calls']];reads=collections.Counter();build=collections.Counter();receivers=collections.Counter();retained=[];groups={};prep=[]
  for c in calls:
   key='RECEIVER'if c.get('role')=='RECEIVER'else'PREPARATION'if not c.get('causal_trace',{}).get('goal_ids')else'VIA_REQUEST';v=groups.setdefault(key,{'calls':0,'input_tokens':0,'output_tokens':0,'known_usd':0.0,'unknown_usage_calls':0});v['calls']+=1;v['input_tokens']+=c.get('input_tokens')or 0;v['output_tokens']+=c.get('output_tokens')or 0;v['known_usd']+=c.get('cost_usd')or 0;v['unknown_usage_calls']+=c.get('cost_usd')is None
  for r in rs:
   starts={}
   for e in r['events']:
    d=e.get('data',{});k=e.get('kind')
    if k=='HANDOFF_SOURCE_READ':reads[d.get('consumer','UNSPECIFIED')]+=1
    elif k=='HANDOFF_BUILD_BRANCH':build[d.get('branch')]+=1
    elif k=='RECEIVER_BRANCH':receivers[d.get('branch')]+=1
    elif k=='COMMON_ORIGINAL_EVIDENCE_RETAINED':retained.append({'run_id':r['run_id'],**d})
    elif k=='PREPARATION_START':starts[d['preparation_id']]=e['elapsed_ns']
    elif k=='PREPARATION_COMPLETE'and d['preparation_id']in starts:prep.append((e['elapsed_ns']-starts[d['preparation_id']])/1e6)
  out.append({'summary':s,'same_peak_memory':next(x for x in memory if x['variant']==s['variant']),'cost_calls_tokens_by_stage_class':groups,'actual_source_file_read_calls_by_consumer':dict(reads),'build_branches':dict(build),'receiver_branches':dict(receivers),'common_original_retention_reports':retained,'pre_request_preparation_windows':len(prep),'pre_request_preparation_elapsed_sum_ms':sum(prep),'scope':'Each alternative12 authored episodes. Preparation completed before following request stimuli; request times are prepared-state observations. Stage cost includes all preparation, request, repair and receiver calls. Source-file read calls are not physical IO. Full wall/receiver union are conditional on actual observed completion.'})
 j={'all_three_observed_goal_intersection':matched_goals(read(P/'episode-metrics.json')),'version':'FINAL_DP05_WHOLE36_NUMERICAL_REPORT','source_phase':str(P.resolve()),'source_freeze_sha256':hashlib.sha256((P/'freeze.json').read_bytes()).hexdigest(),'rows':out,'line_diagnostic_excluded':True,'code_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()};dest=R/'analysis/dp05-final-numerical-summary.json';dest.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n');f=lambda x:'N/A'if x is None else f'{x:.3f}'
 lines=['# DP05 최종 교정36 수치','','동일4사례×3회×ABC의 새전체비교다. 원래36 및 중간11을 덮지 않았다. 첫 callback proxy/동결 host 완료와 본문 의미 충족은 별도다. QA03는 실제 receiver-shim API구간 union을 제외한 scoped proxy이며 전체wall을옆에둔다. 관측평균은 조건부다.','','|안|QA01 ms 관측/예정|QA03 VIA ms 관측/예정|full wall ms|receiver union ms|N-H 구조|QA04|prep USD|request VIA USD|receiver USD|calls|동시 MB(Python/Rust)|실패 episode|','|---|---|---|---:|---:|---|---:|---:|---:|---:|---:|---|---|']
 notes=[]
 for x in out:
  s=x['summary'];m=x['same_peak_memory'];g=x['cost_calls_tokens_by_stage_class'];lines.append(f"|{s['variant']}|{f(s['qa01_ms'])} {s['qa01_n']}/{s['qa01_expected']}|{f(s['qa03_ms'])} {s['qa03_n']}/{s['qa03_expected']}|{f(s['qa03_full_elapsed_ms'])}|{f(s['receiver_shim_excluded_ms'])}|{s['nh_correct']}/{s['nh_total']}|{f(s['qa04_modules_mean'])}|{g.get('PREPARATION',{}).get('known_usd',0):.7f}|{g.get('VIA_REQUEST',{}).get('known_usd',0):.7f}|{g.get('RECEIVER',{}).get('known_usd',0):.7f}|{s['model_calls']}|{f(m['concurrent_peak_mb'])} ({f(m['python_at_same_peak_mb'])}/{f(m['rust_at_same_peak_mb'])})|{s['errors']}/{s['episodes']}|")
  notes+=['',f"{s['variant']} 실제 원문 read 소비자별: `{json.dumps(x['actual_source_file_read_calls_by_consumer'],ensure_ascii=False)}`; build branch: `{json.dumps(x['build_branches'],ensure_ascii=False)}`."]
 lines+=notes
 matched=j['all_three_observed_goal_intersection'];lines+=['','## ABC 모두 완료가 관측된 동일 목표·반복 교집합','',f"고정21개 goal×rep 중 세 안 모두 완료가 관측된 키는 {matched['all_three_observed']}/21이다. 이 사후 교집합은 누락을 제외한 관측 대조이며 원래의 고정 분모나 전체 조건부 표를 대체하지 않는다.",'','|범위|공통/고정|안|VIA ms|full wall ms|receiver union ms|','|---|---|---|---:|---:|---:|']
 for label,expected,n,values in [('전체',21,matched['all_three_observed'],matched['aggregate'])]+[(c['episode'],c['fixed_keys'],c['common_observed'],c['variants'])for c in matched['cases']]:
  for v,x in values.items():lines.append(f"|{label}|{n}/{expected}|{v}|{f(x['via_ms'])}|{f(x['full_wall_ms'])}|{f(x['receiver_union_ms'])}|")
 lines+=['','QA02는DP05에N/A다. 각안 예정 QA01/03은21/21, 예정허가인계는18이다(CLOSE3은인계없음). 목적lexical/본문literal·동치·수동source검토는JSON summary와별도저자검토를함께읽는다. 메모리는동일run/시각의두역할합계이며원인분해나wholeVIA수치가아니다. 각stage calls/tokens/cost와실제원문획득보고/사전생산시간/최대표본전체는JSON에보존한다.'];(R/'analysis/dp05-final-numerical-summary.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'dp05_trials':36,'summary':str(dest.resolve())}))
if __name__=='__main__':main()
