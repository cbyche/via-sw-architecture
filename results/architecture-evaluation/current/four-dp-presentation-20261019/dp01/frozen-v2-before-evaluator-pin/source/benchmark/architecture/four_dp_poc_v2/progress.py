"""Compact read-only progress including terminal calls in the active child trial."""
import argparse,json
from collections import Counter
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();root=a.results
runs=[json.loads(l) for l in (root/'runs.jsonl').read_text().splitlines() if l];paths=[root/'model-calls.jsonl',root/'basic-dp05-v2/model-calls.jsonl',*root.glob('tactics/*/model-calls.jsonl')];calls={}
for path in paths:
    if path.exists():
        for line in path.read_text().splitlines():
            if line:
                c=json.loads(line);calls[c['call_uuid']]=c
print(json.dumps({'phases':dict(Counter(r['phase'] for r in runs)),'latest':runs[-1]['run_id'],'terminal_calls':len(calls),'actual_standard_price_usd':round(sum(c.get('cost_usd') or 0 for c in calls.values()),7),'unknown_usage_calls':sum(c.get('cost_usd') is None for c in calls.values()),'runtime_errors':sum(r['returncode']!=0 or any(e.get('kind')=='EPISODE_ERROR' for e in r['events']) for r in runs)},ensure_ascii=False))
