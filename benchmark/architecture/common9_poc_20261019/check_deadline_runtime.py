"""Actual Rust receipt/cancel acceptance with deterministic outputs; no API."""
import copy,hashlib,json,queue,subprocess,threading,time
from pathlib import Path
import harness,offline
from bridge import dump
ROOT=harness.DEFAULT/'development'/('deadline-runtime-'+str(time.monotonic_ns()));ROOT.mkdir(parents=True,exist_ok=False)
FIXTURES=json.loads((harness.H/'fixtures.json').read_text())

def run(configuration,mode):
 ep=copy.deepcopy(next(e for e in FIXTURES if e['id']=='S02'));event=copy.deepcopy(ep['schedule'][0]['event'])
 proc=subprocess.Popen([str(harness.CRATE/'target/debug/common9_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1)
 messages=queue.Queue();events=[];calls=[];controls=[];ended=False;held=None;first_display=None;receipt_sent=False;deadline=None;awaiting_deadline=False;withdraw_sent=False
 def send(value):
  before=time.monotonic_ns();proc.stdin.write(json.dumps(value,ensure_ascii=False)+'\n');proc.stdin.flush();controls.append({'value':copy.deepcopy(value),'send_begin_ns':before,'send_end_ns':time.monotonic_ns()})
 def reader():
  for line in proc.stdout:
   value=json.loads(line);value['observed_ns']=time.monotonic_ns();messages.put(value)
  messages.put(None)
 def receipt(display,accepted=True,publication=None):
  return {'type':'publication_receipt','publication_id':publication or display['data']['publication_id'],'display_event_id':display['event_id'],'observed_ns':display['observed_ns'],'observation_deadline_ns':deadline,'accepted':accepted}
 threading.Thread(target=reader,daemon=True).start();send({'id':'DEADLINE-ACCEPTANCE','configuration':harness.config(configuration),'initial':ep['initial']})
 event.update(source_monotonic_ns=time.monotonic_ns(),source_node='source:'+event['id']);send({'type':'input_event','event':event});started=time.monotonic()
 while time.monotonic()-started<15:
  try:message=messages.get(timeout=.05)
  except queue.Empty:continue
  if message is None:break
  events.append(message);kind=message.get('kind');data=message.get('data',{})
  if message.get('type')=='model_call':
   payload=message.get('payload',{});value=offline.respond(payload,{'stage':message['stage']});calls.append({'message':message,'value':copy.deepcopy(value)})
   if message['stage']=='meaning_interpret_and_respond'and mode in ['cancel-before-meaning-return','deadline-before-meaning-return']and held is None:
    held=(message,value)
    if mode=='cancel-before-meaning-return':send({'type':'cancel_work','reason':'OFFLINE_INJECTED_CANCELLATION'})
    else:deadline=time.monotonic_ns()+100_000_000;awaiting_deadline=True;send({'type':'observation_deadline','deadline_ns':deadline,'update_id':1})
   else:send({'call_id':message['call_id'],'result':value})
   continue
  if kind=='WORK_CANCELLATION_ACK'and mode=='cancel-before-meaning-return'and held is not None:
   request,value=held;send({'call_id':request['call_id'],'result':value});send({'type':'end_input'});ended=True
  if kind=='OBSERVATION_DEADLINE_ACK'and awaiting_deadline:
   assert data['accepted']is True
   time.sleep(max(0,(deadline-time.monotonic_ns())/1e9)+.01)
   if mode=='deadline-before-meaning-return':request,value=held;send({'call_id':request['call_id'],'result':value});send({'type':'cancel_work','reason':'OFFLINE_DEADLINE'});send({'type':'end_input'});ended=True
   elif mode=='late-ack':send(receipt(first_display));receipt_sent=True
   awaiting_deadline=False
  if kind=='DISPLAY_PROXY'and first_display is None:
   first_display=message
   if mode=='duplicate-unknown-ack':
    send(receipt(message,publication='unknown-publication'));send(receipt(message));send(receipt(message));receipt_sent=True
   elif mode=='rejected-ack':send(receipt(message,False));receipt_sent=True
   elif mode=='late-ack':deadline=time.monotonic_ns()+100_000_000;awaiting_deadline=True;send({'type':'observation_deadline','deadline_ns':deadline,'update_id':1})
   elif mode=='source-withdrawn-during-ack':
    source=copy.deepcopy(next(s for s in ep['initial']['sources']if s['id']=='F01'));source['revoked']=True;send({'type':'input_event','event':{'kind':'source','source':source,'source_monotonic_ns':time.monotonic_ns()}});withdraw_sent=True
   elif mode!='missing-ack':send(receipt(message));receipt_sent=True
  if kind=='STATE_EFFECT_COMMIT'and withdraw_sent and not receipt_sent and data.get('effect_kind')=='SOURCE_REGISTER'and data.get('data',{}).get('source_id')=='F01':send(receipt(first_display));receipt_sent=True
  if kind in ['WORKFLOW_FAILED','GOAL_COMPLETE']and not ended:
   if deadline is not None:send({'type':'cancel_work','reason':'OFFLINE_TEST_FINISH'})
   send({'type':'end_input'});ended=True
 if proc.poll()is None:proc.terminate();proc.wait(timeout=5)
 result=next((e['data']for e in events if e.get('kind')=='EPISODE_RESULT'),{})
 goals=[e for e in events if e.get('kind')=='GOAL_COMPLETE'];displays=[e for e in events if e.get('kind')=='DISPLAY_PROXY'];adoptions=[e for e in events if e.get('kind')=='SEMANTIC_ADOPTION'];failures=[e for e in events if e.get('kind')=='WORKFLOW_FAILED'];receipt_rejections=[e for e in events if e.get('kind')=='PUBLICATION_RECEIPT_REJECTED']
 assert proc.returncode==0,(configuration,mode,proc.returncode)
 if mode in ['normal-ack','duplicate-unknown-ack']:assert len(goals)==1 and len(result.get('delivered',[]))==1 and not failures
 else:assert not goals and failures
 if mode=='duplicate-unknown-ack':assert len(receipt_rejections)>=2
 if mode in ['cancel-before-meaning-return','deadline-before-meaning-return']:assert not displays and not adoptions and calls
 if mode in ['late-ack','rejected-ack','missing-ack']:assert len(displays)==1 and not result.get('delivered')
 if mode=='missing-ack':assert any(e.get('kind')=='PUBLICATION_RECEIPT_TIMEOUT'for e in events)
 if mode=='source-withdrawn-during-ack':
  assert len(displays)==1 and len(result['delivered'])==1 and result['delivered'][0]['current_completion_eligible']is False
  assert 'SOURCE_PERMISSION_CHANGED'in failures[-1]['data']['reason']
  assert not any(e.get('kind')=='STATE_EFFECT_COMMIT'and e['data']['effect_kind']=='SOURCE_REGISTER'and e['data']['data'].get('publication_id')for e in events)
 row={'binary_sha256':hashlib.sha256((harness.CRATE/'target/debug/common9_poc').read_bytes()).hexdigest(),'configuration':configuration,'mode':mode,'status':'PASS','actual_API_calls':0,'returncode':proc.returncode,'events':events,'calls':calls,'controls':controls,'stderr':proc.stderr.read(),'goal_callbacks':len(goals),'raw_display_callbacks':len(displays),'delivered_history_rows':len(result.get('delivered',[])),'scope':'Actual Rust + deterministic test-only responses/control interleavings; no performance/model-quality claim'};dump(ROOT/(configuration+'-'+mode+'.json'),row)
 return {key:row[key]for key in ['configuration','mode','status','goal_callbacks','raw_display_callbacks','delivered_history_rows']}

if __name__=='__main__':
 rows=[run(c,m)for c in harness.CONFIGS for m in ['normal-ack','cancel-before-meaning-return']]
 rows.extend(run('AAAA',mode)for mode in ['duplicate-unknown-ack','rejected-ack','late-ack','missing-ack','deadline-before-meaning-return','source-withdrawn-during-ack'])
 dump(ROOT/'summary.json',{'status':'PASS','rows':rows,'count':len(rows),'actual_API_calls':0});print(json.dumps({'root':str(ROOT),'count':len(rows),'status':'PASS','actual_API_calls':0}))
