"""Actual Rust protocol acceptance with deterministic doubles and old raw replay; no API."""
import copy,json,queue,subprocess,threading,time
from pathlib import Path
import harness,offline
H=Path(__file__).resolve().parent;OUT=harness.DEFAULT/'development'/('protocol-v2-'+str(time.monotonic_ns()));OUT.mkdir(parents=True)
FIXTURES=json.loads((H/'fixtures.json').read_text());OLD=harness.DEFAULT/'formal-v1'
old_trace=json.loads(next((OLD/'raw/runs').glob('AAAA-S02-r1-*/trace.json')).read_text())
REPLAYS=[{'event_id':c['causal_trace']['goal_ids'][0].removeprefix('input:'),'proposal':c['response_value']}for c in old_trace['calls']if c['stage']=='meaning_interpret_and_respond']

def run(cfg,mode):
 ep=copy.deepcopy(next(x for x in FIXTURES if x['id']==('S26'if mode in ['valid-consent','permission-then-fail','batch-atomic']else'S02')))
 replay=REPLAYS[int(mode.rsplit('-',1)[1])-1]if mode.startswith('old-raw-replay-')else None
 event=copy.deepcopy(next(x['event']for x in ep['schedule']if x['event_id']==replay['event_id'])if replay else ep['schedule'][1 if mode in ['valid-consent','permission-then-fail','batch-atomic']else 0]['event'])
 proc=subprocess.Popen([str(harness.CRATE/'target/debug/common9_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1)
 q=queue.Queue();trace=[];calls=[];meanings=[];ended=False;serial=0
 def send(v):proc.stdin.write(json.dumps(v,ensure_ascii=False)+'\n');proc.stdin.flush()
 def read():
  for line in proc.stdout:q.put(json.loads(line))
  q.put(None)
 threading.Thread(target=read,daemon=True).start()
 send({'id':'PROTOCOL-ACCEPTANCE','configuration':harness.config(cfg),'initial':ep['initial']})
 event.update(source_monotonic_ns=time.monotonic_ns(),source_node='source:'+event['id']);send({'type':'input_event','event':event})
 start=time.monotonic()
 while time.monotonic()-start<20:
  try:m=q.get(timeout=.1)
  except queue.Empty:continue
  if m is None:break
  trace.append(m)
  if m.get('kind')=='DISPLAY_PROXY'and m.get('data',{}).get('publication_id'):
   send({'type':'publication_receipt','publication_id':m['data']['publication_id'],'display_event_id':m['event_id'],'observed_ns':time.monotonic_ns(),'observation_deadline_ns':None,'accepted':True})
  if m.get('type')=='model_call':
   payload=m.get('payload',{});result=offline.respond(payload,{'stage':m['stage']})
   if m['stage']=='meaning_interpret_and_respond':
    serial+=1
    if replay and serial==1:result=copy.deepcopy(replay['proposal'])
    elif mode in ['permission-range','literal-revision-key','numeric-claim-id','literal-slash-n','exhaustion']and(serial==1 or mode=='exhaustion'):
     if mode in ['permission-range','exhaustion']:result['permission_operations']=[{'source_id':'F01','read_segments':['1-2']}]
     elif mode=='literal-revision-key':result['source_revisions']={'source_id':1}
     elif mode=='numeric-claim-id':result['source_claims'][0]['source_id']=1
     elif mode=='literal-slash-n':result['source_claims'][0]['quote']=next(p['text']for p in payload['input']['actually_supplied_originals']if p['source_id']=='F01').replace('\n','\\n')
    elif mode=='permission-then-fail'and serial>1:result['source_revisions']={'source_id':1}
    elif mode=='batch-atomic':result['permission_operations']=[{'source_id':'F-PRIVATE','read_segments':['body']},{'source_id':'F-PRIVATE','external_segments':['1-2']}]
    meanings.append({'attempt':serial,'payload':payload,'returned_proposal':copy.deepcopy(result)})
   calls.append({'stage':m['stage'],'payload':payload,'result':result});send({'call_id':m['call_id'],'result':result});continue
  if m.get('kind')in ['WORKFLOW_FAILED','GOAL_COMPLETE']and m.get('data',{}).get('event_id',m.get('data',{}).get('stimulus_id'))==event['id']and not ended:
   send({'type':'end_input'});ended=True
 if proc.poll()is None:
  if not ended:proc.terminate()
  proc.wait(timeout=5)
 failures=[m for m in trace if m.get('kind')=='WORKFLOW_FAILED'];rejected=[m for m in trace if m.get('kind')=='MEANING_PROPOSAL_REJECTED'];applied=[m for m in trace if m.get('kind')=='PERMISSION_OPERATION_APPLIED'];goals=[m for m in trace if m.get('kind')=='GOAL_COMPLETE'];shown=[m for m in trace if m.get('kind')=='DISPLAY_PROXY']
 assert proc.returncode==0,(cfg,mode,proc.returncode)
 assert len(meanings)<=3,(cfg,mode,len(meanings))
 if mode in ['exhaustion','permission-then-fail','batch-atomic']:
  assert len(meanings)==3 and len(failures)==1 and not shown and not goals
  assert 'MEANING_TOTAL_LIMIT_EXHAUSTED'in failures[0]['data']['reason']
  if mode=='permission-then-fail':
   assert len(applied)==1
   current=next(s for s in meanings[-1]['payload']['input']['source_catalog']if s['id']=='F-PRIVATE')
   assert current['allowed_segments']==[[1,2]]and current['external_segments']==[]
  else:assert not applied
  if mode=='batch-atomic':
   for call in meanings:assert next(s for s in call['payload']['input']['source_catalog']if s['id']=='F-PRIVATE')['allowed_segments']==[]
 elif mode=='valid-consent':assert len(applied)==1 and len(meanings)==2 and goals and not failures
 else:
  assert len(meanings)==2 and len(rejected)==1 and not applied and goals and not failures
  expected='not present in actual catalog'if replay or mode=='literal-revision-key'else {'permission-range':'segment not declared by source','numeric-claim-id':'claim source_id must be a string','literal-slash-n':'not an exact substring'}[mode]
  assert expected in rejected[0]['data']['observed_error'],rejected[0]['data']['observed_error']
  assert rejected[0]['data']['proposal']==meanings[0]['returned_proposal'],'Rejected raw proposal was altered'
  assert meanings[1]['payload']['input']['repair_reason']['observed_validation_error']==rejected[0]['data']['observed_error']
  assert set(meanings[1]['payload']['input']['repair_reason'])=={'observed_validation_error','meaning_calls_used','meaning_calls_total_limit'},'Repair must contain error/counter only'
 row={'configuration':cfg,'mode':mode,'status':'PASS','meaning_calls':len(meanings),'rejected_proposals':len(rejected),'permission_batches_applied':len(applied),'goal_callbacks':len(goals),'workflow_failures':len(failures),'actual_API_calls':0,'trace':trace,'calls':calls,'stderr':proc.stderr.read()};path=OUT/(cfg+'-'+mode+'.json');path.write_text(json.dumps(row,ensure_ascii=False,indent=2)+'\n');return {k:row[k]for k in ['configuration','mode','status','meaning_calls','rejected_proposals','permission_batches_applied','goal_callbacks','workflow_failures']}
rows=[]
for cfg in harness.CONFIGS:
 for mode in [*(f'old-raw-replay-{i}'for i in range(1,7)),'permission-range','literal-revision-key','numeric-claim-id','literal-slash-n','exhaustion','valid-consent','permission-then-fail','batch-atomic']:rows.append(run(cfg,mode))
summary={'status':'PASS','rows':rows,'count':len(rows),'actual_API_calls':0,'scope':'Actual Rust with predetermined test-only model outputs; failure rejection/repair/counter/authority acceptance, not real model accuracy or performance'};(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({'status':'PASS','count':len(rows),'out':str(OUT),'actual_API_calls':0}))
