"""Recompute finite-fixture metrics from unaltered traces; no model judge."""
import argparse,csv,json,statistics
from collections import defaultdict
from pathlib import Path
from bridge import dump
BASE=Path(__file__).resolve().parent

def mean(xs):return statistics.mean(xs) if xs else None
def keyed(path):
    v=json.loads(path.read_text());return {x['id']:x for x in v} if isinstance(v,list) else v
def union_duration(spans,lo,hi):
    spans=sorted((max(lo,a),min(hi,b)) for a,b in spans if b>lo and a<hi);total=0;end=lo
    for a,b in spans:
        if b>max(a,end):total+=b-max(a,end);end=max(end,b)
    return total

def subtract_occupied(spans,occupied):
    remaining=[]
    for a,b in spans:
        pieces=[(a,b)]
        for x,y in occupied:
            next_pieces=[]
            for lo,hi in pieces:
                if y<=lo or x>=hi:next_pieces.append((lo,hi))
                else:
                    if lo<x:next_pieces.append((lo,x))
                    if y<hi:next_pieces.append((y,hi))
            pieces=next_pieces
        remaining.extend(pieces)
    return remaining

def target_set(value):
    if not isinstance(value,list) or any(not isinstance(x,str) or not x for x in value):return None
    return set(value)
def source_version_map(value):
    if isinstance(value,dict):
        if any(not isinstance(k,str) or type(v) is not int for k,v in value.items()):return None
        return value
    if not isinstance(value,list):return None
    versions={}
    for x in value:
        if not isinstance(x,dict) or not isinstance(x.get('source_id'),str) or type(x.get('revision')) is not int:return None
        sid=x['source_id'];rev=x['revision']
        if sid in versions and versions[sid]!=rev:return None
        versions[sid]=rev
    return versions

def row(record,oracle):
    events=record['events'];kinds=defaultdict(list)
    for e in events:kinds[e.get('kind')].append(e)
    result=(kinds['EPISODE_RESULT'][-1]['data'] if kinds['EPISODE_RESULT'] else {})
    result=dict(result)
    # Adoption is an event, not an episode-success flag; later failure cannot erase it.
    actual_adoptions=[e['data'] for e in events if e.get('kind') in ('ADOPTION','SEMANTIC_ADOPTION')]
    if actual_adoptions:result['adoptions']=actual_adoptions
    admissions=[e['data'] for e in events if e.get('kind')=='HANDOFF_ADMISSION']
    if admissions:result['handoffs']=admissions
    receipts=[e['data'] for e in events if e.get('kind')=='RECEIVER_READ_RECEIPT']
    if receipts:result['receiver_receipts']=receipts
    if 'associations' not in result:
        result['associations']=[e['data']['decision'] for e in events if e.get('kind')=='DISPLAY_PROXY' and isinstance(e.get('data',{}).get('decision'),dict)]

    stimuli={}; first={}; complete={}
    for e in events:
        d=e.get('data',{}); sid=str(d.get('stimulus_id') or d.get('goal_id') or '')
        if e.get('kind')=='STIMULUS':stimuli[sid]=e['observed_ns']
        if e.get('kind')=='DISPLAY_PROXY' and sid and sid not in first and isinstance(d.get('text'),str) and d['text'].strip():first[sid]=e['observed_ns']
        if e.get('kind')=='GOAL_COMPLETE' and sid and isinstance(d.get('text'),str) and d['text'].strip():complete[sid]=e['observed_ns']
    fixture=json.loads(Path(record['input']).read_text())
    expected_stimuli=set()
    if record['dp'] in ('01','05'):
        expected_stimuli={str(e['stimulus_id']) for e in fixture['events'] if e.get('stimulus_id')}
    elif record['dp']=='02':
        expected_stimuli={str(e['id']) for e in fixture['events']}
        if fixture.get('agent_question'):expected_stimuli.add(fixture['agent_question']['id'])
    elif record['dp']=='04':
        expected_stimuli={f"{fixture['id']}:r{r['revision']}" for r in fixture['revisions']}
    external=[(c['send_ns'],c['terminal_ns']) for c in record['calls'] if c.get('role')=='RECEIVER' and c.get('send_ns') and c.get('terminal_ns')]
    waits=[];start=None
    for e in events:
        if e.get('kind')=='USER_WAIT_START':start=e['observed_ns']
        if e.get('kind')=='USER_WAIT_END' and start:waits.append((start,e['observed_ns']));start=None
    via_occupied=[(c['enqueue_ns'],c['terminal_ns']) for c in record['calls'] if c.get('role')!='RECEIVER' and c.get('enqueue_ns') and c.get('terminal_ns')]
    user_excluded=subtract_occupied(waits,via_occupied)
    qa01=[(first[s]-t)/1e6 for s,t in stimuli.items() if s in first]
    qa03=[(complete[s]-t-union_duration(external+user_excluded,t,complete[s]))/1e6 for s,t in stimuli.items() if s in complete]
    expected=oracle.get('adoptions',[]);adoptions=result.get('adoptions',[]);correct=0;details=[]
    identity_mappings=[];identity_unresolved=[]
    if record['dp']=='04':
        from score_dp04 import canonicalize_adoptions
        identity=canonicalize_adoptions(record,adoptions)
        adoptions=identity['adoptions'];identity_mappings=identity['mappings'];identity_unresolved=identity['unresolved']
    unresolved_keys={(str(x.get('expression_id')),str(x.get('revision'))) for x in identity_unresolved if x.get('expression_id') is not None}
    unresolved_revisions={str(x.get('revision')) for x in identity_unresolved if x.get('expression_id') is None}

    seen={}
    for a in adoptions if isinstance(adoptions,list) else []:
        if not isinstance(a,dict):continue
        key=(str(a.get('expression_id')),str(a.get('revision')))
        if key not in seen:seen[key]=a
    for a in expected:
        key=(str(a.get('expression_id')),str(a.get('revision')));actual=seen.get(key)
        actual_targets=target_set(actual.get('targets')) if actual is not None else None
        reason='MISSING_ADOPTION' if actual is None else 'MALFORMED_TARGETS' if actual_targets is None else None
        ok=actual_targets is not None and actual_targets==target_set(a.get('targets',[]))
        if key in unresolved_keys or key[1] in unresolved_revisions or (actual is not None and actual.get('_target_identity_unresolved')):ok=False;reason='UNRESOLVED_TARGET_IDENTITY'
        if not ok and reason is None:reason='TARGET_SET_MISMATCH'
        if ok and a.get('required_source_revisions'):
            actual_versions=source_version_map(actual.get('source_revisions'))
            if actual_versions is None:ok=False;reason='MALFORMED_SOURCE_REVISIONS'
            else:
                ok=all(actual_versions.get(k)==v for k,v in a['required_source_revisions'].items())
                if not ok:reason='SOURCE_VERSION_MISMATCH'
        correct+=ok;details.append({'expected':a,'actual_first_adoption':actual,'correct':ok,'failure_reason':reason})
    nr_total=nr_correct=0
    association_by_id={}
    for actual_a in result.get('associations',[]):
        association_by_id.setdefault((actual_a.get('event_id'),actual_a.get('revision')),actual_a)
    for expected_a in oracle.get('associations',[]):
        nr_total+=1
        actual_a=association_by_id.get((expected_a.get('event_id'),expected_a.get('revision')), {})
        nr_correct+=bool(actual_a) and all(actual_a.get(k)==v for k,v in expected_a.items())
    nh=[]
    if record['dp']=='05':
        from score_dp05 import score as score_handoff
        nh_result=score_handoff(result,oracle)
        nh=nh_result.get('details',[])
    memory=record['memory'];both=[m for m in memory if len(m['rss_bytes_by_pid'])>=2];baseline=both[0]['concurrent_sum_bytes'] if both else None
    memory_max_gap_ms=max(((b['at_ns']-a['at_ns'])/1e6 for a,b in zip(memory,memory[1:])),default=None)
    peak=max([x['concurrent_sum_bytes'] for x in memory],default=None)
    out={'run_id':record['run_id'],'dp':record['dp'],'episode':record['episode'],'variant':record['variant'],'rep':record['repetition'],'phase':record['phase'],'tactic':record.get('tactic'),'qa01_mean_ms':mean(qa01),'qa01_n':len(qa01),'qa01_sum_ms':sum(qa01),'qa01_noresponse':len(expected_stimuli-set(first)),'qa03_mean_ms':mean(qa03),'qa03_n':len(qa03),'qa03_sum_ms':sum(qa03),'qa03_incomplete':len(expected_stimuli-set(complete)),'qa02_correct':correct,'qa02_total':len(expected),'nr_correct':nr_correct,'nr_total':nr_total,'nh_correct':sum(x['correct'] for x in nh),'nh_total':len(nh),'via_cost_usd':sum(c.get('cost_usd') or 0 for c in record['calls'] if c.get('role')!='RECEIVER'),'receiver_cost_usd':sum(c.get('cost_usd') or 0 for c in record['calls'] if c.get('role')=='RECEIVER'),'unknown_usage_calls':sum(c.get('cost_usd') is None for c in record['calls']),'model_calls':len(record['calls']),'rss_concurrent_first_simultaneous_mb':baseline/1e6 if baseline is not None else None,'memory_sample_count':len(memory),'memory_both_process_sample_count':len(both),'memory_state':'SAMPLED_RSS_ESTIMATE' if both else 'INCOMPLETE_RUST_SHORT_LIVED','memory_max_sample_gap_ms':memory_max_gap_ms,'rss_concurrent_peak_mb':peak/1e6 if peak is not None else None,'functional_error':bool(kinds['EPISODE_ERROR']) or record['returncode']!=0,'wrong_referents':len(expected)-correct,'display_observation':'DISPLAY_PROXY','qa06_scope':'Python bridge+Rust RSS simultaneous estimate; shared dedup unverified; UI excluded','qa02_canonical_mappings':identity_mappings,'qa02_unresolved_identities':identity_unresolved,'qa02_details':details,'nh_details':nh,'timing_details_by_stimulus':{sid:{'qa01_ms':(first[sid]-t)/1e6 if sid in first else None,'qa03_ms':(complete[sid]-t-union_duration(external+user_excluded,t,complete[sid]))/1e6 if sid in complete else None} for sid,t in stimuli.items()},'timing_samples':{'qa01_ms':qa01,'qa03_ms':qa03},'result':result}
    return out

def analyze(root):
    records=[json.loads(l) for l in (root/'runs.jsonl').read_text().splitlines() if l];oracles={dp:keyed(BASE/'oracles'/f'dp{dp}.json') for dp in ['01','02','04','05']}
    rows=[row(r,oracles[r['dp']].get(r['episode'],{})) for r in records];dump(root/'episode-metrics.json',rows)
    flat=[{k:v for k,v in r.items() if k not in ['qa02_details','qa02_canonical_mappings','qa02_unresolved_identities','nh_details','timing_samples','timing_details_by_stimulus','result']} for r in rows]
    with (root/'episode-metrics.csv').open('w') as f:
        w=csv.DictWriter(f,fieldnames=list(flat[0]) if flat else []);w.writeheader();w.writerows(flat)
    groups=defaultdict(list)
    for r in rows:groups[(r['phase'],r['dp'],r['variant'],r.get('tactic'))].append(r)
    summary=[]
    for (phase,dp,variant,tactic),rs in sorted(groups.items()):
        n1=sum(r['qa01_n'] for r in rs);n3=sum(r['qa03_n'] for r in rs);n2=sum(r['qa02_total'] for r in rs);nr=sum(r['nr_total'] for r in rs);nh=sum(r['nh_total'] for r in rs)
        summary.append({'phase':phase,'dp':dp,'variant':variant,'tactic':tactic,'episodes':len(rs),'qa01_ms':sum(r['qa01_sum_ms'] for r in rs)/n1 if n1 else None,'qa01_n':n1,'qa01_noresponse':sum(r['qa01_noresponse'] for r in rs),'qa02_percent':100*sum(r['qa02_correct'] for r in rs)/n2 if n2 else None,'qa02_correct':sum(r['qa02_correct'] for r in rs),'qa02_total':n2,'qa03_ms':sum(r['qa03_sum_ms'] for r in rs)/n3 if n3 else None,'qa03_n':n3,'qa03_incomplete':sum(r['qa03_incomplete'] for r in rs),'qa04_modules_mean':None,'qa05_via_usd':sum(r['via_cost_usd'] for r in rs),'qa05_receiver_usd':sum(r['receiver_cost_usd'] for r in rs),'qa06_incomplete_memory_runs':sum(r['memory_state']!='SAMPLED_RSS_ESTIMATE' for r in rs),'qa06_peak_rss_mb':max((r['rss_concurrent_peak_mb'] for r in rs if r['rss_concurrent_peak_mb'] is not None),default=None),'nr_correct':sum(r['nr_correct'] for r in rs),'nr_total':nr,'nr_percent':100*sum(r['nr_correct'] for r in rs)/nr if nr else None,'nh_correct':sum(r['nh_correct'] for r in rs),'nh_total':nh,'nh_percent':100*sum(r['nh_correct'] for r in rs)/nh if nh else None,'wrong_referent_judgments':sum(r['qa02_total']-r['qa02_correct'] for r in rs),'wrong_association_judgments':sum(r['nr_total']-r['nr_correct'] for r in rs),'wrong_handoff_contracts':sum(r['nh_total']-r['nh_correct'] for r in rs),'errors':sum(r['functional_error'] for r in rs),'unknown_usage_calls':sum(r['unknown_usage_calls'] for r in rs),'model_calls':sum(r['model_calls'] for r in rs)})
    changes=root/'qa04'/'summary.json'
    if changes.exists():
        change=json.loads(changes.read_text())
        for s in summary:
            if s['phase']=='BASIC':s['qa04_modules_mean']=change['mean_changed_operational_modules']
    dump(root/'summary.json',{'rows':summary,'basic_episode_count':sum(r['phase']=='BASIC' for r in rows),'actual_all_episode_count':len(rows),'observation':'MEASURED_MODEL_AND_RUST_WITH_DISPLAY_PROXY','memory':'macOS concurrent Python+Rust RSS sum estimate, no shared dedup, UI and cloud weights excluded','qa04':'Actual C01/C03 patch check evidence; adapter tie when equal','campaign_cost_ledger':json.loads((root/'budget.json').read_text()) if (root/'budget.json').exists() else None,'limitations':['Authored finite inputs repeated three times; no population proof','No audio model or physical audio/display endpoint','No product score or 512MB product claim','Cloud Qwen latency is not local Omni latency']})
    def val(x):return 'N/A' if x is None else f'{x:.3f}' if isinstance(x,float) else str(x)
    lines=['# 이번 PoC 조건의 기본안별 측정 결과','','Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.','','|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|Runtime errors|Missing response / completion|','|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
    for s in summary:lines.append('|'+ '|'.join((f'{s[k]:.7f}' if k in ('qa05_via_usd','qa05_receiver_usd') and s[k] is not None else val(s[k])) for k in ['phase','dp','variant','episodes','qa01_ms','qa02_percent','qa03_ms','qa04_modules_mean','qa05_via_usd','qa05_receiver_usd','qa06_peak_rss_mb','qa06_incomplete_memory_runs','nr_percent','nh_percent','errors'])+'|'+str(s['qa01_noresponse'])+'/'+str(s['qa03_incomplete'])+'|')
    lines+=['','QA01 preserves wrong-answer time; errors are separate. QA03 excludes only actual receiver/user wait intervals. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.','',f'Basic episode rows: {sum(r["phase"]=="BASIC" for r in rows)}. All raw traces and every billed/unknown call remain retained.']
    (root/'report.md').write_text('\n'.join(lines)+'\n');return summary
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();s=analyze(a.results);print(json.dumps(s,indent=2))
