"""Contract20 supplemental actual-time fork; future schedule/oracles remain host-only."""
import argparse,copy,hashlib,heapq,json,os,queue,re,subprocess,sys,threading,time,uuid
from pathlib import Path
COMMON=Path(__file__).resolve().parent.parent/'common9_poc_20261019'
sys.path.append(str(COMMON));sys.path.append(str(Path(__file__).resolve().parent))
from bridge import Client,ModelError,dump
from memory import LogicalBuffers,sample_processes
from lifecycle import ObservationGate,ObservationCancelled
from trigger_rules import FinalAnchorRegistry
H=Path(__file__).resolve().parent;REPO=H.parents[2];CRATE=REPO/'prototypes/candidates/common9_poc_20261019';DEFAULT=REPO/'results/architecture-evaluation/current/common9-poc-20261019'
CONFIGS=['AAAA','BAAA','CAAA','ABAA','AABA','AAAB','AAAC']
def read(p):return json.loads(Path(p).read_text())
def config(name):return dict(zip(['id','evidence','execution','grounding','handoff'],[name,*name]))
def wire_data(data,kind):return data.get(kind,data.get('envelope',{}).get(kind,data))
def actual_external(event,ingress,receipt):
 """Fixed capable adapter reacts to actual submitted operations/contract, never goal oracle."""
 result=copy.deepcopy(event);result['caused_by']=ingress['stimulus_id'];result['receiver_node']=receipt.get('node_id');result['actual_ingress']=ingress;result['actual_receipt']=receipt
 if ingress['kind']=='CONTROL':
  facts=[];artifacts=[]
  for op in ingress.get('control_operations',[]):
   task=op.get('task_id','UNKNOWN');action=str(op.get('action',''));value=str(op.get('value',''));disposition=op.get('disposition')
   if disposition=='DENY':facts.append(f'{task} 미발송 확인. 명시적 거부를 유지했다.')
   elif disposition=='APPROVE' and ('send' in action.lower() or '발송' in action):facts.append(f'{task} 발송 완료.')
   elif disposition in ['UPDATE','APPROVE']and action in ['revise_report','set_format']:
    facts.append(f'{task} 수정 결과: {action} / {value}.')
    artifacts.append({'id':task+':revised-result','revision':1,'title':task+' actual revised result','kind':'artifact','text':f'{task} 수정 결과: {action} / {value}.','local_allowed':True,'external_allowed':True,'extents':{'revised-result':[1,1]}})
   else:facts.append(f'{task} 지원되지 않는 제어 또는 확인되지 않은 상태. {action}/{disposition}')
  result['text']=' '.join(facts)or'제어가 접수되지 않아 결과를 확인할 수 없다.'
  if artifacts:result['result_source']=artifacts[0]
  result['task_id']=ingress.get('control_operations',[{}])[0].get('task_id')
 else:
  contract=ingress['contract'];m=contract['mandatory'];actual=receipt.get('actual_result',{});body=actual.get('receiver_result',{}).get('text','');targets=', '.join(map(str,m.get('targets',[])))
  result['task_id']=m.get('task_id');result['text']=f"{m.get('task_id')} 업무 결과 파일 artifact-{ingress['stimulus_id']}.txt. 실제 목적: {m.get('purpose')}. 적용 대상: {targets}. {body}"
  result['result_source']={'id':f"{m.get('task_id')}:result",'revision':1,'provenance':copy.deepcopy(actual.get('actually_supplied_source_spans',[])),'title':'Actual synthetic downstream artifact','kind':'artifact','text':result['text'],'local_allowed':True,'external_allowed':True,'extents':{'result':[1,1]}}
 return result
class OfflineClient:
 def __init__(self,root):
  import supplement_offline as offline
  self.root=root;self.buffers=LogicalBuffers();self.semaphore=threading.Semaphore(2);self.lock=threading.Lock();self.observation_gate=ObservationGate()
 def call(self,payload,meta):
  import supplement_offline as offline
  self.observation_gate.check();at=time.monotonic_ns();result=offline.respond(payload,meta);end=time.monotonic_ns();return result,{**meta,'input_text_bytes':len(json.dumps(payload.get('input',{}),ensure_ascii=False).encode()),'instruction_bytes':len(payload.get('instruction','').encode()),'max_output_tokens':payload.get('max_tokens',1800),'offline_response_utf8_bytes':len(json.dumps(result,ensure_ascii=False).encode()),'image_bytes':sum(Path(p).stat().st_size for p in payload.get('images',[])),'call_uuid':'OFFLINE-'+uuid.uuid4().hex,'causal_trace':payload.get('_trace',{}),'enqueue_ns':at,'send_ns':at,'terminal_ns':end,'status':'OFFLINE_DOUBLE','input_tokens':0,'output_tokens':0,'cost_usd':0,'role':payload.get('role','VIA'),'evidence_kind':'MEASURED_REFERENCE_HARNESS','response_value':result}
 def configure_slots(self,k):self.semaphore=threading.Semaphore(k)

def execute(root,episode,configuration,rep,offline=False,time_scale=1,input_transform=None):
 if not offline and time_scale!=1:raise ValueError('Formal authored times cannot be scaled')
 folder=root/'raw'/'runs'/f'{configuration}-{episode["id"]}-r{rep}-{uuid.uuid4().hex[:6]}';folder.mkdir(parents=True,exist_ok=False)
 initial={'id':episode['id'],'configuration':config(configuration),'initial':episode['initial']};dump(folder/'candidate-initial.json',input_transform(initial)if input_transform else initial)
 binary=Path(read(root/'freeze.json')['binary']['path'])if(root/'freeze.json').exists()else CRATE/'target/debug/common9_poc'
 if not offline:
  from campaign import validate
  assert validate(root)['purpose']in ['FORMAL_REVIEW_CANDIDATE','DEVELOPMENT_COMPATIBILITY_PROBE_REVIEW_CANDIDATE']
  auth=read(root/'MODEL-CALLS-AUTHORIZED.json');assert auth['freeze_sha256']==hashlib.sha256((root/'freeze.json').read_bytes()).hexdigest();assert {'configuration':configuration,'episode':episode['id'],'rep':rep}in auth['allowed_jobs']
 client=OfflineClient(root)if offline else Client(root,limit=read(DEFAULT/'budget.json')['limit_usd'],ledger_root=DEFAULT)
 observation_gate=ObservationGate();client.observation_gate=observation_gate
 if not offline:client.scope_limit_usd=read(root/'freeze.json').get('scope_limit_usd')
 client.configure_slots(2);client.run_call_count=0
 adapter=None
 if not offline:
  from grounding_adapter import GroundingAdapter
  adapter=GroundingAdapter(root,client.buffers,observation_gate)
 rust_spawn_begin_ns=time.monotonic_ns();proc=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1);rust_spawn_return_ns=time.monotonic_ns();first_positive_rss_ns={};write_lock=threading.Lock();events=[];calls=[];memory=[];provided={};source_nodes=[];threads=[];messages=queue.Queue();stop=threading.Event();ready_signal=threading.Event();clock_ready=threading.Event();clock_validation_finished=threading.Event();clock_proof=None;pending=[];serial=0;started=time.monotonic_ns();host_origin=None;firstseen={};finalseen={};fallbackseen={};ingresses={};receipts={};host_display_receipts=[];scheduled=set();completed=set();failures=[];deadline_updates=[];observation_deadline_ns=None;cancellation_requested_ns=None;effective_admission_stop_ns=None;settlement_deadline_ns=None;technical_abort_reason=None;technical_abort_detection_ns=None;technical_abort_requested_bound_ns=None;control_sends=[];final_anchors=FinalAnchorRegistry();final_anchor_choices=final_anchors.choices;trigger_audit=[]
 def send(value):
  with write_lock:
   before=time.monotonic_ns();proc.stdin.write(json.dumps(input_transform(value)if input_transform else value,ensure_ascii=False)+'\n');proc.stdin.flush();after=time.monotonic_ns()
  return {'send_begin_ns':before,'send_end_ns':after,'provided_at_ns':after}
 def sampler():
  while not stop.is_set():
   roles={os.getpid():'python_bridge',proc.pid:'rust_candidate'}
   if adapter:roles.update(adapter.processes.copy())
   try:
    alive_before=proc.poll()is None;sample=sample_processes(roles,'COMMON9_ACTIVE'if ready_signal.is_set()else'COMMON9_STARTUP',client.buffers);exitcode=proc.poll();sample['process_start_metadata']={str(proc.pid):{'role':'rust_candidate','spawn_begin_ns':rust_spawn_begin_ns,'spawn_return_ns':rust_spawn_return_ns,'registered_at_ns':rust_spawn_return_ns}}
    if adapter:sample['process_start_metadata'].update({str(pid):value for pid,value in adapter.process_start_metadata.copy().items()})
    startup_zeros=[]
    for row in sample['processes']:
     pid=str(row['pid'])
     if row['rss_bytes']>0:first_positive_rss_ns.setdefault(pid,sample['at_ns'])
     elif pid not in first_positive_rss_ns and pid in sample['process_start_metadata']:startup_zeros.append(row['pid'])
    sample['first_positive_rss_ns']=first_positive_rss_ns.copy();sample['startup_zero_rss_pids']=startup_zeros;sample['candidate_lifecycle']={'pid':proc.pid,'alive_at_sample_start':alive_before,'exit_confirmed_by_sample_end':exitcode is not None,'exitcode':exitcode,'episode_ready_observed':ready_signal.is_set()};sample['confirmed_exited_pids']=[proc.pid]if exitcode is not None else[]
    if exitcode is not None:sample['stage']='COMMON9_TEARDOWN_CONFIRMED_EXIT'
    memory.append(sample)
   except Exception as error:
    failures.append({'kind':'MEMORY_SAMPLER_FAILED','at_ns':time.monotonic_ns(),'error':type(error).__name__+': '+str(error)});return
   stop.wait(.05)
 monitor=threading.Thread(target=sampler,daemon=True);monitor.start();initial_send=send(initial);initial_source_provided_ns=initial_send['provided_at_ns'];clock_probe_send=send({'type':'clock_probe','host_probe_id':'initial-clock-proof'})
 def model(message):
  payload=message.get('payload',message.get('request',{}).get('context',{}));meta={'run_id':folder.name,'episode':episode['id'],'configuration':configuration,'rep':rep,'stage':message['stage'],'max_calls_per_run':100}
  try:
   if not clock_validation_finished.wait(5)or not clock_ready.is_set():raise ObservationCancelled('HOST_CLOCK_NOT_VALIDATED_NO_API')
   if message['stage']=='grounding.local_screen_parser' and not offline:
    observation_gate.check();result,record=adapter.call(payload,meta)
   else:result,record=client.call(payload,meta)
   record.setdefault('causal_trace',payload.get('_trace',{}));record['response_value']=result;response={'call_id':message['call_id'],'result':result,'usage':record.get('raw_usage')}
   if message.get('wire_version')==2:response={'call_id':message['call_id'],'provider_response':{'json':result,'usage':{'input_tokens':record.get('input_tokens'),'output_tokens':record.get('output_tokens')}}}
  except ObservationCancelled as error:record={**meta,'status':'CANCELLED_NO_API','actual_api_sent':False,'cost_usd':0,'input_tokens':0,'output_tokens':0,'error':str(error),'terminal_ns':time.monotonic_ns(),'causal_trace':payload.get('_trace',{})};response={'call_id':message['call_id'],'error':str(error)}
  except ModelError as error:
   record=error.record;response={'call_id':message['call_id'],'error':str(error)}
   if record.get('status')in ['BUDGET_BLOCKED_NO_API','PROBE_BUDGET_BLOCKED_NO_API']:dump(root/'BUDGET-BLOCK-OBSERVED.json',record)
  except Exception as error:record={**meta,'status':'HOST_MODEL_ERROR','actual_api_sent':False,'error':type(error).__name__+': '+str(error),'terminal_ns':time.monotonic_ns(),'causal_trace':payload.get('_trace',{})};response={'call_id':message['call_id'],'error':str(error)}
  calls.append(record)
  try:send(response)
  except BrokenPipeError:record['discarded_after_candidate_end']=True
 def stdout():
  for line in proc.stdout:
   try:m=json.loads(line)
   except ValueError:messages.put({'kind':'INVALID_STDOUT','line':line});continue
   if m.get('kind')=='RESPONSE_FINAL' and m.get('data',{}).get('stimulus_id'):
    d=m['data'];parent=d.get('node_id')or next(reversed(m.get('parent_ids',[])),None);m['observed_ns']=final_anchors.observe(d['stimulus_id'],parent)
   else:m['observed_ns']=time.monotonic_ns()
   if m.get('kind')=='OBSERVATION_DEADLINE_ACK':
    d=m.get('data',{});control=d.get('control',{});update=next((u for u in deadline_updates if u['update_id']==control.get('update_id')),None)
    if update is not None:
     with observation_gate.lock:
      if observation_gate.cancelled_ns is not None:
       update['late_ack_after_cancellation']=m;messages.put(m);continue
      update['ack']=m
      timely=update['previous_deadline_ns']is None or m['observed_ns']<update['previous_deadline_ns']
      matching=control.get('deadline_ns')==update['deadline_ns']and control.get('last_actual_source_ns')==update['last_actual_source_ns']
      update['ack_status']='ACCEPTED'if d.get('accepted')is True and timely and matching else'REJECTED_OR_LATE_OR_MISMATCHED'
      if update['ack_status']=='ACCEPTED':observation_gate.pending_update_cutoffs.pop(update['update_id'],None)
   if m.get('kind')=='DISPLAY_PROXY':
    publication=m.get('data',{}).get('publication_id')
    if publication:
     with observation_gate.lock:
      cutoff=observation_gate.deadline_ns;cancelled=observation_gate.cancelled_ns;at=m['observed_ns'];accepted=clock_ready.is_set()and(cutoff is None or at<=cutoff)and(cancelled is None or at<cancelled)
      receipt={'type':'publication_receipt','publication_id':publication,'display_event_id':m['event_id'],'observed_ns':at,'observation_deadline_ns':cutoff,'accepted':accepted}
     try:bracket=send(receipt);host_display_receipts.append({**receipt,**bracket})
     except BrokenPipeError:failures.append({'kind':'PUBLICATION_RECEIPT_PIPE_CLOSED','receipt':receipt})
    else:failures.append({'kind':'DISPLAY_WITHOUT_TECHNICAL_PUBLICATION_ID','event_id':m.get('event_id')})
   if m.get('kind')=='EPISODE_READY':ready_signal.set()
   if m.get('type')=='model_call':t=threading.Thread(target=model,args=(m,),daemon=True);threads.append(t);t.start()
   else:messages.put(m)
  messages.put({'kind':'PROCESS_STDOUT_CLOSED','observed_ns':time.monotonic_ns()})
 reader=threading.Thread(target=stdout,daemon=True);reader.start()
 specs={s['event_id']:s for s in episode['schedule']};duration={s['event_id']:s['authored_utterance_seconds']*time_scale for s in episode['schedule']};specids=set(specs)
 def add(due,kind,data):
  nonlocal serial
  serial+=1;heapq.heappush(pending,(due,serial,kind,data))
 def schedule(spec,due,parent=None,anchor_ns=None):
  eid=spec['event_id']
  if eid in scheduled:return
  scheduled.add(eid);e=copy.deepcopy(spec['event']);parents=list(parent or[]);anchor=anchor_ns if anchor_ns is not None else due
  if e['kind']=='user':
   add(due,'begin',{'event':e,'parents':parents,'anchor':anchor});add(due+int(duration[eid]*1e9),'event',{'event':e,'parents':parents,'anchor':anchor,'begin':due,'wait_class':'USER_WAIT'})
  else:add(due,'event',{'event':e,'parents':parents,'anchor':anchor,'wait_class':'EXTERNAL'})
 def trigger_dependents(anchor,which,at,parent):
  if cancellation_requested_ns is not None:return
  for spec in specs.values():
   if spec['event_id']in scheduled:continue
   trigger=spec['trigger']
   if trigger.get('anchor_event_id')!=anchor:continue
   kind=trigger['kind']
   if (which=='actual_provided'and kind=='after_actual_provided')or(which=='final'and kind=='after_actual_final_or_missing_anchor'):
    due=at+int(trigger['offset_seconds']*time_scale*1e9)
    trigger_audit.append({'event_id':spec['event_id'],'anchor_event_id':anchor,'trigger_kind':kind,'observed_anchor_kind':which,'anchor_ns':at,'scheduled_input_begin_ns':due,'anchor_is_actual_final':which=='final'and final_anchor_choices.get(anchor,{}).get('kind')=='ACTUAL_FINAL','missing_anchor_failure_retained':anchor in fallbackseen})
    schedule(spec,due,[parent]if parent else[],at)
 base=initial_source_provided_ns+int(5*time_scale*1e9)
 for spec in specs.values():
  if spec['trigger']['kind']=='initial':schedule(spec,initial_source_provided_ns+int(spec['trigger']['offset_seconds']*time_scale*1e9),[],base)
 def update_deadline(last_actual_source,reason):
  nonlocal observation_deadline_ns
  deadline=last_actual_source+int(60*time_scale*1e9)
  previous=observation_deadline_ns;at=observation_gate.update(deadline)
  observation_deadline_ns=deadline
  update_id=len(deadline_updates)+1
  observation_gate.pending_update_cutoffs[update_id]=previous
  update={'update_id':update_id,'at_ns':at,'last_actual_source_ns':last_actual_source,'deadline_ns':deadline,'previous_deadline_ns':previous,'reason':reason,'ack_status':'PENDING'}
  deadline_updates.append(update);update['control_send']=send({'type':'observation_deadline',**update})
 def cancel_at_deadline(reason,effective_cutoff=None):
  nonlocal cancellation_requested_ns,settlement_deadline_ns,effective_admission_stop_ns,technical_abort_reason,technical_abort_detection_ns,technical_abort_requested_bound_ns
  if cancellation_requested_ns is not None:return
  technical_abort_detection_ns=time.monotonic_ns();technical_abort_requested_bound_ns=effective_cutoff
  cancellation_requested_ns=observation_gate.cancel();clock_validation_finished.set()
  # Failure-observation clocks are diagnostic only: they did not close admission.
  bounds=[cancellation_requested_ns,*observation_gate.pending_update_cutoffs.values(),observation_deadline_ns]
  effective_admission_stop_ns=min(x for x in bounds if x is not None)
  if reason!='FINAL_ACTUAL_MANDATORY_SOURCE_PLUS_60_SECONDS':technical_abort_reason=reason
  cutoff=effective_admission_stop_ns
  settlement_deadline_ns=cutoff+int(120*max(time_scale,.01)*1e9)
  observation_gate.settlement_deadline_ns=settlement_deadline_ns
  # The Rust input loop treats cancel_work as terminal. A second end_input
  # write races with its normal exit and can turn a complete run into a false
  # technical failure. Keep a failed cancel_work send as a real failure.
  control={'type':'cancel_work','reason':reason,'observation_deadline_ns':cutoff,'settlement_deadline_ns':settlement_deadline_ns}
  try:bracket=send(control)
  except BrokenPipeError:bracket={'send_error':'BROKEN_PIPE','at_ns':time.monotonic_ns()};failures.append({'kind':'CANCELLATION_CONTROL_PIPE_CLOSED','control':control})
  control_sends.append({**control,'requested_ns':cancellation_requested_ns,**bracket})
 try:
  ended=False;ready=False;lastsource=None
  while True:
   now=time.monotonic_ns()
   for update in deadline_updates:
    if not ended and update['ack_status']=='PENDING':
     ack_deadline=min(update['control_send']['send_end_ns']+int(5e9),update['deadline_ns'])
     if update['previous_deadline_ns']is not None:ack_deadline=min(ack_deadline,update['previous_deadline_ns'])
     if now>=ack_deadline:
      update['ack_status']='MISSING_OR_CROSSED_OLD_CUTOFF';failures.append({'kind':'DEADLINE_ACK_MISSING_OR_AFTER_OLD_CUTOFF','at_ns':now,'update':update.copy()});cancel_at_deadline('DEADLINE_ACK_MISSING_OR_AFTER_OLD_CUTOFF',min(now,update['previous_deadline_ns']or now));ended=True
   if not ended and not clock_ready.is_set() and now>=initial_source_provided_ns+int(5e9):
    failures.append({'kind':'HOST_CLOCK_PROOF_NOT_VALID_BEFORE_FIRST_INPUT','at_ns':now});cancel_at_deadline('HOST_CLOCK_PROOF_NOT_VALID_BEFORE_FIRST_INPUT');ended=True
   if observation_deadline_ns is not None and now>=observation_deadline_ns and not ended:
    cancel_at_deadline('FINAL_ACTUAL_MANDATORY_SOURCE_PLUS_60_SECONDS');ended=True
   while pending and pending[0][0]<=now and not ended:
    due,_,kind,data=heapq.heappop(pending);e=data['event'];eid=e['id']
    if kind=='begin':
     try:
      with observation_gate.lock:
       observation_gate.check_locked();send({'type':'input_event','event':{'id':eid,'kind':'input_begin','source_monotonic_ns':due,'source_parent':data['parents']}})
     except ObservationCancelled:cancel_at_deadline('HOST_CLOCK_CUTOFF_BEFORE_INPUT_BEGIN');ended=True
     continue
    # Parent source times are frozen, candidate capture delays do not move the source timestamp.
    try:
     with observation_gate.lock:
      observation_gate.check_locked()
      e['source_monotonic_ns']=due;e['input_begin_monotonic_ns']=data.get('begin');e['source_node']='source:'+eid;e['source_parent']=data['parents'];sent=send({'type':'input_event','event':e})
      provided[eid]={'source_ns':due,'inputbegin_ns':data.get('begin'),'source_node':e['source_node'],'source_parent':data['parents'],'anchor_ns':data['anchor'],'wait_class':data['wait_class'],**sent};source_nodes.append({'id':e['source_node'],'start_ns':data['anchor'],'end_ns':due,'parents':data['parents'],'role':data['wait_class'],'event_id':eid})
      # QA latency roots retain authored source_ns. The observation window is
      # measured from the actual successful pipe flush, never authored due time.
      observation_gate.check_locked()
      if observation_deadline_ns is not None:update_deadline(sent['provided_at_ns'],'LATE_ACTUAL_MANDATORY_SOURCE')
      lastsource=sent['provided_at_ns']
     trigger_dependents(eid,'actual_provided',sent['provided_at_ns'],e['source_node'])
    except ObservationCancelled as error:
     failures.append({'kind':'SOURCE_PROVISION_CONTROL_BOUNDARY_FAILURE','event_id':eid,'at_ns':time.monotonic_ns(),'error':str(error),'provided':provided.get(eid)});cancel_at_deadline('HOST_CLOCK_CUTOFF_BEFORE_ACTUAL_SOURCE');ended=True
   try:m=messages.get(timeout=.01)
   except queue.Empty:m=None
   if m:
    events.append(m);kind=m.get('kind');d=m.get('data',{});at=m.get('observed_ns',now);sid=d.get('stimulus_id');node=d.get('node_id')or(next(reversed(m.get('parent_ids',[])),None))
    if kind=='EPISODE_READY':ready=True;host_origin=at
    if kind=='HOST_CLOCK_PROBE_ACK':
     rust_ns=d.get('host_monotonic_ns');top_ns=m.get('host_monotonic_ns')
     valid=d.get('probe_id')=='initial-clock-proof'and isinstance(rust_ns,int)and isinstance(top_ns,int)and clock_probe_send['send_begin_ns']<=rust_ns<=top_ns<=at
     clock_proof={'status':'VERIFIED'if valid else'FAILED','clock':'Python mach_absolute_time() / Rust mach_absolute_time + actualtimebase','probe_send':clock_probe_send,'rust_probe_ns':rust_ns,'rust_event_ns':top_ns,'host_observed_ns':at,'uncertainty_bracket_ns':at-clock_probe_send['send_begin_ns'],'event_id':m.get('event_id')}
     if valid:clock_ready.set();clock_validation_finished.set()
     else:clock_validation_finished.set();failures.append({'kind':'HOST_CLOCK_DOMAIN_PROOF_FAILED','proof':clock_proof});cancel_at_deadline('HOST_CLOCK_DOMAIN_PROOF_FAILED');ended=True
    if kind=='OBSERVATION_DEADLINE_ACK':
     update=next((u for u in deadline_updates if u['update_id']==d.get('control',{}).get('update_id')),None)
     if update is None:failures.append({'kind':'UNKNOWN_DEADLINE_CONTROL_ACK','at_ns':at,'data':d});cancel_at_deadline('UNKNOWN_DEADLINE_CONTROL_ACK',at);ended=True
     else:
      if update['ack_status']!='ACCEPTED':
       failures.append({'kind':'DEADLINE_CONTROL_REJECTED','at_ns':at,'data':d,'update':update.copy()});cancel_at_deadline('DEADLINE_CONTROL_REJECTED',min(at,update['previous_deadline_ns']or at));ended=True
    if kind=='DISPLAY_PROXY'and sid and sid not in firstseen:firstseen[sid]=at;trigger_dependents(sid,'first',at,node)
    if kind=='RESPONSE_FINAL'and sid and sid not in finalseen:finalseen[sid]=at
    if kind in ['AGENT_INGRESS','CONTROL_INGRESS']:
     record=wire_data(d,'request');ingresses[record['stimulus_id']]={**record,'observed_ns':at}
    if kind=='RECEIVER_RESULT_AVAILABLE':
     receipt=wire_data(d,'receipt');ingress=ingresses.get(receipt['stimulus_id'],{});technical_due=ingress.get('observed_ns',at)+int(.1*time_scale*1e9);receipts[receipt['stimulus_id']]={**receipt,'observed_ns':at,'technical_receipt_profile_ns':int(.1*time_scale*1e9),'receipt_available_ns':max(at,technical_due)}
    if kind in ['AGENT_INGRESS','CONTROL_INGRESS','RECEIVER_RESULT_AVAILABLE']:
     for spec in specs.values():
      if spec['event_id']in scheduled or'ingress'not in spec['original_trigger']:continue
      ref=re.findall(r'S\d\d-U\d\d',spec['original_trigger'])[0];ingress=ingresses.get(ref);receipt=receipts.get(ref)
      if ingress and receipt and receipt.get('accepted')is True:
       actual=actual_external(spec['event'],ingress,receipt);copy_spec={**spec,'event':actual};parents=[n for n in[ingress.get('node_id'),receipt.get('node_id')]if n];due=max(ingress['observed_ns']+int(8*time_scale*1e9),receipt['receipt_available_ns']);schedule(copy_spec,due,parents,ingress['observed_ns'])
    if kind=='PROCESS_STDOUT_CLOSED':
     if cancellation_requested_ns is None:
      failures.append({'kind':'CANDIDATE_EXIT_BEFORE_OBSERVATION_CUTOFF','at_ns':at});cancel_at_deadline('CANDIDATE_EXIT_BEFORE_OBSERVATION_CUTOFF',at)
     break
   # Decide by actual reader receipt clock, not delayed main-loop polling.
   # A timely queued final wins; a truly late final cannot erase missing-at30.
   for eid,source in list(provided.items()):
    if ended or eid in final_anchor_choices:continue
    choice=final_anchors.choose(eid,source['provided_at_ns'],int(30*time_scale*1e9))
    if choice is None:continue
    if choice['kind']=='ACTUAL_FINAL':trigger_dependents(eid,'final',choice['at_ns'],choice['parent_node'])
    else:
     fallbackseen[eid]=choice['at_ns'];failures.append({'event_id':eid,'kind':'RESPONSE_ANCHOR_TIMEOUT','at_ns':choice['at_ns'],'detected_at_ns':time.monotonic_ns(),'late_actual_final_ns':choice.get('late_actual_final_ns')});trigger_dependents(eid,'final',choice['at_ns'],source['source_node']);trigger_dependents(eid,'first',choice['at_ns'],source['source_node'])
   # A missing actual ingress must not fabricate Agent success or deadlock later user input.
   for spec in specs.values():
    eid=spec['event_id'];trigger=spec['original_trigger']
    if ended or spec['event']['kind']!='agent'or eid in provided or eid in finalseen or eid in fallbackseen or'ingress'not in trigger:continue
    refs=re.findall(r'S\d\d-U\d\d',trigger)
    if not refs or refs[0]not in provided:continue
    parent=provided[refs[0]];deadline=parent['source_ns']+int(30*time_scale*1e9)
    if now>=deadline and eid not in scheduled:
     fallbackseen[eid]=deadline;fallback='fallback:'+eid;source_nodes.append({'id':fallback,'start_ns':parent['source_ns'],'end_ns':deadline,'parents':[parent['source_node']],'role':'USER_WAIT','event_id':eid,'missing_anchor':True});failures.append({'event_id':eid,'kind':'MISSING_INGRESS_OR_AGENT_ANCHOR','at_ns':deadline,'source_provided':False});trigger_dependents(eid,'final',deadline,fallback)
   resolved=all(eid in provided or eid in fallbackseen for eid in specids)
   if ready and resolved and observation_deadline_ns is None and not ended:
    try:update_deadline(lastsource,'ALL_MANDATORY_OBLIGATIONS_PROVIDED_OR_RECORDED_MISSING')
    except ObservationCancelled:
     observation_deadline_ns=lastsource+int(60*time_scale*1e9);failures.append({'kind':'OBSERVATION_WINDOW_ARMED_AFTER_EXPIRY','at_ns':time.monotonic_ns()});cancel_at_deadline('OBSERVATION_WINDOW_ARMED_AFTER_EXPIRY');ended=True
   if ended and settlement_deadline_ns and now>=settlement_deadline_ns:
    failures.append({'kind':'POST_SOURCE_TERMINATION_TIMEOUT','at_ns':now});proc.terminate();break
   if not ended and now-started>int(1200*max(time_scale,.02)*1e9):
    failures.append({'kind':'HOST_EPISODE_DEADLINE','at_ns':now});cancel_at_deadline('HOST_EPISODE_DEADLINE');ended=True
  for t in threads:
   remain=max(0,((settlement_deadline_ns or time.monotonic_ns())-time.monotonic_ns())/1e9)
   t.join(timeout=remain)
  if any(t.is_alive()for t in threads):
   failures.append({'kind':'UNSETTLED_MODEL_THREADS_AT_SETTLEMENT_DEADLINE','count':sum(t.is_alive()for t in threads),'at_ns':time.monotonic_ns()})
   if hasattr(client,'unsettled_snapshot'):
    known={c.get('call_uuid')for c in calls}
    calls.extend(c for c in client.unsettled_snapshot()if c.get('call_uuid')not in known)
  try:proc.wait(timeout=10)
  except subprocess.TimeoutExpired:proc.kill();proc.wait(timeout=10);failures.append({'kind':'FORCED_KILL_AFTER_SETTLEMENT','at_ns':time.monotonic_ns()})
 finally:
  stop.set();monitor.join(timeout=5)
  if adapter:adapter.close()
  # Closing the writer after the child has exited is local cleanup, not a
  # second protocol message. TextIOWrapper may flush into the closed pipe.
  try:proc.stdin.close()
  except BrokenPipeError:pass
 stderr=proc.stderr.read();record={'run_id':folder.name,'episode':episode['id'],'configuration':configuration,'rep':rep,'phase':'OFFLINE'if offline else'FORMAL','binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'initial_source_provided_ns':initial_source_provided_ns,'start_ns':started,'end_ns':time.monotonic_ns(),'events':events,'calls':calls,'provided_sources':provided,'source_nodes':source_nodes,'memory':memory,'memory_interval_ms':50,'host_failures':failures,'stderr':stderr,'returncode':proc.returncode,'candidate_input_file':str(folder/'candidate-initial.json'),'voice':'OFF','observation':'COMMON_HOST_TEXT_CALLBACK_PROXY','authored_time_scale':time_scale,'external_profile':{'work_seconds':8,'technical_receipt_seconds':.1,'readiness_rule':'max(ingress + 8s, actual_receiver_ready, ingress + 100ms)'},'actual_API_calls':sum(c.get('actual_api_sent',c.get('evidence_kind')not in ['LOCAL_MODEL','LOCAL_MODEL_FAILED','MEASURED_REFERENCE_HARNESS'])for c in calls)if not offline else 0}
 technical_failures=[f for f in failures if f['kind']not in ['RESPONSE_ANCHOR_TIMEOUT','MISSING_INGRESS_OR_AGENT_ANCHOR']]
 record.update({'population':'COMMON_SUPPLEMENT6','source_scheduler_version':'CONTRACT20_ACTUAL_PROVIDED_TRIGGER_v1','trigger_audit':trigger_audit,'final_anchor_choices':final_anchor_choices,'actual_final_reader_records':final_anchors.snapshot()})
 record.update({'lifecycle_contract':'COMMON9_DEADLINE_V3','initial_source_send':initial_send,'initialization_scope':{'initial_source_provided_ns':initial_source_provided_ns,'episode_ready_observed_ns':host_origin},'clock_probe_send':clock_probe_send,'host_clock_proof':clock_proof,'host_display_receipts':host_display_receipts,'observation_deadline_ns':observation_deadline_ns,'observation_deadline_updates':deadline_updates,'last_actual_mandatory_source_ns':lastsource,'cancellation_requested_ns':cancellation_requested_ns,'effective_admission_stop_ns':effective_admission_stop_ns,'settlement_deadline_ns':settlement_deadline_ns,'technical_abort_reason':technical_abort_reason,'technical_certification':'NOT_CERTIFIED'if technical_abort_reason or technical_failures else'PENDING_INDEPENDENT_EVALUATOR_AUDIT','technical_failures':technical_failures,'control_sends':control_sends,'actual_response_final_ns':finalseen,'anchor_fallback_ns':fallbackseen,'unprovided_mandatory_events':sorted(specids-set(provided))})
 accepted_updates=[u for u in deadline_updates if u['ack_status']=='ACCEPTED']
 record.update({'contract_observation_deadline_ns':observation_deadline_ns,'proposed_observation_deadline_ns':deadline_updates[-1]['deadline_ns']if deadline_updates else None,'last_accepted_observation_deadline_ns':accepted_updates[-1]['deadline_ns']if accepted_updates else None,'technical_abort_requested_ns':cancellation_requested_ns if technical_abort_reason else None,'technical_abort_detection_ns':technical_abort_detection_ns if technical_abort_reason else None,'technical_abort_requested_bound_ns':technical_abort_requested_bound_ns,'settlement_anchor_ns':effective_admission_stop_ns,'pending_deadline_update_cutoffs_at_cancel':observation_gate.pending_update_cutoffs.copy()})
 dump(folder/'trace.json',record)
 with(root/'runs.jsonl').open('a')as f:f.write(json.dumps(record,ensure_ascii=False)+'\n')
 return record
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--episode',required=True);p.add_argument('--configuration',choices=CONFIGS,required=True);p.add_argument('--rep',type=int,default=1);p.add_argument('--root',type=Path,default=REPO/'results/architecture-evaluation/current/common-supplement6-20261019/development');p.add_argument('--offline',action='store_true');p.add_argument('--time-scale',type=float,default=1);p.add_argument('--crate',type=Path,default=CRATE);p.add_argument('--input-transform-module',type=Path);a=p.parse_args();CRATE=a.crate.resolve();transform=None
 if a.input_transform_module:
  if not a.offline:p.error('Input wire transforms are offline QA04 acceptance only')
  import importlib.util
  path=a.input_transform_module.resolve();sys.path.insert(0,str(path.parent));spec=importlib.util.spec_from_file_location('qa04_supplement_input_transform',path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);transform=module.migrate_c03
  dump(a.root/'input-transform-provenance.json',{'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'callable':'migrate_c03','scope':'OFFLINE_QA04_ONLY'})
 ep=next(s for s in read(H/'fixtures.json')if s['id']==a.episode);r=execute(a.root,ep,a.configuration,a.rep,a.offline,a.time_scale,input_transform=transform);print(json.dumps({'run_id':r['run_id'],'calls':len(r['calls']),'sources':len(r['provided_sources']),'failures':r['host_failures'],'returncode':r['returncode'],'technical_failure':bool(r['technical_failures']or r['technical_abort_reason']),'budget_stop':any(c.get('status')in ['BUDGET_BLOCKED_NO_API','PROBE_BUDGET_BLOCKED_NO_API']for c in r['calls'])}))
