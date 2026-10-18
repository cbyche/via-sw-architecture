"""DP04 producer scheduling diagnosis using actual enqueue/send/terminal spans."""
import argparse,json
from pathlib import Path
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();out=[]
for line in (a.results/'runs.jsonl').read_text().splitlines():
    r=json.loads(line)
    if r['dp']!='04' or r['phase']=='SMOKE' or r['episode']!='dp04-dense':continue
    calls=[]
    for c in sorted(r['calls'],key=lambda c:c['enqueue_ns']):
        calls.append({'stage':c['stage'],'enqueue_ms':(c['enqueue_ns']-r['start_ns'])/1e6,'send_ms':(c['send_ns']-r['start_ns'])/1e6 if c.get('send_ns') else None,'terminal_ms':(c['terminal_ns']-r['start_ns'])/1e6,'queue_and_bridge_ms':(c['send_ns']-c['enqueue_ns'])/1e6 if c.get('send_ns') else None,'network_model_ms':(c['terminal_ns']-c['send_ns'])/1e6 if c.get('send_ns') else None,'input_tokens':c.get('input_tokens'),'output_tokens':c.get('output_tokens'),'cost_usd':c.get('cost_usd')})
    screen=next((c for c in calls if c['stage']=='dp04.screen_producer'),None);speech=next((c for c in calls if c['stage']=='dp04.speech_producer'),None)
    overlap=max(0,min(screen['terminal_ms'],speech['terminal_ms'])-max(screen['send_ms'],speech['send_ms'])) if screen and speech and screen['send_ms'] is not None and speech['send_ms'] is not None else None
    span=max(screen['terminal_ms'],speech['terminal_ms'])-min(screen['enqueue_ms'],speech['enqueue_ms']) if screen and speech else None
    out.append({'run_id':r['run_id'],'phase':r['phase'],'variant':r['variant'],'tactic':r.get('tactic'),'rep':r['repetition'],'episode':r['episode'],'initial_producer_network_overlap_ms':overlap,'initial_producer_enqueue_to_both_terminal_ms':span,'calls':calls,'scope':'Dense episode only. B4 parallelizes the same independent producers under K2. Actual calls and later binding/expansion retained. Service/output variation means total before/after delta is not a pure scheduling attribution.'})
dump(a.results/'dp04-dense-callpaths.json',out);print(f'{len(out)} dense paths exported')
