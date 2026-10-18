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
    # Descriptive paired populations preserve missing planned observations.
    common=[m for m in metrics if m['episode']!='dp04-dense']
    paired={v:{'qa01_n':sum(m['qa01_n'] for m in common if m['variant']==v),'qa01_ms':sum(m['qa01_sum_ms'] for m in common if m['variant']==v)/sum(m['qa01_n'] for m in common if m['variant']==v),'qa03_n':sum(m['qa03_n'] for m in common if m['variant']==v),'qa03_ms':sum(m['qa03_sum_ms'] for m in common if m['variant']==v)/sum(m['qa03_n'] for m in common if m['variant']==v)} for v in ['A','B']}
    failed=[]
    for r in rs:
        if not any(e['kind']=='GOAL_FAILED' for e in r['events']):continue
        capture=next(e for e in r['events'] if e['kind']=='INPUT_CAPTURED')
        terminal=next(e for e in r['events'] if e['kind']=='GOAL_FAILED')
        pubs=[e['data']['state'] for e in r['events'] if e['kind']=='SCREEN_CANDIDATES_PUBLISHED']
        failed.append({'run_id':r['run_id'],'episode':r['episode'],'rep':r['repetition'],'failure_observed_ms':(terminal['elapsed_ns']-capture['elapsed_ns'])/1e6,'initial_target_ids':[g['target_ids'] for g in pubs[0]['cue_relations']],'d11_initial_extent':[c for c in pubs[0]['candidates'] if 'D11' in c['target_id']],'expanded_cue_relations':pubs[-1]['cue_relations'],'class':'CORRECT_TARGET_PROPOSAL_BUT_GEOMETRY_INCONSISTENT_THEN_OWNER_UNRESOLVED','provider_or_json_failure':False})
    case_rows=[]
    for episode in sorted({m['episode'] for m in metrics}):
        for v in ['A','B']:
            subset=[m for m in metrics if m['episode']==episode and m['variant']==v]
            case_rows.append({'episode':episode,'variant':v,'qa01_n':sum(m['qa01_n'] for m in subset),'qa01_expected':sum(m['qa01_expected'] for m in subset),'qa02_correct':sum(m['qa02_correct'] for m in subset),'qa02_total':sum(m['qa02_total'] for m in subset),'qa03_n':sum(m['qa03_n'] for m in subset),'qa03_expected':sum(m['qa03_expected'] for m in subset),'qa01_ms':sum(m['qa01_sum_ms'] for m in subset)/sum(m['qa01_n'] for m in subset) if sum(m['qa01_n'] for m in subset) else None,'qa03_ms':sum(m['qa03_sum_ms'] for m in subset)/sum(m['qa03_n'] for m in subset) if sum(m['qa03_n'] for m in subset) else None})
    result.update({'paired_common_observed':{'scope':'Posthoc descriptive common free/page/correction observations; never replaces planned15/12 or missing dense trials','options':paired},'failed_trials_detail':failed,'case_rows':case_rows,'qa04':'NOT_MEASURED_IN_THIS_COMMON_REPRESENTATION_VERSION; previous basic A/B2.5 is historical only','actual_selection':'A conditional finite-case basic selection under common normalized-output contract: A15/15 callbacks,12/12 goals,24/24 exact sets; B dense3 incomplete. Do not generalize observed means into a whole-population speed winner. Original pixel48 remains unchanged and both basics incomplete. Normalized A+A2/A5 and normalized B4 were not executed.'})
    (ROOT/'normalized-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    lines=['# 공통 normalized-output A/B24 비교','','기존48과 분리한 새 공통표현. 각 안4사례×3회. QA01 15입력/QA03 12목표/QA02 24집합을 유지했다.','','|기본안|QA01 초·관측/15|QA02 /24|QA03 초·완료/12|호출|확정USD·미확정|최고동시RSS MB|공통endpoint자격|','|---|---|---|---|---:|---|---:|---|']
    for s in rows:
        vu=[u for u in unknown if next(c for c in calls if c['call_uuid']==u['call_uuid'])['variant']==s['variant']]
        cost=f"${s['qa05_via_usd']:.7f}"+(f" +미확정{len(vu)}(예약상한${sum(u['reservation_upper_bound_usd'] for u in vu):.7f})" if vu else '')
        sec=lambda x:'—' if x is None else f'{x/1000:.6f}'
        lines.append(f"|{s['variant']}|{sec(s['qa01_ms'])} {s['qa01_n']}/15|{s['qa02_correct']}/24|{sec(s['qa03_ms'])} {s['qa03_n']}/12|{s['model_calls']}|{cost}|{s['qa06_peak_rss_mb']:.6f}|{s['common_endpoint_eligible']}|")
    lines+=['',f"실제요청{len(calls)}=usage확정{len(calls)-len(unknown)}+미확정{len(unknown)}. 확정환산${known:.7f}; 미확정예약포함상한${sum(c['amount_usd'] for c in ledger.values()):.7f}.",f"증거대조{sum(c['pass'] for c in checks)}/{len(checks)}. 실제정규화사건{len(normalized_events)}. 자격충족 기본안: {eligible or '없음'}.",'','원래pixel지시도명시돼있었으므로미구현필수기능수정으로단정하지않는다. 새결과는단위표현민감도이며기존실패를사후변환/재채점하지않았다. endpoint미관측조건부평균으로전체속도승자를만들지않으며, 둘다실패하면이번추가연구에서확정기본승자는없다.']
    lines[-1]='실제 판정: 공통 정규화 출력 계약에서 A를 이 유한 사례의 기본안으로 조건부 선택한다. A는 endpoint와 QA02를 전부 관측했고, B는 dense3회 미완료다. B의 관측 평균을 전체 모집단의 속도로 일반화하지 않는다. 기존 pixel48에서는 양쪽 기본안 미완료라는 결과를 유지한다. 정규화 A+A2/A5와 정규화 B4는 미실행이며 기존 tactic 성과를 소급하지 않는다.'
    lines+=['','## 같은 관측의 설명용 비교','','dense를 제외한 free/page/정정의 공통 QA01 12입력·QA03 9목표다. 사후 설명용 교집합이며 원래15/12 분모와 미완료3회를 대체하지 않는다.','','|기본안|공통첫12 평균초|공통최종9 평균초|','|---|---:|---:|']
    lines += [f"|{v}|{paired[v]['qa01_ms']/1000:.6f}|{paired[v]['qa03_ms']/1000:.6f}|" for v in ['A','B']]
    lines+=['','## B의 dense 미완료3회','','세 반복 모두 초기 D06/D11 대상 제안은 정답이었으나 D11의 y정규화 범위 [530,690]→pixel [344,448]이 실제 pointer-2 y=480을 포함하지 못했다(x범위는 반복별로 다르며 JSON에 원값 보존). 공통 기하 gate가 거절했고, 원 screen 생산자 1회 확장 뒤 pointer-2가 unknown/target_ids=[]로 남아 EXPANSION_BUDGET_EXHAUSTED가 됐다. 모델 대상 제안 오류나 provider/JSON 실패와 구분한다. 좌표 변환은 수식대로이며 모델 bbox 자체의 위치 오판이다.','','|실패반복|입력→실패관측 초|','|---|---:|']
    lines += [f"|r{x['rep']}|{x['failure_observed_ms']/1000:.6f}|" for x in failed]
    lines+=['','## 단계/나머지 QA','','QA04는 이번 공통표현 판본에서 미측정이다. 이전 A/B2.5 동점은 이전 change exercise에만 해당한다. QA06은 fresh Python bridge+Rust 동시 RSS 표본 추정이며 cloud/UI 제외, 작은 MB 차이를 구조 우월성으로 단정하지 않는다.','','|안·단계|호출|입력token|출력token|확정USD|누적HTTP초|','|---|---:|---:|---:|---:|---:|']
    for s in rows:
        for stage,t in s['stages'].items():lines.append(f"|{s['variant']} {stage}|{t['calls']}|{t['input_tokens']}|{t['output_tokens']}|{t['known_usd']:.7f}|{t['service_ms']/1000:.6f}|")
    lines+=['','누적HTTP는 역할별 실제 호출 합이며 QA01/QA03 endpoint 시간과 다르다. 대상/기하 계약을 통과한 채택이 callback이며 일반 업무의 내용 정확도/제품 완성이 아니다. 구조 소유권 하나의 정확도 인과를 확정하지 않는다.','']
    (ROOT/'normalized-comparison.md').write_text('\n'.join(lines)+'\n')
    print(json.dumps({'trials':len(rs),'requests':len(calls),'known_cost_usd':known,'eligible':eligible,'checks_pass':result['all_checks_pass']}))
if __name__=='__main__':main()
