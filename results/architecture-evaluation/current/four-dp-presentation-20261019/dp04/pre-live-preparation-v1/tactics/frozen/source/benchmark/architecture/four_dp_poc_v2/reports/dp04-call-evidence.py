"""Read-only DP04 provider-stage extraction; no scoring or provider access."""
import argparse,collections,hashlib,json
from pathlib import Path

p=argparse.ArgumentParser();p.add_argument('phase_root',type=Path);p.add_argument('--output',type=Path);a=p.parse_args()
raw=a.phase_root/'runs.jsonl';records=[json.loads(x) for x in raw.read_text().splitlines() if x]
rows=[]
for r in records:
 if r['dp']!='04':continue
 stages={}
 for c in r['calls']:
  s=stages.setdefault(c['stage'],{'calls':0,'input_tokens':0,'output_tokens':0,'cost_usd':0,'provider_elapsed_ms':[],'queue_ms':[],'call_uuids':[],'unknown_usage':0,'statuses':[]})
  s['calls']+=1;s['input_tokens']+=c.get('input_tokens') or 0;s['output_tokens']+=c.get('output_tokens') or 0;s['cost_usd']+=c.get('cost_usd') or 0;s['call_uuids'].append(c['call_uuid']);s['unknown_usage']+=c.get('cost_usd') is None;s['statuses'].append(c.get('status'))
  if c.get('terminal_ns') is not None and c.get('send_ns') is not None:s['provider_elapsed_ms'].append((c['terminal_ns']-c['send_ns'])/1e6)
  if c.get('send_ns') is not None and c.get('enqueue_ns') is not None:s['queue_ms'].append((c['send_ns']-c['enqueue_ns'])/1e6)
 events=r['events'];kinds=collections.Counter(e['kind'] for e in events)
 rows.append({'run_id':r['run_id'],'phase':r['phase'],'episode':r['episode'],'variant':r['variant'],'repetition':r['repetition'],'freeze_sha256':r.get('freeze_sha256'),'binary_sha256':r.get('candidate_binary_sha256'),'stages':stages,'total_calls':len(r['calls']),'total_input_tokens':sum(s['input_tokens'] for s in stages.values()),'total_output_tokens':sum(s['output_tokens'] for s in stages.values()),'total_cost_usd':sum(s['cost_usd'] for s in stages.values()),'event_counts':dict(kinds),'rejected_proposals':[e['data'] for e in events if e['kind'] in ('GROUNDING_CONSISTENCY_REJECTED','PROPOSAL_REJECTED')],'failures':[e['data'] for e in events if e['kind'] in ('GOAL_FAILED','EPISODE_ERROR')],'raw_first_adoptions':[e['data'] for e in events if e['kind']=='ADOPTION'],'scope':'Raw stage extraction only. Provider span sums are not QA01/03 or a parallel critical path. Token/cost nulls remain explicit unknown_usage; no semantic accuracy claim.'})
out={'source_runs_file':str(raw.resolve()),'source_runs_sha256':hashlib.sha256(raw.read_bytes()).hexdigest(),'rows':rows,'provider_calls_created':0}
target=a.output or a.phase_root.parent/'analysis'/('dp04-'+a.phase_root.name+'-stage-evidence.json');target.parent.mkdir(parents=True,exist_ok=True);target.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'output':str(target),'dp04_rows':len(rows),'provider_calls_created':0}))
