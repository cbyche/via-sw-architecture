"""Hash actual local dependencies after complete parser probe, before formal freeze."""
from pathlib import Path
import json,hashlib,subprocess,difflib,platform
BASE=Path(__file__).resolve().parents[1];REPO=BASE.parents[3];OUT=REPO/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp04';LOCAL=Path('/private/tmp/via-dp04-omniparser');VENV=Path('/private/tmp/via-dp04-omniparser-venv')
def sha(p):
 h=hashlib.sha256()
 with p.open('rb')as f:
  for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
 return h.hexdigest()
weights=[p for p in (LOCAL/'weights').rglob('*')if p.is_file()and p.suffix in ['.pt','.pth','.safetensors']];sources=[p for p in(LOCAL/'util').glob('*.py')if'.upstream.'not in p.name]
source_map={str(p):sha(p)for p in sources};external_configs=[p for p in (LOCAL/'weights').rglob('*.json')if p.is_file()];package_lock=subprocess.check_output([str(VENV/'bin/python'),'-m','pip','freeze'],text=True);(OUT/'parser-packages.lock.txt').write_text(package_lock)
upstream=(LOCAL/'util/utils.upstream.py').read_text();adapted=(LOCAL/'util/utils.py').read_text();(OUT/'omniparser-adapter.patch').write_text(''.join(difflib.unified_diff(upstream.splitlines(True),adapted.splitlines(True),fromfile='pinned-upstream/util/utils.py',tofile='task-local/util/utils.py')))
model_assets=[]
for p in(LOCAL/'hf-cache/hub').rglob('*.py'):
 if p.is_file():model_assets.append({'path':str(p),'sha256':sha(p),'bytes':p.stat().st_size})
manifest={'parser':'OmniParser-v2.0 detector+EasyOCR+Florence2 caption','upstream_revision':'354021201345a96178360b28733573e27269f2de','upstream_url':'https://github.com/microsoft/OmniParser/tree/354021201345a96178360b28733573e27269f2de','deployment':'Actual localCPU diagnostic replacing proposed unvalidated clouddependency; cloudserviceprice/time/localmemory NOT_MEASURED','cpu_threads':2,'formal_cache_policy':'freshresidenthelperpertrial; sourcehashcachedonlywithintrial; actualstartupafterstimulus included','weights':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}for p in weights],'external_source_sha256':source_map,'external_config_sha256':{str(p):sha(p)for p in external_configs},'remote_code_assets':model_assets,'package_lock_sha256':sha(OUT/'parser-packages.lock.txt'),'adapter_patch_sha256':sha(OUT/'omniparser-adapter.patch'),'local_helper_monetary_cost':'NOT_MONETIZED; not a free cloudserviceclaim','python':str(VENV/'bin/python'),'host':platform.platform(),'reproduce':'bundledPython3.12 parser/reproduce.py --install --download; then taskvenv/bin/python parser_preflight.py; actualassets pinned bythese hashes','actual_complete_parser_preflight':json.loads((OUT/'parser-preflight/summary.json').read_text())}
(OUT/'parser-inventory.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'weights':len(weights),'total_weight_bytes':sum(p.stat().st_size for p in weights),'source_files':len(sources)}))
