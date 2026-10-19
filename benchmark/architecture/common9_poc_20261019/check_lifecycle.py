"""Finite offline admission/transport checks. All HTTP calls are mocked."""
import io,json,threading,time,urllib.request
from pathlib import Path
from unittest.mock import patch
from bridge import Client,ModelError,dump
from lifecycle import ObservationGate,ObservationCancelled
from memory import LogicalBuffers
ROOT=Path(__file__).resolve().parents[3]/'results/architecture-evaluation/current/common9-poc-20261019/development'/('lifecycle-guards-'+str(time.monotonic_ns()))
ROOT.mkdir(parents=True,exist_ok=False)

def client(name):
 c=Client.__new__(Client);c.root=ROOT/name;c.root.mkdir();c.ledger_root=c.root;c.limit=40;c.key='OFFLINE_FAKE_NOT_A_CREDENTIAL';c.url='https://invalid.example.invalid';c.lock=threading.Lock();c.resource_lock=threading.Lock();c.buffers=LogicalBuffers();c.slot_owners={};c.last_slot_owner={};c.configure_slots(2);c.observation_gate=ObservationGate();return c
def call(c,records,role='VIA'):
 try:
  _,record=c.call({'input':{'scope':'offline deterministic admission test'},'role':role,'max_tokens':2},{'stage':'OFFLINE_GUARD','max_calls_per_run':100});records.append(record)
 except ModelError as error:records.append(error.record)
def response():
 class R(io.BytesIO):
  status=200
  def __enter__(self):return self
  def __exit__(self,*args):self.close()
 return R(json.dumps({'usage':{'prompt_tokens':10,'completion_tokens':2},'choices':[{'message':{'content':'{"offline":true}'}}]}).encode())
def pure_cutoff():
 now=[10];g=ObservationGate(lambda:now[0]);g.update(20);now[0]=19;g.update(30);now[0]=30
 try:g.check();raise AssertionError('cutoff admitted')
 except ObservationCancelled:pass
 try:g.update(60);raise AssertionError('expired window revived')
 except ObservationCancelled:pass
 assert g.cancel()==30;now[0]=31;assert g.cancel()==30
 return {'name':'host_exact_cutoff_and_no_revival','passed':True}
def queued_cancel():
 c=client('queued');records=[];entered=threading.Event();release=threading.Event();mu=threading.Lock();state={'live':0,'max':0,'total':0}
 def fake(*args,**kwargs):
  with mu:
   state['live']+=1;state['total']+=1;state['max']=max(state['max'],state['live'])
   if state['total']==2:entered.set()
  assert release.wait(5),'mock synchronization failed'
  with mu:state['live']-=1
  return response()
 with patch.object(urllib.request,'urlopen',fake):
  active=[threading.Thread(target=call,args=(c,records,role))for role in ['VIA','RECEIVER']]
  for t in active:t.start()
  assert entered.wait(5),'K2 was serialized by gate'
  queued=threading.Thread(target=call,args=(c,records,'VIA'));queued.start()
  limit=time.monotonic()+5
  while time.monotonic()<limit:
   p=c.root/'budget.json'
   if p.exists()and len(json.loads(p.read_text())['calls'])==3:break
   threading.Event().wait(.001)
  else:raise AssertionError('third call never reserved/queued')
  cancelled=c.observation_gate.cancel();release.set()
  for t in active+[queued]:t.join(5);assert not t.is_alive()
 assert len(records)==3 and state['total']==2 and state['max']==2
 blocked=[r for r in records if not r['actual_api_sent']];assert len(blocked)==1 and blocked[0]['status']=='CANCELLED_NO_API'and blocked[0]['cost_usd']==0
 assert all(r['http_dispatch_registered_ns']<cancelled for r in records if r['actual_api_sent'])
 assert all(r.get('cost_usd')is not None for r in records)and not c.slot_owners
 ledger=json.loads((c.root/'budget.json').read_text());assert ledger['calls'][blocked[0]['call_uuid']]['state']=='CANCELLED_UNSENT_RESERVATION_RELEASED'
 dump(ROOT/'queued-cancel.json',{'passed':True,'transport':'MOCK_ONLY_NO_NETWORK','records':records,'ledger':ledger,'max_concurrent_mock_services':state['max'],'cancel_ns':cancelled})
 return {'name':'K2_retained_queued_cancel_no_dispatch_known_zero_release','passed':True}
def cancel_during_preparation():
 c=client('preparation');image=c.root/'synthetic.png';image.write_bytes(b'OFFLINE_NOT_AN_IMAGE');records=[]
 import bridge
 real=bridge.base64.b64encode
 def encode(data):c.observation_gate.cancel();return real(data)
 with patch.object(bridge.base64,'b64encode',encode),patch.object(urllib.request,'urlopen',side_effect=AssertionError('network forbidden')):
  try:c.call({'input':{},'images':[str(image)]},{'stage':'OFFLINE_PREP'})
  except ModelError as error:records.append(error.record)
 assert len(records)==1 and records[0]['status']=='CANCELLED_NO_API'and not(c.root/'budget.json').exists()
 dump(ROOT/'prepare-cancel.json',{'passed':True,'records':records,'reservation_created':False})
 return {'name':'cancel_during_encoding_blocks_before_reservation','passed':True}
def cancel_after_reservation():
 c=client('reservation');records=[];original=c.budget
 def budget(id,amount,state):
  original(id,amount,state)
  if state=='RESERVED':c.observation_gate.cancel()
 c.budget=budget
 with patch.object(urllib.request,'urlopen',side_effect=AssertionError('network forbidden')):call(c,records)
 assert len(records)==1 and records[0]['status']=='CANCELLED_NO_API'and not records[0]['actual_api_sent']and records[0]['cost_usd']==0
 ledger=json.loads((c.root/'budget.json').read_text());row=next(iter(ledger['calls'].values()));assert row['amount_usd']==0 and row['state']=='CANCELLED_UNSENT_RESERVATION_RELEASED'
 dump(ROOT/'reservation-cancel.json',{'passed':True,'records':records,'ledger':ledger})
 return {'name':'cancel_after_reservation_blocks_queue_and_releases_unsent','passed':True}
def failed_transport_stays_unknown():
 c=client('unknown');records=[]
 with patch.object(urllib.request,'urlopen',side_effect=OSError('OFFLINE mocked transport failure')):call(c,records)
 with patch.object(urllib.request,'urlopen',side_effect=lambda *a,**k:response()):call(c,records)
 assert len(records)==2 and records[0]['cost_usd']is None and records[0]['actual_api_sent']and records[1]['status']=='COMPLETED'and not c.slot_owners
 ledger=json.loads((c.root/'budget.json').read_text());assert ledger['calls'][records[0]['call_uuid']]['state']=='UNKNOWN'
 dump(ROOT/'unknown-preserved.json',{'passed':True,'records':records,'ledger':ledger})
 return {'name':'sent_transport_failure_unknown_and_next_slot_usable','passed':True}

def local_parser_admission():
 import grounding_adapter,hashlib
 image=ROOT/'local-synthetic.png';image.write_bytes(b'OFFLINE_HASH_INPUT_NOT_AN_IMAGE')
 frame={'frame_id':'observed-frame','source_revision':1,'sha256':hashlib.sha256(image.read_bytes()).hexdigest(),'image_path':str(image)};payload={'input':{'frames':[frame]}}
 cases=[]
 # A call waiting for the adapter lock cannot spawn after cancellation.
 gate=ObservationGate();adapter=grounding_adapter.GroundingAdapter(ROOT/'local-queued',LogicalBuffers(),gate);records=[]
 adapter.lock.acquire()
 with patch.object(grounding_adapter.subprocess,'Popen',side_effect=AssertionError('spawn forbidden')):
  worker=threading.Thread(target=lambda:records.append(adapter.call(payload,{'stage':'LOCAL_MOCK'})));worker.start();gate.cancel();adapter.lock.release();worker.join(5);assert not worker.is_alive()
 assert records[0][1]['status']=='LOCAL_MODEL_CANCELLED'and records[0][1]['local_spawn_admitted_ns']is None
 cases.append({'name':'queued_after_cancel','record':records[0][1]})
 class Process:
  pid=987654
  def __init__(self):self.stdin=io.StringIO();self.stdout=io.StringIO()
  def wait(self,timeout=None):return 0
  def kill(self):pass
 for mode in ['cold_ready_after_cutoff','parse_return_after_cutoff']:
  now=[10];gate=ObservationGate(lambda:now[0]);gate.update(20);adapter=grounding_adapter.GroundingAdapter(ROOT/('local-'+mode),LogicalBuffers(),gate);count=[0]
  def line(timeout):
   count[0]+=1
   if count[0]==1:
    if mode=='cold_ready_after_cutoff':now[0]=20
    return {'ready':True,'initialization_ns':100}
   now[0]=20
   return {'frames':[{'frame_id':frame['frame_id'],'source_revision':1,'source_sha256':frame['sha256'],'parse_ns':123,'items':[]}]}
  adapter._line=line
  with patch.object(grounding_adapter.subprocess,'Popen',side_effect=lambda *a,**k:Process()):result,record=adapter.call(payload,{'stage':'LOCAL_MOCK'})
  assert record['status']=='LOCAL_MODEL_CANCELLED'and record['local_spawn_admitted_ns']==10 and not adapter.cache
  if mode=='cold_ready_after_cutoff':assert record['local_parse_dispatch_admitted_ns']is None and count[0]==1
  else:assert record['local_parse_dispatch_admitted_ns']==10 and record['inference_ns']==123 and record['actual_parser_raw_response']['frames']
  cases.append({'name':mode,'record':record})
 dump(ROOT/'local-parser-admission.json',{'passed':True,'scope':'Mock Popen/service/clock; no learned helper loaded, inference or paid API','cases':cases})
 return {'name':'local_queued_cold_ready_parse_return_cutoff_guards','passed':True}

def scoped_probe_budget():
 c=client('scope-cap');c.scope_limit_usd=.001;records=[]
 seed={'limit_usd':40,'calls':{'prior-global':{'amount_usd':.6635124,'state':'USAGE_RECORDED','scope_root':'OTHER_GENERATION'},'probe-unknown':{'amount_usd':.0009,'state':'UNKNOWN','scope_root':str(c.root.resolve())}}};dump(c.root/'budget.json',seed)
 with patch.object(urllib.request,'urlopen',side_effect=AssertionError('network forbidden')):call(c,records)
 assert records[0]['status']=='PROBE_BUDGET_BLOCKED_NO_API'and not records[0]['actual_api_sent']and json.loads((c.root/'budget.json').read_text())==seed
 dump(ROOT/'scoped-probe-budget.json',{'passed':True,'records':records,'unchanged_prior_and_unknown_ledger':seed,'actual_paid_API_calls':0})
 return {'name':'probe_subcap_counts_unknown_keeps_prior_global_and_blocks_before_API','passed':True}

if __name__=='__main__':
 checks=[pure_cutoff(),queued_cancel(),cancel_during_preparation(),cancel_after_reservation(),failed_transport_stays_unknown(),local_parser_admission(),scoped_probe_budget()]
 dump(ROOT/'summary.json',{'status':'PASS','checks':checks,'actual_paid_API_calls':0,'scope':'Mock HTTP/admission tests; no performance or paid provider success claim'})
 print(json.dumps({'root':str(ROOT),'checks':checks,'actual_paid_API_calls':0}))
