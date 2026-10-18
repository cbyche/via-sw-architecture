"""Read-only evidence reconciliation; no model calls or auth access."""
import argparse,hashlib,json
from collections import Counter
from pathlib import Path
from bridge import dump,MODEL
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args();root=a.results
runs=[json.loads(l) for l in (root/'runs.jsonl').read_text().splitlines() if l];calls=[json.loads(l) for l in (root/'model-calls.jsonl').read_text().splitlines() if l];budget=json.loads((root/'budget.json').read_text());freeze=json.loads((root/'freeze.json').read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
basic=[r for r in runs if r['phase']=='BASIC'];keys=[(r['dp'],r['variant'],r['episode'],r['repetition']) for r in basic];callids=[c['call_uuid'] for c in calls];known=set(callids);responses=[]
for c in calls:
    path=c.get('response_file');body=json.loads(Path(path).read_text()) if path and Path(path).exists() else {}
    responses.append({'call_uuid':c['call_uuid'],'actual_model':body.get('model'),'status':c['status'],'usage_present':bool(body.get('usage')),'request_exists':Path(c['request_file']).exists(),'response_exists':bool(path and Path(path).exists())})
# Presentation/analyzer changes are versioned separately; candidate/source/fixture inputs must stay fixed.
critical={k:v for k,v in freeze['source'].items() if k.startswith('prototypes/candidates/four_dp_poc/src/') or any('/'+part+'/' in k for part in ('fixtures','oracles','sources','assets')) or k.endswith('/bridge.py')}
critical_changed=[k for k,v in critical.items() if not Path(k).exists() or sha(Path(k))!=v]
artifact_hashes={str(p.relative_to(root)):sha(p) for p in root.iterdir() if p.is_file() and p.suffix in ('.json','.jsonl','.csv','.md','.html') and p.name!='reconciliation.json'}
result={'basic_rows':len(basic),'unique_basic_rows':len(set(keys)),'expected_basic_rows':120,'basic_by_option':dict(Counter(r['dp']+r['variant'] for r in basic)),'phase_counts':dict(Counter(r['phase'] for r in runs)),'run_ids_unique':len({r['run_id'] for r in runs})==len(runs),'model_call_records':len(calls),'call_ids_unique':len(known)==len(calls),'all_run_calls_in_global_ledger':all(c['call_uuid'] in known for r in runs for c in r['calls']),'all_calls_in_budget':all(x in budget['calls'] for x in known),'budget_ids_not_yet_terminal':sorted(set(budget['calls'])-known),'usage_known_calls':sum(c.get('cost_usd') is not None for c in calls),'unknown_usage_calls':sum(c.get('cost_usd') is None for c in calls),'actual_usage_priced_usd':sum(c.get('cost_usd') or 0 for c in calls),'budget_held_usd':sum(c['amount_usd'] for c in budget['calls'].values()),'global_budget_usd':budget['limit_usd'],'response_model_counts':dict(Counter(str(r['actual_model']) for r in responses)),'all_success_models_match':all(r['actual_model']==MODEL for r in responses if r['status']=='COMPLETED'),'critical_frozen_input_changed':critical_changed,'runtime_process_errors':sum(r['returncode']!=0 or any(e['kind']=='EPISODE_ERROR' for e in r['events']) for r in runs),'responses':responses,'top_level_artifact_sha256':artifact_hashes,'scope':'Read-only finite-campaign reconciliation. Original84 plus pre-DP05 v2 36; tactics/diagnostics separately identified. Physical GUI/audio and statistical population claims excluded.'}
dump(root/'reconciliation.json',result)
print(json.dumps({k:v for k,v in result.items() if k not in ('responses','top_level_artifact_sha256')},indent=2))
