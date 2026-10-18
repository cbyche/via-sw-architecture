"""Observed goal timing, calls/tokens and state/read events for bounded tactics."""
import argparse,json,statistics
from collections import Counter,defaultdict
from pathlib import Path
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();root=a.results
metrics={r['run_id']:r for r in json.loads((root/'episode-metrics.json').read_text())};runs=[json.loads(l) for l in (root/'runs.jsonl').read_text().splitlines() if l];plan=json.loads((root/'selected-tactic-plan.json').read_text());out=[]
for item in plan:
    for episode in item.get('episodes',[item.get('episode')]):
        groups={'before':[r for r in runs if r['phase']=='BASIC' and r['dp']==item['dp'] and r['variant']==item['base_variant'] and r['episode']==episode],'after':[r for r in runs if r['phase']==item.get('phase','TACTIC') and r['dp']==item['dp'] and r.get('tactic')==item['tactic'] and r['episode']==episode]}
        record={'dp':item['dp'],'tactic':item['tactic'],'phase':item.get('phase','TACTIC'),'episode':episode}
        for name,rs in groups.items():
            by_goal=defaultdict(list);details=[];stages=defaultdict(lambda:{'calls':0,'input_tokens':0,'output_tokens':0})
            for r in sorted(rs,key=lambda r:r['repetition']):
                scored=metrics[r['run_id']]
                for goal,timing in scored['timing_details_by_stimulus'].items():by_goal[goal].append({'rep':r['repetition'],**timing})
                events=Counter(e['kind'] for e in r['events']);fragments=[e['data'] for e in r['events'] if e['kind']=='A2_FRAGMENT_INVALIDATION'];eligibility=[e['data'] for e in r['events'] if e['kind']=='A1_SIDE_ELIGIBILITY']
                for c in r['calls']:
                    s=stages[c['stage']];s['calls']+=1;s['input_tokens']+=c.get('input_tokens',0);s['output_tokens']+=c.get('output_tokens',0)
                details.append({'run_id':r['run_id'],'rep':r['repetition'],'model_calls':len(r['calls']),'input_tokens':sum(c.get('input_tokens',0) for c in r['calls']),'output_tokens':sum(c.get('output_tokens',0) for c in r['calls']),'source_read_events':events['SOURCE_READ'],'evidence_read_events':events['EVIDENCE_READ'],'rolling_evidence_reuse_events':events['ROLLING_EVIDENCE_REUSE'],'fragment_invalidation':fragments,'side_eligibility':eligibility})
            goal_summary={}
            for goal,values in by_goal.items():
                valid=[x['qa01_ms'] for x in values if x['qa01_ms'] is not None];goal_summary[goal]={'qa01_mean_ms':statistics.mean(valid) if valid else None,'qa01_max_ms':max(valid,default=None),'qa01_observed':len(valid),'repetitions':values}
            record[name]={'per_goal':goal_summary,'stage_usage':dict(stages),'model_calls':sum(x['model_calls'] for x in details),'input_tokens':sum(x['input_tokens'] for x in details),'output_tokens':sum(x['output_tokens'] for x in details),'source_read_events':sum(x['source_read_events'] for x in details),'evidence_read_events':sum(x['evidence_read_events'] for x in details),'rolling_evidence_reuse_events':sum(x['rolling_evidence_reuse_events'] for x in details),'repetitions':details}
        out.append(record)
dump(root/'tactic-mechanisms.json',{'comparisons':out,'scope':'Same frozen affected case three repetitions only. DP01 SOURCE_READ corresponds to successful common source-contract file reads for these path-backed sources; explicit rolling reuse and retained-fragment events are reported separately. Read events do not establish physical disk I/O. Timing deltas retain model output/service variation.'})
print(f'{len(out)} tactic mechanism groups exported')
