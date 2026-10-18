"""Qwen JSON/vision bridge; single atomic $5 campaign ledger, no auth in evidence."""
import base64, fcntl, json, os, subprocess, threading, time, urllib.request, urllib.error, uuid
from pathlib import Path
MODEL='qwen3.7-plus-2026-05-26'
PRICE={'input_per_million':.40,'output_per_million':1.60,'currency':'USD','region':'Singapore/International','input_bracket':'<=256K','source':'https://www.alibabacloud.com/help/en/model-studio/model-pricing','date':'2026-10-18'}
def dump(path,value):
    path.parent.mkdir(parents=True,exist_ok=True); path.write_text(json.dumps(value,ensure_ascii=False,indent=2))
def credential(service):
    p=subprocess.run(['/usr/bin/security','find-generic-password','-s',service,'-w'],capture_output=True,timeout=15)
    if p.returncode: raise RuntimeError('Credential unavailable: '+service+' (secret not printed)')
    return p.stdout.decode().strip()
class Client:
    def __init__(self,root,limit=5):
        self.root=Path(root); self.root.mkdir(parents=True,exist_ok=True); self.limit=limit
        self.key=credential('via-dashscope-api-key'); workspace=credential('via-dashscope-workspace-id')
        self.url=f'https://{workspace}.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1/chat/completions'
        self.semaphore=threading.Semaphore(4); self.lock=threading.Lock()
    def budget(self,id,amount,state):
        path=self.root/'budget.json'
        with path.open('a+') as f:
            fcntl.flock(f,fcntl.LOCK_EX);f.seek(0); old=f.read(); data=json.loads(old) if old else {'limit_usd':self.limit,'calls':{}}
            total=sum(x['amount_usd'] for x in data['calls'].values())
            if state=='RESERVED' and total+amount>data['limit_usd']: raise RuntimeError('GLOBAL $5 BUDGET STOP')
            data['calls'][id]={'amount_usd':amount,'state':state};f.seek(0);f.truncate();json.dump(data,f,indent=2);f.flush();os.fsync(f.fileno())
    def call(self,payload,meta):
        call_id=str(uuid.uuid4()); start=time.monotonic_ns(); text=json.dumps(payload.get('input',{}),ensure_ascii=False)
        instruction=payload.get('instruction','Return a JSON object.'); max_tokens=int(payload.get('max_tokens',1800))
        content=[{'type':'text','text':text}]; image_bytes=0
        for filename in payload.get('images',[]):
            path=Path(filename); data=path.read_bytes();image_bytes+=len(data); mime='image/jpeg' if path.suffix.lower() in ('.jpg','.jpeg') else 'image/png'
            content.append({'type':'image_url','image_url':{'url':f'data:{mime};base64,'+base64.b64encode(data).decode()}})
        body={'model':MODEL,'messages':[{'role':'system','content':instruction+' Return valid JSON only.'},{'role':'user','content':content if len(content)>1 else text}], 'enable_thinking':False,'temperature':0,'max_tokens':max_tokens,'response_format':{'type':'json_object'}}
        # UTF-8 text bytes + entire image bytes is conservative for these small fixtures.
        reserve=((len(text.encode())+len(instruction.encode())+image_bytes)*.40+max_tokens*1.60)/1e6
        self.budget(call_id,reserve,'RESERVED'); raw=self.root/'raw'/'model';raw.mkdir(parents=True,exist_ok=True)
        dump(raw/(call_id+'-request.json'),body)
        record={**meta,'call_uuid':call_id,'enqueue_ns':start,'model':MODEL,'thinking':False,'price':PRICE,'role':payload.get('role','VIA'),'reservation_usd':reserve,'request_file':str(raw/(call_id+'-request.json'))}
        sent=None; response=None; status=None
        try:
            with self.semaphore:
                sent=time.monotonic_ns();record['send_ns']=sent
                request=urllib.request.Request(self.url,data=json.dumps(body).encode(),headers={'Authorization':'Bearer '+self.key,'Content-Type':'application/json'})
                try:
                    with urllib.request.urlopen(request,timeout=120) as r: response=json.load(r);status=r.status
                except urllib.error.HTTPError as e: status=e.code;response={'http_error':status,'body':e.read().decode(errors='replace')}
                record['terminal_ns']=time.monotonic_ns()
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
        except Exception as error:
            record['status']='FAILED';record['error']=type(error).__name__+': '+str(error);record.setdefault('terminal_ns',time.monotonic_ns());record.setdefault('send_ns',sent)
            if response is None:self.budget(call_id,reserve,'UNKNOWN');record['cost_usd']=None
            raise ModelError(record) from error
        finally:
            with self.lock:
                with (self.root/'model-calls.jsonl').open('a') as f:f.write(json.dumps(record,ensure_ascii=False)+'\n')
class ModelError(Exception):
    def __init__(self,record):self.record=record;super().__init__(record['error'])
