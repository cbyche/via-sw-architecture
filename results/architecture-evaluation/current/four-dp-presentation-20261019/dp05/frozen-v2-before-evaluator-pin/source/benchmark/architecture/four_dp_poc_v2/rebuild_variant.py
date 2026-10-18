"""Rebuild a preserved campaign variant into a NEW scratch crate; no model calls."""
import argparse,shutil,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('variant',choices=['dp05-v2','dp01-A2','dp02-A1','dp04-tactics','dp05-C2']);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
base=Path(__file__).resolve().parents[3]/'prototypes/candidates/four_dp_poc'
if a.output.exists():raise RuntimeError('Choose a new output directory; existing evidence/checkouts are never replaced')
shutil.copytree(base,a.output,ignore=shutil.ignore_patterns('target','variants','__pycache__'))
replacements={'dp05-v2':('dp05-receiver-v2.rs','dp05.rs'),'dp01-A2':('dp01-A2-fragment-invalidation.rs','dp01.rs'),'dp02-A1':('dp02-A1-bounded-side-question.rs','dp02.rs'),'dp05-C2':('dp05-C2-exact-read-cache.rs','dp05.rs')}
if a.variant in replacements:
    source,module=replacements[a.variant];shutil.copyfile(base/'variants'/source,a.output/'src'/module)
subprocess.run(['cargo','build','--offline','--manifest-path',str(a.output/'Cargo.toml')],check=True)
print(a.output.resolve()/'target/debug/four_dp_poc')
