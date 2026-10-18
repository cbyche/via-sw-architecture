"""Post-run aggregation of the separately frozen common-coordinate24 only."""
import collections,hashlib,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
sys.path.insert(0,str(REPO/'benchmark/architecture/four_dp_poc_v2'))
import analyze
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019/dp04'
PHASE=ROOT/'normalized'
def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
    rs=[json.loads(l) for l in (PHASE/'runs.jsonl').read_text().splitlines() if l]
    key=lambda x:(x['dp'],x['episode'],x['variant'],x['repetition'])
    expected=list(map(key,read(PHASE/'schedule.json')));actual=list(map(key,rs))
    checks=[{'check':'all24_unique_planned_keys','pass':len(actual)==len(set(actual))==24 and set(actual)==set(expected)}]
    rows=analyze.analyze(PHASE);metrics=read(PHASE/'episode-metrics.json');manifest=read(PHASE/'freeze.json')
    checks.append({'check':'freeze_binary_plan_schedule','pass':sha(ROOT/'normalized-plan.json')==manifest['plan_sha256'] and sha(PHASE/'schedule.json')==manifest['schedule_sha256'] and all(r['freeze_sha256']==sha(PHASE/'freeze.json') and r['candidate_binary_sha256']==r['postrun_binary_sha256']==manifest['binary']['sha256'] for r in rs)})
    calls=[c for r in rs for c in r['calls']];ids={c['call_uuid'] for c in calls};ledger=read(PHASE/'budget.json')['calls'];local=read(ROOT/'budget.json')['calls'];global_ledger=read(ROOT.parent/'budget.json')['calls']
    checks.append({'check':'all_phase_local_global_call_links','pass':len(ids)==len(calls) and ids==set(ledger) and all(ledger[k]==local[k]==global_ledger[k] for k in ids)})
    checks.append({'check':'phase_cap20c_and_dp04_cap55c','pass':sum(c['amount_usd'] for c in ledger.values())<=.20 and sum(c['amount_usd'] for c in local.values())<=.55})
    for r in rs:
        checks.append({'check':r['run_id']+'_no_free_tactics','pass':r['variant'] in ['A','B'] and r['tactic'] is None and not any(e.get('kind') in ['LOCAL_OWNER_REPAIR_REQUEST','LOCAL_OWNER_PATCH_MERGED'] for e in r['events']) and not any(c.get('coordinate_views') for c in r['calls'])})
    for c in calls:
        request=read(c['request_file']);response=read(c['response_file']) if c.get('response_file') and Path(c['response_file']).is_file() else {}
        checks.append({'check':c['call_uuid']+'_fixed_model_usage','pass':request['model']=='qwen3.7-plus-2026-05-26' and request['enable_thinking'] is False and request['temperature']==0 and response.get('usage')==c.get('raw_usage')})
    eligible=[]
    for s in rows:
        s['common_endpoint_eligible']=s['qa01_n']==s['qa01_expected']==15 and s['qa03_n']==s['qa03_expected']==12 and not s['errors']
        if s['common_endpoint_eligible']:eligible.append(s['variant'])
        s['stages']={}
        for c in (c for r in rs if r['variant']==s['variant'] for c in r['calls']):
            v=s['stages'].setdefault(c['stage'],{'calls':0,'usage_known_calls':0,'usage_unknown_calls':0,'input_tokens':0,'output_tokens':0,'known_usd':0,'service_ms':0,'usage_scope':'reported tokens/cost only; unknown is not zero'})
            v['calls']+=1;v['input_tokens']+=c.get('input_tokens',0);v['output_tokens']+=c.get('output_tokens',0);v['known_usd']+=c.get('cost_usd') or 0
            v['usage_known_calls']+=c.get('cost_usd') is not None;v['usage_unknown_calls']+=c.get('cost_usd') is None
            if c.get('send_ns') is not None:v['service_ms']+=(c['terminal_ns']-c['send_ns'])/1e6
    failures=[{'run_id':r['run_id'],'variant':r['variant'],'episode':r['episode'],'rep':r['repetition'],'events':[e for e in r['events'] if e.get('kind') in ['NORMALIZED_WIRE_REJECTED','GROUNDING_CONSISTENCY_REJECTED','GOAL_FAILED','EPISODE_ERROR']]} for r in rs]
    normalized_events=[e for r in rs for e in r['events'] if e.get('kind')=='COORDINATES_NORMALIZED']
    known=sum(c.get('cost_usd') or 0 for c in calls);unknown=[{'call_uuid':c['call_uuid'],'cost_usd':None,'reservation_upper_bound_usd':ledger[c['call_uuid']]['amount_usd']} for c in calls if c.get('cost_usd') is None]
    result={'scope':'Separate common representation sensitivity; old48/raw/QA02 unchanged; no A2/A5/B4, no oracle conversion/clamping.','analysis_source_sha256':sha(__file__),'phase_freeze_sha256':sha(PHASE/'freeze.json'),'actual_trials':len(rs),'actual_requests':len(calls),'known_usage_requests':len(calls)-len(unknown),'known_cost_usd':known,'unknown_usage':unknown,'known_plus_unknown_reserved_upper_bound_usd':sum(c['amount_usd'] for c in ledger.values()),'options':rows,'eligible_basic_options':eligible,'selection_status':'NO_QUALIFIED_BASIC_WINNER' if not eligible else 'ONE_COMMON_ENDPOINT_ELIGIBLE_BASIC_OPTION' if len(eligible)==1 else 'COMPARE_QA01_FIRST_AMONG_ELIGIBLE','normalization_events':len(normalized_events),'normalized_boxes':sum(len(e['data']['mappings']) for e in normalized_events),'checks':checks,'all_checks_pass':all(c['pass'] for c in checks),'failures':failures,'limits':['Official tool convention is not a snapshot-model API guarantee','No pure causal ownership claim from normalized coordinate presentation','Same finite four cases and three repetitions; no population or statistical significance claim','New QA04 change exercise not run; sampled local Python/Rust RSS excludes cloud weights/UI']}
    (ROOT/'normalized-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    lines=['# 공통 normalized-output A/B24 비교','','기존48과 분리한 새 공통표현. 각 안4사례×3회. QA01 15입력/QA03 12목표/QA02 24집합을 유지했다.','','|기본안|QA01 초·관측/15|QA02 /24|QA03 초·완료/12|호출|확정USD·미확정|최고동시RSS MB|공통endpoint자격|','|---|---|---|---|---:|---|---:|---|']
    for s in rows:
        vu=[u for u in unknown if next(c for c in calls if c['call_uuid']==u['call_uuid'])['variant']==s['variant']]
        cost=f"${s['qa05_via_usd']:.7f}"+(f" +미확정{len(vu)}(예약상한${sum(u['reservation_upper_bound_usd'] for u in vu):.7f})" if vu else '')
        sec=lambda x:'—' if x is None else f'{x/1000:.6f}'
        lines.append(f"|{s['variant']}|{sec(s['qa01_ms'])} {s['qa01_n']}/15|{s['qa02_correct']}/24|{sec(s['qa03_ms'])} {s['qa03_n']}/12|{s['model_calls']}|{cost}|{s['qa06_peak_rss_mb']:.6f}|{s['common_endpoint_eligible']}|")
    lines+=['',f"실제요청{len(calls)}=usage확정{len(calls)-len(unknown)}+미확정{len(unknown)}. 확정환산${known:.7f}; 미확정예약포함상한${sum(c['amount_usd'] for c in ledger.values()):.7f}.",f"증거대조{sum(c['pass'] for c in checks)}/{len(checks)}. 실제정규화사건{len(normalized_events)}. 자격충족 기본안: {eligible or '없음'}.",'','원래pixel지시도명시돼있었으므로미구현필수기능수정으로단정하지않는다. 새결과는단위표현민감도이며기존실패를사후변환/재채점하지않았다. endpoint미관측조건부평균으로전체속도승자를만들지않으며, 둘다실패하면이번추가연구에서확정기본승자는없다.']
    (ROOT/'normalized-comparison.md').write_text('\n'.join(lines)+'\n')
    print(json.dumps({'trials':len(rs),'requests':len(calls),'known_cost_usd':known,'eligible':eligible,'checks_pass':result['all_checks_pass']}))
if __name__=='__main__':main()
