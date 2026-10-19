"""Reproduce task-local OmniParser setup. Network/downloads explicit; never sends model API calls."""
import argparse,urllib.request,subprocess,sys,pathlib,difflib,json,hashlib,os
BASE=pathlib.Path(__file__).resolve().parent;ROOT=pathlib.Path('/private/tmp/via-common9-omniparser');VENV=pathlib.Path('/private/tmp/via-dp04-omniparser-venv');PIN='354021201345a96178360b28733573e27269f2de'
os.environ['PIP_CACHE_DIR']=str(ROOT/'pip-cache')
p=argparse.ArgumentParser();p.add_argument('--download',action='store_true');p.add_argument('--install',action='store_true');a=p.parse_args()
if a.install:
 subprocess.run([sys.executable,'-m','venv',str(VENV)],check=True)
 subprocess.run([str(VENV/'bin/python'),'-m','pip','install','torch==2.5.1','torchvision==0.20.1','transformers==4.49.0','easyocr==1.7.2','ultralytics==8.3.70','supervision==0.18.0','numpy==1.26.4','timm','einops==0.8.0','accelerate','huggingface_hub'],check=True)
if a.download:
 for file in ['util/utils.py','util/omniparser.py','util/box_annotator.py','util/__init__.py']:
  path=ROOT/file;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(urllib.request.urlopen('https://raw.githubusercontent.com/microsoft/OmniParser/'+PIN+'/'+file,timeout=60).read())
 for file in ['icon_detect/model.pt','icon_caption/config.json','icon_caption/generation_config.json','icon_caption/model.safetensors']:
  path=ROOT/'weights'/file;path.parent.mkdir(parents=True,exist_ok=True)
  if not path.exists():urllib.request.urlretrieve('https://huggingface.co/microsoft/OmniParser-v2.0/resolve/main/'+file,str(path))
 path=ROOT/'util/utils.py';original=path.read_text();path.with_suffix('.upstream.py').write_text(original);start=original.index('from paddleocr import PaddleOCR');end=original.index('import time',start);adapted=original[:start]+"reader = easyocr.Reader(['ko','en'], gpu=False, model_storage_directory='/private/tmp/via-common9-omniparser/weights/easyocr')\n"+original[end:];adapted=adapted.replace('from openai import AzureOpenAI','');path.write_text(adapted)
 alias=ROOT/'weights/icon_caption_florence'
 if not alias.exists():alias.symlink_to('icon_caption',target_is_directory=True)
 print('Run parser/service.py in this venv; first load may fetch official Florence processor and EasyOCR checkpoints. Result manifest records hashes and any further compatibility adapter.')
