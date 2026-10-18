"""Fresh-process runner using an explicitly selected scratch crate; no baseline mutation."""
import argparse,sys
from pathlib import Path
import run
p=argparse.ArgumentParser(add_help=False);p.add_argument('--crate',required=True);a,remaining=p.parse_known_args();run.CRATE=Path(a.crate).resolve();sys.argv=['run.py',*remaining];run.main()
