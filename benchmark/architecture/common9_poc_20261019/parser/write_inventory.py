"""Record actual parser dependency hashes without copying weights/caches into Git."""
from pathlib import Path
import hashlib,json,difflib,subprocess,shutil
HERE=Path(__file__).resolve().parent;REPO=HERE.parents[3];LOCAL=Path('/private/tmp/via-common9-omniparser');OUT=REPO/'results/architecture-evaluation/current/common9-poc-20261019/parser-feasibility';VENV=Path('/private/tmp/via-dp04-omniparser-venv')
def sha(p):
 h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
pristine=(LOCAL/'util/utils.upstream.py').read_text();adapted=(LOCAL/'util/utils.py').read_text();(OUT/'upstream-adapter.patch').write_text(''.join(difflib.unified_diff(pristine.splitlines(True),adapted.splitlines(True),fromfile='pinned-upstream/util/utils.py',tofile='common9-runtime/util/utils.py')))
up=HERE/'upstream';up.mkdir(exist_ok=True)
for name in ['utils.upstream.py','omniparser.py','box_annotator.py','__init__.py']:shutil.copy2(LOCAL/'util'/name,up/name.replace('utils.upstream.py','utils.py'))
(OUT/'packages.lock.txt').write_text(subprocess.check_output([str(VENV/'bin/python'),'-m','pip','freeze'],text=True))
files=[LOCAL/'weights/icon_detect/model.pt',LOCAL/'weights/icon_caption/model.safetensors',LOCAL/'weights/easyocr/craft_mlt_25k.pth',LOCAL/'weights/easyocr/korean_g2.pth']
weights=[{'path':str(p),'resolved_path':str(p.resolve()),'bytes':p.stat().st_size,'sha256':sha(p)}for p in files]
external={str(p):sha(p)for p in(LOCAL/'util').glob('*.py')if'.upstream.'not in p.name}
manifest={'upstream_revision':'354021201345a96178360b28733573e27269f2de','upstream_url':'https://github.com/microsoft/OmniParser/tree/354021201345a96178360b28733573e27269f2de','parser':'OmniParser-v2.0 legacy detector + EasyOCR Korean/English + Florence2 caption','deployment':'ACTUAL_LOCAL_CPU; original proposed cloud-service performance/price NOT_MEASURED','cpu_threads':2,'ocr_languages':['ko','en'],'weights_loaded':weights,'unused_cached_weight_files_not_loaded':['english_g2.pth'],'external_runtime_source_sha256':external,'remote_code_loaded_sha256':{str(p):sha(p) for p in (LOCAL/'hf-modules').rglob('*.py') if p.is_file()},'checkpoint_config_sha256':{str(p):sha(p) for p in (LOCAL/'weights/icon_caption').glob('*.json')},'runtime_python':str(VENV/'bin/python'),'python_packages_lock_sha256':sha(OUT/'packages.lock.txt'),'adapter_diff_sha256':sha(OUT/'upstream-adapter.patch'),'source_adjustments':['Removed unused AzureOpenAI import','Removed eagerly initialized unused PaddleOCR; official path uses EasyOCR','EasyOCR CPU and ko+en for actual common Korean sources; full extra checkpoint counted','Official icon_caption_florence checkpoint directory alias restored; otherwise upstream chooses incorrect generation branch'],'source_aliases':{'icon_caption_florence':'icon_caption; same actualweights, no duplicate weights load'},'preflight':{'english_attempts':['attempt01-error.json','attempt02-error.json','attempt03-output.json','actual-adapter/summary.json'],'current_ko_en':'actual-adapter-ko-en/summary.json','formal_QA_or_accuracy_measurement':False,'cloud_model_calls':0},'reproduction':'bundled Python3.12 parser/reproduce.py --install --download; bundled Python3.12 parser/probe.py (actual helper only; no cloud requests)'}
(OUT/'dependency-inventory.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'actual_loaded_weight_files':len(weights),'actual_loaded_weight_file_bytes':sum(w['bytes']for w in weights),'ocr_languages':manifest['ocr_languages'],'cloud_calls':0}))
