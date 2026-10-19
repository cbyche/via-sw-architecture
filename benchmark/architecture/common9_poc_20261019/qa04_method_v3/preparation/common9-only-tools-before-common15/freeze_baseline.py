"""Preserve the accepted candidate/evaluator baseline; grants no execution."""
import argparse
import datetime
import hashlib
import json
import shutil
from pathlib import Path


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    p=argparse.ArgumentParser();p.add_argument('--from-probe',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--independent-audit',type=Path,required=True);a=p.parse_args()
    probe=json.loads(a.from_probe.read_text());audit=json.loads(a.independent_audit.read_text())
    assert probe['binary']['sha256']=='5aea60533c5b2a46c0e22cf8a7e4d86d51b4ba2c3860f6006341f0f073dd2e1d'
    assert sha(probe['binary']['path'])==probe['binary']['sha256']
    a.output.mkdir(parents=True,exist_ok=False);sources=[]
    for row in probe['sources']:
        rel=row['relativePath']
        if not rel.startswith(('prototypes/candidates/common9_poc_20261019/','benchmark/architecture/common9_poc_20261019/')):continue
        frozen=Path(row['frozen_path']);assert sha(frozen)==row['sha256']
        if rel.startswith('prototypes/candidates/')or'/evaluation_v3/'in rel:assert sha(row['path'])==row['sha256'],'Stable common source changed: '+rel
        destination=(a.output/'frozen/source'/rel).resolve();destination.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(frozen,destination)
        sources.append({**row,'frozen_path':str(destination),'copied_from':str(frozen)})
    binary=(a.output/'frozen/bin/common9_poc').resolve();binary.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(probe['binary']['path'],binary)
    record={'version':'QA04-METHOD-V3-BASELINE','purpose':'IMMUTABLE_CANDIDATE_EVALUATOR_BASELINE_NO_EXECUTION_AUTHORIZATION','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'source_probe_freeze':{'path':str(a.from_probe.resolve()),'sha256':sha(a.from_probe)},'independent63_audit':{'path':str(a.independent_audit.resolve()),'sha256':sha(a.independent_audit)},
        'sources':sources,'binary':{'path':str(binary),'sha256':sha(binary)},'qa04_requirements':'C01..C06 unchanged, all7 configurations; actual independent copies and15 regressions each require separate root GO.',
        'excluded_mutable_qa04_tools':{'reason':'Patch generators and acceptance tools are under repair/review; exact versions are snapshotted and hashed in each independent attempt, then final rebase record. They are not candidate baseline inputs.'},
        'supplement_regression_dependency':'PENDING_SEPARATE_EXACT_FREEZE_WHEN_CLI_READY','provider_calls':0,'candidate_selection_or_primary_results':False}
    (a.output/'freeze.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'freeze':str((a.output/'freeze.json').resolve()),'sha256':sha(a.output/'freeze.json'),'sources':len(sources),'binary_sha256':sha(binary)}))


if __name__=='__main__':main()
