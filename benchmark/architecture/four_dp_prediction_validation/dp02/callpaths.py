"""Descriptive actual bridge queues and Rust policy waits; no endpoint changes."""
import csv,json,argparse
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('root',type=Path);a=p.parse_args();rows=[];runs=[]
for line in(a.root/'runs.jsonl').read_text().splitlines():
 r=json.loads(line);policy=[];open_={}
 for e in r['events']:
  d=e.get('data',{});k=e.get('kind','');key=(d.get('stimulus_id'),d.get('goal_id'),d.get('reason'))
  if k=='POLICY_WAIT_START':open_.setdefault(key,[]).append(e)
  if k=='POLICY_WAIT_END':
   # Start/end reason can differ ('step boundary' vs 'step ready'); identity remains.
   match=next((key0 for key0,es in open_.items()if es and key0[:2]==key[:2]),None)
   if match:
    before=open_[match].pop(0);policy.append({'identity':key[:2],'start':before['data'],'end':d,'elapsed_ms':(e['elapsed_ns']-before['elapsed_ns'])/1e6})
 for key,es in open_.items():
  for e in es:policy.append({'identity':key[:2],'start':e['data'],'end':None,'elapsed_ms':None})
 cs=[]
 for c in sorted(r['calls'],key=lambda c:c.get('enqueue_ns',0)):
  data={}
  if c.get('request_file'):
   body=json.loads(Path(c['request_file']).read_text());content=body['messages'][1]['content'];data=json.loads(content)if isinstance(content,str)else{}
  ev=data.get('event',{});num=lambda k:c.get(k)
  def diff(end,start):return(c[end]-c[start])/1e6 if c.get(end)is not None and c.get(start)is not None else None
  x={'run_id':r['run_id'],'episode':r['episode'],'variant':r['variant'],'rep':r['repetition'],'stage':c['stage'],'event_id':ev.get('id'),'goal_id':data.get('goal_id'),'revision':data.get('current_revision',ev.get('revision')),'call_uuid':c.get('call_uuid'),'resource_slot':c.get('resource_slot'),'resource_predecessor_call_uuid':c.get('resource_predecessor_call_uuid'),'pre_slot_bridge_ms':diff('before_slot_wait_ns','enqueue_ns'),'pure_slot_wait_ms':diff('slot_acquired_ns','before_slot_wait_ns'),'slot_to_send_ms':diff('send_ns','slot_acquired_ns'),'queue_and_bridge_ms':diff('send_ns','enqueue_ns'),'actual_API_span_ms':diff('terminal_ns','send_ns'),'input_tokens':c.get('input_tokens'),'output_tokens':c.get('output_tokens'),'cost_usd':c.get('cost_usd'),'status':c.get('status')};rows.append(x);cs.append(x)
 runs.append({'run_id':r['run_id'],'episode':r['episode'],'variant':r['variant'],'rep':r['repetition'],'policy_waits':policy,'calls':cs})
with(a.root/'callpaths.csv').open('w')as f:
 w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
(a.root/'callpaths.json').write_text(json.dumps({'clock_note':'Bridge spans share monotonic_ns. Rust policy waits share one Instant origin. API overlap is actual send-to-return envelope, not GPU scheduling proof. No duplicate sum of policy+slot into QA01/03.','runs':runs},indent=2)+'\n');print({'calls':len(rows),'runs':len(runs)})
