"""Add an immutable supplement dependency snapshot beside a preserved core freeze."""
import argparse
import datetime
import hashlib
import json
import shutil
from pathlib import Path


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    p=argparse.ArgumentParser();p.add_argument('--baseline-freeze',type=Path,required=True);p.add_argument('--baseline-digest',required=True);p.add_argument('--source-checkpoint',type=Path,required=True);p.add_argument('--source-digest',required=True);p.add_argument('--baseline42-acceptance',type=Path,required=True);a=p.parse_args()
    assert sha(a.baseline_freeze)==a.baseline_digest
    assert sha(a.source_checkpoint)==a.source_digest,'Approved supplemental checkpoint hash mismatch'
    source=json.loads(a.source_checkpoint.read_text());accepted=json.loads(a.baseline42_acceptance.read_text())
    assert accepted['status']=='PASS'and accepted['planned']==accepted['completed']==42 and accepted['actual_API_calls']==0,'All42 unchanged-baseline supplement regressions must pass first'
    binary=source['core_frozen_mapping']['binary_sha256']
    assert binary=='5aea60533c5b2a46c0e22cf8a7e4d86d51b4ba2c3860f6006341f0f073dd2e1d'
    assert accepted['binary_sha256']==binary and source['core_frozen_mapping']['all_unchanged']is True
    assert source['runtime_source_matches_pre_run_closure_snapshot']is True
    assert Path(source['acceptance']['fresh42']).resolve()==a.baseline42_acceptance.resolve()
    dependency_sources=source['QA04_runtime_dependency_sources']
    root=a.baseline_freeze.parent;target=root/'frozen/source/benchmark/architecture/common_supplement6_20261019'
    assert not target.exists()and not(root/'supplement-freeze.json').exists(),'Preserve earlier dependency snapshots'
    repo=Path(__file__).resolve().parents[4];sources=[];prefix='benchmark/architecture/common_supplement6_20261019/'
    for row in dependency_sources:
        assert row['path'].startswith(prefix),'Unexpected supplement checkpoint dependency'
        assert sha(repo/row['path'])==row['sha256'],'Supplement source changed since checkpoint: '+row['path']
    for row in dependency_sources:
        rel=row['path'];assert rel.startswith(prefix),'Unexpected supplement checkpoint dependency'
        live=repo/rel;assert sha(live)==row['sha256'],'Supplement source changed since checkpoint: '+rel
        frozen=(root/'frozen/source'/rel).resolve();frozen.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(live,frozen)
        sources.append({'path':str(live),'relativePath':rel,'sha256':row['sha256'],'frozen_path':str(frozen)})
    record={'version':'QA04-SUPPLEMENT6-DEPENDENCIES-v1','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'purpose':'IMMUTABLE_REGRESSION_DEPENDENCIES_NO_EXECUTION_AUTHORIZATION','common_baseline_freeze':str(a.baseline_freeze.resolve()),'common_baseline_freeze_sha256':a.baseline_digest,'benchmark_path':str(target.resolve()),'sources':sources,'excluded_other_source_entries':{'reason':'Final owner checkpoint explicitly identifies QA04 runtime dependencies; authoring, diagnostics and source-review tools do not participate in these regression executions. Full owner checkpoint retained.'},'review_source_checkpoint':{'path':str(a.source_checkpoint.resolve()),'sha256':sha(a.source_checkpoint)},'baseline42_acceptance':{'path':str(a.baseline42_acceptance.resolve()),'sha256':sha(a.baseline42_acceptance)},'expected_per_copy':{'episodes':6,'responses':12,'goals':8,'bindings':6},'same_QA04_changes':42,'provider_calls':0}
    out=root/'supplement-freeze.json';out.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'path':str(out.resolve()),'sha256':sha(out),'sources':len(sources)}))


if __name__=='__main__':main()
