"""Post-hoc DP02 diagnostics; frozen N-R score and raw records remain unchanged."""
import argparse,json,statistics
from collections import defaultdict
from pathlib import Path
from analyze import BASE,keyed,row
from bridge import dump
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();records=[json.loads(x) for x in (a.results/'runs.jsonl').read_text().splitlines() if x];oracle=keyed(BASE/'oracles/dp02.json');details=[];groups=defaultdict(list)
for r in records:
    if r['dp']!='02' or r['phase']=='SMOKE':continue
    scored=row(r,oracle[r['episode']]);actual={(x.get('event_id'),x.get('revision')):x for x in scored['result'].get('associations',[])};unambiguous=[];frozen=[]
    for expected in oracle[r['episode']]['associations']:
        got=actual.get((expected['event_id'],expected['revision']),{});ok=bool(got) and all(got.get(k)==v for k,v in expected.items());item={'expected':expected,'actual_relation':{k:got.get(k) for k in ['event_id','revision','relation','target_request_id','question_id','task_id']},'exact_match':ok}
        frozen.append(item)
        if not (r['episode'] in ('02-E2','02-E3') and expected['event_id']=='U2'):unambiguous.append(item)
    outputs=[]
    for e in r['events']:
        if e.get('kind')=='DISPLAY_PROXY':
            d=e['data'];text=d.get('text');outputs.append({'stimulus_id':d.get('stimulus_id'),'text':text,'shape':'NULL' if text is None else 'EMPTY_STRING' if isinstance(text,str) and not text.strip() else 'NON_STRING' if not isinstance(text,str) else 'TEXT'})
    topic=None
    if r['episode'] in ('02-E2','02-E3'):
        response=next((x for x in outputs if x['stimulus_id']=='U2'),{});text=response.get('text');lower=text.lower() if isinstance(text,str) else ''
        tcp='tcp' in lower or 'transmission control' in lower;udp='udp' in lower or 'user datagram' in lower
        topic={'tcp_marker_present':tcp,'udp_marker_present':udp,'actual_text':text,'inspection':'Marker presence is not factual correctness proof. Absence exposes a concrete topic omission/review candidate; no new QA score.'}
    d={'run_id':r['run_id'],'phase':r['phase'],'variant':r['variant'],'episode':r['episode'],'rep':r['repetition'],'frozen_exact':frozen,'unambiguous_subset':unambiguous,'null_or_malformed_displays':[o for o in outputs if o['shape']!='TEXT'],'tcp_udp_topic_inspection':topic,'timing_by_stimulus':scored['timing_details_by_stimulus'],'noresponse':scored['qa01_noresponse'],'incomplete':scored['qa03_incomplete']};details.append(d);groups[(r['phase'],r['variant'])].append(d)
summary=[]
for (phase,variant),rs in sorted(groups.items()):
    fs=[x for r in rs for x in r['frozen_exact']];us=[x for r in rs for x in r['unambiguous_subset']];topic=[r['tcp_udp_topic_inspection'] for r in rs if r['tcp_udp_topic_inspection']]
    summary.append({'phase':phase,'variant':variant,'runs':len(rs),'frozen_nr_correct':sum(x['exact_match'] for x in fs),'frozen_nr_total':len(fs),'posthoc_unambiguous_correct':sum(x['exact_match'] for x in us),'posthoc_unambiguous_total':len(us),'tcp_or_udp_marker_missing':sum(not(t['tcp_marker_present'] and t['udp_marker_present']) for t in topic),'topic_trials':len(topic),'null_or_malformed_display_count':sum(len(r['null_or_malformed_displays']) for r in rs),'noresponse':sum(r['noresponse'] for r in rs),'incomplete':sum(r['incomplete'] for r in rs)})
dump(a.results/'dp02-diagnostics.json',{'scope':'Post hoc diagnostic only; frozen N-R metric, candidate, fixtures and oracle unchanged. Exclude only mixed continuation+independent question U2 in E2/E3 from this extra subset. No model judge.','summary':summary,'runs':details})
lines=['# DP02 association, response and path diagnostics','','Frozen N-R values are retained. U2 in E2/E3 combines continuation of R1 with a separate TCP/UDP question, while the frozen response schema has one relation for the entire utterance. The additional unambiguous subset excludes only those mixed U2 items, post hoc. It is not a revised QA or a replacement score. TCP/UDP marker inspection and null outputs are separate observed response facts.','','|Phase / option|Runs|Frozen N-R exact|Post hoc unambiguous exact|TCP or UDP marker absent|Null / malformed displays|Missing responses / completions|','|---|---:|---:|---:|---:|---:|---:|']
for s in summary:lines.append(f"|{s['phase']} {s['variant']}|{s['runs']}|{s['frozen_nr_correct']}/{s['frozen_nr_total']}|{s['posthoc_unambiguous_correct']}/{s['posthoc_unambiguous_total']}|{s['tcp_or_udp_marker_missing']}/{s['topic_trials']}|{s['null_or_malformed_display_count']}|{s['noresponse']}/{s['incomplete']}|")
lines+=['','## Actual response facts','']
for r in details:
    topic=r['tcp_udp_topic_inspection'];missing=r['null_or_malformed_displays']
    if missing or (topic and not(topic['tcp_marker_present'] and topic['udp_marker_present'])):
        lines += [f"### {r['run_id']}", '',f"Null/malformed actual displays: `{json.dumps(missing,ensure_ascii=False)}`.",f"TCP/UDP inspection actual text: `{json.dumps(topic,ensure_ascii=False)}`.",f"Per-stimulus times: `{json.dumps(r['timing_by_stimulus'],ensure_ascii=False)}`.",'']
(a.results/'dp02-diagnostics.md').write_text('\n'.join(lines)+'\n');print(json.dumps(summary))
