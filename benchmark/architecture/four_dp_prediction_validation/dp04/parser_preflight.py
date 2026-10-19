"""Actual local parser + actual Rust wire with cloud service doubles. Zero paid calls."""
from pathlib import Path
import sys,json,shutil,hashlib
BASE=Path(__file__).resolve().parent;sys.path.insert(0,str(BASE));import run
from campaign import LiveClient
from bridge import dump
ROOT=BASE.parents[3]/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp04/parser-preflight'
class Probe(run.OfflineClient):
 local_parser=LiveClient.local_parser
 close_parser=LiveClient.close_parser
 def __init__(self,root):
  super().__init__(root);self.parser_process=None;self.parser_cache={};self.parser_log=None;self.process_role_history={}
 def call(self,payload,meta):
  if meta['stage']=='dp04.local_screen_parser':return self.local_parser(payload,meta)
  return super().call(payload,meta)
ROOT.mkdir(parents=True,exist_ok=True);binary=ROOT/'frozen/bin/four_dp_poc';binary.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(run.CRATE/'target/debug/four_dp_poc',binary);dump(ROOT/'freeze.json',{'binary':{'path':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}})
case=next(e for e in json.loads((BASE/'fixtures/dp04.json').read_text())if e['id']=='dp04-held-small-text')
r=run.execute(Probe(ROOT),case,'A4',1,'LOCAL_PARSER_PREFLIGHT_CLOUD_DOUBLES');local=[c for c in r['calls']if c.get('evidence_kind')=='LOCAL_PARSER_NOT_CLOUD_MODEL_CALL'];assert len(local)==1 and local[0]['response_value']['frames'][0]['items'];assert any('real_local_detector_ocr_caption'in m['rss_bytes_by_role'] for m in r['memory']);assert not[e for e in r['events']if e.get('kind')in['GOAL_FAILED','EPISODE_ERROR']]
dump(ROOT/'summary.json',{'provider_calls':0,'actual_local_helper_operations':len(local),'run_id':r['run_id'],'actual_helper_items':len(local[0]['response_value']['frames'][0]['items']),'helper_init_ns':local[0]['initialization']['initialization_ns'],'parse_ns':local[0]['inference_ns_new_frames'],'pass':True,'claim':'Real detector/OCR/caption and actual Rust/bridge protocol; cloud semantic doubles mean no referent quality or formal latency measurement.'});print(json.dumps({'actual_helper_items':len(local[0]['response_value']['frames'][0]['items']),'pass':True,'provider_calls':0}))
