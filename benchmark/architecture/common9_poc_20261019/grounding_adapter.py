"""Real dedicated screen dependency. No evaluator, authoring SVG or target boxes are read."""
from pathlib import Path
import hashlib,json,os,select,subprocess,threading,time,uuid
from lifecycle import ObservationGate,ObservationCancelled
HERE=Path(__file__).resolve().parent
PYTHON='/private/tmp/via-dp04-omniparser-venv/bin/python'
class GroundingAdapter:
 def __init__(self,root,buffers,observation_gate=None):
  self.root=Path(root);self.buffers=buffers;self.processes={};self.process_history={};self.process_start_metadata={};self.proc=None;self.log=None;self.cache={};self.lock=threading.Lock();self.initialization=None
  self.observation_gate=observation_gate or ObservationGate()
 def _dump(self,path,value):path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
 def _line(self,timeout):
  deadline=time.monotonic()+timeout
  while True:
   settlement=getattr(self.observation_gate,'settlement_deadline_ns',None)
   remaining=deadline-time.monotonic()
   if settlement is not None:remaining=min(remaining,(settlement-time.monotonic_ns())/1e9)
   if remaining<=0:raise TimeoutError('Dedicated parser timeout/settlement deadline')
   if select.select([self.proc.stdout],[],[],min(remaining,1))[0]:break
  text=self.proc.stdout.readline()
  if not text:raise RuntimeError('Dedicated parser process ended before protocol response')
  return json.loads(text)
 def call(self,payload,meta):
  enqueue=time.monotonic_ns()
  with self.lock:
   start=time.monotonic_ns();ident='LOCAL-'+uuid.uuid4().hex;folder=self.root/'raw/local-model';folder.mkdir(parents=True,exist_ok=True)
   record={**meta,'call_uuid':ident,'enqueue_ns':enqueue,'send_ns':start,'evidence_kind':'LOCAL_MODEL','local_model_type':'dedicated_screen_parser','provider':'task-local OmniParser-v2.0 legacy YOLO/EasyOCR(Korean+English)/Florence2 CPU','cloud_api_calls':0,'cost_usd':0,'cost_scope':'Zero cloudAPIcharge only; localcompute monetarycost NOT_MEASURED','monetized_local_compute_cost_usd':None,'input_tokens':0,'output_tokens':0,'initialization_ns':0,'inference_ns':0,'new_frames':0,'reused_frames':0,'local_spawn_admitted_ns':None,'local_parse_dispatch_admitted_ns':None,'local_dispatch_admissions':[]}
   try:
    self.observation_gate.check()
    frames=payload['input']['frames']
    for f in frames:
     if hashlib.sha256(Path(f['image_path']).read_bytes()).hexdigest()!=f['sha256']:raise ValueError('Actual parser source hash mismatch')
    self._dump(folder/(ident+'-request.json'),payload);record['request_file']=str(folder/(ident+'-request.json'))
    if self.proc is None:
     self.log=(folder/(ident+'-stderr.log')).open('w')
     with self.observation_gate.lock:
      admitted=self.observation_gate.check_locked();record['local_spawn_admitted_ns']=admitted;record['local_dispatch_admissions'].append({'kind':'PROCESS_SPAWN','admitted_ns':admitted});record['send_ns']=admitted
      spawn_begin=time.monotonic_ns();self.proc=subprocess.Popen([PYTHON,str(HERE/'parser/service.py')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.log,text=True,bufsize=1);spawn_return=time.monotonic_ns()
     self.process_start_metadata[str(self.proc.pid)]={'role':'real_local_detector_ocr_caption','spawn_begin_ns':spawn_begin,'spawn_return_ns':spawn_return,'registered_at_ns':time.monotonic_ns()}
     self.processes[self.proc.pid]='real_local_detector_ocr_caption';self.process_history[self.proc.pid]='real_local_detector_ocr_caption';record['parser_pid']=self.proc.pid
     record['process_start_metadata']=self.process_start_metadata[str(self.proc.pid)].copy()
     ready=self._line(180)
     if not ready.get('ready'):raise RuntimeError('Dedicated parser did not initialize')
     self.initialization=ready;record['initialization_ns']=ready['initialization_ns'];record['initialization']=ready
    record['parser_pid']=self.proc.pid
    needed=[f for f in frames if (f['frame_id'],f['source_revision'],f['sha256']) not in self.cache]
    if needed:
     encoded=json.dumps({'frames':needed})+'\n'
     with self.observation_gate.lock:
      admitted=self.observation_gate.check_locked();record['local_parse_dispatch_admitted_ns']=admitted;record['local_dispatch_admissions'].append({'kind':'PARSE_REQUEST','admitted_ns':admitted})
      record['parse_send_begin_ns']=time.monotonic_ns();self.proc.stdin.write(encoded);self.proc.stdin.flush();record['parse_send_end_ns']=time.monotonic_ns()
     raw=self._line(240);record['actual_parser_raw_response']=raw
     if raw.get('parser_error'):raise RuntimeError(raw['parser_error'])
     record['inference_ns']=sum(f['parse_ns']for f in raw['frames']);record['new_frames']=len(needed)
     with self.observation_gate.lock:
      record['local_cache_commit_ns']=self.observation_gate.check_locked()
      for f in raw['frames']:self.cache[(f['frame_id'],f['source_revision'],f['source_sha256'])]=f
    self.observation_gate.check()
    record['reused_frames']=len(frames)-len(needed)
    result={'parser':'Actual OmniParser detector + EasyOCR Korean+English + Florence2 caption','deployment':'local CPU diagnostic; original proposedcloudservice NOT_MEASURED','frames':[self.cache[(f['frame_id'],f['source_revision'],f['sha256'])]for f in frames],'fallible_derived_interpretation':True,'no_intent_or_final_target_authority':True}
    # Tuple keys are internal cache identities, not additional serialized model data.
    self.buffers.set('dedicated_parser_response_cache',sum(len(json.dumps(v,ensure_ascii=False).encode())for v in self.cache.values()),'local_parser_derived_cache')
    record['status']='LOCAL_MODEL_COMPLETED';record['response_value']=result
   except ObservationCancelled as error:
    result={'parser_error':str(error),'cancelled_after_observation_window':True};record['status']='LOCAL_MODEL_CANCELLED';record['error']=str(error);record['response_value']=result;self.close()
   except Exception as error:
    result={'parser_error':type(error).__name__+': '+str(error),'dedicated_parser_required_but_unavailable':True};record['status']='LOCAL_MODEL_FAILED';record['error']=result['parser_error'];record['response_value']=result;self.close()
   record['terminal_ns']=time.monotonic_ns();record['response_file']=str(folder/(ident+'-response.json'));self._dump(Path(record['response_file']),result);self._dump(folder/(ident+'-record.json'),record)
   return result,record
 def close(self):
  if self.proc is not None:
   try:self.proc.stdin.close();self.proc.wait(timeout=5)
   except (BrokenPipeError,subprocess.TimeoutExpired):self.proc.kill();self.proc.wait()
   self.processes.pop(self.proc.pid,None);self.proc=None
   if self.log:self.log.close();self.log=None
  self.cache.clear();self.buffers.release('dedicated_parser_response_cache')
