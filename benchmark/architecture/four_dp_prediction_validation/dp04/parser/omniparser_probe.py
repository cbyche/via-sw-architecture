"""Task-local complete detector/OCR/caption probe; no cloud calls."""
import os,sys,time,json,base64,hashlib
from pathlib import Path
os.environ['HF_HOME']='/private/tmp/via-dp04-omniparser/hf-cache'
os.environ['YOLO_CONFIG_DIR']='/private/tmp/via-dp04-omniparser/yolo-config'
os.environ['MPLCONFIGDIR']='/private/tmp/via-dp04-omniparser/mpl-config'
os.environ['TORCH_HOME']='/private/tmp/via-dp04-omniparser/torch-cache'
os.environ['HF_MODULES_CACHE']='/private/tmp/via-dp04-omniparser/hf-modules'
sys.path.insert(0,'/private/tmp/via-dp04-omniparser')
start=time.monotonic_ns()
from util.omniparser import Omniparser
parser=Omniparser({'som_model_path':'/private/tmp/via-dp04-omniparser/weights/icon_detect/model.pt','caption_model_name':'florence2','caption_model_path':'/private/tmp/via-dp04-omniparser/weights/icon_caption','BOX_TRESHOLD':.05})
print('READY',flush=True)
for name in sys.argv[1:]:
 t=time.monotonic_ns();image,content=parser.parse(base64.b64encode(Path(name).read_bytes()).decode());out={'parser':'OmniParser-v2.0 legacy YOLO detector + EasyOCR + Florence2 caption CPU; official pinned utils with unused Azure/Paddle imports/init removed','source':name,'parse_ns':time.monotonic_ns()-t,'since_start_ns':time.monotonic_ns()-start,'parsed_content_list':content}
 Path(name+'.omniparser.json').write_text(json.dumps(out,indent=2));print(json.dumps({'source':name,'items':len(content),'parse_s':out['parse_ns']/1e9}),flush=True)
