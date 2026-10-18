"""Per-trial RSS sampling coverage from original samples; no measurement reruns."""
import argparse,csv,json
from pathlib import Path
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();rows=[]
for line in (a.results/'runs.jsonl').read_text().splitlines():
    r=json.loads(line);samples=r['memory'];both=[s for s in samples if len(s['rss_bytes_by_pid'])>=2];first=samples[0] if samples else None;pids=sorted({pid for s in samples for pid in s['rss_bytes_by_pid']},key=int)
    rows.append({'run_id':r['run_id'],'phase':r['phase'],'dp':r['dp'],'variant':r['variant'],'episode':r['episode'],'rep':r['repetition'],'sample_count':len(samples),'simultaneous_both_pid_sample_count':len(both),'pid_count_seen':len(pids),'coverage':'BOTH_PIDS_SAMPLED' if both else 'INCOMPLETE_RUST_SHORT_LIVED','first_sample_since_start_ms':(first['at_ns']-r['start_ns'])/1e6 if first else None,'first_sample_rss_mb_by_pid':{pid:value/1e6 for pid,value in first['rss_bytes_by_pid'].items()} if first else {},'pid_role_caveat':'No per-PID role attribution. Collector contract samples its Python runner PID and spawned Rust PID; raw records retain only observed PID values, without PPID/role labels.','first_simultaneous_sum_mb':both[0]['concurrent_sum_bytes']/1e6 if both else None,'peak_sampled_sum_mb':max((s['concurrent_sum_bytes']/1e6 for s in samples),default=None),'maximum_sample_gap_ms':max(((y['at_ns']-x['at_ns'])/1e6 for x,y in zip(samples,samples[1:])),default=None)})
dump(a.results/'memory-audit.json',{'scope':'Fresh per-trial Python runner/bridge + Rust, macOS ps RSS, decimal MB. Browser/cloud weights/outer orchestrator excluded. Shared-page dedup unverified. 100 ms nominal polling cannot establish true peak or observe a shorter-lived Rust process. First sample is an observed baseline proxy, not pre-start physical commit.','runs':rows})
with (a.results/'memory-audit.csv').open('w') as f:
    w=csv.DictWriter(f,fieldnames=list(rows[0]) if rows else []);w.writeheader();w.writerows([{k:json.dumps(v) if isinstance(v,dict) else v for k,v in r.items()} for r in rows])
print(json.dumps({'runs':len(rows),'incomplete_basic_trials':sum(r['phase']=='BASIC' and r['coverage']!='BOTH_PIDS_SAMPLED' for r in rows)}))
