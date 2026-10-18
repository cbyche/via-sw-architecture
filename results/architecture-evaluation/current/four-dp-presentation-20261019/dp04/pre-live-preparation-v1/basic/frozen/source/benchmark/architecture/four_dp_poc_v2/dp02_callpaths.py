"""Actual per-stimulus DP02 queue, model, usage and overlap diagnostics."""
import argparse,csv,json
from pathlib import Path
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();rows=[];runs=[]
for line in (a.results/'runs.jsonl').read_text().splitlines():
    r=json.loads(line)
    if r['dp']!='02' or r['phase']=='SMOKE':continue
    calls=[]
    for c in sorted(r['calls'],key=lambda c:c.get('enqueue_ns',0)):
        event={}
        try:
            body=json.loads(Path(c['request_file']).read_text());content=body['messages'][1]['content'];payload=json.loads(content) if isinstance(content,str) else {};event=payload.get('event',{})
        except (KeyError,ValueError,TypeError,OSError):pass
        row={'run_id':r['run_id'],'phase':r['phase'],'variant':r['variant'],'episode':r['episode'],'rep':r['repetition'],'stage':c['stage'],'event_id':event.get('id'),'revision':event.get('revision'),'enqueue_ms':(c['enqueue_ns']-r['start_ns'])/1e6,'send_ms':(c['send_ns']-r['start_ns'])/1e6 if c.get('send_ns') else None,'queue_and_bridge_ms':(c['send_ns']-c['enqueue_ns'])/1e6 if c.get('send_ns') else None,'network_model_ms':(c['terminal_ns']-c['send_ns'])/1e6 if c.get('send_ns') else None,'terminal_ms':(c['terminal_ns']-r['start_ns'])/1e6,'input_tokens':c.get('input_tokens'),'output_tokens':c.get('output_tokens'),'cost_usd':c.get('cost_usd'),'status':c['status']};rows.append(row);calls.append(row)
    first=next((c for c in calls if c['event_id']=='R1' and c['revision']==1),None);second=next((c for c in calls if c['event_id']=='U2'),None)
    overlap=max(0,min(first['terminal_ms'],second['terminal_ms'])-max(first['send_ms'],second['send_ms'])) if first and second and first['send_ms'] is not None and second['send_ms'] is not None else None
    runs.append({'run_id':r['run_id'],'phase':r['phase'],'variant':r['variant'],'episode':r['episode'],'rep':r['repetition'],'observed_first_two_network_overlap_ms':overlap,'calls':calls,'interpretation':'E1 non-overlap A/B uses identical code/context path; differences there are observed output/service variation. E2/E3 overlaps and admission queues expose mechanism; model output length remains a separate contributor.'})
dump(a.results/'dp02-callpaths.json',runs)
with (a.results/'dp02-callpaths.csv').open('w') as f:
    w=csv.DictWriter(f,fieldnames=list(rows[0]) if rows else []);w.writeheader();w.writerows(rows)
print(f'{len(rows)} actual call paths exported')
