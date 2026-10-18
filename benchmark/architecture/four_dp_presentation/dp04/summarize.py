"""Read-only measurements aggregation; no model or candidate execution."""
import collections,hashlib,json,statistics,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
sys.path.insert(0,str(REPO/'benchmark/architecture/four_dp_poc_v2'))
import analyze
from evidence import read_evaluator
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019/dp04'
def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,x):p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def main():
    rows=[];records=[];checks=[]
    for phase_name in ['basic','tactics']:
        phase=ROOT/phase_name
        rs=[json.loads(l) for l in (phase/'runs.jsonl').read_text().splitlines() if l]
        key=lambda r:(r['dp'],r['episode'],r['variant'],r['repetition'])
        expected=list(map(key,read(phase/'schedule.json')));actual=list(map(key,rs))
        checks.append({'check':phase_name+'_exact_planned_keys','pass':len(actual)==24 and len(set(actual))==24 and set(actual)==set(expected)})
        checks.append({'check':phase_name+'_freeze_and_binary','pass':all(r['freeze_sha256']==sha(phase/'freeze.json') and r['candidate_binary_sha256']==r['postrun_binary_sha256']==read(phase/'freeze.json')['binary']['sha256'] for r in rs)})
        for r in rs:
            oracle=read_evaluator(r,Path('oracles/dp04.json'));oracle={x['id']:x for x in oracle} if isinstance(oracle,list) else oracle
            rows.append(analyze.row(r,oracle[r['episode']]))
        records.extend(rs)
    summary=[]
    for variant in ['A','B','A2A5','B4']:
        vs=[r for r in rows if r['variant']==variant];rr=[r for r in records if r['variant']==variant]
        n1=sum(r['qa01_n'] for r in vs);n3=sum(r['qa03_n'] for r in vs)
        s={'variant':variant,'episodes':len(vs),'qa01_observed':n1,'qa01_expected':sum(r['qa01_expected'] for r in vs),'qa01_conditional_mean_ms':sum(r['qa01_sum_ms'] for r in vs)/n1 if n1 else None,'qa03_completed':n3,'qa03_expected':sum(r['qa03_expected'] for r in vs),'qa03_conditional_mean_ms':sum(r['qa03_sum_ms'] for r in vs)/n3 if n3 else None,'qa02_correct':sum(r['qa02_correct'] for r in vs),'qa02_total':sum(r['qa02_total'] for r in vs),'via_usd':sum(r['via_cost_usd'] for r in vs),'calls':sum(r['model_calls'] for r in vs),'unknown_usage_calls':sum(r['unknown_usage_calls'] for r in vs),'max_concurrent_rss_mb':max(r['rss_concurrent_peak_mb'] for r in vs),'missing_rss_runs':sum(r['memory_state']!='SAMPLED_RSS_ESTIMATE' for r in vs),'qa04_new_exercise':'NOT_RUN; new tactic N/A; old A/B evidence remains separate','stages':{},'cases':[]}
        s['common_endpoint_eligible']=n1==s['qa01_expected']==15 and n3==s['qa03_expected']==12 and not any(r['functional_error'] for r in vs)
        for c in [c for r in rr for c in r['calls']]:
            stage=s['stages'].setdefault(c['stage'],{'calls':0,'usage_known_calls':0,'usage_unknown_calls':0,'input_tokens':0,'output_tokens':0,'usd':0,'service_ms':0,'render_ms':0,'token_and_usd_scope':'provider-reported usage only; missing usage is not zero'})
            stage['calls']+=1;stage['input_tokens']+=c.get('input_tokens',0);stage['output_tokens']+=c.get('output_tokens',0);stage['usd']+=c.get('cost_usd') or 0
            stage['usage_known_calls']+=c.get('cost_usd') is not None;stage['usage_unknown_calls']+=c.get('cost_usd') is None
            if c.get('send_ns') is not None:stage['service_ms']+=(c['terminal_ns']-c['send_ns'])/1e6
            t=c.get('coordinate_view_render')
            if t:stage['render_ms']+=(t['end_ns']-t['start_ns'])/1e6
        for episode in ['dp04-dense','dp04-free-region','dp04-page-change','dp04-partial-correction']:
            cs=[r for r in vs if r['episode']==episode];a=sum(r['qa01_n'] for r in cs);b=sum(r['qa03_n'] for r in cs)
            s['cases'].append({'episode':episode,'qa01_n':a,'qa01_expected':sum(r['qa01_expected'] for r in cs),'qa01_ms':sum(r['qa01_sum_ms'] for r in cs)/a if a else None,'qa03_n':b,'qa03_expected':sum(r['qa03_expected'] for r in cs),'qa03_ms':sum(r['qa03_sum_ms'] for r in cs)/b if b else None,'qa02_correct':sum(r['qa02_correct'] for r in cs),'qa02_total':sum(r['qa02_total'] for r in cs),'cost_usd':sum(r['via_cost_usd'] for r in cs)})
        summary.append(s)
        checks.append({'check':variant+'_fixed_denominators','pass':(s['episodes'],s['qa01_expected'],s['qa03_expected'],s['qa02_total'])==(12,15,12,24)})
    matched=[]
    for left,right in [('A','B'),('A','A2A5'),('B','B4'),('A2A5','B4')]:
        def endpoints(v):return {(r['episode'],r['rep'],sid):x['qa01_ms'] for r in rows if r['variant']==v for sid,x in r['timing_details_by_stimulus'].items() if x['qa01_ms'] is not None}
        aa,bb=endpoints(left),endpoints(right);keys=aa.keys()&bb.keys()
        matched.append({'left':left,'right':right,'same_observed_inputs':len(keys),'left_ms':statistics.mean(aa[k] for k in keys) if keys else None,'right_ms':statistics.mean(bb[k] for k in keys) if keys else None,'scope':'post-observation intersection descriptive only; never replaces fixed15 inputs'})
    calls=[c for r in records for c in r['calls']];full_ledger=read(ROOT/'budget.json')['calls'];ids={c['call_uuid'] for c in calls};ledger={k:v for k,v in full_ledger.items() if k in ids}
    checks.append({'check':'local_ledger_unique_call_link','pass':len(ids)==len(calls) and ids==set(ledger)})
    checks.append({'check':'known_plus_unknown_reserved_within_budget','pass':sum(c['amount_usd'] for c in ledger.values())<=.55})
    unknown=[{'call_uuid':c['call_uuid'],'run_id':c['run_id'],'reserved_upper_bound_usd':ledger[c['call_uuid']]['amount_usd'],'http_status':c.get('http_status'),'cost_usd':None} for c in calls if c.get('cost_usd') is None]
    for c in calls:
        response=read(c['response_file']);request=read(c['request_file'])
        checks.append({'check':'fixed_provider_'+c['call_uuid'],'pass':request['model']=='qwen3.7-plus-2026-05-26' and request['enable_thinking'] is False and request['temperature']==0 and response.get('usage')==c.get('raw_usage')})
    failures=[{'run_id':r['run_id'],'episode':r['episode'],'variant':r['variant'],'events':[e for e in r['events'] if e.get('kind') in ['GOAL_FAILED','EPISODE_ERROR','GROUNDING_CONSISTENCY_REJECTED','GROUNDING_OUTPUT_REJECTED']]} for r in records]
    output={'plan_sha256':sha(ROOT/'plan.json'),'analysis_source_sha256':sha(__file__),'summary':summary,'matched_observed_inputs':matched,'checks':checks,'all_checks_pass':all(c['pass'] for c in checks),'actual_runs':len(records),'actual_calls':len(calls),'actual_known_cost_usd':sum(c.get('cost_usd') or 0 for c in calls),'unknown_usage_calls':unknown,'known_plus_retained_unknown_reservation_usd':sum(c['amount_usd'] for c in ledger.values()),'limitations':['HTTP failure without usage retains its reservation upper bound; not asserted free or priced as zero.'] if unknown else [],'scope':'Fresh48; unchanged evaluator/scorer semantics; no imputed failures, additional model judge or new metric. Endpoint qualification is distinct from QA02 accuracy.'}
    save(ROOT/'comparison.json',output);save(ROOT/'episode-metrics.json',rows);save(ROOT/'failure-evidence.json',failures)
    lines=['# DP04: fresh BASIC and bounded coordinate repair','','All48 actual outcomes; fixed15 inputs/12 goals/24 exact sets per configuration. Conditional means remain conditional.','','|Configuration|QA01 seconds observed/15|QA02 correct/24|QA03 seconds complete/12|Calls|USD|Peak RSS MB|Common endpoints eligible|','|---|---|---|---|---:|---:|---:|---|']
    fmt=lambda x:'—' if x is None else f'{x/1000:.6f}'
    for s in summary:
        variant_unknown=[x for x in unknown if next(c for c in calls if c['call_uuid']==x['call_uuid'])['variant']==s['variant']]
        cost=f"known ${s['via_usd']:.7f}"+(f" + UNKNOWN {len(variant_unknown)} (reservation upper ${sum(x['reserved_upper_bound_usd'] for x in variant_unknown):.7f})" if variant_unknown else '')
        lines.append(f"|{s['variant']}|{fmt(s['qa01_conditional_mean_ms'])} {s['qa01_observed']}/15|{s['qa02_correct']}/24|{fmt(s['qa03_conditional_mean_ms'])} {s['qa03_completed']}/12|{s['calls']}|{cost}|{s['max_concurrent_rss_mb']:.6f}|{s['common_endpoint_eligible']}|")
    lines+=['','A2+A5 combines affected-expression patching and new coordinate presentation. It does not isolate their individual causal effects. Shared contract, source revisions, count/type/geometry checks and first-adoption scoring remain unchanged. QA04 was not newly exercised. RSS is sampled local Python/Rust, excluding cloud weights/UI.','',f"Checks: {sum(c['pass'] for c in checks)}/{len(checks)}. Known usage at fixed comparison rates: ${output['actual_known_cost_usd']:.7f}; unknown usage calls: {len(unknown)}, retained upper-bound reservations: ${sum(x['reserved_upper_bound_usd'] for x in unknown):.7f}. Unknown is not zero cost."]
    lines += ['',f"Actual provider requests: {len(calls)} = {len(calls)-len(unknown)} usage-confirmed + {len(unknown)} usage-unknown. These are distinct from {len(checks)} read-only evidence checks."]
    (ROOT/'comparison.md').write_text('\n'.join(lines)+'\n')
    print(json.dumps({'runs':len(records),'calls':len(calls),'cost_usd':output['actual_known_cost_usd'],'checks_pass':output['all_checks_pass']}))
if __name__=='__main__':main()
