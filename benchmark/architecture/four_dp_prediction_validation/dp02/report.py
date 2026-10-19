"""Descriptive report from the unchanged frozen evaluator; no API/test execution."""
import json,statistics,hashlib
from pathlib import Path
H=Path(__file__).resolve().parent;R=H.parents[3];ROOT=R/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp02';P=ROOT/'primary'
def read(p):return json.loads(p.read_text())
x=read(P/'summary.json');rows=x['rows'];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def mean(v):return statistics.mean(v)if v else None
def fmt(v,div=1):return f'{v/div:.3f}'if v is not None else'미관측'
def stim(r,s):return next(t['qa01_ms']for t in r['timing']['stimuli']if t['stimulus_id']==s)
def goal(r,g):return next((t['qa03_ms']for t in r['timing']['goals']if t['goal_id']==g),None)
pairs=[]
for a in rows:
 if a['variant']!='A':continue
 b=next((r for r in rows if r['variant']=='B'and r['episode']==a['episode']and r['rep']==a['rep']),None)
 if b is None:continue
 av,bv=stim(a,'U2'),stim(b,'U2');ar,br=goal(a,'R1'),goal(b,'R1');ac,bc=goal(a,'U2'),goal(b,'U2')
 pairs.append({'episode':a['episode'],'rep':a['rep'],'U2first_A_ms':av,'U2first_B_ms':bv,'U2first_B_minus_A_ms':bv-av if bv is not None and av is not None else None,'parent_R1_A_ms':ar,'parent_R1_B_ms':br,'parent_R1_B_minus_A_ms':br-ar if br is not None and ar is not None else None,'child_U2_A_ms':ac,'child_U2_B_ms':bc,'child_U2_B_minus_A_ms':bc-ac if bc is not None and ac is not None else None})
global_=[]
for v in ['A','B']:
 rs=[r for r in rows if r['variant']==v];q1=[t for r in rs for t in r['timing']['qa01_ms']];q3=[t for r in rs for t in r['timing']['qa03_ms']];peak=max((r['qa06_same_sample_peak']for r in rs),key=lambda s:s.get('concurrent_sum_bytes',0),default={})
 global_.append({'variant':v,'runs':len(rs),'qa01_mean_ms':mean(q1),'qa01_observed':len(q1),'qa01_planned':sum(r['timing']['qa01_expected']for r in rs),'qa03_mean_ms':mean(q3),'qa03_observed':len(q3),'qa03_planned':sum(r['timing']['qa03_expected']for r in rs),'qa02':'N/A','qa05_known_usd':sum(r['qa05_known_usd']for r in rs),'qa05_unknown_count':sum(r['qa05_unknown_count']for r in rs),'qa05_unknown_reservation_usd':sum(r['qa05_unknown_reserved_usd']for r in rs),'qa06_max_same_sample_MB':peak.get('concurrent_sum_bytes',0)/1e6,'qa06_roles_at_peak':peak.get('rss_bytes_by_role',{}),'nr_correct':sum(r['nr']['correct']for r in rs),'nr_total':sum(r['nr']['total']for r in rs),'content_marker_correct':sum(r['finite_content']['correct']for r in rs),'content_marker_total':sum(r['finite_content']['total']for r in rs),'calls':sum(r['calls']for r in rs),'failures':sum(len(r['failures'])for r in rs)})
profiles=[]
for eid in dict.fromkeys(r['episode']for r in rows):
 rs=[r for r in rows if r['episode']==eid];ps=[p for p in pairs if p['episode']==eid];d=[p['U2first_B_minus_A_ms']for p in ps if p['U2first_B_minus_A_ms']is not None]
 profiles.append({'episode':eid,'stratum':rs[0]['stratum'],'U2first_paired_B_minus_A_ms':d,'range_ms':[min(d),max(d)]if d else None,'mean_ms':mean(d),'observed_API_overlap_B_ms':[r['max_actual_API_overlap_ms']for r in rs if r['variant']=='B'],'observed_first_call_remaining_at_U2_ms':{v:[r['first_call_remaining_at_U2_ms']for r in rs if r['variant']==v]for v in ['A','B']},'actual_calls_by_alternative':{v:[r['calls']for r in rs if r['variant']==v]for v in ['A','B']},'known_cost_by_alternative_usd':{v:sum(r['qa05_known_usd']for r in rs if r['variant']==v)for v in ['A','B']},'input_tokens_by_alternative':{v:sum(r['input_tokens']for r in rs if r['variant']==v)for v in ['A','B']},'output_tokens_by_alternative':{v:sum(r['output_tokens']for r in rs if r['variant']==v)for v in ['A','B']},'parent_paired_B_minus_A_ms':[p['parent_R1_B_minus_A_ms']for p in ps],'child_paired_B_minus_A_ms':[p['child_U2_B_minus_A_ms']for p in ps]})
report={'source_evaluator_sha256':sha(H/'analyze_v2.py'),'source_report_sha256':sha(Path(__file__)),'global':global_,'pairs':pairs,'profiles':profiles,'pooling':'NO old/new pooling. New conditions only; summary is authored-case mix, profile breakdown is primary.','limits':['Three repeated outputs per authored condition, not independent user population.','Callbacks are instrumented display proxies; no product UI/audio/local Omni.','Currentness check and ctx.complete are not atomic; last-moment concurrent revision races are not generally excluded by this harness.','First trial possible parser interference preserved in isolation-note.json.','Source is compact; planned long call may use one complete-answer branch. Report actual duration/opportunity; no long-working-stage assumption.','Finite content markers do not prove semantic completeness. First long A-r1 output omitted requested three sections and described a30-request limit despite stated36 capacity.','QA04 fresh tasks are code-function checks, separate from model repetitions.']}
# User-requested posthoc sensitivity: retain primary; exclude the first possibly
# interfered A trial, and separately drop its A/B pair to preserve matched weights.
first_id='primary-02-N-long-independent-K1-A-r1-3ece81'
sens=[]
for mode in ['FIRST_A_TRIAL_EXCLUDED','MATCHED_FIRST_PAIR_EXCLUDED']:
 selected=[r for r in rows if r['run_id']!=first_id and not(mode=='MATCHED_FIRST_PAIR_EXCLUDED'and r['episode']=='02-N-long-independent-K1'and r['rep']==1)]
 for v in ['A','B']:
  rs=[r for r in selected if r['variant']==v];q1=[t for r in rs for t in r['timing']['qa01_ms']];q3=[t for r in rs for t in r['timing']['qa03_ms']]
  sens.append({'mode':mode,'variant':v,'runs':len(rs),'qa01_ms':mean(q1),'qa01_observed':len(q1),'qa03_ms':mean(q3),'qa03_observed':len(q3),'qa06_max_same_sample_MB':max((r['qa06_same_sample_peak'].get('concurrent_sum_bytes',0)for r in rs),default=0)/1e6,'scope':'Posthoc sensitivity requested after first-trial interference notice; primary fixed-denominator results remain above. First-A-only changes weights; matched-pair exclusion preserves pair weights.'})
report['isolation_sensitivity']=sens
qa=ROOT/'qa04/summary.json'
if qa.exists():report['qa04']=read(qa)
(ROOT/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
lines=['# DP02 조건부 예측 재검증','',f"새 primary {len(rows)}/72회, 실제 모델 호출{sum(r['calls']for r in rows)}회. 기존 결과와 합치지 않는다. 세 번의 동일 사례 반복이며 사용자 모집단의 통계적 우열을 뜻하지 않는다.",'','| 안 | QA01 초 관측/예정 | QA02 | QA03 초 관측/예정 | QA04 개 | QA05 USD | QA06 최고MB | N-R |','| --- | --- | --- | --- | --- | --- | --- | --- |']
for g in global_:
 q4=report.get('qa04',{}).get('mean_by_alternative',{}).get(g['variant'],'미측정')
 lines.append(f"| {g['variant']} | {fmt(g['qa01_mean_ms'],1000)} ({g['qa01_observed']}/{g['qa01_planned']}) | N/A | {fmt(g['qa03_mean_ms'],1000)} ({g['qa03_observed']}/{g['qa03_planned']}) | {q4} | {g['qa05_known_usd']:.7f} + 미확정{g['qa05_unknown_count']}건 | {g['qa06_max_same_sample_MB']:.3f} | {g['nr_correct']}/{g['nr_total']} |")
lines+=['','완료는 host가 현재 revision의 구조화 출력과 source-quote 검사를 통과해 callback을 낸 사실이다. 본문의 의미적 완결성이 입증된 것은 아니다. 첫 장기 A-r1은 요청한 세 section 대신 두 문장으로 답했고, Cedar의 용량36을 제시한 뒤 30-request limit이라고 잘못 표현했다. 이 결함을 시간 endpoint를 바꾸거나 QA02로 채점하지 않는다.','','## U2 첫 응답과 실제 병행','', '음수는 B가 빠른 관측이다. 각 행은 같은 조건의 세 반복을 A/B로 짝지은 B−A 원값이며 사전 장기 작업 이름만으로 기제가 작동했다고 하지 않는다.','', '| 조건 | U2 B−A 초 세 값 | B API 중첩 초 세 값 | 원래 R1 완료 B−A 초 |','| --- | --- | --- | --- |']
for p in profiles:
 join=lambda vs:', '.join(fmt(v,1000)for v in vs)
 lines.append(f"| {p['episode']} | {join(p['U2first_paired_B_minus_A_ms'])} | {join(p['observed_API_overlap_B_ms'])} | {join(p['parent_paired_B_minus_A_ms'])} |")
lines+=['','## 조건별 실제 호출과 비용','', '| 조건 | A 호출 세 값 | B 호출 세 값 | A 전체 USD | B 전체 USD |','| --- | --- | --- | --- | --- |']
for p in profiles:
 c=p['actual_calls_by_alternative'];cost=p['known_cost_by_alternative_usd'];lines.append(f"| {p['episode']} | {c['A']} | {c['B']} | {cost['A']:.7f} | {cost['B']:.7f} |")
lines+=['','## 첫 trial 간섭 제외 민감도','', '전체 primary 표는 그대로 보존한다. 아래 사후 민감도는 사용자가 요청한 첫 A trial 단독 제외와 대응 A/B pair 제외다. 단독 제외는 조건 가중이 달라지므로 primary의 대체값이 아니다.','', '| 제외 방식 | 안 | 실행 수 | QA01 초 | QA03 초 | QA06 최고 MB |','| --- | --- | --- | --- | --- | --- |']
for v in sens:lines.append(f"| {v['mode']} | {v['variant']} | {v['runs']} | {fmt(v['qa01_ms'],1000)} | {fmt(v['qa03_ms'],1000)} | {v['qa06_max_same_sample_MB']:.3f} |")
lines+=['','## 한계와 보존','',*['- '+s for s in report['limits']],'','전체 원값은 primary/summary.json, report.json의 profile과 pairs, primary/raw/runs/ 및 raw/model/에 남겼다. 예산은 budget.json의 확정 사용량과 미확정 예약을 함께 센다. 실패와 빠진 endpoint의 예정 분모는 고정한다.']
(ROOT/'report.md').write_text('\n'.join(lines)+'\n');print(json.dumps(global_,indent=2))
