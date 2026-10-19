#!/usr/bin/env python3
"""Preserve raw common9/condition evidence in <=20MiB lossless Git-sized chunks.

Includes common9, condition diagnostics and supplement6/common15 provenance.
Does not run candidates/models, modify source evidence, or package build runtimes.
"""
import argparse, hashlib, io, json, os, re, stat, tarfile, time
from pathlib import PurePosixPath
from pathlib import Path

REPO=Path(__file__).resolve().parents[2]
COMMON='results/architecture-evaluation/current/common9-poc-20261019'
ROOTS=['benchmark/architecture/common9_poc_20261019','prototypes/candidates/common9_poc_20261019',COMMON,'benchmark/architecture/four_dp_prediction_validation','prototypes/candidates/four_dp_prediction_validation','results/architecture-evaluation/current/four-dp-prediction-validation-20261019']
LEGACY_ROOTS=tuple(ROOTS)
SUPPLEMENT='results/architecture-evaluation/current/common-supplement6-20261019'
ROOTS += ['benchmark/architecture/common_supplement6_20261019', SUPPLEMENT]
ROOTS += ['scripts/architecture/common9_postanalysis']
PROVENANCE_ROLES=('joint441_final_freeze','joint441_plan','common9_current_evaluation','supplement6_current_evaluation','common15_qa04','generation_mapping')
DOCS=Path('docs/architecture/12-decisions/decision-packages/seven-problem-review')
EXCLUDE={'target','weights','cache','hf-cache','hf-modules','.cache','.git','.venv','venv','node_modules','__pycache__','.pytest_cache','preservation','packaging','keychains','secrets'}
MAGIC={b'\xcf\xfa\xed\xfe',b'\xfe\xed\xfa\xcf',b'\xce\xfa\xed\xfe',b'\xfe\xed\xfa\xce',b'\xca\xfe\xba\xbe',b'\xbe\xba\xfe\xca',b'\x7fELF'}
LIMIT=20*1024*1024
if not __debug__:raise RuntimeError("Run without Python -O: integrity assertions are required")

def dump(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def digest(p):
 h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
def stamp(p):
 s=p.lstat();return {'size':s.st_size,'mtime_ns':s.st_mtime_ns,'mode':stat.S_IMODE(s.st_mode)}
def safe_relative(name):
 if not isinstance(name,str) or not name or '\\' in name:return False
 p=PurePosixPath(name)
 return not p.is_absolute() and str(p)==name and all(x not in ('..','.') for x in p.parts)

def excluded_reason(rel,is_dir=False):
 p=PurePosixPath(str(rel));name=p.name.lower()
 if any(x.lower() in EXCLUDE for x in p.parts):return 'BUILD_WEIGHT_CACHE_ENVIRONMENT_OR_PRESERVATION'
 if any(x.lower().startswith(('packaging-preparation','preservation-','evidence-bundle')) for x in p.parts):return 'RECURSIVE_PACKAGING_OUTPUT'
 if name in {'chunk-manifest.json','packaging-complete.json','packaging-failed.json','preservation-inventory.json'} or '.tar.gz.part' in name:return 'RECURSIVE_PACKAGING_OUTPUT'
 if name=='.env' or name.startswith('.env.') or name in {'credentials','credentials.json','credentials.toml','secrets.json','secrets.toml','id_rsa','id_ed25519'} or name.endswith(('.pem','.key','.p12','.pfx','.keychain','.keychain-db')):return 'SECRET_OR_ENVIRONMENT_FILE'
 if name.endswith(('.safetensors','.pt','.pth','.onnx','.gguf','.ckpt','.h5','.tflite')) or name=='pytorch_model.bin':return 'MODEL_WEIGHTS'
 if name.endswith(('.pyc','.pyo')) or name=='.ds_store':return 'REGENERATED_BYTECODE_OR_OS_METADATA'
 return None

# Fail closed instead of redacting raw evidence. Errors contain the path only.
SECRET_PATTERNS=[re.compile(rb'(?<![A-Za-z0-9])sk-[A-Za-z0-9_-]{24,}'),re.compile(rb'gh[pousr]_[A-Za-z0-9]{24,}'),re.compile(rb'-----BEGIN (?:[A-Z ]+ )?PRIVATE KEY-----'),re.compile(rb'(?i)authorization[\"\' :\t]+bearer[ \t]+[A-Za-z0-9_.-]{16,}'),re.compile(rb'(?i)(?:api[_-]?key|access[_-]?token|client[_-]?secret|password)[\"\' \t]*[:=][ \t]*[\"\'][A-Za-z0-9_./+=-]{16,}[\"\']')]
def check_no_secret(p,rel):
 carry=b''
 with p.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):
   data=carry+block
   if any(pattern.search(data) for pattern in SECRET_PATTERNS):raise RuntimeError('Potential credential content; raw file excluded only after explicit review: '+rel)
   carry=data[-1024:]

def selected(repo):
 files=set();excluded=[]
 for name in ROOTS:
  base=repo/name
  if not base.exists():continue
  if base.is_symlink():raise RuntimeError('Scope root is a symlink: '+name)
  for d,dirs,names in os.walk(base,followlinks=False):
   for child in list(dirs):
    p=Path(d)/child;rel=p.relative_to(repo);reason=excluded_reason(rel,True)
    if reason:dirs.remove(child);excluded.append({'path':str(rel),'kind':'DIRECTORY','reason':reason});continue
    if p.is_symlink():files.add(p);dirs.remove(child)
   for child in names:
    p=Path(d)/child;rel=p.relative_to(repo);reason=excluded_reason(rel)
    if reason:excluded.append({'path':str(rel),'kind':'FILE','reason':reason});continue
    files.add(p)
 for prefix in ['17-','18-','19-','20-']:
  files.update((repo/DOCS).glob(prefix+'*.md'))
 for pattern in ['common9*.md','common-supplement*.md','common15*.md','prediction-validation*.md','execution-independent-review.md','evidence-handoff-independent-review.md']:
  files.update((repo/DOCS/'reviews').glob(pattern))
 for name in ['scripts/architecture/package_common9_evidence.py','scripts/architecture/qa04_format_classification.py']:
  tool=repo/name
  if tool.exists():files.add(tool)
 return sorted(files),excluded

def reject_symlink_ancestors(path):
 """Inspect the lexical absolute path; resolve() must not hide an alias."""
 path=Path(os.path.abspath(path))
 if any(node.is_symlink() for node in (path,*path.parents)):raise ValueError('Provenance path has a symlink leaf or ancestor: '+str(path))

def validate_provenance_membership(records,included_files,require_final=False):
 """Pure inventory validation, also usable on verified historical manifests."""
 seen=set();present=set();links={r['path'] for r in included_files if r.get('kind')=='SYMLINK'}
 for row in records:
  role=row['role']
  if role not in PROVENANCE_ROLES or role in seen:raise ValueError('Unknown or duplicate provenance role')
  seen.add(role)
  if not row.get('status','').startswith('PRESENT_HASHED'):continue
  name=row['path']
  if not safe_relative(name) or any(str(p) in links for p in (PurePosixPath(name),*PurePosixPath(name).parents)):raise ValueError('Provenance member is unsafe or has symlink ancestry')
  matches=[r for r in included_files if r.get('path')==name]
  if len(matches)!=1 or matches[0].get('kind')!='FILE':raise ValueError('Hashed provenance must map to exactly one included regular FILE: '+name)
  member=matches[0]
  if member.get('sha256')!=row.get('sha256') or member.get('size')!=row.get('bytes'):raise ValueError('Provenance/member SHA or byte-size mismatch: '+name)
  present.add(role)
 if require_final and present!=set(PROVENANCE_ROLES):raise ValueError('Final publication requires all six provenance roles as included regular files')
 return present==set(PROVENANCE_ROLES)

def provenance(repo,spec=None,hash_files=True,included_files=None):
 """Caller supplies final paths once named; absence never means final validation."""
 rows=[];seen=set()
 for record in (spec or {}).get('records',[]):
  role=record['role'];name=record['path']
  if role not in PROVENANCE_ROLES or role in seen:raise ValueError('Unknown or duplicate provenance role')
  if not safe_relative(name) or not any(name.startswith(root+'/') for root in ROOTS):raise ValueError('Provenance path must be within a preservation scope')
  command=record.get('reproduction_argv',[])
  if not isinstance(command,list) or not all(isinstance(x,str) for x in command):raise ValueError('reproduction_argv must be a string array')
  if any(pattern.search(arg.encode()) for arg in command for pattern in SECRET_PATTERNS):raise ValueError('Potential credential in reproduction command')
  seen.add(role);p=repo/name
  reject_symlink_ancestors(p)
  if p.exists() and not p.is_file():raise ValueError('Provenance path is not a regular file: '+name)
  row={'role':role,'path':name,'status':'PRESENT_NOT_HASHED' if p.is_file() else 'PENDING_MISSING_PATH','reproduction_argv':record.get('reproduction_argv',[]),'commands_executed_by_packager':False}
  if excluded_reason(name):raise ValueError('Provenance path is excluded')
  if p.is_file() and hash_files:
   before=stamp(p);check_no_secret(p,name);row.update(status='PRESENT_HASHED_NOT_INDEPENDENTLY_VALIDATED',sha256=digest(p),bytes=p.stat().st_size)
   reject_symlink_ancestors(p)
   if stamp(p)!=before:raise RuntimeError('Provenance input changed while hashing: '+name)
  rows.append(row)
 rows += [{'role':role,'path':None,'status':'PENDING_NOT_PROVIDED'} for role in PROVENANCE_ROLES if role not in seen]
 if any(r['status'].startswith('PRESENT_HASHED')for r in rows) and included_files is None:raise ValueError('Included-file inventory is required for every hashed provenance role')
 all_present=validate_provenance_membership(rows,included_files or [])
 return {'records':rows,'final_results_certified':False,'all_final_roles_present_and_included':all_present,'final_role_membership_checked':False,'policy':'Every hashed role binds exactly one included regular file with matching path/SHA/bytes and no symlink ancestry. Preparation without inventory stays unhashed/pending. Final publication requires all six roles; byte linkage does not establish final acceptance or model outcomes. Recorded reproduction commands are caller declarations, never executed or endorsed as API-free by the packager.'}

def prepare(repo,out,spec=None):
 """Small metadata-only readiness record: no recursive inventory, archive or run."""
 out.mkdir(parents=True,exist_ok=False)
 docs=sorted(p for prefix in ('17-','18-','19-','20-') for p in (repo/DOCS).glob(prefix+'*.md'))
 reviews=sorted((repo/DOCS/'reviews').glob('common-supplement*.md'))
 result={'version':'COMMON15-PACKAGING-PREPARATION-v2','status':'PREPARATION_ONLY','recorded_at_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'scope_roots':[{'path':r,'exists':(repo/r).is_dir(),'legacy_scope':r in LEGACY_ROOTS} for r in ROOTS],'documents':[{'path':str(p.relative_to(repo)),'sha256':digest(p)} for p in docs+reviews],'tool_sha256':digest(Path(__file__)),'provenance':provenance(repo,spec,hash_files=False),'archive_created':False,'recursive_evidence_inventory_performed':False,'actual_API_calls':0,'limits':{'part_bytes':LIMIT,'final_results_not_required_to_exist_for_preparation':True},'policies':['All explicit preservation scopes use filesystem traversal, including ignored JSONL/CSV, failed attempts and versioned postanalysis tool preparation.','Weights, environments, caches, secrets, compiled executables and recursive packaging outputs excluded.','Potential embedded credentials fail closed without printing or altering raw contents.','Final packaging requires quiescent evidence and six included-file-bound provenance roles; source stamps checked before/during/after archive.','Restore verifies every recorded part/member hash into a new empty directory only.','Prior generations and failures remain distinct; no automatic primary pooling or final validation.']}
 dump(out/'PREPARATION.json',result);return result

def collect(repo,spec=None):
 paths,excluded=selected(repo);included=[]
 for p in paths:
  rel=str(p.relative_to(repo));before=stamp(p)
  if p.is_symlink():
   target=os.readlink(p);resolved=(p.parent/target).resolve()
   if Path(target).is_absolute() or not resolved.is_relative_to(repo.resolve()) or excluded_reason(resolved.relative_to(repo.resolve())):
    excluded.append({'path':rel,'kind':'SYMLINK','reason':'UNSAFE_OR_EXCLUDED_LINK_TARGET'});continue
   included.append({'path':rel,**before,'kind':'SYMLINK','linkname':target,'sha256':hashlib.sha256(target.encode()).hexdigest()});continue
  if not p.is_file():raise RuntimeError('Unsupported evidence file type: '+rel)
  with p.open('rb')as f:magic=f.read(4)
  compiled=magic in MAGIC or (magic[:2]==b'MZ'and p.suffix.lower()in ['.exe','.dll'])or'/frozen/bin/'in '/'+rel
  if compiled:
   frozen=p.parent.parent;recipes=[]
   for cargo in (frozen/'source/prototypes/candidates').rglob('Cargo.toml')if(frozen/'source/prototypes/candidates').exists()else[]:
    recipes.append('cargo build --locked --manifest-path '+str(cargo.relative_to(repo)))
   excluded.append({'path':rel,**before,'kind':'COMPILED_EXECUTABLE','sha256':digest(p),'reason':'Local original retained; excluded from Git evidence bundle; not a complete executable environment clone','rebuild_commands':recipes or['Build the corresponding preserved Cargo.toml with the recorded Rust toolchain; see build_recipes.']});continue
  check_no_secret(p,rel)
  row={'path':rel,**before,'kind':'FILE','sha256':digest(p)}
  if stamp(p)!=before:raise RuntimeError('Input changed while hashing: '+rel)
  included.append(row)
 # Preserve only links to directly included regular files, never link chains/directories.
 regular={row['path'] for row in included if row['kind']=='FILE'}
 keep=[]
 for row in included:
  if row['kind']=='SYMLINK' and str(Path(os.path.normpath((repo/row['path']).parent/row['linkname'])).relative_to(repo.resolve())) not in regular:
   excluded.append({'path':row['path'],'kind':'SYMLINK','reason':'LINK_TARGET_NOT_DIRECTLY_INCLUDED_REGULAR_FILE'})
  else:keep.append(row)
 included=keep
 build=[r['path']for r in included if r['path'].endswith('/Cargo.toml')]
 phase_counts={'planned_common9_episode_runs':315,'planned_supplement6_episode_runs':126,'planned_joint_episode_runs':441,'actual_counts_policy':'No outcome aggregation is performed by the packager. Consult each retained generation plan, trace, cost ledger and certified evaluation separately; repeated copies and failed generations must not be pooled.'}
 return {'version':'COMMON15-PRESERVATION-v2','provenance':provenance(repo,spec,included_files=included),'population_note':'Planned populations are not completed observations. Final result roles remain pending until explicitly supplied.','phase':'EVIDENCE-CHECKPOINT','recorded_at_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'scope':'Lossless raw evidence and frozen source preservation; not a clone of the complete execution environment. Original files are never deleted or overwritten.','population':phase_counts,'included_files':included,'source_selection':[str(p.relative_to(repo)) for p in paths],'excluded':excluded,'included_logical_bytes':sum(r['size']for r in included),'build_recipes':['cargo build --locked --manifest-path '+p for p in build],'external_environment':'Model weights, Cargo target/cache, Python environment and compiled frozen/bin remain local and are excluded. Pinned dependency inventories, package locks, source hashes, recorded binary SHA/size and rebuild commands remain in this bundle.'}

class SplitWriter:
 def __init__(self,out,limit):self.out=out;self.limit=limit;self.parts=[];self.current=None;self.part_hash=None;self.whole=hashlib.sha256();self.total=0;self.count=0
 def _open(self):
  name=f'common9-evidence.tar.gz.part{len(self.parts)+1:04d}';self.current=(self.out/name).open('xb');self.part_hash=hashlib.sha256();self.parts.append({'name':name,'bytes':0});self.count=0
 def _finish_part(self):
  if self.current:self.current.flush();os.fsync(self.current.fileno());self.current.close();self.parts[-1]['sha256']=self.part_hash.hexdigest();self.current=None
 def write(self,data):
  self.whole.update(data);self.total+=len(data);at=0
  while at<len(data):
   if self.current is None:self._open()
   n=min(len(data)-at,self.limit-self.count);piece=data[at:at+n];self.current.write(piece);self.part_hash.update(piece);self.parts[-1]['bytes']+=n;self.count+=n;at+=n
   if self.count==self.limit:self._finish_part()
  return len(data)
 def flush(self):
  if self.current:self.current.flush()
 def finish(self):self._finish_part();return {'archive_name':'common9-evidence.tar.gz','archive_sha256':self.whole.hexdigest(),'archive_bytes':self.total,'max_chunk_bytes':self.limit,'parts':self.parts}
class ConcatReader:
 def __init__(self,paths):self.paths=iter(paths);self.current=None
 def read(self,n=-1):
  chunks=[];remaining=n
  while n<0 or remaining>0:
   if self.current is None:
    try:self.current=next(self.paths).open('rb')
    except StopIteration:break
   b=self.current.read(-1 if n<0 else remaining)
   if not b:self.current.close();self.current=None;continue
   chunks.append(b)
   if n>=0:remaining-=len(b)
  return b''.join(chunks)
 def close(self):
  if self.current:self.current.close()

RESTORE='''# Evidence checkpoint restoration

This is a lossless raw-evidence/source checkpoint, not a full executable environment or proof of final results. Inspect PRESERVATION-INVENTORY.json provenance records and every generation's own freeze, planned population, failures, cost and measurement-status ledgers. Old common9, old condition diagnostics, supplement6 and joint441 retain separate provenance. A planned population is not a completed measurement; partial or technically invalid generations are not silently pooled.

Verify all ordered parts, whole archive and every included member without API calls:

    python3 scripts/architecture/package_common9_evidence.py verify --bundle BUNDLE_DIRECTORY

Restore only into a previously absent or clearly empty directory; the command verifies all restored byte hashes, modes and symlink records:

    python3 scripts/architecture/package_common9_evidence.py restore --bundle BUNDLE_DIRECTORY --out NEW_EMPTY_DIRECTORY

Do not use unverified wildcard concatenation or extract over an existing checkout. The manifest records ordered <=20MiB parts, SHA256 values and every included raw member. Git-ignored JSONL, CSV, failed attempts and prior generations are included by filesystem traversal. Weights, secrets/environments, caches, compiled binaries and recursive preservation outputs are excluded. Potential embedded credential content causes packaging to fail closed, without redacting evidence.

Original files are never overwritten or removed. Source stamps are checked across packaging; use a quiescent, settled result generation. A failure preserves partial packaging output for diagnosis and is not a complete bundle. Absolute paths in historical evidence identify their original host. Relocation requires explicit locator adaptation and a newly reviewed freeze; it cannot rewrite the original evidence or authorize paid execution.

Use the inventory's build_recipes with its recorded toolchain/Cargo.lock. Weights, Python runtimes and Cargo caches must be recreated separately from preserved dependency locks, manifests and upstream hashes. Rebuilt binary SHA can differ by host/path/toolchain; keep the original binary hash and label the rebuild. No credential or Keychain material is bundled.

Current evaluation reproduction commands must come from the exact final provenance records (reproduction_argv) and their retained source freeze; the packager never executes them. Pending records mean the final path/command is not yet supplied and final reproduction has not been validated. Run API-free reanalysis into a fresh output directory only. Deterministic doubles/scaled preparation remain diagnostics; extracting/building a bundle grants no model-call authorization.
'''

def archive(repo,out,limit=LIMIT,spec=None):
 if limit>LIMIT or limit<=0:raise ValueError('Chunk limit must be1..20MiB')
 repo=repo.resolve();out=out.resolve()
 if any(out.is_relative_to(repo/root) for root in ROOTS) and not any(part in {'preservation','packaging'} or part.startswith(('packaging-preparation','preservation-','evidence-bundle')) for part in out.relative_to(repo).parts):raise ValueError('Bundle inside a preserved scope must be in an explicitly excluded preservation/packaging directory')
 out.mkdir(parents=True,exist_ok=False);writer=SplitWriter(out,limit)
 try:
  inventory=collect(repo,spec)
  validate_provenance_membership(inventory['provenance']['records'],inventory['included_files'],require_final=True)
  inventory['provenance']['final_role_membership_checked']=True
  dump(out/'PRESERVATION-INVENTORY.json',inventory);(out/'RESTORE.md').write_text(RESTORE)
  with tarfile.open(fileobj=writer,mode='w|gz',format=tarfile.PAX_FORMAT)as tf:
   for name in ['PRESERVATION-INVENTORY.json','RESTORE.md']:tf.add(out/name,arcname=name,recursive=False)
   for row in inventory['included_files']:
    p=repo/row['path']
    if stamp(p)!={k:row[k]for k in ['size','mtime_ns','mode']}:raise RuntimeError('Input changed before archive: '+row['path'])
    info=tf.gettarinfo(str(p),arcname=row['path'])
    if row['kind']=='SYMLINK':tf.addfile(info);continue
    with p.open('rb')as stream:tf.addfile(info,stream)
    if stamp(p)!={k:row[k]for k in ['size','mtime_ns','mode']}:raise RuntimeError('Input changed during archive: '+row['path'])
  chunk=writer.finish();chunk.update({'phase':inventory['phase'],'inventory_sha256':digest(out/'PRESERVATION-INVENTORY.json'),'included_logical_bytes':inventory['included_logical_bytes'],'included_files':len(inventory['included_files'])});dump(out/'chunk-manifest.json',chunk)
  (out/'SHA256SUMS').write_text(''.join(r['sha256']+'  '+r['name']+'\n'for r in chunk['parts'])+chunk['archive_sha256']+'  common9-evidence.tar.gz\n')
  verify(out)
  for row in inventory['included_files']:
   if stamp(repo/row['path'])!={k:row[k]for k in ['size','mtime_ns','mode']}:raise RuntimeError('Input changed after archive: '+row['path'])
  if [str(p.relative_to(repo)) for p in selected(repo)[0]]!=inventory['source_selection']:raise RuntimeError('Input file selection changed during archive; partial outputs retained')
  dump(out/'PACKAGING-COMPLETE.json',{'phase':inventory['phase'],'archive_sha256':chunk['archive_sha256'],'all_included_raw_bytes_verified':True,'originals_untouched':True,'full_environment_clone':False});return chunk
 except Exception as error:
  writer._finish_part();dump(out/'PACKAGING-FAILED.json',{'error':type(error).__name__+': '+str(error),'partial_outputs_preserved':True,'originals_untouched':True});raise

def bundle_metadata(out):
 manifest=json.loads((out/'chunk-manifest.json').read_text());inventory=json.loads((out/'PRESERVATION-INVENTORY.json').read_text())
 assert digest(out/'PRESERVATION-INVENTORY.json')==manifest['inventory_sha256']
 assert manifest.get('max_chunk_bytes',LIMIT)<=LIMIT
 paths=[];seen=set();whole=hashlib.sha256()
 for row in manifest['parts']:
  name=row['name'];assert re.fullmatch(r'common9-evidence\.tar\.gz\.part[0-9]{4,}',name) and name not in seen;seen.add(name)
  p=out/name;assert not p.is_symlink() and p.is_file() and p.stat().st_size==row['bytes'] and 0<row['bytes']<=LIMIT and digest(p)==row['sha256'];paths.append(p)
  with p.open('rb') as f:
   for block in iter(lambda:f.read(1024*1024),b''):whole.update(block)
 assert paths and whole.hexdigest()==manifest['archive_sha256'] and sum(r['bytes'] for r in manifest['parts'])==manifest['archive_bytes']
 expected={}
 for row in inventory['included_files']:
  name=row['path'];assert safe_relative(name) and name not in expected;expected[name]=row
 links={name for name,row in expected.items() if row['kind']=='SYMLINK'}
 for name,row in expected.items():
  assert not any(str(parent) in links for parent in PurePosixPath(name).parents)
  if row['kind']=='SYMLINK':
   target=row['linkname'];assert not PurePosixPath(target).is_absolute() and '\\' not in target
   # Resolve relative components without consulting host filesystem.
   components=list(PurePosixPath(name).parent.parts)
   for part in PurePosixPath(target).parts:
    if part=='..':assert components;components.pop()
    elif part!='.':components.append(part)
   target_name='/'.join(components);assert target_name in expected and expected[target_name]['kind']=='FILE'
   assert hashlib.sha256(target.encode()).hexdigest()==row['sha256']
 validate_provenance_membership(inventory.get('provenance',{}).get('records',[]),inventory['included_files'],require_final=inventory.get('provenance',{}).get('final_role_membership_checked',False))
 return manifest,inventory,expected,paths

def verify(out):
 manifest,inventory,expected,paths=bundle_metadata(out);seen=set();reader=ConcatReader(paths)
 try:
  with tarfile.open(fileobj=reader,mode='r|gz') as tf:
   for member in tf:
    assert safe_relative(member.name) and member.name not in seen;seen.add(member.name)
    if member.name in ['PRESERVATION-INVENTORY.json','RESTORE.md']:
     assert member.isfile();data=tf.extractfile(member).read();assert data==(out/member.name).read_bytes();continue
    assert member.name in expected;row=expected[member.name]
    assert member.mode==row['mode'] and member.mode & ~0o777==0
    if row['kind']=='SYMLINK':assert member.issym() and member.linkname==row['linkname'];continue
    assert row['kind']=='FILE' and member.isfile() and member.size==row['size'];h=hashlib.sha256();stream=tf.extractfile(member)
    for block in iter(lambda:stream.read(1024*1024),b''):h.update(block)
    assert h.hexdigest()==row['sha256'],member.name
 finally:reader.close()
 assert seen==set(expected)|{'PRESERVATION-INVENTORY.json','RESTORE.md'}
 return {'verified_files':len(expected),'whole_sha256':manifest['archive_sha256'],'phase':inventory['phase']}

def restore(bundle,out):
 # Verification precedes destination creation. No tar.extract/extractall is used.
 verification=verify(bundle);_,inventory,expected,paths=bundle_metadata(bundle)
 if out.is_symlink() or (out.exists() and (not out.is_dir() or any(out.iterdir()))):raise ValueError('Restore destination must be absent or an empty non-symlink directory')
 destination=out.resolve()
 if destination==bundle.resolve() or destination.is_relative_to(bundle.resolve()):raise ValueError('Restore destination cannot be inside the bundle')
 if any(parent.is_symlink() for parent in out.absolute().parents):raise ValueError('Restore destination has a symlink ancestor')
 out.mkdir(parents=True,exist_ok=True);reader=ConcatReader(paths);links=[]
 try:
  with tarfile.open(fileobj=reader,mode='r|gz') as tf:
   for member in tf:
    p=out/member.name;p.parent.mkdir(parents=True,exist_ok=True)
    if member.issym():links.append((p,member.linkname));continue
    with p.open('xb') as target:
     source=tf.extractfile(member)
     for block in iter(lambda:source.read(1024*1024),b''):target.write(block)
    os.chmod(p,member.mode & 0o777)
  for p,target in links:p.symlink_to(target)
 finally:reader.close()
 for name,row in expected.items():
  p=out/name
  if row['kind']=='SYMLINK':assert p.is_symlink() and os.readlink(p)==row['linkname']
  else:assert p.is_file() and not p.is_symlink() and p.stat().st_size==row['size'] and digest(p)==row['sha256'] and stat.S_IMODE(p.stat().st_mode)==row['mode']
 return {**verification,'restored_files':len(expected),'destination':str(destination),'all_recorded_hashes_verified':True,'actual_API_calls':0}

def self_check(out):
 repo=out/'fixture-repo';root=repo/ROOTS[0];root.mkdir(parents=True);checks=[]
 for index,name in enumerate(ROOTS):
  scoped=repo/name;scoped.mkdir(parents=True,exist_ok=True);(scoped/'ignored-raw.jsonl').write_bytes(b'{"status":"failed"}\n\x00unchanged\n');(scoped/'ignored-cost.csv').write_bytes(b'run,cost,status\nr1,0,failed\n')
 (root/'blob.dat').write_bytes(os.urandom(10000))
 for name in ['target/generated','weights/model.safetensors','.venv/runtime','cache/object','preservation/nested/raw.jsonl','packaging-preparation-v2/raw.jsonl','.git/config','evidence-bundle-old/part','hf-cache/file']:
  path=root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'EXCLUDED_SYNTHETIC')
 for name in ['.env','.env.local','credentials.json','private.key','model.onnx']:(root/name).write_bytes(b'EXCLUDED_SYNTHETIC')
 binary=repo/COMMON/'sample/frozen/bin/candidate';binary.parent.mkdir(parents=True);binary.write_bytes(b'\xcf\xfa\xed\xfeFAKE_COMPILED_FILE')
 (root/'unsafe-link').symlink_to('/outside/not-packaged');(root/'raw-link').symlink_to('ignored-raw.jsonl')
 doc=repo/DOCS/'20-synthetic-contract.md';doc.parent.mkdir(parents=True);doc.write_text('Synthetic contract20\n');review=doc.parent/'reviews/common-supplement6-synthetic.md';review.parent.mkdir();review.write_text('Synthetic review\n')
 synthetic_spec={'records':[{'role':role,'path':ROOTS[0]+'/ignored-cost.csv'} for role in PROVENANCE_ROLES]}
 bundle=out/'bundle';result=archive(repo,bundle,4096,spec=synthetic_spec);inv=json.loads((bundle/'PRESERVATION-INVENTORY.json').read_text());included={r['path'] for r in inv['included_files']};excluded={r['path'] for r in inv['excluded']}
 assert all(name+'/ignored-raw.jsonl' in included and name+'/ignored-cost.csv' in included for name in ROOTS);checks.append('all_explicit_scopes_include_ignored_jsonl_csv_and_failures')
 assert str(doc.relative_to(repo)) in included and str(review.relative_to(repo)) in included;checks.append('contract20_and_supplement_review_included')
 assert not any('/.env' in name or '/preservation/' in name or '/packaging-preparation' in name or '/.git/' in name or '/weights/' in name for name in included);assert str(binary.relative_to(repo)) in excluded and binary.exists();assert str((root/'unsafe-link').relative_to(repo)) in excluded;checks.append('weights_env_secret_cache_binary_recursive_output_and_unsafe_link_excluded_originals_retained')
 assert len(result['parts'])>1 and all(p['bytes']<=4096<=LIMIT for p in result['parts']);checks.append('split_parts_limited_and_whole_member_hashes_verified')
 part=bundle/result['parts'][0]['name'];before=part.read_bytes();part.write_bytes(bytes([before[0]^1])+before[1:])
 try:verify(bundle)
 except AssertionError:checks.append('corrupted_part_rejected')
 else:raise AssertionError('Corrupted chunk was accepted')
 part.write_bytes(before);verified=verify(bundle);restored=restore(bundle,out/'restored');assert restored['all_recorded_hashes_verified'];checks.append('empty_directory_restore_all_raw_hashes_modes_and_safe_symlink_verified')
 try:restore(bundle,out/'restored')
 except ValueError:checks.append('nonempty_restore_rejected')
 else:raise AssertionError('Nonempty destination accepted')
 # Preserve a failed credential-scan attempt without storing actual credentials.
 bad=root/'potential-secret.txt';bad.write_bytes(b'api_key="'+b'SYNTHETIC' * 4+b'"')
 try:archive(repo,out/'rejected-secret-bundle',4096,spec=synthetic_spec)
 except RuntimeError:
  assert (out/'rejected-secret-bundle/PACKAGING-FAILED.json').is_file();checks.append('potential_embedded_credential_fails_closed_failure_record_preserved_without_content_output')
 else:raise AssertionError('Potential secret accepted')
 bad.unlink()
 try:archive(repo,root/'bundle-output',4096)
 except ValueError:checks.append('bundle_inside_preserved_scope_rejected')
 else:raise AssertionError('Recursive output accepted')
 assert not safe_relative('../escape') and not safe_relative('/absolute') and not safe_relative('a/../b');checks.append('unsafe_member_paths_rejected')
 manifest_path=bundle/'chunk-manifest.json';original_manifest=manifest_path.read_bytes();malformed=json.loads(original_manifest);malformed['parts'][0]['name']='../escape';dump(manifest_path,malformed)
 try:verify(bundle)
 except AssertionError:checks.append('manifest_part_traversal_rejected')
 else:raise AssertionError('Part traversal accepted')
 manifest_path.write_bytes(original_manifest)
 marker=out/'restore-symlink';marker.symlink_to(out/'absent-target')
 try:restore(bundle,marker)
 except ValueError:checks.append('symlink_restore_destination_rejected')
 else:raise AssertionError('Symlink destination accepted')
 meta_path=bundle/'PRESERVATION-INVENTORY.json';original_inventory=meta_path.read_bytes();meta_path.write_bytes(original_inventory+b' ')
 try:verify(bundle)
 except AssertionError:checks.append('inventory_hash_mismatch_rejected')
 else:raise AssertionError('Inventory mismatch accepted')
 meta_path.write_bytes(original_inventory)
 included_output=archive(repo,repo/COMMON/'preservation/bounded-test',4096,spec=synthetic_spec);assert included_output['archive_bytes']>0;checks.append('explicit_preservation_output_supported_without_recursive_inclusion')
 prep=prepare(repo,out/'preparation');assert prep['provenance']['final_results_certified'] is False and all(r['status']=='PENDING_NOT_PROVIDED' for r in prep['provenance']['records']);checks.append('missing_final_provenance_explicitly_pending_no_final_validation')
 result={'status':'PASS','check_count':len(checks),'checks':checks,'actual_API_calls':0,'actual_evidence_archive_created':False,'verification':verified,'restore':restored};dump(out/'SELF-CHECK.json',result);return result

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','estimate','create','verify','restore','self-check']);p.add_argument('--repo',type=Path,default=REPO);p.add_argument('--out',type=Path);p.add_argument('--bundle',type=Path);p.add_argument('--provenance',type=Path,help='Optional explicit role/path/reproduction_argv records; missing final roles remain pending.');a=p.parse_args();spec=json.loads(a.provenance.read_text()) if a.provenance else None
 if a.action in ('prepare','create','restore','self-check') and a.out is None:p.error('--out is required')
 if a.action in ('verify','restore') and a.bundle is None:p.error('--bundle is required')
 if a.action=='prepare':result=prepare(a.repo,a.out,spec);print(json.dumps({'status':result['status'],'roots':len(result['scope_roots']),'final_results_certified':False,'archive_created':False}))
 elif a.action=='estimate':
  inv=collect(a.repo,spec);print(json.dumps({'phase':inv['phase'],'included_files':len(inv['included_files']),'logical_bytes':inv['included_logical_bytes'],'excluded_binaries':sum(r['kind']=='COMPILED_EXECUTABLE' for r in inv['excluded'])}))
 elif a.action=='create':print(json.dumps(archive(a.repo,a.out,spec=spec),indent=2))
 elif a.action=='verify':print(json.dumps(verify(a.bundle)))
 elif a.action=='restore':print(json.dumps(restore(a.bundle,a.out)))
 else:a.out.mkdir(parents=True,exist_ok=False);print(json.dumps(self_check(a.out)))
