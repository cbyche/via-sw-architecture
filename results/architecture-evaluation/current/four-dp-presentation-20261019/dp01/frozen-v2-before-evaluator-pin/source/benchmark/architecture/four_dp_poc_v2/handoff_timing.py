"""DP05 actual total elapsed and receiver spans, without counterfactual timing."""
import argparse,json,statistics
from collections import defaultdict
from pathlib import Path
from bridge import dump
from analyze import union_duration
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();goals=[]
for line in (a.results/'runs.jsonl').read_text().splitlines():
    r=json.loads(line)
    if r['dp']!='05' or r['phase']=='SMOKE':continue
    starts={};ends={};ingress={};agent_end={}
    for e in r['events']:
        d=e.get('data',{});key=d.get('goal_id') or d.get('stimulus_id')
        if e['kind']=='STIMULUS':starts[key]=e['observed_ns']
        elif e['kind']=='GOAL_COMPLETE' and isinstance(d.get('text'),str) and d['text'].strip():ends[key]=e['observed_ns']
        elif e['kind']=='AGENT_INGRESS':ingress[key]=e['observed_ns']
        elif e['kind']=='AGENT_RESULT_AVAILABLE':agent_end[key]=e['observed_ns']
    external=[(c['send_ns'],c['terminal_ns']) for c in r['calls'] if c.get('role')=='RECEIVER' and c.get('send_ns') and c.get('terminal_ns')]
    for goal,t in starts.items():
        end=ends.get(goal);ext=union_duration(external,t,end) if end else None
        goals.append({'run_id':r['run_id'],'phase':r['phase'],'variant':r['variant'],'tactic':r.get('tactic'),'episode':r['episode'],'rep':r['repetition'],'goal_id':goal,'handoff':goal in ingress,'total_stimulus_to_complete_ms':(end-t)/1e6 if end else None,'external_receiver_model_ms':ext/1e6 if ext is not None else None,'via_residual_ms':(end-t-ext)/1e6 if end else None,'agent_ingress_to_result_ms':(agent_end[goal]-ingress[goal])/1e6 if goal in ingress and goal in agent_end else None})
groups=defaultdict(list)
for r in goals:groups[(r['phase'],r['variant'],r.get('tactic'),r['episode'])].append(r)
aggregates=[]
for (phase,variant,tactic,episode),rs in sorted(groups.items()):
    def avg(field):
        values=[r[field] for r in rs if r[field] is not None];return statistics.mean(values) if values else None
    aggregates.append({'phase':phase,'variant':variant,'tactic':tactic,'episode':episode,'goals':len(rs),'completed_goals':sum(r['total_stimulus_to_complete_ms'] is not None for r in rs),'total_ms':avg('total_stimulus_to_complete_ms'),'receiver_external_ms':avg('external_receiver_model_ms'),'via_residual_ms':avg('via_residual_ms'),'agent_ingress_to_result_ms':avg('agent_ingress_to_result_ms')})
dump(a.results/'dp05-handoff-timing.json',{'goals':goals,'cases':aggregates,'scope':'Actual observer intervals, DISPLAY_PROXY and Voice OFF. Total includes VIA and external receiver. VIA residual subtracts only receiver send-to-terminal union; receiver read servicing stays VIA. E1 no-handoff close is identified separately. Preparation before STIMULUS is reported in preparation-metrics, not folded into request start.'})
print(f'{len(goals)} goal timing rows exported')
