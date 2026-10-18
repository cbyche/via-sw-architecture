"""DP05 actual read serving events; common observed span for BASIC and C2."""
import argparse,json
from pathlib import Path
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();rows=[]
for line in (a.results/'runs.jsonl').read_text().splitlines():
    r=json.loads(line)
    if r['dp']!='05' or r['phase']=='SMOKE':continue
    selected=None;spans=[]
    for e in r['events']:
        if e['kind']=='RECEIVER_READ_SELECTION':selected=e
        elif e['kind']=='RECEIVER_READ_RECEIPT':
            d=e['data']
            if selected is not None:spans.append({'delegation_id':d.get('delegation_id'),'observed_selection_to_receipt_ms':(e['observed_ns']-selected['observed_ns'])/1e6,'selected_ranges':len(d.get('selected_reads',[])),'served_ranges':len(d.get('served',[])),'denied_ranges':len(d.get('denied',[]))})
            selected=None
    stats=[e['data'] for e in r['events'] if e['kind']=='C2_CACHE_STATS'];source_reads=[e for e in r['events'] if e['kind']=='HANDOFF_SOURCE_READ' and e['data'].get('consumer')=='RECEIVER']
    rows.append({'run_id':r['run_id'],'phase':r['phase'],'variant':r['variant'],'tactic':r.get('tactic'),'episode':r['episode'],'rep':r['repetition'],'common_observed_serving_ms':sum(x['observed_selection_to_receipt_ms'] for x in spans),'common_read_batches':spans,'actual_receiver_source_read_events':len(source_reads),'c2_internal_stats':stats[-1] if stats else None,'scope':'Selection callback receipt to read receipt callback, same event boundaries for baseline and C2; includes serving/serialization/observer overhead. Internal C2 serving_ns has no baseline probe and is not a before/after latency measure. Successful source-file read calls do not establish physical disk I/O.'})
dump(a.results/'dp05-read-service.json',rows)
print(f'{len(rows)} DP05 serving rows exported')
