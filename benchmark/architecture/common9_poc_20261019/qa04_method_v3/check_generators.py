"""In-memory anchor and Rust syntax preparation checks, never candidate acceptance.

Reads an immutable review checkpoint. No copied crate, build, candidate process,
model call, semantic check, module-count result or common9 regression is created.
"""
import argparse
import ast
import hashlib
import json
import subprocess
from pathlib import Path
import extensions
import patches


class MemoryPath:
    def __init__(self, files, path=''):
        self.files, self.path = files, path
    def __truediv__(self, part):
        return MemoryPath(self.files, self.path+'/'+str(part) if self.path else str(part))
    def read_text(self):
        return self.files[self.path]
    def write_text(self, value):
        self.files[self.path] = value


def check(checkpoint):
    manifest=json.loads(checkpoint.read_text());prefix='prototypes/candidates/common9_poc_20261019/'
    originals={}
    for row in manifest['sources']:
        if not row['relative_path'].startswith(prefix) or not row['relative_path'].endswith('.rs'):continue
        path=Path(row['snapshot_path']);raw=path.read_bytes()
        if hashlib.sha256(raw).hexdigest()!=row['sha256']:raise ValueError('Immutable review snapshot changed: '+str(path))
        originals[row['relative_path'][len(prefix):]]=raw.decode()
    rows=[]
    for task in ['C01','C02','C03','C04','C05','C06']:
        files=dict(originals);candidate=MemoryPath(files)
        apply=getattr(patches,'apply_'+task.lower(),None)or getattr(extensions,'apply_'+task.lower())
        apply(candidate)
        syntax=[]
        for name,text in files.items():
            if originals.get(name)==text:continue
            r=subprocess.run(['rustfmt','--edition','2021','--emit','stdout','--config','skip_children=true'],input=text,capture_output=True,text=True)
            syntax.append({'in_memory_path':name,'returncode':r.returncode,'stderr':r.stderr,'generated_text_sha256':hashlib.sha256(text.encode()).hexdigest()})
            if r.returncode:raise ValueError('Generated Rust syntax failed: '+task+'/'+name+'\n'+r.stderr)
        rows.append({'task':task,'status':'ANCHORS_AND_RUST_SYNTAX_ONLY','syntax':syntax,'semantic_acceptance':'NOT_RUN','operational_module_count':'UNKNOWN'})
    return {'status':'PASS_PREPARATION_ONLY','review_checkpoint':str(checkpoint.resolve()),'review_checkpoint_sha256':hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
        'generator_rows':rows,'candidate_copies':0,'candidate_builds':0,'candidate_processes':0,'provider_calls':0,'common9_regressions':0,
        'scope':'In-memory source transformations against a review checkpoint and rustfmt parsing only. Not the final freeze, Rust type checking, feature behavior or acceptance.'}


def main():
    p=argparse.ArgumentParser();p.add_argument('--checkpoint',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    for source in Path(__file__).parent.glob('*.py'):ast.parse(source.read_text(),filename=str(source))
    result=check(a.checkpoint);a.output.parent.mkdir(parents=True,exist_ok=True)
    if a.output.exists():raise ValueError('Preserve prior preparation records; use a new output path')
    a.output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'status':result['status'],'generators':len(result['generator_rows']),'candidate_copies':0,'provider_calls':0}))


if __name__=='__main__':main()
