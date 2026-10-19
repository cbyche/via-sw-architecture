"""Deterministic executable acceptance. Provider-free; never timing/semantic evidence."""
import argparse,atexit,copy,hashlib,io,json,queue,subprocess,sys,threading,time
from unittest.mock import patch
from pathlib import Path
HERE=Path(__file__).resolve().parent
_bootstrap=argparse.ArgumentParser(add_help=False);_bootstrap.add_argument('--benchmark',type=Path);_boot,_=_bootstrap.parse_known_args()
BENCH=_boot.benchmark or HERE.parent
sys.path.insert(0,str(BENCH))
import harness,offline,bridge
from evaluation_v3.evaluate import regression as regression_check
from patches import migrate_c03
from protocol import Controls

def dump(p,x):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def events(trace,kind):return [e.get('data',{})for e in trace['events']if e.get('kind')==kind]
_ACTIVE=[]

class Session:
 def __init__(self,crate,cfg,initial,migrate=False,audit_usage=False,accounting_root=None):
  self.proc=subprocess.Popen([str(crate/'target/debug/common9_poc')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1);self.q=queue.Queue();self.log=[];self.calls=[];self.hold=None;self.held=[];self.migrate=migrate;self.fail_stage=None;self.audit_usage=audit_usage;self.accounting_root=accounting_root;self.host_client=None;self.write_lock=threading.Lock();self.controls=Controls(self.send);_ACTIVE.append(self);atexit.register(self.close)
  def reader():
   for line in self.proc.stdout:
    try:m=json.loads(line)
    except ValueError:m={'kind':'INVALID_STDOUT','data':{'line':line}}
    m['observed_ns']=time.monotonic_ns()
    self.log.append(m)
    try:self.controls.observe(m)
    except Exception as error:self.controls.failures.append({'reason':'FEATURE_STDOUT_PROTOCOL_ERROR','error':str(error),'at_ns':time.monotonic_ns()})
    self.q.put(m)
  self.reader=threading.Thread(target=reader,daemon=True);self.reader.start();self.initial_send=self.send({'id':'QA04-FEATURE-EXERCISE','configuration':cfg,'initial':initial});self.controls.start()
  if not self.controls.clock_ready.wait(5):
   self.controls.cancel('QA04_CLOCK_ACK_MISSING');raise AssertionError('QA04 clock proof missing')
  self.controls.arm(self.initial_send['provided_at_ns'])
  if not self.controls.deadline_ready.wait(5):
   self.controls.cancel('QA04_DEADLINE_ACK_MISSING');raise AssertionError('QA04 deadline ACK missing')
 def close(self):
  if self.proc.poll()is None:
   try:
    self.controls.cancel('QA04_SESSION_CLOSE');self.controls.cancel_ready.wait(.5)
   except (BrokenPipeError,OSError):pass
   self.proc.terminate()
   try:self.proc.wait(timeout=3)
   except subprocess.TimeoutExpired:self.proc.kill();self.proc.wait(timeout=3)
 def send(self,x):
  if self.migrate:x=migrate_c03(x)
  with self.write_lock:
   before=time.monotonic_ns();self.proc.stdin.write(json.dumps(x,ensure_ascii=False)+'\n');self.proc.stdin.flush();after=time.monotonic_ns()
  return {'send_begin_ns':before,'send_end_ns':after,'provided_at_ns':after}
 def input(self,e):
  self.controls.check();self.send({'type':'input_event','event':e})
 def reply(self,m):
  if self.controls.cancelled_ns is not None:
   response={'call_id':m['call_id'],'error':{'kind':'CANCELLED','reason':'QA04_FEATURE_SETTLEMENT_NO_API','cancelled':True}}
   self.calls.append({'stage':m['stage'],'status':'CANCELLED_NO_API','actual_sent_response':response,'no_provider_request':True})
   self.send(response);return
  self.controls.check()
  admitted_ns=time.monotonic_ns();assert self.controls.clock_proof['host_observed_ns']<=admitted_ns
  payload=m.get('payload',m.get('request',{}).get('context',{}));result=offline.respond(payload,{'stage':m['stage']});raw_usage={'prompt_tokens':17,'completion_tokens':11,'total_tokens':28}
  host_record=None
  if self.audit_usage:
   # Execute the unchanged real bridge accounting code against an in-memory HTTP double.
   # Both credential lookup and transport are intercepted; no keychain or network is used.
   response={'choices':[{'message':{'content':json.dumps(result,ensure_ascii=False)}}],'usage':raw_usage}
   class MockHTTP(io.BytesIO):
    status=200
   with patch.object(bridge,'credential',return_value='QA04_NO_REAL_CREDENTIAL'),patch.object(bridge.urllib.request,'urlopen',side_effect=lambda *a,**k:MockHTTP(json.dumps(response).encode())):
    if self.host_client is None:
     self.host_client=bridge.Client(self.accounting_root,limit=4,ledger_root=self.accounting_root);self.host_client.observation_gate=self.controls
    result,host_record=self.host_client.call(payload,{'phase':'OFFLINE_MOCK_TRANSPORT','stage':m['stage'],'synthetic_usage_not_real_cost':True})
   assert host_record['input_tokens']==17 and host_record['output_tokens']==11
   ledger=json.loads((self.accounting_root/'budget.json').read_text());assert ledger['calls'][host_record['call_uuid']]['state']=='USAGE_RECORDED'
   assert abs(ledger['calls'][host_record['call_uuid']]['amount_usd']-(17*.4+11*1.6)/1e6)<1e-12
   raw_usage=host_record['raw_usage']

  reply={'call_id':m['call_id'],'result':result,'usage':raw_usage}
  if m.get('wire_version')==2:reply={'call_id':m['call_id'],'provider_response':{'json':result,'usage':{'input_tokens':raw_usage['prompt_tokens'],'output_tokens':raw_usage['completion_tokens'],'total_tokens':raw_usage['total_tokens']}}}
  if self.fail_stage==m['stage']:reply={'call_id':m['call_id'],'error':{'kind':'CANCELLED','reason':'QA04_SYNTHETIC_MODEL_CANCEL','cancelled':True},'usage':raw_usage}
  # Accounting belongs to the host and remains independent of the Rust response-body projection.
  host_accounting={'input_tokens':raw_usage['prompt_tokens'],'output_tokens':raw_usage['completion_tokens'],'synthetic_fixture_only':True,'cost_usd':0,'no_provider_request':True}
  self.calls.append({'stage':m['stage'],'wire_version':m.get('wire_version',1),'payload':payload,'result':result,'actual_sent_response':reply,'raw_usage':raw_usage,'host_accounting':host_accounting,'real_bridge_mock_transport_record':host_record,'offline_dispatch_admitted_ns':admitted_ns,'host_clock_proof_observed_ns':self.controls.clock_proof['host_observed_ns'],'observation_deadline_ns':self.controls.deadline_ns})
  self.send(reply)
 def pump(self,predicate,timeout=20):
  end=time.monotonic()+timeout
  while time.monotonic()<end:
   try:m=self.q.get(timeout=.05)
   except queue.Empty:
    if self.proc.poll()is not None:raise AssertionError('Candidate exited before acceptance endpoint: '+self.proc.stderr.read())
    continue
   if m.get('type')=='model_call':
    if self.hold and self.hold(m):self.held.append(m)
    else:self.reply(m)
   if predicate(m):return m
  self.controls.cancel('QA04_FEATURE_ENDPOINT_TIMEOUT')
  raise AssertionError('Feature endpoint timeout')
 def kind(self,k):return self.pump(lambda m:m.get('kind')==k)
 def finish(self):
  self.controls.check();self.controls.cancel('QA04_FINITE_FEATURE_COMPLETE');self.send({'type':'end_input'});self.kind('EPISODE_RESULT');self.proc.wait(timeout=5)
  assert self.proc.returncode==0
  assert self.controls.cancel_ready.is_set(),'Actual candidate cancellation ACK missing'
  assert not self.controls.failures,self.controls.failures
  return {'phase':'OFFLINE_FEATURE_ACCEPTANCE','events':self.log,'calls':self.calls,**self.controls.snapshot(),'returncode':self.proc.returncode,'stderr':self.proc.stderr.read(),'actual_API_calls':0,'synthetic_usage_not_API_cost':True,'scope':'Finite deterministic feature window; not primary QA timing. Common9 separately uses the strict evaluator.','mock_transport_note':'Real bridge bookkeeping send branch is exercised only under in-memory HTTP and credential doubles; no real request is sent.'}

def initial_feature():
 original={'id':'F-QA','revision':1,'text':'left public\nPRIVATE MIDDLE\nright public','extents':{'left':[1,1],'middle':[2,2],'right':[3,3]},'segments':{'all':[1,3]},'local_allowed':True,'external_allowed':True}
 text='## Summary\nOld summary\n## Details\nKeep exact details'
 delivered={'event_id':'actual-shown','stimulus_id':'actual-shown','delivery_source_id':'H-QA','revision':1,'phase':'AGENT_RESULT','text':text,'task_id':'TA','goal_id':'goal:earlier','source_spans':[{'source_id':'F-QA','revision':1,'start_line':1,'end_line':1}],'adopted_geometry_provenance':[]}
 return {'sources':[original,{'id':'H-QA','revision':1,'text':text,'kind':'delivered','provenance':delivered['source_spans']}],'raw_history':[],'historical_delivered':[delivered],'tasks':[{'task_id':'TA','state':'COMPLETED','title':'Keep actual Task','artifact':{'uri':'actual-artifact.txt'},'questions':[],'result_response':{'delivery_event_id':'actual-shown','delivery_source_id':'H-QA','goal_revision':1,'section_revision':1,'text':text,'sections':[{'name':'Summary','text':'Old summary'},{'name':'Details','text':'Keep exact details'}]}},{'task_id':'TB','state':'COMPLETED','title':'Other actual Task','result_response':{'delivery_event_id':'other-shown','text':'Keep other Task exact original'}}]}

def _feature(crate,cfg,task,output):
 initial=initial_feature();s=Session(crate,cfg,initial,task=='C03',audit_usage=task=='C01',accounting_root=output/'synthetic-host-accounting');s.kind('EPISODE_READY');checks=[]
 if task=='C04':
  # Obtain a new actual admitted Agent result through the candidate's own lifecycle.
  s.input({'id':'request-agent-result','kind':'user','text':'F-QA를 바탕으로 발표자료 파일을 만들어줘.'})
  ingress=harness.wire_data(s.kind('AGENT_INGRESS')['data'],'request');receipt=harness.wire_data(s.kind('RECEIVER_RESULT_AVAILABLE')['data'],'receipt');assert receipt['accepted'] is True
  outcome=harness.actual_external({'id':'actual-agent-feature-result','kind':'agent','event_class':'RESULT'},ingress,receipt)
  outcome['text']='## Summary\n'+outcome['text']+'\n## Details\nKeep exact details';outcome['result_source']['text']=outcome['text'];s.input(outcome)
  displayed=s.pump(lambda m:m.get('kind')=='DISPLAY_PROXY'and m['data'].get('stimulus_id')==outcome['id']and m['data'].get('phase')=='AGENT_RESULT')['data']
  s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')==outcome['id'])
  edit={'kind':'qa04_section_update','id':'edit-1','target_event_id':outcome['id'],'task_id':displayed['task_id'],'section':'Summary','text':'New summary\nAdditional summary detail','expected_section_revision':1};s.input(edit);applied=s.kind('QA04_SECTION_UPDATE_APPLIED')['data'];after=applied['after'];assert after['text']=='## Summary\nNew summary\nAdditional summary detail\n## Details\nKeep exact details';assert after['task_id']==displayed['task_id'];before_tasks={t['task_id']:t for t in applied['task_state_before']};after_tasks={t['task_id']:t for t in applied['task_state_after']};target_id=displayed['task_id'];assert {k:v for k,v in before_tasks.items()if k!=target_id}=={k:v for k,v in after_tasks.items()if k!=target_id};ta=copy.deepcopy(after_tasks[target_id]);ta.pop('result_response');original=copy.deepcopy(before_tasks[target_id]);original.pop('result_response');assert ta==original;assert after_tasks[target_id]['result_response']['text']==after['text'];assert after_tasks[target_id]['result_response']['sections'][1]==before_tasks[target_id]['result_response']['sections'][1];assert applied['request_state_before']==applied['request_state_after'];s.kind('QA04_FEATURE_RESPONSE_FINAL');checks+=['actual_named_section_callback','unaffected_details_exact','target_Task_result_section_updated_other_sections_and_Tasks_preserved']
  for label,event in [('stale',edit),('wrong-task',{**edit,'id':'edit-2','expected_section_revision':2,'task_id':'TB'}),('duplicate',{**edit,'id':'edit-3','expected_section_revision':2}),('unshown',{**edit,'id':'edit-4','target_event_id':'unshown','expected_section_revision':2})]:
   s.input(event);s.kind('QA04_SECTION_UPDATE_REJECTED');s.kind('QA04_FEATURE_RESPONSE_FINAL');checks.append(label+'_rejected')
  # A subsequent real input reads registered revised displayed text through the normal evidence lifecycle.
  s.input({'id':'read-updated','kind':'user','text':displayed['delivery_source_id']+'의 내용을 설명해줘.'});s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')=='read-updated');checks.append('subsequent_reference_consumes_revised_source')
 elif task=='C05':
  for target,status in [('actual-shown','DELIVERED'),('unshown-proposal','NO_MATCH'),('rejected','NO_MATCH'),('failed','NO_MATCH')]:
   s.input({'id':'query-'+target,'kind':'qa04_provenance_query','target_event_id':target});readonly=s.kind('QA04_PROVENANCE_READONLY_STATE')['data'];assert readonly['before']==readonly['after'];assert readonly['sources_before']==readonly['sources_after'];r=s.kind('QA04_PROVENANCE_PANEL_DISPLAY')['data'];assert r['status']==status;s.kind('QA04_FEATURE_RESPONSE_FINAL')
   if status=='DELIVERED':assert r['source_spans'][0]['source_id']=='F-QA'and r['source_spans'][0]['current_use_status']=='CURRENT_AUTHORIZED'
  s.input({'id':'live-provenance-result','kind':'user','text':'F-QA의 내용을 설명해줘.'});s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')=='live-provenance-result')
  s.input({'id':'query-live','kind':'qa04_provenance_query','target_event_id':'live-provenance-result'});readonly=s.kind('QA04_PROVENANCE_READONLY_STATE')['data'];assert readonly['before']==readonly['after']and readonly['sources_before']==readonly['sources_after'];live=s.kind('QA04_PROVENANCE_PANEL_DISPLAY')['data'];assert live['status']=='DELIVERED'and any(p['source_id']=='F-QA'for p in live['source_spans']);s.kind('QA04_FEATURE_RESPONSE_FINAL')
  s.fail_stage='meaning_interpret_and_respond';s.input({'id':'actually-failed-proposal','kind':'user','text':'F-QA의 내용을 다시 설명해줘.'});s.pump(lambda m:m.get('kind')=='WORKFLOW_FAILED'and m['data'].get('event_id')=='actually-failed-proposal');s.fail_stage=None
  s.input({'id':'query-actual-failure','kind':'qa04_provenance_query','target_event_id':'actually-failed-proposal'});readonly=s.kind('QA04_PROVENANCE_READONLY_STATE')['data'];assert readonly['before']==readonly['after']and readonly['sources_before']==readonly['sources_after'];assert s.kind('QA04_PROVENANCE_PANEL_DISPLAY')['data']['status']=='NO_MATCH';s.kind('QA04_FEATURE_RESPONSE_FINAL');checks+=['live_publication_provenance','actual_failed_workflow_not_delivered','whole_state_and_source_catalog_readonly']
  source=copy.deepcopy(initial['sources'][0]);source['revision']=2;source['text']='updated left\nPRIVATE MIDDLE\nright public';s.input({'kind':'source','source':source});s.input({'id':'query-old','kind':'qa04_provenance_query','target_event_id':'actual-shown'});readonly=s.kind('QA04_PROVENANCE_READONLY_STATE')['data'];assert readonly['before']==readonly['after'];assert readonly['sources_before']==readonly['sources_after'];r=s.kind('QA04_PROVENANCE_PANEL_DISPLAY')['data'];assert r['source_spans'][0]['current_use_status']=='HISTORICAL_AUTHORIZED';s.kind('QA04_FEATURE_RESPONSE_FINAL');checks+=['actual_delivery_only','not_unshown_rejected_or_failed','historical_label','query_readonly']
 elif task=='C06':
  # Hold a model answer that consumed all3lines, apply real authority withdrawal, then release old draft.
  s.hold=lambda m:m['stage']=='meaning_interpret_and_respond';s.input({'id':'pending-draft','kind':'user','text':'F-QA의 내용을 설명해줘.'});s.pump(lambda m:m.get('type')=='model_call'and m['stage']=='meaning_interpret_and_respond');assert s.held
  s.input({'id':'withdraw','kind':'qa04_withdraw_range','source_id':'F-QA','revision':1,'start_line':2,'end_line':2,'access_epoch':2});s.kind('QA04_RANGE_WITHDRAWAL_APPLIED');s.kind('QA04_FEATURE_RESPONSE_FINAL');s.hold=None
  for held in s.held:s.reply(held)
  s.pump(lambda m:m.get('kind')=='WORKFLOW_FAILED'and m['data'].get('event_id')=='pending-draft');assert not any(m.get('kind')=='DISPLAY_PROXY'and m.get('data',{}).get('stimulus_id')=='pending-draft'for m in s.log)
  s.input({'id':'new-after-withdraw','kind':'user','text':'F-QA의 현재 허용한 내용을 설명해줘.'});s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')=='new-after-withdraw');calls=[c for c in s.calls if c['stage']=='meaning_interpret_and_respond'and c['payload']['input']['event']['id']=='new-after-withdraw'];raw=calls[-1]['payload']['input']['actually_supplied_originals'];body=' '.join(p['text']for p in raw if p['source_id']=='F-QA');assert 'left public'in body and'right public'in body and'PRIVATE MIDDLE'not in body;assert 'PRIVATE MIDDLE'not in json.dumps(calls[-1]['payload'],ensure_ascii=False),'Revoked text leaked through non-evidence public-state path';checks+=['old_pending_draft_blocked','new_input_no_withdrawn_text','disjoint_raw_fragments_retained']
 else:
  s.input({'id':'format-check','kind':'user','text':'F-QA의 내용을 설명해줘.'});s.pump(lambda m:m.get('kind')=='RESPONSE_FINAL'and m['data'].get('stimulus_id')=='format-check');checks.append('actual_normal_workflow_after_format_migration')
  if task=='C01':
   assert all(c['wire_version']==2 for c in s.calls)
   adoption=[m['data']['plan']for m in s.log if m.get('kind')=='SEMANTIC_ADOPTION'and m['data'].get('stimulus_id')=='format-check'][-1];assert adoption['_provider_usage']=={'input_tokens':17,'output_tokens':11}
   assert all(c['host_accounting']['input_tokens']==17 and c['host_accounting']['output_tokens']==11 for c in s.calls)
   assert sum(c['host_accounting']['input_tokens']for c in s.calls)>0
   # An actual second call fails through the new wire: preserve structured cancellation/error and charge-known fixture usage.
   s.fail_stage='meaning_interpret_and_respond';s.input({'id':'cancelled-model-call','kind':'user','text':'F-QA의 내용을 다시 설명해줘.'});failure=s.pump(lambda m:m.get('kind')=='WORKFLOW_FAILED'and m['data'].get('event_id')=='cancelled-model-call')['data'];assert 'QA04_SYNTHETIC_MODEL_CANCEL'in failure['reason'];assert 'CANCELLED'in failure['reason'];assert not any(m.get('kind')=='DISPLAY_PROXY'and m.get('data',{}).get('stimulus_id')=='cancelled-model-call'for m in s.log)
   checks+=['actual_model_wire_v2','nonzero_usage_host_and_decoded_plan_preserved','actual_cancellation_error_transport_preserved_no_success_callback']
 trace=s.finish();result=events(trace,'EPISODE_RESULT')[-1]
 if task=='C04':
  calls=[c for c in trace['calls']if c['stage']=='meaning_interpret_and_respond'and c['payload']['input']['event']['id']=='read-updated'];raw=calls[-1]['payload']['input']['actually_supplied_originals'];assert any(p['source_id']==displayed['delivery_source_id']and'New summary'in p['text']and'Keep exact details'in p['text']for p in raw)
 dump(output/'feature-trace.json',trace);return checks

def feature(crate,cfg,task,output):
 try:return _feature(crate,cfg,task,output)
 except Exception as error:
  session=_ACTIVE[-1]if _ACTIVE else None
  if session:
   session.close();dump(output/'feature-failed-trace.json',{'error':type(error).__name__+': '+str(error),'events':session.log,'calls':session.calls,**session.controls.snapshot(),'stderr':session.proc.stderr.read(),'returncode':session.proc.returncode,'provider_calls':0})
  raise
 finally:
  for session in _ACTIVE:session.close()

def regression(crate,cfg,task,output):
 harness.CRATE=crate;fixtures=json.loads((BENCH/'fixtures.json').read_text());oracles=json.loads((BENCH/'oracles.json').read_text());rows=[]
 for episode in fixtures:
  e=copy.deepcopy(episode)
  trace=harness.execute(output/'regression',e,cfg['id'],1,offline=True,time_scale=.05,input_transform=migrate_c03 if task=='C03'else None)
  # The candidate receives only actual sources/stream events. Oracles are host-only checks.
  checks=regression_check(trace,oracles[episode['id']])
  assert checks['evaluation']['version']=='COMMON9-EVALUATOR-v3'
  assert checks['evaluation']['evaluation_mode']=='strict'
  assert checks['evaluation']['primary_measurement_eligibility']=='ELIGIBLE'
  assert trace['actual_API_calls']==0
  assert checks['status']=='PASS',checks
  if task=='C02':
   wire_events=[m['data']for m in trace['events']if m.get('kind')in ['AGENT_INGRESS','CONTROL_INGRESS','RECEIVER_RESULT_AVAILABLE']]
   assert all(w.get('wire_version')==2 and 'envelope'in w for w in wire_events),'Actual Agent wire did not migrate'
  rows.append({'episode':episode['id'],'run_id':trace['run_id'],'checks':checks,'provider_calls':trace['actual_API_calls']})
 totals={k:sum(r['checks']['counts'][k]for r in rows)for k in ['responses_planned','responses_observed','goals_planned','goals_observed','bindings_planned','bindings_correct']}
 assert totals=={'responses_planned':43,'responses_observed':43,'goals_planned':23,'goals_observed':23,'bindings_planned':15,'bindings_correct':15},totals
 dump(output/'regression-checks.json',{'rows':rows,'totals':totals});return rows

def main():
 p=argparse.ArgumentParser();p.add_argument('--benchmark',type=Path,required=True);p.add_argument('--crate',type=Path,required=True);p.add_argument('--configuration',type=Path,required=True);p.add_argument('--task',required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--freeze',type=Path,required=True);p.add_argument('--digest',required=True);a=p.parse_args()
 assert hashlib.sha256(a.freeze.read_bytes()).hexdigest()==a.digest,'Exact QA04 baseline freeze mismatch'
 frozen=json.loads(a.freeze.read_text());bench_relative=Path('benchmark/architecture/common9_poc_20261019');expected_bench=(a.freeze.parent/'frozen/source'/bench_relative).resolve()
 assert BENCH.resolve()==expected_bench,'QA04 must import benchmark and strict evaluator from the exact immutable snapshot'
 assert Path(harness.__file__).resolve()==expected_bench/'harness.py'
 assert Path(sys.modules[regression_check.__module__].__file__).resolve()==expected_bench/'evaluation_v3/evaluate.py'
 for entry in frozen['sources']:
  file=Path(entry['frozen_path']);assert file.is_file()and hashlib.sha256(file.read_bytes()).hexdigest()==entry['sha256'],'Frozen dependency differs: '+entry['relativePath']
 cfg=json.loads(a.configuration.read_text());a.output.mkdir(parents=True,exist_ok=False)
 checks=feature(a.crate,cfg,a.task,a.output);rows=regression(a.crate,cfg,a.task,a.output)
 dump(a.output/'summary.json',{'task_acceptance':'PASS','common9_regression':'PASS','provider_calls':0,'baseline_freeze_sha256':a.digest,'evaluator_version':'COMMON9-EVALUATOR-v3','evaluation_mode':'strict','feature_checks':checks,'common9_rows':rows,'scheduled_common_counts':{'responses':43,'goals':23,'bindings':15},'scope':'Actual candidate with deterministic semantic/vision doubles; no model accuracy/timing claim; regression common9+finite feature checks only'})
if __name__=='__main__':main()
