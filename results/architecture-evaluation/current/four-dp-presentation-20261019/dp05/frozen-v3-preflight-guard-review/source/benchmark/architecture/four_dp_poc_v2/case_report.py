"""Per-case means and all three individual repetitions, without model calls."""
import argparse,csv,json
from collections import defaultdict
from pathlib import Path
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();rows=json.loads((a.results/'episode-metrics.json').read_text());groups=defaultdict(list)
for r in rows:groups[(r['phase'],r['dp'],r['variant'],r['episode'])].append(r)
cases=[]
for (phase,dp,variant,episode),rs in sorted(groups.items()):
    rs.sort(key=lambda r:r['rep']);n1=sum(r['qa01_n'] for r in rs);n3=sum(r['qa03_n'] for r in rs)
    cases.append({'phase':phase,'dp':dp,'variant':variant,'episode':episode,'repetitions_recorded':len(rs),'qa01_mean_ms':sum(r['qa01_sum_ms'] for r in rs)/n1 if n1 else None,'qa01_rep_values_ms':{str(r['rep']):r['qa01_mean_ms'] for r in rs},'qa02_correct':sum(r['qa02_correct'] for r in rs),'qa02_total':sum(r['qa02_total'] for r in rs),'qa02_rep_counts':{str(r['rep']):[r['qa02_correct'],r['qa02_total']] for r in rs},'qa03_mean_ms':sum(r['qa03_sum_ms'] for r in rs)/n3 if n3 else None,'qa03_rep_values_ms':{str(r['rep']):r['qa03_mean_ms'] for r in rs},'qa05_via_total_usd':sum(r['via_cost_usd'] for r in rs),'qa05_via_rep_values_usd':{str(r['rep']):r['via_cost_usd'] for r in rs},'qa05_receiver_total_usd':sum(r['receiver_cost_usd'] for r in rs),'qa06_peak_rss_mb':max((r['rss_concurrent_peak_mb'] for r in rs if r['rss_concurrent_peak_mb'] is not None),default=None),'qa06_rep_peaks_mb':{str(r['rep']):r['rss_concurrent_peak_mb'] for r in rs},'nr_correct':sum(r['nr_correct'] for r in rs),'nr_total':sum(r['nr_total'] for r in rs),'nh_correct':sum(r['nh_correct'] for r in rs),'nh_total':sum(r['nh_total'] for r in rs),'noresponse':sum(r['qa01_noresponse'] for r in rs),'incomplete':sum(r['qa03_incomplete'] for r in rs),'functional_errors':sum(r['functional_error'] for r in rs)})
dump(a.results/'case-metrics.json',cases)
with (a.results/'case-metrics.csv').open('w') as f:
    w=csv.DictWriter(f,fieldnames=list(cases[0]) if cases else []);w.writeheader();w.writerows([{k:json.dumps(v) if isinstance(v,dict) else v for k,v in row.items()} for row in cases])
def v(x):return 'N/A' if x is None else f'{x:.2f}' if isinstance(x,float) else str(x)
def reps(d):return ', '.join(f'r{k}: {v(x)}' for k,x in d.items())
lines=['# Per-case values and individual repetitions','','DISPLAY_PROXY; same fixed authored episodes repeated, not independent user samples. QA06 is simultaneous sampled RSS estimate; shared pages unverified.','','|Phase / case / option|QA01 mean ms|QA01 r1/r2/r3 ms|QA02 exact|QA03 mean ms|QA03 r1/r2/r3 ms|VIA total $|RSS max MB|N-R|N-H|Errors / missing|','|---|---:|---|---:|---:|---|---:|---:|---:|---:|---:|']
for c in cases:
    lines.append('|'+ '|'.join([f"{c['phase']} {c['episode']} {c['variant']}",v(c['qa01_mean_ms']),reps(c['qa01_rep_values_ms']),f"{c['qa02_correct']}/{c['qa02_total']}",v(c['qa03_mean_ms']),reps(c['qa03_rep_values_ms']),f"{c['qa05_via_total_usd']:.7f}",v(c['qa06_peak_rss_mb']),f"{c['nr_correct']}/{c['nr_total']}",f"{c['nh_correct']}/{c['nh_total']}",f"{c['functional_errors']} / {c['noresponse']}"])+'|')
(a.results/'case-report.md').write_text('\n'.join(lines)+'\n');print(f'{len(cases)} case/option groups exported')
