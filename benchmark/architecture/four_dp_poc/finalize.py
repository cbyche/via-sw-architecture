"""Reproduce final derived artifacts from recorded evidence only; never calls models."""
import argparse,csv,json,subprocess,sys
from collections import Counter
from pathlib import Path
BASE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();root=a.results.resolve()
runs=[json.loads(l) for l in (root/'runs.jsonl').read_text().splitlines() if l];counts=Counter(r['phase'] for r in runs)
if counts['BASIC']!=120 or counts['TACTIC']!=12 or counts['DIAGNOSTIC']!=3:raise RuntimeError('Finalization requires120 BASIC,12 TACTIC and3 DIAGNOSTIC rows; failures remain included')
for name in ['analyze.py','case_report.py','preparation_report.py','failure_report.py','dp02_diagnostics.py','dp02_callpaths.py','dp04_callpaths.py','handoff_timing.py','memory_audit.py','read_service_report.py','compare_tactics.py','tactic_mechanisms.py','semantic_conditions.py']:
    completed=subprocess.run([sys.executable,str(BASE/name),str(root)],check=True,capture_output=True,text=True)
    if name!='analyze.py':print(completed.stdout.strip())
summary=json.loads((root/'summary.json').read_text())
content=json.loads((root/'semantic-conditions.json').read_text())
for row in summary['rows']:
    check=next(x for x in content['summary_rows'] if all(x.get(k)==row.get(k) for k in ['phase','dp','variant','tactic']))
    row.update({'content_condition_instances':check['condition_instances'],'content_exact_marker_passes':check['exact_marker_pass_instances'],'content_fact_omission_instances':check['whole_required_fact_omission_instances'],'content_unresolved_or_missing':check['unresolved_or_missing_display'],'content_not_defined_runs':check['runs_without_defined_semantic_checks']})
summary['content_diagnostic']={'source':'semantic-conditions.json','primary_qa_changed':False,'basic_omissions':sum(x['whole_required_fact_omission_instances'] for x in content['summary_rows'] if x['phase']=='BASIC'),'tactic_omissions':sum(x['whole_required_fact_omission_instances'] for x in content['summary_rows'] if x['phase']=='TACTIC'),'limit':'GOAL_COMPLETE is instrumented candidate readiness, not proof of required response content. NOT_DEFINED content checks are not passes.'}
selection=json.loads((root/'poc-selections.json').read_text())
summary['baseline_selections']=selection['baselines']
summary['tactic_adoption']={k:selection[k] for k in ['evaluated_tactics','adopted_tactics','not_adopted_tactics','diagnostic_only']}
summary['selection_scope']='Finite PoC choices; tactics separately evaluated; no accepted ADR change'
post=[{'run_id':r['run_id'],'phase':r['phase'],'dp':r['dp'],'variant':r['variant'],'tactic':r['tactic'],'episode':r['episode'],'repetition':r['repetition'],'raw_trace':str(Path(r['input']).parent/'trace.json'),'runtime_error':r['returncode']!=0 or any(e.get('kind')=='EPISODE_ERROR' for e in r['events'])} for r in runs if r['phase'] in ['TACTIC','DIAGNOSTIC']]
(root/'post-basic-runs.json').write_text(json.dumps(post,indent=2)+'\n')
with (root/'post-basic-runs.csv').open('w') as f:
    w=csv.DictWriter(f,fieldnames=list(post[0]));w.writeheader();w.writerows(post)
calls=[json.loads(line) for line in (root/'model-calls.jsonl').read_text().splitlines() if line]
summary['actual_campaign_usage']={'calls':len(calls),'input_tokens':sum(c.get('input_tokens',0) for c in calls),'output_tokens':sum(c.get('output_tokens',0) for c in calls),'known_usage_calls':sum(c.get('cost_usd') is not None for c in calls),'actual_standard_price_usd':sum(c.get('cost_usd') or 0 for c in calls),'scope':'All access/smoke/preparation/BASIC/tactic/diagnostic and failed JSON calls, at frozen standard price; not invoice.'}
(root/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
with (root/'summary.csv').open('w') as f:
    w=csv.DictWriter(f,fieldnames=list(summary['rows'][0]));w.writeheader();w.writerows(summary['rows'])
with (root/'report.md').open('a') as f:f.write('\n## Separate frozen content conditions\n\nDP01 E3 q2: BASIC ABC×3 has9 required-content omissions; A2×3 has3. All36 frozen fact-marker checks are absent in the displayed answers. Source/set identity QA02 and original timing values remain unchanged. GOAL_COMPLETE is a candidate marker, not proof that summary-content conditions were met. Other cases without frozen semantic_checks are NOT_DEFINED, never a pass. See semantic-conditions.md/json for original text and exact-marker limits.\n')
with (root/'report.md').open('a') as f:f.write('\n## Final finite-PoC selections\n\nBasic choices:01A /02A /04A /05C (read-capable receiver profile). Adopt02 A1 only for bounded short tool-free independent side questions.01 A2 and04 A4 are not adopted;05 C2 basic application is deferred because the observed overall VIA time did not improve and serving savings were tiny.04 B4 remains diagnostic only. See poc-selections.json for exact reasons and tactic-comparison.md for all individual repetitions. No accepted ADR change.\n')
subprocess.run([sys.executable,str(BASE/'export_demo.py'),str(root)],check=True)
# Reconciliation hashes the finished top-level artifacts, including offline replay.
subprocess.run([sys.executable,str(BASE/'reconcile.py'),str(root)],check=True)
print(json.dumps({'final_counts':dict(counts),'models_called':0,'offline_browser':str(root/'evidence-browser.html'),'reproduce_command':f'python3 benchmark/architecture/four_dp_poc/finalize.py {a.results}'}))
