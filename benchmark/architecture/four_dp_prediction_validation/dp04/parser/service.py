"""Actual OmniParser subprocess. stdout contains protocol JSON only. No fixture/oracle reads."""
import os,sys,time,json,base64,contextlib
from pathlib import Path
os.environ['HF_HOME']='/private/tmp/via-dp04-omniparser/hf-cache';os.environ['YOLO_CONFIG_DIR']='/private/tmp/via-dp04-omniparser/yolo-config';os.environ['OMP_NUM_THREADS']='2';os.environ['TOKENIZERS_PARALLELISM']='false'
os.environ['MPLCONFIGDIR']='/private/tmp/via-dp04-omniparser/mpl-config'
os.environ['TORCH_HOME']='/private/tmp/via-dp04-omniparser/torch-cache'
os.environ['HF_MODULES_CACHE']='/private/tmp/via-dp04-omniparser/hf-modules'
sys.path.insert(0,'/private/tmp/via-dp04-omniparser')
start=time.monotonic_ns()
with contextlib.redirect_stdout(sys.stderr):
 import torch
 torch.set_num_threads(2)
 from util.omniparser import Omniparser
 parser=Omniparser({'som_model_path':'/private/tmp/via-dp04-omniparser/weights/icon_detect/model.pt','caption_model_name':'florence2','caption_model_path':'/private/tmp/via-dp04-omniparser/weights/icon_caption','BOX_TRESHOLD':.05})
print(json.dumps({'ready':True,'initialization_ns':time.monotonic_ns()-start,'deployment':'local CPU diagnostic','threads':2}),flush=True)
for line in sys.stdin:
 try:
  r=json.loads(line);out=[]
  for f in r['frames']:
   t=time.monotonic_ns()
   with contextlib.redirect_stdout(sys.stderr):
    image,items=parser.parse(base64.b64encode(Path(f['image_path']).read_bytes()).decode())
   out.append({'frame_id':f['frame_id'],'source_revision':f['source_revision'],'source_sha256':f['sha256'],'items':items,'parse_ns':time.monotonic_ns()-t,'coordinate_space':'ratio_0_1_original_frame'})
  print(json.dumps({'parser':'OmniParser-v2.0 YOLO legacy detector + EasyOCR English + Florence2 caption','frames':out,'scope':'Fallible text/icon boxes/captions only; not speech intent, not complete graph or card extents','deployment':'local CPU diagnostic; cloud service latency/price NOT_MEASURED'}),flush=True)
 except Exception as e:print(json.dumps({'parser_error':type(e).__name__+': '+str(e)}),flush=True)
