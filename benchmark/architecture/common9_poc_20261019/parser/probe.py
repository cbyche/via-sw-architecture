"""Actual common9 local-parser protocol/cache/RSS preflight. No cloud model calls."""
from pathlib import Path
import sys,json,threading,time,os,subprocess
BASE=Path(__file__).resolve().parents[1];REPO=BASE.parents[2];sys.path.insert(0,str(BASE));from grounding_adapter import GroundingAdapter
sys.path.insert(0,str(REPO/'benchmark/architecture/four_dp_poc_v2'));from memory import LogicalBuffers,sample_processes
OUT=REPO/'results/architecture-evaluation/current/common9-poc-20261019/parser-feasibility/actual-adapter-ko-en';OUT.mkdir(parents=True,exist_ok=True)
buffers=LogicalBuffers();adapter=GroundingAdapter(OUT,buffers);stop=threading.Event();samples=[]
def sampler():
 while not stop.is_set():
  roles={os.getpid():'python_dependency_probe',**adapter.processes};samples.append(sample_processes(roles,'LOCAL_DEPENDENCY_PREFLIGHT',buffers));stop.wait(.05)
t=threading.Thread(target=sampler);t.start();outcomes=[]
try:
 obs=json.loads((BASE/'assets/observations.json').read_text())['events']['S08-U01'];payload={'input':{'frames':obs['frames']},'max_tokens':0}
 for i in[1,2]:
  value,record=adapter.call(payload,{'run_id':'COMMON9_LOCAL_DEPENDENCY_PREFLIGHT','stage':'grounding.local_screen_parser','attempt':i});outcomes.append(record)
  if record['status']!='LOCAL_MODEL_COMPLETED':raise RuntimeError(record['error'])
 assert outcomes[0]['new_frames']==1 and outcomes[1]['new_frames']==0 and outcomes[1]['reused_frames']==1
 assert any('real_local_detector_ocr_caption'in m['rss_bytes_by_role']for m in samples)
 summary={'pass':True,'actual_complete_parser_items':len(outcomes[0]['response_value']['frames'][0]['items']),'cloud_model_calls':0,'formal_measurement':False,'scope':'Actual fullparser + Pythonadapter sourcecache/PID registration only; no semantic binding correctness or common9 QA result','initialization_ns':outcomes[0]['initialization_ns'],'parse_ns':outcomes[0]['inference_ns'],'same_frame_reuse':outcomes[1]['reused_frames'],'sampled_same_time_peak':max(samples,key=lambda m:m['concurrent_sum_bytes'])};(OUT/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2));print(json.dumps({k:summary[k]for k in['pass','actual_complete_parser_items','cloud_model_calls','initialization_ns','parse_ns']}),flush=True)
finally:
 adapter.close();stop.set();t.join();(OUT/'memory-samples.json').write_text(json.dumps(samples,ensure_ascii=False));(OUT/'records.json').write_text(json.dumps(outcomes,ensure_ascii=False,indent=2))
