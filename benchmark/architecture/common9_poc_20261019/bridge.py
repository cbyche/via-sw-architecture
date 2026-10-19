"""Qwen JSON/vision bridge; shared atomic authorized-cap ledger with all prior generations, no auth in evidence."""
import base64, fcntl, json, os, subprocess, threading, time, urllib.request, urllib.error, uuid
from pathlib import Path
from memory import LogicalBuffers
from lifecycle import ObservationGate,ObservationCancelled
MODEL='qwen3.7-plus-2026-05-26'
PRICE={'input_per_million':.40,'output_per_million':1.60,'currency':'USD','region':'Singapore/International','input_bracket':'<=256K','source':'https://www.alibabacloud.com/help/en/model-studio/model-pricing','date':'2026-10-18'}
def dump(path,value):
    path.parent.mkdir(parents=True,exist_ok=True); path.write_text(json.dumps(value,ensure_ascii=False,indent=2))
def credential(service):
    p=subprocess.run(['/usr/bin/security','find-generic-password','-s',service,'-w'],capture_output=True,timeout=15)
    if p.returncode: raise RuntimeError('Credential unavailable: '+service+' (secret not printed)')
    return p.stdout.decode().strip()
class Client:
    def __init__(self,root,limit=4,ledger_root=None):
        self.root=Path(root); self.root.mkdir(parents=True,exist_ok=True); self.limit=limit; self.ledger_root=Path(ledger_root or root);self.ledger_root.mkdir(parents=True,exist_ok=True)
        self.key=credential('via-dashscope-api-key'); workspace=credential('via-dashscope-workspace-id')
        self.url=f'https://{workspace}.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions'
        self.semaphore=threading.Semaphore(2); self.lock=threading.Lock(); self.buffers=LogicalBuffers(); self.resource_lock=threading.Lock(); self.slot_owners={}; self.last_slot_owner={}; self.k=2
        self.observation_gate=ObservationGate()
        self.scope_limit_usd=None
        self.active_calls={}
    def configure_slots(self,k):
        self.k=k;self.semaphore=threading.Semaphore(k)
    def budget(self,id,amount,state):
        path=self.ledger_root/'budget.json'
        with path.open('a+') as f:
            fcntl.flock(f,fcntl.LOCK_EX);f.seek(0); old=f.read(); data=json.loads(old) if old else {'limit_usd':self.limit,'calls':{}}
            if data['limit_usd']!=self.limit:raise RuntimeError('Global ledger limit mismatch')
            total=sum(x['amount_usd'] for x in data['calls'].values())
            if state=='RESERVED' and total+amount>data['limit_usd']: raise RuntimeError('GLOBAL CAMPAIGN BUDGET STOP')
            scope=str(self.root.resolve());scope_limit=getattr(self,'scope_limit_usd',None)
            scoped_total=sum(x['amount_usd']for x in data['calls'].values()if x.get('scope_root')==scope)
            if state=='RESERVED'and scope_limit is not None and scoped_total+amount>scope_limit:raise RuntimeError('PROBE SCOPED BUDGET STOP')
            data['calls'][id]={'amount_usd':amount,'state':state,'scope_root':scope};f.seek(0);f.truncate();json.dump(data,f,indent=2);f.flush();os.fsync(f.fileno())
    def blocked(self,payload,meta,status,reason):
        now=time.monotonic_ns();record={**meta,'call_uuid':str(uuid.uuid4()),'status':status,'error':reason,'causal_trace':payload.get('_trace',{}),'enqueue_ns':now,'terminal_ns':now,'actual_api_sent':False,'cost_usd':0,'input_tokens':0,'output_tokens':0,'cost_state':'NO_REQUEST_SENT_NO_API_CHARGE','evidence_kind':'ADMINISTRATIVE_OR_CALL_CAP_BLOCK','role':payload.get('role','VIA')}
        with self.lock:
            with(self.root/'model-calls.jsonl').open('a')as f:f.write(json.dumps(record,ensure_ascii=False)+'\n')
        raise ModelError(record)
    def unsettled_snapshot(self):
        records=[]
        with self.observation_gate.lock,self.lock:
            for ident,value in getattr(self,'active_calls',{}).items():
                row=dict(value);row['terminal_ns']=time.monotonic_ns();row['unfinished_at_settlement_deadline']=True
                if not row.get('actual_api_sent'):
                    row.update(status='CANCELLED_UNSENT_AT_SETTLEMENT',cost_usd=0,input_tokens=0,output_tokens=0,cost_state='NO_REQUEST_SENT_NO_API_CHARGE')
                    self.budget(ident,0,'CANCELLED_UNSENT_RESERVATION_RELEASED')
                elif not isinstance(row.get('cost_usd'),(int,float)):
                    row.update(status='UNSETTLED_AT_SETTLEMENT_DEADLINE',cost_usd=None,cost_state='UNKNOWN_RESERVATION_RETAINED')
                    self.budget(ident,row['reservation_usd'],'UNKNOWN')
                else:row['status']='RESULT_PROCESSING_UNFINISHED_USAGE_KNOWN'
                dump(self.root/'raw/model'/(ident+'-settlement-snapshot.json'),row);records.append(row)
        return records
    def call(self,payload,meta):
        try:self.observation_gate.check()
        except ObservationCancelled as error:self.blocked(payload,meta,'CANCELLED_NO_API',str(error))
        with self.lock:
            count=getattr(self,'run_call_count',0)+1;self.run_call_count=count
        if count>meta.get('max_calls_per_run',100):self.blocked(payload,meta,'CALL_CAP_BLOCKED_NO_API','FROZEN PER-RUN MODEL CALL CAP EXHAUSTED')
        call_id=str(uuid.uuid4()); start=time.monotonic_ns(); text=json.dumps(payload.get('input',{}),ensure_ascii=False)
        instruction=payload.get('instruction','Return a JSON object.'); max_tokens=int(payload.get('max_tokens',1800))
        content=[{'type':'text','text':text}]; image_bytes=0
        for filename in payload.get('images',[]):
            path=Path(filename); data=path.read_bytes();image_bytes+=len(data); mime='image/jpeg' if path.suffix.lower() in ('.jpg','.jpeg') else 'image/png'
            content.append({'type':'image_url','image_url':{'url':f'data:{mime};base64,'+base64.b64encode(data).decode()}})
        body={'model':MODEL,'messages':[{'role':'system','content':instruction+' Return valid JSON only.'},{'role':'user','content':content if len(content)>1 else text}], 'enable_thinking':False,'temperature':0,'max_tokens':max_tokens,'response_format':{'type':'json_object'}}
        # UTF-8 text bytes + entire image bytes is conservative for these small fixtures.
        reserve=((len(text.encode())+len(instruction.encode())+image_bytes+4096)*.40+max_tokens*1.60)/1e6
        try:
            with self.observation_gate.lock:
                reservation_admitted_ns=self.observation_gate.check_locked()
                self.budget(call_id,reserve,'RESERVED')
        except ObservationCancelled as error:self.blocked(payload,meta,'CANCELLED_NO_API',str(error))
        except RuntimeError as error:
            if str(error)=='GLOBAL CAMPAIGN BUDGET STOP':self.blocked(payload,meta,'BUDGET_BLOCKED_NO_API',str(error))
            if str(error)=='PROBE SCOPED BUDGET STOP':self.blocked(payload,meta,'PROBE_BUDGET_BLOCKED_NO_API',str(error))
            raise
        raw=self.root/'raw'/'model';raw.mkdir(parents=True,exist_ok=True)
        dump(raw/(call_id+'-request.json'),body)
        self.buffers.set(call_id+':input_text',len(text.encode()),'model_input')
        self.buffers.set(call_id+':image_binary',image_bytes,'image')
        encoded_body=json.dumps(body).encode()
        self.buffers.set(call_id+':http_request',len(encoded_body),'http_serialization_including_base64')
        record={**meta,'causal_trace':payload.get('_trace',{}),'call_uuid':call_id,'enqueue_ns':start,'model':MODEL,'thinking':False,'actual_api_sent':False,'price':PRICE,'role':payload.get('role','VIA'),'reservation_usd':reserve,'reservation_admitted_ns':reservation_admitted_ns,'scope_root':str(self.root.resolve()),'scope_limit_usd':getattr(self,'scope_limit_usd',None),'global_ledger_root':str(self.ledger_root),'request_file':str(raw/(call_id+'-request.json'))}
        with self.lock:
            if not hasattr(self,'active_calls'):self.active_calls={}
            self.active_calls[call_id]=record
        sent=None; response=None; status=None
        try:
            record['queue_admitted_ns']=self.observation_gate.check()
            record["before_slot_wait_ns"]=time.monotonic_ns()
            with self.semaphore:
                record["slot_acquired_ns"]=time.monotonic_ns()
                with self.resource_lock:
                    slot=next(i for i in range(self.k) if i not in self.slot_owners)
                    record['resource_slot']=slot;record['resource_predecessor_call_uuid']=self.last_slot_owner.get(slot)
                    self.slot_owners[slot]=call_id
                try:
                    request=urllib.request.Request(self.url,data=encoded_body,headers={'Authorization':'Bearer '+self.key,'Content-Type':'application/json'})
                    try:
                        with self.observation_gate.lock:
                            sent=self.observation_gate.check_locked();record['send_ns']=sent
                            record['http_dispatch_registered_ns']=sent
                            record['http_dispatch_timestamp_semantics']='Host admission/call-initiation linearization immediately before urlopen; not a measured socket-wire send'
                            record['actual_api_sent']=True
                        with urllib.request.urlopen(request,timeout=120) as r: response=json.load(r);status=r.status
                    except urllib.error.HTTPError as e: status=e.code;response={'http_error':status,'body':e.read().decode(errors='replace')}
                    record['terminal_ns']=time.monotonic_ns()
                finally:
                    with self.resource_lock:
                        self.slot_owners.pop(slot,None);self.last_slot_owner[slot]=call_id
            self.buffers.set(call_id+':provider_response',len(json.dumps(response,ensure_ascii=False).encode()),'http_response')
            dump(raw/(call_id+'-response.json'),response)
            usage=response.get('usage'); record['http_status']=status;record['raw_usage']=usage
            record['response_file']=str(raw/(call_id+'-response.json'))
            if usage:
                record['input_tokens']=usage.get('prompt_tokens',0);record['output_tokens']=usage.get('completion_tokens',0)
                cost=(record['input_tokens']*.40+record['output_tokens']*1.60)/1e6
                record['cost_usd']=cost;record['cost_state']='PROVIDER_USAGE_X_STANDARD_PRICE_NOT_INVOICE'; self.budget(call_id,cost,'USAGE_RECORDED')
            else:
                record['cost_usd']=None;record['cost_state']='UNKNOWN_RESERVATION_RETAINED';self.budget(call_id,reserve,'UNKNOWN')
            if status!=200: raise RuntimeError('Qwen HTTP '+str(status)+'; response retained')
            text=response['choices'][0]['message']['content']; text=text.strip()
            if text.startswith('```'): text=text.split('\n',1)[1].rsplit('```',1)[0]
            result=json.loads(text)
            if not isinstance(result,dict):raise ValueError('Expected JSON object')
            record['status']='COMPLETED';return result,record
        except ObservationCancelled as error:
            record['status']='CANCELLED_NO_API';record['error']=str(error);record['terminal_ns']=time.monotonic_ns()
            record['cost_usd']=0;record['input_tokens']=0;record['output_tokens']=0;record['cost_state']='NO_REQUEST_SENT_NO_API_CHARGE'
            self.budget(call_id,0,'CANCELLED_UNSENT_RESERVATION_RELEASED')
            raise ModelError(record) from error
        except Exception as error:
            record['status']='FAILED';record['error']=type(error).__name__+': '+str(error);record.setdefault('terminal_ns',time.monotonic_ns());record.setdefault('send_ns',sent)
            if response is None:self.budget(call_id,reserve,'UNKNOWN');record['cost_usd']=None
            raise ModelError(record) from error
        finally:
            with self.resource_lock:
                for resource,owner in list(self.slot_owners.items()):
                    if owner==call_id:self.slot_owners.pop(resource,None);self.last_slot_owner[resource]=call_id
            for name in ('input_text','image_binary','http_request','provider_response'):self.buffers.release(call_id+':'+name)
            with self.lock:
                with (self.root/'model-calls.jsonl').open('a') as f:f.write(json.dumps(record,ensure_ascii=False)+'\n')
                self.active_calls.pop(call_id,None)
class ModelError(Exception):
    def __init__(self,record):self.record=record;super().__init__(record['error'])
