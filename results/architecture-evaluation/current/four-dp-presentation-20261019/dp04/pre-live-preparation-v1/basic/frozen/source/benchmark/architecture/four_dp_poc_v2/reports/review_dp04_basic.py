"""Read-only reconstruction of the 24 frozen DP04 BASIC episodes. No model calls."""
import collections
import hashlib
import json
import math
import statistics
import sys
from pathlib import Path

REPO = next(p for p in Path(__file__).resolve().parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
ROOT = REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
PHASE = ROOT / "basic-v1"
sys.path.insert(0, str(REPO / "benchmark/architecture/four_dp_poc_v2"))
from score_dp04 import canonicalize_adoptions

def read(p): return json.loads(p.read_text())
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def mean(v): return statistics.mean(v) if v else None
def close(a, b): return a is b if a is None or b is None else abs(a-b) < .000001
def stages(calls):
    out = {}
    for c in calls:
        s = out.setdefault(c['stage'], dict(calls=0, input_tokens=0, output_tokens=0, usd=0, service_ms=[], queue_ms=[]))
        s['calls'] += 1
        for k in ('input_tokens', 'output_tokens'): s[k] += c[k]
        s['usd'] += c['cost_usd']
        s['service_ms'].append((c['terminal_ns']-c['send_ns'])/1e6)
        s['queue_ms'].append((c['send_ns']-c['enqueue_ns'])/1e6)
    return out

records = [json.loads(x) for x in (PHASE/'runs.jsonl').read_text().splitlines() if x.strip()]
records = [r for r in records if r['dp'] == '04']
assert len(records) == 24
assert len({(r['variant'], r['episode'], r['repetition']) for r in records}) == 24
frozen = PHASE/'frozen/source/benchmark/architecture/four_dp_poc_v2'
oracle = {x['id']:x['adoptions'] for x in read(frozen/'oracles/dp04.json')}
manifest = read(frozen/'goal_manifests/dp04.json')
metrics = {m['run_id']:m for m in read(PHASE/'episode-metrics.json') if m['dp']=='04'}
summary = {m['variant']:m for m in read(PHASE/'summary.json')['rows'] if m['dp']=='04'}
cases = [m for m in read(PHASE/'case-summary.json') if m['dp']=='04']
checks = []
rows = []
for r in records:
    m = metrics[r['run_id']]
    ev = r['events']
    captures = {e['data']['stimulus_id']:e['data']['input_elapsed_ns'] for e in ev if e['kind']=='INPUT_CAPTURED'}
    qa01 = []
    for stimulus in manifest[r['episode']]['qa01_stimuli']:
        display = next((e for e in ev if e['kind']=='DISPLAY_PROXY' and e['data'].get('stimulus_id')==stimulus and e['data'].get('content_type') in ('ADOPTED_HANDLING','DETAILED_RESULT')),None)
        qa01.append({'stimulus_id':stimulus,'ms':(display['elapsed_ns']-captures[stimulus])/1e6 if display else None})
    qa03 = []
    for goal in manifest[r['episode']]['qa03_goals']:
        complete = next((e for e in ev if e['kind']=='GOAL_COMPLETE' and e['data'].get('goal_id')==goal['goal_id'] and e['data'].get('revision')==goal['final_revision']),None)
        qa03.append({'goal_id':goal['goal_id'],'final_revision':goal['final_revision'],'ms':(complete['elapsed_ns']-captures[goal['origin_stimulus_id']])/1e6 if complete else None})
    ads = [e['data'] for e in ev if e['kind']=='ADOPTION']
    normalized = canonicalize_adoptions(r, ads)
    first = {}
    for a in normalized['adoptions']:first.setdefault((a['expression_id'],a['revision']),a)
    exact = []
    for expected in oracle[r['episode']]:
        actual = first.get((expected['expression_id'],expected['revision']))
        ok = actual is not None and not actual.get('_target_identity_unresolved') and set(actual['targets'])==set(expected['targets'])
        exact.append(dict(expected=expected, first_actual=actual, correct=ok))
    rejected = [dict(elapsed_ns=e['elapsed_ns'], **e['data']) for e in ev if e['kind']=='GROUNDING_CONSISTENCY_REJECTED']
    failures=[]
    for e in ev:
        if e['kind']!='GOAL_FAILED':continue
        d=e['data'];sid=d['stimulus_id']
        failures.append(dict(data=d, input_to_failure_ms=(e['elapsed_ns']-captures[sid])/1e6))
    for label, ok in [
        ('exact_first_adoption',sum(x['correct'] for x in exact)==m['qa02_correct'] and len(exact)==m['qa02_total']),
        ('qa01_direct_capture_to_display',sum(x['ms'] is not None for x in qa01)==m['qa01_n'] and len(qa01)==m['qa01_expected'] and close(mean([x['ms'] for x in qa01 if x['ms'] is not None]),m['qa01_mean_ms'])),
        ('qa03_original_capture_to_final_revision',sum(x['ms'] is not None for x in qa03)==m['qa03_n'] and len(qa03)==m['qa03_expected'] and close(mean([x['ms'] for x in qa03 if x['ms'] is not None]),m['qa03_mean_ms'])),
        ('actual_scoped_peak',max(s['concurrent_sum_bytes'] for s in r['memory'])/1e6==m['rss_concurrent_peak_mb']),
        ('no_provider_or_json_failure',r['returncode']==0 and all(c['status']=='COMPLETED' and c['cost_state']=='PROVIDER_USAGE_X_STANDARD_PRICE_NOT_INVOICE' and c['http_status']==200 and isinstance(c.get('response_value'),dict) for c in r['calls'])),
    ]:
        checks.append(dict(run_id=r['run_id'], check=label, pass_=ok))
    finish=[]
    for c in r['calls']:
        response=read(Path(c['response_file']))
        finish.extend(ch.get('finish_reason') for ch in response.get('choices',[]))
    row=dict(run_id=r['run_id'],episode=r['episode'],variant=r['variant'],rep=r['repetition'],raw_trace=str(PHASE/'raw/runs'/r['run_id']/'trace.json'),raw_input=r['input'],qa01=qa01,qa03=qa03,qa02=exact,identity_mappings=normalized['mappings'],unresolved_identities=normalized['unresolved'],stage_totals=stages(r['calls']),rejections=rejected,failures=failures,event_counts=dict(collections.Counter(e['kind'] for e in ev)),replayed_control_events=[e['data'] for e in ev if e['kind']=='REPLAYED_CONTROL_EVENT'],peak_same_sample=m['rss_peak_same_sample'],memory_max_sample_gap_ms=m['memory_max_sample_gap_ms'],finish_reasons=finish,call_references=[dict(call_uuid=c['call_uuid'],stage=c['stage'],request_file=c['request_file'],response_file=c['response_file']) for c in r['calls']])
    rows.append(row)

variants={}
for v in 'AB':
    rr=[r for r in records if r['variant']==v]; mm=[metrics[r['run_id']] for r in rr]
    peak=max(mm,key=lambda x:x['rss_concurrent_peak_mb'])
    logical=collections.defaultdict(int)
    for r in rr:
        for sample in r['memory']:
            categories=collections.Counter()
            for buffer in sample.get('logical_buffers',{}).values():categories[buffer['category']]+=buffer['bytes']
            for category,n in categories.items():logical[category]=max(logical[category],n)
    variants[v]=dict(summary=summary[v],stage_totals=stages([c for r in rr for c in r['calls']]),first_simultaneous_rss_mb=[m['rss_concurrent_first_simultaneous_mb'] for m in mm],peak_run_id=peak['run_id'],peak_same_sample=peak['rss_peak_same_sample'],sample_count=sum(m['memory_sample_count'] for m in mm),max_sample_gap_ms=max(m['memory_max_sample_gap_ms'] for m in mm),logical_category_maxima_bytes_not_simultaneous_sum=dict(logical),rejection_count=sum(e['kind']=='GROUNDING_CONSISTENCY_REJECTED' for r in rr for e in r['events']),failure_input_to_terminal_ms=[f['input_to_failure_ms'] for row in rows if row['variant']==v for f in row['failures']])
paired={}
for axis in ('qa01','qa03'):
    grouped=collections.defaultdict(dict)
    idkey='stimulus_id' if axis=='qa01' else 'goal_id'
    for row in rows:
        for x in row[axis]:grouped[(row['episode'],row['rep'],x[idkey])][row['variant']]=x['ms']
    eligible=[dict(episode=k[0],rep=k[1],endpoint=k[2],**v) for k,v in grouped.items() if all(v.get(x) is not None for x in 'AB')]
    paired[axis]=dict(scope='Post-observation same-endpoint intersection, descriptive only; does not replace fixed denominators.',n=len(eligible),A_ms=mean([x['A'] for x in eligible]),B_ms=mean([x['B'] for x in eligible]),rows=eligible)

observations=collections.defaultdict(dict)
for r in records:
    value=read(Path(r['input']));value.pop('variant',None)
    observations[(r['episode'],r['repetition'])][r['variant']]=value
caps={'dp04.joint_grounding':2600,'dp04.joint_source_reread':2600,'dp04.screen_producer':3000,'dp04.speech_producer':1400,'dp04.candidate_binding':2000}
call_settings=[]
for r in records:
    for c in r['calls']:
        request=read(Path(c['request_file']))
        call_settings.append({'call_uuid':c['call_uuid'],'request_sha256':sha(Path(c['request_file'])),'response_sha256':sha(Path(c['response_file'])),'pass_':request['model']=='qwen3.7-plus-2026-05-26' and request['enable_thinking'] is False and request['temperature']==0 and request['max_tokens']==caps[c['stage']]})
scope_checks={'same_actual_observations_all12_AB_pairs':len(observations)==12 and all(x.get('A')==x.get('B') for x in observations.values()),'fixed_model_settings_and_disclosed_stage_caps_all64':len(call_settings)==64 and all(x['pass_'] for x in call_settings),'all64_finish_reason_stop':all(x=='stop' for row in rows for x in row['finish_reasons'])}
out=dict(scope='BASIC DP04 24 actual episodes; no API/candidate execution or input/oracle modification. Identity canonicalizer reused; first-set/timing reconstruction independent.',inputs={str(p.relative_to(ROOT)):sha(p) for p in [PHASE/'freeze.json',frozen/'oracles/dp04.json',frozen/'goal_manifests/dp04.json']},records_sha256={r['run_id']:sha(PHASE/'raw/runs'/r['run_id']/'trace.json') for r in records},checks=checks,scope_checks=scope_checks,call_provenance=call_settings,all_checks_pass=all(x['pass_'] for x in checks) and all(scope_checks.values()),variants=variants,cases=cases,paired_observed_endpoints=paired,rows=rows)
target=ROOT/'analysis/dp04-basic-review.json';target.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'output':str(target),'rows':len(rows),'checks':len(checks),'failures':[x for x in checks if not x['pass_']],'paired':{k:{q:v[q] for q in ('n','A_ms','B_ms')} for k,v in paired.items()}},ensure_ascii=False))
