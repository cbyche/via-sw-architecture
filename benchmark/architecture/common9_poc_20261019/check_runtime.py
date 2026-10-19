"""Offline runtime guards; synthetic doubles and injected faults, zero API calls."""
import copy,json,queue,subprocess,threading,time,urllib.request
from pathlib import Path
import harness,offline
from bridge import Client,ModelError,dump
from memory import LogicalBuffers
from lifecycle import ObservationGate
H=Path(__file__).resolve().parent;OUT=harness.DEFAULT/'development'/('runtime-guards-'+str(time.monotonic_ns()));OUT.mkdir(parents=True,exist_ok=True)

def pending_agent_capture():
 ep=next(e for e in json.loads((H/'fixtures.json').read_text())if e['id']=='S18');initial={'id':'GUARD','configuration':harness.config('AAAA'),'initial':ep['initial']}
 p=subprocess.Popen([str(harness.CRATE/'target/debug/common9_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1);lock=threading.Lock();msgs=queue.Queue();trace=[];threads=[];sent2=False;begun=False;closed=False
 def send(v):
  with lock:p.stdin.write(json.dumps(v,ensure_ascii=False)+'\n');p.stdin.flush()
 def event(e):
  e=copy.deepcopy(e);e['source_monotonic_ns']=time.monotonic_ns();e['source_node']='source:'+e['id'];send({'type':'input_event','event':e})
 def reader():
  for line in p.stdout:msgs.put(json.loads(line))
  msgs.put(None)
 threading.Thread(target=reader,daemon=True).start();send(initial);first=ep['schedule'][0]['event'];second=ep['schedule'][1]['event'];event(first);start=time.monotonic()
 def finish_transcript():
  time.sleep(.05);event({'id':'GUARD-U01','kind':'user','text':'TCP가 뭐야?','delivery_source_id':'GUARD:delivered'})
 while time.monotonic()-start<15:
  try:m=msgs.get(timeout=.1)
  except queue.Empty:continue
  if m is None:break
  trace.append(m);kind=m.get('kind');d=m.get('data',{})
  if kind=='DISPLAY_PROXY'and d.get('publication_id'):send({'type':'publication_receipt','publication_id':d['publication_id'],'display_event_id':m['event_id'],'observed_ns':time.monotonic_ns(),'observation_deadline_ns':None,'accepted':True})
  if m.get('type')=='model_call':
   payload=m.get('payload',{});result=offline.respond(payload,{'stage':m['stage']});send({'call_id':m['call_id'],'result':result});continue
  if kind=='GOAL_COMPLETE'and d.get('stimulus_id')==first['id']and not sent2:event(second);sent2=True
  if kind=='CONTROL_INGRESS'and not begun:
   ingress=harness.wire_data(d,'request');send({'type':'input_event','event':{'id':'GUARD-U01','kind':'input_begin','source_monotonic_ns':time.monotonic_ns()}})
   result=harness.actual_external({'id':'S18-A01','kind':'agent','text':'actualresult','event_class':'RESULT'},ingress,{'node_id':None});event(result);begun=True;t=threading.Thread(target=finish_transcript);threads.append(t);t.start()
  if kind=='DISPLAY_PROXY'and d.get('stimulus_id')=='S18-A01'and not closed:send({'type':'end_input'});closed=True
 for t in threads:t.join()
 if p.poll()is None:
  if not closed:p.terminate()
  p.wait(timeout=5)
 captures=[m['data']['stimulus_id']for m in trace if m.get('kind')=='INPUT_CAPTURED'];shown=[m['data']['stimulus_id']for m in trace if m.get('kind')=='DISPLAY_PROXY'];waits=[m for m in trace if m.get('kind')=='POLICY_WAIT_START']
 result={'name':'user_begin_agent_result_then_transcript_capture_liveness','passed':p.returncode==0 and 'GUARD-U01'in captures and 'S18-A01'in shown and bool(waits),'returncode':p.returncode,'captures':captures,'shown':shown,'wait_count':len(waits),'events':trace,'stderr':p.stderr.read(),'scope':'explicit transport interleaving; no API or performance claim'};dump(OUT/'pending-agent.json',result);assert result['passed'],result
 return result['name']

def missing_agent_anchor():
 old=harness.OfflineClient
 class Injected(old):
  def call(self,payload,meta):
   value,record=super().call(payload,meta)
   if meta['stage']=='meaning_interpret_and_respond'and payload.get('input',{}).get('event',{}).get('id')=='S18-U02':
    value['control_operations'][0]['task_id']='NO-SUCH-ACTUAL-TASK';record['injected_failure']='unknown actual Task at control ingress'
   return value,record
 harness.OfflineClient=Injected
 try:
  ep=next(e for e in json.loads((H/'fixtures.json').read_text())if e['id']=='S18');trace=harness.execute(OUT/'missing-anchor',ep,'AAAA',1,True,.03)
 finally:harness.OfflineClient=old
 ok='S18-U03'in trace['provided_sources']and'S18-A01'not in trace['provided_sources']and any(f['kind']=='MISSING_INGRESS_OR_AGENT_ANCHOR'and f['event_id']=='S18-A01'for f in trace['host_failures'])and not any(f['kind']=='HOST_EPISODE_DEADLINE'for f in trace['host_failures'])
 dump(OUT/'missing-anchor-check.json',{'passed':ok,'run_id':trace['run_id'],'original_planned_responses':7,'observed_sources':len(trace['provided_sources']),'failures_preserved':trace['host_failures']});assert ok
 return 'missing_S18_U02_ingress_does_not_deadlock_dependent_U03'

def bridge_exception_slots():
 client=Client.__new__(Client);client.observation_gate=ObservationGate();client.root=OUT/'mock-bridge';client.root.mkdir(exist_ok=True);client.ledger_root=client.root;client.limit=4;client.key='OFFLINE-FAKE-NO-CREDENTIAL';client.url='https://invalid.example.invalid';client.lock=threading.Lock();client.resource_lock=threading.Lock();client.buffers=LogicalBuffers();client.slot_owners={};client.last_slot_owner={};client.configure_slots(2);records=[]
 original=urllib.request.urlopen
 def fail(*args,**kwargs):raise OSError('OFFLINE injected transport exception; network never attempted')
 urllib.request.urlopen=fail
 def call():
  try:client.call({'input':{'guard':'offline'},'max_tokens':1},{'stage':'GUARD','max_calls_per_run':30})
  except ModelError as e:records.append(e.record)
 try:
  ts=[threading.Thread(target=call)for _ in range(16)]
  for t in ts:t.start()
  for t in ts:t.join(timeout=5)
 finally:urllib.request.urlopen=original
 ok=len(records)==16 and all('StopIteration'not in r['error']for r in records)and not client.slot_owners
 dump(OUT/'bridge-slot-exception.json',{'passed':ok,'calls':len(records),'owners_remaining':client.slot_owners,'errors':[r['error']for r in records],'all_unknown_reservations_preserved':all(r['cost_usd']is None for r in records),'no_real_API_calls':True});assert ok
 return 'bridge_exception_releases_owner_before_semaphore'
def bridge_blocks_before_request():
 client=Client.__new__(Client);client.observation_gate=ObservationGate();client.root=OUT/'budget-block';client.root.mkdir();client.ledger_root=client.root;client.limit=0;client.lock=threading.Lock();client.buffers=LogicalBuffers();client.configure_slots(2);client.key='OFFLINE-FAKE';client.url='https://invalid.example.invalid';records=[]
 for meta in [{'stage':'BUDGET_GUARD','max_calls_per_run':100},{'stage':'CAP_GUARD','max_calls_per_run':0}]:
  try:client.call({'input':{'offline':'guard'},'max_tokens':1},meta)
  except ModelError as error:records.append(error.record)
 assert [r['status']for r in records]==['BUDGET_BLOCKED_NO_API','CALL_CAP_BLOCKED_NO_API'];assert all(r['actual_api_sent']is False and r['cost_usd']==0 for r in records)
 dump(OUT/'budget-callcap-block.json',{'passed':True,'records':records,'actual_API_calls':0});return 'budget_and_callcap_block_before_API_request'
def control_question_ownership():
 old=harness.OfflineClient
 class Injected(old):
  def call(self,payload,meta):
   value,record=super().call(payload,meta)
   if meta['stage']=='meaning_interpret_and_respond'and payload.get('input',{}).get('event',{}).get('id')=='S20-U02':value['control_operations'][0]['question_id']='Q-TB';record['injected_failure']='actual TA paired with question actually owned by TB'
   return value,record
 harness.OfflineClient=Injected
 try:trace=harness.execute(OUT/'question-ownership',next(e for e in json.loads((H/'fixtures.json').read_text())if e['id']=='S20'),'AAAA',1,True,.03)
 finally:harness.OfflineClient=old
 fail=any(e.get('kind')=='WORKFLOW_FAILED'and 'CONTROL_QUESTION_NOT_DISPLAYED'in e.get('data',{}).get('reason','')for e in trace['events']);sent=any(e.get('kind')=='CONTROL_INGRESS'and harness.wire_data(e.get('data',{}),'request').get('stimulus_id')=='S20-U02'for e in trace['events']);assert fail and not sent
 dump(OUT/'question-ownership-check.json',{'passed':True,'run_id':trace['run_id'],'wrong_model_output_preserved':True,'actual_ingress_rejected':True,'actual_API_calls':0});return 'control_same_Task_question_ownership_atomicguard'
def ps_failure_evidence():
 import memory
 old=memory.subprocess.run
 memory.subprocess.run=lambda *a,**k:subprocess.CompletedProcess(a[0],1,'','OFFLINE injected ps denial')
 try:sample=memory.sample_processes({12345:'rust_candidate'},'OFFLINE_PS_FAILURE',LogicalBuffers())
 finally:memory.subprocess.run=old
 assert sample['ps_returncode']==1 and sample['ps_stderr']=='OFFLINE injected ps denial'and sample['all_registered_observed']is False
 dump(OUT/'ps-failure-evidence.json',{'passed':True,'sample':sample,'scope':'mocked psfailure, not resource measurement'});return 'ps_failure_preserves_returncode_stderr_and_missing_PID'
def external_artifact_provenance():
 spans=[{'source_id':'DOC','revision':1,'start_line':1,'end_line':1}];actual={'kind':'DELEGATE','stimulus_id':'actual-input','contract':{'mandatory':{'task_id':'ACTUAL-TASK','purpose':'draft','targets':['DOC:v1:P1']}}};receipt={'actual_result':{'receiver_result':{'text':'actual receiver body'},'actually_supplied_source_spans':spans}};result=harness.actual_external({'id':'actual-result','kind':'agent'},actual,receipt);assert result['result_source']['provenance']==spans;dump(OUT/'external-artifact-provenance.json',{'passed':True,'result':result,'actual_API_calls':0});return 'actual_receiver_readset_attached_to_derived_artifact'
if __name__=='__main__':
 checks=[pending_agent_capture(),missing_agent_anchor(),bridge_exception_slots(),bridge_blocks_before_request(),control_question_ownership(),ps_failure_evidence(),external_artifact_provenance()];dump(OUT/'summary.json',{'status':'PASS','checks':checks,'actual_API_calls':0});print(json.dumps(checks))
