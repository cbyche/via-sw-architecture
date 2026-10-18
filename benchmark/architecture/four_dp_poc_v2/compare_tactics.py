"""Same-episode before/after comparisons; preserves each repetition and goal."""
import argparse,json
from pathlib import Path
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args()
rows=json.loads((a.results/'episode-metrics.json').read_text())
plan=json.loads((a.results/'selected-tactic-plan.json').read_text())
def aggregate(rs):
    n1=sum(r['qa01_n'] for r in rs);n3=sum(r['qa03_n'] for r in rs)
    return {'runs':len(rs),'qa01_ms':sum(r['qa01_sum_ms'] for r in rs)/n1 if n1 else None,'qa01_observed':n1,'missing_responses':sum(r['qa01_noresponse'] for r in rs),'qa03_ms':sum(r['qa03_sum_ms'] for r in rs)/n3 if n3 else None,'qa03_observed':n3,'missing_completions':sum(r['qa03_incomplete'] for r in rs),'qa02_correct':sum(r['qa02_correct'] for r in rs),'qa02_total':sum(r['qa02_total'] for r in rs),'nr_correct':sum(r['nr_correct'] for r in rs),'nr_total':sum(r['nr_total'] for r in rs),'nh_correct':sum(r['nh_correct'] for r in rs),'nh_total':sum(r['nh_total'] for r in rs),'via_cost_usd':sum(r['via_cost_usd'] for r in rs),'receiver_cost_usd':sum(r['receiver_cost_usd'] for r in rs),'rss_max_mb':max((r['rss_concurrent_peak_mb'] for r in rs if r['rss_concurrent_peak_mb'] is not None),default=None),'runtime_errors':sum(r['functional_error'] for r in rs),'repetitions':[{'rep':r['rep'],'run_id':r['run_id'],'qa01_ms':r['qa01_mean_ms'],'qa03_ms':r['qa03_mean_ms'],'qa02':[r['qa02_correct'],r['qa02_total']],'via_cost_usd':r['via_cost_usd'],'receiver_cost_usd':r['receiver_cost_usd'],'rss_peak_mb':r['rss_concurrent_peak_mb'],'per_stimulus':r['timing_details_by_stimulus']} for r in sorted(rs,key=lambda r:r['rep'])]}
out=[]
for item in plan:
    for episode in item.get('episodes',[item.get('episode')]):
        before=[r for r in rows if r['phase']=='BASIC' and r['dp']==item['dp'] and r['variant']==item['base_variant'] and r['episode']==episode]
        after=[r for r in rows if r['phase']==item.get('phase','TACTIC') and r['dp']==item['dp'] and r.get('tactic')==item['tactic'] and r['episode']==episode]
        out.append({'dp':item['dp'],'tactic':item['tactic'],'phase':item.get('phase','TACTIC'),'episode':episode,'before_variant':item['base_variant'],'after_variant':item.get('variant',item['base_variant']),'before':aggregate(before),'after':aggregate(after),'complete_three_each':len(before)==3 and len(after)==3,'interpretation':'This affected authored episode only. Repetitions retain service/output variation; no whole-DP or population estimate. All original-goal penalties and missing outputs remain included.'})
dump(a.results/'tactic-comparison.json',out)
def val(x):return 'N/A' if x is None else f'{x:.3f}'
lines=['# Same-case basic and tactic comparison','','DISPLAY_PROXY, Voice OFF. Three repetitions of the same affected authored episode; B4 is a scheduling diagnostic. QA04 was not remeasured for tactics.','','|DP / tactic / case|Stage|Runs|QA01 ms / missing|QA02 exact|QA03 ms / missing|VIA $|Receiver $|RSS max MB|Runtime errors|','|---|---|---:|---|---|---|---:|---:|---:|---:|']
details=[]
for x in out:
    for name in ('before','after'):
        r=x[name];lines.append(f"|{x['dp']} {x['tactic']} {x['episode']} ({x['phase']})|{name}|{r['runs']}|{val(r['qa01_ms'])} / {r['missing_responses']}|{r['qa02_correct']}/{r['qa02_total']}|{val(r['qa03_ms'])} / {r['missing_completions']}|{r['via_cost_usd']:.7f}|{r['receiver_cost_usd']:.7f}|{val(r['rss_max_mb'])}|{r['runtime_errors']}|")
    details+=['',f"## {x['dp']} {x['tactic']} individual repetitions",'','```json',json.dumps({'before':x['before']['repetitions'],'after':x['after']['repetitions']},ensure_ascii=False,indent=2),'```','']
lines+=details
(a.results/'tactic-comparison.md').write_text('\n'.join(lines)+'\n')
print(f'{len(out)} same-case comparisons exported')
