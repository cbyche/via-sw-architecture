"""Observed producer work before eligible requests, separate from QA01/03."""
import argparse,csv,json,statistics
from collections import defaultdict
from pathlib import Path
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();runs=[json.loads(x) for x in (a.results/'runs.jsonl').read_text().splitlines() if x];rows=[]
for r in runs:
    if r['dp'] not in ('01','05'):continue
    stimuli={};complete={};displays=[];activities=[]
    for e in r['events']:
        d=e.get('data',{});kind=e.get('kind');sid=d.get('stimulus_id',d.get('goal_id'))
        if kind=='STIMULUS':stimuli[sid]=e['observed_ns']
        elif kind=='GOAL_COMPLETE':complete[sid]=e['observed_ns']
        elif kind=='DISPLAY_PROXY' and isinstance(d.get('text'),str) and d['text'].strip():displays.append(e)
        elif kind in ('SOURCE_ACTIVITY','PUBLIC_ACTIVITY'):activities.append(e)
    windows=[(t,complete.get(sid,r['end_ns'])) for sid,t in stimuli.items()]
    prep=[]
    for call in r['calls']:
        if call.get('role')=='RECEIVER' or not call.get('enqueue_ns') or not call.get('terminal_ns'):continue
        if not any(lo<=call['enqueue_ns']<=hi for lo,hi in windows):prep.append(call)
    source_paths=[]
    for event in activities:
        next_display=next((e for e in displays if e['observed_ns']>=event['observed_ns']),None)
        next_input=next((e for e in r['events'] if e.get('kind')=='STIMULUS' and e['observed_ns']>=event['observed_ns']),None)
        d=event['data'];source=d.get('source',d)
        source_paths.append({'activity_kind':d.get('kind'),'source_id':source.get('id',d.get('source_id')),'source_revision':source.get('revision'),'activity_ns':event['observed_ns'],'next_stimulus_id':next_input['data'].get('stimulus_id') if next_input else None,'next_first_display_ns':next_display['observed_ns'] if next_display else None,'activity_to_next_first_display_ms':(next_display['observed_ns']-event['observed_ns'])/1e6 if next_display else None,'observed_activity_to_next_stimulus_ms':(next_input['observed_ns']-event['observed_ns'])/1e6 if next_input else None})
    rows.append({'run_id':r['run_id'],'phase':r['phase'],'dp':r['dp'],'variant':r['variant'],'episode':r['episode'],'rep':r['repetition'],'pre_request_model_calls':len(prep),'pre_request_model_ms':sum((c['terminal_ns']-c['enqueue_ns'])/1e6 for c in prep),'pre_request_via_cost_usd':sum(c.get('cost_usd') or 0 for c in prep),'pre_request_stages':[c['stage'] for c in prep],'activity_to_next_first_display_samples_ms':[x['activity_to_next_first_display_ms'] for x in source_paths if x['activity_to_next_first_display_ms'] is not None],'activity_paths':source_paths,'scope':'Observed serialized source/activity release and quiescent pre-query production; separate diagnostic, not QA01/03 or immediate-arrival prediction'})
dump(a.results/'preparation-metrics.json',rows)
with (a.results/'preparation-metrics.csv').open('w') as f:
    fields=[k for k in rows[0] if k!='activity_paths'] if rows else [];w=csv.DictWriter(f,fieldnames=fields);w.writeheader();w.writerows([{k:json.dumps(v) if isinstance(v,list) else v for k,v in r.items() if k!='activity_paths'} for r in rows])
groups=defaultdict(list)
for r in rows:groups[(r['phase'],r['dp'],r['variant'],r['episode'])].append(r)
lines=['# Pre-request production and source-activity diagnostics','','QA01/03 in these serialized fixtures measure requests after preceding source production is quiescent. Source changes, including correction, finish their producer work before the next STIMULUS. The producer is not free: usage/cost is already included in QA05; observed preparation time is reported here separately. No immediate-request-during-production behavior or counterfactual latency is measured. Source/activity-to-next-display spans include all intervening authored activities and producer work.','','|Phase / case / option|Runs|Pre-request model calls|Mean pre-request model ms|Pre-request model $|Source/activity → next display mean ms|','|---|---:|---:|---:|---:|---:|']
for (phase,dp,var,episode),rs in sorted(groups.items()):
    paths=[v for r in rs for v in r['activity_to_next_first_display_samples_ms']];lines.append(f'|{phase} {episode} {var}|{len(rs)}|{sum(r["pre_request_model_calls"] for r in rs)}|{statistics.mean(r["pre_request_model_ms"] for r in rs):.3f}|{sum(r["pre_request_via_cost_usd"] for r in rs):.6f}|{statistics.mean(paths) if paths else 0:.3f}|')
(a.results/'preparation-report.md').write_text('\n'.join(lines)+'\n');print(f'{len(rows)} preparation rows exported')
