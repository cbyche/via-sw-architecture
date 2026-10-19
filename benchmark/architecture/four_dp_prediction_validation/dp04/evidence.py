"""Resolve evaluator files against each immutable phase, not mutable live fixtures."""
import hashlib,json
from pathlib import Path
BASE=Path(__file__).resolve().parent
REPO=BASE.parents[3]
def digest(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def phase_root(record):
    for parent in Path(record['input']).resolve().parents:
        if(parent/'freeze.json').exists():return parent
    return None

def evaluator_base(record):
    phase=phase_root(record)
    if phase is None:
        if record.get('offline_responder'):return BASE
        raise ValueError('Provider record has no immutable phase freeze')
    frozen=phase/'freeze.json';expected=record.get('freeze_sha256')
    if expected and digest(frozen)!=expected:raise ValueError('Record freeze digest mismatch')
    return phase/'frozen/source/benchmark/architecture/four_dp_prediction_validation/dp04'

def frozen_file(record,source):
    source=Path(source)
    if not record.get('freeze_sha256'):return source
    phase=phase_root(record)
    if phase is None:raise ValueError('Missing phase for frozen file')
    relative=source.resolve().relative_to(REPO)
    manifest=json.loads((phase/'freeze.json').read_text());candidate=phase/'frozen/source'/relative
    wanted=manifest['source'].get(str(relative))
    if wanted is None:raise ValueError('Asset absent from frozen source inventory: '+str(relative))
    if candidate.exists() and digest(candidate)==wanted:return candidate
    # Historical identity regressions can have hash-only original freezes.
    if source.exists() and digest(source)==wanted:return source
    raise ValueError('Frozen evaluator/source asset mismatch: '+str(relative))

def read_evaluator(record,relative):
    path=evaluator_base(record)/relative
    if record.get('freeze_sha256'):
        phase=phase_root(record);manifest=json.loads((phase/'freeze.json').read_text());key=str(Path('benchmark/architecture/four_dp_prediction_validation/dp04')/relative)
        if digest(path)!=manifest['source'][key]:raise ValueError('Frozen evaluator hash mismatch: '+key)
    return json.loads(path.read_text())
