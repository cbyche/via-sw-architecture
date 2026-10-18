"""Actual C01/C03 compatibility controls on the corrected immutable baseline; no models."""
from pathlib import Path
import ast,json,shutil,difflib,hashlib,subprocess
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2/run.py').exists());OUT=Path(__file__).resolve().parent;CAMPAIGN=OUT.parents[1];BASE=CAMPAIGN/'calibration-v2/frozen/source/prototypes/candidates/four_dp_poc_v2'
old=REPO/'results/architecture-evaluation/current/four-dp-qwen37-20261018/qa04/exercise.py';constants={}
for node in ast.parse(old.read_text()).body:
 if isinstance(node,ast.Assign) and len(node.targets)==1 and isinstance(node.targets[0],ast.Name) and node.targets[0].id in ['WIRE','NORMALIZER','C01PROBE','C03PROBE']:constants[node.targets[0].id]=ast.literal_eval(node.value)
changes=[]
for task in ['C01','C03']:
 crate=OUT/task/'candidate'
 if crate.exists():raise SystemExit('Preserve existing task snapshot')
 shutil.copytree(BASE,crate);(crate/'src/bin').mkdir()
 if task=='C01':(crate/'src/model_wire.rs').write_text(constants['WIRE']);probe=constants['C01PROBE']
 else:
  source=crate/'src/source_contract.rs';s=source.read_text();source.write_text(s[:s.index('pub fn normalize(')]+constants['NORMALIZER'])
  main=crate/'src/main.rs';s=main.read_text();needle='let input:Value=serde_json::from_str(&line).unwrap();';assert needle in s;main.write_text(s.replace(needle,needle+'\n    let input=source_contract::normalize_input(&input);'));probe=constants['C03PROBE']
 (crate/'src/bin/contract_probe.rs').write_text(probe)
 changed=[];diff=[]
 for p in sorted(BASE.rglob('*.rs')):
  q=crate/p.relative_to(BASE)
  if p.read_bytes()!=q.read_bytes():changed.append(str(p.relative_to(BASE)));diff.extend(difflib.unified_diff(p.read_text().splitlines(True),q.read_text().splitlines(True),fromfile='a/'+str(p.relative_to(BASE)),tofile='b/'+str(p.relative_to(BASE))))
 (OUT/task/'actual.patch').write_text(''.join(diff));changes.append({'task':task,'changed_operational_modules':changed,'count':len(changed),'test_only_new_probe_excluded':True})
(OUT/'manifest.json').write_text(json.dumps({'baseline_freeze_sha256':hashlib.sha256((CAMPAIGN/'calibration-v2/freeze.json').read_bytes()).hexdigest(),'changes':changes,'claim_scope':'C01 is Rust-Python IPC wire/usage, not vendor HTTP API. C03 is actual input source/frame version/time/coordinate metadata normalization. Controls remain limited adapter tasks.'},indent=2));print(json.dumps(changes))
